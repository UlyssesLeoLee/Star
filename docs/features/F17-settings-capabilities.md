# F17 设置总览与能力配置

> 状态摘要: 设置页已实装为 `/settings` 与 `/settings/advanced/[tab]` 两套并行面板, MCP、Skills、Plugins、Hooks 的配置数据主要保存在浏览器 localStorage 中; 仅高级设置的 Hook 策略走服务端 REST 与 Postgres 持久化。MCP/Skills/Plugins 的服务端 CRUD、Mod 运行时隔离、Hook 传输方式均未实现。

> 代码基线: HEAD 1643559f (the single baseline for every feature doc; cite this exact hash); 核对方法: 路径与符号 grep 加工作树文件读取; 设置相关路径 (`frontend/src/app/(app)/settings`、`frontend/src/components/settings`、`frontend/src/lib/settings`、`frontend/src/lib/group`、`crates/star-api-rest/src/group_api`) 相对 HEAD 1643559f 无差异; 核对日期 2026-11-17。

## 1. 目标与边界

### 1.1 目标

- 提供设置总览: 左导航分组、深链、默认面板、开发者入口。
- 提供能力配置: MCP 服务器、Skills、Plugins、Hooks、Mod 的添加、编辑、启停、删除、测试与配置。
- 对齐 Claude Code 的 MCP 管理心智: 连接类型 stdio/sse/http, 作用域 user/project, 健康状态徽章。
- 所有尚未接入后端的能力须显式标注为本地持久化或未接入, 不以示例数据冒充真实状态。

### 1.2 范围

范围内:

- `/settings` 页面 (左导航、主区分派、`?tab=` 深链)。
- `/settings/advanced` 及其 tab 路由 (`skills`、`hooks`、`mcp`、`plugins`)。
- `/settings/developer` (Developer API 参考与出站 Webhook 本地 CRUD) 的入口关系。
- 能力注册表 `capabilityRegistry` 与 `/settings` 下的 MCP、Skills、Hooks、Mod、Integration 面板。
- 高级设置 Hooks 页面对服务端 hook-policy REST 的调用。

范围外 (各有专题文档, 本文件只记录入口与状态):

- 成员与身份、角色与权限、租户、个人资料的业务逻辑 (`components/settings/IdentityPanels.tsx`)。
- 总记忆 (`components/settings/MemorySettingsPanel.tsx`)。
- API Key 与 CLI 配置的完整页面 (`/settings/api-keys`、`/settings/cli-profiles`)。
- Hook 策略在运行时的执行与准入 (由 Rust 运行时消费, 见 4.6)。
- OAuth、mTLS、IP 白名单、Redis 限流等 Phase 2+ 能力。

## 2. 用户与场景

| 角色 | 场景 | 主要入口 |
|---|---|---|
| 个人开发者 | 添加本地 MCP 服务器并测试连接; 安装本地 Skill 或 Plugin; 启停 | `/settings?tab=mcp`, `/settings/advanced/mcp` |
| 项目管理员 | 为 project 作用域配置 Hook 策略草稿并保存 | `/settings/advanced/hooks` (项目/worktree 范围选择) |
| 集成开发者 | 查看 Developer API 参考, 登记出站 Webhook | `/settings/developer` |
| 任意用户 | 通过左导航切换面板, 通过 `?tab=` 深链到指定面板 | `/settings` |

## 3. 需求

