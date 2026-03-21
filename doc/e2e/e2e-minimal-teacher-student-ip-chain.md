# 教师端初始化学生端 IP、教师地址与学生端本机 IP 展示的最小 e2e 链路

## 目标

这份文档现在梳理三条最短业务链：

1. 学生端如何解析本机设备 IP，并把同一份 IP 同时用于 discovery ACK 和前端 Header 展示。
2. 教师端在“设备管理”页面录入教师端地址后，如何把这些地址下发到学生端，并最终写入学生端本地数据库。
3. 教师端在“分配考生”页面点击“连接考生设备”后，如何让学生端建立 WebSocket 连接、预写入本地 `exam_sessions`，并把真实连接状态回流到“分配考生”和“实时监考”页面。

这里先纠正两个容易混淆的点：

1. “下发教师地址”与“连接考生设备”两条链路都不是 UDP 广播。
2. 这两条链路的真正网络传输方式，都是教师端根据已知学生设备 IP，逐台通过 TCP 直连学生端控制端口。
3. 学生端 discovery ACK 里的 `ip` 和 Header 里的“设备 IP”现在都应优先来自同一个本机 IP 解析工具，而不是直接取 `peer.ip()`。

UDP 监听只出现在学生端的设备发现链路里，对应 apps/student/src-tauri/src/network/discovery_listener.rs；但该链路现在也会复用本机 IP 解析工具，把解析出的设备 IP 写进 ACK，并在解析失败时才回退到 `peer.ip()`。

## 2026-03-20 封装更新结论

这次 network 层封装没有改变这两条 e2e 业务链的起点、终点和成败判定。

变化的是：

1. WebSocket 握手、发送通道和写循环开始下沉到 `network/transport/ws_transport.rs`。
2. TCP request-reply 的 connect、timeout、半关闭、读取 ACK 以及服务端 bind/read/write 开始下沉到 `network/transport/tcp_request_reply.rs`。

因此应这样理解本次更新：

1. 业务 e2e 链路不变。
2. 网络层内部调用点发生了收口。
3. 后续查问题时，除了 `ws_server.rs`、`ws_client.rs`、`student_control_client.rs`、`control_server.rs`，还要同步看各自的 `network/transport/*.rs`。

## 2026-03-21 设备 IP 更新结论

这次学生端 IP 改动新增了一条独立且可复用的运行态链路。

变化的是：

1. 学生端 Rust 新增 `network/device_network.rs`，通过 UDP route probing 解析本机出站 IPv4。
2. 学生端 Rust 新增 `controllers/device_controller.rs -> services/device_service.rs`，把设备 IP 作为 Tauri 运行态命令 `get_device_runtime_status` 暴露给前端。
3. 学生端前端新增 `services/deviceService.ts -> store/deviceStore.ts::initDeviceInfo`，Header 不再等待别的业务链间接带出 IP，而是主动调用后端命令并监听 `device_ip_updated` 事件。
4. `discovery_listener.rs` 不再直接把 `peer.ip()` 当作设备 IP 回包，而是优先复用 `network/device_network.rs::resolve_device_ip()`，只有解析失败才回退。

因此应这样理解本次更新：

1. 教师端发现学生设备时看到的 IP。
2. 学生端 Header 中显示的设备 IP。
3. 学生端后台事件里回流给前端的设备 IP。

这三者现在已经开始收敛到同一份本机 IP 来源。

## 最短链路结论

### 链路 A：学生端本机 IP 获取并展示到 Header

最短链路如下：

1. 学生端前端 `layout/AppHeader.tsx` 挂载后调用 `deviceStore.initTeacherInfo()`。
2. `deviceStore.initTeacherInfo()` 会先调用 `initDeviceInfo()`。
3. `initDeviceInfo()` 通过 `services/deviceService.ts` invoke `get_device_runtime_status`。
4. 学生端 Rust `controllers/device_controller.rs` 接收命令后，调用 `DeviceService::get_runtime_status()`。
5. `services/device_service.rs` 再调用 `network/device_network.rs::resolve_device_ip()`。
6. `resolve_device_ip()` 通过 UDP route probing 获取当前出站网卡对应的本机 IPv4。
7. controller 返回 `DeviceRuntimeStatusDto { ip }`，并同步发出 `device_ip_updated` 事件。
8. 学生端前端 `deviceStore` 一方面使用 invoke 返回值设置 `ip`，另一方面订阅 `device_ip_updated` 持续刷新 `ip`。
9. `AppHeader.tsx` 最终从 `deviceStore.ip` 渲染“设备 IP”。

