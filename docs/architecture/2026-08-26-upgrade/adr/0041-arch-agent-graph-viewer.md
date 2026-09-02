# ADR-0041: arch-agent-graph-viewer — Kanban 卡架构查看器 (cypher + memgraph + LLM 增量)

> **状态**: Draft v0.1
> **日期**: 2026-09-02
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02 自审
> **触发**: per 2026-09-02 00:33 JST Ulysses 拍板 (cypher 图 + memgraph + agent 增量 + 幂等排他 + 多人用) + 2026-09-02 00:36/02:00 JST 后续 4+3 拍板 (cytoscape.js / modal 居中 / 1-hop 高亮 / 后台 LLM worker / 幂等+排他 / 本地+git 双源)

> **dual-use 提醒 (per AGENTS.md §5 + 2026-08-31 22:45 JST Q1-D 拍板)**:
> 本 ADR 涉及的"25 domain 节点模型"对应 Star 仓 22 `domain-*` crate 的 DDD bounded context,**不**映射 RGS 5 域业务子域 (player/economy/match/social/admin)。
> 5 域是 RGS 仓历史治理命名, Star 仓**不建立业务子域↔DDD 映射**。

---

## §0 目的

为 Kanban 任务卡 (WorkItem) 提供**架构查看器**: 点击卡片上的 🕸 Arch 按钮, 弹 modal 显示该任务在 25 域 cypher 图中所处位置, 当前任务节点 + 1-hop 邻居高亮, 其余节点 20% opacity 弱化。

数据来源 = 后台 LLM worker 分析**本地代码库**或**git 仓** (D:/Star 当前 worktree), 输出 25 域节点 + 边, **upsert 进 memgraph**。多人同时点击同一 work_item 时, **幂等 (同 fingerprint skip)** + **排他 (per-work_item_id advisory lock)**, 保证数据库不被并发写炸。

本 ADR 拍板 3 件事:
1. **节点/边数据模型** (25 domain 投影, 1-hop 查询形状)
2. **Agent 增量更新 memgraph 形态** (后台 LLM worker + 幂等+排他)
3. **前端渲染契约** (cytoscape.js modal + 1-hop 高亮)

---

## §1 背景与动机

### 1.1 现状 (per 仓 2026-09-02 00:33 JST 实证)

