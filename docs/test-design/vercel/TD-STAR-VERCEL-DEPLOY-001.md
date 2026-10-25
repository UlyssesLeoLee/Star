# TD-STAR-VERCEL-DEPLOY-001

> **STAR Vercel 部署测试设计 v0.1**
>
> - 状态：Draft；测试计划，不是测试结果；所有下列用例初始状态均为 NOT_RUN
> - 需求基线：`docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` v0.4
> - 基本设计：`docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md`
> - 详细设计：`docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md`
> - ADR：`docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md`
> - 工作计划：`docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` 的 VCD-0..VCD-4
> - 日期：2026-10-24
> - 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> - 审批：架构师（Mavis 接手 agent per DEC-008）

本文件定义后续验收方法，不证明代码已实施，也不证明账户、部署、授权、平台运行时或业务结果。T-xxx 是本测试设计内的用例编号；所有 SRS 引用均保持原始 FR-VC、NFR-VC、AC-VC 与 GAP 标识，未创建额外的需求/验收 ID。未取得指定的账户、Owner fixture、数据隔离和执行证据前，用例保持 NOT_RUN。

## 1. 范围、门禁与测试状态

### 1.1 范围

- Phase 1：Next.js 呈现面、同源 BFF、真实宿主会话、当前 Star Project/Run 授权、protected Preview、生产 promotion 和本地能力 fail-closed。
- Phase 2 可行性：仅 VCD-2 隔离 Rust Runtime 构建/运行探针；不得使用生产密钥、生产数据或业务写入。
- Phase 2 条件实施：VCD-3 只有 VCD-2 PASS 且 WLD-0/WLD-6、Owner/API/身份/数据契约均审查通过后才执行。
- 运行验收：VCD-4 回滚、Owner Schedule reconciliation、遥测、成本和运行手册。
- Star Owner、Agent/Task 执行、原生 Hook、RunInfrastructureBinding、Worktree 本地文件系统及 Multipass/K3s 均属于 Star 权威路径；本 TD 不把 Vercel 行为替代其验收。

### 1.2 执行状态

| 状态 | 含义 |
|---|---|
| NOT_RUN | 尚未执行；所有 T-001..T-048 均以此状态开始 |
| PASS | 已在指定目标环境按预期实测，证据可复核 |
| FAIL | 实际结果违反预期；保留日志、请求 ID 和环境信息，不隐藏副作用 |
| BLOCKED | 实际执行前置检查后，由缺少的 Owner、账户、provider、测试数据或决策阻断 |
| UNKNOWN | 官方资料和实测都不足以断定平台能力；不能计为 PASS |

任何测试报告必须写明目标环境/套餐/region、源码 commit、部署 ID、配置 revision、实际前置条件、证据链接/摘要和执行时间。文档静态检查只能证明引用和格式，不得升级 hosted/runtime/test 状态。

### 1.3 安全及运行前置门

1. 测试只能在隔离 Preview 或已授权测试环境执行。先证明没有生产 secret/DB role/Owner mutation/Schedule credential。
2. 先确认宿主身份来源和 Project/Run 权限可审计；没有真实 session contract 时，身份/授权类用例保持 NOT_RUN。
3. 仅创建/使用已存在的 Star 测试 Owner/API 契约；禁止为本测试新增 Owner、数据库、迁移器、worker、scheduler 或 VM execution plane。
4. 任何模糊写入状态必须使用原 Star Owner 和原 idempotency key 查询；测试程序不得盲目重发。
5. 若测试需要 Multipass/K3s，使用受控、带跨进程锁的 launcher；Stopped/Deleted 均计入 inventory，测试总记录不得超过 2。inventory 读取未知或已满时 fail closed，不自动 purge。Docker Desktop 不是前置条件。VCD-2 若无需本地 VM，不创建 Multipass instance。
6. Rust Runtime/计划特有的未知行为使用隔离最小探针和无敏感数据。通用 Functions WebSocket GA 不能作为 Rust WebSocket 通过证据。
7. 测试数据、凭证和日志按已批准的数据保留策略清理；不得删除既有用户或其他工作流数据。

## 2. 环境与证据集

