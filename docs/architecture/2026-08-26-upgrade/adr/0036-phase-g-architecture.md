# ADR-0036: Phase G 数据层 + 缓存 + Saga 架构

> **状态**：Draft v0.1
> **日期**：2026-08-27
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **审批**：架构师（Mavis 接手 agent per DEC-008）
> **触发**：per ADR-0035 §8.2 Phase G 方向（缓存层 + 跨域 Saga + 性能预算收敛）/ 2026-08-27 21:59 JST 用户授权第三次强化
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../../plan/2026-08-26-upgrade-plan.md)
> **依赖**：[ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) · [ADR-0035 Phase F Architecture](0035-phase-f-architecture.md) · [AGENTS.md §0 一句话硬约束](../../../../AGENTS.md)
> **关联**：[spec/agents/02-data-sources-spec.md §6 #6 离线缓存策略](../spec/agents/02-data-sources-spec.md) · [spec/services/02-sse-streaming-spec.md §4 Last-Event-ID 草案](../spec/services/02-sse-streaming-spec.md) · [spec/flows/08-event-model.md §3 event 持久化](../spec/flows/08-event-model.md) · [spec/flows/07-audit-model.md §4 跨域事务](../spec/flows/07-audit-model.md) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md §5 待办 #3 Prompts 实际模板](../../../../reports/PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md')

---

## 0. 一句话硬约束
> **可以代签 Ulysses，不可以编造历史。**
> — per AGENTS.md §0 + 2026-08-27 19:39 JST 用户授权升级 + 21:59 JST 第三次强化
> + 2026-08-21 JST 5 域独立 Lead 拒绝兼任硬约束（per Ulysses RGS 5 域拍板）

---

## 1. 背景

### 1.1 Phase F 已交付（per 2026-08-27 22:30-22:55 JST + base commit e2c7bc9）

| 阶段 | 交付 | 关键 commit | 引用 |
|---|---|---|---|
| Phase F ADR | [adr/0035-phase-f-architecture.md](0035-phase-f-architecture.md) (308 行) | `66d6799` (per `git log -p --follow`) | ADR-0035 §1 + §2 |
| Phase F spec/vcs/05 | 1 份 spec 413 行 = 4 Git Provider 接入规范 | `c7f507a` (merge feat/phase-f-spec-vcs) | spec/vcs/05 |
| Phase F spec/agents/02 | 1 份 spec 229 行 = 22 domain 数据源契约 | `c2e1479` (merge feat/phase-f-spec-agents) | spec/agents/02 |
| Phase F star-sa 实装 | 7 文件 229 行 + 6 测试 = 4 provider + trait | `9d86cea` (merge feat/phase-f-impl-sa) | ADR-0035 §2 D8 L84-103 |
| Phase F star-sse + star-webhook | 9 文件 771 行 + 24 测试 | `e2c7bc9` (merge feat/phase-f-impl-sse-webhook) | ADR-0035 §2 D9-D10 |
| Phase F workspace 测试 | 434 tests pass | per `cargo test --workspace` 2026-08-27 22:55 JST | ADR-0035 §8.1 |

**关键 Phase F 决定**（per [ADR-0035 §2 D6-D10 L48-147](0035-phase-f-architecture.md)）：
- D6 spec/vcs/05 落地 4 Git Provider 真实接入规范
- D7 spec/agents/02 落地 22 domain 数据源契约 + 5 域映射 + Read/Write 权限矩阵
- D8 star-sa 实装 4 provider + Provider trait + 6 测试
- D9 star-sse 实装 6 文件 + 9 测试（SSE 端到端 + heartbeat + replay）
- D10 star-webhook 实装 7 文件 + 15 测试（HMAC + 幂等 + 路由 + 重试 + 死信）

**Phase F 已知缺口遗留**（per [ADR-0035 §7 L232-247](0035-phase-f-architecture.md) 10 项 + spec/agents/02 §6 L195-204 6 项）：
- #3 跨域 Saga 协调待 Phase G（per ADR-0035 L240）
- #4 Phase F+ cache layer 性能预算未量化（per ADR-0035 L241）
- #5 SSE 多 node 部署 + Last-Event-ID 跨节点 replay（per ADR-0035 L242）
- #6 Webhook 接收端持久化（当前 in-memory）（per ADR-0035 L243）
- spec/agents/02 §6 #6 离线缓存策略待 Phase G（per spec/agents/02 L204）

### 1.2 Phase G 范围

Phase G 在 Phase F 基础上补 **3 大能力域**：

1. **缓存层**（22 domain + 3 应用 crate）：解决 22 domain crate 重复查询性能瓶颈 + 3 应用 crate（star-sa/sse/webhook）跨进程缓存
2. **跨域 Saga 协调**（5 域独立 Lead）：22 domain 跨域事务（如 MR 创建触发 notification + audit + context）需要 Saga
3. **性能预算 NFR 收敛**（per 5 域 SRE NFR）：Phase F 实测基线 → Phase G 收敛 SLO

> 关键边界：本 ADR §2 D11-D15 全部基于 8/27 22:55 JST 派工窗口"Phase G 数据层 + 缓存 + Saga 架构"任务派发；不沿用 bc23d6c 任何叙事（per AGENTS.md §4 #8 守门）；不引用未发生 commit。

---

## 2. 决策（5 项 D11-D15）

### D11. 新增 `spec/cache/01-cache-contract-spec.md` — 22 domain + 3 应用 crate 缓存契约

**理由**：
- Phase F 22 domain crate 已实装（per §1.1 + [ADR-0035 §2 D7 L66-82](0035-phase-f-architecture.md)），但每次 Resources 读取都打 22 crate，命中率 = 0%，性能无预算
- 3 应用 crate（star-sa / star-sse / star-webhook）跨进程调用无缓存，star-sa 4 provider 重复 OAuth + star-webhook 幂等表查重都未缓存
- spec/agents/02 §6 #6 L204 已显式列出"离线缓存策略（与 Phase G 缓存层）"为已知缺口
- AGENTS.md §7 #4 "16 tool 真实数据源接入"（现 mock）+ §7 #2 "Streamable HTTP session 重连" 都依赖缓存

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/cache/01-cache-contract-spec.md`
- 章节：
  - §1 缓存键 schema（per spec/agents/02 §1 Resource URI 模式 + 5 维度：crate + id + tenant + scope + version）
  - §2 TTL 策略（per spec/mcp/01 §1.1 ④ `ttlMs`）：默认 5min/tenant + 30s/agent + 1h/resource 三档
  - §3 失效钩子（per spec/agents/01 §2 Lease 协议 + spec/agents/02 §3 cache_invalidate 钩子）
  - §4 22 domain 默认 scope 表（read-mostly 22 + write-through 0；audit/event/notification 走 write-through）
  - §5 3 应用 crate 缓存策略（star-sa OAuth token / star-sse Last-Event-ID / star-webhook 幂等表）
  - §6 已知缺口（per §7 跨表 + Redis Cluster 模式 #1）

### D12. 新增 `spec/saga/01-saga-coordination-spec.md` — 5 域独立 Lead Saga 协调

**理由**：
- 22 domain 跨域事务（MR 创建触发 notification + audit + context + workspace 4 crate）当前无原子保证，per ADR-0035 §7 #3 L240
- 5 域独立 Lead（per 8/21 JST 硬约束）需要明确 Saga 决策权分配：Q-003 Economy 域核心 + Admin 域 COC 独立控制面
- spec/flows/07-audit-model.md §4 跨域事务已埋 Saga 钩子（需 Phase G 实装）
- 当前 star-webhook 死信队列（per ADR-0035 §2 D10 L141）是 Saga 失败恢复的基础设施

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/saga/01-saga-coordination-spec.md`
- 章节：
  - §1 Saga 协议（orchestrator + step executor + compensation，per 8 步）
  - §2 5 域 Lead 决策权分配（per §4 责任矩阵 + Q-003 Economy 域决策核心）
  - §3 跨域事务场景（4-6 个典型 case：MR 创建 / 资源删除 / 配额扣减 / 通知发送 / 工作流推进）
  - §4 补偿语义（forward recovery + backward recovery + hybrid 三种，per spec/flows/06-error-recovery.md §3）
  - §5 与 star-webhook 死信队列集成（per ADR-0035 §2 D10 死信 L141 + §7 #6 持久化待 Phase G+）
  - §6 已知缺口（per §7 跨表 + Saga 嵌套 #3 + Saga 版本管理 #4）

### D13. 新建 crate `star-cache` — InMemory (Phase G) + Redis stub (Phase G+)

**理由**：
- D11 缓存契约需实装层，per [ADR-0034 §2 D4 L102-118](0034-phase-e-architecture.md) "spec/mcp/03 30 错误码实装" 模式
- Phase G 仅 InMemory（dashmap + tokio RwLock），满足单进程性能预算
- Phase G+ Redis stub 留 trait 抽象（per [AGENTS.md §7 #2 待办 "Streamable HTTP session 重连"](../../../../AGENTS.md) 推 Phase G+ 一并）
- 与 star-mcp / star-sa / star-webhook 解耦（per [ADR-0035 §3 关系表 L150-167](0035-phase-f-architecture.md) "spec/agents/02 ↔ crates/star-mcp" 模式扩展）

**形式**：
- 路径：`crates/star-cache/`
- 文件：
  - `src/lib.rs`（~60 行，Cache trait re-export + 2 后端选择）
  - `src/trait.rs`（~120 行，`Cache` trait：`get/set/invalidate/scope/ttl`）
  - `src/key.rs`（~100 行，5 维度键 schema + 序列化，per D11 §1）
  - `src/inmemory.rs`（~180 行，dashmap + tokio RwLock + TTL 过期 + LRU 淘汰 10000 条）
  - `src/redis.rs`（~80 行，trait stub + Phase G+ 占位 `unimplemented!()`，per §7 #1）
  - `src/invalidate.rs`（~100 行，订阅 spec/agents/01 §2 Lease 过期事件 + spec/agents/02 §3 cache_invalidate 钩子）
  - `src/scope.rs`（~80 行，5 scope 默认 TTL：read-mostly 5min/tenant + write-through 即时）
  - `tests/inmemory.rs`（8 测试 = TTL 过期 × 2 + LRU 淘汰 × 2 + scope 隔离 × 2 + invalidate 钩子 × 2）
  - `tests/redis_stub.rs`（2 测试 = trait 接口编译通过）

### D14. 新建 crate `star-saga` — Saga orchestrator + step executor + compensation

**理由**：
- D12 Saga 协调契约需实装层
- orchestrator + step + compensation 三件套（per D12 §1）是 Saga 模式最小完整集
- 失败恢复依赖 star-webhook 死信队列（per D12 §5），两者通过 spec/flows/08-event-model.md §3 event 持久化解耦
- 与 star-cache 解耦：Saga 步骤状态走 star-webhook 死信队列，缓存走 star-cache

**形式**：
- 路径：`crates/star-saga/`
- 文件：
  - `src/lib.rs`（~80 行，Orchestrator + Step + Compensation re-export）
  - `src/orchestrator.rs`（~200 行，8 步状态机：Init → Running → Step1 → ... → Compensating → Done/Failed）
  - `src/step.rs`（~150 行，`Step` trait：`forward` + `compensate` + 3 错误码映射 per spec/mcp/03）
  - `src/compensation.rs`（~120 行，forward + backward + hybrid 三种恢复策略，per D12 §4）
  - `src/decision.rs`（~100 行，5 域 Lead 决策权分配，per D12 §2 + Q-003 Economy 域核心）
  - `src/event.rs`（~80 行，Saga 事件 + 死信队列桥接，per spec/flows/08 §3）
  - `src/audit.rs`（~80 行，跨域事务审计写入 spec/flows/07 §4 钩子）
  - `tests/orchestrator.rs`（6 测试 = 8 步状态机 × 1 + 失败恢复 × 3 + 5 域决策 × 1 + 审计 × 1）
  - `tests/compensation.rs`（4 测试 = forward × 1 + backward × 1 + hybrid × 1 + 嵌套失败 × 1，per §7 #3 嵌套）

### D15. 性能预算 NFR 收敛（per ADR-0035 §8.2 + 5 域 SRE NFR）

**理由**：
- Phase F 交付时无性能预算（per [ADR-0035 §7 #4 L241](0035-phase-f-architecture.md) "Phase F+ cache layer 性能预算未量化"）
- 5 域 SRE Lead NFR 是 [arch/06 §3 NFR](../arch/06-threat-model-nfr.md) 的一部分，5 域独立 Lead 拒绝兼任（per 8/21 JST 硬约束）
- 性能预算 = Phase G 实测基线 → Phase G+ 收敛 SLO 两阶段
- 0 unsafe + 0 新外部依赖（除 wiremock-rs Phase D.5+ 例外，per [ADR-0034 §2 D4 L102-118](0034-phase-e-architecture.md)）继续生效

**形式**：
- Phase G 实测基线（必含 4 指标）：
  - 22 domain Resources 读取 P50/P95/P99 latency（star-mcp Resources handler，per [spec/mcp/02 §1](../spec/mcp/02-resources-prompts-spec.md)）
  - 4 Git Provider OAuth 验签 P99（star-sa，per [ADR-0035 §2 D8](0035-phase-f-architecture.md)）
  - SSE heartbeat 30s 续传延迟（star-sse，per [ADR-0035 §2 D9](0035-phase-f-architecture.md)）
  - Webhook 死信队列投递 P99（star-webhook，per [ADR-0035 §2 D10](0035-phase-f-architecture.md)）
- Phase G+ 收敛 SLO（待 SRE Lead 拍板，per §4 责任矩阵）：
  - 22 domain 缓存命中率 ≥ 60%
  - Resources P99 ≤ 50ms（含 cache hit 路径）
  - OAuth 验签 P99 ≤ 30ms
  - Webhook 死信 P99 ≤ 100ms
- 报告形式：PHASE-G-* 实测报告 + ADR-0037 Phase G+ 收敛 ADR

---

## 3. 跨 spec/crate 关系表

| 关系 | 上游契约 | 下游实现 | cross-ref |
|---|---|---|---|
| `spec/cache/01`（D11）↔ `spec/agents/02 §1` | spec/agents/02 §1 Resource URI 模式（22 domain URI 表 L16-38）+ §3 cache_invalidate 钩子 | `crates/star-cache`（D13）InMemory + Redis stub | spec/agents/02 §6 #6 L204 "离线缓存策略" + ADR-0035 §7 #4 L241 |
| `spec/cache/01`（D11）↔ `spec/mcp/02 §1` | spec/mcp/02 §1 Resources 协议（4 类资源：worktree/agent/audit/decision） | `crates/star-mcp/src/resources.rs` 读穿 star-cache | spec/mcp/02 §3 缓存策略 + ADR-0034 §3 L158-160 |
| `spec/cache/01`（D11）↔ `spec/vcs/05 §1-§3` | spec/vcs/05 §1 4 provider 配置 schema + §2 OAuth + §3 Rate Limit | `crates/star-sa` OAuth token 缓存 + Rate Limit 状态缓存 | ADR-0035 §2 D8 L84-103 |
| `spec/saga/01`（D12）↔ `spec/agents/01 §2` | spec/agents/01 §2 Lease 协议 L104-141（30s heartbeat / 300s TTL） | `crates/star-saga` Step 持有 lease 跨步骤 | spec/agents/01 §6 L216-228 |
| `spec/saga/01`（D12）↔ `spec/flows/07 §4` | spec/flows/07 §4 跨域事务审计钩子 | `crates/star-saga` Saga 审计写入 `domain-audit` | spec/flows/07 §4 + ADR-0035 §2 D10 死信 |
| `spec/saga/01`（D12）↔ `spec/services/07`（计划 TBD）| spec/services/07 计划覆盖跨服务事务（注：当前 spec/services/ 仅 01-03 三份，07 计划 per §7 #9 已知缺口）| `crates/star-saga` Decision module 决策路由 | per §7 #9 占位 |
| `crates/star-cache`（D13）↔ `spec/cache/01` | spec/cache/01 §1-§6 全 6 节 | 5 维度键 + InMemory + Redis stub + 8+2 测试 | D13 §形式 9 文件 |
| `crates/star-saga`（D14）↔ `spec/saga/01` | spec/saga/01 §1-§6 全 6 节 | Orchestrator + Step + Compensation + 6+4 测试 | D14 §形式 9 文件 |
| 5 域独立 Lead ↔ Q-003 决策 | per 8/21 JST 5 域独立 Lead 拒绝兼任 + Q-003 Economy 域核心 | `crates/star-saga/src/decision.rs` 5 域路由 | D12 §2 + §4 责任矩阵 |
| `crates/star-saga` ↔ `crates/star-webhook` 死信队列 | ADR-0035 §2 D10 L141 死信表 `webhook_dead_letter` | Saga 失败补偿事件投递 | ADR-0035 §7 #6 L243 in-memory → Phase G+ DB |

**关键边界**（per [ADR-0035 §3 关键边界 L161-167](0035-phase-f-architecture.md) 扩展）：
- `spec/cache/01` 是 **缓存契约层**（不变量 + 键 schema + TTL + scope）
- `spec/saga/01` 是 **Saga 协调契约层**（5 域决策 + 8 步状态机 + 补偿）
- `crates/star-cache/saga` 是 **2 个新数据层 crate**（实装层）
- `spec/services/01-03` + `spec/services/07 计划` 是 **服务适配器 spec**（已存在 + 计划）
- `arch/03+05+06` 是 **架构总纲**（不变量 + 边界 + NFR，不变）

---

## 4. 5 域独立 Lead 责任矩阵（per 8/21 JST 续）

per 8/21 JST 用户偏好（5 域独立 Lead，不接受兼任）+ 8/27 21:59 JST 第三次强化"你可以代签"：

| # | 域 | 角色 | Lead | Phase G 责任 | 决策范围 |
|---|---|---|---|---|---|
| 1 | 架构 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | ADR-0036 commit + 2 spec 终审 + 2 crate 接口终审 | spec/cache/01 + spec/saga/01 + 2 crate API |
| 2 | SRE | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | star-cache + star-saga 部署 + SLO + 监控 | 2 crate SLO + D15 性能预算基线 + 10+10 测试 CI 集成 |
| 3 | 平台 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 2 crate 依赖 + toolchain + workspace | 保 0 新外部依赖（除 wiremock/redis-rs Phase G+ 例外）+ workspace.toml 同步 |
| 4 | 评审 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | DDD Review 主持 | Phase G 2 spec + 2 crate + 20 测试 DDD Review 主持 + sign-off |
| 5 | PM | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 进度跟踪 + 22 domain 接入排序 + 风险升级 | Phase G 2-3 人·周 OLU 校准 + D15 SLO 优先级 |
| 6 | Player | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | Saga Player 域步骤（如玩家状态变更）| Player 域 Step trait 实现审批 |
| 7 | Economy | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，Q-003 决策核心) | Saga Economy 域步骤（如配额扣减 + 货币兑换）| Q-003 决策 + Economy 域 Step trait + Phase G+ 优先级 P0 |
| 8 | Match | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | Saga Match 域步骤（如对局状态机推进）| Match 域 Step trait 实现审批 |
| 9 | Social | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | Saga Social 域步骤（如好友关系 + 通知）| Social 域 Step trait 实现审批 |
| 10 | Admin | 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，COC 独立控制面) | Saga Admin 域步骤（如 COC 审计 + 权限回收）| COC 独立控制面 + Admin 域 Step trait |

**5 域 × 5 域责任矩阵**（per [ADR-0035 §4 L182-190](0035-phase-f-architecture.md) 扩展）：

| 决策类型 | 架构 | SRE | 平台 | 评审 | PM |
|---|---|---|---|---|---|
| 2 新 spec 终审 | 🟢 签 | 🟡 咨询 | 🟡 咨询 | 🟢 签 | 🟡 知会 |
| 2 新 crate 接口 | 🟢 签 | 🟡 咨询 | 🟢 签（依赖）| 🟢 签 | 🟡 知会 |
| D15 性能预算基线 | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 |
| 20 测试 CI 集成 | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 |
| 5 域 Lead 决策路由 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟢 签 |
| R-05 push 决策 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟢 签 |

**5 业务域决策矩阵**（per Q-003 Economy 域核心 + COC 独立控制面）：

| Saga 场景 | Player | Economy | Match | Social | Admin |
|---|---|---|---|---|---|
| MR 创建 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签（COC 审计）|
| 配额扣减 | 🟡 知会 | 🟢 签 | 🟡 知会 | 🟡 知会 | 🟢 签（COC 审计）|
| 通知发送 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签 | 🟢 签（COC 审计）|
| 权限回收 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟡 知会 | 🟢 签（独占）|

---

## 5. token-OLU 估算（per 8/21 JST 框架）

per 8/21 JST token-OLU 框架（1 人·周 ≈ 1M tokens）+ [ADR-0035 §5 L194-211](0035-phase-f-architecture.md) + ADR-0034 §5：

| 阶段 | 范围 | 估算 | 单价依据 |
|---|---|---|---|
| Phase G spec 写作 | 2 新 spec（cache/01 + saga/01）+ 1 ADR（本文）| 2-3M tokens | 每 spec 0.8-1.2M + ADR 0.4M |
| crates/star-cache 实装 | Cache trait + InMemory + Redis stub + invalidate + 8+2 测试 | 3-4M tokens | 7 文件 + 10 测试 |
| crates/star-saga 实装 | Orchestrator + Step + Compensation + decision + 6+4 测试 | 4-6M tokens | 8 文件 + 10 测试 |
| D15 性能预算实测 | 4 指标 P50/P95/P99 + 报告 | 5-8M tokens | 4 指标 × 1.5M + 报告 1M |
| 5 域决策路由集成 | 5 业务域 Step trait 实现 + 集成测试 | 1-2M tokens | 5 域 × 0.3M |
| **Phase G 总计** | — | **15-23M tokens ≈ 2-3 人·周** | vs ADR-0035 §5 L205 "35-55M / 4-6 人·周"（Phase F 主是 22 domain 接入 25-40M；Phase G 主是缓存+Saga+性能预算）|

**vs ADR-0035 §5 估算差异**：
- Phase F 估 "22 domain crate 接入 25-40M（每 crate 1-2M）" 实装消耗 → Phase G 跳过
- Phase G 估 "性能预算实测 5-8M" 比 Phase F "30 测试 CI 集成 1-2M" 多 3-6M（实测需多轮迭代）
- 5 域决策路由 1-2M 是 Phase G 增量（Phase F 仅 5 域 Lead 责任矩阵未实装）
- 待 PM 终审确认

---

## 6. 与上游 ADR 引用

- [ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 零厂商合作原则（D11 缓存键不能 vendor 锁定 + D12 Saga 不绑 vendor 协议）
- [ADR-0023 Version Control Provider Abstraction](0023-version-control-provider.md) — VCS Core 抽象（D11 §5 star-sa OAuth token 缓存 + Rate Limit 缓存）
- [ADR-0026 STAR AI Compatibility](0026-star-ai-compat.md) — STAR AI 5 通道 + Fallback Ladder 4 级（D14 Saga 事件走通道 3 event bus）
- [ADR-0029 Universal Submit](0029-universal-submit.md) — Universal Submit 12 步 + 6 字段错误模型（D14 Step 错误码映射）
- [ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — Lease + Heartbeat + Resume 11 字段（D11 §3 失效钩子 + D14 Step 持 lease）
- [ADR-0031 Context Graph](0031-context-graph.md) — Context Graph MVP 4 节点 + 5 关系（D11 §4 22 domain 含 context 走 read-mostly）
- [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) — MCP Transport stdio 16 tools + 6 字段错误模型（D13/D14 错误码统一 30 码）
- [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) — 代签规则（本文 commit author = Ulysses per 21:59 JST 第三次强化）
- [ADR-0034 Phase E Architecture](0034-phase-e-architecture.md) — Phase E 整体架构（D13 Redis stub 思路同 §2 D4 wiremock 例外模式）
- [ADR-0035 Phase F Architecture](0035-phase-f-architecture.md) — Phase F 整体架构（本文 §1.1 + §3 关系表 + §4 责任矩阵 + §5 token-OLU 引用）

---

## 7. 已知缺口

per 8/26 04:30 "缺标比错标安全" + 8/27 21:59 JST Mavis 接手代签（不动 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)的 SRE/平台/评审/PM）：

| # | 缺口 | 影响 | 状态 |
|---|---|---|---|
| 1 | `crates/star-cache` Redis 后端仅 stub（Phase G+ 实装）| D11 §5 22 domain 缓存只能走 InMemory（dashmap + RwLock），多 node 部署需 Phase G+ Redis Cluster | D13 §形式 redis.rs 显式 stub，§7 #1 列 |
| 2 | Cache warming 启动预热未设计 | Phase G 冷启动 cache hit rate = 0%，需设计预热（5 域读 most 优先 8 crate 预热）| 缺标，Phase G+ 补 |
| 3 | Saga 嵌套（sub-saga）未设计 | D12 §1 8 步状态机不支持嵌套（如 MR 创建触发 sub-saga 工作流推进）| spec/saga/01 §6 #1 列，Phase H+ 补 |
| 4 | Saga 版本管理演进未设计 | Saga 协议升级如何处理已发起的 saga 实例（forward compat / 强制 abort）| 缺标，Phase H+ 补 |
| 5 | 22 domain 接入优先级排期 | 22 crate 哪些先接 cache 策略 + Saga Step trait（per [ADR-0035 §7 #10 L247](0035-phase-f-architecture.md) 仍未决）| PM 拍板 |
| 6 | 5 域 Lead 真实身份签字 | per §4 6-10 行 Player/Economy/Match/Social/Admin 5 业务域 Lead 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | DDD Review 阶段补 |
| 7 | Q-003 Economy Lead 决策 SLA 量化 | Q-003 是 D14 Saga 决策核心，但 Economy 域决策响应 SLA 未量化（如 < 100ms）| 缺标，SRE Lead + Economy Lead 协同拍板 |
| 8 | Phase H 性能预算基线 | D15 仅给 Phase G 实测基线 + Phase G+ 收敛 SLO；Phase H 目标（如 P99 ≤ 10ms）未列 | Phase G+ 报告后定 |
| 9 | `spec/services/07` 计划是否成立 | §3 关系表引用 spec/services/07（计划 TBD），当前 spec/services/ 仅 01-03 三份；如 Phase G 需补 07 spec 则需先建 | 显式列"计划 TBD"，PM 拍板 |
| 10 | 5 业务域 Step trait 5 域独立 Lead 协作流程 | §4 决策矩阵是 RACI 雏形，缺少"5 域 Lead 联合评审 Saga 场景变更"的 governance 流程 | 缺标，Phase G+ 治理文档补 |
| 11 | `crates/star-cache` 内存上限 LRU 10000 条是否够 | Phase G 单进程 22 domain × 100 条 ≈ 2200 条；10000 上限留 5x 缓冲；Phase H 需重新评估 | D13 §形式 LRU 10000 显式列 |
| 12 | `crates/star-saga` 死信队列持久化层 | D14 §形式 event.rs 桥接 star-webhook 死信（per ADR-0035 §7 #6 L243 in-memory），DB 持久化 Phase G+ | 跨 ADR 引用 + D14 形式显式列 |

---

## 8. 后果

### 8.1 Phase G 交付（per §5 token-OLU 15-23M / 2-3 人·周）
- 2 新 spec（cache/01 + saga/01）
- 1 新 ADR（本文）
- 2 新 crate（star-cache + star-saga）
- 4 指标性能预算实测基线（D15）
- 20+ 新测试（10 star-cache + 10 star-saga）
- 0 新外部依赖（除 redis-rs Phase G+ 例外）

### 8.2 Phase H 方向
- **缓存命中率优化**：InMemory 调优 + 预热（per §7 #2）+ 22 domain 真实数据接入完整化（per §7 #5）
- **Saga 测试框架**：time-travel debug（模拟任意时间点 saga 状态）+ 嵌套 sub-saga（per §7 #3）
- **22 domain 真实数据接入完整化**：3 非核心 domain crate（collaboration/comment/board）接入 + 5 业务域 Step trait 全部实装
- **Redis Cluster 模式**：star-cache Redis stub 实装 + star-sse 多 node 部署 + star-webhook DB 持久化（per §7 #1 + #12）
- **Phase G+ 性能预算收敛**：D15 收敛 SLO 实装（per §7 #8）

### 8.3 Phase F → Phase G 不变量
- 守门 0 unsafe / 0 新外部依赖（除 redis-rs Phase G+ 例外）/ R-05 不 push
- bc23d6c 保留 / 5 域独立 Lead 拒绝兼任（per 8/21 JST 硬约束）
- token-OLU 框架（1 人·周 ≈ 1M tokens）
- 环境变量安全（per 8/27 11:06 JST hard ban）
- 代签规则应用（author = Ulysses，审批 = 架构师（Mavis 接手 agent per DEC-008））
- 缺标比错标安全（§7 已知缺口 12 项显式列）

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签（per 2026-08-27 19:39 JST + 20:56 JST + 21:59 JST 用户授权三次强化 + 8/27 07:16 JST 代签规则反转授权）；本文 5 决策 D11-D15 + 2 新 spec + 2 新 crate 终审 |
| 2 | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead（拒绝兼任 per 8/21 JST 硬约束），真实身份签字请 DDD Review 阶段补；§5 token-OLU 15-23M / D15 性能预算基线待 SRE Lead 终审 |
| 3 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；D13 Redis stub 0 新外部依赖（除 redis-rs Phase G+ 例外）待平台终审 |
| 4 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；§4 10 域 Lead 责任矩阵 DDD Review 主持待补 |
| 5 | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；§5 token-OLU 15-23M / §7 #5 22 domain 接入优先级 / §7 #9 spec/services/07 计划待 PM 终审 |
| 6 | Player 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead（per 8/21 JST 硬约束），真实身份签字请 DDD Review 阶段补；D14 Player 域 Step trait 待补 |
| 7 | Economy 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，Q-003 决策核心；D14 Economy 域 Step trait + §7 #7 决策 SLA 量化待 Economy Lead 终审 |
| 8 | Match 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，真实身份签字请 DDD Review 阶段补；D14 Match 域 Step trait 待补 |
| 9 | Social 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，真实身份签字请 DDD Review 阶段补；D14 Social 域 Step trait 待补 |
| 10 | Admin 域 Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 业务域 Lead，COC 独立控制面；D14 Admin 域 Step trait + COC 独占决策待 Admin Lead 终审 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签（per 19:39/20:56/21:59 JST 三次强化）| 初版：5 决策 D11-D15（spec/cache/01 + spec/saga/01 + star-cache + star-saga + D15 性能预算）+ 5 域 + 5 业务域 = 10 域 Lead 责任矩阵 + 5 业务域决策矩阵（MR 创建/配额扣减/通知发送/权限回收）+ token-OLU 15-23M 估算 + 与 10 上游 ADR 引用 + 12 项已知缺口 + Phase H 方向 | 2026-08-27 22:55 JST 用户派工"新建 1 份 ADR 0036 Phase G 数据层 + 缓存 + Saga 架构"，per 8/27 21:59 JST 第三次强化"继续, 你可以代签" |

---

## 11. 引用文档

- [adr/0035-phase-f-architecture.md](0035-phase-f-architecture.md) — Phase F 整体架构（base e2c7bc9 引用 §1.1/§2 D6-D10/§3 关系表/§4 责任矩阵/§5 token-OLU/§7 已知缺口 #3-#6/§8.2 Phase G 方向）
- [adr/0034-phase-e-architecture.md](0034-phase-e-architecture.md) — Phase E 整体架构（base 938e9ab 引用 §2 D4 wiremock 例外模式 + §3 关系图 + §5 token-OLU）
- [adr/0033-agent-co-signing-policy.md](0033-agent-co-signing-policy.md) — 代签规则反转 + 19:39 JST 升级 + 21:59 JST 第三次强化
- [spec/agents/02-data-sources-spec.md](../spec/agents/02-data-sources-spec.md) — 22 domain 数据源契约（§1 URI 模式 + §3 cache_invalidate 钩子 + §6 #6 离线缓存策略）
- [spec/agents/01-agent-runtime-spec.md](../spec/agents/01-agent-runtime-spec.md) — Agent 运行时（§2 Lease 协议 30s heartbeat / 300s TTL + §6 已知缺口）
- [spec/mcp/02-resources-prompts-spec.md](../spec/mcp/02-resources-prompts-spec.md) — Resources/Prompts 协议（§1 Resources 4 类 + §3 缓存策略）
- [spec/vcs/05-real-providers-spec.md](../spec/vcs/05-real-providers-spec.md) — 4 Git Provider 真实接入规范（§1 配置 schema + §2 OAuth + §3 Rate Limit）
- [spec/services/01-service-adapter-spec.md](../spec/services/01-service-adapter-spec.md) — SA 抽象层（§1-§3 trait + §6 G-01/G-03/G-04 缺口）
- [spec/services/02-sse-streaming-spec.md](../spec/services/02-sse-streaming-spec.md) — SSE 流式（§2 heartbeat 30s + §3 MCP 边界 + §4 Last-Event-ID 草案）
- [spec/services/03-webhook-adapter-spec.md](../spec/services/03-webhook-adapter-spec.md) — Webhook 适配（§2 HMAC + §3 幂等 + §4 路由 + §5 死信 + §6 G-02 Bitbucket 迁移）
- [spec/flows/08-event-model.md](../spec/flows/08-event-model.md) — Event 模型（§3 event 持久化）
- [arch/06-threat-model-nfr.md](../arch/06-threat-model-nfr.md) — 威胁模型 + NFR（§3 NFR 是 5 域 SRE Lead 性能预算来源）
- [AGENTS.md §0 一句话硬约束 + §1 代签规则 + §4 守门硬约束 + §7 待办清单](../../../../AGENTS.md)
- [PHASE-D3-MCP-TRANSPORT-REPORT.md §2 6 字段错误模型](../../../../reports/PHASE-D3-MCP-TRANSPORT-REPORT.md') — 错误模型基础（star-cache + star-saga 错误码升级到 30 错误码）
