# network 层 WS 封装与 TCP 浅封装计划（2026-03-20）

## 目标

在不一次性打爆现有业务链路的前提下，完成以下两件事：

1. 对 `tokio-tungstenite` 做一层稳定、可复用的 network 层封装。
2. 对当前基于 Tokio 的 TCP request-reply 通道做浅封装，隐藏底层连接、超时、读写与 ACK 细节。

本计划严格遵循以下实施顺序：

1. 先封装函数。
2. 再替换已有业务逻辑。

---

## 结论

### 1. WS 需要封装

原因如下：

1. 当前 WebSocket 连接建立、消息发送、心跳、断线清理与消息解析分散在教师端和学生端网络文件中。
2. 后续主业务协议的收敛方向是 WebSocket，因此 WebSocket 封装应优先于 TCP 封装。
3. 若不先封装，后续继续新增开考、暂停、结束、强制交卷、答案同步等逻辑时，业务层会不断侵入底层收发代码。

### 2. TCP 只做浅封装

原因如下：

1. 当前 TCP 主要承担短连接 request-reply 型控制动作。
2. 后续主业务通道不建议继续扩展裸 TCP。
3. 因此 TCP 本次不做大重构，不引入复杂长连接分帧协议，只抽出稳定的薄层函数，统一连接、发送、关闭写端、读取 ACK 与错误包装。

### 3. 封装位置放在 network 下，不放在 utils 下

本次明确采用以下原则：

1. 传输层封装属于 network 语义，不属于通用 utils。
2. 新增封装统一放在各端 `src/network/` 下。
3. 若需要再分层，优先考虑 `network/transport/`、`network/ws/`、`network/tcp/` 这样的语义目录。

---

## 现状与问题

### 一、当前 WS 现状

当前 WebSocket 相关代码主要分布于：

1. 教师端 `apps/teacher/src-tauri/src/network/ws_server.rs`
2. 学生端 `apps/student/src-tauri/src/network/ws_client.rs`
3. 两端各自的 `apps/*/src-tauri/src/network/protocol.rs`

主要问题：

1. 连接管理、消息解析、发送队列、心跳逻辑未抽象成稳定接口。
2. 协议结构虽然相似，但教师端与学生端各自维护，后续扩展容易发散。
3. 业务动作与传输细节混写在同一文件中，后续替换或重构风险高。

### 二、当前 TCP 现状

当前 TCP 相关代码主要分布于：

1. 教师端 `apps/teacher/src-tauri/src/network/student_control_client.rs`
2. 学生端 `apps/student/src-tauri/src/network/control_server.rs`

主要问题：

1. 业务函数直接操作 `TcpStream` 与 `TcpListener`。
2. 超时、连接、写入、半关闭、读取 ACK 的处理逻辑耦合在业务函数中。
3. 学生端服务端当前自行负责 JSON 请求体读取与边界判断，不利于后续复用。

---

## 迁移原则

本计划执行时必须遵守以下原则：

1. 先新增封装函数，再让旧业务调用新封装。
2. 在封装层稳定前，不直接改动业务语义。
3. 迁移期间允许旧消息结构继续存在，但新代码默认必须走新封装入口。
4. TCP 本次只做浅封装，不顺手升级为长连接统一总线。
5. WS 是后续主业务方向，因此先做 WS 再做 TCP。

---

## 推荐目录方案

以下为建议目录落点，重点是语义归属，不要求一步到位全部创建：

### 教师端

1. `apps/teacher/src-tauri/src/network/protocol.rs`
2. `apps/teacher/src-tauri/src/network/mod.rs`
3. `apps/teacher/src-tauri/src/network/transport/ws_transport.rs`
4. `apps/teacher/src-tauri/src/network/transport/tcp_request_reply.rs`
5. `apps/teacher/src-tauri/src/network/ws_server.rs`
6. `apps/teacher/src-tauri/src/network/student_control_client.rs`

### 学生端

