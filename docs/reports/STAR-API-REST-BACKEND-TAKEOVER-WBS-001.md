# STAR-API-REST-BACKEND-TAKEOVER-WBS-001 — REST 层真实业务接入 + 非 Mock 端到端部署 WBS

> **Status**: 🟡 Draft v0.1 — 计划文档,待下游 AI 执行 (本文件不含任何代码改动)
> **Created**: 2026-09-09 (JST)
> **Author**: Claude Code (Sonnet 5), per Ulysses 2026-09-09 发令"把完成后端接管的计划写成 spec,然后更新 handoff,我让下游 AI 完成真正的部署,而不是 mock 版本"
> **本文件性质**: 执行计划 (WBS),不是完成报告 — §2 验证摘要记录的是**当前基线实测**,不是完成断言
> **前置阅读**: `deploy/k3s-local/README.md`(k3s 部署现状)+ `crates/star-api-rest/src/lib.rs`(路由骨架自述)+ `AGENTS.md` §7 #1 #2(domain-* crate / MCP tool 真实化现状)

---

## §0 目的

**用户诉求**(2026-09-09 对话原文): "前端不是应该访问部署的 k3s 服务器就能打得开像正常的 saas 那样吗" → 用户要的是**真实可点击的 SaaS**,不是 mock 数据 + 501 stub 拼出来的空壳。

**当前实测状态**(本 session 逐文件核实,非转述):

