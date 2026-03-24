---
description: "Use when editing React, TypeScript, Zustand, hooks, stores, services, or page components in apps/student/src or apps/teacher/src. Covers Tauri service boundaries, protocol source checks, import aliases, and frontend naming patterns."
name: "Frontend Boundaries"
applyTo: "apps/student/src/**, apps/teacher/src/**"
---

# 前端边界规范

- 应用内部导入优先使用 `@/` 路径别名，避免深层相对路径。
- 前端访问 Tauri 时，优先通过 `src/services` 封装，再由 store、hook 或页面消费；不要在页面组件中直接散落 `invoke`。
- 当前不要把 `packages/shared-types` 视为前端共享协议的现行来源；涉及跨端协议字段时，先核对对应端的 `services`、`types`、Tauri command 返回结构与正式文档，再决定落点。
- 仅限单端使用的视图模型、组件类型与页面类型保留在各自应用内部；不要为了“看起来共享”就提前把未复用的类型抽到预留目录。
- 延续现有命名习惯：自定义 Hook 使用 `use*.ts`，store 使用 `*Store.ts`，service 使用 `*Service.ts`。
