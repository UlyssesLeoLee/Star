# ADR-0037: Phase H 22 domain 真实数据接入 + Saga 测试框架 + 缓存优化

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **审批**：架构师（Mavis 接手 agent per DEC-008）
> **触发**：per ADR-0036 §8.2 Phase H 方向（22 domain 真实接入 + Saga 测试 + 性能基线）/ 2026-08-27 21:59 JST 用户授权第三次强化
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../../plan/2026-08-26-upgrade-plan.md)
> **依赖**：[ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) · [ADR-0036 Phase G Architecture](0036-phase-g-architecture.md) · [AGENTS.md §0 一句话硬约束](../../../../AGENTS.md)
> **关联**：[spec/agents/02-data-sources-spec.md §1 22 domain URI 模式](../spec/agents/02-data-sources-spec.md) · [spec/cache/01-cache-contract-spec.md §4 22 domain scope 表](../spec/cache/01-cache-contract-spec.md) · [spec/saga/01-saga-coordination-spec.md §1 8 步状态机](../spec/saga/01-saga-coordination-spec.md) · [spec/mcp/02-resources-prompts-spec.md §1 Resources 协议](../spec/mcp/02-resources-prompts-spec.md) · [arch/06-threat-model-nfr.md §3 NFR 性能预算](../arch/06-threat-model-nfr.md)

---

## 0. 一句话硬约束
> **可以代签 Ulysses，不可以编造历史。**
> — per AGENTS.md §0 + 2026-08-27 19:39 JST 用户授权升级 + 21:59 JST 第三次强化
> + 2026-08-21 JST 5 域独立 Lead 拒绝兼任硬约束（per Ulysses RGS 5 域拍板）

---

## 1. 背景

### 1.1 Phase G 已交付（per 2026-08-27 22:30-22:55 JST + base commit 6eb3cb5）

| 阶段 | 交付 | 关键 commit | 引用 |
|---|---|---|---|
| Phase G ADR | [adr/0036-phase-g-architecture.md](0036-phase-g-architecture.md) (350 行) | `863b69b` (per `git log -p --follow`) | ADR-0036 §1 + §2 |
| Phase G spec/cache/01 | 1 份 spec 262 行 = 22 domain + 3 应用 crate 缓存契约 | `9a7c7d7` (feat/cache) | spec/cache/01 |
| Phase G spec/saga/01 | 1 份 spec 231 行 = 5 域 Lead Saga 协调 + Q-003 流程 | `dd31f2b` (feat/saga) | spec/saga/01 |
| Phase G star-cache 实装 | 7 文件 372 行 + 7 测试 = InMemory + Redis stub | `79a9643` (feat(cache)) | ADR-0036 §2 D13 |
| Phase G star-saga 实装 | 7 文件 154 行 + 3 测试 = Saga orchestrator + Step + 补偿 | `addc955` (feat(saga)) | ADR-0036 §2 D14 |
| Phase G workspace 测试 | 465 tests pass | per `cargo test --workspace` 2026-08-27 22:55 JST | ADR-0036 §1.1 |

**关键 Phase G 决定**（per [ADR-0036 §2 D11-D15 L60-159](0036-phase-g-architecture.md)）：
- D11 spec/cache/01 落地 22 domain + 3 应用 crate 缓存契约 + 5 维度键 schema + 3 档 TTL
- D12 spec/saga/01 落地 5 域 Lead Saga 协调 + 8 步状态机 + forward/backward/hybrid 补偿
- D13 star-cache 实装 Cache trait + InMemory + Redis stub + 7 测试
- D14 star-saga 实装 Orchestrator + Step + Compensation + 3 测试
- D15 性能预算 NFR 收敛（4 指标 P50/P95/P99 实测基线 + Phase G+ 收敛 SLO）

**Phase G 已知缺口遗留**（per [ADR-0036 §7 L262-279](0036-phase-g-architecture.md) 12 项）：
- #1 Redis Cluster stub 待 Phase G+ 实装
- #2 Cache warming 启动预热未设计
- #3 Saga 嵌套（sub-saga）未设计 → Phase H 主战场
- #4 Saga 版本管理演进未设计
- #5 22 domain 接入优先级排期（per spec/agents/02 §1 22 domain URI 表）→ Phase H 主战场
- #6 5 域 Lead 真实身份签字（DDD Review 阶段补）
- #7 Q-003 Economy Lead 决策 SLA 量化
- #8 Phase H 性能预算基线 → Phase H 主战场
- #9 spec/services/07 计划是否成立
- #10 5 业务域 Step trait 协作流程
- #11 star-cache 内存上限 LRU 10000 条是否够
- #12 star-saga 死信队列持久化层

### 1.2 Phase H 范围

Phase H 在 Phase G 基础上接 **3 大能力域**（per ADR-0036 §8.2 L293-298 续）：

