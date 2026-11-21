# F18 MCP、Skills、Hooks 与 PreToolUse 守卫

> 状态摘要: Hooks 的策略 REST 接口、Rust evaluator 与 Worktree 归档门禁已在代码中实现，Run admission 生产 producer 与工具调用阶段未接入；Python PreToolUse 守卫在 console 与 dispatcher 中生效，但 ASK/WARN 无动作，MCP、Skills 与 Plugins 面板仍以 localStorage seed 运行，无 REST 接口与 Skill 运行时。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

在"高级设置"统一容器下，为租户与项目提供四类扩展能力的配置入口，并为安全关键操作提供同步决策：

- **Hooks**：以无代码 typed 规则编辑 Project 基线与 Worktree 收紧覆盖，经发布形成不可变版本，由 Rust evaluator 在 Worktree 归档/清理与 Run admission 前给出 `allow / deny / require_human / defer`。
- **PreToolUse 守卫**：在工具调用与子代理派发前按危险模式库决策，阻断凭据外泄、不可逆破坏性命令。
- **MCP 与 Skills**：登记 MCP 服务器与可复用 Skill，供后续 Agent 运行时消费。
- **统一导航**：Skills、Hooks、MCP、Plugins 为同一父入口下的并列标签，Hooks 不成为 Worktree 树节点。

### 1.2 范围内 / 范围外

| 范围 | 内容 |
|---|---|
| 范围内 | `/settings/advanced` 标签壳与 Hooks、Skills、MCP 三个标签页；Hook policy REST 接口与 Rust evaluator；Worktree 归档门禁；Run admission producer 接缝；Python PreToolUse 守卫与其在 console/dispatcher 的接入；Python Hook 引擎（非权威兼容代码）；MCP 与 Skill 的登记 UI；`/settings` 页面中与之重叠的 mcp、skills、hooks 三个 tab |
| 范围外 | Plugins 标签（路由 `/settings/advanced/plugins` 分派到 `PluginsSettingsPanel`，见 `frontend/src/app/(app)/settings/advanced/[tab]/page.tsx:40`，由 Plugins 自身特性文档负责）；Commands 与 Agents 标签（仅为预留位）；网络层出站拦截（egress filtering）；Hook/Skill 市场与跨组织共享；Hook 凭据库；LLM 决策审计 |

## 2. 用户与场景

| 角色 | 场景 |
|---|---|
| 租户管理员、项目管理员 | 查看并编辑 Project 基线规则，发布新版本，回滚到历史版本（服务端强制 admin 角色） |
| Worktree 管理者 | 编辑 Worktree 覆盖规则（只能增加限制），查看生效版本与阻断摘要 |
| 项目成员（只读） | 查看有效策略与 Hook 执行事件，不能保存或发布 |
| 安全与运维人员 | 查看 Hook 执行事件与 summary，了解覆盖范围（partial/unknown） |
| 开发者与 Agent 运维 | 登记 MCP 服务器、登记 Skill、查看 PreToolUse 守卫的规则与审计 |

