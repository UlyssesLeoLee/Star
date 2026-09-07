# Brief: OPT-NEXT-01 — Phase D T3.2/5.6/G-10 (per OPT-WBS-12..14, 推下 session)

**Agent**: worker (推下 session 派)
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST (per Mavis 接手代签)
**Status**: 🟡 等待 5 域 Lead 真人到位 (T3 ~ 2026-09-26 JST)
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)

---

## 1. 任务目标 (per WBS §4.2 #1)

完成 Phase D 3 子项, 实证 H2-EXT 强类型重构 + Saga 80% 覆盖 (per `STAR-P4-UNIMPL-WBS-001.md` §5):

- **D.1**: G-10 H2 类型不兼容修法 (DeviceId→Uuid 强类型 + String→Uuid 业务语义, 5 domain 跨域字段扩展)
- **D.2**: T3.2 Saga ≥80% 覆盖 (5 域 Lead 反转可启动 per 守门 #3 v2)
- **D.3**: 5.6 H2 原 3 domain 改造 (feedback/validation/integration ~150+ call sites)

**触发**: 5 域 Lead 真人 T3 (≥1 域) 到位 (per `docs/recruitment/5-business-domain-lead-referral.md` v0.1 T3 ~ 9/26)
**阻塞**: 等 `per守门 #3` 5 域 Lead 真人 (Mavis 临时代签 per 9/3 11:35 JST 反转)
**估 token**: 0.5-1.7M (per 守门 #1 v18 H2 实证 3-5x 超支, 0.3-0.5M 估 → 1.1-1.6M 实测)

## 2. 依赖 (per HANDOFF-ST-001 §1 + §5.3 5 项 Blocker)

| # | 依赖 | 状态 |
|---|---|---|
| 1 | H2-EXT #4 `domain-identity` `DeviceId→Uuid` 强类型重构 | 🔴 阻塞 (per HANDOFF v0.4 §1) |
| 2 | H2-EXT #5 `domain-work-item` `device_id: Option<String>` String→Uuid 业务语义 | 🟡 partial (拍板 String=hostname per v0.5 Q1) |
| 3 | D.1 helper `ActorContext::as_local_runtime` | 🟢 已落地 (per commit `413f5bd` Phase B B.1) |
| 4 | T3.1 共享 star-dto 公共字段 | 🟢 已落地 (per commit `5502aaf` Phase C C.2) |
| 5 | 5 域 Lead 真人到位 (match 域最关键, 触发 T3.2 Saga 覆盖) | 🔴 阻塞 |

## 3. 实装要求 (per P4-UNIMPL-WBS-001 §5 D.1-D.3)

### D.1 — H2 类型不兼容修法

- 强类型 ID 重构: `DeviceId` → `Uuid` (消除 5 域 type mismatch)
- 业务语义重设: `device_id: Option<String>` → `Option<Uuid>` (per `domain-work-item`)
- 跨域字段扩展: `workspace_ids: Vec<WorkspaceId>` (domain-project) + `tenant_policy_id: Option<TenantPolicyId>` (domain-tenant) 加到 `star_context::ActorContext` (per commit `68ae5ff` 阶段 1)
- 实证 0 err: `cargo check --workspace --all-targets -j 4`

### D.2 — T3.2 Saga ≥80% 覆盖

- 5 域 Lead match 域触发 (`docs/recruitment/5-business-domain-lead-referral.md` v0.1 5 域话术)
- Saga 跨域补偿 + 失败回滚 (per Q-003 拍板)
- E2E 测试覆盖 ≥80% (per P4-UNIMPL-WBS §5 D.2)

### D.3 — 5.6 H2 原 3 domain service.rs 改造

- 3 domain (feedback/validation/integration) port/service/invariants 改用 `star_context::ActorContext`
- 删除 3 domain 内部 `pub mod context` 子模块
- 实证 0 err + 全 pass (per H2 stage 2-3 失败实证 revert `8364223` 后 跨 session 续)

## 4. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-phase-d D:/Star/.worktrees/wt-opt-phase-d main
cd D:/Star/.worktrees/wt-opt-phase-d
```

## 5. 守门硬约束

- 守门 #1 v19: `cargo check --workspace --all-targets -j 4` 0 err
- 守门 #1 v25: `cargo test -p star-context --lib -j 4` 21+ pass
- 守门 #3: Mavis 临时代签 (5 域 Lead 真人到位后追溯)
- 守门 #10: commit author = `Ulysses <ulysses@mavis.local>`
- 守门 #12: 禁回溯叙事, BAS 引用 git 实证
- 守门 #13: W/T/M 派生 (新类型按 9/1 18:30 JST 拍板分类)
- 守门 #19: 子代理 dispatch 必先 brief (本 brief 已落档)

## 6. 提交形式

2-3 commit:
- Commit 1 (D.1 + D.3 代码): "feat(refactor): H2 强类型重构 + 3 domain 改造"
- Commit 2 (D.2 Saga): "feat(saga): T3.2 5 域 Lead 跨域补偿 + ≥80% 覆盖"
- Commit 3 (docs 实证): "docs(report): PHASE-P4-D-IMPL-REPORT.md v0.1 (7 段结构)"

## 7. 失败处理

- 5 域 Lead 真人未到位 → 报告阻塞, 不 commit
- 类型不兼容 3-5x 超支 → 跨 sub-session 续, 0.6-0.8M token 上限 (per 9/4 实证)

## 8. 状态

🟡 **推下 session** (等 5 域 Lead T3 到位触发, per `5-business-domain-lead-referral.md` v0.1)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #1
- 基线: `docs/reports/STAR-P4-UNIMPL-WBS-001.md` §5
- HANDOFF: `docs/reports/HANDOFF-ST-001.md` §1 H2-EXT 表格 + §5.3 5 项 Blocker
- 依赖: `docs/recruitment/5-business-domain-lead-referral.md` v0.1 (T0-T5 timeline)
