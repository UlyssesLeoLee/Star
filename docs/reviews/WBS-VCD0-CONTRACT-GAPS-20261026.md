# WBS-VCD0-CONTRACT-GAPS-20261026

> 审计基线：dev@7a82c098fe1dc021c8702e468768d68e8b0ee03e；lane：codex/wbs-vcd0-contract-gaps-20261026
> 日期：2026-10-26
> 结论：**VCD-0 仍 OPEN。** 本报告是文档与既有审计证据的静态收敛，不是设计批准、实现验收或部署证明。

## §0 目的与结论

本报告把 Vercel 部署设计与 Project/Repository、身份授权及实时协作设计放在同一条追踪链中，列出 VCD-0 关闭前仍需解决的跨文档合同和验收缺口。检查范围包括当前 Vercel SRS/ADR/BD/DD/TD、WLD1 Project/Repository SRS/BD/DD/TD、协作规格、WBS 与已有 W0/W16/R3 审计。

架构边界总体一致：Vercel 只作为受保护 Web/BFF 呈现路径；Star 原生 Owner 保持身份、授权和业务事实权威；Agent、Hook、Worktree 文件系统、长连接 Gateway、持久 Outbox/Inbox 和执行面继续属于 Star。主导航和 hostless Run 语义在基本设计中明确。当前阻塞点是这些边界还没有形成一份可批准的端到端接口和测试闭环，且部分最新的 Project/Repository 与数据库决策尚未回写 Vercel canonical 文档。

当前源文件状态：Vercel SRS v0.4 为 Draft；ADR v0.2、BD v0.3、DD v0.3、TD v0.4 为 Draft。WBS v4.26 明确 VCD-0 OPEN、VCD-1..VCD-4 未开始，托管账户、数据库/RLS、部署、运行时和性能证据 NOT_RUN。此审计不改变这些状态。

## §1 优先级缺口矩阵

