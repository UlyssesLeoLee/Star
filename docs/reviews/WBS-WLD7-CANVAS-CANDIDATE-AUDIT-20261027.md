# WBS-WLD7 Canvas 候选文档差异审计

- 日期：2026-10-27
- 范围：只读审阅旧 WLD-7 Canvas 候选与当前本地 dev 文档
- dev 基线：9494e842f06206c9112148f722bdbcb9756761fb
- 候选：codex/wld7-canvas-a12-alignment-20261025，提交 5e5463bb12cf34a53d779e63f7ddc1b139236b76
- 候选与 dev 的共同祖先：6b7543e17fd0f64f23f94ddea9807293167843aa
- 结论：候选的 owner、revision、草稿保全和资源授权方向有可复用内容；不得整分支或整文件照搬。先修正当前 WBS 指出的到达时序、Resolve 验收和版本引用，再以新隔离文档 lane 做最小同步。

## §0 范围与方法

本审计只比较以下三份候选文档与 dev 基线的对应文件，并只在本报告中写结论：

- docs/requirements/SRS-CANVAS-AGENT-001.md
- docs/design/BD-CANVAS-AGENT-001.md
- docs/design/DD-CANVAS-AGENT-001.md

同时查阅当前 dev 的 WBS §2.1、§22.2、§23.2–§23.4，WLD1 测试设计 §2.3/§7，Canvas 测试设计 v1.7 §4.8，以及 W16 事件推送审计 §1–§5。旧候选仅从提交对象读取；未访问其工作树，不读取或触碰 D:\Star 中用户保留的移动/删除文件。

## §1 差异摘要

| 文档 | 候选版本 | dev 基线版本 | 变更量（dev 基线对候选差异） | 观察 |
|---|---|---|---|---|
| SRS-CANVAS-AGENT-001 | v1.4 | v1.2 | +117 / −274 行 | 保留 A12 范围，新增当前协作契约并将旧 transport、CRDT/LWW、旧表结构标为历史；A12 需求语义和验收引用也有较大改写。 |
| BD-CANVAS-AGENT-001 | v0.2 | v0.1 | +61 / −37 行 | 新增 §1.5 owner、Run/resource scope、Profile 与低消耗协作绑定；旧接口和 schema 多处改为历史方案。 |
| DD-CANVAS-AGENT-001 | v0.2 | v0.1 | +113 / −38 行 | 新增 §8.7 当前详细契约；旧 WSS、路由、schema 和时序仍保留为历史提案。 |
| 合计 | — | — | +291 / −349 行 | 是明显的规范重写切片，不是可自动接受的文档小补丁。 |

候选保留了 FR-A12.1–A12.8 的编号，但这些编号下的规范语义、验收锚点、旧 endpoint/schema 的权威性和测试覆盖发生变化。下游引用必须一起核对。

## §2 Keep / Adapt / Reject 矩阵

