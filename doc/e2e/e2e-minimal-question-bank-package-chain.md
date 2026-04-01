# 教师端题目列表导出资源包并回导入考试题库的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端在题目列表页勾选题目并点击“导出题目”之后，资源包如何生成；以及该资源包如何在题库导入页被重新导入到指定考试的 `questions` 表。

这里刻意不展开：

1. 题目图片后续如何下发到学生端。
2. 学生端考试页如何渲染题干图与选项图。
3. 发卷、开考、答案同步与结束考试链路。

只聚焦当前已经形成的前置闭环：

1. 题目列表勾选导出。
2. 导出 zip 资源包。
3. 题库导入页选择 zip。
4. 指定考试题目被覆盖写入。

## 最短链路结论

最短链路如下：

1. 教师端进入题目列表页，在 `QuestionBank` 表格中勾选若干题目。
2. 教师端点击 toolbar 的“导出题目”。
3. 前端 `QuestionBankPage` 基于勾选项生成 `question_bank.xlsx` 的行数据，并收集题干图与选项图的相对路径。
4. 前端通过 `questionService.exportQuestionBankPackage` 发起 Tauri `invoke`：`export_question_bank_package`。
5. 教师端 Rust `question_bank_controller::export_question_bank_package` 调用 `question_bank_service::export_question_bank_package`。
6. 服务层把前端传入的 xlsx 字节写入临时 `question_bank.xlsx`，再把题目图片整理为 `assets/content` 与 `assets/options` 条目，最后调用 `utils::asset_zip::create_asset_zip` 输出 zip 到本机下载目录。
7. 用户进入题库导入页 `QuestionImport`，选择目标考试后点击“导入资源包”。
8. 前端通过 `useQuestion.importQuestionPackageByExamId` 发起 Tauri `invoke`：`import_question_package_by_exam_id`。
9. 教师端 Rust `question_controller::import_question_package_by_exam_id` 调用 `question_service::import_question_package_by_exam_id`。
10. 服务层先解压 zip，再读取其中的 `question_bank.xlsx`，按表头解析为 `QuestionWritePayload` 数组。
11. 服务层调用 `replace_questions_by_exam_id`，先清空该考试现有 `questions`，再插入导入题目。
12. 前端刷新题目列表，`QuestionImport` 表格显示导入后的最新题目。

到第 6 步为止，已经完成“题目列表导出资源包”的最短闭环。

到第 11 步为止，已经完成“资源包回导入考试题库”的最短后端闭环。

## 关键入口

这条链路当前有两个明确前端入口：

1. 题目列表导出入口：
   - [apps/teacher/src/pages/QuestionBank/index.tsx](../../apps/teacher/src/pages/QuestionBank/index.tsx)
   - `handleExport`
2. 题库导入入口：
   - [apps/teacher/src/pages/QuestionImport/index.tsx](../../apps/teacher/src/pages/QuestionImport/index.tsx)
   - `handleImportPackage`

对应前端 service 入口分别在：

1. [apps/teacher/src/services/questionService.ts](../../apps/teacher/src/services/questionService.ts)
   - `exportQuestionBankPackage`
   - `importQuestionPackageByExamId`

对应教师端 Rust 命令入口分别在：

1. [apps/teacher/src-tauri/src/controllers/question_bank_controller.rs](../../apps/teacher/src-tauri/src/controllers/question_bank_controller.rs)
   - `export_question_bank_package`
2. [apps/teacher/src-tauri/src/controllers/question_controller.rs](../../apps/teacher/src-tauri/src/controllers/question_controller.rs)
   - `import_question_package_by_exam_id`

## 当前实现边界

当前这条链路已经成立，但边界需要明确：

1. 导出范围是题目列表页当前勾选项，不是全量题库自动导出。
2. zip 目录结构固定为：
   - `question_bank.xlsx`
   - `assets/content/*`
   - `assets/options/*`
3. 导出的 xlsx 当前主字段对齐现有导入能力：`序号 / 题型 / 题目内容 / 选项 / 答案 / 分值 / 解析`。
4. xlsx 里虽然已经附带 `题干图片` 与 `选项图片映射` 字段，zip 里也已经携带图片资源，但当前导入落库仍以 `questions` 表现有字段为准。
5. 也就是说，当前资源包链路已经把“可打包、可携带、可被导入页消费”的前置条件建立起来，但图片资源本身还没有继续写入考试题目的专用图片字段，因为 `questions` 当前仍是文本结构。
6. 旧的纯 xlsx 导入仍然保留；zip 导入是新增分支，不替代原入口。

## 哪些内容不属于这条链路

以下内容与资源包业务相关，但不属于“题目列表导出资源包并回导入考试题库”的最短链路：

1. `QuestionBank` 中单题新增、编辑、删除的 CRUD 流程。
2. 题目图片后续如何发卷到学生端。
3. 学生端收到试卷后如何恢复本地图片路径。
4. `questions_payload` 如何下发到学生端控制端口。

其中第 4 项应回看：

1. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)

## 最小验收步骤

1. 进入教师端题目列表页，勾选 2 到 3 条题目。
2. 点击“导出题目”，确认本机生成 zip 文件。
3. 解压 zip，确认结构包含 `question_bank.xlsx`、`assets/content/`、`assets/options/`。
4. 进入题库导入页，选择某个考试，点击“导入资源包”。
5. 选择上一步导出的 zip，确认提示导入成功。
6. 观察题库导入页表格，确认已显示资源包中的题目。
7. 若该考试原先存在题目，再次查询时应确认旧题目已被覆盖，而不是追加。

## 相关阅读

1. [项目依赖拓扑图](../project_dependency_topology.md)
2. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)
3. [教师端与学生端题目图片下发与渲染计划（2026-03-31）](../plans/2026_03_31_题目图片下发与学生端渲染计划.md)