## 3. 需求

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-18-01 | 设置页中"高级设置"为父入口，内容区含 Skills、Hooks、MCP、Plugins 四个并列标签 | P0 | ✅ | `frontend/src/app/(app)/settings/advanced/layout.tsx:37-42`（标签数组）、`:55-74`（标签导航） |
| FR-18-02 | 访问 `/settings/advanced` 默认进入 `/settings/advanced/hooks` | P0 | ✅ | `frontend/src/app/(app)/settings/advanced/page.tsx:12`；`[tab]/page.tsx:37` |
| FR-18-03 | 按 Project 读取基线策略，按 Worktree 读取有效策略 | P0 | ✅ | `crates/star-api-rest/src/group_api/hook_policies.rs:301-343`（路由）；`frontend/src/lib/group/worktreeGroupApi.ts:880,898` |
| FR-18-04 | Draft 保存带 CAS，发布与回滚创建新版本，发布仅限 tenant_admin/project_admin | P0 | ✅ | `hook_policies.rs:323-339`（PUT draft、POST publish/rollback）；`:1072`（version conflict）；`:863-866`（publish 角色门禁）；`HookPolicyPage.tsx:486-519, 616-617, 635` |
| FR-18-05 | 无宿主认证时不显示 seed 策略、不发送读写请求 | P0 | ✅ | `frontend/src/app/providers.tsx:50-59`（仅在 run-directory session 存在时挂载 provider）；`frontend/src/lib/group/groupProjection.tsx:104-106`；`HookPolicyPage.tsx:532-536`；测试 `HookPolicyPage.test.tsx:83` |
| FR-18-06 | 规则编辑器以结构化表单编辑决策、优先级、启用状态与有限条件，builtin 基线只读 | P0 | 🟡 | 编辑器 `HookPolicyPage.tsx:839-884`；决策选项常量 `:128`，阶段选项 `:135`；条件字段集合 `FACT_FIELDS` 存在（`:881`）；决策集合是否仅含 Deny/RequireHuman/Defer 未逐项核对（见 §7） |
| FR-18-07 | Run admission 阶段仅在服务端 producer 能力为真时可选 | P0 | 🟡 | UI 禁用逻辑 `HookPolicyPage.tsx:611, 854`；能力回传 `hook_policies.rs:900-903`；默认值 `crates/star-api-rest/src/group_api/cli_sessions.rs:256`（`supports_run_admission` 默认 false，仓内无返回 true 的实现） |
| FR-18-08 | Hook 执行事件只读列表、summary 与显式 partial coverage | P0 | 🟡 | 事件与 summary 路由 `hook_policies.rs:305-313`；覆盖声明 `:413-423, 603-624`；面板 `HookPolicyPage.tsx:741-817`，覆盖文案 `:775-794`。当前仅 archive 阶段接入 |
| FR-18-09 | Rust evaluator 为纯函数式决策，输出 allow/deny/require_human/defer，策略无效或超界时 fail-closed | P0 | ✅ | `crates/domain-hook/src/lib.rs:96-116`（HookPhase、HookDecision）；`:80-92`（版本与上界常量）；`:373, 418`（decode/verify）；`:548`（`evaluate`）；测试 `:1075-1282` |
| FR-18-10 | Builtin 安全基线总是装载，用户策略不能禁用、覆盖或放宽 | P0 | 🟡 | 基线原因码 `lib.rs:120`；`worktree_allow_rule_is_rejected_before_evaluation`（`:1148`）；归档路径生效（`worktrees.rs:1187-1202`）。工具调用、validation、review 的 builtin 规则不存在（见 FR-18-13） |
| FR-18-11 | Worktree 归档与物理清理在 Domain Command 事务内调用 evaluator，结果写入审计投影 | P0 | ✅ | `crates/star-api-rest/src/group_api/worktrees.rs:626-707`（归档 gate 进入事务）、`:686`（`evaluate_archive_gate`）、`:1187-1202`（`evaluate_hook` 调用）、`:1311`（新鲜度检查）；测试 `:1634, 1656, 1667, 1698` |
| FR-18-12 | Run admission 生产 producer：readiness 在事务外取得，事务内重授权并求值，Allow 原子写 Run、HookSet 快照与 Hook 投影，Deny 只写 ledger | P0 | 🟡 | `cli_sessions.rs:254-256`（trait 与能力方法）、`:556`（能力门禁）、`:754`（`evaluate_hook`）、`:1442-1444`（`hook_evaluated` 投影写入）。无生产 adapter，不会在运行环境中启用 |
| FR-18-13 | 工具调用前后、validation 前后、review/complete 为同步 Hook 阶段 | P0 | ⬜ | `lib.rs:96-103` 的 `HookPhase` 仅两个变体（BeforeRunAdmission、BeforeWorktreeArchiveCleanup） |
| FR-18-14 | PreToolUse 守卫规则库：15 条规则（BLOCK 8、ASK 5、WARN 2） | P0 | 🟡 | `scripts/automation/guardian/rules/pre_tool_use_rules.json`（按 level 计数：BLOCK 8、ASK 5、WARN 2）；决策枚举 `scripts/automation/guardian/pre_tool_use_guard.py:22-27` |
| FR-18-15 | 对 bash、write、edit 工具调用按规则扫描（FR-1.1） | P0 | 🟡 | 扫描字段提取 `pre_tool_use_guard.py:62-82`（bash 67-68；write/edit 69-75；read 76-77 不扫描）。运行时调用方只传 `run_script`（`console_server.py:263-284`）与 `mcp_dispatch`（`dispatcher.py:238-273`），bash/write/edit 路径无调用方 |
| FR-18-16 | ASK 决策走 ask_user；WARN 注入 system_reminder（守卫 FR-2.2、FR-2.3） | P0 | ⬜ | `console_server.py:278-279` 与 `dispatcher.py:265-266` 仅为注释，ASK/WARN 无动作分支 |
| FR-18-17 | 规则加载失败 fail-open；审计写失败 fail-closed（守卫 FR-6） | P0 | 🟡 | 规则异常转 PASS：`pre_tool_use_guard.py:160-163`；审计失败转 BLOCK：`:185-188`。调用方对守卫自身异常同样 fail-open（`console_server.py:282-284`、`dispatcher.py:271-273`），与 Hook SRS 的 critical fail-closed 不一致 |
| FR-18-18 | 每次守卫决策写入审计（守卫 FR-4） | P0 | 🟡 | `scripts/automation/guardian/audit_logger.py:54`（`AuditLogger`）、`:74`（`log`）、`:91`（`query`）；append-only 的权限层保证未核实 |
| FR-18-19 | 规则文件变更自动热更新 | P1 | 🟡 | `scripts/automation/guardian/rule_database.py:146-177`（可选 watcher）、`:186`（`reload`）；dispatcher 侧 `enable_watcher=False`（`dispatcher.py:249`） |
| FR-18-20 | Hook 注册中心：JSON 注册表与内置 Hook 装载（Python 兼容侧） | P1 | 🟡 | `scripts/automation/hooks/registry.json` 仅含 3 条用户示例（无内置条目）；`scripts/automation/hooks/builtin_hook_loader.py`；仅 `tests/automation/hooks/*` 导入，运行时无调用方 |
| FR-18-21 | Hook 事件 14 类词汇（兼容输入） | P2 | ❓ | 词汇声明在 `scripts/automation/hooks/event_emitter.py`（模块 docstring 见 `scripts/automation/hooks/__init__.py:1-30`），事件枚举未逐行核对 |
| FR-18-22 | MCP 服务器登记：stdio、sse、http 连接类型，user/project 作用域，启停，删除 | P0 | 🟡 | `frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx:38-76`（表单与 `addMcpServer`）；数据层 `frontend/src/lib/settings/capabilityRegistry.ts:192-270`，持久化键 `:76`，seed `:78-106` |
| FR-18-23 | MCP 服务器 REST CRUD 与异步健康检查 | P0 | ⬜ | 设计 `docs/api-design-mcp-capabilities.md` §2（:29-53）；仓内无 `/api/v1/capabilities` 路由；`crates/domain-mcp/src/lib.rs:29-91` 的 `McpServer` 与 `McpServerRegistry` 无 crate 外调用方 |
| FR-18-24 | MCP 健康检查（客户端探测） | P1 | 🟡 | `frontend/src/components/settings/McpSettingsPanel.tsx:204`（`fetch` HEAD）、`:222-229`（错误归类）；仅在浏览器内执行，服务端无探测 |
| FR-18-25 | Skill 面板：列表、添加、启停、删除 | P0 | 🟡 | `frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx:31-48`；数据 `capabilityRegistry.ts`（seed `:107-124`） |
| FR-18-26 | Skill 生命周期 draft/active/deprecated/archived 与 create/refresh/deprecate/archive | P0 | ⬜ | 仓内无 Skill 生命周期代码；`docs/skills/` 在基线中不存在 |
| FR-18-27 | SKILL.md 五字段模板与 local/server/workspace 三级作用域 | P0 | ⬜ | UI 作用域仅 `user \| project`（`capabilityRegistry.ts:32`）；无 SKILL.md 读写代码 |
| FR-18-28 | Skill 按 (title, description) 关键词匹配与自动加载进入 Agent 上下文 | P0 | ⬜ | 仓内无 matcher 或 loader 代码（对 `SkillMatcher`、`skill_match`、`SKILL.md` 的 grep 无结果） |
| FR-18-29 | 独立 Skill CLI 注册表（MVP，in-memory） | P2 | 🟡 | `crates/star-cli/src/skill_registry.rs:122`（`SkillRegistry`）、`:128`（`new`）、`:195`（`list`）、`:202`（`list_for_agent`）、`:210`（`remove`）；挂载 `crates/star-cli/src/main.rs:19, 70, 76`。与 SRS 的 status/version/scope 模型不一致 |
| FR-18-30 | 唯一 Hook 配置入口位于高级设置 Hooks 标签 | P0 | 🟡 | 高级设置入口存在（FR-18-01）；但 `/settings` 另有 Hooks 配置面板，写入 `localStorage` 键 `star-hooks` 的 `command` 字段（`frontend/src/components/settings/HooksSettingsPanel.tsx:53, 66-76, 152-156`），与 AGENTS.md:22 的约定及 Hook SRS FR-4.1 相冲突 |

## 4. 设计

### 4.1 概念模型

- **HookPolicySet**：每个 Project 有一组版本化规则（Project 基线）；每个 Worktree 可绑定一个 Project 版本并增加限制（overlay）。版本不可变，由 `valid_to` 界定（`db/migrations/2026-09-30-multica-hook-policy.sql:29`）。
- **HookRule**：`phase`、`decision`、`priority`、`enabled`、`conditions`（最多 `MAX_CONDITIONS_PER_RULE=16`），规则数上限 `MAX_RULES_PER_SCOPE=64`（`crates/domain-hook/src/lib.rs:88-90`）。
- **HookEvaluation**：一次求值的结果，含 phase、decision、reason code、rule id 与 evaluator/policy provenance。
- **HookPhase**：当前只有 `BeforeRunAdmission` 与 `BeforeWorktreeArchiveCleanup`（`lib.rs:96-103`）。
- **PreToolUse 守卫**（Python 兼容侧）：`ToolCall`（tool_name、tool_args、session_id、agent_role）经 `PreToolUseGuard.evaluate` 得到 `Decision`（BLOCK、ASK、WARN、PASS），规则为正则匹配的 `Rule`（id、level、pattern、reason）。
- **Skill**（设计）：name、description、when_to_use、steps、tools，状态与作用域见 SRS-MULTICA-SKILL-001。当前仅 CLI 内存注册表（`crates/star-cli/src/skill_registry.rs`）与 UI seed 存在。
- **McpServerConfig**（UI）：name、transport（stdio/sse/http）、command、args、scope（user/project）、enabled、health（`capabilityRegistry.ts:35-49`）。

### 4.2 数据模型

