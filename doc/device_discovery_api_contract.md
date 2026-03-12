# 设备发现功能 - 接口契约文档

> 创建日期：2026-03-10  
> 适用模块：教师端 `apps/teacher`  
> 覆盖范围：搜索设备弹窗功能（发现局域网学生端 + 一键替换设备库）

---

## 背景 & 功能概述

用户点击"搜索设备"按钮后，教师端后端向局域网广播探测请求，收集在线的学生端设备 IP，返回候选列表供用户确认。用户确认后，后端以事务方式先清空旧设备表、再批量写入新设备，完成设备库替换。

**整体交互流程**

```
[用户点击"搜索设备"]
       │
       ▼
前端 invoke discover_student_devices() ──► 后端广播局域网，等待响应（超时 5s）
       │
       ◄── 返回 DiscoveredDevice[]（只含 ip）
       │
[弹窗展示候选 Table，用户勾选或全选]
       │
       ▼
[用户点击"确认"]
       │
[二次确认弹窗：将清空原有设备列表，是否继续？]
       │
       ▼
前端 invoke replace_devices_by_discovery(payload) ──► 后端事务：清空 → 补全 → 批量插入
       │
       ◄── 返回 DeviceDto[]（含完整 id / ip / name）
       │
[设备列表页刷新]
```

---

## 接口 1：discover_student_devices

### 用途

扫描当前局域网，返回可用的学生端设备 IP 列表，**不写入数据库**，仅作为发现结果供用户预览。

### Tauri Command 签名（Rust 侧）

```rust
#[tauri::command]
pub async fn discover_student_devices(
    state: State<'_, AppState>,
) -> Result<Vec<DiscoveredDeviceDto>, String>
```

### 前端调用（TypeScript）

```ts
// services/deviceService.ts
export async function discoverStudentDevices(): Promise<DiscoveredDevice[]> {
  return invoke<DiscoveredDevice[]>("discover_student_devices");
}
```

### Request

无 payload，直接调用。

### Response

```ts
// 单个发现结果
interface DiscoveredDevice {
  ip: string;       // e.g. "192.168.1.42"
}
```

```json
// 响应样例
[
  { "ip": "192.168.1.42" },
  { "ip": "192.168.1.55" },
  { "ip": "192.168.1.78" }
]
```

> 空数组 `[]` 代表本次扫描未发现任何学生端设备，属于正常返回。

### 错误情况

| 场景 | 行为 |
|------|------|
| 无法获取本机网卡信息 | 返回 `Err("无法获取本机 IP 地址")` |
| 超时（> 5s 无任何响应） | 返回当前已发现结果（可能为空数组），不报错 |
| 广播 socket 绑定失败 | 返回 `Err("广播端口绑定失败: <OS 错误信息>")` |

### 后端发现逻辑建议

- 方案一（推荐）：学生端实现 UDP 广播监听，收到教师端探测报文后立即回包自身 IP；教师端收集响应，5s 后截止。
- 方案二（备选）：固定端口 TCP connect 探测网段 `x.x.x.1-254`，仅保留成功建连的 IP。
- 不建议使用 ICMP ping（Windows 非管理员权限受限）。

> **注意**：`mdns.rs` 模块已预留，发现逻辑集中实现在 `network/mdns.rs` 或新增 `network/scanner.rs`，不在 controller 层写业务逻辑。

---

## 接口 2：replace_devices_by_discovery

### 用途

接收用户在弹窗中确认的设备 IP 列表，后端以**单事务**方式先清空 `devices` 表，再批量补全 `id`、`name` 后插入，最终返回完整的新设备列表。

### Tauri Command 签名（Rust 侧）

```rust
#[tauri::command]
pub async fn replace_devices_by_discovery(
    state: State<'_, AppState>,
    payload: ReplaceDevicesInput,
) -> Result<Vec<DeviceDto>, String>
```

### 前端调用（TypeScript）

```ts
// services/deviceService.ts
export interface ReplaceDevicesPayload {
  devices: { ip: string }[];
}

export async function replaceDevicesByDiscovery(
  payload: ReplaceDevicesPayload,
): Promise<DeviceListItem[]> {
  return invoke<DeviceListItem[]>("replace_devices_by_discovery", { payload });
}
```

