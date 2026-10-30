# PHASE-WBS-W16-NEXT-AUDIT-20261030

> 状态：只读审计完成；W16 / WLD-7 维持 OPEN / PARTIAL。
> 审查基线：dev@313e5741fb0dc557874e5a37bc05e8839211cd90

## §0 目的

按 WBS-W16-NEXT-AUDIT-20261030 brief 核对 W16 当前实施、验收记录、规范状态、MAGOS path claims 与 Orca terminal inventory；提出一项依赖就绪的下一步工作。此报告不关闭 W16-A、W16-B/C、W16-D/E 或 WLD-7。

## §1 改动矩阵

| 文件 | 内容 | 状态 |
|---|---|---|
| docs/reviews/WBS-W16-NEXT-AUDIT-20261030.md | 记录当前证据、优先差距、规范矛盾、下一条 acceptance lane 与限制 | 完成 |
| 本阶段报告 | 记录审计方法、验证门与未完成项 | 完成 |
| brief 与 dispatcher log | 此 lane 已注册输入 | 未修改 |
| WBS、SRS、BD、DD、源码、迁移与既有候选工作树 | 本 lane 未获写入范围 | 审计 worktree 内未修改 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 当前 branch / baseline | agent/wbs-next-20261030/w16-audit；HEAD 为基线 dev SHA 313e5741fb0dc557874e5a37bc05e8839211cd90 |
| Git 状态 | 初始仅本 lane brief 与 dispatcher log 未跟踪；本 lane 只新增已登记 review 与 phase report。 |
| CodeGraph | 当前 worktree 无 .codegraph/，查询返回未索引；按约束停止并使用有限路径读取/搜索。 |
| MAGOS claims | 只读 list 确认 PG device-context 与 OAuth PG acceptance 仍 active；具体路径见 review §3。新 proposed 9-path HTTP acceptance lane 的 check 返回无重叠、未登记。远端 tracking refs 只截至上次 fetch，backstop 未安装。 |
| Orca W16 terminals | w16-a-device-auth-20261028、w16-a-pg-device-context、w16-a-security-status-sync-20261029、w16-b-kms-contract-20261028、w16-b-release-closure、w16a-doc-sync-20261029、w16a-oauth-pg-acceptance-20261029 七个既有候选工作树均 inactive，liveTerminalCount=0、unverifiableTerminalCount=0、agents=[]；本次未向其发送输入、未改动或清理。 |
| Canvas SSE PG | 当前 dev 的 phase report 记录 disposable PG18.6 run 49b512fbfe334f0e98e46fe7eae7ea39：41/41 runner steps 成功，精确用例 1 passed、0 failed、0 ignored，0.17s。仅该窄用例 PASS。 |
| W16-A / OAuth PG | 现有报告分别记录 run 6af04e5a765446c8b3baaffff126bd52 的 70/70 harness steps 和 run 1b4673269a04435daa1fab5c93333133 的 OAuth 3/3；均是 disposable local PG 局部证据。 |
| 静态路径核对 | 找到 authenticated revoke route、单事务 generation/status + refresh family + audit 写入以及通用 device wake trigger；定向搜索未发现 API tests 对该 revoke route/audit 的直接验收。 |
| tests / Cargo / DB / runtime | NOT_RUN；brief 明确禁止。 |

当前 dev 证据见 docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md §23.13、docs/reports/PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md 与 docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md v2.1。规范状态漂移见 SRS/BD/DD v0.4 的旧 candidate 记录。路径 blob SHA 与确切限制见同目录 review report §2。

## §3 已知缺口

