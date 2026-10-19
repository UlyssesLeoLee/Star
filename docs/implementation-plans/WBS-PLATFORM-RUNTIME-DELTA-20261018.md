# WBS Platform Runtime Delta — 2026-10-18

> 补充 `WORKTREE-GROUP-IMPL-PLAN-001.md`，仅拆分 BI/transport、Multipass/K3s、Agent 扩展运行时、性能与商业发行的未完成工作；不改变 Project → Engineering Run → Worktree → Branch 导航，不把源切片、隔离库或宿主 PoC 记为生产闭环。
>
> 状态：提议中的执行 WBS；以下能力除明确列出的源码/fixture 证据外均为未完成或未验收。目标分支基线由集成协调者复核。

## 1. 设计现状与权威证据

| 主题 | 当前 dev 证据 | 已有内容 | 尚未关闭的边界 |
|---|---|---|---|
| BI 原生事件链路 | `docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md` §§6.151–6.156；`docs/reports/PHASE-9F4I-NATIVE-EVENT-REPORT.md` §3 | BI owner Inbox/fact 与 allowlist projection；provider-neutral receiver core、Run scoped query source、BI read UI；native transport 的 `event` 类别授权基础 | BI 专用 native client、可信 route-pin/factory、Task sender/dispatcher、真实网络“owner 提交后 ACK”联测、目标身份与 DB/RLS、accepted settlement、恢复/checkpoint、fan-out、aggregate/Benchmark 和性能仍未完成；ingress 与执行保持 fail closed |
| Multipass/K3s | `docs/design/DD-LOCAL-INFRASTRUCTURE-001.md` §§1–3、§5、§7.19–7.20；实施计划 §6.156 顶部基础设施状态 | Multipass 首选本地/CI Linux guest provider；身份分层、Run 精确 cluster generation 绑定、双 owner reservation contract、只读 inventory 与部分 admission/delivery primitives | 权威 Rust Manager/provider、生产 resolver 与 owner consumer、target DB/grants、集群生命周期/readiness、workspace bridge、sandbox、恢复与负载验收未闭合；测试 inventory 总记录数上限为 2（包含 Stopped/Deleted），不能自动 purge |
| Plugin / Workflow runtime | 实施计划 §1 “Group Chat + LangGraph”“Plugin 热插拔”与 §WG-ACC-07/08/17–19；实施计划状态摘要 | Workflow 与 app registry 分离的设计；插件 registry schema/API/UI 的部分源码；禁用新调用、撤权、resume 重新授权的验收约束 | 生产 capability gateway/隔离执行、版本与 digest 固定、启停/卸载/热撤权到实际调用路径、LangGraph checkpoint/resume 与当前 RunContext/Hook/Plugin 权限联测尚未形成闭环 |
| Rust 桌面资源 / 商业发行 | `docs/design/DD-LOCAL-INFRASTRUCTURE-001.md` §§2、§5；`docs/design/DD-LOCAL-INFRASTRUCTURE-001.md` §1 | 有界队列、按需执行、分页状态、有界日志与 digest cache、公平准入的设计；完整 SPDX/SBOM、许可履约和近 12 个月维护证据规则 | Rust 桌面/Host/Agent/K3s 总预算、可观测阈值与负载基线未验收；实际打包闭包 SBOM 和许可 obligations 未逐发行物验证 |

### 1.1 9F4J.1 native Event adapter review evidence

Local `dev` already contains the 9F4J.1 native Event adapter source patch at `4ed8e6f891e4e1fcfdb1b5d86e48d678e26889fa`. Private lane commit `3c7fa58c8bce4b5083460ee24bbffa6307acd7d1` was recognized as already applied and skipped when rebased onto `59fbdacbb106898fded81f5d0455b53d43dbaff3`. This follow-up records review, revalidation and documentation only; it does not duplicate or modify the source patch.

The adapter uses unary `PullOne`, `VerifyOrigin` and `AcknowledgeOne`. One outstanding permit limits each route to a single unreleased delivery, without background tasks or prefetch. Client request timeout is capped at 10 seconds; encoded/decoded frames are capped at 32 KiB; delivery and `VerifyOrigin` exact wire payloads are capped at 16 KiB. Endpoint tenant must match the fixed scope and the profile endpoint identity/digest and mTLS CA/server-name/client-identity configuration are checked. Pull, Verify and ACK requests pin the Profile digest. Event origin identity derives from provider ID/version, profile digest and endpoint identity. Before ACK, each delivery's sender, Event kind, log ID/generation, sequence, message ID, lease ID/attempt and exact-wire SHA-256 are checked; the ACK pins sender, kind, log generation, sequence, lease, message and digest. The adapter does not access BI owner storage or ACK automatically.