| 对象 | 存储 | 类别 | 证据 |
|---|---|---|---|
| `multica.hook_policy_set` | PostgreSQL；SCD2 触发器 `guard_hook_policy_set_scd2` | Master（版本化） | `db/migrations/2026-09-30-multica-hook-policy.sql:29, 190` |
| `multica.hook_policy_draft` | PostgreSQL；TTL 过期索引 | Work | `2026-09-30-multica-hook-policy.sql:75, 110` |
| `multica.hook_policy_audit_event` | PostgreSQL；触发器禁止改写 `reject_hook_policy_audit_mutation` | Transaction（append-only） | `2026-09-30-multica-hook-policy.sql:121, 245` |
| `multica.hook_execution_event` | PostgreSQL；触发器禁止改写 `reject_hook_execution_event_mutation` | Transaction（append-only） | `db/migrations/2026-10-01-multica-hook-execution-event.sql:20, 69` |
| `task_execution_run_event`（`hook_evaluated` 行） | PostgreSQL；summary 索引 | Transaction | `2026-10-01-multica-task-run-hook-summary-index.sql:11`；`cli_sessions.rs:1442-1444` |
| 规则文件 `pre_tool_use_rules.json` | 文件系统，启动加载，mtime 热更新 | Master（配置） | `scripts/automation/guardian/rules/pre_tool_use_rules.json`；`rule_database.py:94` |
| 守卫审计 `pre_tool_use_audit.log` | JSON Lines 文件 | Transaction（设计要求 append-only） | `audit_logger.py:54, 74`；AuditEvent 字段在 `evaluate` 内构造（`pre_tool_use_guard.py:141-190`） |
| `scripts/automation/hooks/registry.json` | 文件系统 | Master（兼容） | 3 条用户示例 |
| UI 注册表 `star-capability-registry:v1`、`star-mcp-servers`、`star-skills`、`star-hooks` | 浏览器 localStorage | 无服务端事实 | `capabilityRegistry.ts:76`；`components/settings/McpSettingsPanel.tsx:70`；`SkillsSettingsPanel.tsx:70`；`HooksSettingsPanel.tsx:53` |

BD §7.7 描述 Run 写入 `task_execution_run.hook_set_snapshot`，代码中的写入位置未逐行核对（❓）。Skill 与 MCP 没有服务端表。

### 4.3 接口（routes、APIs、ports）

**REST（Group API，`group_api.rs:827` 挂载 `hook_policies::router()`）**

| 方法 | 路径 | 权限 | 证据 |
|---|---|---|---|
| GET | `/api/v1/projects/{project_id}/hook-policy` | `hook:read` | `hook_policies.rs:303-306` |
| GET | `/api/v1/projects/{project_id}/hook-events` | `hook:read` | `:307-310` |
| GET | `/api/v1/projects/{project_id}/hook-events/summary` | `hook:read` | `:311-314` |
| GET | `/api/v1/worktrees/{worktree_id}/hook-policy/effective` | `hook:read` | `:315-318` |
| PUT | `/api/v1/projects/{project_id}/hook-policy/draft` | `hook:write` | `:319-322` |
| PUT | `/api/v1/worktrees/{worktree_id}/hook-policy/draft` | `hook:write` | `:323-326` |
| POST | `/api/v1/projects/{project_id}/hook-policy/publish` | `hook:publish` + admin | `:327-330` |
| POST | `/api/v1/worktrees/{worktree_id}/hook-policy/publish` | `hook:publish` + admin | `:331-334` |
| POST | `/api/v1/projects/{project_id}/hook-policy/rollback` | `hook:publish` + admin | `:335-338` |
| POST | `/api/v1/worktrees/{worktree_id}/hook-policy/rollback` | `hook:publish` + admin | `:339-342` |

Scope 映射在 `hook_policies.rs:851-853`；admin 角色判断在 `:863-866`。

**Rust 接口**

- `domain_hook::evaluate(event, policy) -> HookEvaluation`（`crates/domain-hook/src/lib.rs:548`）。
- `HookPolicyDocument::decode_and_verify(bytes)`（`:373`）、`verify`（`:418`）、`computed_digest`（`:387`）。
- `TaskCliSessionProvisioner::supports_run_admission(&self) -> bool`，默认 false（`cli_sessions.rs:254-256`）。

**Python 接口**

- `PreToolUseGuard.evaluate(tool_call) -> Decision`（`pre_tool_use_guard.py:141`）；`evaluate_for_dispatch`（`:192`）。
- `RuleDatabase.reload()`（`rule_database.py:186`）；`AuditLogger.log`（`audit_logger.py:74`）、`query`（`:91`）。
- `console_server.py` 对 `run_script` 调用守卫（`:263-284`）；`dispatcher.py` 对派发调用守卫（`:238-273`）。

**MCP 与 Skill（设计，未实现）**

`/api/v1/capabilities/mcp-servers` 的 list、create、patch、delete、check（`docs/api-design-mcp-capabilities.md:29-37`）；Skill 的 create、list、refresh、deprecate（SRS-MULTICA-SKILL 设计段，无代码）。

### 4.4 状态机与关键流程

**Hook 策略版本（Project/Worktree）**

`draft（TTL）` → `publish`（写入新 `hook_policy_set` 版本，旧版本 `valid_to` 置位，`hook_policies.rs:1490, 1565`）→ `rollback`（产生新版本，保留审计，`:1609-1692` 的 Worktree 重基线流程）。草稿过期由 `DELETE … WHERE expires_at <= now()` 清理（`:1084`）。版本号不符返回 `hook_policy_version_conflict`（`:1072, 1249, 1348`）。

**Worktree 归档门禁**

`management_plan_requires_archive_gate`（`worktrees.rs:203`）判定后，`prepare_archive_gate_facts`（`:1073`）读取事实，`evaluate_archive_gate`（`:1187`）加载 verified 有效快照并调用 `evaluate_hook`（`:1202`）。结果为 `Defer` 时写入 `worktree_archive_gate_deferred`（`:742`）。事实过期时拒绝（`:812`，`archive_gate_facts_are_fresh` `:1311`）。

**Run admission（设计，producer 已写、adapter 未装配）**

`cli_sessions.rs:556` 检查 `supports_run_admission()`，为假则拒绝；为真则在事务内重授权、读取 effective 快照、`evaluate_hook`（`:754`），Allow 时写 Run、HookSet 快照与 `hook_evaluated` 投影（`:1442-1444`）。

**PreToolUse 守卫（运行时）**

`console_server.py:263-284`：脚本运行前调用 `evaluate`；BLOCK 抛 403（`:273-277`），其余决策放行。`dispatcher.py:238-273`：派发前以 brief 内容调用 `evaluate`；BLOCK 抛 `DispatchBlockedError`（约 `:261`）。守卫自身异常时放行（fail-open）。

### 4.5 权限与安全

- **Hook 管理**：读需 `hook:read`，写需 `hook:write`，发布和回滚需 `hook:publish` 且为 tenant_admin 或 project_admin（`hook_policies.rs:851-866`）。前端按 `role` 禁用发布（`HookPolicyPage.tsx:395, 617`），但服务端为权威。
- **Hook 事件不含敏感正文**：投影字段只含 scope、decision、reason class、duration、version 与 digest（SRS FR-5.1 与 BD §7.6）。仓内 `hook_policies.rs` 的 summary 说明与 SRS 一致（`:626-632`）。
- **Rust evaluator 无外部回调**：规则为 typed 条件，无 shell、脚本或网络（SRS FR-4.1）。
- **Python 守卫**：纯正则匹配，无 LLM 推理；环境变量只记录键名哈希（`pre_tool_use_guard.py:128-131`）。已知风险：审计事件写入 `args_excerpt`，即扫描文本的前 200 字符（`evaluate` 内 AuditEvent 构造，`pre_tool_use_guard.py:141-190`），可能包含粘贴在命令中的凭据文本，与 SRS NFR-S-1 的"0 外泄"目标相冲突（见 §6）。
- **UI 认证**：页面仅在 `WorktreeGroupApiProvider` 存在时可用，provider 由 run-directory session 挂载（`providers.tsx:50-59`）；会话过期清空策略 UI（测试 `HookPolicyPage.test.tsx:92`）。
- **MCP 与 Skill**：UI 为本地 localStorage，不做服务端鉴权；stdio 传输不可测试连接（`components/settings/McpSettingsPanel.tsx:188`）。

