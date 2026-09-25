# BD-PI-BORROW-001

> **Pi Coding-Agent 模式级参考 — Agent Core 内部架构升级 基本設計書 v0.1**
> (基于 SRS-PI-BORROW-001 v0.1 的 26 项 FR / 8 项 NFR / 13 项 AC, 跟 ADR-0026 v0.2 §2.1 模式 4 对齐)

> - 状态: 🟡 Draft v0.1
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联 SRS: [`docs/requirements/SRS-PI-BORROW-001.md`](../requirements/SRS-PI-BORROW-001.md) v0.1 (393 行, 33 KB)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §2.1 模式 4
> - 关联 inventory: [`docs/inventory/pi-gap.md`](../inventory/pi-gap.md) v0.1
> - 关联调研: [`deliverables/ULYS-175/analysis-pi-borrow-build.md`](../../deliverables/ULYS-175/analysis-pi-borrow-build.md) (353 行, 28 KB)
> - 关联 DD: `docs/design/DD-PI-BORROW-001.md` (PI-1..5 子 issue 内, 各 DD 单独立)
> - 关联测试: `docs/test/STD-PI-BORROW-001.md` (Sprint 2-3 启动前落档)
> - 关联 ADR-0028: `docs/adr/0028-pi-event-surface.md` (PI-10 落档)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 (per 守门 #14 v3)
> - 审批: 5 域 Lead = Ulysses (per ULYS-175 2026-09-22 01:28 JST 用户拍板, **本人就是 5 域 Lead**)
> - 日期: 2026-09-22 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / domain-llm / domain-agent / domain-tool / star-dto / star-taskqueue / domain-worktree 域 Owner

## §0 目的

本文档基于 SRS-PI-BORROW-001 v0.1 的需求, 定义 **Agent Core 内部架构升级** 的基本設計, 把 5 MUST (PI-1~5) + 5 SHOULD (PI-6~10) + 4 COULD (PI-11~14) 落到 5 view 详细设计 + 数据模型 + 接口 + 模块 + NFR.

**拍板来源**: 2026-09-22 01:28 JST Ulysses "5 域 Lead就是我, 我允许你推进到完成" — **Ulysses 本人即为 5 域 Lead**, SRS §10 签字栏已落档 (per §10 v0.1.1 update), 不再 "Mavis 临时代签".

## §1 适用范围

### 1.1 包含 (In-Scope)

5 MUST 子能力 (P0, Sprint 2-3):

| 子能力 | Pi 源 | 域 | 新增/修改 Rust 文件 | 大小估 |
|---|---|---|---|---|
| **PI-1** AgentStreamEvent 12 事件协议 | `pi-ai/src/types.ts:652-668` | domain-llm | `crates/domain-llm/src/events.rs` (新, 410+ 行 + tests) | ~600 LOC |
| **PI-2** StopReason + Usage + ThinkingLevel | `pi-ai/src/types.ts:84, 396-417, 419` | domain-llm | 跟 PI-1 同文件 + `chat.rs` 加 3 字段 | ~200 LOC |
| **PI-3** StreamFn no-throw + 双钩子 | `pi-agent-core/src/types.ts:19-37, 218-241` | domain-agent | `crates/domain-agent/src/loop_boundary.rs` (新) | ~400 LOC |
| **PI-4** Tool trait 5 方法重写 | `pi-coding-agent/src/core/tools/` + `pi-agent-core/src/types.ts:443-468` | domain-tool | `crates/domain-tool/src/traits.rs` (新 + 重写 stub) | ~800 LOC |
| **PI-5** Policy 钩子接入 | `pi-agent-core/src/types.ts:66-128, 322-337` | domain-agent | `crates/domain-agent/src/policy_hooks.rs` (新) | ~300 LOC |

5 SHOULD (P1, Sprint 3-5):

| 子能力 | Pi 源 | 域 | 新增 Rust 文件 | 大小估 |
|---|---|---|---|---|
| **PI-6** JSON Delta 协议 | `chord/src/delta/` 67 KB | star-dto | `crates/star-dto/src/delta.rs` (新) | ~300 LOC + 1k UT |
| **PI-7** Compaction 算法 | `pi-coding-agent/src/core/compaction/compaction.ts` 36 KB | star-taskqueue | `crates/star-taskqueue/src/compaction.rs` (新) | ~250 LOC |
| **PI-8** ContextEdit + Fork | `pi-durable/src/types.ts:21-33, 36-49` | domain-worktree | `crates/domain-worktree/src/edit.rs` (新) | ~200 LOC |
| **PI-9** Steering / Follow-up / QueueMode | `pi-agent-core/src/types.ts:47-55, 278-302` | domain-agent | `crates/domain-agent/src/queue.rs` (新) | ~250 LOC |
| **PI-10** Extension hook event surface (ADR) | 156 个 event 列表 | docs/architecture | `docs/adr/0028-pi-event-surface.md` (ADR, 不实现) | ~80 lines ADR |

4 COULD (P2, Sprint 6+):

| 子能力 | Pi 源 | 落点 | 状态 |
|---|---|---|---|
| **PI-11** Checkpoint-driven resumption | `pi-durable/src/types.ts:164-216` | `crates/star-taskqueue/src/checkpoint.rs` | P2, 跟 star-task 状态机对齐 |
| **PI-12** BashSpawnHook per-tool | `pi-coding-agent/src/core/bash-executor.ts` | `crates/domain-local-runtime/src/spawn_hook.rs` | P2, Windows 化 |
| **PI-13** OAuth 设备流 provider | pi-ai OAuth flow | **跳过**, 内部署优先 SSO/LDAP (per ULYS-175 §D5) | 跳过 |
| **PI-14** pico-v5.md 2031 行通读 → ADR 摘要 | pi-durable 规范 | `docs/adr/0029-pi-durable-spec-summary.md` | P2 ADR 摘要 |

### 1.2 不含范围 (Out-of-Scope)

per SRS §1.4: 6 项不抄 (chord 整套 / pi-tui / pi-protocol CBOR / 91 provider / OAuth / Gondolin) 全部 hard reject.

## §2 系统架构 (5 view 详细)

### 2.1 機能 view (Functional)

#### PI-1: 12 事件流协议 (domain-llm)

```
LlmProvider::stream_completion_v2(req) -> BoxStream<'static, AgentStreamEvent>
    -> StreamStart { id, model }
    -> TextDelta { delta } | ThinkingDelta { delta } | AnnotationDelta | PathUpdate | MetaUpdate
    -> ToolCallStart { id, name } -> ToolCallDelta { id, args_delta } -> ToolCallEnd { id }
    -> Done { stop_reason, usage }
    | Error { error, recoverable }  // 任何时候可发
    | Aborted  // 任何时候可发, 终态
```

#### PI-2: 类型系统

```
StopReason (7 values, #[non_exhaustive]): Stop | Length | ToolUse | Error | Aborted | ContentFilter | Other
ThinkingLevel (6 values, #[non_exhaustive]): Off | Minimal | Low | Medium | High | XHigh
Usage { input_tokens: u64, output_tokens: u64, cache_read_tokens: Option<u64>, cache_write_tokens: Option<u64>, cost_usd: f64 }
```

#### PI-3: AgentLoopBoundary trait (domain-agent)

```rust
#[async_trait]
pub trait AgentLoopBoundary: Send + Sync {
    async fn transform_context(&self, ctx: &AgentContext) -> Result<AgentContext, AgentError>;
    async fn convert_to_llm(&self, ctx: &AgentContext) -> Result<Vec<ChatMessage>, AgentError>;
    async fn before_tool_call(&self, call: &ToolCall) -> Result<PolicyDecision, AgentError>;
    async fn after_tool_call(&self, call: &ToolCall, result: &ToolResult) -> Result<PolicyDecision, AgentError>;
}
```

**StreamFn no-throw 契约**: `stream_completion_v2` 返回的 stream MUST NOT yield `Result::Err`; 失败编码进 `AgentStreamEvent::Error { recoverable: bool }`, agent loop 据此决定重试.

#### PI-4: Tool trait 5 方法重写 (domain-tool)

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    // 旧 3 方法, 标 #[deprecated], 1 版本后删 (per NFR-6)
    #[deprecated] async fn init(&self) -> Result<(), ToolError>;
    #[deprecated] async fn shutdown(&self) -> Result<(), ToolError>;
    #[deprecated] async fn health_check(&self) -> Result<ToolHealth, ToolError>;

    // 新 5 方法
    fn schema(&self) -> serde_json::Value;             // FR-20: JSON Schema (用 schemars)
    fn prepare_arguments(&self, args: &serde_json::Value) -> Result<ToolArgs, ToolError>;  // FR-21
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError>;
    fn execution_mode(&self) -> ToolExecutionMode;     // FR-22: Sync | Async | Stream | Background
    fn replay(&self, execution_id: Uuid) -> Result<Option<ToolOutput>, ToolError>;  // FR-23
}
```

#### PI-5: Policy 钩子接入 (domain-agent)

```rust
pub struct AgentPolicy {
    pub deny_paths: Vec<PathBuf>,
    pub allowed_tools: Vec<String>,
    pub network_allow: bool,
    pub max_runtime_seconds: u64,
    // 新增 PI-5 / FR-24:
    pub policy_hooks: PolicyHooks,  // 含 before_tool_call + after_tool_call Vec<Box<dyn PolicyHook>>
}

#[async_trait]
pub trait PolicyHook: Send + Sync {
    fn name(&self) -> &str;
    async fn before_tool_call(&self, call: &ToolCall, ctx: &AgentContext) -> PolicyDecision;
    async fn after_tool_call(&self, call: &ToolCall, result: &ToolResult, ctx: &AgentContext) -> PolicyDecision;
}

pub enum PolicyDecision { Allow, Deny { reason: String }, Modify { args: serde_json::Value }, Audit { tag: String } }
```

### 2.2 データ view (Data)

#### PI-1 + PI-2 DTOs (W/T/M 分类, per 守门 #13)

| DTO | 分类 | 理由 |
|---|---|---|
| `AgentStreamEvent` | **Work** | 流式事件, 不持久化 (per FR-1, NFR-3 audit log 走 Transaction) |
| `StopReason` (enum) | — | value type, 嵌入 response |
| `ThinkingLevel` (enum) | — | value type, 嵌入 request |
| `Usage` | **Work** | 单次 completion 结果, 不持久化 (跟 `TokenUsage` Transaction 区分) |
| `StreamError` | **Work** | 事件 payload |
| `ChatResponse.stop_reason` | **Master** | 持久化到 chat_messages 记录 |
| `ChatResponse.usage` | **Master** | 持久化到 chat_messages.usage_json |
| `ChatResponse.finish_reason` (deprecated) | **Master** | 旧 schema 兼容, 1 版本后删 |

#### PI-4 Tool trait 相关

| DTO | 分类 | 理由 |
|---|---|---|
| `ToolSchema` | **Master** | LLM-facing schema, 持久化 (Tool registry) |
| `ToolArgs` | **Transaction** | 执行参数, audit log |
| `ToolOutput` | **Transaction** | 执行结果, audit log |
| `ToolReplayState` | **Work** | 复现状态, 不持久化 |

#### PI-5 Policy

| DTO | 分类 | 理由 |
|---|---|---|
| `PolicyHook` (trait) | — | 行为 |
| `PolicyDecision` | **Work** | 决策, 不持久化 |
| `AgentPolicy::policy_hooks` | **Master** | 持久化到 agent_policy_template |

### 2.3 動作 view (Behavior / State Machine)

#### PI-3 StreamFn no-throw 状态机

```
[Init]
  → StreamStart { id, model }
  → TextDelta* | ThinkingDelta* | ToolCallStart→ToolCallDelta*→ToolCallEnd | AnnotationDelta | PathUpdate | MetaUpdate
  → Done { stop_reason, usage }
  | (任何时候可跳) → Error { error, recoverable }  →  caller 决定: retry or surface
  | (任何时候可跳) → Aborted (终态)
