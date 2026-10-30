# WBS-W16-NEXT-AUDIT-20261030

> 状态：🟡 只读就绪度审计完成。W16 与 WLD-7 仍为 OPEN / PARTIAL；本报告不代表生产验收。
> 审查基线：本地 dev @ 313e5741fb0dc557874e5a37bc05e8839211cd90
> Lane：agent/wbs-next-20261030/w16-audit

## §0 目的与范围

只读核对当前 dev 的 W16 需求、基本设计、详细设计、测试设计、实施记录、事件唤醒/设备撤销源码和并行工作登记，确定一项依赖就绪且路径不重叠的下一步工作。未运行测试、Cargo、数据库、迁移或应用运行时；未修改源码、WBS、SRS、BD、DD、现有候选分支或候选工作树。

CodeGraph 返回当前 worktree 未索引（没有 .codegraph/），故按仓库约束改用精确文件读取和有限路径字符串搜索；没有图谱覆盖结论。未读取 D:\Star 工作区文件。一次默认目录 Add File 写入是否落入该工作区尚未核实；该调用后未检查或处理默认工作区，已告知协调者。

## §1 当前 W16 状态与优先缺口

| 优先级 | 工作流 | 当前 dev 证据 | 未关闭的边界 |
|---|---|---|---|
| P1 | W16-A 设备身份与 owner/RLS | PHASE-W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028.md 记录 disposable PostgreSQL 18 run 6af04e5a765446c8b3baaffff126bd52 共 70/70 步骤、设备 repository 2/2，并包含所选 owner/ACL/RLS 与 Run/Worktree binding 场景。PHASE-W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028.md 说明这些是局部本机证据。 | 生产 grants/RLS、生产审计/SCD2、并发 revoke 等待及完整设备身份 E2E 未验证。 |
| P1 | OAuth repository | PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029.md 记录 disposable PostgreSQL 18.6 run 1b4673269a04435daa1fab5c93333133 的 3/3 named tests 通过。 | superuser 与 test-only prerequisites 不证明生产 role/RLS；HTTP OAuth flow、E2E-COLLAB-14 与目标部署仍 OPEN。 |
| P1 | Canvas SSE 撤销失效 | WBS §23.13 和 PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md 记录候选最终提交 4bd9b6704e4866f052dc97d6b2cf8394c847b43a 已进入本地 dev；PG18.6 run 49b512fbfe334f0e98e46fe7eae7ea39 的精确用例 1/1 通过，runner 41/41 步骤成功，测试耗时 0.17 秒。 | 测试通过只关闭一个单例断言；没有证明真实 HTTP revoke/audit、多 gateway、listener-loss/replay、生产 grants/RLS、目标部署、p99 或完整 E2E。 |
| P1 | HTTP 设备撤销与审计验收 | 当前实现含 POST /oauth/device/{device_id}/revoke 路由；repository 在单事务中推进 generation/status、撤销 refresh family 并插入 revoked audit。对 crates/star-api-rest/tests 的定向搜索未发现该 HTTP 路由或 oauth_device_binding_audit 的直接测试。静态复核报告 W16-A-DEVICE-REVOKE-API-REVIEW-20261030.md 的 F-02 仍成立。 | 尚无经认证 HTTP handler 对成功/拒绝、审计行内容/数量、回滚与并发结果的 PostgreSQL 验收。该缺口是当前最窄、可独立验证的下一片。 |
| P1 | 撤销失效事件的持久化 | DD §2.3 要求 Identity owner 在撤销事务中写 Outbox 并通知 API 节点。当前 2026-10-28-w16-device-auth.sql:34–37 仅以通用 pg_notify('star_device_binding_wake','changed') 唤醒；Canvas SSE 的 60 秒重验是有界授权复验，不是持久事件重放。 | DD §1 的模块表将 Outbox 列为 Collaboration owner 权威事实，而 DD §2.3 又要求 Identity owner 同事务写入；必须先收敛 owner 边界，不能静默选择跨 owner 写入或把 NOTIFY 当作 Outbox。 |
| P1 | W16 文档状态一致性 | WBS §23.13 与 TEST-DESIGN-WORKTREE-CANVAS-001.md v2.1 已记录 Canvas SSE 单例 PASS。 | SRS-W16-PAYLOAD-SECURITY-001.md、BD-W16-PAYLOAD-SECURITY-001.md、DD-W16-PAYLOAD-SECURITY-001.md 的 v0.4 状态仍写 candidate 未合入或 NOT_RUN；需独立文档同步。历史 §23.12 可保留为当时快照，不应覆盖或改写历史。 |
| P1 | W16-B / W16-C 生产闭环 | W16-B report 只证明 provider-neutral KMS contract；W16-C report 记录 domain-kms envelope core 的 21/21 包级单测。 | 没有获批生产 provider/profile resolver、持久化 owner、真实 KMS、跨进程/重启恢复、轮换、备份恢复、商业发行 SBOM/NOTICE 完整闭包。LocalMockKms 不可作为生产回退。 |
| P2 | W16-D / W16-E 与 WLD-7 | WBS §23.7–23.13 明确草稿/冲突主链尚未完成；W16 测试设计仍将 E2E-COLLAB-12..19 与 PT-09/10 保持 OPEN / NOT_RUN。 | 加密草稿、双方冲突事实/未读提醒、显式 Resolve/CAS、多实例恢复、Neon 或最多 2 个 Multipass 管理实例的 K3s 环境、资源与性能验收尚未闭合。 |

