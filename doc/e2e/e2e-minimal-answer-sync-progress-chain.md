# 教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端点击“开始考试”之后，学生端如何进入答题态、按题保存并同步最新答案，以及教师端如何把这些答案落库并更新监考/报告页的真实答题进度。

这里刻意不展开：

1. 连接考生设备
2. 分发试卷
3. 断网补发完整重试机制
4. 历史答案轨迹回放
5. 结束考试后的 final `ANSWER_SYNC` 与教师端拒收门禁

只聚焦这次更新真正形成的最短闭环：

1. 开考指令下发
2. 学生端在开考时物化已收到的试卷包
3. 学生端按题保存最新答案
4. 学生端即时发送 `ANSWER_SYNC`
5. 教师端持久化最新答案
6. 教师端聚合并展示真实进度

## 最短链路结论

最短链路如下：

1. 教师端考试管理页点击“开始考试”。
2. 教师端前端调用 `start_exam_by_exam_id`。
3. 教师端 Rust `student_exam_service::start_exam_by_exam_id` 为每个已连接学生构造 `EXAM_START`，并通过 `ws_server::send_exam_start_to_student` 下发。
4. 学生端 `ws_client::handle_server_message` 收到 `EXAM_START` 后，调用 `ExamRuntimeService::mark_exam_started`。
5. `mark_exam_started` 会先选中当前考试对应且已存在 snapshot 的本地会话，再调用 `materialize_exam_package_if_needed`。
6. `materialize_exam_package_if_needed` 会校验本地 zip 包、解压 `question_bank.xlsx + assets/*`、复制图片资源、重写 `questions_payload`，并把 `exam_snapshots.package_status` 更新为 `extracted`。
7. 只有当试卷包物化完成且 `questions_payload` 已准备好后，学生端才会把本地 `exam_sessions.status` 更新为 `active`。
8. 学生端答题页选择某个选项时，`pages/Exam/index.tsx` 调用前端 service `sendAnswerSync`。
9. 学生端 Rust `commands.rs::send_answer_sync` 先按题 upsert 本地 `local_answers`，再写入一条 `sync_outbox` 记录。
10. 若当前 WebSocket 发送通道可用，学生端通过 `network/ws_client.rs::build_answer_sync_message` 构造 `ANSWER_SYNC` 并立即发往教师端。
11. 教师端 `network/ws_server.rs` 收到 `ANSWER_SYNC` 后，调用 `persist_answer_sync`，把最新答案 upsert 到 `answer_sheets`，并返回 `ANSWER_SYNC_ACK`。
12. 学生端收到 ACK 后才会回写本地同步状态：对成功题目标记 `synced`，对失败题目标记 `failed`，等待后续补发或重连自愈。
13. 同一个教师端处理链里，`persist_answer_sync` 会基于 `answer_sheets` 重新计算该 `student_exam` 的 `answered_count / total_questions / progress_percent`，再 upsert 到 `student_exam_progress`。
14. 教师端前端 `useMonitor.ts` 与 `useReport.ts` 继续通过 `get_student_device_connection_status_by_exam_id` 读取真实 `progress_percent`，分别渲染监考页和报告页；其中 Report 页还会额外通过 `get_student_score_summary_by_exam_id` 读取成绩总分。
15. 若教师端后续触发“结束考试”，则答案同步链会切换到 final `ANSWER_SYNC` 与 `finished` 后拒收答案的口径，详见独立的结束考试 e2e 文档。

到第 7 步为止，已经完成“开始考试 -> 学生端具备可答题的本地试卷数据”的最短开考闭环。

到第 9 步为止，已经完成“学生端每题最新答案先落本地”的最短本地闭环。

到第 12 步为止，已经完成“学生端作答 -> 教师端最新答案落库 -> ACK 回写本地状态”的最短业务闭环。

到第 13 步为止，已经完成“教师端真实进度可见”的最短页面闭环。