优先级约定: P1 = SRS 或设计文档明确要求的能力; P2 = 预览、占位或扩展能力。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-17-01 | 左导航分组 (SRS 写为 5 组: 账户/组织与权限/集成/扩展/开发者) | P1 | 🟡 | 代码实际 6 组, 含 `Agent` 组: `frontend/src/app/(app)/settings/page.tsx:58-107`; `frontend/src/app/(app)/__tests__/panels.test.tsx:136` 注释仍写 5 组 |
| FR-17-02 | 默认面板为 MCP 服务器 | P1 | ✅ | `settings/page.tsx:169`; `panels.test.tsx:133-146` |
| FR-17-03 | `?tab=` 深链, 旧 tab 名经 `LEGACY_TAB_MAP` 映射 | P1 | ✅ | `settings/page.tsx:138-168` (客户端解析 `window.location.search`, 162-168); `panels.test.tsx:148-158` |
| FR-17-04 | 搜索设置入口 | P2 | ⬜ | `settings/page.tsx:211-213` 按钮无 `onClick`, 无搜索实现 |
| FR-17-05 | 开发者模式开关 (SRS FR-ST-004) | P1 | 🟡 | 开关按钮渲染于 `settings/page.tsx:208-210` 但无 `onClick`; 开发者入口仅经 StubPanel 跳转 `/settings/developer` (`page.tsx:238-247`, realHref 映射 244) |
| FR-17-06 | 高级设置 Hooks 深链 (SRS FR-ST-005) | P1 | 🟡 | 深链目标存在: `frontend/src/lib/nav/registry.ts:244-253` (href 247); `frontend/src/components/run/RunWorkspaceShell.tsx:231`; `/settings` 的 Hooks 项渲染本地面板而非深链 (`page.tsx:230-231`) |
| FR-17-07 | 高级设置 tab 导航 (skills、hooks、mcp、plugins) | P1 | ✅ | `frontend/src/app/(app)/settings/advanced/layout.tsx:37-42, 55-74`; `advanced/[tab]/page.tsx:37-41`; `advanced/[tab]/layout.test.tsx:26-51` |
| FR-17-08 | `/settings/advanced` 重定向到 `hooks` | P2 | ✅ | `settings/advanced/page.tsx:12` |
| FR-17-09 | 未知 tab 返回 404 | P2 | ✅ | `settings/advanced/[tab]/page.tsx:41` (`notFound()`) |
| FR-17-10 | MCP 官方目录与"已连接"徽标 (SRS FR-ST-002) | P2 | ⬜ | `settings/page.tsx:109-116` 的 `MCP_SERVERS` 常量全文件无引用; "已连接"徽标仅出现在未被引用的 `ModPanel` (`page.tsx:281-283`) |
| FR-17-11 | `/settings` MCP 服务器列表、新增、编辑、删除 | P1 | ✅ | `frontend/src/components/settings/McpSettingsPanel.tsx:450-704` (弹窗 246-443); 存储键 `star-mcp-servers` (`:70`) |
| FR-17-12 | 连接类型 stdio / sse / http (设计 §1) | P1 | 🟡 | 高级面板三种 (`frontend/src/lib/settings/capabilityRegistry.ts:31`; `advanced/[tab]/McpSettingsPanel.tsx:32-36`); `/settings` 面板仅 `stdio \| http` (`components/settings/McpSettingsPanel.tsx:74`), 无 sse |
| FR-17-13 | 作用域 user / project | P1 | 🟡 | 高级面板有 (`capabilityRegistry.ts:32, 43`; `advanced/[tab]/McpSettingsPanel.tsx:46, 155-163`); `/settings` 面板的 `McpServer` 类型无 scope 字段 (`components/settings/McpSettingsPanel.tsx:76-86`) |
| FR-17-14 | 健康状态徽章 unknown / connecting / healthy / error 与 last_error | P1 | 🟡 | 渲染存在 (`advanced/[tab]/McpSettingsPanel.tsx:190-197, 226-230`); 写入仅有 `addMcpServer` 置 `"unknown"` (`capabilityRegistry.ts:198`), 无代码路径更新 health 或 last_error |
| FR-17-15 | 健康探测 `POST .../check` (202 异步回写) | P2 | ⬜ | `docs/api-design-mcp-capabilities.md:35` 定义; 无对应路由 (见 FR-17-20) |
| FR-17-16 | 测试连接 (HEAD 失败或 405 回退 GET、超时、认证头) | P1 | 🟡 | `/settings` 已实现: `components/settings/McpSettingsPanel.tsx:186-233` (HEAD→GET 204-213, 超时 227); 高级面板无测试连接 |
| FR-17-17 | URL 与超时字段校验 | P1 | 🟡 | `/settings` 已实现: `validateMcpServerUrl` (`:159-174`), 超时范围 1000–60000 (`:273-275`); 高级面板仅校验名称与命令必填 (`advanced/[tab]/McpSettingsPanel.tsx:52-61`) |
| FR-17-18 | 认证方式 bearer / api_key 与凭据字段 | P1 | 🟡 | `/settings` 已实现 (`components/settings/McpSettingsPanel.tsx:73, 190-196`, 凭据仅本地持久化说明见 `:399`); 高级面板的 `McpServerConfig` 无认证字段 (`capabilityRegistry.ts:35-48`) |
| FR-17-19 | MCP 启用与停用 | P1 | ✅ | 高级面板: `advanced/[tab]/McpSettingsPanel.tsx:235-248`, `capabilityRegistry.ts:214-221`; `/settings`: `components/settings/McpSettingsPanel.tsx:499-501` |
| FR-17-20 | Capability REST CRUD (`/api/v1/capabilities/*`) | P2 | ⬜ | 设计定义于 `docs/api-design-mcp-capabilities.md:29-51`; 仓库 Rust 代码中无 `capabilities` 或 `mcp-servers` 路由 |
| FR-17-21 | Skills 列表、启停、移除 | P1 | ✅ | 高级: `advanced/[tab]/SkillsPluginsPanels.tsx:31-122`; `/settings`: `components/settings/SkillsSettingsPanel.tsx:581-744` |
| FR-17-22 | Skills 安装: 从 URL、手动 JSON | P1 | 🟡 | `/settings` 已实现 (`SkillsSettingsPanel.tsx:250-382`, `fetchSkillFromUrl` 160-165); 高级面板仅手填 name、version、description、scope (`SkillsPluginsPanels.tsx:66-89`) |
| FR-17-23 | configSchema 驱动的配置表单 | P2 | 🟡 | `/settings` 已实现 (`SkillsSettingsPanel.tsx:388-536`); 高级面板无 |
| FR-17-24 | Skills 官方目录 | P2 | ⬜ | `SkillsSettingsPanel.tsx:350-357` 占位, 提交按钮禁用 (`:374`), 目录页签报错 (`:265-267`) |
| FR-17-25 | 卸载二次确认 | P1 | 🟡 | `/settings` 已实现 (`SkillsSettingsPanel.tsx:542-575`); 高级面板直接删除无确认 (`SkillsPluginsPanels.tsx:112-113`) |
| FR-17-26 | Plugins 管理: 新增、启停、移除、权限 chip | P1 | 🟡 | 高级面板已实现 (`SkillsPluginsPanels.tsx:124-235`); `/settings` 为静态预览, 启停仅内存态不持久 (`components/settings/IntegrationPanels.tsx:397-431`, 注释 404) |
| FR-17-27 | 权限字符串解析 (逗号或空白分隔) | P1 | ✅ | `SkillsPluginsPanels.tsx:145`; `advanced/[tab]/CapabilityPanels.test.tsx:90-101` |
| FR-17-28 | 插件运行时隔离与执行 | P1 | ⬜ | 无执行代码; 启停仅改注册表 (`capabilityRegistry.ts:260-267`); 隔离说明文字仅存于未被引用的 `ModPanel` (`settings/page.tsx:312-316`) |
| FR-17-29 | `/settings` Hooks 本地管理: 按类型分组、增改删、启停、配置、删除确认 | P1 | ✅ | `frontend/src/components/settings/HooksSettingsPanel.tsx:514-731`; 存储键 `star-hooks` (`:53`) |
| FR-17-30 | `/settings` Hooks 执行 | P1 | ⬜ | `HooksSettingsPanel.tsx:37-38, 565-566` 明示执行引擎未接入 |
| FR-17-31 | 高级 Hooks 策略编辑: 范围 project/worktree, 决策 Deny/RequireHuman/Defer, 阶段 | P1 | ✅ | `advanced/[tab]/HookPolicyPage.tsx:128-138, 164-165`; 客户端 `frontend/src/lib/group/worktreeGroupApi.ts:879-898`; 服务端 `crates/star-api-rest/src/group_api/hook_policies.rs:301-344`; 测试 `advanced/[tab]/HookPolicyPage.test.tsx` (11 个用例) |
| FR-17-32 | Hook 策略草稿保存、发布、回滚 | P1 | 🟡 | 服务端与客户端方法齐备 (`worktreeGroupApi.ts:901-937`; 路由 `hook_policies.rs:319-342`); 页面仅调用草稿保存 (`HookPolicyPage.tsx:473-474`), 页面中未发现发布或回滚调用 |
| FR-17-33 | Hook 传输方式配置 (SRS §1 "策略/传输方式") | P1 | ⬜ | `HookPolicyPage.tsx` 无 `transport` 或"传输"相关代码 |
| FR-17-34 | 两处 Hooks 数据源统一 | P1 | 🟡 | `/settings` 使用 localStorage (`settings/page.tsx:230-231`), 高级设置使用服务端策略 (`advanced/[tab]/page.tsx:37`) |
| FR-17-35 | Mod 双源目录 (官方 / 自定义) (SRS FR-ST-003) | P2 | ⬜ | `components/settings/ModDirectory.tsx` 无源切换; 双源按钮仅存在于未被引用的 `ModPanel` (`settings/page.tsx:266-284`) |
| FR-17-36 | 浏览 Mod 目录并安装 | P2 | 🟡 | 本地模拟目录 `ModDirectory.tsx:39-44, 128-168`; 已安装列表为 `useState`, 不持久 (`:63`) |
| FR-17-37 | 从文件安装 Mod (.json, 校验 name/type/abilities/version) | P2 | 🟡 | `ModDirectory.tsx:48-56, 72-95`; 本地态, 不持久 |
| FR-17-38 | 用模板新建 Mod | P2 | 🟡 | `ModDirectory.tsx:97-116`; 本地态 |
| FR-17-39 | Mod 表含启用列 (SRS FR-ST-003) | P2 | 🟡 | 已安装列表存在 (`ModDirectory.tsx:236-257`), 无启用列 |
| FR-17-40 | Mod 隔离运行时说明与隔离执行 | P1 | ⬜ | `ModDirectory.tsx` 无隔离说明文本; 无 Mod 执行代码 |
| FR-17-41 | Mod 面板"已连接"徽标 | P2 | ⬜ | `ModDirectory.tsx` 无; 仅存在于未被引用的 `ModPanel` (`settings/page.tsx:281-283`) |
| FR-17-42 | 开发者控制台: Developer API 参考 (只读) | P2 | ✅ | `frontend/src/app/(app)/settings/developer/page.tsx:118-160` (静态端点表 119-128) |
| FR-17-43 | 出站 Webhook 本地 CRUD 与测试 | P2 | 🟡 | `developer/page.tsx:175-324`, 存储键 `star.outbound-webhooks.v1` (`:36`); 测试仅回显 demo 文本, 不做网络投递 (`:220-228`) |
| FR-17-44 | Webhook 密钥生成与轮换 | P2 | ⬜ | `components/settings/IntegrationPanels.tsx:322-334` 按钮禁用, 原因说明 `:331-333` |
| FR-17-45 | `/settings` 面板的诚实徽标 (标注本地持久化或未接入) | P1 | ✅ | `components/settings/McpSettingsPanel.tsx:692-698`; `SkillsSettingsPanel.tsx:705-711`; `HooksSettingsPanel.tsx:675-681`; `ModDirectory.tsx:121-124`; 高级面板无此徽标 |
| FR-17-46 | 其余左导航面板渲染 (个人资料、身份、角色与权限、租户、总记忆、代码托管、自动化、Feature Flags) | P2 | ✅ | `settings/page.tsx:43-47, 216-248`; 写操作未接入项禁用 (`components/settings/IdentityPanels.test.tsx:62-70, 154`) |
| FR-17-47 | 凭据与 CLI 配置跳转 | P2 | ✅ | `settings/page.tsx:241-246` (realHref); 目标路由存在: `settings/api-keys/page.tsx`, `settings/cli-profiles/page.tsx` |