1. `apps/student/src-tauri/src/network/protocol.rs`
2. `apps/student/src-tauri/src/network/mod.rs`
3. `apps/student/src-tauri/src/network/transport/ws_transport.rs`
4. `apps/student/src-tauri/src/network/transport/tcp_request_reply.rs`
5. `apps/student/src-tauri/src/network/ws_client.rs`
6. `apps/student/src-tauri/src/network/control_server.rs`

### 共享协议参考

1. `packages/shared-types/src/protocol.ts`

说明：

1. 若第一阶段希望改动更小，也可先不建 `transport/` 目录，先在 `network/` 下新增独立文件。
2. 但不建议放入 `utils/`，避免后续 network 职责继续发散。

---

## 第一阶段：先封装函数

本阶段目标只有一个：

把“传输细节”从“业务逻辑”里剥离出去。

本阶段结束标准：

1. 业务层不再需要直接操心底层 WS 发送、接收、心跳与连接清理细节。
2. 业务层不再需要直接操心 TCP connect、write、shutdown、read ACK 与超时细节。
3. 旧业务逻辑仍可暂时存在，但有明确可切换的新封装入口。

### 1.1 先抽统一协议辅助函数

目标：

1. 先让消息构造、序列化、反序列化有统一入口。
2. 为后续 WS V2 信封升级留接口。

建议事项：

1. 在 `protocol.rs` 中补消息构造 helper。
2. 将现有“构造消息 + JSON 序列化”逻辑从业务函数中挪出。
3. 先做到统一 build/send 所需 helper，不强求一次切到完整 V2。

建议先抽出的能力：

1. `build_message(...)`
2. `encode_message(...)`
3. `decode_message(...)`
4. `build_ack(...)`

### 1.2 封装 WS 基础函数

目标：

1. 封装 `tokio-tungstenite` 的底层使用方式。
2. 让业务层只关心“连接成功没有”“发送什么消息”“收到什么消息”。

教师端建议封装能力：

1. 启动 WS server。
2. 接受连接并注册 session。
3. 统一接收文本消息并解析为协议对象。
4. 统一向指定学生发送消息。
5. 统一广播消息。
6. 统一断连清理与日志。

学生端建议封装能力：

1. 连接教师端 WS。
2. 统一发送文本消息。
3. 统一启动心跳任务。
4. 统一接收教师端消息并分发给业务 handler。
5. 统一断连状态清理与事件上报。

本阶段不要求：

1. 立即修改所有业务 payload。
2. 立即迁移所有业务消息到新信封。

### 1.3 浅封装 TCP 基础函数

目标：

1. 让业务代码不再直接操心 `TcpStream` 细节。
2. 保持现有短连接 request-reply 语义不变。

教师端建议封装能力：

1. 建立 TCP 连接。
2. 发送 JSON 请求。
3. 半关闭写端。
4. 读取 ACK。
5. 统一超时与错误包装。

学生端建议封装能力：

1. 启动 TCP listener。
2. 统一读取单次请求。
3. 统一解析 JSON 请求体。
4. 统一写回 ACK。
5. 将真正业务处理交回上层 handler。

本阶段不做：

1. 长连接改造。
2. 长度分帧协议升级。
3. TCP 统一承载全部业务。

---

## 第二阶段：再替换已有业务逻辑

本阶段目标：

在封装函数已经稳定的前提下，将现有业务逐步改为调用新封装，而不是继续直接操作底层网络库。

### 2.1 先替换 WS 业务入口

建议替换顺序：

1. 学生端 `connect` 调用路径。
2. 学生端心跳发送路径。
3. 教师端消息接收解析入口。
4. 教师端按 student_id 定向发送路径。
5. 学生端接收 `EXAM_START` 的处理入口。

这个顺序的原因：

1. WS 是后续主业务方向。
2. 当前 WS 已经承担连接状态维护与开考控制，价值更高。
3. 先替换 WS，可以提前稳定新的主通道。

### 2.2 再替换 TCP 业务入口

建议替换顺序：

1. 教师端 `apply_teacher_endpoints`。
2. 教师端 `distribute_exam_paper`。
3. 学生端 `control_server` 请求读取与 ACK 输出。

要求：

1. 替换后业务请求 DTO 与 ACK DTO 先保持不变。
2. 优先做到行为等价。
3. 不在本阶段把 TCP 业务直接迁移成 WS 业务。