## 入口到出口的精简调用链

### 1. 教师端前端开考入口

入口在：

- [apps/teacher/src/pages/ExamManage/index.tsx](apps/teacher/src/pages/ExamManage/index.tsx)

页面点击“开始考试”后，会调用 Hook 中的 `startExam()`。

### 2. 教师端前端 Hook

Hook 在：

- [apps/teacher/src/hooks/useExamManage.ts](apps/teacher/src/hooks/useExamManage.ts)

`startExam()` 的职责是：

1. 读取当前 `selectedExamId`
2. 调用 `startExamByExamId(selectedExamId)`
3. 刷新考试状态与学生表格

### 3. 教师端前端 Service -> Tauri IPC

前端 service 在：

- [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts)

它通过 Tauri invoke 调用：

`start_exam_by_exam_id`

到这里为止，仍然只是教师端前端到教师端 Rust 的本地 IPC。

### 4. 教师端 Rust 开考入口

教师端真实开考链在：

- [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs)
- [apps/teacher/src-tauri/src/network/ws_server.rs](apps/teacher/src-tauri/src/network/ws_server.rs)

`start_exam_by_exam_id` 的最小职责是：

1. 读取当前考试与已分配设备的学生记录
2. 为每个有效目标构造 `ExamStartPayload`
3. 调用 `ws_server::send_exam_start_to_student`，按 `student_id` 把 `EXAM_START` 发给已建立 WebSocket 的学生端

因此，这一段不是 TCP 控制端口链路，而是已经切换到教师端 WebSocket 服务。

## 学生端进入答题态的入口

学生端收到开考指令的入口在：

- [apps/student/src-tauri/src/network/ws_client.rs](apps/student/src-tauri/src/network/ws_client.rs)

`handle_server_message` 在收到 `MessageType::ExamStart` 后会：

1. 校验消息里的 `student_id` 是否等于当前连接学生
2. 调用 `ExamRuntimeService::mark_exam_started`
3. 在更新成功后发出 `exam_status_changed` 事件

真实状态更新出口在：

- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

当前 `mark_exam_started` 的关键变化不是“直接 active”，而是：

1. 优先选择当前考试下已经存在快照的本地会话。
2. 若 `exam_snapshots.package_status = receiving`，则拒绝进入考试。
3. 若 `exam_snapshots.package_status = received`，则先调用 `materialize_exam_package_if_needed`。
4. 物化阶段会校验 zip、解压 xlsx 与图片资源、写回 `questions_payload`，并把 `package_status` 更新为 `extracted`。
5. 只有试卷数据准备完成后，才会把 `exam_sessions.status` 更新为 `active`，并写入开考时间与结束时间。

因此，“学生端进入答题态”的真实出口不是前端按钮变色，而是本地 `exam_sessions` 已被标记为 `active`。

同时也要注意一个新的失败边界：

若试卷包尚未接收完成、sha256 校验失败、zip 解压失败或 xlsx 解析失败，`EXAM_START` 会在学生端被门禁拦截，不会进入 `active`。

## 学生端每题作答的本地闭环

### 1. 学生端前端入口

答题入口在：

- [apps/student/src/pages/Exam/index.tsx](apps/student/src/pages/Exam/index.tsx)

`handleSelectAnswer` 的当前行为是：

1. 先把当前题选项写入组件本地 `selectedAnswers`
2. 从题目选项里推导出要同步的答案值，优先使用 `option.key`
3. 调用 `sendAnswerSync(examId, studentId, questionId, answerValue)`

### 2. 学生端前端 Service -> Tauri IPC

前端 service 在：

- [apps/student/src/services/examRuntimeService.ts](apps/student/src/services/examRuntimeService.ts)

它通过 Tauri invoke 调用：

`send_answer_sync`

### 3. 学生端 Rust 命令入口

学生端 Rust 命令入口在：

