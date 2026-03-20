# 教师端随机分配考生到学生端 exam_sessions 的最短链路

## 目标

这份文档回答两个问题：

1. 教师端“分配考生”页点击“随机分配考生”之后，业务最短链路到底走到哪里。
2. 学生端接收到考生与考试数据并写入 `exam_sessions` 表的链路，和“随机分配考生”之间是什么关系。

结论先说清楚：

1. “随机分配考生”本身的真实出口不在学生端，也不会直接写入学生端 `exam_sessions`。
2. 它只是在教师端本地把 `student_exams.ip_addr` 更新为某台设备 IP，建立“考生 -> 设备”的映射。
3. 学生端 `exam_sessions` 现在会在后续“连接考生设备”阶段先预写入一条最小会话，在“分发试卷”阶段再补齐或更新试卷快照。
4. 如果你想要从教师端“随机分配考生”一路追到学生端页面头部，那么中间至少要经过“连接考生设备”这段链路；若要拿到题目快照，还要继续经过“分发试卷”。

## 一句话结论

最短业务事实链是：

教师端点击“随机分配考生” -> 教师端前端 Hook 生成随机 `student_exam_id -> ip_addr` 映射 -> Tauri `assign_devices_to_student_exams` -> 教师端 Rust repo 更新 `student_exams.ip_addr`。

而学生端 `exam_sessions` 的最短落库链是另一段：

教师端“连接考生设备” -> 教师端 Rust 读取已分配的 `student_exams.ip_addr`、考试信息和考生信息 -> 逐台 TCP 发 `APPLY_TEACHER_ENDPOINTS` -> 学生端 `control_server` 先落库 `teacher_endpoints`，再调用 `ExamRuntimeService::upsert_connected_session` 预写入 `exam_sessions`。

所以，如果把你要的链路压缩成“最短且真实”的形式，应该拆成两段，而不是误写成一条：

1. 分配链：随机分配考生，只发生在教师端。
2. 会话链：基于分配结果连接设备，先进入学生端 `exam_sessions`。
3. 发卷链：基于已建立会话分发试卷，补齐或更新 `exam_snapshots`。

## 链路 1：教师端随机分配考生

### 入口

入口在教师端分配考生页面的“随机分配考生”按钮：

- [apps/teacher/src/pages/DeviceAssign/index.tsx](apps/teacher/src/pages/DeviceAssign/index.tsx)

页面点击后执行 `handleRandomAssign`，内部调用 `randomAssign()`。

### 前端 Hook

Hook 在：

- [apps/teacher/src/hooks/useDeviceAssign.ts](apps/teacher/src/hooks/useDeviceAssign.ts)

`randomAssign` 的职责很集中：

1. 读取当前考试下的考生分配视图 `allAssignments`。
2. 读取全部设备 `allDevices`。
3. 分别打乱考生和设备数组。
4. 生成 `student_exam_id -> ip_addr` 的随机映射。
5. 组装 `StudentDeviceAssignPayloadItem[]`。
6. 调用前端 service `assignDevicesToStudentExams(selectedExamId, payload)`。
7. 完成后再次调用 `loadAssignments(selectedExamId)` 刷新表格。

这里有一个关键事实：

这个 Hook 完全没有和学生端通信，它只是在教师端本地准备好“哪个考生分到哪个 IP”。

### 前端 Service -> Tauri IPC

前端 service 在：

- [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts)

`assignDevicesToStudentExams` 实际发起：

`invoke("assign_devices_to_student_exams", { payload: { exam_id, assignments } })`

这一步仍然是教师端前端到教师端 Rust 的本地 IPC。

### 教师端 Rust Controller

命令入口在：

- [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs)

`assign_devices_to_student_exams` 只做一件事：

调用 `student_exam_service::assign_devices_to_student_exams(pool, payload.exam_id, payload.assignments)`。

### 教师端 Rust Service

服务层在：

- [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs)

这里的 `assign_devices_to_student_exams` 也没有额外逻辑，只继续把调用下沉到 repo。

### 教师端 Rust Repo，真实出口

真实写库出口在：

- [apps/teacher/src-tauri/src/repos/student_exam_repo.rs](apps/teacher/src-tauri/src/repos/student_exam_repo.rs)

`assign_devices_to_student_exams` 会在事务里循环更新：

