# SRS-MULTICA-POISON-001

> **Multica Session Poisoning 域要件定義書 v0.1** (per ADR-0026 v0.2 §1.1 抽象, 跟 v33 候选配套)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-POISON-001.md`](../design/BD-MULTICA-POISON-001.md) (下个 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 (v33 候选 配套)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> - 日期: 2026-09-11 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-POISON-001 |
| 文书名 | Multica Session Poisoning 域要件定義書 (v33 候选配套) |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.1 抽象表 "Poisoned 语义" |
| 关联 inventory | inventory §2.2 v33 候选 (跟 SRS-MULTICA-TASK-001 配套) |
| 上位要件 | SRS-MULTICA-TASK-001 (Task Lifecycle 是 Session Poison 的前置) |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 3 子能力 (PS-1 ~ PS-3) × 14 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.1 抽象 (Poisoned 语义: **(agent, issue) session 不可恢复**, 4 类原因) + inventory §2.2 v33 候选配套, 定义 STAR 平台 **Multica Session Poisoning 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: 不只是"探测失败标 unusable" (per v0.1 旧理解), 而是"`(agent, issue) session` resume 必然坏, 4 类原因分类, 下次 fresh session 起步" (per v0.2 修正 2 + `poisoned.go:10-217` 实证)。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 3 类具体痛点:

1. **同 session 反复 resume 烂状态** — Mavis 接到 task, 之前的 `(agent, issue) session` 已坏, 但仍 resume, 必失败
2. **Provider-specific 错误无分类** — Anthropic 400 invalid_request_error / Codex semantic inactivity / Codex resume oversized 各自处理, 没集中
3. **Output 包含 fallback marker 仍当 success** — subagent 报 "I reached the iteration limit" 或 "put your final update inside the content string" 但 status=succeeded (per Multica `poisonedMarkers`)

### 1.3 包含范围 (In-Scope)

3 子能力 (per Multica `poisoned.go:10-217`):

| 子能力 | Multica 源 | 关键 file:line |
|---|---|---|
| PS-1 4 类 poison 原因分类 | poisoned.go:10-46 (FailureReason × 5) | IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized |
| PS-2 4 classify 函数 | poisoned.go:71-217 | classifyPoisonedOutput / classifyPoisonedError / classifyResumeUnsafeTimeout / classifyResumeUnsafeTransport |
| PS-3 GetLastTaskSession 过滤 (Mavis-side) | 自定 (跟 Multica `taskfailure` package 对齐) | session_poisoned=true 时 fresh session |

### 1.4 不含范围 (Out-of-Scope)

- ❌ Multica `taskfailure` 整套 (有 21 个 reason, 本 SRS 仅用 5 个)
- ❌ Multica server-side `GetLastTaskSession` SQL (本 SRS 走 WBS row `session_poisoned` 字段)
- ❌ Resume 协议 (Multica ACP resume) — Mavis 走 fresh session 起步

### 1.5 受众范围 disclaimer

- 本 SRS 跟 SRS-MULTICA-TASK-001 强绑定, WBS row `session_poisoned` 字段在两个 SRS 都引用
- "Session" 在本 SRS = `(agent_id, issue_id) session pair`, **不等于** Mavis root session

---

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Session pair** | `(agent_id, issue_id) session`, 一个 subagent + 一个 task 的会话 |
| **Session poisoned** | 该 session pair resume 必然坏, 标 `session_poisoned: bool = true` |
| **IterationLimit** | 原因 1: subagent 报 "I reached the iteration limit" |
| **AgentFallbackMsg** | 原因 2: subagent 报 "put your final update inside the content string" |
| **APIInvalidRequest** | 原因 3: LLM API 返 400 invalid_request_error (含 image 超大) |
| **CodexSemanticInactivity** | 原因 4: Codex semantic inactivity 标记, session stuck 无进展 |
| **CodexResumeOversized** | 原因 5: Codex thread/resume 响应溢出 stdout line buffer, 每次 resume 必溢出 |
| **Fresh session** | 新 session pair 起步, 不 resume 旧的 |
| **GetLastTaskSession** | 找该 `(agent_id, issue_id)` 上一次成功完成的 session, 跳过 poisoned (per Multica) |

---

## §3 业务背景

### 3.1 Multica 实证 (per ADR-0026 v0.2 §1.1 + §1.3 修正 2)

| 文件:line | 关键 |
|---|---|
| `poisoned.go:10-46` | 5 类 FailureReason 集中 (IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized) |
| `poisoned.go:57-105` | `classifyPoisonedOutput` (output 包含 fallback marker, length cap 320) |
| `poisoned.go:107-170` | `classifyPoisonedError` (4 类: image dimensions + API 400 invalid_request_error + unresumable history) |
| `poisoned.go:172-201` | `classifyResumeUnsafeTransport` (Codex resume oversized, provider-specific) |
| `poisoned.go:203-217` | `classifyResumeUnsafeTimeout` (Codex semantic inactivity) |
| `poisoned.go:10-15` | GetLastTaskSession 过滤 (本 SRS 走 WBS row 字段, 简化) |

**关键设计 (per Multica)**:
- `poisonedOutputMaxLen = 320` (cap 避免长 output 引用 marker 误判)
- 4 classify 函数互补, 不会重复分类
- 跟 `taskfailure` package 共享 canonical reason (MUL-2946)
- provider-specific (Codex) 错误单独分类

### 3.2 拍板来源

- 2026-09-11 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 + inventory_opt1 + code_opt1

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发)
- 守门 #5 (env 安全)
- 守门 #6 (PowerShell only)
- 守门 #9 v27 (RPC fallback) — session 不可恢复时直接 fresh session 起步
- 守门 #11 (缺标比错标) — session_poisoned 不删标
- 守门 #12 v21 ([P] docs 同步) — 每次 poison 检测落 log
- 守门 #14 v4 (Mavis 审核)

---

## §4 功能需求 (FR, 14 项)

### PS-1 5 类 poison 原因分类 (FR-1 ~ FR-5)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-1 | `IterationLimit` 原因: subagent output 包含 "i reached the iteration limit" (case-insensitive) | poisoned.go:67 | P0 |
| FR-2 | `AgentFallbackMsg` 原因: subagent output 包含 "put your final update inside the content string" (case-insensitive) | poisoned.go:68 | P0 |
| FR-3 | `APIInvalidRequest` 原因: LLM API 返 400 + "invalid_request_error" + (image dimensions exceed max + image.source.base64.data) | poisoned.go:146-168 | P0 |
| FR-4 | `CodexSemanticInactivity` 原因: Codex 报 semantic inactivity 或 first turn no progress marker | poisoned.go:207-216 | P0 (Codex only, 其他 provider N/A) |
| FR-5 | `CodexResumeOversized` 原因: Codex thread/resume 响应溢出 stdout line buffer | poisoned.go:193-200 | P0 (Codex only) |

### PS-2 4 classify 函数 (FR-6 ~ FR-10)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-6 | `classify_poisoned_output` (本地版): output 字符串 keyword match + length cap (320 字符) | poisoned.go:77-105 | P0 |
| FR-7 | `classify_poisoned_error` (本地版): error 字符串 keyword match (400 + invalid_request_error + image 维度) | poisoned.go:131-170 | P0 |
| FR-8 | `classify_resume_unsafe_timeout` (本地版): provider-specific, Codex 报 inactivity | poisoned.go:207-217 | P0 |
| FR-9 | `classify_resume_unsafe_transport` (本地版): provider-specific, Codex 报 resume oversized | poisoned.go:193-201 | P0 |
| FR-10 | 4 classify 函数互补, 不会重复分类 (per Multica 注释) | poisoned.go:71-217 | P0 |

### PS-3 GetLastTaskSession 过滤 (FR-11 ~ FR-14)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-11 | subagent dispatcher 接到 task 时, 检查 `(agent_id, issue_id) session pair` 是否 poisoned | P0 |
| FR-12 | `session_poisoned: true` 时, dispatcher 自动 fresh session 起步, 不 resume 旧 session | P0 |
| FR-13 | `session_poisoned: false` (默认) 时, dispatcher resume 旧 session (per Multica 默认行为) | P0 |
| FR-14 | 每次 poison 检测 + 标 session_poisoned 写 audit log (per 守门 #1 v15) | P0 |

---

## §5 非功能需求 (NFR, 4 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | classify 函数 < 10ms (本地 keyword match) |
| NFR-2 可靠性 | 误判率 < 1% (per Multica `poisonedOutputMaxLen = 320` 防长 output 误判) |
| NFR-3 可观测 | 每次 poison 检测 + 标 session_poisoned 写 audit log |
| NFR-4 易用 | dispatcher 自动处理, Mavis 不需要手动标 |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #5 env 安全 | 不读 env 值 |
| 守门 #11 缺标比错标 | session_poisoned 不删标 |
| 守门 #12 v21 [P] docs 同步 | 每次 poison 检测落 log |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Codex 错误分类不准确 (其他 provider 误判 Codex-only reason) | 中 | 中 | FR-8 + FR-9 显式 provider-specific, 其他 provider 跳过 |
| Length cap 320 字符误判长 output | 低 | 中 | per Multica 经验值, 落档后看 1 周用例 |
| 4 classify 函数优先级冲突 | 低 | 中 | Multica 实证互补, 本 SRS 沿用 |
| session_poisoned 标了之后没及时清, 阻塞正常 task | 低 | 中 | 30 天后自动清 (per Multica) + 周报提醒 |

---

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | 5 类 reason (IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized) 全部能识别 |
| AC-2 | 4 classify 函数全部跑通, 各 1 case |
| AC-3 | session_poisoned=true 时, dispatcher 自动 fresh session 起步 |
| AC-4 | Length cap 320 字符验证 (长 output 不误判) |
| AC-5 | Codex-only reason (SemanticInactivity / ResumeOversized) 在非 Codex provider 跳过 |
| AC-6 | 每次 poison 检测写 audit log |
| AC-7 | 跟 SRS-MULTICA-TASK-001 配套, WBS row `session_poisoned` 字段落档 |

---

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 Multica `taskfailure` 21 个 reason 中其他 16 个 (e.g. rate_limit, network_error) 不在本 SRS | P1 | 不阻塞 (5 类够用) | 后续 v41+ 扩 |
| #2 Codex-only reason 在非 Codex provider N/A, 但 mavis 跑 mcode 时不命中 | P0 | 不阻塞 (mcode 不报 Codex 错) | 文档加 disclaimer |
| #3 4 classify 函数没 fallback chain (Multica 注释说不会重复, 但本 SRS 简化版可能漏) | P0 | 阻塞 AC-2 | BD 阶段详 classify 流程图 |
| #4 Length cap 320 字符是经验值, 没实证 | P1 | 不阻塞 | 落档后看 1 周用例, 调阈值 |
| #5 GetLastTaskSession 走 WBS row 字段 (per FR-11), 跟 Multica SQL 走法不同 | P0 | 不阻塞 (Mavis 单一 root session) | 文档加 disclaimer |
| #6 session_poisoned 标了之后, 30 天自动清的策略跟守门 #11 "不删" 冲突 | P0 | 阻塞 AC-7 | 改为"30 天后 deprecated, 永久不删" |

---

## §9 关联文档

| 类型 | 文档 |
|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.1 抽象表 + §1.3 修正 2 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 v33 候选 配套 |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-TASK-001.md`](../requirements/SRS-MULTICA-TASK-001.md) (Task Lifecycle 强绑定) |
| BD | [`docs/design/BD-MULTICA-POISON-001.md`](../design/BD-MULTICA-POISON-001.md) (下个 turn 落档) |
| Multica 源 | `server/internal/daemon/poisoned.go:10-217` (5 reason + 4 classify) |
| 守门 | AGENTS.md §4 + §4.1 #1 v15 / #5 / #6 / #9 v27 / #11 |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（14 FR / 4 NFR / 6 已知缺口 + 5 角色签字栏） | 2026-09-11 20:43 JST ask_user 选项 form_opt2 |
