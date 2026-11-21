# F17 设置总览与能力配置

> 状态摘要: `/settings` 已实装为左导航加面板分派的设置页（默认 MCP 服务器），`/settings/advanced/[tab]` 提供 Skills、Hooks、MCP、Plugins 四个 tab；MCP、Skills、Plugins 的配置以浏览器 localStorage 持久化，服务端仅有 Hook 策略的路由与表结构，且默认 Web 构建下 Hook 策略页处于宿主认证未接入状态。能力 CRUD、健康探测、Mod 运行时隔离与 Hook 执行均未实现。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

1. 设置总览：`/settings` 以分组左导航承载账户、Agent、组织与权限、集成、扩展、开发者面板，并支持 `?tab=` 深链（SRS-SETTINGS-001 §1）。
2. 能力配置：对 MCP 服务器、Skills、Plugins、Hook 策略、Mod 目录提供添加、启停、移除与配置（settings-capabilities-design.md §1、§4）。
3. 能力模型使用中性术语：连接类型 `stdio` / `sse` / `http`，作用域 `user` / `project`，健康状态 `unknown` / `connecting` / `healthy` / `error`。
4. 尚未接入后端的能力显式标注为本地持久化或未接入，不以示例数据冒充真实状态（AGENTS.md 第 0 节硬约束，637ed19d 版 `AGENTS.md:22`）。

### 1.2 范围内 / 范围外

范围内：
- `/settings` 与其 `?tab=` 分派、页内左导航、页内头部。
- `/settings/advanced` 与 `/settings/advanced/[tab]`（skills、hooks、mcp、plugins）。
- 设置入口：顶栏齿轮、右上角用户菜单、全局导航注册表中的设置模块。
- 能力注册表 `lib/settings/capabilityRegistry.ts`，以及 MCP、Skills、Plugins、Hooks、Mod、集成面板组。
- Hook 策略页的入口与可达性，以及 Hook 策略服务端路由与表结构的存在性。

范围外（各有专题，本文只记录入口与状态）：
- 成员与身份、角色与权限、租户、个人资料的业务（`components/settings/IdentityPanels.tsx`）。
- 总记忆（`components/settings/MemorySettingsPanel.tsx`）。
- API Key、CLI 配置、凭证管理页面（`settings/api-keys`、`settings/cli-profiles`、`settings/credentials`）。
- 开发者 API 参考与出站 Webhook 的行为（`settings/developer/page.tsx`）。
- 代码托管、自动化触发、Feature Flags 面板的业务数据。
- Hook 策略的运行时准入与执行；插件与 Mod 的运行时隔离；OAuth、mTLS、IP 白名单、Redis 限流等 Phase 2+ 能力。

## 2. 用户与场景

| 角色 | 场景 | 主要入口 |
|---|---|---|
| 个人开发者 | 添加 MCP 服务器、测试 HTTP 连接、启停 Skills 与 Plugins | `/settings?tab=mcp`；`/settings/advanced/mcp` |
| 项目管理员（`tenant_admin` / `project_admin`） | 为 Project 或 Worktree 编辑 Hook 策略，保存草稿、发布或回滚 | `/settings/advanced/hooks` |
| 集成开发者 | 查看 Developer API 参考，登记出站 Webhook | `/settings?tab=dev-mode` 跳转 `/settings/developer` |
| 任意用户 | 通过左导航或深链切换面板 | 顶栏齿轮；右上角用户菜单；`/settings?tab=<id>` |

## 3. 需求

证据路径相对 `frontend/src/`，后端路径相对仓库根。优先级：P1 表示 SRS-SETTINGS-001 或 settings-capabilities-design.md 明确要求；P2 表示原型、占位或扩展能力。

