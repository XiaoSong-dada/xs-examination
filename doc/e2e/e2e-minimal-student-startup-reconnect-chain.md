# 学生端启动恢复与断线重连的最短 e2e 链路

## 目标

这份文档只回答一个问题：

学生端在本地已经有 `exam_sessions` 与 `teacher_endpoints(is_master)` 缓存的前提下，冷启动后如何恢复头部信息、自动连接教师端，以及在连接失败或后续断开时如何持续重连。

这里不展开发卷、开考、答题同步，只聚焦：

1. 启动恢复
2. 自动连接
3. 首连失败重试
4. 已连接后断线重连

## 最短链路结论

最短链路如下：

1. 学生端应用启动，`lib.rs` 在 `setup()` 中启动 `discovery_listener`、`control_server`，并追加启动 `ws_reconnect_service::bootstrap_from_local_state`。
2. `bootstrap_from_local_state` 从本地 `teacher_endpoints` 读取 `is_master` endpoint。
3. 同一个启动流程中，它再通过 `ExamRuntimeService::get_current_exam_bundle` 读取最近一条 `exam_sessions`，拿到当前 `student_id`。
4. 若 `endpoint` 与 `student_id` 都存在，则交给 `WsReconnectService::start_or_update` 设定当前连接目标。
5. 重连服务启动后台循环：只要当前未连接，就持续调用 `ws_client::connect(endpoint, student_id)`。
6. 若首次连接失败，则发出 `ws_disconnected` 事件，并在固定间隔后继续重试。
7. 若连接成功，则 `ws_client` 建立发送通道、reader/writer/heartbeat 循环，并发出 `ws_connected` 事件。
8. 若后续 writer 出错、reader 出错或 heartbeat 发送失败，`ws_client` 会统一清理连接状态并再次发出 `ws_disconnected` 事件。
9. 重连服务检测到当前目标仍存在且当前未连接，会再次进入下一轮重试，直到恢复连接。
10. 与连接链并行，前端 `App.tsx -> examStore` 会继续读取最近 `exam_sessions`，`AppHeader.tsx` 会直接恢复考试标题和学生名称；`deviceStore` 则根据 `get_teacher_runtime_status` 与 `ws` 事件显示 `connected / connecting / disconnected`，并在 `connecting` 时显示闪烁的 `link.png`。

到第 5 步为止，已经形成“启动即自动连接”的最短后台闭环。

到第 10 步为止，已经形成“启动恢复头部信息 + 后台自动重连状态可见”的最短页面闭环。

## 入口到出口的精简调用链

### 1. 学生端启动入口

启动入口在：

- [apps/student/src-tauri/src/lib.rs](apps/student/src-tauri/src/lib.rs)

`setup()` 当前会启动三条后台链：

1. `network::discovery_listener::start(...)`
2. `network::control_server::start(...)`
3. `services::ws_reconnect_service::bootstrap_from_local_state(...)`

前两条分别负责设备发现与教师控制消息，第三条才是这次断线重连更新的真正启动入口。

### 2. 启动恢复的本地数据来源

主教师端地址来自：

- [apps/student/src-tauri/src/services/teacher_endpoints_service.rs](apps/student/src-tauri/src/services/teacher_endpoints_service.rs)

其中 `get_master_endpoint` 会从 `teacher_endpoints` 读取 `is_master=1` 的 endpoint。

最近考试会话与 `student_id` 来自：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

当前实现复用 `get_current_exam_bundle` 取最近一条 `exam_sessions`，并从 `session.student_id` 拿到重连所需的学生标识。

因此，启动恢复依赖的真实最小上下文是：

1. `teacher_endpoints.is_master.endpoint`
2. 最近一条 `exam_sessions.student_id`

如果只有 endpoint 没有 session，则当前实现会只恢复头部教师端地址，不盲目发起连接。

### 3. 统一自动重连入口

统一重连入口在：

- [apps/student/src-tauri/src/services/ws_reconnect_service.rs](apps/student/src-tauri/src/services/ws_reconnect_service.rs)

`start_or_update` 的职责是：

1. 保存当前目标 `endpoint + student_id`
2. 若当前已连的是其他 endpoint，则先主动断开旧连接
3. 启动唯一的一条后台重连循环
4. 只要当前目标未消失且当前未连接，就持续尝试重新连接

这意味着：

1. 启动恢复走它
2. 控制服务收到新的 `APPLY_TEACHER_ENDPOINTS` 也走它
3. 手工调用 `connect_teacher_ws` 也走它