| 环境/层级 | 用途 | 必需隔离 | 证据 |
|---|---|---|---|
| Local CI | 构建、typecheck、BFF unit、lint、产物与依赖扫描 | 假 upstream 仅用于确定性单测；不得把 fake 标为系统通过 | CI job、commit、测试报告、bundle/SBOM 摘要 |
| Protected Preview | Phase 1 hosted build、真实会话与授权读取、能力 fail-closed、promotion 保护 | 只用 Preview secret/数据；不得调用生产写 API/Cron | Preview deployment ID、角色矩阵、截图、审计和请求 trace |
| Isolated Rust target | VCD-2 最小 Rust handler/依赖/网络与运行时行为 | 单独 Project/environment；无生产数据、secret、DB role 和业务写入 | 实际账户/套餐/region、源 commit、构建包摘要、原始指标和官方文档快照 |
| Star-owned test fixture | Owner 交易、幂等、RLS、Outbox/Inbox、Cron catch-up | 由 Star Owner 隔离租户与 Run 范围；测试不能新增业务 Owner | Owner receipt、审计、Outbox 消费证据、RLS 断言 |
| Production promotion | 仅 Phase 1 发布门和回滚演练 | 受保护审批；生产数据变更须另有既有授权 | promotion actor/audit、目标部署定义、回滚和对账记录 |

测试数据使用明确的测试租户/Project/Run/Worktree。验证跨范围拒绝时，准备两个独立身份或授权范围，证明 B 的身份不能读取/修改 A 的事实；不得把客户端 Run id 视作授权证明。

## 3. 测试分层和顺序

推荐顺序：静态/构建门 -> BFF unit/security -> hosted Preview identity/read smoke -> Preview isolation negatives -> VCD-2 isolated Rust proof -> 如获批准的 Owner-backed VCD-3 tests -> rollback/reconciliation/operations。上游失败时后续依赖用例保持 NOT_RUN 或在有实际执行记录时记 BLOCKED，不可跳过失败门并把发布标成就绪。

| 层 | 验证目标 | 用例 |
|---|---|---|
| 构建与静态 | 必需环境、Next.js build、类型阻塞门、产物、SBOM、secret scan | T-001..T-003, T-010, T-012 |
| BFF 单测/安全 | 方法/来源/cookie/上游/path/body/header/cache/redirect 约束 | T-004..T-009, T-013 |
| Hosted Preview | 真实宿主会话、Project/Run/Worktree 读取、跨 Run deny、不可用状态 | T-014..T-019 |
| Cron/Schedule | 漏发、失败、重复、延迟、重叠、回滚、run-as 和 owner recovery | T-020..T-027 |
| Rust VCD-2 | 构建根、MSRV、函数包、运行限制、支持矩阵和恢复 | T-028..T-036 |
| Owner/PG VCD-3 条件 | 持久回执、RLS/连接池、迁移边界、Outbox/Inbox | T-037..T-042 |
| Preview/运营 | Preview no-side-effect、Promotion、rollback、telemetry/cost、依赖许可 | T-043..T-048 |

## 4. 测试用例

所有以下用例的当前状态均为 **NOT_RUN**。每个用例只有在其 WBS 阶段和真实前置依赖被批准后方可执行。

### 4.1 构建、BFF 与安全

