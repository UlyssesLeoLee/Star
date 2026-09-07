# Brief: EX-04 — L1 SubAgentLock (Python)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-04-subagent-lock`
> **base**: `main` (阶段 1 merge 后)
> **关联**: [03-detailed-design.md §1.1 + §2.2.2 + §3.3 + §4.3](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-04](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 1. 目标

在 `scripts/automation/exclusion/` 落档 `subagent_lock.py` + `lock_watcher.py`, 实现 L1 任务卡持锁 (lease + heartbeat 30s + 过期 300s) + 锁过期检测.

## 2. 范围

### 2.1 In-Scope

- `subagent_lock.py` — `SubAgentLock` 类 (acquire / heartbeat / release) + `LeaseHandle` dataclass + `ReleaseReason` enum + `SubAgentLockHeld` exception + `_start_heartbeat` (per `03 §2.2.2`)
- `lock_watcher.py` — `LockWatcher` 类 (lease 过期检测 + 告警)
- 1 集成测试 (per `02 §7.2 IT-03` lease_log heartbeat 30s 续约)
- 3 UT (per `02 §7.1 UT-15` + `UT-16` + `UT-17`)

### 2.2 Out-of-Scope

- ❌ DispatchLockManager (EX-03 范围)
- ❌ star-mutex crate 共享 (EX-02 范围)
- ❌ UI / MCP / 可观测性 (EX-05/06/07 范围)

## 3. 已知缺口

- 无真实 PG, IT 用 mock asyncpg pool
- heartbeat 30s 真实环境测需 30+ s, 测用 mock time 注入
- 锁泄漏告警阈值 24h 1h (per G-EI-05) 需 SRE Lead 拍板, 暂用 1h 保守值

## 4. 守门

1. 守门 #1 Python 4 步 (pytest + py_compile + ruff + mypy --strict)
2. 守门 #5 env 安全
3. 守门 #6 PowerShell
4. 守门 #9 v3: Mavis 直接落地 (实证 RPC 不可靠)
5. 守门 #10 author=Ulysses
6. 守门 #11 缺标比错标
7. 守门 #13 a L0 协调: L1 SubAgent 不直接调 L3 Domain 互抢, 必须经 L0 派发 (注释明示)
8. 守门 #19 v19: 走 `scripts/automation/exclusion/`
9. 守门 #22 控制台不污染

## 5. 依赖

### 5.1 上游

- EX-01 (lease_log 表存在)

### 5.2 下游

- EX-05 (UI 显示 SubAgent 锁状态)
- EX-07 (可观测性 LockWatcher 暴露 metric)

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `scripts/automation/exclusion/subagent_lock.py` | SubAgentLock + heartbeat (~5KB) |
| 2 | `scripts/automation/exclusion/lock_watcher.py` | LockWatcher (~3KB) |
| 3 | `scripts/automation/exclusion/tests/test_subagent_lock.py` | UT-15 + UT-16 + UT-17 (~3KB) |
| 4 | `scripts/automation/exclusion/tests/it_lease_heartbeat.py` | IT-03 (~2KB) |
| 5 | `docs/briefs/ex-04-subagent-lock.md` | 本 brief |

**总 5 文件, ~13KB raw**

## 7. 验收

- [ ] `python -m py_compile scripts/automation/exclusion/{subagent,lock_watcher}_*.py` 0 err
- [ ] `python -m pytest scripts/automation/exclusion/tests/test_subagent_lock.py` 100% pass (3/3 UT)
- [ ] `python -m pytest scripts/automation/exclusion/tests/it_lease_heartbeat.py` 100% pass (1/1 IT, mock time)
- [ ] `python -m mypy --strict scripts/automation/exclusion/` 0 err
- [ ] `git log -p --follow scripts/automation/exclusion/subagent_lock.py` 实证类完整
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 Mavis 直接落地

1. 创建 wt (post EX-01 merge): `git worktree add ../.worktrees/wt-ex-04-subagent-lock -b wt-ex-04-subagent-lock main`
2. Mavis 在 wt 内写 5 文件
3. 实证守门 #1 Python 4 步
4. `git add` + `git commit -m "..."` author=Ulysses
5. 切回 main + `git merge --no-ff wt-ex-04-subagent-lock`
6. 阶段 2 跨 3 wt cargo check 实证

## 9. 风险

- heartbeat 真实环境 30s 等待测时间敏感 → 测用 mock clock 注入
- L1 SubAgent 崩溃模拟 → 测用 `kill -9` 不可, 改 mock subprocess crash

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿.