| 优先级 | 缺口与追踪链 | 证据及当前状态 | 最小关闭动作与验收证据 |
|---|---|---|---|
| **P0** | **真实会话、当前授权和撤权链未冻结。** Hosted UI/BFF 必须使用嵌入宿主提供的真实 session，并让 Star API/Owner 对 Project、Run、Worktree、资源 capability 在读取、写入、连接和重连时复验。 | SRS AC-VC-005 要求真实登录与授权 Project/Run/Worktree 读取（SRS §7，230）；BD 明确实际 host-session issuer/audience 与 Run contract 待验证（BD §3.1，89）；DD 要求会话取自真实宿主、BFF 不得伪造身份、401/403 分开处理（DD §3.1，72–76），但 issuer/audience、授权 API 映射仍是 OPEN（DD §12，307–308）。T-014..T-016 有效/无效身份及跨 Run 负向用例，T-049 覆盖连接握手、授权、撤权及重连前置门（TD §4.7，163–177）；WBS 另要求 T-049 明确证明“已认证但不具目标 Run/capability 的 principal 收不到任何受保护数据”（WORKTREE-GROUP-NEXT-WBS，346）。当前 T-049 尚未将该零数据断言写成明确的操作/预期结果；T-016 的跨 Run 请求是相邻覆盖，不应静默视为同一验收。 | 从真实嵌入宿主确认 issuer/audience、cookie/token 传递、生命周期和 Star principal 映射；把当前 Project/Run/resource 授权与活动连接撤权行为写入 canonical 合同。沿用现有 T-014..T-016、T-049 追踪，明确增加已认证但无目标 Run/capability 时零受保护数据、401 失效与 403 范围拒绝、重连重新授权、撤权后关闭或阻止下一条受保护数据的断言。未取得真实契约前保持相关 UI/连接 unavailable。 |
| **P0** | **Gateway 与 chat SSE 路由/恢复合同未定，且浏览器通道清单不一致。** 需要先区分已有连接、目标连接、排除能力和不可用能力，再决定 Vercel 页面访问的 Star 服务与安全边界。 | SRS 代码库存记录 terminal WS、remote file-browser WS、ARG WS、chat SSE（SRS §0，35），而 FR-VC-013 列 terminal/PTY、noVNC、Canvas WSS、ARG 与单独可选 chat SSE（SRS §4.4，176）。TD T-049 测试 terminal PTY/session、noVNC、ARG、Canvas WSS 与 chat SSE，并明确排除 remote file-browser（TD §4.7，163–177）。既有 TD R3 报告将此判断为通道替换/范围缺口，要求统一“existing/proposed/excluded/unavailable”分类（VCD0-TD-COMPATIBILITY，22–28、66–72）；DD 要求真实 handshake、credential、Origin allow/deny、当前授权/撤权、协议事件、heartbeat、关闭/重连合同（DD §2.3，62–66）。SRS 允许 Next Route Handler 透传 chat SSE，但 ADR/BD/DD 都未选定最终路由（ADR §6，57；BD §5，138；DD §2.3，66）。T-049 因前置契约缺失而保持 NOT_RUN，VCD-0 保持 OPEN（TD，163、177）。 | 在 SRS/ADR/BD/DD/TD 与实际 Star Gateway/API 合同中用一张通道表逐一分类上述通道，并说明每项的 owner、数据方向、认证方式、Origin/跨站防护、撤权行为、stream/backpressure、disconnect、reconnect/cursor recovery 和不可用表现。明确 chat SSE 的唯一批准路由及其取消/恢复行为。更新 FR-VC-013 的验收表和 T-049；不得推定 remote file-browser 授权或从通道盘点推出暴露权限。以获准/拒绝 Origin、有效/缺失/伪造身份、撤权、Gateway 故障/恢复和 SSE 上游故障/恢复的脱敏证据完成 T-049 后，才可关闭相应门。 |
| **P1** | **Neon 的已选部署目标与 Vercel Owner/数据库/迁移路径未映射一致。** 目标 provider 已决定，但这不等于某个 Vercel function 可以直连或拥有数据库，也不证明迁移身份/RLS 已验收。 | WBS v4.26 记录 Vercel 使用 Neon、K3s 版使用 Multipass VM 内 PostgreSQL，并要求两部署分别验收（WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE，56、188）。WLD1 SRS 与 BD 已记录 Star Project Owner、Neon/K3s VM profile 和数据隔离（SRS-WLD1 §4.1，46–53；BD-WLD1 §9，113–116）。但 Vercel SRS 仍称 hosted PG provider 仅在 Phase 2 评估后选型，并把“Vercel Postgres 迁 Neon”标作第三方来源待官方复核（SRS §3.1，116；§8 GAP-06，251）；BD 仍让服务商选择受 GAP-06 约束（BD §6.3，174–175）；DD §5.2 仍把 Provider/Owner-to-storage/API 映射、pool、迁移身份列为 UNKNOWN 并 fail closed（DD，132–143）。WLD1 BD/DD 明确 Neon 的具体 Owner runtime 放置、代理/连接池、branch/region 和 DB roles/grants 仍 OPEN；TD 的 DEP-WLD1-01/02 是 Neon 与 VM DB 分别验收的计划，不是结果（BD-WLD1，113–116、144；DD-WLD1，200–207；TD-WLD1，3、9–11、118–121）。 | 更新 Vercel SRS/ADR/BD/DD/TD，记录用户已选 Neon 作为 Vercel 部署 profile 的目标，同时清楚界定既有 Star Owner/API 的实际运行位置、谁访问 Neon、BFF/函数是否只能通过 Owner API、数据库角色与 RLS context、连接池模式/预算、独立直接迁移端点及其作业身份。官方来源核验“平台产品事实”与目标选择分开记录。两部署共用版本化业务契约但保持数据库、凭据、迁移状态隔离；用 Neon 和 Multipass VM PostgreSQL 分别完成目标 migration replay、owner/app grants、FORCE RLS、事务级 SET LOCAL、授权竞争、Outbox/rollback 验收。不要新建第二业务 Owner 或 Vercel 专属迁移权威。 |
| **P1** | **Project identity、0..N Repository、hostless Run 与迁移后解绑已有规范设计，但尚未进入 Vercel hosted acceptance 的端到端追踪。** | WLD1 SRS 把 Project identity、成员授权与 0..N Repo association 交给 Star 原生 Project Owner，Run 可无 Repo/Host/Branch，且分别指定 Vercel/Neon 与 K3s/Multipass VM profile（SRS-WLD1 §4.1，46–53）。FR-WLD1-01..14 指定同 tenant 多仓、Run binding 独立、每次读取复验授权/版本、retiring fence，以及解绑前迁移或显式退役并用新鲜零引用证明（SRS-WLD1 §5，66–79）；AC-WLD1-01..12 覆盖 hostless、迁移/解绑、目标 DB/RLS（§7，96–107）。BD-WLD1 §3.4 与 DD-WLD1 §2.2 已细化 retiring、同 Project 迁移、owner receipt 和 CAS（BD，79–85；DD，69–75）。相比之下，Vercel AC-VC-005 和 TD T-014 只泛称读授权 Project/Run/Worktree，没有显式注明 no-Repo/hostless Run 场景及 WLD1 关联验收。WBS 记载有 repository master、Run-scoped API 等来源切片，但 Project association writer、binding lifecycle、Host root/provider 与目标 DB/RLS 尚未闭环（WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE，22–25、35–36）。WLD1 自身也明确 durable Project writer/API/schema、binding、Host registry 与目标 Postgres 验收缺失（SRS-WLD1 §8，111–116；DD-WLD1，207）。 | 只引用现有 FR/AC/T IDs，不新造验收编号：在 Vercel AC-VC-005/T-014 trace 中链接 WLD1 AC-WLD1-02（no Repo/Host/Branch 的 Run）以及 AC-WLD1-01/12（0..N 和 migrate-before-unlink）；将“只展示当前 Star Owner 授权数据、无 binding 显式显示 unavailable/empty state”写清。WLD1 owner/API/provider/RLS 与数据库目标验收按其独立 WLD0/WLD1 门执行；在真实持久化 owner 和部署 profile 完成前，Hosted 能力不得伪报为已可用。 |
| **P1** | **Push collaboration、双边草稿和恢复尚未接入 Vercel 通道验收。** T-049 目前是通道安全/数据流检查，不是 durable collaboration acceptance。 | 协作规格要求 owner commit 触发有界 wake-up 和 durable Outbox relay，正常路径无固定客户端轮询/周期扫表，reconnect/gap 使用授权 cursor catch-up（spec §13，315–319、347–351）。WorkItem/Canvas owner 各自持有 CAS、draft/conflict、audit/outbox；SaveDraft 与 ReadConflict 都重新验证当前权限（§13.1–13.2，321–335）。双方精确 draft 只有在完整、校验、sealed 后才 pin；两方保留本地/远端草稿并可按 participant auth 读取 bounded diff，私有草稿不广播给无权订阅（§13.3、345；AC-COLLAB-02..05，359–363）。但规格声明当前 Group 仍 interval-poll，BFF SSE/WSS 不是 durable 跨实例 relay（§13，319、369–373）；W16 源码审计报告也记录 poller、process-local hub、无 durable cross-instance fan-out，未运行测试/数据库验收（WBS-W16-COLLABORATION-RUNTIME-AUDIT，22–34）。Vercel T-049 只测通道 fixture、授权、Origin、撤权、断连/恢复；没有验证 owner commit wake-up、跨实例 Outbox relay、两份持久草稿、唯一 conflict_id、cursor catch-up 或参与者差异读取（TD T-049，167–177）。 | 把 T-049 限定为 Vercel 接入与通道安全门，并明确它不替代 AC-COLLAB-01..08、WLD-6/7 和 E2E-COLLAB/PT-09。将 Vercel 上的 collaboration UI 状态绑定到实际 Star resource owner 与 durable event path；若不可用则明确 unavailable。按现有协作 gates 验证 owner transaction、双方 draft pin/recovery、participant-only diff、跨 Gateway relay/restart/gap/revoke；不要增加 BFF 内存业务队列或用 SSE/WS 连接成功代替持久事件确认。 |
| **P1（语义保全）** | **两种“≤1 秒”验收必须分开，不能转成单一延迟指标。** 设计本身已分别定义 arrival-trigger 与 conflict-delivery SLO；跨 Vercel 测试追踪仍应显式保留此区别。 | 协作规格 §13.3 规定第二份完整、摘要校验通过的重叠 proposal 若在 owner 收到第一份后的 1000 ms 内到达，必须立即进入 resource-fenced conflict transaction，不得等待 debounce/batching/arbitration；1000 ms 是 test stimulus 而非 holdback（spec，343；AC-COLLAB-02，360）。§13.4 的 p99 ≤1 s 从 conflict transaction commit 开始，要求在线双方显示同一持久 unread conflict_id 并能读首个 bounded diff；它是投递/可读 SLO，不是冲突检测窗口或上传 deadline（spec，353、362）。WBS 与 W16 报告亦明确 E2E-COLLAB-09 与 PT-09 是两道门（WORKTREE-GROUP-NEXT-WBS，345、352；WBS-W16，27、40）。 | 在 Vercel 文档/traceability 中引用现有 E2E-COLLAB-09 与 PT-09，但保持其 owner-runtime 归属，不把测量塞入 T-049。分别留证：(a) ≤1000 ms 内第二份完整 proposal 到达 owner 后无 holdback、冲突事实立即入账；(b) conflict commit 至双方 alert + 首个有界 diff 可读的 p99 ≤1 s，在版本化 Collaboration Profile 和记录的 host/DB/gateway/workload 下实测。不得以 network request cadence 或客户端时钟判定冲突。 |
| **P2** | **文档追踪元数据已过时。** 这不会改变业务合同，但会误导后续实现者认为测试/详细设计尚不存在或 WBS 仍是旧基线。 | Vercel SRS header 仍说 BD/DD/TD 尚未建立（SRS，11），但当前三份文件与 TD 已存在，且 WBS v4.26 记录 R3 复核（WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE，56、188）。ADR header 上游仍锁定 WBS v4.19（ADR，5），而当前 parent WBS 为 v4.26；较早 Group WBS 的 Vercel 行也保留 “provider ... UNKNOWN” 措辞（WORKTREE-GROUP-NEXT-WBS，346），需要与新决策精确区分：Neon provider 选择已定，账号、project/branch/region、Owner runtime、roles/grants、pooler、migration identity 和 RLS 仍未知。 | 只更新版本/状态/引用，不回溯改写旧决策历史。让 SRS header 列出现有 ADR/BD/DD/TD；将 ADR/WBS 引用更新到当前基线；将 provider “已选择”与部署/Owner/RLS “未验证”分开表述。所有 SRS FR/NFR/GAP/AC IDs 保持原样。 |

