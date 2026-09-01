# STAR-BATCH-WBS-001: domain-batch v0 phase 1+2 详细拆 WBS + 跨 session HANDOFF 计划 v0.1

> **Status**: 🟡 Draft v0.1 (2026-09-01 19:43 JST Mavis 起草, 等 Ulysses review)
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 代签
> **触发**: per [BATCH-REQ-001 v0.1.2 业务需求](../requirements/batch-001.md) + [ADR-0040 commit aeaf213 架构决策](../architecture/2026-08-26-upgrade/adr/0040-domain-batch.md) + [domain-batch-spec v0.1](../specs/domain-batch-spec.md) + [commit a8fb5b6 v0 phase 1 骨架](https://github.com/UlyssesLeoLee/Star/commit/a8fb5b6) + 2026-09-01 19:43 JST Ulysses 拍板 next-wbs-detail-now
>
> **dual-use 警告 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**:
> 本 WBS 涉及的 domain-batch 是 DDD bounded context 第 23 个 crate, **不**映射 RGS 5 域业务子域 (player/economy/match/social/admin)。
> 5 域是 RGS 仓历史治理命名, Star 仓**不建立业务子域↔DDD 映射**; 5 域 DAG 视图隔离走 Master schema (per ADR-0040 §D36 + NFR-006)。

---

## §0 现状 (per 2026-09-01 19:43 JST 拍板时)

- ✅ **BATCH-REQ-001 v0.1.2** (需求 doc, review 草稿未 commit) — 32KB / 11 § / 4 拍板落地
- ✅ **ADR-0040 commit `aeaf213`** (架构决策) — 18.5KB / 7 § / 7 决策 D33-D39 + 9 GAP + D40 WBS 附录 9.0M/7.5 周
- ✅ **domain-batch-spec v0.1** (规格 doc, review 草稿未 commit) — 31.8KB / 16 § / 12 INV + 5 Port + 16 错误 + 11 事件 + T1~T13 估 0.94M
- ✅ **commit `a8fb5b6`** domain-batch crate v0 phase 1 骨架 (8 实体 3 分类 + 5 Port + 16 错误 + 11 事件 + 12 INV stub + 10 ID) — 8 files / 1744 insertions / 10/10 test pass / 守门 #1 v1+v2+v3 三过
- ⏳ v0 phase 2 实装 (T5~T9 + T11~T13, 估 ~0.48M, 跨 1-2 session)
- ⏳ v0 末期验证 (12 AC + 性能 benchmark + 守门 #1+#9+#12, 估 ~0.2M)
- ⏸ 1.0M 整体 (v0 phase 1+2 + 末期验证 ≈ 0.94M+0.2M = 1.14M)

## §1 v0 phase 1 详细拆 (4 子任务, ~0.4M token)

per [domain-batch-spec §9 T1~T4+T10](../specs/domain-batch-spec.md) + commit `a8fb5b6` 现状 (T1+T2+T3+T4+T10 骨架已落地, 等 v0 phase 1 末期验证 0.1M 收尾).

| # | 子任务 | 依赖 | token 估 | 软参考周 | 状态 |
|---|---|---|---|---|---|
| **WB-1.1** | T1 8 schema 实体 + T2 RLS + 索引 (已落地 per commit `a8fb5b6`) | — | 0.09M | 0.075 | ✅ Done |
| **WB-1.2** | T3 BatchCommandPort 12 方法 + T4 BatchQueryPort 9 方法 (trait stub 已落地, 实装 v0 phase 2) | WB-1.1 | 0.14M | 0.117 | 🟡 T3+T4 trait stub done; v0 phase 2 实装 |
| **WB-1.3** | T10 6 MCP tool 暴露 (per [ADR-0032 Streamable HTTP](../architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md)) | WB-1.2 | 0.03M | 0.025 | 🟡 v0 phase 2 实装 |
| **WB-1.4** | v0 phase 1 末期验证: 守门 #1+#9+#12 三过 + 单元测试覆盖率 ≥60% | WB-1.1~1.3 | 0.1M | 0.083 | 🟡 pending |

**v0 phase 1 累计**: ~0.4M / ~0.33 周 (4 子任务)

## §2 v0 phase 2 详细拆 (6 子任务, ~0.48M token)

per [domain-batch-spec §9 T5~T9+T11~T13](../specs/domain-batch-spec.md) + [ADR-0040 §D35 5 节点类型 + §D39 状态机](../architecture/2026-08-26-upgrade/adr/0040-domain-batch.md) + [ADR-0030 Lease + Heartbeat + Resume 复用](../architecture/2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md).

| # | 子任务 | 依赖 | token 估 | 软参考周 | 状态 |
|---|---|---|---|---|---|
| **WB-2.1** | T5 NodeExecutor trait + 5 runtime_kind 实现 (domain-service / mcp-tool / http / shell / sql, per ADR-0040 §D35) | WB-1.2 | 0.12M | 0.10 | 🟡 pending |
| **WB-2.2** | T6 DagOrchestrator trait + 拓扑排序 + 并行/串行 (per INV-BA-03 无环 + BA-006 错误码) | WB-2.1 | 0.08M | 0.067 | 🟡 pending |
| **WB-2.3** | T7 Scheduler trait + cron + 事件触发 + 手动 (per F-010~014) | WB-1.2 | 0.05M | 0.042 | 🟡 pending |
| **WB-2.4** | T8 状态机/重试/幂等/取消 (per F-020~026 + INV-BA-04/07 + ADR-0030 Lease 复用) | WB-2.1, WB-2.2 | 0.06M | 0.05 | 🟡 pending |
| **WB-2.5** | T9 11 类 Domain Event 发布 + 订阅 (per spec §5) + T11 告警 + SLA | WB-1.2, WB-2.4 | 0.08M | 0.067 | 🟡 pending |
| **WB-2.6** | T12 单元测试 + RLS + 拓扑校验 + 幂等 + 超时 + T13 集成测试 (DAG 跑通 + 跨 session 恢复 + 冷热分层) | WB-2.1~2.5 | 0.14M | 0.117 | 🟡 pending |

**v0 phase 2 累计**: ~0.48M / ~0.4 周 (6 子任务)

## §3 v0 末期验证 (2 子任务, ~0.2M token)

per [BATCH-REQ-001 §7 12 AC](../requirements/batch-001.md) + [domain-batch-spec §10 12 AC gherkin](../specs/domain-batch-spec.md) + 守门 #1+#9+#12 三过.

| # | 子任务 | 依赖 | token 估 | 软参考周 | 状态 |
|---|---|---|---|---|---|
| **WB-3.1** | 12 AC 验收 (gherkin 端到端) + cargo test --workspace --release --lib 100% pass + cargo check --workspace --all-targets 0 err | WB-2.6 | 0.12M | 0.10 | 🟡 pending |
| **WB-3.2** | 性能 benchmark (NFR-002 50 worker / 500 节点/秒, per 9/1 18:43 JST 拍板 B) + e2e 测试 + 文档同步 | WB-3.1 | 0.08M | 0.067 | 🟡 pending |

**v0 末期验证 累计**: ~0.2M / ~0.17 周 (2 子任务)

## §4 v0 全 phase 累计 (per ADR-0040 §D40)

| Phase | 子任务数 | token 估 | 软参考周 |
|---|---|---|---|
| v0 phase 1 | 4 | 0.4M | 0.33 周 |
| v0 phase 2 | 6 | 0.48M | 0.40 周 |
| v0 末期验证 | 2 | 0.2M | 0.17 周 |
| **v0 累计** | **12** | **1.08M** | **0.90 周** |
| v1 phase 1 (DAG 拖拽 + 模板市场 + CronJob 迁移) | TBD | ~2.5M | ~2.1 周 |
| v1 phase 2 (多集群/多云 + ML 编排) | TBD | ~1.5M | ~1.2 周 |
| **v0 + v1 累计** | TBD | **~5.08M** | **~4.2 周** |

> 整体 9.0M / 7.5 周 含 v0 + v1 + 余量, per [ADR-0040 §D40 WBS 附录](../architecture/2026-08-26-upgrade/adr/0040-domain-batch.md) + [9/1 18:43 JST Ulysses 拍板 C](../requirements/batch-001.md).
> v1 phase 1+2 详细拆待 v0 末期验证后启动, 走 9/1 18:43 JST 拍板 C 含 v1 拖拽承诺.

## §5 跨 session HANDOFF 计划

per [HANDOFF-ST-001 v0.4 H2-EXT 0.6-0.8M 单 session 上限实证](../../../HANDOFF-ST-001.md) (单 session 估上限 0.8M token), 1.08M 整体需跨 2-3 session 续做. 设计 4 个 HANDOFF 文档, 每个 session 收尾时落地.

| HANDOFF | session 范围 | token 估 | 累计 | 触发 |
|---|---|---|---|---|
| **HANDOFF-BATCH-001** | session 1: v0 phase 1 收尾 (WB-1.4 验证 0.1M) | 0.1M | 0.1M | 当前 session 收尾时落地 (per 19:43 JST 拍板) |
| **HANDOFF-BATCH-002** | session 2: v0 phase 2 启动 (WB-2.1 + WB-2.2 节点执行器 + DAG 编排) | 0.2M | 0.3M | session 1 收尾后下次 session 启动 |
| **HANDOFF-BATCH-003** | session 3: v0 phase 2 续 (WB-2.3 + WB-2.4 + WB-2.5 scheduler + 状态机 + 事件) | 0.19M | 0.49M | session 2 收尾后下次 session 启动 |
| **HANDOFF-BATCH-004** | session 4: v0 phase 2 末期 + v0 末期验证 (WB-2.6 + WB-3.1 + WB-3.2) | 0.34M | 0.83M | session 3 收尾后下次 session 启动 |

> **单 session 上限 (per HANDOFF-ST-001 v0.4 H2-EXT 0.6-0.8M 实证)**: 每次 session 内 token 估 0.1-0.34M, 安全在 0.6M 上限内. v0 末期跨 4 session 完成.
> **跨 session 风险 (per AGENTS.md §4.1 v18 H2 范围扩量触发)**: 实证 H2 0.3-0.5M 估 → 1.1-1.6M 实测 (3-5x 超支). v0 整体估 1.08M, 实际可能 3.2-5.4M, 需跨 4-7 session 续做. HANDOFF 计划保留扩展空间.

## §6 守门 #15 死循环饱和约束 (per AGENTS.md §4.1 v15)

- 当前 ahead 13 (per `git rev-list --count origin/main..HEAD`)
- 饱和点 113 (per AGENTS.md v0.15 5cfb7b3 实证)
- 距离饱和: 100 commits
- v0 phase 1 落地 1 commit (a8fb5b6) + v0 phase 2 估 6-8 commit + 末期验证估 2-3 commit = 9-12 commit
- 仍远低于 113 饱和点, 安全推进
- **守门 #12 commit-time 同步**: 每次 docs 同步 commit 必先有新事件触发 (代码改动 / Ulysses 拍板), 守门 #15 派生饱和约束不触发

## §7 风险 + 假设

### 7.1 风险 (per 守门 #12 实证 + HANDOFF-ST-001 v0.4 H2-EXT)

| # | 级别 | 风险 | 缓解 |
|---|---|---|---|
| R-WB-1 | **P1** | v0 phase 2 跨 session token 超支 (per H2 0.3-0.5M 估 → 1.1-1.6M 实测) | 4 session 续做, 每个 HANDOFF 锁 session token 预算, 超支即停 |
| R-WB-2 | **P2** | 5 节点类型实装 (T5) 涉及 33 domain 调用, 跨域 API 兼容性 | 走 v0 phase 1 现有 `star_context::ActorContext`, 不另起类型, 走 spec §D35 集成点 |
| R-WB-3 | **P2** | 状态机/重试/幂等 (T8) 涉及 ADR-0030 Lease 复用, 跨 crate 集成复杂 | 走 v0 phase 1 已 commit `star-context` + `star-saga` 已有基础设施, 不重写 |
| R-WB-4 | **P3** | 性能 benchmark (NFR-002 50 worker / 500 节点/秒) 实测可能不达 | per 9/1 18:43 JST 拍板 B 务实路径, 预留 100 worker 横向扩展空间 |
| R-WB-5 | **P3** | 12 AC gherkin 验收跨测试平台 (test-design v0.6 7 シナリオ) 集成复杂 | 走 v0 phase 1 已 commit `domain-automation` 6 scenarios 模板复用 |
| R-WB-6 | **P3** | v1 拖拽 + 模板市场 + CronJob 迁移 估 2.5M 跨 3-4 session, 跟 v0 跨 session 续并行可能冲突 | 守门 #15 死循环饱和约束, v0 末期后启动 v1 |

### 7.2 假设 (Assumption)

- **A-WB-1**: 当前 session token 余量 0.1M (估), 写 WBS-001 0.1M + HANDOFF-001 0.05M = 0.15M, 仍在单 session 安全范围
- **A-WB-2**: v0 phase 1 骨架 (commit a8fb5b6) 0 err + 10/10 test pass, v0 phase 2 实装无隐藏 crate 集成问题
- **A-WB-3**: 5 域 Lead 真人 + SRE Lead 真人 (per 9/1 18:43 JST 拍板 A 架构师代签) 到位后回填, 不阻塞 v0 phase 2 实装
- **A-WB-4**: v1 phase 1 启动时机 = v0 末期验证后, 9/1 18:43 JST 拍板 C 含 v1 拖拽承诺有效
- **A-WB-5**: HANDOFF-BATCH-001/002/003/004 文档创建后, 下次 session 续做时按 HANDOFF 任务清单执行, 不需重新对齐需求

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 12 子任务 (4 phase 1 + 6 phase 2 + 2 末期验证) + 4 HANDOFF session 续计划 + 守门 #15 饱和 + 6 风险 + 5 假设 + 1.08M/0.9 周 整体估 | 2026-09-01 19:43 JST Ulysses 拍板 next-wbs-detail-now + ADR-0040 commit aeaf213 + crate commit a8fb5b6 |

---

> **下一步 (HANDOFF-BATCH-001 触发)**: 写 [HANDOFF-BATCH-001.md](./HANDOFF-001.md) (session 1 收尾任务清单, 估 ~0.05M) → 收尾当前 session
