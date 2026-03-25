# xs-examination 项目工作规范

## 构建与校验

- 在仓库根目录执行 `pnpm install` 安装依赖。
- 在仓库根目录执行 `pnpm dev:teacher` 或 `pnpm dev:student` 启动教师端或学生端桌面应用。
- 前端改动后，使用 `pnpm --filter @xs/teacher build` 或 `pnpm --filter @xs/student build` 校验对应端的构建。
- Tauri 后端改动后，在 `apps/teacher/src-tauri` 或 `apps/student/src-tauri` 目录执行 `cargo check` 做最小编译校验。
- 当前工作区没有成熟的 JS 测试框架。除非任务本身引入测试框架，否则优先使用构建校验和针对性的运行时验证。

## 架构

- 本仓库是一个 pnpm workspace monorepo：`apps/teacher` 和 `apps/student` 分别是独立的 Tauri 应用；`packages/shared-types` 当前仍保留在仓库中，但已确认不是现行代码路径依赖。
- 每个应用都分为 `src` 下的 React 前端和 `src-tauri` 下的 Rust Tauri 后端。
- 前端边界、类型入口与目录职责的细化规则，统一以下列拆分规范为准：`.github/instructions/frontend-boundaries.instructions.md`、`.github/instructions/frontend-structure-and-comments.instructions.md`。
- 跨端协议、shared-types 与后端 DTO 的细化规则，统一以下列拆分规范为准：`.github/instructions/protocol-and-shared-types.instructions.md`、`.github/instructions/tauri-backend.instructions.md`、`.github/instructions/backend-structure-and-comments.instructions.md`。

## 约定与易错点

- 教师端和学生端的 Vite 开发端口都是固定的，并启用了 `strictPort: true`：教师端为 `1420`，学生端为 `1430`。如果开发命令启动即失败，先检查端口冲突。
- 若任务涉及业务逻辑、跨层调用、链路排查或入口定位，先阅读 `doc/project_dependency_topology.md`；如果拓扑图已映射对应业务的最短 e2e 文档，再继续阅读对应 `doc/e2e/*.md`。
- 学生端运行时以会话为中心。修改发卷、启动恢复、断线重连或答案同步时，不要只补 UI，要同时核对 `exam_sessions` 和 `exam_snapshots` 的前提是否仍成立。
- 教师端答案同步的持久化依赖 SQLite migration，以及 `student_exams` 到 `answer_sheets` 的数据链路。修改同步逻辑前，应先核对相关 migration，不要先假定运行时数据结构正确。
- 计划文档与 e2e 最短链路文档是实际开发流程的一部分；更新条件、命名和写法以 `.github/instructions/e2e-docs.instructions.md` 为准。

## 关键文档

- 产品范围与目标：[doc/PRD.md](../doc/PRD.md)
- 技术设计：[doc/TECH_DESIGN.md](../doc/TECH_DESIGN.md)
- 调研记录：[doc/RESEARCH.md](../doc/RESEARCH.md)
- 项目依赖拓扑图：[doc/project_dependency_topology.md](../doc/project_dependency_topology.md)
- 考试状态流转：[doc/exam_status_flow.md](../doc/exam_status_flow.md)
- 设备发现协议：[doc/device_discovery_api_contract.md](../doc/device_discovery_api_contract.md)
- E2E 业务链路文档：[doc/e2e](../doc/e2e)
- 持续更新中的实施计划：[doc/plans](../doc/plans)

## 区域入口

- 学生端前端参考入口：`apps/student/src/store`、`apps/student/src/services`、`apps/student/src/pages/Exam`
- 教师端前端参考入口：`apps/teacher/src/hooks`、`apps/teacher/src/services`、`apps/teacher/src/pages`
- 学生端后端入口：`apps/student/src-tauri/src/commands.rs`、`apps/student/src-tauri/src/services`、`apps/student/src-tauri/src/network`
- 教师端后端入口：`apps/teacher/src-tauri/src/commands`、`apps/teacher/src-tauri/src/services`、`apps/teacher/src-tauri/src/network`

## 后续定制建议

- 更细粒度的规范已拆分到 `.github/instructions/`，修改前端、Tauri 后端、跨端协议或 `doc/` 文档时，优先复用对应 instructions，而不是继续把细节堆回这个全局文件。
- 如果这份工作区说明逐渐变得过宽，应为 `apps/student/**`、`apps/teacher/**` 或 `doc/**` 拆分更聚焦的 `*.instructions.md`，而不是继续把所有内容堆进这个文件。
- 这份文件只保留对绝大多数任务都有效的事实；更详细的内容优先通过链接指向现有文档，而不是在这里重复复制。
