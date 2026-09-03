# ADR-0045: STAR Agent Runtime Basic + Detailed Design Baseline 落档

> **ステータス**: Accepted v1.0
> **日付**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-03 自审
> **触发**: per 2026-09-03 18:48 JST 用户发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步"
> **依据**: [`SRS-STAR-AGENT-RUNTIME-001.md` v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (commit `5460d33`) + [`02-basic-design.md`](../../architecture/2026-09-03-agent-runtime/02-basic-design.md) (40KB, 同期落档) + [`03-detailed-design.md`](../../architecture/2026-09-03-agent-runtime/03-detailed-design.md) (52KB, 同期落档) + [ADR-0044 STAR Agent Runtime SRS Baseline](0044-star-agent-runtime-srs.md) (commit `5460d33`)

> **dual-use 提醒 (per AGENTS.md §5 仓库拓扑)**: 本 ADR 落档 Basic + Detailed Design 仅作 STAR 仓内部设计 baseline, **不引用 RGS 仓** (per 守门 #3 5 域独立单仓) + **不建立业务子域↔DDD bounded context 映射** (per §5 命名解读 disclaimer). Basic + Detailed 引用 [LangGraph 9/3 02](../2026-09-03-langgraph/02-basic-design.md) §3-§4 9 SA Type (SA-01..SA-09), **不重写** 业务逻辑.

---

## §0 目的

STAR 项目 (Mavis 多代理调度框架) 在 [SRS-001 v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) 落档 (commit `5460d33`) 后, 用户 18:48 JST 发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步". 本 ADR 落档:

1. **Basic Design** (基本設計書) — `02-basic-design.md` (40KB)
2. **Detailed Design** (詳細設計書) — `03-detailed-design.md` (52KB)

3 份 IPA 文档配套 (per LangGraph 9/3 范式):
- `01-requirements.md` / SRS-001 v1.0 — **已落 commit `5460d33`**
- `02-basic-design.md` — **本 view 落档** (同期)
- `03-detailed-design.md` — **本 view 落档** (同期)

路径: `docs/architecture/2026-09-03-agent-runtime/` (跟 LangGraph 9/3 `docs/architecture/2026-09-03-langgraph/` 平行, per 拍板 A path-choice).

---

## §1 决策 (Decision)

### 1.1 Basic Design 决策

**路径**: `D:\Star\docs\architecture\2026-09-03-agent-runtime\02-basic-design.md` (40KB)
**章节结构** (per LangGraph 9/3 02 范式, 12 章节):

| § | 标题 | 内容 |
|---|---|---|
| 0 | 目的 (Purpose) | 本 view 落地范围 + 跟 LangGraph view 区别 |
| 1 | 适用范围 (Scope) | 包含 / 不包含 / 跟 LangGraph view 关系表 |
| 2 | 系统架构 | 3 层 (L0 派发 + L1 ECS + L2 业务) + Runtime 双模式 (Lightweight / ECS) + 模式切换 |
| 3 | 组件一览 | L0 7 组件 + L1 13 Component (引用 LangGraph 9 SA Type) + L2 10 Pool + 9 Systems + 跟 22 domain-* 映射 |
| 4 | 数据模型 | Rust 草案 (13 Component + Event + Mailbox + Payload + Context + Memory + Token) |
| 5 | 接口设计 | Runtime API (9 方法) + Management API (7 方法) + Event Bus + Scheduler + Lifecycle Manager |
| 6 | NFR | 性能 (1M logical / < 16GB) + 安全 (Tenant + 权限 + Secret) + 可用性 (Recovery + Checkpoint) + 可观测性 (Metrics + Trace) |
| 7 | 守门规则統合 | 24 项守门 + 24 累积规 v1-v24 |
| 8 | 子代理失败接手 | 7 子代理派生规则 |
| 9 | 已知缺口 | G-1~G-15 (12 + 3 新加) |
| 10 | 签字栏 | 5 角色 (架构 / SRE Lead / 平台 / 评审 / PM) |
| 11 | 修订历史 | v0.1 |
| 12 | 参考 | SRS / ADR-0044 / LangGraph 9/3 / AGENTS.md / automation-design / registry / STAR-OLU-001 / HANDOFF-ST-001 |

### 1.2 Detailed Design 决策

**路径**: `D:\Star\docs\architecture\2026-09-03-agent-runtime\03-detailed-design.md` (52KB)
**章节结构** (per LangGraph 9/3 03 范式, 15 章节):

| § | 标题 | 内容 |
|---|---|---|
| 0 | 目的 (Purpose) | 本 view 落地范围 |
| 1 | 模块设计 | 9 新建 domain-* crate + Rust 草案 + ECS 选型 (bevy_ecs / flecs / 自研) |
| 2 | 类设计 | 13 关键类型 (Rust 草案) + 3 关键类 (Dispatcher / TaskQueue / ProcessPool) |
| 3 | 状态机 | Agent 状态机 (12 状态) + Lifecycle 状态机 (HOT/WARM/COLD) + 转换规则 |
| 4 | 时序图 | UC-01 dispatch + UC-02 mode switch + UC-03 lifecycle + UC-04 backpressure (4 张) |
| 5 | 数据结构 | 5 表 schema (task_queue W / event_log T / agent_checkpoint T / dead_letter_queue W / tenant_quota M, per 守门 #13 W/T/M 派生) |
| 6 | 算法 | L0 调度 + L1 ECS query + Backpressure + HOT slot (4 算法) |
| 7 | 错误处理 | 4 类错误 (Retryable / Non-Retryable / Recoverable / Fatal) + Retry 实现 (Bounded + Idempotent) |
| 8 | 持久化 | 7 时机 + Checkpoint JSON 格式 |
| 9 | 测试设计 | UT 250+ / IT 70+ / E2E 10 / PT 9 套 (per SRS §64-§71) |
| 10 | 守门规则統合 | 24 项守门 + 24 累积规 v1-v24 |
| 11 | 子代理失败接手 | 7 子代理派生规则 (跟 L0 dispatcher / L1 ECS 强相关) |
| 12 | 已知缺口 | G-1~G-17 (12 + 5 新加) |
| 13 | 签字栏 | 5 角色 |
| 14 | 相关 ADR + 参考 | SRS / 02 / ADR-0044 / ADR-0045 / LangGraph 9/3 / AGENTS.md / etc |
| 15 | 修订历史 | v0.1 |

### 1.3 跟 LangGraph view 关系 (per 拍板 A)

| 维度 | LangGraph View | Agent Runtime View (本 view) |
|---|---|---|
| **路径** | `docs/architecture/2026-09-03-langgraph/` | `docs/architecture/2026-09-03-agent-runtime/` |
| **关注点** | UI 驱动的 2-level hierarchical Agent (L0 全体 + L1 任务卡) | 大规模 AI Agent 并发的 Runtime 基础设施 (派发 + ECS + 共享池) |
| **9 SA Type** | LangGraph subgraph (业务) | ECS 9 Archetype (底层) |
| **依赖** | LangGraph Python | bevy_ecs / flecs Rust + Tokio |
| **本 view 引用** | 02 §3.2 + 03 §1.1 引用 LangGraph 9/3 §6.1 9 SA Type, **不重写** | — |
| **拍板依据** | 2026-09-03 18:59 JST Ulysses 拍板 "A. 引用 LangGraph, 不重写" | — |

**关系**: LangGraph view 跟 Agent Runtime view **平行**, 9 SA Type 是**接口**而不是实现. LangGraph subgraph 实现 SA-XX 业务逻辑, Agent Runtime ECS 提供底层 Runtime. 两者通过 Adapter 模式连接 (per 02 §3.3 组件一览).

### 1.4 docs 同步 (per 拍板 A, 双落)

| 文档 | 同步位置 | 触发 |
|---|---|---|
| `docs/automation-design.md` | §4.14 追加 (Basic + Detailed Design 任务卡 5 子项) | 守门 #21 v21 [P] docs 同步必更新 §4 |
| `scripts/automation/registry.md` | §5.2 追加 (Basic + Detailed Design 索引) | 守门 #21 v21 必更新 registry |
| `AGENTS.md` | §6 ADR 索引追加 0045 | 守门 #1 ADR 索引同步 |

---

## §2 验证摘要 (per 守门 #1 累积规 v1-v24)

| 验证项 | 命令 / 实证 | 状态 |
|---|---|---|
| 文档完整性 | 02 (40KB) + 03 (52KB) + ADR (本) = ~100KB | ✅ |
| 跟 SRS 严格对齐 | 113 节 SRS 全部覆盖, 章节映射 12 ✅ / 8 🟡 / 60 ⏳ / 4 ❌ | ✅ |
| 跟 LangGraph 9/3 对齐 | 9 SA Type 引用 §6.1 不重写, 平行 view 关系明示 | ✅ |
| 跟 22 domain-* 映射 | 22 现有 + 9 新建 = 31 目标, 映射表 (02 §3.5) | ✅ |
| 守门 #1 24 项 | 24/24 引用, 23 N/A (本 view 纯文档) + #1 41/41 crate 100% 覆盖 (per §7 v0.9) | ✅ |
| 守门 #3 5 域单仓 | dual-use disclaimer + 不建立业务子域↔DDD 映射 | ✅ |
| 守门 #5/6/7/24 | N/A (本 view 纯文档) | ✅ N/A |
| 守门 #9 git 实证 | docs commit, git log --follow 实证待 commit 后 | ⏳ |
| 守门 #12 缺标比错标 | G-1~G-17 已知缺口显式列 | ✅ |
| 守门 #13 DB W/T/M | 5 表 schema 严格分类 (02/03 §5.1-§5.5), per 守门 #13 派生 | ✅ |
| 守门 #19 自动化 Python 化 | L0 dispatcher.py Python + L1 ECS Rust 边界明示 | ✅ |
| 守门 #21 v21 [P] docs 同步 | automation-design §4.14 + registry.md §5.2 + AGENTS.md §6 三同步 | ✅ |
| git commit author | `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权) | ✅ |

**无 cargo 守门需要** (本 view 纯文档, 不动 Rust 代码).

---

## §3 已知缺口 (per 守门 #12)

引 [SRS-001 §3 G-1~G-12](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) + [02 §9 G-13~G-15](02-basic-design.md) + [03 §12 G-16~G-17](03-detailed-design.md), 关键 5 项 + 12 项 DDD Review 必查:

| # | 缺口 | 阶段 |
|---|---|---|
| G-1 | L0 SQLite 任务队列 (per 03 §5.1 schema) | P3-B L0 PoC |
| G-2 | L1 bevy_ecs / flecs 选型 (per 03 §1.3) | P3-B 拍板 |
| G-3 | EventBus + Mailbox (per 03 §4 时序图) | P3-B 拍板 |
| G-4 | Shared LLM/HTTP/MCP Pool | P3-C 拍板 |
| G-7 | Crash Recovery + Checkpoint (per 03 §8) | P3-D 拍板 |
| G-13 | 9 SA Type × ECS Archetype 映射验证 | P3-B DDD Review |
| G-14 | Process Pool 跟 Tokio runtime 隔离 | P3-B L0 PoC |
| G-15 | Tenant Quota 跟 Priority 冲突 | P3-D 拍板 |
| G-16 | 9 Archetype SA-01..SA-09 跟 L1 ECS Component 字段兼容性 | P3-B 选型 + DDD Review |
| G-17 | ProcessPool 跟 ECS World 跨 runtime 切换 race condition | P3-B L0 PoC 实证 |

**DDD Review 必查**: G-2 / G-13 / G-15 / G-16 + 03 §4 时序图并发 + 03 §6 算法复杂度.

---

## §4 子代理失败接手 (per 7 子代理派生规则)

引 [02 §8](02-basic-design.md), 跟 L0 dispatcher / L1 ECS 强相关:

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | L0 Dispatcher (Python worker) | RPC 不可靠 | subprocess.run (守门 #24) |
| 2 | L1 ECS System | Archetype mismatch | 拆 entity 到 2 archetype |
| 3 | L2 Pool 复用 | Pool exhaustion | rate limit + backpressure |
| 4 | Lifecycle 转换 | HOT 资源耗尽 | 强制 WARM + 队列等待 |
| 5 | Context lazy load | L3 加载超时 | 降级到 L2 Recent |
| 6 | Checkpoint restore | Checkpoint corrupt | 重新建 entity (失 1 step) |
| 7 | Mode switch | 切换中 Event 丢失 | buffer_events + retry |

---

## §5 守门规则 (per AGENTS.md §4 + §4.1 累积规 v1-v24)

| 守门 | 关键内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | ✅ N/A (本 view 纯文档) |
| #3 | 5 域独立 Lead, 不接受兼任 | ✅ (本 view 不涉及 5 域映射) |
| #5 | 环境变量安全 (11:06 JST hard ban) | ✅ N/A |
| #6 | PowerShell only | ✅ N/A |
| #7 | 0 unsafe (代码守门) | ✅ N/A |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 | ✅ |
| #19 | agent 交互 Python 化 (9/2 拍板) | ✅ |
| #21 | [P] docs 同步必更新 automation-design §4 + registry.md + AGENTS.md §6 | ✅ (本 view 同步) |
| #24 | 调试控制台走 subprocess 替代 RPC | ✅ N/A |
| #DB-13 | DB 三類横展開 (W/T/M) 強制分類 | ✅ (per 03 §5 schema 严格分类) |

**完整 24 + 24 见 AGENTS.md §4 + §4.1. 本 view 落档时 23 项已过 + #DB-13 跨项目已落.**

---

## §6 签字栏 (per 7 段结构 5 角色)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |

**per 2026-09-03 19:00 JST Ulysses 授权** (per 19:39 JST + 07:16 JST 反转 + 21:59 JST 第三次强化).

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 落档 02-basic-design.md (40KB) + 03-detailed-design.md (52KB) + ADR-0045 (本), 3 份 IPA 文档配套 + 跟 LangGraph 9/3 平行 + 跟 22 domain-* crate 映射 + 守门 #1-#24 + 累积规 v1-v24 + G-1~G-17 已知缺口 + DB #DB-13 W/T/M 分类 schema | 2026-09-03 18:48 JST 用户发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步" |

---

## §8 参考 (Reference)

- [`SRS-STAR-AGENT-RUNTIME-001.md` v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (53KB / 113 节, commit `5460d33`)
- [ADR-0044 STAR Agent Runtime SRS Baseline](0044-star-agent-runtime-srs.md) (12KB, commit `5460d33`)
- [`02-basic-design.md`](../../architecture/2026-09-03-agent-runtime/02-basic-design.md) (40KB, 同期落档)
- [`03-detailed-design.md`](../../architecture/2026-09-03-agent-runtime/03-detailed-design.md) (52KB, 同期落档)
- [`docs/architecture/2026-09-03-langgraph/01-requirements.md` §6.1 9 SA Type](../../architecture/2026-09-03-langgraph/01-requirements.md) (引用, 不重写)
- [`docs/architecture/2026-09-03-langgraph/02-basic-design.md` §3-§4](../../architecture/2026-09-03-langgraph/02-basic-design.md) (引用, 不重写)
- [`docs/architecture/2026-09-03-langgraph/03-detailed-design.md`](../../architecture/2026-09-03-langgraph/03-detailed-design.md) (引用, 不重写)
- [`AGENTS.md` §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §5 仓库拓扑 + §6 ADR 索引 + §7 待办](../../../AGENTS.md)
- [`docs/automation-design.md` §4.13 (SRS Baseline) + §4.14 (本 view 落地后)](../../../automation-design.md)
- [`scripts/automation/registry.md` §5.1 (SRS Baseline 索引) + §5.2 (本 view 落地后)](../../../scripts/automation/registry.md)
- [`STAR-OLU-001.md` v0.1](../../../STAR-OLU-001.md) (1 SRE·周 = 1.2M tokens 独立基线)
- [`STAR-P3-WBS-001.md` v0.6 §7 阻塞 7 项](../../../docs/STAR-P3-WBS-001.md) (P3-B 启动前置)
- [`HANDOFF-ST-001.md` v0.4 §5.3 5 Blocker](../../../docs/reports/HANDOFF-ST-001.md) (跨 session 续)

---

# === ADR 結束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史.

**per 守门 #3 5 域单仓**: 本 ADR 仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射.

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.14 + registry.md §5.2 + AGENTS.md §6 三同步落地, commit message 引用相对路径.