## 4. 设计

### 4.1 概念模型

两套能力面板并存, 共享同一组能力类型, 但实体与存储不同:

| 能力 | 高级设置 (`/settings/advanced/[tab]`) | 设置页 (`/settings?tab=`) | 服务端 |
|---|---|---|---|
| MCP | `CapabilityRegistry` (`star-capability-registry:v1`) | `star-mcp-servers` | 设计存在, 未实现 |
| Skills | `CapabilityRegistry` | `star-skills` | 设计存在, 未实现 |
| Plugins | `CapabilityRegistry` | 静态预览, 启停不持久 | 设计存在, 未实现 |
| Hooks | 服务端 hook-policy (project 或 worktree 范围) | `star-hooks` (本地) | 已实现 (见 4.3) |
| Mod | 无 | 本地态 `ModDirectory` | 无 |
| 出站 Webhook | 无 | `/settings/developer` 使用 `star.outbound-webhooks.v1` | 无 |

设计意图 (`docs/frontend/design/settings-capabilities-design.md` §3): 注册表接口形状对齐未来 REST CRUD, Web 无后端时降级为 localStorage seed, 后端就绪后仅替换 `CapabilityRegistryApi` 的内部实现。

### 4.2 数据模型

客户端实体 (代码定义):

| 实体 | 字段 | 存储 | 位置 |
|---|---|---|---|
| `McpServerConfig` | id, name, transport (stdio/sse/http), command, args, env, scope (user/project), enabled, health, last_error, updated_at_ms | `star-capability-registry:v1` | `frontend/src/lib/settings/capabilityRegistry.ts:35-48`, 键 `:76` |
| `SkillEntry` | id, name, version, scope, enabled, description | 同上 | `capabilityRegistry.ts:50-57` |
| `PluginEntry` | id, name, version, scope, enabled, permissions[], description | 同上 | `capabilityRegistry.ts:59-68` |
| `McpServer` (设置页) | id, name, url, authType (bearer/api_key/none), authValue, enabled, transport (stdio/http), timeoutMs, createdAt | `star-mcp-servers` | `frontend/src/components/settings/McpSettingsPanel.tsx:76-86`, 键 `:70` |
| `StarSkill` | id, name, description, version, author, capabilities[], configSchema, config, enabled, installedAt | `star-skills` | `frontend/src/components/settings/SkillsSettingsPanel.tsx:72-86`, 键 `:70` |
| `HookConfig` | id, name, type (pre_commit/pre_push/post_merge/custom), command, args, env, enabled, description, matchPattern | `star-hooks` | `frontend/src/components/settings/HooksSettingsPanel.tsx:66-76`, 键 `:53` |
| `ModEntry` | id, name, type, abilities[], version, source (directory/file/template) | 不持久 | `frontend/src/components/settings/ModDirectory.tsx:28-35` |
| `OutboundWebhook` | id, name, url, events[], active, created_at | `star.outbound-webhooks.v1` | `frontend/src/app/(app)/settings/developer/page.tsx:27-36` |

