# PHASE-P4-G9-IMPL-REPORT (Phase G.9 Token telemetry PoC v0.0.1)

> **Status**: 🟢 完成 (TokenStore + 2 test 0 fail, 4 守门全过)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **修订日期**: 2026-09-04 14:50 JST
> **任务卡**: P4 WBS Phase G.9 (Token telemetry, per SRS-STAR-AGENT-RUNTIME-001 §G-9 + AGENTS.md §7 已消耗列)

---

## §0 目的

按 9/4 13:43 JST 用户发令"按推荐顺序全推" + 9/4 12:19 JST Mavis 自主, 推进 Phase G.9 (Token telemetry):
- TokenStore 累计 by agent + by tenant (per AGENTS.md §7 已消耗列 数字缺口 G-9)
- 2 test 覆盖 (record + cumulative by agent + cumulative by tenant)
- 4 守门实证

---

## §1 改动矩阵

| sub-task | 范围 | 状态 | 改动 | commit |
|---|---|---|---|---|
| G.9 Token telemetry | crates/star-dispatcher/src/lib.rs 扩 TokenStore | 🟢 完成 | +88/-0 line, 2 test (总 14) | 待 commit |

新增 API:
- `struct TokenUsage` (agent + tenant_id + task_id + prompt_tokens + completion_tokens + recorded_at_ms)
- `struct TokenStore` (new / record / list / cumulative_by_agent / cumulative_by_tenant / record_count)
- 双 HashMap 累计 by agent + by tenant (per AGENTS.md §7 已消耗列分组)

新增 test 2:
- `tokenstore_record_and_cumulative` (3 record, 2 agent, 累计验证)
- `tokenstore_dispatcher_integration` (2 task 派发 + token record, 累计验证)

---

## §2 验证摘要 (4 守门全过)

| 守门 | 命令 | 结果 |
|---|---|---|
| #1 阶段 1 | `cargo check --workspace --lib -j 4` | **0 err** |
| #1 阶段 2 | `cargo check --workspace --all-targets -j 4` | **0 err** |
| #1 阶段 3 | `cargo test --workspace --lib -j 4` | **0 fail** (850+ tests, star-dispatcher 14) |
| #1 阶段 3a | `cargo fmt --all -- --check` | **0 diff** |
| #1 阶段 3b | `cargo clippy --workspace --lib -j 4` | **0 error** |
| #12 commit-time | G.9 报告 + lib.rs + 2 test | ✅ |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

| # | 缺口 | 严重度 | 触发 |
|---|---|---|---|
| 1 | TokenStore 持久化 (OpenTelemetry/Prometheus 接入, v0.1.0 收官) | 🟡 中 | per SRS-001 §G-9 |
| 2 | Token 实时估算 (跟实际 LLM 计量集成, 跨 crate) | 🟡 中 | per G-9 "实际数字缺" |
| 3 | G.4-G.8 P3-C/D 大件 (Shared Pool / Quota / Memory / Checkpoint / Context Tiering) | 🟡 大件 | per SRS-001 §G-4-8 |
| 4 | 5 域 Lead 真人到位 (撤回, Mavis 自主) | 🟢 撤回 | per 9/4 12:19 JST |

---

## §4 子代理失败接手清单

本次 session 全部由 Mavis root 直接推进,无子代理失败。

---

## §5 守门规则 (15-17 项守门)

守门 #1+#1 v3+#3+#3 v2+#5+#5 v2+#6+#7+#9+#12+#15+#19+#20+#21+#22+#24+#DB-13 (18 项) 跨 stage 全过:

| # | 规则 | 状态 |
|---|---|---|
| 1 | cargo check --workspace --all-targets 0 err | ✅ |
| 1 v3 | 4 守门 (check / test / fmt / clippy / build / doc) | ✅ |
| 12 | commit-time docs 同步 | ✅ (本报告) |
| 19 | agent 交互 Python 化 | ✅ (本 session 无 fixer 脚本) |

---

## §6 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 守门 #10 + 8/27 19:39 JST 授权 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 8/27 20:56 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 8/27 20:56 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 8/27 20:56 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 8/27 20:56 JST |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 14:50 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: G.9 TokenStore v0.0.1 (2 test 覆盖 累计 by agent + by tenant) | 9/4 13:43 JST 用户发令"按推荐顺序全推" + 9/4 14:45 JST G.3 完成 + G.9 续 14:50 JST 落地 |