- [apps/student/src-tauri/src/commands.rs](apps/student/src-tauri/src/commands.rs)

`send_answer_sync` 的最小职责链如下：

1. 按 `exam_id + student_id` 找到最近一条本地 `exam_sessions`
2. 按 `session_id + question_id` 查找本题是否已有本地答案
3. 计算新的 `revision`
4. upsert 本地 `local_answers`
5. 写入一条 `sync_outbox` 记录，事件类型为 `ANSWER_SYNC`
6. 若当前没有 WebSocket 发送通道，则直接返回“已保存到本地，等待连接恢复后同步”
7. 若发送通道存在，则构造并发送 `ANSWER_SYNC`
8. 发送成功后仅把 `sync_outbox` 更新为 `sent`
9. 收到教师端 `ANSWER_SYNC_ACK` 后，按 `questionIds/failedQuestionIds` 分题回写本地状态
10. 成功题目标记为 `synced`，失败题目标记为 `failed`，等待后续 flush 或重连后一轮 full 同步

因此，学生端这次更新的真实出口首先是：

本地 `local_answers` 与 `sync_outbox` 已经开始真正承接按题答案同步，而不再只是预留表结构。

## 学生端发给教师端的消息

学生端构造消息的位置在：

- [apps/student/src-tauri/src/network/ws_client.rs](apps/student/src-tauri/src/network/ws_client.rs)

`build_answer_sync_message` 当前会生成：

1. `examId`
2. `studentId`
3. `answers[]`
4. 每个答案项里的 `questionId`
5. `answer`
6. `revision`
7. `answerUpdatedAt`

这意味着当前消息模型已经从“只有题号和答案”扩展成“最新答案 + 版本号 + 学生端更新时间”的形式，用于教师端覆盖保护与聚合更新。

## 教师端接收与聚合的真实出口

教师端 WebSocket 接收入口在：

- [apps/teacher/src-tauri/src/network/ws_server.rs](apps/teacher/src-tauri/src/network/ws_server.rs)

`handle_text_message` 在收到 `MessageType::AnswerSync` 后会：

1. 解析 `AnswerSyncPayload`
2. 刷新该学生的心跳在线态
3. 调用 `persist_answer_sync(app_handle, &payload, envelope.timestamp)`

`persist_answer_sync` 的最小职责链如下：

1. 按 `exam_id + student_id` 找到对应的 `student_exam_id`
2. 按当前考试统计 `total_questions`
3. 把每条最新答案 upsert 到 `answer_sheets`
4. 基于 `answer_sheets` 重新统计当前 `student_exam_id` 的 `answered_count`
5. 计算 `progress_percent`
6. 把聚合结果 upsert 到 `student_exam_progress`

所以教师端这条链路的两个真实出口分别是：

1. `answer_sheets` 保存“每题最新答案事实”
2. `student_exam_progress` 保存“监考与报告页读取的聚合进度事实”

## 教师端页面为什么能看到真实进度

教师端前端当前不是直接去读 `answer_sheets`。

它走的是统一状态查询链：

1. [apps/teacher/src/hooks/useMonitor.ts](apps/teacher/src/hooks/useMonitor.ts) 通过 `getStudentDeviceConnectionStatusByExamId` 拉取监考表格数据
2. [apps/teacher/src/hooks/useReport.ts](apps/teacher/src/hooks/useReport.ts) 通过同一个 service 拉取报告页的进度数据，并通过额外的成绩汇总命令读取 `score_summary`
3. [apps/teacher/src/services/studentService.ts](apps/teacher/src/services/studentService.ts) 调用 `get_student_device_connection_status_by_exam_id`
4. [apps/teacher/src-tauri/src/services/student_exam_service.rs](apps/teacher/src-tauri/src/services/student_exam_service.rs) 把连接状态与进度聚合结果合并成 `StudentDeviceConnectionStatusDto`
5. 前端最终使用返回值里的 `progress_percent`

