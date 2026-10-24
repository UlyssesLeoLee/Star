# PHASE-WLD5-HOST-AUTH-BRIDGE-20261023

- 版本：v0.1
- 状态：审计完成；生产 host session owner/provider 未证实，认证 bridge 保持关闭
- 基线：`e5c692772261a72fe07f4211e635d422344de07e`
- 任务 brief：`docs/briefs/WLD5-HOST-AUTH-BRIDGE-20261023.md`
- 唯一改动：本报告

## §0 目的与决策

复核当前 RootLayout、Providers、RunDirectoryHostSession、Group API、可见认证/session owner 与相关测试，判断能否在当前仓库证据上接通 WLD-5 host authentication bridge。

**决策：当前仓库没有可验证且已接入的生产 host session provider。保持 Hooks、Run Directory 与 Group live API disabled；不扩展 manifest，不添加 token issuer/cookie setter，不读写浏览器凭证。** 现有接口是给真实 embedding host 注入认证会话的边界，不是会话实现。Group cookie proxy 与 Rust OAuth2 scaffold 均未提供可供 RootLayout/Providers 使用的完整用户 session owner 契约。

## §1 审计矩阵

| 核查面 | 当前行为与证据 | 判定 |
|---|---|---|
| RootLayout | `frontend/src/app/layout.tsx:63-76` 渲染 `<Providers>` 且不传 `runDirectorySession`。 | 无根布局认证/session provider。 |
| Providers | `frontend/src/app/providers.tsx:31, 51-60` 默认 session 为 `null`；只有显式注入 session 才挂载 `WorktreeGroupApiProvider`。 | 无 session 时 Run 与 Group live client 不创建。 |
| RunDirectoryHostSession | `frontend/src/lib/run/runDirectorySession.tsx:32-39, 49-52, 56-83` 仅声明由真实 host 提供的 generation 与 `getAccessToken` callback；无 token issuer/storage。缺失 session 返回 `status: missing` 且 client/focus 均为空。 | 注入契约，不是生产身份来源。 |
| Run API | `frontend/src/lib/run/runDirectoryApi.ts:489-500, 590-612` 拒绝未授权 origin；每次请求取 token、`credentials: "omit"`、401 清空 session 并阻断、403 返回 scoped forbidden 且保留有效 session。 | 保持 fail-closed。 |
| Group API | `frontend/src/lib/group/worktreeGroupApi.ts:775-812` 每次请求取 token；缺 token 不发网络请求；401 使该 client 失效；403 不触发 session invalidation；每次调用单次 fetch，未实现 mutation replay。 | 已有边界不重复、不放松。 |
| Generation / stale response | `frontend/src/lib/group/groupProjection.tsx:78-96` 将失效状态绑定捕获的 `sessionKey` 并与当前 key 比较；`:183-224` 在 effect cleanup 后停止旧请求写回并 abort poller。`groupProjection.test.tsx:68-74` 覆盖 session key 替换和新 token 请求。 | 现有 generation 防护保留；报告不新增 wiring。 |
| Hook Policy | `frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx:146-190` 使用 Group API client 作授权目录；无 API 时不能满足 project/policy selection gate。测试 `HookPolicyPage.test.tsx:83-142` 覆盖无 host session 不显示 seed policy、session 401 后清除策略 UI。 | 无 session 时策略读取/写入保持关闭。 |
| 同源 Group proxy | `frontend/src/app/api/group/[...path]/route.ts:106-124` 读取配置名 cookie（默认 `star_access_token`），缺失时 401；`:126-177, 197-218` 校验上游 HTTPS/路径并转发 bearer。路由没有签发/刷新 cookie 的逻辑；对 `frontend`、`crates`、`server` 源码的定向搜索仅发现该 cookie reader。Group client 默认请求 `/api/v1/...` 且总要求 callback token（`worktreeGroupApi.ts:781-804`），当前 Providers 未配置其 proxy 路径或 cookie 模式。 | 这是 cookie-to-bearer proxy，不是已验证的 session provider；不能据此启用。 |
| Rust OAuth/JWT | `crates/star-api-rest/src/auth/oauth/handlers.rs:67-102` 的 authorize 明确未实现用户登录/consent，创建 code 时 `user_id = Uuid::nil()`；`:279-295` 的 client secret 校验是 placeholder；`:507-516` introspect 返回 501。`crates/star-api-rest/src/lib.rs:185-209` 定义独立 OAuth router builder；`crates/**/*.rs` 定向搜索只发现定义和测试，未见 runtime caller。`middleware/auth.rs:18-31` 明确是 pass-through stub。 | 有 OAuth/JWT scaffold 和验签代码，但没有可用的真实用户 session owner 或生产接线证据。 |