1. 通过 `student_exam_id` 精确定位某条 `student_exams` 记录。
2. 校验同属当前 `exam_id`。
3. 把 `student_exams.ip_addr` 更新为随机分配出来的设备 IP。

因此，这条链路的真实出口是：

教师端数据库中的 `student_exams.ip_addr` 被更新。

不是学生端，不是 WebSocket，也不是 `exam_sessions`。

## 链路 2：从分配结果进入学生端 exam_sessions

### 为什么需要第二段链路

教师端完成随机分配后，系统只知道“某个学生考试记录应该对应某个设备 IP”。

但学生端本机还不知道：

1. 自己对应哪个学生。
2. 自己属于哪场考试。
3. 试卷内容是什么。
4. 应该生成哪条 `exam_sessions`。

这些信息不是在“随机分配考生”阶段推给学生端的，而是在后续“连接考生设备”和“分发试卷”阶段才送达。

### 2.1 连接考生设备：把教师地址推给学生端

这一步现在不再只是“教师地址下发”的中间桥，而是第一阶段真正把当前考试会话预写入学生端的入口。

入口仍在同一个页面：

- [apps/teacher/src/pages/DeviceAssign/index.tsx](apps/teacher/src/pages/DeviceAssign/index.tsx)

按钮“连接考生设备”会调用 Hook 中的 `connectDevices()`。

Hook 实现在：

- [apps/teacher/src/hooks/useDeviceAssign.ts](apps/teacher/src/hooks/useDeviceAssign.ts)

前端 service 仍在：

- [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts)

它通过 Tauri 调用 `connect_student_devices_by_exam_id`。

教师端 Rust 控制器在：

- [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs)

`connect_student_devices_by_exam_id` 会：

1. 查询该考试下所有已分配的 `student_exams`。
2. 通过 IP 关联设备表，整理目标列表。
3. 读取当前考试信息。
4. 组装 `APPLY_TEACHER_ENDPOINTS` 请求，其中除了 `payload.student_id` 外，还会带上 `session_id`、`exam_id`、`exam_title`、`student_no`、`student_name`、`assigned_ip_addr`、考试时间等连接阶段会话字段。
4. 逐台通过 `student_control_client::apply_teacher_endpoints` 连接学生端控制端口。

学生端接收入口在：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

收到 `APPLY_TEACHER_ENDPOINTS` 后会调用：

- [apps/student/src-tauri/src/services/teacher_endpoints_service.rs](apps/student/src-tauri/src/services/teacher_endpoints_service.rs)

并继续调用：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

`TeacherEndpointsService::replace_all` 会把教师 WebSocket 地址写入学生端本地 `teacher_endpoints` 表。随后 `ExamRuntimeService::upsert_connected_session` 会把连接阶段携带的考试与考生信息预写入学生端本地 `exam_sessions` 表，然后学生端尝试主动连接教师端 WebSocket。

这里要特别澄清一个容易误判的点：

1. `APPLY_TEACHER_ENDPOINTS` 现在除了 `configVersion`、`studentId` 和 `endpoints` 外，还会携带连接阶段最小会话字段。
2. 它仍然不包含 `questionsPayload` 这类试卷快照内容，所以连接阶段只会建立会话，不会写入题目快照。
3. 学生端 `control_server` 在这条分支里会先持久化 `teacher_endpoints`，再预写入 `exam_sessions`。
4. `req.payload.student_id` 仍然会继续传给 `ws_client::connect`，用于后续 WebSocket 心跳和开考指令过滤。

这一段的真实出口是：

学生端已经同时拿到：

1. 要连接哪个教师端
2. 当前设备对应哪场考试、哪个学生
3. 一条最小的本地 `exam_sessions`

换句话说，“连接考生设备”现在已经会把考试信息和考生信息预写入学生端本地数据库，但还不会写入 `exam_snapshots`。

### 2.2 分发试卷：把考生与考试数据推到学生端

学生端 `exam_snapshots` 的写入从这里开始。

教师端调用入口在考试管理相关链路，不在分配页本身。最终前端 service 还是走：

- [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts)

对应 Tauri 命令：`distribute_exam_papers_by_exam_id`。

教师端 Rust 控制器在：

- [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs)

控制器继续调用：

- [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs)

`distribute_exam_papers_by_exam_id` 会：