| 用例 | 前置/操作 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-001 | 使用固定 commit 在目标 Preview 配置构建 Next.js Project，并部署独立 artifact；再验证受控回滚 | build、deployment 和 rollback 可独立复核；Rust Project 不属于 Phase 1 | FR-VC-001; AC-VC-001 |
| T-002 | 生成并扫描客户端 bundle、静态资源、source map 与可发布产物 | 无 Star 上游主机名、后端地址或可复用凭证；public 配置只包含允许的非敏感值 | FR-VC-006/008/018; AC-VC-002 |
| T-003 | 删除 Build 环境的一项必需 build-time config 后重新构建 | 构建失败并定位缺项；不能以 mock/seed 继续 | FR-VC-005; AC-VC-004 |
| T-004 | 构建 BFF 边界单测：允许方法与路径成功；非法方法、编码路径穿越、斜杠变体、未知路径被拒 | 所有分支一致执行 allowlist/规范化；留单测报告和拒绝原因 | FR-VC-007; AC-VC-003 |
| T-005 | 对写方法分别用合法同源、缺 Origin、错误 Origin 与跨站来源发请求 | 只有合法来源继续；CSRF 失败不触发 upstream | FR-VC-007; AC-VC-003 |
| T-006 | 测 cookie 名、缺失值、超长/格式非法值和上游 HTTPS/host 配置 | 不允许任意 cookie/header 冒充身份；Production HTTP/不受信任 host/带 userinfo 上游被拒 | FR-VC-007/008/017; AC-VC-003 |
| T-007 | 提交刚低于和超过既有 BFF 2,000,000 bytes 上限的 body；再测试平台 4.5MB 上限之外的请求 | BFF 对超限请求在转发前拒绝且不缓冲无界 body；大文件使用 Star 自有路径 | FR-VC-007; NFR-VC-02; AC-VC-003 |
| T-008 | 让 upstream 返回 redirect、Set-Cookie、hop-by-hop header 和 actor-scoped cache header | 自动 redirect 被关闭，意外 3xx 拒绝；Set-Cookie/hop-by-hop 不泄漏；actor-scoped response no-store | FR-VC-007/008; AC-VC-003 |
| T-009 | 在 upstream 返回错误/延迟时检查 BFF log、trace、客户端错误和 terminal 输出 | 不含 token、cookie、完整敏感 body 或凭证；只保留最小可追踪 request/correlation 元数据 | FR-VC-008/018; AC-VC-003 |
| T-010 | 令 CI typecheck 产生一个类型错误，随后运行 Preview build gate | 阻塞发布；不依赖 ignoreBuildErrors 或 advisory 门 | GAP-07; FR-VC-005; AC-VC-004 |
| T-011 | 构建生产形态时访问 Automation Debug UI/route | 本机 Python console 依赖明确不暴露或返回有解释的 disabled UI，不产生误导性 loopback 502 | FR-VC-016; GAP-08; AC-VC-006 |
| T-012 | 扫描实际部署包、依赖闭包、构建日志、错误日志和 telemetry payload | 无 secret 泄漏；生成 Node/Rust 适用 SPDX/SBOM 与逐项许可/NOTICE 结果 | FR-VC-018/019; AC-VC-001 |
| T-013 | 配置缺失/损坏服务端上游地址，及尝试从用户输入改写目标 host | 缺失或无效配置在其定义的 readiness/request 边界 fail closed；host/path 不可由用户改变，不发生 SSRF 或 silent mock | FR-VC-005/006/007; AC-VC-003/004 |

### 4.2 身份、授权、能力与服务语义

| 用例 | 前置/操作 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-014 | 用真实嵌入宿主 session 登录 Preview，读取当前用户有权限的 Project、Run、Worktree | 显示来自 Star API/Owner 的实际返回和当前授权范围；留请求与部署证据 | FR-VC-008/016/021; AC-VC-005 |
| T-015 | 使用过期/无效 session 请求受保护资源；再用有效身份但无该 Run 权限请求 | 认证失效与范围拒绝分别可见；401 失效会话，403 不降级成登录成功或重放写入 | FR-VC-008/016; AC-VC-003/005 |
| T-016 | 以用户 A 登录后修改客户端 Run/Worktree 标识为 B 的资源，再直接请求 upstream 受保护读取 | Star API/Owner 对越权范围拒绝；BFF 和前端 selection 不授予访问 | FR-VC-008/016/021; AC-VC-005 |
| T-017 | 尝试用自定义 user/role/tenant header、伪造转发头或仅靠 allowlisted egress IP 获得权限 | 身份只由已验证宿主会话建立；伪造 header/IP 不改变授权 | FR-VC-008/017; AC-VC-003/005 |
| T-018 | 在尚未接入真实 Owner/provider/runtime 的页面点击启动 Agent/Task CLI/Hook/本地 Worktree 操作 | 明确 unavailable 和所需 Star 本地 Runtime；无进程、mock 成功、假回执或业务副作用 | FR-VC-013/016; AC-VC-005/006 |
| T-019 | 在 Star API 不可达、授权服务不可用和 capability/profile 缺失三种情况下打开 Run tabs | 显示 unavailable/unknown，写能力 fail closed；不使用缓存授权或 seed/mocks 隐藏故障 | FR-VC-003/016/021; AC-VC-005/006/013 |

### 4.3 Cron 与 Schedule recovery

仅当存在经过核实的 Star Schedule intake 和 Owner fixture 后，测试 Cron 场景。每一情形都检查 Star owner 持久状态、回执、审计和副作用，而不是只看函数 HTTP 码。

