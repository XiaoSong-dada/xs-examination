# 教师端初始化学生端 IP/教师地址的最小 e2e 链路

## 目标

这份文档只梳理一条最短业务链：教师端在“设备管理”页面录入教师端地址后，如何把这些地址下发到学生端，并最终写入学生端本地数据库。

这里先纠正两个容易混淆的点：

1. 这条“下发教师地址”链路不是 UDP 广播。
2. 这条链路的真正网络传输方式，是教师端根据已选中的学生设备 IP，逐台通过 TCP 直连学生端控制端口。

UDP 监听只出现在学生端的设备发现链路里，对应 apps/student/src-tauri/src/network/discovery_listener.rs；它不参与本次“下发教师地址”的最短链路。

## 最短链路结论

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

## 入口到出口的精简调用链

### 1. 教师端页面入口

入口在 apps/teacher/src/pages/Devices/index.tsx 的 handlePushTeacherEndpoints。

它做了三件事：

1. 从表单读取 masterEndpoint、slaveEndpoint、controlPort、remark。
2. 组装 endpoints 数组，其中主地址 isMaster=true，备地址 isMaster=false。
3. 调用前端服务 pushTeacherEndpointsToDevices(payload)。

这里并没有任何 UDP 广播逻辑，只是组装数据并发起一次 Tauri IPC。

### 2. 教师端前端到 Rust IPC

apps/teacher/src/services/deviceService.ts 中的 pushTeacherEndpointsToDevices 通过 invoke 调用 Tauri 命令 push_teacher_endpoints_to_devices。

对应 Rust 命令入口在 apps/teacher/src-tauri/src/controllers/device_controller.rs。

这一步只是桥接，不做实际网络下发。

### 3. 教师端 Rust 真实下发点

真实下发逻辑在 apps/teacher/src-tauri/src/services/device_service.rs 的 push_teacher_endpoints_to_devices。

这个函数的实际行为是：

1. 遍历前端传入的 deviceIds。
2. 通过 device_repo::get_device_by_id 从教师端本地设备表查出每台学生设备的 IP。
3. 组装 ApplyTeacherEndpointsRequest，请求类型为 APPLY_TEACHER_ENDPOINTS。
4. 调用 student_control_client::apply_teacher_endpoints(&device.ip, control_port, &req)。

因此，链路依赖的是“教师端本地已经知道学生设备 IP”。这些 IP 来自更早的设备发现/录入流程，不是在这里临时广播得到的。

### 4. 教师端到学生端的真实传输协议

apps/teacher/src-tauri/src/network/student_control_client.rs 中的 apply_teacher_endpoints 明确使用的是 TCP：

1. 用 TcpStream::connect 连接 student_ip:control_port。
2. 把请求 JSON 写入连接。
3. 读取学生端返回的 ACK。

默认 control_port 由前端传入，未传时教师端后端默认使用 18889。学生端默认 control_port 也是 18889，定义在 apps/student/src-tauri/src/config.rs。

所以这条链路的网络本质是：

教师端逐台 TCP 单播下发，不是 UDP 广播，也不是 UDP 组播。

## 学生端监听入口

学生端应用启动时，在 apps/student/src-tauri/src/lib.rs 的 setup 里会异步启动两个后台任务：

1. discovery_listener::start
2. control_server::start

其中和这条链路直接相关的是 control_server::start。

apps/student/src-tauri/src/network/control_server.rs 中：

1. 读取配置里的 control_port。
2. 绑定 0.0.0.0:control_port。
3. accept 新的 TCP 连接。
4. 每个连接交给 handle_client 处理。

因此，你说“通过监听对应端口获取教师端广播内容”这半句不准确。

更准确的说法应该是：

学生端通过 control_server 监听 TCP 控制端口，接收教师端直连发送的 APPLY_TEACHER_ENDPOINTS 请求内容。

## 学生端落库出口

apps/student/src-tauri/src/network/control_server.rs 的 handle_client 在收到请求后：

1. 反序列化为 ApplyTeacherEndpointsRequest。
2. 校验 type 是否为 APPLY_TEACHER_ENDPOINTS。
3. 调用 TeacherEndpointsService::replace_all(&app_handle, &req.payload.endpoints)。

真正写库逻辑在 apps/student/src-tauri/src/services/teacher_endpoints_service.rs：

1. 校验 endpoints 非空。
2. 校验 isMaster=true 的记录必须且只能有一条。
3. 开启数据库事务。
4. 先 delete_many 清空 teacher_endpoints 表。
5. 再逐条 insert 新的 endpoints。
6. 最后 txn.commit()。

所以，如果要严格定义“出口”，应该分两层：

1. 业务出口：replace_all 的 txn.commit() 成功，teacher_endpoints 表完成批量替换。
2. 链路出口：学生端返回 APPLY_TEACHER_ENDPOINTS_ACK，教师端拿到 successCount 和每台设备的回执。

如果你关注的是“IP/教师地址何时真正进入学生端数据库”，那真正出口应写成：

apps/student/src-tauri/src/services/teacher_endpoints_service.rs 的 replace_all 完成事务提交。

而不是 handle_client 本身。

## 落库后的附加动作

handle_client 在 replace_all 成功后，还会做一个附加动作：

1. 从 endpoints 中取出主教师端地址。
2. 调用 apps/student/src-tauri/src/network/ws_client.rs 的 connect。
3. 让学生端立刻尝试连接主教师端 WebSocket。

这一步是“落库成功后的后续动作”，不是“地址入库”的出口。

另外，当前实现里这次 WebSocket 连接失败不会反向影响本次 ACK 的 success 字段，因为 success 只取决于 replace_all 是否成功。

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

教师端“下发教师地址”的最短链路是：前端表单提交 -> Tauri IPC -> 教师端 Rust 根据已知学生 IP 逐台 TCP 直连 -> 学生端 control_server::handle_client 收包 -> TeacherEndpointsService::replace_all 事务提交落库 -> 可选发起 WS 连接 -> 返回 ACK。

因此，真正出口应写为 replace_all 的事务提交成功，而不是“监听端口的 handle_client 本身”。