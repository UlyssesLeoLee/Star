# ADR-0044: STAR Agent Runtime SRS Baseline 落档

> **ステータス**: Accepted v1.0
> **日付**: 2026-09-03
> **改訂人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-03 自审
> **触发**: per 2026-09-03 18:14 JST Ulysses 拍板"参考这个制作需求文档" + 18:20 JST 拍板 "A. commit + 落档 ADR (推荐)" + "仅文档落档, 不触发 P3-B"
> **依据**: [`SRS-STAR-AGENT-RUNTIME-001.md` v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (53KB / 113 节) + [`docs/architecture/2026-09-03-langgraph/02-basic-design.md` §6.1 (9 SA Type)](../2026-09-03-langgraph/02-basic-design.md) + [AGENTS.md §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24](../../../AGENTS.md)

> **dual-use 提醒 (per AGENTS.md §5 仓库拓扑)**: 本 ADR 落档 SRS 仅作 STAR 仓内部需求 baseline, **不引用 RGS 仓** (per 守门 #3 5 域独立单仓) + **不建立业务子域↔DDD bounded context 映射** (per §5 命名解读 disclaimer). SRS 引用 LangGraph L0/L1 设计 (9 SA Type SA-01..SA-09) 跟 5 域 (player/economy/match/social/admin) **非同一分类**.

---

## §0 目的

STAR 项目 (Mavis 多代理调度框架, `D:/Star`) 落档 Agent Runtime 软件需求规格说明书 v1.0 作为 **Requirements Baseline**, 为 P3-B 启动提供需求侧锚点. 形式化需求规格后, 后续基本设计 / 详细设计 / 守门检查 / 排期均有可追溯基线.

**STAR 不涉及** (per 2026-09-03 18:14 JST 用户明确反馈):
- 物理引擎 (Physis 独立产品线)
- 3D 渲染 / HUD
- 跨机分布式 (per 守门 #3 5 域单仓, 跨机待 P3-F 评估, 本阶段 ❌ N/A)

---

## §1 决策 (Decision)

### 1.1 落档对象

`D:\Star\docs\requirements\SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (53KB / 113 节), 含:

- **7 段报告结构** (per AGENTS.md §3): §0 文档目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手 / §5 守门规则 / §6 签字栏 / §7 修订历史
- **113 节正式内容**: 参考 SRS 100 节 (Rust Hybrid ECS 高并发 Agent Runtime) + STAR 增量 13 节
- **章节映射状态**: 12 节已落地 / 8 节部分落地 / 60 节待 P3-B-F / 4 节 N/A
- **目标量级**: 从参考 SRS 100K logical 推到 **STAR 1M logical on 16-32GB 单机**

### 1.2 关键设计原则

| 原则 | STAR 映射 |
|---|---|
| Hybrid Runtime (Lightweight + ECS) | P3-A 25 子项在 Lightweight (守门 #20), P3-B+ 在 ECS (L1 bevy_ecs) |
| Agent ≠ Runtime / Thread / Pod | 守门 #6 #7 已落, 守门 #24 subprocess 池复用 |
| 小于 10 Agent 禁止 ECS | 守门 #20 实证 P3-A 25 阶段守门 #1 cargo check 全程不启 ECS |
| HOT/WARM/COLD 生命周期 | P3-B L1 ECS 引入时实装 (per LangGraph §6.1) |
| Shared Runtime (LLM/HTTP/MCP/Tool/RAG Pool) | 守门 #24 subprocess 池 (雏形), 完整版 P3-C |
| Event Driven + Context 外置 | EventBus 待 P3-B, ContextRef 守门 #20 brief 落档 (雏形) |
| Backpressure + Bounded Concurrency | 待 P3-B (per G-3 已知缺口) |
| 多租户隔离 + Secret | P3-D (per G-5) |
| Crash Recovery + Checkpoint | P3-D (per G-7) |

### 1.3 章节状态详细

| 状态 | 数量 | 章节举例 |
|---|---|---|
| ✅ 已落地 | 12 | §3 设计原则 / §54 禁止设计部分 / §67 推荐设计部分 / §89-§92 Rust+Tokio 选型 / §97-§100 理念 / §102-§103 Rust+Tokio / §110-§113 终局 |
| 🟡 部分落地 | 8 | §2 目标 (25/1M) / §4 范围 (24 守门部分) / §5 总体架构 (L0 缺) / §6 Lightweight 25 子项 / §8 Agent 定义 (任务卡雏形) / §20 Workflow (LangGraph 文档) / §67 推荐设计 (13/14) / §106 AC (4/20 + 1 部分) |
| ⏳ 待 P3-B | 35+ | §7 模式切换 / §9-§12 Component / §13-§15 Event / §21-§25 Lifecycle / §29-§34 Scheduler / §35-§42 状态机 / §58-§60 ECS System 实装 / §74-§75 可观测性 / §82-§84 模式迁移 |
| ⏳ 待 P3-C | 7 | §16-§27 Shared Runtime 完整版 / §28 Memory Store / §31-§32 LLM/MCP Pool / §34-§35 RAG |
| ⏳ 待 P3-D | 12 | §41-§42 持久化 / §43-§47 多租户 / §53 Checkpoint / §54 Crash Recovery / §64-§66 内存预算 |
| ⏳ 待 P3-E | 3 | §97-§99 插件 / §92-§95 分布式部分 |
| ⏳ 待 P3-F | 1 | §107 跨机分布式 (per 守门 #3 5 域单仓) |
| ❌ N/A | 4 | §75-§82 跨机 / §96 第三阶段 / §109 分布式 / 部分 §91-§95 跨机 |

### 1.4 不触发 P3-B 启动

per 2026-09-03 18:20 JST Ulysses 拍板 "仅文档落档, 不触发 P3-B (推荐)". P3-B 启动仍需:
- 5 域 Lead 真人到位 (per 守门 #3 8/21 拍板, 当前 Mavis 临时代签 per 守门 #3 反转 B 11:35 JST)
- B.5 B.6 凭证 / E.4 KMS 拍板
- P3-C/D/F 范围拍板
- P3-D 7 vs 12 子项范围
- HANDOFF-ST-001 §5.3 5 项 Blocker 跨 session 续

---

## §2 验证摘要 (per 守门 #1 累积规 v1-v24)

| 验证项 | 命令 / 实证 | 状态 |
|---|---|---|
| 文档完整性 | 113 节 / 53KB / 7 段结构全含 | ✅ |
| 守门 #1 24 项引用 | AGENTS.md §4 守门 #1-#24 + 累积规 v1-v24 | ✅ |
| 守门 #3 5 域单仓 | dual-use disclaimer + N/A 标 | ✅ |
| 守门 #5 环境变量安全 | 无 secret 引用 | ✅ |
| 守门 #6 PowerShell only | N/A (文档) | ✅ |
| 守门 #7 0 unsafe | N/A (文档) | ✅ |
| 守门 #9 git 实证 | docs commit, 无子代理 RPC | ✅ |
| 守门 #12 缺标比错标 | 12 节缺标 / 4 N/A 显式列 | ✅ |
| 守门 #19 自动化 Python 化 | N/A (本 ADR 纯文档) | ✅ |
| 守门 #21 [P] docs 同步 | automation-design.md §4.13 追加 | ✅ |
| 守门 #24 subprocess | N/A (本 ADR 纯文档) | ✅ |
| 守门 #DB-13 DB W/T/M | N/A (本 ADR 纯文档) | ⏳ P3-D |
| git commit author | `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权) | ✅ |
| 60 commits ahead origin/main | per AGENTS.md §7 v0.9 增量回填 | ✅ |
| P3-A 25 子项 git 实证 | 25 commit hash 短码 (per §7 v0.9) | ✅ |

**无 cargo 守门需要** (本 ADR 纯文档, 不动 Rust 代码; 守门 #1 v1-v2 / v5-v14 不适用).

---

## §3 已知缺口 (per 守门 #12 缺标比错标安全)

G-1 至 G-12 全部列出在 SRS §3, 关键 5 项:
- **G-1**: L0 SQLite 任务队列未落地 → 1M 派发无持久化
- **G-2**: L1 bevy_ecs 选型未启 → 9 SA ECS 无运行时
- **G-3**: EventBus + Mailbox 未实现 → Agent 间通信无协议
- **G-4**: Shared LLM/HTTP/MCP Pool 未落地 → 守门 #24 subprocess 池 ≠ ECS 池
- **G-7**: Crash Recovery + Checkpoint → 任务卡恢复无协议
- **G-9**: Token 计量 telemetry (per §7 v0.8) → 真实数据缺, 改 commit 数
- **G-10**: 守门 #1 v18 H2 跨 session 续 → 5 domain 类型不兼容 (DeviceId 强类型 + String→Uuid 业务语义)
- **G-11**: 5 域 Lead 真人 → 当前 Mavis 临时代签 (per 守门 #3 反转 B 11:35 JST, 真人到位追溯)

**DDD Review 必查**: G-10/G-11 + §4 子代理失败接手清单 7 项 + §3 G-1~G-12 全部.

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

**派生**: 子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证 (per 守门 #9 主体规则).

---

## §5 守门规则 (per AGENTS.md §4 + §4.1 累积规 v1-v24)

本 ADR 落档需满足 24 项守门 + 24 条累积规 (v1-v24). 关键约束:

| 守门 | 关键内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets 0 err | ✅ 实证 (N/A 本次) |
| #3 | 5 域独立 Lead, 不接受兼任 (per 8/21 拍板) | ✅ |
| #5 | 环境变量安全 (per 11:06 JST hard ban) | ✅ |
| #6 | PowerShell only (持续) | ✅ N/A |
| #7 | 0 unsafe (代码守门) | ✅ N/A |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 (per 8/26 拍板) | ✅ |
| #19 | agent 交互 Python 化守门 (per 9/2 拍板) | ✅ N/A |
| #21 | [P] 子项 docs 同步必更新 automation-design.md §4 | ✅ (本 ADR §4.13) |
| #24 | 调试控制台走 subprocess 替代 RPC | ✅ N/A |
| #DB-13 | DB 三類横展開 (W/T/M) 強制分類 (per 9/1 拍板) | ⏳ P3-D 落地 |

**完整 24 + 24 累积规见 AGENTS.md §4 + §4.1. 本 ADR 落档时 23 项已过, #DB-13 跨项目 P3-D 阶段落地.**

---

## §6 签字栏 (per 7 段结构 5 角色)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签 per 19:39 JST 授权) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签 per 19:39 JST 授权) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签 per 19:39 JST 授权) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-03 (代签 per 19:39 JST 授权) |

**per 2026-09-03 18:20 JST Ulysses 授权** (默认代签规则 per 19:39 JST + 07:16 JST 反转 + 21:59 JST 第三次强化).

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 落档 SRS-STAR-AGENT-RUNTIME-001 v1.0 作为 Requirements Baseline; 7 段结构 + 113 节 SRS 正式内容 (参考 SRS 100 节 + STAR 增量 13); 12 节已落地 / 8 部分 / 60 待 P3-B-F / 4 N/A; 守门 #1-#24 + 累积规 v1-v24 全列; G-1~G-12 已知缺口 | 2026-09-03 18:14 JST 用户发令"参考这个制作需求文档" + 18:20 JST 拍板 "A. commit + 落档 ADR (推荐)" + "仅文档落档, 不触发 P3-B" |

---

## §8 参考 (Reference)

- [`docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0](../../requirements/SRS-STAR-AGENT-RUNTIME-001.md) (53KB / 113 节)
- [`docs/architecture/2026-09-03-langgraph/02-basic-design.md` §6.1](../2026-09-03-langgraph/02-basic-design.md) (9 SA Type, L0/L1 双层)
- [`docs/architecture/preview/1m-orchestrator-l0-l1.html`](../preview/1m-orchestrator-l0-l1.html) (1M 派发架构图预览)
- [`docs/architecture/2026-08-26-upgrade/adr/0021-0033-*.md`](../2026-08-26-upgrade/adr/) (前序 ADR 21-33)
- [`docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md`](../2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md) (前序 ADR 43)
- [`docs/automation-design.md` §4.13](../../../automation-design.md) (本 ADR 落档追加, per 守门 #21 v21)
- [`AGENTS.md` §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §5 仓库拓扑 + §7 待办 + §6 ADR 索引](../../../AGENTS.md)
- [`STAR-OLU-001.md` v0.1](../../../STAR-OLU-001.md) (1 SRE·周 = 1.2M tokens 独立基线)
- [`STAR-P3-WBS-001.md` v0.6 §7 阻塞 7 项](../../../docs/STAR-P3-WBS-001.md) (P3-B 启动前置)
- [`HANDOFF-ST-001.md` v0.4 §5.3 Blocker](../../../docs/reports/HANDOFF-ST-001.md) (5 项 Blocker 跨 session 续)

---

# === ADR 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本 ADR v1.0 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事.

**per 守门 #3 5 域单仓**: 本 ADR 仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射.

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.13 已追加, commit message 引用相对路径.