### Request

```ts
interface ReplaceDevicesPayload {
  devices: { ip: string }[];   // 用户在弹窗中确认选中的设备列表
}
```

```json
// 请求样例
{
  "devices": [
    { "ip": "192.168.1.42" },
    { "ip": "192.168.1.55" }
  ]
}
```

> - `devices` 为空数组时，后端仍执行清空操作，设备表将变为空表。  
> - 前端在此情况下应给二次提示。

### Response

```ts
// 与现有 DeviceListItem 一致
interface DeviceListItem {
  id: string;    // 后端生成的 UUID
  ip: string;
  name: string;  // 后端生成，命名规则见下方
}
```

```json
// 响应样例
[
  { "id": "550e8400-...", "ip": "192.168.1.42", "name": "学生设备1" },
  { "id": "f47ac10b-...", "ip": "192.168.1.55", "name": "学生设备2" }
]
```

返回的列表顺序与传入 `devices` 数组顺序一致，便于前端直接刷新。

### 命名规则

> 命名逻辑固定在**后端**执行，前端不参与命名。

按 `devices` 数组的下标顺序（从 1 开始）依次命名：

```
学生设备1, 学生设备2, ..., 学生设备N
```

即：`name = format!("学生设备{}", index + 1)`

### 后端事务逻辑

```
BEGIN TRANSACTION
  1. DELETE FROM devices              -- 清空旧表
  2. for each (index, item) in input.devices:
       id   = uuid::Uuid::new_v4()
       name = format!("学生设备{}", index + 1)
       INSERT INTO devices (id, ip, name) VALUES (?, ?, ?)
  3. SELECT * FROM devices ORDER BY name  -- 查询插入结果
COMMIT
```

唯一性约束：同一 IP 在同一次批量中按前端传入去重，后端以第一次出现为准消除重复。

### 错误情况

| 场景 | 行为 |
|------|------|
| `devices` 字段缺失 | Deserialize 失败，自动返回 400-like 的 `Err` |
| 数据库事务失败 | 事务完整回滚，原有设备数据保留，返回 `Err("设备替换失败: <原因>")` |
| 同一 IP 在传入列表中重复 | 仅保留第一条，重复项静默忽略 |

---

## Rust 数据结构（schemas/device_schema.rs 新增）

```rust
/// discover_student_devices 的返回 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDeviceDto {
    pub ip: String,
}

/// replace_devices_by_discovery 的入参
#[derive(Debug, Clone, Deserialize)]
pub struct ReplaceDevicesInput {
    pub devices: Vec<DiscoveredDeviceDto>,
}
```

> `DeviceDto` 复用现有 `schemas/device_schema.rs` 中已定义的结构，无需新增。

---

## 前端 UI 流程补充

| 阶段 | UI 状态 |
|------|---------|
| 点击"搜索设备" | 按钮 loading，调用 `discoverStudentDevices()` |
| 返回结果 | 弹窗打开，Table 展示候选 IP 列表，loading 结束 |
| 发现结果为空 | 弹窗提示"未发现学生端设备，请确认学生端已启动" |
| 点击弹窗"确认" | 弹出二次确认：**"此操作将清空原有设备列表，是否继续？"** |
| 二次确认 OK | 按钮 loading，调用 `replaceDevicesByDiscovery()` |
| 成功 | 弹窗关闭，设备列表页刷新，提示"设备替换成功，共 N 台" |
| 失败 | 弹窗保持，`message.error("设备替换失败")` |

---

## 依赖 & 扩展说明

当前项目 `Cargo.toml` 已包含：

| 依赖 | 用途 |
|------|------|
| `mdns-sd` | 备用：mDNS 探测学生端服务类型（学生端也需要广播） |
| `tokio` | 并发探测、超时控制（`tokio::time::timeout`） |
| `uuid` | 生成设备 ID |
| `axum` / `tokio-tungstenite` | 如需学生端主动上报 IP，可复用 WS 服务 |

以上依赖均无需新增，开箱可用。
