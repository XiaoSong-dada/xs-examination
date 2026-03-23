# 教师端异常恢复后学生端全量答案同步与 ACK 收敛最短 e2e 链路

## 目标

这份文档聚焦一个问题：

教师端异常关闭期间，学生端继续作答后，如何在连接恢复后自动追平答案，并仅在教师端 ACK 成功后收敛本地同步状态。

## 最短链路结论

1. 学生端作答时，`send_answer_sync` 先落本地 `local_answers`，并写 `sync_outbox`（`pending`）。
2. 若 WebSocket 可用，学生端发送 `ANSWER_SYNC(syncMode=incremental)`，将该 outbox 状态更新为 `sent`，不直接标记为 `synced`。
3. 教师端 `ws_server` 收到 `ANSWER_SYNC` 后按题幂等 upsert 到 `answer_sheets`，并重算 `student_exam_progress`。
4. 教师端落库后返回 `ANSWER_SYNC_ACK`，同时给出细粒度结果：`questionIds`（成功题目）、`failedQuestionIds`（失败题目）、`successCount`、`failedCount`。
5. 学生端收到 ACK：
   - 对 `questionIds` 调用 `mark_answers_synced`，将对应本地状态标记为 `synced`。
   - 对 `failedQuestionIds` 调用 `mark_answers_failed`，将对应 outbox 标记为 `failed`，并保留后续重试机会。
6. 学生端每次重连成功后会自动触发一轮 `syncMode=full` 全量答案同步（数据源为当前会话 `local_answers`）。
7. 学生端连接期间后台会持续冲刷 `sync_outbox(pending/failed)`，发送增量补发消息。
8. 若同一教师 endpoint 下切换了 `student_id`，学生端会先强制断开旧连接再重连，避免把旧身份的心跳与 ACK 混到新会话里。
9. full `ANSWER_SYNC` 发送带冷却保护，避免一次连接抖动期间重复触发多轮全量同步。

## 关键入口

### 学生端发送与状态收敛

- 增量发送入口：`apps/student/src-tauri/src/commands.rs::send_answer_sync`
- 重连后全量同步入口：`apps/student/src-tauri/src/network/ws_client.rs::send_full_answer_sync_for_current_session`
- ACK 成功回写：`apps/student/src-tauri/src/services/exam_runtime_service.rs::mark_answers_synced`
- ACK 失败回写：`apps/student/src-tauri/src/services/exam_runtime_service.rs::mark_answers_failed`
- 后台补发：`apps/student/src-tauri/src/services/exam_runtime_service.rs::flush_pending_answer_sync`

### 教师端接收与 ACK

- 接收入口：`apps/teacher/src-tauri/src/network/ws_server.rs::handle_text_message`
- 幂等落库：`apps/teacher/src-tauri/src/network/ws_server.rs::persist_answer_sync`
- ACK 返回：`apps/teacher/src-tauri/src/network/ws_server.rs::send_answer_sync_ack`

## 当前实现边界

1. 教师端 ACK 已支持分题级别结果（成功/失败题目列表与计数）。
2. 全量同步为重连后的自愈主路径，增量补发用于连接期间的常规重试。
3. 目前 `answer_sheets` 幂等冲突键已收敛为 `UNIQUE(student_exam_id, question_id)`，避免跨考试误覆盖。
4. 同一 endpoint 下的 `student_id` 切换已被视为重连目标切换，而不是旧连接复用。
5. full 同步带冷却保护，目标是避免抖动期间重复全量，而不是禁止后续正常自愈。

## 最小验收步骤

1. 正常开考并作答 2-3 题，观察学生端出现发送日志。
2. 强制关闭教师端，学生端继续作答 2-3 题。
3. 重启教师端并等待学生端重连。
4. 若此期间教师端重新下发了同 endpoint 但不同 `student_id` 的连接目标，观察学生端日志：应先断开旧连接，再按新身份重连。
5. 观察学生端日志：应出现 full 同步发送、ACK 成功、`synced_count` 增长，且同一轮重连内不会无节制重复 full sync。
6. 观察教师端日志：应持续出现 `answer_sync`，并无落库失败；若存在部分失败，ACK 中应带 `failedQuestionIds` 与失败计数。
7. 打开教师端监考/报告页，确认进度追平到恢复后最新状态。

## 相关阅读

1. [重连后学生答案全量同步与 ACK 收敛计划](../plans/2026_03_23_重连后学生答案全量同步与ACK收敛计划.md)
2. [教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路](./e2e-minimal-answer-sync-progress-chain.md)
3. [学生端启动恢复与断线重连的最短 e2e 链路](./e2e-minimal-student-startup-reconnect-chain.md)
