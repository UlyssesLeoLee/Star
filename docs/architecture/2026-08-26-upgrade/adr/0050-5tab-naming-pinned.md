# ADR-0050: Star 5 Tab 命名拍板 (B-7 阻塞项解除)

> **状态**: 🟢 Accepted v1.0 (per 2026-09-08 05:25 JST 用户指令"5Tab 命名按照你推荐即可" + per AGENTS.md §7 #15 v0.15 现有引用)
> **生效**: 2026-09-08
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**: 🟢 架构师 (Mavis 接手 agent per DEC-008) (per 2026-08-27 19:39 + 21:59 JST 用户授权"允许你代签" + 守门 #10 author=Ulysses)
> **解除**: WBS §14.4 B-7 阻塞项 (5 tab 命名拍板 (UI 端) DDD Review 拍板具体名字)
> **关联**: [AGENTS.md §7 #15 v0.15 5-tab 引用](../../../AGENTS.md) · [STAR-P3-WBS-001.md §14.4 B-7](../../reports/STAR-P3-WBS-001.md) · [守门 #14 v2 5 域 Lead 临时代签](../../AGENTS.md)

---

## 1. 背景与问题

### 1.1 业务背景 (per 2026-09-08 05:25 JST 用户发令原话 + WBS §14.4 B-7)

Ulysses 在 2026-09-08 05:25 JST 明确发令:

> **"5Tab 命名按照你推荐即可"**

**关键 3 维**:
1. **gm-console AppShell 5-tab** (per AGENTS.md §7 #15 v0.15): 现有实现已用占位名 (Kanban / Timeline / Backlog / Agents / Worktrees)
2. **WBS §14.4 B-7 阻塞项**: "5 tab 命名拍板 (UI 端) DDD Review 拍板具体名字" — DDD Review 真人未到位, 阻塞中
3. **守门 #14 v2 拍板 D 维持**: Mavis 临时代签 5 域 Lead 决策, 真人到位后追溯签字

### 1.2 现状缺口

| 已落地 (v0.x) | 缺口 (本 ADR 补) |
|---|---|
| AGENTS.md §7 #15 v0.15 5-tab 占位 (Kanban / Timeline / Backlog / Agents / Worktrees) | 占位名未拍板, WBS B-7 阻塞 |
| gm-console AppShell 5-tab 渲染 | 5 tab 名尚未通过 DDD Review 拍板, 缺正式 ADR |
| 守门 #14 v2 拍板 D 维持 Mavis 临时代签 | DDD Review 真人到位前, Mavis 临时代签拍板 |

**关键缺口**:
- **B-7 阻塞**: WBS §14.4 B-7 "5 tab 命名拍板 (UI 端) DDD Review 拍板具体名字" 需解除
- **守门 #15 饱和**: docs 同步需新事件触发, 用户发令"按你推荐即可"=新事件, 符合
- **守门 #12 v2 派生规**: 任何 docs 同步必更新相关章节 + 守门派生规

### 1.3 架构冲突 (守门 #14 v2 + #15 派生)

per [AGENTS.md §4 #14 v2](../../../AGENTS.md): **5 域 Lead 临时代签**. Mavis 临时代签 5 域 Lead 决策 (破 8/21 拒绝兼任硬约束), 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事).

per [AGENTS.md §4 #15](../../../AGENTS.md): **守门 #12 死循环饱和**. commit-time docs 同步触达饱和后, 任何后续 docs 同步 commit 必先有**新事件触发** (代码改动 / Ulysses 拍板), 否则违反饱和约束.

---

## 2. 决策

**5 Tab 命名按现有 AGENTS.md §7 #15 v0.15 占位名落地拍板, 0 文档改动 (per 守门 #15 饱和约束), 解除 WBS §14.4 B-7 阻塞项.**

### 2.1 5 Tab 命名 (per 现有 AGENTS.md §7 #15 v0.15)

| # | Tab 名 | 中文 | 职责 | 跟 LangGraph / Agent Runtime / Star-EI view 关系 |
|---|---|---|---|---|
| 1 | **Kanban** | 看板 | 任务状态流转 (To Do / In Progress / Review / Done) | TMO 7 节点 (per ADR-0046) 状态可视化入口 |
| 2 | **Timeline** | 时间线 | 任务甘特图 + Agent timeline | L0/L1 Agent 时间线 (per L0 TopAgent) |
| 3 | **Backlog** | 积压 | 待规划任务池 | 5 域 Lead 任务积压池 |
| 4 | **Agents** | Agent 监控 | L0/L1 9 SA + SA-10 状态可视化 | per ADR-0045 Agent Runtime Hybrid Runtime |
| 5 | **Worktrees** | Worktree | Git worktree 管理 (per `domain-worktree` 22 crate) | TMO merge/split 节点 (per ADR-0046) |

### 2.2 拍板理由

- **0 文档改动**: 跟 AGENTS.md §7 #15 v0.15 现有占位名完全一致, 符合守门 #15 饱和约束
- **跟 LangGraph view (per ADR-0046) 对齐**: TMO 7 节点映射到 Kanban / Timeline 入口
- **跟 Agent Runtime view (per ADR-0045) 对齐**: 9 SA + SA-10 映射到 Agents 监控
- **跟 Star-EI view (per ADR-0048) 对齐**: 4 层锁链状态通过 Agents / Timeline 实时可视化
- **跟守门 #14 v2 拍板 D 维持**: Mavis 临时代签, 真人到位后追溯签字

### 2.3 跟 LangGraph / Agent Runtime / Star-EI view 映射

| 5 Tab | LangGraph view (per ADR-0046) | Agent Runtime view (per ADR-0045) | Star-EI view (per ADR-0048) |
|---|---|---|---|
| **Kanban** | TMO 7 节点状态 (merge/split/reorder/bulk/summarize/reassign/metadata) | TaskCard 1:1 mirror | L1 SubAgent 持锁 (per EX-04) |
| **Timeline** | GanttChart | L0/L1 ECS timeline | L0 dispatch 60s TTL + L1 lease 300s |
| **Backlog** | 任务池 + TMO bulk_node | Task queue (L0 dispatcher) | 派发级排他 (per EX-03) |
| **Agents** | SA-01..SA-09 + SA-10 状态 | L1 SubAgent 9 类型 | 4 层锁状态 (L0/L1/L2/L3) |
| **Worktrees** | merge_request_node + worktree_node | worktree entity | Domain 行锁 (per EX-02 star-mutex) |

---

## 3. 拒绝方案 (3 备选 + 理由)

### 3.1 改名方案 (per 用户"按你推荐即可")

| 备选 | 拒绝理由 |
|---|---|
| Tasks / Sprint / Ideas / Bots / Branches | 跟 AGENTS.md §7 #15 v0.15 不一致, 需重写文档, 违反守门 #15 饱和 |
| Board / Gantt / Pool / Bots / Trees (中文友好) | 跟现有占位英文名不一致, 改动 5 tab 渲染代码, 跨多个文件 |
| 保留 Kanban / Timeline / Backlog / Agents / Worktrees (本拍板) | ✅ 0 文档改动 + 跟现有占位一致 + 跟 3 view 关系映射清晰 |

### 3.2 拍板权备选

| 备选 | 拒绝理由 |
|---|---|
| 等 DDD Review 真人到位再拍 | 阻塞 WBS B-7, 5 tab 渲染代码已用占位名, 拍板后 0 改动; 真人到位后追溯签字 (per 守门 #14 v2) |
| ask_user 让 Ulysses 拍 5 选项 | 用户已明确发令"按你推荐即可", 拍板直接落地 (per 拍板推荐项直接执行 2026-09-05 04:03 JST) |

---

## 4. 后果

### 4.1 正面

1. **WBS B-7 阻塞解除**: 5 tab 命名拍板落地, 0 文档改动
2. **0 代码改动**: 现有 gm-console AppShell 渲染代码已用占位名, 拍板后无需重写
3. **跟 3 view 关系清晰**: 5 tab 跟 LangGraph / Agent Runtime / Star-EI 双向映射
4. **守门 #15 饱和合规**: 拍板落地是 docs 同步, 用户发令=新事件触发
5. **守门 #14 v2 拍板 D 维持**: Mavis 临时代签, 真人到位后追溯签字

### 4.2 负面 / 风险

1. **DDD Review 真人到位后追溯签字**: 守门 #14 v2 拍板 D 要求"真人到位后追溯签字", 不沿用代签决策
2. **跨 session 续**: 5 tab 渲染代码细节跨 session 续
3. **国际化 (i18n)**: 现有 tab 名为英文, 未来 i18n 跨 session 续

### 4.3 5 域 Lead RACI (per 守门 #14 v2)

| 域 | RACI | 拍板权 |
|---|---|---|
| admin 域 | R+A (Mavis 临时代签) | UI 端 5 tab 命名 (per 守门 #14 v2 拍板 D) |
| 其他 4 域 | I 通知 | 无影响 |

---

## 5. 验证 (per 守门 #1 + #14 v2 + #15)

### 5.1 守门实证

- (1) `git log -p --follow AGENTS.md` 实证 §7 #15 v0.15 现有 5-tab 引用
- (2) `git log -p --follow frontend/src/app/AppShell.tsx` (跨 session 续, 实证 5 tab 渲染)
- (3) 守门 #10 author=Ulysses
- (4) 守门 #15 饱和: 用户发令"按你推荐即可"=新事件, 符合触发

### 5.2 拍板落地验证

- ✅ WBS §14.4 B-7 阻塞项解除
- ✅ 0 文档改动
- ✅ 跟 3 view 关系映射清晰

---

## 6. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 (per 守门 #14 v2 拍板 D 临时代签) |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-08 05:25 JST | Ulysses — Mavis 接手 | 初稿, 5 tab 命名跟 AGENTS.md §7 #15 v0.15 一致, 0 改动 | per 2026-09-08 05:25 JST 用户发令"5Tab 命名按照你推荐即可" + WBS §14.4 B-7 阻塞项 |
| v1.0 | 2026-09-08 05:25 JST | Ulysses — Mavis 接手 | Accepted v1.0 拍板落地, WBS B-7 阻塞项解除 | 拍板 + 跟 AGENTS 一致 0 改动 |