Two production integration gates remain open. `receiver_service_id` is local-only and absent from the Pull wire; the server resolves receiver identity from the authenticated client certificate fingerprint. Trusted resolver/profile composition must prove that fingerprint maps to the configured local receiver. Also, `acknowledge()` does not require a typed owner-commit receipt or a prior verification token; caller-owned ordering remains a contract, not an adapter-enforced Inbox commit-before-ACK guarantee. No production profile loader, certificate rotation, target DB/RLS/grants, owner persistence, process recovery or live network proof is recorded. BI ingress and task execution remain fail closed until all 9F4J gates pass.

Post-rebase evidence: Rust 1.99.0 focused crate check passed (2 existing dead-code warnings); 50 library tests passed; all-targets Clippy passed with warnings only in unchanged files/dependencies; changed-file rustfmt passed. Crate-wide `cargo fmt --check` remains blocked by pre-existing drift in five files outside this lane manifest. The original source commit recorded the Cypher header guard for two Rust files; this documentation-only follow-up guard checked zero source files. See `docs/reports/PHASE-9F4J1-NATIVE-EVENT-REPORT.md` for exact commands and outcomes.
## 2. 执行包

| WBS ID / 顺序 | 工作包与交付物 | 依赖 | 可并行边界 | 可核验完成门 |
|---|---|---|---|---|
| **PR-1 / P0** | **BI native transport 闭环**：实现受信身份与当前 Profile/endpoint resolver、BI native client 和 route-pin verifier；接通 Task source dispatcher → native event append/pull → BI owner Inbox/fact durable commit → ACK/accepted settlement。定义 provider-neutral receipt、幂等 replay、consumer checkpoint/gap recovery、quarantine disposition 和 Run BI/Benchmark 投影。 | 9F4I `event` 授权基础；9F4E/9F4F owner 持久化；9F4G receiver/query source；生产 identity 与目标 DB/RLS provisioning | 与 PR-2 的 Manager/VM 实现可分开；BI UI/Benchmark projection 可在固定事件 contract 后并行，但不得另造事实 owner 或 ACK 语义 | Rust 1.99.0 focused/workspace gates；隔离 PG 与目标 DB 两类结果分开报告；真实 transport 网络联测覆盖 commit 前故障不 ACK、commit 后崩溃重投返回同一 receipt、scope/route/wire 冲突拒绝、授权撤销不 ACK；恢复后 checkpoint 与聚合可重建；公平性、bounded bytes、RSS/延迟和生产 grants 均有证据。未全过前 BI ingress/执行保持关闭 |
| **PR-2 / P0** | **Multipass/K3s owner 与 Manager**：完成 trusted Infrastructure resolver、owner Inbox/consumer、Host+cluster ledger/reservation 与过期恢复、版本化 Profile placement、受限 Rust Multipass Manager、operation lease/audit/outbox、Run binding、aggregate K3s readiness 与受授权 workspace bridge；独立接 Agent Runtime sandbox provider。 | 既有 Infrastructure schema/contract、profile digest、host/cluster reservation 原语；owner 身份、target DB/grants；PR-4 的资源预算门 | 不依赖 BI UI；可与 PR-1 并行开发，但共享 transport 只通过版本化 provider-neutral owner API/Outbox/Inbox 契约；最多两个测试 inventory 记录 | 两记录上限按全部保留记录计数（含 Stopped/Deleted）；并发 launch 经跨进程锁，无效 inventory/达限 fail closed 且绝不 auto-purge；两个 owner 各自在本事务内幂等处理、不跨 owner 写事务；Run 精确绑定 cluster ID+generation，Host/cluster 双 reservation 独立确认；过期/崩溃恢复不产生超额容量；manager 不向 Agent/Plugin/CLI 暴露 host shell、daemon/socket 或任意 mount；无当前 ACL、binding、Hook、双 reservation、K3s/CRI 和 sandbox readiness 时不启动 TaskExecutionRun |
| **PR-3 / P1** | **Plugin 与 Workflow/Agent 运行时闭环**：版本化 manifest/capability 与隔离 runtime；Group App Registry 到 gateway 的真实调用、enable/disable/uninstall、热撤权和保留 EntityRef；LangGraph start/checkpoint/resume 每次重载当前 Actor/Run/Worktree grants、HookSet、Plugin compatibility/digest 和资源预算；完善 Memory/Skill/Context/Validation/Loop 的版本/证据接口。 | RunContext/ACL、PR-1 的事件/Evidence contract、PR-2 sandbox/readiness、原生 Hook policy/evaluator 与审计 | Manifest/schema 和 UI 可并行；实际调用 gateway、Hook 与 resume 安全测试必须汇合后才可启用能力 | 端到端证明：权限撤销立即拒绝新/恢复调用；禁用插件后不再发 capability，但历史业务事实和引用保留；checkpoint 只恢复计算状态，不恢复旧授权；未知/不兼容 digest、缺 Hook/验证器、越界预算 fail closed；取消/drain 与重启恢复可审计；插件隔离测试证明不能获得主进程/Multipass/任意宿主 shell 能力 |
| **PR-4 / P1** | **Rust 桌面资源与商业发行门**：将 Desktop/Manager/Agent/guest/K3s/未托管负载预算落为可测配置；实行虚拟化、增量分页/按需读取、有界队列/缓存/日志、fair scheduling/backpressure/cancel/drain；建立可复现负载与 SPDX/SBOM/许可义务发行流程。 | PR-1/2 提供端到端样本与资源指标；PR-3 提供 Agent/Plugin 并发场景；发布流水线与打包清单 | SBOM/license inventory 可独立于 runtime 功能并行；最终内存/延迟预算需以统一端到端场景和目标设备确定 | 预先批准 Desktop RSS、CPU、UI frame/p95 更新、事件延迟、队列/缓存上限与 1/2/4/8 Agent 场景阈值；在测试 2 Multipass 记录预算内实测冷启动、峰值、长跑漂移、取消与恢复，并发布原始结果。每个发行包生成完整 SPDX/SBOM，覆盖传递依赖、随包/默认下载工具、镜像、guest/kernel/userspace/firmware，digest 固定；核对对应修改、链接、分发义务与 NOTICE/源码/安装信息；近 12 个月 release/maintenance 和公开社区活动证据留档；未知许可/义务未满足的打包形态阻断发行，但不限制产品商业用途、席位或用量 |

