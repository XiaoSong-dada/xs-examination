# 教师端题目列表导出资源包并回导入全局题库的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端在题目列表页勾选题目并点击“导出题目”之后，资源包如何生成；以及该资源包如何在题目列表页被重新导入到 `question_bank_items` 表。

这里刻意不展开：

1. 题目图片后续如何下发到学生端。
2. 学生端考试页如何渲染题干图与选项图。
3. 发卷、开考、答案同步与结束考试链路。

只聚焦当前已经形成的前置闭环：

1. 题目列表勾选导出。
2. 导出 zip 资源包。
3. 题目列表页选择 zip。
4. 用户确认后清空全局题库，再把资源包写回 `question_bank_items`。

## 最短链路结论

最短链路如下：

1. 教师端进入题目列表页，在 `QuestionBank` 表格中勾选若干题目。
2. 教师端点击 toolbar 的“导出题目”。
3. 前端 `QuestionBankPage` 基于勾选项生成 `question_bank.xlsx` 的行数据，并收集题干图与选项图的相对路径。
4. 前端通过 `questionService.exportQuestionBankPackage` 发起 Tauri `invoke`：`export_question_bank_package`。
5. 教师端 Rust `question_bank_controller::export_question_bank_package` 调用 `question_bank_service::export_question_bank_package`。
6. 服务层把前端传入的 xlsx 字节写入临时 `question_bank.xlsx`，再把题目图片整理为 `assets/content` 与 `assets/options` 条目，最后调用 `utils::asset_zip::create_asset_zip` 输出 zip 到本机下载目录。
7. 用户仍在题目列表页 `QuestionBank`，点击 toolbar 的“导入资源包”。
8. 前端先弹出确认框，明确提示“导入将先清空当前题目列表中的全部数据”。
9. 用户确认后，前端通过 `questionService.importQuestionBankPackage` 发起 Tauri `invoke`：`import_question_bank_package`。
10. 教师端 Rust `question_bank_controller::import_question_bank_package` 调用 `question_bank_service::import_question_bank_package`。
11. 服务层先解压 zip，再读取其中的 `question_bank.xlsx`，并将 `assets/content`、`assets/options` 复制到教师端受控图片目录。
12. 服务层先清空 `question_bank_items`，再把解析后的题目逐条写入 `question_bank_items`。
13. 前端刷新题目列表，`QuestionBank` 表格显示导入后的最新题目。

到第 6 步为止，已经完成“题目列表导出资源包”的最短闭环。

到第 12 步为止，已经完成“资源包回导入全局题库”的最短后端闭环。

## 关键入口

这条链路当前有两个明确前端入口：

1. 题目列表导出入口：
   - [apps/teacher/src/pages/QuestionBank/index.tsx](../../apps/teacher/src/pages/QuestionBank/index.tsx)
   - `handleExport`
2. 题目列表导入入口：
   - [apps/teacher/src/pages/QuestionBank/index.tsx](../../apps/teacher/src/pages/QuestionBank/index.tsx)
   - `handleImportPackage`

并行存在但不属于这条主链的旧入口：

1. [apps/teacher/src/pages/QuestionImport/index.tsx](../../apps/teacher/src/pages/QuestionImport/index.tsx)
   - 仍可把资源包导入到指定考试的 `questions` 表。

对应本链路前端 service 入口在：

1. [apps/teacher/src/services/questionService.ts](../../apps/teacher/src/services/questionService.ts)
   - `exportQuestionBankPackage`
   - `importQuestionBankPackage`

对应本链路教师端 Rust 命令入口分别在：

1. [apps/teacher/src-tauri/src/controllers/question_bank_controller.rs](../../apps/teacher/src-tauri/src/controllers/question_bank_controller.rs)
   - `export_question_bank_package`
   - `import_question_bank_package`

并行存在但不属于这条主链的旧入口：

1. [apps/teacher/src/pages/QuestionImport/index.tsx](../../apps/teacher/src/pages/QuestionImport/index.tsx)
   - 仍可把资源包导入到指定考试的 `questions` 表。
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
4. 当前导入主目标是 `question_bank_items`，不是指定考试的 `questions` 表。
5. 导入前会先弹确认框；用户确认后，后端会先清空 `question_bank_items` 再逐条写入。
6. xlsx 里的 `题干图片` 与 `选项图片映射` 会被解析，zip 中图片也会复制到教师端 `uploads/images/question-bank/content|options` 目录，然后把新的相对路径写入题库表字段。
7. `QuestionImport` 中按考试导入 zip 到 `questions` 的能力仍并行存在，但它不是这条“题目列表导出/导入”主链的真实出口。

## 哪些内容不属于这条链路

以下内容与资源包业务相关，但不属于“题目列表导出资源包并回导入全局题库”的最短链路：

1. `QuestionBank` 中单题新增、编辑、删除的 CRUD 流程。
2. `QuestionImport` 如何把资源包导入指定考试的 `questions` 表。
3. 题目图片后续如何发卷到学生端。
4. 学生端收到试卷后如何恢复本地图片路径。
5. `questions_payload` 如何下发到学生端控制端口。

其中第 4 项应回看：

1. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)

## 最小验收步骤

1. 进入教师端题目列表页，勾选 2 到 3 条题目。
2. 点击“导出题目”，确认本机生成 zip 文件。
3. 解压 zip，确认结构包含 `question_bank.xlsx`、`assets/content/`、`assets/options/`。
4. 回到题目列表页，点击“导入资源包”。
5. 选择上一步导出的 zip，确认先出现“将清空当前题目列表全部数据”的提示。
6. 点击确认后，提示导入成功。
7. 刷新题目列表，确认表格数据已替换成资源包中的题目。
8. 检查图片字段对应的相对路径，确认已指向教师端受控目录，而不是 zip 内临时路径。

## 相关阅读

1. [项目依赖拓扑图](../project_dependency_topology.md)
2. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)
3. [教师端与学生端题目图片下发与渲染计划（2026-03-31）](../plans/2026_03_31_题目图片下发与学生端渲染计划.md)