如果只看“学生端 Header 中的设备 IP 从哪里来”，真正业务出口不是 `AppHeader` 本身，也不是 discovery ACK，而是 `apps/student/src-tauri/src/network/device_network.rs` 成功解析出本机 IP，并由 `deviceStore` 写入前端状态。

### 链路 A 的 discovery 一致性补充

最短链路如下：

1. 教师端设备发现时向局域网广播 `DISCOVER_STUDENT_DEVICES`。
2. 学生端 `network/discovery_listener.rs` 收到广播请求。
3. `discovery_listener.rs` 现在优先调用 `network/device_network.rs::resolve_device_ip()`。
4. 若解析成功，则把该 IP 填入 `DiscoverAckPayload.ip`。
5. 若解析失败，才回退为 `peer.ip().to_string()`。
6. 同时学生端发出一次 `device_ip_updated` 事件，把当前解析到的 IP 回流给前端。

因此，这条链路的更新点不是“教师端怎么发现学生端”，而是“学生端回包给教师端的 IP 已改为和本机运行态查询共用同一来源”。

### 链路 B：设备管理页统一下发教师地址

最短链路如下：

1. 教师端前端 handlePushTeacherEndpoints 收集表单中的主/备教师端地址。
2. 前端通过 Tauri invoke 调用 push_teacher_endpoints_to_devices。
3. 教师端 Rust 后端根据已选择的 deviceIds 查询学生设备 IP。
4. 教师端 Rust 后端对每个学生设备发起 TCP 连接，目标是 student_ip:control_port。
5. 学生端启动时已常驻启动 control_server，并在 control_port 上监听 TCP。
6. 学生端 control_server::handle_client 读取 APPLY_TEACHER_ENDPOINTS 请求。
7. handle_client 调用 TeacherEndpointsService::replace_all 批量替换 teacher_endpoints 表中的数据。
8. replace_all 在事务 commit 成功后，教师端地址才算真正落库完成。
9. 落库成功后，学生端会尝试用主教师端地址发起一次 WebSocket 连接。
10. 学生端最后返回 APPLY_TEACHER_ENDPOINTS_ACK 给教师端。

如果只看“教师端输入的地址最终进入学生端数据库”这个目标，那么真正出口不是 handle_client 本身，而是 apps/student/src-tauri/src/services/teacher_endpoints_service.rs 中 replace_all 的事务提交成功。

### 链路 C：分配页一键连接考生设备并回流真实状态

最短链路如下：

1. 教师端前端在 `pages/DeviceAssign/index.tsx` 中点击“连接考生设备”。
2. 前端通过 `services/studentService.ts` invoke 调用 `connect_student_devices_by_exam_id`。
3. 教师端 Rust 在 `controllers/student_exam_controller.rs` 中按 `exam_id` 读取 `student_exams` 的分配记录。
4. 仅保留 `ip_addr` 非空的记录，并为每条记录构造一次 `APPLY_TEACHER_ENDPOINTS` 请求。
5. 这里请求里的 `payload.student_id` 必须使用真实学生 `student_id`，不能使用设备 `device_id`。
6. 教师端通过 `network/student_control_client.rs` 逐台 TCP 直连学生端 `control_port` 发送请求。
7. 学生端 `control_server::handle_client` 收到后先调用 `TeacherEndpointsService::replace_all` 完成地址落库。
8. 随后学生端调用 `ExamRuntimeService::upsert_connected_session`，把 `session_id/exam_id/exam_title/student_no/student_name/assigned_ip_addr` 等最小会话信息预写入本地 `exam_sessions`。
9. 学生端落库成功后调用 `network/ws_client.rs::connect` 主动连接教师端 WebSocket。
10. 学生端建立 WebSocket 后按 5 秒周期发送 `HEARTBEAT`，payload 中带上同一个 `student_id`。
11. 教师端 `network/ws_server.rs` 收到心跳后调用 `state.touch_connection(student_id, timestamp)` 更新运行时连接快照。
12. 教师端前端 `hooks/useDeviceAssign.ts` 与 `hooks/useMonitor.ts` 再通过 `get_student_device_connection_status_by_exam_id` 查询按考试聚合后的真实状态。
13. 学生端前端则通过 `get_current_exam_bundle -> examStore -> AppHeader` 在 WebSocket 真连接成功后显示考试标题、学生名称等信息。
14. 最终“分配考生”和“实时监考”页面都会展示同一套四态，学生端 Header 也会展示当前会话信息。

