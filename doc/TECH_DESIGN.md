# 局域网分布式在线考试系统 - 技术设计文档 (TECH_DESIGN)

## 1. 技术栈选择

### 1.1 前端（Webview 渲染层）

| 类别 | 技术 | 版本 | 说明 |
|------|------|------|------|
| UI 框架 | React | 18.x | 并发模式、Suspense、自动批量更新 |
| UI 组件库 | Ant Design | 5.x | 表格、表单、弹窗等重度后台组件 |
| 原子化 CSS | Tailwind CSS | 3.x | 布局、间距、响应式微调，与 AntD 互补 |
| 路由 | React Router | 6.x | 基于 Data Router，嵌套路由 |
| 全局状态 | Zustand | 4.x | 轻量、无样板代码，支持中间件持久化 |
| 构建工具 | Vite | 5.x | 极快冷启动，原生 ESM，HMR |
| 语言 | TypeScript | 5.x | 全栈类型安全 |

### 1.2 后端（Tauri Core - Rust 层）

| 类别 | 技术 | 说明 |
|------|------|------|
| 桌面运行时 | Tauri | 2.x，跨平台桌面壳，IPC 通信 |
| 异步运行时 | Tokio | 高性能异步并发，处理 100~200 并发连接 |
| WebSocket 服务 | tokio-tungstenite | 教师端作为 WS Server，学生端作为 WS Client |
| 局域网发现 | mdns-sd | mDNS/Bonjour 协议，学生端自动发现教师端 |
| HTTP 服务 | axum | 教师端内嵌轻量 HTTP 服务，供文件下载、题库分发 |
| 加密 | aes-gcm + sha2 | 本地缓存 AES-256-GCM 加密，请求签名 SHA-256 |
| 序列化 | serde + serde_json | Rust 结构体与 JSON 的双向序列化 |
| 系统控制 | windows-rs / enigo | 热键拦截（学生端防作弊） |
| 录屏 | scrap | 跨平台屏幕捕获 |

### 1.3 数据库

| 场景 | 技术 | 说明 |
|------|------|------|
| 教师端主库 | SQLite（sqlx） | 存储考试配置、题库、学生答卷、成绩 |
| 学生端本地缓冲库 | SQLite（sqlx） | 离线缓存当前题目和答案，断网后可恢复 |
| 文件格式 | WAL 模式 SQLite | 提高并发写入性能 |
### 数据库变更规范

所有模式变更必须通过迁移文件实施。迁移位于
`src-tauri/migrations/`，使用递增编号命名（例如
`0002_add_column_to_questions.sql`）。每个文件应只描述一次
DDL 变动，sqlx 的 `sqlx::migrate!("migrations")` 会在启动时自动
依次执行未应用的脚本，并在 `_sqlx_migrations` 中记录版本。

> 禁止在代码中直接使用 `CREATE TABLE IF NOT EXISTS` 进行结构
> 修改；这会绕过版本管理并导致 schema 不一致。上一阶段的
> schema 建表操作仅存在于迁移脚本，运行时读取这些脚本即可
> 重建数据库。

---

## 2. 项目结构

