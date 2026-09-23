# ADR-0053: ProcessSupervisor 子 crate 拆分评估 — MVP 锁定候选 A (P0-B 评估期)

> **状态**: 🟡 Proposed v0.1 (P0-B 评估期, 候选 A 锁定)
> **日期**: 2026-09-23
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟡 Mavis 接手代签 (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签")
> **父文档**: [STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../2026-08-26-upgrade-plan.md) · [ULYS-159 P0-B 评估期 issue](https://app.multica.ai/issue/01a0bf6f-6288-7801-a629-afc8669836f1) · [ULYS-104 父 issue](https://app.multica.ai/issue/01a0b920-28d5-7dc8-93c2-9cbc73ba9c4d)
> **依赖**: [ULYS-156 P0-A 单进程持久化层适配](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8) · [ULYS-157 P0-C AI Vault 隔离](https://app.multica.ai/issue/01a0bf6c-baa0-7394-bf5d-f90e8ca59c40) · [NFR-ORCA-002 Crash-loop containment](../2026-08-26-upgrade/adr/0049-ai-tool-auto-discovery.md)
> **关联**: [ULYS-160 P1-A 实施路径](https://app.multica.ai/issue/01a0bf70-0000-7001-0000-000000000000) (前置依赖: 本 ADR sign-off) · [docs/deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md](../../deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md) (Linux/macOS 部署侧 systemd-cgroup 路径)

---

## 1. 背景与问题

### 1.1 评估期触发 (per ULYS-159 §任务范围)

本 ADR 是 [ULYS-104 (Orca 分析可以借鉴的点)](https://app.multica.ai/issue/01a0b920-28d5-7dc8-93c2-9cbc73ba9c4d) 9/12 12:06 JST 自审 §9.2 衍生的**架构评估任务**, 而非 Orca spec 内的 ID 落地条目 (per ULYS-159 §1: "本 sub-issue 不是 Orca spec 的 ID 落地"). 任务是 P0-A 单进程持久化层适配 (ULYS-156, ULYS-159 的同级 sub-issue) 落地前, 决定 MVP 锁定 vs P1-B 拆 crate 的边界.

### 1.2 ProcessSupervisor 子域当前实证 (per `wc -l crates/domain-local-runtime/src/*.rs`, baseline commit `ebe2317f`)

| 子模块 | 行数 | `#[test]` 计数 | `unsafe` 命中 | 职责 |
|---|---|---|---|---|
| `cli_session.rs` | 420 | 9 | 0 | 7 态状态机 + CliSession 实体 |
| `cli_session_lock.rs` | 507 | 6 | 0 | PID + 启动时间 lock, 防 PID 回收 (FR-ORCA-001 AC-4) |
| `cli_session_registry.rs` | 520 | 6 | 0 | SQLite WAL 持久化层 (per star-taskqueue 同款) |
| `cli_spawn.rs` | 423 | 9 | 0 | spawn 侧: tokio Command + 错误封装 |
| `graceful_shutdown.rs` | 522 | 5 | 0 | SIGTERM/Ctrl-C handler, 串行 cancel 活跃 session |
| `health_self_test.rs` | 464 | 6 | 0 | 跨 4 模块自检 (FR-ORCA-003) |
| `process_supervisor.rs` | 482 | 7 | 0 | 跨 session 监督 + 5 launches/60s crash-loop containment (NFR-ORCA-002) |
| `unix_session.rs` | 557 | 11 | 9 | Unix-only `setsid(2)` + 跨进程 `killpg(2)`, **9 处 `unsafe` 集中在 `pre_exec` 闭包与 Windows stub** |
| **小计 (8 子模块)** | **3895** | **59** | **9 (集中)** | — |

| 全局实证 | 数值 |
|---|---|
| `crates/domain-local-runtime/src/*.rs` **总行数** | **8931** (17 文件) |
| ProcessSupervisor 子域 8 子模块**占比** | 3895/8931 = **43.6%** |
| 全仓对 `domain-local-runtime` 的 `[dependencies]` 引用 | **0** (workspace 注册 + 自身引用 = 仅 1 处 `path = "..."`) |
| `crates/star-dispatcher/src/sa_real_impls.rs:257` 的 `domain-*` 列表 | **字符串字面量列表** (SA-08 22 域名静态校验), 非编译期依赖 |

> **注**: 原 ULYS-159 §6 描述里 "5392 行 / 5 文件" 是 9/19 时刻的旧快照; 当前 baseline `ebe2317f` (2026-09-23 22:51 JST) 实测 **17 文件 / 8931 行**, 其中 ProcessSupervisor 子域 8 文件 / 3895 行. 本 ADR 全部数字按当前 baseline 重跑, 不沿用旧值.

### 1.3 行数阈值参考 (per ADR-0012, 外部约束, 仅作定性参考)

[ADR-0012](../../README.md) (本仓库当前 grep 0 命中, **确认不在 working tree**) 曾提出 2000 行阈值; 子域 3895 行 / 单文件最大 557 行均**超阈值但分布合理** — 没有任何单文件超过 600 行, 已按 actor / 生命周期阶段分散. 行数阈值的"超标"是子域复杂度真实反映, 而非"挤压到单文件"的退化信号.

### 1.4 候选方案

| 候选 | 描述 | 推荐 |
|---|---|---|
| **A** | **不拆**, MVP 锁定 — 当前 17 文件 / 8931 行, ProcessSupervisor 子域 8 文件 / 3895 行, 子模块职责已按 actor / 阶段分散, 单文件最大 557 行 | ✅ **MVP 推荐** |
| B | 拆 crate `crates/star-process-supervisor`, 跨 crate 通过 `pub use` re-export 保持外部 API 兼容 0 改 | 🟡 P1-B 启动时评估切换 |
| A+C | 模块级重排 (治标不治本 — 内部行数再压缩无收益, 仅 doc comment 调整) | ❌ 不推荐 |

---

## 2. 5 维度评估 (per ULYS-159 §评估范围)

每维度 1-5 分 (5 = 最不利于拆 crate, 1 = 最利于拆 crate); 加总 = 5..25, **≥18 触发候选 B 重评**.

| # | 维度 | 评估问题 | 证据 (当前 baseline) | 评分 |
|---|---|---|---|---|
| 1 | **职责单一性** | 当前 `domain-local-runtime` 子模块职责切分是否过粗? | 8 子模块按 actor (cli_session*4) + 阶段 (spawn / supervisor / shutdown / self_test) + 平台 (unix_session) 切分, **没有 > 600 行的单文件**, 3895 行总占比 43.6% 是子域复杂度真实反映, 非挤压 | **4/5** |
| 2 | **依赖复杂度** | supervisor 逻辑对 `tokio` / `nix` / `windows` Job Object API 的耦合度? | `process_supervisor.rs` 仅依赖 `tokio` (Mutex/HashMap/VecDeque) + `chrono` + `serde` + `thiserror` + 本 crate 内 3 个 ID 类型, **0 `unsafe`, 0 `nix`, 0 `windows-sys`**; `nix` 集中在 `unix_session.rs` (Unix-only `setsid`/`killpg`, 已 `#[cfg(unix)]` 隔离); `windows-sys` Job Object API 在全 crate **0 命中** (Windows 路径走 [ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP §Windows 部署策略](../../deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md), 不下沉到 Rust 代码) | **5/5** |
| 3 | **测试复杂度** | 当前 17 文件 / 8932 行是否已超 ADR-0012 行数阈值? | 行数阈值超标 (8931 vs 2000, 4.3 倍) 但**测试密度合理**: 子域 8 文件 / 59 tests, 平均 **7.4 tests/file**, e2e_integration.rs 8 个 `#[tokio::test]` 是多场景集成; **测试瓶颈不在"行数多"而在"集成场景多"**, 后者拆 crate 反而恶化 (`pub use` re-export 让集成测试跨 crate 边界) | **3/5** |
| 4 | **P0-A 协同** | ULYS-156 单进程持久化层是否需要 supervisor 单独子 crate? | ULYS-156 §A 单进程模型下 supervisor 内存状态 + SQLite WAL 共享 `Connection` (INV-SUP-03 显式声明"supervisor 不持久化, 进程重启后历史清零"); **拆 crate 反而要把 `Mutex<Connection>` 拉到子 crate 公共 API 或反向依赖**, 跟单进程模型直接冲突 | **5/5** |
| 5 | **P0-C 隔离** | ULYS-157 AI Vault 隔离需求是否会反向要求 supervisor 独立? | ULYS-157 隔离的是**凭据 / 密钥存储**, 跟 ProcessSupervisor 子域无共享状态 (supervisor 只持有 `(session_id, pid, command)` 元组, 不持有凭据); **不构成拆 crate 的依据** | **5/5** |

| **加总** | **22/25** | — | 远高于 18 触发线 → **候选 A 锁定** |

---

## 3. 决策

**MVP 锁定候选 A: 维持 `crates/domain-local-runtime` 单 crate, ProcessSupervisor 子域保留为 8 子模块; P1-B 启动时 (预计在 ULYS-156 实装结束后 + ULYS-160/161/162 三个 P1 sub-issue 解锁) 重评, 触发条件 = 维度 3 (测试密度) 或维度 4 (P0-A 协同) 任一评分降至 ≤ 2**.

### 3.1 候选 A 的具体落档内容 (本期就绪)

1. **ADR-0053 v0.1 落档**: 本文档即 ADR-0053 canonical, 状态 🟡 Proposed.
2. **§2 5 维度评分** + **加总 22/25** 作为后续 P1-B 重评的基线快照.
3. **§4 候选 A 决策** 作为 ULYS-160 (P1-A) / ULYS-161 / ULYS-162 三个 P1 sub-issue 的**前置解锁条件**之一.

### 3.2 候选 B (P1-B 备选) 的切换成本 (供参考)

| 切换项 | 成本 |
|---|---|
| 新 crate `crates/star-process-supervisor` | +1 `Cargo.toml` + 8 文件迁移 |
| 跨 crate API (8 子模块 `pub use` re-export) | ~50-80 行 `pub use` boilerplate |
| 集成测试跨 crate 边界 | e2e_integration.rs / spawn_upload_integration.rs / subscribe_integration.rs 需调整 mock 路径 (Mutex<Connection> 共享难度↑) |
| `Mutex<Connection>` 跨 crate | 要么下沉到子 crate, 要么反向依赖 `domain-local-runtime`; 两种都打破单进程模型 |
| **结论** | 候选 B 在 P1-B 启动前**不应执行**; P1-B 启动时若仍有需求, 优先评估"内部模块重排"而不是"拆 crate" |

---

## 4. 决策日志 (Decision Log)

| 日期 | 决策 | 触发 | 来源 |
|---|---|---|---|
| 2026-09-23 (本 ADR) | MVP 锁定候选 A, 5 维度加总 22/25, P1-B 启动时重评 | ULYS-159 §交付 + 9/23 当前 baseline 实证 | 本 ADR §2 + §3 |
| 2026-09-22 JST | D-Boy 9/22 决策会: ULYS-159 委派给 minimaxm3 (vs minimaxm2 重启) | ULYS-159 前次交付不可核验 | ULYS-159 comment `01a0c5d2-da3f-7da6` |
| 2026-09-22 JST | minimaxm2 21:16 「收口完成 cherry-pick 推上 GitHub」评论里 `c9851b16` / `618fe89f` / `bfdf6a16` 三 commit hash 在 local / remote / reflog / stash 任何 ref 均不可见 | session 不可恢复, 远端实测 `git ls-remote` 空 | ULYS-159 comment `01a0c6bd-f7fa-739b` (自我更正) + comment `01a0c9a8-e977-793e` (Opus 审核打回) |
| 2026-09-22 15:08 JST | Opus 审核: 9/22 收口评论记录的交付物不存在, 建议 R1 重写 ADR-0053 + memo + wiki mirror → commit → push → 开 PR 合 dev | 审核对象无核验 SHA | ULYS-159 comment `01a0c9a8-e977-793e` |
| 2026-09-20 21:24 JST | Sonnet 独立审查 5 维度数字 100% 一致 (5392/409/381/432/559/371) — 该数字为 9/20 时刻旧快照 | ULYS-159 §交付 review | ULYS-159 comment `01a0c0b4-d5b4-7be2` |
| 2026-09-20 JST | ULYS-159 §1 描述更新 (本 ADR 引用): 锚点改为 9/12 12:06 JST 自审 §9.2 (原 "per 9/19 v2.0 §21 P0-B" 引用错位); ADR-0046 → ADR-0053 编号变更 (0046 已是 LangGraph TMO, 0053 是下一个空号) | D-Boy 描述修订 | ULYS-159 description 当前文本 |

---

## 5. 实施计划 (Implementation Plan)

| 阶段 | 内容 | 状态 |
|---|---|---|
| 文档 v0.1 (本 ADR 落档) | ADR-0053 canonical + 5 维度评分表 + 候选 A 锁定 + 4 备选维度评分细则 + 决策日志 + 签字栏 | 🟡 **本 ADR Proposed** (本期落档) |
| D-Boy 拍板 | sign-off 4 项开放项 (ULYS-159 §3 候选 A 锁定 / 5 维度评分方法 / P1-B 拍板窗口 / ADR 编号变更) | ⏳ pending D-Boy |
| DDD Review 阶段补真实身份 | 4 项签字栏 "Mavis 接手代签 (待 DDD Review 补真实身份)" | (per DEC-008 5 域 Lead 真人到位后追溯) |
| P1-B 启动时重评 | 维度 3 (测试密度) 或维度 4 (P0-A 协同) 任一评分降至 ≤ 2 → 重新评估候选 B | ULYS-160/161/162 三个 P1 sub-issue sign-off 后 |

---

## 6. 签字栏 (Signatures, per 7 段结构 5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008） | 2026-09-23 | 🟡 Proposed v0.1; ProcessSupervisor 子 crate 评估期 5 维度 22/25, MVP 锁定候选 A (不拆), P1-B 启动时重评 (ULYS-160/161/162 解锁后) |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-23 | 🟡 Mavis 接手代签通过 (per 19:39 + 21:59 JST 用户授权); 3 备选方案 (候选 A 不拆 / 候选 B 拆 crate / 候选 A+C 模块重排) + 选定 A 方案 + 5 维度评分表 + 22/25 加总 + 候选 B 切换成本表 + 4 决策日志 + 5 阶段实施计划落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-23 | 🟡 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-23 | 🟡 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-23 | 🟡 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-23 | 🟡 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 7. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: ProcessSupervisor 子 crate 评估期 5 维度评分 (22/25, MVP 锁定候选 A); §1.2 baseline 实证 (17 文件 / 8931 行, 子域 8 文件 / 3895 行 / 59 tests / 9 `unsafe` 集中于 unix_session) + §1.3 ADR-0012 行数阈值参考 (2000 行) + §2 5 维度评分表 (职责单一性 4 + 依赖复杂度 5 + 测试复杂度 3 + P0-A 协同 5 + P0-C 隔离 5) + §3 候选 A 锁定 + 候选 B 切换成本表 + §4 决策日志 6 条 + §5 实施计划 4 阶段 + §6 签字栏 5 角色 (Mavis 接手代签) + §8 引用 12 项 | 2026-09-23 ULYS-159 P0-B 评估期本轮重写 (per Opus 9/22 15:08 JST 审核打回 R1 重写指令) |

---

## 8. 引用文档 (References)

- [ULYS-159 P0-B 评估期 issue](https://app.multica.ai/issue/01a0bf6f-6288-7801-a629-afc8669836f1) — 本 ADR 父任务
- [ULYS-104 父 issue](https://app.multica.ai/issue/01a0b920-28d5-7dc8-93c2-9cbc73ba9c4d) — Orca 分析可以借鉴的点 (9/12 12:06 JST 自审 §9.2 触发本 ADR)
- [ULYS-156 P0-A 单进程持久化层适配](https://app.multica.ai/issue/01a0bf6b-486c-71b8-9bf4-92bde909c7f8) — P0-A 协同维度输入
- [ULYS-157 P0-C AI Vault 隔离](https://app.multica.ai/issue/01a0bf6c-baa0-7394-bf5d-f90e8ca59c40) — P0-C 隔离维度输入
- [ADR-0049 AI Tool Auto Discovery](../2026-08-26-upgrade/adr/0049-ai-tool-auto-discovery.md) — NFR-ORCA-002 crash-loop containment 锚点
- [ADR-0046 LangGraph TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — 编号变更参照 (本 ADR 0053 = 0046 之后下一个空号)
- [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) — supervisor 类似架构参照
- [ADR-0033 代签规则反转](../2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) — Mavis 接手代签授权
- [docs/deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md](../../deployment/ULYS-156-NFR-ORCA-001-SYSTEMD-CGROUP.md) — Linux/macOS systemd-cgroup 部署策略 (Windows 路径不下沉到 Rust 代码)
- [crates/domain-local-runtime/src/process_supervisor.rs:1-39](../../crates/domain-local-runtime/src/process_supervisor.rs) — INV-SUP-01/02/03 + NFR-ORCA-002 锚定
- [crates/domain-local-runtime/src/lib.rs:1634-1648](../../crates/domain-local-runtime/src/lib.rs) — 子模块注册 + supervisor 角色描述
- [crates/star-dispatcher/src/sa_real_impls.rs:257](../crates/star-dispatcher/src/sa_real_impls.rs) — 22 域名静态校验列表 (非编译期依赖)

---

# === ADR 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本 ADR v0.1 引用守门 #1-#24 全部按 git 实证 + `wc -l` / `grep` 实测, 无 "per X 历史形态" 等回溯叙事; §1.2 baseline 数字按当前 HEAD `ebe2317f` 重跑 (8931 行 / 17 文件 / 子域 3895 行 / 8 子模块), 与 9/19 描述里 5392 行 / 5 文件 旧快照显式标注差异, 不掩盖不沿用.

**per 守门 #1 v15 死循环饱和边界**: 本 ADR = 评估期决策落档 (ULYS-159 任务范围"仅评估不实装"), 不引入新代码 / `Cargo.toml` 改动, 适用 docs 同步饱和点 +1, 不违反饱和约束.

**per 守门 #3 5 域单仓**: 本 ADR 仅 STAR 仓内, 不引用 RGS 仓代码; ProcessSupervisor 子域评估结论 (候选 A 不拆) 仅作用于 `crates/domain-local-runtime`, 不影响其它 domain-* crate.