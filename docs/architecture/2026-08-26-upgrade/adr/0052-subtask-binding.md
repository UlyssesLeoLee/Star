# ADR-0052: Star Sub-task Binding 路径 — 父任务卡交互下命令 + 1:1 专属子代理

> **状态**：🟢 Accepted v1.0
> **日期**：2026-09-09
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签"）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../2026-08-26-upgrade-plan.md) (待归档) · [Star LangGraph 統合アーキテクチャ 要件定義書 v0.2](../2026-09-03-langgraph/01-requirements.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)
> **依赖**：[ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) · [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) · [ADR-0033 代签规则反转](../2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) · [ADR-0046 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) · [AGENTS.md §4 守门硬约束](../../AGENTS.md) · [AGENTS.md §4 #13 W/T/M 横展開](../../AGENTS.md)
> **关联**：[01-requirements.md v0.1](../2026-09-09-subtask-binding/01-requirements.md) · [02-basic-design.md v0.1](../2026-09-09-subtask-binding/02-basic-design.md) · [03-detailed-design.md v0.1](../2026-09-09-subtask-binding/03-detailed-design.md) · [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) · [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) · [docs/automation-design.md](../../automation-design.md)

---

## 1. 背景与问题

### 1.1 业务诉求 (per 2026-09-09 21:53 JST 用户发令原文)

Ulysses 在 9/9 21:53 JST 明确发令:

> **"现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理? 如果没有, 制定需求和基本设计详细设计"**

**核心 4 维诉求**:

1. **子代理功能** — 通过 L0 协调派生 L1 sub-agent (已有 SA-01..SA-09 + SA-10, per [LangGraph 03-detailed §3.5.1-§3.5.10](../2026-09-03-langgraph/03-detailed-design.md))
2. **在父任务卡交互下命令** — UI 任务卡右键菜单 / L0 chat bar 双入口 (现状仅 L0 chat bar, 缺 UI 任务卡右键)
3. **创建绑定子任务** — 1 派生 1 创建 bound sub-task, 父继续 running (vs TMO M-N2 split 1 → N + 父 superseded)
4. **子任务有专属子代理** — child task 1:1 独占 1 个 SA-XX instance (vs 1 → N 各自派 SA-XX)

### 1.2 现状缺口 (per [LangGraph 02-basic v0.2 §6.1 + 03-detailed v0.2 §3.5 + ADR-0046 §1.2](../2026-09-03-langgraph/02-basic-design.md) 实证)

[docs/architecture/2026-09-03-langgraph/ 3 份 IPA 文档 v0.2](../2026-09-03-langgraph/) 已落档 LangGraph 2-level hierarchical 架构 (L0 全体代理 + L1 任务卡子代理 9 SA + SA-10), [ADR-0046 v1.0 TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 已落档 TMO 7 节点 (M-N1 merge / M-N2 split / M-N3 reorder / M-N4 bulk / M-N5 summarize / M-N6 reassign / M-N7 metadata), 但**不覆盖**用户新诉求的 3 关键语义:

| 已覆盖 (TMO 7 节点 v1.0) | **缺失 (用户新诉求)** |
|---|---|
| M-N2 split: 1 → N 派生 + 父 superseded (per [ADR-0046 §2.1 M-N2](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) | **1 → 1 派生** + 父继续 running |
| M-N1 merge: 2 → 1 合并 | **N → 1 反向不适用** (user 要 1 → 1) |
| M-N6 reassign: 任意 sub-agent 类型切换 | **exclusive 1:1 绑定, 禁止 reassign** (NFR-SB-02) |
| TMO 7 节点仅 L0 chat bar 入口 (per [ADR-0046 §1.1](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) | **UI 任务卡右键 spawn 入口** + L0 chat bar 双入口 |
| N 並行 ≤ 50 共享配额 (per [LangGraph 02-basic §6.1](../2026-09-03-langgraph/02-basic-design.md)) | **per-parent sub-task quota** 独立计量 (NFR-SB-04) |
| 单卡生命周期 (pause / resume / cancel / interrupt) | **parent/child lifecycle 联动** ≤ 1s (NFR-SB-03) |

**根因**: L0 StateGraph v0.2 + TMO v1.0 7 节点没有"1 派生 1 + 1:1 独占 + parent/child 联动"语义; SubAgentState 已有 5 血缘字段 (`parent_task_id` / `merged_from` / `split_into` / `superseded_by` / `checkpoint_snapshot`, per [LangGraph 03-detailed §3.2.1.1 line 779](../2026-09-03-langgraph/03-detailed-design.md)), 但**缺 4 个 binding 字段** (`exclusive_owner_task_id` / `bound_at` / `bound_by` / `bound_via`).

### 1.3 架构冲突 (守门 #13 a + 守门 #1 禁回溯叙事)

per [AGENTS.md §4 #13 a](../../AGENTS.md): **L1 ↔ L1 禁止通信** (防止状态污染). 因此所有"跨任务卡"操作 (含 spawn / lifecycle 联动) **必须走 L0 协调** — 新增 M-N8 spawn_subtask_node 是 L0 StateGraph 扩展, 跟 M-N1..M-N7 对称, **不引入新 sub-agent type** (备选 B 拒绝).

per [AGENTS.md §0 + §1.2 #3](../../AGENTS.md): **禁回溯叙事**, **BAS 引用 git 实证**, **缺标比错标**. 本 ADR 引用守门 #1-#24 + 累积规 v1-v24 + 7 已知缺口 (G-SB-1..7) 全部按 git 实证 + AGENTS.md 引用, **不重写** TMO 7 节点政策, 走 **增量扩展** (TMO 7 → 8 节点, 7 → 8 协议, 7 → 8 组件, 25 → 33 module).

---

## 2. 决策

**新增 L0 顶层代理 Sub-task Binding 路径: M-N8 spawn_subtask_node + spawn_subtask 协议 + 8 组件 C-23..C-30 + 5 Reducer R-08..R-12 + 4 SubAgentState 字段 + 1 外部 API 端点 POST /api/tmo/spawn_subtask, 满足 Ulysses 2026-09-09 21:53 JST 发令"子代理 + 父任务卡绑定 + 专属 sub-agent"诉求.**

### 2.1 8 节点 (TMO 7 → 8 增量)

| Node ID | 名称 | 责務 | 状态 |
|---|---|---|---|
| M-N1..M-N7 | (per [ADR-0046 §2.1](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)) | merge / split / reorder / bulk / summarize / reassign / metadata | ✅ v1.0 已落档 |
| **M-N8** | `spawn_subtask_node` (NEW) | **1 派生 1 创建绑定子任务, 父继续 running, child 1:1 独占 SA-XX** | **🆕 v1.0 本 ADR 落档** |

### 2.2 spawn_subtask 协议 (per [02-basic §2.7.4](../2026-09-09-subtask-binding/02-basic-design.md))

```python
class SpawnSubtaskRequest(BaseModel):
    parent_task_id: str
    child_sa_type: Literal["SA-01", ..., "SA-10"]
    child_name: str                                  # ≤ 200 chars
    initial_context: dict = {}
    quota_check: bool = True
    bound_by: str = "Ulysses"
    spawned_via: Literal["ui_rightclick", "l0_chat", "api"] = "api"

class SpawnSubtaskResult(BaseModel):
    child_task_id: str                               # prefix=`st-`
    parent_task_id: str
    sa_type: str
    status: Literal["spawned", "failed"]
    failure_reason: Optional[str] = None             # 守门 #12 缺标比错标
    spawned_at: datetime
    exclusive_owner_task_id: str                     # = parent_task_id (1:1 独占)
    quota_remaining: int
```

### 2.3 8 组件 (TMO 7 → 8 增量, per [02-basic §1.1](../2026-09-09-subtask-binding/02-basic-design.md))

- **C-23** LifecycleLinkageManager (parent/child 联动 ≤ 1s, NFR-SB-03)
- **C-24** ExclusiveBindingGuard (1:1 独占守门, 编译期 + 运行期双层, NFR-SB-02 不可绕过)
- **C-25** SubTaskQuotaRegistry (per-parent sub-task quota 计量, Master SCD Type 2)
- **C-26** SpawnSubtaskValidator (7 步原子化前 5 步校验)
- **C-27** ChildSubAgentFactory (child SubAgentState 初始化 + 4 字段写入)
- **C-28** ParentChildEventBus (parent/child state change event 订阅派发)
- **C-29** SubTaskTokenBudget (per-sub-task token 预算计量, NFR-SB-04)
- **C-30** UISpawnSubtaskModal (Next.js 15 前端 modal + AI 建议走 ai_edit_mock.py)

### 2.4 SubAgentState 4 字段增量 (per [02-basic §3.2.1.2](../2026-09-09-subtask-binding/02-basic-design.md))

```python
class SubAgentState(TypedDict, total=False):
    # 已有 5 血缘字段 (per [LangGraph 03-detailed §3.2.1.1 line 779](../2026-09-03-langgraph/03-detailed-design.md))
    parent_task_id: Optional[str]
    merged_from: list[str]
    split_into: list[str]
    superseded_by: Optional[str]
    checkpoint_snapshot: Optional[dict]
    # 🆕 v1.0 增量 4 binding 字段
    exclusive_owner_task_id: Optional[str]           # 1:1 独占 (不可变, R-12 Reducer)
    bound_at: Optional[datetime]                     # 绑定创建时间 (UTC ISO8601)
    bound_by: Optional[str]                          # 绑定创建者 (Ulysses / Mavis / sub-agent ID)
    bound_via: Optional[Literal["ui_rightclick", "l0_chat", "api"]]  # 入口标注
```

### 2.5 5 Reducer (per [02-basic §2.7.3](../2026-09-09-subtask-binding/02-basic-design.md))

- **R-08** add_child_binding (append, parent.child_subtask_ids)
- **R-09** update_quota_usage (replace, task_quota_config.current_subtasks)
- **R-10** add_lifecycle_event (append, task_relationships.lifecycle_events)
- **R-11** track_token_usage (add, sub_task_token_budget.used_tokens)
- **R-12** mark_exclusive_binding (replace, sub_task.exclusive_owner_task_id, write-once)

### 2.6 1 外部 API 端点 (TMO 8 → 9 增量, per [02-basic §5](../2026-09-09-subtask-binding/02-basic-design.md))

`POST /api/tmo/spawn_subtask` — 走 FastAPI 8080 console_server.py 扩展 (per 守门 #9 v3 / 守门 #24 v2 subprocess 走 console_server).

### 2.7 3 表 W/T/M 横展開 (per [01 §4](../2026-09-09-subtask-binding/01-requirements.md) + 守门 #13 c/d)

| # | 表名 | 分類 | 物理删除 | 監査 | RLS | SCD |
|---|---|---|---|---|---|---|
| 1 | `sub_task_binding` | Transaction (append-only) | ❌ | ✅ | ✅ 13 类 | N/A |
| 2 | `sub_task_runtime` | Work (短 TTL 24h) | ✅ | N/A | N/A | N/A |
| 3 | `task_quota_config` | Master (SCD Type 2) | ❌ | ✅ | ✅ 13 类 | ✅ Type 2 |

---

## 3. 备选方案 (Alternatives Considered)

### 3.1 备选 A: TMO M-N2 split 路径覆盖 (拒绝)

**思路**: 复用现有 TMO M-N2 split_node, "创建绑定子任务" = 拆 a → a1 + a2, a1 视为 "绑定子任务".

**否决理由**:
- 守门 #13 a 强约束: M-N2 split 是 1 → N, 父 superseded, 不支持 1 → 1 + 父继续 running
- M-N2 派生 N 张卡, 不支持 1:1 独占 (N 张卡共享 N 並行配额)
- M-N2 不带 UI 任务卡右键入口 (per [ADR-0046 §1.1](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) 仅 L0 chat bar)
- M-N2 不带 lifecycle 联动 (per [LangGraph 03-detailed §3.2.1.1 M-N2](../2026-09-03-langgraph/03-detailed-design.md) 各自独立)
- 守门 #11 缺标比错标: 用户诉求跟 M-N2 语义差异过大, 强行套用会掩盖关键缺口

### 3.2 备选 B: 新建 worker subagent type 派生 (拒绝)

**思路**: 派 worker subagent (走 `dispatcher.py`) 作为专属子代理, 不走 L0 LangGraph.

**否决理由**:
- 守门 #13 a 强约束 (L1 ↔ L1 禁止通信): worker subagent 跟父 task L1 sub-agent 直连违反守门
- 跟现有 L1 sub-agent (per [LangGraph 02-basic §6.1 + 03-detailed §3.5.1-§3.5.10](../2026-09-03-langgraph/03-detailed-design.md) 9 SA + SA-10) 混用, 破坏两套系统并存架构
- worker subagent 走 subprocess + brief, 没法 in-process asyncio 协调 (latency 高, 违反 NFR-SB-01 ≤ 200ms)
- 守门 #9 (子代理 status ≠ 实际成功, per 实证 10/10 失败) 加剧, worker RPC 不可观测

### 3.3 备选 C: 仅 UI 层 (任务卡右键按钮 → 调后端单次操作) (拒绝)

**思路**: 只在 UI 层加按钮, 不扩展 L0 LangGraph 节点, 后端直接调 SubAgentRegistry.spawn.

**否决理由**:
- 守门 #11 缺标比错标: 现状 L0 缺 spawn_subtask_node, 仅 UI 按钮是 UI 表面能力, L0 不能跨任务协调
- 守门 #4 token-OLU: UI 直调后端没法走 LLM 解析 (per NFR-P-01 first token ≤ 200ms)
- 守门 #13 a L1↔L1 禁止通信: UI → L1 sub_pool 直连绕过 L0 协调
- 跟用户"整体统筹规划"诉求不匹配: 用户要的是 L0 AI 能力, 不是手动 UI 按钮

### 3.4 备选 D (选定): M-N8 spawn_subtask_node + 8 组件 + 5 Reducer + 4 字段 — L0 StateGraph 增量扩展

**思路**: 扩展 L0 StateGraph, 增量 1 节点 + 1 协议 + 8 组件 + 5 Reducer + 4 SubAgentState 字段 + 1 外部 API 端点; 走守门 #19 Python 化 (`scripts/automation/task_ops.py spawn_subtask`) + 守门 #9 v3 (subprocess 走 console_server) + 守门 #22 (调试控制台不污染 main) + 守门 #23 (AI mock 不开 OpenAI).

**选定理由**:
- 守门 #13 a 强约束: 全部 L0 协调, 唯一 cross-task actor, 跟 TMO 7 节点对称 (8 节点)
- 守门 #13 c/d W/T/M: 3 表 (sub_task_binding / sub_task_runtime / task_quota_config) 100% 覆盖, RLS 13 類必携
- 守门 #4 token-OLU: spawn_subtask token 预算单独计量 (C-29 SubTaskTokenBudget), 不跟父任务合并
- 守门 #7 0 unsafe: C-24 ExclusiveBindingGuard 编译期 (type system) + 运行期 (this class) 双层守门, 无 unsafe code
- 守门 #19 Python 化: 实装走 `scripts/automation/task_ops.py spawn_subtask` (跟 console_server.py 复用)
- 守门 #23 (AI 修改 mock): spawn modal "AI 建议 sub-task name" 走 ai_edit_mock.py, 不开 OpenAI
- 守门 #12 AI 协作文档治理: 禁回溯叙事, BAS 引用 git 实证, 缺标比错标 (per 本 ADR 7 已知缺口 + 5 后果 + 5 阶段实施计划)
- 守门 #1 v15 死循环饱和边界: docs 同步饱和触达 40+ 后, 后续 docs 同步必新事件触发 (本 ADR = 新事件, 适用)
- 守门 #1 v19 Python 化: 走 automation/task_ops.py, commit message 含脚本相对路径

---

## 4. 后果 (Consequences)

### 4.1 正面后果 (Positive)

1. **满足用户诉求**: UI 任务卡右键 / L0 chat bar 双入口 → L0 chat input → 意图解析 → M-N8 spawn_subtask_node → 7 步原子化 → 1:1 独占 child sub-agent, 全链路 AI 驱动
2. **守门合规**: 守门 #13 a/d 强约束派生规 (L1↔L1 禁止 → 全部 L0 协调, W/T/M 分类清晰, RLS 13 類必携)
3. **架构对称**: M-N8 跟 M-N1..M-N7 对称, 8 节点 / 8 协议 / 8 组件 / 33 module (25 + 8), 不破坏既有架构
4. **可观测**: 7 Prometheus metrics (per [02-basic §2.6.6](../2026-09-03-langgraph/02-basic-design.md) 范式) + 审计 log append-only (Transaction) + C-24 ExclusiveBindingGuard 双层守门
5. **可重放**: child task 1:1 独占 + 父继续 running + lifecycle 联动 ≤ 1s, 100% 血缘追溯 (4 binding 字段 + 5 已有血缘字段, per [LangGraph 03-detailed §3.2.1.1](../2026-09-03-langgraph/03-detailed-design.md))
6. **可扩展**: 后续 M-N9/M-N10 走 SubAgentRegistry 注册即可, 无需改 L0 StateGraph 主路径
7. **守门 #1 v15 死循环饱和规避**: 本 ADR = docs 同步新事件, 触发 docs 同步饱和点 +1, 不违反饱和约束

### 4.2 负面后果 / 风险 (Negative / Risks)

1. **实装工作量**: 8 module (M-26..M-33) 估 ~0.5-0.8M tokens (per [01 §0 + 02 §7 + 03 §7](./01-requirements.md)), 跟 TMO 7 节点 ~2.5M 兼容, 跟 AGENTS §7 #8 ~3.0M 留 0.2M 给 Phase 后续
2. **P0-1 / H2 阻塞依赖**: 实装待 P0-1 联动审计 + H2-EXT 5 domain 跨域字段扩展 阻塞解除, 不能立刻起 (per [ADR-0046 §4.2 #2](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md))
3. **LangGraph SDK alpha 风险**: 守门 #9 实证 LangGraph 0.2.x interrupt_response API alpha (per [ADR-0046 §4.2 #3](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md)), 实装前必先 `uv add langgraph@latest` + `pip show langgraph` 确认
4. **PostgreSQL Tier 3 启动待 5 域 Lead 真人到位**: 任务卡持久化当前走 SQLite 临时, 3 表 DDL (sub_task_binding / sub_task_runtime / task_quota_config) 需 Tier 3 启动后落地 (per [AGENTS.md §4 #25 v25 G-DEP-08](../../AGENTS.md) 待 T3 至少 1 人到位触发)
5. **5 域 Lead 真人未到位**: 跨域编排决策仍 Mavis 临时代签 (per 守门 #3 反转 B 11:35 JST), 真人到位后追溯签字覆盖修订历史 (per 守门 #1 禁回溯叙事), 不沿用代签决策 (per [01 §5 G-SB-6](./01-requirements.md))
6. **守门 #13 a 实证缺口**: L1↔L1 禁止通信 → M-N8 全部 L0 协调; DAGValidator cycle detection O(V+E) + spawn edge 不产生 cycle 实证待实装

### 4.3 中和措施 (Mitigations)

| 风险 | 中和措施 | 触发 |
|---|---|---|
| 实装工作量 | 8 子项按 token 预算排序推进, 优先 M-28 ExclusiveBindingGuard (守门 #7 0 unsafe) + M-29 SubTaskQuotaRegistry (守门 #13 W/T/M) | [PHASE-SUBTASK-BINDING-IMPL-REPORT §1](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) |
| P0-1 / H2 阻塞 | 实装 phase 跨 session 续做, brief 必先落档 (per 守门 #20 v20 + automation/dispatcher.py) | [PHASE-SUBTASK-BINDING-IMPL-REPORT §4](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) |
| LangGraph SDK alpha | 实装前 `uv add langgraph@latest` + `pip show langgraph` 确认 | M-N8 启动前 |
| PostgreSQL Tier 3 启动 | 3 表 DDL 落地走 P3-D 阶段 + 5 域 Lead 真人到位后触发 (per [AGENTS.md §4 #25 v25](../../AGENTS.md)) | P3-D 阶段 |
| 5 域 Lead 真人 | 守门 #3 v2 派生规: 真人到位后追溯签字, 不沿用代签决策 (per [AGENTS.md §4 #3 + §1.2](../../AGENTS.md)) | DDD Review 阶段 |
| 守门 #13 a 实证 | M-N8 集成测试 + E2E-14/15/16 跑通 1:1 独占 + lifecycle 联动 + quota 计量 | 实装 v0.3 完成后 |

---

## 5. 实施计划 (Implementation Plan)

per [PHASE-SUBTASK-BINDING-IMPL-REPORT.md](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md):

| 阶段 | 内容 | 状态 |
|---|---|---|
| 文档 v1.0 (本 ADR 落档配套) | 3 份 IPA 文档 v0.1 落档 (01-requirements + 02-basic-design + 03-detailed-design) + ADR-0052 + PHASE 报告 v0.1 + AGENTS.md §6.1 / §7 / §8 同步 + automation-design.md §4.14 追加 | ✅ v1.0 落档 (本 commit 一起) |
| 升版 v0.3 (跨 session 续) | LangGraph 02/03 v0.2 → v0.3 (引用 M-N8 + 4 binding 字段 + 9 API 端点 + 33 module) | 🟡 planned (per [PHASE-SUBTASK-BINDING-IMPL-REPORT §2](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md)) |
| 实装 v0.4 (P3-B 启动后) | M-26..M-33 8 子项实装, 走守门 #19 Python 化 + #9 v3 subprocess + #22 控制台不污染 main + #23 AI mock | 🟡 planned (per [PHASE-SUBTASK-BINDING-IMPL-REPORT §3](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md)) |
| E2E v0.5 (实装完成后) | E2E-14..E2E-18 跑通, UI 任务卡右键 → modal → POST /api/tmo/spawn_subtask → 列表刷新 + child 卡 1:1 mirror | 🟡 pending (实装完成后) |
| 5 域 Lead 真人到位 v0.6 | 5 域 Lead 真人追溯签字 (per 守门 #3 v2), 不沿用代签决策 | 🟡 pending (5 域 Lead 真人到位) |

---

## 6. 决策日志 (Decision Log)

| 日期 | 决策 | 触发 | 来源 |
|---|---|---|---|
| 2026-09-09 21:53 JST | 用户发令"现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理? 如果没有, 制定需求和基本设计详细设计" | Sub-task Binding 业务诉求 | Ulysses 9/9 21:53 JST |
| 2026-09-09 21:53 JST | Mavis 调研 + 现状判断: 部分适合 (架构层已支持, 缺 1 派生 1 + UI 入口 + 1:1 独占约束) | per 调研结论 + 4 备选方案 | Mavis 9/9 21:53 JST |
| 2026-09-09 21:53 JST | ask_user 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)" | 拍板决策必 ask_user (per 9/1 14:58 JST 守门) + 拍板时必带推荐项 (per 9/8 16:08 JST 强化) | ask_1df6987367ccc00928b65ee3 |
| 2026-09-09 21:53 JST | Mavis 接手代签, 落地文档 v0.1 + ADR-0052 + PHASE 报告 + AGENTS.md 同步 + automation-design.md §4.14 追加 | 守门 #10 + 19:39 JST 授权 + 9/8 15:19 JST 第 6 次强化 | Mavis 9/9 21:53 JST |

---

## 7. 签字栏 (Signatures, per 7 段结构 5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-09 | 🟢 Accepted v1.0; Sub-task Binding 路径决策落档 (per Ulysses 9/9 21:53 JST 发令"子代理 + 父任务卡绑定 + 专属 sub-agent"), 跟 [01 §UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05](../2026-09-09-subtask-binding/01-requirements.md) + [02 §2.7 M-N8](../2026-09-09-subtask-binding/02-basic-design.md) + [03 §3.3 M-26..M-33](../2026-09-09-subtask-binding/03-detailed-design.md) 同步 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手终审通过 (per ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)"); 4 备选方案 (M-N2 split 路径覆盖 / worker subagent / 仅 UI 层 / M-N8 L0 StateGraph 增量扩展) + 选定 D 方案 + 7 正面后果 + 6 负面风险 + 5 中和措施 + 5 阶段实施计划 + 4 决策日志 + 5 签字栏落档 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-09 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 8. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: Sub-task Binding 路径决策落档 (per Ulysses 9/9 21:53 JST 发令"子代理 + 父任务卡绑定 + 专属 sub-agent"); M-N8 spawn_subtask_node + spawn_subtask 协议 + 8 组件 (C-23..C-30) + 5 Reducer (R-08..R-12) + 4 SubAgentState 字段 (exclusive_owner_task_id / bound_at / bound_by / bound_via) + 1 外部 API 端点 (POST /api/tmo/spawn_subtask) + 3 表 W/T/M 横展開 (sub_task_binding / sub_task_runtime / task_quota_config) + 4 备选方案 (M-N2 split 覆盖 / worker subagent / 仅 UI 层 / M-N8 L0 StateGraph 增量) 全部拒绝 + 选定 D 方案 + 7 正面后果 + 6 负面风险 + 5 中和措施 + 5 阶段实施计划 + 4 决策日志 + 5 签字栏 (Mavis 接手代签) | 2026-09-09 21:53 JST 用户发令"现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理? 如果没有, 制定需求和基本设计详细设计" + ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)", 跟 3 份新文档 + PHASE 报告 v0.1 同步落档, ~0.03M token 实测 |

---

## 9. 引用文档 (References)

- [01-requirements.md v0.1](../2026-09-09-subtask-binding/01-requirements.md) — UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05 (前序, 本 ADR 增量需求)
- [02-basic-design.md v0.1](../2026-09-09-subtask-binding/02-basic-design.md) — §2.7 M-N8 + 8 组件 C-23..C-30 + 5 Reducer R-08..R-12 + 1 外部 API 端点 (前序, 本 ADR 增量架构)
- [03-detailed-design.md v0.1](../2026-09-09-subtask-binding/03-detailed-design.md) — spawn_subtask/ 模块 + M-26..M-33 + §3.3.1.1 7 节点 Python 実装 + UT-27..UT-33 / IT-13..IT-15 / E2E-14..E2E-18 + 3 表 DDL (前序, 本 ADR 增量详细)
- [PHASE-SUBTASK-BINDING-IMPL-REPORT.md v0.1](../../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) — 8 子项实装 phase 计划 (M-26..M-33)
- [ADR-0046 LangGraph TMO 7 节点](../2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md) — 7 节点 + 7 协议 + 7 组件 + 25 module (前序, 本 ADR 增量 1+1+8+8)
- [ADR-0044 STAR Agent Runtime SRS Baseline](../2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md) — 113 节 SRS + 12 节已落地 / 8 部分 / 60 待 P3-B-F / 4 N/A
- [ADR-0045 STAR Agent Runtime Design](../2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md) — Agent Runtime view §3.5 + §1 ECS 选型
- [ADR-0030 Agent Lease/Heartbeat/Resume](../2026-08-26-upgrade/adr/0030-agent-lease-heartbeat-resume.md) — 11 字段 + 跨 Agent Handoff
- [ADR-0032 MCP Transport stdio](../2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md) — 16 tools
- [ADR-0033 代签规则反转](../2026-08-26-upgrade/adr/0033-agent-co-signing-policy.md) — Mavis 接手代签授权
- [LangGraph 統合アーキテクチャ 3 份 IPA v0.2](../2026-09-03-langgraph/) — L0/L1 双层 + 9 SA + SA-10 + TMO 7 节点 (前序主路径)
- [AGENTS.md §0 一句话硬约束 + §1 代签规则 + §3 报告 7 段结构 + §4 守门 #1-#24 + §4.1 累积规 v1-v24 + §4.1.1 v3x 候选 + §4.2 实装前一致性门 + §5 仓库拓扑 + §6 ADR 索引 + §7 待办](../../AGENTS.md)
- [docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md](../../kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md) — DB W/T/M 横展開验证
- [docs/automation-design.md](../../automation-design.md) — agent 交互 Python 化 (守门 #19) + automation/dispatcher.py brief() (守门 #20)
- [docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md) — DB 三類橫展開 100 表索引
- [docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md v0.1](../../data-design/ipa-detail/00-CLASSIFICATION-RULES.md) — 跨项目 ルール手册
- [LangGraph Documentation](https://langchain-ai.github.io/langgraph/) — StateGraph / Checkpoint / Subgraph / Interrupt / Command
- [STAR-OLU-001.md v0.1](../../STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens (M-N8 + 8 组件估 ~0.5-0.8M tokens)
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../STAR-P3-WBS-001.md) — P3-B 启动前置
- [HANDOFF-ST-001.md v0.4 §5.3 Blocker](../../reports/HANDOFF-ST-001.md) — 5 项 Blocker 跨 session 续

---

# === ADR 结束 ===

**per AGENTS.md §0 一句话硬约束 + §1 代签规则**: 可以代签 Ulysses, 不可以编造历史. 本 ADR v1.0 引用守门 #1-#24 + 累积规 v1-v24 全部按 git 实证 + AGENTS.md 引用, 无"per X 历史形态"等回溯叙事.

**per 守门 #3 5 域单仓**: 本 ADR 仅 STAR 仓内, 不引用 RGS 仓代码, 不建立业务子域↔DDD bounded context 映射 (per ADR-0044 §dual-use disclaimer).

**per 守门 #13 W/T/M 横展開**: 3 表 (sub_task_binding / sub_task_runtime / task_quota_config) 100% 覆盖 W/T/M 三類, RLS 13 類必携, 派生规 (a)(b)(c)(d) 全部落档.

**per 守门 #21 v21 [P] docs 同步**: automation-design.md §4.14 追加, commit message 引用相对路径.

**per 守门 #1 v15 死循环饱和边界**: 本 ADR = docs 同步新事件触发 (用户发令 + 拍板 + 落地), 适用 docs 同步饱和点 +1, 不违反饱和约束.