服务端实体 (Hook 策略): PostgreSQL schema `multica`, 表 `hook_policy_set` (SCD Type 2 守卫)、`hook_policy_draft`、`hook_policy_audit_event` (追加式, 禁止更新删除), 启用租户 RLS; 来源 `db/migrations/2026-09-30-multica-hook-policy.sql:1-12` (表与函数清单); 写入 `crates/star-api-rest/src/group_api/hook_policies.rs:1135` (`INSERT INTO multica.hook_policy_draft`), `:1490` (`UPDATE multica.hook_policy_set ... valid_to`). 迁移在目标环境中的应用状态: ❓ (未核实)。

### 4.3 接口

前端路由:

| 路由 | 文件 | 说明 |
|---|---|---|
| `/settings` | `frontend/src/app/(app)/settings/page.tsx` | 左导航与主区分派; `?tab=` |
| `/settings/advanced` | `frontend/src/app/(app)/settings/advanced/page.tsx:12` | 重定向至 `/settings/advanced/hooks` |
| `/settings/advanced/[tab]` | `frontend/src/app/(app)/settings/advanced/[tab]/page.tsx:37-41` | tab 为 hooks、mcp、skills、plugins |
| `/settings/developer` | `frontend/src/app/(app)/settings/developer/page.tsx` | `?tab=webhooks` 切换 (`:74`) |