### 4.6 依赖关系

| 依赖方 | 被依赖方 | 证据 |
|---|---|---|
| `star-api-rest` Group API | `domain-hook` | `hook_policies.rs:140`；`worktrees.rs:60`；`cli_sessions.rs:85` |
| `run-owner-runtime` 调度策略 | `domain-hook` | `crates/run-owner-runtime/src/schedule_policy.rs:32, 42, 156, 185`（`VerifiedHookPolicySnapshot`） |
| 前端高级设置页 | Group API 客户端 | `HookPolicyPage.tsx:147`；`groupProjection.tsx:104-106`；`worktreeGroupApi.ts:880-937` |
| 应用根 Providers | Group API provider | `providers.tsx:50-59` |
| 运行工作区 | Hooks 高级设置深链 | `frontend/src/components/run/RunWorkspaceShell.tsx:231`（`/settings/advanced/hooks?…`） |
| 导航注册表 | 高级设置入口 | `frontend/src/lib/nav/registry.ts:244-247`（`advanced-settings` 指向 `/settings/advanced/hooks`） |
| console_server / dispatcher | `guardian.pre_tool_use_guard` | `console_server.py:41-58`；`dispatcher.py:239-249` |
| 守卫规则与审计 | 文件系统 | `guardian/paths.py`（路径常量）、`rule_database.py`、`audit_logger.py` |
| Python Hook 引擎 | 无运行时调用方 | `tests/automation/hooks/*` 为唯一导入方 |
| `domain-mcp` | 无调用方 | `crates/domain-mcp/src/lib.rs` |

### 4.7 页面设计

### 4.7.1 /settings/advanced

- **文件**：`frontend/src/app/(app)/settings/advanced/page.tsx`
- **布局**：应用外壳由 `frontend/src/app/(app)/layout.tsx:20, 49`（`AppShell`）提供；本页不渲染内容，直接重定向。
- **组件树**：`AdvancedSettingsIndexPage`（`page.tsx:11`）→ `redirect("/settings/advanced/hooks")`（`:12`）。
- **状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 访问父路径 | 重定向到 Hooks | ✅ `page.tsx:12` |
| 其他 | — | 不适用 | — |

- **交互**：无。
- **数据**：无。
- **与文档设计的差距**：无（FR-18-01、FR-18-02 已覆盖）。

### 4.7.2 /settings/advanced/hooks

- **文件**：`frontend/src/app/(app)/settings/advanced/[tab]/page.tsx:37`（分派到 `HookPolicyPage`）；页面实体 `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx`。
- **布局**：应用外壳 `(app)/layout.tsx:49`；高级设置区块与标签导航 `advanced/layout.tsx:44-75`（区块 `data-testid="advanced-settings"`）；页内主区为 `HookPolicyPage` 根容器（`HookPolicyPage.tsx:544`）。右侧栏与底栏由 `AppShell` 决定，`components/AppShell` 未核对（❓）。
- **组件树**：`AdvancedSettingsTabPage`（`[tab]/page.tsx:33`）→ `HookPolicyPage`（`HookPolicyPage.tsx:146`）→ 范围与对象选择（`:549-559`）、未保存草稿条（`:575`）、规则面板与编辑器（`:595-617`）→ `HookRuleEditor`（`:839`）→ `HookConditionEditor`（`:868`）；发布/回滚历史（`:635-637`）；Hook 执行事件面板 `hook-execution-events`（`:741`）→ summary（`:750-783`）与事件列表（`:803-817`）。
- **状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 已加载策略与草稿 | 编辑器与操作按钮可用（`canEdit` `:396`） | ✅ |
| 加载 | `loading` 为真 | "正在加载服务端策略…"（`:581`） | ✅ |
| 空（无 Project） | 无可读 Project | "当前账号没有可读取的 Project，或授权目录尚不可用。"（`:582`） | ✅ |
| 空（无 Worktree） | 所选 Project 无 Worktree | 说明文字（`:583`） | ✅ |
| 空（无基线） | 策略存在但无 document | "Project Hook 基线尚未 provision"（`:584`） | ✅ |
| 空（无审计） | 无策略变更记录 | "暂无策略变更记录。"（`:637`） | ✅ |
| 错误 | 请求失败 | 页面存在 busy 与错误处理，具体文案未逐行核对 | ❓ |
| 未授权 | 无 `WorktreeGroupApiProvider` | 琥珀色提示 `hook-auth-required`（`:532-536`），不展示 seed，不发写请求（测试 `HookPolicyPage.test.tsx:83`） | ✅ |
| 会话失效 | 会话过期 | 清空策略 UI（测试 `:92`） | ✅ |
| 禁用（发布） | 非 admin、草稿过期、有未保存修改、忙碌 | 按钮禁用，提示"先保存或放弃本地修改"（`:617`） | ✅ |
| 禁用（Run admission） | producer 能力为假 | 阶段选项禁用并标注"（producer 未安装）"（`:854`） | ✅ |
| 覆盖 partial | summary 覆盖为 partial | "部分接入"并列出已接入阶段（`:775-776, 791-794`） | ✅ |

- **交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换范围、Project、Worktree | 无未保存修改，或确认放弃 | 重新加载 | `confirmDiscardUnsavedChanges`（`:549-559`）；测试 `:296` | ✅ |
| 保存草稿 | 可编辑、非过期 | 写入 draft，CAS 版本 | PUT `…/draft`（`worktreeGroupApi.ts:902, 909`） | ✅ |
| 发布版本 | admin、草稿有效、无未保存修改 | 新不可变版本 | POST `…/publish`（`worktreeGroupApi.ts:916, 923`） | ✅ |
| 回滚到历史版本 | admin、无未保存修改 | 产生新版本 | POST `…/rollback`（`:930, 937`；`HookPolicyPage.tsx:506-519`） | ✅ |
| 以当前生效版本重编辑 | 草稿过期 | 替换为生效版本基线 | 本地状态（`:594`） | ✅ |
| 添加规则、条件 | 可编辑；规则 ≤ 64，条件 ≤ 8 | 本地草稿变化 | 本地（`:597, 862`） | ✅ |
| 刷新 | 非忙碌 | 重新读取 | GET（`:618`） | ✅ |
| 加载更多事件 | 有 next_cursor | 追加事件页（上限 300 条驻留） | GET `hook-events`（`:817`；`worktreeGroupApi.ts:889`） | ✅ |
| 选择 Run admission 阶段 | producer 为真 | 规则阶段改为 Run admission | 本地草稿 | 🟡（producer 从未为真） |

