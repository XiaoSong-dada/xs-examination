# 项目依赖拓扑图

## 目标

这份文档用于在进入具体实现前，快速回答三个问题：

1. 当前任务属于教师端、学生端，还是共享包与文档层。
2. 入口更可能在前端页面、前端 service/store、Tauri 命令、Rust service/repo，还是数据库迁移。
3. 该业务是否已经有对应的最短 e2e 链路文档可以直接复用。

## 使用方式

当任务涉及业务逻辑、跨层调用、链路排查或文档更新时，优先按以下顺序阅读：

1. 先阅读本拓扑图，确定入口模块、跨层边界和相关文档。
2. 若本拓扑图已经给出该业务的最短 e2e 链路映射，再继续阅读对应的 `doc/e2e/*.md`。
3. 若任务会改变入口、出口、关键持久化落点或页面验证面，处理完成后同步更新对应 e2e 文档和本拓扑图。

## 工作区总览

| 模块 | 位置 | 角色 |
|------|------|------|
| root-workspace | `./` | pnpm workspace 根，负责脚本编排 |
| teacher-frontend | `apps/teacher/src` | 教师端 React 前端 |
| teacher-rust | `apps/teacher/src-tauri/src` | 教师端 Tauri Rust 后端 |
| student-frontend | `apps/student/src` | 学生端 React 前端 |
| student-rust | `apps/student/src-tauri/src` | 学生端 Tauri Rust 后端 |
| shared-types | `packages/shared-types/src` | 预留的跨端 TypeScript 共享类型包 |
| docs | `doc` | PRD、技术设计、计划、e2e 与依赖拓扑 |
| github-customizations | `.github` | Copilot 工作区说明与按场景拆分的 instructions |

## 入口与边界

### 教师端前端

- 入口链：`apps/teacher/src/main.tsx` -> `App.tsx` -> `router/index.tsx` -> `pages/*`
- 常见边界：`pages` -> `hooks` -> `services` -> Tauri invoke -> teacher-rust
- 快速定位：
  - 页面展示与交互：`pages/`、`hooks/`
  - 前端调用后端命令：`services/`
  - 页面数据形状：`types/main.ts`

### 教师端 Rust

- 入口链：`apps/teacher/src-tauri/src/main.rs` -> `lib.rs` -> `controllers/`
- 常见边界：`controllers` -> `services` -> `repos/models`，以及 `network/` 负责控制与 WebSocket 链路
- 快速定位：
  - 前端 invoke 对应命令：`controllers/`
  - 业务规则与聚合流程：`services/`
  - 落库与实体：`repos/`、`models/`、`migrations/`
  - 网络与 ACK：`network/`

### 学生端前端

- 入口链：`apps/student/src/main.tsx` -> `App.tsx` -> `layout/AppLayout.tsx` -> `pages/Exam`
- 常见边界：`layout/pages` -> `store` -> `services` -> Tauri invoke -> student-rust
- 快速定位：
  - 考试页面：`pages/Exam/`、`components/ExamContent/`
  - 头部状态与设备信息：`layout/AppHeader.tsx`、`store/deviceStore.ts`
  - 本地会话与试卷恢复：`store/examStore.ts`、`services/examRuntimeService.ts`

### 学生端 Rust

- 入口链：`apps/student/src-tauri/src/main.rs` -> `lib.rs` -> `commands.rs` / `controllers/`
- 常见边界：`commands/controllers` -> `services` / `network` -> `db`
- 快速定位：
  - 连接与重连：`network/ws_client.rs`、`services/ws_reconnect_service.rs`
  - 控制端口与教师地址下发：`network/control_server.rs`
  - 本地会话、快照、答案与 outbox：`services/exam_runtime_service.rs`
  - 设备 IP：`controllers/device_controller.rs`、`services/device_service.rs`、`network/device_network.rs`

## 业务主题到最短 e2e 映射

| 业务主题 | 适用范围简述 | 对应 e2e 文档 |
|------|------|------|
| 教师端发现学生设备 IP / 学生端 Header 设备 IP | 排查 discovery ACK、本机 IP 解析与 Header 展示是否同源 | `doc/e2e/e2e-minimal-teacher-student-ip-chain.md` |
| 分配页随机分配考生到设备 | 区分教师端分配落点与学生端本地会话落点 | `doc/e2e/e2e-minimal-device-assign-student-session-chain.md` |
| 分配页连接考生设备并在学生端预写入会话 | 排查 `APPLY_TEACHER_ENDPOINTS`、`teacher_endpoints`、`exam_sessions` 与 Header 展示 | `doc/e2e/e2e-minimal-connect-student-device-chain.md` |
| 教师端分发试卷到学生端本地落库 | 排查 `DISTRIBUTE_EXAM_PAPER`、`exam_sessions/exam_snapshots` 与“已收到试卷”状态来源 | `doc/e2e/e2e-minimal-exam-paper-distribution-chain.md` |
| 学生端启动恢复 / 自动重连 / 本地会话与答案恢复 | 排查启动恢复、持续重连、答案回填与自愈前置链路 | `doc/e2e/e2e-minimal-student-startup-reconnect-chain.md` |
| 教师端开始考试后学生端按题同步答案并更新监考进度 | 排查 `EXAM_START`、`ANSWER_SYNC`、ACK 与 Monitor/Report 进度来源 | `doc/e2e/e2e-minimal-answer-sync-progress-chain.md` |
| 教师端异常恢复后学生端全量答案同步与 ACK 收敛 | 排查 full sync、pending/failed flush、部分成功失败与去重保护 | `doc/e2e/e2e-minimal-answer-sync-ack-reconnect-chain.md` |

## 当前已确认的特殊事实

### shared-types 现状

- `apps/student/package.json` 和 `apps/teacher/package.json` 仍声明依赖 `@xs/shared-types`。
- `packages/shared-types/src/index.ts` 当前只导出 `exam.ts` 与 `protocol.ts`。
- 目前在 `apps/**` 源码中没有检索到对 `@xs/shared-types` 的实际 import。
- 因此当前仓库对 `shared-types` 更接近“预留依赖”，而不是正在被前端源码真实消费的共享层。

### 旧 .copilot 文档现状

- 旧的依赖图谱、memorybank 与 AGENTS 仍位于 `.copilot/`。
- 这些文件包含历史阶段信息，其中部分内容已经过时，不能继续作为唯一规范入口。
- 当前应以 `.github/copilot-instructions.md` 和 `.github/instructions/*.instructions.md` 作为自动加载的规范入口，以 `doc/` 下的正式文档作为项目资料入口。

## 关联文档

- 全局工作区说明：[.github/copilot-instructions.md](../.github/copilot-instructions.md)
- E2E 业务链路目录：[doc/e2e](./e2e)
- 实施计划目录：[doc/plans](./plans)
- 旧版依赖图谱来源：[.copilot/project-dependency-map.md](../.copilot/project-dependency-map.md)