后端 REST (Hook 策略, `crates/star-api-rest/src/group_api/hook_policies.rs:303-342`, 合并于 `group_api.rs:817`):

| 方法 | 路径 | 处理函数 |
|---|---|---|
| GET | `/api/v1/projects/{project_id}/hook-policy` | `get_project_policy` (`:706-713`) |
| GET | `/api/v1/projects/{project_id}/hook-events` | `list_hook_events` (`:346`) |
| GET | `/api/v1/projects/{project_id}/hook-events/summary` | `summarize_hook_events` |
| GET | `/api/v1/worktrees/{worktree_id}/hook-policy/effective` | `get_worktree_effective_policy` (`:715`) |
| PUT | `/api/v1/projects/{project_id}/hook-policy/draft` | `save_project_draft` |
| PUT | `/api/v1/worktrees/{worktree_id}/hook-policy/draft` | `save_worktree_draft` |
| POST | `/api/v1/projects/{project_id}/hook-policy/publish` | `publish_project_draft` (`:778`) |
| POST | `/api/v1/worktrees/{worktree_id}/hook-policy/publish` | `publish_worktree_draft` (`:788`) |
| POST | `/api/v1/projects/{project_id}/hook-policy/rollback` | `rollback_project_policy` (`:806`) |
| POST | `/api/v1/worktrees/{worktree_id}/hook-policy/rollback` | `rollback_worktree_policy` (`:816`) |

Capability 的 REST 端点 (`/api/v1/capabilities/mcp-servers`、`/skills`、`/plugins`) 仅见于设计文档 `docs/api-design-mcp-capabilities.md:29-51`, 代码中未实现 (⬜)。

客户端 API 封装: `frontend/src/lib/group/worktreeGroupApi.ts:879-940` 与 `frontend/src/lib/group/groupProjection.tsx:104` (`useWorktreeGroupApi`)。

### 4.4 状态机与关键流程

- Hook 高级设置页: `loading`、`busy`、`error`、`notice` (`advanced/[tab]/HookPolicyPage.tsx:170-173`); 深链解析状态为 `pending` / `ready` / `blocked` (`:162, 177-183`); 策略读取前必须完成授权目录校验 (`:186-189`); 未保存修改通过导航守卫拦截 (`:144, 148`)。
- `/settings` Hooks 弹窗状态: `none` / `form(create|edit)` / `config` / `delete` (`components/settings/HooksSettingsPanel.tsx:91-96`)。
- MCP 测试连接: 每个服务器独立的 `running` 与 `result` 槽位 (`components/settings/McpSettingsPanel.tsx:447, 503-507`)。
- 面板三态 (`/settings` MCP): `loading` / `ready` / `error`, 读取失败进入 `error` (`components/settings/McpSettingsPanel.tsx:448, 451-474`)。
- 草稿发布流程 (设计意图): 草稿保存 → 发布 → 回滚; 服务端端点齐备 (4.3), 页面侧只接入保存 (FR-17-32)。

### 4.5 权限与安全

- `/settings` 页面本身不做角色判断 (`settings/page.tsx` 全文无角色或权限分支)。
- 高级设置的 Hook 策略端点均使用 `AuthenticatedUser` 提取器 (`hook_policies.rs:708`), 作用域经 `authorize_scope` 校验 (`:841`)。
- 未登录时不展示种子策略 (`advanced/[tab]/HookPolicyPage.test.tsx:83`)。
- 本地面板保存的凭据为明文 localStorage: MCP 的 `authValue` (`components/settings/McpSettingsPanel.tsx:81`), Hook 的 `env` (`HooksSettingsPanel.tsx:72`)。此为现状缺口, 见第 6 节。
- 插件与 Mod 的隔离运行时未实现 (FR-17-28、FR-17-40); 设计要求的"安全 Hook 由 Star 核心执行、Mod 不能替代"无代码支撑。
- Webhook 密钥在 `/settings` 的 Integration 面板中为静态掩码常量 (`components/settings/IntegrationPanels.tsx:116-118`), 不接入真实密钥。

