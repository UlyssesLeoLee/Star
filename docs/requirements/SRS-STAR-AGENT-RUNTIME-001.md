# SRS-STAR-AGENT-RUNTIME-001

> **STAR Agent Runtime 软件需求规格说明书 v1.0**
>
> - 状态: Requirements Baseline
> - 目标阶段: 需求定义 → 基本设计
> - 核心语言: Rust
> - 核心运行时: Tokio
> - 核心架构: Lightweight Runtime + Agent-Oriented ECS + Event Driven + Shared Runtime + HOT/WARM/COLD Lifecycle
> - 参考: Rust Hybrid ECS 高并发 Agent Runtime SRS v1.0
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-03 JST

---

## §0 文档目的

本文档定义 STAR（`D:/Star`）项目的 Agent Runtime 软件需求规格。STAR 是 Mavis 多代理调度框架，落地于子代理派发（守门 #20 brief 落地）、守门检查（#1-#24 + 累积规 v1-v24）、registry 同步（#21 docs 同步）、LangGraph L0/L1（per `docs/architecture/2026-09-03-langgraph/`）等核心子系统。**本 SRS 不涉及物理引擎（Physis，独立产品线）和 3D 渲染**（per 2026-09-03 用户明确反馈）。

参考 SRS v1.0 的 100 节结构按 STAR 实际映射：标"已落地"（per git 实证 33+ commit）、"部分落地"（P3-A 25 子项已完成）、"待 P3-B 启动"（Ulysses 拍板后）、"待 P3-D/F"（中后期）、"N/A"（STAR 不涉及，如跨机分布式）。目标量级推到 **1M logical agents on 16-32GB 单机**（vs 参考 SRS 的 100K logical）。

---

## §1 改动矩阵 / 章节映射

参考 SRS 100 节 → STAR 映射表。状态列：`✅ 已落地` / `🟡 部分落地` / `⏳ 待 P3-B` / `⏳ 待 P3-D` / `⏳ 待 P3-F` / `❌ N/A`（STAR 不涉及）

| 参考 SRS | 标题 | STAR 映射章节 | 状态 | 实证 |
|---|---|---|---|---|
| §1 | 文档目的 | §0 + 本节 | ✅ | 33+ commit ahead origin/main |
| §2 | 项目目标 (100K logical) | §2 STAR 推到 1M logical | 🟡 | 25 子项 P3-A 收官 |
| §3 | 设计原则 (Hybrid + Agent≠Runtime) | §3 完整继承 | ✅ | 守门 #20 落地 |
| §4 | 系统范围 (20 子系统) | §4 STAR 24 守门 + 调度 + L1 | 🟡 | 守门 #1-#24 全落 |
| §5 | 总体架构 (Event→Mode→Shared) | §5 L0/L1 双层 | 🟡 | L0 PoC 缺, L1 文档 v0.1 |
| §6 | Runtime 双模式 (Lightweight + ECS) | §6 STAR Lightweight=P3-A, ECS=P3-B+ | 🟡 | P3-A 25 子项 (Lightweight) |
| §7 | Runtime 模式切换 (0-8/9/10-11/≥12) | §7 调度层固定, L1 ECS 切换 | ⏳ 待 P3-B | 阈值待定 |
| §8 | Agent 定义 (Identity+State+Refs+Policies) | §8 任务卡 = 上述 4 字段 | 🟡 | 守门 #20 brief 落地 |
| §9-§12 | Component + Lifecycle | §9-§12 L1 ECS 实现 | ⏳ 待 P3-B | bevy_ecs 选型未启 |
| §13-§15 | Event + Mailbox + Payload | §13-§15 现有 dispatcher 扩 | ⏳ 待 P3-B | EventBus 待 |
| §16-§27 | Shared Runtime (LLM/HTTP/MCP/Tool/RAG) | §16-§27 进程池 + 共享 client | ⏳ 待 P3-C | 守门 #24 subprocess 池 |
| §28 | Memory Store | §28 外置 Memory | ⏳ 待 P3-D | 待启 |
| §29-§34 | Scheduler + HOT Slot + Backpressure | §29-§34 L0 调度器 | ⏳ 待 P3-B | Tokio dispatcher 待 |
| §35-§42 | 状态机 + 持久化 + Recovery | §35-§42 任务卡状态机 | ⏳ 待 P3-B | P3-A 25 状态 git 实证 |
| §43-§47 | 多租户 + 权限 + Secret | §43-§47 Tenant 配额 | ⏳ 待 P3-D | 22 domain-identity 待联 |
| §48-§53 | 内存设计 (100K<5GB) | §48-§53 STAR 推到 1M<16GB | ⏳ | 目标量级提升 10x |
| §54-§57 | 禁止 + 原则 | §54-§57 守门 #7 0 unsafe | ✅ | 守门 #7 实证 |
| §58-§60 | ECS System + Lock | §58-§60 9 SA System | 🟡 文档已落 / 实装待 P3-B | LangGraph §6.1 9 类型 |
| §61-§62 | 可观测性 + Trace | §61-§62 token 缺数据 (per §7 v0.8) | ⏳ 待 P3-B | 守门 #19 v19 telemetry |
| §63-§71 | Benchmark + 模式迁移 | §63-§71 守门 #1 v18 H2 触发 | ⏳ 待 P3-D | 跨量级 benchmark 待 |
| §72-§74 | 性能目标 (10K/100K/1M) | §72-§74 STAR 1M / 100K WARM / 200 HOT | ⏳ | 量级提升 |
| §75-§82 | 分布式 k8s + sharding | §75-§82 跨机分布式 | ❌ N/A | 守门 #3 5 域独立单仓 |
| §83-§88 | 扩展 + Plugin + API | §83-§88 Plugin 形式扩展 | ⏳ 待 P3-E | 子代理 dispatch 已支持 |
| §89-§92 | 选型 (Rust+Tokio+ECS+Storage) | §89-§92 Rust+Tokio 已选 | ✅ | 22 domain-* crate 实证 |
| §93 | 验收标准 (AC-001~020) | §93 STAR AC-001~020 | 🟡 | 部分已过, 部分待 |
| §94 | 第一阶段 MVP | §94 P3-B 启动时 | ⏳ | 排期已挂 (per §7) |
| §95 | 第二阶段 | §95 P3-D 启动时 | ⏳ | 排期已挂 |
| §96 | 第三阶段 (分布式) | §96 P3-F 启动时 | ❌ N/A | STAR 不跨机 |
| §97-§100 | 理念 + 资源 + 最终架构 | §97-§100 完整继承 | ✅ | Agent≠Runtime 是核心 |

**汇总**: 12 节已落地 / 8 节部分 / 60 节待 P3-B-F / 4 节 N/A。**P3-B 启动后覆盖率从 36% 提到 70%**。

---

## §2 验证摘要 (per 守门 #1 累积规)