1. HTTP device-revoke handler 与 audit row 没有直接 PG/API acceptance；前次静态报告 F-02 在当前测试树搜索后仍成立。
2. DD §1 把 Outbox 列为 Collaboration owner 事实，DD §2.3 要求 Identity owner 与 device revoke 同事务写 Outbox；当前迁移触发器只发 generic pg_notify。durable Outbox 设计/实现前需先收敛 owner 权威、版本化事件和恢复/消费契约。
3. SRS/BD/DD v0.4 仍描述 Canvas SSE candidate 未集成或未运行；WBS §23.13 和 TD v2.1 已记录窄 PG 用例通过。规范状态需另行同步，不可把历史 §23.12 当当前状态。
4. 本地 disposable superuser/test-only placeholder 结果不能证明目标应用角色的 production grants/RLS、生产 audit/SCD2 或生产部署。
5. W16-B 尚无生产 provider 与 release closure；W16-C core 未接入生产 owner/persistence/provider；W16-D drafts/conflicts、W16-E 双实例恢复/目标环境/性能仍未验收。
6. 没有运行本地或远端生产环境，没有刷新远端 refs；p99、RSS/CPU、backpressure、Neon、Multipass/K3s 和完整 E2E 均未验证。

## §4 子代理失败接手清单

- 本只读审计没有派发子代理；失败/接手项：N/A。
- 其他并行 lane 的工作区保持原样。

## §5 守门规则与下一步

**唯一建议的下一 W16 lane：W16-A authenticated device-revoke HTTP + audit PostgreSQL acceptance。** 当前代码已经有 route 与事务性 repository 路径；缺少直接 API/audit 运行证据。它是本次确认可先行的最小 acceptance 实现，不能冒充 durable Outbox 或完整 W16 闭环。

建议新 lane 的精确 9-path manifest：

1. crates/star-api-rest/tests/device_revocation_http_pg.rs
2. scripts/automation/w16a_device_revocation_http_pg_acceptance.py
3. scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py
4. docs/automation-design.md
5. scripts/automation/registry.md
6. docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md
7. docs/briefs/W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
8. docs/reports/PHASE-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
9. docs/reports/WBS-DISPATCH-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.jsonl

MAGOS check 对上述路径给出“无重叠（未登记）”。它不等于注册或创建；开始 lane 时须以当前 HEAD/claims 重新 check/plan。新测试应自带 fixture，避免修改 OAuth lane 声明的 tests/common/mod.rs、oauth_device_auth_pg.rs 与 oauth_refresh_tokens.rs。

至少验收：实际 authenticated HTTP route；合法撤销返回固定契约响应；同一事务更新 device generation/status、撤销相关 refresh family 并恰写一条 append-only audit；无效 caller 与跨 actor/tenant 请求不产生部分事实；并发撤销最多一条状态转换与审计；使用受限 app role 验证测试范围内的 owner/RLS 行为。重复请求响应语义需先在 API 契约中冻结，不能从现状擅自推断。Disposable PG harness 必须 fail closed、只连自有 loopback 数据库，且 superuser schema setup 不作为生产授权证据。

独立后续门：在任何设备失效 durable Outbox 实施前，先修订并评审 DD owner 归属、事件契约与 replay/gap recovery 语义；实现后再验证 HTTP revoke 到两 gateway SSE fail-closed、listener loss/reconnect 和 catch-up。随后同步 SRS/BD/DD/WBS/TD 的真实状态。W16-D/E、生产 RLS/provider/部署和性能门均保持 OPEN。

守门：本 lane 未运行测试或服务；未访问生产数据库；未清理现有 worktree；未对 W16 或 WLD-7 作完成声明；不 push。一次默认目录 Add File 调用的两个请求路径为 docs/reviews/WBS-W16-NEXT-AUDIT-20261030.md 与 docs/reports/PHASE-WBS-W16-NEXT-AUDIT-20261030.md；可能已写入默认工作区，未检查或清理，已交由协调者单独处理。

## §6 签字栏

| 角色 | 审核者 | 结论 |
|---|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008） | 审计完成；Outbox owner 冲突保持显式开放。 |
| SRE Lead | Mavis 接手审核 | 仅复核现存报告；无目标环境、恢复或性能验收。 |
| 平台 | Mavis 接手审核 | 未修改 Rust service、Multipass/K3s 或 Neon 配置。 |
| 评审主持 | Mavis 接手审核 | 记录证据范围与未知项；后续 MAGOS check 需刷新。 |
| PM | Mavis 接手审核 | 只推荐一项可先行 HTTP/audit acceptance lane；W16/WLD-7 仍开放。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W16 read-only readiness audit 与下一步 lane 建议；tests 明确 NOT_RUN。 | WBS-W16-NEXT-AUDIT-20261030 brief |