## §2 证据清单

基线提交为 dev@313e5741fb0dc557874e5a37bc05e8839211cd90。以下为该提交中的 Git blob SHA，便于复核文档快照：

| 当前 dev 路径 | HEAD:path blob SHA | 相关内容 |
|---|---|---|
| docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md | 868ec52ad647892c35a2743a31293b08d839a05e | §23.7–23.13；§23.13 为最近 SSE PG18 记录 |
| docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md | c901c85c777186f0c3dd7053f071a40c4f3fc90d | v0.4 当前状态与 W16-A 部分要求 |
| docs/design/BD-W16-PAYLOAD-SECURITY-001.md | 697b6206b16ee6ee51c6301af8f8be3a0db5cb65 | v0.4 owner/Outbox 与撤销边界 |
| docs/design/DD-W16-PAYLOAD-SECURITY-001.md | 8a7b287eb6bae549532b04cf78be27afa04937f0 | §1 owner 清单、§2.3 撤销 Outbox、§2.4 历史状态 |
| docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md | 1638126c129912c45f5dbe309d767372174d7528 | v2.1 的 W16 identity/SSE acceptance mapping |
| docs/reviews/W16-A-DEVICE-REVOKE-API-REVIEW-20261030.md | b974cf2e6dae5c3d06ef767a2a7f43a36a81e92e | F-01/F-02 静态审查；审查基线为较早 dev SHA |
| docs/reports/PHASE-W16-A-CANVAS-SSE-REVOCATION-20261029.md | 0f664d1ec04efd4219b48a90ec5eca5ddbcf8781 | 精确 PG18.6 单用例与限制 |
| crates/star-api-rest/src/group_api/canvas_event_stream.rs | 6a87c8e01863ab2257bb9059a4ccfc5a3ebfe584 | 当前进程 listener、wake 与有界 stream |
| crates/star-pg-adapter/src/repository/device_bindings.rs | 9a420719bf962190d6fb774f9173133464f414b9 | revoke 事务及 refresh/audit 写入 |
| crates/star-api-rest/src/auth/oauth/device_enrollment.rs | b713c1667b1050a1e54c621c89918c2a544977e0 | revoke HTTP route/handler |
| db/migrations/2026-10-28-w16-device-auth.sql | 8c788ecf88ffe0c38b3da5af5958a2995d1dd39d | device audit 与通用 wake trigger |
| crates/star-api-rest/tests/canvas_device_auth_pg.rs | 683fb520bac4bce4d88bc6805121c47feaa3e8e6 | Canvas SSE PG 用例；本次审计未运行 |

