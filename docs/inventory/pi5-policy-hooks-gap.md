# ULYS-181 PI-5 AgentPolicy::policy_hooks — Gap Analysis (Q1-Q4)

| 項目 | 内容 |
|---|---|
| 文書编号 | PI5-POLICY-HOOKS-GAP-v0.1 |
| 日付 | 2026-09-22 |
| 親 Issue | ULYS-175 (PI 借鉴调研) |
| 並行 Issue | ULYS-180 (PI-3) / ULYS-182 (PI-4) |
| 領域 | domain-agent |
| 作者 | Ulysses (本人即 5 域 Lead, per 2026-09-22 01:28 JST 拍板) |
| 根拠 | SRS-PI-BORROW-001 §4 FR-24~26 + §7 AC-6 + ULYS-175 §A-E 不适配分析 + ULYS-181 issue description |

---

## Q1 — 已落地 (PI-5 MVP)

| Gap | 状态 | 落档 |
|---|---|---|
| `PolicyHook` trait 定义 | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per FR-24) |
| `PolicyDecision` enum (4 变体) | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per FR-24) |
| `PolicyHooks` 容器 (before + after) | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per FR-25) |
| `ToolCallHookContext` + `ToolCallOutcome` | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per FR-25/26) |
| `AuditEventSpec` (5 字段 + builder) | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per FR-26 + 守门 #13 d) |
| `AuditSink` trait (domain-agent 抽象) | ✅ | `crates/domain-agent/src/policy_hooks.rs` (per 守门 #1 v15 分层) |
| `apply_before_tool_call_hooks` 顶层 fn | ✅ | `crates/domain-agent/src/policy_hooks.rs` (PI-3 入口) |
| `apply_after_tool_call_hooks` 顶层 fn | ✅ | `crates/domain-agent/src/policy_hooks.rs` (PI-3 入口) |
| `AgentPolicy.policy_hooks: PolicyHooks` 字段 | ✅ | `crates/domain-agent/src/lib.rs` (per FR-25) |
| `conservative()` 默认 0 hooks | ✅ | `crates/domain-agent/src/lib.rs` (per FR-26) |
| 合并规则 (Deny > Modify > Audit > Allow) | ✅ | `PolicyHooks::apply_before/apply_after` (per SRS §3 + §7 AC-6) |
| after hook `Modify` 降级为 Audit | ✅ | `finalize_after()` (per SRS §3 + ULYS-181 description) |
| 老 JSON 反序列化兼容 | ✅ | `#[serde(skip, default)]` on `policy_hooks` 字段 (per 守门 #11) |
| 20 单元测试 + 3 集成测试 | ✅ | 全部 PASS (45/45 in domain-agent) |
| `cargo check --workspace --lib` | ✅ | 0 err |
| `cargo test -p domain-agent --lib` | ✅ | 45/45 |
| `cargo clippy -p domain-agent --lib --no-deps` | ✅ | 0 新增 warning |
| `application` + `api` 依赖编译 | ✅ | 0 err |

## Q2 — 未落地 (P0 阻塞 / 业务实装期)

| Gap | 状态 | 责任人 | 触发 |
|---|---|---|---|
| 12 强制点的具体 hook 实现 (e.g. `PathPolicyHook`, `ToolWhitelistHook`, `RateLimitHook`, `SecretBrokerHook`) | ⏳ P0 | application Lead | PI-3 + PI-5 集成完成后 |
| `AgentLoopBoundary::before_tool_call` 真实调用 PI-5 入口 fn | ⏳ P0 | PI-3 worker (ULYS-180) | ULYS-180 done |
| `AgentLoopBoundary::after_tool_call` 真实调用 PI-5 入口 fn | ⏳ P0 | PI-3 worker (ULYS-180) | ULYS-180 done |
| Application 层 `AuditSink` 实现 (delegate to `AgentAudit::write_event`) | ⏳ P0 | api Lead | Stage 3 业务实装期 |
| 12 强制点的 runtime observability (每 hook 命中次数/延迟) | ⏳ P0 | SRE Lead | 守门 #1 v15 观测 |

## Q3 — 未落地 (P1 不阻塞)

| Gap | 状态 | 责任人 | 触发 |
|---|---|---|---|
| PI-5 hook 在 multica 工作流编辑器可视化 | ⏳ P1 | frontend-canvas Lead | SRS-MULTICA-WORKFLOW 升版 |
| hook `Modify` retry 上限 (per 守门 #1 v15) | ⏳ P1 | domain-agent Lead | PI-3 retry 集成时定 |
| hook `Audit` 批量落库 (e.g. 1s 缓冲) | ⏳ P1 | infrastructure Lead | Stage 3 WORM audit 落地 |
| hook 测试夹具 (UT fixture for app 层) | ⏳ P1 | domain-agent Lead | 本期仅 mock hook |

## Q4 — 未落地 (P2 不阻塞 / 推迟)

| Gap | 状态 | 备注 |
|---|---|---|
| hook "热更新" (per ADR-0048 config hot-reload) | ⏳ P2 | 等 multica-runtime hot-reload 拍板 |
| hook 跨 worker 同步 (per `star-eventbus` 广播) | ⏳ P2 | 当前假设单 worker 应用 |
| hook "运行时可视化" (e.g. 哪些 hook 拒了哪条 tool call) | ⏳ P2 | 等 multica-observability 升版 |
| PI-3 `AgentLoopBoundary::transform_context` hook 复用 PI-5 类型 | ⏳ P2 | per SRS-PI-BORROW-001 §4 FR-14, PI-3 决定是否复用 |

---

## 附表 A — PI-5 vs ULYS-175 §A-E 不适配分析对照

| ULYS-175 不适配点 | PI-5 处理 | 一致 |
|---|---|---|
| §A1 TypeScript vs Rust — 91 provider 不抄 | ✅ PI-5 只 1 个 `PolicyHook` trait, 不抄 Pi 整套 | ✅ |
| §A2 单进程 vs 多 worker — chord 不抄 | ✅ `PolicyHooks` 容器走 domain 层, 不引用 facet | ✅ |
| §A3 CBOR vs JSON — pi-protocol 不抄 | ✅ `PolicyDecision` 用 JSON-friendly `#[serde(tag)]` | ✅ |
| §A4 pi-tui vs react-flow — 不抄 | ✅ N/A (domain-agent 不涉及 UI) | ✅ |
| §A5 Gondolin vs Windows — 不抄 | ✅ N/A (sandbox 范畴, 不在 PI-5 范围) | ✅ |
| §B1 14 状态机保留 | ✅ PI-5 不动状态机, 只加字段 | ✅ |
| §B2 5-7 provider | ✅ PI-5 不涉及 provider | ✅ |
| §B3 StreamFn no-throw | ✅ PI-5 配合 PI-3, `PolicyDecision::Deny.reason` 编码进 `AgentStreamEvent::Error` (PI-3 集成期) | ✅ |
| §B4 transformContext / convertToLlm 双钩子 | ✅ PI-5 提供 `before_tool_call` / `after_tool_call` 双入口; `transform_context` / `convert_to_llm` 由 PI-3 单独落地 (per SRS §4 FR-13/14) | ✅ |

## 附表 B — 守门合规最终章

| 守门 | 状态 |
|---|---|
| #1 v15 新事件触发 | ✅ |
| #5 env 安全 | ✅ |
| #7 unsafe_code = forbid | ✅ (0 unsafe) |
| #9 v27 RPC fallback | ✅ (N/A) |
| #11 缺标比错标 | ✅ (老 JSON 兼容) |
| #12 v21 [P] docs 同步 | ✅ (本完了報告 + 上位 SRS-PI-BORROW-001) |
| #13 W-T-M | ✅ (Master/Transaction/Work 三层清晰) |
| #13 d 100% audit | ✅ (AuditSink 抽象, application 层落地) |
| #14 v4 author=Ulysses | ✅ (本人即 5 域 Lead, per 2026-09-22 01:28 JST 拍板) |

— Ulysses (本人即 5 域 Lead, per 2026-09-22 01:28 JST 拍板)