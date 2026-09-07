# Brief: EX-05 — UI IdempotencyManager (TypeScript)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-05-ui-idem`
> **base**: `main` (阶段 2 merge 后)
> **关联**: [03-detailed-design.md §1.1 + §2.1.1 + §2.1.2 + §6](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-05](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 1. 目标

在 `frontend/src/lib/exclusion/` 落档 4 源文件 + `frontend/src/components/exclusion/` 落档 3 component, 实现客户端 dedup (双键 + AbortController) + 锁状态可视化 (角标 + 详情面板 + 错乱 toast) + 跨层 trace_id 传递.

## 2. 范围

### 2.1 In-Scope

- `frontend/src/lib/exclusion/idempotency.ts` — `IdempotencyManager` 类 (per `03 §2.1.1`)
- `frontend/src/lib/exclusion/lock_status.ts` — `LockStatusTracker` 类
- `frontend/src/lib/exclusion/trace_propagator.ts` — `TraceIdPropagator` (browser)
- `frontend/src/lib/exclusion/business_hash.ts` — `BusinessKeyHasher` (per `03 §2.1.2`)
- `frontend/src/components/exclusion/LockStatusBadge.tsx` — 顶部角标 (per `03 §6.2`)
- `frontend/src/components/exclusion/LockStatusPanel.tsx` — 详情面板
- `frontend/src/components/exclusion/LockConflictToast.tsx` — 错乱 toast (per `03 §6.3`)
- 4 UT (per `02 §7.1 UT-01` + `UT-02` + `UT-03` + `UT-04`)

### 2.2 Out-of-Scope

- ❌ 任何 backend (EX-02/03/04/06 范围)
- ❌ AppShell 集成 (后续 H2 阶段, 本子项仅组件实现)

## 3. 已知缺口

- AppShell 集成在 H2 阶段, 本子项仅 4 lib + 3 component
- 双键 hash 算法版本兼容 (per G-EI-04) 跟 EX-03 一致, SHA-256

## 4. 守门

1. 守门 #1 TypeScript 4 步: `pnpm tsc --noEmit` 0 err + `pnpm eslint` 0 警告 + `pnpm test` 100% pass + `pnpm build` 0 错
2. 守门 #6 PowerShell
3. 守门 #7 0 unsafe: TypeScript 无 unsafe
4. 守门 #9 v3: 派 worker 子代理 (per 实证 5/5 RPC 不可靠, 但前端代码相对独立可派)
5. 守门 #10 author=Ulysses
6. 守门 #11 缺标比错标
7. 守门 #19 v19: 守门 #19 仅适用 Python 子项, TypeScript 例外
8. 守门 #22 控制台不污染: console_server 是 Python, 跟本子项无关

## 5. 依赖

### 5.1 上游

- EX-03 (L0 dispatch 接口协议已落, UI 调用 ID 格式确定)

### 5.2 下游

- 后续 H2 阶段集成到 AppShell

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `frontend/src/lib/exclusion/idempotency.ts` | IdempotencyManager (~4KB) |
| 2 | `frontend/src/lib/exclusion/lock_status.ts` | LockStatusTracker (~2KB) |
| 3 | `frontend/src/lib/exclusion/trace_propagator.ts` | TraceIdPropagator (~1.5KB) |
| 4 | `frontend/src/lib/exclusion/business_hash.ts` | BusinessKeyHasher (~1.5KB) |
| 5 | `frontend/src/components/exclusion/LockStatusBadge.tsx` | 顶部角标 (~2KB) |
| 6 | `frontend/src/components/exclusion/LockStatusPanel.tsx` | 详情面板 (~3KB) |
| 7 | `frontend/src/components/exclusion/LockConflictToast.tsx` | 错乱 toast (~2KB) |
| 8 | `frontend/src/lib/exclusion/__tests__/idempotency.test.ts` | UT-01 + UT-02 (~2KB) |
| 9 | `frontend/src/lib/exclusion/__tests__/business_hash.test.ts` | UT-03 (~1KB) |
| 10 | `frontend/src/components/exclusion/__tests__/LockStatusBadge.test.tsx` | UT-04 (~1KB) |
| 11 | `docs/briefs/ex-05-ui-idem.md` | 本 brief |

**总 11 文件, ~20KB raw**

## 7. 验收

- [ ] `pnpm tsc --noEmit` exit 0, 0 err (在 frontend 目录)
- [ ] `pnpm eslint src/lib/exclusion src/components/exclusion` 0 警告
- [ ] `pnpm test --testPathPattern=exclusion` 100% pass (4/4 UT)
- [ ] `pnpm build` 0 错
- [ ] `git log -p --follow frontend/src/lib/exclusion/idempotency.ts` 实证类完整
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 派 worker 子代理 (守门 #9 v3 推荐)

1. 创建 wt (post 阶段 2 merge): `git worktree add ../.worktrees/wt-ex-05-ui-idem -b wt-ex-05-ui-idem main`
2. 派 worker 子代理: prompt 含本 brief + 守门 + 路径
3. worker 在 wt 内写 11 文件 + 实证守门 #1 TypeScript 4 步
4. Mavis 端实证: `git log -p --follow frontend/src/lib/exclusion/idempotency.ts` 实证类完整
5. 切回 main + `git merge --no-ff wt-ex-05-ui-idem`
6. 阶段 3 跨 2 wt cargo check 实证 (本子项不动 Rust, 应 0 err)

### 8.2 worker 失败接手 (per 守门 #9 v3 fallback)

- worker status="succeeded" 但 git log 没看到 commit → Mavis 直接接手
- worker RPC 失败 → Mavis 直接接手

## 9. 风险

- pnpm install / pnpm test 在本机时间 (估 30-60s) → 可接受
- 跟 frontend 现有 CI (per PR #12) 同步, 走守门 #6 v2 advisory 模式
- 守门 #5 v2 调试页 AI 修改 mock 不开外部 API (per 守门 #23 ai_edit_mock.py) — 跟本子项无关

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿.
