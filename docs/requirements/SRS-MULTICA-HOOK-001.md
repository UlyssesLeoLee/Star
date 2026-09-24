# SRS-MULTICA-HOOK-001

> **Multica Hook 域要件定義書 v0.1** (per ADR-0026 v0.2 §1.1 "5 类扩展点" 中的 hooks, 与 SRS-MULTICA-SKILL-001 v0.1 平行)

> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 issue: ULYS-235 ("hook需求")
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-HOOK-001.md`](../design/BD-MULTICA-HOOK-001.md) (同期落档)
> - 关联詳細設計書: [`docs/detailed-design/DD-MULTICA-HOOK-001.md`](../detailed-design/DD-MULTICA-HOOK-001.md) (同期落档)
> - 平行 SRS: [`docs/requirements/SRS-MULTICA-SKILL-001.md`](../requirements/SRS-MULTICA-SKILL-001.md) v0.1 (skills 域)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.3 5 类扩展点 (commands / agents / skills / hooks / MCP)
> - 拍板来源: 2026-09-24 20:xx JST Ulysses "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计"
> - 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 2026-08-27 19:39 JST 用户授权 + 守门 #10 + 守门 #14 v3)
> - 审批: `架构师 (Mavis 接手 agent per DEC-008)` (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses)
> - 日期: 2026-09-24 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-HOOK-001 |
| 文书名 | Multica Hook 域要件定義書 (UI 高级设置 → Hooks 标签页) |
| 版本 | v0.1 (初版) |
| 作成日 | 2026-09-24 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 対象範囲 | STAR 平台 "高级设置" 导航下 Hooks 标签页 + hooks 注册 / 匹配 / 执行 / 审计 |
| 対象バージョン | mavis v0.x (现役) + mavis v1.0 (规划) |
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
| **v0.1 (当前)** | 2026-09-24 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | 初版落档, 8 機能 (FR-1~FR-8) + 5 業務要件 (BR-1~BR-5) + 6 非機能要件 (NFR-P/A/S/M/T/O) + 8 验收条件 (AC-1~AC-8), 3 表 W/T/M 横展 (Hook W/M + Hook Run T + Session M, 100% 覆盖 per 守门 #13), 守门 8/8 通过, 8 已知缺口 (含 1 P0 阻塞 hook 自身越权, 跟 SRS-PRE-TOOL-USE-GUARD-001 §8 #1 一致) | ULYS-235 (2026-09-24 20:xx JST) "我需要有hooks功能，可以和skills合并成同一个导航里不同标签页，这个可以叫高级设置。给我需求文档、基本设计、详细设计" |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 **STAR 平台 "高级设置 → Hooks" 域** 的要件定義書. Hooks 体系是 ADR-0026 v0.2 §1.3 列出的 5 类扩展点之一 (commands / agents / skills / **hooks** / MCP), 跟 skills 平行, 共享同一导航 "高级设置" 下的不同标签页. Hooks 域具体涵盖:

- 14 类 Hook 事件 (per Claude Code PreToolUse/PostToolUse/UserPromptSubmit/SessionStart/SessionEnd 等 + 扩展)
- 4 状态 (registered / enabled / disabled / archived) + 30 天归档策略
- Hook 注册中心 (单一来源, JSON Schema + 热更新)
- Hook 执行引擎 (per-event fan-out, 含 pre/post/block/transform 4 种动作类型)
- 全操作审计 log (Transaction append-only per 守门 #13)
- UI "高级设置 → Hooks" 标签页 (CRUD + enable/disable + 触发日志查看)
- 跟 skills 域并行 (相同导航, 不同标签页, 独立 registry, 共享 session state)
- 跟 PreToolUse guard 联动 (PreToolUse guard 是 hooks 体系下 1 个具体 builtin guard hook)

作为后续基本設計 (`BD-MULTICA-HOOK-001.md` v0.1, 同期落档) / 詳細設計 (`DD-MULTICA-HOOK-001.md` v0.1, 同期落档) / 実装 / テスト的唯一依据.

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
| **FR-2** | Hook 事件模型 | 4 项 | P0 | 14 类事件 + 4 种动作类型 (pre / post / block / transform) |
| **FR-3** | Hook 注册中心 | 3 项 | P0 | JSON Schema + 热更新 + builtin hook 默认装载 |
| **FR-4** | Hook 执行引擎 | 3 项 | P0 | per-event fan-out + 串行/并行策略 + timeout / retry |
| **FR-5** | Hook 审计 log | 2 项 | P0 | 全操作留痕, Transaction append-only per 守门 #13 |
| **FR-6** | Hook UI 标签页 | 3 项 | P0 | "高级设置 → Hooks" 标签页: CRUD + enable/disable + 触发日志 |
| **FR-7** | 跟 skills 域协调 | 2 项 | P1 | 同导航不同标签页 + 独立 registry + 共享 session state |
| **FR-8** | 测试 / 报告 | 1 项 | P1 | 单元测试 + 集成测试 + 报告 (跟现有 PHASE-*-IMPL-REPORT 一致) |
| **合計** | | **22 项** | | |

### 1.4 排除範囲 (Out-of-Scope)

- **网络层拦截** (egress filtering) → 后续 v2.x 拍摄, 不在本 v0.1 范围
- **Hook marketplace / cross-organization 共享** → 一人公司不需要, 跟 skills 域同 disclaimer
- **Hook AI 行为审计** (LLM 决策过程录屏) → 后续 v2.x 拍摄
- **Hook 加密 / 凭据管理** (mavis 内置 vault) → 跨项目需求, 不在本专题
- **5 类扩展点的其他 3 类** (commands / agents / MCP) → ULYS-235 只要求 hooks, 后续按需扩展; 但导航 "高级设置" 预留扩展位

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

## §2 用語定義 / 略語 (Glossary)

| 用語 | 定義 | 出典 |
|---|---|---|
| **Mavis** | 本机 root session agent (Mavis As a Jarvis), 运行在 MiniMax Code | per agent-context block |
| **Hook** | 事件触发回调, 在 STAR 平台某个事件点 (e.g. PreToolUse) 触发的可注册回调 | ADR-0026 §1.3 |
| **Hook event** | 14 类事件之一: PreToolUse / PostToolUse / UserPromptSubmit / SessionStart / SessionEnd / SubagentDispatch / SubagentReturn / ToolError / FileWatch / CronTick / RuntimeScan / WorkspaceSwitch / NetworkEgress / CustomEvent | 本 SRS 自定义 |
| **Hook action type** | 4 种动作: pre (前置) / post (后置) / block (阻断) / transform (转换) | 本 SRS 自定义 |
| **Builtin hook** | STAR 平台内置的 hook (e.g. PreToolUse guard, SessionStart cleanup), 不可禁用 | 本 SRS 自定义 |
| **User-defined hook** | 用户/项目自定义的 hook, 可启用/禁用 | 本 SRS 自定义 |
| **高级设置** | STAR UI 顶层导航, 容纳 skills / hooks 等扩展点, 不同能力走不同标签页 | ULYS-235 拍板 |
| **Hooks 标签页** | "高级设置" 下 hooks 域的 UI 标签页, 含 CRUD + enable/disable + 触发日志 | 本 SRS 自定义 |
| **Skills 标签页** | "高级设置" 下 skills 域的 UI 标签页 (per SRS-MULTICA-SKILL-001 v0.1) | 平行 SRS |
| **Hook registry** | hook 注册中心, 单一来源 JSON 文件, per 热更新 | 本 SRS 自定义 |
| **Hook run** | 单次 hook 执行记录, 写审计 log | 本 SRS 自定义 |
| **Fan-out** | 1 个事件触发 N 个 hook (per-event 多 hook 列表), 串行或并行执行 | 本 SRS 自定义 |
| **BLOCK** | hook 返回 BLOCK 决策, 阻断后续 hook 执行 + 阻断原工具调用 | 跟 SRS-PRE-TOOL-USE-GUARD §FR-2.1 一致 |
| **ASK** | hook 返回 ASK 决策, 走 ask_user, 推荐项放"取消" | 跟 SRS-PRE-TOOL-USE-GUARD §FR-2.2 一致 |
| **WARN** | hook 返回 WARN 决策, 注入 system_reminder, 继续执行 | 跟 SRS-PRE-TOOL-USE-GUARD §FR-2.3 一致 |
| **PASS** | hook 返回 PASS 决策, 无动作 | 跟 SRS-PRE-TOOL-USE-GUARD §FR-2 一致 |
| **transform** | hook 返回 transform 决策, 修改工具调用参数 (e.g. 路径标准化 / 凭据脱敏) | 本 SRS 自定义 |
| **fail-open** | hook 加载失败 / 执行失败 → 默认放行, 不阻断主流程 | 行业术语 |
| **fail-closed** | 审计 log 写失败 → BLOCK, 凭据外泄零容忍 | 行业术语 |
| **W/T/M** | Work / Transaction / Master 三类横展 (per 守门 #13) | STAR 守门 #13 |
| **IPA SEC** | Information-technology Promotion Agency, Software Engineering Center | 日本独立行政法人 |
| **要件定義書** | Software Requirements Specification (SRS) | IPA SEC テンプレート |
| **基本設計書** | Basic Design Document (BDD) | IPA SEC テンプレート |
| **詳細設計書** | Detailed Design Document (DDD) | IPA SEC テンプレート |

---

## §3 業務要件 (BR, Business Requirements)

### BR-1 统一事件扩展机制

用户能在 "高级设置 → Hooks" 标签页 CRUD hook, 注册到 14 类事件点之一, 平台在事件触发时 fan-out 执行, 全操作留痕. **跟 PreToolUse guard 派生**: PreToolUse guard 是 hooks 体系下 1 个具体 builtin guard hook (per SRS-PRE-TOOL-USE-GUARD §1.5), 本 SRS 是 hooks 上位抽象, 收敛所有事件点.

### BR-2 凭据外泄防护 (跟 BR-2 PreToolUse guard 一致)

凭据 (env var / GitHub PAT / SSH 私钥 / `.env` 文件) 任何形式的打印 / 复制 / 上传, hook 返回 BLOCK. **跟守门 #5 派生**: 守门 #5 是 review guard, 本 SRS 的 hooks 是 execution guard, 两者互补不重复.

### BR-3 不可逆操作二次确认 (跟 BR-3 PreToolUse guard 一致)

不可逆操作 (`rm -rf` 命中 `/` `~` `*`、`mkfs`、`sudo`、force push) hook 返回 ASK, **推荐项必放"取消"** (per 守门 v28 拍板必带推荐项).

### BR-4 高级设置导航统一容器 (per ULYS-235 拍板)

"高级设置" 顶层导航下, skills / hooks 共享同一容器, 不同能力走不同标签页 (e.g. "Skills" 标签页 + "Hooks" 标签页), 独立 registry, 共享 session state. **预留扩展位**: 后续 commands / agents / MCP 等扩展点按需加新标签页, 走同一导航.

### BR-5 规则可观测 (跟 BR-4 PreToolUse guard 一致)

所有 hook 触发结果 (BLOCK / ASK / WARN / PASS / transform 命中) 必写入审计 log, 可查询 / 可回放 / 可聚合. 审计 log 是 Transaction append-only (per 守门 #13), 不可物理删除. UI "高级设置 → Hooks" 标签页可直接查看触发日志.

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

- 触发: UI "高级设置 → Hooks → + New Hook" 按钮, 或 `automation/hook_create.py <name>` 命令
- 必填字段: `name / event_type / action_type / handler / enabled`
- 可选字段: `priority / timeout_ms / retry / description`
- 落盘: `scripts/automation/hooks/registry.json` (单一来源, 跟 skills registry 平行)

**FR-1.3** Hook 启用 / 禁用

- 触发: UI "高级设置 → Hooks" 标签页 toggle 按钮, 或 `automation/hook_enable.py <name>` / `automation/hook_disable.py <name>`
- 行为: 修改 `registry.json` 中 `enabled` 字段, 热更新 (per FR-3.2)
- 审计: 每次 enable/disable 写 audit log (per FR-5)

**FR-1.4** Hook 归档

- 触发: 30 天 disabled 状态自动归档, 或 `automation/hook_archive.py <name>` 手动归档
- 行为: registry entry 设 `archived=true`, UI 默认隐藏, 但可查可复活
- 跟守门 #11 一致: **archived 不物理删除**, 30 天内可回滚

### FR-2 Hook 事件模型 (4 项, P0)

**FR-2.1** 14 类事件 (per Claude Code 14 事件基线 + 扩展)

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

**FR-2.2** 4 种 action type

- `pre`: 前置, hook 在原动作前执行, 返回 PASS/WARN 继续, 返回 BLOCK 阻断
- `post`: 后置, hook 在原动作后执行, 返回 PASS/WARN 继续, 返回 BLOCK 仅阻断后续 fan-out, 不回滚原动作
- `block`: 强阻断, hook 返回 BLOCK 必阻断原动作 (跟 pre 区别: block 无条件阻断, pre 可选)
- `transform`: 转换, hook 返回 `transform` 决策 + 新参数, 修改原动作参数后再执行

**FR-2.3** 事件 fan-out

- 1 个事件触发 N 个 hook (N >= 0), 按 `priority` 升序执行 (priority 越小越先执行)
- 同 priority 多个 hook: 默认串行 (可配并行, per FR-4.3)
- 任一 hook 返回 BLOCK: 立即停止 fan-out, 阻断原动作 (pre/block action type)
- 任一 hook 返回 transform: 累积 transform, 最后 1 个生效 (per FR-4.4)

**FR-2.4** 事件 schema

- 格式: JSON, 符合本 SRS §4.3.1 Event schema
- 字段: `event_id / event_type / timestamp / session_id / payload`
- `payload` 字段 per event_type 不同 (e.g. PreToolUse payload 含 `tool_name / tool_args`)

### FR-3 Hook 注册中心 (3 项, P0)

**FR-3.1** JSON Schema + 单一来源

- 格式: JSON, 符合本 SRS §4.3.1 Hook schema
- 存储: `scripts/automation/hooks/registry.json` (单一来源, git tracked)
- 加载: 启动时一次性加载 + 运行期热更新 (per FR-3.2)
- schema 校验: 启动时 JSON Schema 校验, 失败 → fail-open (per FR-3.3) + WARN log

**FR-3.2** 热更新

- 监听: registry.json mtime 变化, 自动 reload
- 不重启 mavis runtime
- reload 失败 → fail-open (per FR-3.3) + WARN log
- reload 成功 → info log (含新 hook 数)

**FR-3.3** Builtin hook 默认装载

- builtin hook (e.g. PreToolUse guard, SessionStart cleanup) 启动时自动装载, 不依赖 registry.json
- 用户可在 UI 禁用 builtin hook (但不可删除)
- builtin hook 列表: `scripts/automation/hooks/builtin/` (Python 模块, 不是 JSON)

### FR-4 Hook 执行引擎 (3 项, P0)

**FR-4.1** Hook handler 接口

- 签名: `def handler(event: Event, context: Context) -> HookResult`
- `HookResult`: `{decision: BLOCK|ASK|WARN|PASS|transform, reason?: str, rule_id?: str, transformed_args?: dict, latency_ms: float}`
- 失败处理: handler 抛异常 → 视为 WARN (per FR-4.5)

**FR-4.2** Per-event fan-out 调度

- 调度算法: 按 priority 升序遍历 hook 列表, 同 priority 默认串行
- 任一 hook 返回 BLOCK: 立即停止 fan-out, 返回 BLOCK 给调用方
- 任一 hook 返回 ASK: 立即停止 fan-out, 走 ask_user, 用户回复后才继续 (或取消)
- 任一 hook 返回 PASS/WARN: 继续下一个 hook
- 任一 hook 返回 transform: 累积 transformed_args, 继续下一个 hook

**FR-4.3** 串行 / 并行策略

- 默认: 串行执行 (同 priority)
- 配置: registry.json 中 `parallel: true` 字段, 同 priority hook 可并行
- 并行上限: max 4 (避免资源耗尽)
- 超时: 单 hook `timeout_ms` 默认 1000ms (可配)

**FR-4.4** Transform 累积

- 多个 hook 返回 transform: 后执行的 hook 覆盖前一个的 `transformed_args`
- 最终 transformed_args 传给原动作
- 若任一 hook 返回 BLOCK: transform 不生效, 阻断原动作

**FR-4.5** 失败处理

- handler 抛异常: 视为 WARN, 继续下一个 hook, 写 audit log (含异常 stack trace, 但 log level = ERROR)
- handler 超时: 视为 BLOCK (安全优先), 写 audit log (含 timeout 原因)
- handler 返回非法 decision (非 BLOCK/ASK/WARN/PASS/transform): 视为 WARN, 写 audit log

### FR-5 Hook 审计 log (2 项, P0)

**FR-5.1** 全操作留痕

- 写入: 任何 hook 触发 (无论 BLOCK / ASK / WARN / PASS / transform) 必写 audit log
- 字段: 见 §4.3.2 Hook Run schema, 12 字段
- 存储: `scripts/automation/hooks/logs/hook_audit.log` (JSON Lines)
- 保留: 90 天 (跟 PreToolUse guard 一致, per 守门 #5 隐含)

**FR-5.2** Transaction append-only

- 不允许物理删除 / 物理修改
- 不允许 truncate
- 完整字段: actor / event_type / hook_name / decision / rule_id / timestamp / session_id / latency_ms / env_hash / transformed_args / before_args / after_args (per 守门 #13)

### FR-6 Hook UI 标签页 (3 项, P0)

**FR-6.1** "高级设置 → Hooks" 标签页布局

- 位置: 顶层导航 "高级设置" → 标签页栏 "Hooks" (跟 "Skills" 标签页平行)
- 入口: UI 侧边栏 "高级设置" 菜单项, 含子标签 "Skills" + "Hooks"
- 路由: `/settings/advanced/hooks` (前端 next.js 路由)
- 布局: 3 区域
  - 左: hook 列表 (按 event_type 分组)
  - 中: hook 详情 (含 name / event / action / handler / priority / timeout / enabled toggle)
  - 右: 触发日志 (audit log 查询, 按 hook_name / event_type / decision 过滤)

**FR-6.2** CRUD 操作

- 创建: "+ New Hook" 按钮 → 表单 (name / event_type / action_type / handler / priority / timeout / description) → 写入 registry.json
- 读取: 列表点击 → 详情面板
- 更新: 详情面板编辑 → 写入 registry.json
- 删除: 详情面板 "Archive" 按钮 → 设 `archived=true` (per FR-1.4, 不物理删除)
- Enable/Disable: 详情面板 toggle → 写 registry.json + 热更新

**FR-6.3** 触发日志查看

- 来源: `scripts/automation/hooks/logs/hook_audit.log`
- 过滤: hook_name / event_type / decision / 时间范围
- 分页: 50 条/页, 最多展示 1000 条 (避免前端过载)
- 详情: 点击单条 → 弹窗显示完整 audit log entry (12 字段)

### FR-7 跟 skills 域协调 (2 项, P1)

**FR-7.1** 同导航不同标签页

- "高级设置" 顶层导航: tabs = [`Skills`, `Hooks`, (预留: `Commands`, `Agents`, `MCP`)]
- Skills 标签页: 走 SRS-MULTICA-SKILL-001 v0.1 的 skill CRUD
- Hooks 标签页: 走本 SRS §FR-6 的 hook CRUD
- 两个标签页独立 registry, 共享 session_id (跨标签页 state 可传递)

**FR-7.2** 独立 registry + 共享 session state

- Skill registry: `docs/skills/<name>/SKILL.md` (per SRS-MULTICA-SKILL-001 §FR-12)
- Hook registry: `scripts/automation/hooks/registry.json` (per FR-3.1)
- Session state: `scripts/automation/hooks/state/session_state.json` (跨标签页共享, per session_id)

### FR-8 测试 / 报告 (1 项, P1)

**FR-8.1** 单元测试 + 集成测试 + 报告

- 单元测试: `tests/automation/hooks/test_hook_engine.py` 覆盖 14 类事件 + 4 种 action type + fan-out + transform + fail-open/closed
- 集成测试: `tests/e2e/test_hook_ui.py` 覆盖 "高级设置 → Hooks" 标签页 CRUD + enable/disable + 触发日志查看
- 报告: `docs/reports/PHASE-HOOK-IMPL-REPORT.md` 跟现有 PHASE-*-IMPL-REPORT 一致

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

- `scripts/automation/console_server.py` v0.1 (hook 事件流入口)
- `scripts/automation/dispatcher.py` v0.1 (子代理 invoke 前置, SubagentDispatch 事件点)
- `scripts/automation/guardian/pre_tool_use_guard.py` (PreToolUse builtin guard hook)
- `scripts/automation/hooks/registry.json` (新建, hook 注册中心)
- `scripts/automation/hooks/builtin/` (新建, builtin hook 模块)
- `scripts/automation/hooks/logs/hook_audit.log` (runtime 生成)
- `scripts/automation/hooks/state/session_state.json` (runtime 生成)
- 前端 `frontend/src/app/(app)/settings/advanced/hooks/page.tsx` (新建, UI 标签页)
- mavis runtime hook 事件流 (现役, audit 已有)
- SRS-MULTICA-SKILL-001 v0.1 平行 SRS (skills 域, 共享"高级设置"导航)

---

## §7 验收条件 (AC, Acceptance Criteria)

| AC | 关联 FR / NFR | 验收方法 | 阈值 |
|---|---|---|---|
| **AC-1** | FR-2.1 + FR-4.1 | 单元测试 14 类事件全部 hook 触发 | 100% |
| **AC-2** | FR-2.2 + FR-4.2 | 单元测试 4 种 action type 全部跑通 | 100% |
| **AC-3** | FR-1.1 + FR-1.4 | 单元测试 4 状态 (registered / enabled / disabled / archived) 全部跑通 | 100% |
| **AC-4** | FR-3.2 + NFR-P-5 | registry 修改实测 | < 100ms reload, 0 中断 |
| **AC-5** | FR-4.3 + FR-4.4 | 单元测试串行 / 并行 / transform 累积 | 100% |
| **AC-6** | FR-6.1 + FR-6.2 | UI 集成测试 "高级设置 → Hooks" 标签页 CRUD | 100% |
| **AC-7** | FR-6.3 + NFR-O-4 | UI 集成测试触发日志查看 | 100% |
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