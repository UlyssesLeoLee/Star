# 01. Star Multi-User & Multi-Agent 排他与幂等架构系统 - 需求定义书 (Requirements Definition)

> **状态**: 🟢 Accepted v1.0
> **生效**: 2026-09-07 (初版 2026-09-07)
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签" + 守门 #10 author=Ulysses)
> **关联**: [ADR-0048 排他与幂等架构 view 拍板](../2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md) · [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0043 audit_audit_event WORM](../2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md) · [ADR-0046 LangGraph TMO 任务卡管理操作](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [ADR-0047 PostgreSQL Checkpointer Tier 3](../2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展开强约束](../../AGENTS.md)
> **下游文档**: [02-basic-design.md v1.0](02-basic-design.md) (基本設計書) · [03-detailed-design.md v1.0](03-detailed-design.md) (詳細設計書) · [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) (8 子项实施计划)
> **实施范围**: STAR 主仓 (`D:\Star`) 全体, gm-console frontend / star-mcp / 22 domain-* crates / scripts/automation/ / PostgreSQL (跟 ADR-0047 checkpointer Tier 3 共享) 全栈

---

## 0. 目的 (Purpose)

本文档定义 Star 项目的 **多用户 & 多 agent 排他 (exclusion) 与幂等 (idempotency) 架构系统** (Star-EI) — 专门处理多用户并发 + 多 agent 并发场景下的资源竞争与重复执行问题.

通过本架构实现:

1. **多用户并发安全** — 同一 tenant 内多用户同时操作, 4 层锁链防止 race condition
2. **多 agent 协调** — L0 TopAgent 作为唯一协调者, 所有 L1↔L1 排他经 L0 (per 守门 #13 a 派生规)
3. **TMO 操作原子性** — 7 节点 TMO 操作加 L0 dispatch lock + Domain advisory lock 双层保护
4. **16 tool 幂等** — star-mcp 16 tool 全部走 idempotency_keys 表 (T, 24h dedup), 三重试安全
5. **审计完整** — 所有锁获取/释放事件进 audit_audit_event (per ADR-0043 WORM), 错乱后可追溯

## 1. 业务需求 (Business Requirements)

### 1.0 术语澄清 (排他 vs 幂等)

| 术语 | 定义 | 例子 |
|---|---|---|
| **排他 (Exclusion)** | 同一时刻只允许 1 个 actor 持有资源, 其他人等待或拒绝 | User A + User B 同时"暂停 task X", 后到者收到 409 |
| **幂等 (Idempotency)** | 同一操作执行 N 次效果 = 执行 1 次, 网络/客户端/服务端重试均安全 | 客户端重复点 Send 按钮, 1 次服务端写入 |
| **锁 (Lock)** | 实现排他的机制, 分为 advisory lock (PG) / lease (time-bounded) / optimistic (version CAS) | `pg_try_advisory_lock(work_item:UUID)` |
| **幂等键 (Idempotency Key)** | 实现幂等的标识, 双键 (client_uuid + business_hash) 兜底 | `Idempotency-Key: 7f3e-...` header |

**关系**:
- 排他解决"谁先谁后"问题 (序列化)
- 幂等解决"重复执行副作用"问题 (去重)
- 两者正交但常组合: 锁内操作可幂等, 幂等操作不一定要锁

### 1.1 背景 (Background)

Star 项目当前 (per 2026-09-07 main HEAD `6dba137`) 状态:

- **2-level hierarchical LangGraph**: L0 TopAgent + L1 9 类 SubAgent (SA-01..SA-09) + SA-10 task-orchestrator
- **22 domain-* crate**: DDD bounded context, 业务写入无统一锁机制
- **16 MCP tools**: star-mcp stdio + Streamable HTTP, 重试无去重
- **gm-console frontend**: Next.js + AppShell 5-tab, 客户端无 dedup
- **PostgreSQL Checkpointer Tier 3 (per ADR-0047)**: 5 表 schema, 共享 PG
- **AGENTS.md §4 守门**: 13 main (#1-#13) + 24 派生 (v1-v24) = 37 全局硬约束
- **守门 #13 a L0 协调**: L1↔L1 禁止直连, 排他经 L0

### 1.2 业务痛点 (Business Challenges)

| # | 痛点 | 影响范围 |
|---|---|---|
| **B-01** | 多用户并发操作 Kanban / worktree / agent 无锁, race condition 高发 | 全 5 域, gm-console 5-tab 全场景 |
| **B-02** | 16 tool 重试无去重, 客户端/网络/服务端三重试叠加副作用 | star-mcp 全 16 tool |
| **B-03** | TMO 7 节点 bulk / merge / split 跨用户原子性无保证, 中途被修改导致错乱 | TMO M-N1..M-N7 全节点 |
| **B-04** | L1 SubAgent 崩溃锁泄漏, lease heartbeat 缺失导致锁永远不释放 | L1 SubAgent 全 9 类型 + SA-10 |
| **B-05** | 多 agent 同时改 work_item 同一行, 缺行级 fence + version CAS | 22 domain-* crate, 业务行级更新 |
| **B-06** | 锁错乱后无审计追溯, 难定位是哪个 user / agent 何时持锁 / 是否正常释放 | 全栈, observability 缺位 |
| **B-07** | 客户端 SDK 漏带 Idempotency-Key 立即失去幂等保护 | gm-console frontend + star-mcp client |

### 1.3 期望效果 (Expected Effects)

| # | 效果 | 衡量指标 | 当前基线 | 目标 |
|---|---|---|---|---|
| **E-01** | 多用户并发安全 | 后到者收到 409 的概率 | 0% (无锁) | 100% |
| **E-02** | 多 agent 协调 | 业务行 race condition 发生率 | 估 5-10% | 0% |
| **E-03** | TMO 跨用户原子性 | bulk_node 部分回滚成功率 | 70% (per PHASE-LANGGRAPH-TMO-IMPL-REPORT v0.3 §1.1) | 100% |
| **E-04** | 16 tool 幂等 | 重复执行副作用率 | 估 3-5% | 0% |
| **E-05** | 锁不泄漏 | 24h 锁泄漏告警数 | 不可测 | 0 |
| **E-06** | 审计完整 | 锁错乱定位时间 | 估 30+ min | < 1 min |
| **E-07** | 客户端兜底 | 漏带 key 的副作用率 | 估 100% (无兜底) | 0% |
| **E-08** | 跨层 trace | 锁链端到端追踪 | 不可测 | 100% trace_id 贯穿 |

## 2. 功能需求 (Functional Requirements)

### 2.1 排他 (Exclusion) 功能

#### 2.1.1 F-01: 4 层锁链 (4-Tier Lock Chain)

**描述**: UI / L0 / L1 / Domain 4 层各持一种锁, 跨层锁链 trace_id 贯穿.

| 层 | 锁类型 | 用途 | 超时 |
|---|---|---|---|
| **L0 UI** | `IdempotencyManager` (browser) | 客户端 dedup 锁 | 30s (页面内) |
| **L1 TopAgent** | `DispatchLockManager` (PG advisory) | 派发级排他 (同一 task 拒绝并发 dispatch) | 60s |
| **L2 SubAgent** | `SubAgentLock` (lease + heartbeat) | task card 持锁 | 300s (heartbeat 30s) |
| **L3 Domain** | `DomainMutex` (PG advisory + version CAS) | 业务行锁 | transaction-bound |

**验收**: 4 层锁链全部实现 + 跨层 trace_id 一致 + 锁超时自动升级.

#### 2.1.2 F-02: 派发级排他 (Dispatch-Level Exclusion)

**描述**: L0 TopAgent 拒绝并发派发同一 task, 后到者收到 `409 Conflict` + `Retry-After`.

| 字段 | 类型 | 说明 |
|---|---|---|
| task_id | UUID | 被派发的 task |
| dispatch_id | UUID | 本次派发标识 (per F-04) |
| acquired_at | TIMESTAMPTZ | 派发时刻 |
| expires_at | TIMESTAMPTZ | 派发锁过期时刻 (60s) |
| actor_id | UUID | 派发者 (L0) |
| tenant_id | UUID | 租户 |

**验收**: 并发派发同一 task, 第二个 dispatch 立即收到 409.

#### 2.1.3 F-03: 任务卡持锁 (Task Card Lease)

**描述**: L1 SubAgent 持 task card 锁, lease 机制 + heartbeat + 自动过期.

| 字段 | 类型 | 说明 |
|---|---|---|
| lease_id | UUID | 持锁标识 |
| task_id | UUID | 被持锁的 task |
| lock_token | TEXT | 唯一 token, 释放时校验 |
| heartbeat_at | TIMESTAMPTZ | 最近心跳 |
| expires_at | TIMESTAMPTZ | 过期时刻 (300s) |
| released_at | TIMESTAMPTZ NULL | 释放时刻 |
| release_reason | TEXT NULL | 释放原因 (normal/timeout/error/forced) |

**验收**: SubAgent 每 30s 发心跳, 超过 300s 无心跳自动释放 + 触发告警.

#### 2.1.4 F-04: 业务行级 fence (Domain Row-Level Fence)

**描述**: 22 domain-* crate 业务行更新走 PG advisory lock + version CAS, 防多 agent 改同一行.

```rust
// 伪代码
async fn update_work_item(id: WorkItemId, new_state: WorkItemState, expected_version: u64, actor: ActorId) -> Result<()> {
    let lock_key = compute_lock_key("work_item", id); // BIGINT hash
    let _guard = pg_try_advisory_xact_lock(lock_key, 5000).await?; // 5s timeout
    let current = read_with_version(id).await?;
    if current.version != expected_version {
        return Err(DomainError::VersionMismatch { expected: expected_version, actual: current.version });
    }
    write_with_version(id, new_state, expected_version + 1).await?;
    Ok(())
}
```

**验收**: 1000 并发 agent 改同一行, 999 个收到 VersionMismatch + 1 个成功.

#### 2.1.5 F-05: 锁超时与升级 (Lock Timeout & Escalation)

**描述**: 4 层锁都有超时, 超时后自动升级 (L0 UI 超时 → 通知用户重试 / L1 dispatch 超时 → 释放重派 / L2 lease 超时 → 自动释放 + 告警 / L3 advisory 超时 → 事务回滚).

**验收**: 4 层超时场景全部测试通过 + 升级路径可视化.

### 2.2 幂等 (Idempotency) 功能

#### 2.2.1 F-06: 客户端 dedup (UI Layer)

**描述**: 客户端 `IdempotencyManager` 在用户点击 Send 时生成 UUID, 写入 `localStorage` + `AbortController` 取消 in-flight 重复请求.

```typescript
// frontend/src/lib/exclusion/idempotency.ts
class IdempotencyManager {
  private inflight = new Map<string, AbortController>();
  
  async dispatch<T>(key: string, fn: () => Promise<T>): Promise<T> {
    if (this.inflight.has(key)) {
      // 取消旧请求, 用新请求
      this.inflight.get(key)!.abort();
    }
    const ctrl = new AbortController();
    this.inflight.set(key, ctrl);
    try {
      return await fn();
    } finally {
      this.inflight.delete(key);
    }
  }
}
```

**验收**: 用户双击 Send 按钮, 1 次服务端写入.

#### 2.2.2 F-07: 派发级幂等 (Dispatch-Level Idempotency)

**描述**: L0 TopAgent 接受 `X-Dispatch-Id` header, 24h dedup.

| 字段 | 类型 | 说明 |
|---|---|---|
| key | TEXT UNIQUE | 客户端 UUID 或 dispatch_id |
| key_type | TEXT | client / business / dual |
| request_fingerprint | TEXT | request body hash, 防 key 复用但内容不同 |
| response_status | INT | 首次响应状态码 |
| response_body | JSONB | 首次响应体 (用于重放) |
| expires_at | TIMESTAMPTZ | 过期时刻 (24h) |

**验收**: 同一 dispatch_id 重试 N 次, 1 次副作用 + N-1 次返回首次响应.

#### 2.2.3 F-08: 16 tool 幂等 (MCP Tool Idempotency)

**描述**: star-mcp 16 tool 全部支持 `Idempotency-Key` header, 写入 idempotency_keys 表.

| tool | 幂等语义 |
|---|---|
| `create_merge_request` | 同 key 不创建重复 MR |
| `create_worktree` | 同 key 不创建重复 worktree |
| `search_issues` | 不幂等 (read-only) |
| `search_code` | 不幂等 (read-only) |
| `get_symbol` | 不幂等 (read-only) |
| `bulk_operation` | 同 key 跳过重复 TMO |
| `merge_tasks` | 同 key 不合并 |
| `split_task` | 同 key 不拆分 |
| `reorder_dependencies` | 同 key 不重排 |
| `summarize` | 不幂等 (idempotent by nature) |
| `reassign_agent` | 同 key 跳过 |
| `update_metadata` | 同 key 跳过 (双键校验) |
| ... 其余 4 tool 同 | |

**验收**: 16 tool 全部加幂等, 客户端/网络/服务端三重试安全.

#### 2.2.4 F-09: 业务主键 hash 兜底 (Business Key Fallback)

**描述**: 客户端漏带 Idempotency-Key, 服务端用 (tenant_id, work_item_id, operation, version) hash 作兜底 key.

**验收**: 漏带 key 仍能 dedup, 但仅在客户端故意省略时 (e.g. SDK 未集成), 不应作为默认.

#### 2.2.5 F-10: 双键 dedup (Dual-Key Deduplication)

**描述**: 同时使用 client_uuid + business_hash, 优先 client, 缺失 fallback business, 两者都缺失时拒绝 (返回 400 + 提示).

**验收**: 双键场景全部测试通过 + 缺失拒绝行为正确.

### 2.3 审计与可观测性 (Audit & Observability) 功能

#### 2.3.1 F-11: 锁审计 (Lock Audit)

**描述**: 所有锁获取/释放事件进 `audit_audit_event` (per ADR-0043 WORM).

| 字段 | 类型 | 说明 |
|---|---|---|
| event_type | TEXT | lock.acquired / lock.released / lock.timeout / lock.escalated |
| lock_target | TEXT | e.g. `work_item:UUID` |
| actor_id | UUID | user/agent |
| tenant_id | UUID | 租户 |
| trace_id | UUID | 跨层 trace |
| acquired_at | TIMESTAMPTZ | 获取时刻 |
| released_at | TIMESTAMPTZ NULL | 释放时刻 |
| wait_ms | INT | 等待锁时长 |
| hold_ms | INT NULL | 持锁时长 |

**验收**: 所有锁事件 100% 落 audit, WORM 防篡改.

#### 2.3.2 F-12: 跨层 trace (Cross-Tier Trace)

**描述**: UI → L0 → L1 → Domain 4 层共享 `trace_id` (UUID), 日志 + metric 全部带 trace_id, 可端到端追踪.

**验收**: 任一锁错乱可 1 click 看到 4 层日志.

#### 2.3.3 F-13: 锁 metric (Lock Metrics)

**描述**: Prometheus 暴露锁指标.

| metric | type | 说明 |
|---|---|---|
| `star_lock_acquired_total` | counter | 锁获取总数 (按 tier/result) |
| `star_lock_wait_seconds` | histogram | 锁等待时间分布 |
| `star_lock_hold_seconds` | histogram | 锁持有时间分布 |
| `star_lock_timeout_total` | counter | 锁超时总数 (按 tier) |
| `star_idempotency_dedup_total` | counter | 幂等去重总数 (按 key_type) |

**验收**: 5 指标全部暴露 + Grafana dashboard 可视化.

#### 2.3.4 F-14: 锁泄漏告警 (Lock Leak Alert)

**描述**: 24h 内锁持有 > 1h 触发 Slack 告警 + 自动 force-release (admin 域).

**验收**: 模拟锁泄漏 1h, 告警 + 自动释放.

## 3. 非功能需求 (NFR)

### 3.1 性能 NFR

| # | 指标 | 目标 | 测量方法 |
|---|---|---|---|
| **NFR-PERF-01** | 锁获取延迟 (P50) | < 5ms | benchmark 100K 次 |
| **NFR-PERF-02** | 锁获取延迟 (P99) | < 50ms | benchmark 100K 次 |
| **NFR-PERF-03** | 锁 QPS (PG advisory) | > 5K QPS 单 PG | benchmark |
| **NFR-PERF-04** | 幂等 dedup 延迟 | < 2ms (命中) | benchmark |
| **NFR-PERF-05** | 跨层 trace 关联延迟 | < 10ms | e2e |

### 3.2 可靠性 NFR

| # | 指标 | 目标 | 测量方法 |
|---|---|---|---|
| **NFR-REL-01** | 锁不泄漏 | 24h 0 泄漏 | chaos test |
| **NFR-REL-02** | 锁自动恢复 | SubAgent 崩溃 30s 内自动释放 | kill -9 test |
| **NFR-REL-03** | 幂等命中率 | > 99% (客户端 SDK 集成) | production metric |
| **NFR-REL-04** | 审计完整性 | 100% 锁事件落 audit | chaos test + audit query |

### 3.3 安全 NFR

| # | 指标 | 目标 | 测量方法 |
|---|---|---|---|
| **NFR-SEC-01** | RLS 13 类必携 | 100% 通过 | 22 domain 表扫描 |
| **NFR-SEC-02** | 跨 tenant 锁隔离 | 0 越权 | e2e |
| **NFR-SEC-03** | lock_token 不可猜 | 256-bit random | code review |
| **NFR-SEC-04** | 审计 WORM | 0 篡改 | WORM check |

### 3.4 可观测性 NFR

| # | 指标 | 目标 | 测量方法 |
|---|---|---|---|
| **NFR-OBS-01** | trace_id 覆盖率 | 100% (4 层) | e2e |
| **NFR-OBS-02** | 锁错乱定位时间 | < 1 min | manual drill |
| **NFR-OBS-03** | 锁 metric 暴露 | 5/5 | Prometheus check |

### 3.5 运维 NFR

| # | 指标 | 目标 | 测量方法 |
|---|---|---|---|
| **NFR-OPS-01** | 锁表容量规划 | 1000 并发用户 30 天不爆 | benchmark |
| **NFR-OPS-02** | 锁表归档 | 24h 后归档到 archive 表 | cron + 容量 |
| **NFR-OPS-03** | 锁 schema 演进 | 走 DDD Review Lead 拍板 | 流程 |

## 4. 约束 (Constraints)

### 4.1 守门 #13 强约束 (per AGENTS.md §4)

1. **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention** — 所有 Work 表 (含 24h TTL idempotency_keys 归档后) 物理删除 OK
2. **T = 物理删除禁止 + 監査必須 + RLS 13 類必携** — `idempotency_keys` / `lease_log` / `advisory_lock_audit` 全部 Transaction
3. **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携** — `exclusion_policy_master` Master 类
4. **Master 100% RLS / Transaction 100% audit / Work 100% retention_period** — 3 类分别校验

### 4.2 守门 #13 a L0 协调派生规

- **L1↔L1 禁止通信** — 多 SubAgent 排他必须经 L0 TopAgent 协调, 不能直接互锁
- 违反: L1 直接调 `pg_try_advisory_lock` 互抢 → 状态污染

### 4.3 守门 #5 (per 2026-08-27 11:06 JST 确立)

- **环境变量安全** — 锁服务连接字符串 (DATABASE_URL) 不打印, 仅引用

### 4.4 守门 #6 (持续)

- **PowerShell only** — 锁实施脚本 (.py / .ps1) 全部 PowerShell 兼容

### 4.5 守门 #1 v19 (per 2026-09-02 00:39 JST 拍板)

- **agent 交互 Python 化** — 排他/幂等实施走 `scripts/automation/exclusion/<purpose>.py`

### 4.6 守门 #9 v3 (per 2026-09-02 09:01 JST 拍板)

- **子代理 dispatch 走 subprocess 替代 RPC** — 排他/幂等调试走 console_server.py

### 4.7 守门 #14 v2 (per 2026-09-03 19:43 JST 拍板)

- **5 域 Lead 临时代签维持** — 5 域 Lead 真人到位前 Mavis 临时代签

### 4.8 跨项目持久约束 (per user_profile)

- **DB 三分類横展開 (W/T/M) 強制分類** — 5 张新表必 100% 表覆盖, 禁止混在一括列举
- **拍板决策必须用选项** — 后续如有重大拍板 (e.g. 锁服务换 Etcd), 必须 ask_user
- **拍板推荐项直接执行** — 已拍板项立即执行, 不需多确认
- **AI 协作文档治理** — 禁回溯叙事, 引用 BAS 必须 git log --follow 实证, 缺标比错标安全

### 4.9 部署约束

- **PG 单 cluster** — 跨 cluster 部署暂不支持 (锁绑单 PG), 后续如需扩展引入 Etcd
- **5 域 Lead 真人到位前不进入"装装"** — 排他/幂等实施进入实施阶段需 5 域 Lead T3 至少 1 人到位 (per 守门 #14 v2 拍板 D 维持)

## 5. 想定シナリオ (Use Case Scenarios)

### 5.1 S-01: 同一用户重复点 Send 按钮 (P0)

**场景**: User A 在 gm-console 点 Send 按钮, 浏览器卡顿, 重复点了 3 次.

**预期**:
1. 客户端 `IdempotencyManager` 检测 in-flight 请求, 取消前 2 次, 只发第 3 次
2. 即使 3 次都发出, 服务端 idempotency_keys 表 dedup, 1 次副作用 + 2 次返回首次响应

**验收**: 1 次服务端写入, 3 次 UI 显示一致结果.

### 5.2 S-02: 网络抖动服务端重试 (P0)

**场景**: L0 → L1 RPC 超时, L0 自动重试 3 次, 但第一次其实已成功.

**预期**:
1. L0 持 `X-Dispatch-Id` 重试, 服务端 idempotency_keys 表命中, 返回首次响应
2. 副作用仅发生 1 次

**验收**: 1 次 L1 执行, 3 次 L0 RPC 全部成功.

### 5.3 S-03: User A + User B 同时"暂停 task X" (P0)

**场景**: 同一 tenant 内 User A 和 User B 同时点 task X 的"暂停"按钮.

**预期**:
1. L0 dispatch_lock 检测 task X 已被 User A 派发, 拒绝 User B
2. User B 收到 409 Conflict + "task X 正在被 User A 操作, 请稍后重试"
3. User A 释放锁后, User B 可重新操作

**验收**: User B 收到 409, User A 操作完整, 0 数据错乱.

### 5.4 S-04: SA-01 + SA-04 同时改 work_item (P0)

**场景**: L0 同时派发 SA-01 (update work_item status) + SA-04 (add work_item comment) 给同一 work_item.

**预期**:
1. 两个 SubAgent 都获取 L1 task card lease (不同 lease_id)
2. Domain 层 `update_work_item` 走 advisory lock + version CAS
3. 后到者收到 VersionMismatch, 重试 1 次成功

**验收**: 2 个 SubAgent 都成功, 数据无丢失, version 单调递增.

### 5.5 S-05: bulk_node 暂停 5 任务中途 User C 修改 1 个 (P1)

**场景**: User A 用 TMO bulk_node 暂停 5 个 task, 执行到第 3 个时, User C 修改了第 3 个 task.

**预期**:
1. bulk_node 检测到第 3 个 task 已被 User C 修改 (version 不匹配)
2. 触发部分回滚: 已暂停的 1-2 个恢复, 3-5 个跳过
3. 通知 User A: "bulk 暂停部分失败, 1-2 已成功, 3 被 User C 修改跳过, 4-5 未执行"

**验收**: 0 数据错乱, User A 收到明确的部分失败通知.

### 5.6 S-06: L1 SubAgent 崩溃, lease 过期 (P0)

**场景**: SA-01 在持锁期间 OOM 崩溃, 锁未释放.

**预期**:
1. 300s 后 lease 自动过期, 锁释放
2. L0 检测到 lease 过期, 触发 SA-01 重试 (新进程, 新 lease_id)
3. 告警: "SA-01 崩溃, 自动重试 1 次"

**验收**: 锁自动释放, 任务自动恢复, 0 死锁.

### 5.7 S-07: 客户端漏带 Idempotency-Key (P1)

**场景**: 老版本 gm-console 客户端 (未集成 IdempotencyManager) 调用 star-mcp, 漏带 Idempotency-Key header.

**预期**:
1. 服务端检测 key 缺失
2. 自动用 business_hash = (tenant_id, work_item_id, operation) hash 兜底
3. 写入 idempotency_keys 表 (key_type=business)
4. 重试仍能 dedup

**验收**: 漏带 key 仍能 dedup, 行为与 client key 一致.

### 5.8 S-08: 1000 并发用户压测 advisory lock QPS (P1)

**场景**: 1000 并发用户同时改不同 work_item 行, 验证 PG advisory lock 极限.

**预期**:
1. PG advisory lock QPS > 5K (单 PG 16GB 内存)
2. P50 延迟 < 5ms, P99 延迟 < 50ms
3. 0 锁泄漏, 0 死锁

**验收**: 1000 并发 30 min 稳定运行, 性能指标达标.

## 6. 术语表 (Glossary)

| 术语 | 英文 | 定义 |
|---|---|---|
| 排他 | Exclusion | 同一时刻只允许 1 个 actor 持有资源 |
| 幂等 | Idempotency | 同一操作执行 N 次效果 = 执行 1 次 |
| 锁 | Lock | 实现排他的机制 |
| 幂等键 | Idempotency Key | 实现幂等的标识 |
| advisory lock | - | PG 提供的应用层协作锁, 不阻塞读 |
| lease | - | 限时持锁 + 心跳续约机制 |
| heartbeat | - | 持锁者定时上报存活 |
| version CAS | Compare-And-Swap | 基于版本号比较的乐观锁 |
| trace_id | - | 跨层调用追踪标识 |
| WORM | Write Once Read Many | 一次写入多次读取, 防篡改 |
| RLS 13 类 | Row-Level Security 13 类 | 13 类行级安全策略 (per 守门 #13 c) |

## 7. 已知缺口 (Known Gaps)

| # | 缺口 | 影响 | 拍板需求 |
|---|---|---|---|
| **G-EI-01** | 5 域 Lead 真人未到位, 排他/幂等实施进入"装装"阶段触发条件需 DDD Review Lead 拍板 | 中 | DDD Review Lead 真人到位后 |
| **G-EI-02** | PG 跨 cluster 锁不支持, 后续如需扩展需引入 Etcd 备选 | 中 | 跨 cluster 部署需求出现时 |
| **G-EI-03** | 锁 schema 演进走 DDD Review 拍板, 5 张新表 DDL 需 PG 容量规划 | 中 | 实施阶段 |
| **G-EI-04** | 双键 dedup 业务主键 hash 算法的版本兼容性 (e.g. work_item 主键从 UUID 改 (tenant, work_id) 复合) 需 DDD Review 拍板 | 中 | 业务 schema 演进时 |
| **G-EI-05** | 锁泄漏告警阈值 (24h 1h) 需 SRE Lead 拍板 | 低 | 部署阶段 |
| **G-EI-06** | 锁 metric 5 项 + Grafana dashboard 需 SRE Lead 评审 | 低 | 部署阶段 |

## 8. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 14 功能 (F-01..F-14) + 5 NFR (P/R/S/O/Ops) + 4 制約章 + 8 想定シナリオ | per `ask_37d138ffb93a12279b35a46e` 4 推荐项拍板 |
| v1.0 | 2026-09-07 20:55 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | Accepted v1.0 拍板落地, 同步 02/03 IPA 文档 + PHASE report 落档 | 拍板 + 02/03 IPA 文档齐备 |
