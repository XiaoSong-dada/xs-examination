# 教师端发放试卷到学生端接收试卷的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端点击“分发试卷”之后，试卷如何以最短链路到达学生端，并在学生端形成“已收到试卷”的事实。

这里刻意不展开“开始考试”“答题同步”“监考状态”等后续链路，只聚焦发卷本身。

## 校验结论

你的理解有一半是对的：

1. 项目入口确实是教师端的“分发试卷”功能。
2. 但项目出口如果写成“学生端的接收试卷功能”，还需要再精确定义。

更准确的出口分两层：

1. 业务真实出口：学生端控制服务收到 `DISTRIBUTE_EXAM_PAPER` 请求后，调用 `ExamRuntimeService::upsert_distribution`，把试卷会话和试卷快照落到本地数据库成功。
2. 页面可见出口：学生端前端轮询 `get_current_exam_bundle`，读到刚刚落库的 session 和 snapshot，然后页面从“尚未收到教师下发的考试试卷”切换为“试卷已下发，请等待教师开始考试指令”。

如果要写“最短 e2e 链路”，建议把出口定义为第 1 层，也就是学生端落库成功，而不是前端页面展示。

原因很简单：页面展示只是落库之后的一个验证面，不是网络链路真正结束的地方。

## 最短链路结论

最短链路如下：

1. 教师端前端在考试管理页点击“分发试卷”。
2. 前端 Hook 调用前端 service，发起 Tauri `invoke`：`distribute_exam_papers_by_exam_id`。
3. 教师端 Rust 命令层进入 `student_exam_controller::distribute_exam_papers_by_exam_id`。
4. 命令层调用 `student_exam_service::distribute_exam_papers_by_exam_id`。
5. 服务层读取考试详情、题目列表、当前考试下已分配设备的学生记录。
6. 服务层为每个已分配 IP 的学生构造一条 `DISTRIBUTE_EXAM_PAPER` TCP 请求。
7. 教师端通过 `student_control_client::distribute_exam_paper` 直连学生端 `ip:38888`。
8. 学生端 `control_server::handle_client` 收到 `DISTRIBUTE_EXAM_PAPER` 请求。
9. 学生端调用 `ExamRuntimeService::upsert_distribution`，把 `exam_sessions` 和 `exam_snapshots` 写入本地数据库。
10. 学生端返回 `DISTRIBUTE_EXAM_PAPER_ACK` 给教师端。
11. 教师端汇总 ACK 结果，如果全部成功，再把教师端本地考试状态更新为 `published`。

到第 9 步为止，已经完成“教师端发卷 -> 学生端接收试卷”的最短业务闭环。

## 入口到出口的精简调用链

### 1. 教师端前端入口

入口不是一个模糊概念，而是明确落在教师端考试管理页：

- `apps/teacher/src/pages/ExamManage/index.tsx`
- `handleDistribute`

这个入口只做两件事：

1. 调用 `distributePapers()`。
2. 根据返回结果给出成功、部分成功或失败提示。

真正的业务跳转发生在 Hook 里。

### 2. 教师端前端 Hook

教师端前端的页面逻辑继续进入：

- `apps/teacher/src/hooks/useExamManage.ts`

其中 `distributePapers` 的行为是：

1. 取当前选中的 `selectedExamId`。
2. 调用 `distributeExamPapersByExamId(selectedExamId)`。
3. 分发结束后刷新考试列表和考试状态。

### 3. 教师端前端 service -> Tauri IPC

前端 service 在：

- `apps/teacher/src/services/studentService.ts`

关键调用是：

`invoke("distribute_exam_papers_by_exam_id", { payload: { exam_id } })`

到这里为止，仍然只是教师端前端到教师端 Rust 的本地 IPC，不涉及学生端。

### 4. 教师端 Rust 命令入口

教师端 Rust 命令入口在：

- `apps/teacher/src-tauri/src/controllers/student_exam_controller.rs`

函数：

- `distribute_exam_papers_by_exam_id`

它做两件事：

1. 调用 `student_exam_service::distribute_exam_papers_by_exam_id(pool, exam_id)` 执行真实发卷。
2. 如果所有目标都成功，则调用 `exam_service::update_exam_status(..., "published")`，把教师端本地考试状态更新为已发卷。