因此，这次更新后的页面验证面是：

1. Monitor 页的 `answerProgress`
2. Report 页的 `answerProgress`
3. Report 页在统计成绩完成后的 `score`

它们现在都不再是占位 0，而是教师端真实聚合结果。

## 学生端重启后为什么还能恢复已答状态

这不属于教师端进度聚合的出口，但属于这次答案同步更新的补充恢复链。

入口在：

- [apps/student/src/pages/Exam/index.tsx](apps/student/src/pages/Exam/index.tsx)
- [apps/student/src/services/examRuntimeService.ts](apps/student/src/services/examRuntimeService.ts)
- [apps/student/src-tauri/src/commands.rs](apps/student/src-tauri/src/commands.rs)
- [apps/student/src-tauri/src/services/exam_runtime_service.rs](apps/student/src-tauri/src/services/exam_runtime_service.rs)

链路如下：

1. 考试页面加载后调用 `getCurrentSessionAnswers()`
2. Tauri `get_current_session_answers` 读取最近一条 `exam_sessions` 对应的 `local_answers`
3. 前端把 `question_id + answer` 映射回当前题目的选项下标
4. 页面恢复 `selectedAnswers`

所以学生端重启后看到之前已答选项，依赖的是本地 `local_answers` 回填，而不是教师端反向下发答案。

## 不属于这条最短链路的内容

以下内容与本业务相关，但不属于“开始考试 -> 按题同步 -> 教师端更新进度”的最短链路：

1. 连接考生设备
2. 发卷时的 `PaperPackageManifest/PaperPackageChunk/PaperPackageAck` 收包细节
3. 重连后一轮 full 同步与 `sync_outbox` 的完整自动补发重试循环
4. 历史答案版本回放
5. 自动交卷、强制交卷、暂停考试
6. 结束考试触发的 final `ANSWER_SYNC`、学生端 `exam_sessions.status=ended` 与教师端 `finished` 后拒收答案

这些要么属于前置链路，要么属于下一阶段增强。

## 一句话总结

教师端开始考试后，会经 `ExamManage/useExamManage/studentService -> start_exam_by_exam_id -> student_exam_service -> ws_server` 把 `EXAM_START` 下发给学生端；学生端 `ws_client` 收到后并不会直接进入答题态，而是先在 `ExamRuntimeService::mark_exam_started` 中校验并物化此前已收到的试卷包，成功后才把本地 `exam_sessions` 标记为 `active`；随后答题页在每次选择答案时通过 `send_answer_sync` 先把最新答案落到本地 `local_answers` 与 `sync_outbox`，再通过 WebSocket 发送 `ANSWER_SYNC`；教师端 `ws_server` 收到后把最新答案 upsert 到 `answer_sheets`，返回分题结果的 `ANSWER_SYNC_ACK`，并把进度聚合 upsert 到 `student_exam_progress`，最终由 Monitor 与 Report 统一读取真实 `progress_percent` 展示。若连接在此过程中中断，则由重连链触发 full `ANSWER_SYNC` 与 `pending/failed` flush 完成后续自愈，详见 `e2e-minimal-answer-sync-ack-reconnect-chain.md`。

## 相关阅读

1. [重连后学生答案全量同步与 ACK 收敛计划](../plans/2026_03_23_重连后学生答案全量同步与ACK收敛计划.md)
2. [教师端结束考试并触发学生端最终同步的最短 e2e 链路](./e2e-minimal-end-exam-final-sync-chain.md)
3. [学生端启动恢复与断线重连的最短 e2e 链路](./e2e-minimal-student-startup-reconnect-chain.md)
4. [教师端异常恢复后学生端全量答案同步与 ACK 收敛最短 e2e 链路](./e2e-minimal-answer-sync-ack-reconnect-chain.md)
5. [教师端成绩报告统计与导出的最短 e2e 链路](./e2e-minimal-score-report-chain.md)