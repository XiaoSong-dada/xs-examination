# AGENTS — xs-examination AI 工作规范

> 本文件定义了 AI 助手（GitHub Copilot / 其他 Agent）在本项目中的行为规范、编码约定与禁止事项。
> 每次开始任务前必须阅读本文件、`memorybank.md` 与 `project-dependency-map.md`。

---

## 0. 启动检查清单

在开始任何编码任务前，必须确认：

- [ ] 已阅读 `.copilot/memorybank.md`，恢复项目上下文
- [ ] 已阅读 `.copilot/project-dependency-map.md`，确认当前模块入口、扇入与扇出
- [ ] 若任务命中已有业务闭环，已阅读 `doc/e2e/` 下对应的最短链路文档
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
| ✅ 使用 SeaORM 2.x（稳定版）+ SQLite | 禁止引入 diesel 或其他 ORM |
| ✅ 使用 sqlx 迁移能力（仅 migrations） | 禁止在仓储层继续手写 SQL CRUD |
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
| 类型定义 | 统一放 `src/types/main.ts` | — |
| 样式入口 | `styles/global.css` | — |

> **导入约定**：所有模块之间的引用在无特殊提示时应使用 `@/` 别名，例如 `import { useTableHeight } from "@/hooks/useTableHeight"`，以避免深层相对路径。

### Rust 文件命名

| 类型 | 命名规范 | 示例 |
|------|---------|------|
| 模块文件 | `snake_case.rs` | `ws_server.rs`, `exam.rs` |
| Tauri 命令函数 | `snake_case` | `create_exam`, `get_questions` |
| 结构体 / 枚举 | `PascalCase` | `ExamStatus`, `WsMessage` |

> **新增约定**：所有用于前后端交互或控制层的纯数据结构（DTO、输入/输出 payload）应放在 `src-tauri/src/schemas/` 文件夹内，使用同名 `*_schema.rs` 文件管理。例如 `question_schema.rs` 存放与题目相关的 DTO。该目录仅包含结构体声明，不包含业务逻辑。
>
> **Rust 补充约定**：新增功能前必须优先检查 `src-tauri/src/utils/` 下是否已存在可复用函数；若逻辑具有跨模块复用价值，应优先沉淀到 `utils/`，避免在 controller、service、network、repo 中重复实现。
>
> **Rust 结构体放置约定**：除 SeaORM 实体模型、数据库实体映射及框架强约束类型外，业务层、控制层、通信层使用的结构体声明统一放在 `src-tauri/src/schemas/` 中管理，禁止将普通 DTO、输入输出参数或中间载荷分散声明在其他层级文件内。
>
> **环境变量约定**：与环境变量相关的默认值、读取入口和全局配置优先集中在 `src-tauri/src/core/setting.rs` 管理；开发前若需查找环境变量定义或读取方式，应先查看该文件，再决定是否补充 `utils/` 中的辅助函数。

### 共享类型包

- 前后端通信协议类型必须定义在 `packages/shared-types/src/` 中
- 教师端前端和学生端前端均从 `@xs/shared-types` 导入，禁止各端自行重复定义协议类型
 - 组件/页面或本端专用的类型（例如组件 Props、本地工具类型）默认在 `src/types/main.ts` 中声明并导出；跨端或需与后端共享的协议/接口类型仍放在 `packages/shared-types/src/`。

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

### Rust ORM 访问规范（SeaORM）

```rust
// ✅ 正确：在 models/ 下维护实体模型，仓储层通过 Entity/ActiveModel 读写
use crate::models::exam::{ActiveModel, Column, Entity as ExamEntity};
use sea_orm::{EntityTrait, QueryOrder, ActiveModelTrait, Set};

let list = ExamEntity::find()
  .order_by_desc(Column::CreatedAt)
  .all(db)
  .await?;

let exam = ActiveModel {
  id: Set(id),
  title: Set(title),
  ..Default::default()
}
.insert(db)
.await?;

// ❌ 错误：在 repo 层继续手写 query/query_as 进行 CRUD
sqlx::query("INSERT INTO exams ...")
```


