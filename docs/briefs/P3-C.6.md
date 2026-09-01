# Brief: P3-C.6

**Agent**: worker
**Phase**: P3-C
**Created**: 2026-09-02 02:38:59

---

C.6 Saga 跨 5 域补偿 + 失败回滚 (per docs/automation-design.md v0.1 §4.2 + WBS §2 C.6 commit 25d086e)

scope: scripts/automation/saga_e2e.py 跨 5 域 (player/economy/match/social/admin) 补偿 + 回滚 e2e 实证
base: 094284b (per automation v0.1)
mode: worker 子代理, 走 exec 替代 RPC (per 守门 #9 实证 + 守门 #20 v2)
交付:
  1. scripts/automation/saga_e2e.py 新建, 5 域补偿链: player (创建角色) → economy (扣费) → match (匹配对手) → social (发通知) → admin (审计), 任何 1 步失败回滚前 4 步
  2. SagaStep dataclass (id / domain / action / compensation / idempotency_key per INV-SG-05)
  3. 5 域 × 2 case (成功 + 失败回滚) = 10 case 实证
  4. scripts/automation/__tests__/saga_e2e_test.py 10 测试
守门: cargo check --workspace --lib 0 err + python smoke_test.py 5/5 + author Ulysses + 1 commit 1 wt
docs: commit message 含 scripts/automation/saga_e2e.py 路径 + 引用 WBS §2 C.6
已知: C.6 已收官 commit 25d086e (per WBS §2), star-saga crate 增强; 5 域 Lead 真人到位前 e2e 用 mock 域