## 3. 依赖与并行排程

```text
PR-1 BI native E2E ─┐
PR-2 Infrastructure ├──> PR-3 Agent / Plugin / Workflow E2E ──┐
PR-4 budgets + SBOM ┘                                         └──> integrated release gate
```

- **第一批并行**：PR-1 的 trusted BI transport/receipt/recovery；PR-2 的 resolver/Manager/owner admission；PR-4 的目标设备指标定义与发行闭包 inventory。三者先冻结跨 owner API、事件版本、资源预算和验收数据格式。
- **第二批汇合**：PR-3 在 PR-1/2 的真实授权、事件与 sandbox seams 上闭环；PR-4 用已接通路径执行并发、RSS、取消、恢复与发行验证。
- **最终发布门**：PR-1 至 PR-4 的目标 DB/身份/RLS、实网络、故障恢复、Hook、sandbox、资源与合规证据全部具名审查；未通过项保留为明确 blocker，生产消费/执行不得因 UI、fixture、隔离数据库或 PoC 而启用。

## 4. 追踪规则

1. 每一工作包分别记录 source/fixture、isolated PostgreSQL、target DB、真实网络/VM、运行性能和发行审查证据；不同层级的通过结果不互相替代。
2. 每个 PR/WBS 子项明确事实 owner、授权身份、API/event/schema 版本、outbox/inbox/idempotency、限额、取消/drain、恢复、回滚与文档更新。跨 owner 不共享写事务。
3. 任何消息 provider 可替换；业务契约不依赖 NATS API。自有 Rust transport 是最终替代方向，NATS 只能保留为 adapter。
4. 真实运行界面可展示 provider 事实名；产品导航、帮助和对标文案使用 Star 自有中性能力名称。
5. 本 delta 是 WBS 拆解，不代表已开工、已验证或完成；统一主 WBS 的版本与合并由协调者处理。