```
xs-examination/
├── doc/                          # 文档
│   ├── PRD.md
│   └── TECH_DESIGN.md
│
├── apps/
│   ├── teacher/                  # 教师端 Tauri 应用
│   │   ├── src-tauri/            # Rust 后端
│   │   │   ├── src/
│   │   │   │   ├── main.rs           # 入口，Tauri Builder
│   │   │   │   ├── lib.rs            # 公共模块导出
│   │   │   │   ├── commands/         # Tauri IPC 命令（前端可调用）
│   │   │   │   │   ├── exam.rs       # 考试 CRUD
│   │   │   │   │   ├── question.rs   # 题库管理
│   │   │   │   │   ├── student.rs    # 学生状态管理
│   │   │   │   │   └── score.rs      # 成绩批阅与导出
│   │   │   │   ├── network/          # 网络服务
│   │   │   │   │   ├── mdns.rs       # mDNS 服务广播
│   │   │   │   │   ├── ws_server.rs  # WebSocket 服务端
│   │   │   │   │   └── protocol.rs   # 消息协议结构体定义
│   │   │   │   ├── db/               # 数据库访问层
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── migrations/   # SQL 迁移脚本
│   │   │   │   │   └── models.rs     # 数据模型映射
│   │   │   │   ├── crypto.rs         # AES 加密/解密、请求签名
│   │   │   │   └── state.rs          # Tauri AppState（共享状态）
│   │   │   ├── Cargo.toml
│   │   │   └── tauri.conf.json
│   │   │
│   │   └── src/                  # React 前端（教师端 UI）
│   │       ├── main.tsx
│   │       ├── App.tsx
│   │       ├── router/
│   │       │   └── index.tsx         # React Router 路由表
│   │       ├── pages/
│   │       │   ├── Dashboard/        # 首页 - 考试列表
│   │       │   ├── ExamCreate/       # 新建/编辑考试
│   │       │   ├── QuestionImport/   # 题库导入
│   │       │   ├── Monitor/          # 实时监考大屏
│   │       │   ├── Grading/          # 主观题阅卷
│   │       │   └── Report/           # 成绩报告
│   │       ├── components/           # 公共 UI 组件
│   │       │   ├── StudentCard/      # 监考大屏学生状态卡片
│   │       │   ├── QuestionEditor/   # 题目编辑器
│   │       │   └── ExamTimer/        # 倒计时组件
│   │       ├── store/                # Zustand 状态
│   │       │   ├── examStore.ts      # 考试状态
│   │       │   ├── studentStore.ts   # 学生实时状态（监考用）
│   │       │   └── uiStore.ts        # UI 状态（侧边栏收缩等）
│   │       ├── hooks/                # 自定义 React Hooks
│   │       │   ├── useWebSocket.ts   # WS 消息订阅
│   │       │   └── useTauriCommand.ts# IPC 封装
│   │       ├── services/             # 前端服务层（IPC 调用封装）
│   │       │   ├── examService.ts
│   │       │   └── importService.ts
│   │       ├── types/                # TypeScript 类型定义
│   │       │   └── index.ts
│   │       └── styles/
│   │           └── global.css        # Tailwind 入口 + 全局覆盖
│   │
│   └── student/                  # 学生端 Tauri 应用
│       ├── src-tauri/
│       │   └── src/
│       │       ├── main.rs
│       │       ├── commands/
│       │       │   ├── answer.rs     # 答案保存（加密写入本地 DB）
│       │       │   └── sync.rs       # 断网重连后答案同步
│       │       ├── network/
│       │       │   ├── mdns.rs       # mDNS 服务发现（查找教师端）
│       │       │   └── ws_client.rs  # WebSocket 客户端
│       │       ├── anti_cheat/
│       │       │   ├── hotkey.rs     # 热键拦截
│       │       │   ├── fullscreen.rs # 全屏锁定
│       │       │   └── recorder.rs   # 屏幕录制（scrap）
│       │       ├── db/
│       │       │   ├── migrations/
│       │       │   └── models.rs
│       │       └── crypto.rs
│       │
│       └── src/                  # React 前端（学生端 UI）
│           ├── pages/
│           │   ├── Discovery/        # 自动发现考试/局域网搜索页
│           │   ├── Assignment/       # 设备分配确认页（展示本机对应考生与考试）
│           │   ├── WaitingRoom/      # 候考室
│           │   ├── Exam/             # 考试主页面（沉浸模式）
│           │   │   ├── QuestionPanel.tsx  # 题目显示区
│           │   │   ├── ProgressBar.tsx    # 题号矩阵导航
│           │   │   └── SubmitButton.tsx   # 交卷按钮（防误触）
│           │   └── Result/           # 考试结束提示页
│           ├── store/
│           │   ├── examStore.ts      # 考试及题目数据
│           │   ├── answerStore.ts    # 本地答案缓存
│           │   └── networkStore.ts   # 网络连接状态
│           └── hooks/
│               ├── useAutoSync.ts    # 断网重连自动同步
│               └── useAntiCheat.ts   # 防作弊监听
│
├── packages/
│   └── shared-types/             # 前后端共享 TypeScript 类型（monorepo）
│       ├── src/
│       │   ├── exam.ts           # Exam、Question 等核心类型
│       │   ├── protocol.ts       # WS 消息协议类型
│       │   └── index.ts
│       └── package.json
│
├── package.json                  # Monorepo 根（pnpm workspace）
├── pnpm-workspace.yaml
└── turbo.json                    # Turborepo 构建编排（可选）
```