### 函数注释规范（必须遵守）

**所有函数（包括 React 组件、自定义 Hook、服务层函数、Rust 命令函数、工具函数）上方必须添加注释**，说明三项内容：

1. **函数作用**：这个函数做什么
2. **参数说明**：每个参数的含义与类型
3. **返回值说明**：返回什么，可能的错误情况

#### TypeScript / React：使用 JSDoc 格式

```typescript
/**
 * 创建新考试并持久化到数据库
 *
 * @param data - 考试创建表单数据，包含标题、题目列表、时长等
 * @returns 创建成功后的完整考试对象（含服务器生成的 id 与 created_at）
 * @throws 若 IPC 调用失败则抛出 AppError
 */
export async function createExam(data: CreateExamInput): Promise<Exam> {
  return invoke<Exam>('create_exam', { data })
}

/**
 * 监控 WebSocket 心跳，超时后自动触发重连
 *
 * @param ws - 当前 WebSocket 实例
 * @param timeoutMs - 心跳超时阈值（毫秒），默认 5000
 * @returns 清理函数，组件卸载时调用以取消定时器
 */
export function useHeartbeat(ws: WebSocket | null, timeoutMs = 5000): () => void {
  ...
}

/**
 * 考试状态全局 Store
 *
 * @param set - Zustand 内部状态更新函数
 * @returns ExamState 对象，包含当前考试信息与操作方法
 */
export const useExamStore = create<ExamState>((set) => ({
  ...
}))
```

#### Rust：使用 Rustdoc 格式（`///`）

```rust
/// 创建新考试记录并写入数据库
///
/// # 参数
/// * `state` - Tauri 管理的应用全局状态（含数据库连接池）
/// * `data`  - 前端传入的考试创建参数（标题、时长、题目 ID 列表等）
///
/// # 返回值
/// 成功返回插入后的 `Exam` 对象（含自增 `id`）；
/// 失败返回 `AppError`（数据库错误或参数校验失败）
#[tauri::command]
pub async fn create_exam(
    state: tauri::State<'_, AppState>,
    data: CreateExamInput,
) -> Result<Exam, AppError> {
    ...
}

/// 广播消息到当前房间内所有已连接的学生
///
/// # 参数
/// * `sessions` - 所有在线 WebSocket 会话的共享映射（DashMap）
/// * `msg`      - 待广播的 WS 消息体（已序列化为 JSON 字符串）
///
/// # 返回值
/// 无返回值；发送失败的连接将被静默移除并记录到 tracing warn 日志
pub async fn broadcast(sessions: &SessionMap, msg: &str) {
    ...
}
```

> **注意**：工具函数、私有辅助函数同样需要注释，可适当缩短，但必须覆盖参数与返回值描述。

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
- sqlx 仅用于迁移与连接期初始化，不承担业务 CRUD
- 业务 CRUD 统一使用 SeaORM 实体模型与仓储层
- **若需列出或检查数据库表结构**，优先在 `src-tauri/src/db/migrations/` 目录查找已有迁移定义；若无对应文件，再考虑从模型代码或运行时反查。

---

## 6.5 业务链路文档约定

- `doc/e2e/` 目录中的文档用于沉淀“某个业务闭环的最短真实 e2e 链路”，用于快速定位入口、出口、跨端传输、数据库落点与页面验证面。
- 若任务涉及已有业务链路，开始修改前必须先查找并阅读对应 e2e 文档；若图谱中已有“业务与 e2e 映射”，按映射优先打开对应文件。
- 若本次改动影响了已有链路的入口、出口、关键持久化落点、主查询来源、运行态事件或页面验证面，必须同步更新对应 `doc/e2e/*.md`。
- 若本次改动形成新的独立业务闭环，必须在 `doc/e2e/` 下新增一份最短链路文档，并同步更新 `.copilot/project-dependency-map.md` 中的“业务与 e2e 映射”。
- e2e 文档命名保持 `e2e-minimal-<business>-chain.md` 风格，正文优先回答“入口在哪里、真实出口在哪里、最短调用链是什么、哪些内容不属于这条链路”。

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
