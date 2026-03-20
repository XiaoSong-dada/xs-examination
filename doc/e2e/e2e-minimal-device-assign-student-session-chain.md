# 教师端随机分配考生到学生端 exam_sessions 的最短链路

## 目标

这份文档回答两个问题：

1. 教师端“分配考生”页点击“随机分配考生”之后，业务最短链路到底走到哪里。
2. 学生端接收到考生与考试数据并写入 `exam_sessions` 表的链路，和“随机分配考生”之间是什么关系。

结论先说清楚：

1. “随机分配考生”本身的真实出口不在学生端，也不会直接写入学生端 `exam_sessions`。
2. 它只是在教师端本地把 `student_exams.ip_addr` 更新为某台设备 IP，建立“考生 -> 设备”的映射。
3. 学生端 `exam_sessions` 的写入发生在后续“分发试卷”链路，而不是“随机分配考生”链路。
4. 如果你想要从教师端“随机分配考生”一路追到学生端页面头部，那么中间最少还要经过“连接考生设备”和“分发试卷”两段链路。

## 一句话结论

最短业务事实链是：

教师端点击“随机分配考生” -> 教师端前端 Hook 生成随机 `student_exam_id -> ip_addr` 映射 -> Tauri `assign_devices_to_student_exams` -> 教师端 Rust repo 更新 `student_exams.ip_addr`。

而学生端 `exam_sessions` 的最短落库链是另一段：

教师端“分发试卷” -> 教师端 Rust 读取已分配的 `student_exams.ip_addr` 和考生信息 -> 逐台 TCP 发 `DISTRIBUTE_EXAM_PAPER` -> 学生端 `control_server` 调用 `ExamRuntimeService::upsert_distribution` -> 写入 `exam_sessions`。

所以，如果把你要的链路压缩成“最短且真实”的形式，应该拆成两段，而不是误写成一条：

1. 分配链：随机分配考生，只发生在教师端。
2. 落库链：基于分配结果发卷，最终进入学生端 `exam_sessions`。

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

虽然这一步不写 `exam_sessions`，但它是从“已分配 IP”走向“学生端可被教师控制”的中间桥。

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
3. 组装 `APPLY_TEACHER_ENDPOINTS` 请求，其中 `payload.student_id` 使用的是考试分配记录里的 `student_id`。
4. 逐台通过 `student_control_client::apply_teacher_endpoints` 连接学生端控制端口。

学生端接收入口在：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

收到 `APPLY_TEACHER_ENDPOINTS` 后会调用：

- [apps/student/src-tauri/src/services/teacher_endpoints_service.rs](apps/student/src-tauri/src/services/teacher_endpoints_service.rs)

`TeacherEndpointsService::replace_all` 会把教师 WebSocket 地址写入学生端本地 `teacher_endpoints` 表，然后学生端尝试主动连接教师端 WebSocket。

这里要特别澄清一个容易误判的点：

1. `APPLY_TEACHER_ENDPOINTS` 的 payload 只有 `configVersion`、`studentId` 和 `endpoints`。
2. 它不包含 `examId`、`studentNo`、`studentName`、`examTitle`、`questionsPayload` 这些考试或考生展示信息。
3. 学生端 `control_server` 在这条分支里只调用 `TeacherEndpointsService::replace_all` 持久化 `teacher_endpoints`。
4. `req.payload.student_id` 没有被写入本地数据库，只被继续传给 `ws_client::connect`，用于后续 WebSocket 心跳和开考指令过滤。

这一段的真实出口是：

学生端已经知道“要连接哪个教师端”，但仍然没有 `exam_sessions`。

换句话说，“连接考生设备”这一步并不会把考试信息和考生信息写入学生端本地数据库。

### 2.2 分发试卷：把考生与考试数据推到学生端

学生端 `exam_sessions` 的写入从这里开始。

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

### 学生端接收入口

学生端接收发卷请求的入口在：

- [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs)

`handle_client` 对 `type == "DISTRIBUTE_EXAM_PAPER"` 的分支会：

1. 反序列化 `DistributeExamPaperRequest`。
2. 调用 `ExamRuntimeService::upsert_distribution(&app_handle, &req.payload)`。
3. 根据结果返回 `DISTRIBUTE_EXAM_PAPER_ACK`。

### 学生端真实出口：exam_sessions 落库

真实出口在：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

`ExamRuntimeService::upsert_distribution` 会按 `payload.session_id` 执行 upsert。

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

所以，如果你要找“学生端接收学生端分配的考生与考生数据并储存到 exam_sessions 表中的业务逻辑链条”，最准确的写法是：

教师端先通过随机分配把 `student_exams.ip_addr` 建好，然后在分发试卷时把该条分配记录上的考生与考试数据打包发送到学生端，最终由 `ExamRuntimeService::upsert_distribution` 写入 `exam_sessions`。

这里和“连接考生设备”形成清晰对照：