针对 HTTP revoke 的前次静态 review 基线为 dev@be3dd7b42881cc0ebd229a622f87bc5f407e9f69；其候选 NOT_RUN 陈述早于 §23.13 的验收结果。HTTP/audit 缺口经本次有限源码/测试路径搜索仍存在；不沿用该报告中已经过时的 Canvas SSE 状态。

## §3 建议的下一 W16 lane

**建议：W16-A authenticated device-revoke HTTP + audit PostgreSQL acceptance。** 这是现有 route/repository 上可独立补齐的验收切片；不声称它实现 durable Outbox 或关闭 W16-A。现有 route、owner transaction 与 disposable PG harness 已具备，且可用专属新测试文件避免共享 OAuth helper。

建议路径清单（当前 MAGOS check 通过、未登记；它只是当前登记状态的无重叠预览，创建前须重新 check/plan）：

1. crates/star-api-rest/tests/device_revocation_http_pg.rs
2. scripts/automation/w16a_device_revocation_http_pg_acceptance.py
3. scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py
4. docs/automation-design.md
5. scripts/automation/registry.md
6. docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md
7. docs/briefs/W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
8. docs/reports/PHASE-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.md
9. docs/reports/WBS-DISPATCH-W16-A-DEVICE-REVOCATION-HTTP-PG-20261030.jsonl

建议验收：

- 仅连 runner 自有 disposable loopback PostgreSQL；限定目标、实例生命周期和迁移顺序，失败时保留日志/数据，不连接生产或用户数据库。
- 通过实际 authenticated HTTP revoke handler 调用，而非直接 SQL；首次合法撤销验证响应与 device generation/status、对应 refresh family、恰好一条 revoked audit 同事务提交。
- 验证缺失/无效 bearer、失效 caller device、跨 actor/tenant 目标拒绝且不留下部分写入；以 rollback/fault injection 证明设备状态、refresh 与 audit 原子性。
- 使用并发目标撤销证明最多一次状态迁移和一次审计。重复请求响应仍须作为 API 契约单独冻结；不得仅从当前 400/invalid_grant 实现推断产品契约。
- 若为迁移设置使用 superuser，测试请求应使用单独受限 app role；superuser-only 结果不得用来关闭生产 grants/RLS 门。
- 更新测试设计和 phase report，只关闭这组实际断言；保持 Neon/Multipass-K3s、多实例恢复、生产 role/RLS 与性能门 OPEN。

当前 MAGOS 对分支 agent/wbs-next-20261030/device-revoke-http-pg 的只读 check 结果为无重叠（未登记）。它涵盖上列 9 个路径；与现有 active OAuth claim 的 crates/star-api-rest/tests/common/mod.rs、oauth_device_auth_pg.rs、oauth_refresh_tokens.rs 不相交。远端 refs 仅截至上次 fetch，git backstop 未安装；这不是 lane 注册或分支创建授权。

本次 MAGOS list 中与 W16/当前 WBS set 相关的 active claims：
- codex/w16-a-pg-device-context-20261028：scripts/automation/phase9f2_binding_http_acceptance.py、scripts/automation/test_phase9f2_binding_http_acceptance.py、docs/briefs/W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028.md、docs/reports/PHASE-W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028.md、docs/reports/WBS-DISPATCH-W16-A-PG-DEVICE-CONTEXT-HARNESS-20261028.jsonl。
- codex/w16a-oauth-pg-acceptance-20261029：crates/star-api-rest/tests/oauth_device_auth_pg.rs、crates/star-api-rest/tests/common/mod.rs、crates/star-pg-adapter/src/repository/oauth_refresh_tokens.rs、scripts/automation/w16a_oauth_pg_acceptance.py、scripts/automation/test_w16a_oauth_pg_acceptance.py、docs/briefs/W16-A-OAUTH-PG-ACCEPTANCE-20261029.md、docs/reports/PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029.md、docs/reports/WBS-DISPATCH-W16-A-OAUTH-PG-ACCEPTANCE-20261029.jsonl。
- agent/wbs-next-20261030/w16-audit owns this audit brief, review, phase report and dispatch log. agent/wbs-next-20261030/wbs-sync owns docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md and its brief, phase report and dispatch log.