### 3.1 设置总览与能力配置

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-17-01 | 设置页左导航 5 组（账户、组织与权限、集成、扩展、开发者） | P1 | 🟡 部分实现 | 代码为 6 组，多出 `Agent` 组：`app/(app)/settings/page.tsx:58-107`（Agent 组 `:68-72`）；原型为 5 组：`docs/prototypes/6 · 设置-html/Settings.dc.html:206-210` |
| FR-17-02 | 左导航条目对应的面板或跳转目标均可达 | P1 | ✅ 已实现 | 导航 `settings/page.tsx:60-105`；分派 `:216-248`；`LEGACY_TAB_MAP` 的全部取值均已分派 |
| FR-17-03 | 默认面板为 MCP 服务器 | P1 | ✅ 已实现 | `settings/page.tsx:169` |
| FR-17-04 | `?tab=` 深链，旧 tab 名映射到对应面板 | P1 | 🟡 部分实现 | 映射表 `settings/page.tsx:138-160`；URL 读取 `:164-168`。重定向目标 `permissions` / `members` / `workspace` / `integrations`（`lib/redirects.shim.cjs:62-65`，经 `frontend/next.config.js:39-45` 生效）与 `components/run/BranchWorkspace.tsx:210` 的 `tab=integrations` 均不在映射表中，回落到 MCP（`settings/page.tsx:169`） |
| FR-17-05 | 搜索设置 | P2 | ⬜ 未实现 | `settings/page.tsx:211-213` 按钮无 `onClick` |
| FR-17-06 | 开发者模式：导航入口与顶部开关（SRS FR-ST-004） | P1 | 🟡 部分实现 | 入口存在：导航 `settings/page.tsx:101`，经 StubPanel 指向 `/settings/developer`（`:241-246`）；顶部开关无行为（`:208-210`），无全局开发者模式状态 |
| FR-17-07 | 设置页 Hooks 项深链到高级设置 Hooks（SRS FR-ST-005） | P1 | 🟡 部分实现 | 设置页 Hooks 渲染本地面板（`settings/page.tsx:230-231`）；指向 `/settings/advanced/hooks` 的链接仅见 `lib/nav/registry.ts:247` 与 `components/run/RunWorkspaceShell.tsx:231` |
| FR-17-08 | 顶栏设置入口 | P1 | ✅ 已实现 | `components/AppHeader.tsx:127-137`（`settings-gear`） |
| FR-17-09 | 右上角用户菜单设置入口 | P1 | ✅ 已实现 | `components/UserMenu.tsx:351-359` |
| FR-17-10 | 用户菜单"个人资料"入口有效 | P2 | 🟡 部分实现 | 入口指向 `/settings/profile`（`components/UserMenu.tsx:342-344`），637ed19d 中无该路由文件；设置页内 `?tab=profile` 可用（`settings/page.tsx:222-223`） |
| FR-17-11 | MCP 官方目录与"已连接"徽标（SRS FR-ST-002） | P2 | ⬜ 未实现 | `settings/page.tsx:109-116` 的 `MCP_SERVERS` 声明后全文件无引用；面板无目录渲染 |
| FR-17-12 | 设置页 MCP 服务器的本地新增、编辑、删除、启停 | P1 | ✅ 已实现 | `components/settings/McpSettingsPanel.tsx:450-704`；新增弹窗 `:246-443`；删除确认 `:494-497`；启停 `:499-501` |
| FR-17-13 | MCP 测试连接（仅 http；HEAD 失败或 405 回退 GET；超时；认证头） | P2 | ✅ 已实现 | `components/settings/McpSettingsPanel.tsx:186-233`、`:596-610`；stdio 不可测说明 `:678-681` |
| FR-17-14 | 连接类型 stdio / sse / http | P1 | 🟡 部分实现 | 高级面板三种：`app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx:128-130`，类型 `lib/settings/capabilityRegistry.ts:31`；设置页仅 `stdio` / `http`：`components/settings/McpSettingsPanel.tsx:74`、`:342-343` |
| FR-17-15 | 作用域 user / project | P1 | 🟡 部分实现 | 高级面板：`advanced/[tab]/McpSettingsPanel.tsx:161-162`，`capabilityRegistry.ts:32`；设置页 `McpServer` 无 scope 字段：`components/settings/McpSettingsPanel.tsx:76-86` |
| FR-17-16 | 健康徽章与 last_error | P1 | 🟡 部分实现 | 四种徽章渲染：`advanced/[tab]/McpSettingsPanel.tsx:190-197`、`:226-234`；写入仅产生 `unknown`（`capabilityRegistry.ts:89`、`:102`、`:198`），无代码更新 health 或 last_error |
| FR-17-17 | 健康探测（`POST .../check`，202 异步回写） | P2 | ⬜ 未实现 | 端点仅见 `docs/api-design-mcp-capabilities.md` §2；`crates/star-api-rest/src` 无对应路由 |
| FR-17-18 | 能力 REST CRUD（`/api/v1/capabilities/*`） | P2 | ⬜ 未实现 | 设计见 `docs/api-design-mcp-capabilities.md` §2；`git grep` 在 `crates/star-api-rest/src` 中无 `capabilities` 或 `mcp-servers` 路由 |
| FR-17-19 | MCP 启用与停用 | P1 | ✅ 已实现 | 高级：`advanced/[tab]/McpSettingsPanel.tsx:235-248`，`capabilityRegistry.ts:214-221`；设置页：`components/settings/McpSettingsPanel.tsx:619-637` |
| FR-17-20 | MCP 移除需确认 | P1 | 🟡 部分实现 | 设置页有 `window.confirm`（`components/settings/McpSettingsPanel.tsx:494-497`）；高级面板直接删除（`advanced/[tab]/McpSettingsPanel.tsx:252`） |
| FR-17-21 | 连接参数校验（名称、命令或 URL、超时） | P1 | 🟡 部分实现 | 设置页：`components/settings/McpSettingsPanel.tsx:159-174`，超时 1000–60000 `:272-275`；高级面板仅校验名称与命令（`advanced/[tab]/McpSettingsPanel.tsx:53-61`） |
| FR-17-22 | 认证方式 bearer / api_key 与凭据字段 | P2 | 🟡 部分实现 | 设置页有认证字段与密码输入（`components/settings/McpSettingsPanel.tsx:369-401`），服务器对象含 `authValue` 并整体写入 localStorage（`:151`）；高级 `McpServerConfig` 无认证字段（`capabilityRegistry.ts:35-48`） |
| FR-17-23 | Skills 列表、新增、启停、移除 | P1 | ✅ 已实现 | `advanced/[tab]/SkillsPluginsPanels.tsx:31-122` |
| FR-17-24 | Skills 名称、版本、描述、作用域字段 | P1 | ✅ 已实现 | `advanced/[tab]/SkillsPluginsPanels.tsx:66-89`、`:101-106` |
| FR-17-25 | Skills 卸载二次确认 | P2 | 🟡 部分实现 | 设置页 `UninstallConfirmModal`（`components/settings/SkillsSettingsPanel.tsx:542-575`）；高级面板直接删除（`advanced/[tab]/SkillsPluginsPanels.tsx:112-113`） |
| FR-17-26 | Skills 从 URL 或 JSON 安装，configSchema 配置 | P2 | 🟡 部分实现 | 设置页：`components/settings/SkillsSettingsPanel.tsx:160-165`（`fetchSkillFromUrl`）、`:250`（`AddSkillModal`）、`:388`（`ConfigSkillModal`）；高级面板无 |
| FR-17-27 | Skills 官方目录 | P2 | ⬜ 未实现 | `components/settings/SkillsSettingsPanel.tsx:265-266`、`:350-357` 提示未接入，`:374` 提交按钮禁用 |
| FR-17-28 | Plugins 新增、启停、移除与权限 chip | P1 | ✅ 已实现 | `advanced/[tab]/SkillsPluginsPanels.tsx:124-235`；权限 chip `:211-219` |
| FR-17-29 | 权限字符串解析（逗号或空白分隔） | P1 | ✅ 已实现 | `advanced/[tab]/SkillsPluginsPanels.tsx:145`；`advanced/[tab]/CapabilityPanels.test.tsx`（权限解析用例） |
| FR-17-30 | 集成面板组可在 `/settings` 内切换（7 个 tab） | P2 | 🟡 部分实现 | tab 栏 `components/settings/IntegrationPanels.tsx:551-572`；容器以受控 `active` 渲染（`:541-543`），`settings/page.tsx:234-236` 不传 `onActiveChange`，点击 tab 不改变面板 |
| FR-17-31 | `/settings?tab=plugins` 的插件启停 | P2 | 🟡 部分实现 | 本地预览数据，启停仅为 UI 状态，不持久化（`components/settings/IntegrationPanels.tsx:397-431`，说明 `:404`） |
| FR-17-32 | 插件运行时隔离与执行 | P1 | ⬜ 未实现 | 无执行代码；`IntegrationPanels.tsx:404` 注明插件运行时未接入；隔离说明文案仅见未被渲染的 `settings/page.tsx:312-316` |

### 3.2 Hook 与 Mod

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-17-33 | Hook 策略编辑：范围 project / worktree，决策 Deny / RequireHuman / Defer，阶段 | P1 | 🟡 部分实现 | 选项 `advanced/[tab]/HookPolicyPage.tsx:128-138`；范围选择 `:548-560`；默认 Web 构建不可达（见 4.7.6） |
| FR-17-34 | Hook 草稿保存、发布、回滚 | P1 | 🟡 部分实现 | `saveDraft` `:443`、`publishDraft` `:486-495`、`rollback` `:506-519`；按钮 `:616-635`；同 FR-17-33 不可达 |
| FR-17-35 | Hook 策略服务端路由与持久化 | P1 | 🟡 部分实现 | 10 条路由 `crates/star-api-rest/src/group_api/hook_policies.rs:303-342`，合并于 `group_api.rs:827`；权限键 `hook:write` / `hook:publish`（`hook_policies.rs:852-853`）；迁移 `db/migrations/2026-09-30-multica-hook-policy.sql`（表 `:29`、`:75`、`:121`）；目标环境迁移应用状态 ❓ 未核实 |
| FR-17-36 | 设置页 Hooks 本地管理（按类型分组、增改删、启停） | P2 | ✅ 已实现 | `components/settings/HooksSettingsPanel.tsx:514`；存储键 `:53`；诚实说明 `:46` |
| FR-17-37 | Hook 执行 | P1 | ⬜ 未实现 | `components/settings/HooksSettingsPanel.tsx:37-38`、`:566` 注明执行引擎未接入 |
| FR-17-38 | Hook 传输方式配置（SRS §1"策略/传输方式"） | P1 | ⬜ 未实现 | `advanced/[tab]/HookPolicyPage.tsx` 无 transport 相关控件 |
| FR-17-39 | Hook 配置唯一入口为高级设置 → Hooks | P1 | 🟡 部分实现 | 高级入口存在；设置页并存本地 Hooks 面板（`settings/page.tsx:230-231`），与 AGENTS.md 第 0 节"Hook 配置必须沿用既有高级设置 → Hooks"（`AGENTS.md:22`）不符 |
| FR-17-40 | Mod 目录双源（官方 / 自定义）（SRS FR-ST-003） | P1 | ⬜ 未实现 | `components/settings/ModDirectory.tsx` 无源切换；切换按钮仅见未渲染的 `settings/page.tsx:266-284` |
| FR-17-41 | Mod 目录浏览与安装（本地模拟） | P2 | 🟡 部分实现 | `components/settings/ModDirectory.tsx:39-44`、`:128-168`；已安装列表为 `useState`，刷新即丢（`:63`） |
| FR-17-42 | 从文件安装 Mod 与用模板新建 | P2 | 🟡 部分实现 | `ModDirectory.tsx:46-56`、`:72-95`（文件校验）、`:97-116`（模板）；仅本地态 |
| FR-17-43 | Mod 表含启用列 | P2 | ⬜ 未实现 | `ModDirectory.tsx:228-259` 无启用控件 |
| FR-17-44 | Mod 隔离运行时说明与隔离执行 | P1 | ⬜ 未实现 | 说明文案仅见未被渲染的 `settings/page.tsx:312-316`；无 Mod 执行代码 |
| FR-17-45 | Mod 面板的本地预览标注 | P1 | ✅ 已实现 | `components/settings/ModDirectory.tsx:120-125`（"本地目录预览 — 目录服务未接入"） |