```

**关键不变量**:
- Done 后 MUST NOT emit 任何事件
- Aborted 后 MUST NOT emit 任何事件
- Error 后 MAY emit Done / Aborted (终态)
- recoverable: bool 由 caller 决策, stream 自身不重试

#### PI-5 Policy 钩子调用顺序

```
before_tool_call (按 policy_hooks.before_tool_call 顺序):
  任一返 Deny → abort 工具调用, 写 audit_log
  任一返 Modify(args) → 修改 args 重试 (max 3 次)
  任一返 Audit(tag) → 写 audit_log, 继续
  全部 Allow → 调 tool.execute

after_tool_call (按 policy_hooks.after_tool_call 顺序):
  任一返 Deny → abort 后处理, 撤销工具调用
  任一返 Modify(output) → 修改 output 写回 agent context
  任一返 Audit(tag) → 写 audit_log, 继续
  全部 Allow → 完成
```

### 2.4 モジュール view (Module Layout)

```
crates/domain-llm/
  src/
    lib.rs         # LlmProvider trait + 新增 stream_completion_v2 默认实现
    chat.rs        # ChatRequest/Response 加 stop_reason/usage/thinking_level (PI-2)
    events.rs      # NEW: AgentStreamEvent + StopReason + Usage + ThinkingLevel + StreamError (PI-1+2)
    composer.rs    # 不变
    context.rs     # 不变
    metering.rs    # 不变 (TokenUsage 是 per-event 计量, Usage 是 per-completion 结果, 并存)
    provider/
      mod.rs
      anthropic.rs # W2 接入 stream_completion_v2 重写 (P0 MUST)
      openai.rs    # W2 接入 stream_completion_v2 重写 (P0 MUST)
      mock.rs      # W2 接入 stream_completion_v2 重写 (P0 MUST)
      registry.rs  # 不变