现在不再让多个入口各自直接调用一次性 `ws_client::connect`。

### 4. 真实 WebSocket 建连点

真实 WebSocket 建连仍在：

- [apps/student/src-tauri/src/network/ws_client.rs](apps/student/src-tauri/src/network/ws_client.rs)

它会：

1. 调用 `network/transport/ws_transport.rs::connect_ws(...)`
2. 建立 writer、reader、heartbeat 三条异步循环
3. 在连接成功时写入 `AppState.ws_sender/ws_connected/ws_endpoint`
4. 发出 `ws_connected` 事件

因此，重连服务负责“何时不断重试”，`ws_client` 负责“每一次具体怎么连接”。

### 5. 断线后的统一收口

当前断线收口也集中在：

- [apps/student/src-tauri/src/network/ws_client.rs](apps/student/src-tauri/src/network/ws_client.rs)

以下场景都会进入统一清理：

1. writer loop 退出
2. reader loop 出错
3. heartbeat 发送失败
4. 因切换目标 endpoint 触发主动断开

统一动作包括：

1. 清理 `ws_sender`
2. 清理 `ws_endpoint`
3. 设置 `ws_connected=false`
4. 发出 `ws_disconnected` 事件

这一步是自动重连能持续工作的前提，因为如果状态不收口，重连循环会误以为当前仍在连接中。

### 6. 前端状态与头部展示链

学生端前端当前有两条并行链：

#### 业务会话恢复链

1. [apps/student/src/App.tsx](apps/student/src/App.tsx) 启动后调用 `refreshCurrentExam()`
2. [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 通过 `getCurrentExamBundle()` 拉取最近会话
3. [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 直接使用 `examStore.currentSession` 恢复考试标题和学生名称

#### 连接状态链

1. [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 挂载后调用 `deviceStore.initTeacherInfo()`
2. [apps/student/src/store/deviceStore.ts](apps/student/src/store/deviceStore.ts) 通过 `get_teacher_runtime_status` 读取主教师端地址与当前连接状态
3. 同时订阅 `teacher_endpoint_applied`、`ws_connected`、`ws_disconnected`
4. 若已有 endpoint 但当前未连接，则状态进入 `connecting`
5. [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 在 `connecting` 时显示 `link.png`，并以 0.5 秒显隐切换方式闪烁

因此，当前页面出口应分成两层理解：

1. 头部业务信息出口：本地 `exam_sessions` 已成功恢复
2. 头部连接状态出口：自动重连是否正在进行、是否已连上教师端

## 与控制服务链路的关系

断线重连不是只在冷启动时触发，也会被教师端下发链路复用。

当前：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

在收到 `APPLY_TEACHER_ENDPOINTS` 成功后，不再直接做一次性 `connect`，而是把新的 `master endpoint + student_id` 交给 `WsReconnectService::start_or_update`。

所以这条链路现在统一成：

1. 启动恢复会设定重连目标
2. 教师端重新下发地址也会更新重连目标
3. 目标变更时旧连接会先断开，再切换到新目标

## 最短 e2e 验收点

### A. 启动恢复

1. 本地已有 `teacher_endpoints(is_master)` 与最近 `exam_sessions`
2. 冷启动学生端
3. Header 立即显示考试标题、学生名称、教师端 IP

### B. 启动自动连接

1. 教师端可达
2. 冷启动学生端
3. Header 连接状态进入 `connecting`
4. 随后切到 `connected`

### C. 首连失败持续重试

1. 本地已有缓存，但教师端暂不可达
2. 冷启动学生端
3. Header 持续显示 `connecting` 与闪烁 `link.png`
4. 教师端恢复后自动切到 `connected`

### D. 已连接后断线重连

1. 学生端已与教师端建立 WebSocket
2. 教师端异常退出或网络中断
3. Header 从 `connected` 切到 `connecting`
4. 自动重试，教师端恢复后再次切回 `connected`

## 一句话总结

学生端现在已经形成完整的“启动恢复 + 自动连接 + 断线后重连”最短链路：应用启动时由 `lib.rs` 触发 `ws_reconnect_service::bootstrap_from_local_state`，从本地 `teacher_endpoints(is_master)` 与最近 `exam_sessions.student_id` 恢复连接目标，再由 `ws_reconnect_service` 统一驱动 `ws_client` 持续建连；连接失败或后续断线时，`ws_client` 统一清理状态并发出断线事件，重连服务继续按固定间隔重试；前端 Header 则一边直接恢复本地考试与学生信息，一边独立显示教师端连接状态和重连中的闪烁图标。