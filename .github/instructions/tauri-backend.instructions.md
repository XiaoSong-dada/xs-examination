---
description: "Use when editing Rust, Tauri commands, repos, services, network handlers, or persistence code under apps/*/src-tauri/src. Covers command naming, DTO placement, migration-only schema changes, and safe backend coding."
name: "Tauri Backend"
applyTo: "apps/student/src-tauri/src/**, apps/teacher/src-tauri/src/**"
---

# Tauri 后端规范

- Rust 模块文件与 Tauri 命令函数保持 `snake_case` 命名；结构体和枚举保持 `PascalCase`。
- 后续新增 Tauri 命令统一通过 `controllers/` 暴露；学生端现有 `commands.rs` 视为历史遗留入口，不再作为新增命令的规范落点。
- 前后端交互、控制层或网络载荷使用的 DTO、payload、schema，优先放在 `src-tauri/src/schemas/`，不要把普通传输结构分散在各层文件里。
- 新增公共逻辑前，先检查 `utils` 或已有 service/repo 是否已经有可复用实现，避免跨模块重复实现。
- 生产代码路径避免使用 `.unwrap()` 或 `.expect()`；优先返回 `Result` 并显式传播错误。
- 数据库结构变更只能通过 `src-tauri/migrations` 下的 SQL 文件完成，不要在业务代码里隐式改 schema。