1. **22 domain 真实数据接入**（mock → 真实最后一公里）：3 非核心 domain crate（collaboration/comment/board）+ 5 业务域 Step trait 全部实装（per ADR-0036 §7 #5）
2. **Saga 测试框架**（time-travel + chaos + property-based 三件套）：解决 [ADR-0036 §7 #3 Saga 嵌套](0036-phase-g-architecture.md) + #4 Saga 版本管理 + #10 5 业务域协作流程
3. **缓存优化 + 性能基线**（per D15 + ADR-0036 §7 #1 + #2 + #11）：InMemory 调优 + Cache warming + LRU 上限重新评估 + 6 指标 P50/P95/P99/error rate 完整基线

> 关键边界：本 ADR §2 D16-D20 全部基于 8/28 07:35 JST 派工窗口"Phase H 22 domain 真实数据接入 + Saga 测试框架 + 缓存优化"任务派发；不沿用 bc23d6c 任何叙事（per AGENTS.md §4 #8 守门）；不引用未发生 commit。

---

## 2. 决策（5 项 D16-D20）

### D16. 新增 `spec/integration/01-22-domain-integration-spec.md` — 6 Tier 接入顺序 + 5 验收 + 7 已知缺口

**理由**：
- Phase G 22 domain crate 已 stub（per §1.1 + [ADR-0036 §1.1 L22-32](0036-phase-g-architecture.md)），但 star-mcp 16 tool 仍 mock（per [AGENTS.md §7 #4](../../../../AGENTS.md) "16 tool 真实数据源接入（现 mock）"）
- spec/agents/02 §1 L16-38 已列 22 domain URI 表，但 Tier 分类（核心 / 重要 / 边缘 / 业务域 / 实验性 / 离线）未排
- 5 业务域（Player / Economy / Match / Social / Admin）Step trait 待 Phase H 全部实装（per [ADR-0036 §7 #5 L272](0036-phase-g-architecture.md)）
- 6 Tier 顺序决定 D17 Saga 测试用例的覆盖范围 + D19 性能基线优先级

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/integration/01-22-domain-integration-spec.md`
- 章节：
  - §1 22 domain Tier 分类（6 Tier：T1 核心 5 + T2 重要 6 + T3 业务域 5 + T4 边缘 3 + T5 实验性 2 + T6 离线 1 = 22）
  - §2 接入顺序（per §1 6 Tier，T1 → T2 → T3 → T4 → T5 → T6）
  - §3 5 验收（per 5 域 SRE NFR：read 命中率 ≥ 60% + write-through 100% + cache invalid < 100ms + cold start < 5s + error rate < 0.1%）
  - §4 5 业务域 Step trait 实装（Player 玩家状态 + Economy 配额扣减 + Match 对局状态 + Social 好友关系 + Admin COC 审计）
  - §5 跨 spec 引用（spec/agents/02 §1 URI + spec/cache/01 §4 scope + spec/saga/01 §1 8 步 + spec/mcp/02 §1 Resources）
  - §6 已知缺口（per §7 跨表 + 7 项 + Phase H+ 真实外部服务接入）

### D17. 新增 `spec/saga/02-test-framework-spec.md` — time-travel + chaos + property-based 三件套

**理由**：
- [ADR-0036 §7 #3 L270](0036-phase-g-architecture.md) Saga 嵌套（sub-saga）未设计 → Phase H 主战场
- [ADR-0036 §7 #4 L271](0036-phase-g-architecture.md) Saga 版本管理演进未设计 → time-travel test 验证 forward compat
- [ADR-0036 §7 #10 L277](0036-phase-g-architecture.md) 5 业务域 Step trait 协作流程 governance 缺失 → chaos test 模拟 5 域 Lead 拒绝响应
- star-saga 现有 3 测试（per §1.1 addc955）仅覆盖正向 8 步状态机，缺失败恢复 / 嵌套 / 版本演进测试
- property-based test 防止 Step 状态机回归（per spec/flows/06-error-recovery.md §3 错误码一致）

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/saga/02-test-framework-spec.md`
- 章节：
  - §1 time-travel debug（saga 状态快照 + 任意时间点回放 + 决策点暂停/恢复）
  - §2 chaos test（5 域 Lead 拒绝响应 + 网络分区 + 时钟偏移 + 节点故障）
  - §3 property-based test（proptest 框架：Step 状态机不变量 + 补偿幂等性 + 嵌套 sub-saga 顺序）
  - §4 5 业务域联合评审 governance（per ADR-0036 §7 #10，5 域 Lead 联合评审 Saga 场景变更的 PR 模板）
  - §5 与 crates/star-saga 实装映射（3 测试 → 扩展 12+ 测试 = time-travel 4 + chaos 5 + property-based 3）
  - §6 已知缺口（per §7 跨表 + 4 项 + chaos tool 选型 + property-based 终止条件）

### D18. `crates/star-mcp/src/handlers/` 新增 22 domain handler（Phase H 真实数据接入框架）

**理由**：
- AGENTS.md §7 #4 "16 tool 真实数据源接入" P2 待办 = Phase H 主战场
- star-mcp 当前 16 tool handler（per [ADR-0034 §2 D3 L82-100](0034-phase-e-architecture.md)）走 mock 数据源
- Phase G D13 star-cache + D14 star-saga 已就位（per §1.1），Phase H 只缺 handler ↔ 22 domain crate ↔ 真实数据源三层穿透
- 22 handler × 5 测试（read / write / list / invalidate / error path）= 110 测试（per [AGENTS.md §7 #4](../../../../AGENTS.md) "16 tool 真实数据源接入（现 mock）"）

**形式**：
- 路径：`crates/star-mcp/src/handlers/`
- 文件结构（22 domain handler × 3 模式）：
  - `mod.rs`（~80 行，22 handler trait 抽象 + 5 模式 trait：`read/write/list/invalidate/error`）
  - 22 handler 文件（`{domain}_handler.rs`，每文件 ~100-150 行 = trait 实现 + 缓存接入 + Saga 触发点 + 错误码映射 per spec/mcp/03）
  - 22 handler 测试（`tests/{domain}_handler.rs`，每文件 ~80-120 行 = 5 模式 × 16-24 测试）
  - `integration.rs`（~150 行，22 handler 跨域 Saga 集成测试 = D16 §4 5 业务域 Step trait 协同）
- handler 三层穿透（per §3 关系表）：handler → star-cache（read 命中 + write-through）→ 22 domain crate → 真实数据源

### D19. `bench/perf-baseline.md` + `scripts/bench-runner.sh` — 6 指标 P50/P95/P99/error rate

**理由**：
- [ADR-0036 §2 D15 L140-159](0036-phase-g-architecture.md) 4 指标基线仅 Phase G 实测（无 error rate + 无 cold start）
- Phase H 需 6 指标完整基线 = Phase G 4 指标 + cold start + error rate
- [ADR-0036 §7 #8 L275](0036-phase-g-architecture.md) "Phase H 性能预算基线" 显式列待办
- 0 unsafe + 0 新外部依赖（除 wiremock-rs Phase D.5+ 例外 + criterion-rs 性能测试库）继续生效
- bench-runner.sh 跨平台（per 当前系统 win32）需 PowerShell 兼容（per [AGENTS.md §4 #6](../../../../AGENTS.md) "PowerShell only"）

**形式**：
- `bench/perf-baseline.md`（~250 行，6 指标定义 + 实测方法 + 报告模板）：
  - 6 指标 = 22 domain Resources P50/P95/P99 + 4 Git Provider OAuth P99 + SSE heartbeat P99 + Webhook 死信 P99 + cold start P99 + error rate %
  - 实测环境（per arch/06 §3 NFR：4 节点 K8s 模拟 + 100 RPS 压测 + 1h soak）
  - 报告模板（per ADR-0036 §8.1 L285-291 Phase G 报告 7 段结构）
- `scripts/bench-runner.sh`（~120 行，6 指标采集脚本 = criterion-rs + 报告 JSON 输出 + 历史对比）

### D20. Phase H+ 接真实外部服务（domain-integration 真 Git provider per Phase F）

**理由**：
- Phase F star-sa 4 provider（github/gitlab/bitbucket/gitea）已实装（per [ADR-0035 §2 D8 L84-103](0035-phase-f-architecture.md)）
- 但 Phase F/G/H 22 domain handler 调用 star-sa 走 mock（per §1.2 AGENTS.md §7 #4）
- Phase H+ 真实外部服务接入 = 4 provider 真 OAuth 走通 + 22 handler 穿透至真实 Git provider
- [ADR-0036 §7 #1 L268](0036-phase-g-architecture.md) Redis Cluster stub → Phase H+ 同步推真实接入

**形式**：
- 4 provider 配置（per spec/vcs/05 §1 配置 schema + §2 OAuth）：
  - GitHub：App OAuth（per 5 域 SRE NFR：≤ 5k req/h + ≤ 30 OAuth/scope）
  - GitLab：PAT + OAuth（≤ 5k req/h）
  - Bitbucket：OAuth 1.0a（per [AGENTS.md §7 G-02 缺口](../../../../AGENTS.md)）
  - Gitea：PAT + Webhook（self-host 模式）
- 22 handler → 真实 provider 穿透（per §3 关系表）
- 报告形式：PHASE-H-* 真实接入报告 + ADR-0038 Phase H+ 收尾

---

## 3. 跨 spec/crate 关系表

| 关系 | 上游契约 | 下游实现 | cross-ref |
|---|---|---|---|
| `spec/integration/01`（D16）↔ `spec/agents/02 §1` | spec/agents/02 §1 22 domain URI 模式 L16-38 + 5 域映射 | `crates/star-mcp/src/handlers/` 22 handler（D18）| spec/agents/02 §6 #6 L204 + ADR-0036 §7 #5 L272 |
| `spec/integration/01`（D16）↔ `spec/cache/01 §4` | spec/cache/01 §4 22 domain 默认 scope 表（read-mostly 22 + write-through 0）| 22 handler 接入 star-cache D13 | spec/cache/01 §1-§6 + ADR-0036 §2 D11 L62-78 |
| `spec/integration/01`（D16）↔ `spec/mcp/02 §1` | spec/mcp/02 §1 Resources 协议（4 类资源：worktree/agent/audit/decision）| 22 handler 走 Resources 协议 | spec/mcp/02 §1 + ADR-0034 §3 L158-160 |
| `spec/integration/01`（D16）↔ `spec/saga/01 §1` | spec/saga/01 §1 8 步状态机 + §2 5 域决策 | 22 handler 触发 5 业务域 Step trait | spec/saga/01 §1-§4 + ADR-0036 §2 D12 L80-96 |
| `spec/saga/02`（D17）↔ `spec/saga/01 §1` | spec/saga/01 §1 8 步状态机 + §4 补偿语义 | `crates/star-saga` 测试扩 3 → 12+ | spec/saga/01 §6 #1 + ADR-0036 §7 #3-#4 L270-271 |
| `spec/saga/02`（D17）↔ `crates/star-saga`（D14）| D14 Orchestrator + Step + Compensation + event | time-travel debug 模块 + chaos 注入点 + proptest 集成 | ADR-0036 §2 D14 L119-138 |
| `crates/star-mcp/handlers/`（D18）↔ 22 domain crates | star-mcp 16 tool handler trait + spec/mcp/02 §1 Resources | 22 handler × 5 模式 = 110 测试 | AGENTS.md §7 #4 P2 + ADR-0034 §2 D3 L82-100 |
| `bench/perf-baseline.md`（D19）↔ `ADR-0036 §2 D15` | D15 4 指标 P50/P95/P99 + Phase G+ 收敛 SLO | 6 指标扩展 + cold start + error rate | ADR-0036 §2 D15 L140-159 + §7 #8 L275 |
| `bench/perf-baseline.md`（D19）↔ `arch/06 §3 NFR` | arch/06 §3 NFR 性能预算（5 域 SRE Lead 来源）| 6 指标基线对齐 NFR | arch/06 §3 + ADR-0036 §4 责任矩阵 |
| Phase H+ 真接入（D20）↔ `spec/vcs/05 §1-§3` | spec/vcs/05 §1 4 provider 配置 schema + §2 OAuth + §3 Rate Limit | `crates/star-sa` 真实 OAuth + 22 handler 穿透 | ADR-0035 §2 D8 + spec/vcs/05 §1-§3 |

**关键边界**（per [ADR-0036 §3 关键边界 L178-184](0036-phase-g-architecture.md) 扩展）：
- `spec/integration/01` 是 **22 domain 接入契约层**（Tier 分类 + 接入顺序 + 5 验收 + 5 业务域 Step）
- `spec/saga/02` 是 **Saga 测试框架契约层**（time-travel + chaos + property-based + 5 业务域 governance）
- `crates/star-mcp/handlers/` 是 **22 handler 实装层**（mock → 真实最后一公里）
- `bench/perf-baseline.md` + `scripts/bench-runner.sh` 是 **6 指标性能基线层**（D15 4 指标扩展）
- `arch/03+05+06` 是 **架构总纲**（不变量 + 边界 + NFR，不变）

---

## 4. 5 域 + 5 业务域 + Performance Lead 责任矩阵（per 8/21 JST + ADR-0036 §4 续）

per 8/21 JST 用户偏好（5 域独立 Lead，不接受兼任）+ 8/27 21:59 JST 第三次强化"你可以代签" + Phase H 新增 Performance Lead：

| # | 域 | 角色 | Lead | Phase H 责任 | 决策范围 |
|---|---|---|---|---|---|
| 1 | 架构 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | ADR-0037 commit + 2 spec 终审 + 22 handler 接口终审 | spec/integration/01 + spec/saga/02 + 22 handler API |
| 2 | SRE | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 22 handler 部署 + D19 6 指标 SLO + 监控 + 110 测试 CI 集成 | 22 handler SLO + D19 性能基线 + 110 测试 CI |
| 3 | 平台 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 22 handler 依赖 + toolchain + workspace | 保 0 新外部依赖（除 criterion-rs 性能测试库）+ workspace.toml 同步 |
| 4 | 评审 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | DDD Review 主持 | Phase H 2 spec + 22 handler + 110+ 测试 DDD Review 主持 + sign-off |
| 5 | PM | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 进度跟踪 + 22 domain 接入排序 + 风险升级 | Phase H 4-6 人·周 OLU 校准 + D16 §2 接入顺序 + D19 6 指标优先级 |
| 6 | Player | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | 22 handler Player 域（如 player_state_handler.rs）+ Step trait | Player 域 Step trait + 22 handler 验收 |
| 7 | Economy | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，Q-003 决策核心) | 22 handler Economy 域（如 wallet/quota/ledger_handler.rs）+ Step trait | Q-003 决策 + Economy 域 Step trait + Phase H 优先级 P0 |
| 8 | Match | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | 22 handler Match 域（如 match_session/result_handler.rs）+ Step trait | Match 域 Step trait + 22 handler 验收 |
| 9 | Social | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | 22 handler Social 域（如 friend/notification_handler.rs）+ Step trait | Social 域 Step trait + 22 handler 验收 |
| 10 | Admin | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，COC 独立控制面) | 22 handler Admin 域（如 audit/permission_handler.rs）+ Step trait | COC 独立控制面 + Admin 域 Step trait |
| 11 | Performance | 域 Lead（Phase H 新增）| (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | D19 6 指标性能基线 + bench-runner.sh 维护 + cold start 优化 | 6 指标 SLO + bench 报告 sign-off + 性能回归门禁 |

**5 域 × 11 域责任矩阵**（per [ADR-0036 §4 L204-213](0036-phase-g-architecture.md) 扩展）：

| 决策类型 | 架构 | SRE | 平台 | 评审 | PM | 5 业务域 | Performance |
|---|---|---|---|---|---|---|---|
| 2 新 spec 终审 | 🟢 签 | 🟡 咨询 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟡 咨询 | 🟡 咨询 |
| 22 handler 接口 | 🟢 签 | 🟡 咨询 | 🟢 签（依赖）| 🟢 签 | 🟡 知会 | 🟢 签（业务域）| 🟡 咨询 |
| D17 Saga 测试框架 | 🟢 签 | 🟡 咨询 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟢 签（联合评审）| 🟡 咨询 |
| D19 6 指标性能基线 | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟢 签（独占）|
| 110 测试 CI 集成 | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟢 签（性能门禁）|
| D20 真 Git provider | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟡 咨询 |
| R-05 push 决策 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟢 签 | 🟡 知会 | 🟡 知会 |

**5 业务域决策矩阵**（per Q-003 Economy 域核心 + COC 独立控制面 + [ADR-0036 §4 L215-222](0036-phase-g-architecture.md) 续）：

| 22 handler 场景 | Player | Economy | Match | Social | Admin |
|---|---|---|---|---|---|
| player_state_handler | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签（COC 审计）|
| wallet_handler | 🟡 知会 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟢 签（COC 审计）|
| quota_handler | 🟡 知会 | 🟢 签（Q-003 核心）| 🟡 知会 | 🟡 知会 | 🟢 签（COC 审计）|
| match_session_handler | 🟡 知会 | 🟡 知会 | 🟢 签 | 🟡 知会 | 🟢 签（COC 审计）|
| friend_handler | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签 | 🟢 签（COC 审计）|
| audit_handler | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签（独占）|

---

## 5. token-OLU 估算（per 8/21 JST 框架）

per 8/21 JST token-OLU 框架（1 人·周 ≈ 1M tokens）+ [ADR-0036 §5 L226-243](0036-phase-g-architecture.md) + [ADR-0035 §5 L194-211](0035-phase-f-architecture.md)：

| 阶段 | 范围 | 估算 | 单价依据 |
|---|---|---|---|
| Phase H spec 写作 | 2 新 spec（integration/01 + saga/02）+ 1 ADR（本文）| 2-3M tokens | 每 spec 0.8-1.2M + ADR 0.4M |
| 22 domain handler 实装 | 22 handler × 3 模式 + 22 tests × 5 模式 + integration.rs | 25-40M tokens | 22 handler × 1-1.5M + 110 测试 × 0.05-0.1M |
| Saga 测试框架 | time-travel + chaos + property-based + 5 业务域联合评审 | 5-8M tokens | 3 模块 × 1.5-2.5M + governance 1M |
| 性能基线 | 6 指标 P50/P95/P99/error rate + cold start + bench-runner.sh | 1-2M tokens | 6 指标 × 0.2M + 报告 + 脚本 |
| **Phase H 总计** | — | **33-53M tokens ≈ 4-6 人·周** | vs ADR-0036 §5 L237 "15-23M / 2-3 人·周"（Phase G 主是缓存+Saga+性能预算基线；Phase H 主是 22 handler 实装 + Saga 测试 + 6 指标基线） |

**vs ADR-0036 §5 估算差异**：
- Phase H 估 "22 domain handler 实装 25-40M" 是最大块（每 handler 1-1.5M × 22 = 22-33M + 110 测试 5-7M）
- Phase G 估 "D15 性能预算实测 5-8M" 是 4 指标 → Phase H 6 指标扩展 + bench-runner.sh 1-2M（增量小）
- Saga 测试框架 5-8M 是 Phase H 增量（Phase G 仅 3 测试）
- 5 业务域联合评审 governance 1M 是 Phase H 增量
- 待 PM + Performance Lead 终审确认

---

## 6. 与上游 ADR 引用

- [ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 零厂商合作原则（D16 22 domain 接入不 vendor 锁定 + D17 不用 vendor 测试工具）
- [ADR-0023 Version Control Provider Abstraction](0023-version-control-provider.md) — VCS Core 抽象（D20 4 provider 真接入 + D18 handler 穿透）
- [ADR-0026 STAR AI Compatibility](0026-star-ai-compat.md) — STAR AI 5 通道 + Fallback Ladder 4 级（D18 handler 错误码走通道 1 直接返回）
- [ADR-0029 Universal Submit](0029-universal-submit.md) — Universal Submit 12 步 + 6 字段错误模型（D18 22 handler 错误码映射）
- [ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — Lease + Heartbeat + Resume 11 字段（D17 chaos test 模拟 lease 过期）
- [ADR-0031 Context Graph](0031-context-graph.md) — Context Graph MVP 4 节点 + 5 关系（D16 22 domain 含 context 走 T1 核心）
- [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) — MCP Transport stdio 16 tools + 6 字段错误模型（D18 22 handler 错误码统一 30 码）
- [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) — 代签规则（本文 commit author = Ulysses per 21:59 JST 第三次强化）
- [ADR-0034 Phase E Architecture](0034-phase-e-architecture.md) — Phase E 整体架构（D18 handler 思路同 §2 D3 MCP 16 tool 模式）
- [ADR-0035 Phase F Architecture](0035-phase-f-architecture.md) — Phase F 整体架构（D20 4 provider 真接入引用 §2 D8 L84-103 + §7 #10 L247 22 domain 接入）
- [ADR-0036 Phase G Architecture](0036-phase-g-architecture.md) — Phase G 整体架构（本文 §1.1 + §3 关系表 + §4 责任矩阵 + §5 token-OLU + §7 已知缺口 12 项 Phase H 承接 + §8.2 Phase H 方向 L293-298 落地）

---

## 7. 已知缺口

per 8/26 04:30 "缺标比错标安全" + 8/27 21:59 JST Mavis 接手代签（不动 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)的 SRE/平台/评审/PM/5 业务域/Performance）：

| # | 缺口 | 影响 | 状态 |
|---|---|---|---|
| 1 | 22 domain 实际接入优先级排期（per spec/integration/01 §2）| D16 6 Tier 顺序（T1 → T6）是草案，22 crate 哪些先接 5 模式 handler 待 PM + 5 业务域联合评审拍板 | D16 §2 列，PM 拍板 |
| 2 | 真实外部服务接入（per §2 D20）| D20 4 provider 真 Git provider 接入是 Phase H+，Phase H 仅 mock 穿透，Phase I 才真 OAuth | D20 §形式 显式列，Phase H+ 推 |
| 3 | 5 业务域 Lead 真实身份签字（DDD Review 阶段）| per §4 6-10 行 Player/Economy/Match/Social/Admin 5 业务域 Lead 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | DDD Review 阶段补 |
| 4 | Q-003 Economy Lead 决策 SLA 量化 | Q-003 是 D18 Economy handler 决策核心，但 Economy 域决策响应 SLA 未量化（如 < 100ms）| 缺标，SRE Lead + Economy Lead 协同拍板 |
| 5 | Saga 测试框架的 chaos tool 选型 | D17 §2 chaos test 5 域 Lead 拒绝响应注入用 chaos-mesh（K8s 侵入式）还是手动 mock（轻量）待 SRE Lead + 平台工程师拍板 | D17 §2 列，SRE + 平台拍板 |
| 6 | 22 domain 性能基线跨节点测试（Phase H+）| D19 6 指标基线是单进程，跨节点 K8s 模拟是 Phase H+（per arch/06 §3 NFR 4 节点要求）| 显式列，Phase H+ 推 |
| 7 | Cache 命中率优化（Phase G+）| D13 star-cache LRU 10000 条是否够（per [ADR-0036 §7 #11 L278](0036-phase-g-architecture.md)）+ Cache warming 启动预热（per [ADR-0036 §7 #2 L269](0036-phase-g-architecture.md)）| 跨 ADR 引用 + D19 cold start 间接测 |
| 8 | Phase H 完成后 acceptance/01-17 重新跑 | acceptance/01-17 是 5/26 旧版（per [AGENTS.md §7 #6](../../../../AGENTS.md) "9 个 wt 是否 merge 到 main"），22 handler 实装后需重跑 | 显式列，Phase H+ 推 |
| 9 | Property-based test 终止条件 | D17 §3 proptest 跑多少 case（默认 256 还是 1000）+ 收缩策略（shrinking）待 D17 spec 定 | D17 §3 列，Phase H spec 写时定 |
| 10 | time-travel debug 状态快照存储 | D17 §1 saga 状态快照存哪里（per 22 domain crate in-memory vs star-cache vs star-saga event log）待 SRE Lead 拍板 | D17 §1 列，SRE 拍板 |
| 11 | Performance Lead 真实身份签字（DDD Review 阶段）| per §4 #11 行 Performance Lead（Phase H 新增）🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，6 指标基线 + 性能回归门禁待 Performance Lead 终审 | DDD Review 阶段补 |
| 12 | 22 handler 跨域 Saga 触发点 governance | D16 §4 5 业务域 Step trait 22 handler 触发 Saga 流程，缺"5 域 Lead 联合评审 22 handler 触发点变更" governance（per [ADR-0036 §7 #10 L277](0036-phase-g-architecture.md) 续）| 缺标，Phase H+ 治理文档补 |

---

## 8. 后果

### 8.1 Phase H 交付（per §5 token-OLU 33-53M / 4-6 人·周）
- 2 新 spec（integration/01 + saga/02）
- 1 新 ADR（本文）
- 22 domain handler 实装（mock → 真实穿透）
- Saga 测试框架三件套（time-travel + chaos + property-based）
- 6 指标性能基线（D19）+ bench-runner.sh
- 110+ 新测试（22 handler × 5 模式）+ 12+ Saga 测试
- 0 新外部依赖（除 criterion-rs 性能测试库）

### 8.2 Phase I 方向（per MVP v1 ready 后 production rollout）
- **真实外部服务接入完整化**：D20 4 provider 真 OAuth 走通 + 22 handler 穿透至真实 Git provider + 5 业务域 Step trait production 数据
- **K8s 多节点部署**：arch/06 §3 NFR 4 节点 K8s 模拟 → production 部署
- **监控 + SLA 收口**：Performance Lead 6 指标 SLO → production SLA 签约
- **MVP v1 sign-off**：11 域 Lead 签字完整 + DDD Review 主持终审
- **Phase I 报告**：PHASE-I-* 报告 + ADR-0038 Phase I production rollout

### 8.3 Phase G → Phase H 不变量
- 守门 0 unsafe / 0 新外部依赖（除 criterion-rs 性能测试库例外）/ R-05 不 push
- bc23d6c 保留 / 5 域独立 Lead 拒绝兼任（per 8/21 JST 硬约束）
- token-OLU 框架（1 人·周 ≈ 1M tokens）
- 环境变量安全（per 8/27 11:06 JST hard ban）
- 代签规则应用（author = Ulysses，审批 = 架构师（Mavis 接手 agent per DEC-008））
- 缺标比错标安全（§7 已知缺口 12 项显式列）

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-28 | 🟢 Mavis 接手代签（per 2026-08-27 19:39 JST + 20:56 JST + 21:59 JST 用户授权三次强化 + 8/27 07:16 JST 代签规则反转授权）；本文 5 决策 D16-D20 + 2 新 spec + 22 handler 框架 + 6 指标性能基线终审 |
| 2 | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead（拒绝兼任 per 8/21 JST 硬约束），真实身份签字请 DDD Review 阶段补；§5 token-OLU 33-53M / D19 6 指标性能基线 / D17 chaos tool 选型待 SRE Lead 终审 |
| 3 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；D18 22 handler 0 新外部依赖（除 criterion-rs 性能测试库例外）待平台终审 |
| 4 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；§4 11 域 Lead 责任矩阵 DDD Review 主持待补 |
| 5 | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；§5 token-OLU 33-53M / §7 #1 22 domain 接入优先级 / D16 §2 6 Tier 顺序待 PM 终审 |
| 6 | Player 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead（per 8/21 JST 硬约束），真实身份签字请 DDD Review 阶段补；D18 player_state_handler + Step trait 待补 |
| 7 | Economy 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，Q-003 决策核心；D18 wallet/quota/ledger_handler + §7 #4 决策 SLA 量化待 Economy Lead 终审 |
| 8 | Match 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，真实身份签字请 DDD Review 阶段补；D18 match_session_handler + Step trait 待补 |
| 9 | Social 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，真实身份签字请 DDD Review 阶段补；D18 friend_handler + Step trait 待补 |
| 10 | Admin 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，COC 独立控制面；D18 audit/permission_handler + COC 独占决策待 Admin Lead 终审 |
| 11 | Performance 域 Lead（Phase H 新增）| (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — Phase H 新增 11 域（per 8/21 JST 5 域硬约束续），真实身份签字请 DDD Review 阶段补；D19 6 指标性能基线 + 性能回归门禁待 Performance Lead 终审 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签（per 19:39/20:56/21:59 JST 三次强化）| 初版：5 决策 D16-D20（spec/integration/01 + spec/saga/02 + 22 handler + D19 6 指标性能基线 + D20 真 Git provider）+ 5 域 + 5 业务域 + Performance = 11 域 Lead 责任矩阵 + 5 业务域决策矩阵（22 handler 触发 Saga 场景）+ token-OLU 33-53M 估算 + 与 11 上游 ADR 引用 + 12 项已知缺口 + Phase I 方向 | 2026-08-28 07:35 JST 用户派工"新建 1 份 ADR 0037 Phase H 22 domain 真实数据接入 + Saga 测试框架 + 缓存优化"，per 8/27 21:59 JST 第三次强化"继续, 你可以代签" |

---

## 11. 引用文档

- [adr/0036-phase-g-architecture.md](0036-phase-g-architecture.md) — Phase G 整体架构（base 6eb3cb5 引用 §1.1/§2 D11-D15/§3 关系表/§4 责任矩阵/§5 token-OLU/§7 已知缺口 12 项/§8.2 Phase H 方向 L293-298）
- [adr/0035-phase-f-architecture.md](0035-phase-f-architecture.md) — Phase F 整体架构（base e2c7bc9 引用 §2 D8 4 provider + §7 #10 22 domain 接入）
- [adr/0034-phase-e-architecture.md](0034-phase-e-architecture.md) — Phase E 整体架构（base 938e9ab 引用 §2 D3 MCP 16 tool 模式 + §3 关系图 + §5 token-OLU）
- [adr/0033-agent-co-signing-policy.md](0033-agent-co-signing-policy.md) — 代签规则反转 + 19:39 JST 升级 + 21:59 JST 第三次强化
- [spec/integration/01-22-domain-integration-spec.md](../spec/integration/01-22-domain-integration-spec.md)（Phase H 新建，per D16）
- [spec/saga/02-test-framework-spec.md](../spec/saga/02-test-framework-spec.md)（Phase H 新建，per D17）
- [spec/agents/02-data-sources-spec.md](../spec/agents/02-data-sources-spec.md) — 22 domain 数据源契约（§1 URI 模式 + §5 5 域映射 + §6 #6 离线缓存策略）
- [spec/agents/01-agent-runtime-spec.md](../spec/agents/01-agent-runtime-spec.md) — Agent 运行时（§2 Lease 协议 30s heartbeat / 300s TTL + §6 已知缺口）
- [spec/cache/01-cache-contract-spec.md](../spec/cache/01-cache-contract-spec.md) — 22 domain + 3 应用 crate 缓存契约（§1 5 维度键 + §3 失效钩子 + §4 scope 表 + §6 已知缺口）
- [spec/saga/01-saga-coordination-spec.md](../spec/saga/01-saga-coordination-spec.md) — 5 域 Lead Saga 协调（§1 8 步状态机 + §2 5 域决策 + §4 补偿语义 + §6 已知缺口）
- [spec/mcp/02-resources-prompts-spec.md](../spec/mcp/02-resources-prompts-spec.md) — Resources/Prompts 协议（§1 Resources 4 类 + §3 缓存策略）
- [spec/mcp/03-error-model-spec.md](../spec/mcp/03-error-model-spec.md) — 30 错误码（D18 22 handler 错误码映射）
- [spec/vcs/05-real-providers-spec.md](../spec/vcs/05-real-providers-spec.md) — 4 Git Provider 真实接入规范（§1 配置 schema + §2 OAuth + §3 Rate Limit）
- [spec/flows/06-error-recovery.md](../spec/flows/06-error-recovery.md) — 错误恢复（§3 错误码一致 → D17 property-based test 终止条件）
- [arch/06-threat-model-nfr.md](../arch/06-threat-model-nfr.md) — 威胁模型 + NFR（§3 NFR 是 5 域 SRE Lead + Performance Lead 性能预算来源）
- [AGENTS.md §0 一句话硬约束 + §1 代签规则 + §4 守门硬约束 + §7 待办清单](../../../../AGENTS.md)
- [PHASE-D3-MCP-TRANSPORT-REPORT.md §2 6 字段错误模型](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) — 错误模型基础（D18 22 handler 错误码升级到 30 错误码）
