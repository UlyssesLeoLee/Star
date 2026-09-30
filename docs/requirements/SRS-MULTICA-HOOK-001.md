# SRS-MULTICA-HOOK-001

> **Multica Hook 域要件定义书 v0.5.4** (沿用 Advanced Settings Hooks tab；规定 Phase 9D summary v2 可联合读取 Hook ledger 与 RunEvent、按稳定 event ID 去重并关联最新 Run 状态，但仍明确 partial/unknown coverage 与未接入 producer)

> - 状态: 🟡 Draft v0.5.4 (2026-10-01 JST，Phase 9D Run state read-model contract)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 issue: ULYS-235 ("hook需求")
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-HOOK-001.md`](../design/BD-MULTICA-HOOK-001.md) v0.5.6
> - 关联詳細設計書: [`docs/detailed-design/DD-MULTICA-HOOK-001.md`](../detailed-design/DD-MULTICA-HOOK-001.md) v0.5.12
> - 平行 SRS: [`docs/requirements/SRS-MULTICA-SKILL-001.md`](../requirements/SRS-MULTICA-SKILL-001.md) v0.1 (skills 域)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.3 5 类扩展点 (commands / agents / skills / hooks / MCP)
> - 拍板来源: 2026-09-24 20:xx JST Ulysses "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计"
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-30 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-HOOK-001 |
| 文书名 | Multica Hook 域要件定義書 (UI 高级设置 → Hooks 标签页) |
| 版本 | v0.5.2 |
| 作成日 | 2026-09-24 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 対象範囲 | STAR Rust 核心 Hook Engine + 高级设置 Hooks 标签页 + Project/Worktree Hook policy + Run/Worktree/BI 集成 |
| 対象バージョン | Star Rust Agent/Worktree Runtime；Mavis/Python 仅限显式兼容 adapter，不具备核心 guard authority |
| 適用プラットフォーム | Windows (PowerShell) + POSIX (bash) 双平台 |
| 関連 commit | (root 统一 commit 时填, per 守门 #1 v15) |
| 上位文書 | `AGENTS.md` §4 守门硬约束 (守门 #1+#5+#6+#9+#10+#13+#14 v3+#14 v4) |
| 平行 SRS | `SRS-MULTICA-SKILL-001.md` v0.1 (skills 域, 同走"高级设置"导航 Skills 标签页) |
| 関連文書 | `docs/automation-design.md` v0.1 + `scripts/automation/console_server.py` v0.1 + `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 (PreToolUse guard 是 hook 体系下 1 个具体 guard, 本 SRS 是 hook 上位抽象) |
| 機能数 | 8 機能 (FR-1 ~ FR-8), 業務要件 5 (BR-1 ~ BR-5), 非機能要件 6 類 (NFR-P/A/S/M/T/O), 受理条件 8 (AC-1 ~ AC-8) |
| データモデル | 3 表 W/T/M 横展 (Hook / Hook Run / Hook Session State, 100% 覆盖 per 守门 #13) |
| UI 容器 | "高级设置" 导航 (per ULYS-235 拍板) → Skills 标签页 + Hooks 标签页 (per ADR-0026 §1.3 5 类扩展点) |

### 0.2 修订履历

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-24 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 機能 (FR-1~FR-8) + 5 業務要件 (BR-1~BR-5) + 6 非機能要件 (NFR-P/A/S/M/T/O) + 8 验收条件 (AC-1~AC-8), 3 表 W/T/M 横展 (Hook W/M + Hook Run T + Session M, 100% 覆盖 per 守门 #13), 守门 8/8 通过, 8 已知缺口 (含 1 P0 阻塞 hook 自身越权, 跟 SRS-PRE-TOOL-USE-GUARD-001 §8 #1 一致) | ULYS-235 (2026-09-24 20:xx JST) "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计" |
| **v0.2** | 2026-09-24 15:01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | MCP 升格标签页: §1.4 排除範囲 + §FR-7.1 tabs 列表 (`Skills`, `Hooks`, `MCP`, 预留 `Commands`, `Agents`), MCP 标签页走独立 SRS-MULTICA-MCP-001 (v0.1 stub, 列出已注册 servers + 启停 toggle + transport 类型 stdio/sse/http); commands / agents 仍"预留"占位 | 2026-09-24 15:01 JST Ulysses 评论 "MCP也应该是一个标签页" |
| **v0.3** | 2026-09-24 22:04 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | Plugins 升格标签页: §1.4 排除範囲 + §FR-7.1 tabs 列表 (`Skills`, `Hooks`, `MCP`, `Plugins`, 预留 `Commands`, `Agents`), Plugins 标签页走独立 SRS-MULTICA-PLUGIN-001 (v0.1 stub, 列出已安装 plugin 包 `~/.multica/plugins/` + 内置 builtin + 启停 toggle + 版本显示); commands / agents 仍"预留"占位 | 2026-09-24 22:04 JST Ulysses 评论 "还有plugins也应该是一个标签页" |
| **v0.4** | 2026-09-24 22:13 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 自审饱和: §0.1 版本号 v0.3 (头部 banner 已标 v0.3) + §1.1 后续 BD/DD 引用 v0.3 (头部 banner 已同步) + §1.4 排除範囲 v0.1→v0.3 (新增 MCP 升格 + Plugins 升格行, 旧 4 类→3 类枚举补全) + §10 修订履歴 v0.2/v0.3/v0.4 三行同步追加; 修正 4 处 cross-reference staleness | 2026-09-24 22:13 JST Ulysses 评论 "自审, 各级文档都要做到位" |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 **STAR 平台 "高级设置 → Hooks" 域** 的要件定義書. Hooks 体系是 ADR-0026 v0.2 §1.3 列出的 5 类扩展点之一 (commands / agents / skills / **hooks** / MCP), 跟 skills 平行, 共享同一导航 "高级设置" 下的不同标签页. Hooks 域具体涵盖:

- 14 类 Hook 事件 (per Claude Code PreToolUse/PostToolUse/UserPromptSubmit/SessionStart/SessionEnd 等 + 扩展)
- 4 状态 (registered / enabled / disabled / archived) + 30 天归档策略
- Hook 注册中心 (单一来源, JSON Schema + 热更新)
- Hook 执行引擎 (Rust builtin typed evaluator；关键 phase 执行 allow/deny/require_human/defer；post-commit advisory 走有界隔离队列)
- 全操作审计 log (Transaction append-only per 守门 #13)
- UI "高级设置 → Hooks" 标签页 (CRUD + enable/disable + 触发日志查看)
- 跟 skills 域并行 (相同导航, 不同标签页, 独立 registry, 共享 session state)
- 跟 PreToolUse guard 联动 (PreToolUse guard 是 hooks 体系下 1 个具体 builtin guard hook)

本 SRS 与总要件 `docs/requirements.md` v5.21 §50.8D 同步；基本设计见 BD v0.5.6、详细设计见 DD v0.5.12。若 v0.1-v0.4 的 Python runner、用户 handler、任意 transform 或 fail-open 文字与本版冲突，以 v0.5 Rust-native、typed-rule、critical-hook fail-closed 安全边界为准；高级设置导航承接 ULYS-235 的既有决定，不另造 Worktree 级入口。

**派生来源**: ULYS-235 (2026-09-24) "hook需求" + ADR-0026 v0.2 §1.3 "5 类扩展点: commands / agents / skills / hooks / MCP" + Claude Code `plugins/hookify` + 9/10 PreToolUse guard 实测 (`SRS-PRE-TOOL-USE-GUARD-001.md` v0.1).

### 1.2 背景 (用户痛点)

**课题 1: 扩展点零散**

当前 STAR / Mavis 体系下的扩展机制按"事件触发"维度分散在 4-5 个不同脚本:

- `scripts/automation/dispatcher.py` v0.1 (子代理 dispatch 前置)
- `scripts/automation/console_server.py` v0.1 line 80-92 (PreToolUse hook)
- `scripts/automation/console_server.py` v0.1 (subagent RPC)
- `scripts/automation/guardian/pre_tool_use_guard.py` (PreToolUse 具体 guard, 15 条规则)

每加 1 个新事件点 = 改 1 个脚本, 无统一 registry / 无统一审计 / 无统一热更新, 扩展面零散.

**课题 2: skills / hooks 概念混淆**

per ADR-0026 v0.2 §1.3 5 类扩展点, skills 是"可复用 procedure (mark down 描述)" (per SRS-MULTICA-SKILL-001 §1.2), hooks 是"事件触发回调 (per-event fan-out)". 当前 2 者无统一容器, 用户找不到入口.

**课题 3: 高级设置导航缺失**

ULYS-235 拍板: 把 skills + hooks + 未来 commands / agents 扩展点统一收口到 1 个 "高级设置" 导航, 不同能力走不同标签页. 当前 0 这个导航.

**课题 4: 不可逆操作缺二次确认的统一机制**

当前 PreToolUse guard 已经覆盖 15 条危险模式 (per SRS-PRE-TOOL-USE-GUARD-001 §FR-3.2), 但其他事件点 (e.g. PostToolUse 后置审计 / SessionEnd 退出确认) 没有统一拦截. Hooks 体系可以收敛所有事件点.

**课题 5: UI 可发现性差**

当前 hooks / skills / guards 全在 `scripts/automation/` 跟 JSON 配置文件, 0 UI 入口, 终端用户看不到. ULYS-235 要求"高级设置 → Hooks" 标签页, 至少要 CRUD + enable/disable + 触发日志查看.

### 1.3 包含範囲 (In-Scope, **8 機能**)

| 機能 ID | 名称 | 项数 | 优先级 | 概要 |
|---|---|---|---|---|
| **FR-1** | Hook 生命周期 | 4 项 | P0 | 4 状态 (registered / enabled / disabled / archived) + 30 天归档 |
| **FR-2** | Hook 事件模型 | 4 项 | P0 | Rust schema 定义的关键 phase + allow/deny/require_human/defer 四种 decision；旧 14 个 Python event name 仅作兼容输入 |
| **FR-3** | Hook 注册中心 | 3 项 | P0 | JSON Schema + 热更新 + builtin hook 默认装载 |
| **FR-4** | Hook 执行引擎 | 3 项 | P0 | 核心同步策略确定性/有界执行与 fail-closed；非关键 post-commit event 有界重试 |
| **FR-5** | Hook 审计 log | 2 项 | P0 | 全操作留痕, Transaction append-only per 守门 #13 |
| **FR-6** | Hook UI 标签页 | 3 项 | P0 | Advanced Settings → Hooks typed Project/Worktree policy Builder、Draft/publish/rollback 与策略 Audit；执行事件/BI 在 Phase 9D 接入 |
| **FR-7** | 跟 Skills/MCP/Plugins 导航协调 | 2 项 | P1 | Advanced Settings 父入口 + 并列 tabs；各域数据独立、仅共享当前授权 UI session |
| **FR-8** | 测试 / 报告 | 1 项 | P1 | 单元测试 + 集成测试 + 报告 (跟现有 PHASE-*-IMPL-REPORT 一致) |
| **合計** | | **22 项** | | |

### 1.4 排除範囲 (Out-of-Scope)

- **网络层拦截** (egress filtering) → 后续 v2.x 拍摄, 不在本 v0.1 范围
- **Hook marketplace / cross-organization 共享** → 一人公司不需要, 跟 skills 域同 disclaimer
- **Hook AI 行为审计** (LLM 决策过程录屏) → 后续 v2.x 拍摄
- **Hook 加密 / 凭据管理** (mavis 内置 vault) → 跨项目需求, 不在本专题
- **5 类扩展点的其他 3 类** (commands / agents) → 仍按"预留"占位, 后续按需扩展; **MCP (per 2026-09-24 15:01 JST, v0.2 升格) + Plugins (per 2026-09-24 22:04 JST "还有plugins也应该是一个标签页", v0.3 升格) 均升格为本 v0.3 同导航下的实装标签页**, 不再属于本专题排除范围

### 1.5 关联文档

| 文档 | 关系 | 关键引用 |
|---|---|---|
| `AGENTS.md` §4 守门硬约束 | 上位 | 守门 #1+#5+#6+#9+#10+#13+#14 v3+#14 v4 |
| `ADR-0026` v0.2 §1.3 | 上位 | 5 类扩展点 (commands / agents / skills / hooks / MCP) |
| `SRS-MULTICA-SKILL-001.md` v0.1 | 平行 | skills 域 SRS (共享"高级设置"导航) |
| `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 | 下位 | PreToolUse guard 是 hook 体系下 1 个具体 guard, 本 SRS 是 hook 上位抽象 |
| `docs/automation-design.md` v0.1 | 平行 | §1.2 [P]/[M]/[S] 判定 + §3.1 dispatcher.py brief 落地 |
| `scripts/automation/console_server.py` v0.1 | 现役 | hook 事件流入口 (line 80-92 范围) |
| `scripts/automation/dispatcher.py` v0.1 | 现役 | 子代理 invoke 前置 (per 守门 #9 v20) |
| `scripts/automation/guardian/pre_tool_use_guard.py` (规划) | 下位实装 | PreToolUse builtin guard hook 的实装位置 |
| Claude Code `plugins/hookify` | 对照基线 | user-defined rule 机制 → 本 v0.1 v0.1 扩展 |
| Claude Code PreToolUse / PostToolUse / UserPromptSubmit / SessionStart / SessionEnd 等 14 事件 | 对照基线 | 14 类事件 schema |

---

### 1.6 渡口新架构补充（v0.5，优先于旧实现草案）

本版 Hook 是 Star Rust 执行核心内的强约束能力。`HookSet / HookRule / evaluator` 具有稳定 schema、version、digest 与 capability scope；核心 builtin rules 不可关闭，Project policy 作为基线、Worktree policy 只能继承或追加限制。Hook 不授予权限、不改写 Task Contract/验收事实，也不替代 ACL、Domain Command 或独立 Validation。

用户配置入口是已拍板的 **高级设置 → Hooks** tab，与 Skills/MCP/Plugins 共享同一高级设置导航；Hook 不是 Worktree Group App，也不新增 Worktree 树层级。UI 必须提供无代码的可视化 builder：规则/状态列表、结构化事件与条件、有限动作、优先级/范围、继承与覆盖视图、核心规则不可覆盖说明、冲突提示、版本 diff、dry-run/历史事件模拟、审批发布、rollback 与运行日志。禁止任意 Python/JavaScript/shell/动态库和无界 DSL。高级设置管理定义；Worktree Index 展示当前有效 HookSet/version/健康与阻断摘要；Run detail / Project Quality & Improvement 能筛选并下钻 HookEvent。

安全关键同步 hook 至少覆盖 Run admission、tool call 前后、validation 前后、review/complete，以及 Worktree create/import/archive/restore/owner-transfer/binding-change/cleanup。决策为 `allow / deny / require_human / defer`。引擎、版本、规则或 append-only audit 不可验证/不可用/超时，则关键操作 fail closed；无影响决策的通知类 post-commit hook 可以有界重试。Plugin Hook 只能隔离执行 advisory/post-commit action，永远不能替换或削弱 builtin guard。

Worktree archive、binding removal 与 checkout cleanup 之前重新校验 actor membership/role、lifecycle version、Runtime health、活跃 Run/Agent lease、file claim、PTY/process drain 与新鲜 Git retention-lock observation；unknown/expired/conflict 阻断操作。Hook decision 是 veto/approval gate，实际状态仍由 Worktree domain command 在写事务中再次原子校验。操作完成通过 Audit/Outbox 刷新 Worktree Index。

每次 Hook evaluation 作为 RunEvent/Transaction audit 投影保存 hook_set/rule/evaluator version+digest、phase、decision/reason class、duration、timeout/fail-closed/override、Project/Worktree/Run/Task/actor/correlation scope；不保存 Secret、原始 prompt、完整 stdout 或隐式推理。BI 按 HookSet/rule version 计算覆盖率、阻断、人审、超时/失败、override 和处置耗时，并按 Project/Worktree/任务类型/复杂度与 Run 的验证/接受/返工/冲突关联；coverage 不足显示 unknown。Hook policy 改进必须经独立评审和固定 Benchmark，不得自行降低规则或评分基准。

| Requirement | Added acceptance |
|---|---|
| HOOK-001/002 | Rust builtin fail-closed policy + visual no-code builder, inherited/versioned HookSet; fail unavailable and no arbitrary code |

**v0.5 优先级说明**：本节定义当前 normative behavior。后续 §3-§9 保留 v0.1-v0.4 的 FR ID 以维持 traceability；凡描述 Python handler/JSON registry、任意 fan-out/transform、用户可关闭 builtin 或 critical fail-open 的段落均为历史实现草案，不适用于 Star 产品 runtime。旧 BLOCK→deny、ASK→require_human 只可用于兼容映射；WARN/PASS 不能授权放行，transform/未知 event 必须拒绝执行。
| HOOK-003/004 | Run/Tool/Validation/Review and Worktree lifecycle gate; cleanup checks ACL/lease/claims/drain/fresh lock before Domain Command CAS |
| HOOK-005/006 | Append-only scoped HookEvent linked to Run/Worktree/Audit and BI with coverage and quality/operations drilldown |
| HOOK-007 | Plugin hook isolated/advisory only; cannot replace or weaken core policy |

## §2 用語定義 / 略語 (Glossary)

| 用語 | 定義 | 出典 |
|---|---|---|
| **Mavis** | 本机 root session agent (Mavis As a Jarvis), 运行在 MiniMax Code | per agent-context block |
| **Hook** | 事件触发回调, 在 STAR 平台某个事件点 (e.g. PreToolUse) 触发的可注册回调 | ADR-0026 §1.3 |
| **Hook event** | 14 类事件之一: PreToolUse / PostToolUse / UserPromptSubmit / SessionStart / SessionEnd / SubagentDispatch / SubagentReturn / ToolError / FileWatch / CronTick / RuntimeScan / WorkspaceSwitch / NetworkEgress / CustomEvent | 本 SRS 自定义 |
| **Hook decision** | `allow / deny / require_human / defer`；decision 不可改写命令、授权或任务验收事实 | v0.5 Rust-native contract |
| **Builtin hook** | STAR 平台内置的 hook (e.g. PreToolUse guard, SessionStart cleanup), 不可禁用 | 本 SRS 自定义 |
| **User-defined hook** | 用户/项目自定义的 hook, 可启用/禁用 | 本 SRS 自定义 |
| **高级设置** | STAR UI 顶层导航, 容纳 skills / hooks 等扩展点, 不同能力走不同标签页 | ULYS-235 拍板 |
| **Hooks 标签页** | `/settings/advanced/hooks`；高级设置下的 typed policy Builder、发布/回滚与策略 Audit 视图 | ULYS-235 + FR-6 |
| **Skills 标签页** | "高级设置" 下 skills 域的 UI 标签页 (per SRS-MULTICA-SKILL-001 v0.1) | 平行 SRS |
| **Hook policy store** | Rust-owned Project/Worktree versioned HookPolicySet；Python JSON 仅兼容迁移源，不是生产 SoR | v0.5 Rust-native contract |
| **Hook run** | 单次 hook 执行记录, 写审计 log | 本 SRS 自定义 |
| **Fan-out** | 1 个事件触发 N 个 hook (per-event 多 hook 列表), 串行或并行执行 | 本 SRS 自定义 |
| **deny** | 安全关键 rule 拒绝本次操作；不改变其它 scope 的授权事实 | v0.5 Rust-native decision |
| **require_human** | 暂停本次操作并等待具备当前权限的人类决策；撤权/超时不得默认为通过 | v0.5 Rust-native decision |
| **legacy BLOCK/ASK/WARN/PASS** | 仅作为旧 Python event/result 的可映射输入；映射到 Rust typed decision 后仍由核心 evaluator 决策，不能直接放行 | v0.5 兼容说明 |
| **legacy transform** | 不支持执行，不得改写 executable/argv/Task Contract/acceptance 或权限 scope | v0.5 明确拒绝 |
| **critical fail-closed** | policy/evaluator/audit unverifiable、unavailable 或超时 → 阻断关键操作；只对无决策影响的 after-commit notification 允许有界重试 | v0.5 安全要求 |
| **W/T/M** | Work / Transaction / Master 三类横展 (per 守门 #13) | STAR 守门 #13 |
| **IPA SEC** | Information-technology Promotion Agency, Software Engineering Center | 日本独立行政法人 |
| **要件定義書** | Software Requirements Specification (SRS) | IPA SEC テンプレート |
| **基本設計書** | Basic Design Document (BDD) | IPA SEC テンプレート |
| **詳細設計書** | Detailed Design Document (DDD) | IPA SEC テンプレート |

---

## §3 業務要件 (BR, Business Requirements)

### BR-1 统一事件扩展机制

用户能在 **高级设置 → Hooks** 用无代码 typed rule Builder 管理规则与版本；Rust engine 在受支持的 Run/tool/validation/review/Worktree lifecycle phase 计算有限 decision 并留存 scope/version provenance。核心 rule 不可关闭，未知 Python event、任意 handler 和不支持 action 必须拒绝或保留为待映射 draft。既有 PreToolUse guard 迁移为 builtin HookRule；Hook 不能成为 Worktree tree app。

### BR-2 凭据外泄防护 (跟 BR-2 PreToolUse guard 一致)

凭据 (env var / GitHub PAT / SSH 私钥 / `.env` 文件) 任何形式的打印 / 复制 / 上传均触发 Rust builtin deny；decision 由 core engine 作出，不委托用户代码。**跟守门 #5 派生**: 守门 #5 是 review guard，本 SRS 的 Hook 是 execution guard。

### BR-3 不可逆操作二次确认 (跟 BR-3 PreToolUse guard 一致)

不可逆操作 (`rm -rf` 命中 `/` `~` `*`、`mkfs`、`sudo`、force push) 由 builtin policy deny 或 require_human；人类决定必须在完成二次授权后经 Domain Command 执行，超时/身份失效不得转为 allow。

### BR-4 高级设置导航统一容器 (per ULYS-235 拍板)

"高级设置" 顶层导航下, skills / hooks 共享同一容器, 不同能力走不同标签页 (e.g. "Skills" 标签页 + "Hooks" 标签页), 独立 registry, 共享 session state. **预留扩展位**: 后续 commands / agents 等扩展点按需加新标签页, 走同一导航 (MCP 在 v0.2 升格 + plugins 在 v0.3 升格为实装标签页, per 2026-09-24 15:01 + 22:04 JST 用户两次拍板).

### BR-5 规则可观测 (跟 BR-4 PreToolUse guard 一致)

所有 Hook decision、timeout、failure、override 和 post-commit advisory outcome 均作为 scope/version/correlation 完整的 append-only event/audit；敏感正文不得进入日志。可视化日志查看仍在 **高级设置 → Hooks** tab，Run detail/BI 只提供授权筛选和回链。

---

## §4 機能要件 (FR, Functional Requirements)

### FR-1 Hook 生命周期 (4 项, P0)

**FR-1.1** Hook 4 状态

- 状态机: `registered → enabled → disabled → archived`
- `registered`: hook 已写入 registry, 但未启用 (新创建默认状态)
- `enabled`: hook 启用, 事件触发时 fan-out 执行
- `disabled`: hook 禁用, 事件触发时跳过, 但 30 天内可复活
- `archived`: 30 天 disabled 后自动归档, registry 保留 entry 但 `archived=true`, UI 默认隐藏

**FR-1.2** Hook 创建

- 触发: UI "高级设置 → Hooks → 新建规则"；本表不提供 Python/CLI handler 注册入口
- 必填字段: `rule_id / event_phase / typed_conditions / decision / scope / version`
- 可选字段: `priority / timeout_budget / human_review / description`
- 落盘: Star-owned HookPolicySet/HookRule versioned store；发布与审计经 Rust service，不以本地 JSON 为生产 SoR

**FR-1.3** Hook 启用 / 禁用

- 触发: UI "高级设置 → Hooks" 标签页草稿编辑与版本发布
- 行为: 发布创建新的不可变 HookSet version；builtin safety rule 不提供禁用/删除开关；Worktree overlay 只能增加限制
- 审计: 发布/停用/回滚均记录 actor、scope、from/to version、reason 与 correlation ID

**FR-1.4** Hook 归档

- 触发: 策略负责人在高级设置中归档不再生效的用户规则版本
- 行为: 新有效版本不再引用该用户规则；历史版本和 Audit 保留可查，不物理删除；builtin baseline 不可归档
- 回滚: 仅授权角色可恢复先前策略版本；恢复时仍需检查当前 ACL、schema 和 Worktree inheritance constraints

### FR-2 Hook 事件模型 (4 项, P0)

**FR-2.1** 既有来源事件词汇 (兼容输入，不等同于 Rust 可执行 hook point)

Rust 核心可执行 phase 以 §1.6 明列的 Run/tool/validation/review/Worktree lifecycle 为准。下表 14 个名称保留作已存在 Python tooling 或外部 CLI event 的映射来源；`CustomEvent`、未映射名称和用户注册的回调不能直接进入决策路径。

| Event ID | 名称 | 触发时机 | 典型用法 |
|---|---|---|---|
| EVT-001 | PreToolUse | LLM 决定调 tool, tool 实际执行前 | 安全拦截 (PreToolUse guard) |
| EVT-002 | PostToolUse | tool 执行完成后 | 后置审计 / 日志记录 |
| EVT-003 | UserPromptSubmit | 用户输入 prompt 后, LLM 推理前 | prompt 增强 / 凭据脱敏 |
| EVT-004 | SessionStart | session 启动时 | 环境检查 / cleanup |
| EVT-005 | SessionEnd | session 退出时 | 资源清理 / 总结 |
| EVT-006 | SubagentDispatch | 子代理 invoke 前 | dispatch 前置安全检查 |
| EVT-007 | SubagentReturn | 子代理完成后, 结果返回前 | 结果审计 / 凭据脱敏 |
| EVT-008 | ToolError | tool 调用异常时 | 错误日志 / 自动 fallback |
| EVT-009 | FileWatch | 文件 mtime 变化 (per `watchdog`) | 配置热更新 / 自定义触发 |
| EVT-010 | CronTick | 定时任务 tick | 周期任务触发 |
| EVT-011 | RuntimeScan | 本机 agent CLI 扫描完成 | 通知 UI 更新 runtime 状态 |
| EVT-012 | WorkspaceSwitch | workspace 切换时 | registry 切换 / 状态清理 |
| EVT-013 | NetworkEgress | 网络出站调用前 | egress filtering (v0.1 stub, v2.x 实装) |
| EVT-014 | CustomEvent | 用户自定义事件 | 自定义触发 |

**FR-2.2** Rust typed decision (取代旧 pre/post/block/transform action type)

- `allow`: 本 rule 没有否决；所有 ACL、Domain Command 和其它 mandatory rules 仍需通过
- `deny`: 阻止本次操作
- `require_human`: 等待当前有权角色显式决策；超时、撤权或审计失败不得默认为 allow
- `defer`: 仅用于受界限的非破坏性异步准备；不能延后后绕过同步 Worktree safety gate
- 旧 `transform` 一律不执行；pre/post 只是 evaluation phase，不是用户可编写 handler 的 action type

**FR-2.3** 有界确定性 policy evaluation

- 由 Rust evaluator 按固定 baseline → Project → Worktree restrictive overlay 顺序合并 typed rules
- 相同优先级按稳定 rule ID 排序；关键决策串行、确定性执行，不允许任意用户回调并行 fan-out
- 任一 mandatory rule deny / evaluator error / deadline 超限 / 审计不可写，立即 fail-closed
- 非关键 after-commit event 可进入有界、可重放队列，不回滚已提交业务事实

**FR-2.4** 事件 schema

- 格式: JSON, 符合本 SRS §4.3.1 Event schema
- 字段: `event_id / event_type / timestamp / session_id / payload`
- `payload` 字段 per event_type 不同 (e.g. PreToolUse payload 含 `tool_name / tool_args`)

### FR-3 Hook 注册中心 (3 项, P0)

**FR-3.1** Versioned typed HookPolicySet + 单一来源

- 存储: Star-owned HookPolicySet/HookRule typed model 与受控版本化 store；本地 Python JSON 不作生产 SoR
- 每次 publish 先验证 schema、scope、capability、继承约束、冲突、mandatory baseline 和审批，再原子切换当前 version
- 加载/校验失败: 关键 operation fail-closed，并记录不含敏感正文的状态事件

**FR-3.2** Policy version rollout

- Draft 编辑不影响已发布 HookSet；发布形成不可变新版本和 digest
- Project→Worktree effective policy 更新须明确 preview、审批结果和生效范围
- evaluator 缺少新版本/版本不兼容时阻断关键操作并提供恢复/rollback 路径

**FR-3.3** Builtin guard 保留

- Rust builtin safety rules 总是装载，用户和 Plugin 均不能禁用、删除、覆盖或放宽
- 已有 PreToolUse guard 逻辑迁移为 builtin typed rule；Python 代码可供行为对照，但不能作为执行核心

### FR-4 Hook 执行引擎 (3 项, P0)

**FR-4.1** Rust typed evaluator interface

- Evaluator 输入为 immutable `HookEvent + HookPolicySnapshot + authorized HookContext`，输出 `HookDecision + sanitized reason class + evaluator provenance`
- 仅支持 `allow / deny / require_human / defer`；无 Python/JavaScript/shell callback、动态库、网络访问或命令参数 transform
- HookContext 只提供当前 scope 与已授予 capability 的最小引用；不得从 hook 自行获取新权限

**FR-4.2** Bounded deterministic evaluation

- 基线、Project rule 与 Worktree restrictive overlay 按固定顺序合并；同层按 priority、rule ID 稳定排序
- Critical decision 串行确定性执行，并限制 CPU/memory/instruction/event payload 与 wall-clock deadline
- 规则冲突以 deny/require_human precedence 和显式冲突诊断处理；policy version mismatch 阻断关键操作

**FR-4.3** Admission 与并发

- 同步安全 gate 不得排入无界队列；队列满时返回明确 busy/defer 状态且不能先执行被保护动作
- after-commit advisory 才可使用有界、可取消、可重放的 worker pool；子任务继承 Project/Worktree/Run 资源额度

**FR-4.4** No transform of authority-bearing data

- Hook 不能改写 executable/argv/cwd/Task Contract/acceptance/capability scope；数据规范化必须由其所属 Domain Command 执行
- dry-run/impact simulation 只读已授权脱敏 event snapshot，不写业务事实，不触发实际副作用

**FR-4.5** Failure handling

- policy/evaluator/schema/scope/audit error 或关键 phase 超时 → fail-closed，追加 failure classification；敏感异常正文不进入日志
- `require_human` timeout、撤权、身份切换或审批版本过期 → stop；不得默认 allow
- 非关键 after-commit notification 可 bounded retry；失败与重试计数可观测且不能回写原 Run/Worktree 事实

### FR-5 Hook 审计 log (2 项, P0)

**FR-5.1** Scoped immutable Hook event

- 每次 decision/timeout/failure/override/post-commit result 写入 append-only RunEvent/Audit projection
- 字段至少包括 tenant/project/worktree/task/run/actor/correlation、HookSet/rule/evaluator version+digest、phase、decision/reason class、duration、timeout/fail-closed/override
- event 不存 Secret、原始 prompt、完整 stdout/stack trace、命令参数正文或思维过程；保留期遵从所属 Transaction/Audit policy，不使用本地 JSONL 作为生产 SoR

**FR-5.2** Transaction append-only

- 不允许物理删除、原位改写或 truncate；policy update 追加新版本，audit/event 只追加
- BI 从版本化事件重算 coverage/deny/human/timeout/override 与 Worktree/validation/review 关联；缺少事件保留 unknown，不补零
- 高速 telemetry 进入有界短 TTL buffer，不把每帧采样复制进 durable Transaction history

### FR-6 Hook UI 标签页 (3 项, P0)

**FR-6.1** Advanced Settings 导航与可视化策略布局

- 路由：`/settings/advanced/hooks`；容器为 Advanced Settings，与 `Skills`、`MCP`、`Plugins` 并列；不得在 Worktree 树中新增 Hook App。
- 三个工作区：左侧 Project/Worktree scope 和限制规则列表；中间以结构化表单编辑决策、优先级、启用状态与 typed conditions；右侧展示策略版本、继承/覆盖、草稿状态与策略变更 Audit。
- 用户规则只能选择 `Deny / RequireHuman / Defer`；条件字段、比较符和值由有限类型目录选择；builtin safety baseline 只读。
- 页面必须说明当前接入的 Hook phase。Phase 9C 仅覆盖 Worktree archive/cleanup 策略；其它 Run/tool/validation/review 触发点由对应阶段接入，不得显示为已支持。

**FR-6.2** Draft / publish / rollback 操作

- Project 与 Worktree policy 分别读取 `GET /api/v1/projects/{project_id}/hook-policy` 和 `GET /api/v1/worktrees/{worktree_id}/hook-policy/effective`。
- 编辑通过对应 scope 的 `PUT .../hook-policy/draft`，提交 expected policy set ID 与 draft version 做 CAS；保存草稿不得修改已发布版本。
- 发布使用 `POST .../hook-policy/publish` 创建不可变策略版本；Project/tenant admin 才能发布。回滚使用 `POST .../hook-policy/rollback` 产生新版本并保留 Audit。
- 不直接写 `registry.json`，不执行 Python/JavaScript/shell handler，不允许 UI 放宽或覆盖 builtin rules。缺少认证、scope、权限或 API Provider 时 fail closed，不展示本地 seed policy。

**FR-6.3** 策略 Audit 与执行事件区分

- Phase 9C 右侧列表来源为 Hook policy API 的 append-only 配置 Audit；不能将配置变更称作 Hook execution log。
- RunEvent/outbox execution events、失败和 coverage 的过滤/下钻属于 Phase 9D；summary v2 在 1–90 天窗口内合并 Hook ledger 与字段完整的 `hook_evaluated` RunEvent projection，按 tenant + event_id 去重，并按 tenant/project/task/run 完整键关联最新 Run 状态。缺字段且无 ledger 镜像的 RunEvent 单独计为 excluded；Run producers 尚未完整接入，覆盖仍为 partial/unknown，不能把状态 join 完成误作全阶段 coverage 或完整 Project BI。完整结果接入后由 Project Quality & Improvement / Run Detail 深链回 Advanced Settings Hooks。
- 读接口不可用时展示明确错误/unknown，不把空列表解释为零次触发，也不回退 Python JSONL 文件。

### FR-7 跟 skills 域协调 (2 项, P1)

**FR-7.1** 同导航不同标签页

- Settings 主侧栏只提供“高级设置”父入口；进入 `/settings/advanced` 后，在页面内容区显示 `Skills`、`Hooks`、`MCP`、`Plugins` 四个并列的局部选项卡导航条；默认进入 `/settings/advanced/hooks`。该标签条沿用 ULYS-235，不能变成 Worktree 主导航节点或独立侧栏入口。
- Hooks 不成为独立主导航项，也不插入 Project→Worktree→Group Apps 树；Worktree Index 只显示当前 effective policy/version/health 与阻断摘要，并深链到同一个 Hooks tab。
- 各 tab 分别遵循其领域 SRS 与授权 API：Skills registry、Hook policy、MCP connection 和 Plugin manifest/grant 不共享数据所有权，也不互相授予权限。
- 只有非敏感的显示状态（当前 tab、筛选器和 Project/Worktree scope）可在高级设置页间保留；认证身份、授权事实、策略快照与凭据必须由当前 session/server 重新解析，不可跨 session 复制。
- Commands / Agents 如后续成为 tab，必须沿用相同父导航和独立领域权限模型；当前不代表这些 tab 已实现。

**FR-7.2** 独立领域数据 + 有界 UI session state

- Skill registry 按 `SRS-MULTICA-SKILL-001` 管理；Hook policy 使用 Rust-owned Project/Worktree versioned store (Master/SCD2) 与 TTL Draft；MCP/Plugin 分别使用其专属连接和 capability/grant store。
- `registry.json`、Python handler 与旧 JSONL 只可进入显式只读迁移/兼容流程，不能提供在线 Hook 决策或覆盖 Rust builtin baseline。
- 当前 Advanced Settings 页面若没有 host auth session/provider，必须不显示 seed policy 且禁用所有 read/write；接入 Provider 后也要对 principal/session generation 变化清除旧 API 投影。
- tab、筛选器与选中的 Project/Worktree 只是 UI projection；每个 API request 都由服务端重新校验当前 tenant/Project membership、role、scope 与 policy revision。

### FR-8 测试 / 报告 (1 项, P1)

**FR-8.1** 单元测试 + 集成测试 + 报告

- Rust unit tests 覆盖 typed policy bounds/digest/scope、builtin fail-closed evaluator 与 restrictive rule 校验；REST tests 覆盖 Project/Worktree authorization、CAS/TTL、publish/rollback、Audit/RLS 边界。
- UI tests 验证高级设置父导航与并列 tabs、无 auth Provider 时 fail closed、policy API 路由契约；后续 browser E2E 需在实际 session Provider/目标 DB 下覆盖 read/draft/publish/rollback、继承与错误恢复。
- 验证报告分开记录 compile/typecheck、unit、API/RLS integration、browser/E2E、production runtime 和性能结果；旧 Python handler/fan-out/transform/fail-open 测试仅作为迁移参考，不作为产品验收。

---

## §5 非機能要件 (NFR, Non-Functional Requirements)

### NFR-P 性能 (Performance)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-P-1** | 单 hook 延迟 p50 | 单元测试 benchmark | < 5ms (跟 PreToolUse guard 一致) |
| **NFR-P-2** | 单 hook 延迟 p99 | 单元测试 benchmark | < 10ms |
| **NFR-P-3** | 14 事件 fan-out 总延迟 p99 | 单元测试 benchmark | < 50ms (5 hook 平均) |
| **NFR-P-4** | audit log 写延迟 | benchmark | < 1ms (异步 fsync) |
| **NFR-P-5** | registry 热更新延迟 | 实测 | < 100ms |
| **NFR-P-6** | 不影响 mavis runtime 主流程 | 对比 baseline | 0% 主流程额外延迟 |

### NFR-A 可用性 (Availability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-A-1** | registry 加载失败 → fail-open | 单元测试 | 100% 放行 |
| **NFR-A-2** | audit log 写失败 → fail-closed | 单元测试 | 100% 阻断 |
| **NFR-A-3** | hook handler 抛异常 → 视为 WARN | 单元测试 | 100% 继续 fan-out |
| **NFR-A-4** | registry 热更新不中断 hook 调用 | 实测 | 0 中断 |
| **NFR-A-5** | mavis runtime 启动时间不显著增加 | 实测 | < 100ms (14 事件装载 + builtin hook 装载) |

### NFR-S 安全性 (Security)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-S-1** | 凭据 0 外泄 | 单元测试 + 渗透测试 | 0 命中 |
| **NFR-S-2** | audit log 不可物理删除 | OS 权限验证 | 0 修改 |
| **NFR-S-3** | registry 文件只读 (mavis 进程外) | OS 权限验证 | 0 篡改 |
| **NFR-S-4** | builtin hook 不可被用户删除 | UI 测试 | 0 删除 |
| **NFR-S-5** | 跟守门 #5 (env 安全) 联动 | 单元测试 | 守门 #5 命中场景 100% 被 hook BLOCK |
| **NFR-S-6** | 跟 SRS-PRE-TOOL-USE-GUARD-001 §NFR-S-4 一致 | 单元测试 | 子代理 dispatch 前置 100% 覆盖 |

### NFR-M 保守性 / 维护性 (Maintainability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-M-1** | registry JSON Schema 文档化 | 文档 | 100% |
| **NFR-M-2** | hook 增删不改 Python 代码 (user-defined) | 实测 | 0 代码改动 |
| **NFR-M-3** | audit log JSON Lines 格式 | 实测 | 100% JSON.parseable |
| **NFR-M-4** | 测试覆盖率 | pytest coverage | ≥ 90% |
| **NFR-M-5** | 跟 skills 域 registry 不冲突 | 实测 | 0 互相覆盖 |

### NFR-T 移植性 (Portability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-T-1** | Windows PowerShell 兼容 | CI 实测 | 100% pass |
| **NFR-T-2** | POSIX bash 兼容 | CI 实测 | 100% pass |
| **NFR-T-3** | path 跨平台 (`\` + `/`) | 单元测试 | 100% 覆盖 |
| **NFR-T-4** | Python 3.10+ | 实测 | 0 兼容问题 |
| **NFR-T-5** | Next.js 14+ 前端 | 实测 | 0 兼容问题 |

### NFR-O 可观测性 (Observability)

| ID | 要求 | 計測方法 | 阈值 |
|---|---|---|---|
| **NFR-O-1** | 命中统计 metric | 实测 | decision / event_type / hook_name / session_id 维度 |
| **NFR-O-2** | 错误 trace | 实测 | 0 静默吞错 |
| **NFR-O-3** | BLOCK 事件触发 root session 通知 | 实测 | < 500ms 推送 |
| **NFR-O-4** | UI "触发日志" 标签可见 | 实测 | 100% 可访问 |

---

## §6 制約条件 / 前提 / 依赖

### 6.1 制約条件

- 必须在 `scripts/automation/hooks/` 目录下落地 (跟现有 guardian / skill 目录一致)
- 必须用 Python 3.10+ (跟 mavis runtime 一致)
- registry 必须 JSON 格式 (跨工具可读, 跟 skill registry 一致)
- audit log 必须 JSON Lines (append-only 友好)
- UI 必须 Next.js 14+ (跟现有 frontend 一致)
- 不引入新的外部依赖 (用 stdlib `re` / `json` / `pathlib` / `logging` + `jsonschema` + `watchdog`, 跟 SRS-PRE-TOOL-USE-GUARD-001 §6.1 一致)
- 跟守门 #1+#5+#6+#9+#10+#13+#14 v3+#14 v4 8 项必过

### 6.2 前提

- 守门 #5 (env 安全) 已确立, 本 SRS 是 execution guard 升级
- 守门 #9 (子代理 RPC 不可靠) 已确立, 本 SRS 在 SubagentDispatch / SubagentReturn 事件点补强
- 守门 #13 (W/T/M 横展 100% 覆盖) 已确立, 本 SRS 3 表 100% 覆盖
- `console_server.py` v0.1 + `dispatcher.py` v0.1 现役, hook 事件流入口存在
- SRS-PRE-TOOL-USE-GUARD-001 v0.1 落档, PreToolUse guard 是 hooks 体系下 1 个具体 builtin guard

### 6.3 依赖

- Rust evaluator：`crates/domain-hook` 的 typed policy verifier/evaluator；critical decision 必须在 Rust builtin baseline 下 fail closed。
- Policy API：`crates/star-api-rest/src/group_api/hook_policies.rs` 与 Worktree lifecycle gate；依赖当前 actor/tenant/Project scope，目标 DB/RLS/grants 部署另列生产启用门。
- Policy storage migration：`db/migrations/2026-09-30-multica-hook-policy.sql`；源码与隔离环境验证不代表目标数据库已部署。
- Advanced Settings UI：`frontend/src/app/(app)/settings/advanced/layout.tsx`、`[tab]/page.tsx`、`[tab]/HookPolicyPage.tsx`；调用 `frontend/src/lib/group/worktreeGroupApi.ts` 的 scope-aware API adapter。
- 宿主前置条件：应用根 `Providers` 注入认证 `WorktreeGroupApiProvider`，提供 token 与非敏感 session generation；当前未接入时页面必须 fail closed。
- 历史兼容参考：`scripts/automation/console_server.py`、`dispatcher.py`、`guardian/pre_tool_use_guard.py` 与旧 JSON/JSONL；不得作为产品 policy store、决策引擎或 authoritative execution audit。
- SRS-MULTICA-SKILL-001 v0.1 为并列 Skills tab 的领域要求；MCP/Plugins 继续由各自领域 API 与 capability grant 约束。

---

## §7 验收条件 (AC, Acceptance Criteria)

| AC | 关联 FR / NFR | 验收方法 | 阈值 |
|---|---|---|---|
| **AC-1** | FR-2.1 + FR-4.1 | Rust unit tests 证明 builtin 在 scope/authorization/runtime/retention lock 事实缺失、冲突或过期时 fail closed | 所有 mandatory negative cases 均阻断 |
| **AC-2** | FR-2.2 + FR-4.2 | typed policy decoder 验证 schema、大小/数量边界、scope/version、restrictive-only action 与 canonical digest | 有效 fixture 接受；每类无效/越界 fixture 拒绝 |
| **AC-3** | FR-1.1 + FR-1.4 | Project baseline 与 Worktree overlay inheritance/rebase 测试证明 overlay 只能追加限制且 revision 不可变 | 所有跨 scope/放宽规则用例拒绝 |
| **AC-4** | FR-3.2 + FR-7.1 | Navigation contract test 验证 Settings 侧栏为“高级设置”父入口，Hooks 与 Skills/MCP/Plugins 并列且路径固定 | 四个 tab route 100% 匹配；Hooks 不出现为 Worktree App |
| **AC-5** | FR-4.3 + FR-4.4 | REST/API tests 覆盖权限/RLS、Draft CAS/TTL、publish/rollback Audit、冲突和错误时的 fail-closed 行为 | 全部授权负例拒绝；状态变更有审计事实 |
| **AC-6** | FR-6.1 + FR-6.2 | UI 集成测试验证 `/settings/advanced/hooks`、scope 选择、typed 条件编辑、Draft CAS 与 admin-gated publish/rollback；无认证 Provider 时不得显示 seed policy 或发送写请求 | 100% |
| **AC-7** | FR-6.3 + NFR-O-4 | 9C 验证配置 Audit 与执行日志明确分开；9D 验证有界 summary 窗口/公式/partial coverage 呈现，并验证 RunEvent/outbox 查询、Run outcome 与授权下钻在未接入前不冒充完整 BI | 100% |
| **AC-8** | NFR-S-1 + NFR-S-5 | 渗透测试 (14 事件 × 4 action type × 10 攻击场景) | 0 命中 |

---

## §8 已知缺口 / 风险

| # | 缺口 | 严重度 | 触发条件 | 缓解 / 后续 |
|---|---|---|---|---|
| **#1** | Hook handler 内部越权 (handler 调 subprocess 不受 hook 约束) | **P0 阻塞** | 用户注册恶意 hook handler 调 `subprocess.run(['rm', '-rf', '/'])` | v0.2 拍摄 handler sandbox 隔离 (e.g. bwrap / docker --read-only), 跟 SRS-PRE-TOOL-USE-GUARD-001 §8 #1 一致 |
| **#2** | 编码绕过 (base64 / hex handler 命令) | P1 | handler 调 `echo cm0gLXJmIA== \| base64 -d \| bash` | v0.2 拍摄 decode-then-scan 二级匹配 |
| **#3** | Hook registry 体积膨胀 (用户加 1000+ hook) | P1 | 单 event fan-out > 100 hook, 延迟失控 | v0.2 拍摄 hook priority 智能排序 + 上限配置 |
| **#4** | 并行 hook 资源竞争 (e.g. 两个 hook 同时写同一文件) | P1 | parallel=true 同 priority hook 冲突 | v0.2 拍摄 resource lock 机制 |
| **#5** | audit log 体积 (90 天 × 14 事件 × 高频触发) | P2 | 单 session 1h 可能 10000+ 命中 | v0.2 拍摄 log rotation + 压缩 |
| **#6** | Hook UI 国际化 (i18n) | P2 | 标签页含中英文混合 | v0.2 完善 i18n |
| **#7** | 跨平台 path 字符 (Windows `;` vs POSIX `:`) | P2 | PATH 解析 | v0.2 扩展, 跟 SRS-PRE-TOOL-USE-GUARD-001 §8 #6 一致 |
| **#8** | mavis runtime 升级不兼容 | P2 | 未来 mavis v1.0 API 变化 | 关注 mavis changelog, 同步升级, 跟 SRS-PRE-TOOL-USE-GUARD-001 §8 #8 一致 |

---

## §9 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-24 | 2026-09-24 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-24 | 2026-09-24 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-24 | 2026-09-24 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-24 | 2026-09-24 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-24 | 2026-09-24 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)

---

## §10 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-24 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 機能 (FR-1~FR-8, 22 项) + 5 業務要件 (BR-1~BR-5) + 6 非機能要件 (NFR-P/A/S/M/T/O 30 项) + 8 验收条件 (AC-1~AC-8) + 8 已知缺口 (含 1 P0 阻塞 hook handler 越权), 3 表 W/T/M 横展 (Hook W/M + Hook Run T + Session M, 100% 覆盖 per 守门 #13), 守门 8/8 通过, IPA 10 段结构 (目的 / 範囲 / 用語 / 業務 / 機能 / 非機能 / 制約 / 验收 / 缺口 / 签字 + 修订), 14 类事件 + 4 种 action type + UI "高级设置 → Hooks" 标签页 + 跟 skills 域同导航不同标签页协调 | ULYS-235 (2026-09-24 20:xx JST) "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计" |
| **v0.2** | 2026-09-24 15:01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | MCP 升格标签页: §1.4 范围 + §FR-7.1 tabs 列表更新 (`Skills`, `Hooks`, `MCP`, 预留 `Commands`, `Agents`), MCP 标签页走独立 SRS-MULTICA-MCP-001 (v0.1 stub, 列出已注册 servers + 启停 toggle + transport 类型 stdio/sse/http); commands / agents 仍"预留"占位 | 2026-09-24 15:01 JST Ulysses 评论 "MCP也应该是一个标签页" |
| **v0.3** | 2026-09-24 22:04 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Plugins 升格标签页: §1.4 范围更新 + §FR-7.1 tabs 列表增加 `Plugins` 实装位 + 新增 Plugins 标签页条目 (走 SRS-MULTICA-PLUGIN-001 v0.1 stub, 列出已安装 plugin 包 `~/.multica/plugins/` + 内置 builtin + 启停 toggle + 版本显示) + §BR-4 描述细化; commands / agents 仍"预留"占位 | 2026-09-24 22:04 JST Ulysses 评论 "还有plugins也应该是一个标签页" |
| **v0.4** | 2026-09-24 22:13 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 自审饱和: §0.1 版本号 v0.3 (头部 banner 已标 v0.3) + §1.1 后续 BD/DD 引用 v0.3 (头部 banner 已同步) + §1.4 排除範囲 v0.1→v0.3 (新增 MCP 升格 + Plugins 升格行, 旧 4 类→3 类枚举补全) + §10 修订履歴 v0.2/v0.3/v0.4 三行同步追加; 修正 4 处 cross-reference staleness | 2026-09-24 22:13 JST Ulysses 评论 "自审, 各级文档都要做到位" |
| **v0.5** | 2026-09-30 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 保留 ULYS-235 高级设置同导航不同 tab 决策；增加 Rust builtin typed HookSet、不可关闭 fail-closed 规则、Project/Worktree 继承、视觉无代码 builder、Run/Worktree lifecycle/BI 事件联动；旧 Python handler 降为兼容历史，产品 Hook runtime/UI 仍待实施 | 用户要求 Hook 是原生强约束、与 BI/Worktree 联动、可视配置并指出它属于高级设置 tab |
| **v0.5.1** | 2026-09-30 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将总需求交叉引用更新到 v5.20；该版本的 Hook 规则和 Advanced Settings 标签页要求未变 | Phase 8B Run requirement 增补后，同步当前总需求基线引用 |
| **v0.5.2** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | FR-6 改为 Project/Worktree policy Builder 与 typed Audit API；FR-7 明确“高级设置”父入口和并列 tabs，不复制跨 session 授权事实；FR-8/AC 改为 Rust evaluator、policy API 与当前 UI/navigation 验收；更新当前代码文件和认证 Provider 缺口；区分 Phase 9D 执行 RunEvent/BI | Phase 9C 实装 Advanced Settings 导航与 Hooks 策略编辑页，清除旧 registry.json/handler 需求歧义 |
| **v0.5.3** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Advanced Settings 内部局部标签条与 Settings 主侧栏父入口的层级；规定 Phase 9D `hook_execution_summary_v1` 只覆盖已记录 archive ledger、明确窗口和 partial/unknown 状态，不得当作 Run outcome join 或完整 BI；更新上/下游设计版本 | Phase 9D summary API/UI consumer 接入既有 ULYS-235 Hooks 标签 |
| **v0.5.4** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 summary 升为 v2：合并 Hook ledger 与完整 RunEvent 投影，使用共享 event_id 去重并以 tenant/project/task/run 键关联最新 Run 状态；将无效投影计数显式暴露，仍保持 Run producer 未接入与 partial/unknown coverage 边界 | Phase 9D-4 加入双来源受限 read model 与 Run 状态 join |