### 3.3 高级设置与横切

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-17-46 | 高级设置 tab 导航（skills、hooks、mcp、plugins） | P1 | ✅ 已实现 | `advanced/layout.tsx:37-42`、`:55-74`；`advanced/[tab]/page.tsx:37-41` |
| FR-17-47 | `/settings/advanced` 重定向到 hooks；未知 tab 返回 404 | P2 | ✅ 已实现 | `advanced/page.tsx:11-13`；`advanced/[tab]/page.tsx:41` |
| FR-17-48 | 高级设置面板的未接入或本地持久化标注 | P1 | ⬜ 未实现 | `advanced/[tab]/McpSettingsPanel.tsx`、`SkillsPluginsPanels.tsx` 无此类标注（`git grep` 无命中）；对照设置页 `components/settings/McpSettingsPanel.tsx:692-698` |
| FR-17-49 | 配置数据以浏览器本地持久化，刷新不丢 | P1 | ✅ 已实现 | `lib/settings/capabilityRegistry.ts:161-168`；`components/settings/McpSettingsPanel.tsx:476-479`；`components/settings/SkillsSettingsPanel.tsx:111-118` |
| FR-17-50 | 注册表 API 可替换为后端实现（接口形状对齐设计） | P1 | 🟡 部分实现 | `CapabilityRegistryApi` 仅 add / remove / toggle（`capabilityRegistry.ts:192-268`）；设计要求的 PATCH 未提供（`docs/api-design-mcp-capabilities.md` §2） |
| FR-17-51 | 三语文案与 key 一致 | P1 | ✅ 已实现 | `lib/i18n/zh-CN.ts:766`（`settingsCapabilities`）；`lib/i18n/dictionaryConsistency.test.ts:35-49` |
| FR-17-52 | 面板浮层实色 | P2 | ✅ 已实现 | `components/settings/McpSettingsPanel.tsx:303`（`popover-solid`） |
| FR-17-53 | 窄屏布局 | P2 | ❓ 未核实 | 页内左导航固定 `w-64 shrink-0`（`settings/page.tsx:175`），无断点；需移动视口实测 |
| FR-17-54 | 开发者入口页 `/settings/developer` | P2 | ✅ 已实现 | `app/(app)/settings/developer/page.tsx:12-20`；入口 `settings/page.tsx:101-102`、`:241-246`；注册表 `lib/nav/registry.ts:439-453` |

## 4. 设计

### 4.1 概念模型

设置域由三层组成：设置壳（`/settings` 页内左导航与面板分派）、高级设置壳（`/settings/advanced` 的 tab 导航）、能力实体（MCP 服务器、Skill、Plugin、Hook 策略、Hook 规则、Mod、出站 Webhook）。

同一能力在两个表面各有一份数据：设置页表面（`/settings?tab=`）与高级表面（`/settings/advanced/[tab]`）。两者存储键不同，互不同步。

| 能力 | 设置页表面（`/settings?tab=`） | 高级表面（`/settings/advanced/[tab]`） | 服务端 |
|---|---|---|---|
| MCP 服务器 | `star-mcp-servers`（localStorage） | `star-capability-registry:v1` | 无 |
| Skills | `star-skills`（localStorage） | `star-capability-registry:v1` | 无 |
| Plugins | 集成面板预览，启停不持久 | `star-capability-registry:v1` | 无 |
| Hooks | `star-hooks`：Git 生命周期 hook（pre_commit / pre_push / post_merge / custom） | Worktree 生命周期策略（Project 或 Worktree 范围） | 有（路由与表） |
| Mod | 本地 state，不持久 | 无 | 无 |
| 出站 Webhook | `/settings/developer` 使用 `star.outbound-webhooks.v1`；`?tab=webhooks` 为集成预览 | 无 | 无 |

两种 Hook 概念不同：设置页 Hooks 描述 Git 生命周期钩子（`HooksSettingsPanel.tsx:55`）；高级 Hook 策略描述 Worktree 生命周期阶段（Run admission、archive cleanup，见 `HookPolicyPage.tsx:135-138`）上的准入规则。两者不互相参与决策。

词汇约定：作用域 `user` / `project`（`capabilityRegistry.ts:32`）；启用为布尔值 `enabled`；健康为 `unknown` / `connecting` / `healthy` / `error`（`capabilityRegistry.ts:33`），当前代码仅写入 `unknown`。

### 4.2 数据模型

| 实体 | 字段 | 存储 | 位置 |
|---|---|---|---|
| `McpServerConfig`、`SkillEntry`、`PluginEntry`（高级注册表） | id、name、transport、command、args、env、scope、enabled、health、last_error、updated_at_ms（MCP）；version、description、permissions（Plugin） | localStorage `star-capability-registry:v1` | `lib/settings/capabilityRegistry.ts:35-68`，键 `:76`，seed `:78-136` |
| 设置页 `McpServer` | id、name、url、authType（bearer / api_key / none）、authValue、enabled、transport（stdio / http）、timeoutMs、createdAt | localStorage `star-mcp-servers` | `components/settings/McpSettingsPanel.tsx:76-86`，键 `:70` |
| 设置页 `StarSkill` | id、name、description、version、author、capabilities、configSchema、config、enabled、installedAt | localStorage `star-skills` | `components/settings/SkillsSettingsPanel.tsx:72-86`，键 `:70` |
| 设置页 `HookConfig` | id、name、type、command、args、env、enabled、description、matchPattern | localStorage `star-hooks` | `components/settings/HooksSettingsPanel.tsx:66-76`，键 `:53` |
| `ModEntry` | id、name、type、abilities、version、source（directory / file / template） | 组件 state，不持久 | `components/settings/ModDirectory.tsx:28-35`，state `:63` |
| `OutboundWebhook` | id、name、url、events、active、created_at | localStorage `star.outbound-webhooks.v1` | `app/(app)/settings/developer/page.tsx:27-36`，键 `:36` |
| Hook 策略（服务端） | `multica.hook_policy_set`（SCD2 守卫）、`hook_policy_draft`、`hook_policy_audit_event`（追加式，禁止更新删除） | PostgreSQL，三表启用并强制 RLS | `db/migrations/2026-09-30-multica-hook-policy.sql:29`、`:75`、`:121`；RLS `:260-277`；守卫 `:190`、`:245` |

### 4.3 接口

前端路由：

| 路由 | 文件 | 说明 |
|---|---|---|
| `/settings` | `app/(app)/settings/page.tsx` | 左导航与分派，`?tab=` 在客户端解析 |
| `/settings/advanced` | `app/(app)/settings/advanced/page.tsx:11-13` | 重定向到 `/settings/advanced/hooks` |
| `/settings/advanced/[tab]` | `app/(app)/settings/advanced/[tab]/page.tsx:33-41` | `hooks`、`mcp`、`skills`、`plugins`；其余 `notFound()` |
| `/settings/developer` | `app/(app)/settings/developer/page.tsx` | `?tab=webhooks` 切换（`:74`） |
| `/settings/api-keys`、`/settings/cli-profiles`、`/settings/credentials` | 各自 `page.tsx` | 设置页经 StubPanel 链接前往（`settings/page.tsx:241-246`）；`credentials` 无导航入口 |