### 4.6 依赖关系

- `/settings/advanced` 面板依赖 `frontend/src/lib/settings/capabilityRegistry.ts` 与 i18n `settingsCapabilities` 段 (`frontend/src/lib/i18n/zh-CN.ts:765`, 另有 en、ja、dictionary 同步)。
- `/settings` 面板依赖 `frontend/src/components/status/StatusNotice.tsx` (`McpSettingsPanel.tsx:68`) 与各自的 localStorage 键。
- 高级 Hooks 页依赖 `frontend/src/lib/group/worktreeGroupApi.ts` 与 `crates/star-api-rest/src/group_api/hook_policies.rs`; 运行时消费策略的调用点为 `group_api/worktrees.rs:1194`、`group_api/cli_sessions.rs:712`、`group_api/execution_catalogs.rs:193`。
- 导航注册: `frontend/src/lib/nav/registry.ts:232-253` (`settings` 指向 `/settings`; `advanced-settings` 指向 `/settings/advanced/hooks`)。
- MCP 服务端的已有基础: `crates/domain-mcp/src/lib.rs:29` 定义 `trait McpServer`, 含 `health_check` (`:37`); 设计要求的 list、add、remove 未在该 trait 中出现。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 设置页主框架 | `frontend/src/app/(app)/settings/page.tsx` | ✅ | `:58-107` 导航, `:162-253` 主区分派 |
| 设置页内联 Mod 面板 | `settings/page.tsx:256-319` (`ModPanel`) | 🗑 | 函数已定义但全文件无引用 (`ModDirectory` 取代, 提交 95ab45bf) |
| 设置页 MCP 常量 | `settings/page.tsx:109-116` (`MCP_SERVERS`) | 🗑 | 全文件无引用 |
| 高级设置布局与 tab | `settings/advanced/layout.tsx`, `advanced/[tab]/page.tsx` | ✅ | `layout.tsx:37-42`; `page.tsx:37-41` |
| 能力注册表 | `frontend/src/lib/settings/capabilityRegistry.ts` | ✅ | `:76-268` |
| 高级 MCP 面板 | `advanced/[tab]/McpSettingsPanel.tsx` | 🟡 | `:38-188` 表单与列表; 无测试连接、无认证 |
| 高级 Skills、Plugins 面板 | `advanced/[tab]/SkillsPluginsPanels.tsx` | 🟡 | `:31-122`, `:124-235`; 无安装来源、无配置表单、无卸载确认 |
| 设置页 MCP 面板 | `frontend/src/components/settings/McpSettingsPanel.tsx` | ✅ | `:450-704` (测试连接 `:186-233`) |
| 设置页 Skills 面板 | `frontend/src/components/settings/SkillsSettingsPanel.tsx` | ✅ | `:581-744` |
| 设置页 Hooks 面板 | `frontend/src/components/settings/HooksSettingsPanel.tsx` | ✅ | `:514-731`; 执行未接入 (`:37-38`) |
| 设置页 Integration 面板组 | `frontend/src/components/settings/IntegrationPanels.tsx` | 🟡 | `:397-431` Plugins 不持久; `:437-465` Hooks 静态; `:471-508` Flags 不持久 |
| Mod 目录 | `frontend/src/components/settings/ModDirectory.tsx` | 🟡 | `:62-262`; 本地态; 无双源、无启用列、无隔离说明 |
| 身份、角色、租户、个人资料面板 | `frontend/src/components/settings/IdentityPanels.tsx` | ✅ | 页面引入 `settings/page.tsx:43`; 测试 `IdentityPanels.test.tsx` 13 个用例 |
| 总记忆面板 | `frontend/src/components/settings/MemorySettingsPanel.tsx` | ✅ | 页面引入 `settings/page.tsx:46`; 测试 10 个用例 |
| 高级 Hook 策略页 | `advanced/[tab]/HookPolicyPage.tsx` | ✅ | `:146` 默认导出; 数据来自 `worktreeGroupApi` |
| Hook 策略 REST | `crates/star-api-rest/src/group_api/hook_policies.rs` | ✅ | 路由 `:301-344`, 鉴权 `:708`, 持久化 `:1135, :1490`; 合并 `group_api.rs:817` |
| Hook 策略 DDL | `db/migrations/2026-09-30-multica-hook-policy.sql` | ✅ | 表与守卫函数清单 `:1-12`; 应用状态 ❓ |
| Developer 控制台 | `frontend/src/app/(app)/settings/developer/page.tsx` | 🟡 | `:118-160` 静态参考; `:175-324` 本地 Webhook CRUD |
| 开发者模式开关 | `settings/page.tsx:208-210` | ⬜ | 无 `onClick` |
| 搜索设置 | `settings/page.tsx:211-213` | ⬜ | 无 `onClick` |
| Capability REST | (无) | ⬜ | 仅设计, 见 `docs/api-design-mcp-capabilities.md` |
| 插件与 Mod 运行时 | (无) | ⬜ | 无执行代码 |
| 凭据与 CLI 配置页 | `settings/api-keys/page.tsx`, `settings/cli-profiles/page.tsx` | ✅ | 路由存在; 由 `settings/page.tsx:241-246` 深链 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| MCP、Skills、Hooks 各有两套实现且存储键不同 (`star-mcp-servers` 对 `star-capability-registry:v1` 等) | 同一配置在两处不一致, 用户在一处添加的服务器另一处不可见 | 选定一套作为权威实现 (建议保留注册表接口形状), 迁移存储键, 另一套改为调用同一 API |
| `/settings` 导航 6 组, SRS 与测试注释写 5 组 | 文档与代码不符 | 修正 SRS FR-ST-001 与 `panels.test.tsx:136` 注释, 或删除 `Agent` 组 |
| `ModPanel` 与 `MCP_SERVERS` 为死代码 | 误导阅读者, 与 SRS FR-ST-002/003 描述冲突 | 删除, 或将 Mod 双源与隔离说明迁入 `ModDirectory` |
| 开发者模式与搜索设置按钮无行为 | 控件可见但无效 | 接入状态与搜索, 或在未实现前隐藏 |
| `ModDirectory` 与 SRS 不符 (无双源、无启用列、无隔离说明) | FR-ST-003 无法验收 | 按 SRS 补齐, 隔离说明为文字即可补入 |
| Capability REST 未实现 | 无服务端持久化, 无多端一致 | 按 `docs/api-design-mcp-capabilities.md` 实现路由并替换 `CapabilityRegistryApi` 内部实现 |
| 健康状态无探测, 只显示初始值 | 健康徽章无信息量 | 实现 `POST .../check` 与异步回写 |
| 高级面板缺测试连接、认证、配置表单、卸载确认、诚实徽标 | 与 `/settings` 面板能力不一致 | 复用 `/settings` 的实现或抽取共享组件 |
| 高级 Hook 页未调用发布与回滚 | 草稿无法在该页发布 | 核实是否由其他页面调用; 若无则接入 |
| 高级 Hook 无传输方式配置 | SRS §1 要求缺失 | 确认需求后补充 `transport` 字段与 UI, 或修正 SRS |
| `/settings` Hooks 与高级 Hooks 并存 | 用户面对两种 Hook 概念 | 统一入口, `/settings` Hooks 项改为深链高级设置 |
| 本地面板明文保存凭据 | 凭据泄露风险 | 凭据迁移到服务端密钥管理; 本地仅保存引用 |
| 插件与 Mod 运行时未实现 | 启停无实际效果, 与隔离承诺不符 | 在 Runtime 落地前保持徽标标注未接入 |
| 设计文档称 `HookPolicyPage` 有 13 条测试, 代码为 11 条 | 文档与代码不符 | 以代码为准更正设计文档 |

