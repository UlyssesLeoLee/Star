# SRS-PI-BORROW-001

> **Pi Coding-Agent 模式级参考 — Agent Core 内部架构升级要件定義書 v0.2**
> (基于 ULYS-175 调研报告 + ULYS-175 §F 不适配分析, 跟 ADR-0026 v0.2 §2.1 模式 4 对齐)
>
> - 状态: ✅ v0.2 完成 (5 域 Lead 拍板签字, 本期不立调研 issue 完成, PI 子 issue 已派工)
> - 目标阶段: 要件定義 ✅ → 基本設計 ✅ (BD-PI-BORROW-001 v0.1 落档) → 詳細設計 → 実装 → テスト
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-PI-BORROW-001.md`](../design/BD-PI-BORROW-001.md) v0.1 (本 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §2.1 模式 4 (新增)
> - 关联 inventory: [`docs/inventory/pi-gap.md`](../inventory/pi-gap.md) v0.1 (本 turn 落档, 跟 multica-gap.md 对齐)
> - 关联调研报告: [`deliverables/ULYS-175/analysis-pi-borrow-build.md`](../../deliverables/ULYS-175/analysis-pi-borrow-build.md) (353 行, 28 KB)
> - 关联前一轮分析: ULYS-175 评论区 (2026-09-22 00:41 JST "我的技术选型与 Pi 的不适配点" 9 项对照)
> - 关联本期拍板: ULYS-175 评论区 (2026-09-22 01:28 JST "5 域 Lead就是我, 我允许你推进到完成")
> - 关联 PI 子 issue: PI-1..PI-9 + PI-10 (per SRS §12, 本 turn spawn, Stage 1 即时启动)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— 兼 5 域 Lead (per 2026-09-22 01:28 JST 拍板)
> - 审批: 架构师 = Ulysses 兼 (per 守门 #14 v4)
> - 日期: 2026-09-22 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead (本 turn = Ulysses 兼) / 域 Owner

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-PI-BORROW-001 |
| 文书名 | Pi Coding-Agent 模式级参考 — Agent Core 内部架构升级要件定義書 |
| 版本 | **v0.2** (本 turn 升版) |
| 作成日 | 2026-09-22 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— 兼 5 域 Lead (per 2026-09-22 01:28 JST 拍板) |
| 承認者 | 架构师 = Ulysses 兼 (per 守门 #14 v4) |
| 关联 ADR | ADR-0026 v0.2 §2.1 模式 4 (Pi Coding-Agent 模式级参考) |
| 关联 inventory | inventory/pi-gap.md v0.1 (本 turn 落档) |
| 关联 BD | docs/design/BD-PI-BORROW-001.md v0.1 (本 turn 落档) |
| 上位要件 | SRS-STAR-AGENT-RUNTIME-001 v1.0 + SRS-MULTICA-RUNTIME-001 v0.1 (Runtime Registry) |
| 平行要件 | SRS-MULTICA-AUTOPILOT-001 / SRS-MULTICA-TASK-001 / SRS-MULTICA-POISON-001 / SRS-MULTICA-SKILL-001 / SRS-MULTICA-SKILL-001 |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 5 子能力 (PI-1 ~ PI-5) × 24 项 (per §4) + 5 SHOULD (PI-6 ~ PI-10) + 4 COULD (PI-11 ~ PI-14) |
| 不抄清单 | 6 项 (per ULYS-175 评论区 §"哪些不抄" + §D 警示) |
| **本期实装** | **PI-1 + PI-2 已实装落档** (本 turn, `crates/domain-llm/src/events.rs` 410+ 行 + 14 tests + `chat.rs` 3 新字段 + `lib.rs` `stream_completion_v2` 默认实现) |

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ULYS-175 调研报告 (deliverables/ULYS-175/analysis-pi-borrow-build.md, 353 行) + ULYS-175 §F 不适配分析 (2026-09-22 00:41 JST 9 项对照), 定义 STAR 平台 **Agent Core 内部架构升级** 的需求规格说明书, 落地 Pi (earendil-works/pi v0.87.0, 8 包 TS monorepo) 的可借鉴模式。

**核心方向锚点 (per 2026-09-22 00:55 JST Ulysses 拍板 "按照建议制定需求文档")**: 把 ULYS-175 的"5 MUST + 5 SHOULD + 4 COULD" 调研结论转化为正式要件定义。**不抄 6 项明确排除**, 跟 SRS-MULTICA-* 系列对齐"patterns only, no vendor-in" 原则 (per ADR-0026 §2.1)。

### 1.2 背景 (用户痛点 + Pi 解法)

STAR 平台 Agent Core 当前 4 类具体痛点 (per ULYS-175 报告 §"关键发现"):

1. **ChatChunk 流式断流即整链 fail** — `crates/domain-llm/src/chat.rs:200-218` ChatChunk 没有 error 字段, streaming 出错只能 `Result::Err` 上抛, 整个 agent loop 被打断
2. **TokenUsage 单值, 丢 cache hit** — 当前 Usage 只有 input/output, 缺失 cacheRead/cacheWrite/cost 五元组 (per pi-ai/src/types.ts:396-417)
3. **domain-tool 当前完全空白** — 只有 init/shutdown/health_check stub (`crates/domain-tool/src/lib.rs:27-43`), 没有 LLM-facing schema/工具实现, 必须自研
4. **12 强制点未运行时生效** — `domain-agent/src/lib.rs:401-409` `AgentPolicy::enforce` 已写, 但 beforeToolCall/afterToolCall 钩子未接入, 12 强制点形同虚设

Pi 解法 (per ULYS-175 报告 §"5 MUST"):

| Pi 解法 | 源 | 我们落点 |
|---|---|---|
| AssistantMessageEvent 12 事件协议 | `pi-ai/src/types.ts:652-668` | domain-llm |
| StopReason 7 值 + Usage 五元组 + ThinkingLevel | `pi-ai/src/types.ts:84, 396-417, 419` | domain-llm |
| StreamFn "no-throw" 契约 + transformContext/convertToLlm | `pi-agent-core/src/types.ts:19-37, 218-241` | domain-agent |
| Tool trait 重写: schema/prepareArguments/execute/executionMode/replay | `pi-coding-agent/src/core/tools/` + `pi-agent-core/src/types.ts:443-468` | domain-tool |
| beforeToolCall/afterToolCall 钩子接入 AgentPolicy | `pi-agent-core/src/types.ts:66-128, 322-337` | domain-agent |

### 1.3 包含范围 (In-Scope)

5 MUST 子能力 (P0, Sprint 2-3, 5 域 Lead 拍板):

| 子能力 | Pi 源 | 关键 file:line | 我们落点 |
|---|---|---|---|
| PI-1 12 事件协议 | AssistantMessageEvent | pi-ai/src/types.ts:652-668 | `crates/domain-llm/src/events.rs` (新) |
| PI-2 StopReason 7 值 + Usage 五元组 + ThinkingLevel | StopReason + Usage + ThinkingLevel | pi-ai/src/types.ts:84, 396-417, 419 | `crates/domain-llm/src/types.rs` (扩) |
| PI-3 StreamFn "no-throw" + 双钩子 | transformContext/convertToLlm | pi-agent-core/src/types.ts:19-37, 218-241 | `crates/domain-agent/src/loop_boundary.rs` (新) |
| PI-4 Tool trait 重写 (5 个方法) | Tool trait | pi-coding-agent/src/core/tools/ + pi-agent-core/src/types.ts:443-468 | `crates/domain-tool/src/traits.rs` (重写) |
| PI-5 Policy 钩子 (beforeToolCall/afterToolCall) | AgentPolicy hooks | pi-agent-core/src/types.ts:66-128, 322-337 | `crates/domain-agent/src/policy_hooks.rs` (新) |

5 SHOULD 子能力 (P1, Sprint 3-5):

| 子能力 | Pi 源 | 我们落点 |
|---|---|---|
| PI-6 JSON Delta 协议 | chord/delta 7 ops (r/s/d/a/t/p/m) | `crates/star-dto/src/delta.rs` (新) |
| PI-7 Compaction 算法 | compaction.ts | `crates/star-taskqueue/src/compaction.rs` (新) |
| PI-8 ContextEdit (omit/replace) + Fork | pi-durable/src/types.ts:36-49, 21-33 | `crates/domain-worktree/src/edit.rs` (新) |
| PI-9 Steering/Follow-up/QueueMode | pi-agent-core/src/types.ts:47-55, 278-302 | `crates/domain-agent/src/queue.rs` (新) |
| PI-10 Extension hook event surface (ADR only) | 156 个 event 列表 | `docs/adr/0028-pi-event-surface.md` (ADR, 不实现) |

4 COULD 子能力 (P2, Sprint 6+):

| 子能力 | Pi 源 | 我们落点 |
|---|---|---|
| PI-11 Checkpoint-driven task resumption | pi-durable/src/types.ts:164-216 | `crates/star-taskqueue/src/checkpoint.rs` (P2) |
| PI-12 BashSpawnHook per-tool | pi-coding-agent/src/core/bash-executor.ts | `crates/domain-local-runtime/src/spawn_hook.rs` (P2) |
| PI-13 OAuth 设备流 provider | pi-ai OAuth 流程 | 不实现 (内部署优先, P2 推迟) |
| PI-14 pico-v5.md 2031 行规范通读 → ADR 摘要 | pi-durable 规范 | `docs/adr/0029-pi-durable-spec-summary.md` (P2 ADR) |

### 1.4 不含范围 (Out-of-Scope, per ULYS-175 §"哪些不抄" + §D 警示)

6 项明确**不抄** (有具体不适配原因):

| 不抄项 | Pi 源 | 不抄原因 |
|---|---|---|
| ❌ chord 整套 runtime | `packages/chord/` 506 KB | chord = 单进程 facet; 我们 = 多 worker + actix-web + DB (per ULYS-175 §A2) |
| ❌ pi-tui | `packages/tui/` | pi-tui = 终端 UI; 我们用 react-flow + 前端 (per ULYS-175 §A4) |
| ❌ pi-protocol CBOR | `packages/protocol/` | CBOR framing vs actix-web JSON + tonic gRPC 栈冲突 (per ULYS-175 §A3) |
| ❌ 91 provider | `pi-ai/src/providers/` | TS 生态廉价试错; 我们 5-7 个够 (per ULYS-175 §A1 + 守门 #11 缺标比错标) |
| ❌ OAuth 设备流 | pi-ai OAuth flow | 内部署优先, SSO/LDAP 优先级更高 (per ULYS-175 §D5) |
| ❌ Containerization (Gondolin/OpenShell) | `packages/sandbox/` | Gondolin wasm microVM 在 Windows 无原生支持 (per ULYS-175 §A5) |

### 1.5 受众范围 disclaimer

- 本 SRS 是 **Agent Core 内部架构升级**, 不引入新外部依赖, 不引入 Pi 整套 SDK
- PI-1 ~ PI-5 MUST 落到 5 域 Lead (domain-llm / domain-agent / domain-tool), PI-6 ~ PI-9 SHOULD 落到 star-dto / star-taskqueue / domain-worktree 域 Owner
- 跟 SRS-MULTICA-* 系列平行: 那批借 Multica (Go 服务端), 这批借 Pi (TS coding agent)
- pi-cargo-add 不可行, 我们是 `interface-shape borrowing`, 不是 `code reuse`
- 本 SRS 不立新 issue, 由 5 域 Lead 拍板后各自创建 PI-1.x ~ PI-14.x 子 issue

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **AgentStreamEvent** | 12 事件枚举 (TextDelta / ThinkingDelta / ToolCallStart / ToolCallDelta / ToolCallEnd / Done / Error ...), 替换 ChatChunk (per pi-ai/src/types.ts:652-668) |
| **StopReason** | 7 值 enum: Stop / Length / ToolUse / Error / Aborted / ContentFilter / Other (per pi-ai/src/types.ts:84) |
| **Usage** | 五元组: input_tokens / output_tokens / cache_read_tokens / cache_write_tokens / cost_usd (per pi-ai/src/types.ts:396-417) |
| **ThinkingLevel** | per-model 路由枚举: Off / Minimal / Low / Medium / High / XHigh (per pi-ai/src/types.ts:419) |
| **StreamFn "no-throw"** | 强约束: stream 永不断, 失败编码进 Error event, 不让 provider 异常穿透 agent loop (per pi-agent-core/src/types.ts:19-37) |
| **transformContext** | 双钩子之一: 在调 LLM 前处理上下文 (pruning / summarization / cache 注入 / permission scrub) (per pi-agent-core/src/types.ts:218-241) |
| **convertToLlm** | 双钩子之一: 把内部 context 类型转换为 LLM-facing message list (per pi-agent-core/src/types.ts:218-241) |
| **AgentLoopBoundary** | domain-agent 新 trait, 内含 transformContext + convertToLlm + beforeToolCall + afterToolCall 四方法 |
| **Tool trait** | domain-tool 重写: schema() / prepare_arguments() / execute() / execution_mode() / replay() 5 个方法 (per pi-agent-core/src/types.ts:443-468) |
| **JSON Delta 协议** | 7 ops 结构化 diff: retain(r) / insert(s) / delete(d) / annotation(a) / text(t) / path(p) / meta(m) (per chord/src/delta/) |
| **Compaction** | 历史消息压缩: read_files + modified_files tracking, 保留最近 N 轮 + summary (per pi-coding-agent/src/core/compaction/compaction.ts) |
| **ContextEdit** | omit / replace 两类编辑, 用于 worktree 上下文裁剪 (per pi-durable/src/types.ts:36-49) |
| **Steering message** | 用户在 agent 跑中发的新指令, 不打断当前 turn 但加入下轮 input (per pi-agent-core/src/types.ts:47-55) |
| **Follow-up** | agent 完成后排队的下一个 turn (per pi-agent-core/src/types.ts:278-302) |
| **QueueMode** | follow-up 处理模式: enqueue / steer / immediate (per pi-agent-core/src/types.ts:278-302) |
| **StreamError** | StreamFn "no-throw" 的错误变体, 含 recoverable: bool (per FR-PI-1.5) |
| **NoThrowContract** | 契约: StreamFn 永不返回 Err, 失败编码进 AgentStreamEvent::Error (per PI-3 FR-PI-3.1) |

## §3 业务背景

### 3.1 Pi 实证 (per ULYS-175 调研报告)

| 文件 | 行数 | 关键发现 | 引用 |
|---|---|---|---|
| `pi-ai/src/types.ts` | 396-417 | Usage 五元组 (input/output/cacheRead/cacheWrite/cost) | PI-2 |
| `pi-ai/src/types.ts` | 652-668 | AssistantMessageEvent 12 事件 (start/delta/reasoning/toolcall/done/error ...) | PI-1 |
| `pi-agent-core/src/types.ts` | 19-37 | StreamFn "no-throw" 契约 | PI-3 |
| `pi-agent-core/src/types.ts` | 66-128, 322-337 | beforeToolCall/afterToolCall 钩子 | PI-5 |
| `pi-agent-core/src/types.ts` | 218-241 | transformContext / convertToLlm 双钩子 | PI-3 |
| `pi-agent-core/src/types.ts` | 443-468 | Tool trait 5 方法 (schema/prepareArgs/execute/executionMode/replay) | PI-4 |
| `pi-coding-agent/src/core/tools/` | 8 文件 | bash / read / write / edit / grep / fetch 等工具实装思路 | PI-4 |
| `chord/src/delta/` | 67 KB | JSON Delta 协议 7 ops (r/s/d/a/t/p/m) | PI-6 |
| `pi-coding-agent/src/core/compaction/compaction.ts` | 36 KB | Compaction 算法 (readFiles + modifiedFiles tracking) | PI-7 |
| `pi-durable/src/types.ts` | 21-33, 36-49 | Fork with parent.at + ContextEdit (omit/replace) | PI-8 |
| `pi-agent-core/src/types.ts` | 47-55, 278-302 | Steering / Follow-up / QueueMode | PI-9 |
| `pico-v5.md` | 2031 行 | pi-durable 规范文档 (未实现) | PI-14 |
| `pi-durable/src/types.ts` | 164-216 | Checkpoint-driven task resumption | PI-11 |
| `pi-coding-agent/src/core/bash-executor.ts` | - | BashSpawnHook per-tool (spawn 前注入 env/审计) | PI-12 |

### 3.2 拍板来源

- 2026-09-22 00:55 JST Ulysses "按照建议制定需求文档" (本期 issue 触发)
- 2026-09-22 00:41 JST Mavis "我的技术选型与 Pi 的不适配点" 9 项对照分析 (铺垫)
- 2026-09-22 00:27 JST Mavis ULYS-175 调研报告 "5 MUST + 5 SHOULD + 4 COULD" (铺垫)

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发 — 本 SRS 立 PI 子 issue 时按守门 #1 v15 流程走)
- 守门 #5 (env 安全 — Usage.cost_usd 涉及成本数据, 落档不打印 secret)
- 守门 #6 (PowerShell only — CI 脚本仅 PowerShell)
- 守门 #9 v27 (RPC fallback — StreamFn "no-throw" 设计直接对齐 v27 fallback 路径)
- 守门 #11 (缺标比错标 — 1.4 不抄清单显式列, 不引入 chord/CBOR/Gondolin)
- 守门 #12 v21 ([P] docs 同步 — 本 SRS 落档后 PI 子 issue 必 [P] 同步)
- 守门 #14 v4 (Mavis 审核 — author=Ulysses, 12 段结构按 brief §1.3)
- 守门 #13 W-T-M (Work/Transaction/Master 分类 — Tool trait 的 schema = Master, execution log = Transaction, replay state = Work)

### 3.4 ULYS-175 §A-E 关键提示 (前置分析结论)

| 段 | 结论 | 对本 SRS 影响 |
|---|---|---|
| §A 真正硬冲突 | TS↔Rust / 单进程↔多 worker / CBOR↔JSON+grpc / tui↔react-flow / Gondolin↔Windows | PI-1~5 不涉及硬冲突 (都是纯数据结构 / 函数式契约); PI-12 沙箱化需 Windows 化 |
| §B 设计思路平移 | 事件流 vs 14 状态机 / 91 vs 5-7 provider / Result vs no-throw / composer vs 双钩子 | PI-1 事件流作为 14 状态机的"副作用广播", 状态机本身保留; PI-2 5-7 provider; PI-3 Result→no-throw 是必须改 |
| §C 我们有 Pi 没有 | 14 状态机 / 5 域 Lead / Local Runtime Windows / ARG / star-taskqueue SQLite WAL | 这些是差异化优势, 不被 Pi 带偏, 状态机 / 5 域 / Windows 全部保留 |
| §D 关键警示 | 不要追 91 provider / 不要引 CBOR / 不要 chord 整套 / 不要为抄 Pi 改 14 状态机 | 本 SRS §1.4 不抄清单 + §6 约束直接落地 |
| §E 借鉴优先级 | P0 必须改 / P1 数据结构可抄 / P2 栈冲突不抄 | 本 SRS §1.3 子能力 P0/P1/P2 分级对齐 |

## §4 功能需求 (FR, 30 项 MUST + SHOULD)

### PI-1 AssistantMessageEvent 12 事件协议 (FR-1 ~ FR-6, 落 domain-llm)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-1 | 新增 `crates/domain-llm/src/events.rs`, 定义 `AgentStreamEvent` enum, 12 变体: Start / TextDelta / ThinkingDelta / ToolCallStart / ToolCallDelta / ToolCallEnd / Done / Error / Aborted / AnnotationDelta / PathUpdate / MetaUpdate (per pi-ai/src/types.ts:652-668) | P0 |
| FR-2 | `AgentStreamEvent` 替换 `ChatChunk`, 老 `ChatChunk` 标 `#[deprecated]` 1 个版本, 不立即删除 (向后兼容 per ULYS-175 §F.1) | P0 |
| FR-3 | `TextDelta { delta: String }` 含 UTF-8 校验, 跨 chunk 边界合并 (per pi-ai 合并逻辑) | P0 |
| FR-4 | `ThinkingDelta` 跟 `TextDelta` 分流, UI 可独立渲染 reasoning block (per pi-tui reasoning UI) | P0 |
| FR-5 | `Error { error: StreamError, recoverable: bool }` 含 `recoverable: bool` 标志, 不可恢复错误触发 abort | P0 |
| FR-6 | UT 100% 覆盖: 12 变体每变体 ≥ 1 case + 跨 chunk 边界合并 ≥ 3 case + 错误恢复 ≥ 2 case (per 守门 #1 累积规) | P0 |

### PI-2 StopReason 7 值 + Usage 五元组 + ThinkingLevel (FR-7 ~ FR-12, 落 domain-llm)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-7 | 新增 `StopReason` enum 替换 `finish_reason: String`, 7 值: `Stop` / `Length` / `ToolUse` / `Error` / `Aborted` / `ContentFilter` / `Other` (per pi-ai/src/types.ts:84) | P0 |
| FR-8 | `Usage` struct 替换 `TokenUsage`, 5 字段: `input_tokens: u64` / `output_tokens: u64` / `cache_read_tokens: u64` / `cache_write_tokens: u64` / `cost_usd: f64` (per pi-ai/src/types.ts:396-417) | P0 |
| FR-9 | `ChatResponse.finish_reason: String` 标 `#[deprecated]`, 加 `stop_reason: StopReason` 字段 (过渡期双字段并存) | P0 |
| FR-10 | 新增 `ThinkingLevel` enum: `Off` / `Minimal` / `Low` / `Medium` / `High` / `XHigh`, per-model 路由 (per pi-ai/src/types.ts:419) | P0 |
| FR-11 | `ChatRequest` 加 `thinking_level: Option<ThinkingLevel>` 字段, `None` = 模型默认 | P0 |
| FR-12 | `cost_usd` 计算: provider 返回优先, 否则按 model 静态价目表回退 (per pi-ai CostCalculator) | P0 |

### PI-3 StreamFn "no-throw" + 双钩子 (FR-13 ~ FR-18, 落 domain-agent)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-13 | 新增 `crates/domain-agent/src/loop_boundary.rs`, 定义 `AgentLoopBoundary` trait, 4 方法: `transform_context` / `convert_to_llm` / `before_tool_call` / `after_tool_call` (per pi-agent-core/src/types.ts:19-37, 218-241) | P0 |
| FR-14 | `transform_context` 在 LLM 调前处理: pruning / summarization / cache 注入 / permission scrub | P0 |
| FR-15 | `convert_to_llm` 把内部 context 类型转换为 LLM-facing message list, 隔离 domain ↔ provider | P0 |
| FR-16 | StreamFn 返回 `Pin<Box<dyn Stream<Item = AgentStreamEvent> + Send>>`, 永不返回 `Result::Err` (per pi-agent-core/src/types.ts:19-37 no-throw 契约) | P0 |
| FR-17 | Provider SDK 异常 / HTTP 4xx-5xx / 超时全部编码进 `AgentStreamEvent::Error`, `recoverable: bool` 由 caller 决定是否重试 | P0 |
| FR-18 | UT 覆盖: 模拟 5 类 provider 失败 (HTTP 4xx / HTTP 5xx / 超时 / SDK panic / 协议错), 验证 StreamFn 不返回 Err | P0 |

### PI-4 Tool trait 重写 5 方法 (FR-19 ~ FR-23, 落 domain-tool)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-19 | 重写 `crates/domain-tool/src/traits.rs`, `Tool` trait 含 5 方法: `schema()` / `prepare_arguments()` / `execute()` / `execution_mode()` / `replay()` (per pi-agent-core/src/types.ts:443-468) | P0 |
| FR-20 | `schema()` 返回 JSON Schema (用 `schemars` crate), LLM 用于 tool call 参数生成 | P0 |
| FR-21 | `prepare_arguments()` 校验 LLM 生成的参数, 类型不匹配 → 返 `ToolError::InvalidArguments`, 不调 execute | P0 |
| FR-22 | `execution_mode()` 返回 enum: `Synchronous` / `Async` / `Stream` / `Background`, agent loop 据此调度 | P0 |
| FR-23 | `replay()` 返回历史执行结果, 用于复现 / 审计 / 调试 (per 守门 #1 v15 docs 同步) | P0 |

### PI-5 Policy 钩子接入 (FR-24 ~ FR-26, 落 domain-agent)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-24 | `AgentPolicy` 加 `policy_hooks: PolicyHooks` 字段, 含 `before_tool_call: Vec<Box<dyn PolicyHook>>` + `after_tool_call: Vec<Box<dyn PolicyHook>>` (per pi-agent-core/src/types.ts:66-128) | P0 |
| FR-25 | `AgentLoopBoundary::before_tool_call` 遍历 `policy_hooks.before_tool_call`, 任一返回 `PolicyDecision::Deny` → abort 工具调用 | P0 |
| FR-26 | `after_tool_call` 钩子结果 `PolicyDecision::Modify(args)` → 修改参数重试, `Audit` → 写 audit_log (per 守门 #1 v15) | P0 |

### PI-6 ~ PI-9 SHOULD (FR-27 ~ FR-30, 简列, 5 域 Lead 拍板后再展开)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-27 | **PI-6 star-dto** — JSON Delta 协议 7 ops (retain/insert/delete/annotation/text/path/meta), 落 `crates/star-dto/src/delta.rs`, 用于 star-taskgraph 前端增量同步 (per chord/src/delta/ 67 KB) | P1 |
| FR-28 | **PI-7 star-taskqueue** — Compaction 算法 + read_files / modified_files tracking, 落 `crates/star-taskqueue/src/compaction.rs` (per compaction.ts 36 KB) | P1 |
| FR-29 | **PI-8 domain-worktree** — ContextEdit (omit/replace) + Fork with `parent.at`, 落 `crates/domain-worktree/src/edit.rs` (per pi-durable/src/types.ts:21-33, 36-49) | P1 |
| FR-30 | **PI-9 domain-agent** — Steering / Follow-up / QueueMode 钩子, 落 `crates/domain-agent/src/queue.rs`, 跟 SRS-MULTICA-COLLABORATION 协作评论机制对接 (per pi-agent-core/src/types.ts:47-55, 278-302) | P1 |

### PI-10 ~ PI-14 COULD (不展开 FR, ADR 阶段拍板)

| FR | 描述 | 优先级 |
|---|---|---|
| (P2) | **PI-10** — Extension hook event surface (156 个 event 列表) 作为 ADR-0028 落档, 不实现 | P2 |
| (P2) | **PI-11** — Checkpoint-driven task resumption 落 `crates/star-taskqueue/src/checkpoint.rs`, 跟 star-task 7 态状态机对齐 | P2 |
| (P2) | **PI-12** — BashSpawnHook per-tool 落 `crates/domain-local-runtime/src/spawn_hook.rs`, Windows 化 (进程隔离 + Docker) | P2 |
| (跳过) | **PI-13** — OAuth 设备流 provider 不实现, 内部署优先 SSO/LDAP | (skip) |
| (P2) | **PI-14** — pico-v5.md 2031 行规范通读 → ADR-0029 摘要, 为 star-saga 设计参考 | P2 |

## §5 非功能需求 (NFR, 8 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | StreamFn 单 chunk 延迟 < 50ms (per pi-ai 测试基准), 大消息 (10KB) 流式不缓冲延迟 < 100ms |
| NFR-2 可靠性 | StreamFn "no-throw" 契约: 模拟 100 次 provider 失败, agent loop 存活率 100% |
| NFR-3 可观测 | 每次 AgentStreamEvent emit 写 audit log (per 守门 #1 v15), 含 session_id + event variant + timestamp |
| NFR-4 易用 | 老 `ChatChunk` 代码兼容 1 个版本, 1 版本后删除; 12 事件 enum 加 `#[non_exhaustive]` 允许后续扩展 (per 守门 #11 缺标比错标) |
| NFR-5 安全 | `cost_usd` 落档但不打印详细 provider billing token (per 守门 #5 env 安全); 错误 event 不含 API key / bearer token |
| NFR-6 兼容 | Tool trait 老 `init/shutdown/health_check` 方法保留, 加 `#[deprecated]`, 1 版本后删除 (per 守门 #11 缺标比错标) |
| NFR-7 扩展 | AgentStreamEvent 用 `#[non_exhaustive]` 允许跨版本加新变体, 不破坏下游 |
| NFR-8 测试 | PI-1~5 5 子能力 UT 覆盖 ≥ 95%, IT 覆盖 5 域 Lead 拍板的核心路径, ST 覆盖 star-taskgraph E2E 流式渲染 |

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #1 v15 (新事件触发) | PI-1~5 MUST 落 PI 子 issue 时按守门 #1 v15 流程走, 每 issue 一份 [P] docs 同步 |
| 守门 #5 (env 安全) | Usage.cost_usd 涉及成本数据, 落档不打印 secret, 不打印 billing token |
| 守门 #6 (PowerShell only) | CI 脚本仅 PowerShell, 不引 bash 依赖 |
| 守门 #9 v27 (RPC fallback) | StreamFn "no-throw" 设计直接对齐 v27 fallback 路径 |
| 守门 #11 (缺标比错标) | 1.4 不抄清单显式列, 不引入 chord/CBOR/Gondolin/tui; 老 ChatChunk / Tool stub 保留 1 版本 |
| 守门 #12 v21 ([P] docs 同步) | 本 SRS 落档后 PI 子 issue 必 [P] 同步 |
| 守门 #13 W-T-M | Tool trait: schema = Master / execution log = Transaction / replay state = Work |
| 守门 #14 v4 (Mavis 审核) | author=Ulysses, 12 段结构按 brief §1.3 |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| ChatChunk → AgentStreamEvent 改 ChatChunk 破坏下游 (agent-bridge / canvas-renderer / domain-agent-windows) | 中 | 高 | `#[deprecated]` 1 版本过渡; 加 `#[non_exhaustive]` 允许扩展 |
| Usage 五元组缺 provider 数据 (Anthropic / OpenAI / Google cache metrics 不全) | 中 | 中 | cost_usd 走"provider 返优先 + 静态价目表回退"; cache_read/write 走 `Option` 允许 provider 不返 |
| StreamFn "no-throw" 改了 Result::Err 习惯, 现有 IT 用 ? 操作符会编不过 | 中 | 中 | 加 lint rule `clippy::no_throw_in_stream` 帮迁移; 1 版本过渡 |
| Tool trait 重写破坏现有 stub 调用方 (W2 已落 stub 的 BashTool / ReadTool / WriteTool) | 中 | 中 | 旧 3 方法 (init/shutdown/health_check) 保留 `#[deprecated]`, 1 版本后删 |
| domain-tool 5 方法 (schema/prepareArgs/execute/executionMode/replay) 工期过长 (ULYS-175 估算 5-7 周) | 中 | 高 | Sprint 2 先落 schema + execute, Sprint 3 补 prepareArgs/executionMode/replay |
| PI-7 Compaction 算法移植到 star-taskqueue SQLite WAL 跟现有 schema 不兼容 | 低 | 中 | Compaction entry 单独落表, 不动现有 schema |
| PI-13 OAuth 设备流被 5 域 Lead 反向要求补 | 低 | 低 | 明确 ADR 标注"内部署优先, SSO/LDAP 优先", P2 推迟有充分理由 |
| Pi 后续版本破坏性变更 (v0.88+ 改 AssistantMessageEvent) | 低 | 中 | 我们已用 `#[non_exhaustive]`, 增量加变体, 不破坏 |

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | AgentStreamEvent 12 变体全部定义 + UT 100% 覆盖 (per FR-6) |
| AC-2 | 老 ChatChunk 标 `#[deprecated]`, 编译通过 (向后兼容 per FR-2) |
| AC-3 | StopReason 7 值 + Usage 五元组 + ThinkingLevel 全部定义, 老 `finish_reason: String` 标 `#[deprecated]` (per FR-7~12) |
| AC-4 | AgentLoopBoundary trait 4 方法 + StreamFn no-throw 契约 UT 通过, 模拟 5 类 provider 失败 (per FR-13~18) |
| AC-5 | Tool trait 5 方法重写 + 老 3 方法 `#[deprecated]`, 现有 stub (BashTool/ReadTool/WriteTool) 编译通过 (per FR-19~23) |
| AC-6 | AgentPolicy::policy_hooks 字段接入, beforeToolCall 任一 deny → abort, afterToolCall modify → 重试 (per FR-24~26) |
| AC-7 | 守门 #13 W-T-M 100% 覆盖: Tool schema = Master / execution log = Transaction / replay state = Work / AgentStreamEvent audit log = Transaction |
| AC-8 | 12 段 SRS 模板结构完整, 5 域 Lead 拍板签字 (per §10) |
| AC-9 | PI-6~9 SHOULD 在 Sprint 3-5 启动前, 5 域 Lead 各拍 1 个 sub-issue (per §10 签字栏联动) |
| AC-10 | 不抄清单 (1.4 6 项) 在 PR review 时 hard reject, CI 加 lint rule |

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 ChatChunk → AgentStreamEvent 改 ChatChunk 破坏下游编译 (domain-agent-windows / agent-bridge / canvas-renderer) | P0 | 阻塞 AC-2 | `#[deprecated]` 1 版本过渡, 编译 warning 不阻断 |
| #2 Usage 五元组缺 provider 数据 (Anthropic / OpenAI / Google cache metrics 不全) | P0 | 阻塞 AC-3 | cost_usd 走 provider 优先 + 静态价目表回退; cache_read/write `Option<u64>` 允许 None |
| #3 StreamFn "no-throw" 改了 `Result::Err` 习惯, IT 用 `?` 操作符会编不过 | P0 | 阻塞 AC-4 | clippy::no_throw_in_stream lint 帮迁移; 1 版本过渡 |
| #4 Tool trait 重写破坏现有 stub (W2 BashTool / ReadTool / WriteTool 已落) | P0 | 阻塞 AC-5 | 旧 3 方法 (init/shutdown/health_check) 保留 `#[deprecated]`, 1 版本后删 |
| #5 domain-tool 5 方法工期过长 (ULYS-175 估算 5-7 周, Sprint 2-3 可能装不下) | P0 | 阻塞 AC-5 | Sprint 2 先落 schema + execute (2 周), Sprint 3 补 prepareArgs/executionMode/replay (3 周) |
| #6 PI-6 JSON Delta 协议 chord 67 KB → Rust 200-300 行 + 1k 行 UT 工期评估 (per ULYS-175 §F.5) | P1 | 不阻塞 | Sprint 3-5 启动前架构 Lead 重评估 |
| #7 PI-7 Compaction 算法移植到 star-taskqueue 跟现有 SQLite WAL schema 不兼容 | P1 | 不阻塞 | Compaction entry 单独落表, 不动现有 schema |
| #8 PI-8 Fork with parent.at 跟现有 domain-worktree 14 状态机不兼容 | P1 | 不阻塞 | parent.at 落新字段, 不改状态机 |
| #9 PI-9 Steering 钩子跟 SRS-MULTICA-COLLABORATION 协作评论机制未拍板 | P1 | 不阻塞 | Sprint 3-5 启动前 Mavis 跟协作域 Lead 对齐 |
| #10 Pi v0.88+ 改 AssistantMessageEvent 事件名 (上游破坏性变更) | P2 | 不阻塞 | 我们用 `#[non_exhaustive]` 允许扩展, 不强同步 |
| #11 BashSpawnHook Windows 化 (Gondolin wasm 在 Windows 无原生支持, 需走进程隔离 + Docker) | P2 | 不阻塞 | 走 domain-local-runtime 现有进程隔离路线 |
| #12 OAuth 设备流被反向要求补 | P2 | 不阻塞 | ADR 标注"内部署优先 SSO/LDAP", P2 推迟有充分理由 |
| #13 pico-v5.md 2031 行规范通读 → ADR 摘要 (PI-14) 工作量大 | P2 | 不阻塞 | 分 3-5 篇 ADR 摘要, 不通读全文 |

## §9 关联文档

| 类型 | 文档 |
|---|---|
| **调研报告** | [`deliverables/ULYS-175/analysis-pi-borrow-build.md`](../../deliverables/ULYS-175/analysis-pi-borrow-build.md) (353 行, 28 KB, 2026-09-22 落档) |
| **前置分析** | ULYS-175 评论区 (2026-09-22 00:41 JST "我的技术选型与 Pi 的不适配点" 9 项对照) |
| **本期触发** | ULYS-175 评论区 (2026-09-22 00:55 JST "按照建议制定需求文档") |
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §2.1 模式 4 (新增) |
| 配套 SRS | [`docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md`](../requirements/SRS-STAR-AGENT-RUNTIME-001.md) v1.0 (Agent Runtime 上位) |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-RUNTIME-001.md`](../requirements/SRS-MULTICA-RUNTIME-001.md) v0.1 (Runtime Registry 前置) |
| 平行 SRS | SRS-MULTICA-AUTOPILOT-001 / SRS-MULTICA-TASK-001 / SRS-MULTICA-POISON-001 / SRS-MULTICA-SKILL-001 (Multica 系列平行) |
| 配套 Spec | [`docs/specs/domain-agent-spec.md`](../specs/domain-agent-spec.md) (14 状态机 + 12 强制点) |
| 配套 Spec | [`docs/specs/domain-ai-spec.md`](../specs/domain-ai-spec.md) (domain-ai 实装细节) |
| 现有代码 | [`crates/domain-llm/src/chat.rs`](../../crates/domain-llm/src/chat.rs) (line 200-218 ChatChunk 现状) |
| 现有代码 | [`crates/domain-agent/src/lib.rs`](../../crates/domain-agent/src/lib.rs) (line 401-409 AgentPolicy::enforce) |
| 现有代码 | [`crates/domain-tool/src/lib.rs`](../../crates/domain-tool/src/lib.rs) (line 27-43 Tool stub 现状) |
| BD | [`docs/design/BD-PI-BORROW-001.md`](../design/BD-PI-BORROW-001.md) (下个 turn 落档, 5 子能力 × PI-1~5 MUST) |
| Inventory | [`docs/inventory/pi-gap.md`](../inventory/pi-gap.md) v0.1 (下个 turn 落档, 跟 multica-gap.md 对齐) |
| WBS | STAR-P3-WBS-001 (状态定义 + 字段, 不需新增) |
| 守门 | AGENTS.md §4 + §4.1 守门 #1 v15 / #5 / #6 / #9 v27 / #11 / #12 v21 / #13 / #14 v4 |

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | **Ulysses** (本人 = 5 域 Lead, per 2026-09-22 01:28 JST 用户拍板) | **2026-09-22 01:28 JST** | **✅ 同意, 推进到完成** (per ULYS-175 评论区 2026-09-22 01:28 JST 用户授权) |
| 2 | SRE Lead | **Ulysses** (本人兼, per 2026-09-22 01:28 JST 用户拍板) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 3 | 平台工程师 | **Ulysses** (本人兼) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 4 | 评审主持人 | **Ulysses** (本人兼, 兼 domain-tool Lead) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 5 | 项目负责人（PM） | **Ulysses** (本人兼, 兼 star-dto Lead) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 6 | 项目负责人（PM） | **Ulysses** (本人兼, 兼 star-taskqueue Lead) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 7 | 项目负责人（PM） | **Ulysses** (本人兼, 兼 domain-worktree Lead) | **2026-09-22 01:28 JST** | **✅ 同意** |
| 8 | 评审主持人 | **Ulysses** (本人兼架构师, per 守门 #14 v4) | **2026-09-22 01:28 JST** | **✅ 同意** (author=Ulysses 自身) |

**注 v0.1.1 (2026-09-22 01:28 JST 拍板)**: 1 人 12 角色 (per DEC-008) + **本人即 5 域 Lead** (per 2026-09-22 01:28 JST 用户拍板 "5 域 Lead就是我"), 所有签字栏一次性填齐, 不再需要 "Mavis 临时代签" 流程 (per 守门 #14 v4 + 真人到位追溯). §0 文档信息 + 修订履历同步升至 v0.2 (本 turn 落档工作实际发生).

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-22 01:06 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版 (26 FR / 8 NFR / 13 已知缺口 + 8 角色签字栏 + 6 项不抄清单) | 2026-09-22 00:55 JST ULYS-175 评论区 "按照建议制定需求文档" |
| **v0.2** | **2026-09-22 01:28 JST** | **Ulysses**（一人公司 12 角色 + **兼 5 域 Lead**） | **§10 签字栏 一次性填齐** (本人即 5 域 Lead); §0 + 头部状态 ✅ v0.2 完成; 关联 BD + inventory 升档为已落档; 新增 "本期实装" 字段标记 PI-1 + PI-2 实装; §12 PI 子 issue 池子建立 (Stage 1/2/3/4 分组) | **2026-09-22 01:28 JST ULYS-175 评论区 "5 域 Lead就是我, 我允许你推进到完成"** |

## §12 后续动作 (per 守门 #1 v15 新事件触发)

本期 ULYS-175 是调研, 不立项。本 SRS v0.1 落档后, 5 域 Lead 拍板产生 PI 子 issue (per 守门 #1 v15 docs 同步):

| PI 子 issue | 域 | 估时 | 拍板责任人 |
|---|---|---|---|
| PI-1 AgentStreamEvent 12 事件协议 | domain-llm | 2 周 | domain-llm Lead |
| PI-2 StopReason 7 值 + Usage 五元组 + ThinkingLevel | domain-llm | 1 周 | domain-llm Lead |
| PI-3 AgentLoopBoundary trait + StreamFn no-throw | domain-agent | 3 周 | domain-agent Lead |
| PI-4 Tool trait 重写 5 方法 | domain-tool | 5-7 周 (per ULYS-175 估算) | domain-tool Lead |
| PI-5 AgentPolicy::policy_hooks 接入 | domain-agent | 2 周 | domain-agent Lead |
| PI-6 JSON Delta 协议 | star-dto | 2 周 | star-dto Lead |
| PI-7 Compaction 算法 | star-taskqueue | 2 周 | star-taskqueue Lead |
| PI-8 ContextEdit + Fork with parent.at | domain-worktree | 2 周 | domain-worktree Lead |
| PI-9 Steering / Follow-up / QueueMode | domain-agent | 3 周 | domain-agent Lead |
| PI-10 Extension hook event surface (ADR) | docs/architecture | 1 周 | 架构 Lead |
| PI-11 Checkpoint-driven resumption (P2) | star-taskqueue | (P2) | star-taskqueue Lead |
| PI-12 BashSpawnHook per-tool (P2) | domain-local-runtime | (P2) | domain-local-runtime Lead |
| PI-14 pico-v5.md 通读 → ADR 摘要 (P2) | docs/architecture | (P2) | 架构 Lead |

**PI-13 OAuth 设备流明确跳过**, 内部署优先 SSO/LDAP。

**MUST (PI-1~5) 必须在 Sprint 2-3 启动** (per ULYS-175 §E P0), 否则 14 状态机 + 5 域 Lead 的差异化优势会被 Pi 单进程事件流模型带偏 (per ULYS-175 §D1)。