| 候选内容 | 当前 dev / 用户架构证据 | 决策与原因 | 新文档中的落点 |
|---|---|---|---|
| SRS §4.12.1、§8.5；BD §1.5；DD §8.7.1/8.7.5：业务 owner 在自己的事务中提交 revision、audit、Outbox；通知只携带轻量版本事件，relay 可合并唤醒；连接与事件按资源授权范围 fan-out。 | WBS §22.2、§2.1 W16 与 W16 审计确认 owner 事实、事件推送和同 Run/资源的授权范围是目标；当前运行仍由 CanvasOutboxPoller 轮询且 BFF hub 进程内。 | **Keep** 逻辑 owner/outbox 契约和“不广播到无关 Run/资源”的边界。继续标为 design-only，不能据此声称 push、持久 relay 或跨实例恢复已实现。 | SRS §4.12.1/§4.12.8；BD 新增的 A12 当前绑定节；DD §8.7.1/§8.7.5。 |
| SRS §4.12.6、BD §1.5 A12.6、DD §8.7.2/§8.7.3/§8.7.6：完整的重叠提案立即生成持久冲突，封存双方稿件、common base、双边 unread 和稳定 conflict_id；较晚的重叠仍检测。 | WBS §22.2 与当前 requirements v7.08 已明确：≤1000 ms 是第二份完整提案抵达 owner 的测试刺激，不是 commit-time 时间差、检测窗口或 holdback；PT-09 的 p99 ≤1 s 从冲突提交开始计时。 | **Adapt**。候选把 ≤1000 ms 写成 server commit time 的差值，与当前定义不一致。改用 owner 到达第二份完整提案的时点；任意间隔的语义重叠都必须检测；把冲突判定与 p99 投递目标分开表述。 | SRS §4.12.6/§9.1；BD A12.6；DD §8.7.2/§8.7.3/§8.7.6。 |
| SRS §4.12.3/§4.12.7、DD §8.7.2–§8.7.3：作者未完成草稿默认私有； sealed 冲突稿只向授权参与者开放；所有订阅、草稿/差异读取、恢复和解决都重新校验授权；撤权后取消订阅并拒读。 | 当前用户架构要求 Project → Engineering Run → Worktree → 本地 Branch；Run 可无宿主 Branch；远端 provider 只是可选元数据。TD-WLD1 v0.3 §2.3/§7 也把 hostless Run 的身份、授权和管理与远端仓库/Branch 解耦。 | **Adapt**。保留 RunContext 和逐次授权；补上父子导航与业务 scope：Canvas 和任务/工作项能力是选定 Run/Worktree 下的同级 App，Branch 是 Worktree 本地 checkout 状态，不能成为 Run 父级或授权前提。 | SRS §1.2–§1.4、§3.2、§4.12.7；BD §1/§1.5、§2 架构图；DD §8.7.1/§8.7.3。 |
| BD §1.5、DD §8.7.4：单个多路复用连接，队列同时按事件数量和字节数限界；presence 短 TTL、合并限频；Profile 固定资源预算；慢消费者可恢复。 | WBS §2.1 W16 与 §22.2 要求版本化 Profile、背压、恢复和测量；100 subscriptions、64 events、128 KiB 仍只是候选，PT-09 未运行。WBS §23.2 另将最多两个 Multipass inventory 实例作为当前测试阶段上限。 | **Keep**, 但只能保留为有界设计要求。候选数值不能变成已接受容量；双 gateway/目标 PostgreSQL 验收与 Multipass 两实例资源限制都要单独注明。 | BD A12 Profile 表；DD §8.7.4/§8.7.6；验收引用当前 PT-09 和测试设计。 |
| SRS §7.3、§8.5；BD §4.3、§5.1/§5.2；DD §7.1/§9.3 的旧表、REST/WSS endpoint、Envoy、CRDT/LWW 与固定容量/延迟提案。 | 候选 §1/§8.7 声明逻辑 owner 契约优先，旧 A12 schema、路由和 transport 仅为历史；当前 WBS §22.2 也拒绝把候选整体合入。 | **Reject** 作为当前 requirement、实现事实或 migration specification。可保留历史记录，但必须持续标注非规范，不能让旧表或 endpoint 和新契约并列生效。 | SRS §1 当前规范优先级与 §7/§8；BD §1.5；DD §8.7。 |
| 候选仅映射 AC-COLLAB-01–08、E2E-COLLAB-01–09；还引用 spec v1.1、数据设计 v0.3、测试设计 v1.6。 | WBS §22.2、当前协作 spec v1.2、owner-local data design v0.4、Canvas test design v1.7 要求 AC-COLLAB-09、E2E-COLLAB-10/11：可变换的文本重叠仍建立冲突；OT 只作无副作用预览，用户确认后以当前 revision CAS 提交；取消无副作用，stale CAS 保留双方草稿/冲突。 | **Adapt**。候选验收缺 AC-09/E2E-10/11，版本引用落后；不能按旧 8/9 用例集形成完成门。 | SRS §9.1/§10；BD §1.5 的验收映射；DD §8.7.6 和交叉引用/修订记录。 |
| 候选整体三文件替换。 | 三文件共 +291/−349 行；candidate 保留很多 legacy 段并改其规范权威性。当前 WBS 指定选择性复用，而不是无差别重写。 | **Reject** 整体合入。只选择矩阵中 Keep/Adapt 的 current sections，保留原历史修订轨迹，并逐项核验引用、ID、AC、数据和测试映射。 | 作为新的、文件清单明确的 SRS/BD/DD 文档 lane；本审计不改设计文件。 |
| 候选遗留段中的外部产品比较或命名类比。 | 用户要求产品表面与规范文案不出现竞品名或仿制式宣传。 | **Reject** 这类措辞进入当前规范；历史摘录只能作为非规范记录且不应继续复制到产品面说明。 | 未来同步时统一改为中性能力描述，不在报告中复述名称。 |

## §3 证据与限制

已检查的当前 dev 材料：Canvas SRS/BD/DD 基线、WBS §2.1、§22.2、§23.2–§23.4、TD-WLD1 v0.3 §2.3/§7、TEST-DESIGN-WORKTREE-CANVAS v1.7 §4.8，以及 WBS-W16-EVENT-PUSH-AUDIT-20261027 §1–§5。候选比较限定为列出的三个文档，使用共同祖先与候选提交对象，未读旧候选工作树。