---

## 3. 数据模型

### 3.1 教师端数据库（SQLite）

```sql
-- 考试表
CREATE TABLE exams (
    id          TEXT PRIMARY KEY,       -- UUID
    title       TEXT NOT NULL,          -- 考试名称
    description TEXT,                   -- 考试须知（富文本 HTML）
    start_time  INTEGER,                -- Unix 时间戳（毫秒）
    end_time    INTEGER,
    pass_score  INTEGER NOT NULL,       -- 及格分数
    status      TEXT NOT NULL DEFAULT 'draft',
                      -- draft | published | active | finished | archived
    shuffle_questions INTEGER DEFAULT 0,-- 是否随机题序
    shuffle_options   INTEGER DEFAULT 0,-- 是否随机选项序
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

-- 题目表
CREATE TABLE questions (
    id           TEXT PRIMARY KEY,
    exam_id      TEXT NOT NULL REFERENCES exams(id) ON DELETE CASCADE,
    seq          INTEGER NOT NULL,      -- 题目序号（原始顺序）
    type         TEXT NOT NULL,         -- single | multi | judge | fill | essay
    content      TEXT NOT NULL,         -- 题干（支持 Markdown）
    options      TEXT,                  -- JSON 数组 [{"key":"A","text":"..."}]，客观题用
    answer       TEXT NOT NULL,         -- 标准答案（客观题：选项key；主观题：参考答案）
    score        INTEGER NOT NULL,      -- 分值
    explanation  TEXT                   -- 解析（可选）
);

-- 学生信息拆分说明：
-- 原来的 `students`（每次考试独立注册）拆为两个表：
-- 1) `students`：持久化的学生信息（一条记录表示一个真实学生/学号，合并原 students 中的 student_no/name 字段）
-- 2) `student_exams`：学生参加某次考试的记录（保留原 students 中的 exam_id/ip_addr/status/join_time/submit_time 字段）

-- 持久化学生信息表（智能合并旧 students + 新增时间戳字段）
CREATE TABLE students (
  id          TEXT PRIMARY KEY,       -- UUID
  student_no  TEXT NOT NULL UNIQUE,   -- 学号（全局唯一，代表同一个真实学生）
  name        TEXT NOT NULL,          -- 姓名
  created_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')*1000),
  updated_at  INTEGER NOT NULL DEFAULT (strftime('%s','now')*1000)
);

-- 学生参加考试的记录表（每次参考一条）
CREATE TABLE student_exams (
  id          TEXT PRIMARY KEY,       -- 记录 id（沿用旧 students.id）
  student_id  TEXT NOT NULL REFERENCES students(id),
  exam_id     TEXT NOT NULL REFERENCES exams(id),
  ip_addr     TEXT,
  status      TEXT NOT NULL DEFAULT 'waiting',  -- waiting|active|submitted|offline|forced
  join_time   INTEGER,
  submit_time INTEGER,
  UNIQUE(exam_id, student_id)
);

-- 答卷表
CREATE TABLE answer_sheets (
    id          TEXT PRIMARY KEY,
    student_id  TEXT NOT NULL REFERENCES students(id),
    exam_id     TEXT NOT NULL REFERENCES exams(id),
    question_id TEXT NOT NULL REFERENCES questions(id),
    answer      TEXT,                   -- 学生作答内容
    is_correct  INTEGER,                -- 客观题自动评分结果（0/1/NULL）
    score       INTEGER,                -- 最终得分（主观题手工填入）
    synced_at   INTEGER,                -- 最后同步时间戳
    PRIMARY KEY (student_id, question_id)
);

-- 成绩汇总表（考试结束后触发计算填入）
CREATE TABLE score_summary (
    id          TEXT PRIMARY KEY,
    exam_id     TEXT NOT NULL REFERENCES exams(id),
    student_id  TEXT NOT NULL REFERENCES students(id),
    total_score INTEGER,
    is_passed   INTEGER,
    graded_at   INTEGER,
    UNIQUE(exam_id, student_id)
);

-- 异常事件日志（防作弊告警）
CREATE TABLE cheat_logs (
    id          TEXT PRIMARY KEY,
    student_id  TEXT NOT NULL REFERENCES students(id),
    event_type  TEXT NOT NULL,          -- focus_lost | hotkey_detected | vm_detected | ...
    detail      TEXT,
    occurred_at INTEGER NOT NULL
);
```

