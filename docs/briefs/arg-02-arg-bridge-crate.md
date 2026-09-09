# ARG.2 Brief — crates/arg-bridge 4 子模块实装

> **Task ID**: arg-02-arg-bridge-crate
> **WT**: `wt-arg-02-bridge` (branch: `wt-arg-02-bridge`, base: main @ f476147 含 ARG.1 + v0.50 docs)
> **触发**: 2026-09-10 06:53 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱 第 7 次强化 + 守门 #14 v3 Mavis 永久代签维持 + 守门 #1 v15 docs 同步饱和第 44 次新事件触发 仍允许)
> **依赖**: ARG.1 (`crates/arg` 6 子模块) 已 merge @ commit `651117e` + ARG.4 merge @ `1d894ab` + v0.50 §14.11 docs 9 完整化 @ `f476147`
> **拍板来源**: per WBS-001 v0.19/v0.50 §14.11 ARG.2 (拍板 4M tokens / 0.7 周) + v0.50 §14.11 跨 session 续做 (估 7M / 5-7 session)
> **守门合规**: #1 v15 (本轮第 44 次新事件, docs 同步允许) + #1 v19 ([P] 必先 `scripts/automation/arg_bridge_test.py` 落地) + #1 v25 (cargo test -p star-arg-bridge --lib -j 4 100% pass) + #3 (5 域 Lead 跨域边强制 consults) + #5 (env 安全) + #6 (PowerShell only) + #7 (0 unsafe) + #9 (RPC 不可靠) + #10 (author=Ulysses) + #12 ([P] docs 同步) + #13 (W/T/M 三类) + #14 v3 (Mavis 永久代签维持) + #19 v19 (守门 #12 死循环饱和边界, 新事件触发都允许)

---

## 1. Objective

落地 ARG 4 新 crate 的**第 2 个 — Bridge Tier**：`crates/arg-bridge`。跨 ARG.1 (Data Tier) 和 ARG.3 (Effect Tier) 之间的同步桥。

ARG.2 完成后:
- P3-C W2 完成
- MemGraph Bolt subscription → in-process EventBus 桥接落地
- 30s 周期 flush + sled 离线降级 落地
- PyO3 调用 LangGraph state updater 协议落地 (per DD §3.2.5 + DD §4.11)

---

## 2. Scope (In-Scope per DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.1 + §4.10-4.11)

### 2.1 必做

#### A. `crates/arg-bridge/` 新 crate 4 子模块

- `Cargo.toml` (per star-cache 模板 + 6 dep: arg / tokio / serde / serde_json / chrono / tracing + 4 dev-dep: tokio / tempfile / sled / mockall)
- `src/lib.rs` 模块入口 + re-export
- `src/memgraph_listener.rs` `MemgraphEventListener` struct (per DD §4.10):
  - `event_tx: tokio::sync::broadcast::Sender<ARGEvent>`
  - `client: Arc<MemgraphClient>` (引用 ARG.1)
  - `start()`: Bolt subscription "MATCH (a)-[r]->(b) WHERE r.updated_at > $last_seen RETURN r" → 转换 `ARGEvent::from_edge_row` → broadcast
  - `subscribe()`: 返回 broadcast::Receiver
- `src/langgraph_updater.rs` `LangGraphStateUpdater` struct (per DD §4.11 + DD §5.3 PyO3):
  - `py_state_module: PyObject` (Python LangGraph state module 句柄)
  - `update_state(event: ARGEvent)`: Python::with_gil 调 `update_arg_state` 函数
  - 提供 5 Reducer 协议: `merge_arg_agents` / `merge_arg_edges` / `merge_trust_scores` / `merge_dispatch` / `add` (per DD §5.1)
- `src/period_flush.rs` `PeriodFlushWorker` struct:
  - 30s 周期的 tokio task (per BD §7.2)
  - 合并 in-process state 变更加 version 后批量写 Memgraph
  - 优雅关闭: axum::serve with_shutdown 模式
