# PHASE-9F4J-3 — 独立代码复审

> **状态：NOT APPROVED FOR INTEGRATION** · 修订 v0.2 · 2026-11-07
> 复审分支：`codex/9f4j-3-independent-review-20261107`
> Source：`codex/9f4j-3-native-composition-20261107` · HEAD `e22d92bcbefaf009e18d043ab2c8a762c150fc01`（相关改动未提交）
> 目标：本地 `dev@8c2d78cbe77aeb81ed28d2396c319a1bbec24ed9`

## §0 目的

记录 9F4J.3 Task Event composition 的第二轮只读复审。检查 source HEAD `e22d92b` 的 BI writer 身份/角色权限、Profile 定期复验、panic/readiness、shutdown drain 与阶段证据。未修改 source lane，未运行 Cargo/PostgreSQL；本报告不构成生产验收。

**结论：NOT APPROVED。**身份比较、Profile 复验和 worker 失败 readiness 有正向修复；仍有一个 P1 runtime 权限隔离缺口和一个 P2 drain 失败未传递进程退出码的问题。

## §1 改动与复审矩阵

| 范围 | 第二轮结果 | 证据/状态 |
|---|---|---|
| Writer 与 app/query 身份 | 已修复 | 比较 app、BI query、writer 的 current_user/session_user；`bi_task_event_ingestion.rs:974-1101` |
| BI writer runtime 权限边界 | 未通过 | 仅拒绝 `*_owner` 成员/SET 路径；未限制其他 runtime role 和非 BI EXECUTE；P1 |
| Profile pin | 正向修复 | 每 30 秒 uncached resolver 复验；revision/digest/endpoint drift 阻断且不热切换；`:615-670,1125-1156` |
| Panic/意外退出 readiness | 正向修复 | exit guard/readiness 映射阻止 worker 退出后仍 Ready；`:1103-1113,1207-1239` |
| Drain timeout | 部分通过 | readiness 设为 Blocked，但 shutdown 不返回错误，main 仍可能成功退出；P2 |
| Inbox、ACK、quarantine | 静态路径保留 | commit-before-ACK/quarantine 可读；PG 与真实 transport 未运行 |
| 测试与文档 | 部分一致 | source 报告基线/target/Cargo.lock 状态匹配；其测试结果未由本 reviewer 重跑 |

## §2 验证摘要

| 验证项 | 结果 | 证据与限制 |
|---|---|---|
| Source 文件与阶段报告静态读取 | 完成 | 只读 source HEAD `e22d92b`；source 改动未提交 |
| CodeGraph | 未覆盖 | source worktree 不匹配；直接读指定文件，未索引/修改 `.codegraph` |
| Rust/Cargo 检查与测试 | **本轮 NOT RUN** | source 报告自述 all-targets 检查通过、定向测试 10 项通过；本轮未独立验证 |
| PostgreSQL role matrix、grants、RLS | **NOT RUN** | 未连接测试 DB，runtime-role 组合缺口仍待真实 PG 验收 |
| gRPC/mTLS 投递、ACK/恢复、部署与 runtime | **NOT RUN** | 未执行 transport、部署、跨实例恢复或性能验收 |
| drain 失败进程退出码测试 | **NOT RUN** | 静态发现 main 未传播 shutdown 失败；需实现后验证 |

## §3 已知缺口