| 用例 | 注入/步骤 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-020 | 故意让一个 due 时间窗没有 Cron 请求到达；确认该窗无函数执行记录；在下一次唤醒扫描 | Star 持久 due work 被后续扫描补齐；同一逻辑 occurrence 只有一个 Owner receipt | FR-VC-014/015; NFR-VC-07 |
| T-021 | 令 Cron invocation 超时或返回失败，并观察平台是否自动重试；再发后续唤醒 | 该失败 invocation 不依赖平台重试；后续 Owner reconciliation 可以查找未完成工作，保持幂等 | FR-VC-014; NFR-VC-07 |
| T-022 | 对同一 occurrence 发送重复和迟到请求 | Star Owner 幂等去重，单一业务事实/执行需求/receipt；迟到输入进入对账而非重复创建 | FR-VC-014/015; AC-VC-012 |
| T-023 | 保持一轮触发处理跨越下一个计划间隔，再让下一轮并发到达 | 允许出现重叠，但 Owner 锁/租约 fencing 与幂等边界避免双重执行事实；Vercel handler 不持有业务锁 | FR-VC-014; NFR-VC-07 |
| T-024 | 分别在首次 trigger、retry、resume 前撤销固化创建者 run_as_actor_id 的 Project/Run 权限或 execution capability | 每一入口实时重新授权并 fail closed；写审计；不能用 Rule 创建时授权缓存继续执行 | FR-VC-014; AC-VC-005 |
| T-025 | 令授权目录不可用、run-as 身份缺失、权限事实不完整或 RunInfrastructureBinding 不可解析 | 不接受新的执行需求、不预留容量、不启动 executor；记录阻断原因 | FR-VC-014/015/016; AC-VC-005/006 |
| T-026 | 在隔离环境切换到目标部署回滚；核查其精确 Cron 定义和项目 Disable Cron Jobs 设置，再和 Star Schedule 状态对账 | 回滚后定义与目标部署一致，项目禁用状态保留；Star owner 对账持久工作，不只验证“存在一个 Cron” | FR-VC-014; AC-VC-001 |
| T-027 | 让 Owner Outbox/consumer 暂时不可达或 receipt 延迟，再重复唤醒 | Vercel handler 不本地持久化/消费；Owner 恢复后以原幂等身份完成并返回可查询的持久状态 | FR-VC-014/015; AC-VC-012 |

### 4.4 Rust Runtime VCD-2 可行性（全部隔离）

| 用例 | 前置/操作 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-028 | 在目标账户/套餐构建最小 Rust handler，验证工具链最低 Rust 1.99.0 和 edition 2021 | 保留实际构建日志、工具链和目标计划；低于 MSRV 或 workspace lint 被放宽时失败 | FR-VC-002/004; NFR-VC-08; AC-VC-007; GAP-03 |
| T-029 | 比较 SRS 指定的构建形态 A 与 B；验证至少一个候选可从 Vercel 构建并访问仓内 crates/star-api-rest | 不复制 domain logic；记录上传根、manifest、产物与失败证据；未通过仍关闭 Phase 2 | FR-VC-002; AC-VC-007; GAP-10 |
| T-030 | 对最小 handler 和代表性薄 adapter 分别测量每个实际部署函数的 release 依赖包 | 记录包清单/size；超出保守 250MB 则阻断 Rust 部署；不以 workspace crate 总数推算 | FR-VC-004; NFR-VC-03; AC-VC-011; GAP-01 |
| T-031 | 注入 in-memory repository、进程 EventBus、star-taskqueue 或进程内事实 cache 的配置 | 启动自检 fail closed；保存配置和启动证据 | FR-VC-003; AC-VC-008 |
| T-032 | 在隔离 endpoint 记录冷启动、首字节、p95、峰值 RSS、并发、duration 与目标 region | 实测与目标套餐绑定；只按 SRS 已有阈值判定；unknown runtime entitlement 保持 UNKNOWN | NFR-VC-01/04/05/06; AC-VC-011; GAP-11 |
| T-033 | 测试请求/响应载荷上限、请求超时/取消和平台强制终止行为 | 不超过 SRS 4.5MB；无后台函数内长轮询；超时不被误报为成功 | NFR-VC-01/02; GAP-11 |
| T-034 | 查阅官方 Rust Runtime feature table，并分别实测 streaming/WebSocket（若产品存在实际 Rust handler 需求） | 逐项标注官方证据、实测、UNKNOWN；不得从通用 Functions WebSocket GA 外推 Rust 支持；PTY/Canvas WSS 仍在 Star | FR-VC-013; GAP-02/09/11 |
| T-035 | 检查 Rust 函数日志、请求指标、回滚和新部署恢复；与同账户/计划绑定 | 未支持项记录 UNKNOWN；发布失败/回滚不改变 Owner 事实或启用未获证功能 | FR-VC-018/020; NFR-VC-04/05; GAP-09/11 |
| T-036 | 检查真实 runtime 上函数内启动 DDL、worker loop、queue、迁移或业务 cache 的尝试 | 设计/启动门拒绝；数据库迁移仅由独立 CI/批准的 Star migrator 执行 | FR-VC-003/011/015; AC-VC-008/010 |