### 3.1.1 考试状态语义与流转

教师端 `exams.status` 字段采用如下状态集合：

| 状态码 | 展示文案 | 含义 | 触发条件 |
|------|------|------|------|
| `draft` | 草稿 | 考试配置与编辑阶段。 | 新建考试默认值 |
| `published` | 已发卷 | 试卷已由教师端分发到学生端。 | 点击“分发试卷” |
| `active` | 考试中 | 学生端进入正式作答阶段。 | 点击“开始考试” |
| `finished` | 已结束 | 到达考试结束时间，停止作答与答案继续同步。 | `end_time` 到时自动切换 |
| `archived` | 已归档 | 成绩报告已导出，考试流程归档。 | 导出成绩成功后切换 |

当前版本不使用 `paused` 状态。完整状态图见 [exam_status_flow.md](exam_status_flow.md)。

### 3.2 学生端本地缓冲数据库（SQLite，加密文件，待评审建议）

学生端数据库不建议直接复制教师端主库结构，而应定位为“单机缓存 + 离线恢复 + 出站同步队列”。
教师端仍然是权威数据源，学生端只保留作答过程中必须本地持久化的数据副本。

在当前真实流程下，学生端不是“学生登录后选择考试”，而是“教师先完成设备分配，学生到指定设备就座后等待试卷分发与开考”。
这意味着学生端通常不需要传统登录页；如果确实需要一层交互，也更适合是“设备分配确认页/到场确认页”，用于展示本机已绑定的考试、考生和设备信息，而不是让学生自行输入学号姓名。

基于教师端现有表结构，学生端数据库建议遵循以下原则：

1. 与教师端对齐标识，不对齐职责。学生端保留 exam_id、student_id、student_exam_id、assigned_ip_addr、question_id 等关键标识，便于回传时与教师端的 exams、students、student_exams、devices、answer_sheets 对应，但不在本地重复维护教师端完整业务表。
2. 不在学生端落地标准答案。教师端 questions.answer、score_summary、cheat_logs 属于服务端权威数据，学生端本地只保存考试快照、我的答案、待同步事件，避免泄题和双向冲突。
3. 所有本地写操作都围绕“考试会话”建模。教师端已经把学生信息拆成 students 与 student_exams，学生端也应以一次参考记录为核心，而不是仅用 question_id 做主键，否则无法支持补考、多场考试或缓存残留清理。
4. 同步采用 Outbox 模式。离线期间先入本地队列，网络恢复后按顺序回放，避免把同步状态直接揉进业务表导致状态不清。
5. 迁移粒度保持小而明确。延续教师端做法，每个迁移只做一次 DDL 变化，避免把整个学生端 schema 堆进一个大文件里，便于后续增量演进。

建议的学生端本地库结构如下：