1. 读取考试信息。
2. 读取题目列表。
3. 再次读取当前考试下已经分配了设备 IP 的 `student_exams`。
4. 按每个 `device_ip` 组装 `DISTRIBUTE_EXAM_PAPER`。
5. 在报文里带上：
   - `session_id = student_exam_id`
   - `exam_id`
   - `student_id`
   - `student_no`
   - `student_name`
   - `assigned_ip_addr`
   - `exam_title`
   - `questions_payload`
   - 其他会话与时间字段
6. 通过 TCP 逐台发送给学生端控制服务。

因此，学生端得到“考生信息 + 考试信息 + 试卷内容”这件事，不是在随机分配时发生，而是在发卷时发生。

但需要更新的是：学生端得到“考生信息 + 考试基础信息”这件事，现在已经可以在“连接考生设备”阶段发生。

### 学生端接收入口

学生端接收发卷请求的入口在：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

`handle_client` 对 `type == "DISTRIBUTE_EXAM_PAPER"` 的分支会：

1. 反序列化 `DistributeExamPaperRequest`。
2. 调用 `ExamRuntimeService::upsert_distribution(&app_handle, &req.payload)`。
3. 根据结果返回 `DISTRIBUTE_EXAM_PAPER_ACK`。

### 学生端真实出口：试卷快照落库与会话补齐

真实出口在：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

`ExamRuntimeService::upsert_distribution` 现在会先按 `exam_id` 检查本地是否已存在会话：

1. 若不存在同 `exam_id` 的本地会话，则按原逻辑写入或更新 `exam_sessions`，并写入 `exam_snapshots`。
2. 若已存在同 `exam_id` 的本地会话，则保留本地已有的 `exam_sessions` 基础信息，不再覆盖更新考试基础字段；同时把试卷快照写入或更新到本地已有会话对应的 `exam_snapshots`。

写入 `exam_sessions` 的字段包括：

1. `id = session_id`
2. `exam_id`
3. `student_id`
4. `student_no`
5. `student_name`
6. `assigned_ip_addr`
7. `exam_title`
8. `status = "waiting"`
9. `assignment_status`
10. `ends_at`
11. `paper_version`
12. `created_at / updated_at`

对应表结构定义在：

- [apps/student/src-tauri/migrations/0002_create_exam_sessions.sql](apps/student/src-tauri/migrations/0002_create_exam_sessions.sql)

所以，如果你要找“学生端接收学生端分配的考生与考生数据并储存到 exam_sessions 表中的业务逻辑链条”，最新代码下最准确的写法是：

教师端先通过随机分配把 `student_exams.ip_addr` 建好，然后在连接考生设备时把该条分配记录上的考生与考试基础信息打包发送到学生端，由 `ExamRuntimeService::upsert_connected_session` 预写入 `exam_sessions`；后续发卷时再由 `ExamRuntimeService::upsert_distribution` 写入或更新 `exam_snapshots`，并在命中相同 `exam_id` 时保留本地已有的会话基础信息。

这里和“连接考生设备”形成清晰对照：

1. `connect_student_devices_by_exam_id` 现在负责教师地址下发、当前会话预写入，以及后续心跳身份问题。
2. `distribute_exam_papers_by_exam_id` 负责试卷快照下发；命中相同 `exam_id` 时不再覆盖本地已有会话基础信息。

## 学生端头部内容和这条链的关系

你提到的出口是“学生端头部内容”，这个说法需要拆开看。

### 头部中真正已经接上线的部分

头部组件在：

- [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx)

这里现在有两类稳定数据源：

1. `currentSession/currentExam`，来自 `useExamStore`。
2. `teacherMasterEndpoint/teacherConnectionStatus`，来自 `useDeviceStore`。

`currentExam` 的链路是接通的：