### 4.5 Owner-backed VCD-3 条件测试

以下仅在 VCD-2 PASS、WLD-0/WLD-6 已满足且真实 Star Owner/API contract 确认后可执行。否则这些用例仍为 NOT_RUN。

| 用例 | 前置/操作 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-037 | 经实际 Star Owner 做一笔授权写入，记录事务和持久幂等 receipt | Owner 负责授权与持久事实；没有函数侧 repo/EventBus/cache | FR-VC-003/015; AC-VC-008 |
| T-038 | 复用数据库连接池，以两个独立租户执行并发事务；各事务依次 SET LOCAL、业务语句、提交/回滚 | 租户数据互不可见；连接复用后没有上下文串号；session-level SET 不存在 | FR-VC-009/010/012; AC-VC-009; GAP-04 |
| T-039 | 扫描函数产物与请求路径，检查 DDL、schema migration 和 database role grants | 函数无 DDL 权限/无运行期迁移；独立迁移工作成功且有独立记录 | FR-VC-009/011; AC-VC-010 |
| T-040 | 事务提交后追踪 Owner transactional Outbox 到 Star 自有消费者、处理结果和可查询 receipt | owner-side 消费、幂等和审计闭环；函数本身不循环消费 | FR-VC-015; AC-VC-012 |
| T-041 | 在 Owner 已接收但 HTTP response 丢失时，以原幂等键查询，并尝试重放 | 状态先为 unknown；同一 Owner 的原 key 得到最终回执；不得生成重复副作用 | FR-VC-007/015; AC-VC-012 |
| T-042 | 复核任何新增表/迁移、RLS 类、owner、API endpoint 或授权范围 | 未经 W/T/M、RLS、DDD/WBS 审核的改动不得部署；不存在第二数据 Owner | FR-VC-009/010/011; GAP-04/06 |

### 4.6 Preview 隔离、发布与运营

| 用例 | 前置/操作 | 预期结果 / 留证 | 需求映射 |
|---|---|---|---|
| T-043 | 在隔离 Preview 中尝试读取生产资源、创建/修改 Run/Task/Schedule、写 Owner、产生 Outbox 或调用 Cron；将 production 目标指向拒绝记录 egress 的测试拦截器，不提供生产凭证，并通过只读审计核对无生产请求/副作用 | Preview 无生产 secret/data/Owner side effect；生产持久状态和 Outbox 不变；禁止 Promotion | FR-VC-021; AC-VC-013 |
| T-044 | 在隔离 promotion-gate 测试项目/平台模拟中，以无授权 actor 和批准 actor 分别请求 Production Promotion | 只有受保护 actor 可获准；Promotion 的身份、commit、artifact 和审计记录可复核。实际生产发布不属于此用例授权 | FR-VC-021; AC-VC-013 |
| T-045 | 模拟运行时函数失败、上游不可达、平台不可用及不明写入结果 | UI 不展示假成功；明确不可用/unknown，写入通过原 Owner/key 恢复 | FR-VC-003/016; AC-VC-005/006 |
| T-046 | 在隔离目标 Project/Preview 回滚到一个已知 artifact，并核对 Active Cron 定义与 Owner schedule state；Production 只可在单独授权的维护窗口执行 | 回滚版本可识别；Cron 精确匹配目标部署且 disabled setting 持续；Star business facts 不反向覆盖 | FR-VC-001/014/018; AC-VC-001 |
| T-047 | 检查 OTLP/平台 trace 与 Star Owner receipt 关联，并触发已批准的成本告警配置 | Telemetry 不含 secret/过量个人数据；部署健康与业务完成区分；告警阈值按批准配置实际触发 | FR-VC-020; AC-VC-001; NFR-VC-04 |
| T-048 | 审查实际 Node 与 Rust 供应链部署闭包，包括传递依赖、二进制、安装器/镜像、许可证和 NOTICE | 有 SPDX/SBOM；每项许可义务有处置记录；未决限制许可为 0；完整闭包而非只看主依赖许可证 | FR-VC-019; AC-VC-001 |