- **数据**：来源为 Group API（`worktreeGroupApi.ts:880-937`）。字段包括 `policy_set_id`、`draft`（`is_stale`）、`audit`、`producer_capabilities.run_admission`、事件 `hook_phase/hook_decision/event_id`、summary `coverage`（`status`、`instrumented_phases`、`not_yet_instrumented_phases`、`run_state_join`）与 `excluded_incomplete_run_event_count`。
- **与文档设计的差距**
  - FR-18-06：决策集合与条件目录未逐项核对。
  - FR-18-07、FR-18-12：Run admission 无生产 producer，选项恒禁用。
  - FR-18-08：只有 archive 阶段接入。
  - FR-18-05：宿主认证 issuer 未接入，生产环境中页面仅在 run-directory session 存在时可用（WLD5 report §3 gap 1）。
  - FR-18-10：builtin 基线未在 UI 中以只读规则列出（`HookPolicyPage.tsx` 中未核对到"builtin 只读"段落）。

### 4.7.3 /settings/advanced/skills

- **文件**：`frontend/src/app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx:31`（`SkillsSettingsPanel`）。
- **布局**：区块与标签导航同 4.7.2（`advanced/layout.tsx:44-75`）；页内主区为面板根容器 `data-testid="advanced-settings-skills-panel"`（`SkillsPluginsPanels.tsx:51`）。
- **组件树**：`AdvancedSettingsTabPage`（`[tab]/page.tsx:39`）→ `SkillsSettingsPanel` → 添加表单（`:41-48`，`formError` `:83`）→ 列表（空态 `:92`）。
- **状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 注册表中有 skill | 列表（名称、版本、作用域、启停） | 🟡（数据为 seed + localStorage） |
| 空 | 无 skill | `t.settingsCapabilities.skills.empty`（`:92`） | ✅ |
| 表单错误 | 名称为空 | `role="alert"` 文案（`:83`） | ✅ |
| 加载 | 无异步读取 | 不适用 | — |
| 错误（服务端） | 无服务端 | 不存在 | ⬜ |
| 未授权 | 无认证模型 | 不存在 | ⬜ |
| 未接入 | 无 Skill 运行时 | 未显示未接入状态；seed 作为数据展示 | 🟡（违反 AGENTS.md:22） |

- **交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 添加 skill | 名称非空 | 写入注册表 | `capabilityRegistryApi.addSkill`（`:46`）→ localStorage `star-capability-registry:v1` | 🟡 |
| 启停 | 已存在 | 切换 enabled | `capabilityRegistry.ts:240` | 🟡 |
| 删除 | 已存在 | 移除 | `capabilityRegistry.ts:232` | 🟡 |
| 匹配或自动加载 | — | 无 | 无 | ⬜ |

- **数据**：`capabilityRegistry.ts` 的 `SkillEntry`（`name`、`version`、`scope`、`enabled`、`description`，`:50-58`），seed 两条 `skill-seed-code-review`、`skill-seed-tdd`（`:107-124`）。
- **与文档设计的差距**
  - FR-18-25：面板存在，但数据是 seed。
  - FR-18-26、FR-18-27、FR-18-28：status、版本、三级作用域、SKILL.md 模板、匹配与自动加载均缺失。

### 4.7.4 /settings/advanced/mcp

- **文件**：`frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx:38`（`McpSettingsPanel`）。
- **布局**：同 4.7.2；面板根容器 `data-testid="advanced-settings-mcp-panel"`（`:79`）。
- **组件树**：`AdvancedSettingsTabPage`（`[tab]/page.tsx:38`）→ `McpSettingsPanel` → 添加表单（`:52-76`，`formError` `:47`）→ 服务器列表（空态 `:178`，`enabledCount` `:90`，状态图标 `:193`，`last_error` `:226-228`）。
- **状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 注册表中有服务器 | 名称、transport、scope、健康徽章、启停、删除 | 🟡（seed 数据） |
| 空 | 无服务器 | `mcp-empty`（`:178`） | ✅ |
| 表单错误 | 名称或命令为空 | `mcp-form-error`（`:165`） | ✅ |
| 错误（服务器） | `last_error` 存在 | 错误图标与文本（`:193, 226-228`） | 🟡（健康值来自 seed 或本地写入，服务端不探测） |
| 加载 | 无异步读取 | 不适用 | — |
| 未授权 | 无认证模型 | 不存在 | ⬜ |
| 未接入 | 无 MCP REST | 未显示未接入状态 | 🟡（违反 AGENTS.md:22） |

- **交互**

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 添加服务器 | 名称与命令非空 | 写入注册表，`enabled: true`，`scope` 为 user 或 project | `capabilityRegistryApi.addMcpServer`（`:62-70`） | 🟡 |
| 启停、删除 | 已存在 | 切换或移除 | `capabilityRegistry.ts:209, 217` | 🟡 |
| 健康检查 | — | 无服务端入口 | `POST …/check`（设计 `api-design-mcp-capabilities.md:35`） | ⬜ |

- **数据**：`McpServerConfig`（`capabilityRegistry.ts:35-49`）；seed `mcp-seed-filesystem`（stdio，user，`:81`）与 `mcp-seed-github`（http，project，`:94-97`）。
- **与文档设计的差距**
  - FR-18-22：登记与启停存在，但仅为本地数据。
  - FR-18-23：REST CRUD 与异步健康检查缺失（`docs/api-design-mcp-capabilities.md` §2、§5）。
  - 设计 `settings-capabilities-design.md` §3 要求"web 无后端时降级 localStorage seed"，与 AGENTS.md:22 的"未接通授权数据须显示未接入状态"冲突，见 §6。

### 4.7.5 /settings 页面的 mcp、skills、hooks 标签

- **文件**：`frontend/src/app/(app)/settings/page.tsx`；标签面板 `frontend/src/components/settings/McpSettingsPanel.tsx`、`SkillsSettingsPanel.tsx`、`HooksSettingsPanel.tsx`。
- **布局**：左侧导航分组（"扩展"组，`settings/page.tsx:89-96`，mcp `:92`、skills `:93`、plugins `:94`、hooks `:95`）；主区 `<main>`（`:201`）。
- **组件树**：`SettingsPage`（行号未核对）→ 分派（`:216-233`）：mcp → `McpSettingsPanel`（`:217`）；hooks → `HooksSettingsPanel`（`:231`）；skills → `SkillsSettingsPanel`（`:233`）。旧 tab 映射 `LEGACY_TAB_MAP`（`:144-153`），未知值回退到 mcp（`:169`）。
- **状态矩阵**

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常（Hooks） | 有 localStorage 中的 hook | 分组列表，启停即时写入（`HooksSettingsPanel.tsx:621`） | 🟡（与高级设置的 Hooks 并存） |
| 空（Hooks） | 无条目 | 空态与"诚实徽章"（测试 `HooksSettingsPanel.test.tsx`） | ✅ |
| 错误（存储） | localStorage 不可用或损坏 | 安全降级为空列表（`readHooks` `:138-150`） | ✅ |
| 未接入（MCP 测试连接） | stdio 传输 | 返回"stdio 传输无 HTTP 端点"（`components/settings/McpSettingsPanel.tsx:188`） | ✅ |
| 错误（Skill 添加） | 校验失败 | `skills-add-error`（`components/settings/SkillsSettingsPanel.tsx:360-362`） | ✅ |