### Owner 判定与接入选项

本次未找到可以负责登录、用户/tenant 绑定、token 更新与撤权通知的生产 owner。当前可见的 `RunDirectoryHostSession` 已提供最小 callback 注入点；若 embedding host 的真实认证服务补齐契约，接入应由该 host owner 提供 session object，再由一个 client-side adapter 装配到 `Providers.runDirectorySession`。RootLayout 目前不读取或构造 session。

若选择 host token callback：owner 必须提供每次请求返回当前有效 token 的 callback、非 secret generation、身份/tenant/授权变更和 logout 时的 generation 替换语义、token 过期/刷新/撤权语义，以及允许的 API base URL/trusted origins。缺失或过期 token 返回 `null` 并维持 fail-closed；401 由现有 client 失效；403 维持 scoped denial。当前 `RunAccessTokenProvider` 是 `() => Promise<string | null>`（`frontend/src/lib/run/runDirectoryApi.ts:102`），没有独立 `expires_at` 字段，不能推定 refresh 时序。

若选择同源 BFF：BFF/session owner 必须定义 cookie 的签发、Secure/HttpOnly/SameSite 与路径域策略、过期/轮换/撤权、CSRF 与 origin 检查，以及上游 token 的服务端刷新责任。当前 `/api/group` route 只读取一个 opaque cookie 值并将其作为 bearer 透传；没有 cookie writer/session validation/refresh owner 证据。现有 Group client 仍要求 bearer callback 并设置 `credentials: "omit"`，因此不能仅把 `baseUrl` 改为该 route 就宣称完成 BFF 认证。

| 维度 | Host callback | Same-origin BFF |
|---|---|---|
| 信任边界 | embedding host 的认证运行时把短期 bearer token 暴露给 Star client；请求只发往已授权 origin，浏览器 credentials 不随请求发送。 | 浏览器持有 host/Star session cookie；可信 BFF 负责解析 session、刷新并向上游附加 bearer，浏览器不接触上游 token。 |
| 轮换/过期 | owner 的 callback 每次取当前 token；token 过期/refresh 由 owner 决定；身份/tenant/授权切换必须更新 generation。 | BFF owner 管理服务端 session 与 cookie 轮换；当前 route 没有实现这些行为。 |
| 撤权 | owner 通知 generation/logout；API 401 使 client 失效，403 只拒绝当前 scope。 | BFF owner 撤销 session 并处理上游撤权；当前 route 仅透传上游响应，未证明本地 session 撤销机制。 |
| Origin / CSRF | Run API 使用 `trustedOrigins` allowlist，且 `credentials: omit`；redirect 被拒绝。 | 同源减少跨域暴露但 cookie 需要 CSRF 防护；当前 proxy 对非 GET 校验 exact Origin，GET 不校验 Origin。 |
| 现有验证 | Providers/Run/Group 注入与失效测试覆盖 callback 路径；这些是注入式测试，不是生产登录验证。 | 只发现 route 实现，`frontend/src/app/api/group/` 下未发现 route 专属测试；cookie issuer 与部署配置未验证。 |
| 必需 owner | embedding host 的实际认证/session 服务。 | 实际运行的 Next/BFF session owner 与上游身份 provider。 |

## §2 验证摘要

- 基线核实：worktree branch `codex/wld5-host-auth-bridge-20261023` 的 HEAD 与指定 base 均为 `e5c692772261a72fe07f4211e635d422344de07e`。
- 已定向读取 RootLayout、Providers、Run/Group client/provider、Hook Policy、同源 proxy、Rust OAuth/JWT 与关联测试源码；现有测试只证明注入式行为，不证明生产 token issuer/session owner。
- 未运行测试或构建：本阶段只交付决策报告，任务 brief 指定纯文档门禁。测试证据是对测试源码的核对，不冒称本轮测试通过。
- worktree 根下存在 `.codegraph` 目录，但 `codegraph status` 返回 `Not initialized`；未初始化/重建索引。图谱 coverage 未验证，代码结论来自定向源码核查。
- 文档写入后执行 `git diff --check`，提交前检查通过；未运行测试或构建。

## §3 已知缺口