`/settings?tab=` 接受的取值（`settings/page.tsx:138-160`）：`profile`、`account`、`team`、`billing`、`apikeys`、`mcp`、`identity`、`permission`、`tenant`、`scm`、`automation`、`mod`、`skills`、`plugins`、`hooks`、`dev-mode`、`debug`、`webhooks`、`flags`、`cli-profiles`、`memory`。其中 `account`、`team`、`billing` 为旧别名。

后端路由（Hook 策略，`crates/star-api-rest/src/group_api/hook_policies.rs:303-342`，挂载于 `group_api.rs:827`）：

| 方法 | 路径 |
|---|---|
| GET | `/api/v1/projects/{project_id}/hook-policy` |
| GET | `/api/v1/projects/{project_id}/hook-events`、`/api/v1/projects/{project_id}/hook-events/summary` |
| GET | `/api/v1/worktrees/{worktree_id}/hook-policy/effective` |
| PUT | `/api/v1/projects/{project_id}/hook-policy/draft`；`/api/v1/worktrees/{worktree_id}/hook-policy/draft` |
| POST | `/api/v1/projects/{project_id}/hook-policy/publish`；`/api/v1/worktrees/{worktree_id}/hook-policy/publish` |
| POST | `/api/v1/projects/{project_id}/hook-policy/rollback`；`/api/v1/worktrees/{worktree_id}/hook-policy/rollback` |

能力 CRUD 无后端路由。客户端调用由 `lib/group/worktreeGroupApi` 提供，页面调用其发布与回滚方法（`HookPolicyPage.tsx:494-519`）。

注册表操作（`lib/settings/capabilityRegistry.ts:192-268`）：`addMcpServer`、`removeMcpServer`、`toggleMcpServer`，以及 Skill、Plugin 的同名三组方法。无 update 方法。

### 4.4 状态机与关键流程

1. 设置页分派：首帧 `rawTab` 为空，解析结果回落到 MCP（`settings/page.tsx:164-169`）；挂载后读取 `window.location.search` 并切换（`:165-168`）。首帧固定显示 MCP，深链会先闪现 MCP 面板。
2. MCP 测试连接（设置页）：空闲 → 运行中（`testSlots`，`components/settings/McpSettingsPanel.tsx:503-507`）→ 成功（延迟与状态码）或失败（错误文本，`:658-676`）。stdio 传输直接禁用（`:600`）。
3. Hook 策略编辑（高级）：未接入（`HookPolicyPage.tsx:530-541`）→ 已加载 → 本地修改（`hasUnsavedChanges`）→ 保存草稿（`saveDraft` `:443`）→ 发布（`publishDraft` `:486`；需 `tenant_admin` 或 `project_admin`，草稿非 stale，且无未保存修改，`:395-396`、`:617`）→ 回滚到历史版本（`rollback` `:506`，`:635`）。
4. 注册表写入（高级）：构造新状态对象 → `persistRegistry`（`capabilityRegistry.ts:161-168`）→ `emitRegistryChange`（`:157-159`）→ `useSyncExternalStore` 重渲染（`:185-187`）。持久化失败时仅保留内存态，无界面提示（`:165-166`）。

### 4.5 权限与安全

| 项 | 现状 | 证据 |
|---|---|---|
| 设置页与高级本地面板 | 无服务端鉴权，无 scope 权限判断；数据仅在当前浏览器 | `components/settings/*` 各面板；`advanced/[tab]/*` |
| Hook 策略客户端门控 | 发布需角色 `tenant_admin` 或 `project_admin`；编辑需 API 可用且草稿非 stale | `HookPolicyPage.tsx:395-396`、`:617` |
| Hook 策略服务端门控 | 路由按权限键 `hook:read` / `hook:write` / `hook:publish` 与角色检查 | `hook_policies.rs:852-866` |
| Hook 表 | 三表启用并强制 RLS；审计表禁止更新与删除 | `db/migrations/2026-09-30-multica-hook-policy.sql:260-277`、`:245` |
| 能力 scope 权限 | 设计要求：user 变更需 `tenant_admin`，project 变更需 `project_admin`；代码未实现 | `docs/api-design-mcp-capabilities.md` §3；⬜ 未实现，对应 FR-17-18 |
| MCP 凭据 | 输入为 password 类型（`components/settings/McpSettingsPanel.tsx:388`），但随服务器对象明文写入 localStorage（`:151`），界面说明"仅本地持久化"（`:399`） | 见第 6 节差距 |
| 插件 permissions | 仅解析与展示（`advanced/[tab]/SkillsPluginsPanels.tsx:145`、`:211-219`），无执行或强制代码 | FR-17-32 |
| Hook 安全基线文案 | 页面声明 Rust 内置安全基线不可关闭、删除或放宽（`HookPolicyPage.tsx:546`） | 文案；执行侧证据不在本功能内 |

### 4.6 依赖关系

- `/settings` 依赖：`lib/i18n`（`settingsCapabilities`）、`components/status/StatusNotice`（MCP 面板三态）、`lib/store`（仅 `memory` 面板的 `projects`，`settings/page.tsx:229`）、`components/settings/*` 各面板、`lib/nav/registry.ts` 的设置模块定义。
- 高级 Hook 页依赖：`lib/group/groupProjection.tsx` 的 `useWorktreeGroupApi`（`:104-106`）与 `WorktreeGroupApiProvider`（`:64-102`）；该 Provider 只在 `Providers` 收到 `runDirectorySession` 时挂载（`app/providers.tsx:52-60`）；根布局调用 `<Providers>` 时未传入（`app/layout.tsx:75`）。
- 服务端依赖：`crates/star-api-rest` 的 group_api 路由（`group_api.rs:827`）与 Hook 策略迁移。
- 无后端依赖：`capabilityRegistry`、设置页 MCP / Skills / Hooks / Mod 面板。
- 导航依赖：全局 `Sidebar`（`app/layout.tsx:76-79`）按 `lib/nav/registry.ts:684-711` 的分组渲染设置模块（`settings` 在 `admin-org`，`advanced-settings` 在 `admin-dev`）。该全局侧栏与设置页内左导航是两套结构。

### 4.7 页面设计

共同布局（本功能全部页面适用）：顶栏为 `components/AppHeader.tsx`（`AppShell.tsx:48-50`）；全局左侧栏为 `Sidebar`（`app/layout.tsx:76-79`）；主区为 `AppShell` 的 `main`（`AppShell.tsx:61-73`）；右侧栏为 `RightChatRail`（`AppShell.tsx:77`）；桌面底栏不存在。`/settings` 前缀属于全高工作区（`app/(app)/layout.tsx:32`），`main` 使用 `overflow-y-hidden`（`AppShell.tsx:66`）。

#### 4.7.1 /settings

- 文件：`app/(app)/settings/page.tsx`（默认导出 `:321-323`）。
- 布局：页内左导航 `aside`（`settings/page.tsx:175`）位于主区内；页内主区 `main`（`:201`）；头部行（`:203-214`）含"开发者模式"与"搜索设置"两个无行为按钮。
- 组件树：`SettingsPage`（`:321`）→ `SettingsInner`（`:162`）→ 左导航（`:175-198`）、头部行（`:203-214`）、分派：`McpSettingsPanel`（`components/settings/McpSettingsPanel.tsx:450`）、`ModDirectory`（`components/settings/ModDirectory.tsx:62`）、`IdentityPanel` / `ProfilePanel` / `PermissionPanel` / `TenantPanel`（`components/settings/IdentityPanels.tsx:95`、`:177`、`:335`、`:419`）、`MemorySettingsPanel`（`components/settings/MemorySettingsPanel.tsx:79`）、`HooksSettingsPanel`（`components/settings/HooksSettingsPanel.tsx:514`）、`SkillsSettingsPanel`（`components/settings/SkillsSettingsPanel.tsx:581`）、`IntegrationPanels`（`components/settings/IntegrationPanels.tsx:540`）、`StubPanel`（`settings/page.tsx:120`）。

