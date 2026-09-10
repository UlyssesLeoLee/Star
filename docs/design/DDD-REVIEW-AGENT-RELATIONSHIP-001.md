# DDD-REVIEW-AGENT-RELATIONSHIP-001 — Star Agent Relationship Graph (ARG) DDD Review

> **状态**: 🟢 v0.1 (ARG.10 DDD Review 拍板落地, 3 known gaps 全部决议, per 9/10 15:30 JST Mavis 审核决策 author=Ulysses + 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签维持 + 守门 #1 v15 docs 同步饱和第 64 次新事件触发 仍允许 + 守门 #11 缺标比错标)
> **日期**: 2026-09-10
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 8/27 19:39 JST 授权)
> **签批**: 🟢 Mavis 审核决策 author=Ulysses (per 守门 #14 v4 9/10 12:45 JST 反转, 真人代签流程全部取消改为 mavis 审核)
> **依赖**: [SRS-AGENT-RELATIONSHIP-001.md v0.1](../requirements/SRS-AGENT-RELATIONSHIP-001.md) · [BD-AGENT-RELATIONSHIP-001.md v0.1](BD-AGENT-RELATIONSHIP-001.md) · [DD-AGENT-RELATIONSHIP-001.md v0.1.1](DD-AGENT-RELATIONSHIP-001.md) · [WBS-001 §14.11](STAR-P3-WBS-001.md) · [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) · [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md)
> **关联文档**: [PHASE-ARG-IMPL-REPORT.md v0.1](../reports/PHASE-ARG-IMPL-REPORT.md) · [PHASE-ARG-08-IMPL-REPORT.md](../reports/PHASE-ARG-08-IMPL-REPORT.md) · [PHASE-ARG-07-IMPL-REPORT.md](../reports/PHASE-ARG-07-IMPL-REPORT.md) · [PHASE-ARG-06-IMPL-REPORT.md](../reports/PHASE-ARG-06-IMPL-REPORT.md) · [PHASE-ARG-04-IMPL-REPORT.md](../reports/PHASE-ARG-04-IMPL-REPORT.md) · [PHASE-ARG-01-IMPL-REPORT.md](../reports/PHASE-ARG-01-IMPL-REPORT.md) · [AGENTS.md §4 #14 v4 Mavis 审核决策](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [守门 #14 v3→v4 反转 v0.62 修订历史](STAR-P3-WBS-001.md)

---

## 0. 目的 (Purpose)

本文档是 **Star Agent Relationship Graph (ARG)** DDD Review 拍板记录, 覆盖 [DD-AGENT-RELATIONSHIP-001.md v0.1.1 §13](DD-AGENT-RELATIONSHIP-001.md) 8 项已知缺口中, 3 项需要 DDD Review 拍板的 (G-9 / G-4 / G-10), 以及 1 项已闭环的 5 域 Lead 真人到位 (G-8, per 守门 #14 v4 真人代签全取消, 不再 trace 到位 timeline).

**核心目标**: 拍板 ARG 跟 TMO 任务卡 DAG 边界 (G-9) + trusts 跳过 verify 安全审计 (G-4) + ARG Schema V2 迁移路径 (G-10), 为 ARG 11 子项 (除 ARG.10 自身) 全部 🟢 收官 + 跨 session 续做项提供 DDD 治理层拍板.

**v0.1 状态 (2026-09-10 落档)**: 3 已知 gaps 全部决议, 7 段结构 (目的 / 决议矩阵 / 验证摘要 / 已知缺口 / 跨 session 续 / 守门合规 / 签字栏 / 修订历史), 5 签字栏 v0.1 升版 (Mavis 审核决策 author=Ulysses per 守门 #14 v4 反转 9/10 12:45 JST).

**实装路径 (per 守门 #9 v19 + #9 v20 + #12 v21 + #14 v4 + #19 v19)**: 7 段结构 + 3 决议明确 (推荐 + 备选) + 4 已知缺口的守门合规 + 1 跨 session 续 (P3-E write `arg_migration` v1→v2 脚本, per G-10 拍板) + 5 签字栏 v0.1 升版. 0 子代理 RPC 派, 0 cargo 改动, 0 自动 follow-up (Mavis 审核决策 author=Ulysses).

---

## 1. DDD Review 决议矩阵 (3 gaps + 1 已闭环 gap, per 守门 #11 缺标比错标)

### 1.1 G-9: ARG 跟 TMO 9 节点 (任务卡 DAG) 边界 (per DD §13 G-9)

**背景**: ARG 关系图 (10 类关系, 4 核心 + 6 扩展) 跟 TMO 任务卡 DAG (M-N1 merge / M-N2 split / M-N3 reorder / M-N4 bulk / M-N5 summarize / M-N6 reassign / M-N7 metadata, per [DD-LangGraph-TMO-001 §3.2.1.1](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-09-03-langgraph/03-detailed-design.md)) 都涉及"边"概念, 边界不清会导致 L0 协调冲突 + 数据冗余.

**潜在冲突 (per DD §13 G-9 调研)**:
- **M-N3 reorder** (任务卡 DAG 边排序) vs **ARG delegates_to** (代理委派关系): 任务卡 DAG 边是 task_id 之间的依赖关系, 临时 + transactional; ARG delegates_to 是 agent_id 之间的关系, 长期 + SCD Type 2
- **M-N4 bulk** (批量操作任务卡) vs **ARG collaborates_with** (代理协作): 批量操作是 action (单次执行), 协作是 relation (长期关系)
- **M-N6 reassign** (任务卡重新分配 sub-agent) vs **ARG stand_in_for** (代理代理): reassign 是 sub-agent 调度, stand_in_for 是代理间关系

**DDD 拍板 (推荐)**: **ARG 跟 TMO 是两个独立 bounded context, 边界通过 L0 协调, 不共享数据模型**:
- (a) **数据隔离**: ARG 存 agent_id 关系, TMO 存 task_id 关系, 两者不直接引用. 跨域信息通过 L0 dispatch 协调 (per 守门 #13 a L0 协调, 禁止 L1↔L1 直连)
- (b) **M-N3 reorder 不影响 ARG**: reorder 只改任务卡 DAG 边, 不改代理关系. ARG 在 reorder 触发时不需要更新 (ARG 只关心 agent_id, 不关心 task_id)
- (c) **M-N4 bulk 通过 ARG collaborators 触发影响**: bulk 创建/删除任务卡时, L0 dispatcher 调 ARG ARGDispatchRouter.query_collaborators() 查相关代理, bulk 通知 ARG collaborators (per DD §4.5 实证)
- (d) **M-N6 reassign 触发 ARG stand_in_for 关联**: reassign 时, 如果新 sub-agent 跟旧 sub-agent 有 stand_in_for 关系, ARG 自动更新 (per DD §4.6 ARGContextInjector)
- (e) **L0 协调接口**: TMO 7 节点 (M-N1..M-N7) 通过 PyO3 binding 跟 ARG 5 effect 模块 (ARGDispatchRouter / ARGContextInjector / ARGTrustEngine / ARGOutputEvaluator / ARGAchievementEngine) 通信 (per DD §5 PyO3 协议)

**备选方案 (拒绝)**: 合并 ARG 跟 TMO 单一图数据库, 共享节点 + 边. 拒绝原因: 边界混乱 + 数据冗余 + 查询性能下降 + 5 域 Lead 责任不清 (per 守门 #3 5 域独立 Lead).

**实装位置**: DD-AGENT-RELATIONSHIP-001 §4.5-4.9 + DD-LangGraph-TMO-001 §3.2.1.1 (M-N1..M-N7) + PyO3 binding §5 (per DD §5 已细化协议).

**状态**: 🟢 **已闭环 (per ARG.3 18 UT 实证 + DD §5 PyO3 协议细化)**.

---

### 1.2 G-4: trusts 跳过 verify 安全审计 (per DD §13 G-4)

**背景**: trusts 关系创建时, 当前实现允许 trust_score ≥ 0.9 + agent 类型包含双约束绕过 verify (per DD §4.7 ARGTrustEngine). 担心被恶意利用 (创建虚假 trusts 关系, 跳过 verify 影响 trust 推荐).

**潜在风险 (per DD §13 G-4 调研)**:
- **风险 1**: 恶意 agent 反复创建 self-trusts 关系, 提升 trust_score, 影响其他 agent 信任推荐
- **风险 2**: verify 跳过 = trust_score 不能审计, 出现"虚假信任"链
- **风险 3**: trust_score ≥ 0.9 阈值未明确推导, 不知道是统计还是规则

**DDD 拍板 (推荐)**: **trusts 跳过 verify 加 4 重安全审计**:
- (a) **trust_score 阈值收紧**: ≥ 0.9 → ≥ 0.95 (更严格, 仅前 5% 高信任代理可跳过 verify)
- (b) **agent 类型包含双约束加 3 重验证**: (1) mutual trust (A trusts B AND B trusts A 才允许), (2) historical evidence (过去 30 天有 ≥ 10 次真实协作记录), (3) multi-source attestation (≥ 2 个独立源头确认, e.g. 5 域 Lead 签字 + 系统审计)
- (c) **审计日志强制**: 跳过 verify 的 trusts 关系必须写 audit_audit_event 表 (per 守门 #13 d Transaction 100% audit, WORM append-only per ADR-0043), 包含 trust_score + 双 agent_id + 跳过原因 + 审计时间
- (d) **撤销机制**: 任何 agent 可在 7 天内通过 DDD Review 撤销未审计的 trusts 关系 (撤回窗口), 撤销后 trust_score 立即降回 0.5 默认

**备选方案 (拒绝)**: 完全禁止跳过 verify. 拒绝原因: 牺牲合理用例效率 (5 域 Lead 之间互信已建立的场景), 改用 audit + 阈值 + 4 重验证足够安全.

**实装位置**: DD §4.7 ARGTrustEngine 新增 4 重审计 (trust_score 阈值 + mutual + evidence + multi-source + audit log) + DD §3.2.5 共享类型 (TrustAuditLog) + 守门 #13 d 审计 trigger 100% 必携.

**状态**: 🟡 **本 DDD Review 拍板 (本 commit 落地), 实装落地 跨 session 续** (per 真人到位时跟 5 域 Lead 协审, per守门 #14 v4 不再 trace 到位 timeline, 改 Mavis 审核直接落地).

---

### 1.3 G-10: ARG Schema V2 迁移路径 (V1 → V2 加新关系类型时怎么处理存量数据)

**背景**: ARG 当前 10 类关系 (delegates_to / consults / collaborates_with / reports_to / mentors / peer_reviews / stand_in_for / shadows / challenges / trusts, per BD §3.1 + DD §3.3.1). 未来扩展加 V2 关系类型 (e.g. escalates_to / delegates_to_v2) 时, 存量 V1 数据怎么处理.

**潜在风险 (per DD §13 G-10 调研)**:
- **风险 1**: 硬切换 V1 → V2 时, 存量 V1 edge 在 V2 schema 下无对应字段, 关系失效
- **风险 2**: 双写 V1 + V2 期间, 读路径选 V1 还是 V2 不一致
- **风险 3**: V1 数据迁移到 V2 时, edge_type 字段映射缺失

**DDD 拍板 (推荐)**: **P3-E 写 `arg_migration` v1→v2 脚本 + 4 阶段渐进式迁移**:
- (a) **阶段 1 (并行运行, 2 周)**: V1 + V2 schema 并存, 双写 (V1 旧 path + V2 新 path). 读路径优先 V2, fallback V1. 此期间不删除 V1 字段
- (b) **阶段 2 (V2 优先, 2 周)**: 读路径全部走 V2, 写路径继续双写. 监控 V1 读 fallback 比例 < 5%
- (c) **阶段 3 (V1 只读, 1 周)**: 写路径切到 V2 only. V1 表 mark read-only (DDL trigger block INSERT/UPDATE, allow SELECT). 跑 `arg_migration` v1→v2 脚本批量迁移剩余 V1 edge 到 V2
- (d) **阶段 4 (V1 退役, 1 周)**: V1 表 archive 到 _archive schema (per 守门 #13 d Transaction 100% audit), 0 V1 edge remaining. 监控 1 周 0 V1 读

**实装位置**: `scripts/automation/arg_migration.py` v1.0 (per DD §14 跨 session 续, P3-E 阶段 5/7 任务) + DD §3.3 Edge schema 扩展 V2 字段 (`edge_type_v2`, `v1_migration_status` enum: `pending` / `migrated` / `verified`).

**守门合规**: 4 阶段渐进式迁移期间, 守门 #13 d 100% audit + WORM append-only (per ADR-0043) 维持; 守门 #12 v15 docs 同步饱和新事件触发 (per 守门 #1 v15).

**状态**: 🟡 **本 DDD Review 拍板, 实装落地 P3-E 阶段 5/7 (per DD §14.3 跨 session 续, Mavis 自驱)**.

---

### 1.4 G-8: 5 域 Lead 真人到位 timeline (per 守门 #14 v4 反转 9/10 12:45 JST)

**背景**: G-8 原始定义 (per DD §13) = 5 域 Lead 真人到位 timeline 待定, 需要真人到位时追溯签字覆盖修订历史.

**DDD 拍板 (per 守门 #14 v4 反转 9/10 12:45 JST Ulysses 发令"真人代签流程全部取消, 改为 mavis 审核")**: **G-8 已闭环, 不再 trace 到位 timeline**:
- (a) 真人代签流程**全部取消**, 改为 Mavis 审核决策 author=Ulysses (per v0.62 修订历史 + v0.32 候选拍板激活)
- (b) 5 域 Lead 寻访流程保留 (per docs/recruitment/5-business-domain-lead-referral.md v0.3), 由 Mavis 审核直接落地, 不等真人到位
- (c) 真人到位后**不**追溯签字覆盖修订历史 (per 守门 #14 v4 真人代签全取消), 但 author=Ulysses 维持 (per 8/27 19:39 JST 授权)
- (d) 真人到位后可以独立发令修改策略 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理 Ulysses 决策)

**实装位置**: AGENTS.md §4 #14 v3 → v4 反转落地 (per v0.62 修订历史) + docs/guardian/v30_signature_boundary.md + v31_lead_traceback.md 政策全部取消 + v32_audit_boundary.md 替代政策 active (per v0.70 拍板激活).

**状态**: 🟢 **已闭环 (per 守门 #14 v4 反转 + v0.70 v32 拍板激活)**.

---

## 2. 验证摘要 (Verification Summary)

### 2.1 文档 v0.1 阶段 (本 DDD Review 落地)

| 验证项 | 状态 | 证据 |
|---|---|---|
| 3 gaps 拍板 (G-9 / G-4 / G-10) 决议明确 | ✅ | §1.1 G-9 (ARG 跟 TMO 边界, 5 子项 (a)-(e)) + §1.2 G-4 (trusts 跳过 verify 4 重审计) + §1.3 G-10 (Schema V2 4 阶段渐进式迁移) |
| 1 gap 已闭环 (G-8 5 域 Lead 真人到位) | ✅ | §1.4 G-8 守门 #14 v4 反转落地, 不再 trace 到位 timeline |
| 7 段结构 (per AGENTS.md §3) 落档 | ✅ | §0 目的 / §1 DDD Review 决议矩阵 / §2 验证摘要 / §3 已知缺口 / §4 跨 session 续 / §5 守门合规 / §6 签字栏 / §7 修订历史 |
| 5 签字栏 v0.1 升版 | ✅ | §6 5 角色 (架构 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 审核决策 author=Ulysses (per 守门 #14 v4) |
| WBS-001 §14.11 ARG.10 升 🟡→🟢 | ✅ | 跨 session 续, 待本 commit + WBS row 落地 |
| 守门 #11 缺标比错标 显式列 | ✅ | §3 已知缺口 4 项 (3 拍板 + 1 跨 session 续 + 1 闭环) 显式列, 不沿用 v0.1 错标 |

### 2.2 守门合规 (本 DDD Review commit, 18 项 0 违反)

| 守门 | ARG.10 派生约束 | 验证位置 |
|---|---|---|
| **#1 R-05** | docs 改动不跑 cargo, 不 push origin (本 commit 落档后等 user 拍板推) | §5 守门合规 |
| **#1 v15 docs 同步饱和** | 本 commit 1 新事件触发, 守门 #12 第 64 次仍允许 | §5 + §7 修订历史 |
| **#1 v19 自动化档判定** | [S] 父会话直跑, 0 子代理 RPC, 0 cargo 改动, 0 测试 stage | §2 验证摘要 |
| **#3 5 域独立 Lead** | ARG 关系图 enforce delegates_to / consults / collaborates_with 跨域协调 (per G-9 拍板 (a) L0 协调) | §1.1 G-9 (a) + ARG.1 EdgeOps |
| **#9 v19 Mavis 自驱** | 0 子代理 RPC 派, 父会话直跑 0 重试 (per守门 #9 子代理不可靠实证 #7 父会话接手模式) | §0 目的 + §5 守门合规 |
| **#9 v20 子代理 dispatch 必先 brief** | 0 子代理派, 不需要 brief 落档 (per §0 实装路径) | §0 目的 |
| **#10 代签规则** | 修订人 = `Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手` (per 19:39 JST 授权) | §7 修订历史 |
| **#11 缺标比错标** | 4 已知缺口 显式列, 不沿用 v0.1 错标 (per守门 #11) | §3 已知缺口 |
| **#12 AI 協作文档治理** | 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 (per ARG DD §13) | §5 + §7 修订历史 |
| **#13 a L1↔L1 禁止通信** | G-9 拍板 (a) ARG 跟 TMO 边界 L0 协调, 禁止 L1↔L1 直连 (per DD §5 PyO3 协议) | §1.1 G-9 (a) |
| **#13 c Master 100% RLS** | agents + edges 2 张 Master 表 100% RLS 必携 + SCD Type 2 (per G-10 拍板 (a) 阶段 1 双写期间) | §1.3 G-10 (a) + ARG.1 + ARG.6 |
| **#13 d Transaction 100% audit / Work 100% retention** | decision_audit + relationship_events + unlocks 3 T 表 100% audit + WORM; template_instances 1 W 表 100% retention 30d; **G-4 拍板 (c) trusts 跳过 verify 强制写 audit_audit_event 表 (WORM append-only per ADR-0043)** | §1.2 G-4 (c) + ADR-0043 |
| **#14 v4 Mavis 审核决策 author=Ulysses** | 9/10 12:45 JST 守门 #14 v3→v4 反转落地, Mavis 审核决策 author=Ulysses, 真人代签全取消, 5 签字栏 v0.1 升版 | §0 目的 + §6 签字栏 + AGENTS.md §4 #14 |
| **#19 v19 agent 交互 Python 化** | G-10 拍板 (a) P3-E 写 `arg_migration` v1→v2 脚本走 Python (per守门 #19 v19 [P] docs 同步) | §1.3 G-10 (a) |
| **#22 调试控制台不污染 main** | 0 子代理派, 0 cargo 改动, 0 测试 stage | §5 守门合规 |
| **#23 AI 修改 mock** | 0 OpenAI/Anthropic 外部 API, 全本地 Python 脚本 | §5 守门合规 |
| **#24 调试控制台走 subprocess** | 0 console_server.py 调用, 0 subprocess 派 | §5 守门合规 |

---

## 3. 已知缺口 (Known Gaps, per 缺标比错标)

### 3.1 ARG.10 DDD Review 3 gaps 拍板 + 1 gap 已闭环

| # | 缺口 | 拍板 | 状态 |
|---|---|---|---|
| **G-9** | ARG 跟 TMO 9 节点 (任务卡 DAG) 边界 (e.g. M-N3 reorder 跟 ARG delegates_to 区别) | L0 协调 5 子项 (a)-(e) (per §1.1) | 🟢 **已闭环** (per ARG.3 18 UT 实证 + DD §5 PyO3 协议) |
| **G-4** | trusts 跳过 verify 安全审计 (trust_score ≥ 0.9 + agent 类型包含双约束) | 4 重审计 (trust_score 阈值 0.95 + mutual + evidence + multi-source + audit log + 7 天撤回窗口) (per §1.2) | 🟡 **本 DDD Review 拍板 (本 commit 落地), 实装落地 跨 session 续** (per守门 #14 v4 不再 trace 真人到位, 改 Mavis 审核直接落地) |
| **G-10** | ARG Schema V2 迁移路径 (V1 → V2 加新关系类型时怎么处理存量数据) | 4 阶段渐进式迁移 (双写 2 周 + V2 优先 2 周 + V1 只读 1 周 + V1 退役 1 周) (per §1.3) | 🟡 **本 DDD Review 拍板, 实装落地 P3-E 阶段 5/7** (per DD §14.3 Mavis 自驱) |
| **G-8** | 5 域 Lead 真人到位 timeline (per 真人到位时追溯签字覆盖) | 守门 #14 v4 反转 9/10 12:45 JST, 真人代签全取消, 不再 trace 到位 timeline (per §1.4) | 🟢 **已闭环** (per守门 #14 v4 反转 + v0.70 v32 拍板激活) |

### 3.2 ARG DD §13 8 项已知缺口 整体盘点 (per §1 拍板 + 历史)

| # | 缺口 | 状态 | 解决路径 |
|---|---|---|---|
| **G-1** | Memgraph 客户端 crate 缺 (`r2d2-memgraph` 待调研) | 🟢 **已闭环** (per ARG.1 33 UT 实证) | 自实现 (Bolt protocol) 选型, ARG.1 落地 |
| **G-3** | L0 ↔ L1 通信协议 + ARG 集成 (PyO3 协议未细化) | 🟢 **已闭环** (per DD §5 细化协议 + ARG.3 18 UT) | DD §5 + ARG.3 |
| **G-5** | challenges 双向论证 prompt 模板 10 套 (5 decision × 2 trust) | 🟢 **已闭环** (per DD §7 + ARG.3 10 challenges) | DD §7 + ARG.3 |
| **G-6** | 8 拓扑成就 Cypher 模板 | 🟢 **已闭环** (per DD §6 + ARG.3 8 Cypher) | DD §6 + ARG.3 |
| **G-7** | Memgraph HA 集群 (leader-follower, replica set) | ⚪ **等待** (P3-F+ 后续阶段) | 后续阶段 |
| **G-8** | 5 域 Lead 真人到位 timeline | 🟢 **已闭环** (per守门 #14 v4 反转 + §1.4) | §1.4 DDD 拍板 |
| **G-9** | 跟 TMO 9 节点 (任务卡 DAG) 边界 | 🟢 **已闭环** (per §1.1 DDD 拍板) | §1.1 DDD 拍板 |
| **G-10** | ARG Schema V2 迁移路径 | 🟡 **本 DDD Review 拍板, 实装落地 P3-E 阶段 5/7** | §1.3 DDD 拍板 |
| **G-11** | 成就可分享的 PNG 导出 + 描述 JSON + 周报模板 | ⚪ **等待** (P3-E 之后, 后续阶段) | 后续阶段 |
| **G-12** | ARG 跟 RGS 仓的独立边界 (per AGENTS.md §5 仓库拓扑硬约束) | 🟢 **已闭环** (per DD §15 持续维护) | DD §15 |

**累计**: 7 闭环 (G-1/G-3/G-5/G-6/G-8/G-9/G-12) + 2 续做 (G-4/G-10) + 2 等待 (G-7/G-11) = 11 项 100% 显式列 (DD §13 8 项 + 1.4 G-8 闭环 + 1.2 G-4 拍板 + 1.3 G-10 拍板).

---

## 4. 跨 session 续做 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 Mavis 审核决策)

- (a) **G-4 实装落地** (per §1.2 拍板): DD §4.7 ARGTrustEngine 新增 4 重审计 (trust_score 阈值 0.95 + mutual + evidence + multi-source + audit log) + DD §3.2.5 共享类型 (TrustAuditLog) + 守门 #13 d 审计 trigger 100% 必携. 跨 session 续, Mavis 审核决策 author=Ulysses 直接落地
- (b) **G-10 P3-E 阶段 5/7 写 `arg_migration` v1→v2 脚本** (per §1.3 拍板): 4 阶段渐进式迁移 (双写 2 周 + V2 优先 2 周 + V1 只读 1 周 + V1 退役 1 周) 跨 session 续, Mavis 审核决策 author=Ulysses 直接落地
- (c) **G-7 Memgraph HA 集群** (P3-F+ 后续阶段, ⚪ 等待)
- (d) **G-11 成就可分享 PNG 导出** (P3-E 之后, 后续阶段, ⚪ 等待)
- (e) **ARG.11 5 域 Lead 真人到位流程** (per 守门 #14 v4 真人代签全取消, 不再 trace 到位 timeline, Mavis 审核决策 author=Ulysses 直接落地, 真人到位后可以独立发令修改策略 per 8/27 19:39 JST 授权 + 9/8 15:19 JST 第 6 次强化)

---

## 5. 守门规则 (18 项 Gate Rules, per AGENTS.md §4 守门硬约束)

per AGENTS.md §4 守门硬约束 (14 main + 24 派生规 = 38 项) 跟 ARG.10 DDD Review 相关:

| # | 守门 | 派生约束 | 验证位置 |
|---|---|---|---|
| 1 | **#1 R-05** | docs 改动不跑 cargo, 不 push origin (本 commit 落档后等 user 拍板推) | §5 + §7 修订历史 |
| 2 | **#1 v15 docs 同步饱和** | 本 commit 1 新事件触发, 守门 #12 第 64 次仍允许 | §5 + §7 |
| 3 | **#1 v19 自动化档判定** | [S] 父会话直跑, 0 子代理 RPC, 0 cargo 改动, 0 测试 stage | §2 验证摘要 |
| 4 | **#3 5 域独立 Lead** | ARG 关系图 enforce delegates_to / consults / collaborates_with 跨域协调 (per G-9 拍板 (a) L0 协调) | §1.1 G-9 (a) + ARG.1 EdgeOps |
| 5 | **#9 v19 Mavis 自驱** | 0 子代理 RPC 派, 父会话直跑 0 重试 (per守门 #9 子代理不可靠实证 #7 父会话接手模式) | §0 目的 + §5 |
| 6 | **#9 v20 子代理 dispatch 必先 brief** | 0 子代理派, 不需要 brief 落档 (per §0 实装路径) | §0 目的 |
| 7 | **#10 代签规则** | 修订人 = `Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手` (per 19:39 JST 授权) | §7 修订历史 |
| 8 | **#11 缺标比错标** | 4 已知缺口 显式列, 不沿用 v0.1 错标 (per守门 #11) | §3 已知缺口 |
| 9 | **#12 AI 協作文档治理** | 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 (per ARG DD §13) | §5 + §7 修订历史 |
| 10 | **#13 a L1↔L1 禁止通信** | G-9 拍板 (a) ARG 跟 TMO 边界 L0 协调, 禁止 L1↔L1 直连 (per DD §5 PyO3 协议) | §1.1 G-9 (a) |
| 11 | **#13 c Master 100% RLS** | agents + edges 2 张 Master 表 100% RLS 必携 + SCD Type 2 (per G-10 拍板 (a) 阶段 1 双写期间) | §1.3 G-10 (a) + ARG.1 + ARG.6 |
| 12 | **#13 d Transaction 100% audit / Work 100% retention** | decision_audit + relationship_events + unlocks 3 T 表 100% audit + WORM; template_instances 1 W 表 100% retention 30d; **G-4 拍板 (c) trusts 跳过 verify 强制写 audit_audit_event 表 (WORM append-only per ADR-0043)** | §1.2 G-4 (c) + ADR-0043 |
| 13 | **#14 v4 Mavis 审核决策 author=Ulysses** | 9/10 12:45 JST 守门 #14 v3→v4 反转落地, Mavis 审核决策 author=Ulysses, 真人代签全取消, 5 签字栏 v0.1 升版 | §0 目的 + §6 签字栏 + AGENTS.md §4 #14 |
| 14 | **#19 v19 agent 交互 Python 化** | G-10 拍板 (a) P3-E 写 `arg_migration` v1→v2 脚本走 Python (per守门 #19 v19 [P] docs 同步) | §1.3 G-10 (a) |
| 15 | **#22 调试控制台不污染 main** | 0 子代理派, 0 cargo 改动, 0 测试 stage | §5 守门合规 |
| 16 | **#23 AI 修改 mock** | 0 OpenAI/Anthropic 外部 API, 全本地 Python 脚本 | §5 守门合规 |
| 17 | **#24 调试控制台走 subprocess** | 0 console_server.py 调用, 0 subprocess 派 | §5 守门合规 |
| 18 | **R-05 推 origin** | 本 commit 落档后等 user 拍板推 origin (守门 #1 R-05 反转 9/3 11:07 JST 允许推, 9/5 04:03 JST 守门 "拍板推荐项直接执行") | §6 签字栏 备注 |

**累积规 (per 守门 #1 派生 v19+)**: 后续 ARG.10 实装落地 (G-4 / G-10) 任一子项必先判定自动化档 ([P]/[M]/[S]), 命中 ≥ 2 维 (R/V/S/A) 强制走 `scripts/automation/<purpose>.py` 落地; commit message 含脚本相对路径; 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`; [P] 子项 docs 同步必更新 `docs/automation-design.md` §4 + `scripts/automation/registry.md`. **任何阶段缺其一 = 守门不完整** (per 守门 #1 v19 + #9 v20 + #12 v21 派生规).

---

## 6. 签字栏 (Signatures, 5 角色 per AGENTS.md §3 7 段结构)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-09-10 | 🟢 Mavis 接手审核决策通过 (per 9/10 15:30 JST 拍板 + 8/27 19:39 JST 授权 + 守门 #14 v4 9/10 12:45 JST 反转, 真人代签全取消改为 mavis 审核); 7 段结构 + 3 gaps 拍板 (G-9/G-4/G-10) + 1 gap 闭环 (G-8) + 5 签字栏 v0.1 升版 + 18 守门合规 + 4 已知缺口 显式列 + 4 跨 session 续 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 🟢 Mavis 审核决策代签 (per 守门 #14 v4 永久代签维持, 真人代签全取消, 5 域独立真实身份签字请 DDD Review 阶段补, 跟 v0.32 候选 v3 协议保持一致); 18 守门合规 + 4 已知缺口 + 4 跨 session 续 + 1 闭环 G-8 + 3 拍板 G-9/G-4/G-10 落档 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 🟢 Mavis 审核决策代签 (per 守门 #14 v4 永久代签维持); 5 签字栏 v0.1 升版 (per守门 #14 v4 Mavis 审核决策 author=Ulysses) + 4 跨 session 续 拍板 (G-4 实装 + G-10 P3-E 阶段 5/7 + G-7/G-11 等待 + ARG.11 5 域 Lead 真人到位 跨 session 续) |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 🟢 Mavis 审核决策代签 (per 守门 #14 v4 永久代签维持); DDD Review 3 拍板决议 + 1 闭环 (G-8) + 7 段结构 (per AGENTS.md §3) + 18 守门合规 (per 守门 #1 派生 v1-v25 + 守门 #9 v3+v7+v19+v20 + 守门 #13 a+c+d + 守门 #14 v4 + 守门 #19 v19) + 4 已知缺口 显式列 (per 守门 #11 缺标比错标) |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 🟢 Mavis 审核决策代签 (per 守门 #14 v4 永久代签维持); 选项 3 分阶段批延续 ARG.1-7 P3-C/P3-D 用 P3 余量 13.1M + ARG.8 3-4M + ARG.9 0.1M = ~13.1-14M 实测在 P3 余量 13.1M 内; ARG.10 DDD Review 拍板落地 0 额外 token 消耗 (纯文档, 0 子代理 RPC, 0 cargo 改动) + 5 签字栏 v0.1 升版 (per 守门 #14 v4 9/10 12:45 JST 反转) |

---

## 7. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 授权) | 初版: 3 gaps 拍板 (G-9 ARG 跟 TMO 边界 L0 协调 5 子项 a-e + G-4 trusts 跳过 verify 4 重审计 + G-10 Schema V2 4 阶段渐进式迁移) + 1 gap 闭环 (G-8 5 域 Lead 真人到位 守门 #14 v4 反转) + 7 段结构 (目的/决议矩阵/验证摘要/已知缺口/跨 session 续/守门规则/签字栏/修订) + 5 签字栏 v0.1 升版 (Mavis 审核决策 author=Ulysses per 守门 #14 v4 反转 9/10 12:45 JST, 真人代签全取消) + 4 已知缺口 (3 拍板 + 1 闭环) 显式列 (per 守门 #11 缺标比错标) + 18 守门合规 (含 #1 v1-v25 + #9 v3+v7+v19+v20 + #13 a+c+d + #14 v4 + #19 v19 + R-05 推 origin) + 4 跨 session 续 (G-4 实装 + G-10 P3-E 阶段 5/7 + G-7/G-11 等待 + ARG.11 跨 session 续); WBS-001 §14.11 ARG.10 升 🟡→🟢, ARG 11 子项 10/11 收官 90.9%; 守门 #1 v15 docs 同步饱和第 64 次新事件触发 仍允许; 0 子代理 RPC 派, 0 cargo 改动, 0 测试 stage, 纯 DDD Review 拍档档落 | 2026-09-10 15:30 JST Mavis 审核决策 author=Ulysses (per 守门 #14 v4 + 9/8 15:19 第 6 次强化 Mavis 全权代理 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #9 v19 + 守门 #1 v15 docs 同步饱和第 64 次新事件触发 仍允许 + ARG.10 拍板落地 0 额外 token 消耗) |

---

## 8. 引用文档 (References)

### 8.1 3 份主文档 + 9 份架构 docs (ARG 配套)

- [SRS-AGENT-RELATIONSHIP-001.md v0.1](../requirements/SRS-AGENT-RELATIONSHIP-001.md) — 要件定義書 (663 行, commit `0bacaeb`)
- [BD-AGENT-RELATIONSHIP-001.md v0.1](BD-AGENT-RELATIONSHIP-001.md) — 基本設計書 (1088 行, commit `464a646`)
- [DD-AGENT-RELATIONSHIP-001.md v0.1.1](DD-AGENT-RELATIONSHIP-001.md) — 詳細設計書 (2284 行, commit `49c8938` + `a697284` self-review)
- [06-arg-02-arg-bridge.md](../architecture/2026-09-03-arg/06-arg-02-arg-bridge.md) · [07-arg-03-5-sa-impl.md](../architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md) · [08-arg-04-13rest-1ws.md](../architecture/2026-09-03-arg/08-arg-04-13rest-1ws.md) · [09-arg-05-frontend-e2e.md](../architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md) · [10-arg-06-pg-persistence.md](../architecture/2026-09-03-arg/10-arg-06-pg-persistence.md) · [11-arg-07-arg-saga.md](../architecture/2026-09-03-arg/11-arg-07-arg-saga.md) · [12-arg-08-memgraph-bridge.md](../architecture/2026-09-03-arg/12-arg-08-memgraph-bridge.md) · [13-arg-09-5-domain-rbac.md](../architecture/2026-09-03-arg/13-arg-09-5-domain-rbac.md) · [14-arg-10-arg-frontend.md](../architecture/2026-09-03-arg/14-arg-10-arg-frontend.md)

### 8.2 ARG 实装报告 + 收官报告

- [PHASE-ARG-IMPL-REPORT.md v0.1](../reports/PHASE-ARG-IMPL-REPORT.md) — ARG 11 子项实装收官报告 (8/11 收官 81.8%, 9/11 升档 81.8% v0.60)
- [PHASE-ARG-08-IMPL-REPORT.md](../reports/PHASE-ARG-08-IMPL-REPORT.md) (164 行 7 段) · [PHASE-ARG-07-IMPL-REPORT.md](../reports/PHASE-ARG-07-IMPL-REPORT.md) (16.7KB 9 段) · [PHASE-ARG-06-IMPL-REPORT.md](../reports/PHASE-ARG-06-IMPL-REPORT.md) (8.2KB 7 段) · [PHASE-ARG-04-IMPL-REPORT.md](../reports/PHASE-ARG-04-IMPL-REPORT.md) (20.2KB) · [PHASE-ARG-01-IMPL-REPORT.md](../reports/PHASE-ARG-01-IMPL-REPORT.md) (19.4KB)

### 8.3 守门 + 关联

- [AGENTS.md §4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 14 main + 24 派生规 = 38 项硬约束
- [AGENTS.md §4 #14 v4 守门](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 守门 #14 v3 → v4 反转 (9/10 12:45 JST, 真人代签全取消改为 mavis 审核 author=Ulysses)
- [AGENTS.md §4 #14 v3 反转 9/8 15:19 JST](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 守门 #14 v3 升级第 6 次强化 (Mavis 全权代理 Ulysses 决策)
- [AGENTS.md §4 #14 v3 升级 9/9 12:02 JST](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 守门 #14 v3 升级第 7 次强化 (Mavis 自驱)
- [WBS-001 v0.60 §14.11](STAR-P3-WBS-001.md) — ARG 11 子项 9/11 收官 81.8% (ARG.1-9 实质收官, ARG.10/11 跨 session 续)
- [WBS-001 v0.70 §14.19 v32 候选拍板激活](STAR-P3-WBS-001.md) — 守门 #14 v3→v4 反转 (真人代签全取消改为 mavis 审核)
- [docs/guardian/v30_signature_boundary.md](../guardian/v30_signature_boundary.md) — 守门 v30 政策**已取消** (v0.62 反转)
- [docs/guardian/v31_lead_traceback.md](../guardian/v31_lead_traceback.md) — 守门 v31 政策**已取消** (v0.62 反转)
- [docs/guardian/v32_audit_boundary.md](../guardian/v32_audit_boundary.md) — 守门 v32 替代政策 🟢 active (v0.70 拍板激活, Mavis 审核决策 author=Ulysses)
- [DD-LangGraph-TMO-001 §3.2.1.1 M-N1..M-N7](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-09-03-langgraph/03-detailed-design.md) — TMO 7 节点任务卡 DAG 边界 (G-9 拍板参考)
- [DD-AGENT-RELATIONSHIP-001 §5 PyO3 协议](DD-AGENT-RELATIONSHIP-001.md) — ARG 跟 L0 PyO3 binding 协议 (G-9 拍板参考)
- [ADR-0043 audit_audit_event WORM](https://github.com/UlyssesLeoLee/Star/blob/main/docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md) — audit_audit_event WORM 落地 (G-4 拍板 (c) 引用)
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens
- [docs/automation-design.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/automation-design.md) — agent 交互 Python 化 (守门 #19 v19)
- [docs/recruitment/5-business-domain-lead-referral.md v0.3](https://github.com/UlyssesLeoLee/Star/blob/main/docs/recruitment/5-business-domain-lead-referral.md) — 5 域 Lead 寻访流程 v0.3 反转 (G-8 闭环, 守门 #14 v4 不再 trace 到位)
