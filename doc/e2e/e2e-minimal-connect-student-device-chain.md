# 教师端连接考生设备到学生端显示会话信息的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端在“分配考生”页面点击“连接考生设备”之后，如何把当前考试与考生会话送到学生端，完成本地 `exam_sessions` 预写入，并在学生端 Header 中显示考试标题、学生名称以及当前设备 IP。

这里不展开“发卷”“开始考试”“答题同步”等后续链路，只聚焦连接阶段本身，以及 Header 展示依赖的最小运行态链路。

## 最短链路结论

最短链路如下：

1. 教师端分配页点击“连接考生设备”。
2. 教师端前端调用 `connect_student_devices_by_exam_id`。
3. 教师端 Rust 读取当前考试下的分配记录和考试信息。
4. 教师端为每台已分配设备构造带有 `session_id/exam_id/exam_title/student_id/student_no/student_name/assigned_ip_addr` 的 `APPLY_TEACHER_ENDPOINTS` 请求。
5. 教师端通过 TCP 直连学生端控制端口下发请求。
6. 学生端 `control_server` 收到请求后先落库 `teacher_endpoints`。
7. 学生端再调用 `ExamRuntimeService::upsert_connected_session`，把连接阶段最小会话写入 `exam_sessions`，状态设为 `connected_pending_distribution`。
8. 学生端随后尝试连接教师端 WebSocket。
9. 学生端 WebSocket 真正连接成功后，前端通过 `get_current_exam_bundle -> examStore -> AppHeader` 读取刚刚预写入的会话，并显示考试标题、会话状态、学生名称。
10. 与会话链并行，学生端前端还会通过 `deviceStore -> deviceService -> get_device_runtime_status` 主动查询后端设备 IP，并在 Header 中显示“设备 IP”。

到第 7 步为止，已经完成“连接考生设备 -> 学生端本地会话建立”的最短业务闭环。

到第 10 步为止，已经完成“连接考生设备 -> 学生端 Header 显示会话信息与设备 IP”的最短页面闭环。

## 入口到出口的精简调用链

### 1. 教师端前端入口

入口在：

- [apps/teacher/src/pages/DeviceAssign/index.tsx](apps/teacher/src/pages/DeviceAssign/index.tsx)

点击“连接考生设备”后，页面会调用 Hook 中的 `connectDevices()`。

### 2. 教师端前端 Hook

Hook 在：

- [apps/teacher/src/hooks/useDeviceAssign.ts](apps/teacher/src/hooks/useDeviceAssign.ts)

`connectDevices()` 的职责是：

1. 校验当前考试已选择。
2. 调用前端 service `connectStudentDevicesByExamId(selectedExamId)`。
3. 连接完成后刷新当前考试下的分配与连接状态。

### 3. 教师端前端 Service -> Tauri IPC

前端 service 在：

- [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts)

它通过 Tauri invoke 调用：

`connect_student_devices_by_exam_id`

到这里为止，仍然只是教师端前端到教师端 Rust 的本地 IPC。

### 4. 教师端 Rust 命令入口

教师端 Rust 命令入口在：

- [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs)

`connect_student_devices_by_exam_id` 的当前行为是：

1. 按 `exam_id` 查询当前考试下的 `student_exams` 分配记录。
2. 查询当前考试详情，拿到 `exam_title/start_time/end_time`。
3. 按已分配 `ip_addr` 过滤目标设备。
4. 为每台设备构造 `ApplyTeacherEndpointsRequest`。

当前下发的 `payload` 里除了教师端地址外，还会携带：

1. `session_id`
2. `exam_id`
3. `exam_title`
4. `student_id`
5. `student_no`
6. `student_name`
7. `assigned_ip_addr`
8. `assignment_status`
9. `start_time`
10. `end_time`

这里有两个关键事实：

1. `payload.student_id` 仍然必须是考试分配记录里的真实 `student_id`，不能使用设备 `device_id`。
2. 连接阶段现在已经不只是“下发教师地址”，而是“下发教师地址 + 下发当前最小会话信息”。

### 5. 教师端到学生端的真实传输方式

教师端通过：

- [apps/teacher/src-tauri/src/network/student_control_client.rs](apps/teacher/src-tauri/src/network/student_control_client.rs)

调用：

`apply_teacher_endpoints(device_ip, control_port, &req)`

底层仍是 TCP request-reply，不是 UDP 广播。

## 学生端接收入口

学生端入口在：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

`handle_client` 在收到 `APPLY_TEACHER_ENDPOINTS` 后，会按顺序执行：

1. `TeacherEndpointsService::replace_all`，把教师端地址写入本地 `teacher_endpoints`。
2. `ExamRuntimeService::upsert_connected_session`，把连接阶段会话写入本地 `exam_sessions`。
3. `ws_client::connect`，尝试主动连接教师端 WebSocket。