```sql
-- 当前设备收到并激活的考试会话，一次参考一条记录
-- 对齐教师端 student_exams，但只保留学生端运行必需字段
CREATE TABLE exam_sessions (
  id                TEXT PRIMARY KEY,       -- 本地会话 id，可直接复用 teacher.student_exams.id
  exam_id           TEXT NOT NULL,
  assigned_ip_addr  TEXT NOT NULL,
  assigned_device_name TEXT,
  student_id        TEXT NOT NULL,
  student_no        TEXT NOT NULL,
  student_name      TEXT NOT NULL,
  exam_title        TEXT NOT NULL,
  status            TEXT NOT NULL,          -- waiting | active | submitted | finished | expired
  assignment_status TEXT NOT NULL,          -- assigned | paper_distributed | started | submitted
  started_at        INTEGER,
  ends_at           INTEGER,
  paper_version     TEXT,                   -- 试卷版本号/签名，用于判断是否需要重拉快照
  encryption_nonce  BLOB,                   -- 本地加密字段使用的随机参数
  last_synced_at    INTEGER,
  created_at        INTEGER NOT NULL,
  updated_at        INTEGER NOT NULL
);

CREATE INDEX idx_exam_sessions_exam_id ON exam_sessions(exam_id);
CREATE INDEX idx_exam_sessions_assigned_ip_addr ON exam_sessions(assigned_ip_addr);
CREATE INDEX idx_exam_sessions_student_id ON exam_sessions(student_id);

-- 试卷快照，只保存学生端真正需要展示的内容
-- 不保存标准答案、评分结果、教师备注等敏感字段
CREATE TABLE exam_snapshots (
  session_id        TEXT PRIMARY KEY REFERENCES exam_sessions(id) ON DELETE CASCADE,
  exam_meta         BLOB NOT NULL,          -- 加密后的考试元数据 JSON
  questions_payload BLOB NOT NULL,          -- 加密后的题目列表 JSON（不含 answer/explanation）
  downloaded_at     INTEGER NOT NULL,
  expires_at        INTEGER,
  updated_at        INTEGER NOT NULL
);

-- 本地答案表，按考试会话 + 题目维度存储
-- revision 用于解决重复提交和增量同步
CREATE TABLE local_answers (
  id                TEXT PRIMARY KEY,
  session_id        TEXT NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
  question_id       TEXT NOT NULL,
  answer            TEXT,
  answer_blob       BLOB,                   -- 可选：敏感题型采用加密 BLOB 落地
  revision          INTEGER NOT NULL DEFAULT 1,
  sync_status       TEXT NOT NULL DEFAULT 'pending',   -- pending | syncing | synced | failed
  last_synced_at    INTEGER,
  updated_at        INTEGER NOT NULL,
  UNIQUE(session_id, question_id)
);

CREATE INDEX idx_local_answers_session_id ON local_answers(session_id);
CREATE INDEX idx_local_answers_sync_status ON local_answers(sync_status);

-- 出站同步队列表，统一承载答案同步、状态上报、作弊告警、主动交卷等事件
CREATE TABLE sync_outbox (
  id                INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id        TEXT NOT NULL REFERENCES exam_sessions(id) ON DELETE CASCADE,
  event_type        TEXT NOT NULL,          -- ANSWER_SYNC | STATUS_UPDATE | CHEAT_ALERT | SUBMIT
  aggregate_id      TEXT,                   -- 例如 question_id，便于幂等去重
  payload           BLOB NOT NULL,          -- 加密后的消息体
  status            TEXT NOT NULL DEFAULT 'pending',   -- pending | sending | sent | failed
  retry_count       INTEGER NOT NULL DEFAULT 0,
  next_retry_at     INTEGER,
  last_error        TEXT,
  created_at        INTEGER NOT NULL,
  updated_at        INTEGER NOT NULL
);

CREATE INDEX idx_sync_outbox_session_status ON sync_outbox(session_id, status);
CREATE INDEX idx_sync_outbox_next_retry_at ON sync_outbox(next_retry_at);

-- 本地运行时状态表，用于恢复当前题号、剩余时间、最后心跳等轻量状态
-- 这类数据不值得单独建业务表，但放内存里又无法抗崩溃恢复
CREATE TABLE runtime_kv (
  key               TEXT PRIMARY KEY,
  value             TEXT NOT NULL,
  updated_at        INTEGER NOT NULL
);
```

建议说明：