## 7. 验收与测试

| 测试文件 | 用例数 (静态计数) | 覆盖 |
|---|---|---|
| `frontend/src/app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx` | 7 | 高级 MCP、Skills、Plugins 的渲染、添加、校验、启停、移除、权限解析、localStorage 持久化 |
| `frontend/src/app/(app)/settings/advanced/[tab]/layout.test.tsx` | 1 | 高级设置 tab 导航与模块注册 |
| `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx` | 11 | 未登录态、会话过期、授权事件、Run admission 阶段、未保存修改守卫、深链校验与分页 |
| `frontend/src/app/(app)/__tests__/panels.test.tsx` | 含设置页用例 | 默认面板为 MCP、深链、旧 tab 映射 |
| `frontend/src/components/settings/McpSettingsPanel.test.tsx` | 22 | 设置页 MCP: 持久化、校验、编辑、启停、删除确认、测试连接 (成功、回退、失败、超时、stdio 拒绝、api_key 头) |
| `frontend/src/components/settings/SkillsSettingsPanel.test.tsx` | 12 | 设置页 Skills: 安装 (JSON、URL)、configSchema、卸载确认、启停、损坏数据容错 |
| `frontend/src/components/settings/HooksSettingsPanel.test.tsx` | 14 | 设置页 Hooks: 分组、增改删、启停、配置、删除确认、跨标签同步 |
| `frontend/src/components/settings/IntegrationPanels.test.tsx` | 15 | 集成面板组: 各面板列表、禁用按钮与原因、本地 toggle |
| `frontend/src/components/settings/ModDirectory.test.tsx` | 8 | Mod 目录: 本地安装、文件安装与校验、模板新建 |
| `frontend/src/components/settings/IdentityPanels.test.tsx` | 13 | 身份、个人资料、权限、租户面板 |
| `frontend/src/components/settings/MemorySettingsPanel.test.tsx` | 10 | 总记忆面板 |
| `frontend/src/app/(app)/settings/developer/page.test.tsx` | 4 | Developer API 参考、webhook 深链、CRUD 写入 localStorage、测试回显 demo |
| `frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts` | 含 hook-policy 路由用例 | 客户端端点路径与方法 (`:396-417`) |
| `frontend/src/lib/i18n/dictionaryConsistency.test.ts` | 三语键一致性 | `settingsCapabilities` 段三语同构 (设计文档声明; 文件存在) |