因此，这条链路的学生端出口不再只有 `teacher_endpoints` 落库，而是多了一个明确的会话出口。

## 学生端真实出口

学生端连接阶段的真实业务出口在：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

函数：

- `upsert_connected_session`

它会按 `session_id` 写入或更新 `exam_sessions`，核心字段包括：

1. `id = session_id`
2. `exam_id`
3. `student_id`
4. `student_no`
5. `student_name`
6. `assigned_ip_addr`
7. `exam_title`
8. `status = connected_pending_distribution`
9. `assignment_status`
10. `ends_at`

此时不会写入：

1. `exam_snapshots`
2. `questions_payload`
3. 任何试卷快照内容

所以最准确的表述是：

连接考生设备阶段会在学生端建立“最小考试会话”，但不会下发试卷内容。

## 学生端 Header 为什么会显示信息

学生端 Header 现在有两条并行读取链。

### 会话信息链

1. [apps/student/src/App.tsx](apps/student/src/App.tsx) 启动后定时调用 `refreshCurrentExam()`。
2. [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 通过 `getCurrentExamBundle()` 拉取最新会话。
3. [apps/student/src/services/examRuntimeService.ts](apps/student/src/services/examRuntimeService.ts) 调用 Tauri `get_current_exam_bundle`。
4. [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 从本地 `exam_sessions/exam_snapshots` 返回最新 bundle。
5. [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 使用 `examStore.currentSession` 作为考试标题和学生名称来源。

### 设备 IP 链

1. [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 挂载时调用 `deviceStore.initTeacherInfo()`。
2. [apps/student/src/store/deviceStore.ts](apps/student/src/store/deviceStore.ts) 会先执行 `initDeviceInfo()`。
3. `initDeviceInfo()` 通过 [apps/student/src/services/deviceService.ts](apps/student/src/services/deviceService.ts) invoke `get_device_runtime_status`。
4. 学生端 Rust [apps/student/src-tauri/src/controllers/device_controller.rs](apps/student/src-tauri/src/controllers/device_controller.rs) 调用 [apps/student/src-tauri/src/services/device_service.rs](apps/student/src-tauri/src/services/device_service.rs)。
5. `device_service.rs` 再调用 [apps/student/src-tauri/src/network/device_network.rs](apps/student/src-tauri/src/network/device_network.rs) 解析本机出站 IPv4。
6. controller 返回设备 IP，并发出 `device_ip_updated` 事件。
7. `deviceStore` 用 invoke 返回值和事件 payload 双重刷新 `ip`。
8. `AppHeader.tsx` 最终从 `deviceStore.ip` 渲染“设备 IP”。

现在 Header 的展示口径已经拆成两条独立链：

1. 只要 `currentSession` 已存在，Header 就会直接显示考试标题与学生名称，用于启动恢复与本地缓存展示。
2. 教师端连接状态图标与文案则独立来自 `teacherConnectionStatus`，并在 `connecting` 时显示 `link.png` 闪烁图标，用于表达自动重连中的运行态。

所以页面出口的准确写法应当是：

1. 学生端本地 `exam_sessions` 一旦已可被 `get_current_exam_bundle` 读取，前端就能在 Header 中恢复考试标题与学生名称。
2. 学生端 WebSocket 是否已连接、是否正在重连，则独立通过 `deviceStore -> teacherEndpointService -> ws_connected/ws_disconnected` 这条运行态链表达。
3. 学生端设备 IP 则独立来自 `deviceStore -> deviceService -> get_device_runtime_status -> device_controller -> device_service -> device_network` 这条运行态查询链，不依赖 `exam_sessions`，也不依赖 `assigned_ip_addr`。

## 与发卷链路的边界

这条连接链路只负责：

1. 下发教师端地址
2. 建立 WebSocket 连接
3. 预写入最小考试会话
4. 驱动 Header 恢复基础信息
5. 通过独立设备运行态链路展示当前设备 IP

它不负责：

1. 下发题目列表
2. 写入 `exam_snapshots`
3. 进入答题页

这些都属于后续的“分发试卷”链路。

## 一句话总结

教师端分配页点击“连接考生设备”后，经前端 invoke 调用教师端 Rust `connect_student_devices_by_exam_id`，教师端会把教师端地址与当前考试/考生最小会话信息一起封装为 `APPLY_TEACHER_ENDPOINTS`，通过 TCP 逐台发送到学生端控制端口；学生端 `control_server` 收到后先落库 `teacher_endpoints`，再由 `ExamRuntimeService::upsert_connected_session` 预写入 `exam_sessions`，随后主动连接教师端 WebSocket；当 WebSocket 真连接成功后，学生端前端再通过 `get_current_exam_bundle -> examStore -> AppHeader` 显示考试标题与学生名称，同时通过 `deviceStore -> deviceService -> get_device_runtime_status` 这条独立链路显示当前设备 IP。