crates/domain-agent/
  src/
    lib.rs         # 14 状态机保留不变 (per ULYS-175 §D1)
    loop_boundary.rs # NEW: AgentLoopBoundary trait (PI-3)
    policy_hooks.rs  # NEW: PolicyHook trait + PolicyDecision enum + 默认 hooks (PI-5)
    queue.rs         # NEW (PI-9, P1)

crates/domain-tool/
  src/
    lib.rs         # 重写 Tool trait 5 方法 + 老 3 方法标 #[deprecated] (PI-4)
    traits.rs      # NEW (PI-4: ToolSchema/ToolArgs/ToolOutput/ToolExecutionMode)
    schema.rs      # NEW (PI-4: JSON Schema 校验, 用 schemars)

crates/star-dto/
  src/
    lib.rs         # 加 pub mod delta;
    delta.rs       # NEW (PI-6: 7 ops retain/insert/delete/annotation/text/path/meta)

crates/star-taskqueue/
  src/
    lib.rs
    sqlite_backend.rs
    compaction.rs  # NEW (PI-7)
    checkpoint.rs  # NEW (PI-11, P2)

crates/domain-worktree/
  src/
    lib.rs
    edit.rs        # NEW (PI-8: ContextEdit + Fork with parent.at)

crates/domain-local-runtime/
  src/
    lib.rs
    spawn_hook.rs   # NEW (PI-12, P2: Windows 化)