## 入口到出口的精简调用链

### 1. 教师端页面入口（设备管理页）

入口在 apps/teacher/src/pages/Devices/index.tsx 的 handlePushTeacherEndpoints。

它做了三件事：

1. 从表单读取 masterEndpoint、slaveEndpoint、controlPort、remark。
2. 组装 endpoints 数组，其中主地址 isMaster=true，备地址 isMaster=false。
3. 调用前端服务 pushTeacherEndpointsToDevices(payload)。

这里并没有任何 UDP 广播逻辑，只是组装数据并发起一次 Tauri IPC。

### 2. 教师端前端到 Rust IPC（设备管理页）

apps/teacher/src/services/deviceService.ts 中的 pushTeacherEndpointsToDevices 通过 invoke 调用 Tauri 命令 push_teacher_endpoints_to_devices。

对应 Rust 命令入口在 apps/teacher/src-tauri/src/controllers/device_controller.rs。

这一步只是桥接，不做实际网络下发。

### 3. 教师端 Rust 真实下发点（设备管理页）

真实下发逻辑在 apps/teacher/src-tauri/src/services/device_service.rs 的 push_teacher_endpoints_to_devices。

这个函数的实际行为是：

1. 遍历前端传入的 deviceIds。
2. 通过 device_repo::get_device_by_id 从教师端本地设备表查出每台学生设备的 IP。
3. 组装 ApplyTeacherEndpointsRequest，请求类型为 APPLY_TEACHER_ENDPOINTS。
4. 调用 student_control_client::apply_teacher_endpoints(&device.ip, control_port, &req)。
5. `apply_teacher_endpoints` 进一步调用 `network/transport/tcp_request_reply.rs` 中的 `send_json_request(...)` 执行 TCP request-reply。

因此，链路依赖的是“教师端本地已经知道学生设备 IP”。这些 IP 来自更早的设备发现/录入流程，不是在这里临时广播得到的。

### 4. 教师端到学生端的真实传输协议

apps/teacher/src-tauri/src/network/student_control_client.rs 中的 apply_teacher_endpoints 明确使用的是 TCP，只是现在底层细节已通过 transport 薄封装收口：

1. `student_control_client.rs` 负责组装业务请求与选择超时模板。
2. `network/transport/tcp_request_reply.rs::send_json_request(...)` 用 TcpStream::connect 连接 student_ip:control_port。
3. transport 薄层把请求 JSON 写入连接，并按配置决定是否半关闭写端。
4. transport 薄层统一读取学生端返回的 ACK。

默认 control_port 由前端传入，未传时教师端后端默认使用 18889。学生端默认 control_port 也是 18889，定义在 apps/student/src-tauri/src/config.rs。

所以这条链路的网络本质是：

教师端逐台 TCP 单播下发，不是 UDP 广播，也不是 UDP 组播。

## 分配页一键连接的精简调用链

### 1. 教师端页面入口（分配页）

入口在 apps/teacher/src/pages/DeviceAssign/index.tsx 的 handleConnectDevices。

它做了三件事：

1. 确认当前已选择考试。
2. 确认当前考试下至少存在一条已分配设备的记录。
3. 调用前端服务 connectStudentDevicesByExamId(examId)。

这里并不让用户手工录入教师地址，而是复用教师端后端在本机解析出的主 WebSocket 地址。

### 2. 教师端前端到 Rust IPC（分配页）

apps/teacher/src/services/studentService.ts 中的 connectStudentDevicesByExamId 通过 invoke 调用 Tauri 命令 connect_student_devices_by_exam_id。

对应 Rust 命令入口在 apps/teacher/src-tauri/src/controllers/student_exam_controller.rs。

### 3. 教师端 Rust 真实下发点（分配页）

`connect_student_devices_by_exam_id` 的实际行为是：

