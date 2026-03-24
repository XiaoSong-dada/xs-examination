---
description: "Use when editing React, TypeScript, Zustand, hooks, stores, services, or page components in apps/student/src or apps/teacher/src. Covers Tauri service boundaries, shared types, import aliases, and frontend naming patterns."
name: "Frontend Boundaries"
applyTo: "apps/student/src/**, apps/teacher/src/**"
---

# 前端边界规范

- 应用内部导入优先使用 `@/` 路径别名，避免深层相对路径。
- 前端访问 Tauri 时，优先通过 `src/services` 封装，再由 store、hook 或页面消费；不要在页面组件中直接散落 `invoke`。
- 跨端共享的协议类型放在 `packages/shared-types`；仅限单端使用的视图模型、组件类型与页面类型保留在各自应用内部。
- 延续现有命名习惯：自定义 Hook 使用 `use*.ts`，store 使用 `*Store.ts`，service 使用 `*Service.ts`。