注意：

教师端考试状态更新不是学生端接收试卷的组成部分，它是教师端的后置状态维护。

## 教师端真实发卷点

真正的发卷逻辑在：

- `apps/teacher/src-tauri/src/services/student_exam_service.rs`

函数：

- `distribute_exam_papers_by_exam_id`

这里有一个需要额外明确的上游边界：

当前 `question_service::list_questions` 读取的考试题目，主要来自按考试写入 `questions` 表的上游入口，例如旧的纯 xlsx 导入或 `QuestionImport` 页的按考试导入链路；2026-04-01 起，这条按考试导入链路已支持把资源包中的题干图、选项图映射为 `questions.content_image_paths` 与 `options.image_paths`。

需要明确一个边界：

当前 `QuestionBank` 页新增的“资源包导入”真实出口是 `question_bank_items`，不是 `questions`。因此它不会直接成为这条发卷链路的上游数据来源，除非后续再有单独的“从全局题库选题写入考试题目”步骤。

所以，对这条发卷最短链路来说，真正的起点仍然是“教师端已经能从 `questions` 表读到该考试题目”，而不是题目最初是怎么被准备出来的。

这个函数的最小职责链如下：

1. `exam_service::get_exam_by_id` 读取考试信息。
2. `question_service::list_questions` 读取该考试题目。
3. `student_exam_repo::get_student_device_assignments_by_exam_id` 读取该考试下的学生-设备分配。
4. 仅保留 `ip_addr` 非空且非空串的记录。
5. 把考试元信息序列化成 `exam_meta`。
6. 把题目列表序列化成 `questions_payload`。
7. 针对每个学生设备构造 `DistributeExamPaperRequest`。
8. 通过 `student_control_client::distribute_exam_paper(device_ip, 38888, &req)` 发送到学生端。
9. 读取学生端 ACK，并汇总为 `DistributeExamPapersOutput`。

因此，这条链路的网络事实是：

教师端不是广播试卷，而是按已分配学生设备 IP 逐台 TCP 直连学生端控制端口发卷。

## 教师端发给学生端的关键载荷

教师端发出的 `DistributeExamPaperPayload` 里，最关键的字段是：

1. `session_id`
2. `exam_id`
3. `student_id`
4. `student_no`
5. `student_name`
6. `assigned_ip_addr`
7. `exam_title`
8. `status`
9. `assignment_status`
10. `paper_version`
11. `exam_meta`
12. `questions_payload`
13. `downloaded_at`
14. `expires_at`

其中最重要的两个大字段是：

1. `exam_meta`：考试元信息 JSON 字符串。
2. `questions_payload`：题目列表 JSON 字符串。

2026-04-01 起还需要额外注意一个字段事实：

1. 教师端发卷时已开始把 `questions.content_image_paths` 透传为 `questions_payload[*].contentImagePaths`。
2. 当前选项图片仍保留在 `options` JSON 结构内，尚未在这条 e2e 文档中展开最终学生端渲染格式。

这意味着学生端“接收试卷”并不是再回源查询教师端数据库，而是直接接收教师端已经打包好的试卷内容并本地落库。

## 学生端接收入口

学生端真正收到试卷的入口在：

- `apps/student/src-tauri/src/network/control_server.rs`

函数：

- `handle_client`

这个函数先读取 TCP 请求，再根据 `type` 分流。

当 `req_type == "DISTRIBUTE_EXAM_PAPER"` 时：

1. 把 JSON 反序列化为 `DistributeExamPaperRequest`。
2. 调用 `ExamRuntimeService::upsert_distribution(&app_handle, &req.payload)`。
3. 根据执行结果构造 `DISTRIBUTE_EXAM_PAPER_ACK`。
4. 把 ACK 写回教师端。

因此，学生端的“接收试卷功能”如果只从入口来描述，应该写成：

学生端 `control_server::handle_client` 对 `DISTRIBUTE_EXAM_PAPER` 请求的处理分支。

## 学生端真正出口

学生端真正的业务出口在：