## §4 阻塞、先决与顺序

1. 建议 HTTP/audit acceptance lane 可先行；若测试揭示生产源码路径缺陷，应停止扩写并通过新的 MAGOS check/amend 明确申请对应源文件。
2. durable device-revocation Outbox 实现须先修复 DD §1 与 §2.3 对 Outbox owner 的冲突，明确 Identity owner 的事务事实、事件 schema/version、消费/recovery/cursor 与 RLS。不得由 Collaboration owner 跨 owner 回写 Identity 事实，也不得把通用 PostgreSQL NOTIFY 当成 durable delivery。
3. SRS/BD/DD v0.4 与 WBS §23.13 / TD v2.1 的 SSE 状态需要有边界的文档同步。此审计 lane 未被授权改这些规范文件；WBS-sync lane 也只拥有其已登记路径。
4. W16-C 接入生产 owner/provider 前需获批实际可商用分发的 provider、固定 profile/version/digest 并满足商业许可/SBOM、持久化、恢复与 key lifecycle 门。
5. W16-D encrypted draft/conflict 和 W16-E 目标部署、双 API 实例、恢复、p99、RSS/CPU/背压测试继续依赖前述安全与身份门。

## §5 不采用的捷径

- 不把 Canvas SSE 的 direct-SQL 单用例通过扩大解释为 HTTP revoke、审计、生产 RLS、跨节点或 replay 通过。
- 不把 pg_notify 当作持久 Outbox，也不在 owner 边界冲突未收敛前先写新表/消费者。
- 不使用 mock/no-op audit、superuser-only 请求测试或内存队列证明生产授权、审计和耐久性。
- 不因 W16-B contract、W16-C envelope 单测或本地数据库切片通过而提前开放 W16-D 明文/密文协作或关闭 W16/WLD-7。
- 不修改、归档或清理任何已有 W16 候选分支/工作树；Orca 当前快照显示这些 W16 工作树无 live/unverifiable terminals，但 inactivity 不是删除或覆盖许可。

## §6 已知限制

- 证据仅来自固定本地 dev@313e5741；未 fetch，远端新鲜度未知。
- MAGOS list 中可见的 W16 active claims 为 PG device-context harness、OAuth PG acceptance 与本审计 lane。Orca worktree ps 观察到 w16-a-device-auth-20261028、w16-a-pg-device-context、w16-a-security-status-sync-20261029、w16-b-kms-contract-20261028、w16-b-release-closure、w16a-doc-sync-20261029、w16a-oauth-pg-acceptance-20261029 七个既有候选工作树均 inactive，liveTerminalCount=0、unverifiableTerminalCount=0、agents=[]；现存候选数据保持原样。
- 审计证据读取限于当前 audit worktree 内文档/源码；Git worktree 的 common dir 指向 D:/Star/.git。一次默认目录 Add File 调用可能已写入默认工作区的 docs/reviews/WBS-W16-NEXT-AUDIT-20261030.md 与 docs/reports/PHASE-WBS-W16-NEXT-AUDIT-20261030.md；该目标未检查或清理，协调者已收到准确调用结果并负责单独处理。后续恢复报告使用显式 lane workdir。常规 ref/status/commit 共享仓库 Git 元数据，协调者已明确授权仅维护隔离 lane refs。
- 本次未运行任何测试、Cargo、数据库、迁移、应用、生产环境、性能或完整 E2E 验证。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 固定 dev 快照的 W16 证据、状态漂移、Outbox owner 冲突、登记冲突与下一项无重叠验收切片。 | WBS-W16-NEXT-AUDIT-20261030 brief |