1. 按 `exam_id` 读取当前考试下的 `StudentDeviceAssignDto` 列表。
2. 仅保留 `ip_addr` 非空的记录。
3. 使用教师端本机 IP 和 `WS_SERVER_PORT` 组装主教师端地址，例如 `ws://teacher-ip:18888`。
4. 读取当前考试信息，并对每条分配记录构造 `ApplyTeacherEndpointsRequest`。
5. 关键约束：请求中的 `payload.student_id` 必须取自分配记录里的 `student_id`，不能取设备表里的 `device_id`。
6. 当前请求除了教师端地址，还会携带 `session_id`、`exam_id`、`exam_title`、`student_no`、`student_name`、`assigned_ip_addr`、考试时间等连接阶段会话字段。
7. 再调用 `student_control_client::apply_teacher_endpoints(device_ip, 18889, &req)` 逐台下发。
7. 该函数底层通过 `network/transport/tcp_request_reply.rs::send_json_request(...)` 执行 TCP 单播与 ACK 读取。

### 4. 为什么 `student_id` 映射是关键

这条链路里有两个看起来都像“主键”的字段：

1. 设备表主键 `device_id`
2. 考生分配记录中的 `student_id`

但教师端 WebSocket 心跳聚合和 UI 状态匹配都是按 `student_id` 做的：

1. 学生端心跳消息 payload 带的是 `student_id`
2. 教师端 `ws_server` 调用 `state.touch_connection(student_id, timestamp)`
3. 教师端 `student_exam_service` 再用分配记录中的 `student_id` 去匹配 `state.connections`

因此，如果下发时误把 `payload.student_id` 写成 `device_id`，就会出现一个非常典型的问题：

1. 终端里能看到心跳日志
2. 但 UI 仍显示“未连接”

原因不是没收到心跳，而是心跳写进了错误的键空间，导致分配页和监考页按学生键查询不到。

### 5. 教师端真实状态回流点

分配页和监考页现在都不再只按 `ip_addr` 推导状态，而是走统一的状态查询链路：

1. 前端调用 `get_student_device_connection_status_by_exam_id`
2. Rust 命令入口仍在 `controllers/student_exam_controller.rs`
3. 服务层在 `services/student_exam_service.rs` 中按考试聚合分配记录与内存心跳快照
4. 最终输出四态：待分配、未连接、正常、异常

所以这条链路的真正“UI 出口”不是 WebSocket 收到心跳本身，而是：

`state.touch_connection` 更新成功后，`get_student_device_connection_status_by_exam_id` 能否按同一个 `student_id` 返回正确状态。

## 学生端监听入口

学生端应用启动时，在 apps/student/src-tauri/src/lib.rs 的 setup 里会异步启动两个后台任务：

1. discovery_listener::start
2. control_server::start

其中和这条链路直接相关的是 control_server::start。

apps/student/src-tauri/src/network/control_server.rs 中：

1. 读取配置里的 control_port。
2. 通过 `network/transport/tcp_request_reply.rs::bind_listener(...)` 绑定 0.0.0.0:control_port。
3. accept 新的 TCP 连接。
4. 每个连接交给 handle_client 处理。

因此，你说“通过监听对应端口获取教师端广播内容”这半句不准确。

更准确的说法应该是：

学生端通过 control_server 监听 TCP 控制端口，接收教师端直连发送的 APPLY_TEACHER_ENDPOINTS 请求内容。

## 学生端落库出口

apps/student/src-tauri/src/network/control_server.rs 的 handle_client 在收到请求后：

1. 先通过 `network/transport/tcp_request_reply.rs::read_json_request(...)` 统一读取 JSON 请求体。
2. 反序列化为 ApplyTeacherEndpointsRequest。
3. 校验 type 是否为 APPLY_TEACHER_ENDPOINTS。
4. 调用 TeacherEndpointsService::replace_all(&app_handle, &req.payload.endpoints)。
5. 地址落库成功后，再调用 `ExamRuntimeService::upsert_connected_session(&app_handle, &req.payload)` 预写入本地 `exam_sessions`。

地址写库逻辑在 apps/student/src-tauri/src/services/teacher_endpoints_service.rs：

1. 校验 endpoints 非空。
2. 校验 isMaster=true 的记录必须且只能有一条。
3. 开启数据库事务。
4. 先 delete_many 清空 teacher_endpoints 表。
5. 再逐条 insert 新的 endpoints。
6. 最后 txn.commit()。

连接阶段会话写库逻辑在 apps/student/src-tauri/src/services/exam_runtime_service.rs：