1. 用 exam_sessions 替代当前过于宽泛的 local_exam。这样能显式对齐教师端 student_exams 和 devices，也方便未来支持“一个设备上连续参加多场考试”而不串数据。
2. exam_snapshots 继续保留“整包快照”思路，但拆成 exam_meta 与 questions_payload 两块，更利于局部失效、版本判断和后续加密策略调整。
3. local_answers 不能只用 question_id 做主键。当前文档里的设计一旦出现两场考试题目 id 重复，或者同一设备上有历史缓存，就会直接冲突。
4. sync_queue 建议升级成 sync_outbox，并显式记录 event_type、状态、重试时间、错误信息；否则后续做断网补发、失败重放、幂等去重时会很被动。
5. runtime_kv 是一个很实用的小表。当前题号、最后一次心跳时间、全屏锁定状态、最近一次成功同步时间都可以先放这里，避免为每个轻量状态额外建表。
6. 在当前业务流程里，student_no 和 student_name 不是由学生输入，而是由教师端设备分配结果下发到本机并缓存，用于界面展示和最终交卷校验。

不建议在学生端本地建立的表：

1. 不建议建立教师端的 questions 全量镜像表，除非后续明确需要题目级检索、增量下载或复杂草稿恢复；现阶段用加密快照更简单、安全。
2. 不建议建立本地 score_summary。学生端不负责最终评分，尤其主观题评分结果只能以教师端为准。
3. 不建议建立本地 cheat_logs 历史主表。学生端更适合将作弊事件写入 sync_outbox，必要时另加极简审计缓存，而不是复制教师端告警日志模型。

推荐迁移拆分顺序：

1. 0001_bootstrap.sql：框架初始化，占位迁移。
2. 0002_create_exam_sessions.sql：创建 exam_sessions。
3. 0003_create_exam_snapshots.sql：创建 exam_snapshots。
4. 0004_create_local_answers.sql：创建 local_answers 与相关索引。
5. 0005_create_sync_outbox.sql：创建 sync_outbox 与相关索引。
6. 0006_create_runtime_kv.sql：创建 runtime_kv。

如果后续确认学生端确实只允许“同一时刻一个活动考试”，仍然建议保留 session_id 这层抽象。它会让本地数据清理、重连恢复、故障排查和未来扩展都简单很多。

从交互角度看，学生端建议采用以下流程：设备启动 → 自动发现/接收教师端分配 → 展示 Assignment 页面确认本机绑定的考试与考生 → 接收试卷快照 → 进入 WaitingRoom → 教师点击开始考试后进入 Exam 页面。这样比“登录 -> 选考试”更符合当前业务真相。

### 3.3 WebSocket 消息协议（TypeScript 类型）

```typescript
// packages/shared-types/src/protocol.ts

export type MessageType =
  | 'EXAM_START'        // 教师端 → 广播：考试开始
  | 'EXAM_PAUSE'        // 教师端 → 广播：考试暂停
  | 'EXAM_END'          // 教师端 → 广播：考试结束
  | 'FORCE_SUBMIT'      // 教师端 → 单播：强制交卷
  | 'HEARTBEAT'         // 双向：心跳保活
  | 'ANSWER_SYNC'       // 学生端 → 服务：答案同步
  | 'SUBMIT'            // 学生端 → 服务：主动交卷
  | 'STATUS_UPDATE'     // 学生端 → 服务：状态上报
  | 'CHEAT_ALERT'       // 学生端 → 服务：防作弊告警

export interface WsMessage<T = unknown> {
  type: MessageType
  timestamp: number           // Unix ms，用于防重放
  signature: string           // HMAC-SHA256 签名
  payload: T
}

export interface AnswerSyncPayload {
  examId: string
  studentId: string
  answers: { questionId: string; answer: string }[]
}

export interface StatusUpdatePayload {
  studentId: string
  progress: number            // 已答题数
  currentQuestion: number     // 当前题号
}
```

---

## 4. 关键技术点

### 4.1 局域网自动发现（mDNS）

**问题**：学生端打开后需零配置找到教师端，不能依赖手动输入 IP。

**方案**：
- 教师端启动考试服务时，通过 `mdns-sd` crate 广播服务记录：`_xs-exam._tcp.local.`，携带服务端口和考试 ID。
- 学生端启动后执行 mDNS Browse，监听该服务类型，自动获取教师端 IP 和端口。
- 同一局域网内若有多个教师端（集群场景），学生端列表展示所有可用考试供选择。

