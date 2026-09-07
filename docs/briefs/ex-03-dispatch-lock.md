# Brief: EX-03 — L0 DispatchLockManager (Python)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-03-dispatch-lock`
> **base**: `main` (阶段 1 merge 后)
> **关联**: [03-detailed-design.md §1.1 + §2.2.1 + §4.1 + §4.2](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-03](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 1. 目标

在 `scripts/automation/exclusion/` 落档 `dispatch_lock.py` + `idempotency_key_store.py`, 实现 L0 派发级排他 (60s TTL) + 派发级幂等 (24h dedup, 双键 client_uuid + business_hash).

## 2. 范围

### 2.1 In-Scope

- `dispatch_lock.py` — `DispatchLockManager` 类 (acquire / release / is_locked) + `LockGuard` dataclass + `DispatchLockHeld` exception + `_compute_lock_class_id` 静态方法 (per `03 §2.2.1`)
- `idempotency_key_store.py` — `IdempotencyKeyStore` 类 (insert / get / dedup 双键) + 双键 dedup 逻辑
- 1 集成测试 (per `02 §7.2 IT-02` + `IT-08` idempotency RLS)
- 2 UT (per `02 §7.1 UT-13` + `UT-14` + `UT-18`)

### 2.2 Out-of-Scope

- ❌ SubAgentLock (EX-04 范围)
- ❌ star-mutex crate 共享 (EX-02 范围)
- ❌ UI 客户端 (EX-05 范围)
- ❌ star-mcp 16 tool 幂等 (EX-06 范围)

## 3. 已知缺口

- 无真实 PG, IT 用 mock asyncpg pool (per H2 v18 实证)
- 双键 hash 算法版本兼容 (per G-EI-04) 需 DDD Review Lead 拍板, 暂用 SHA-256
- 跨 cluster 锁不支持 (per G-EI-02) 已知

## 4. 守门

1. **守门 #1 Python 化**: 走 `python -m pytest` + `py_compile` + `ruff` + `mypy --strict` 4 步
2. **守门 #5 env 安全**: 不打印 DATABASE_URL, 仅引用
3. **守门 #6 PowerShell**: 测试运行命令 PowerShell 兼容
4. **守门 #7 0 unsafe**: Python 无 unsafe, 走 mypy --strict 验证
5. **守门 #9 v3**: Mavis 直接落地 (per 守门 #19 v19 + 实证 5/5 subagent RPC 不可靠)
6. **守门 #10 author=Ulysses**
7. **守门 #11 缺标比错标**: §3 显式列 3 缺口
8. **守门 #13 a L0 协调**: 类注释明示 L2 SubAgent 不直接调
9. **守门 #19 v19**: 走 `scripts/automation/exclusion/` (per 守门 #19 强制)
10. **守门 #22 控制台不污染**: 不动 console_server.py (后续 EX-07 扩展)
11. **守门 #23 AI mock**: 不开外部 API

## 5. 依赖

### 5.1 上游

- EX-01 (idempotency_keys 表存在)

### 5.2 下游

- EX-05 (UI IdempotencyManager) — 调 L0 dispatch 接口
- EX-06 (star-mcp 16 tool 幂等) — 调 IdempotencyKeyStore
- EX-07 (可观测性) — 调 DispatchLockManager 暴露 metric

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `scripts/automation/exclusion/__init__.py` | 包 init (~0.5KB) |
| 2 | `scripts/automation/exclusion/dispatch_lock.py` | DispatchLockManager (~5KB) |
| 3 | `scripts/automation/exclusion/idempotency_key_store.py` | IdempotencyKeyStore 双键 (~3KB) |
| 4 | `scripts/automation/exclusion/tests/__init__.py` | 测试包 init (~0.2KB) |
| 5 | `scripts/automation/exclusion/tests/test_dispatch_lock.py` | UT-13 + UT-14 (~2KB) |
| 6 | `scripts/automation/exclusion/tests/test_idempotency_key_store.py` | UT-18 (~2KB) |
| 7 | `scripts/automation/exclusion/tests/it_idempotency_rls.py` | IT-02 (~2KB) |
| 8 | `docs/briefs/ex-03-dispatch-lock.md` | 本 brief |

**总 8 文件, ~15KB raw**

## 7. 验收

- [ ] `python -m py_compile scripts/automation/exclusion/*.py` 0 err
- [ ] `python -m pytest scripts/automation/exclusion/tests/` 100% pass (UT 5/5 + IT 1/1)
- [ ] `python -m ruff check scripts/automation/exclusion/` 0 警告
- [ ] `python -m mypy --strict scripts/automation/exclusion/` 0 err
- [ ] `git log -p --follow scripts/automation/exclusion/dispatch_lock.py` 实证类完整
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 Mavis 直接落地 (per 守门 #19 v19 + #9 v3 fallback)

1. 创建 wt (post EX-01 merge): `git worktree add ../.worktrees/wt-ex-03-dispatch-lock -b wt-ex-03-dispatch-lock main`
2. Mavis 在 wt 内写 8 文件 (per 守门 #9 v3 实证 5/5 subagent RPC 不可靠, 简单 Python 任务 Mavis 直接落地最稳)
3. 实证守门 #1 Python 4 步
4. `git add` + `git commit -m "..."` author=Ulysses
5. 切回 main + `git merge --no-ff wt-ex-03-dispatch-lock`
6. 阶段 2 跨 3 wt cargo check 实证 (本子项不动 Rust, 应 0 err)

## 9. 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 无 PG asyncpg pool mock 复杂度 | 中 | 中 | 用 `aiopg` 替代或简化 mock |
| 双键 hash 算法版本兼容 | 中 | 中 | SHA-256 稳定, 不影响 |
| 60s TTL 测时间敏感 | 中 | 低 | 测用 mock time, 注入 clock |

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿 (per 拍板).
