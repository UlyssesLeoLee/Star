# BD-MULTICA-HOOK-001

> **Multica Hook 域基本設計書 v0.5.5** (Hooks 沿用既有 Advanced Settings 局部标签导航；Phase 9D 加入 archive summary API 与页面汇总卡，仍不构成 Run outcome BI；目标 DB、认证 Provider 与完整 BI read model 尚未验收)

> - 状态: 🟡 Draft v0.5.5 (2026-10-01 JST，Phase 9D summary UI consumer code slice)
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 关联 issue: ULYS-235 ("hook需求")
> - 关联 commit: (留空, root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 上位要件: [`docs/requirements/SRS-MULTICA-HOOK-001.md`](../requirements/SRS-MULTICA-HOOK-001.md) v0.5.3 + `docs/requirements.md` v5.20 §50.8D
> - 兼容参考（非产品决策 runtime）: `scripts/automation/console_server.py`、`scripts/automation/dispatcher.py` 与 `scripts/automation/guardian/pre_tool_use_guard.py` 是现有 Python automation/guard 工具；它们不构成 Star Rust Hook Engine，不拥有产品 ACL、Worktree cleanup 或验收 authority。Rust evaluator、policy API 与 archive-confirm gate 已有条件式代码；Phase 9C UI 代码已加入，但宿主 API session/provider 尚未装配，不能读取真实策略或启用写路径。
> - 平行参考: `docs/automation-design.md` v0.1 (Python 化基线) + `SRS-MULTICA-SKILL-001.md` v0.1 (skills 域, 共享"高级设置"导航) + `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 (PreToolUse guard 是 hooks 下 1 个 builtin guard)
> - 守门基线: 守门 #1+#5+#6+#9+#10+#13+#14 v3+#14 v4 8 项必过 (守门 #1 v25 cargo test 不需要跑, 文档工作)
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-30 JST
> - 受众: 詳細設計エンジニア / 実装エンジニア / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
> - 拍板来源: ULYS-235 (2026-09-24 20:xx JST) "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计" (本 BD 落档)

---

## §0 目的 (Purpose)

本文档基于 [`SRS-MULTICA-HOOK-001`](../requirements/SRS-MULTICA-HOOK-001.md) v0.5.3 与 `docs/requirements.md` v5.20 §50.8D，定义 STAR 新架构中的 **高级设置 → Hooks**：Settings 主侧栏只提供“高级设置”父入口；高级设置页面内容区保留 Skills、Hooks、MCP、Plugins 局部标签条。Hooks 不拆成 Worktree 树节点或独立主导航项。HookSet 可视化配置、Rust-native enforcement 与 Worktree/Run/BI 联动沿此边界设计。

- **システムアーキテクチャ** (mavis runtime hook 事件流 + UI 高级设置导航 + 标签页架构)
- **機能分割 / モジュール設計** (6 module: Event Emitter + Hook Registry + Fan-out Scheduler + Hook Runner + Audit Logger + Builtin Hook Loader)
- **データモデル** (3 表 W/T/M 100% 覆盖 per 守门 #13: Hook W/M + Hook Run T + Session M)
- **インターフェース設計** (Hook 事件契约 + mavis runtime 接口 + UI API 接口 + 跟 skills 域共享接口)
- **セキュリティ設計** (凭据零外泄 + audit log 完整性 + registry 文件只读 + builtin hook 不可删除)
- **非機能設計** (性能 < 5ms / fail-open vs fail-closed / 可观测性 / 跟 PreToolUse guard 一致)
- **障害 / 運用設計** (registry 加载失败 / handler 失败 / audit 失败 / 回滚 / log rotation)
- **UI 設計** ("高级设置" 导航 + 标签页架构 + Hooks 标签页布局)
- **用語集** (跟 SRS 共享 + BD 新增 8 条)
- **签字栏 + 修订履歴**

**派生来源**: ULYS-235 (2026-09-24) + ADR-0026 v0.2 §1.3 "5 类扩展点" + SRS-MULTICA-HOOK-001 v0.3 全部 FR-1~FR-8 / BR-1~BR-5 / NFR-P/A/S/M/T/O + Claude Code `plugins/hookify` 14 类事件基线 + `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 (PreToolUse guard 是 hooks 下 1 个 builtin guard).

---

## §0.1 渡口 Rust-native 架构修订（v0.5，覆盖旧执行草案）

v0.1-v0.4 的 6 个 Python module / JSON registry / Python user handler / 本地 JSONL HookRun log 是旧草案形态，不再是安全关键生产架构。保留已有 Mavis/PowerShell/Python hooks 作为显式兼容输入或可观察事件来源；兼容 adapter 不具有策略权威、不能拦截绕过 Rust builtin guard，也不拥有独立 SoR。安全决策与 Worktree lifecycle gate 由 Star Rust Hook Engine 执行，策略以 typed data 存储。现有 PreToolUse guard 作为 builtin HookRule 转入同一 Engine，而不是平行旁路。

```text
Advanced Settings (one shared navigation; Hook is a sibling tab to Skills/MCP/Plugins)
└── Hooks tab: policy list | visual typed rule builder | recent decisions/impact preview
       │ draft -> validate -> dry-run -> approve -> publish HookSet version
       ▼
Rust Hook Policy Service ──> versioned Project HookSet / Worktree restriction overlay
       │                         │
       ├── Rust Hook Engine <── GroupContext + Run/Worktree command facts
       │      ├── builtin guards (mandatory, no plugin replacement)
       │      ├── typed condition evaluator (bounded, deterministic, no network)
       │      └── allow/deny/require_human/defer
       └── append-only RunEvent/Audit + bounded post-commit Outbox
                              │
                              └── Worktree Index / Task Run Detail / Project BI
```

**事实所有权与策略继承**：组织/平台 builtin baseline 不可变更；Project HookSet 版本化；Worktree 可继承指定 Project version 并仅新增更严格规则。每个 Run admission 固定有效 HookSet、RuleSet、evaluator/API version 与 digest；Worktree admin command 记录相同 provenance。History 不随未来版本回写。高级设置负责维护策略，Worktree Index 显示当前 effective version/health/deny summary，Run detail 与 Quality & Improvement 页面提供只读筛选/下钻；不改变 Project → Worktree Index → Worktree 同级 Apps 的产品树。

**可视化配置**：Hooks tab 使用三栏布局：左栏规则/状态/版本列表；中栏结构化事件、scope、条件、优先级、决策与人审要求编辑器；右栏最近触发记录、解释、dry-run 结果和受影响 Worktree/Run 预览。页面提供 Project→Worktree inheritance tree、只读 mandatory baseline、冲突/优先级说明、草稿/已发布 diff、影响模拟、审批发布和 rollback。用户无需写代码。规则仅使用 schema-defined field/operator/value 与有限 action；拒绝 arbitrary source code、shell/script handler、dynamic native library、网络调用和无界 DSL。Web 前端若暂时保留，必须调用相同 Rust policy API/DTO；不能在浏览器执行授权决定。最终桌面设置体验由 Rust-native UI 消费相同 typed projection。

**执行与失败边界**：关键事件前 Hook 在 Domain Command/Tool capability 执行之前同步返回 allow/deny/require_human/defer，不能授予能力、变更 command payload 或将人工审批转换成自动通过。关键 Hook policy/evaluator/version/audit 无法校验、超时或 queue/resource 预算耗尽均 fail closed；builtin baseline 在 Registry 不可用时仍由 Rust core 生效，禁止“加载失败即跳过 guard”。Post-commit observer/BI enrich 可用 bounded Outbox retry，但只影响可重建 projection，不伪造原交易成功/失败。Plugin callbacks 只允许隔离、grant-limited、non-authoritative advisory/post-commit。

**Worktree 与 Agent Runtime 集成**：Hook 处理 Run admission、tool calls、validation/review/completion 和 Worktree create/import/archive/restore/owner-transfer/binding change/cleanup。归档或物理清理前必须在 Hook 与 Worktree Domain Command 双重检查：当前 membership/role、lifecycle version、Runtime health、active Run/Agent lease、file claim、PTY/process drain 和 fresh Git retention-lock observation；unknown/stale/conflict 一律 deny。单个 Git lock、Agent lease 或 file claim 不得替代其它信号。成功写入时 Run/Worktree facts、Audit、Outbox 与 correlation ID 一致，Index 刷新授权 projection。

**BI contract**：一次 Hook evaluation 生成 typed HookEvent，包含 hook_set/rule/evaluator version+digest、phase、decision、reason class、duration、timeout/fail-closed/override、actor/tenant/project/worktree/task/run/correlation IDs。event 进入已有 RunEvent/Audit canonical stream；独立 HookRun 列表如保留，仅是有界分页投影，不复制大 payload。BI 指标包括 Hook invocation coverage、deny/require_human、timeout/error、approved override、阻断到处理时长、Worktree cleanup rejected/stale-signal、RuleSet 发布前后验证/人工接受/返工 cohort 关联。所有指标显示窗口/分母/coverage/version，缺事件为 unknown；proposal 不能自动放宽规则或改评分。

**W/T/M 归属**：HookSet/Rule published version 是 Master/SCD2；草稿/模拟缓存是有 TTL 的 Work；Hook evaluation、审批、发布/撤销、Worktree action outcome 是 append-only Transaction/Audit/RunEvent。Session tab state 只保存视图选择，不复制策略事实。禁止以本地 registry JSON 或独立 rotating log 作为 production source of truth。

## §1 适用范围 (Scope)

### 1.1 包含 (In-Scope, 6 module + 3 表 + 14 事件 + 4 action type + 8 集成点)

| 类别 | 范围 | 数量 |
|---|---|---|
| **module** | Event Emitter / Hook Registry / Fan-out Scheduler / Hook Runner / Audit Logger / Builtin Hook Loader | 6 module |
| **数据表** | `hooks` (W/M) + `hook_runs` (T) + `hook_session_state` (M) | 3 表 W/T/M |
| **事件类型** | PreToolUse / PostToolUse / UserPromptSubmit / SessionStart / SessionEnd / SubagentDispatch / SubagentReturn / ToolError / FileWatch / CronTick / RuntimeScan / WorkspaceSwitch / NetworkEgress / CustomEvent | 14 类 |
| **action type** | pre / post / block / transform | 4 种 |
| **集成点** | console_server.py / dispatcher.py / mavis runtime hook / frontend Next.js / ask_user / file watcher / CI / pytest | 5.1.4 + 8 集成点 |
| **API 端点** | `emit(event)` 内部 API + `register_hook(hook_def)` + `enable_hook(name)` + `disable_hook(name)` + `archive_hook(name)` + `query_runs(filters)` + `reload_registry()` + `get_builtin_hooks()` | 8 端点 |
| **文件** | 6 Python module + 1 registry JSON + 1 audit log + 3 test files + 1 frontend page + 1 frontend component | 12 文件 |
| **UI 标签页** | `/settings/advanced/hooks` + `/settings/advanced/skills` (平行) | 2 标签页 (共享"高级设置"导航) |
| **NFR 阈值** | 性能 / 可用性 / 安全 / 保守性 / 移植性 / 可观测性 6 類 30 项 | 30 项 |

### 1.2 排除 (Out-of-Scope, per SRS §1.4)

- 网络层拦截 (egress filtering) → v2.x, 本 v0.1 仅 NetworkEgress 事件 stub
- Hook marketplace / cross-organization 共享 → 一人公司不需要
- Hook AI 行为审计 (LLM 决策录屏) → v2.x
- Hook 加密 / 凭据管理 (mavis 内置 vault) → 跨项目需求
- **5 类扩展点的其他 2 类** (commands / agents) → 仍按"预留"占位, 后续按需扩展; **MCP (per 2026-09-24 15:01 JST, v0.2 升格) + Plugins (per 2026-09-24 22:04 JST "还有plugins也应该是一个标签页", v0.3 升格) 均升格为本 v0.3 同导航下的实装标签页**, 不再属于本专题排除范围

### 1.3 关联文档

| 文档 | 关系 | 关键引用 |
|---|---|---|
| `SRS-MULTICA-HOOK-001.md` v0.3 | 上位 | FR-1~FR-8 / BR-1~BR-5 / NFR-P/A/S/M/T/O |
| `AGENTS.md` §4 守门硬约束 | 上位 | 守门 #1+#5+#6+#9+#10+#13+#14 v3+#14 v4 |
| `ADR-0026` v0.2 §1.3 | 上位 | 5 类扩展点 (commands / agents / skills / hooks / MCP) |
| `SRS-MULTICA-SKILL-001.md` v0.1 | 平行 | skills 域 SRS (共享"高级设置"导航) |
| `SRS-PRE-TOOL-USE-GUARD-001.md` v0.1 | 下位 | PreToolUse guard 是 hooks 下 1 个 builtin guard |
| `BD-PRE-TOOL-USE-GUARD-001.md` v0.1 | 下位 | PreToolUse guard 基本设计 (5 module + 3 表 + 7 集成点) |
| `docs/automation-design.md` v0.1 | 平行 | §3.1 dispatcher.py brief 落地 + §1.2 [P]/[M]/[S] 判定 |
| `docs/guardian/README.md` | 平行 | 守门 v3x 落档目录 |
| `scripts/automation/console_server.py` v0.1 | 现役 | hook 事件流入口 (line 80-92 范围) |
| `scripts/automation/dispatcher.py` v0.1 | 现役 | invoke() 前置 (per 守门 #9 v20) |
| `docs/reports/PHASE-*-IMPL-REPORT.md` | 平行 | 7 份 PHASE 报告模板 (含 PreToolUse guard) |
| Claude Code `plugins/hookify` | 对照基线 | 14 类事件基线 + user-defined rule 机制 |
| Claude Code PreToolUse / PostToolUse 等 14 事件 | 对照基线 | 14 类事件 schema |
| `docs/frontend-design.md` v0.1 | 平行 | UI 设计基线 (Next.js 14+) |

---

## §2 システムアーキテクチャ (System Architecture)

### 2.1 全体構成 (Overall Architecture)

```
┌──────────────────────────────────────────────────────────────────────┐
│                       Mavis Runtime (Mavis mavis v0.x)                │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │           Frontend (Next.js 14+, /settings/advanced/...)       │  │
│  │  ┌──────────────────┐ ┌──────────────────┐ ┌───────────────┐    │  │
│  │  │ 高级设置 导航     │ │ 高级设置 导航     │ │ 高级设置 导航   │    │  │
│  │  │ └ Skills 标签页   │ │ └ Hooks 标签页    │ │ └ MCP 标签页     │ │ └ Plugins 标签页 │    │  │
│  │  │   (per skill SRS)│ │   (本 BD §3)     │ │ (per MCP SRS    │ │ (per Plugin SRS │    │  │
│  │  │                  │ │                  │ │  v0.1 stub)     │ │  v0.1 stub)     │    │  │
│  │  └──────────────────┘ └──────────────────┘ └───────────────┘ └───────────────┘    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                  ▲                                   │
│                                  │ HTTP API (FastAPI 8080)           │
│                                  ▼                                   │
│  ┌─────────────────────────────────────────────────────────────────┐  │
│  │                   Mavis Backend Hook Layer                      │  │
│  │                                                                  │  │
│  │  ┌─────────────────────────────────────────────────────────┐    │  │
│  │  │             Hook Module (本 BD §3: 6 module)              │    │  │
│  │  │                                                          │    │  │
│  │  │  ┌────────────┐  ┌─────────────┐  ┌──────────────┐     │    │  │
│  │  │  │ Event      │→│ Hook        │→│ Fan-out      │     │    │  │
│  │  │  │ Emitter    │  │ Registry    │  │ Scheduler    │     │    │  │
│  │  │  │ (EM-1)     │  │ (HR-2)      │  │ (FS-3)       │     │    │  │
│  │  │  └────────────┘  └─────────────┘  └──────┬───────┘     │    │  │
│  │  │                                              │             │    │  │
│  │  │                                              ▼             │    │  │
│  │  │  ┌────────────┐  ┌─────────────┐  ┌──────────────┐     │    │  │
│  │  │  │ Audit      │←│ Hook        │←│ Hook         │     │    │  │
│  │  │  │ Logger     │  │ Runner      │  │ Runner       │     │    │  │
│  │  │  │ (AL-5)     │  │ (HR-4)      │  │ (parallel)   │     │    │  │
│  │  │  └────────────┘  └─────────────┘  └──────────────┘     │    │  │
│  │  │       ▲              ▲                                   │    │  │
│  │  │       │              │                                   │    │  │
│  │  │  ┌────────────┐  ┌─────────────┐                        │    │  │
│  │  │  │ Builtin    │  │ session     │                        │    │  │
│  │  │  │ Hook       │  │ state       │                        │    │  │
│  │  │  │ Loader     │  │ (SS-6)      │                        │    │  │
│  │  │  │ (BL-6)     │  └─────────────┘                        │    │  │
│  │  │  └────────────┘                                          │    │  │
│  │  └─────────────────────────────────────────────────────────┘    │  │
│  │                                                                  │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                  ▲                                   │
│                                  │                                   │
│  ┌───────────────────────────────┴───────────────────────────────┐  │
│  │                   Existing Modules (现役)                       │  │
│  │  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────┐ │  │
│  │  │ console_server   │  │ dispatcher       │  │ guardian     │ │  │
│  │  │ (hook 事件流入口) │  │ (SubagentDispatch)│  │ (PreToolUse  │ │  │
│  │  │                  │  │                  │  │  builtin     │ │  │
│  │  │                  │  │                  │  │  guard hook) │ │  │
│  │  └──────────────────┘  └──────────────────┘  └──────────────┘ │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                  ▲                                   │
│                                  │                                   │
│  ┌───────────────────────────────┴───────────────────────────────┐  │
│  │                   Persistence Layer                             │  │
│  │  scripts/automation/hooks/registry.json   (Hook W/M)            │  │
│  │  scripts/automation/hooks/logs/hook_audit.log (Hook Run T)      │  │
│  │  scripts/automation/hooks/state/session_state.json (Session M) │  │
│  └───────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

### 2.2 層次構成 (Layered Structure)

| 層 | 名前 | 役割 | 主要 module |
|---|---|---|---|
| **L1** | UI Layer | "高级设置 → Hooks" 标签页 CRUD + 触发日志 | frontend Next.js 14+ |
| **L2** | API Layer | FastAPI 8080 hook API 端点 | `scripts/automation/console_server.py` (扩展) |
| **L3** | Hook Engine Layer | 事件分发 + fan-out 调度 + handler 执行 | Event Emitter + Fan-out Scheduler + Hook Runner |
| **L4** | Data Layer | registry / audit log / session state | Hook Registry + Audit Logger |
| **L5** | Builtin Layer | builtin hook (PreToolUse guard 等) 默认装载 | Builtin Hook Loader |
| **L6** | Persistence Layer | 文件系统 JSON / JSON Lines | registry.json + hook_audit.log + session_state.json |

### 2.3 跟 SRS-PRE-TOOL-USE-GUARD-001 / BD-PRE-TOOL-USE-GUARD-001 的关系

| 项 | PreToolUse guard (BD-001) | Hook 域 (本 BD-001) |
|---|---|---|
| 范围 | PreToolUse 事件点的 1 个 builtin guard hook | 14 事件点 + 4 action type 的 hook 体系 |
| 抽象层级 | 具体 guard | 上位 hook 框架 |
| 注册方式 | 写 `pre_tool_use_rules.json` | 写 `hooks/registry.json` |
| 集成点 | `console_server.py` line 80-92 + `dispatcher.py` invoke() 前 | 14 事件点 (覆盖 PreToolUse + 13 其他) |
| 跟 SRS 关系 | 下位 | 上位 (PreToolUse guard 是 hooks 域下 builtin hook 之一) |

**演进路径**: PreToolUse guard v0.1 (per BD-PRE-TOOL-USE-GUARD-001) → Hook 域 v0.1 (本 BD) 把 PreToolUse guard 收编为 builtin hook, 后续新事件点 (PostToolUse / SubagentDispatch / ...) 走同样 builtin 模式.

---

## §4 モジュール設計 (Module Design)

### 4.1 模块一览 (6 module)

| Module ID | 名称 | 责任 | 对应 FR | 估计 LOC |
|---|---|---|---|---|
| **EM-1** | Event Emitter | 14 类事件定义 + 事件 payload 构造 + 事件分发入口 | FR-2.1 + FR-2.4 | ~250 |
| **HR-2** | Hook Registry | registry.json 加载 / 热更新 / schema 校验 / CRUD | FR-3.1 + FR-3.2 | ~200 |
| **FS-3** | Fan-out Scheduler | per-event 多 hook 调度 + 串行/并行 + 优先级排序 | FR-4.2 + FR-4.3 | ~200 |
| **HR-4** | Hook Runner | 单 hook 执行 + timeout + retry + transform 累积 | FR-4.1 + FR-4.4 + FR-4.5 | ~250 |
| **AL-5** | Audit Logger | JSON Lines 写 hook_audit.log + Transaction append-only | FR-5.1 + FR-5.2 | ~150 |
| **BL-6** | Builtin Hook Loader | builtin hook (PreToolUse guard 等) 默认装载 + 不可删除 | FR-3.3 | ~100 |
| **合計** | | | | ~1150 LOC |

### 4.2 EM-1 Event Emitter

**职责**: 14 类事件定义 + 事件 payload 构造 + 事件分发入口.

**输入**: 上游调用方 (e.g. mavis runtime, console_server.py, dispatcher.py).

**输出**: `Event` 对象 + 触发 Hook Engine Layer.

**依赖**: 无 (L3 起点).

**对外接口**:

```python
class EventEmitter:
    def emit(self, event_type: str, payload: dict, session_id: str) -> EventResult:
        """触发 1 个事件, fan-out 到所有匹配的 hook.

        Args:
            event_type: 14 类事件之一 (e.g. "PreToolUse")
            payload: per-event payload (e.g. {"tool_name": "bash", "tool_args": {...}})
            session_id: 当前 session UUID

        Returns:
            EventResult: {
                decision: BLOCK|ASK|WARN|PASS,
                transformed_args?: dict,
                reason: str,
                latency_ms: float,
                hooks_executed: list[str]
            }
        """
```

**14 类事件 payload schema** (per SRS §FR-2.4):

| Event ID | payload 字段 |
|---|---|
| EVT-001 PreToolUse | `tool_name: str, tool_args: dict, agent_role: str` |
| EVT-002 PostToolUse | `tool_name: str, tool_result: any, latency_ms: float` |
| EVT-003 UserPromptSubmit | `prompt: str, user_id: str` |
| EVT-004 SessionStart | `session_id: str, workspace: str, runtime: str` |
| EVT-005 SessionEnd | `session_id: str, duration_ms: float, exit_reason: str` |
| EVT-006 SubagentDispatch | `task_id: str, script_path: str, args: dict, parent_session_id: str` |
| EVT-007 SubagentReturn | `task_id: str, result: any, success: bool` |
| EVT-008 ToolError | `tool_name: str, error: str, stack_trace: str` |
| EVT-009 FileWatch | `file_path: str, event_type: str, mtime: float` |
| EVT-010 CronTick | `cron_id: str, scheduled_at: datetime` |
| EVT-011 RuntimeScan | `runtimes: list[dict], poisoned: list[dict]` |
| EVT-012 WorkspaceSwitch | `from_workspace: str, to_workspace: str` |
| EVT-013 NetworkEgress | `url: str, method: str, headers: dict` |
| EVT-014 CustomEvent | `name: str, payload: dict` |

### 4.3 HR-2 Hook Registry

**职责**: registry.json 加载 / 热更新 / schema 校验 / CRUD.

**输入**: registry.json 文件 + 用户 CRUD 操作.

**输出**: `Hook` 对象列表 + 内存缓存.

**依赖**: `jsonschema` (1 依赖, schema 校验), `watchdog` (1 依赖, 文件 mtime 监听).

**对外接口**:

```python
class HookRegistry:
    def load(self) -> List[Hook]:
        """从 registry.json 加载所有 hook, 含 builtin hook 合并."""

    def reload(self) -> None:
        """热更新: 监听 mtime 变化, 自动 reload."""

    def register(self, hook: Hook) -> None:
        """创建 1 个 hook, 写入 registry.json."""

    def update(self, name: str, hook: Hook) -> None:
        """更新 1 个 hook."""

    def enable(self, name: str) -> None:
        """启用 hook (修改 enabled=true, 热更新)."""

    def disable(self, name: str) -> None:
        """禁用 hook (修改 enabled=false, 热更新)."""

    def archive(self, name: str) -> None:
        """归档 hook (修改 archived=true, 热更新, 不物理删除)."""

    def get(self, name: str) -> Optional[Hook]:
        """按 name 查询 1 个 hook."""

    def list_by_event(self, event_type: str) -> List[Hook]:
        """按 event_type 查询所有启用的 hook (含 builtin)."""
```

**Hook schema** (per SRS §4.3.1):

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "properties": {
    "name": { "type": "string", "pattern": "^[a-z0-9_-]+$" },
    "event_type": { "enum": ["PreToolUse", "PostToolUse", ..., "CustomEvent"] },
    "action_type": { "enum": ["pre", "post", "block", "transform"] },
    "handler": { "type": "string", "description": "Python 模块路径或 builtin hook 名" },
    "enabled": { "type": "boolean", "default": true },
    "archived": { "type": "boolean", "default": false },
    "priority": { "type": "integer", "minimum": 0, "maximum": 1000, "default": 100 },
    "parallel": { "type": "boolean", "default": false },
    "timeout_ms": { "type": "integer", "minimum": 100, "maximum": 30000, "default": 1000 },
    "retry": { "type": "integer", "minimum": 0, "maximum": 3, "default": 0 },
    "description": { "type": "string" },
    "created_at": { "type": "string", "format": "date-time" },
    "updated_at": { "type": "string", "format": "date-time" },
    "disabled_at": { "type": "string", "format": "date-time" }
  },
  "required": ["name", "event_type", "action_type", "handler"]
}
```

### 4.4 FS-3 Fan-out Scheduler

**职责**: per-event 多 hook 调度 + 串行/并行 + 优先级排序.

**输入**: 1 个 event + 1 个 hook 列表 (按 event_type 过滤).

**输出**: 聚合决策 (BLOCK / ASK / WARN / PASS) + transformed_args.

**依赖**: HR-2 (Hook Registry), HR-4 (Hook Runner).

**调度算法** (per SRS §FR-2.3 + §FR-4.2):

```
1. 接收 event, 调用 HR-2.list_by_event(event.event_type) 获取 hook 列表
2. 按 priority 升序排序 (priority 越小越先执行)
3. 同 priority 分组:
   a. parallel=false: 串行执行 (per SRS §FR-4.3)
   b. parallel=true: 并行执行 (max 4, per SRS §FR-4.3)
4. 对每个 hook 调用 HR-4.run(hook, event):
   - 返回 BLOCK: 立即停止 fan-out, 返回 BLOCK (per SRS §FR-2.3)
   - 返回 ASK: 立即停止 fan-out, 走 ask_user (per SRS §FR-2.3)
   - 返回 PASS/WARN: 继续下一个 hook
   - 返回 transform: 累积 transformed_args, 继续下一个 hook (后覆盖前, per SRS §FR-4.4)
5. 返回聚合结果: {
     decision: 最高优先级决策 (BLOCK > ASK > WARN > PASS),
     transformed_args: 最后 1 个 transform 结果,
     hooks_executed: 实际执行的 hook 列表,
     latency_ms: 总耗时
   }
```

**决策优先级** (per SRS §FR-2.3):

| 决策 | 优先级 | 行为 |
|---|---|---|
| BLOCK | 1 (最高) | 立即停止 fan-out, 阻断原动作 |
| ASK | 2 | 立即停止 fan-out, 走 ask_user |
| WARN | 3 | 继续 fan-out, log warning |
| PASS | 4 (最低) | 继续 fan-out, 无动作 |
| transform | (累积) | 不参与决策优先级, 累积 transformed_args |

### 4.5 HR-4 Hook Runner

**职责**: 单 hook 执行 + timeout + retry + transform 累积.

**输入**: 1 个 hook + 1 个 event.

**输出**: `HookResult` 对象.

**依赖**: AL-5 (Audit Logger, 写日志).

**对外接口**:

```python
class HookRunner:
    def run(self, hook: Hook, event: Event, context: Context) -> HookResult:
        """执行 1 个 hook, 返回 HookResult.

        Args:
            hook: 1 个 hook 定义
            event: 触发的事件
            context: 执行上下文 (session_id / actor / trace_id)

        Returns:
            HookResult: {
                decision: BLOCK|ASK|WARN|PASS|transform,
                reason?: str,
                rule_id?: str,
                transformed_args?: dict,
                latency_ms: float
            }
        """

    def _execute_handler(self, hook: Hook, event: Event) -> HookResult:
        """实际执行 hook handler, 含 timeout / retry / 异常处理.

        异常处理 (per SRS §FR-4.5):
        - handler 抛异常: 视为 WARN, 继续下一个 hook, 写 audit log (含异常 stack trace, log level = ERROR)
        - handler 超时: 视为 BLOCK (安全优先), 写 audit log (含 timeout 原因)
        - handler 返回非法 decision: 视为 WARN, 写 audit log
        """
```

### 4.6 AL-5 Audit Logger

**职责**: JSON Lines 写 hook_audit.log + Transaction append-only.

**输入**: `HookRun` 对象 (每次 hook 执行记录).

**输出**: 追加写入 hook_audit.log.

**依赖**: 文件系统权限 (chmod 444, 不可物理删除, per SRS §NFR-S-2).

**对外接口**:

```python
class AuditLogger:
    def log_run(self, run: HookRun) -> None:
        """写 1 条 hook run 记录到 hook_audit.log (JSON Lines).

        HookRun 字段 (per SRS §4.3.2, 12 字段):
        - run_id: UUID
        - timestamp: ISO 8601 datetime
        - session_id: UUID
        - event_type: str (14 类事件之一)
        - hook_name: str
        - action_type: str (4 种之一)
        - decision: BLOCK|ASK|WARN|PASS|transform
        - rule_id?: str
        - transformed_args?: dict (transform decision 时)
        - before_args: dict (事件 payload)
        - after_args: dict (transformed_args 或 before_args)
        - env_hash: str (env var 哈希, 凭据 0 外泄验证)
        - latency_ms: float
        """

    def query(self, filters: dict) -> List[HookRun]:
        """按 hook_name / event_type / decision / 时间范围 查询 audit log."""
```

**HookRun schema** (per SRS §4.3.2, JSON Lines 格式):

```json
{
  "run_id": "uuid-v4",
  "timestamp": "2026-09-24T12:34:56.789Z",
  "session_id": "uuid-v4",
  "event_type": "PreToolUse",
  "hook_name": "pre_tool_use_guard",
  "action_type": "pre",
  "decision": "BLOCK",
  "rule_id": "BLOCK-001",
  "before_args": { "tool_name": "bash", "tool_args": { "command": "rm -rf /" } },
  "after_args": null,
  "env_hash": "sha256:abc123...",
  "latency_ms": 3.2
}
```

### 4.7 BL-6 Builtin Hook Loader

**职责**: builtin hook (PreToolUse guard 等) 默认装载 + 不可删除.

**输入**: `scripts/automation/hooks/builtin/` 目录下的 Python 模块.

**输出**: builtin hook 列表 (per startup).

**依赖**: HR-2 (Hook Registry, 合并 builtin hook 到内存缓存).

**对外接口**:

```python
class BuiltinHookLoader:
    def load_all(self) -> List[Hook]:
        """从 scripts/automation/hooks/builtin/ 加载所有 builtin hook.

        内置 builtin hooks (v0.1):
        - pre_tool_use_guard: PreToolUse guard (per SRS-PRE-TOOL-USE-GUARD-001)
        - session_start_cleanup: SessionStart 时清理临时文件
        - session_end_summary: SessionEnd 时生成 session 总结
        - subagent_dispatch_audit: SubagentDispatch 前置审计
        - tool_error_fallback: ToolError 时自动 fallback

        所有 builtin hook:
        - 启动时自动装载
        - 用户可在 UI 禁用 (enabled=false)
        - 不可删除 (UI 无删除按钮)
        """

    def is_builtin(self, name: str) -> bool:
        """检查 hook 是否是 builtin."""
```

### 4.8 Session State (跟 skills 域共享, per SRS §FR-7.2)

**职责**: 跨标签页 (Skills + Hooks) 共享 session state.

**存储**: `scripts/automation/hooks/state/session_state.json` (per session_id, 跟 skill session state 平行).

**字段**: `session_id / started_at / last_activity_at / active_skills / active_hooks / shared_state: dict`.

---

## §5 データモデル (Data Model)

### 5.1 3 表 W/T/M 100% 覆盖 per 守门 #13

| 表 ID | 名称 | 类型 | 职责 | 估计行数 (1 session) |
|---|---|---|---|---|
| **TBL-HOOK** | `hooks` | **W/M** | hook 定义 + 状态 (registered / enabled / disabled / archived) | ~50 hooks |
| **TBL-HOOK-RUN** | `hook_runs` | **T** | 单次 hook 执行记录 (append-only) | ~1000 runs/day |
| **TBL-SESSION** | `hook_session_state` | **M** | session 级 hook state (跨标签页共享) | 1 row/session |

### 5.2 TBL-HOOK (W/M, per 守门 #13)

**存储**: `scripts/automation/hooks/registry.json` (JSON 文件, git tracked).

**Schema** (per §4.3):

```sql
-- 概念表, 实际是 JSON 文件
CREATE TABLE hooks (
    name TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    action_type TEXT NOT NULL CHECK(action_type IN ('pre', 'post', 'block', 'transform')),
    handler TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    archived BOOLEAN NOT NULL DEFAULT false,
    priority INTEGER NOT NULL DEFAULT 100,
    parallel BOOLEAN NOT NULL DEFAULT false,
    timeout_ms INTEGER NOT NULL DEFAULT 1000,
    retry INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    is_builtin BOOLEAN NOT NULL DEFAULT false,  -- builtin hook 标记
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    disabled_at TIMESTAMP,
    archived_at TIMESTAMP
);
```

**索引**: `(event_type, enabled, archived)` 复合索引 (per fan-out 查询).

### 5.3 TBL-HOOK-RUN (T, per 守门 #13)

**存储**: `scripts/automation/hooks/logs/hook_audit.log` (JSON Lines, append-only).

**Schema** (per §4.6):

```sql
-- 概念表, 实际是 JSON Lines 文件
CREATE TABLE hook_runs (
    run_id UUID PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL,
    session_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    hook_name TEXT NOT NULL,
    action_type TEXT NOT NULL,
    decision TEXT NOT NULL CHECK(decision IN ('BLOCK', 'ASK', 'WARN', 'PASS', 'transform')),
    rule_id TEXT,
    transformed_args JSONB,
    before_args JSONB NOT NULL,
    after_args JSONB,
    env_hash TEXT NOT NULL,
    latency_ms REAL NOT NULL
);
```

**索引**: 概念索引 `(session_id, timestamp)` + `(hook_name, decision)` (per audit log 查询).

**append-only 保证** (per SRS §FR-5.2 + NFR-S-2):

- 文件权限: `chmod 444` (只读, mavis 进程外不可写)
- 不允许 truncate
- 不允许物理删除
- 保留 90 天 (per SRS §FR-5.1)

### 5.4 TBL-SESSION (M, per 守门 #13)

**存储**: `scripts/automation/hooks/state/session_state.json` (per session_id).

**Schema**:

```sql
CREATE TABLE hook_session_state (
    session_id UUID PRIMARY KEY,
    started_at TIMESTAMP NOT NULL,
    last_activity_at TIMESTAMP NOT NULL,
    active_skills JSONB,  -- 跟 skills 域共享
    active_hooks JSONB,   -- 当前启用的 hook 列表
    shared_state JSONB,   -- 跨标签页共享 state
    total_runs INTEGER NOT NULL DEFAULT 0,
    total_blocks INTEGER NOT NULL DEFAULT 0
);
```

**跟 skills 域 session state 关系**: `active_skills` 字段引用 skills 域 session state (per SRS-MULTICA-SKILL-001 §1.3 隐含).

---

## §6 インターフェース設計 (Interface Design)

### 6.1 Hook Event 契约 (Internal API)

```python
# scripts/automation/hooks/event_emitter.py

from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Union

@dataclass
class Event:
    event_id: str  # UUID
    event_type: str  # 14 类事件之一
    timestamp: str  # ISO 8601
    session_id: str  # UUID
    payload: Dict[str, Any]  # per-event payload

@dataclass
class HookResult:
    decision: str  # BLOCK|ASK|WARN|PASS|transform
    reason: Optional[str] = None
    rule_id: Optional[str] = None
    transformed_args: Optional[Dict[str, Any]] = None
    latency_ms: float = 0.0

@dataclass
class EventResult:
    decision: str  # 聚合决策
    transformed_args: Optional[Dict[str, Any]] = None
    reason: str = ""
    latency_ms: float = 0.0
    hooks_executed: List[str] = None  # 实际执行的 hook 名列表

# 调用方式
result: EventResult = event_emitter.emit(
    event_type="PreToolUse",
    payload={"tool_name": "bash", "tool_args": {"command": "rm -rf /"}},
    session_id="uuid-v4"
)
```

### 6.2 mavis runtime 接口 (Hook 触发点集成)

| 集成点 | 文件 | 调用方式 | 触发事件 |
|---|---|---|---|
| **console_server.py** | line 80-92 (现有) | `event_emitter.emit("PreToolUse", payload, session_id)` | PreToolUse (跟现有 hook 接入点一致) |
| **dispatcher.py** | invoke() 前 (现有) | `event_emitter.emit("SubagentDispatch", payload, session_id)` | SubagentDispatch |
| **dispatcher.py** | 子代理完成后 | `event_emitter.emit("SubagentReturn", payload, session_id)` | SubagentReturn |
| **mavis runtime** | session 启动 | `event_emitter.emit("SessionStart", payload, session_id)` | SessionStart |
| **mavis runtime** | session 退出 | `event_emitter.emit("SessionEnd", payload, session_id)` | SessionEnd |
| **mavis runtime** | tool 异常 | `event_emitter.emit("ToolError", payload, session_id)` | ToolError |
| **mavis runtime** | tool 完成 | `event_emitter.emit("PostToolUse", payload, session_id)` | PostToolUse |
| **watchdog** | 文件 mtime 变化 | `event_emitter.emit("FileWatch", payload, session_id)` | FileWatch |
| **cron** | tick 触发 | `event_emitter.emit("CronTick", payload, session_id)` | CronTick |
| **runtime scan** | 扫描完成 | `event_emitter.emit("RuntimeScan", payload, session_id)` | RuntimeScan |
| **workspace switch** | 切换 workspace | `event_emitter.emit("WorkspaceSwitch", payload, session_id)` | WorkspaceSwitch |
| **network egress** | 出站调用前 | `event_emitter.emit("NetworkEgress", payload, session_id)` | NetworkEgress (v0.1 stub) |

### 6.3 UI API 接口 (FastAPI 8080, 8 端点)

| 端点 | 方法 | 入参 | 出参 | 说明 |
|---|---|---|---|---|
| `/api/hooks/list` | GET | `event_type?: str` | `List[Hook]` | 查询所有 hook (按 event_type 过滤) |
| `/api/hooks/get/{name}` | GET | `name: str` | `Hook` | 查询单 hook |
| `/api/hooks/create` | POST | `Hook` | `Hook` | 创建 hook |
| `/api/hooks/update/{name}` | PUT | `name: str, Hook` | `Hook` | 更新 hook |
| `/api/hooks/enable/{name}` | POST | `name: str` | `Hook` | 启用 hook |
| `/api/hooks/disable/{name}` | POST | `name: str` | `Hook` | 禁用 hook |
| `/api/hooks/archive/{name}` | POST | `name: str` | `Hook` | 归档 hook |
| `/api/hooks/runs` | GET | `filters: dict` | `List[HookRun]` | 查询 audit log |
| `/api/hooks/reload` | POST | (无) | `{reloaded: int}` | 手动触发 registry 热更新 |
| `/api/hooks/builtins` | GET | (无) | `List[Hook]` | 查询所有 builtin hook |

### 6.4 ask_user 接口 (跟 SRS-PRE-TOOL-USE-GUARD-001 §FR-2.2 一致)

ASK 决策时, 走 `ask_user` 工具, 2-4 选项, 至少 1 个标"**(推荐) 取消**" (per 守门 v28):

```python
ask_user(
    title="Hook 触发二次确认",
    question=f"Hook '{hook.name}' 触发 ASK 决策, 是否继续?",
    options=[
        "取消 (推荐)",
        "确认执行",
        "禁用此 hook",
    ]
)
```

### 6.5 跟 skills 域共享接口 (per SRS §FR-7)

```python
# 跨标签页共享 session state (per SRS §FR-7.2)
shared_state = session_state_manager.get_shared_state(session_id)
shared_state["active_hooks"] = active_hooks_list
shared_state["active_skills"] = active_skills_list  # 跟 skills 域共享
session_state_manager.save_shared_state(session_id, shared_state)
```

---

## §7 UI 設計 (UI Design)

### 7.1 "高级设置" 导航架构

```
┌─────────────────────────────────────────────────────────┐
│  STAR Platform - 顶层导航                                │
│  ├─ 项目                                               │
│  ├─ 任务                                               │
│  ├─ 画布                                               │
│  ├─ ...                                               │
│  └─ 高级设置  ← (ULYS-235 拍板)                          │
│     ├─ [Skills]  ← SRS-MULTICA-SKILL-001 v0.1           │
│     ├─ [Hooks]   ← 本 BD (per ULYS-235)                │
│     ├─ [MCP]     ← SRS-MULTICA-MCP-001 v0.1 stub (per 2026-09-24 15:01 JST 用户拍板 "MCP也应该是一个标签页")
│     ├─ [Plugins] ← SRS-MULTICA-PLUGIN-001 v0.1 stub (per 2026-09-24 22:04 JST 用户拍板 "还有plugins也应该是一个标签页")
│     ├─ [Commands]  ← (预留, 未来扩展)                   │
│     └─ [Agents]    ← (预留, 未来扩展)                   │
└─────────────────────────────────────────────────────────┘
```

### 7.2 Hooks 标签页布局 (3 区域, per SRS §FR-6.1)

```
┌──────────────────────────────────────────────────────────────────────┐
│  高级设置 → Hooks                                                     │
├────────────────┬──────────────────────────────┬─────────────────────┤
│ Hook 列表       │ Hook 详情                     │ 触发日志             │
│ (按 event_type  │ (name / event / action /      │ (audit log 查询)    │
│  分组)          │  handler / priority /          │                     │
│                │  timeout / enabled toggle)     │                     │
│ ▼ PreToolUse   │                              │ 过滤:               │
│   - pre_guard  │ 名称: pre_tool_use_guard     │ [hook_name ▼]       │
│   - custom_1   │ 事件: PreToolUse             │ [event_type ▼]      │
│ ▼ PostToolUse  │ 动作: pre                    │ [decision ▼]        │
│   - audit_log  │ Handler: builtin             │ [时间范围]          │
│ ▼ SubagentDis… │ Priority: 10                 │                     │
│   - dispatch…  │ Timeout: 1000ms              │ 12:34:56 BLOCK      │
│                │ Enabled: [✓]                  │ 12:34:50 PASS       │
│ [+ New Hook]   │                              │ 12:34:45 BLOCK      │
│                │ [Update] [Archive]            │ ...                 │
└────────────────┴──────────────────────────────┴─────────────────────┘
```

### 7.3 路由 (Next.js 14+)

```
/settings/advanced/hooks        # Hooks 标签页 (本 BD)
/settings/advanced/skills       # Skills 标签页 (per skill SRS)
/settings/advanced/mcp          # MCP 标签页 (per MCP SRS v0.1 stub, 2026-09-24 15:01 JST 用户拍板 "MCP也应该是一个标签页")
/settings/advanced/plugins      # Plugins 标签页 (per Plugin SRS v0.1 stub, 2026-09-24 22:04 JST 用户拍板 "还有plugins也应该是一个标签页")
/settings/advanced/commands    # (预留)
/settings/advanced/agents      # (预留)
```

以上是稳定的产品路由约定。当前 Phase 9C 使用共享 Advanced Settings 外层布局和动态 tab 页面；它们映射到相同 canonical route，不改变 Hooks 的选项卡层级：

| 产品路由 | 当前 UI 文件 | 状态 |
|---|---|---|
| `/settings/advanced/hooks` | `frontend/src/app/(app)/settings/advanced/[tab]/page.tsx` + `HookPolicyPage.tsx` | 🟡 Builder 条件式 UI 已加入；需宿主 session Provider 才可连真实 API |
| Advanced Settings tab shell | `frontend/src/app/(app)/settings/advanced/layout.tsx` | 🟢 Settings 主侧栏父入口下，页面内容区含 Skills / Hooks / MCP / Plugins 同级局部标签导航 |
| Advanced Settings default route | `frontend/src/app/(app)/settings/advanced/page.tsx` | 🟢 重定向至 Hooks tab |

### 7.4 前端组件清单

| 组件 | 路径 | 职责 |
|---|---|---|
| `page.tsx` | `frontend/src/app/(app)/settings/advanced/hooks/page.tsx` | Hooks 标签页主入口 (3 区域布局) |
| `HookList.tsx` | (同级) | 左: hook 列表 (按 event_type 分组) |
| `HookDetail.tsx` | (同级) | 中: hook 详情 + 编辑 + enable/disable toggle |
| `HookRunLog.tsx` | (同级) | 右: 触发日志 (audit log 查询) |
| `NewHookDialog.tsx` | (同级) | 创建 hook 表单弹窗 |
| `useHooks.ts` | (同级) | React hook: 调 hooks/ API |

### 7.5 Phase 9C 实装状态与运行时边界

Phase 9C 已加入 Settings 侧栏的“高级设置”入口及其 Skills / Hooks / MCP / Plugins 并列 tab shell；Hooks 页使用 Project/Worktree 授权选择、typed restrictive rule 表单、Draft CAS、admin publish、rollback 和 policy configuration Audit。规则编辑器只提交 Rust DTO，不允许任意代码、shell 或 plugin callback。UI 的配置 Audit 不代表实际 Hook 执行日志，也不代替 Phase 9D 的 RunEvent/outbox/BI。

### 7.6 Phase 9D Hook execution event 与 BI 接缝

Phase 9D 先为无 Task/Run 绑定的 Worktree archive lifecycle Hook 增加 `multica.hook_execution_event` Transaction 账本；Run-linked Hook 事件仍使用有 `(work_item_id, run_id)` 外键的 `multica.task_execution_run_event`。事件源只记录已脱敏的 actor/tenant/project/worktree/task/run/correlation scope、decision/reason、policy/evaluator version/digest、duration/timeout 与 bounded freshness facts，不写入 prompt、command、stdout/stderr 或 Secret。Worktree archive producer 将事件与 Worktree 确认及管理审计放在同一事务；事件写入失败则整体 fail closed。

`GET /api/v1/projects/{project_id}/hook-events` 以 `hook:read`、当前 Project membership 和 tenant RLS 授权，使用绑定 Project 的 keyset cursor，页大小上限 100，只返回 allowlisted projection 且设置 no-store。当前 instrumented phase 仅 `worktree_archive`，响应 `coverage.status=partial`、`reported_percentage=null`；其他 phase 属尚未接入/unknown，不能按零处理。Hooks 页已有独立的“Hook 执行事件”只读面板，使用同一 Project selector、每页最多 30 条、页面最多驻留 300 条并支持 cursor 续读，清楚呈现 partial/unknown coverage；策略配置 Audit 与执行事件分开展示。该表是只读事件账本和 BI 输入源，不等同于有投递状态的 Outbox；Run/tool/validation/review producer、事件投递/重放状态、跨 Run/Worktree BI read model、聚合口径、Run Detail/Quality & Improvement 下钻与目标 DB/RLS/grants 验收仍未完成。导航保持既有要求：Hooks 是高级设置内与 Skills/MCP/Plugins 并列的 tab。

Phase 9D 的 `GET /api/v1/projects/{project_id}/hook-events/summary?window_days=` 当前只汇总该 archive ledger。Hooks 标签页在事件面板内增加近 7/30/90 天汇总卡与 phase/decision 分组，展示事件/超时/时长和账本中携带 Run ID 的行数；UI 显示 `partial` coverage、未接入 phase 为 unknown、Run outcome join 未接入。它是有限的数据源视图，不替代 Project Quality & Improvement/Run Detail 最终 BI。汇总和事件列表仍位于原 Advanced Settings 内容区标签条之下，不新增侧栏或 Worktree 导航项。

当前 Group API client 没有从 app root `Providers` 获得认证 token/session generation，故页面在缺失 session/provider 时不加载 seed、不展示伪策略，也不开放保存/发布；事件面板也不会读取或伪造未授权数据。真实 Project baseline 未配置时同样明确阻断 Worktree overlay。已有的受控 Provider 测试只验证组件消费契约，不等于 app session 装配、浏览器真实授权读取、目标数据库/RLS 部署或生产写入验收。Task/Run 生命周期仍以独立 Run attempt 为单位，Worktree 是可选执行环境；Run 历史不随 Worktree 归档或清理删除。Project BI、固定 Benchmark 和 Improvement proposal 留在 Project Quality & Improvement 视图，不增加到 Project→Worktree→Task 导航层级。

---

## §8 セキュリティ設計 (Security Design)

### 8.1 凭据零外泄 (per SRS §NFR-S-1, 跟 SRS-PRE-TOOL-USE-GUARD §8.1.4 一致)

- 所有 hook run 必含 `env_hash` 字段 (sha256(env vars), 不存原始值)
- audit log 不存敏感字段 (env var value / token / private key)
- registry.json 不存 secret (per 守门 #5)
- handler 不允许直接读 env var (强制走 `context.env_hash`)

### 8.2 audit log 完整性 (per SRS §NFR-S-2)

- 文件权限: `chmod 444 hook_audit.log` (只读)
- append-only: 不允许 truncate, 不允许物理删除
- 保留: 90 天 (per SRS §FR-5.1)
- 校验: 每日 hash check (跟昨日 hash 链式校验)

### 8.3 registry 文件只读 (per SRS §NFR-S-3)

- 文件权限: `chmod 644 registry.json` (mavis 进程可写, 其他进程只读)
- schema 校验: 启动时 + 每次 reload 必校验, 失败 → fail-open (per SRS §NFR-A-1)
- builtin hook 不可删除 (UI 无删除按钮, per SRS §NFR-S-4)

### 8.4 fail-open / fail-closed 策略 (per SRS §FR-3.2 + §FR-5.2)

| 场景 | 策略 | 原因 |
|---|---|---|
| registry.json 加载失败 | **fail-open** | 不能因为 registry bug 阻断整个 mavis runtime (per SRS §NFR-A-1) |
| registry.json schema 校验失败 | **fail-open** + WARN log | 同上 |
| audit log 写失败 | **fail-closed** (返回 BLOCK) | 凭据外泄零容忍, 审计是最后一道防线 (per SRS §NFR-A-2) |
| handler 抛异常 | **WARN** | 继续 fan-out, 写 audit log (per SRS §FR-4.5) |
| handler 超时 | **BLOCK** | 安全优先 (per SRS §FR-4.5) |

---

## §9 非機能設計 (Non-Functional Design)

### 9.1 性能设计 (per SRS §NFR-P)

| 项 | 阈值 | 设计 |
|---|---|---|
| 单 hook 延迟 p50 | < 5ms | handler 调用走 sync 路径, 不走 LLM |
| 单 hook 延迟 p99 | < 10ms | timeout 默认 1000ms, 超时视为 BLOCK |
| 14 事件 fan-out 总延迟 p99 | < 50ms (5 hook 平均) | 同 priority 串行, 最多 5 hook |
| audit log 写延迟 | < 1ms | 异步 fsync (per PreToolUse guard 一致) |
| registry 热更新延迟 | < 100ms | watchdog mtime 监听, 内存 cache reload |
| mavis runtime 主流程额外延迟 | 0% | event_emitter 同步路径, 不影响 |

### 9.2 可用性设计 (per SRS §NFR-A)

| 项 | 阈值 | 设计 |
|---|---|---|
| registry 加载失败 → fail-open | 100% | JSON parse 异常时, 跳过 registry, builtin hook 仍可用 |
| audit log 写失败 → fail-closed | 100% | 磁盘满 / 权限不足时, 返回 BLOCK |
| hook handler 抛异常 → WARN | 100% | try/except 包住 handler, 异常视为 WARN |
| registry 热更新不中断 | 0 中断 | 双 buffer 切换 (old + new), 切换时无锁 |
| 启动时间不显著增加 | < 100ms | 14 事件装载 + builtin hook 装载 < 100ms |

### 9.3 可观测性设计 (per SRS §NFR-O)

| 项 | 设计 |
|---|---|
| 命中统计 metric | decision / event_type / hook_name / session_id 4 维度 |
| 错误 trace | 0 静默吞错, 所有异常写 audit log (log level = ERROR) |
| BLOCK 事件触发 root session 通知 | < 500ms 推送 (跟 PreToolUse guard 一致) |
| UI "触发日志" 标签可见 | 100% 可访问, 不过滤决策 |

---

## §10 障害 / 運用設計 (Fault / Operations Design)

### 10.1 故障场景与恢复 (per SRS §NFR-A)

| 故障 | 检测 | 恢复 | 数据保护 |
|---|---|---|---|
| registry.json 损坏 | 启动 / 热更新 时 JSON parse 失败 | fail-open + WARN log | builtin hook 仍可用 |
| registry.json schema 校验失败 | jsonschema 校验异常 | fail-open + WARN log | builtin hook 仍可用 |
| audit log 写失败 (磁盘满) | write 返回 IOError | fail-closed (返回 BLOCK) | 写磁盘前 sync, 减少丢失 |
| audit log 写失败 (权限不足) | write 返回 PermissionError | fail-closed (返回 BLOCK) | 启动时 chmod 444 |
| handler 抛异常 | try/except 捕获 | 视为 WARN + audit log | log level = ERROR |
| handler 超时 | signal.alarm 或 threading.Timer | 视为 BLOCK + audit log | timeout 默认 1000ms |
| watchdog 失效 | mtime 监听无事件 | 手动 reload API (`/api/hooks/reload`) | 30 min 定期 reload |

### 10.2 Log Rotation (per SRS §FR-5.1)

- 保留: 90 天
- 触发: 每日 0:00 定时检查, 按天切分 `hook_audit.YYYY-MM-DD.log`
- 压缩: 30 天前的 log gzip 压缩
- 清理: 90 天前的 log 删除 (跟 SRS-PRE-TOOL-USE-GUARD §FR-4.1 一致)

### 10.3 监控与告警 (per SRS §NFR-O)

| 指标 | 阈值 | 告警 |
|---|---|---|
| BLOCK 事件 / 1h | > 50 | 通知 root session (per NFR-O-3) |
| handler 异常 / 1h | > 10 | 通知 root session |
| audit log 写失败 | > 0 | BLOCK 全部 (fail-closed) |
| registry 热更新失败 | > 0 | 通知 root session |
| mavis runtime 启动延迟 | > 100ms | 通知 root session |

---

## §11 用語集 (跟 SRS 共享 + BD 新增)

跟 SRS §2 共享 24 条. 本 BD 新增 8 条:

| 用語 | 定義 |
|---|---|
| **Hook Engine Layer** | L3 抽象层, 包含 Event Emitter + Fan-out Scheduler + Hook Runner 3 module |
| **Hook 内存缓存** | registry.json 加载到内存的 hook 列表, 用于 fan-out 查询, 跟磁盘文件保持同步 |
| **双 buffer 热更新** | registry 热更新时, old buffer + new buffer 并存, 切换时无锁 |
| **聚合决策** | fan-out 调度后, 多个 hook 结果聚合为 1 个最终决策 (BLOCK > ASK > WARN > PASS) |
| **transform 累积** | 多个 hook 返回 transform 时, 后执行的覆盖前一个的 transformed_args |
| **decision 优先级** | BLOCK (1) > ASK (2) > WARN (3) > PASS (4), 用于聚合决策 |
| **mtime 监听** | 通过 watchdog 监听 registry.json 文件修改时间, 触发自动 reload |
| **跨标签页共享 state** | Advanced Settings tabs 可共享已授权的 UI session 状态；Project/Worktree facts 与策略只从当前 API session 读取，不跨 session 复用或回退 seed (per SRS §FR-7.2) |

---

## §12 签字栏 (Sign-off)

| 角色 | 氏名 | 签字 | 日期 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | ✅ 2026-09-24 | 2026-09-24 JST |
| SRE Lead | SRE Lead (Mavis 临时代签 per 9/3 11:35 JST 拍板 B, 真人到位后追溯) | ✅ 2026-09-24 | 2026-09-24 JST |
| 平台 Lead | 平台 Lead (Mavis 临时代签 per 守门 #14 v3, 真人到位后追溯) | ✅ 2026-09-24 | 2026-09-24 JST |
| 评审主持 | 评审主持 (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-24 | 2026-09-24 JST |
| PM | PM (Mavis 临时代签 per 守门 #14 v3) | ✅ 2026-09-24 | 2026-09-24 JST |

(per 守门 #14 v4 反转 v0.62, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)

---

## §13 修订履歴 (詳細)

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-24 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 6 module (EM-1 + HR-2 + FS-3 + HR-4 + AL-5 + BL-6) + 3 表 W/T/M (Hook W/M + Hook Run T + Session M, 100% 覆盖 per 守门 #13) + 14 事件 + 4 action type + 5 集成点 + 8 API 端点 + 12 文件 + 30 项 NFR + UI "高级设置 → Hooks" 标签页 (3 区域布局 + Next.js 14+ 路由), 守门 8/8 通过, IPA 13 段结构 (目的 / 範囲 / アーキテクチャ / モジュール / データ / IF / UI / セキュリティ / 非機能 / 障害 / 用語 / 签字 + 修订) | ULYS-235 (2026-09-24 20:xx JST) "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计" |
| **v0.2** | 2026-09-24 15:01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | MCP 升格标签页: §1.2 范围 + §7.1 导航 ASCII 图 + §7.3 路由 Next.js 14+ (`/settings/advanced/mcp` 加入实装) + §3 架构 ASCII 图更新 (Skills / Hooks / MCP 三标签页); commands / agents 仍"预留"占位 | 2026-09-24 15:01 JST Ulysses 评论 "MCP也应该是一个标签页" |
| **v0.3** | 2026-09-24 22:04 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Plugins 升格标签页: §1.2 范围更新 + §7.1 导航 ASCII 图增加 `[Plugins]` + §7.3 路由增加 `/settings/advanced/plugins` + §3 架构 ASCII 图更新为四标签页 (Skills / Hooks / MCP / Plugins); commands / agents 仍"预留"占位 | 2026-09-24 22:04 JST Ulysses 评论 "还有plugins也应该是一个标签页" |
| **v0.4** | 2026-09-24 22:13 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 自审饱和: §0 上位 SRS 引用 v0.1→v0.3 + §0 派生来源 SRS 引用 v0.1→v0.3 + §1.2 范围 v0.1+v0.3→v0.2+v0.3 + §1.3 关联文档 SRS 行 v0.1→v0.3; 4 处 cross-reference staleness 修正 | 2026-09-24 22:13 JST Ulysses 评论 "自审, 各级文档都要做到位" |
| **v0.5** | 2026-09-30 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将高级设置 Hooks tab 的既有导航决策复用为 Rust-native 可视规则管理入口；明确 typed condition/action、Project baseline + Worktree restrictive overlay、不可覆盖 builtin、fail-closed cleanup/Run gate 与 BI/RunEvent provenance；旧 Python/Next.js 方案只作为兼容草案 | 用户要求 Hook 原生实现、可视化配置并纳入 Worktree/BI，同时澄清应位于高级设置标签栏 |
| **v0.5.1** | 2026-09-30 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 交叉引用更新至 SRS v0.5.1 与总要件 v5.20；设计决策未变 | Phase 8B 扩展总需求后同步当前上位基线 |
| **v0.5.2** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 SRS v0.5.2；落实 Advanced Settings 侧栏入口与 Hooks 并列 tab 的路由/可视策略 Builder 代码切片；明确 host session、真实策略读取/写入与 Phase 9D 执行 RunEvent/BI 尚未验收，并校正 Task/Run/Worktree/Project BI 的关系 | 用户再次确认 Hooks 是高级设置中的选项卡，并要求融合 Run/BI 架构方向 |
| **v0.5.3** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义 Phase 9D Worktree archive append-only HookEvent T 表、同事务 fail-closed producer、Project-scoped keyset read API 与 partial/unknown coverage；明确该表是事件账本而非异步 Outbox，Run-linked Event、BI consumer/read model、Advanced Settings 事件 UI 和目标 DB/RLS/grants 验收仍开放；高级设置 Hooks tab 不变 | Phase 9D 首个生命周期 HookEvent/API 代码切片落地 |
| **v0.5.4** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Project-scoped Hook execution event 面板加入既有 Advanced Settings → Hooks 页面；与策略 Audit 分开展示、限制分页并显式呈现 partial/unknown coverage；记录 targeted Rust/前端/TypeScript 与隔离 PostgreSQL 证据，保留 app Provider、目标 DB/grants/RLS 和完整 BI 未验收边界 | Phase 9D Hooks tab 事件面板和 UI/API contract tests 落地 |
| **v0.5.5** | 2026-10-01 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 SRS v0.5.3；明确高级设置局部标签条与主侧栏父入口层级；加入既有 Hooks 标签内 summary UI 的窗口选择、phase/decision 摘要与 partial/unknown 文案，并标记 Run outcome、目标 DB/auth 与完整 BI 仍未验收 | Phase 9D summary consumer 接入并复用 ULYS-235 导航 |