```rust
// 教师端广播示例（Rust）
let mdns = ServiceDaemon::new()?;
let service_info = ServiceInfo::new(
    "_xs-exam._tcp.local.",
    "teacher-node-1",
    &hostname,
    ip_addr,
    port,
    &[("exam_id", &exam_id)],
)?;
mdns.register(service_info)?;
```

---

### 4.2 断网本地缓存与无感重连

**问题**：局域网波动时学生作答不能中断，网络恢复后数据不能丢失。

**方案**：三层容错设计

1. **答案实时落盘**：每次学生切换题目或修改答案，立即写入本地 SQLite `local_answers` 表（同步写入，非异步防丢失）。
2. **同步队列**：WS 连接断开时，待同步数据压入 `sync_queue`，并标记 `synced=0`。
3. **后台重连器**：Tokio 后台任务每 3 秒尝试重连，重连成功后将队列中所有未同步记录批量发送，并更新 `synced=1`。

```
学生答题 → 写入 local_answers
         → （若 WS 连接正常）直接发送 ANSWER_SYNC
         → （若 WS 断开）写入 sync_queue
                         ↓
               后台重连任务（3s 轮询）
                         ↓ 重连成功
               批量发送 sync_queue → 清空队列
```

---

### 4.3 客观题自动评分

**问题**：交卷后需快速完成批量客观题评分。

**方案**：
- 题库导入时，标准答案加密存储在教师端，**不下发到学生端**（避免被截获）。
- 学生交卷时，答卷上传至教师端，由 Rust 层在内存中进行批量比对（无 IO 瓶颈）。
- 填空题支持配置"精确匹配"或"包含匹配"两种评分策略。
- 评分完成后写入 `answer_sheets.is_correct` 和 `score_summary`，触发前端实时刷新。

---

### 4.4 本地数据 AES 加密存储

**问题**：学生机器上的题库和答案不能被直接读取修改。

**方案**：
- 使用 `aes-gcm` crate（AES-256-GCM）对 SQLite 数据库文件整体加密（配合 SQLCipher），或对敏感字段单独加密存储为 BLOB。
- 加密密钥 = `HMAC(考试ID + 学生ID + 设备指纹)`，由教师端在设备分配确认或试卷下发阶段下发密钥种子，本地派生实际密钥，避免密钥明文传输。
- 学生端进程退出后，内存中的明文密钥即清除。

---

### 4.5 防作弊机制实现

**问题**：在 Windows 环境下限制学生离开考试窗口。

**方案**（Windows 优先）：

| 功能 | 实现方式 |
|------|---------|
| 全屏锁定 | Tauri `window.set_fullscreen(true)` + 禁用窗口最小化按钮 |
| Alt+Tab 拦截 | `windows-rs` 注册低级键盘钩子（`SetWindowsHookEx` LowLevel Keyboard） |
| Win 键屏蔽 | 同上，拦截 VK_LWIN / VK_RWIN |
| 焦点丢失检测 | Tauri `on_window_event` 监听 `Focused(false)`，触发 CHEAT_ALERT 上报 |
| 静默录屏 | `scrap` crate 每 N 秒截帧，编码为 H.264 视频切片（ffmpeg-sys），考后上传 |

---

### 4.6 大并发 WebSocket 管理（教师端）

**问题**：100~200 个学生同时连接，心跳和消息处理不能阻塞。

**方案**：
- 使用 Tokio 的 `task::spawn` 为每个学生 WS 连接独立分配异步任务。
- 用 `DashMap<StudentId, WsSender>` 管理所有活跃连接句柄，支持无锁并发读写。
- 心跳检测：每个连接独立 30s 超时 Timer，超时未收到 HEARTBEAT 则标记学生离线，通知前端更新监考大屏。
- 广播消息（如考试开始/暂停）使用 `tokio::sync::broadcast` channel，O(1) 扇出到所有连接任务。