- **交互**：Hook 增删改、启停写 `localStorage` 键 `star-hooks`（`HooksSettingsPanel.tsx:152-156`）；Command 字段为任意字符串（`:70`）。MCP 测试连接执行浏览器 `fetch` HEAD（`McpSettingsPanel.tsx:204`）。
- **数据**：localStorage 三个键（`star-hooks` `HooksSettingsPanel.tsx:53`、`star-mcp-servers` `components/settings/McpSettingsPanel.tsx:70`、`star-skills` `components/settings/SkillsSettingsPanel.tsx:70`）。
- **与文档设计的差距**
  - FR-18-30：此处是第二个 Hook 配置入口，存储与语义与 Hook SRS 不符（`command` 字符串、无 policy 版本、无 CAS、无审计）。
  - 同一能力存在两套 MCP、Skills 面板（`components/settings/*` 与 `advanced/[tab]/*`），数据键不同（`star-mcp-servers` 与 `star-capability-registry:v1`），两处状态不同步。
  - 三个 tab 的默认值与 `LEGACY_TAB_MAP` 的回退行为与"高级设置为唯一入口"的约定不一致。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| Rust Hook evaluator | `crates/domain-hook/src/lib.rs` | ✅ | `:80-116`、`:373-418`、`:548` |
| Hook policy REST | `crates/star-api-rest/src/group_api/hook_policies.rs` | ✅ | `:301-343`、`:851-866`、`:900-903` |
| Group API 挂载 | `crates/star-api-rest/src/group_api.rs` | ✅ | `:215`、`:827` |
| Worktree 归档门禁 | `crates/star-api-rest/src/group_api/worktrees.rs` | ✅ | `:626-707`、`:1187-1202`、`:1311` |
| Run admission producer 接缝 | `crates/star-api-rest/src/group_api/cli_sessions.rs` | 🟡 | `:254-256`、`:556`、`:754`、`:1442-1444` |
| Hook 策略迁移 | `db/migrations/2026-09-30-multica-hook-policy.sql` | 🟡（源码存在；目标库部署未核实） | `:29, 75, 121, 190, 245` |
| Hook 事件迁移 | `db/migrations/2026-10-01-multica-hook-execution-event.sql` | 🟡（同上） | `:20, 69` |
| 高级设置标签壳 | `frontend/src/app/(app)/settings/advanced/layout.tsx`、`page.tsx`、`[tab]/page.tsx` | ✅ | `layout.tsx:37-75`；`page.tsx:12`；`[tab]/page.tsx:37-41` |
| Hooks 策略页 | `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` | 🟡 | `:146-147`、`:532-536`、`:741-817` |
| Hook 执行事件 API 客户端 | `frontend/src/lib/group/worktreeGroupApi.ts` | 🟡 | `:478-613`（类型）、`:880-937`（方法） |
| Group API provider 挂载 | `frontend/src/app/providers.tsx` | 🟡 | `:50-59`（仅在 session 存在时） |
| 守卫规则库 | `scripts/automation/guardian/rules/pre_tool_use_rules.json` | 🟡 | 15 条（BLOCK 8、ASK 5、WARN 2） |
| 守卫决策与扫描 | `scripts/automation/guardian/pre_tool_use_guard.py` | 🟡 | `:22-27`、`:62-82`、`:141-210` |
| 守卫规则加载与审计 | `scripts/automation/guardian/rule_database.py`、`audit_logger.py` | 🟡 | `rule_database.py:94, 146, 186`；`audit_logger.py:49, 54, 74, 91` |
| console 接入 | `scripts/automation/console_server.py` | 🟡 | `:41-58`、`:263-284` |
| dispatcher 接入 | `scripts/automation/dispatcher.py` | 🟡 | `:238-273` |
| 守卫 hook 片段（未被导入） | `scripts/automation/guardian/hooks/console_server_hook.py`、`dispatcher_hook.py` | 🟡（片段未被导入；仅作接入示例） | 文件头注释 |
| Python Hook 引擎 | `scripts/automation/hooks/`（EM-1 至 BL-6） | 🟡（非权威兼容；仅测试导入） | `__init__.py:1-30`；`tests/automation/hooks/*` |
| 内置守卫 Hook（Python） | `scripts/automation/hooks/builtin/pre_tool_use_guard.py` | 🟡（仅测试导入） | 文件 99 行 |
| MCP 领域 crate | `crates/domain-mcp/src/lib.rs` | 🟡（无调用方） | `:29, 46, 74, 91` |
| MCP、Skills、Plugins 高级面板 | `frontend/src/app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx`、`SkillsPluginsPanels.tsx` | 🟡（seed + localStorage） | `McpSettingsPanel.tsx:38-76`；`SkillsPluginsPanels.tsx:31-48` |
| 能力注册表（UI 数据层） | `frontend/src/lib/settings/capabilityRegistry.ts` | 🟡 | `:76`、`:78-125`、`:141-164`、`:192-270` |
| 旧版设置面板（MCP、Skills、Hooks） | `frontend/src/components/settings/{McpSettingsPanel,SkillsSettingsPanel,HooksSettingsPanel}.tsx` | 🟡（与高级面板重复） | `HooksSettingsPanel.tsx:53, 66-76, 514`；`McpSettingsPanel.tsx:70`；`SkillsSettingsPanel.tsx:70` |
| CLI Skill 注册表 | `crates/star-cli/src/skill_registry.rs` | 🟡（in-memory MVP） | `:122-210`；`crates/star-cli/src/main.rs:19, 70, 76` |
| Skill 生命周期、匹配、自动加载、SKILL.md 模板 | — | ⬜ | 仓内无对应代码；`docs/skills/` 不存在 |
| MCP REST 与健康检查 | — | ⬜ | 无 `/api/v1/capabilities` 路由 |
| Rust 工具调用、validation、review Hook 阶段 | — | ⬜ | `lib.rs:96-103` |
| Rust builtin PreToolUse 规则（迁移自 Python） | — | ⬜ | `HookPhase` 无工具调用阶段 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| Run admission 无生产 `TaskCliSessionProvisioner` adapter（`cli_sessions.rs:256` 恒为默认 false） | Run 在启动前无 Hook 决策，覆盖只限 archive 阶段 | 实装并装配 adapter，实现 readiness 与 fence，并以集成测试覆盖 SRS AC-10 的正负路径 |
| Rust 侧缺少工具调用、validation、review 阶段（`lib.rs:96-103`） | Hook SRS §1.6 的安全关键阶段未覆盖，工具调用无 Rust 决策点 | 扩展 `HookPhase` 与对应 producer；为每个阶段定义 typed 事实与 fail-closed 行为 |
| PreToolUse 的 ASK、WARN 决策无动作（`console_server.py:278-279`、`dispatcher.py:265-266`） | 需人工确认的破坏性命令（如 sudo、强制推送）直接放行 | 接入 ask_user（推荐项为取消），并实现 WARN 注入；或将这些规则迁入 Rust 并以 require_human 处理 |
| 守卫调用方对异常 fail-open（`console_server.py:282-284`、`dispatcher.py:271-273`），与 Hook SRS 的 critical fail-closed 相反 | 规则或守卫自身异常时危险命令被放行 | 将守卫迁入 Rust evaluator；过渡期将调用方异常改为 BLOCK，并在设计中记录裁决 |
| 按工具调用拦截（bash、write、edit）无调用方 | 守卫规则在主要工具路径上不生效，仅在脚本与派发入口生效 | 在工具调用边界接入守卫，并增加对应集成测试 |
| 审计写入 `args_excerpt`（命令前 200 字符，`pre_tool_use_guard.py` 的 `evaluate`） | 命令中粘贴的凭据可能进入审计日志，与 SRS NFR-S-1 冲突 | 对 `args_excerpt` 做凭据模式脱敏，或仅保留 `tool_args_hash`；增加渗透测试用例 |
| 宿主认证 provider 仅在 run-directory session 下挂载（`providers.tsx:50-59`） | 生产环境中 Hooks 页面可能无法读取真实策略 | 接入认证 issuer 与 session callback（WLD5 report §3 gap 1），并执行端到端 read/draft/publish/rollback 验证 |
| MCP、Skills、Plugins 面板展示 seed 数据并写入 localStorage | 违反 AGENTS.md:22"未接通授权数据须显示未接入状态"，用户会把示例当作真实配置 | 在无后端时显示未接入状态，移除 seed 展示；仅在开发环境保留 fixture |
| `/settings` 页面的 Hooks 面板写入 `star-hooks`，`command` 为任意字符串 | 第二个 Hook 配置入口可保存 shell 命令字符串，绕过 policy 版本、CAS 与审计，违反 Hook SRS FR-4.1、FR-6.2 | 删除 `/settings` 的 Hooks 配置入口，使其导向高级设置；迁移或丢弃 `star-hooks` 数据 |
| MCP、Skills 两套面板并存（`components/settings/*` 与 `advanced/[tab]/*`），数据键不同 | 同一能力两处状态不一致，维护成本高 | 保留高级设置版本，删除 `components/settings` 中的重复面板与测试 |
| MCP REST CRUD、健康检查与 `domain-mcp` 未接入（`api-design-mcp-capabilities.md` §2、§5） | MCP 配置无法被 Agent 运行时消费，健康状态不真实 | 在 `star-api-rest` 增加 `routes/capabilities.rs`，接入 `McpServerRegistry` 的 list、add、remove、health_check |
| Skill 运行时缺失（生命周期、模板、匹配、自动加载） | Skill 无法跨 Agent 复用，设计中的主要价值未落地 | 先定义 `docs/skills/` 存储与 SKILL.md schema，再实装 registry、matcher 与 loader；与 Agent 运行时的上下文组装对接 |
| Hook 执行事件与 BI 仅覆盖 archive 阶段 | coverage 为 partial，BI 指标不能代表全阶段 | 随 producer 扩展 `instrumented_phases`，并保持 unknown 而非 0 的口径 |
| 守卫审计的 append-only 仅在 Python 文件层面声明（`pre_tool_use_guard.py`、`audit_logger.py`），未验证 | NFR-S-2 未证明 | 迁移到 Rust audit 写入路径，或增加文件权限与不可改写校验 |
| Python 守卫规则热更新 watcher 在 dispatcher 中关闭（`dispatcher.py:249`） | 规则变更需重启派发侧进程才生效 | 统一热更新语义，或在文档中明确生效时机 |
| 守卫测试路径与 spec 不一致（spec 要求 `tests/e2e/test_pre_tool_use_hook.py` 与 `tests/security/test_pre_tool_use_penetration.py`，仓内缺失） | 验收 AC-1、AC-7 缺少集成与渗透证据 | 补齐测试文件，或修订 spec 中的交付清单 |
| 目标数据库中的 migrations、RLS、grants 未核实 | Hook 策略与事件的生产可用性未证明 | 在目标环境执行 migrations，验证 RLS 与 grants，并记录核对结果 |

