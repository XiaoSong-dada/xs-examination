# 教师端按考试导入题目资源包的最短 e2e 链路

## 目标

这份文档只回答一个问题：

教师端在题库导入页选择某场考试并导入 zip 资源包后，题目数据和图片资源如何被写入该考试的 `questions` 表，以及这条链路如何为后续发卷准备图片字段。

这里刻意不展开：

1. `QuestionBank` 如何把资源包导回 `question_bank_items`。
2. 教师端如何把题目图片二进制继续下发到学生端。
3. 学生端最终如何渲染题干图和选项图。

只聚焦当前已经形成的按考试导入闭环：

1. `QuestionImport` 选择考试。
2. 选择 zip 资源包。
3. 后端解压 `question_bank.xlsx + assets/content + assets/options`。
4. 后端把图片复制到考试维度目录并映射路径。
5. 后端按 `exam_id` 覆盖写入 `questions`。

## 最短链路结论

最短链路如下：

1. 教师端进入 `QuestionImport` 页面并先选择目标考试。
2. 教师端点击“导入资源包”，前端通过文件对话框选择 zip 文件。
3. 前端 `QuestionImportPage` 调用 `useQuestion.importQuestionPackageByExamId`。
4. 前端 `questionService.importQuestionPackageByExamId` 发起 Tauri `invoke`：`import_question_package_by_exam_id`。
5. 教师端 Rust `question_controller::import_question_package_by_exam_id` 调用 `question_service::import_question_package_by_exam_id`。
6. 服务层先把 zip 解压到临时目录，并定位其中的 `question_bank.xlsx`。
7. 服务层扫描 `assets/content` 与 `assets/options`，把图片复制到教师端受控目录 `uploads/images/questions/{exam_id}/content|options`。
8. 服务层解析 xlsx：
   - 题干图片列映射到 `content_image_paths`
   - 选项 JSON 中的 `image_paths` 映射到新的本地相对路径
9. 服务层调用 `replace_questions_by_exam_id`，先清理该考试原有 `answer_sheets` 与 `questions`，再把解析后的题目覆盖写入 `questions`。
10. 前端收到导入后的题目列表，页面刷新后显示该考试最新题目。
11. 教师端后续发卷时，`student_exam_service::distribute_exam_papers_by_exam_id` 会继续从 `questions` 读取这些题目，并把 `contentImagePaths` 一并写入 `questions_payload`。

到第 9 步为止，已经完成“按考试导入资源包并落库”的最短后端闭环。

## 关键入口

这条链路当前有一个明确前端入口：

1. [apps/teacher/src/pages/QuestionImport/index.tsx](../../apps/teacher/src/pages/QuestionImport/index.tsx)
   - `handleImportPackage`

对应前端 hook / service 入口在：

1. [apps/teacher/src/hooks/useQuestion.ts](../../apps/teacher/src/hooks/useQuestion.ts)
   - `importQuestionPackageByExamId`
2. [apps/teacher/src/services/questionService.ts](../../apps/teacher/src/services/questionService.ts)
   - `importQuestionPackageByExamId`

对应教师端 Rust 命令与服务入口在：

1. [apps/teacher/src-tauri/src/controllers/question_controller.rs](../../apps/teacher/src-tauri/src/controllers/question_controller.rs)
   - `import_question_package_by_exam_id`
2. [apps/teacher/src-tauri/src/services/question_service.rs](../../apps/teacher/src-tauri/src/services/question_service.rs)
   - `import_question_package_by_exam_id`
   - `replace_questions_by_exam_id`

与后续发卷直接衔接的下游读取入口在：

1. [apps/teacher/src-tauri/src/services/student_exam_service.rs](../../apps/teacher/src-tauri/src/services/student_exam_service.rs)
   - `distribute_exam_papers_by_exam_id`

## 当前实现边界

当前这条链路已经成立，但边界需要明确：

1. 导入目标是指定考试的 `questions` 表，不是全局题库 `question_bank_items`。
2. 资源包目录结构固定为：
   - `question_bank.xlsx`
   - `assets/content/*`
   - `assets/options/*`
3. 当前导入仍保持覆盖语义，不支持增量合并；同一 `exam_id` 再次导入会先删除旧题目再重写。
4. 题干图片当前存入 `questions.content_image_paths`，格式为 JSON 数组字符串。
5. 选项图片当前仍存于 `questions.options` 的 JSON 结构中，通过每个选项的 `image_paths` 表示。
6. 教师端受控图片目录按考试维度隔离，当前路径口径为 `uploads/images/questions/{exam_id}/...`。
7. 这条链路只负责把图片路径准备到教师端本地数据库，不负责把图片二进制发给学生端。
8. 学生端目前已为后续图片同步预留 `exam_question_assets` 表，以及 `exam_snapshots.assets_sync_status/assets_synced_at` 状态字段，但本链路不会直接写学生端数据库。

## 哪些内容不属于这条链路

以下内容与资源包业务相关，但不属于“按考试导入题目资源包”的最短链路：

1. `QuestionBank` 勾选导出资源包与导回 `question_bank_items` 的流程。
2. 教师端题目 CRUD 的纯前端 Excel 预览导入分支。
3. 教师端发卷时图片二进制的同步方式。
4. 学生端 `exam_question_assets` 何时真正写入图片记录。
5. 学生端题目页最终如何把 `contentImagePaths`、选项图片和本地缓存路径渲染出来。

其中第 3 至第 5 项应回看：

1. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)
2. [教师端与学生端题目图片下发与渲染计划（2026-03-31）](../plans/2026_03_31_题目图片下发与学生端渲染计划.md)

## 最小验收步骤

1. 进入教师端 `QuestionImport` 页面并选择一场考试。
2. 选择一个包含 `question_bank.xlsx + assets/content + assets/options` 的 zip 资源包。
3. 导入成功后刷新题目列表，确认题目条数与 xlsx 一致。
4. 查询该考试 `questions`，确认 `content_image_paths` 已写入题干图片相对路径。
5. 检查 `options` JSON，确认选项中的 `image_paths` 已从 zip 内路径映射到教师端受控目录。
6. 再次对同一考试导入另一份资源包，确认旧题目被覆盖而不是累加。
7. 触发一次发卷，确认教师端发出的 `questions_payload` 已包含 `contentImagePaths` 字段。

## 相关阅读

1. [项目依赖拓扑图](../project_dependency_topology.md)
2. [教师端题目列表导出资源包并回导入全局题库的最短 e2e 链路](./e2e-minimal-question-bank-package-chain.md)
3. [教师端发放试卷到学生端接收试卷的最短 e2e 链路](./e2e-minimal-exam-paper-distribution-chain.md)