## 5. AC 与 SRS FR 全量追踪

### 5.1 Acceptance Criteria

| SRS acceptance | 测试用例 | 当前状态 |
|---|---|---|
| AC-VC-001 | T-001, T-046, T-047, T-048 | NOT_RUN |
| AC-VC-002 | T-002 | NOT_RUN |
| AC-VC-003 | T-004..T-009, T-013, T-015, T-017 | NOT_RUN |
| AC-VC-004 | T-003, T-010, T-013 | NOT_RUN |
| AC-VC-005 | T-014..T-017, T-019, T-024..T-025 | NOT_RUN |
| AC-VC-006 | T-011, T-018..T-019, T-045 | NOT_RUN |
| AC-VC-007 | T-028..T-030 | NOT_RUN |
| AC-VC-008 | T-031, T-036..T-037 | NOT_RUN |
| AC-VC-009 | T-038, T-042 | NOT_RUN |
| AC-VC-010 | T-039 | NOT_RUN |
| AC-VC-011 | T-030, T-032 | NOT_RUN |
| AC-VC-012 | T-022, T-027, T-040..T-041 | NOT_RUN |
| AC-VC-013 | T-043..T-044 | NOT_RUN |

### 5.2 Functional Requirements

| SRS requirement | Test coverage |
|---|---|
| FR-VC-001 | T-001, T-046 |
| FR-VC-002 | T-028..T-029 |
| FR-VC-003 | T-019, T-031, T-036..T-037, T-045 |
| FR-VC-004 | T-028, T-030 |
| FR-VC-005 | T-003, T-010, T-013 |
| FR-VC-006 | T-002, T-013 |
| FR-VC-007 | T-004..T-008, T-041 |
| FR-VC-008 | T-002, T-006, T-009, T-014..T-015 |
| FR-VC-009 | T-038..T-039, T-042 |
| FR-VC-010 | T-038, T-042 |
| FR-VC-011 | T-036, T-039 |
| FR-VC-012 | T-038 |
| FR-VC-013 | T-018, T-034 |
| FR-VC-014 | T-020..T-027 |
| FR-VC-015 | T-022, T-027, T-036..T-041 |
| FR-VC-016 | T-011, T-014..T-019, T-045 |
| FR-VC-017 | T-006, T-017 |
| FR-VC-018 | T-002, T-009, T-012, T-035, T-046 |
| FR-VC-019 | T-012, T-048 |
| FR-VC-020 | T-035, T-047 |
| FR-VC-021 | T-014..T-016, T-019, T-043..T-044 |

## 6. NFR and GAP closure matrix

### 6.1 Non-functional requirements

| SRS NFR | Evidence required | Test |
|---|---|---|
| NFR-VC-01 | Target Runtime p95 below SRS threshold; proven maxDuration; no function long-poll | T-032..T-033 |
| NFR-VC-02 | Request/response ≤ 4.5MB and BFF enforced body bound | T-007 |
| NFR-VC-03 | Actual Rust function package within conservative 250MB budget | T-030 |
| NFR-VC-04 | Actual instance entitlement, RSS, concurrency and cost recorded; no assumed Rust budget | T-032, T-047 |
| NFR-VC-05 | Verified region and data residency/network compatibility recorded | T-032 |
| NFR-VC-06 | Cold-start distribution measured and returned to SRS before assigning a target | T-032 |
| NFR-VC-07 | No SLA scheduling claim; Star owner recovery behavior proven under Cron failure modes | T-020..T-027 |
| NFR-VC-08 | Rust 1.99.0/edition/lints supported without weakening workspace safety | T-028 |

### 6.2 Known gaps