| 层 | 现状 | 证据 |
|---|---|---|
| k3s 部署 | 仅 `star-api-rest` 一个服务部署(NodePort 30081),其余 7 个 chart 声明的服务(context/sa/sse/webhook/cache/saga/star-cli/star-ops)只有 lib crate 或 CLI 工具,无 `fn main()`,不适合长驻 Deployment | `deploy/k3s-local/star-api-rest-deploy.yaml:1-6` 注释 |
| REST 路由层 | `crates/star-api-rest/src/routes/` 下 **27 个业务 handler 全部**是 `RestError::not_implemented(...)` 一行 stub,0 个已接真实逻辑;`GET /api/v1/health` 是唯一非 stub 端点 | 本 session 逐文件 `grep -c not_implemented` 实测(见 §1.1) |
| 领域逻辑层 | `star-api-rest/Cargo.toml` 已声明 9 个 domain-* path-dep(Tier 1/2/3)+ `star-webhook` + `star-context`,但 **0 个被 routes/ 下的 handler 调用**——依赖已声明,接线从未发生 | `Cargo.toml` 全文 + `routes/work_items.rs` 全文实读 |
| MCP 工具层(对照组) | `star-mcp` 的 16 个 tool 全部已接 `domain_*::InMemory*Service` 真实业务逻辑(0 硬编码 mock),per `AGENTS.md §7 #2` "16/16 REAL 化 done"(2026-09-05);但 `star-mcp` 只有 `[[bin]]` 无 `[lib]`,其代码**不可被 star-api-rest 当作 library 引用**,只能作为"已验证过的接线范式"抄写,不能 `use star_mcp::...` | `star-mcp/Cargo.toml` 无 `[lib]` 段;16 个 `crates/star-mcp/src/tools/*.rs` 逐文件实读(见 §1.2) |
| 持久化层 | 支撑 REST 路由的 domain-* 服务(work-item/workspace/worktree/search/scm/validation 等)**全部**是 `InMemory*Service`——业务逻辑真实,但状态只存在进程内存,pod 重启/多副本即丢失/不一致。这不是"mock",是"真实逻辑 + 无持久化",两者是不同性质的缺口,不可混为一谈 | `grep -rl "struct InMemory" crates/domain-*/src` 命中 domain-work-item / domain-workspace / domain-worktree / domain-search / domain-scm / domain-validation 等全部相关域 |
| 中间件层 | `AuthLayer` / `RateLimitLayer` / `AuditLayer` 三个中间件均是 **no-op pass-through**——任何请求不经鉴权直接放行,无限流,无审计落库 | `middleware/auth.rs` 全文实读:"现状: pass-through, 不注入 ActorContext" |
| 前端 | Next.js 应用(`frontend/`)**未容器化、无 k8s manifest、未部署到 k3s 任何形式**,仅能通过 `maintenance/start-frontend.ps1` 本地 `npm run dev` 启动;mock/real API 切换靠 `NEXT_PUBLIC_USE_REAL_API` / `NEXT_PUBLIC_API_BASE_URL` 两个 build-time 环境变量(Next.js `NEXT_PUBLIC_*` 前缀变量在 build 阶段被打包进客户端 bundle,**不是** runtime 可注入的 k8s env,这一点极易被下游 AI 搞错,见 §3 已知缺口 #6) | `maintenance/start-frontend.ps1` 全文 + `frontend/src/mocks/real-mode.ts` grep |

**本文件目标**: 给下游 AI 一份可执行、按已验证事实分层的 WBS,把"真正的部署,而不是 mock 版本"拆成 4 个 Phase,每个 Phase 明确"抄哪个文件的哪个范式""改哪个 Cargo.toml""新增哪个依赖"。

**明确不做**(沿用 `star-api-rest/src/lib.rs` 已声明的"不做什么" + 本次新增排除项):
- 不实装 OAuth 2.0 / mTLS(Phase 2+ 评估)
- 不实装 5 域预置 vendor 模板
- 不实装 OpenAPI 3.1 spec 自动生成
- 不处理 H2-EXT / Agent-Runtime / ARG(Agent Relationship Graph)/ TMO 等 `HANDOFF-ST-001.md` 追踪的其他工作流——那是完全独立的工作线,本文件只覆盖 `star-api-rest` + 前端部署这一条线
- 不在本文件里实装代码——本文件是给下游 AI 的计划,执行留给下游 AI

---

## §1 改动矩阵 / 任务完成矩阵

### 1.1 REST 路由 → MCP 工具 → domain crate 完整映射表(本 session 逐文件核实,27 route 全覆盖)

| # | REST 路由 | 文件 | 对应 MCP 工具(范式来源) | 支撑 domain crate | `star-api-rest/Cargo.toml` 是否已声明该 dep |
|---|---|---|---|---|---|
| 1-5 | `GET /work-items`,`GET /work-items/current`,`GET /work-items/{id}`,`POST /work-items`,`PATCH /work-items/{id}` | `work_items.rs` | `search_issues.rs` / `get_current_task.rs` / `get_issue.rs` | `domain-work-item::InMemoryWorkItemService` | ✅ 已声明 |
| 6 | `GET /workspaces/{id}` | `workspaces.rs` | `get_workspace.rs` | `domain-workspace::InMemoryWorkspaceService` | ✅ 已声明 |
| 7-8 | `POST /worktrees`,`GET /worktrees/{id}` | `worktrees.rs` | `create_worktree.rs` / `get_worktree.rs` | `domain-worktree::InMemoryWorktreeService` | ✅ 已声明 |
| 9-12 | `GET /code/search`,`GET /code/symbols/{id}`,`GET /code/symbols/{id}/references`,`GET /code/context` | `code.rs` | `search_code.rs` / `get_symbol.rs` / `find_references.rs` / `get_code_context.rs` | `domain-search::InMemorySearchService` | ❌ **未声明,需新增 path-dep** |
| 13 | `GET /context` | `context.rs` | `get_context.rs`(混用 `domain_search` + `domain_work_item`) | `domain-search` + `domain-work-item` | ❌ `domain-search` 未声明(work-item 已声明) |
| 14 | `POST /merge-requests` | `merge_requests.rs` | `create_merge_request.rs` | `domain-scm::InMemoryScmService` | ❌ **未声明,需新增 path-dep** |
| 15 | `POST /reviews` | `reviews.rs` | `request_review.rs` | `domain-scm::InMemoryScmService` | ❌ 同上(与 #14 共用同一个新 dep) |
| 16 | `GET /pipelines/{id}` | `pipelines.rs` | `get_pipeline_status.rs` | `domain-scm::InMemoryScmService` | ❌ 同上 |
| 17 | `POST /validations` | `validations.rs` | `run_validation.rs` | `domain-validation::InMemoryValidationService` | ❌ **未声明,需新增 path-dep** |
| 18 | `POST /submissions` | `submissions.rs` | `submit.rs` — **注意**: 该 MCP 工具自己的文档注释承认 "step 6-12 简化 mock (per brief §1.5「mock 简化版 OK」)",即便照抄这个范式,submissions 端点也只能做到"部分真实" | `domain-validation::InMemoryValidationService` | ❌ 同 #17 |
| 19-27 | webhooks 9 端点(list/create/get/update/delete endpoint + test + list/get delivery + replay) | `webhooks.rs` | **无对应 MCP 工具**(webhook 管理是 REST 独有端点,per `Cargo.toml` 注释"reuse `DeliveryStore`/`RetryPolicy`") | `star-webhook`(现有 `DeliveryStore` + `RetryPolicy`) | ✅ 已声明,但**无现成范式可抄** — 需要下游 AI 参照 `star-webhook` 自身的 CRUD API 原创接线,工作量与其余 18 条不同质 |

**统计**: 27 条业务路由,17 条(#1-8 + #19-27)已有 Cargo.toml 依赖覆盖,10 条(#9-18)需先给 `star-api-rest/Cargo.toml` 新增 3 个 path-dep(`domain-search`、`domain-scm`、`domain-validation`,三者均已是 workspace 内存在的真实 crate,只是从未被 star-api-rest 引用)。

**mod.rs 自述的"22 路由 (16 MCP 镜像 + 6 webhook)" 与本次实测的 27 路由(18 MCP 镜像 + 9 webhook)不一致** — `lib.rs`/`mod.rs` 的模块级文档注释数字已过期,未跟代码同步,这是一个待修正的文档漂移,已计入 §3 已知缺口 #1,不影响本 WBS 的路由清单(以本表 27 条实测为准)。

### 1.2 4-Phase 任务矩阵

| Phase | 主题 | 前置条件 | 涉及路由数 | 预估复杂度 | 产出 |
|---|---|---|---|---|---|
| **Phase 0** | 基线复核(数字有时效性,下游 AI 执行前必跑) | 无 | — | 极低 | 重新实测 §1.1 表格是否漂移 + `cargo test -p star-api-rest` 当前 pass/fail 基线 |
| **Phase I** | REST handler 接线(17 条已有 dep 覆盖的路由,含 3 个新增 path-dep 后的 10 条) | Phase 0 | 27 | 中(机械抄写既有范式,但 webhook 9 条无范式需原创) | 27 个 handler 从 501 变为真实调用 domain-* / star-webhook,单元测试断言从 `NOT_IMPLEMENTED` 改为真实数据 |
| **Phase II** | 持久化决策 + (可选)最小实装 | Phase I 完成或部分完成 | — | 高(可推迟) | 明确拍板:维持 in-memory(数据 pod 重启即丢)或投入 SQLite/Postgres 落地;无论哪种都必须**在部署文档里显式声明**,不能让用户以为"真实部署"等于"生产可用持久化" |
| **Phase III** | 前端容器化 + k3s 部署(real-API 模式) | Phase I 至少部分完成(否则前端打开后大部分调用仍是 501) | — | 中 | `frontend/Dockerfile` + k8s Deployment/Service manifest + 一个可从浏览器打开、调真实(非 mock)后端的 URL |
| **Phase IV** | 鉴权/限流/审计中间件真实化(可选,视 Phase III 后用户是否要多用户/对外暴露而定) | Phase I | — | 中 | 视野公开程度决定是否必须做;纯本机验收可保持 no-op 并在文档中明示风险 |

---

## §2 验证摘要(实测,非断言)

本节记录**本 session 实际执行过的验证命令与结果**,供下游 AI 复核基线是否漂移。

| 验证项 | 命令 | 结果 |
|---|---|---|
| REST 业务路由 501 现状 | `curl http://172.28.176.169:30081/api/v1/health` | `200 OK`(唯一非 stub 端点) |
| REST 业务端点现状(既往验证,per 前序 session) | `curl http://172.28.176.169:30081/api/v1/work-items` 等 | `501 Not Implemented`,`error.code=NOT_IMPLEMENTED`(参照 `error.rs::not_implemented`) |
| stub handler 数量 | `grep -c not_implemented crates/star-api-rest/src/routes/*.rs` | 27(见 §1.1,已排除 `mod.rs` 文档注释误命中) |
| 现有 Cargo.toml path-dep | `sed -n '1,60p' crates/star-api-rest/Cargo.toml` | 9 个 domain-*(tenant/identity/permission/workspace/project/work-item/worktree/agent/feedback)+ `star-webhook` + `star-context` |
| MCP 工具真实性 | 逐文件读 `crates/star-mcp/src/tools/*.rs`(16 文件全覆盖) | 16/16 均已接 `InMemory*Service`,0 硬编码 mock,注释显式标注"0 mock 硬编码 (per P0/P1 派生规)" |
| MCP 工具持久化性质 | `grep -rl "struct InMemory" crates/domain-*/src` | 命中 30+ domain crate(含全部支撑 REST 27 路由的 domain crate),确认无一使用 sqlx/Postgres |
| `star-mcp` 可否作为 library 引用 | `grep -A2 "^\[lib\]\|^\[\[bin\]\]" crates/star-mcp/Cargo.toml` | 只有 `[[bin]]`,无 `[lib]` 段 → 确认不可 path-dep 引用,只能抄写范式 |
| 中间件真实性 | 读 `middleware/auth.rs` 全文 | `auth_layer_stub` 显式 no-op pass-through,tracing 标注待 P2 阶段实装 |
| 前端部署产物 | `find` 全仓搜索 `frontend/Dockerfile`、`deploy/**/frontend*` | 0 命中,确认前端无任何容器化/k8s 产物 |

**本节未执行、留给下游 AI 的验证**:`cargo test -p star-api-rest --all-features` 完整跑一遍当前基线(本 session 未跑,只读了 `lib.rs` 内嵌的 2 个测试用例源码);`cargo clippy -p star-api-rest`。

---

## §3 已知缺口(per 缺标比错标)

1. **文档漂移**:`lib.rs`/`mod.rs` 模块文档声称"22 路由 (16 MCP 镜像 + 6 webhook)",实测代码是 27 路由(18 镜像 + 9 webhook)。下游 AI 接线时应顺手把文档注释数字改对,不必单独立项。
2. **submissions 端点先天只能部分真实**:其抄写范式 `submit.rs` 自身文档承认 step 6-12 是"简化 mock(brief 授权 OK)"。照抄之后 `/api/v1/submissions` 依然不是 100% 真实,这不是接线失误,是上游 MCP 工具本身的既有设计决策,下游 AI 不应误判为自己接线出错。
3. **持久化缺口是全局性的,不是某几条路由的问题**:支撑全部 27 条路由的 domain 服务清一色 `InMemory*Service`。Phase I 做完后,"real"只解决了"是否调用真实业务逻辑",没有解决"数据是否在 pod 重启/多副本间存活"。这两者必须在给用户的说明里分开陈述,不能让"REST 层已接线"被误读为"持久化数据库已就绪"。
4. **鉴权是不设防的**:`AuthLayer`/`RateLimitLayer`/`AuditLayer` 全部 no-op。如果 Phase III 把前端 + 后端一起暴露到比"本机测试"更广的网络范围(哪怕只是同一 WiFi 下的另一台设备),任何人都能不带凭证调用全部写操作(`POST /work-items`、`POST /worktrees` 等)。下游 AI 必须在部署说明里显式提示这一风险,而不是默认视而不见。
5. **AGENTS.md §7 #1 domain-* crate 真实数据接入现状是"21/25 done, 4/25 缺口"**,不是全量完成。4 个缺口子批次是治理域(audit/tenant/relation/kms),与本 WBS 覆盖的 work-item/workspace/worktree/search/scm/validation 不重叠,不阻塞本 WBS,但下游 AI 如果后续要扩展到 webhook 之外的治理类路由,需先确认这 4 个缺口是否已被填上(数字有时效性,需重新查 `AGENTS.md §7`)。
6. **`NEXT_PUBLIC_*` 环境变量是 build-time,不是 runtime**:Next.js 会把 `NEXT_PUBLIC_USE_REAL_API` / `NEXT_PUBLIC_API_BASE_URL` 在 `next build` 阶段直接打包进客户端 JS bundle。如果下游 AI 按常见 k8s 习惯把这两个变量写进 Deployment 的 `env:` 字段(运行时注入),前端会在构建时读到未设置的默认值,\*\*静默地\*\*继续跑 mock 模式而不报错——用户会看到"页面能打开但数据像是假的",且没有任何报错线索。正确做法是把这两个变量当 **Docker build ARG**,在 `docker build --build-arg NEXT_PUBLIC_API_BASE_URL=http://<k3s-node-ip>:30081 ...` 时传入,烘焙进镜像。
7. **webhooks.rs 的 9 条路由没有 MCP 范式可抄**,是本 WBS 里唯一需要"原创"而非"移植"的部分,复杂度与工作量应单独估算,不要跟其余 18 条一起摊薄估计。
8. **`code.rs`/`context.rs` 的路由虽有 `domain-search` 真实业务逻辑支撑,但 `domain-search::InMemorySearchService` 索引的是什么数据源(是否需要实际扫描仓库文件,还是纯内存 fixture)本 session 未深入验证** — 下游 AI 在 Phase I 接线 #9-13 时应先读一遍 `domain-search` 自身的 lib.rs,确认 `search`/`get_symbol`/`find_references`/`get_code_context` 四个方法是否已有真实数据可查,还是同样是空的 in-memory HashMap(即"业务逻辑真实,但没有任何数据被写入过,查询永远返回空")。
9. **k3s 部署清单当前只传 `STAR_API_REST_BIND_ADDR` 一个环境变量**,Phase II 若决定接入真实持久化,需要同步扩充 `deploy/k3s-local/star-api-rest-deploy.yaml` 的 `env:`(数据库连接串等)与可能的 `PersistentVolumeClaim`。

---

## §4 子代理失败接手清单(per 7 子代理派生规则)

若下游 AI 以子代理分工执行,建议按以下边界拆分,任一子代理中途失败时,接手者按此表定位断点:

| 子代理范围 | 输入 | 产出 | 失败时如何判断"真的完成没有" |
|---|---|---|---|
| A. Phase 0 基线复核 | 本文件 §1.1/§2 | 更新后的路由清单 + `cargo test` 基线报告 | 检查是否真的跑了 `cargo test -p star-api-rest`,不能只看 exit code,要看具体哪几个测试 pass |
| B. Phase I 批次 1(work-items/workspaces/worktrees,8 条,dep 已就绪) | §1.1 表 #1-8 | 8 个 handler 改为真实调用,单测断言从 501 改为真实数据校验 | `git log -p --follow` 该 worktree branch,确认改动真的落到 `routes/work_items.rs` 等文件,不是只改了测试断言掩盖未接线 |
| C. Phase I 批次 2(code/context/merge-requests/reviews/validations/pipelines/submissions,10 条,需先加 3 个 Cargo.toml dep) | §1.1 表 #9-18 | `Cargo.toml` diff(新增 3 个 path-dep)+ 10 个 handler 改为真实调用 | 确认 `cargo build -p star-api-rest` 在新增 dep 后仍能通过工作区编译,无循环依赖 |
| D. Phase I 批次 3(webhooks 9 条,原创接线) | §1.1 表 #19-27,`star-webhook` 自身 API | 9 个 handler 接 `DeliveryStore`/`RetryPolicy` | 因无 MCP 范式可比对,验收标准是"9 个端点各自的 CRUD 语义自洽"(建端点能查到、删端点查不到等),需要子代理自己写集成测试而非照抄 |
| E. Phase III 前端容器化 | `frontend/package.json`、`frontend/src/mocks/real-mode.ts` | `Dockerfile` + Deployment/Service manifest + 验证浏览器打开可用 | 必须**用真实浏览器或 curl 验证页面渲染出的数据不是 mock 数据**,不能只看容器 `Running` 状态(参照 §3 缺口 #6 的静默失败模式) |

---

## §5 守门规则(本文件专属)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件是计划草案,**不实施任何代码改动**,实施前需 Ulysses 拍板确认 Phase 顺序 | 本 session 用户发令原文"我让下游 ai 完成真正的部署" |
| 2 | Phase I 接线时必须逐条对照 §1.1 表格里标注的 MCP 工具文件,禁止凭空重写业务逻辑——`InMemory*Service` 的接口签名已经在 `domain-*` crate 里定义好,照抄参数/返回值映射即可 | per `domain-work-item::InMemoryWorkItemService` 等既有实装 |
| 3 | 新增 Cargo.toml path-dep(`domain-search`/`domain-scm`/`domain-validation`)后必须跑一次 `cargo check --workspace` 确认无循环依赖 / feature 冲突,不能只 `cargo check -p star-api-rest` | AGENTS.md 守门 #1 "--all-targets 全仓检查" 精神一致 |
| 4 | 任何"看起来完成"的路由,验收标准是**真实 curl/集成测试返回非 501 且返回体是可验证的具体数据**,不是"编译通过"或"看代码 diff 顺眼" | AGENTS.md §3 报告 7 段结构 §2 验证摘要要求"实测"而非转述 |
| 5 | 持久化(Phase II)与鉴权(Phase IV)如果决定"暂不做",必须在交付说明里显式写明"当前为 in-memory / no-op 鉴权,重启丢数据 / 无访问控制",不能默默略过 | per 守门"缺标比错标"原则 |
| 6 | 前端容器化(Phase III)构建命令必须把 `NEXT_PUBLIC_*` 变量作为 `docker build --build-arg` 传入,不能写成 k8s Deployment 的 runtime `env:` | 本文件 §3 缺口 #6 |
| 7 | Phase 完成后的报告如果要落成正式 `PHASE-*`/`STAR-*` 报告,必须遵循 `AGENTS.md §3` 的 7 段结构(目的/矩阵/验证/缺口/子代理清单/守门/签字/修订历史) | AGENTS.md §3 |

---

## §6 签字栏

| # | 角色 | 姓名 | 日期 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses | 待拍板 | 🟡 待用户确认 Phase 顺序 / 是否需要 Phase II 持久化 / Phase III 是否现在就做 |
| 2 | 执笔 | Claude Code (Sonnet 5) | 2026-09-09 | 🟡 已完成逐文件实测调研(27 路由、16 MCP 工具、9 domain-* dep、3 中间件全部读过原文件),未做任何代码改动,按用户要求仅交付计划文档 |

*(本文件为纯计划文档,非实施报告,SRE Lead / 平台工程师 / 评审主持人 3 项待下游 AI 实际执行后在对应的 `PHASE-*-REPORT.md` 里补签)*

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-09 | Claude Code (Sonnet 5) | 初版:逐文件核实 27 条 REST 路由 → 16 个 MCP 工具范式 → 9+3 个 domain crate 的完整映射表;识别 3 个未声明的 Cargo.toml path-dep(domain-search/domain-scm/domain-validation);识别持久化全局缺口(全 InMemory)、鉴权全局缺口(全 no-op)、前端 build-time env 陷阱;拆 4 Phase(基线复核/REST 接线/持久化决策/前端容器化 + 可选鉴权真实化) | 用户 2026-09-09 发令"把完成后端接管的计划写成 spec,然后更新 handoff,我让下游 ai 完成真正的部署,而不是 mock 版本" |