```
┌─────────────────────────────────┐
│      Tokio Runtime               │
│                                  │
│  WS Listener Task                │
│         │ accept()               │
│         ▼                        │
│  ┌─────────────────────────┐     │
│  │  Per-Student Task (×N)  │     │
│  │  - recv / send loop     │     │
│  │  - heartbeat timeout    │     │
│  │  - write to DB          │     │
│  └─────────────────────────┘     │
│                                  │
│  broadcast::Sender ─────────────►│ → 所有 Per-Student Task
└─────────────────────────────────┘
```

---

### 4.7 请求签名防重放攻击

**问题**：恶意学生可能抓包重放交卷请求，或提前获取试题。

**方案**：
- 所有 WS 消息附带 `timestamp`（Unix ms）和 `signature`（HMAC-SHA256）。
- 教师端维护一个 最近 5 分钟已处理消息签名的滑动窗口集合（HashSet），收到重复签名直接丢弃。
- 密钥在设备绑定后的握手阶段通过一次性临时 Token 协商，每次考试会话独立。

---

### 4.8 Excel 题库导入与验证

**问题**：题库格式错误需给出精确的行级错误提示。

**方案**：
- 使用 `calamine` crate（Rust）解析 Excel/.xlsx 文件。
- 逐行验证：必填字段缺失、题型枚举合法性、选项与答案一致性等。
- 错误以结构化列表返回前端，Ant Design `Table` 组件高亮显示具体行列错误。
- 提供模板文件下载，模板包含示例行和字段说明注释。

**Excel 模板列定义**：

| 列名 | 说明 | 示例 |
|------|------|------|
| type | 题型 | single / multi / judge / fill / essay |
| content | 题干 | 以下哪个是... |
| option_a ~ option_e | 选项文本（客观题） | 选项内容 |
| answer | 标准答案 | A 或 AB 或 对 或 填空答案 |
| score | 分值 | 5 |
| explanation | 解析（可选） | 因为... |

---

## 5. 前端架构关键设计

### 5.1 Zustand Store 划分（教师端）

```typescript
// 考试状态（持久化到 localStorage）
interface ExamStore {
  currentExam: Exam | null
  examStatus: ExamStatus
  setExam: (exam: Exam) => void
  updateStatus: (status: ExamStatus) => void
}

// 实时监考状态（不持久化，由 WS 实时更新）
interface StudentStore {
  students: Map<string, StudentRealTimeState>
  updateStudent: (id: string, state: Partial<StudentRealTimeState>) => void
  forceSubmit: (studentId: string) => Promise<void>
}

// StudentRealTimeState
interface StudentRealTimeState {
  studentNo: string
  name: string
  status: 'waiting' | 'active' | 'submitted' | 'offline'
  progress: number        // 已答题数
  lastHeartbeat: number   // 时间戳
  alertCount: number      // 防作弊告警次数
}
```

### 5.2 Tailwind + Ant Design 协同规范

- **Ant Design**：负责复杂交互组件（Table、Form、Modal、DatePicker 等）。
- **Tailwind CSS**：负责页面布局、间距、颜色微调、自定义卡片样式。
- 在 `tailwind.config.ts` 中配置 `content` 路径，并设置 `corePlugins.preflight: false` 避免与 Ant Design 的 CSS Reset 冲突。

```typescript
// tailwind.config.ts
export default {
  content: ['./src/**/*.{tsx,ts}'],
  corePlugins: {
    preflight: false,   // 关闭 Tailwind reset，避免与 AntD 冲突
  },
  theme: {
    extend: {
      colors: {
        primary: '#1677ff',   // 与 AntD 主色保持一致
      },
    },
  },
}
```

---

## 6. 开发阶段规划对应技术实现

| 版本 | 功能 | 重点技术实现 |
|------|------|-------------|
| V1.0 MVP | 创建考试、Excel 导入、设备分配、试卷分发、学生就座答题、客观题自动评分、成绩导出 | mDNS 发现、WS 基础通信、本地 SQLite、断网缓存、客观题评分引擎 |
| V1.5 | 主观题阅卷、防作弊全屏+热键拦截、录屏上传 | windows-rs 键盘钩子、scrap 录屏、阅卷工作流 API |
| V2.0 | 教师端集群高可用、故障自动转移 | 基于 Raft 的状态同步或 SQLite WAL 共享 + Leader 选举 |