| 现状 | 引用 |
|---|---|
| 22 `domain-*` crate DDD bounded context | `crates/` 目录 2026-09-01 实证 (per [ADR-0040 §1](../../adr/0040-domain-batch.md)) |
| 25 Module Resource (MRU) | [docs/api-design.md §2.1](../../../../api-design.md) |
| 16 MCP tool 真实数据接入 (现 mock) | [AGENTS.md §7 #2](../../../../AGENTS.md) |
| 5 tab: Kanban / Timeline / Backlog / Agents / Worktrees | commit `7d85c34` |
| KanbanCard 已支持 onClick 跳 /work-item/{id} | `frontend/src/components/board/KanbanCard.tsx:73-80` |
| 前端无 graph 渲染库 | `frontend/package.json` 无 cytoscape / reactflow / sigma |
| 前端无 memgraph client | `frontend/` 全文 grep 无 memgraph / cypher / bolt 关键字 |

### 1.2 痛点 (per 2026-09-02 00:33 JST Ulysses 描述)

> 我需要 kanban 里的任务卡有一个架构查看器功能, 点击后将其关联内容的 cypher 图以弹窗方式显示出来, 可以使用 memgraph 支持这个功能, 并且当前任务的节点和边应该高亮。以此便于用户查阅当前架构情况和任务负责的功能所处位置, 提高把控架构的能力。

翻译:
- **任务卡 ↔ 架构位置** 不直观, 看 task 不知道它触达 25 域哪几个
- **多人协作** 时, agent 重跑可能写炸 memgraph (并发 upsert)
- **数据陈旧** 时, 看不出当前 code → domain 真实对应

### 1.3 不在本 ADR 范围 (NG-001~005)

| # | Non-Goal | 备注 |
|---|---|---|
| NG-001 | 不做完整 IDE 跳转 (点节点跳代码) | Phase 2+ |
| NG-002 | 不做实时 git push 触发 (要 webhook) | Phase 3+ |
| NG-003 | 不做多仓 / monorepo 跨仓分析 | Phase 3+ |
| NG-004 | 不做节点/边手动编辑 (DB 写) | Phase 2+ (现仅看) |
| NG-005 | 不做 export PNG / SVG / JSON | Phase 2+ (现仅 in-browser 渲染) |

---

## §2 设计 (3 段: 数据模型 / Agent 形态 / 前端契约)

### 2.1 节点/边数据模型 (25 domain 投影, 1-hop)

**节点类型** (per 25 MRU + 22 domain-* crate 投影, 取并集去重):

| Node Kind | 来源 | 关键属性 | 示例 |
|---|---|---|---|
| `work_item` | MRU 5 | id, key, title, status, priority, assignee_id | `WI:PHYSIS-123` |
| `worktree` | MRU 10 | id, name, branch, status, last_event_at | `WT:wt-d5-impl` |
| `agent_session` | MRU 11 | id, agent_kind, status, token_usage | `AGT:claude-sonnet-42` |
| `change_set` | MRU 9 | id, title, status, symbol_index | `CS:cs-007` |
| `scm_repository` | MRU 16 | id, full_name, default_branch | `REPO:star-mono` |
| `pull_request` | MRU 16 | id, number, title, status, source_branch | `PR:#456` |
| `feedback` | MRU 12 | id, status, severity, category | `FB:fb-12` |
| `validation_case` | MRU 14 | id, kind, result, coverage | `VC:vc-89` |
| `comment` | MRU 6 | id, target_kind, author_id | `CM:cm-3` |
| `design_artifact` | §5b | id, title, status, version | `DA:da-001` |
| `identity` | MRU 3 | id, display_name, provider | `ID:usr-002` |
| `cratemodule` | 22 domain-* (cypher-only) | crate, module_path, kind (lib/bin/test) | `MOD:domain-work-item` |
| `symbol` | cypher-only (从 AST 提) | crate, file, line, kind (fn/struct/trait), name | `SYM:WorkItem::transition` |
| `tenant` | MRU 1 | id, name, plan | `T:physis-corp` |
| `project` | MRU 2 | id, key, name | `P:PHYSIS` |
| `workspace` | MRU 4 | id, name, kind | `WS:ws-001` |
| `permission_scheme` | MRU 7 | id, name, is_default | `PS:ps-default` |
| `workflow` | MRU 8 | id, name, is_default | `WF:wf-3state` |
| `local_runtime` | MRU 15 | id, hostname, status | `LR:dev-laptop-01` |
| `context_packet` | MRU 13 | id, kind, priority, token_estimate | `CP:cp-spec-7` |
| `notification` | MRU 13 | id, target_kind, status | `NT:nt-12` |
| `audit_event` | MRU 13 | id, action, actor_id | `AE:ae-345` |
| `automation_rule` | MRU 13 | id, trigger, action | `AR:ar-2` |
| `incident_record` | MRU 26 | id, title, source | `IR:ir-1` |
| `integration_webhook` | MRU 18 | id, source, status | `IW:iw-gh-3` |

**边类型** (typed edge label, 1-hop 内出现的):

| Edge | From | To | Cardinality | 业务含义 |
|---|---|---|---|---|
| `ASSIGNED_TO` | work_item | identity / agent_session | N:1 | 负责人 |
| `REPORTED_BY` | work_item | identity | N:1 | 报告人 |
| `IN_PROJECT` | work_item | project | N:1 | 所属项目 |
| `IN_WORKSPACE` | work_item | workspace | N:1 | 工作区 |
| `ON_WORKTREE` | work_item | worktree | N:1 | 执行 worktree |
| `PRODUCED` | work_item | change_set | 1:N | 产出 changeset |
| `HAS_FEEDBACK` | work_item | feedback | 1:N | 反馈 |
| `VALIDATED_BY` | work_item | validation_case | 1:N | 测试用例 |
| `COMMENTED_ON` | comment | work_item / pr / context_packet | N:1 | 评论 |
| `DESIGNED_BY` | work_item | design_artifact | 1:1 | 设计稿 |
| `RUNS_ON` | worktree | local_runtime | N:1 | 本地 runtime |
| `POWERS` | worktree | agent_session | 1:1 | agent session |
| `INTEGRATES` | agent_session | context_packet | 1:N | context |
| `REFERENCES` | work_item | symbol | N:1 (多对一 task) | 代码符号 (cypher 投影) |
| `LIVES_IN` | symbol | cratemodule | N:1 | symbol 归属 crate |
| `DEPENDS_ON` | cratemodule | cratemodule | N:M | crate 间 dep (Cargo.toml) |
| `INHERITS_FROM` | cratemodule | cratemodule | 1:N | DDD 限界上下文继承 |
| `TRIGGERS` | automation_rule | work_item | 1:N | 自动化 |
| `RAISED_INCIDENT` | incident_record | work_item | 1:N | 事件 |
| `WEBHOOK_FOR` | integration_webhook | scm_repository | N:1 | webhook 源 |
| `HAS_PR` | work_item | pull_request | 1:1..N | 关联 PR |
| `TARGETS_BRANCH` | pull_request | scm_repository / worktree | N:1 | 目标分支 |
| `WITH_PERMISSION` | work_item | permission_scheme | N:1 | 权限方案 |
| `FOLLOWING_WORKFLOW` | work_item | workflow | N:1 | workflow |

**1-hop 查询形状** (per work_item_id):

```cypher
MATCH (w:work_item {id: $work_item_id, tenant_id: $tenant_id})
OPTIONAL MATCH (w)-[r1]-(n1)  // 1 跳
WHERE n1:work_item OR n1:identity OR n1:worktree OR n1:agent_session
   OR n1:change_set OR n1:scm_repository OR n1:pull_request
   OR n1:feedback OR n1:validation_case OR n1:comment
   OR n1:design_artifact OR n1:cratemodule OR n1:symbol
   OR n1:project OR n1:workspace OR n1:context_packet
   OR n1:permission_scheme OR n1:workflow
OPTIONAL MATCH (n1)-[r2]-(n2)
WHERE n2:cratemodule OR n2:symbol
RETURN w, collect(DISTINCT n1) AS hops, collect(DISTINCT r1) AS edges,
       collect(DISTINCT n2) AS ext, collect(DISTINCT r2) AS ext_edges
```

> 注释: 1-hop 严格按 1 跳, 但 symbol/cratemodule 这类**代码侧**节点因为是 cypher 投影, 允许再扩一跳让"这个 worktree 涉及哪些 crate"可见。其它域严格 1 跳。

**节点视觉编码** (per cytoscape stylesheet):

| Node Kind | 颜色 (主) | 形状 | 大小 (px) |
|---|---|---|---|
| `work_item` (当前) | `#00f0ff` cyan (主) | round-rectangle | 64 |
| `work_item` (其它) | `#7c8499` ink-mute | round-rectangle | 48 |
| `worktree` | `#a78bfa` purple | hexagon | 48 |
| `agent_session` | `#f59e0b` warn | diamond | 44 |
| `change_set` | `#10b981` info | ellipse | 44 |
| `scm_repository` | `#22c55e` ok | round-triangle | 48 |
| `pull_request` | `#ec4899` magenta | round-pentagon | 44 |
| `feedback` | `#f43f5e` err | octagon | 40 |
| `validation_case` | `#3b82f6` blue | round-diamond | 40 |
| `comment` | `#94a3b8` slate | tag | 36 |
| `design_artifact` | `#fbbf24` amber | round-octagon | 44 |
| `identity` | `#0ea5e9` sky | circle | 40 |
| `cratemodule` | `#475569` ink | round-rectangle | 44 |
| `symbol` | `#64748b` ink-dim | ellipse | 28 |
| 其它 (project/workspace/...) | `#6b7280` ink | rectangle | 36 |

**边视觉编码**:

| Edge | 线色 | 宽 (px) | dash |
|---|---|---|---|
| 1-hop 关联 | `#00f0ff` cyan 半透 | 2 | solid |
| 2-hop 代码侧 (cratemodule / symbol) | `#475569` ink-mute | 1 | dotted |
| 当前 work_item 自身边 | `#00f0ff` cyan (全亮) | 3 | solid |
| 非当前 work_item 1 跳 | `#7c8499` ink-mute 半透 | 1.5 | dashed |

### 2.2 Agent 增量更新 memgraph 形态 (后台 LLM worker + 幂等+排他)

#### 2.2.1 触发

| 触发点 | 形态 | 频次 |
|---|---|---|
| 用户点 🕸 Arch 按钮 | modal 弹起 + 触发 `POST /graph/ensure-fresh` | 每次点击 (幂等 coalesce) |
| git push webhook (Phase 2+) | 自动触发 | 推代码时 |

#### 2.2.2 后台 LLM Worker 流程 (per 2026-09-02 02:00 JST 拍板 agentshape_opt1)

```text
[用户点 Arch 按钮]
       ↓
[ArchGraphModal 打开, loading state]
       ↓
POST /graph/ensure-fresh { work_item_id, tenant_id, source: "local" | "git" }
       ↓
[后端 GraphService.ensure_fresh()]
  ├─ 1. 取 work_item (per 25 MRU store)
  ├─ 2. 计算 fingerprint = sha256(
  │       work_item_id + worktree_branch + worktree_sha
  │       + source_kind + project_id
  │     )
  ├─ 3. 查 memgraph: 是否有 (work_item_id, fingerprint) 标记为 fresh?
  │     ├─ 是 → skip agent, 直接返回 1-hop 查询
  │     └─ 否 → 进入 4
  ├─ 4. advisory lock (per-work_item_id, 5 分钟 TTL)
  │     ├─ 抢到 → 继续 5
  │     └─ 没抢到 → 等待 + poll, 或返回 202 Accepted
  ├─ 5. spawn AgentSession (复用 14 状态机, per AGENTS.md §7 #2)
  │     agent_kind: "internal-vibe-coder"
  │     context: { work_item, worktree, source: local|git, fingerprint }
  │     ├─ step 1: 列文件 (git diff / local scan)
  │     ├─ step 2: AST 提 symbol + crate 引用
  │     ├─ step 3: LLM 推断 → 25 domain 节点/边
  │     ├─ step 4: 写 memgraph (Cypher UPSERT)
  │     │   MERGE (n:work_item { id: $wi_id })
  │     │   SET n.fingerprint = $fp, n.updated_at = now()
  │     │   MERGE (n)-[r:LIVES_IN]->(m:cratemodule { path: $path })
  │     │   ... 25 域逐项 MERGE
  │     ├─ step 5: 标 (work_item_id, fingerprint) fresh
  │     └─ step 6: 释放 lock
  └─ 7. 返回 1-hop 查询结果 (含新数据)
       ↓
[ArchGraphModal cytoscape 渲染 + 高亮]
```

#### 2.2.3 幂等 (per 2026-09-02 02:00 JST 拍板 concur_opt3)

| 维度 | 实现 | 证据 |
|---|---|---|
| **同 fingerprint skip** | `fingerprint = sha256(work_item_id + worktree_branch + worktree_sha + source_kind + project_id)`, memgraph `(:work_item).fingerprint` 字段 | fingerprint 不变 = 代码未变, skip agent |
| **同 work_item_id 多请求 coalesce** | 后端 in-process map: `pending[work_item_id] = oneshot::Receiver`, 多请求 await 同一结果 | 多人同点不重跑 |
| **LLM 输出 deterministic 化** | temperature=0, top_p=0.1, seed=work_item_id.hash() | LLM 抖动消解 |
| **memgraph 写幂等** | 全程 `MERGE ... ON MATCH SET ... ON CREATE SET ...` (Cypher) | 重复跑不破坏数据 |

#### 2.2.4 排他 (per 2026-09-02 02:00 JST 拍板 concur_opt3)

| 维度 | 实现 | TTL |
|---|---|---|
| **advisory lock** | Postgres `pg_try_advisory_xact_lock(work_item_id_hash)` 或 Redis `SETNX graph:lock:{work_item_id} 1 EX 300` | 5 分钟 (超长 fingerprint 计算) |
| **lock 失败回退** | 返 202 Accepted + `Retry-After: 3s`, 前端轮询 (max 30s) | - |
| **lock 释放** | 任一: agent 成功 / agent 失败 / 5 分钟 TTL 到期 | 显式 + 隐式双保险 |
| **死锁检测** | agent session 14 状态机里 `failed/cancelled` 自动释放 lock | - |

#### 2.2.5 数据源双支持 (per 2026-09-02 02:00 JST 拍板 dataorigin_opt3)

| source_kind | 取法 | 适用 |
|---|---|---|
| `"local"` | 直接读 `D:/Star/.worktrees/{wt}/` 路径, AST 提 | 本地开发, 当前 worktree |
| `"git"` | 拿 git remote URL + branch + sha, `libgit2` clone 到 ephemeral dir | 多人/CI/远程仓 |
| 后端选哪个 | 入参 `source: "local" \| "git"`, 客户端默认 `local` (per `ActorContext.local_runtime_id` 判定) | 灵活 |

### 2.3 前端渲染契约 (cytoscape.js + modal + 1-hop 高亮)

#### 2.3.1 触发点 (per 2026-09-02 00:36 JST 拍板 triggerscope_opt1)

- **位置**: KanbanCard 第 4 行 (priority + assignee) 旁, 加 🕸 Arch icon 按钮
- **行为**: `e.stopPropagation()` 阻止冒泡到 `onClick` (避免同时跳详情)
- **title**: "View architecture context"
- **size**: 11px, 同 Flag / User icon 风格

#### 2.3.2 Modal (ArchGraphModal 组件)

| 维度 | 取值 |
|---|---|
| 位置 | `position: fixed inset-0`, 居中, z-50 |
| 尺寸 | 默认 80vw × 80vh, 最小 800×600, Esc/背景/X 关闭 |
| 内容 | Header (work item key + title + "Refresh agent" 按钮) + cytoscape canvas + Footer (节点/边统计 + 关闭) |
| 加载态 | spinner + "Agent is analyzing code context..." |
| 错误态 | error 卡片 + retry 按钮 |
| 多租户 | `tenant_id` 必带 (per REQ-SEC-001, 13 类) |

#### 2.3.3 Cytoscape 配置 (per 2026-09-02 00:36 JST 拍板 highlightscope_opt1 + graphlib_opt1)

```ts
const cy = cytoscape({
  container: el,
  elements: [
    // 节点: { data: { id, label, kind, ...props } }
    // 边:   { data: { id, source, target, label, hop_level: 1|2 } }
  ],
  style: [
    // per §2.1 节点视觉编码
    {
      selector: 'node[kind = "work_item"][is_current = "true"]',
      style: { "background-color": "#00f0ff", width: 64, height: 64,
               "border-color": "#00f0ff", "border-width": 3,
               "font-size": 11, color: "#0a0d12" }
    },
    {
      selector: 'node[hop_level = "2"]',  // 非 1-hop 节点
      style: { opacity: 0.2 }
    },
    {
      selector: 'edge[hop_level = "1"]',
      style: { "line-color": "#00f0ff", "target-arrow-color": "#00f0ff",
               width: 2, "curve-style": "bezier" }
    },
    {
      selector: 'edge[hop_level = "2"]',
      style: { "line-color": "#475569", "line-style": "dotted", width: 1, opacity: 0.3 }
    },
  ],
  layout: { name: "cose", animate: true, padding: 30,
            idealEdgeLength: () => 100, nodeRepulsion: () => 8000 },
  // 交互
  minZoom: 0.3, maxZoom: 3, wheelSensitivity: 0.2,
});
```

#### 2.3.4 API 契约 (3 endpoint)

```ts
// 1. 确保数据最新 (幂等+排他, 后端触发 agent 增量)
POST /graph/ensure-fresh
  Request:  { work_item_id: Uuid, tenant_id: Uuid, source: "local" | "git" }
  Response 200: { status: "fresh", graph: GraphPayload }  // 数据已最新
  Response 202: { status: "running", retry_after_ms: 3000 }  // agent 正在跑
  Response 401: { error: "tenant_mismatch" }
  Response 404: { error: "work_item_not_found" }

// 2. 1-hop 查询 (读 memgraph)
POST /graph/cypher
  Request:  { work_item_id: Uuid, tenant_id: Uuid, max_hop: 1|2 }
  Response: { nodes: GraphNode[], edges: GraphEdge[], fingerprint: string,
              stats: { node_count, edge_count, kind_breakdown } }

// 3. 健康检查 (用于 modal 加载失败诊断)
GET /graph/health
  Response: { memgraph: "up"|"down", agent_runtime: "up"|"down",
              last_successful_run: Iso8601, queue_depth: number }
```

#### 2.3.5 数据形状 (TypeScript, 投影层)

```ts
// frontend/src/types/graph.ts
export type GraphNodeKind =
  | "work_item" | "worktree" | "agent_session" | "change_set"
  | "scm_repository" | "pull_request" | "feedback" | "validation_case"
  | "comment" | "design_artifact" | "identity" | "cratemodule"
  | "symbol" | "tenant" | "project" | "workspace"
  | "permission_scheme" | "workflow" | "local_runtime"
  | "context_packet" | "audit_event" | "automation_rule"
  | "incident_record" | "integration_webhook";

export interface GraphNode {
  id: string;            // 内部 id "WI:xxx"
  kind: GraphNodeKind;
  label: string;         // 显示文本
  is_current: boolean;   // 是否是当前 work_item
  hop_level: 1 | 2;      // 1 = 1-hop 邻居, 2 = 2-hop (仅 code-side)
  properties: Record<string, unknown>;  // 透传原始属性
}

export type GraphEdgeKind =
  | "ASSIGNED_TO" | "REPORTED_BY" | "IN_PROJECT" | "IN_WORKSPACE"
  | "ON_WORKTREE" | "PRODUCED" | "HAS_FEEDBACK" | "VALIDATED_BY"
  | "COMMENTED_ON" | "DESIGNED_BY" | "RUNS_ON" | "POWERS"
  | "INTEGRATES" | "REFERENCES" | "LIVES_IN" | "DEPENDS_ON"
  | "INHERITS_FROM" | "TRIGGERS" | "RAISED_INCIDENT" | "WEBHOOK_FOR"
  | "HAS_PR" | "TARGETS_BRANCH" | "WITH_PERMISSION" | "FOLLOWING_WORKFLOW";

export interface GraphEdge {
  id: string;
  kind: GraphEdgeKind;
  source: string;
  target: string;
  hop_level: 1 | 2;
}

export interface GraphPayload {
  work_item_id: Uuid;
  fingerprint: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  stats: {
    node_count: number;
    edge_count: number;
    kind_breakdown: Record<GraphNodeKind, number>;
  };
  generated_at: Iso8601;
}
```

---

## §3 阶段拆解 (per 守门 #1 阶段化 + token OLU)

### Phase 1: 前端契约 + mock 跑通 (本 session 内)

| # | 工作 | token 估 | 守门 |
|---|---|---|---|
| 1.1 | `package.json` 加 `cytoscape` + `@types/cytoscape` | 0.05M | cargo 不变, 走 tsc |
| 1.2 | `frontend/src/types/graph.ts` 节点/边类型 | 0.02M | tsc --noEmit 0 |
| 1.3 | `KanbanCard` 加 🕸 Arch 按钮 (stopPropagation) | 0.05M | KanbanCard.test pass |
| 1.4 | `ArchGraphModal` 组件 (cytoscape + 1-hop 高亮) | 0.4M | modal test + 视觉检查 |
| 1.5 | MSW mock: `POST /graph/ensure-fresh` + `POST /graph/cypher` + `GET /graph/health` | 0.3M | handler test pass |
| 1.6 | Playwright 冒烟 (点 Arch 按钮 → modal 弹起 → 看到高亮) | 0.1M | e2e 1 路径 pass |
| 1.7 | `ARCH-AGENT-GRAPH-001-REPORT.md` 7 段 | 0.1M | docs 同步 |
| **小计** | | **~1.0M** | |

### Phase 2: 后端 LLM worker (跨 session, 等 P3-B 拍板)

| # | 工作 | token 估 |
|---|---|---|
| 2.1 | `crates/star-graph-agent/` 新 crate (per 22 domain 平行) | 1.5M |
| 2.2 | LLM worker (internal-vibe-coder 复用) | 2.0M |
| 2.3 | advisory lock + 幂等 (fingerprint) | 0.5M |
| 2.4 | agent-runtime 14 状态机集成 | 0.8M |
| **小计** | | **~4.8M** |

### Phase 3: memgraph 真实例 (跨 session, 等部署拍板)

| # | 工作 | token 估 |
|---|---|---|
| 3.1 | memgraph 实例 + Bolt/HTTP client | 0.6M |
| 3.2 | 25 domain schema + 索引 | 0.4M |
| 3.3 | 多租户 RLS 13 类 (per §6.1 REQ-SEC-001) | 0.6M |
| 3.4 | 备份 / 恢复 / 监控 | 0.4M |
| **小计** | | **~2.0M** |

**总估**: ~7.8M tokens (per STAR-OLU-001 v0.1 1 SRE·周 = 1.2M, 约 6.5 周)

---

## §4 守门对齐 (per AGENTS.md §4 13+ 守门)

| # | 守门 | 本 ADR 体现 |
|---|---|---|
| 1 | 守门 #1 禁回溯叙事 | §1.1/§2.2 引用全部 commit hash + 文档路径, 无"per X 历史形态" |
| 2 | 守门 #2 bc23d6c 保留 | 不动 bc23d6c |
| 3 | 守门 #3 5 域独立 Lead | §0 dual-use 提醒: 5 域 ≠ 25 domain, 不建立映射 |
| 4 | 守门 #4 token-OLU | §3 阶段 token 估 1.0M / 4.8M / 2.0M |
| 5 | 守门 #5 环境变量安全 | 不打印 env value, 走 $env:VAR pipe |
| 6 | 守门 #6 PowerShell only | Phase 1 实装时验证 |
| 7 | 守门 #7 0 unsafe | 后续 Phase 2/3 Rust 代码 `unsafe_code = "forbid"` |
| 8 | 守门 #8 不沿用 bc23d6c 叙事 | §1.1 用 9/2 实证, 不引 bc23d6c |
| 9 | 守门 #9 子代理实证 | Phase 1 root 直实装, 不委派 (P3-A.6/A.7 实证 RPC 失败) |
| 10 | 守门 #10 代签规则 | 本 ADR 审批 = Mavis 接手代签 (per 19:39 JST 用户授权) |
| 11 | 守门 #11 缺标比错标 | §1.3 NG-001~005 显式列非目标 |
| 12 | 守门 #12 文档治理 | 本 ADR commit 必引, 守门 #12 实证待 Phase 1 commit |
| 13 | 守门 #13 DB 三類横展開 | Phase 3 schema 落: Work (TaskRun pending 短 TTL) / Transaction (fingerprint 审计 append-only) / Master (Node/Edge SCD Type 2) |

---

## §5 引用基线

| 引用 | 路径 | 版本 |
|---|---|---|
| AGENTS.md | `D:/Star/AGENTS.md` | v0.15 (per commit 29692a7) |
| HANDOFF-ST-001 | `D:/Star/..\..\..\..\reports\HANDOFF-ST-001.md` | v0.4 |
| ADR-0040 domain-batch | `docs/architecture/2026-08-26-upgrade/adr/0040-domain-batch.md` | v0.1 |
| ADR-0032 MCP Transport | `docs/architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md` | - |
| API Design 25 MRU | `docs/api-design.md` | - |
| KanbanCard | `frontend/src/components/board/KanbanCard.tsx` | per 7d85c34 |
| WorkItemDetailDrawer | `frontend/src/components/board/WorkItemDetailDrawer.tsx` | per 9/1 12:07 JST 拍板 |

---

## §6 签字栏

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-02 | Mavis 接手代签 (per 19:39/20:56/21:59 JST 用户授权) |
| SRE Lead | ⏳ 待签 | - | 5 域独立真实身份 (per 8/21 JST 拒绝兼任), DDD Review 阶段补 |
| 平台 | ⏳ 待签 | - | 同上 |
| 评审主持 | ⏳ 待签 | - | 同上 |
| PM | ⏳ 待签 | - | 同上 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 | 初版: 25 domain 节点/边模型 + 1-hop 查询 + Agent 增量 + 幂等+排他 + cytoscape modal + 3 阶段拆解 (1.0M + 4.8M + 2.0M = 7.8M tokens) | 2026-09-02 00:33/00:36/02:00 JST Ulysses 4 轮拍板 (cypher + memgraph + agent + 幂等排他 + cytoscape + modal + 1-hop + 后台 LLM + 幂等+排他 + 本地+git 双源) |