状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 点击导航项 | 导航高亮，对应面板 | ✅ 已实现（`settings/page.tsx:182-192`、`:216-248`） |
| 加载（首帧） | 首次渲染，`?tab=` 尚未解析 | 固定显示 MCP 面板 | 🟡 部分实现（`:164-169`） |
| 加载（MCP 本地读取） | 面板挂载 | `StatusNotice` loading | ✅ 已实现（`components/settings/McpSettingsPanel.tsx:451`、`:545-547`） |
| 空 | 无服务器、技能或 Hook | 空态文案 | ✅ 已实现（`McpSettingsPanel.tsx:555-561`；`ModDirectory.tsx:230-233`） |
| 错误（本地存储读取） | `localStorage` 抛错 | `StatusNotice` failed | ✅ 已实现（`McpSettingsPanel.tsx:462-468`、`:548-554`） |
| 错误（写入失败） | `localStorage` 写入失败 | 持久化失败提示 | ✅ 已实现（`McpSettingsPanel.tsx:478`、`:690`） |
| 未知 tab | `?tab=` 不在映射表 | 回落到 MCP，无提示 | 🟡 部分实现（`settings/page.tsx:169`） |
| 未授权 | — | 页面不做权限判断 | ⬜ 未实现（无对应门控） |
| 未接入 | 后端能力 | 诚实徽标或禁用按钮：MCP `McpSettingsPanel.tsx:692-698`；Skills `SkillsSettingsPanel.tsx:704-711`；Hooks `HooksSettingsPanel.tsx:46`；Mod `ModDirectory.tsx:120-125`；集成 `IntegrationPanels.tsx:150-170` | 🟡 部分实现（高级面板无） |
| 禁用 | stdio 测试连接；停用的服务器 | 按钮禁用；"已禁用"状态 | ✅ 已实现（`McpSettingsPanel.tsx:600`、`:593`、`:678-681`） |

交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 点击左导航项 | — | 整页跳转到 `/settings?tab=<id>`（普通 `<a>`） | 无 | ✅ 已实现（`settings/page.tsx:182-192`） |
| 点击"搜索设置" | — | 无动作 | 无 | ⬜ 未实现（`:211-213`） |
| 点击头部"开发者模式" | — | 无动作 | 无 | ⬜ 未实现（`:208-210`） |
| MCP 新增、编辑、删除、启停、测试连接 | 名称与 URL 合法；删除需确认；测试仅 http | 写入或展示结果 | `star-mcp-servers` | ✅ 已实现（`McpSettingsPanel.tsx:481-507`、`:509-517`） |
| Skills 从 URL / JSON 安装、卸载、配置 | 合法 URL 或 JSON；卸载需确认 | 本地写入 | `star-skills` | ✅ 已实现（`SkillsSettingsPanel.tsx:160-165`、`:542-575`） |
| Skills 官方目录 | — | 提示未接入，提交禁用 | 无 | ⬜ 未实现（`:265-266`、`:374`） |
| Hooks 新增、编辑、删除、启停 | 名称与命令必填；删除需确认 | 本地写入 | `star-hooks` | ✅ 已实现（`HooksSettingsPanel.tsx:514`） |
| Mod 安装、从文件安装、模板新建 | 文件合法；名称非空 | 仅加入本地列表，刷新即丢 | 组件 state | 🟡 部分实现（`ModDirectory.tsx:63`、`:72-116`、`:150-163`） |
| 集成面板 tab 切换 | — | 面板不切换 | 无 | 🟡 部分实现（`settings/page.tsx:234-236`；`IntegrationPanels.tsx:545-548`） |
| 插件启停（集成预览） | — | 本地 UI 状态，不持久 | 无 | 🟡 部分实现（`IntegrationPanels.tsx:404`、`:420-425`） |
| 开发者模式入口 | — | 经 StubPanel 链接前往 `/settings/developer` | 无 | ✅ 已实现（`settings/page.tsx:241-246`） |

数据：`useStore` 仅为 `memory` 面板提供 `projects`（`settings/page.tsx:229`）；`IdentityPanels` 读取 store；MCP、Skills、Hooks 读写各自 localStorage 键；集成面板与 Mod 目录使用常量（`components/settings/IntegrationPanels.tsx:103-146`，`ModDirectory.tsx:39-44`）；无 REST 调用。

与文档设计的差距：
- 左导航为 6 组，SRS 与原型为 5 组（FR-17-01）。
- 深链映射缺 `permissions`、`members`、`workspace`、`integrations` 四个取值，重定向落回 MCP（FR-17-04）。
- 设置页 Hooks、MCP、Skills、Plugins 与高级设置同名面板并存（FR-17-39、FR-17-14）。
- 开发者模式与搜索设置按钮无行为（FR-17-05、FR-17-06）。
- MCP 官方目录未渲染（FR-17-11）。

#### 4.7.2 /settings/advanced

- 文件：`app/(app)/settings/advanced/layout.tsx`（`AdvancedSettingsLayout`，`:33`）；`app/(app)/settings/advanced/page.tsx`（重定向，`:11-13`）。
- 布局：继承 4.7 共同布局；内容区为 `section`（`advanced/layout.tsx:45`），无自身滚动容器。`main` 为 `overflow-y-hidden`（`AppShell.tsx:66`），长内容是否被裁切需浏览器实测（❓ 未核实）。
- 组件树：`AdvancedSettingsLayout`（`:33`）→ 标题区（`:46-53`）、tab 导航（`:55-74`）、`children`（`:75`）→ `AdvancedSettingsTabPage`（`advanced/[tab]/page.tsx:33`）。

状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 访问 tab 路由 | tab 高亮（`aria-current`，`advanced/layout.tsx:62`）与对应面板 | ✅ 已实现 |
| 重定向 | 访问 `/settings/advanced` | 跳转 `/settings/advanced/hooks` | ✅ 已实现（`advanced/page.tsx:12`） |
| 404 | 未知 tab | `notFound()` | ✅ 已实现（`advanced/[tab]/page.tsx:41`） |
| 错误 | 面板内部错误 | 布局不处理，由各面板自行呈现 | 🟡 部分实现 |

交互：tab 链接（`advanced/layout.tsx:59-71`）；返回设置页链接（`:52`）。

数据：布局标签来自 `t.navModules["advanced-settings"]`（`:36`），tab 列表为静态数组（`:37-42`）。

与文档设计的差距：tab 顺序为 Skills、Hooks、MCP、Plugins（`:37-42`），与设计 §2 的 `hooks / mcp / skills / plugins` 不同，设计未规定顺序，此项仅为记录。

#### 4.7.3 /settings/advanced/mcp

- 文件：`app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx`（`McpSettingsPanel`，`:38`）。
- 布局：继承 4.7.2。
- 组件树：`McpSettingsPanel`（`:38`）→ 头部卡片（`:80-102`）→ 新增表单（`:104-175`）→ 列表（`:177-185`）→ `McpServerRow`（`:199`）。

状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 正常 | 有服务器 | 列表行，含传输与作用域徽章 | ✅ 已实现（`:180-184`、`:199-234`） |
| 空 | 无服务器 | `mcp-empty` 文案 | ✅ 已实现（`:177-178`） |
| 加载 | — | 同步读取模块状态，无加载态（`capabilityRegistry.ts:154`） | 不适用 |
| 错误 | localStorage 异常 | 静默降级为内存态，无界面提示（`capabilityRegistry.ts:149-150`、`:165-166`） | 🟡 部分实现 |
| 未接入 | — | 无标注 | ⬜ 未实现（FR-17-48） |
| 禁用 | 停用 | 行降透明度（`:207`）；开关 `aria-pressed`（`:239`） | ✅ 已实现 |