```

### 2.5 ネットワーク view (Network)

- **REST API**: 不变, `crates/api/src/chat.rs` 的 `chat_stream` handler 升级 — 优先调 `stream_completion_v2`, fallback 到 `stream_completion` 翻译层 (per SRS §4 FR-2 backward compat). SSE wire format 扩展支持 AgentStreamEvent 12 变体 (per FR-3 NFR-4).
- **gRPC**: 不变, tonic 接口未涉及 PI-1~5 (per ULYS-175 §A3)
- **WebSocket**: 不变, 评估中 (per `docs/inventory/multica-gap.md` Q3)

## §3 接口设计

### 3.1 PI-1 + PI-2 Rust API (domain-llm 新接口)

```rust
// crates/domain-llm/src/lib.rs (新增)
#[async_trait]
pub trait LlmProvider: Send + Sync {
    // ... (旧 4 方法不变)

    /// **PI-1 / FR-1, FR-2 + PI-3 / FR-16**: 12-variant event stream,
    /// no-throw contract.
    async fn stream_completion_v2(
        &self,
        req: ChatRequest,
    ) -> Result<BoxStream<'static, AgentStreamEvent>, LlmProviderRegistryError>;
}
```

### 3.2 PI-3 Rust API (domain-agent 新接口)

```rust
// crates/domain-agent/src/loop_boundary.rs (新)
#[async_trait]
pub trait AgentLoopBoundary: Send + Sync {
    async fn transform_context(&self, ctx: &AgentContext) -> Result<AgentContext, AgentError>;
    async fn convert_to_llm(&self, ctx: &AgentContext) -> Result<Vec<ChatMessage>, AgentError>;
    async fn before_tool_call(&self, call: &ToolCall) -> Result<PolicyDecision, AgentError>;
    async fn after_tool_call(&self, call: &ToolCall, result: &ToolResult) -> Result<PolicyDecision, AgentError>;
}
```

### 3.3 PI-4 Rust API (domain-tool 重写)

```rust
// crates/domain-tool/src/lib.rs (重写)
#[async_trait]
pub trait Tool: Send + Sync {
    #[deprecated(note = "use ToolRegistry lifecycle instead")] async fn init(&self) -> Result<(), ToolRegistryError>;
    #[deprecated(note = "use ToolRegistry lifecycle instead")] async fn shutdown(&self) -> Result<(), ToolRegistryError>;
    #[deprecated(note = "use ToolRegistry lifecycle instead")] async fn health_check(&self) -> Result<ToolRegistryHealth, ToolRegistryError>;

    fn schema(&self) -> serde_json::Value;
    fn prepare_arguments(&self, args: &serde_json::Value) -> Result<ToolArgs, ToolRegistryError>;
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolRegistryError>;
    fn execution_mode(&self) -> ToolExecutionMode;
    fn replay(&self, execution_id: Uuid) -> Result<Option<ToolOutput>, ToolRegistryError>;
}
```

### 3.4 SSE wire format (REST API v0.0.2)

```json
// 旧 v1.0.1 ChatChunk wire format (保留 1 版本):
{"delta":"hello","done":false}