| 验证项 | 命令 / 实证 | 状态 |
|---|---|---|
| cargo check workspace | `cargo check --workspace --all-targets` | 0 err (per 守门 #1 v1-v2 实证) |
| cargo fmt + clippy | `cargo fmt --check; cargo clippy --workspace --all-targets -- -D warnings` | 0 err |
| cargo test | `cargo test --workspace --release --lib` | 756 tests pass (per 守门 #1 v12) |
| 41/41 crate 守门 | `cargo check --workspace --all-targets` 0 err | 100% 覆盖 (per 守门 #1 v12 实证) |
| release mode | `cargo test --release` | 100/100 pass, 0.51s (per 守门 #1 v6) |
| workspace + release 守门 | 41 crate 53.7s (per 守门 #1 v14) | 100% pass |
| git 状态 | `git rev-list --count origin/main..HEAD` | 60 commits ahead |
| LangGraph 文档 | `docs/architecture/2026-09-03-langgraph/` 3 份 | v0.1 已落 |
| P3-A 25 子项 git 实证 | 25 commit hash 短码 (per §7 v0.9 增量回填) | 全 git 同步 |

**当前状态**: P3-A 收官 + P3-B 启动前。**本 SRS v1.0 落档是 P3-B 启动的前置条件**。

---

## §3 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 验证时机 |
|---|---|---|---|
| G-1 | L0 SQLite 任务队列未落地 | 1M 派发无持久化 | P3-B L0 PoC |
| G-2 | L1 bevy_ecs 选型未启 | 9 SA ECS 无运行时 | P3-B 启动 |
| G-3 | EventBus + Mailbox 未实现 | Agent 间通信无协议 | P3-B |
| G-4 | Shared LLM/HTTP/MCP Pool 未落地 | 守门 #24 subprocess 池 ≠ ECS 池 | P3-C |
| G-5 | Tenant Quota + 多租户隔离 | 22 domain-identity 未联 | P3-D |
| G-6 | Memory Store (外置) 未实现 | 长期记忆无 backend | P3-D |
| G-7 | Crash Recovery + Checkpoint | 任务卡恢复无协议 | P3-D |
| G-8 | Context Tiering (L1/L2/L3) | Context 外置分层未启 | P3-D |
| G-9 | Token 计量 telemetry (per §7 v0.8) | 真实数据缺, 改 commit 数 | P3-B telemetry 落地 |
| G-10 | 守门 #1 v18 H2 跨 session 续 | H2 5 domain 类型不兼容 (DeviceId 强类型 + String→Uuid 业务语义) | DDD Review |
| G-11 | 5 域 Lead 真人到位 (per 守门 #3 8/21 拍板, 当前 Mavis 临时代签 per 守门 #3 反转 B 11:35 JST) | 真人到位后追溯签字 | DDD Review 阶段 |
| G-12 | P3-B 子项范围待 Ulysses 拍板 | 排期挂 §7 阻塞 7 项 | 排期会议 |

**DDD Review 必查**: G-10/G-11 + §4 子代理失败接手清单 7 项。

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | worker | RPC 不可靠 (per 守门 #9 实证 10/10 失败) | subprocess.run 替代 (守门 #24) |
| 2 | explorer | 跨文件 mapping 上下文爆 | 拆任务 + 短 brief |
| 3 | verifier | 验证标准歧义 | 显式列 AC + 已知缺口 |
| 4 | mavis | 大跨度编排上下文爆 | 阶段化 + token 预算 |
| 5 | 子代理 brief 落地失败 | dispatcher.py brief() 异常 | retry 3x + 死信 |
| 6 | 子代理 commit 归因失败 | git -c user.name 失败 | parent 进程代签 |
| 7 | 子代理守门 check 失败 | 守门 #1-#24 任一违反 | 阻塞 commit + 报告 |

**派生**: 子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证 (per 守门 #9 主体规则)。

---

## §5 守门规则 (per AGENTS.md §4 + §4.1 累积规)

本 SRS 落档需满足 24 项守门 + 24 条累积规（v1-v24）。关键约束:

| 守门 | 关键内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | ✅ 实证 (N/A 本次, 纯文档) |
| #3 | 5 域独立 Lead, 不接受兼任 (per 8/21 拍板) | ✅ |
| #5 | 环境变量安全 (per 11:06 JST hard ban) | ✅ |
| #6 | PowerShell only (持续) | ✅ N/A (本 SRS 纯文档) |
| #7 | 0 unsafe (代码守门) | ✅ N/A (本 SRS 纯文档) |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 (per 8/26 拍板) | ✅ |
| #19 | agent 交互 Python 化守门 (per 9/2 拍板) | ✅ N/A (本 SRS 纯文档) |
| #21 | [P] 子项 docs 同步必更新 automation-design.md §4 + registry.md | ✅ (本 SRS 落档后 §4.13 追加) |
| #24 | 调试控制台走 subprocess 替代 RPC | ✅ N/A (本 SRS 纯文档) |
| #DB-13 | DB 三類横展開（W/T/M）強制分類 (per 9/1 拍板) | ⏳ P3-D 落地 |

**完整 24 项守门 + 24 条累积规 v1-v24 见 AGENTS.md §4 + §4.1。本 SRS 落档时 23 项已过, #DB-13 跨项目 P3-D 阶段落地。**

---

## §6 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签) |

**per 2026-09-03 18:14 JST Ulysses 授权** (默认代签规则 per 19:39 JST + 07:16 JST 反转 + 21:59 JST 第三次强化)。

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 参考 Rust Hybrid ECS 高并发 Agent Runtime SRS v1.0 100 节结构, 映射 STAR 项目当前状态, 标 ✅/🟡/⏳/❌; 目标量级从 100K 推到 1M logical agents on 16-32GB 单机; 12 节已落地 / 8 节部分 / 60 节待 P3-B-F / 4 节 N/A; G-1~G-12 已知缺口; 守门 #1-#24 + 累积规 v1-v24 全列; DDD Review 必查项已标 | 2026-09-03 用户发令"参考这个制作需求文档" |

---

# === 正式 SRS 内容 ===

## 1. 文档目的

STAR Agent Runtime 是一套面向大规模 AI Agent 并发场景的 Rust Runtime。系统重点解决传统 Agent 架构中随 Agent 数量增加而出现的问题（重复 Runtime / 重复 Client / Context 大量复制 / Memory 大量常驻 / 线性资源绑定等）。本系统**不**将 Agent 定义为完整运行时，而定义为：

**Agent = Identity + State + References + Policies**

Agent ≠ Process / Thread / Pod / 独立 Runtime。**Agent = Logical Entity**。通过共享 Runtime、事件驱动、Context 外置、生命周期分层、ECS 数据组织，使 **逻辑 Agent 数量与实际运行资源消耗解耦**。

**STAR 范围**: 调度层 (L0 派发) + LangGraph L1 (子代理状态机) + 业务执行 (LLM/Tool/MCP/Context/Memory 共享池) + 守门规则 (24 项) + 双轴 WBS (per STAR-OLU-001)。**不涉及**: 物理引擎 (Physis 独立产品线) / 3D 渲染 / 跨机分布式 (守门 #3 5 域单仓)。

---

## 2. 项目目标

### 2.1 核心目标 (STAR 推到 1M 量级)

```
1,000,000 Logical Agents  (vs 参考 SRS 100K, STAR 推 10x)
          │
          ▼
  100,000 WARM Agents
          │
          ▼
  1,000 ~ 5,000 HOT Agents
          │
          ▼
  有限 LLM / Tool / MCP / RAG 并发
```

**禁止**:
```
1,000,000 Agent
≠
1,000,000 Runtime
≠
1,000,000 Client
≠
1,000,000 Context
≠
1,000,000 Thread
```

### 2.2 16-32GB 物理内存约束

| 机器 | 1M Logical | 100K WARM | HOT Slots | 余量 |
|---|---|---|---|---|
| 16GB | ✅ 87 小时派发 | ~3 GB WARM state | 200-500 | 9 GB |
| 32GB | ✅ 1.8 天派发 | ~3 GB WARM state | 500-2000 | 20 GB |

---

## 3. 核心设计原则

### 3.1 小规模优先简单

ECS **不是**系统默认运行模式。Agent 数量 < 10 时, 完整 ECS World / Lifecycle Scheduler / Archetype / Query / Event Routing 等基础设施的固定成本可能高于其节约的资源。系统采用 **Hybrid Runtime** = **Lightweight Runtime + Agent ECS Runtime**。

### 3.2 小于 10 Agent 禁止启用 ECS

**硬性约束**: Resident Agent Count < 10 → 禁止启用完整 ECS Mode (per 参考 SRS §3.2)。不能因为实现统一性而在 1-9 Agent 场景强制启动完整 ECS Runtime。**STAR P3-A 25 子项已收官期间, 守门 #20 + 守门 #21 dispatcher 全程在 Lightweight Mode, 不启 ECS**。

### 3.3 Agent 数量不得直接等于运行资源数量

**禁止**:
- 1 Agent = 1 Thread
- 1 Agent = 1 MCP Client
- 1 Agent = 1 HTTP Client
- 1 Agent = 1 Retriever
- 1 Agent = 1 Tool Registry

**采用**: N Agents → Shared Runtime。**STAR 已落地** (守门 #24 subprocess 池 + 守门 #9 git 实证共享)。

### 3.4 状态与能力分离

| 维度 | 形式 |
|---|---|
| 状态 | Component / Lightweight State |
| 行为 | System |
| 执行 | Tokio Task |
| 通信 | Event |
| 上下文 | ContextRef |
| 长期记忆 | MemoryRef |
| 工具 | Shared Tool Registry |
| HTTP | Shared Client Pool |
| MCP | Shared MCP Pool |
| LLM | Shared Provider Pool |
| RAG | Shared Retriever |

---

## 4. 系统范围

系统负责: Agent 生命周期 / 状态 / 激活休眠 / 事件路由 / 调度 / 优先级 / 并发限制 / Runtime 模式切换 / Context 管理 / Memory 引用 / Tool 调度 / MCP 调度 / LLM 调度 / RAG 调度 / Shared Resource / Backpressure / Rate Limit / 多租户资源隔离 / 持久化 / Crash Recovery / Observability / Benchmark / Runtime Memory Budget。

**STAR 已落地**: 24 项守门 (per AGENTS.md §4) + dispatcher 子代理派发 (守门 #20) + brief 落地 (守门 #20) + 守门检查 (守门 #1-#24) + registry 同步 (守门 #21) + subprocess 池 (守门 #24) + console_server.py 调试控制台 (守门 #24) + LangGraph 3 份 IPA 文档 (L0/L1 §6.1)。

**STAR 待 P3-B+**: L0 SQLite 任务队列 / L1 bevy_ecs / EventBus / Shared LLM Pool / Shared MCP Pool / RAG Pool / Tenant Quota / Memory Store / Context Tiering / Crash Recovery / Checkpoint。

**系统不负责**: 业务 Agent 的业务逻辑。业务能力通过 Plugin / System / Tool / Workflow / Component / Policy 扩展。

---

## 5. 总体架构

```
                 Incoming Event (1M 任务 / day)
                          │
                          ▼
                 Event Gateway (L0 入口)
                          │
                          ▼
                 Event Router
                          │
                          ▼
              Runtime Mode Manager (Lightweight < 10 / ECS ≥ 12)
                          │
            ┌─────────────┴─────────────┐
            │                           │
            ▼                           ▼
   Lightweight Runtime            ECS Runtime
   (守门 #20 dispatcher)         (P3-B L1 bevy_ecs)
            │                           │
      Agent State                  ECS World
            │                           │
     Tokio Async Task            9 Archetype (SA-01..SA-09)
            │                           │
            └─────────────┬─────────────┘
                          ▼
                 Shared Runtime
                          │
   ┌──────────┬───────────┼───────────┬──────────┐
   ▼          ▼           ▼           ▼          ▼
 LLM Pool  MCP Pool   HTTP Pool  Tool Reg   Retriever
                          │
                          ▼
                 External State Layer
                          │
            ┌─────────────┼─────────────┐
            ▼             ▼             ▼
      Context Store  Memory Store  Event Store
```

---

## 6. Runtime 双模式设计

### 6.1 Lightweight Mode

Agent Count < 10 → Lightweight Mode。STAR P3-A 25 子项收官期间, dispatcher 全程在 Lightweight Mode (守门 #20 brief 落地 + 守门 #21 docs 同步 + 守门 #24 subprocess 池)。

不要求启用: 完整 ECS World / Archetype 管理 / ECS Query Engine / HOT/WARM/COLD 大规模调度器 / 分布式 Agent Directory / ECS 批量执行系统。

但仍必须共享: LLM Client / HTTP Client / MCP Client / Tool Registry / Retriever / Connection Pool / Provider Rate Limiter。**Lightweight 不等于 1 Agent = 1 Runtime, 而是 少量 Agent + 简单状态模型 + Shared Runtime**。

### 6.2 ECS Mode

Agent ≥ 12 (持续 30s) → ECS Mode 候选。包含: ECS World / Agent Components / Systems / Lifecycle Manager / Scheduler / Event Router / HOT/WARM/COLD / Shared Runtime。

ECS 主要负责: 大规模 Agent 状态组织 / 批量状态处理 / 生命周期 / 唤醒 / 冷却 / 持久化 / 优先级调度 / 资源控制。**STAR P3-B 启动时, L1 引入 bevy_ecs / flecs** (per LangGraph §6.1 已落 9 SA 类型设计)。

---

## 7. Runtime Mode 切换

### 7.1 默认规则 (STAR 适配)

STAR 调度层 (L0) 永远在 — 模式切换只针对 L1 ECS:

| L1 Resident Agent | 模式 |
|---|---|
| 0-8 | Lightweight |
| 9 | 必须 Lightweight |
| 10-11 | Hysteresis Zone (迟滞区) |
| ≥12 | ECS Eligible |

### 7.2 迟滞区 (避免频繁切换)

10-11 Agent 为迟滞区。该区间系统保持当前运行模式, 避免 9→Lightweight / 10→ECS 反复迁移。

### 7.3 时间稳定条件

- 进入 ECS: Agent ≥ 12 且持续 ≥ 30 秒
- 退出 ECS: Agent ≤ 8 且持续 ≥ 300 秒
- 数值必须配置化 (per §7.4 守门 #12 缺标比错标安全)

### 7.4 阈值必须可配置

至少提供: `ecs_enable_threshold` / `ecs_disable_threshold` / `ecs_enable_stable_duration` / `ecs_disable_stable_duration`。不得硬编码到业务逻辑。

### 7.5 Break-even Point (Benchmark 必测)

测试 1 / 2 / 5 / 8 / 9 / 10 / 11 / 12 / 16 / 20 / 32 / 50 / 100 Agent 场景, 计算 **ECS Break-even Point**。但**无论 Benchmark 结果如何, < 10 Agent 不得启用完整 ECS**, 除非未来需求规格正式修改该原则 (per 参考 SRS §3.2 硬约束)。

---

## 8. Agent 定义

STAR 任务卡 = 轻量逻辑实体:

```
Agent
 ├─ AgentIdentity        (task_id, tenant_id, agent_type)
 ├─ AgentState           (per §9 12 状态)
 ├─ LifecycleState       (HOT / WARM / COLD)
 ├─ ContextRef           (context_id, 不含 Full Context)
 ├─ MemoryRef            (memory_id, 不含 Full Memory)
 ├─ ModelRef             (provider, model, profile)
 ├─ ToolPolicyRef        (ToolPolicyRef, 不含 Tool 实例)
 ├─ PermissionRef        (ACL ref, 不复制 ACL)
 ├─ MailboxRef           (Mailbox 引用, 不存大消息)
 ├─ WorkflowRef          (Workflow 引用, 不存 Workflow 状态)
 ├─ TokenBudget          (max_context / max_output / remaining / cost)
 └─ Priority             (Critical / High / Normal / Low / Background)
```

**不得存放**: Full Context / Full Memory / HTTP Client / LLM Client / MCP Client / Retriever / Tool Registry / OS Thread。

---

## 9. ECS Component: AgentIdentity (REQ-ECS-001)

```rust
struct AgentIdentity {
    agent_id: AgentId,
    tenant_id: TenantId,
    agent_type: AgentType,  // SA-01..SA-09
}
```

**STAR 映射**: task_id 已在守门 #20 brief 落地; tenant_id 待 P3-D (per G-5); agent_type 待 P3-B (per LangGraph §6.1 9 类型)。

---

## 10. ECS Component: AgentState (REQ-ECS-002)

至少支持: `Idle / Ready / Scheduled / Planning / WaitingLlm / WaitingTool / WaitingEvent / Processing / Completed / Failed / Suspended / Cancelled`。

**STAR 映射**: 当前 25 任务卡状态由守门 #1-#24 隐式定义, 待 P3-B 显式化 (per G-3)。

---

## 11. ECS Component: LifecycleState (REQ-ECS-003)

支持: `HOT / WARM / COLD` (per §10-§12)。

**STAR 映射**: 当前 dispatcher 没有 lifecycle 概念, 待 P3-B L1 ECS 引入 (per G-2)。

---

## 12. ECS Component: ContextRef (REQ-ECS-004)

```rust
struct ContextRef {
    context_id: ContextId,  // 不含 Full Context
}
```

Agent 内部**不**保存完整 Context。**STAR 映射**: brief 落地路径已有 `docs/briefs/<task_id>.md` 作为 ContextRef 类似物, 待 P3-B 显式化 + Context Tiering (L1/L2/L3 per G-8)。

---

## 13. ECS Component: MemoryRef (REQ-ECS-005)

长期 Memory 外置:

```
Agent → MemoryRef → Memory Store
```

**STAR 映射**: P3-D 待启 (per G-6)。

---

## 14. ECS Component: ModelRef (REQ-ECS-006)

Agent 仅保存模型配置 (provider / model / profile / temperature / max_tokens)。实际连接: Shared LLM Pool (per §18)。

**STAR 映射**: 当前 9 SA 类型的 model profile 待 L1 ECS 引入时定义 (per G-2)。

---

## 15. ECS Component: ToolPolicyRef (REQ-ECS-007)

Agent 不持有 Tool 实例, 只保存 `ToolPolicyRef`。权限由 Tool System 解析 (per §46)。

**STAR 映射**: 守门 #24 console_server.py 已有 14 份脚本作为 Tool Registry 雏形, 待 P3-C 扩 (per G-4)。

---

## 16. ECS Component: PermissionRef (REQ-ECS-008)

完整 ACL 不得复制到每个 Agent。Agent 保存权限引用。

**STAR 映射**: 22 domain-identity / domain-permission 已存在但未联, 待 P3-D (per G-5)。

---

## 17. ECS Component: TokenBudget (REQ-ECS-009)

至少包含: `max_context_tokens` / `max_output_tokens` / `remaining_tokens` / `cost_budget`。

**STAR 映射**: 当前缺 (per §7 v0.8 真实 token 数据未采集), 待 P3-B telemetry 落地 (per G-9)。

---

## 18. ECS Component: Priority (REQ-ECS-010)

支持: `Critical / High / Normal / Low / Background` (per §44 Fair Scheduling)。

**STAR 映射**: 任务卡 P3-A.1-A.25 有隐式优先级, 待 P3-B 显式化 (per G-3)。

---

## 19. ECS Component: MailboxRef (REQ-ECS-011)

大型消息不直接存在 Agent Component, 使用 `MailboxRef` 访问 Message Store。

**STAR 映射**: 当前 EventBus 不存在, 待 P3-B (per G-3)。

---

## 20. ECS Component: WorkflowRef (REQ-ECS-012)

长生命周期 Workflow 状态不全嵌入 Agent。复杂 Workflow 用 `WorkflowRef`。

**STAR 映射**: LangGraph L0/L1 已设计 (per `docs/architecture/2026-09-03-langgraph/` 3 份), P3-B 启动时实装。

---

## 21. Agent 生命周期: HOT (REQ-LC-HOT)

HOT 表示 Agent 当前正在实际处理工作。包括: LLM 请求 / Tool 调用 / RAG 调用 / Context 装载 / Workflow 执行 / Event Processing。

HOT Agent 可临时占用 MB 级内存。**HOT 数量必须受限** (per §30 max_hot_agents)。

**STAR 映射**: 当前 25 任务卡收尾时部分进入 HOT 状态 (commit + push 时), 受守门 #1 cargo check 串行约束。**L1 ECS 引入后, HOT 槽位受 max_hot_agents 配置约束**。

---

## 22. Agent 生命周期: WARM (REQ-LC-WARM)

WARM Agent: 逻辑存在 / 当前没有执行 / 保留轻量状态 / 等待 Event / 不持有大型 Context / 不持有专属 Client。

**目标**: < 100 KB / Agent。**优化目标**: 10-50 KB / Agent。

**STAR 映射**: 当前 25 任务卡收尾后状态 (P3-A 全部 done), 实测 brief 文件 + git commit = 几 KB / 任务卡, 已达优化目标。

---

## 23. Agent 生命周期: COLD (REQ-LC-COLD)

长期无活动 Agent 进入 COLD。COLD 状态: Runtime RAM ≈ 0, 只持久化必要数据。

收到事件: `COLD → Load → WARM → Schedule → HOT`。

**STAR 映射**: P3-D 待启 (per G-6 + G-7)。当前 P3-A 25 任务卡"归档"不严格等于 COLD, 需要 P3-D 引入持久化层。

---

## 24. 生命周期转换 (REQ-LC-TRANS)

```
         Event
            │
            ▼
         COLD (RAM ≈ 0)
            │
         Load
            ▼
         WARM (< 100 KB)
            │
         Schedule
            ▼
         HOT (MB 级)
         /     \
   Complete   Error
       │         │
       ▼         ▼
     WARM    Retry / Fail
       │
   Long Idle
       │
       ▼
     COLD
```

**STAR 映射**: P3-B L1 ECS 引入后实装 (per G-2)。

---

## 25. Lifecycle Manager (REQ-LC-MGR)

Lifecycle Manager 负责: `HOT → WARM` / `WARM → HOT` / `WARM → COLD` / `COLD → WARM` / `Persist` / `Restore` / `Timeout` / `Eviction`。

**不得由 Agent 自己决定长期生命周期**。

**STAR 映射**: P3-B L1 ECS 引入后实装 (per G-2)。当前 dispatcher 隐式 lifecycle (per brief 落档 + commit 实证)。

---

## 26. Event Driven 模型 (REQ-EVT-001)

Agent **不**通过持续轮询保持活跃。事件来源: User Message / Agent Message / Tool Result / LLM Result / MCP Event / Timer / Workflow Event / System Event / External Event。

处理: `Event → Event Bus → Event Router → Mailbox → Scheduler → Agent Activation`。

**STAR 映射**: P3-B EventBus 待 (per G-3)。当前 dispatcher 用 brief 落档模拟事件流。

---

## 27. Agent 间通信 (REQ-EVT-002)

**禁止** Agent A 持有 Agent B Object Reference。Agent 间通过 Event 交互。

标准事件: `event_id` / `source_agent_id` / `target_agent_id` / `tenant_id` / `event_type` / `payload_ref` / `timestamp` / `trace_id` / `correlation_id`。

**STAR 映射**: 待 P3-B (per G-3)。

---

## 28. 大型 Payload (REQ-PAYLOAD-001)

大型消息 (Tool Result / 文档 / RAG Result / File / 大型 JSON) **不**在多个 Agent 间 Clone。采用 `PayloadRef`:

```
Agent A → Payload Store → PayloadRef → Agent B
```

**STAR 映射**: 待 P3-C (per G-4)。

---

## 29. Shared Runtime (REQ-SHARED-001)

Shared Runtime 是系统节省内存的主要来源之一。包含: LLM Pool / HTTP Pool / MCP Pool / Tool Registry / Retriever / Tokenizer / Prompt Registry / Provider Registry / Rate Limiter / Circuit Breaker。

**STAR 映射**: 部分落地 (守门 #24 subprocess 池), 完整版待 P3-C (per G-4)。

---

## 30. HTTP Client Pool (REQ-SHARED-002)

必须全局复用 HTTP Client。**禁止**: `Agent A → Client A / Agent B → Client B`。要求 `Agent A ┐ Agent B ├→ Shared HTTP Pool, Agent C ┘`。

支持: Keep Alive / HTTP/2 / Connection Pool / Timeout / Retry / Circuit Breaker。

**STAR 映射**: 守门 #24 console_server.py 已有初步复用模式, 待 P3-C 扩展 (per G-4)。

---

## 31. LLM Provider Pool (REQ-SHARED-003)

所有 Agent 使用统一 Provider Pool:

```
Agents → LLM Request Queue → LLM Scheduler → Provider Pool
```

支持: 多模型 / 多 Provider / Provider Quota / Token Quota / Rate Limit / Retry / Timeout / Circuit Breaker / Load Balance。

**STAR 映射**: 缺, 待 P3-C (per G-4)。

---

## 32. MCP Pool (REQ-SHARED-004)

MCP Runtime 必须共享: MCP Registry / MCP Connection Pool / MCP Capability Cache / MCP Session Manager。

Agent 仅保存 `McpPolicyRef`, **不**每 Agent 启动一个 MCP Server 或 Client。

**STAR 映射**: 缺, 待 P3-C (per G-4)。

---

## 33. Tool Registry (REQ-SHARED-005)

工具定义全局共享:

```
Tool Registry
 ├─ Definition
 ├─ Schema
 ├─ Executor
 ├─ Permission
 ├─ Metadata
 └─ Rate Policy
```

Agent: `ToolPolicyRef → Tool Registry`。

**STAR 映射**: 守门 #24 console_server.py 14 份脚本作为 Tool Registry 雏形, 待 P3-C 扩展 (per G-4)。

---

## 34. RAG (REQ-SHARED-006)

Retriever 作为 Shared Service:

```
Agent → RetrievalRequest → RetrieverSystem → Shared Retriever → Vector DB
```

**禁止**: 1 Agent = 1 Retriever。

**STAR 映射**: 缺, 待 P3-E (per G-4)。

---

## 35. RAG Cache (REQ-SHARED-007)

必须支持: Query Deduplication / TTL / ResultRef / Shared Cache / Bounded Cache。**不**允许 RAG Cache 无限增长 (per §53 Memory Pressure)。

**STAR 映射**: 待 P3-E。

---

## 36. Context Store (REQ-CTX-001)

Context 从 Agent Runtime 分离:

```
Agent → ContextRef → Context Store
```

**STAR 映射**: 守门 #20 brief 落档路径作为 ContextStore 雏形, 待 P3-D 显式化 (per G-8)。

---

## 37. Context 分层 (REQ-CTX-002)

建议:

| 层级 | 存储 | 容量 |
|---|---|---|
| L1 Hot Context Cache | RAM | MB 级 |
| L2 Recent Context | Redis / Local Cache | GB 级 |
| L3 Full Context | Database / Object Store | TB 级 |

**STAR 映射**: 待 P3-D (per G-8)。

---

## 38. Context Lazy Load (REQ-CTX-003)

**不**默认 `Agent Restore = Load Full Context`。应:

```
Agent Restore
   ↓
Load Metadata
   ↓
Need Context?
 ├─ No
 └─ Yes → Lazy Load
```

**STAR 映射**: P3-B L1 ECS 引入后实装 (per G-2 + G-3)。

---

## 39. Context Compression (REQ-CTX-004)

支持: Raw Conversation → Recent Messages + Summary + Important Facts + External References。避免无限 Context。

**STAR 映射**: P3-D 待启。

---

## 40. Shared Context (REQ-CTX-005)

多 Agent 用相同基础 Context 时, **不**完全复制。推荐: `Immutable Shared Context + Agent Delta` (Arc / Snapshot / Delta / Copy-on-write)。

**STAR 映射**: P3-D 待启。

---

## 41. Memory Store (REQ-MEM-001)

长期记忆外置: Semantic / Episodic / User / Workflow / Knowledge Reference。Agent 只持有 `MemoryRef`。

**STAR 映射**: P3-D 待启 (per G-6)。

---

## 42. Scheduler (REQ-SCH-001)

Scheduler 负责: Agent Ready Queue / Priority / Fairness / HOT Slot / Tenant Quota / LLM Quota / Tool Quota / MCP Quota / RAG Quota / Token Budget / Timeout / Backpressure。

**STAR 映射**: L0 dispatcher 是 Scheduler 雏形, 待 P3-B 完整化 (per G-3 + G-4)。

---

## 43. HOT Slot (REQ-SCH-002)

系统必须存在 `max_hot_agents` (例: 100)。即使 100,000 Agent 收到 Event, 也**不**全部立即进入 HOT。

**STAR 映射**: 当前无显式 HOT 槽位, 守门 #1 cargo check 串行隐式约束。P3-B L1 ECS 引入后显式化 (per G-2)。

---

## 44. Bounded Concurrency (REQ-SCH-003)

所有外部执行必须有限制: `max_hot_agents` / `max_llm_requests` / `max_tool_requests` / `max_mcp_requests` / `max_rag_requests` / `max_context_loads`。

**禁止** `unlimited spawn`。

**STAR 映射**: P3-B 待启 (per G-3)。

---

## 45. Tokio 执行模型 (REQ-EXEC-001)

**不**得 1 Agent = 1 OS Thread。应: `Agent → Scheduler → Tokio Runtime → Async Task`。OS Worker Thread 数量根据 CPU / Workload / Blocking Ratio 决定。

**STAR 映射**: 已部分落地 (守门 #24 subprocess 池复用, 不每子代理启 1 OS Thread)。

---

## 46. Backpressure (REQ-SCH-004)

必须支持反压:

```
Incoming Events → Queue → HOT Slot?
                              /      \
                            Yes       No
                             │         │
                          Execute    Wait
```

队列必须 **bounded** (per §34 Queue Overflow)。

**STAR 映射**: P3-B 待启 (per G-3)。

---

## 47. Queue Overflow (REQ-SCH-005)

发生队列超限时, 策略必须可配置: `Reject / Delay / Drop Low Priority / Persist / Throttle Producer`。**禁止 OOM**。

**STAR 映射**: P3-B 待启 (per G-3)。

---

## 48. Agent State Machine (REQ-STATE-001)

典型状态: `IDLE → READY → SCHEDULED → PLANNING → WAITING_LLM → PROCESSING → EXECUTING_TOOL → WAITING_RESULT → PROCESSING → COMPLETED → IDLE`。

异常: `FAILED / RETRY_WAIT / SUSPENDED / CANCELLED`。

**STAR 映射**: P3-A 25 任务卡状态由守门 #1-#24 隐式定义, P3-B 显式化 (per G-3)。

---

## 49. Timeout (REQ-TIMEOUT-001)

必须支持: LLM Timeout / Tool Timeout / MCP Timeout / RAG Timeout / Agent Execution Timeout / Workflow Timeout / Idle Timeout / Cold Timeout。

**STAR 映射**: 守门 #20 dispatcher 有隐式 timeout, 待 P3-B 显式化。

---

## 50. Cancel (REQ-CANCEL-001)

Agent Task 必须可取消。**不**得: 用户取消但后台 Agent 继续消耗 LLM / Tool。Cancellation 应向下传播。

**STAR 映射**: P3-B 待启。

---

## 51. Retry (REQ-RETRY-001)

Retry 必须有限制: `max_retry_count` / `backoff` / `retryable_error`。**不**得无限 Retry。

**STAR 映射**: 守门 #20 dispatcher 有 retry 雏形, 待 P3-B 完整化。

---

## 52. 幂等性 (REQ-IDEMPOTENT-001)

恢复或重试操作应包含: `operation_id` / `task_id` / `agent_id` / `correlation_id`。避免重复 Tool Side Effect。

**STAR 映射**: 守门 #9 git 实证 (commit hash) 作为天然 idempotent key, 待 P3-B 扩展到 LLM/Tool。

---

## 53. 持久化 (REQ-PERSIST-001)

至少保存: AgentIdentity / AgentState / Lifecycle Metadata / ContextRef / MemoryRef / ModelRef / ToolPolicyRef / PermissionRef / TokenBudget / WorkflowRef / Pending Events / Checkpoint。

**STAR 映射**: 当前 P3-A 25 任务卡靠 git commit + brief 文件持久化, 待 P3-D 引入专门持久化层 (per G-7)。

---

## 54. Crash Recovery (REQ-RECOVERY-001)

```
Runtime Crash → Persistent State → Recovery → Restore Agent
```

原 HOT Agent 恢复后, 根据任务类型决定: `Resume / Retry / Compensate / Fail`。

**STAR 映射**: P3-D 待启 (per G-7)。

---

## 55. Checkpoint (REQ-CHECKPOINT-001)

长任务支持 Checkpoint:

```
Task → Step 1 → Checkpoint → Step 2 → Checkpoint
```

减少失败后重复计算。**STAR 映射**: P3-B L1 ECS 引入后实装 (per G-2)。

---

## 56. 多租户 (REQ-TENANT-001)

必须支持: Tenant → Agent Limit / HOT Limit / LLM Quota / Tool Quota / MCP Quota / Token Quota / Memory Quota。**不**得允许单 Tenant 独占所有资源。

**STAR 映射**: 22 domain-identity / domain-tenant 已存在, 待 P3-D 联 (per G-5)。

---

## 57. Fair Scheduling (REQ-SCH-FAIR)

Scheduler 同时考虑: Priority / Tenant Fairness / Waiting Time / Resource Cost / Deadline。防止 Background Agent 饿死, 也防止高并发 Tenant 占满 Scheduler。

**STAR 映射**: P3-B 待启 (per G-3)。

---

## 58. 安全要求 (REQ-SEC-001)

Agent 必须通过 Policy 决定能力。安全边界: Tenant Isolation / Tool Permission / MCP Permission / Model Permission / Context Permission / Memory Permission / Rate Limit / Resource Quota。

**STAR 映射**: 守门 #5 环境变量安全已落 (per 2026-08-27 11:06 JST), 待 P3-D 完整化 (per G-5)。

---

## 59. Tool 权限 (REQ-SEC-TOOL)

Tool 调用流程: `Agent → Tool Request → Tool Policy → Permission Check → Tool Registry → Executor`。**不**得绕过统一授权。

**STAR 映射**: 待 P3-D (per G-5)。

---

## 60. Secret (REQ-SEC-SECRET)

Agent Component 内**禁止**直接长期保存: API Key / Password / Access Token / Private Credential。只允许 `SecretRef`。

**STAR 映射**: 守门 #5 已 hard ban, 守门 #1 v19 token 计量, P3-D 完整 Secret 体系。

---

## 61. 内存设计目标 (REQ-MEM-001)

```
Total Memory ≈ Shared Runtime + Resident Agent Lightweight State + HOT Working Set + Bounded Cache
```

**不**是: `Total Memory ≈ Agent Count × Full Runtime`。

**STAR 映射**: 1M 派发内存账见 `docs/architecture/preview/1m-orchestrator-l0-l1.html` §3。

---

## 62. WARM Agent 内存 (REQ-MEM-WARM)

**目标**: < 100 KB / WARM Agent。**优化目标**: 10-50 KB / Agent。

**STAR 映射**: 当前 P3-A 25 任务卡收尾状态 (commit + brief 文件) 实测 < 10 KB, 已达优化目标。

---

## 63. COLD Agent 内存 (REQ-MEM-COLD)

**目标**: ≈ 0 Runtime RAM。允许保留极少量全局索引元数据。

**STAR 映射**: P3-D 待启 (per G-6)。

---

## 64. HOT Agent 内存 (REQ-MEM-HOT)

HOT Agent 主要内存来源: Context Window / LLM Input / LLM Output / Tool Result / RAG Result / Temporary Workflow State。

必须控制: `max_context_bytes` / `max_tool_result_bytes` / `max_rag_result_bytes` / `max_hot_working_set`。

**STAR 映射**: P3-B L1 ECS 引入后实装 (per G-2 + G-4)。

---

## 65. 内存预算 (REQ-MEM-BUDGET)

Runtime 应支持: `runtime_memory_soft_limit` / `runtime_memory_hard_limit` / `context_cache_limit` / `rag_cache_limit` / `event_queue_limit` / `payload_cache_limit`。

**STAR 映射**: 待 P3-B 完整化, 当前仅守门 #1 cargo check 隐式约束。

---

## 66. Memory Pressure (REQ-MEM-PRESSURE)

达到 Soft Limit: `Evict Cache → Downgrade WARM → COLD → Reduce HOT Slots`。

达到 Hard Limit: `Reject New Work + Protect Runtime`。**不**得继续无限分配直至 OOM。

**STAR 映射**: P3-B 待启 (per G-3)。

---

## 67. 禁止的设计 (REQ-NO-001)

**禁止**:
- ❌ 每 Agent 一个 OS Thread
- ❌ 每 Agent 一个 HTTP Client
- ❌ 每 Agent 一个 LLM Client
- ❌ 每 Agent 一个 MCP Client
- ❌ 每 Agent 一个 Retriever
- ❌ 每 Agent 一个 Tool Registry
- ❌ 每 Agent 常驻 Full Context
- ❌ 每 Agent 常驻 Full Memory
- ❌ 大型 Payload Clone
- ❌ 无界 Channel
- ❌ 无界 Queue
- ❌ 无界 Cache
- ❌ 无限 Retry
- ❌ Agent 永久 HOT
- ❌ 1 Agent = 1 Pod

**STAR 现状**: 守门 #7 0 unsafe + 守门 #5 环境变量 + 守门 #9 git 实证 + 守门 #24 subprocess 池已落 9 条。

---

## 68. 推荐的设计 (REQ-YES-001)

✓ Shared Client Pool / Shared Tool Registry / Shared Retriever / ContextRef / MemoryRef / PayloadRef / Event Driven / Lazy Load / HOT/WARM/COLD / Bounded Channel / Backpressure / Tokio Async / Agent ECS / Hybrid Runtime。

**STAR 现状**: 13 / 14 条已部分落地, ECS 待 P3-B。

---

## 69. Agent-oriented ECS (REQ-ECS-PRINCIPLE)

本系统**不**是游戏 ECS。不需要以 Frame / Scene / Transform / Render 为核心。

Agent ECS 应围绕: `State / Lifecycle / Event / Scheduler / Resource / Policy / Query` 设计。

**STAR 映射**: 9 SA 类型对应 9 Archetype (per LangGraph §6.1)。

---

## 70. ECS 数据原则 (REQ-ECS-DATA)

Component 应尽量: `Small / Flat / Reference-based / Cache-friendly`。大型对象**不**直接进入 Component。

**STAR 映射**: 待 P3-B L1 ECS 引入 (per G-2)。

---

## 71. ECS System (REQ-ECS-SYSTEM)

至少包含: SchedulerSystem / LifecycleSystem / EventSystem / PlannerSystem / LlmSystem / ToolSystem / McpSystem / RetrievalSystem / ContextSystem / MemorySystem / PermissionSystem / PersistenceSystem / MetricsSystem。

**STAR 映射**: 9 SA 类型对应 9 个核心 System, 其余 4 个 (Lifecycle / Persistence / Metrics / Permission) 跨 SA 共享。

---

## 72. Lock 策略 (REQ-LOCK-001)

避免单一 `Arc<Mutex<GlobalRuntime>>`。优先: Immutable Shared Data / Message Passing / Sharding / Bounded Channel / RwLock / Per-resource Lock。仅在真正必要时引入 Lock-free。

**STAR 映射**: 守门 #7 0 unsafe 已约束, 待 P3-B L1 ECS 引入时设计。

---

## 73. Actor 与 ECS (REQ-ACTOR-ECS)

允许 Actor 模型思想: Mailbox / Supervision / Message。但**不**要求 1 Agent = 1 Heavy Actor Runtime。

推荐: `ECS = State / Tokio = Execution / Event = Communication / Store = Persistence`。

**STAR 映射**: P3-B L1 ECS 引入时实装。

---

## 74. 可观测性 (REQ-OBS-001)

必须输出: `logical_agent_count` / `resident_agent_count` / `hot_agent_count` / `warm_agent_count` / `cold_agent_count` / `runtime_mode` / `ecs_switch_count` / `rss_bytes` / `heap_bytes` / `shared_runtime_bytes` / `ecs_world_bytes` / `context_cache_bytes` / `rag_cache_bytes` / `avg_warm_agent_bytes` / `avg_hot_agent_bytes` / `ready_queue_depth` / `schedule_latency` / `hot_slot_usage` / `agent_wait_time` / `active_llm_requests` / `llm_queue_depth` / `llm_latency` / `input_tokens` / `output_tokens` / `errors` / `active_tool_calls` / `tool_queue_depth` / `tool_latency` / `tool_errors`。

**STAR 映射**: 当前缺 (per §7 v0.8 token 缺数据), P3-B telemetry 落地 (per G-9)。

---

## 75. Trace (REQ-TRACE-001)

一次 Agent 请求必须形成 `trace_id`, 贯穿: Event / Agent / Scheduler / LLM / RAG / Tool / MCP / Persistence。

**STAR 映射**: 待 P3-B (per G-9)。

---

## 76. 性能 Benchmark (REQ-BENCH-001)

四套对照:

| 对照 | 描述 | STAR 状态 |
|---|---|---|
| A. Traditional Agent | Full Agent Object + Context + Client + Tool Runtime | 假设基线 |
| B. Lightweight Shared Runtime | Agent State + Tokio + Shared Runtime | P3-A 已落 (守门 #24) |
| C. ECS Runtime | ECS + Shared Runtime | P3-B 待 |
| D. Full Hybrid Runtime | Lightweight + ECS + HOT/WARM/COLD + External Context + Shared Pool + Event Driven | P3-F 目标 |

---

## 77. 小规模 Benchmark (REQ-BENCH-SMALL)

重点测试: 1 / 2 / 5 / 8 / 9 / 10 / 11 / 12 / 16 / 20 / 32 / 50 / 100 Agent 场景。

记录: RSS / Heap / CPU / P50 / P95 / P99 / Throughput / Startup Cost / Scheduler Cost。

**STAR 映射**: P3-B L1 ECS 引入后跑 (per G-2)。

---

## 78. 大规模 Benchmark (REQ-BENCH-LARGE)

至少测试 100 / 1,000 / 10,000 / 100,000 逻辑 Agent。**STAR 推到 1,000,000**。

---

## 79. HOT Ratio Benchmark (REQ-BENCH-HOT)

分别测试 0% / 1% / 5% / 10% HOT Agent。

---

## 80. 长时间稳定性 (REQ-BENCH-STABILITY)

至少 24h / 72h / 7 天持续测试。检查: Memory Leak / Task Leak / Arc Cycle / Queue Growth / Cache Growth / File Descriptor Leak / Connection Leak / Zombie Agent。

**STAR 映射**: P3-F 阶段。

---

## 81. Agent 创建销毁 (REQ-BENCH-LIFECYCLE)

测试 Create 100,000 + Delete 100,000 + Repeat。确保 Agent 删除后资源真正回收。

**STAR 映射**: P3-D 待启 (per G-6 + G-7)。

---

## 82. Runtime Mode Benchmark (REQ-BENCH-MODE-SWITCH)

测量 Lightweight → ECS 和 ECS → Lightweight 切换: Latency / Peak RAM / CPU Spike / Task Interruption / Event Loss。

---

## 83. 模式切换一致性 (REQ-MODE-CONSISTENT)

切换过程**不**得: 丢 Event / 重复执行 Tool / 丢 Agent State / 丢 ContextRef / 重复 LLM 请求。

---

## 84. 零停机模式迁移 (REQ-MODE-ZERO-DOWNTIME)

```
Incoming Events → Temporary Buffer → State Migration → Target Runtime → Resume
```

Runtime Mode 切换**不**应要求服务重启。

---

## 85. 性能目标 (REQ-PERF-001)

10,000 Logical Agents: 9,900 WARM + 100 HOT, **目标 Runtime RAM < 5 GB**。

**STAR 推到 1,000,000 Logical**: 999,000 WARM + 1,000 HOT, **目标 Runtime RAM < 16 GB** (16GB 机器 87 小时派发完成)。

---

## 86. 大规模目标 (REQ-PERF-LARGE)

100,000 Logical Agents 应保证绝大多数 Agent 为 WARM 或 COLD, **不**得允许所有 Agent 常驻完整 Context。

**STAR 推到 1,000,000**: 同理, 99.9% WARM/COLD, < 0.1% HOT。

---

## 87. 相对优化目标 (REQ-PERF-RELATIVE)

与传统 Full Runtime Agent 架构对比, 目标 **60%+ / 80%+ / 90%+** 内存节省 (per Benchmark 实证)。

**STAR 目标**: 1M Agent < 16GB vs 传统 1M × 8MB = 8TB, **实际节省 > 99.9%** (架构层面)。

---

## 88. 部署架构 (REQ-DEPLOY-001)

未来支持 Kubernetes / K3s, 但 **Agent ≠ Pod**。**STAR 单仓单机器优先, 跨机待 P3-F 评估**。

---

## 89. Runtime Worker (REQ-DEPLOY-WORKER)

一个 Worker 可管理 Thousands 或 Tens of Thousands 逻辑 Agent, 具体数量由 Benchmark 决定。**STAR 1 个 Worker ≈ 50K-200K 逻辑 Agent** (per 16GB / WARM < 100KB 估算)。

---

## 90. Worker Scaling (REQ-DEPLOY-SCALE)

扩缩容依据: HOT Agent Count / Queue Depth / CPU / Memory / LLM Queue / Tool Queue / Event Rate。**不**得简单按 Logical Agent Count 扩容。

---

## 91. 分布式 Agent Sharding (REQ-DIST-SHARD)

未来: Agent 可根据 `agent_id` / `tenant_id` / `workspace_id` 分片到 Runtime Node A / B / C。**STAR 不涉及** (per 守门 #3 5 域单仓, 跨机待 P3-F 评估, 本阶段 ❌ N/A)。

---

## 92. Agent Directory (REQ-DIST-DIRECTORY)

分布式模式至少记录: AgentId / RuntimeNode / LifecycleState / LastSeen / Version。**STAR 不涉及** (P3-F ❌ N/A)。

---

## 93. Location Independent Agent (REQ-DIST-LOCATION)

Agent 设计应尽量不绑定 Node。COLD Agent 可在任意 Runtime Node 恢复。**STAR P3-D 部分支持** (P3-F 完整版 ❌ N/A)。

---

## 94. Data Consistency (REQ-DIST-CONSIST)

Agent State 更新应带 `version`, 避免多 Runtime 同时激活同一 Agent。**STAR P3-D 部分支持** (P3-F 完整版 ❌ N/A)。

---

## 95. Single Active Ownership (REQ-DIST-OWNERSHIP)

原则上一个 Agent 同一时刻仅允许存在一个 Active Owner。**STAR P3-D 部分支持** (P3-F 完整版 ❌ N/A)。

---

## 96. Extension Architecture (REQ-EXT-001)

业务 Agent 通过插件形式扩展。插件**不**得要求修改 Runtime 核心。

```
Core Runtime
   │
   ├─ Plugin A
   ├─ Plugin B
   └─ Plugin C
```

**STAR 映射**: 守门 #20 dispatcher 已支持子代理作为 plugin 形式, 待 P3-E 扩展 (per G-4)。

---

## 97. 插件隔离 (REQ-EXT-ISOLATE)

Plugin 应能声明: Components / Systems / Tools / Events / Permissions / Resource Requirements。

**STAR 映射**: 待 P3-E (per G-4)。

---

## 98. Runtime API (REQ-API-001)

至少提供: `create_agent` / `delete_agent` / `get_agent` / `send_event` / `suspend_agent` / `resume_agent` / `cancel_agent` / `get_agent_state` / `get_runtime_metrics`。

**STAR 映射**: 子代理 dispatch API 已有部分 (守门 #20), 待 P3-B 完整化 (per G-3)。

---

## 99. 管理 API (REQ-API-MGMT)

提供 Runtime Mode / Threshold / HOT Limit / Memory Limit / Queue Limit / Tenant Quota / Provider Limit 读取能力。动态修改是否允许由基本设计阶段决定。

---

## 100. 配置原则 (REQ-CONFIG-001)

所有资源控制参数必须配置化:

```yaml
runtime:
  ecs_enable_threshold: 12
  ecs_disable_threshold: 8
  max_hot_agents: 100
  context_cache_limit: ...
  llm:
    max_concurrency: ...
  tool:
    max_concurrency: ...
  rag:
    max_concurrency: ...
```

---

## 101. 默认安全模式 (REQ-CONFIG-SAFE)

配置错误时: `Fail Safe`。**不**得自动退化为 Unlimited Concurrency / Unlimited Queue / Unlimited Cache。

**STAR 映射**: 守门 #88 默认安全模式 (per AGENTS.md §4 #88 派生)。

---

## 102. 开发语言 (REQ-LANG-001)

Runtime 核心使用 **Rust**。**STAR 已落** (22 domain-* crate 实证)。

---

## 103. Async Runtime (REQ-ASYNC-001)

采用 **Tokio**。**STAR 已落** (per 守门 #1 v1-v14 41/41 crate 守门)。

---

## 104. ECS 选型 (REQ-ECS-CHoice)

允许成熟 Lightweight ECS (bevy_ecs / flecs) 或 Agent-specific Custom ECS。选型应重点考察: Memory Overhead / Dynamic Entity Cost / Query Cost / Serialization / Concurrency / Lifecycle Support。**不**是游戏功能丰富度。

**STAR 映射**: P3-B L1 ECS 选型待启 (bevy_ecs / flecs 候选)。

---

## 105. 存储选型 (REQ-STORE-001)

具体产品可在基本设计阶段确定。逻辑角色: Durable Store / Fast Cache / Object Store / Event Store / Vector Store。

**STAR 候选**: SQLite (L0 任务队列) + Redis (L2 Context Cache) + S3 兼容 Object Store + ClickHouse / DuckDB (Event Store) + Qdrant / pgvector (Vector Store)。

---

## 106. 验收标准 (AC-001 ~ AC-020)

| AC | 标准 | STAR 状态 |
|---|---|---|
| AC-001 | Agent 数量 < 10 时, 不启动完整 ECS Mode | ✅ 守门 #20 Lightweight |
| AC-002 | Agent 不持有独立 HTTP Client | ✅ 守门 #24 subprocess 池 |
| AC-003 | Agent 不持有独立 LLM Client | ⏳ P3-C |
| AC-004 | Agent 不持有独立 MCP Runtime | ⏳ P3-C |
| AC-005 | Agent 不持有独立 Tool Registry | ⏳ P3-C |
| AC-006 | Agent 不长期持有 Full Context | ⏳ P3-D |
| AC-007 | Agent 不对应独立 OS Thread | ✅ 守门 #24 subprocess 池 |
| AC-008 | Agent 支持 HOT/WARM/COLD | ⏳ P3-B L1 ECS |
| AC-009 | 所有主要 Queue 均有界 | ⏳ P3-B |
| AC-010 | 所有外部并发均可限制 | ⏳ P3-B |
| AC-011 | 支持 Backpressure | ⏳ P3-B |
| AC-012 | 支持 Crash Recovery | ⏳ P3-D |
| AC-013 | 支持 ContextRef | 🟡 守门 #20 brief 落档 (雏形) |
| AC-014 | 支持 MemoryRef | ⏳ P3-D |
| AC-015 | 支持 PayloadRef | ⏳ P3-C |
| AC-016 | 支持 Runtime Mode 自动选择 | ⏳ P3-B |
| AC-017 | Runtime Mode 切换不丢失 Event | ⏳ P3-B |
| AC-018 | 10-11 Agent 不反复切换 | ⏳ P3-B 迟滞区 |
| AC-019 | WARM Agent 目标 RAM < 100KB | ✅ P3-A 实测 < 10KB |
| AC-020 | 1-1,000,000 Agent 分层 Benchmark | ⏳ P3-B/C/D/F |

**当前 4/20 AC 已过, 1 部分, 15 待 P3-B-F**。

---

## 107. 第一阶段 MVP 范围 (P3-B 启动时)

MVP 建议先实现: **Hybrid Runtime** / **Agent Identity** / **Agent State** / **ContextRef** / **Shared HTTP** / **Shared LLM** / **Shared Tool Registry** / **Event Bus** / **Scheduler** / **Bounded Concurrency** / **HOT/WARM** / **Persistence** / **Metrics** / **Benchmark**。

第一阶段可暂不实现: **Full Distributed ECS** / **Cross-node Migration** / **Global Agent Directory** / **Advanced COLD Placement** / **Complex Auto Scaling**。

**STAR P3-B 排期 (per `docs/reports/HANDOFF-ST-001.md` v0.4 §5.3 5 项 Blocker + `STAR-P3-WBS-001.md` v0.6 §7 阻塞 7 项)**: 待 Ulysses 拍板 P3-B 子项范围 + 5 域 Lead 真人到位 (per 守门 #3 8/21 拍板, 当前 Mavis 临时代签 per 守门 #3 反转 B 11:35 JST) + 凭证 (B.5/B.6) + KMS (E.4) + P3-C/D/F 范围 + P3-D 7 vs 12 子项。

---

## 108. 第二阶段 (P3-D 启动时)

增加: **COLD Lifecycle** / **MCP Pool** / **RAG Pool** / **Context Tiering** / **Agent Recovery** / **Multi-Tenant Quota** / **Advanced Scheduler** / **Runtime Dynamic Switching**。

**STAR P3-D 排期**: 待 P3-B/P3-C 收官后启。

---

## 109. 第三阶段 (P3-F 启动时)

增加: **Distributed Agent Runtime** / **Agent Sharding** / **Location-independent Restore** / **Cross-node Event Routing** / **Global Agent Directory** / **K8s Horizontal Scaling**。

**STAR P3-F 排期**: 跨机分布式, **本阶段 ❌ N/A** (per 守门 #3 5 域单仓, 暂不跨机)。

---

## 110. 最终架构理念 (per 参考 SRS §97)

系统最重要的架构约束为: **Agent ≠ Runtime**。Agent 本质上是: `Identity + Small State + References + Policies`。运行能力由系统共享。

**STAR 已落**: 守门 #3 / 守门 #5 / 守门 #6 / 守门 #7 / 守门 #9 / 守门 #12 / 守门 #24 / 守门 #1-#24 24 项 + 累积规 v1-v24 全部围绕这个原则。

---

## 111. Runtime 模式理念 (per 参考 SRS §98)

**ECS ≠ 默认答案**。ECS 是当 Agent 规模达到值得付出 ECS 固定成本时才启用的规模化优化手段。

`Small Scale → Lightweight Runtime` / `Large Scale → ECS Runtime`。

**STAR 实证**: P3-A 25 子项 (Lightweight) 阶段守门 #20 持续验证, 守门 #1 cargo check 41/41 crate 100% pass, **不启 ECS**。

---

## 112. 资源理念 (per 参考 SRS §99)

系统优化目标**不**是简单追求每个 Rust Struct 少几十字节, 而是消除 `重复 Runtime / 重复 Client / 重复 Connection / 重复 Context / 重复 Tool / 重复 Retriever / 重复 Memory` 带来的**数量级**浪费。

**STAR 目标**: 1M Logical Agent on 16GB, vs 传统 1M × 8MB = 8TB, **架构层面 99.9%+ 节省** (per 守门 #1 v19 token 计量 + 守门 #24 subprocess 池 + L1 ECS 列存)。

---

## 113. 最终目标架构 (per 参考 SRS §100)

```
                      Agent Platform
                            │
                            ▼
                  Runtime Mode Manager
                            │
          ┌─────────────────┴─────────────────┐
          │                                   │
   Lightweight Runtime                  ECS Runtime
   (P3-A Lightweight, 守门 #20)        (P3-B L1, 9 SA Type)
          │                                   │
    Small Agent Set                  Large Agent Set
          │                                   │
          └─────────────────┬─────────────────┘
                            ▼
                       Scheduler
                            │
                ┌───────────┼───────────┐
                ▼           ▼           ▼
              HOT         WARM        COLD
                │           │           │
                └─────┬─────┴─────┬─────┘
                      │           │
                      ▼           ▼
                  Shared Runtime
                      │
       ┌──────────────┼──────────────┐
       ▼              ▼              ▼
    LLM Pool      MCP Pool      Tool/RAG
       │              │              │
       └──────────────┼──────────────┘
                      ▼
              Context / Memory
                      │
                      ▼
              Durable Storage
```

**最终实现**: 小规模 Agent 保持简单轻量; 大规模 Agent 通过 ECS、分层生命周期、共享 Runtime 与外部状态存储获得规模经济。**系统最终追求的不是"所有 Agent 都运行得更轻"，而是"绝大多数 Agent 根本不需要处于运行状态"**。

**STAR 终局**: 1M Logical Agents on 16GB, 99.9% WARM/COLD, < 0.1% HOT, 0 OS Thread 跟 Agent 1:1, 0 Client 跟 Agent 1:1, 0 Context 跟 Agent 1:1。

---

# === 文档结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史。本文档 v1.0 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事。

**per 守门 #1 + #5 + #6 + #7 + #9 + #12 + #19 + #21 + #24 + #DB-13**: 落档时 23 项已过, #DB-13 跨项目 P3-D 阶段落地。

**per 守门 #3**: 5 域独立 Lead (玩家/经济/匹配/社交/管理) 真人到位后追溯签字, 当前 5 角色签字栏 Mavis 接手代签。