W16 审计报告给出的有界实现证据包括：Group projection 以 1,000 ms 间隔运行 CanvasOutboxPoller；浏览器 localStorage 游标只是非可信恢复提示；BFF 广播为进程内状态；限定源码搜索未证明 durable draft/conflict 和跨实例链路；Canvas 与 tenant 身份字段比较可误过滤合法事件，但这不是跨租户泄漏证据。报告列出的源码范围含 frontend/src/lib/group/groupProjection.tsx、frontend/src/lib/group/worktreeGroupApi.ts、crates/star-api-rest/src/group_api/canvas.rs、bff/src/collaboration/wss_hub.rs 及 DD-WORKTREE-GROUP-001.md §8.27。

本工作树没有 .codegraph 索引；不对索引覆盖或全仓代码作完备性断言。实现状态以集成的 W16 有界审计为证据边界。文档验证仅包含静态审阅及 rtk git diff --no-index --check（新文件以 NUL 为对照）；命令退出码 1 表示文件有差异，输出无空白错误，仅有 CRLF 转换提示。没有运行测试、构建、数据库、VM、浏览器、桌面或性能验证。

## §4 推荐的最小文档改动与依赖顺序

1. 先在新的 SRS 文档 lane 将 A12 当前逻辑契约合并为唯一规范：owner-local事务、Outbox wake-up、双稿冲突、精确授权和 Run/Worktree scope；按当前 requirements v7.08 将 ≤1000 ms 定义为到达刺激，并同步 AC-COLLAB-09、E2E-COLLAB-10/11。
2. 在 BD 增加与 SRS 一致的 A12 owner / Collaboration 分工、Project → Engineering Run → Worktree → 本地 Branch 层级、Canvas 与任务/工作项并列 App、hostless Run 与可选 provider 元数据约束，以及有界 Profile；所有候选容量标为待测。
3. 在 DD 将候选 §8.7 作为可选择的结构起点，补齐显式 Resolve 的预览/确认/取消/CAS/stale 语义，更新 AC/E2E/PT 锚点；把旧路由、schema 和物理 transport 留在明确的历史区域。
4. 更新三份文档引用到当前 source versions：domain collaboration spec v1.2、WLD-7 data design v0.4、Canvas test design v1.7，以及 WBS §22.2 的 AC/E2E 追踪。同步各自修订历史，不改写旧版本叙事。
5. 由独立审阅确认 SRS→BD→DD→测试映射与 Worktree 导航一致；通过后再单独推进 owner API / durable delivery 实施。WLD-7 和 PT-09 继续 OPEN/NOT_RUN，不能由文档工作关闭。

## §5 已知缺口

- 候选里的冲突时间表述需从 server commit-time 差值改为第二份完整 proposal 到达 owner 的测试刺激，并与 p99 送达目标分开。
- AC-COLLAB-09、E2E-COLLAB-10/11 与当前 resolve 无副作用约束未在候选追踪中完整体现。
- 当前用户要求的 Project / hostless Run / Worktree / 本地 Branch 主导航关系尚未在候选 A12 文档绑定中明确写出；实施 lane 必须从当前 WLD1 与产品架构继承。
- durable owner Outbox、push、双 gateway、目标 PostgreSQL/RLS、draft/conflict persistence、授权撤销、cursor recovery、资源预算与 PT-09 均未验证。
- 当前 WBS 的 1 秒 p99 服务目标与 ≤1000 ms 提案到达测试刺激是两个不同验收维度，必须保持分开。

## §6 审阅与签署状态

| 角色 | 状态 | 结论 / 边界 |
|---|---|---|
| 架构 | 本次有界审阅完成 | 接受选择性吸收方向；拒绝整组三文档原样合入。 |
| SRE Lead | NOT_RUN | 双 gateway、重启/恢复、RSS/CPU、队列背压与 p99 均未测量。 |
| 平台 | NOT_RUN | 未运行目标数据库迁移、RLS/grants、事务/恢复或实际 VM 验收。 |
| 评审主持 | 本次范围复核完成 | 结论仅基于 §3 列出的提交、文档与受限源码证据。 |
| PM | 待后续设计 lane | 需先看到对齐 AC-COLLAB-09 和 E2E-COLLAB-10/11 的三文档结果，再审阅设计同步状态。 |

- 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
- 审批状态：本报告只给出审计建议，不代表人工产品批准或运行验收。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WLD-7 Canvas 候选文档有界差异审计；给出 Keep/Adapt/Reject 与新 lane 最小同步顺序。仅静态文档审查；未运行测试、构建、数据库、VM、浏览器、桌面或性能验收。 | WBS-WLD7-CANDIDATE-AUDIT-20261027 brief |
