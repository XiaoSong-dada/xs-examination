# 教师端成绩报告统计与导出的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端考试已经结束后，成绩报告页如何统计学生总分、把结果写入数据库，并在导出时生成本地 Excel 文件。

这里刻意不展开：

1. 连接考生设备
2. 分发试卷
3. 开始考试
4. 结束考试时的 final `ANSWER_SYNC`
5. 主观题人工批阅
6. Office / WPS 对导出文件格式的兼容性差异

只聚焦当前已经形成的最短闭环：

1. 教师端进入成绩报告页
2. 教师端点击“统计成绩”
3. 教师端按 `answer_sheets + questions + exams` 聚合总分
4. 教师端把总分覆盖写入 `score_summary`
5. 教师端点击“导出成绩”
6. 前端生成 Excel 并触发本地下载，同时提示预期下载位置

## 最短链路结论

最短链路如下：

1. 教师端进入成绩报告页，`useReport.ts` 先通过 `get_student_device_connection_status_by_exam_id` 读取答题进度，再通过 `get_student_score_summary_by_exam_id` 读取当前考试已有的成绩汇总。
2. 若当前考试尚未统计过成绩，则 Report 表格里的 `score` 默认回退为 0，但不会自动触发统计。
3. 教师端点击“统计成绩”后，前端调用 `calculate_student_score_summary_by_exam_id`。
4. 教师端 Rust `student_exam_service::recalculate_student_score_summary_by_exam_id` 先校验 `exams.status == finished`，只有已结束考试才允许统计。
5. 教师端 Rust `student_exam_repo::recalculate_student_score_summary_by_exam_id` 读取 `student_exams`、`answer_sheets`、`questions` 与 `exams.pass_score`，按学生聚合总分，并先清空当前考试旧的 `score_summary`，再重算覆盖写入新结果。
6. 前端统计成功后重新调用 `refresh()`，再次读取 `get_student_score_summary_by_exam_id`，把表格中的 `score` 替换为真实总分。
7. 教师端点击“导出成绩”后，前端 `useReport.ts` 用当前表格数据生成工作簿，并通过 `XLSX.writeFile` 触发本地下载。
8. 导出成功后，前端再调用 `resolve_report_download_path`，根据系统下载目录拼出预期文件路径，并在成功提示中展示该路径。
9. 若当前考试尚未统计成绩，或表格没有学生成绩数据，则前端不会继续导出，而是提示先统计或检查数据。
10. 导出成功后，前端会把该考试状态从 `finished` 更新为 `archived`，表示本次考试已进入成绩沉淀后的归档阶段。

到第 5 步为止，已经完成“成绩统计并落库”的最短后端闭环。

到第 8 步为止，已经完成“成绩报告导出并给出下载位置”的最短页面闭环。

## 入口在哪里

教师端页面入口在：

- [apps/teacher/src/pages/Report/index.tsx](../../apps/teacher/src/pages/Report/index.tsx)

页面中的两个关键动作分别是：

1. 点击“统计成绩” -> `handleCalculateScores`
2. 点击“导出成绩” -> `handleExport`

这两个动作都下沉到同一个 Hook：

- [apps/teacher/src/hooks/useReport.ts](../../apps/teacher/src/hooks/useReport.ts)

## 真实出口在哪里

这条链路当前有三个真实出口：

1. `score_summary` 中保存了当前考试每个学生的总分、是否及格与统计时间。
2. Report 页表格中的 `score` 不再是占位值，而是来自 `get_student_score_summary_by_exam_id` 的真实汇总结果。
3. 前端触发下载后，会在系统下载目录生成导出文件，并在提示中显示预期保存路径。

其中数据库出口在：

- `score_summary`

其中前端查询出口在：

- [apps/teacher/src/services/studentService.ts](../../apps/teacher/src/services/studentService.ts)
- [apps/teacher/src/hooks/useReport.ts](../../apps/teacher/src/hooks/useReport.ts)

其中后端统计入口与出口在：

- [apps/teacher/src-tauri/src/controllers/student_exam_controller.rs](../../apps/teacher/src-tauri/src/controllers/student_exam_controller.rs)
- [apps/teacher/src-tauri/src/services/student_exam_service.rs](../../apps/teacher/src-tauri/src/services/student_exam_service.rs)
- [apps/teacher/src-tauri/src/repos/student_exam_repo.rs](../../apps/teacher/src-tauri/src/repos/student_exam_repo.rs)

## 最短调用链是什么

### 1. 报告页加载与数据读取

1. [apps/teacher/src/pages/Report/index.tsx](../../apps/teacher/src/pages/Report/index.tsx)
2. [apps/teacher/src/hooks/useReport.ts](../../apps/teacher/src/hooks/useReport.ts)
3. [apps/teacher/src/services/studentService.ts](../../apps/teacher/src/services/studentService.ts)
4. `get_student_device_connection_status_by_exam_id`
5. `get_student_score_summary_by_exam_id`

这条读取链的作用是把“答题进度”和“成绩总分”组合成 Report 页当前表格数据。

### 2. 统计成绩并落库

1. `ReportPage.handleCalculateScores`
2. `useReport.calculateScores`
3. `studentService.calculateStudentScoreSummaryByExamId`
4. `student_exam_controller::calculate_student_score_summary_by_exam_id`
5. `student_exam_service::recalculate_student_score_summary_by_exam_id`
6. `student_exam_repo::recalculate_student_score_summary_by_exam_id`
7. 清空并重写 `score_summary`

### 3. 导出本地 Excel

1. `ReportPage.handleExport`
2. `useReport.exportReport`
3. `XLSX.utils.json_to_sheet`
4. `XLSX.writeFile`
5. `studentService.resolveReportDownloadPath`
6. `student_exam_controller::resolve_report_download_path`
7. 页面提示导出成功与预期下载位置

## 哪些内容不属于这条链路

以下内容与成绩报告业务相关，但不属于“统计成绩 -> 导出成绩”的最短链路：

1. 结束考试时的 final `ANSWER_SYNC` 与 `finished` 门禁
2. 学生端按题答案同步与 `student_exam_progress` 聚合细节
3. 主观题单题人工评分
4. 导出文件在 Office/WPS 中被识别成 `xlsm` 的兼容性处理
5. 成绩导出后是否自动打开所在文件夹

其中第 1、2 项应分别回看已有 e2e：

1. [教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路](./e2e-minimal-answer-sync-progress-chain.md)
2. [教师端结束考试并触发学生端最终同步的最短 e2e 链路](./e2e-minimal-end-exam-final-sync-chain.md)

## 最小验收步骤

1. 教师端完成连接、发卷、开始考试与结束考试，使 `exams.status = finished`。
2. 进入成绩报告页，确认表格能看到学生姓名、设备 IP 与答题进度。
3. 点击“统计成绩”，确认前端提示成功。
4. 检查数据库 `score_summary`，确认存在对应 `exam_id` 的学生总分记录。
5. 再次观察 Report 页，确认 `score` 不再是 0，而是统计后的真实值。
6. 点击“导出成绩”，确认浏览器下载动作已触发，且成功提示中出现预期下载位置。
7. 导出成功后，确认 `exams.status` 从 `finished` 切换为 `archived`。

## 相关阅读

1. [教师端开始考试到学生端按题同步并更新监考进度的最短 e2e 链路](./e2e-minimal-answer-sync-progress-chain.md)
2. [教师端结束考试并触发学生端最终同步的最短 e2e 链路](./e2e-minimal-end-exam-final-sync-chain.md)
3. [项目依赖拓扑图](../project_dependency_topology.md)