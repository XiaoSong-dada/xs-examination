# xs-examination

局域网分布式在线考试系统。

本项目面向无公网或弱网环境下的学校机房、培训教室等场景，提供教师端与学生端两个桌面应用，目标是在局域网内完成考试创建、发卷、作答、监考、评分与成绩导出等核心流程，并在网络波动场景下保持稳定运行。

## 项目目标

- 教师端支持考试管理、题库导入、监考、阅卷与成绩统计。
- 学生端支持自动发现考试、身份进入、沉浸式答题与断网续答。
- 系统基于局域网运行，强调稳定性、可恢复性与数据安全。

## 当前仓库结构

```text
xs-examination/
├─ apps/
│  ├─ teacher/          # 教师端 Tauri 应用（React + Rust）
│  └─ student/          # 学生端 Tauri 应用（React + Rust）
├─ packages/
│  └─ shared-types/     # 前后端共享 TypeScript 类型
├─ doc/                 # 产品、设计、调研文档
├─ package.json         # Workspace 根脚本
└─ pnpm-workspace.yaml
```

## 技术栈

### 前端

- React
- TypeScript
- Ant Design
- Zustand
- Vite

### 桌面与后端

- Tauri 2
- Rust
- Tokio
- SeaORM
- SQLite
- sqlx migrations
- tokio-tungstenite
- axum
- mdns-sd

## Monorepo 说明

本项目使用 pnpm workspace 组织多包结构：

- `apps/teacher`：教师端应用
- `apps/student`：学生端应用
- `packages/shared-types`：共享类型包，供两端前端复用

根目录脚本当前提供：

```bash
pnpm dev:teacher
pnpm dev:student
pnpm build:teacher
pnpm build:student
```

## 开发前置要求

开始前请确保本机具备以下环境：

- Node.js
- pnpm
- Rust toolchain
- Tauri 所需本地开发环境

Windows 下运行 Tauri 通常还需要完整的 Rust/桌面构建依赖。如果本地尚未配置，可先按 Tauri 官方文档完成环境安装。

## 快速开始

### 1. 安装依赖

```bash
pnpm install
```

### 2. 启动教师端

```bash
pnpm dev:teacher
```

### 3. 启动学生端

```bash
pnpm dev:student
```

### 4. 构建桌面应用

```bash
pnpm build:teacher
pnpm build:student
```

## 各应用内部脚本

教师端与学生端各自都包含以下前端脚本：

```bash
pnpm --filter @xs/teacher dev
pnpm --filter @xs/teacher build
pnpm --filter @xs/teacher preview

pnpm --filter @xs/student dev
pnpm --filter @xs/student build
pnpm --filter @xs/student preview
```

若要直接调用 Tauri CLI，也可以使用：

```bash
pnpm --filter @xs/teacher tauri dev
pnpm --filter @xs/student tauri dev
```

## 业务模块概览

### 教师端

- 考试列表与考试管理
- 题库导入
- 实时监考
- 主观题阅卷
- 成绩报告

### 学生端

- 自动发现考试
- 登录与候考
- 考试作答
- 本地缓存与同步

## 数据库设计

当前教师端数据库迁移已定义在 `apps/teacher/src-tauri/migrations/0001_initial.sql`，核心表如下：

- `exams`：考试主表
- `questions`：题目表
- `students`：考生记录表
- `answer_sheets`：答卷明细表
- `score_summary`：成绩汇总表
- `cheat_logs`：异常事件日志表

数据库结构以迁移文件为准，后续 schema 变更应继续通过 `src-tauri/migrations/` 下的 SQL 脚本维护。

## 共享类型约定

- 前端本地类型默认放在 `src/types/main.ts`
- 跨端共享类型放在 `packages/shared-types/src/`
- 项目内部导入优先使用 `@/` 别名引用 `src/`

## 文档索引

- `doc/PRD.md`：产品需求文档
- `doc/TECH_DESIGN.md`：技术设计文档
- `doc/RESEARCH.md`：需求调研记录
- `doc/project_dependency_topology.md`：项目依赖拓扑图与业务定位入口
- `.github/copilot-instructions.md`：项目工作区规范入口
- `.copilot/AGENTS.md`：历史 AI 规范文档（待迁移清理）

## 开发约定

- 前端组件与服务优先使用 TypeScript 严格类型。
- 前端调用 Tauri IPC 时通过 `services/` 封装，不在页面组件内直接散落调用。
- 数据库 schema 只通过迁移文件管理，不在业务代码中隐式建表。
- 共享协议或跨端类型统一沉淀到 `packages/shared-types`。

## 路线图

### V1.0 MVP

- 考试 CRUD
- Excel 导入题库
- 局域网发现
- 基础 WebSocket 通信
- 断网缓存与同步
- 客观题自动评分

### V1.5

- 主观题阅卷
- 防作弊能力
- 自动录屏

### V2.0

- 教师端集群
- 自动故障转移
- 更完整的数据分析与看板

## 说明

当前仓库已具备教师端、学生端与共享类型包的基础目录结构，并已包含教师端初始数据库迁移。README 会继续随着功能推进补充页面截图、接口说明和部署文档。
