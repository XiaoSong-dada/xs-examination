# AGENTS — xs-examination AI 工作规范

> 本文件定义了 AI 助手（GitHub Copilot / 其他 Agent）在本项目中的行为规范、编码约定与禁止事项。
> 每次开始任务前必须阅读本文件与 `memorybank.md`。

---

## 0. 启动检查清单

在开始任何编码任务前，必须确认：

- [ ] 已阅读 `.copilot/memorybank.md`，恢复项目上下文
- [ ] 已确认当前工作的是 `teacher/` 端还是 `student/` 端
- [ ] 已确认当前修改属于哪个版本里程碑（V1.0 / V1.5 / V2.0）
- [ ] 未修改已定稿的技术栈选型（见下方禁止事项）

---

## 1. 技术栈约束（不可更改）

以下技术选型已定稿，**禁止替换或引入替代方案**：

### 前端（React 端）

| 约束 | 说明 |
|------|------|
| ✅ 使用 React 18 函数组件 + Hooks | 禁止使用 Class Component |
| ✅ 使用 Ant Design 5 组件 | 禁止引入其他 UI 库（如 MUI、Chakra） |
| ✅ 使用 Tailwind CSS 做布局与间距 | 禁止写内联 style 处理布局，使用 Tailwind 类名 |
| ✅ 使用 Zustand 管理全局状态 | 禁止引入 Redux、MobX、Context API 管理全局状态 |
| ✅ 使用 React Router 6（Data Router） | 禁止使用 Next.js 路由、Hash Router |
| ✅ 使用 TypeScript，严格类型 | 禁止使用 `any`，除非有明确注释说明理由 |
| ✅ 构建工具为 Vite | 禁止引入 Webpack、CRA |

### 后端（Rust / Tauri 端）

| 约束 | 说明 |
|------|------|
| ✅ 使用 Tauri 2.x IPC（`#[tauri::command]`） | 禁止直接暴露 HTTP 接口给前端调用 |
| ✅ 使用 Tokio 异步运行时 | 禁止使用 `std::thread::spawn` 做 IO 密集任务 |
| ✅ 使用 sqlx + SQLite | 禁止引入 diesel 或其他 ORM |
| ✅ 使用 serde/serde_json 序列化 | 禁止手动拼接 JSON 字符串 |
| ✅ 错误处理使用 `thiserror` + `anyhow` | 禁止直接 `.unwrap()` 或 `.expect()` 在生产代码路径 |

---

## 2. 目录与文件规范

### 前端文件命名

| 类型 | 命名规范 | 示例 |
|------|---------|------|
| 页面组件 | `PascalCase/index.tsx` | `pages/Dashboard/index.tsx` |
| 复用组件 | `PascalCase/index.tsx` | `components/StudentCard/index.tsx` |
| Zustand Store | `camelCase + Store.ts` | `store/examStore.ts` |
| 自定义 Hook | `use + PascalCase.ts` | `hooks/useWebSocket.ts` |
| 服务层（IPC 封装） | `camelCase + Service.ts` | `services/examService.ts` |
| 类型定义 | 统一放 `types/index.ts` | — |
| 样式入口 | `styles/global.css` | — |

### Rust 文件命名

| 类型 | 命名规范 | 示例 |
|------|---------|------|
| 模块文件 | `snake_case.rs` | `ws_server.rs`, `exam.rs` |
| Tauri 命令函数 | `snake_case` | `create_exam`, `get_questions` |
| 结构体 / 枚举 | `PascalCase` | `ExamStatus`, `WsMessage` |

### 共享类型包

- 前后端通信协议类型必须定义在 `packages/shared-types/src/` 中
- 教师端前端和学生端前端均从 `@xs/shared-types` 导入，禁止各端自行重复定义协议类型

---

## 3. 编码规范

### TypeScript / React

```typescript
// ✅ 正确：使用具名导出
export function ExamCard({ exam }: ExamCardProps) { ... }

// ❌ 错误：使用默认导出（共享组件）
export default function ExamCard() { ... }

// ✅ 正确：Zustand store 使用 slice 模式，带类型
interface ExamState {
  currentExam: Exam | null
  setExam: (exam: Exam) => void
}
export const useExamStore = create<ExamState>((set) => ({
  currentExam: null,
  setExam: (exam) => set({ currentExam: exam }),
}))

// ✅ 正确：Tauri IPC 调用统一封装在 services/
// services/examService.ts
import { invoke } from '@tauri-apps/api/core'
export const createExam = (data: CreateExamInput) =>
  invoke<Exam>('create_exam', { data })

// ❌ 错误：在组件内直接调用 invoke
invoke('create_exam', { data })  // 禁止
```

### Tailwind + Ant Design 协同

```tsx
// ✅ 正确：AntD 组件 + Tailwind 布局类名
<Card className="p-4 rounded-xl shadow-md">
  <Table columns={columns} dataSource={data} />
</Card>

// ❌ 错误：使用内联 style 做布局
<Card style={{ padding: '16px', borderRadius: '12px' }}>

// ✅ 正确：颜色使用 AntD Token（Design Token），通过 ConfigProvider 统一
// 小范围微调使用 Tailwind 已定义的 primary 色
<span className="text-primary">在线</span>
```

