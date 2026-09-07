# Brief: EX-08 — 集成测试 + 性能压测 (Py + Rust + TS)

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项
> **wt-branch**: `wt-ex-08-tests-load`
> **base**: `main` (阶段 3 merge 后)
> **关联**: [03-detailed-design.md §7 18 UT + 8 IT + 12 E2E](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [PHASE §1.1 EX-08](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md)

---

## 1. 目标

落档完整测试套件: 18 UT (跨 Rust/Python/TypeScript) + 8 IT (跨模块) + 12 E2E (8 想定シナリオ + 4 跨层) + S-08 1000 并发用户压测, 100% 覆盖 8 想定シナリオ (per `01 §5 S-01..S-08`) + 守门 #1 4 步实证.

## 2. 范围

### 2.1 In-Scope

#### 18 UT (per `03 §7.1`)

- **TS (4)**: UT-01/02/03/04 (frontend/src/lib/exclusion + components/exclusion)
- **Rust (8)**: UT-05/06/07/08/09/10/11/12 (crates/star-mutex)
- **Python (6)**: UT-13/14/15/16/17/18 (scripts/automation/exclusion)

#### 8 IT (per `03 §7.2`)

- IT-01: PG advisory lock 获取/释放事务一致性 (Rust, mock pool)
- IT-02: idempotency_keys 表 RLS 13 类隔离 (Python, mock pool)
- IT-03: lease_log 表 + heartbeat 30s 续约 (Python, mock time)
- IT-04: advisory_lock_audit 写 audit_audit_event (Rust, mock pool)
- IT-05: 22 domain crate 通过 StarMutexAdapter 接入 (Rust, mock pool)
- IT-06: star-mcp 16 tool 加 Idempotency-Key 解析 (Rust, mock pool)
- IT-07: 锁 metric 5 项 Prometheus 暴露 (Python, mock prom)
- IT-08: 锁泄漏告警 24h 1h 阈值 (Python, mock time + mock slack)

#### 12 E2E (per `03 §7.3`)

- E2E-01: S-01 同一用户重复点 Send 按钮 → 1 次服务端写入
- E2E-02: S-02 网络抖动服务端重试 → 1 次副作用
- E2E-03: S-03 User A + User B 同时"暂停 task X" → 后到者 409
- E2E-04: S-04 SA-01 + SA-04 同时改 work_item → version CAS 重试
- E2E-05: S-05 bulk_node 暂停 5 任务中途 User C 修改 1 个
- E2E-06: S-06 L1 SubAgent 崩溃 lease 过期
- E2E-07: S-07 客户端漏带 Idempotency-Key 双键 fallback
- E2E-08: S-08 1000 并发用户压测 advisory lock QPS
- E2E-09: 跨层 trace_id 4 层贯穿 → 1 click 定位
- E2E-10: 16 tool 幂等: 同 key 重试 → 1 次副作用
- E2E-11: 锁泄漏 24h 1h 阈值告警 + 自动 force-release
- E2E-12: 22 domain crate 并发改同 work_item → 1 个成功 + 21 个 VersionMismatch

#### S-08 1000 并发压测

- 1000 并发用户同时改不同 work_item 行
- 验证 PG advisory lock QPS > 5K (per NFR-PERF-03)
- P50 延迟 < 5ms, P99 延迟 < 50ms
- 0 锁泄漏, 0 死锁

### 2.2 Out-of-Scope

- ❌ 任何业务代码 (EX-01..EX-07 范围)
- ❌ 真实 PG 部署 (mock pool 用于本地测试)
- ❌ 真实 Slack 告警 (mock 实现)

## 3. 已知缺口

- 真实 PG 实例缺失, 所有测试用 mock (per 守门 #13 实证 5/5 subagent RPC 不可靠)
- 1000 并发压测受本机资源限制, 估 5K-10K QPS 上限
- E2E 跨层测需 UI + L0 + L1 + L3 全部跑, 需 console_server 启动 + Rust 编译

## 4. 守门

1. 守门 #1 4 守门 (per 守门 #1 v19 + #1 v25 + 拍板 D 选项 1):
   - `cargo check --workspace --all-targets -j 4` 0 err
   - `cargo fmt + clippy` 0 警告
   - `cargo test -p star-mutex -p star-mcp --lib -j 4` 100% pass (单 crate, 跳 workspace per 守门 #1 v25)
   - `python -m pytest scripts/automation/exclusion/tests/` 100% pass
   - `pnpm test --testPathPattern=exclusion` 100% pass
   - `cargo build --release` 0 err
2. 守门 #5 env 安全
3. 守门 #6 PowerShell
4. 守门 #7 0 unsafe
5. 守门 #9 v3: 派 worker 子代理 (测试量大)
6. 守门 #10 author=Ulysses
7. 守门 #11 缺标比错标
8. 守门 #22 控制台不污染 (per 守门 #22 实证, 测试跑后 cargo check 0 err)
9. 守门 #23 AI mock

## 5. 依赖

### 5.1 上游

- EX-01..EX-07 全部 merge (本子项是综合验证)

### 5.2 下游

- 无 (本子项是最后阶段)

## 6. 交付物

| # | 路径 | 描述 |
|---|---|---|
| 1 | `crates/star-mutex/tests/lib_test.rs` (扩) | 6 UT (per `03 §7.1 UT-05..UT-10`) (~6KB) |
| 2 | `crates/star-mutex/tests/version_cas_test.rs` | UT-07 + UT-08 (~3KB) |
| 3 | `crates/star-mutex/tests/it_advisory_lock.rs` | IT-01 + IT-04 + IT-05 (~5KB) |
| 4 | `crates/star-mcp/tests/it_idempotency_middleware.rs` | IT-06 (~3KB) |
| 5 | `frontend/src/lib/exclusion/__tests__/*.test.ts` | 3 UT (~5KB) |
| 6 | `frontend/src/components/exclusion/__tests__/*.test.tsx` | 1 UT (~2KB) |
| 7 | `scripts/automation/exclusion/tests/test_*.py` | 6 UT (~10KB) |
| 8 | `scripts/automation/exclusion/tests/it_*.py` | 4 IT (~8KB) |
| 9 | `tests/e2e/s01_*.spec.ts` ~ `tests/e2e/s08_*.spec.ts` + `tests/e2e/e09_*.spec.ts` ~ `tests/e2e/e12_*.spec.ts` | 12 E2E (~30KB) |
| 10 | `tests/load/s08_1000_concurrent.py` | S-08 1000 并发压测脚本 (~5KB) |
| 11 | `docs/briefs/ex-08-tests-load.md` | 本 brief |

**总 11 类文件, ~77KB raw**

## 7. 验收

### 7.1 守门实证 (per 守门 #1 4 守门 + 拍板 D 选项 1)

- [ ] `cargo check --workspace --all-targets -j 4` exit 0, 0 err
- [ ] `cargo fmt --all -- --check` 0 diff
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 0 err
- [ ] `cargo test -p star-mutex -p star-mcp --lib -j 4` 100% pass (14/14 UT, 跨 2 crate, 走守门 #1 v25 跳过 workspace)
- [ ] `python -m pytest scripts/automation/exclusion/tests/` 100% pass (6/6 UT + 4/4 IT = 10/10)
- [ ] `pnpm test --testPathPattern=exclusion` 100% pass (4/4 UT)
- [ ] `cargo build --release` exit 0
- [ ] S-08 1000 并发压测 30 min 稳定: P50 < 5ms, P99 < 50ms, 0 锁泄漏, 0 死锁

### 7.2 8 想定シナリオ E2E

- [ ] E2E-01..E2E-08 8 想定シナリオ 100% pass (per `01 §5`)
- [ ] E2E-09..E2E-12 4 跨层验证 100% pass (per `03 §7.3`)

### 7.3 git 实证

- [ ] `git log -p --follow tests/e2e/s01_client_dedup.spec.ts` 实证 E2E-01 完整
- [ ] 1-2 commit 含全部 11 类文件
- [ ] commit author = Ulysses per 守门 #10

## 8. 实施路径

### 8.1 派 worker 子代理 (守门 #9 v3 推荐, 但需强监控)

1. 创建 wt (post 阶段 3 merge): `git worktree add ../.worktrees/wt-ex-08-tests-load -b wt-ex-08-tests-load main`
2. 派 worker 子代理: prompt 含本 brief + 守门 + 路径
3. worker 在 wt 内写 11 类文件 + 实证守门 #1 4 步
4. **Mavis 端强监控**: worker RPC 不可靠实证, 必 `git log -p --follow` 实证各文件 + 守门 4 步实证
5. 切回 main + `git merge --no-ff wt-ex-08-tests-load`
6. 阶段 4 后跨全 8 子项 cargo check + cargo test 实证 (per 拍板 D 选项 1)

### 8.2 worker 失败接手 (per 守门 #9 v3 fallback)

- worker status="succeeded" 但 git log 没看到 commit → Mavis 直接接手
- worker RPC 失败 → Mavis 直接接手
- 测试 fail 不收敛 → 暂停, 跨 session 续

## 9. 风险

- 守门 #1 v25 cargo test 跳过 workspace, 但 EX-08 需跨 crate IT → 走 `cargo test -p star-mutex -p star-mcp --tests` 双 crate IT
- 1000 并发压测本机资源限制 → 用 `wrk` 或 `locust` 而非 Python 简易并发
- 12 E2E 跨 UI + backend 需多进程协调 → 测用 docker-compose 或 subprocess

## 10-11. 签字 / 修订历史

5 角色 Mavis 临时代签 + v0.1 初稿.