1. **P1：BI writer 权限边界不够窄。**`bi_task_event_ingestion.rs:185-197,220-224,261-282` 只排除 `*_owner` membership/SET，不限制所有 `star_*` runtime 角色，也不审计非 BI schema/function EXECUTE。`star_platform_transport_runtime` 在 `2026-10-12-platform-native-transport-log.sql:45-46,627-630` 获得 transport API 权限，认证 API grants 见 `2026-10-12-platform-transport-z-native-server-auth.sql:542-547`。需显式 BI allowlist 和 PostgreSQL role/EXECUTE 矩阵测试。
2. **P2：drain 失败未传递到进程结果。**`bi_task_event_ingestion.rs:438-484` 仅将 readiness 置为 Blocked；`main.rs:127-133` 不检查 shutdown 返回值且 shutdown 当前无错误返回，drain timeout 仍可能成功退出。需返回并传播错误。
3. 首轮指出的 writer current/session 与 app/query 身份比较缺失已修复；每 30 秒 Profile pin 复验和 drift fail-closed、worker panic/意外停止 readiness Blocked 均为正向修复，不再列为开放缺陷。
4. Profile drift 要求 drain/migrate/restart，不自动热切换；该策略仍需端到端验证。
5. source 报告自述的 Cargo PASS 不是本轮证据；PostgreSQL/grants/RLS、enabled receiver、真实 mTLS transport、部署和资源性能均未验收。9F4J.3 不可标记完成或 production-ready。

## §4 子代理复审与失败接手清单

- 第一轮 review commit：`08e59cf32bf07e44fc9d8d3fa1acda4d72a5a425`；其 app/query identity、Profile 复验及 panic readiness 发现已由 source 新增实现处理。
- 第二轮针对 source HEAD `e22d92b` 只读复核；新增 P1 runtime role/function allowlist 与 P2 shutdown error propagation。
- Review worker 未修改 source、未运行 Cargo/PG，未触碰 W8/Dependabot、`D:\Star`、main 或 remote。
- 后续 owner 修复两项并提供 PostgreSQL 角色矩阵、shutdown failure exit 验证后，再提交独立复审。

## §5 守门规则

1. BI owner runtime 与 transport runtime 保持独立边界。
2. BI writer 使用明确的最小权限角色 allowlist。
3. 约束 writer 的 membership 与可 SET ROLE 目标，不能只筛选 `*_owner` 名称。
4. writer 不得拥有 BI 事件提交/隔离以外的 schema/table/function 权限。
5. 分别读取并比较 app、BI query、BI writer 的 current_user/session_user。
6. 授权/角色事实查询失败时启动必须 fail closed。
7. Profile version/digest/endpoint/log generation 与消息首次领用时 pin 保持一致。
8. Profile 复验读取权威、未缓存配置；漂移后禁止继续 pull/ACK。
9. Profile 改动按 drain/migrate/restart 流程应用，不热切换旧 client。
10. Inbox durable commit 与 exact replay receipt 先于 ACK。
11. identity/content conflict 走持久化 quarantine，不直接 ACK。
12. panic、意外退出、授权失败或 Profile drift 均不得保持 Ready。
13. drain timeout/JoinError 反映为 readiness failure 并传播到进程失败结果。
14. 队列、in-flight、wire/frame、deadline 与 shutdown drain 有界。
15. 静态测试不能代替 PostgreSQL grants/RLS、真实 transport、恢复或部署验收。
16. 来源报告中的 PASS 标为自述，除非有本轮/CI 可复核实证。
17. 集成前复核 source HEAD、dev target、登记和未提交状态；本报告不授权触碰其他 lane。

## §6 签字栏

| 角色 | 复审责任 | 状态 |
|---|---|---|
| 架构 | Mavis 接手复核 owner 隔离与 transport 边界 | NOT APPROVED：P1 未关闭 |
| SRE Lead | Mavis 接手复核 worker、readiness、drain 与退出行为 | NOT APPROVED：P2 未关闭 |
| 平台 | Mavis 接手复核 Rust native transport / PostgreSQL 权限证据 | 静态复核完成；PG/grants/RLS NOT RUN |
| 评审主持 | Mavis 接手复核范围、证据和验证限制 | 第二轮只读复核完成 |
| PM | Mavis 接手核对阶段状态与 WBS | 9F4J.3 OPEN/PARTIAL；集成门未通过 |

修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首轮记录 writer identity、Profile drift、panic readiness 和基线同步问题；NOT APPROVED | 9F4J.3 首轮复审 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新 source HEAD e22d92b / dev 8c2d78c；确认身份比较、Profile 复验、panic readiness 修复；新增 runtime allowlist P1、shutdown 退出传播 P2；记录 Cargo/PG 未运行 | 9F4J.3 第二轮复审 |
