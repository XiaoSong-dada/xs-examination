# 教师端发放试卷到学生端接收试卷包的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端点击“分发试卷”之后，试卷如何以当前真实链路到达学生端，并在学生端形成“试卷包已收到”的事实。

这里刻意不展开“开始考试”“答题同步”“监考状态”等后续链路，只聚焦发卷本身。

## 最短链路结论

1. 教师端前端在考试管理页点击“分发试卷”。
2. 前端 Hook 调用前端 service，发起 Tauri `invoke`：`distribute_exam_papers_by_exam_id`。
3. 教师端 Rust 命令层进入 `student_exam_controller::distribute_exam_papers_by_exam_id`。
4. 命令层调用 `student_exam_service::distribute_exam_papers_by_exam_id`。
5. 服务层读取考试详情、题目列表、当前考试下已分配设备的学生记录。
6. 服务层构建 zip 试卷包，包内至少包含 `question_bank.xlsx` 与 `assets/content`、`assets/options`。
7. 教师端先通过 `ws_server::send_paper_package_manifest_to_student` 按 `student_id` 下发 `PaperPackageManifest`。
8. 教师端继续通过 `ws_server::send_paper_package_chunk_to_student` 按顺序下发 `PaperPackageChunk` 分片。
9. 学生端 `ws_client::handle_server_message` 收到 manifest 后，调用 `ExamRuntimeService::prepare_exam_package_receive`，先写入本地 `exam_sessions` 与 `exam_snapshots.package_*` 元数据。
10. 学生端收齐 chunk、完成 sha256 校验后，调用 `ExamRuntimeService::mark_exam_package_received`，把 `exam_snapshots.package_status` 标记为 `received`。
11. 学生端通过 WebSocket 回传 `PaperPackageAck`。
12. 教师端等待 ACK 并汇总结果；若当前批次全部成功，再把教师端考试状态更新为 `published`。

到第 10 步为止，已经完成“教师端发卷 -> 学生端接收试卷包”的最短业务闭环。

## 关键入口

### 1. 教师端前端入口

入口明确落在教师端考试管理页：

- `apps/teacher/src/pages/ExamManage/index.tsx`
- `handleDistribute`

页面入口负责调用 `distributePapers()`，并根据汇总结果提示成功、部分成功或失败。

### 2. 教师端前端 Hook 与 service

前端页面逻辑继续进入：

- `apps/teacher/src/hooks/useExamManage.ts`
- `apps/teacher/src/services/studentService.ts`

关键调用仍是：

`invoke("distribute_exam_papers_by_exam_id", { payload: { exam_id } })`

到这里为止仍然只是教师端前端到教师端 Rust 的本地 IPC。

### 3. 教师端 Rust 发卷入口

教师端命令入口在：

- `apps/teacher/src-tauri/src/controllers/student_exam_controller.rs`

真实发卷逻辑在：

- `apps/teacher/src-tauri/src/services/student_exam_service.rs`

`distribute_exam_papers_by_exam_id` 的最小职责是：

1. 读取考试、题目与学生分配信息。
2. 调用 `build_exam_package_zip` 生成当前考试的 zip 试卷包。
3. 为每个在线学生生成一组 `batch_id + session_id + manifest/chunk`。
4. 通过教师端 WebSocket 服务下发 manifest 与分片。
5. 等待学生端 `PaperPackageAck` 并汇总结果。

因此，这条链路已经不再是旧的 TCP 控制端口发卷，而是完全切到 WebSocket 试卷包下发。

## 当前实现边界

### 1. 教师端上游数据来源

当前 `question_service::list_questions` 读取的考试题目，仍来自按考试写入 `questions` 表的上游入口，例如纯 xlsx 导入或 `QuestionImport` 页的按考试资源包导入链路。

需要明确一个边界：

当前 `QuestionBank` 页导入资源包的真实出口是 `question_bank_items`，不是 `questions`。因此它不会直接成为这条发卷链路的上游来源，除非后续再有单独的“从全局题库写入考试题目”步骤。

### 2. 学生端接收入口已从 control_server 切到 ws_client

当前学生端接收试卷包的入口在：

- `apps/student/src-tauri/src/network/ws_client.rs`

对应消息类型为：

1. `MessageType::PaperPackageManifest`
2. `MessageType::PaperPackageChunk`

因此，学生端“为什么没收到试卷”这一类问题，不应再默认先查 `control_server.rs` 的 `DISTRIBUTE_EXAM_PAPER` 分支，而应优先核对 WebSocket 收包、batch 状态与 `exam_snapshots.package_*` 字段。

### 3. 当前发卷的真实出口不是 questions_payload 已落库

当前学生端发卷阶段的真实出口在：

1. `ExamRuntimeService::prepare_exam_package_receive`
2. `ExamRuntimeService::mark_exam_package_received`

发卷阶段会落下这些事实：

1. `exam_sessions` 已存在并绑定当前 `session_id/exam_id/student_id`。
2. `exam_snapshots.package_path` 已写入本地 zip 路径。
3. `exam_snapshots.package_status` 已从 `receiving` 进入 `received`。
4. `exam_snapshots.package_sha256/package_batch_id/package_received_at` 已落库。

当前 `questions_payload` 的生成已延后到“开始考试 -> 本地物化试卷包”阶段，而不在发卷阶段完成。

### 4. 页面验证面的含义也已经变化

当前页面可见验证面依然是学生端前端轮询 `get_current_exam_bundle` 后读到本地 `session + snapshot`，但这里的“已收到试卷”不再表示 `questions_payload` 已准备好，只表示试卷包已经完整接收并通过本地校验，等待开始考试时再做物化。

## 最小验收步骤

1. 教师端考试管理页点击“分发试卷”。
2. 教师端日志看到当前批次的 manifest 与全部 chunk 已发送。
3. 学生端日志看到收到 `PaperPackageManifest` 与全部 `PaperPackageChunk`。
4. 学生端本地 `exam_snapshots` 中当前会话的 `package_status = received`，且 `package_path/package_sha256/package_received_at` 非空。
5. 教师端收到对应批次的 `PaperPackageAck`，汇总结果成功或部分成功。
6. 学生端前端显示“试卷已下发，请等待教师开始考试指令”。

## 相关阅读

1. [项目依赖拓扑图](../project_dependency_topology.md)
2. [教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路](./e2e-minimal-answer-sync-progress-chain.md)
3. [教师端按考试导入题目资源包的最短 e2e 链路](./e2e-minimal-question-import-package-chain.md)