已验证 (静态, 通过代码阅读与计数): 上表覆盖的面板存在、路由存在、本地存储键与测试名称一致。

未验证: 本次核对未执行任何测试; 设计文档中"44/44 实测"的结果未复现; 服务端 hook-policy 的集成测试与数据库迁移应用状态未核实; 页面在浏览器中的交互未实测。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-SETTINGS-001.md` | FR-ST-001..005、实现文件映射、5 组导航、Mod 目录、开发者模式、Hooks 深链 | 状态 ✅ 写于 commit c0f149f2 (存在), 但 Mod 面板描述 (双源、三按钮禁用) 已被 95ab45bf 替换为 `ModDirectory` 本地实现; 导航实际 6 组; `/settings` 开发者模式开关无行为 | 以代码为准, FR-17-01、05、06、03 与 SRS 不一致, 标 🟡; SRS 需修订 |
| `docs/frontend/design/settings-capabilities-design.md` | 高级设置三 tab 的数据模型、交互、i18n、验收标准, 注册表形状 | 面板与注册表已实装 (`capabilityRegistry.ts`, `advanced/[tab]/`); 设计未提及 `/settings` 下并行实现; §4 "健康状态" 与 §6 "popover-solid" 的落实未逐项核对 | 高级设置部分采用; `/settings` 并行实现在本文件记录为差距 |
| `docs/frontend/design/settings-capabilities-test-design.md` | 测试分层、7 条用例清单、门禁、后续 v0.2 项 | 7 条用例与代码一致; "44/44 实测"未复现; v0.2 后端项未实现 | 采用测试清单; v0.2 项记为 ⬜ |
| `docs/prototypes/6 · 设置-html/README.md` | 原型用途与查看方式 | 仅为参考原型, 不含需求; `Settings.dc.html` 本次未逐行读取, 分组数以 SRS 与测试注释为据 | 仅作设计参考, 不作为状态依据 |
| `docs/api-design-mcp-capabilities.md` | Capability REST 资源模型与端点草案 | 端点未实现 (代码无对应路由) | 作为设计意图, 记为 ⬜ |
| `docs/prototypes/INDEX.md`、`docs/requirements.md`、`docs/basic-design.md`、`docs/plans/post-merge-development-plan-2026-09-02.md`、`docs/refactor/ui-audit-20261113.md` | 关键词命中 | 本次仅 grep 命中, 未读取正文; 与 F17 的设置面板范围关联较弱 | 未采用, 原因: 未读取正文, 且不属于 F17 核心范围 |
| `docs/design/BD-MULTICA-HOOK-001.md`、`docs/detailed-design/DD-MULTICA-HOOK-001.md`、`docs/requirements/SRS-MULTICA-HOOK-001.md`、`docs/reports/WLD5-HOOK-POLICY-UI-SLICE-20261020.md` | Hook 策略专题 (关键词命中) | 本次未读取; Hook 策略的服务端与运行时由专题文档覆盖 | 未采用于状态判定; F17 仅引用其代码路径 |
| `docs/features/F02-branch-workspace.md` | 关键词命中 | 其他功能的特性文档, 与 F17 无直接对应 | 不采用 |
| `frontend/src/app/(app)/settings/page.tsx` (代码) | 导航、分派、深链、死代码 | 与 SRS 不一致之处见第 3、6 节 | 代码为当前状态的权威 |
| `frontend/src/components/settings/ModDirectory.tsx` 与 `McpSettingsPanel.tsx` 等组件 (代码) | 设置页面板的实现与测试 | 与高级设置面板数据源不同 | 代码为当前状态的权威 |
| 提交 c0f149f2 | SRS 与 ModPanel 首次引入 | 其后 95ab45bf 引入 `ModDirectory` 并替换 StubPanel, `ModPanel` 变为未引用代码 | 以后续提交为准 |
| 提交 95ab45bf、7ac1465f、9c881ded | 设置页面板的实装历史 | 与 HEAD 一致 | 作为版本依据 |