交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 新增 | 名称与命令非空（`:53-61`） | `addMcpServer`，`env` 固定为空对象（`:67`） | `star-capability-registry:v1` | ✅ 已实现 |
| 启停 | — | 切换 `enabled`（`capabilityRegistry.ts:214-221`） | 同上 | ✅ 已实现（`:235-248`） |
| 移除 | — | 直接删除，无确认（`:252`） | 同上 | 🟡 部分实现 |
| 测试连接 | — | 无 | — | ⬜ 未实现（设计 §4 未列出，见 FR-17-13 对照） |

数据：`useCapabilityRegistry`（`:40`）。健康字段始终为 `unknown`（`capabilityRegistry.ts:198`）。

与文档设计的差距：健康徽章无写入路径（FR-17-16）；无未接入标注（FR-17-48）；移除无确认（FR-17-20）；认证字段缺失（FR-17-22）。

#### 4.7.4 /settings/advanced/skills

- 文件：`app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx`（`SkillsSettingsPanel`，`:31`）。
- 布局：继承 4.7.2。
- 组件树：`SkillsSettingsPanel`（`:31`）→ 头部（`:52-64`）→ 新增表单（`:66-89`）→ 列表（`:91-119`）。

状态矩阵：正常（`:94-118`）✅ 已实现；空（`:91-92`）✅ 已实现；错误（静默降级，同 registry）🟡 部分实现；未接入 ⬜ 未实现；禁用（`:96` 降透明度）✅ 已实现。

交互：新增（`:41-48`，仅校验名称，`:43-44`）✅ 已实现；启停（`:108-111`）✅ 已实现；移除（`:112-115`，无确认）🟡 部分实现。

数据：`registry.skills`（`capabilityRegistry.ts:50-57`）。

与文档设计的差距：无 URL 或 JSON 安装（FR-17-26）；无配置表单（FR-17-26）；无官方目录（FR-17-27）；无卸载确认（FR-17-25）。

#### 4.7.5 /settings/advanced/plugins

- 文件：`app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx`（`PluginsSettingsPanel`，`:124`）。
- 布局：继承 4.7.2。
- 组件树：`PluginsSettingsPanel`（`:124`）→ 头部（`:153-165`）→ 新增表单（`:167-193`）→ 列表（`:195-232`），权限 chip（`:211-219`）。

状态矩阵：正常（`:198-231`）✅ 已实现；空（`:195-196`）✅ 已实现；错误（静默降级）🟡 部分实现；未接入 ⬜ 未实现；禁用（`:200` 降透明度）✅ 已实现。

交互：新增（`:135-149`；仅校验名称；权限按逗号或空白分隔，`:145`）✅ 已实现；启停（`:221-224`）✅ 已实现；移除（`:225-228`，无确认）🟡 部分实现。

数据：`registry.plugins`（`capabilityRegistry.ts:59-68`）。默认作用域为 `project`（`:132`），Skills 默认为 `user`（`:38`）。

与文档设计的差距：权限不强制（FR-17-32）；无运行时（FR-17-32）；无未接入标注（FR-17-48）。

#### 4.7.6 /settings/advanced/hooks

- 文件：`app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx`（默认导出）；`[tab]/page.tsx:37` 分派。
- 布局：继承 4.7.2。
- 组件树：`HookPolicyPage` → 未接入分支（`:530-541`）或编辑器（`:543`）：头部与范围选择（`:545-560`）、`HookExecutionEventPanel`（`:650`）、`HookRuleEditor`（`:839`）、`HookConditionEditor`（`:868`）。API 客户端来自 `useWorktreeGroupApi()`（`:147`）。

状态矩阵：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 未接入（宿主会话） | `useWorktreeGroupApi()` 为 null | `hook-auth-required`：说明需要宿主认证会话，不显示样例，不允许保存或发布（`:530-541`） | ✅ 已实现；默认 Web 构建下为唯一可达状态（见下） |
| 正常 | 有 Provider 与会话 | 策略编辑器与事件面板 | 🟡 部分实现（默认 Web 构建不可达） |
| 错误 | API 调用失败 | 错误文本（`:524`） | ✅ 已实现（代码路径存在，同样不可达） |
| 过期草稿 | `policy.draft.is_stale` | 发布禁用（`:395-396`、`:617`） | ✅ 已实现（代码路径） |
| 未授权角色 | 非 `tenant_admin` / `project_admin` | 发布禁用，提示需要角色（`:617`） | ✅ 已实现（代码路径） |
| 禁用 | 忙碌或有未保存修改 | 保存、发布、回滚禁用（`:616-635`） | ✅ 已实现（代码路径） |

默认 Web 构建的可达性：根布局渲染 `<Providers>` 时未传入 `runDirectorySession`（`app/layout.tsx:75`），`Providers` 因此不挂载 `WorktreeGroupApiProvider`（`app/providers.tsx:52-60`），`useWorktreeGroupApi()` 返回 null（`lib/group/groupProjection.tsx:104-106`），页面只能显示未接入态。

交互：

| 操作 | 前置条件 | 结果 | 接口或存储 | 现状 |
|---|---|---|---|---|
| 切换策略范围 project / worktree | 无未保存修改（`:549`） | 切换范围 | — | 🟡 部分实现（同上不可达） |
| 保存草稿 | `canEdit`（`:396`） | `saveDraft`（`:443-484`） | `PUT .../hook-policy/draft` | 🟡 部分实现 |
| 发布 | `canPublish`、草稿非 stale、无未保存（`:617`） | `publishDraft`（`:486-504`） | `POST .../publish` | 🟡 部分实现 |
| 回滚到历史版本 | `canPublish`、无未保存（`:635`） | `rollback`（`:506-528`） | `POST .../rollback` | 🟡 部分实现 |
| 深链定位 Worktree | URL 参数（`parseHookDeepLinkHints`，`:1054`） | 经分页目录校验后定位 | 目录读取 | 🟡 部分实现（代码与 `HookPolicyPage.test.tsx` 深链用例存在，默认 Web 构建不可达） |

数据：Project 与 Worktree 目录、策略文档与草稿、事件列表与汇总，均来自 `WorktreeGroupApiClient`（`HookPolicyPage.tsx:147`，`:494-519`）。

与文档设计的差距：默认 Web 构建不可达（FR-17-33、FR-17-34）；无传输方式（FR-17-38）；与设置页 Hooks 并存（FR-17-39）。

#### 4.7.7 /settings/developer（入口关系）

