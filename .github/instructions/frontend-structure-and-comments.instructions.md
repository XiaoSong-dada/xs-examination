---
description: "Use when editing frontend structure, adding files, moving business logic, or writing React/TypeScript functions in apps/student/src or apps/teacher/src. Covers folder responsibilities, type placement, JSDoc comments, and code placement rules."
name: "Frontend Structure And Comments"
applyTo: "apps/student/src/**, apps/teacher/src/**"
---

# 前端目录与注释规范

- 新增或修改前端业务代码时，必须先判断应该落到哪个职责目录；不要把页面状态、协议类型、通用工具或 IPC 封装混写在同一层。
- 前端类型统一放在 `src/types/`；`services/` 中已有的历史遗留类型不作为类型入口，后续改动时优先迁回或归并到 `types/`。
- `pages/` 负责路由页面级容器、页面编排与页面态触发；不要在这里直接散落 `invoke` 或长期保存可复用逻辑。
- `hooks/` 负责页面间或模块间可复用的前端业务逻辑；涉及轮询、表格态、表单态、派生状态时，优先沉淀到 hook。
- `services/` 负责 Tauri IPC 调用、请求参数组装、返回值适配；不要把页面展示逻辑、组件渲染逻辑或类型入口放在这里。
- `store/` 负责应用级共享状态，仅在确有跨页面共享需求时使用；局部页面态不要上提到全局 store。
- `components/` 负责可复用的展示组件；页面专有且无复用价值的结构优先留在页面目录内。
- `layout/` 负责应用骨架、导航、头部与通用容器，不承载具体业务流程判断。
- `router/` 负责路由注册与页面装配，不承载页面业务逻辑。
- `utils/` 负责纯函数工具、格式化、校验、转换逻辑；不要在 `utils/` 中放 IPC、副作用或页面状态读写。
- `settings/` 仅用于当前应用的静态配置、字典或 UI 选项组织；不要把运行时业务状态放进 `settings/`。
- `styles/` 负责全局样式入口与主题样式，不承载组件级业务逻辑。

## 类型放置约定

- 组件 Props、页面表格行、表单值、视图模型、前端消费的协议映射类型，统一放在 `src/types/`。
- 若某个类型只被单页面使用，也仍应优先放在对应领域的 `types` 文件中，而不是写在 `services/` 内联导出。
- 若历史代码已在 `services/` 内声明类型，本次任务若触及该类型，应优先评估归并回 `src/types/`；若本次不适合迁移，也不得把该文件继续当成新增类型的入口。

## 注释规范

- 新增或修改的导出函数、React 组件、自定义 Hook、service 函数、store action 必须补 JSDoc 注释；仅为追补存量未改动代码而大面积补注释不属于本阶段要求。
- JSDoc 至少写明三项：函数作用、`@param` 参数说明、`@returns` 返回值说明；如果函数会抛错或向上透传错误，再补 `@throws`。
- React 组件注释写在组件定义上方；若组件通过 props 对外暴露关键交互或数据依赖，应在 `@param` 中明确对应 props 含义。
- Hook 注释需要说明它管理什么状态、依赖什么输入、返回什么能力；不要只写“封装某某逻辑”这类空话。
- service 注释需要说明它调用哪个 Tauri 命令或数据链路、输入参数是什么、返回给前端的是什么形状。

## 命名与落点补充

- 延续现有命名习惯：Hook 使用 `use*.ts`，store 使用 `*Store.ts`，service 使用 `*Service.ts`。
- 页面组件、复用组件优先保持现有目录组织方式；若新增目录，应优先使用与现有页面一致的领域命名，而不是临时缩写。
- 修改业务代码时，若发现代码当前落在错误目录，优先先判断本次任务是否足以安全迁移；若不足以迁移，至少不要继续沿着错误目录扩散相同模式。