## §2 需求到实现/验收的状态判断

| 范围 | 文档中已明确的目标 | 当前可确认的状态 |
|---|---|---|
| Project 与 Repository | Star Project Owner 单独持有 Project identity/member grants/association；0..N Repo；Run 可 hostless；解绑先迁移或显式退役，之后用 owner receipt、零引用 reconciliation 和 CAS 收口。 | 规范覆盖充分，需补 Vercel AC-VC-005/T-014 trace。WLD1 API/持久化/Host provider/数据库实现仍不闭环；WBS 记录的 HTTP 与本地 PostgreSQL fixture 结果是有限切片证据，不代表 Neon/Multipass VM 目标验收。 |
| Vercel 数据边界 | 不增加业务 Owner、不绕过 Star Owner；目标部署 provider 已选 Neon，K3s profile 是 Multipass VM PostgreSQL。 | provider 选择是设计决定；Vercel SRS/BD/DD 尚未同步到一致的 owner-runtime、API/direct DB、RLS/pool/migration 身份映射。两套真实数据库验收均未由本 lane 执行。 |
| Session/BFF/Gateway | BFF 只是 allowlisted transport；浏览器、BFF、Star API/Owner 与 Gateway 使用同一真实授权事实；chat SSE 路由需批准。 | 接口和来源身份仍未冻结；Vercel preview、account、登录、授权、Origin/revoke、streaming/recovery 尚无本 lane 的运行证据。T-049 NOT_RUN。 |
| 协作推送与冲突 | Owner commit wake-up → durable event relay → 授权的资源级 fan-out；保存和锁定双方精确 draft，双方有持久提醒与 participant-only diff；恢复使用 durable cursor。 | 规格明确目标，但现有审计记录浏览器固定轮询和单进程 hub；跨实例持久 relay、双草稿闭环、DB/RLS 和目标 p99 都未验收。Vercel T-049 不覆盖这些 owner-runtime gates。 |

## §3 建议的下一步顺序

1. **先同步授权与数据权威合同。** 更新 SRS/ADR/BD/DD/TD，明确 Neon 是 Vercel deployment profile 的目标；把 Star Project/Run Owner runtime、Owner API 与数据库访问路径、Pool/RLS context 和独立 migration identity 定义清楚。保持 no-second-owner/no-bypass-role。
2. **冻结真实 session 与通道边界。** 从嵌入宿主/Gateway 合同取得可核实的 session、授权刷新、Origin、撤权、关闭/重连、stream/recovery 事实；完成通道分类和唯一 chat SSE 路由。
3. **闭合 Project/Repository trace。** 将 WLD1 现有 hostless、0..N、migrate-before-unlink 的 FR/AC/TD 作为依赖映射进 hosted acceptance；在 WLD1 目标 API/Neon/VM DB 运行验证前维持不可用显示和 fail closed。
4. **保留协作门分层。** T-049 负责 hosted route/security；WLD-6/WLD-7 负责 owner truth、durable delivery、双 draft 与冲突语义、撤权和 PT-09；E2E-COLLAB-09 的 ≤1000 ms arrival trigger 不与 commit 后 p99 ≤1 s delivery/readability 合并。
5. **完成横向复核后才申请关闭 VCD-0。** 对齐 SRS/ADR/BD/DD/TD/WBS，更新 stale refs，复核 acceptance IDs 和每项 unavailable 状态；VCD-0 批准仍不代表 VCD-1 部署、VCD-2 Rust runtime 或数据库验收 PASS。

## §4 验证摘要

- 本 lane 执行了当前文档与既有 review/WBS 的只读交叉核对；本报告与 brief 的暂存差异已通过 rtk proxy git diff --cached --check。
- 本 lane **没有运行** Rust/前端测试、构建、静态分析、代码修改、Vercel 账户检查、Preview 部署、网络探测或数据库迁移/RLS/连接池测试。
- Vercel AC-VC-001..013 和 TD T-001..T-049 在 canonical test design 中仍 NOT_RUN；T-049 当前因真实 Gateway/session/Origin/chat SSE route contract 未批准而不可执行（TD，31、161–177）。
- 集成前 main/dev 比较由 parent agent 完成：五个 main-only commits 对当前 dev 均为 patch-equivalent（git cherry 标记为 -），未产生 cherry-pick，dev 仍为本 lane 基线 7a82c098。main 上 Vercel SRS v0.1/v0.2 已被 dev 同路径 v0.4 超越，本 lane 不回带旧正文；Neon 是已记录的用户目标选择，但 SRS GAP-06 对平台迁移事实的官方来源核验/同步仍开放。由于 dev 基线未移动，本 lane 以原基线继续，无需改写历史或重放。
- WLD1 及协作现有审计记录中的 source-level/本地 fixture 测试只按各报告所述范围引用；本 lane 未重跑这些测试，也不把它们扩写为 hosted Neon、Multipass VM、生产 RLS 或 cross-instance 验收。
- 文档静态审计和本文 diff 检查均不能关闭 VCD-0 的真实授权、目标部署、数据库/RLS、协作恢复、性能或商业运行门。

## §5 已知限制与守门

1. 本报告只提出闭合路径，不批准具体 issuer、token/cookie、Gateway wire format、Origin allowlist、chat SSE endpoint 或数据库连接凭证。
2. Neon 目标选择不授权 BFF/Next function 绕过 Star Owner；数据访问必须属于明确的 Owner runtime 和独立迁移身份。
3. Vercel 与 K3s 的 DB 数据、凭据、迁移状态分别隔离；测试阶段 Multipass inventory 总实例记录不超过 2，达到上限或 inventory 无法确认时 fail closed，不自动 purge。
4. remote file-browser 不因仓库里存在客户端组件而自动获准部署到 Vercel；channel inventory 需分类，路径和能力按 Star 当前授权。
5. 协作草稿与冲突文本默认不进入广播、BI、Plugin、日志或审计 payload；恢复必须重新授权并读取 owner 持久事实。
6. 设计文档、WBS 或 T-049 通过都不能把目前的轮询/进程内投递变成生产级 push，也不能证明双边草稿、跨实例 durable relay 或 ≤1 秒运行目标已达成。

## §6 评审签署

| 角色 | 状态 |
|---|---|
| 架构 / 本报告自审 | Mavis 接手 agent per DEC-008；仅确认静态证据与追踪结论 |
| SRE Lead | 本 lane 未执行运行/容量/可靠性验证，不作批准 |
| 平台 | 未执行 Vercel/Neon/Multipass 环境验收，不作批准 |
| 评审主持 | 由 parent agent 独立复核 |
| PM | 本报告不构成 VCD-0 或产品阶段批准 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-26 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 汇总 VCD-0 跨层合同缺口；追踪 Star Project Owner、hostless Run、0..N Repository、迁移后解绑、Neon/K3s VM 数据 profile、真实 session/Gateway/chat SSE、推送协作、双边草稿与两种一秒语义；区分设计目标与实现/运行证据。 |