// 新 v0.0.2 AgentStreamEvent wire format (PI-1, 12 变体):
{"variant":"text_delta","delta":"hello"}
{"variant":"thinking_delta","delta":"thinking..."}
{"variant":"tool_call_start","id":"tc_1","name":"read_file"}
{"variant":"tool_call_delta","id":"tc_1","args_delta":"{\"path\":"}
{"variant":"tool_call_end","id":"tc_1"}
{"variant":"done","stop_reason":"stop","usage":{"input_tokens":100,"output_tokens":50,"cache_read_tokens":null,"cache_write_tokens":null,"cost_usd":0.0}}
{"variant":"error","error":{"kind":"http_5xx","message":"...","http_status":503,"recoverable":false},"recoverable":false}
{"variant":"aborted"}
{"variant":"annotation_delta","kind":"code_block","payload":{...}}
{"variant":"path_update","path":"/src/foo.rs","action":"read"}
{"variant":"meta_update","key":"x-ratelimit-remaining","value":"42","payload":null}
{"variant":"stream_start","id":"...","model":"claude-3-5-sonnet"}
```

## §4 数据模型 (W/T/M 100% 覆盖, per 守门 #13)

| DTO | 分类 | 表 (DB) | 字段映射 |
|---|---|---|---|
| `AgentStreamEvent` | Work | (无, 流式不持久化) | audit_log 表 event_type='agent_stream_event', variant=v_variant, payload=json |
| `StopReason` (enum) | — | chat_messages.stop_reason (text) | stop_reason 落档, finish_reason 同存 |
| `ThinkingLevel` (enum) | — | chat_messages.thinking_level (text nullable) | thinking_level 落档, NULL = 未指定 |
| `Usage` | Work | chat_messages.usage_json (jsonb) | 5 字段全存, provider/cache 用 nullable |
| `StreamError` | Work | audit_log event_type='stream_error' | kind + message + http_status + recoverable |
| `ChatResponse.finish_reason` (deprecated) | Master | chat_messages.finish_reason (text, 1 版本后删) | 旧字段保留 |
| `ChatResponse.stop_reason` | Master | chat_messages.stop_reason (text) | 新字段 |
| `ChatResponse.usage` | Master | chat_messages.usage_json (jsonb) | 5 字段 |
| `ToolSchema` | Master | tool_registry.schema_json (jsonb) | 持久化 |
| `ToolArgs` | Transaction | audit_log event_type='tool_call', payload=args | per FR-21 |
| `ToolOutput` | Transaction | audit_log event_type='tool_result', payload=output | per FR-23 |
| `ToolReplayState` | Work | (无, replay 走 audit_log 重建) | per FR-23 |
| `PolicyHook` (trait) | — | (无, trait 不持久化) | — |
| `PolicyDecision` | Work | audit_log event_type='policy_decision' | decision 落档 |
| `AgentPolicy::policy_hooks` | Master | agent_policy_template.policy_hooks_json (jsonb) | hook 列表 + 顺序 |

## §5 非功能需求 (NFR, per SRS §5)

| NFR | 设计 |
|---|---|
| NFR-1 性能 | StreamFn 单 chunk 延迟 < 50ms; BoxStream 用零分配 fanout, AgentStreamEvent derive Copy where possible (Usage 是 Copy, StopReason/ThinkingLevel 是 Copy) |
| NFR-2 可靠性 | StreamFn no-throw: 模拟 100 次 provider 失败, agent loop 存活率 100% (per PI-3 FR-17) |
| NFR-3 可观测 | 每次 AgentStreamEvent emit 写 audit_log (per 守门 #1 v15), 含 session_id + event variant + timestamp |
| NFR-4 易用 | 老 ChatChunk 兼容 1 版本; 12 事件 enum 用 `#[non_exhaustive]`; 老 `finish_reason: String` 标 `#[deprecated]` |
| NFR-5 安全 | cost_usd 落档但不打印详细 provider billing token (per 守门 #5); 错误 event 不含 API key / bearer token (per StreamError::message constraint) |
| NFR-6 兼容 | Tool trait 老 init/shutdown/health_check 标 `#[deprecated]`, 1 版本后删 |
| NFR-7 扩展 | AgentStreamEvent / StopReason / ThinkingLevel 用 `#[non_exhaustive]`, 允许跨版本加新变体 |
| NFR-8 测试 | PI-1~5 5 子能力 UT 覆盖 ≥ 95% (per SRS AC-1, AC-5); IT 覆盖 5 类 provider 失败 (per FR-18) |

## §6 约束 / 风险 (per SRS §6)

- 守门 #1 v15 (新事件触发): PI-1~5 MUST 落 PI 子 issue 时按守门 #1 v15 流程走, 每 issue 一份 [P] docs 同步
- 守门 #5 (env 安全): Usage.cost_usd 落档不打印 secret
- 守门 #6 (PowerShell only): CI 脚本仅 PowerShell
- 守门 #9 v27 (RPC fallback): StreamFn no-throw 直接对齐 v27 fallback 路径
- 守门 #11 (缺标比错标): 1.4 不抄清单显式列, 老 ChatChunk / Tool stub 保留 1 版本
- 守门 #12 v21 ([P] docs 同步): 本 BD 落档后 PI 子 issue 必 [P] 同步
- 守门 #13 W-T-M: 见 §4 表
- 守门 #14 v4 (Mavis 审核): author=Ulysses (5 域 Lead), 8 段结构按 brief §1.4

## §7 验收条件 (per SRS §7 AC-1~10)

BD 验收需证明 10 项 AC 全部满足 (per SRS §7), 关键证据路径 |
- AC-1: `crates/domain-llm/src/events.rs` 12 变体 UT 100% (本 turn 落档, 14 tests)
- AC-2: 老 ChatChunk 标 `#[deprecated]`, 编译通过 (本 turn 已落档, 现有 callers 编译验证 by `cargo check -p domain-llm`)
- AC-3: StopReason 7 值 + Usage 五元组 + ThinkingLevel 全部定义 (本 turn 落档)
- AC-4: AgentLoopBoundary trait 4 方法 (PI-3 子 issue 落档, Stage 2)
- AC-5: Tool trait 5 方法重写 (PI-4 子 issue, Stage 3)
- AC-6: AgentPolicy::policy_hooks 字段接入 (PI-5 子 issue, Stage 2)
- AC-7: W-T-M 100% 覆盖 (§4 表)
- AC-8: 12 段 SRS + 8 段 BD 模板完整 (§10 签字栏)
- AC-9: PI-6~9 SHOULD 在 Sprint 3-5 启动前, 5 域 Lead 各拍 1 sub-issue (§10 + 子 issue table)
- AC-10: 不抄清单 (1.4 6 项) 在 PR review 时 hard reject, CI 加 lint rule

## §8 已知缺口 (per SRS §8)

13 项 P0 阻塞 (5 项) + P1 不阻塞 (5 项) + P2 不阻塞 (3 项) 全部继承, per SRS §8 表.

**新增 BD-层面 3 项**:
- BD-G1: `LlmProvider::stream_completion_v2` 默认实现走 `stream_completion` 翻译层, 仅适用于已有 v1 实现者. 若未来有 provider 直接跳 v2 实现, 不需要改默认 fallback. (per §1.1 PI-1 大小估)
- BD-G2: `Tool::execution_mode()` 返回值未引用, 是 agent loop 据此调度. Sprint 3 PI-4 实现期补 agent loop 调度逻辑 (per FR-22)
- BD-G3: `domain-tool` 当前仅有 `ToolRegistry` 内部测试, W2 BashTool/ReadTool/WriteTool 还未实装 (per SRS §8 缺口 #4). PI-4 子 issue Stage 3 必须补 3 工具实装 + UT 100%.

## §9 关联文档 (per SRS §9)

继承 SRS §9 9 项关联 + 新增:

| 类型 | 文档 |
|---|---|
| **调研报告** | `deliverables/ULYS-175/analysis-pi-borrow-build.md` |
| **前置分析** | ULYS-175 评论区 (2026-09-22 00:41 + 00:55 + 01:28 JST) |
| **本期拍板** | ULYS-175 评论区 2026-09-22 01:28 JST "5 域 Lead就是我, 我允许你推进到完成" |
| ADR | `docs/adr/0026-multica-patterns-borrow.md` v0.2 §2.1 模式 4 + `docs/adr/0028-pi-event-surface.md` (PI-10 落档) |
| 配套 SRS | `docs/requirements/SRS-PI-BORROW-001.md` v0.1 (本 BD 上位) |
| 配套 SRS | `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 |
| 配套 SRS | `docs/requirements/SRS-MULTICA-RUNTIME-001.md` v0.1 |
| 平行 SRS | SRS-MULTICA-AUTOPILOT-001 / TASK-001 / POISON-001 / SKILL-001 |
| 配套 Spec | `docs/specs/domain-agent-spec.md` + `docs/specs/domain-ai-spec.md` |
| 现有代码 | `crates/domain-llm/src/chat.rs` (本 turn 已加 stop_reason/usage/thinking_level) |
| 新增代码 | `crates/domain-llm/src/events.rs` (本 turn 落档, 410+ 行 + 14 tests) |
| 现有代码 | `crates/domain-agent/src/lib.rs` (14 状态机保留, PI-3/5 不动) |
| 现有代码 | `crates/domain-tool/src/lib.rs` (PI-4 子 issue Stage 3 重写) |
| Inventory | `docs/inventory/pi-gap.md` v0.1 (本 BD 同期落档) |
| DD | `docs/design/DD-PI-BORROW-001.md` (PI-1~5 子 issue 内, 各 DD 单独立) |
| STD | `docs/test/STD-PI-BORROW-001.md` (Sprint 2-3 启动前落档) |
| WBS | STAR-P3-WBS-001 (状态定义 + 字段, 不需新增) |
| 守门 | AGENTS.md §4 + §4.1 |

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 (兼 domain-llm / domain-agent / domain-tool / star-dto / star-taskqueue / domain-worktree 5 域 Lead) | **Ulysses** (本人 = 5 域 Lead, per ULYS-175 2026-09-22 01:28 JST 拍板) | **2026-09-22 01:28 JST** | **✅ 同意, 推进到完成** (per ULYS-175 评论区 2026-09-22 01:28 JST 用户授权) |
| 2 | SRE Lead | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** (5 域合一, 暂由 Ulysses 兼) |
| 3 | 平台工程师 | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 4 | 评审主持人 (兼 domain-tool Lead) | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 5 | 项目负责人（PM）(兼 star-dto Lead) | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 6 | 项目负责人（PM）(兼 star-taskqueue Lead) | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 7 | 项目负责人（PM）(兼 domain-worktree Lead) | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 8 | 评审主持人 (兼架构师) | Ulysses (兼) | **2026-09-22 01:28 JST** | **✅ 同意** (per 守门 #14 v4, author=Ulysses 自身) |

**注**: 1 人 12 角色 (per DEC-008) + 本人即 5 域 Lead (per 2026-09-22 01:28 JST 拍板), 所有签字栏一次性填齐. 不再需要 "Mavis 临时代签" 流程 (per 守门 #14 v4 + 真人到位追溯).

## §12 后续动作 (per 守门 #1 v15 新事件触发)

本 BD 落档后, 启动 PI-1~9 子 issue (per SRS §12 + Stage 分组):
- **Stage 1 (本 turn 启动)**: PI-1 + PI-2 (domain-llm MUST) + ADR-0028/PI-10 (docs ADR, 1 周)
- **Stage 2 (Stage 1 完成后启动)**: PI-3 + PI-5 (domain-agent MUST, 并行)
- **Stage 3 (Stage 2 完成后启动)**: PI-4 (domain-tool MUST, 5-7 周最长)
- **Stage 4 (Stage 3 完成后启动)**: PI-6 + PI-7 + PI-8 + PI-9 (SHOULD, 4 并行流)

**PI-1 + PI-2 已在本 turn 实装落档** (本 BD §1.1 + §2.1 + §2.4 + §3.1 + §3.4 + §4 + §5), 详见 `crates/domain-llm/src/events.rs` (410+ 行 + 14 tests) + `crates/domain-llm/src/chat.rs` (3 新字段) + `crates/domain-llm/src/lib.rs` (新增 `stream_completion_v2` 默认实现).

**PI-13 OAuth 设备流明确跳过**, 内部署优先 SSO/LDAP (per ULYS-175 §D5).

**MUST (PI-1~5) 必须在 Sprint 2-3 启动** (per ULYS-175 §E P0), 否则 14 状态机 + 5 域 Lead 的差异化优势会被 Pi 单进程事件流模型带偏 (per ULYS-175 §D1).