- 文件：`app/(app)/settings/developer/page.tsx`（`DeveloperSettingsInner` `:72`、`ApiTab` `:118`、`WebhooksTab` `:175`）。
- 布局：继承 4.7 共同布局。
- 与本功能的关系：仅入口。页面内 Developer API 参考与出站 Webhook 的行为由开发者 API 专题承担（637ed19d 中无对应专题文档，见第 8 节）。
- 入口：导航 `settings/page.tsx:101-102`；注册表 `lib/nav/registry.ts:439-453`（`/settings/developer` 与 `/settings/developer?tab=webhooks`）。
- 差距：webhook 测试仅回显 demo 文本（`developer/page.tsx:221`），无网络投递；存储为 localStorage（`:36`）。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 设置页壳与分派 | `app/(app)/settings/page.tsx` | 🟡 部分实现 | 分派与深链完整；映射缺口见 FR-17-04 |
| 高级设置壳与路由 | `app/(app)/settings/advanced/layout.tsx`、`page.tsx`、`[tab]/page.tsx` | ✅ 已实现 | `advanced/layout.tsx:33-75`；`advanced/page.tsx:11-13`；`[tab]/page.tsx:33-41` |
| 能力注册表 | `lib/settings/capabilityRegistry.ts` | 🟡 部分实现 | 仅 localStorage；无 update；健康不写入（`:192-268`） |
| 高级 MCP 面板 | `app/(app)/settings/advanced/[tab]/McpSettingsPanel.tsx` | ✅ 已实现 | 本地 CRUD 与启停；无测试连接、无确认、无未接入标注 |
| 高级 Skills、Plugins 面板 | `app/(app)/settings/advanced/[tab]/SkillsPluginsPanels.tsx` | ✅ 已实现 | 本地 CRUD 与启停；无确认、无安装流程 |
| 设置页 MCP 面板 | `components/settings/McpSettingsPanel.tsx` | ✅ 已实现 | 本地 CRUD、测试连接（仅 http）、校验；存储键与高级面板不同 |
| 设置页 Skills 面板 | `components/settings/SkillsSettingsPanel.tsx` | 🟡 部分实现 | 本地安装、配置、卸载确认已实现；官方目录 ⬜ |
| 设置页 Hooks 面板 | `components/settings/HooksSettingsPanel.tsx` | 🟡 部分实现 | 本地 Git hook 配置；执行 ⬜；与高级 Hooks 并存 |
| Mod 目录 | `components/settings/ModDirectory.tsx` | 🟡 部分实现 | 本地模拟；双源与启用列 ⬜ |
| 集成面板组 | `components/settings/IntegrationPanels.tsx` | 🟡 部分实现 | 静态预览与未接入徽标；容器 tab 在设置页不可切换 |
| Hook 策略页 | `app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx` | 🟡 部分实现 | 编辑、发布、回滚代码完整；默认 Web 构建不可达 |
| Hook 策略路由 | `crates/star-api-rest/src/group_api/hook_policies.rs`、`group_api.rs:827` | 🟡 部分实现 | 10 条路由已挂载；运行时执行与准入消费 ❓ 未核实 |
| Hook 策略表 | `db/migrations/2026-09-30-multica-hook-policy.sql` | 🟡 部分实现 | 表、守卫、RLS 已定义；目标环境应用状态 ❓ 未核实 |
| 能力 REST | `crates/star-api-rest/src` | ⬜ 未实现 | 无 capabilities 路由 |
| 设置入口（顶栏、用户菜单） | `components/AppHeader.tsx:127-137`；`components/UserMenu.tsx:351-359` | ✅ 已实现 | 齿轮与菜单项指向 `/settings` |
| 旧路径重定向 | `frontend/next.config.js:39-45`；`lib/redirects.shim.cjs:62-65`；`lib/redirects.ts:124-144` | 🟡 部分实现 | 重定向已接入；目标 tab 未映射 |
| 导航注册 | `lib/nav/registry.ts:231-253`、`:684-711`、`:801-803` | ✅ 已实现 | 设置模块与分组定义 |
| 三语文案 | `lib/i18n/zh-CN.ts:766` 及 en、ja 对应段 | ✅ 已实现 | 同构测试 `dictionaryConsistency.test.ts:35-49` |
| 开发者入口页 | `app/(app)/settings/developer/page.tsx` | ✅ 已实现 | 入口可达；内容归开发者专题 |
| 未引用代码 | `settings/page.tsx:109-116`（`MCP_SERVERS`）、`:260-319`（`ModPanel`） | 🟡 部分实现 | 声明存在但无引用；`ModPanel` 内含双源、隔离说明与"已连接"徽标，未被渲染 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 同一能力两套存储（`star-mcp-servers` 与 `star-capability-registry:v1` 等）（FR-17-12、FR-17-14） | 同一 MCP 服务器在两处显示不同数据 | 选定高级设置为权威表面，迁移或停用设置页的同类面板，统一存储键 |
| Hook 配置存在两个入口（设置页 `star-hooks` 与高级策略）（FR-17-39） | 与 AGENTS.md 第 0 节"Hook 配置必须沿用既有高级设置 → Hooks"（`AGENTS.md:22`）不符 | 设置页 Hooks 项改为深链 `/settings/advanced/hooks`，并停用本地 Hooks 面板 |
| 默认 Web 构建下 Hook 策略页恒为未接入态（`app/layout.tsx:75`，`app/providers.tsx:52-60`）（FR-17-33） | 服务端策略能力对最终用户不可达 | 由宿主或应用根提供会话代际（`runDirectorySession`），或为 Web 构建明确说明未接入并保留入口 |
| 旧路径目标 tab 未映射（`permissions`、`members`、`workspace`、`integrations`）（FR-17-04）；`BranchWorkspace.tsx:210` 同样指向 `integrations` | `/permission`、`/identity`、`/tenant`、`/integration` 及分支页入口落回 MCP 面板 | 按设计确定每个目标的面板，在 `LEGACY_TAB_MAP` 补映射；同步修正 `app/(app)/__tests__/panels.test.tsx:162-167`（该用例以 `permissions` 作为未实现样例） |
| 用户菜单"个人资料"指向无路由的 `/settings/profile`（FR-17-10） | 点击进入 404 | 改指 `/settings?tab=profile`，或新增路由 |
| 集成面板组 tab 在设置页不可切换（FR-17-30） | 7 个 tab 中仅当前面板可见，Skills 与 Hooks 预览不可达 | 将 `onActiveChange` 与 `?tab=` 同步，或改为独立分派 |
| SRS 状态与代码不符：FR-ST-002、FR-ST-003 标为已实现，FR-ST-001 写为 5 组（FR-17-01、FR-17-11、FR-17-40） | 文档状态失真 | 按本文状态更新 SRS-SETTINGS-001；删除或接入未引用代码 `MCP_SERVERS`、`ModPanel` |
| 开发者模式开关与搜索设置按钮无行为（FR-17-05、FR-17-06） | 控件误导用户 | 接入或移除 |
| 高级面板无未接入或本地持久化标注；Skills、Plugins、MCP 移除无确认（FR-17-20、FR-17-25、FR-17-48） | 与"未接通功能须显示未接入状态"（AGENTS.md 第 0 节）不符 | 复用设置页的诚实徽标与 `window.confirm` 或确认弹窗 |
| 健康状态从不更新，无探测（FR-17-16、FR-17-17） | 健康徽章恒为 `unknown` | 实现 `POST .../check` 路由，或在 UI 标注未接入 |
| 设置页 MCP 凭据明文写入 localStorage（FR-17-22） | 本地存储泄露面 | 不持久化 secret，或接入受控凭据存储；至少在界面说明泄露面 |
| 两侧能力字段不对称：设置页无 scope，高级面板无 URL 校验与测试连接（FR-17-15、FR-17-21） | 同一能力的配置能力不一致 | 统一字段模型与校验 |
| 设置页 `?tab=` 首帧固定 MCP（`settings/page.tsx:164-169`） | 深链先闪现 MCP 面板 | 首帧读取 URL（如 `useSearchParams` 加 Suspense，同 `developer/page.tsx:64-74` 的模式） |
| `useStore` 在 JSX 三元分支内调用（`settings/page.tsx:229`） | ❓ 未核实：hook 调用次数随 tab 变化，可能触发 React hooks 顺序告警 | 将 `useStore` 提到组件顶层；浏览器内切换 `?tab=memory` 验证 |
| 高级设置 `main` 为 `overflow-y-hidden` 且内容区无滚动容器（`AppShell.tsx:66`、`advanced/layout.tsx:45`） | ❓ 未核实：长列表或 Hook 编辑器可能被裁切 | 浏览器实测；必要时为高级设置内容区增加滚动容器 |
| 设置页左导航固定宽度（`settings/page.tsx:175`） | ❓ 未核实：窄屏下挤压主区 | 移动视口实测后增加断点折叠 |
| 文案陈旧：`lib/i18n/en.ts:448` 的 `/settings` 副标题仍写 5 tabs；`UserMenu.tsx:227` 的 CLI 数量为常量 6 | 信息不准 | 改为数据来源，或删除常量 |
| 插件运行时与 Mod 隔离未实现（FR-17-32、FR-17-44） | 隔离边界仅为文案，无执行约束 | 属于后续 capability gateway 与隔离运行时，超出本功能范围 |
| Hook 传输方式未定义（FR-17-38） | SRS §1 要求未覆盖 | 按 SRS 定义传输字段，或更新 SRS |

## 7. 验收与测试

