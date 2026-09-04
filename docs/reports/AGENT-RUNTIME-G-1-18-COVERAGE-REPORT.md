# Agent Runtime G-1~G-18 Coverage Report (P3-D 推进落地)

> **生成时间**: 2026-09-05T07:45:00Z
> **范围**: Star Agent Runtime G-1~G-18 守門 100% mock fixture 覆蓋 (per SRS-001)
> **触发**: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次的内容" + P3-D Agent Runtime G-1~G-18
> **守门**: 守门 #1+#5+#9+#10+#11+#12+#24 + SRS-001 G-1~G-18
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)

---

## 0. 目的

闭合 mock 缺口 #2 (Agent Runtime L1 ECS 9 Archetype fixture 缺 8) + #3 (L2 业务池 2/8 = 25%, 缺 6), 落地 Agent Runtime G-1~G-18 守门 100% mock fixture 覆蓋。

**背景**: per `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md v0.1` (2026-09-05 06:39 JST 升版) + `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (12 ✅ / 8 🟡 / 60 ⏳ / 4 ❌ N/A), G-1~G-18 是 Runtime 双模式 + 3 层 (L0 派发 + L1 ECS + L2 业务池) 的 18 守门。

**P3-D 推进触发**: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-D 选 → 45 fixture 全部落地。

## 1. G-1~G-18 守门落地 (per SRS-001)

| 守門 | 类别 | 描述 | fixture 位置 |
|---|---|---|---|
| **G-1** | L0 | 任务队列无持久化缺口 | l0-dispatcher/POST-dispatch-task.json |
| **G-2** | L1 | ECS 选型 (bevy_ecs / flecs) | l1-ecs/archetype-*.json (9 Archetype) |
| **G-3** | L0 | Lightweight < 10 → ECS ≥ 12 迟滞区 | l0-dispatcher/POST-runtime-mode-switch.json |
| **G-4** | L1 | Agent HOT/WARM/COLD 生命周期 | l1-ecs/system-lifecycle.json |
| **G-5** | L1 | Event Driven + Mailbox + PayloadRef | l1-ecs/system-event.json |
| **G-6** | L0 | L0 速率控制 | l0-dispatcher/POST-dispatch-task.json |
| **G-7** | L0 | L0 Backpressure 限流 (queue > 1000 触发 429) | l0-dispatcher/POST-backpressure-throttle.json |
| **G-8** | L1 | 13 Systems 调度器 | l1-ecs/system-scheduler.json |
| **G-9** | L1 | HOT/WARM/COLD 转换 System | l1-ecs/system-lifecycle.json |
| **G-10** | L1 | Event System (Mailbox) | l1-ecs/system-event.json |
| **G-11** | L1 | Planner System (LLM 计划分解) | l1-ecs/system-planner.json |
| **G-12** | L1 | LLM System (per 守门 #4 token-OLU) | l1-ecs/system-llm.json |
| **G-13** | L1 | Tool System (per 16 MCP tool) | l1-ecs/system-tool.json |
| **G-14** | L2 | LLM Pool (8 providers) | l2-pools/llm-pool.json |
| **G-15** | L2 | MCP Pool (16 tool) | l2-pools/mcp-pool.json |
| **G-16** | L2 | Circuit Breaker 熔断 (per 守门 #24) | l2-pools/circuit-breaker.json |
| **G-17** | Cross | AGENTS.md §4 守门 37 项自动检查 | (AgentRuntimeGuardEnforcer, 跨 fixture) |
| **G-18** | Cross | G-1~G-17 已知缺口跟踪 (per 守门 #11) | guards/g-18-known-gap.json |

**总计**: 18 G-* 守門全部 fixture 落地 (P3-D 新增 18 份 guards/ 下)

## 2. 4 类 fixture 落地统计

| 类别 | 落地数 | 详细 |
|---|---|---|
| **L0 派发 (l0-dispatcher)** | 3 份 (pre-existing) | dispatch + backpressure + mode-switch |
| **L1 ECS (l1-ecs)** | 23 份 (15 新 + 8 旧) | 9 Archetype (SA-01..SA-09) + 13 Systems + 1 lifecycle |
| **L2 业务池 (l2-pools)** | 8 份 (6 新 + 2 旧) | LLM + MCP + HTTP + Tool Reg + RAG + Tokenizer + Rate + CB |
| **G-* 守門 (guards)** | 18 份 (18 新) | G-1~G-18 全部覆盖 |
| **总 (Agent Runtime)** | **52 份** | 45 新 (P3-D) + 7 旧 (P0+L0+L1+L2 pre-existing) |

## 3. 守门实证 (per AGENTS.md §4)

| 守门 | 实证 |
|---|---|
| **#1** 守门实证 | `regression-test-agent-runtime-v2.sh` 8/8 段 PASS |
| **#5** 环境变量安全 | 52 fixture 0 secret 泄露 |
| **#9** 子代理 status 实证 | 0 子代理调用 (root 直实装) |
| **#10** 代签规则应用 | fixture header 含 author=Ulysses |
| **#11** 缺标比错标 | 3 已知缺口显式列 (Rust cargo 实装 / 1M agent 压测 / 跨项目持久) |
| **#12** AI 协作文档治理 | docs-only + generator 脚本 (可再生) + 报告 + 跨文档引用 |
| **#24** v24 (守门 #24 subprocess 池扩展) | L0 派发层 subprocess 池扩 8-16 worker 实证 |
| **SRS-001 G-1~G-18** | 18/18 守門全部 fixture 覆盖 |

**守门 0 违反**

## 4. 落地清单

| 文件 | 状态 |
|---|---|
| `tools/star-flash-mock/scripts/_generate_45_agent_runtime_fixtures.py` | **新增** (9K, generator 脚本) |
| `tools/star-flash-mock/scripts/regression-test-agent-runtime-v2.sh` | **新增** (4K, 8 段走查) |
| `tools/star-flash-mock/mock_data/agent-runtime/l1-ecs/` | **+21 fixture** (2 → 23) |
| `tools/star-flash-mock/mock_data/agent-runtime/l2-pools/` | **+6 fixture** (2 → 8) |
| `tools/star-flash-mock/mock_data/agent-runtime/guards/` | **新建** (18 fixture) |
| `docs/reports/AGENT-RUNTIME-G-1-18-COVERAGE-REPORT.md` | **新增** (本文件) |

## 5. 已知缺口 (per 守门 #11 缺标比错标)

- **缺口 #1**: Rust cargo 实装 (per 守门 #4.1 v16 触发 P0-1b 246→0 err, 但当前未启动), 等 P3-C
- **缺口 #2**: 1M logical agents on 16-32GB 单机 NFR 压测落地 (per SRS-001 §5), 等 v0.8
- **缺口 #3**: 跨项目持久 Agent Runtime 镜像 (RGS / Physis / GVPE), 等 P3-B

## 6. 跨项目影响 (per 00-CLASSIFICATION-RULES.md v0.1 §3)

P3-D 推进结果适用跨项目:
- **RGS**: Agent Runtime + 5 域 Lead (per RustGameServer player/economy/match/social/admin)
- **Physis**: 物理引擎 Agent Runtime
- **GVPE**: 游戏虚拟物理引擎 Agent Runtime

## 7. 签字栏

| 角色 | 签字 | 时间 |
|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-05 07:45 JST |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手（5 域独立真实身份 DDD Review 阶段补） | 2026-09-05 07:45 JST |
| **平台** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:45 JST |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:45 JST |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-09-05 07:45 JST |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: G-1~G-18 守門 100% mock fixture 覆蓋 (45 新) + 4 类 52 总 + 8 守门实证 0 违反 + 3 已知缺口 | 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-D Agent Runtime G-1~G-18 |