1. [apps/student/src/App.tsx](apps/student/src/App.tsx) 启动后定时调用 `refreshCurrentExam()`。
2. [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 通过 `getCurrentExamBundle()` 拉取最新会话。
3. [apps/student/src/services/examRuntimeService.ts](apps/student/src/services/examRuntimeService.ts) 调用 Tauri `get_current_exam_bundle`。
4. [apps/student/src-tauri/src/commands.rs](apps/student/src-tauri/src/commands.rs) 转到 `ExamRuntimeService::get_current_exam_bundle`。
5. [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 从 `exam_sessions` 和 `exam_snapshots` 读取最新记录返回给前端。

因此，头部里考试标题、状态和学生名称，已经改为走 `exam_sessions -> get_current_exam_bundle -> examStore -> AppHeader` 这条链。

### 头部中与连接成功口径相关的部分

当前代码里，Header 会在学生端 WebSocket 真正连接教师端成功后，基于 `currentSession` 显示：

1. 考试标题
2. 会话状态
3. 学生名称

因此，若把“学生端头部内容”当作出口，需要精确写成：

1. 头部中的考试标题、状态与学生名称，已经能通过 `exam_sessions -> get_current_exam_bundle -> examStore -> AppHeader` 体现出来。
2. 这些业务信息的最终展示时机仍受 `deviceStore.teacherConnectionStatus === connected` 控制，也就是以 WebSocket 真连接成功为准。

## 最短 e2e 链路图

### A. 随机分配链

1. 教师端 [apps/teacher/src/pages/DeviceAssign/index.tsx](apps/teacher/src/pages/DeviceAssign/index.tsx) 点击“随机分配考生”
2. 教师端 [apps/teacher/src/hooks/useDeviceAssign.ts](apps/teacher/src/hooks/useDeviceAssign.ts) `randomAssign()` 生成随机映射
3. 教师端 [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts) 调用 `assign_devices_to_student_exams`
4. 教师端 [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs) 进入命令层
5. 教师端 [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs) 下沉到 repo
6. 教师端 [apps/teacher/src-tauri/src/repos/student_exam_repo.rs](apps/teacher/src-tauri/src/repos/student_exam_repo.rs) 更新 `student_exams.ip_addr`
7. 结束，出口仍在教师端数据库

### B. 基于分配结果的连接会话链

1. 教师端读取已分配的 `student_exams.ip_addr`
2. 教师端发起 `APPLY_TEACHER_ENDPOINTS`
3. 学生端 [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs) 接收请求
4. 学生端 [apps/student/src-tauri/src/services/teacher_endpoints_service.rs](apps/student/src-tauri/src/services/teacher_endpoints_service.rs) 落库 `teacher_endpoints`
5. 学生端 [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 执行 `upsert_connected_session`
6. 学生端 [apps/student/src-tauri/migrations/0002_create_exam_sessions.sql](apps/student/src-tauri/migrations/0002_create_exam_sessions.sql) 对应的 `exam_sessions` 被预写入
7. 学生端 [apps/student/src-tauri/src/network/ws_client.rs](apps/student/src-tauri/src/network/ws_client.rs) 建立 WebSocket 连接
8. 学生端 [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) `get_current_exam_bundle`
9. 学生端 [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 更新 `currentExam/currentSession`
10. 学生端 [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 在 ws connected 后显示考试标题、状态与学生名称

### C. 基于分配结果的发卷快照链

1. 教师端读取已分配的 `student_exams.ip_addr`
2. 教师端发起 `DISTRIBUTE_EXAM_PAPER`
3. 学生端 [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs) 接收请求
4. 学生端 [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 执行 `upsert_distribution`
5. 学生端 [apps/student/src-tauri/migrations/0002_create_exam_sessions.sql](apps/student/src-tauri/migrations/0002_create_exam_sessions.sql) 对应的 `exam_sessions` 被写入
6. 学生端 `exam_snapshots` 被写入或更新
7. 若已存在相同 `exam_id` 的本地会话，则保留原有 `exam_sessions` 基础信息，仅更新快照

## 最终结论

如果严格按代码事实来写：

1. 教师端“随机分配考生”的最短出口是教师端 `student_exams.ip_addr` 更新成功。
2. 学生端 `exam_sessions` 的最短入口不是“随机分配考生”，而是后续“连接考生设备”的 `APPLY_TEACHER_ENDPOINTS`。
3. 学生端头部里，考试标题、状态和学生名称现在都已接到 `examStore.currentSession`，但最终展示仍以 WebSocket 真连接成功为准。

所以这条业务现在应描述成：

随机分配先建立教师端的考生-设备映射，后续连接设备阶段复用这份映射，把考生与考试基础信息送到学生端并预写入 `exam_sessions`；再在发卷阶段补齐或更新试卷快照，并在命中相同 `exam_id` 时保留本地已有的会话基础信息。