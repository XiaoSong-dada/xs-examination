---
description: "Use when editing Rust backend structure, adding modules, moving backend logic, or writing Rust functions under apps/*/src-tauri/src. Covers layer responsibilities, teacher/student structure mapping, Rustdoc comments, and code placement rules."
name: "Backend Structure And Comments"
applyTo: "apps/student/src-tauri/src/**, apps/teacher/src-tauri/src/**"
---

# 后端目录与注释规范

- 后端新增或修改业务代码时，必须先判断代码应落到哪个职责层；不要把控制层、业务层、数据库交互层、网络通讯层和结构体声明混写在同一个文件或目录。
- 当前后端分层以教师端现有结构为主要参考口径：`controllers` 作为控制层、`core` 作为统一核心配置层、`models` 作为 ORM 映射层、`network` 作为网络通讯层、`repos` 作为数据库交互层、`services` 作为业务实现层、`utils` 作为统一工具类、`schemas` 作为实例层与结构体声明层。
- 学生端结构与教师端不完全一致；写规范时要做职责映射，不要机械要求学生端复制教师端目录名。

## 层次职责约定

- `controllers/`：控制层。负责承接 Tauri 命令入口、参数接收、调用业务服务并返回前端可消费结果。教师端以 `lib.rs` 中统一注册的 controller 命令为主；学生端若仍通过 `commands.rs` 暴露命令，也视为同一职责的入口层。
- `core/`：统一核心配置层。负责系统级设置、运行时核心配置、基础初始化辅助能力；不要把具体业务逻辑塞进 `core/`。
- `models/`：ORM 映射层。负责数据库实体模型、表字段映射与持久化实体定义；不要把控制层 payload 或网络消息结构放进 `models/`。
- `network/`：网络通讯层。负责 WebSocket、TCP、控制消息、广播、连接维护等通信逻辑；不要把纯数据库交互写在这里。
- `repos/`：数据库交互层。负责数据库读写、查询封装与持久化访问；不要在 `services/` 或 `controllers/` 中长期散落重复 SQL / ORM 查询。
- `services/`：业务实现层。负责跨层编排、业务规则、流程推进、调用 repo/network/utils；不要把 Tauri 命令注册或前端返回协议直接写成 service 职责。
- `utils/`：统一工具类。负责通用纯辅助逻辑、转换、时间处理、环境读取辅助、可复用工具函数；不要把具体业务流程状态放进 `utils/`。
- `schemas/`：实例层。负责声明 DTO、payload、输入输出结构体、控制层与网络层共享的载荷结构；新增结构体默认优先放在 `schemas/`，不要把普通实例结构分散到 service、controller、network 文件内部。

## teacher / student 结构映射

- 教师端当前已有 `controllers`、`core`、`models`、`network`、`repos`、`services`、`utils`、`schemas`，新增后端代码优先沿用这套职责落点。
- 教师端 `lib.rs` 负责统一注册命令，因而 controller 是教师端 Tauri 命令的主入口；若新增前端可调用能力，优先从 controller 进入，再分派到 service。
- 学生端当前以 `commands.rs` 作为主要命令入口，这一层在职责上等价于教师端的控制层入口；若学生端后续逐步细化 controller，仍应保持“入口层只做参数接收与分派”的边界。
- 学生端 `layers/` 当前用于承载基础设施辅助分层；在职责映射上，可视为围绕配置、持久化等底层能力的支撑层，而不是 `services`、`repos`、`schemas` 的替代品。
- 学生端当前 `repos/` 目录为空；若确实出现稳定的数据库交互逻辑，再把重复持久化访问下沉到 `repos/`，不要为了形式统一先空建复杂层次。

## 默认落点规则

- 新增业务规则、业务编排、跨模块流程推进，优先放到 `services/`。
- 新增数据库读写、查询封装、实体查找，优先放到 `repos/`；若当前端侧应用尚未形成 repo 抽象，则至少保持持久化逻辑集中，不要在多个入口层重复实现。
- 新增网络消息、连接处理、推送/广播逻辑，优先放到 `network/`。
- 新增 DTO、payload、命令入参、命令出参、通信结构体，优先放到 `schemas/`。
- 新增 ORM 实体、数据库表映射，优先放到 `models/` 或 student 端现有 `db/entities` 一类映射层位置，不要混入 `schemas/`。
- 新增全局配置与核心设置读取，优先放到 `core/` 或现有配置入口；教师端当前可先看 `core/setting.rs` 与 `config.rs`，学生端按现有 `config.rs` 与 `layers/config` 结构落点。
- 新增通用辅助函数，优先放到 `utils/`；若函数强依赖具体业务上下文，则不要硬塞进 `utils/`。

## 术语映射约定

- 控制层：教师端主要对应 `controllers/`，学生端主要对应 `commands.rs` 与 `controllers/`。
- 核心配置层：教师端主要对应 `core/` 与 `config.rs`，学生端主要对应 `config.rs` 与 `layers/config/`。
- ORM 映射层：教师端主要对应 `models/`，学生端主要对应 `db/entities` 等实体映射位置。
- 实例层：统一对应 `schemas/`，用于声明结构体、DTO、payload、输入输出实例。
- 数据库交互层：统一对应 `repos/`。
- 业务实现层：统一对应 `services/`。
- 网络通讯层：统一对应 `network/`。
- 工具类：统一对应 `utils/`。

## 注释规范

- 新增或修改的 Rust 导出函数、Tauri 命令函数、service 方法、repo 方法、network 关键函数、utils 公共函数，必须补 Rustdoc 注释；仅为追补存量未改动代码而大面积补注释不属于本阶段要求。
- Rustdoc 至少写明三项：函数作用、`# 参数`、`# 返回值`；若返回 `Result`，应在返回值说明中写明主要错误来源或错误类型。
- Tauri 命令函数注释要说明该命令给前端提供什么能力、依赖哪些入参、返回什么数据或错误。
- service 函数注释要说明其业务职责、调用了哪些下游层、输出结果是什么；不要只写“处理某某逻辑”这类空话。
- repo 函数注释要说明其查询或持久化对象、关键筛选条件以及返回实体含义。
- network 函数注释要说明处理的消息、连接或广播语义，以及失败后的处理方式。

## 边界补充

- `schemas/` 是实例层，不是 ORM 映射层；数据库实体与表映射不要放进 `schemas/`。
- `models/` 是 ORM 映射层，不是前后端传输结构体入口；不要把普通命令入参、网络 payload 放进 `models/`。
- `core/` 负责核心配置与初始化辅助，不是通用工具目录；纯工具函数仍应放在 `utils/`。
- `controllers` 或 `commands` 只作为入口层，不承担完整业务实现；复杂流程必须下沉到 `services/`。