## 7. 验收与测试

**前端（Vitest，无端到端浏览器测试）**

- `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx`：共 17 个 `it`。已列出的用例包括无宿主会话时不显示 seed（`:83`）、会话过期清空（`:92`）、只显示授权执行事件并明示 partial coverage（`:144`）、producer 存在时才启用 Run admission（`:226`）、本地、应用壳与浏览器退出的未保存守卫（`:296`）、深链授权目录（`:502`、`:536`、`:556`、`:566`、`:581`、`:599`）。已验证：上述 UI 行为与未授权时不发写请求。未验证：真实认证、真实数据库。
- `frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx`：7 个 `it`，覆盖 MCP 列表、添加、校验、启停与删除，Skills 列表、添加与删除。已验证：本地 registry 的 UI 行为。未验证：服务端持久化。
- `frontend/src/app/(app)/settings/advanced/[tab]/layout.test.tsx`：高级设置标签壳。
- `frontend/src/components/settings/HooksSettingsPanel.test.tsx`、`McpSettingsPanel.test.tsx`、`SkillsSettingsPanel.test.tsx`：旧版面板，覆盖 localStorage 持久化与表单校验。

**Rust**

- `crates/domain-hook/src/lib.rs` 测试模块（`:1075-1282`），包括 `unknown_retention_lock_fails_closed`、`safe_archive_gate_allows`、`bounded_policy_loader_verifies_serialized_snapshot`、`project_deny_overrides_other_restrictions`、`worktree_allow_rule_is_rejected_before_evaluation`、`scope_mismatch_fails_closed`、`wrong_event_schema_fails_closed`、`tenant_policy_mismatch_fails_closed`、`changed_policy_payload_is_rejected_even_when_digest_is_present`、`oversized_policy_is_rejected_before_digest_serialization`、`unknown_policy_fields_are_rejected`。已验证：evaluator 的 fail-closed 与策略校验。
- `crates/star-api-rest/src/group_api/worktrees.rs` 的 `archive_gate_tests`（`:1630-1698`）：`lock_facts_fail_closed`、`readiness_rejects_stale_and_future_observations`、`archive_gate_facts_require_fresh_observations`、`admission_fence_requires_commit_margin`。
- `crates/run-owner-runtime/tests/schedule_admission_pg.rs:1465-1477` 使用 `HookPolicyDocument` 构造 PostgreSQL 测试夹具。未验证：本文档未运行 PostgreSQL 集成测试。
- Run admission producer 的正负路径测试（SRS AC-10）未在 `cli_sessions.rs` 的测试模块中核对到（❓）。

**Python（pytest，位于仓内实际路径）**

- `scripts/automation/guardian/tests/test_pre_tool_use_guard.py`：16 个 `def test_`，覆盖规则匹配、决策路由与 fail-open。
- `scripts/automation/guardian/tests/test_penetration.py`：3 个 `def test_`。
- `scripts/automation/guardian/tests/test_rule_database.py`、`test_audit_logger.py`、`test_e2e_integration.py`、`test_e2e_dispatcher.py`：规则库、审计与集成。
- `tests/automation/hooks/test_event_emitter.py`：13 个 `def test_`；`test_hook_registry.py`：16 个；另有 `test_audit_logger.py`、`test_fanout_runner.py`、`fixtures/test_hooks.json`。这些测试仅覆盖非权威 Python 引擎。

**未验证**