### 2.3 最后清理网络文件中的传输细节

待替换稳定后，再清理以下内容：

1. 网络文件中重复的 JSON 序列化代码。
2. 网络文件中重复的错误包装与日志。
3. 网络文件中直接操作底层 socket 的代码。

目标：

1. `ws_server.rs` 和 `ws_client.rs` 更偏向 session 管理与 handler 分发。
2. `student_control_client.rs` 和 `control_server.rs` 更偏向业务请求编排与 handler 调度。
3. 真正的连接读写细节收敛到 `network/transport`。

---

## 建议实施顺序

### Phase 1：边界冻结

1. 确认封装放在 `network` 下。
2. 确认 TCP 仅做浅封装。
3. 确认 WS 为主业务通道优先封装对象。

### Phase 2：协议辅助函数

1. 在 `protocol.ts`、教师端 `protocol.rs`、学生端 `protocol.rs` 中补齐辅助构造函数。
2. 建立统一 encode/decode 入口。

### Phase 3：WS 封装函数

1. 新增 `ws_transport.rs`。
2. 抽离 connect、send、recv、heartbeat、disconnect cleanup。
3. 教师端抽离 session register / route send / broadcast。

### Phase 4：TCP 浅封装函数

1. 新增 `tcp_request_reply.rs`。
2. 抽离 connect、write、shutdown、read reply、timeout。
3. 抽离服务端 read request、decode request、write ack。

### Phase 5：替换 WS 业务调用

1. 先替换学生端连接与心跳。
2. 再替换教师端接收与发送入口。
3. 保持现有业务 handler 不变。

### Phase 6：替换 TCP 业务调用

1. 替换教师端配置下发。
2. 替换教师端发卷。
3. 替换学生端控制服务入口。

### Phase 7：清理与收口

1. 清理旧的传输细节代码。
2. 让新增业务只能走封装层。
3. 为后续 `EXAM_PAPER_DISTRIBUTE` 迁移到 WS 预留接口。

---

## 风险与控制措施

### 风险 1：边封装边改业务，回归面失控

控制措施：

1. 严格分成“先封装函数”和“再替换业务”两个阶段。
2. 每个阶段完成后先做编译与行为验证，再进入下一阶段。

### 风险 2：WS 封装过程中把现有心跳链路搞断

控制措施：

1. 先保持现有消息格式不变。
2. 优先替换底层 transport，不同时改业务 payload。

### 风险 3：TCP 浅封装误改现有请求边界行为

控制措施：

1. 继续保留短连接 request-reply 模式。
2. 保留 `write -> shutdown -> read ACK` 的现有语义。
3. 不在本次计划中引入长度分帧协议。

### 风险 4：目录职责再次发散

控制措施：

1. 明确封装放在 `network` 下。
2. 禁止把网络传输层继续塞进 `utils`。

---

## 验证要求

每个阶段完成后，至少验证以下内容：

1. 教师端 Rust 编译通过。
2. 学生端 Rust 编译通过。
3. 学生端仍能建立 WS 连接并上报心跳。
4. 教师端仍能按 student_id 定向发送 `EXAM_START`。
5. TCP 配置下发仍能收到 ACK。
6. TCP 发卷仍能收到 ACK。

---

## 后续衔接

本计划完成后，后续可继续进入两项工作：

1. 按 `doc/plans/2026_03_20_协议统一与WS信封设计方案.md` 的方向，将消息模型升级到统一 WS V2 信封。
2. 在 WS transport 稳定后，评估是否将 `EXAM_PAPER_DISTRIBUTE` 从 TCP 逐步迁移至 WS 或“WS 控制 + HTTP 下载”模式。

---

## 最终结论

本次计划的明确执行策略如下：

1. 封装位置放在 `network`，不放在 `utils`。
2. 先封装 `tokio-tungstenite` 的 WS 基础函数。
3. 再浅封装现有 TCP request-reply 基础函数。
4. 封装完成后，再逐步替换已有业务逻辑。
5. 本阶段目标是建立稳定边界，不是一次性改完所有协议。
