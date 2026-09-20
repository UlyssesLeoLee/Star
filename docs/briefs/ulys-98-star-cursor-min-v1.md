# ULYS-98 Star Cursor 极简版 — 派工 brief (2026-09-20 14:55 JST, per D-Boy 批准选项 A)

> **派工模型**: 单 agent 串行 (W1~W4 同 agent)；W4 收口可切 Codex (per守门 #22 跨 agent 实证 + W4 Agent 模式 multi-step task 适配)
>
> **worktree**: `/d/Star/.worktrees/wt-cursor-min-v1` (branch `agent/cursor-min/wt-cursor-min-v1`, from `dev` HEAD `793cfcec`)
>
> **author 闸门**: ULYS-98 关联 PR 由 `Mavis (ULYS-2 coordinator)` 永久代签 (per #14 v4); commit author 字段需写 Hermes/Mavis 而非 c557dae5 原始
>
> **守门**: #1 v25 单 crate + #7 `unsafe_code = "forbid"` + #10 author + #11 缺标比错标 + #13 W/T/M 100% + #22 跨仓独立性 (单 Star 仓)

---

## 已实读现状 (2026-09-20)

| 模块 | LOC | 现状 |
|---|---|---|
| `crates/agent-bridge` | 624 | `AgentRuntime` Trait 8 方法 + `InMemoryAgentRuntime` + `TokenUsage` + 14 状态机实装 |
| `crates/agent-domain` | 2552 | Agent 一级领域对象 + 14 状态机 + 12 强制点 + INV-AGT-01~10 + 支持 Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local |
| `crates/domain-llm` | 212 | **v0.0.1 stub**: 只有 `LlmProvider` trait (init/shutdown/health_check) + `LlmProviderRegistry` (in-memory HashMap) + 4 unit tests; **无任何 LLM 推理路径** |
| `crates/action-engine` | 2087 | `ActionEngine` + 18 Action (Safe 6 / Warning 5 / Destructive 7) + Idempotency 24h + RBAC 5 角色 + Retry 3 档实装 |
| `crates/canvas-collab` | 372 | 阶段 1 骨架, 0 业务方法 |
| `crates/bff` | 5450 | 已有 HTTP routes, 需加 chat/completion/composer/agent endpoints |
| `frontend/` (Next.js) | — | **无 Monaco / CodeMirror / LSP client**; **无 /agent / /ai / /chat / /compose / /edit 路由** |

---

## W1 (2026-09-21 ~ 2026-09-27): 基础设施 — 编辑器 + Chat RPC 框架

**目标**: 搭建 Monaco editor + Chat panel + 后端 Chat RPC stub; 零 LLM 推理, 仅消息传递

### Sub-task 1.1: frontend 接入 monaco editor

**文件**:
- 新建 `frontend/components/editor/MonacoEditor.tsx`
- 新建 `frontend/components/editor/index.ts`
- 修改 `frontend/app/(worktree)/page.tsx` 嵌入 MonacoEditor
- 修改 `frontend/package.json` 加 `@monaco-editor/react` `^4.6.0`

**实现要点**:
- 使用 `@monaco-editor/react` (React wrapper)
- Props: `value` / `language` / `onChange` / `path` (从 worktree file path 推断 language)
- 不做 AI 补全 (W2)

**验收**:
- `cd frontend && npm install && npm run build` 0 错误
- 浏览器打开 `/<worktree>/<path>` 路由 → 显示 Monaco + 代码高亮
- 1 unit test (`MonacoEditor.test.tsx` 渲染 + change event)

### Sub-task 1.2: frontend Chat panel UI

**文件**:
- 新建 `frontend/components/chat/ChatPanel.tsx` (右侧抽屉)
- 新建 `frontend/components/chat/MessageList.tsx`
- 新建 `frontend/components/chat/InputBox.tsx`
- 新建 `frontend/app/(worktree)/chat/page.tsx` 路由

**实现要点**:
- 抽屉组件, 监听 `Ctrl+L` 打开/关闭
- `MessageList` 显示 user / assistant 消息气泡
- `InputBox` 多行 textarea + Enter 发送
- 默认 stub: 用户发消息 → 后端返 "AI 暂未接入" (W2 接真 LLM)

**验收**:
- `npm run build` 0 错误
- 浏览器测试: `Ctrl+L` 打开 → 发空消息 → 看到 stub 回复
- 1 unit test (`ChatPanel.test.tsx`)

### Sub-task 1.3: backend Chat RPC stub

**文件**:
- 新建 `crates/bff/src/routes/chat.rs`
- 修改 `crates/bff/src/lib.rs` 注册 routes

**接口**:
```rust
// POST /v1/chat/send
#[derive(Deserialize)]
struct ChatSendReq {
    session_id: String,        // UUID, 前端生成
    user_id: String,           // from JWT
    content: String,           // 用户消息
}

#[derive(Serialize)]
struct ChatSendResp {
    session_id: String,
    message_id: String,         // 服务端 UUID
    assistant_content: String,  // stub: "AI 暂未接入" (W2 改真 LLM)
    created_at: DateTime<Utc>,
}

// GET /v1/chat/messages?session_id=...
#[derive(Serialize)]
struct ChatMessagesResp {
    session_id: String,
    messages: Vec<ChatMessage>,
}
```

**验收**:
- `cargo test -p bff --lib chat` 3 unit tests pass
- `cargo check -p bff --lib -j 4` 0 错误
- 手动 curl 测试 POST + GET 走通

### Sub-task 1.4: backend LlmProvider trait 扩展 chat_completion stub

**文件**:
- 修改 `crates/domain-llm/src/lib.rs` 加 `chat_completion` + `stream_completion` trait 方法
- 新建 `crates/domain-llm/src/chat.rs` (request/response types)

**接口**:
```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn init(&self) -> Result<(), LlmProviderRegistryError>;
    async fn shutdown(&self) -> Result<(), LlmProviderRegistryError>;
    async fn health_check(&self) -> Result<LlmProviderRegistryHealth, LlmProviderRegistryError>;

    // W1 新增 (stub return Err NotImplemented)
    async fn chat_completion(&self, req: ChatRequest) -> Result<ChatResponse, LlmProviderRegistryError>;
    async fn stream_completion(&self, req: ChatRequest) -> Result<BoxStream<'_, Result<ChatChunk, LlmProviderRegistryError>>, LlmProviderRegistryError>;
}
```

**验收**:
- `cargo test -p domain-llm --lib` 原有 4 tests + 2 新增 (chat_completion + stream_completion stub) pass
- trait 方法签名已定义, 实现返 `Err(NotImplemented)`

### Sub-task 1.5: 依赖 + W1 收尾

**文件**:
- 修改 `frontend/package.json` 加 `@monaco-editor/react` `^4.6.0`
- 修改 `Cargo.toml` workspace deps (如需)

**验收**:
- `npm install` 成功
- `cargo check --workspace --all-targets -j 4` 0 错误
- 1 commit: `feat(ULYS-98-W1): monaco editor + chat panel + chat RPC stub + LlmProvider trait 扩展`

### W1 Gate (per守门 #1 v25)

- `cargo test -p bff -p domain-llm --lib -j 4` 全部 pass
- `cd frontend && npm run build` 0 错误
- 1 commit 落档到 `agent/cursor-min/wt-cursor-min-v1`
- **不开 PR 到 dev** (W1 是基础设施, 等 W4 一起)

---

## W2 (2026-09-28 ~ 2026-10-04): AI 代码补全 + Cmd-K inline

**依赖**: W1.4 LlmProvider::chat_completion trait 已定义; W1.3 bff chat RPC 已存在

**目标**: 接入 Anthropic Claude + OpenAI SDK 实现真 LLM 代码补全 + Cmd-K inline edit

### Sub-task 2.1: domain-llm 真实 LLM provider 接入

**文件**:
- 新建 `crates/domain-llm/src/provider/anthropic.rs` (Claude)
- 新建 `crates/domain-llm/src/provider/openai.rs` (OpenAI Compatible)
- 新建 `crates/domain-llm/src/provider/registry.rs` (provider 选择)
- 修改 `crates/domain-llm/src/lib.rs` re-export

**实现要点**:
- 用 `reqwest` + `tokio` + API key from env (`ANTHROPIC_API_KEY` / `OPENAI_API_KEY`)
- Anthropic provider: POST `https://api.anthropic.com/v1/messages`
- OpenAI provider: POST `https://api.openai.com/v1/chat/completions`
- chat_completion 真返回 LLM response
- stream_completion 真返回 SSE stream

**验收**:
- `cargo test -p domain-llm --lib` ≥ 8 tests pass (原有 4 + 4 新增)
- 手动测试: 设置 `ANTHROPIC_API_KEY` 后调用 `chat_completion("hello")` 收到真回复
- mock provider (在 `provider/mock.rs`) 用于 CI 无 key 环境

### Sub-task 2.2: AI 补全 RPC

**文件**:
- 新建 `crates/bff/src/routes/completion.rs`

**接口**:
```rust
// POST /v1/completion/inline
#[derive(Deserialize)]
struct CompletionInlineReq {
    file_path: String,
    cursor_line: u32,
    cursor_col: u32,
    prefix: String,         // 光标前内容 (max 8KB)
    suffix: String,         // 光标后内容 (max 8KB)
    language: String,
}

#[derive(Serialize)]
struct CompletionInlineResp {
    completion: String,
    token_usage: TokenUsage,
}
```

**验收**:
- `cargo test -p bff --lib completion` 4 tests pass
- 调用 `chat_completion` 走 Anthropic 真 LLM (有 key 时) / mock provider

### Sub-task 2.3: frontend monaco inline completion

**文件**:
- 新建 `frontend/components/editor/inlineCompletion.ts`

**实现要点**:
- 使用 Monaco `monaco.languages.registerCompletionItemProvider`
- 触发: 用户停顿 500ms 自动触发 / `Ctrl+Space` 手动触发
- 调用 `POST /v1/completion/inline` → 收到 `completion`
- 显示为灰色 ghost text (`InlineCompletion`)
- `Esc` 取消 / `Tab` 接受

**验收**:
- Monaco 敲代码停顿 500ms → 自动出现 ghost text
- `Tab` 接受后文本插入
- 1 unit test (`inlineCompletion.test.ts`)

### Sub-task 2.4: Cmd-K inline edit

**文件**:
- 新建 `frontend/components/editor/CmdKAction.tsx`

**实现要点**:
- 监听 Monaco `Ctrl+K`
- 弹出 prompt 输入框 (overlay)
- 接收用户选区 + prompt → 调 `POST /v1/composer/edit?inline=true` (W3.4 完整版; W2.4 stub 走 completion)
- 返回 diff → Monaco diff editor preview
- `Accept` 落地 / `Reject` 取消

**验收**:
- `Ctrl+K` 弹窗, 可输入 prompt, 收到 diff
- UI 演示通过 (手动测试)

### Sub-task 2.5: LLM 调用计费埋点

**文件**:
- 新建 `crates/domain-llm/src/metering.rs`
- 修改 `crates/bff/src/middleware.rs` 加 metering middleware

**实现要点**:
- 每个 LLM 调用记录 token usage (input / output / total)
- 写入 `llm_usage` 表 (新 schema) 或 in-memory HashMap (W2 阶段)
- 提供 `GET /v1/metering/usage?user_id=...` query API

**验收**:
- 1 unit test (`metering.test.rs`)
- `cargo test -p domain-llm --lib metering` pass

### W2 Gate

- `cargo test -p domain-llm -p bff --lib -j 4` 全部 pass
- `npm run build` 0 错误
- 实测 (有 API key): inline 补全触发 → 收到有效 ghost text
- 1 commit: `feat(ULYS-98-W2): anthropic/openai provider + inline completion + Cmd-K inline + metering`

---

## W3 (2026-10-05 ~ 2026-10-11): Chat 完整对话 + Composer 多文件编辑

**依赖**: W2.1 LLM provider 已接入; W1.1 monaco editor 已嵌入

**目标**: Chat panel 接 LLM 真推理 + 持久化; Composer 支持跨多文件 diff

### Sub-task 3.1: Chat session 持久化

**文件**:
- 新建 `crates/bff/src/repo/chat_session.rs`
- 新建 `db/migrations/2026-10-XX_chat_sessions.sql`
- 修改 `crates/bff/Cargo.toml` 加 sqlx 依赖 (如未加)

**表 schema**:
```sql
CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    title TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE TABLE chat_messages (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL REFERENCES chat_sessions(id),
    role TEXT NOT NULL CHECK (role IN ('user','assistant','system')),
    content TEXT NOT NULL,
    token_usage JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id, created_at);
```

**验收**:
- migration 文件落档
- `cargo test -p bff --test chat_session_repo` 2 IT pass (mock DB)
- `cargo test -p bff --lib chat` 5 tests pass

### Sub-task 3.2: Chat stream RPC (SSE)

**文件**:
- 修改 `crates/bff/src/routes/chat.rs` 加 SSE endpoint

**接口**:
```rust
// GET /v1/chat/stream?session_id=...
// Server-Sent Events
// chunk format: data: {"delta":"...","done":false}\n\n
```

**验收**:
- `cargo test -p bff --lib chat_stream` 1 IT pass (mock provider)
- 手动测试: 流式输出 token-by-token

### Sub-task 3.3: frontend Chat 接 SSE

**文件**:
- 新建 `frontend/components/chat/useChatStream.ts` (EventSource hook)
- 修改 `frontend/components/chat/ChatPanel.tsx` 用 useChatStream

**验收**:
- Chat 真流式输出 (调用 `/v1/chat/stream`)
- 1 unit test (`useChatStream.test.ts`)

### Sub-task 3.4: Composer 多文件 diff RPC

**文件**:
- 新建 `crates/bff/src/routes/composer.rs`

**接口**:
```rust
// POST /v1/composer/edit
#[derive(Deserialize)]
struct ComposerEditReq {
    files: Vec<ComposerFileContext>,  // 多个文件 + 选区
    instruction: String,               // 用户指令
}
struct ComposerFileContext {
    path: String,
    content: String,
    selection: Option<Selection>,
}

#[derive(Serialize)]
struct ComposerEditResp {
    edits: Vec<FileEdit>,  // 多文件 diff
}
struct FileEdit {
    path: String,
    diff: String,           // unified diff
    new_content: String,
}
```

**验收**:
- `cargo test -p bff --lib composer` 1 IT pass (mock LLM 返 3 文件 diff)

### Sub-task 3.5: frontend Composer UI

**文件**:
- 新建 `frontend/components/composer/ComposerPanel.tsx`
- 修改 `frontend/app/(worktree)/composer/page.tsx` 路由

**验收**:
- UI 可接受/拒绝每个文件 diff, 接受后 git apply 落地 (走 action-engine `ActionType::ApplyPatch`)
- 1 unit test

### Sub-task 3.6: 上下文 RAG (基础)

**文件**:
- 新建 `crates/domain-llm/src/context.rs`

**验收**:
- 1 unit test (mock embedding)

### W3 Gate

- `cargo check --workspace --all-targets -j 4` 0 错误
- `cargo test --workspace --lib -j 4` 全部 pass
- 实测: Chat 真对话 3 轮上下文保持; Composer 3 文件 diff Accept 后 git diff 落地
- 1 commit: `feat(ULYS-98-W3): chat persistence + SSE + composer multi-file + RAG`

---

## W4 (2026-10-12 ~ 2026-10-18): Agent 模式 + Cmd-K 升级 + 收口文档

**依赖**: W3 Chat + Composer + W2 completion + W1 frontend

**目标**: Agent 模式 (自动执行命令 + tool calling); Cmd-K 多次迭代; 文档收口

> **agent 切换选项 (per D-Boy 批准)**: W4 可切 Codex (per守门 #22 + Codex 在 multi-step task 适配); 切换前需先确认 W3 已合或 W3-W4 在同一 agent 串行; 实际切不切由父会话 + W3 验收结果决定

### Sub-task 4.1: Agent 沙箱 + tool calling

**文件**:
- 新建 `crates/agent-bridge/src/sandbox.rs`
- 新建 `crates/agent-bridge/src/tool.rs`

**tools (4 个)**:
- `read_file(path)` — 读 worktree 文件
- `edit_file(path, diff)` — 编辑文件 (走 action-engine Destructive 走二次确认)
- `run_cmd(cmd)` — 受限 shell (黑名单: `rm -rf /`, `mkfs`, `dd if=`, `shutdown`, `reboot`, etc.)
- `web_search(query)` — 调外部 search API (W4 stub: 返回固定 mock)

**验收**:
- `cargo test -p agent-bridge --lib sandbox` 2 IT pass (沙箱禁危险命令)

### Sub-task 4.2: action-engine 加 3 类 AI Action

**文件**:
- 修改 `crates/action-engine/src/action.rs` 加 3 个 ActionType

**新增**:
```rust
pub enum ActionType {
    // ... 原有 18 项 ...
    AiChat,              // W1 Chat RPC
    AiCompletion,        // W2 inline completion
    AiComposerEdit,      // W3 multi-file diff
}
```

**验收**:
- `cargo test -p action-engine --lib` 18 → 21 actions pass

### Sub-task 4.3: Agent mode 接入 domain-agent

**文件**:
- 新建 `crates/domain-agent/src/ai_session.rs`

**实现要点**:
- AgentSession 14 状态机扩展: 加 `ToolRunning` / `ToolCompleted` 状态 (per DD §7.4)
- 工具调用循环: LlmProvider::chat_completion → parse tool calls → execute → feed back → continue

**验收**:
- `cargo test -p domain-agent --lib ai_session` 1 IT pass (mock Agent 完成 3-step task)

### Sub-task 4.4: frontend Agent mode UI

**文件**:
- 新建 `frontend/components/agent/AgentRunner.tsx`
- 修改 `frontend/app/(worktree)/agent/page.tsx` 路由

**验收**:
- UI 触发 Agent 后, tool calls 实时显示在底部 log
- 手动测试通过

### Sub-task 4.5: Cmd-K 升级 (多次迭代)

**文件**:
- 修改 `frontend/components/editor/CmdKAction.tsx`

**验收**:
- UI flow 验证: 第一次不满意可继续 prompt 精化

### Sub-task 4.6: Star-Cursor RFC + Spec 收口文档

**文件**:
- 新建 `docs/rfc/star-cursor-min-v1.md` (≥ 100 行 RFC)
- 新建 `docs/specs/star-cursor-min-spec.md` (≥ 100 行 Spec)
- 新建 `docs/evidence/ulys-98-cursor-min-evidence.md` (验收 evidence)

**验收**:
- 3 文件落档, 各 ≥ 100 行
- RFC 含 5 大功能需求 + NFR + ADR 关联
- Spec 含接口契约 + 数据模型 + 验收 AC
- Evidence 含 22 sub-task 全部完成 evidence

### W4 Gate

- `cargo check --workspace --all-targets -j 4` 0 错误
- `cargo test --workspace --lib -j 4` 全部 pass
- #6 v2 错误码 6-field (sandbox / tool error)
- #7 `unsafe_code = "forbid"`
- #10 author=Ulysses (Mavis 永久代签 per #14 v4)
- #13 W/T/M 100% 覆盖 (tool_audit / sandbox_audit / agent_session 表)
- 1 commit: `feat(ULYS-98-W4): agent mode + sandbox + tool calling + RFC + Spec`
- 推送 branch + 创建 PR → dev (per守门 #14 v4 闸门)

---

## 总览

| Week | 主题 | sub-task | 关键交付 | commit |
|---|---|---|---|---|
| **W1** (9/21-9/27) | 基础设施 | 5 | monaco + chat panel + chat RPC stub + LlmProvider trait | 1 commit |
| **W2** (9/28-10/4) | AI 补全 + Cmd-K | 5 | anthropic/openai provider + inline completion + Cmd-K + metering | 1 commit |
| **W3** (10/5-10/11) | Chat + Composer | 6 | chat persistence + SSE + composer + RAG | 1 commit |
| **W4** (10/12-10/18) | Agent + 收口 | 6 | sandbox + tool calling + agent mode + RFC/Spec/Evidence | 1 commit + 1 PR |

**估 token 总耗**: ~50M tokens
**估 sub-session 总耗**: 1 agent × 4 周 × ~3-5 sub-session/周 = ~16 sub-session

---

## 风险与约束

1. **LLM API key 环境**: W2 起需 `ANTHROPIC_API_KEY` / `OPENAI_API_KEY`; 沙盒跑测试需 mock provider (per Sub-task 2.1)
2. **frontend monaco bundle size**: 首次加载 ~2MB, 需 code-split + lazy load (per W1.1 提示)
3. **Composer diff 落地**: 涉及 git apply, 跨 worktree 边界要谨慎 (per守门 #13)
4. **Agent 沙箱安全**: tool calling 必须严格 RBAC + 危险命令黑名单 (per action-engine Destructive 7 项参考)
5. **worktree `wt-cursor-min-v1`**: 单 worktree 4 周使用, 避免与其他 agent worktree (wt-arg-* / wt-ex-*) 互锁 (per守门 #1 v25)
6. **W4 agent 切换 Codex**: 切换前需先确认 W3 已 commit, Codex 接手 W4 起; 不切换则延续 W1-W3 同 agent

---

## 派工指令 (per D-Boy 2026-09-20)

- **派工模型**: 单 agent 串行 (W1~W4 同 agent)
- **W4 切 Codex 可选**: 父会话视 W3 验收结果决定
- **API key**: 派工前 D-Boy 提供 `ANTHROPIC_API_KEY` + `OPENAI_API_KEY` (注入 multica agent env vars)
- **worktree**: 已建 `/d/Star/.worktrees/wt-cursor-min-v1` (branch `agent/cursor-min/wt-cursor-min-v1`)
