# 教师端结束考试并触发学生端最终同步的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端点击“结束考试”之后，如何让在线学生先完成最后一轮答案同步，再把学生端本地考试状态切到已结束，并让教师端在考试结束后拒收任何新答案。

这里刻意不展开：

1. 连接考生设备
2. 分发试卷
3. 开始考试
4. 教师端异常重启后的全量自愈
5. 评分、交卷与成绩统计

只聚焦这次新增的最短闭环：

1. 教师端点击“结束考试”
2. 教师端向在线学生发送最终同步请求
3. 教师端继续向同一批学生发送结束考试指令
4. 学生端发送 final `ANSWER_SYNC`
5. 学生端把本地 `exam_sessions.status` 更新为 `ended`
6. 教师端在收到所有在线学生 final ACK 后把考试状态更新为 `finished`
7. 教师端对 `finished` 之后的新 `ANSWER_SYNC` 一律拒收

## 最短链路结论

最短链路如下：

1. 教师端考试管理页点击“结束考试”。
2. 教师端前端调用 `end_exam_by_exam_id`。
3. 教师端 Rust `student_exam_service::end_exam_by_exam_id` 先筛选当前考试下“设备状态为正常”的在线学生。
4. 教师端通过 `ws_server::send_final_sync_request_to_student` 为每个在线学生发送 `FINAL_SYNC_REQUEST`，并为这次结束动作生成 `request_id + batch_id`。
5. 教师端继续通过 `ws_server::send_exam_end_to_student` 向同一批学生发送 `EXAM_END`。
6. 学生端 `ws_client::handle_server_message` 收到 `FINAL_SYNC_REQUEST` 后，调用 `ExamRuntimeService::send_current_session_answer_sync(sync_mode=final)` 发送一轮 final `ANSWER_SYNC`。
7. 学生端收到 `EXAM_END` 后，再补发一轮带相同 `finalBatchId` 的 final `ANSWER_SYNC`，然后调用 `ExamRuntimeService::mark_exam_ended`，把本地 `exam_sessions.status` 更新为 `ended`。
8. 教师端 `ws_server::handle_text_message` 收到 `ANSWER_SYNC(syncMode=final)` 后，仍走 `persist_answer_sync` 的幂等落库与进度聚合逻辑；若落库成功，则把该 `batch_id` 记为已完成。
9. 教师端 `student_exam_service::end_exam_by_exam_id` 等待本次在线目标学生的 final `batch_id` 全部确认成功；全部完成后再把 `exams.status` 更新为 `finished`。
10. 一旦 `exams.status = finished`，教师端 `persist_answer_sync` 会直接返回失败结果，不再写 `answer_sheets` 和 `student_exam_progress`。

到第 7 步为止，已经完成“教师端结束考试 -> 学生端本地考试结束”的最短学生端闭环。

到第 9 步为止，已经完成“教师端等待 final ACK 收敛后结束考试”的最短教师端闭环。

到第 10 步为止，已经完成“考试结束后教师端拒收答案”的最小状态门禁闭环。

## 关键入口

### 教师端前端入口

- `apps/teacher/src/pages/ExamManage/index.tsx`
- `apps/teacher/src/hooks/useExamManage.ts`
- `apps/teacher/src/services/studentService.ts`

页面工具栏新增“结束考试”按钮后，前端链路为：

`ExamManagePage.handleEndExam -> useExamManage.endExam -> studentService.endExamByExamId -> invoke(end_exam_by_exam_id)`

### 教师端 Rust 编排入口

- `apps/teacher/src-tauri/src/controllers/student_exam_controller.rs::end_exam_by_exam_id`
- `apps/teacher/src-tauri/src/services/student_exam_service.rs::end_exam_by_exam_id`
- `apps/teacher/src-tauri/src/network/ws_server.rs::send_final_sync_request_to_student`
- `apps/teacher/src-tauri/src/network/ws_server.rs::send_exam_end_to_student`

这里的真实出口不是“按钮点击成功”，而是：

1. 在线学生的 final `batch_id` 全部被教师端确认
2. `exam_service::update_exam_status(..., "finished")` 写回教师端数据库成功

### 学生端消息接收入口

- `apps/student/src-tauri/src/network/ws_client.rs::handle_server_message`
- `apps/student/src-tauri/src/services/exam_runtime_service.rs::send_current_session_answer_sync`
- `apps/student/src-tauri/src/services/exam_runtime_service.rs::mark_exam_ended`

学生端真实出口不是页面提示，而是：

1. final `ANSWER_SYNC` 已成功发出
2. `exam_sessions.status` 已切换为 `ended`

### 教师端答案拒收入口

- `apps/teacher/src-tauri/src/network/ws_server.rs::persist_answer_sync`

这条门禁在答案持久化入口生效：

1. 若 `exams.status != finished`，继续按原链路落库并返回 `ANSWER_SYNC_ACK`
2. 若 `exams.status = finished`，直接返回失败结果，不再写 `answer_sheets`

## 当前实现边界

1. 结束考试仅面向当前“在线且设备状态正常”的学生等待 final ACK；不在线学生不作为本轮阻塞条件。
2. 教师端是否更新为 `finished`，以本次结束请求里的在线目标集合为准，而不是以全量 `student_exams` 为准。
3. final `ANSWER_SYNC` 仍复用既有 `answer_sheets` 幂等落库与 `student_exam_progress` 聚合逻辑，不单独新增一套表。
4. 学生端一旦本地状态变为 `ended`，后续 `send_answer_sync` 与 `flush_pending_answer_sync` 都会停止继续产生新的常规答案同步。
5. 教师端对 `finished` 后的 `ANSWER_SYNC` 统一拒收，当前口径不区分“延迟重传”“重复 ACK 修正”或“补写最后一次 revision”。
6. 这份链路文档只覆盖“结束考试”与“最终同步”的闭环，不覆盖倒计时自动结束、强制交卷、评分与报表统计；`finished` 之后进入 Report 页的“统计成绩 -> 导出成绩”闭环，详见独立的成绩报告 e2e 文档。

## 最小验收步骤

1. 正常完成连接、发卷与开始考试，让至少 1 台学生端进入答题态。
2. 学生端作答 1-2 题，确认教师端监考页已有进度。
3. 在教师端考试管理页点击“结束考试”。
4. 观察教师端日志：应先发送 `FINAL_SYNC_REQUEST`，再发送 `EXAM_END`。
5. 观察学生端日志：应收到两类消息，并至少发送一轮 `syncMode=final` 的 `ANSWER_SYNC`。
6. 观察学生端本地状态：`exam_sessions.status` 应更新为 `ended`。
7. 观察教师端结果：当在线学生 final ACK 全部到齐后，`exams.status` 应更新为 `finished`。
8. 在 `finished` 之后，再人为触发一次学生端答案同步，确认教师端返回失败 ACK，且 `answer_sheets` 与 `student_exam_progress` 不再继续变化。

## 不属于这条最短链路的内容

1. 连接考生设备
2. 分发试卷
3. 开始考试
4. 教师端异常重启后的 full `ANSWER_SYNC` 自愈
5. 强制交卷、评分、成绩汇总
6. 离线学生后补同步策略

## 相关阅读

1. [教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路](./e2e-minimal-answer-sync-progress-chain.md)
2. [教师端异常恢复后学生端全量答案同步与 ACK 收敛最短 e2e 链路](./e2e-minimal-answer-sync-ack-reconnect-chain.md)
3. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)
4. [项目依赖拓扑图](../project_dependency_topology.md)
5. [教师端成绩报告统计与导出的最短 e2e 链路](./e2e-minimal-score-report-chain.md)