| SRS gap | Resolution evidence | Test / status |
|---|---|---|
| GAP-01 | Per-handler real dependency closure and deployed package size | T-030; NOT_RUN |
| GAP-02 | Separate general Functions GA from Rust streaming/WebSocket support | T-034; UNKNOWN until proof |
| GAP-03 | Actual Vercel builder supports Rust MSRV 1.99.0 | T-028; NOT_RUN |
| GAP-04 | Hosted PG and transaction pooling preserve all approved RLS behavior | T-038/T-042; NOT_RUN |
| GAP-05 | Next.js 16, TypeScript 7.0.2, Monaco/Three build behavior and duration | T-001; NOT_RUN |
| GAP-06 | Verify PostgreSQL Marketplace/vendor facts from official provider sources | T-038/T-042 plus source review; NOT_RUN |
| GAP-07 | Blocking typecheck and build behavior after existing type errors are resolved | T-010; NOT_RUN |
| GAP-08 | Local Python debug dependency is absent/disabled in hosted production shape | T-011; NOT_RUN |
| GAP-09 | Rust Runtime Beta lifecycle and capability support remain unknown until official update or account test | T-034/T-035; UNKNOWN |
| GAP-10 | Compare approved Cargo build-root candidates and prove access to actual domain crate | T-029; NOT_RUN |
| GAP-11 | Full target-plan matrix for duration, streaming/WS, concurrency, RSS, logs/metrics, payload/package and rollback | T-030..T-035, T-046; NOT_RUN |

## 7. Schedule deterministic evidence requirements

The Vercel Cron documentation and candidate reviews describe best-effort delivery; therefore Cron tests must not assert exact scheduled wall-clock behavior as a service guarantee. For T-020..T-027, collect:

1. Planned occurrence identity and persisted due-work state from the Star owner fixture.
2. Platform request/invocation log for the tested period, including proof of the deliberately absent/failed/overlapping invocation where applicable.
3. Owner-side idempotency and fencing receipts, Outbox state, audit, and final occurrence count.
4. Current run_as_actor_id authorization result at each trigger/retry/resume; proof that revocation denies new execution.
5. Fixed RunInfrastructureBinding identity/generation and separate Host/cluster reservation receipts; no claim of reserved capacity before owner confirmation.
6. For rollback, exact Cron definitions associated with the target deployment and the project Disable Cron Jobs setting before/after rollback, followed by Owner Schedule reconciliation.

If no durable Owner path, receipt API, binding, reservation or safe fixture exists, production Schedule integration remains disabled and the test has no PASS evidence.

## 8. Release exit criteria

- **VCD-0:** canonical SRS/ADR/basic/detail/test documents align; no invented SRS IDs or alternate Owner/DB/worker/scheduler/VM architecture; every test state is honestly recorded.
- **VCD-1:** AC-VC-001..006 and AC-VC-013 have actual required CI/hosted/identity/isolation evidence. Preview has no production secrets/data or Owner side effects.
- **VCD-2:** AC-VC-007..011 and GAP-01/02/03/04/10/11 are evidenced in the target plan; Rust-specific unknowns are resolved only by scoped official evidence or reproducible runtime tests. If a blocking limit fails, Rust control-plane deployment remains on Star runtime.
- **VCD-3:** only after VCD-2 PASS plus WLD-0/WLD-6 and concrete Owner/API/session/data/recovery review; AC-VC-012 proves the actual Owner-to-consumer receipt path without a new owner.
- **VCD-4:** rollback, schedule reconciliation, observability, cost alerts and operational recovery are rehearsed and recorded; no invented performance, coverage or rollback time thresholds.

No exit criterion is satisfied by this test design alone.

## 9. Open conditions

- Target account/plan, domains, region and protected Preview settings are not evidenced here.
- Real host-session contract and Owner authorization API mapping must be provided/verified before hosted auth tests.
- Owner-backed due-work scan, idempotency lookup, Schedule trigger intake and Outbox consumer fixture remain prerequisite contracts.
- Rust runtime capability and Vercel build image support for Rust 1.99.0 remain UNKNOWN.
- PostgreSQL Marketplace/vendor selection, transaction pooling and RLS compatibility are not established.
- Test-specific budgets, alert thresholds and data-retention durations need approved operational baselines before execution.

## Revision history

| Version | Date | Reviser | Change |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses — Mavis review | Established NOT_RUN test plan against SRS v0.4 with complete FR/NFR/AC/GAP traceability, Preview no-side-effect, Cron recovery, Rust VCD-2, owner/data gates and two-record Multipass safety; no implementation or runtime claim. |