1. 当 payload 中携带 `session_id/exam_id/exam_title/student_no/student_name/assigned_ip_addr` 时，执行 `upsert_connected_session`。
2. 若本地已存在相同 `session_id`，则更新为 `connected_pending_distribution` 状态。
3. 若不存在，则插入一条最小会话记录。
4. 此时不写入 `exam_snapshots`。

所以，如果要严格定义“出口”，应该分三层：

1. 业务出口：replace_all 的 txn.commit() 成功，teacher_endpoints 表完成批量替换。
2. 会话出口：`upsert_connected_session` 成功，学生端 `exam_sessions` 完成连接阶段预写入。
3. 链路出口：学生端返回 APPLY_TEACHER_ENDPOINTS_ACK，教师端拿到 successCount 和每台设备的回执。

如果你关注的是“IP/教师地址何时真正进入学生端数据库”，那真正出口应写成：

apps/student/src-tauri/src/services/teacher_endpoints_service.rs 的 replace_all 完成事务提交。

而不是 handle_client 本身。

## 落库后的附加动作

handle_client 在 replace_all 成功后，还会做一个附加动作：

1. 从 endpoints 中取出主教师端地址。
2. 调用 apps/student/src-tauri/src/services/ws_reconnect_service.rs 的统一重连入口，而不是只做一次性 `connect`。
3. `ws_reconnect_service` 会记录当前目标 endpoint 与 `student_id`，并在启动恢复、首次失败重试、断线后重连、目标切换时复用同一套逻辑。
4. 真正建连时仍由 apps/student/src-tauri/src/network/ws_client.rs 调用 `network/transport/ws_transport.rs::connect_ws(...)` 建立 WebSocket 连接。
5. 后续发送通道和写循环由 `network/transport/ws_transport.rs::run_text_writer_loop(...)` 承接。

这一步是“落库成功后的后续动作”，不是“地址入库”的出口。

另外，当前实现里这次 `APPLY_TEACHER_ENDPOINTS` 的 ACK success 仍只取决于地址与会话落库是否成功，不直接取决于该次 WebSocket 是否马上连上；若首次建连失败，后续会交给 `ws_reconnect_service` 持续重试。

对分配页链路再补充一点：

1. 分配页里“连接请求已下发：成功 X/Y”现在说明 TCP 下发、学生端地址落库，以及连接阶段会话预写入成功。
2. UI 最终是否显示“正常”，还取决于学生端后续是否真的连上教师端 WebSocket，并且教师端是否按正确的 `student_id` 聚合到了心跳。
3. 学生端 Header 的考试标题和学生名称现在会按本地 `currentSession` 直接恢复；连接状态是否正常、是否处于重连中，则由独立的连接状态图标表达。

## 相关数据表

学生端落库目标表是 apps/student/src-tauri/migrations/0007_create_teacher_endpoints.sql 中定义的 teacher_endpoints：

1. id
2. endpoint
3. name
4. remark
5. is_master
6. last_seen
7. created_at
8. updated_at

当前 replace_all 采用“全量删除，再全量插入”的替换式写法，不是增量更新。

## 一句话总结

教师端“下发教师地址”的最短链路是：前端表单提交 -> Tauri IPC -> 教师端 Rust 根据已知学生 IP 逐台 TCP 直连 -> 教师端 transport 薄层执行 request-reply -> 学生端 control_server::handle_client 收包 -> 学生端 transport 薄层完成 bind/read/write -> TeacherEndpointsService::replace_all 事务提交落库 -> 可选发起 WS 连接 -> 返回 ACK。

因此，真正出口应写为 replace_all 的事务提交成功，而不是“监听端口的 handle_client 本身”。

而教师端“分配页连接考生设备并刷新真实状态”的最短链路是：分配页按钮 -> Tauri IPC -> 教师端按考试分配记录逐台下发携带会话字段的 `APPLY_TEACHER_ENDPOINTS` -> 教师端 transport 薄层执行 TCP request-reply -> 学生端先落库 `teacher_endpoints` 再预写入 `exam_sessions` -> 学生端连接教师端 WebSocket -> 学生端 transport 薄层执行 WS connect / writer loop -> 学生端以真实 `student_id` 持续发送心跳 -> 教师端按 `student_id` 更新连接快照 -> 分配页与监考页统一查询四态状态，同时学生端 Header 在 ws connected 后显示考试标题和学生名称。