- `src/offline_queue.rs` `OfflineQueue` struct:
  - 基于 `sled` 嵌入式数据库 (per BD §7.2)
  - 容量上限 10000 (per 守门 #12 已知缺口)
  - 重连后批量 flush 协议
  - 7 天保留期 (per 守门 #7 守门)

#### B. 共享类型引用 (from crates/arg)

- `arg::models::*` (Agent / Edge / RelationshipType / ARGEvent / Decision / Output / Verdict / LLMClient 等 per DD §3.2.5)
- `arg::ops::*` (EdgeOps / AgentNodeOps)
- `arg::client::MemgraphClient` (Bolt client, G-1 stub 仍接受)

#### C. 1 脚本 (per 守门 #1 v19 [P])

- `scripts/automation/arg_bridge_test.py` v0.1:
  - 10 IT: Bolt subscription 启动 / 30s 周期 flush / sled 离线降级 / 重连 flush / PyO3 binding / 5 Reducer 协议
  - 用 `pytest` + `requests` + `websockets` 库
  - mock Memgraph 用 stub server (per 守门 #9 子代理 RPC 不可靠)

#### D. 2 文档更新 (per 守门 #12 v21 [P])

- `docs/automation-design.md` §4 追加 ARG.2 行
- `scripts/automation/registry.md` §1 + §5 索引追加 arg_bridge_test.py

#### E. 1 报告 (per AGENTS.md §3 7 段结构)

- `docs/reports/PHASE-ARG-02-IMPL-REPORT.md` v0.1

#### F. 1 commit + merge

- 1 个 commit: `feat(arg-bridge): crates/arg-bridge 4 子模块 + 10 IT + 1 Python 脚本 (ARG.2 子项, P3-C W2)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 不推 origin (per 守门 #1 R-05 反转, 等 merge 走守门)

### 2.2 不做 (Out-of-Scope)

- 不写 `crates/arg-effect/` (ARG.3 后续)
- 不写 `frontend/src/app/agent-relationships/` (ARG.5 后续)
- 不写 ArgBridgeTestCase 真实环境 (跟 ARG.1 G-1 一样, MemGraphClient 仍 stub)
- 不写 实际 LangGraph state module 集成 (PyO3 binding + mock state module 已落地, 真实集成在 ARG.3 L0↔L1 协议)

---

## 3. Acceptance Criteria (per DD §10.1.2 10 UT 模式)

### AC-1: 编译守门
- [ ] `cargo check --workspace --lib -j 4` exit 0, 0 error
- [ ] `cargo fmt --all -- --check` exit 0
- [ ] `cargo clippy --workspace --lib -j 4 -- -D warnings` exit 0
- [ ] `cargo build --release -p star-arg-bridge` exit 0

### AC-2: 测试守门
- [ ] `cargo test -p star-arg-bridge --tests -j 4` exit 0
- [ ] **10 UT 100% pass** (per DD §10.1.2 UT-25..34: 3 listener + 3 flush + 4 offline)
- [ ] 0 failed, 0 ignored

### AC-3: 集成守门
- [ ] `crates/arg-bridge/Cargo.toml` 6 dep + `arg` workspace path
- [ ] `[workspace].members` 追加 `"crates/arg-bridge"`
- [ ] 0 unsafe 块 (守门 #7)
- [ ] 0 missing_docs warn (workspace lint)

### AC-4: 同步桥协议
- [ ] `MemGraphEventListener` 接收 ARGEvent 通过 broadcast channel
- [ ] `LangGraphStateUpdater` PyO3 binding 落地 (per DD §4.11 + §5.3)
- [ ] 5 Reducer 协议实现 (per DD §5.1)
- [ ] `PeriodFlushWorker` 30s 周期可配置
- [ ] `OfflineQueue` sled 持久化 + 10000 上限 + 7 天 TTL

### AC-5: 自动化档守门 (per 守门 #1 v19)
- [ ] `scripts/automation/arg_bridge_test.py` 存在
- [ ] `docs/automation-design.md` §4 追加 1 行
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-6: 守门 v25 必填
- [ ] 单 crate 模式实证: `cargo test -p star-arg-bridge --lib -j 4` exit 0
- [ ] workspace 兼容: `cargo check --workspace --lib -j 4` 0 err
- [ ] author = Ulysses (per 守门 #10 + 9/8 15:19 第 6 次强化)

---

## 4. 已知约束 + 处理

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [P] 必先 Python 脚本 | 落 `arg_bridge_test.py` + 注册到 `registry.md` |
| 守门 #7 `unsafe_code = "forbid"` | 代码无 unsafe 块 |
| 守门 #12 docs 同步 [P] 子项 | `automation-design.md` §4 + `registry.md` 同步 |
| 守门 #10 代签 | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local'` |
| 守门 #9 RPC 不可靠 | 不调任何 RPC, mock MemGraphClient 接受 stub |
| G-1 MemGraphClient stub (ARG.1) | bridge 调用 client 接受 stub, 不报错 |
| 守门 #1 v15 docs 饱和 | 本次新事件触发, 允许 |
| 守门 #14 v3 Mavis 永久代签 | 维持, 真人到位不追溯 |

---

## 5. Deliverable

- `crates/arg-bridge/Cargo.toml` + 5 src + 10 tests ≈ 16 文件
- `Cargo.toml` 末尾追加 `"crates/arg-bridge"`
- `scripts/automation/arg_bridge_test.py` (~200 行)
- `docs/automation-design.md` §4 + `scripts/automation/registry.md` 索引
- `docs/reports/PHASE-ARG-02-IMPL-REPORT.md` v0.1
- 1 commit (author=Ulysses, 不推 origin)

---

## 6. Token 预算 (per 选项 3 分阶段批 + 用户拍板"超预算可接受")

- 软预算 4M (per WBS-001 v0.18 §14.11 ARG.2)
- v0.50 跨 session 估 7M
- 本子代理预期 5-7M
- 触发熔断 8M
- 超出记录即可 (per 2026-09-10 用户拍板)

---

## 7. 子代理执行守门 (per 守门 #9 v3 + 守门 #20 派生)

- 不调任何 RPC (守门 #9 实证)
- 不用 RPC 派子代理
- 不写跨 crate 改动 (本任务 crates/arg-bridge + scripts/automation/1 份 + docs/automation-design.md + scripts/automation/registry.md + Cargo.toml 1 行)
- 不调 cargo publish / git push
- 不写无 git 实证叙事
- 每步必跑守门, exit 0 才进下一步

---

## 8. 父会话职责

- 接收子代理 final report
- self-review 子代理产出 (对照 AC-1..AC-6)
- 跑守门 #1 v3 + v25 全套
- merge 走 `git merge --no-ff wt-arg-02-bridge` (守门 #1 v3 必先 cargo test 收敛)
- WBS-001 升版 (ARG.2 升 🟡→🟢, 累计 98/119)
- 触发 ARG.3 派新子代理

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | 立刻 task_stop, 不重试, 父会话接手 |
| cargo test 失败 | 子代理本地 fix, 跑 cargo test 实证 100% pass |
| 10 UT < 100% | 子代理补 UT 至 AC-2 必填 |
| 守门 #7 unsafe | 必移除, 改 safe 代码 |
| 守门 #12 docs 同步漏 | 必补 `automation-design.md` §4 + `registry.md` |
| Token 超 8M | task_stop, 报告 owner, 父会话决定是否续 |
| merge 冲突 | 父会话手动 rebase 或 cherry-pick |
| cargo test --workspace 跨 crate 失败 | 父会话排查, 不强推 |

---

## 10. 引用

- `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §3.1 + §4.10-4.11 + §5.1 + §5.3
- `docs/architecture/2026-09-03-arg/06-arg-02-arg-bridge.md` (v0.50 §14.11 落档, 447 行)
- `docs/reports/STAR-P3-WBS-001.md` v0.50 §14.11 (commit `f476147`)
- `crates/arg/` ARG.1 merge commit `651117e` (已存在)
- `docs/briefs/arg-01-arg-crate-skeleton.md` (13.8KB 模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
- `docs/automation-design.md` v0.1
- `scripts/automation/registry.md` v0.6+
