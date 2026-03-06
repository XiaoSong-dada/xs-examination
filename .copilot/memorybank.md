# Memory Bank — xs-examination

> 本文件是 AI 助手的项目记忆库，每次开始新对话时应优先阅读此文件以恢复上下文。

---

## 1. 项目身份

| 属性 | 值 |
|------|-----|
| 项目名称 | 局域网分布式在线考试系统（xs-examination） |
| 仓库地址 | https://github.com/XiaoSong-dada/xs-examination |
| 工作目录 | `E:\code\xs-examination` |
| 当前阶段 | 规划 / 架构设计阶段（文档已完成，代码未启动） |
| 目标版本 | V1.0 MVP → V1.5 → V2.0 |

---

## 2. 产品核心定位

- **场景**：局域网（无公网）环境下的桌面端考试系统
- **两类用户**：
  - **教师端**：创建考试、导入题库、实时监考、批阅主观题、导出成绩
  - **学生端**：自动发现考试、登录候考、沉浸式答题、断网无感续答

---

## 3. 已确定技术栈（不可随意更改）

### 前端（两端通用）

| 技术 | 版本 | 用途 |
|------|------|------|
| React | 18.x | UI 框架 |
| Ant Design | 5.x | 组件库（表格、表单、弹窗等） |
| Tailwind CSS | 3.x | 原子化布局/间距/颜色微调 |
| React Router | 6.x | 客户端路由（Data Router） |
| Zustand | 4.x | 全局状态管理 |
| Vite | 5.x | 构建工具 |
| TypeScript | 5.x | 全栈类型安全 |

### 前端类型声明位置

- 当 React 端需要声明类型（组件 Props、局部接口、辅助类型等）时，默认在 `src/types/main.ts` 中声明并导出；仅当类型需跨端或被后端共享时，才放在 `packages/shared-types/src/`。

### 导入路径约定

- 组件或模块之间引用时，若无特殊说明，统一使用 `@` 别名指向 `src/` 目录，例如 `import { useFoo } from "@/hooks/useFoo"`；避免大量相对路径 `../../`。


### 后端（Tauri Rust 层）

| 技术 | 用途 |
|------|------|
| Tauri 2.x | 桌面运行时 + IPC 通信 |
| Tokio | 异步运行时（高并发 100~200 连接） |
| SeaORM 2.x（稳定版） | Rust 业务 ORM（实体模型 + 仓储层） |
| tokio-tungstenite | WebSocket 服务端 / 客户端 |
| mdns-sd | 局域网 mDNS 自动发现（`_xs-exam._tcp.local.`） |
| axum | 教师端内嵌 HTTP 服务 |
| sqlx + SQLite | 数据持久化与迁移执行（migrations） |
| aes-gcm + sha2 | AES-256-GCM 加密 + HMAC-SHA256 签名 |
| serde / serde_json | Rust ↔ JSON 序列化 |
| calamine | 解析 Excel/.xlsx 题库文件 |
| scrap | 跨平台屏幕捕获（录屏） |
| windows-rs | Windows 低级键盘钩子（防作弊） |

### 数据库

- **教师端主库**：SQLite（WAL 模式），存考试、题库、答卷、成绩
- **学生端本地库**：SQLite（加密），存题目快照、答案缓存、同步队列

---

## 4. 项目结构（已定稿）

```
xs-examination/
├── doc/                        # 文档（PRD.md, TECH_DESIGN.md）
├── apps/
│   ├── teacher/                # 教师端 Tauri 应用
│   │   ├── src-tauri/          # Rust 后端
│   │   └── src/                # React 前端
│   └── student/                # 学生端 Tauri 应用
│       ├── src-tauri/          # Rust 后端
│       └── src/                # React 前端
├── packages/
│   └── shared-types/           # 共享 TypeScript 类型（Monorepo）
├── package.json                # pnpm workspace 根
├── pnpm-workspace.yaml
└── turbo.json                  # Turborepo 构建编排
```

---

## 5. 核心数据模型速查

### 教师端数据库表

| 表名 | 说明 |
|------|------|
| `exams` | 考试元数据（id, title, status, start/end_time, pass_score…） |

---

## 数据库迁移规范（提醒）

所有模式变更应通过在 `src-tauri/migrations/` 下创建新的 SQL 脚本实现，
例如 `0002_add_column.sql`。数据库的表结构默认由该目录中的迁移文件决定，
运行时不要手动创建或修改表。切勿直接在运行时的 Rust 代码里写
`CREATE TABLE` 或 `ALTER TABLE`，这会绕过版本控制。sqlx 会在应用
启动时自动执行未运行的迁移并记录历史。
业务读写查询默认使用 SeaORM 实体模型（`src-tauri/src/models/`）和仓储层，
不再在 repo 中直接写 SQL CRUD。
| `questions` | 题库（type: single/multi/judge/fill/essay, options JSON, answer, score） |
| `students` | 考生注册（student_no, name, ip_addr, status） |
| `answer_sheets` | 学生答卷（answer, is_correct, score） |
| `score_summary` | 成绩汇总（total_score, is_passed） |
| `cheat_logs` | 防作弊告警日志（event_type, occurred_at） |

### 学生端本地表

| 表名 | 说明 |
|------|------|
| `local_exam` | 加密考试快照（AES-256-GCM BLOB） |
| `local_answers` | 本地答案缓存（synced 标志位） |
| `sync_queue` | 断网时积攒的待上传消息队列 |

### WS 消息类型（protocol）

```
EXAM_START / EXAM_PAUSE / EXAM_END        # 教师 → 广播
FORCE_SUBMIT                              # 教师 → 单播
HEARTBEAT                                 # 双向
ANSWER_SYNC / SUBMIT / STATUS_UPDATE      # 学生 → 教师
CHEAT_ALERT                               # 学生 → 教师（告警）
```

---

## 6. 三大版本里程碑

| 版本 | 核心目标 | 关键特性 |
|------|---------|---------|
| **V1.0 MVP** | 跑通核心考试闭环 | 创建考试、Excel 导入、mDNS 发现、WS 通信、断网缓存、客观题自动评分、成绩导出 |
| **V1.5** | 安全性与严肃性 | 主观题阅卷、防作弊全屏+热键拦截、录屏上传 |
| **V2.0** | 高可用分布式 | 教师端集群 Raft 状态同步、自动故障转移 |

---

## 7. 关键技术决策备忘

| 决策点 | 结论 | 原因 |
|--------|------|------|
| Monorepo 方案 | pnpm workspace + Turborepo | 教师/学生端共享类型，统一构建 |
| 局域网发现 | mDNS（mdns-sd），服务名 `_xs-exam._tcp.local.` | 零配置，无需手动输入 IP |
| 通信协议 | WebSocket（长连接） | 实时心跳、状态推送、双向通信 |
| 离线容错 | 三层：实时落盘 → sync_queue → 后台重连（3s 轮询） | 保证断网数据不丢失 |
| 答案加密密钥派生 | `HMAC(考试ID + 学生ID + 设备指纹)` | 避免密钥明文传输 |
| 大并发管理 | `DashMap` + `tokio::sync::broadcast` | 无锁并发 + O(1) 广播扇出 |
| Tailwind vs AntD 冲突 | `preflight: false` | 避免 CSS Reset 互相覆盖 |
| 题库标准答案位置 | 仅存教师端，**不下发学生端** | 防止学生截包获取答案 |

---

## 8. 文档索引

| 文档 | 路径 | 说明 |
|------|------|------|
| 产品需求文档 | `doc/PRD.md` | 功能列表、用户故事、优先级、UI 设计要求 |
| 技术设计文档 | `doc/TECH_DESIGN.md` | 技术栈、项目结构、数据模型、关键技术点 |
| 记忆库 | `.copilot/memorybank.md` | 本文件，AI 上下文快速恢复 |
| 智能体规范 | `.copilot/AGENTS.md` | AI 编码规范与行为约定 |