### Rust 异步与错误处理

```rust
// ✅ 正确：Tauri 命令返回 Result，错误使用自定义 Error 类型
#[tauri::command]
async fn create_exam(
    state: tauri::State<'_, AppState>,
    data: CreateExamInput,
) -> Result<Exam, AppError> {
    let db = state.db.lock().await;
    exam_repo::create(&db, data).await.map_err(AppError::from)
}

// ❌ 错误：直接 unwrap
let db = state.db.lock().unwrap();  // 禁止

// ✅ 正确：数据库操作使用 sqlx query! 宏（编译期检查）
let exam = sqlx::query_as!(Exam, "SELECT * FROM exams WHERE id = ?", id)
    .fetch_one(&pool)
    .await?;
```

---

## 4. 数据安全约定

| 规则 | 说明 |
|------|------|
| 标准答案不出教师端 | `questions.answer` 字段**绝对禁止**通过任何 WS/HTTP 接口发送到学生端 |
| 本地缓存必须加密 | 学生端 SQLite 文件使用 AES-256-GCM，明文密钥不落磁盘 |
| 加密密钥派生规则 | `HMAC-SHA256(考试ID \|\| 学生ID \|\| 设备指纹)`，每次考试会话独立 |
| WS 消息必须签名 | 所有消息体携带 `timestamp`（Unix ms）+ `signature`（HMAC-SHA256） |
| 防重放窗口 | 教师端维护 5 分钟滑动窗口 HashSet，重复签名直接丢弃 |

---

## 5. WS 消息协议约定

- 所有消息格式严格遵循 `packages/shared-types/src/protocol.ts` 中的 `WsMessage<T>` 泛型结构
- 新增消息类型必须先在 `shared-types` 中声明 `MessageType` 枚举值，再实现处理逻辑
- **消息类型命名规则**：`动作_主体`，全大写下划线分隔，例：`EXAM_START`、`ANSWER_SYNC`

---

## 6. 数据库变更规范

- 所有 Schema 变更必须通过 `src-tauri/src/db/migrations/` 下的 `.sql` 迁移文件实现
- 迁移文件命名：`{版本号}_{功能描述}.sql`，例：`0001_create_exams.sql`
- 禁止在代码中手写 `CREATE TABLE IF NOT EXISTS` 做隐式建表
- sqlx 使用 `query!` / `query_as!` 宏，保证编译期 SQL 正确性

---

## 7. 防作弊功能约定（学生端）

- 防作弊相关代码统一放在 `src-tauri/src/anti_cheat/` 模块，禁止散落在其他模块
- 热键拦截钩子注册/注销逻辑必须与考试生命周期绑定：
  - `EXAM_START` 消息 → 注册钩子
  - `EXAM_END` / `SUBMIT` / 应用退出 → 必须注销钩子（防止系统级钩子泄漏）
- 焦点丢失、热键触发等事件必须异步上报 `CHEAT_ALERT`，不得阻塞答题主流程

---

## 8. 版本开发范围约定

开发新功能时必须对照以下范围，不得超前实现高版本特性：

| 版本 | 允许开发的特性 | 不允许提前引入的特性 |
|------|--------------|-----------------|
| **V1.0 MVP** | 考试 CRUD、Excel 导入、mDNS 发现、WS 基础通信、断网缓存、客观题自动评分、成绩导出 | 主观题阅卷、防作弊、录屏、集群 |
| **V1.5** | 在 V1.0 基础上加入主观题阅卷、全屏+热键拦截、录屏切片上传 | Raft 集群、故障转移 |
| **V2.0** | 教师端集群 Raft 状态同步、自动 failover、考情分析看板 | — |

---

## 9. Git 提交规范

提交信息格式：`<type>(<scope>): <描述>`

| type | 适用场景 |
|------|---------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `refactor` | 代码重构（不涉及功能变化） |
| `docs` | 文档变更 |
| `chore` | 构建脚本、依赖更新 |
| `test` | 添加/修改测试 |
| `style` | 代码格式调整（不影响逻辑） |

**scope 约定**：`teacher-fe`、`teacher-rs`、`student-fe`、`student-rs`、`shared-types`、`docs`

示例：
```
feat(teacher-fe): 实现监考大屏学生状态实时更新
fix(student-rs): 修复断网重连后 sync_queue 未清空的问题
docs: 更新 TECH_DESIGN.md 数据模型章节
```

---

## 10. 禁止事项（红线）

以下行为**严格禁止**，违反将导致代码被驳回：

1. ❌ 在学生端任何接口/消息中暴露题目标准答案
2. ❌ 在前端组件中直接调用 `invoke`（必须通过 `services/` 封装）
3. ❌ 在 Rust 生产代码路径中使用 `.unwrap()` / `.expect()`
4. ❌ 引入已定稿技术栈之外的 UI 库、状态管理库、路由库
5. ❌ 在迁移文件之外修改数据库 Schema
6. ❌ 在考试结束/退出后遗漏注销操作系统级键盘钩子
7. ❌ 明文存储任何加密密钥或学生答案到磁盘
8. ❌ 跨版本里程碑超前实现功能（如在 V1.0 阶段写录屏代码）