1. `connect_student_devices_by_exam_id` 只解决教师地址下发和后续心跳身份问题。
2. `distribute_exam_papers_by_exam_id` 才真正把 `student_id/student_no/student_name/exam_id/exam_title` 等数据送到学生端并落库。

## 学生端头部内容和这条链的关系

你提到的出口是“学生端头部内容”，这个说法需要拆开看。

### 头部中真正已经接上线的部分

头部组件在：

- [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx)

这里有两类数据源：

1. `currentExam`，来自 `useExamStore`。
2. `assignedStudent`，来自 `useDeviceStore`。

`currentExam` 的链路是接通的：

1. [apps/student/src/App.tsx](apps/student/src/App.tsx) 启动后定时调用 `refreshCurrentExam()`。
2. [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 通过 `getCurrentExamBundle()` 拉取最新会话。
3. [apps/student/src/services/examRuntimeService.ts](apps/student/src/services/examRuntimeService.ts) 调用 Tauri `get_current_exam_bundle`。
4. [apps/student/src-tauri/src/commands.rs](apps/student/src-tauri/src/commands.rs) 转到 `ExamRuntimeService::get_current_exam_bundle`。
5. [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 从 `exam_sessions` 和 `exam_snapshots` 读取最新记录返回给前端。

因此，头部左侧显示的考试标题与状态，确实是 `exam_sessions` 落库之后可见的前端验证面。

### 头部中当前未接通的部分

头部右侧显示：

`学生: ${assigned.studentNo} ${assigned.name}`

但全局检索结果表明，`useDeviceStore.setAssignedStudent` 只有定义，没有实际业务调用方：

- [apps/student/src/store/deviceStore.ts](apps/student/src/store/deviceStore.ts)
- [apps/student/src/types/main.ts](apps/student/src/types/main.ts)

也就是说，当前代码里“头部右侧的学生信息”并没有通过 `exam_sessions` 或控制协议被真正填充，默认会显示“未分配学生”。

所以，若把“学生端头部内容”当作出口，需要精确写成：

1. 头部左侧考试标题/状态，已经能通过 `exam_sessions -> get_current_exam_bundle -> examStore -> AppHeader` 体现出来。
2. 头部右侧 `assignedStudent` 当前并未接通，不应被当作这条链已完成的出口。

## 最短 e2e 链路图

### A. 随机分配链

1. 教师端 [apps/teacher/src/pages/DeviceAssign/index.tsx](apps/teacher/src/pages/DeviceAssign/index.tsx) 点击“随机分配考生”
2. 教师端 [apps/teacher/src/hooks/useDeviceAssign.ts](apps/teacher/src/hooks/useDeviceAssign.ts) `randomAssign()` 生成随机映射
3. 教师端 [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts) 调用 `assign_devices_to_student_exams`
4. 教师端 [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](apps/teacher/src-tauri/src/controllers/student_exam_controller.rs) 进入命令层
5. 教师端 [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs) 下沉到 repo
6. 教师端 [apps/teacher/src-tauri/src/repos/student_exam_repo.rs](apps/teacher/src-tauri/src/repos/student_exam_repo.rs) 更新 `student_exams.ip_addr`
7. 结束，出口仍在教师端数据库

### B. 基于分配结果的落库链

1. 教师端读取已分配的 `student_exams.ip_addr`
2. 教师端发起 `DISTRIBUTE_EXAM_PAPER`
3. 学生端 [apps/student/src-tauri/src/network/control_server.rs](apps/student/src-tauri/src/network/control_server.rs) 接收请求
4. 学生端 [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) 执行 `upsert_distribution`
5. 学生端 [apps/student/src-tauri/migrations/0002_create_exam_sessions.sql](apps/student/src-tauri/migrations/0002_create_exam_sessions.sql) 对应的 `exam_sessions` 被写入
6. 学生端 [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs) `get_current_exam_bundle`
7. 学生端 [apps/student/src/store/examStore.ts](apps/student/src/store/examStore.ts) 更新 `currentExam/currentSession`
8. 学生端 [apps/student/src/layout/AppHeader.tsx](apps/student/src/layout/AppHeader.tsx) 显示考试标题与状态

## 最终结论

如果严格按代码事实来写：

1. 教师端“随机分配考生”的最短出口是教师端 `student_exams.ip_addr` 更新成功。
2. 学生端 `exam_sessions` 的最短入口不是“随机分配考生”，而是后续“分发试卷”的 `DISTRIBUTE_EXAM_PAPER`。
3. 学生端头部里，真正已接通这条链的是考试标题/状态，不是右侧 `assignedStudent` 文案。

所以这条业务不应被描述成“随机分配考生直接让学生端写入 exam_sessions”，而应描述成：

随机分配先建立教师端的考生-设备映射，后续发卷阶段复用这份映射，把考生与试卷数据送到学生端并落入 `exam_sessions`。