| 测试文件 | 用例数 | 断言范围 |
|---|---|---|
| `app/(app)/settings/advanced/[tab]/CapabilityPanels.test.tsx` | 7 | 高级 MCP 列表与徽章、新增、空名拒绝、启停与移除；Skills 列表、新增、移除；Plugins 权限解析与新增；注册表写入 localStorage |
| `app/(app)/settings/advanced/[tab]/layout.test.tsx` | 1 | 高级 tab 导航的四个链接与 `href`；`advanced-settings` 位于 `admin-dev` 组 |
| `app/(app)/settings/advanced/[tab]/HookPolicyPage.test.tsx` | 11 | 无会话不显示样例；会话过期清空；授权事件；准入阶段启用条件；离开保护；深链授权与分页上限 |
| `components/settings/McpSettingsPanel.test.tsx` | 22 | localStorage 往返与损坏数据；加载、空、错误态；新增、编辑、校验；认证字段显示；启停；删除确认；测试连接成功、HEAD 回退 GET、失败、超时、stdio 拒绝；URL 校验 |
| `components/settings/SkillsSettingsPanel.test.tsx` | 12 | 空态与诚实徽标；列表；JSON 与 URL 安装；官方目录占位；启停；configSchema 表单与原始 JSON；卸载确认；损坏数据自愈 |
| `components/settings/HooksSettingsPanel.test.tsx` | 14 | 空态；按类型分组；启停；增改删；配置弹窗；删除确认；读写与 storage 事件刷新 |
| `components/settings/ModDirectory.test.tsx` | 8 | 本地预览徽标；目录安装；文件安装成功与失败；模板新建与按钮禁用 |
| `components/settings/IntegrationPanels.test.tsx` | 15 | 容器默认面板与 `active` 切换；各面板列表与禁用按钮；插件与 flag 的本地 toggle |
| `components/settings/MemorySettingsPanel.test.tsx` | 10 | 总记忆面板（不在本功能范围，仅列示） |
| `app/(app)/__tests__/panels.test.tsx`（设置页部分） | 4 | 导航 id 存在；默认 MCP 面板；`?tab=apikeys` 深链；`?tab=billing` 映射到租户；`?tab=permissions` 回落到 MCP |
| `app/(app)/settings/developer/page.test.tsx` | 4 | 默认 API tab；`?tab=webhooks`；webhook CRUD 写入 localStorage；测试仅回显 |
| `lib/i18n/dictionaryConsistency.test.ts` | 2 | en、zh-CN、ja 的扁平 key 集合一致（覆盖 `settingsCapabilities`） |

已验证（静态读取）：上表用例的断言范围与代码路径的对应关系；第 3 节证据行的代码位置。

未验证：
- 测试执行结果在核对中未运行，通过与否标为 ❓ 未核实。
- 设置页无专门的单元测试文件；`settings/page.tsx` 的分派与导航仅由 `panels.test.tsx` 覆盖。
- `frontend/e2e` 中无设置相关用例；`redirects.spec.ts` 覆盖重定向表，不覆盖目标 tab 的落地结果。
- 浏览器内行为未核实：首帧闪烁、hydration、`useStore` 调用顺序、高级设置的长内容裁切、窄屏布局。
- 设计书 `settings-capabilities-test-design.md` §4 列为后续的 API 契约测试、健康探测测试、E2E 测试，在 637ed19d 中均不存在（⬜ 未实现）。
- 数据层用例（设计 §2 所述的 CRUD 与幂等）没有独立的 `capabilityRegistry` 测试文件，仅经组件测试覆盖。

## 8. 来源与裁决

| 源 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-SETTINGS-001.md`（本功能唯一权威） | §1 设计要点（5 组、MCP 目录、Mod 目录、开发者模式、Hooks 在高级设置）；§2 FR-ST-001 至 005；§3 文件映射 | FR-ST-001 写 5 组，代码 6 组；FR-ST-002、FR-ST-003 标为已实现，代码中官方目录、双源、隔离说明未在渲染路径出现；FR-ST-005 的深链不由设置页发起 | 需求内容采纳；状态按代码重定（第 3 节） |
| `docs/frontend/design/settings-capabilities-design.md` | §2 路由与分派；§3 数据层（类型、seed、接口、localStorage 键）；§4 交互；§5 i18n；§6 与弹层的关系；§7 验收 | §1 含对外部产品的概念对照表。AGENTS.md（637ed19d 版 `:22`）禁止以第三方产品对标，因此对照表不写入正文，只采纳中性模型（连接类型、作用域、健康状态）；seed 与代码一致 | 模型、接口与交互采纳；对照表不采纳 |
| `docs/frontend/design/settings-capabilities-test-design.md` | §1 测试分层；§2 七条用例；§4 后续项（API 契约、健康探测、E2E）；§5 门禁 | 数据层独立测试不存在；§5 中的用例数字为设计时的记录，核对时未运行测试 | 用例清单采纳；后续项作为 ⬜ 未实现列出；数字不采纳 |
| `docs/prototypes/6 · 设置-html/README.md` 与 `Settings.dc.html` | 导航 5 组及条目（`Settings.dc.html:206-210`）；Mod 面板文案（官方与自定义、三按钮、已连接计数、隔离说明）；MCP 官方目录描述 | README 说明该文件为参考稿，不是生产代码；原型 5 组，代码 6 组；原型的 Mod 与 MCP 目录在代码中仅见于未引用代码 | 作为设计意图保留，状态按代码标注 |
| `docs/api-design-mcp-capabilities.md` | §1 资源模型；§2 端点（含 PATCH 与 `check`）；§3 权限规则；§4 错误码；§6 降级模式 | 后端无实现；文档自称"v0.1 前端已实装（localStorage 降级）" | 设计意图采纳；端点状态 ⬜ 未实现 |
| `AGENTS.md`（637ed19d 版，`:22`） | 未接通功能须显示未接入状态；Hook 配置必须沿用高级设置 → Hooks；禁止第三方产品对标 | 与设置页现状（FR-17-39、FR-17-48）不符 | 作为验收约束，用于第 3、6 节 |
| `frontend/src/lib/nav/registry.ts` | 设置模块与 `advanced-settings` 模块定义；`NAV_GROUPS` 分组；`SETTINGS_DOMAIN_IDS` | `NAV_GROUPS` 是全局侧栏分组（9 组），与设置页内 5 或 6 组不是同一结构 | 作为导航注册事实采纳 |
| `frontend/next.config.js`、`lib/redirects.shim.cjs`、`lib/redirects.ts` | 旧路径到 `/settings?tab=` 的重定向 | 目标 tab 未映射 | 重定向事实采纳；目标缺口见第 6 节 |
| `docs/features/F17-settings-capabilities.md`（637ed19d 中的同名文件，其核对基线为 1643559f） | 作为核对清单使用 | 部分结论与 637ed19d 不符，例如该文称 Hook 页未调用发布或回滚，而 637ed19d 中 `HookPolicyPage.tsx:486-519`、`:617`、`:635` 存在相应调用 | 不采用其状态与行号；本文所有事实均在 637ed19d 重新核对 |
| 代码（637ed19d） | 第 3 至 7 节全部证据 | — | 现状的唯一依据 |
| 检索命中、未逐条采用：`docs/requirements/SRS-MULTICA-HOOK-001.md`、`docs/design/BD-MULTICA-HOOK-001.md` | Hook 策略专题 | Hook 策略内核归专题文档 | 不展开，本文仅引用路由与表的存在性 |
| 检索命中、未逐条采用：`docs/basic-design.md`、`docs/requirements.md`、`docs/refactor/ui-audit-20261113.md`、`docs/arch-live/02-navigation-model.html`、`docs/prototypes/INDEX.md` | 上位或相关背景 | 本功能以 SRS-SETTINGS-001 为唯一权威，上位文档不覆盖其状态 | 不采用 |