- `apps/student/src-tauri/src/services/exam_runtime_service.rs`

函数：

- `ExamRuntimeService::upsert_distribution`

这个函数做了两次 upsert：

### 1. upsert `exam_sessions`

它会按 `session_id` 查找是否已有记录：

1. 有记录则更新。
2. 无记录则插入。

写入的核心内容包括：

1. `exam_id`
2. `student_id`
3. `student_no`
4. `student_name`
5. `assigned_ip_addr`
6. `exam_title`
7. `status = "waiting"`
8. `assignment_status`
9. `ends_at`
10. `paper_version`

### 2. upsert `exam_snapshots`

它同样按 `session_id` 查找是否已有快照：

1. 有记录则更新。
2. 无记录则插入。

写入的核心内容包括：

1. `exam_meta`
2. `questions_payload`
3. `downloaded_at`
4. `expires_at`
5. `assets_sync_status = pending`
6. `assets_synced_at = null`

所以最准确的“项目出口”写法应当是：

学生端 `ExamRuntimeService::upsert_distribution` 成功把试卷会话和试卷快照写入本地数据库，并把图片资源同步状态初始化为待完成。

## 学生端页面为什么会显示“已收到试卷”

这不是发卷链路本身的出口，而是出口之后的验证链。

验证链如下：

1. 学生端前端 `App.tsx` 启动后定时调用 `refreshCurrentExam()`。
2. `refreshCurrentExam()` 位于 `apps/student/src/store/examStore.ts`。
3. 该方法调用 `getCurrentExamBundle()`。
4. `getCurrentExamBundle()` 位于 `apps/student/src/services/examRuntimeService.ts`，通过 Tauri invoke 调用 `get_current_exam_bundle`。
5. 学生端 Rust 命令 `commands::get_current_exam_bundle` 再调用 `ExamRuntimeService::get_current_exam_bundle`。
6. 该服务从本地 `exam_sessions` 和 `exam_snapshots` 读取最新一套试卷数据返回给前端。
7. 当前端读到 `session` 但考试尚未 `active` 时，页面显示“试卷已下发，请等待教师开始考试指令”。

所以页面文案变化依赖的是“本地库里已经有数据”，而不是“教师端当前又发来了一次实时事件”。

## 不属于这条最短链路的内容

以下内容和发卷相关，但不属于“教师端发卷 -> 学生端接收试卷”的最短链路：

1. 教师端“开始考试”按钮。
2. 教师端 WebSocket 服务推送 `EXAM_START`。
3. 学生端把 `exam_sessions.status` 从 `waiting` 改成 `active`。
4. 学生端答题同步。
5. 教师端监考页状态聚合。
6. 教师端题目列表勾选导出资源包，以及 `QuestionBank` 页对 zip 资源包的清空后回导入。

这些都是发卷之后的下一段链路，不应和“接收试卷”混写在同一条最短路径里。

## 一句话版本

如果只保留一句话，这条最短 e2e 链路可以写成：

教师端 `ExamManage -> useExamManage -> studentService -> student_exam_controller -> student_exam_service -> student_control_client` 把 `questions` 表中的考试题目打包成 `DISTRIBUTE_EXAM_PAPER` 下发到学生端，学生端 `control_server -> ExamRuntimeService::upsert_distribution` 成功把试卷会话与快照落入本地数据库。

## 相关阅读

1. [项目依赖拓扑图](../project_dependency_topology.md)
2. [教师端题目列表导出资源包并回导入全局题库的最短 e2e 链路](./e2e-minimal-question-bank-package-chain.md)
3. [教师端按考试导入题目资源包的最短 e2e 链路](./e2e-minimal-question-import-package-chain.md)

教师端考试管理页点击“分发试卷”后，经前端 invoke 调用教师端 Rust `distribute_exam_papers_by_exam_id`，教师端按已分配学生设备 IP 逐台 TCP 发送 `DISTRIBUTE_EXAM_PAPER` 到学生端控制端口，学生端 `control_server` 收到请求后调用 `ExamRuntimeService::upsert_distribution` 把 `exam_sessions` 和 `exam_snapshots` 落库成功，这就是“学生端已接收试卷”的真正出口。