- 守卫 spec 要求的 `tests/e2e/test_pre_tool_use_hook.py` 与 `tests/security/test_pre_tool_use_penetration.py` 在基线中不存在。
- 无测试证明 Run admission 的 `supports_run_admission()` 可为真。
- 无测试覆盖 ASK、WARN 的动作路径（因其无动作）。
- 无端到端测试覆盖 `/settings/advanced/hooks` 的 draft、publish、rollback 完整链路（BD §7 与 SRS FR-8.1 要求）。
- 无测试覆盖 MCP、Skill 的服务端行为（因其不存在）。
- 决策选项常量 `DECISIONS`（`HookPolicyPage.tsx:128`）是否排除 Allow，未逐项核对。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-MULTICA-SKILL-001.md`（全文） | Skill 生命周期、SKILL.md 模板、三级作用域、匹配与自动加载、FR-1 至 FR-18 | 代码中无对应运行时；`docs/skills/` 不存在 | 采纳为目标需求，状态按代码判定为 ⬜（FR-18-26 至 FR-18-28）或 🟡（UI 层） |
| `docs/design/DD-MULTICA-SKILL-001.md`（全文） | 四个类、状态机、三张表（W/T/M）、四个 API 端点 | 类与表无代码；引用的 `docs/design/BD-MULTICA-SKILL-001.md` 与 `docs/detailed-design/DD-MULTICA-SKILL-001.md` 不存在 | 采纳为设计意图，不作为已实现证据 |
| `docs/requirements/SRS-MULTICA-HOOK-001.md`（全文，v0.5.6） | Rust-native HookSet、四种决策、phase、fail-closed、Project/Worktree 继承、事件 FR-1 至 FR-8 | v0.1 至 v0.4 的 Python 模型已由 §1.6 与 §0 标注为历史草案 | 以 v0.5.6 为准；Python 模型仅作兼容输入 |
| `docs/design/BD-MULTICA-HOOK-001.md`（§0、§0.1、§2.3、§5.1、§6.3、§7.1 至 §7.7、章节标题） | Rust engine 的 W/T/M 归属、UI 布局、Phase 9C、9D 与 9D-5a/5b 边界、Run admission 契约 | §4 至 §6.3 的 Python 模块、JSON registry、四种 action type 与 `transform` 已被 §0.1 标注为旧草案；§7.2 的三栏布局与代码的表单结构不完全一致 | 以 §0.1 与 §7 为准；§4 至 §6.3 仅作历史参考 |
| `docs/detailed-design/DD-MULTICA-HOOK-001.md`（仅标题） | §7.1"Rust-native Hook Engine 实装契约（v0.5 authority）"为权威位置 | 正文未逐段核对 | 标注 ❓：§7.1 契约未逐条与代码对照 |
| `docs/requirements/SRS-PRE-TOOL-USE-GUARD-001.md`（全文，v0.1） | 15 条规则、三级决策、审计字段、fail-open 与 fail-closed、NFR | 规则定义为 Python 实现；与 SRS-HOOK v0.5.6 的 Rust-native 决策相冲突 | 依 SRS-HOOK v0.5.6 为准。守卫的 Python 行为作为当前实现与缺口记录（§6） |
| `docs/design/BD-PRE-TOOL-USE-GUARD-001.md`（§0、§2.1、§3.1 至 §3.2、§4.1 至 §4.2、§5.1、§5.4、§8.1） | 模块划分、数据表 W/T/M 归类、接口签名、故障模式 F-1 至 F-8 | 假设接入点在 `console_server.py` 80-92 行（实际 263-284）；假设按 bash、write、edit 扫描（运行时无调用方） | 以代码为准；接入点与工具路径按实际位置记录 |
| `docs/specs/guard-pre-tool-use-spec.md`（§1、§2.1、§3.1 至 §3.2、§7） | 公共 API、交付清单、ToolCall 约束 | 测试路径（`tests/automation/guardian/`、`tests/e2e/`、`tests/security/`）与代码路径（`scripts/automation/guardian/tests/`）不一致；`tests/e2e`、`tests/security` 缺失 | 以代码路径为准；缺失项记入 §6 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` §23.33（:949 起） | MCP、Skills、Plugins 面板的实装拆分（A 实色弹层、B 三面板、C 文档）；当时现状为占位 | 当时为占位，代码中面板已存在但为 seed 数据 | 作为面板设计的最新依据；数据层为 seed 与 localStorage，与 AGENTS.md:22 冲突，记入 §6 |
| `docs/frontend/design/settings-capabilities-design.md`（§2、§3、§6、§7） | 路由 tab 分派、`capabilityRegistry` 数据层、seed 与 localStorage 降级、验收项 | 设计允许 seed 降级，与 AGENTS.md:22 冲突 | 路由与组件结构采纳；seed 降级不采纳，记入 §6 |
| `docs/api-design-mcp-capabilities.md`（§2、§5） | MCP REST 端点与后端落地路径（`domain-mcp` 扩展、`routes/capabilities.rs`） | 端点在代码中不存在 | 作为 FR-18-23 的设计依据；状态 ⬜ |
| `docs/briefs/WLD5-HOOK-POLICY-UI-20261020.md`（仅标题） | WLD-5 的 Hooks UI 切片任务与门禁 | 正文未逐条核对 | 仅作为背景；状态以代码与 slice report 为准 |
| `docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md`（§0 至 §3） | 未保存草稿守卫、WLD-5 仍开放、宿主认证与 Route Guard 缺口 | 报告中提到的 `WLD5-GLOBAL-ROUTE-GUARD-20261021.md` 存在但未读取 | 采纳为当前 UI 状态；WLD-5 视为未关闭 |
| `docs/reports/WLD5-GLOBAL-ROUTE-GUARD-20261021.md` | 未读取（存在于基线） | — | 未纳入状态判断；未读取 |
| `docs/detailed-design/DD-PRE-TOOL-USE-GUARD-001.md`、`docs/plans/WBS-002-pre-tool-use-guard.md`、`docs/reports/PHASE-PRE-TOOL-USE-GUARD-IMPL-REPORT.md` | 存在于基线，未读取 | 报告的完成声明与代码可能不一致 | 不采用报告中的完成声明作为证据；状态以代码为准 |
| `docs/requirements/SRS-MULTICA-MCP-001.md` | 参照于 Hook SRS 与 Settings 设计，但基线中不存在 | — | 不作为设计来源；MCP 需求以 `api-design-mcp-capabilities.md` 为准 |
| `docs/features/F17-settings-capabilities.md` | 与本文 MCP、Skills 面板重叠的相邻特性文档，基线中存在 | 未读取 | 未以其状态作为证据；边界以本文 FR-18-22 至 FR-18-25 为准，重叠部分需在 F17 中核对 |
| `scripts/automation/hooks/`（Python 引擎） | 6 个模块与 registry.json | 仅测试导入，无运行时调用方；内置装载 Hook 无注册条目 | 非权威兼容代码，状态 🟡 |
| `frontend/src/lib/ipa-skills/catalog.ts` | 存在，未读取内容 | 名称指向 IPA 工作流目录，不是 SRS-SKILL 的 registry | 不纳入 Skill 状态判断 |
| `crates/star-cli/src/skill_registry.rs` | 文件头 MVP 说明与 `SkillRegistry` 公共方法 | 与 SRS-SKILL 的 status、version、scope 模型不一致；未接入 REST 或 Agent 运行时 | 作为 CLI 局部能力，状态 🟡 |
| `AGENTS.md`（基线版本，第 22 行） | 高级设置 → Hooks 为唯一 Hook 配置入口；未接通授权数据须显示未接入状态；不得以本地 seed 冒充生产管理 | 与 `capabilityRegistry` seed 与 `/settings` Hooks 面板冲突 | 作为约束采纳；冲突记入 §6 |

**互相冲突的源与裁决**

- **守卫的失败语义**：SRS-PRE-TOOL-USE-GUARD v0.1 要求规则加载失败 fail-open；SRS-MULTICA-HOOK v0.5.6 要求关键 Hook 的策略或审计不可验证时 fail-closed，并把 Python handler 降为非权威兼容输入。裁决：采用 SRS-HOOK v0.5.6，因其为最近的已接受设计。守卫当前的 fail-open 视为待修复的现状缺口，而非设计目标。
- **Hook 实现位置**：BD-HOOK §4 至 §6.3 描述的 Python 注册中心与 JSON 存储，与 §0.1 及 §7 的 Rust 权威模型冲突。裁决：以 §0.1 与 §7 为准。
- **面板数据来源**：settings-capabilities-design §3 允许 seed 降级，AGENTS.md:22 要求未接通时显示未接入状态。裁决：采用 AGENTS.md:22，seed 降级记为缺口。
- **守卫接入点**：BD-PRE-TOOL-USE 假设的接入点与实际代码位置不同。裁决：以代码为准。