1. 缺少宿主实际登录/session owner：当前没有用户/tenant 的权威来源、token 生命周期、刷新和 logout/generation 变更契约。
2. Group cookie proxy 的 cookie 创建者、属性、session 校验、刷新、撤权与生产部署配置未证实；本次未发现该 route 的专属测试。
3. Star OAuth2 authorize 尚未做人类用户登录/consent，用户 ID 使用 nil placeholder；`client_credentials` 凭证比较仍是 placeholder，introspect 未实现，OAuth route builder 未见生产调用点。
4. 当前 `RunAccessTokenProvider` 为无参数 callback，没有独立过期元数据；宿主 owner 必须提供有效期/刷新语义，Star 不自行推定。
5. Group generation 测试覆盖 session key 更换和新 token 使用；未发现专门断言“旧 session 的延迟 401 callback 不会影响新 generation”的测试。源码状态按 session key 比较，projection cleanup 阻止旧 effect 写回。本报告不扩展该机制；若后续改动触及这段逻辑，应补针对性测试。
6. 无 session 时 Group live API 不挂载；projection hook 仍可能返回 preview mode。该 preview 不是认证或生产数据证明，必须与 live API disabled 状态区分。

## §4 子代理失败接手清单

本 lane 作为单一 worker 执行，未派生子代理；因此没有子代理 RPC、未提交产物或失败接手项。核查仅基于当前 worktree 的只读源码与测试文件。协调者可按 §1 owner 判定和 §3 缺口复核最终集成边界。

## §5 守门规则核对（17 项）

| # | 守门项 | 本轮证据 / 结果 |
|---:|---|---|
| 1 | 唯一写入目标为指定报告 | 仅创建本文件；提交前用 `git status` / `git diff --name-only` 核对。 |
| 2 | 不扩 manifest、API schema 或生产 auth 配置 | 无此类改动。 |
| 3 | RootLayout 不伪装 authenticated | `layout.tsx:63-76` 未传 session。 |
| 4 | 不发明 token issuer 或 cookie setter | 未新增 issuer/setter；只记录当前 owner 缺口。 |
| 5 | 不读 cookie 或浏览器凭证建立 session | 未作此类运行时读取；静态审阅了已有 route cookie reader。 |
| 6 | 不 mock/seed auth | 未新增 auth mock/seed；已有测试 fixture 不作为生产证据。 |
| 7 | 无 token 时 fail closed | Run/Group client 源码有缺 token 401/阻断行为。 |
| 8 | 401 与 403 保持不同语义 | 401 invalidates；403 保持 scoped denial。 |
| 9 | generation 为非 secret 且身份切换时更新 | 类型注释与 provider keying 维持此契约。 |
| 10 | retired-generation callback 不污染当前 session | Group invalidation state 与 captured key 比较；未改动该边界。 |
| 11 | stale projection 不写回 | effect cleanup 置 inactive 并 abort；未改动。 |
| 12 | failed auth 后不 replay mutation | 单次 request 失败即 throw；本轮未加 retry/replay。 |
| 13 | 仅向允许 origin 发送 bearer | Run URL 校验 allowlist，fetch 使用 `credentials: omit` 和 redirect error。 |
| 14 | 不持久化或日志化 token | 本轮未写入/记录 token；未把测试 token 当真实凭证。 |
| 15 | 不将 Group cookie route 当作完整 session owner | route 无签发/refresh；仍需外部 owner contract。 |
| 16 | 不将 OAuth scaffold 当作生产认证 | nil user、placeholder secret check、501 introspect、stub auth middleware 均已记录。 |
| 17 | Hooks/Run/Group live API 在 owner 缺失时保持关闭 | RootLayout/Providers 默认无 session；Hook Policy 测试覆盖无 session 不显示 seed。 |

通用流程门：遵守 PowerShell-only；不 rebase、merge、push 或 cleanup；提交只包含本报告。`git diff --check` 是本轮唯一要求执行的验证命令。

## §6 签字栏

本节确认报告的审计范围；不代表 production auth 已启用或批准上线。

| 角色 | 记录 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)：当前没有可验证 host session owner，保持 fail-closed。 |
| SRE Lead | 不适用：本报告未更改部署、运行配置或 secret。 |
| 平台 | 不适用：本报告未更改平台/API manifest。 |
| 评审主持 | 🟢 Mavis 接手终审：仅审计报告范围，不构成上线授权。 |
| PM | 不适用：未提供认证 owner 或产品层授权决策。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建 WLD-5 host auth bridge 决策/审计报告；确认 callback wiring 已有但无生产 provider；记录 Group cookie proxy 与 OAuth scaffold 的边界及后续 owner 契约。 | `WLD5-HOST-AUTH-BRIDGE-20261023` brief；base `e5c692772261a72fe07f4211e635d422344de07e` |
