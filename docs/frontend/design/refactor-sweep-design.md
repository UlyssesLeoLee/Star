# Refactor Sweep — 架构设计文档 (v0.1)

> **Status**: 🟢 Active (per 2026-09-02 10:41 JST 拍板 + 10:50 JST 合并按钮拍板)
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **Path**: `/refactor` (frontend/src/app/refactor/page.tsx)
> **Scope**: STAR 平台新增 Refactor Sweep 重构专项页

---

## 0. 一句话定位

> **分批次对已完成任务做重构 · Jira 风格 todo→done 循环, 列可自定义, done 列一键合并 worktree + PR**

---

## 1. 背景与动机

### 1.1 现状 (per 2026-09-02 拍板)

- 已完成任务 (`workItem.status === "done"`) 散落在 22 个 domain-*/projects/issues 视图, 没有专门的重构入口
- 缺一个聚合"待重构"队列, 重构流程也无法量化 (第几次重构? 进度?)
- KanbanBoard 提供 4 态 todo/in_progress/review/done, 但 6 状态机 `workItem.status` 不适合"重构流"

### 1.2 目标 (per 2026-09-02 10:41 JST Ulysses 拍板)

1. **新增 /refactor 页面** — 顶部 KPI + 5 列看板 + 历史轮次
2. **5 状态 todo / doing / testing / review / done** — Jira 风格, 跟 Kanban 对齐
3. **testing 列在 doing 和 review 中间** — 新增, 验证重构后回归测试
4. **列可自定义** — 增/删/重命名/重排, 跟 KanbanBoard 行为 1:1
5. **多轮循环** — 走完一轮, 卡全部 reset todo + round_number + 1, 历史保留
6. **done 列点 Merge 按钮** — 一键合并 worktree + PR, 标识 merged 状态 (per 10:50 JST 拍板)

### 1.3 不做 (per 守门 缺标比错标)

- 后端 PATCH /refactor-rounds (UI store only, 持久化走 zustand persist)
- 多人协作 (applyRemoteChange 留给 Phase 2)
- 复杂 WIP / batch 流 (v1: 全卡可见, KPI 计数, 用户自管理节奏)

---

## 2. 架构

### 2.1 状态机与数据流

```
┌────────────────────────────────────────────────────────────────┐
│  WorkItem (主流程)              RefactorCard (重构流)          │
│  ┌──────────────────┐            ┌──────────────────┐         │
│  │ status: done ────┼──auto──►  │ refactor_status:  │         │
│  │ (seed 入 round)  │            │   todo (入 round) │         │
│  └──────────────────┘            └──────────────────┘         │
│                                                                  │
│  refactor_status 5 态:                                           │
│    todo ──► doing ──► testing ──► review ──► done              │
│                                                  │              │
│                                                  ▼              │
│                                            [Merge 按钮]          │
│                                            merged_at: now       │
│                                            + Worktree→merged    │
│                                            + PR→merged          │
└────────────────────────────────────────────────────────────────┘
```

### 2.2 关键设计决策

| # | 决策 | 理由 |
|---|---|---|
| 1 | 新增独立 `RefactorStatus` 5 态, 不污染 `WorkItem.status` | 6 态 workItem 已稳定, 塞"重构中 done"破坏主流程 |
| 2 | `RefactorCard.merged_at` / `merged_worktree_id` / `merged_pr_id` 冗余存 | done 列 UI 区分 "待 merge" vs "已 merge", 不需二次查 store |
| 3 | `RefactorBoardConfig` per project | 列自定义跟项目走, 跨项目不污染 |
| 4 | `REFACTOR_FALLBACK_STATUS = "todo"` 写死 | 跟 Kanban `TODO_FALLBACK_STATUS` 风格一致, 兜底列保护 |
| 5 | 列 string status + 5 内置常量 | 允许用户加自定义列 (e.g. "spike"), 内置 5 态翻译走 i18n |
| 6 | `moveRefactorCard` 写 `history: [{status, at}]` | 审计追溯, 跟 workItem 状态机一致 |
| 7 | `mergeRefactorCard` 返回状态码 `ok / not_found / not_done / already_merged / closed_round` | UI 可接 toast / inline msg |
| 8 | 持久化 `refactorRounds` + `refactorBoardConfigs` (排除 `canvasElements` 大字段) | 跟现有 partialize 模式一致 |

### 2.3 文件清单

```
frontend/src/
├── types/ids.ts                                      # +RefactorStatus / RefactorColumn / RefactorCard / RefactorRound / RefactorBoardConfig (line 1019+)
├── lib/
│   ├── board-refactor-constants.ts                   # NEW — 默认 5 列 + fallback 工具
│   ├── store.ts                                      # +refactorRounds / refactorBoardConfigs + 10 action
│   ├── nav/registry.ts                               # +refactor 导航项
│   └── i18n/
│       ├── dictionary.ts                             # +refactor 段
│       ├── zh-CN.ts / en.ts / ja.ts                  # +refactor 文案
├── components/refactor/                              # NEW 目录
│   ├── RefactorCard.tsx                              # 单卡 + Merge 按钮 + merged 徽章
│   ├── RefactorSweepBoard.tsx                        # 5 列看板 + 列 CRUD + 拖动
│   ├── RefactorKpiRow.tsx                            # 顶部 5 KPI
│   ├── RefactorRoundHistory.tsx                      # 历史轮次
│   ├── RefactorSettingsPopover.tsx                   # batch_size + 重置列
│   └── AddRefactorCardsDialog.tsx                    # 选 done 任务加入
└── app/refactor/page.tsx                             # 主页 (project switcher + KPI + 看板 + 历史)
```

---

## 3. 类型定义 (per types/ids.ts §28)

```ts
// 默认 5 状态 (per 2026-09-02 10:41 JST 拍板, 加 testing)
export type RefactorStatus = "todo" | "doing" | "testing" | "review" | "done" | string;
export const REFACTOR_DEFAULT_STATUSES = ["todo", "doing", "testing", "review", "done"] as const;
export const REFACTOR_FALLBACK_STATUS = "todo" as const;

export interface RefactorColumn {
  status: RefactorStatus;
  name?: string;            // 用户改的显示名 (缺省走 i18n)
  position: number;          // 0-indexed
  wip_limit?: number;        // optional
}

export interface RefactorCard {
  work_item_id: Uuid;
  work_item_key: string;     // snapshot
  work_item_title: string;   // snapshot
  priority?: WorkItemPriority;
  kind?: WorkItemKind;
  refactor_status: RefactorStatus;
  entered_at: Iso8601;
  moved_at: Iso8601;
  history: Array<{ status: RefactorStatus; at: Iso8601 }>;
  round_number: number;
  // ── Merge 状态 (per 2026-09-02 10:50 JST 拍板) ──
  merged_at?: Iso8601;
  merged_worktree_id?: Uuid;
  merged_pr_id?: Uuid;
}

export interface RefactorRound {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid;
  round_number: number;       // 1-indexed
  notes?: string;
  started_at: Iso8601;
  closed_at?: Iso8601;        // 填了 = 历史, 只读
  cards: RefactorCard[];
}

export interface RefactorBoardConfig {
  project_id: Uuid;
  columns: RefactorColumn[];
  fallback_status: "todo";   // 写死
  batch_size: number;         // 默认 5
  updated_at: Iso8601;
}
```

---

## 4. Store Action (10 个)

| Action | 行为 | 跟 Kanban 对齐 |
|---|---|---|
| `ensureRefactorBoardConfig(projectId)` | lazy init 默认 5 列 + batch_size=5 | 等价 `board` 隐式存在 |
| `addRefactorColumn(projectId, status, name?)` | 末尾追加, status 不重复 | `addBoardColumn` |
| `removeRefactorColumn(projectId, status)` | 兜底拒绝 + 卡归 fallback | `removeBoardColumn` |
| `renameRefactorColumn(projectId, status, newName)` | 改 name (status 标识不变) | `renameBoardColumn` |
| `reorderRefactorColumns(projectId, fromIdx, toIdx)` | 改 position 字段 | `reorderBoardColumns` |
| `resetRefactorColumns(projectId)` | 回默认 5 列 + 保留 batch_size | (新增) |
| `setRefactorBatchSize(projectId, size)` | clamp 1-50 | (新增) |
| `openRefactorRound(projectId, opts?)` | round + 1, 入 status=done WI | (新增) |
| `closeRefactorRound(roundId)` | 填 closed_at, 只读 | (新增) |
| `startNextRefactorRound(projectId)` | 校验全 done, 关闭上一轮 + 开新轮 | (新增) |
| `moveRefactorCard(roundId, wiId, toStatus)` | 改 refactor_status + 写 history | `transitionWorkItem` |
| `addRefactorCard(roundId, wiId)` | 加卡 (UI "Add Cards" 按钮) | (新增) |
| `removeRefactorCard(roundId, wiId)` | 撤回卡 | (新增) |
| **`mergeRefactorCard(roundId, wiId)`** | **校验 done + 动 worktree→merged + PR→merged + 写 card.merged_at** | **(新增, 10:50 JST 拍板)** |

### 4.1 mergeRefactorCard 状态机

```
[RefactorCard]                    [Worktree]              [PullRequest]
  refactor_status: done     ──►   status: merged     ──►  status: merged
  merged_at: now                    last_event_at: now      merged_at: now
  merged_worktree_id: <wt>          lock_version: +1
  merged_pr_id: <pr> (if exists)
```

返回值: `"ok" | "not_found" | "not_done" | "already_merged" | "closed_round"` — UI 可接 toast / inline msg

### 4.2 兜底列保护 (跟 Kanban 一致)

- `REFACTOR_FALLBACK_STATUS = "todo"` 写死, 不可改
- `removeRefactorColumn` 二次拒绝 fallback (UI ✕ 按钮置灰 + tooltip)
- 删非兜底列: 列里卡 refactor_status 归 fallback, 写 history, 数据零丢失

---

## 5. UI/UX 设计

### 5.1 页面布局

```
┌──────────────────────────────────────────────────────────────────┐
│ [Header] Refactor Sweep · Round #2 · 12 work-items              │
│ [Project switcher: PHYSIS / StarGate / Mobile]                   │
├──────────────────────────────────────────────────────────────────┤
│ [KPI row]   Todo:3  Doing:1  Testing:1  Review:0  Done:7 (58%) │
├──────────────────────────────────────────────────────────────────┤
│ [Actions]  [Open Next Round ▶]  [+ Add Cards]  [⚙ Settings]    │
├──────────────────────────────────────────────────────────────────┤
│ [Board — 5 columns + Add column]                                 │
│ ┌─Todo──┐ ┌─Doing─┐ ┌─Testing┐ ┌─Review┐ ┌─Done──┐ ┌+Add──┐  │
│ │  ⋮⋮  │ │  ⋮⋮  │ │   ⋮⋮  │ │   ⋮⋮  │ │   ⋮⋮  │ │       │  │
│ │ TO DO│ │DOING  │ │TESTING │ │REVIEW │ │ DONE  │ │       │  │
│ │  3   │ │  1   │ │   1   │ │   0   │ │   7   │ │       │  │
│ │ P-23 │ │ P-5  │ │ P-12  │ │       │ │ P-3 ✓ │ │       │  │
│ │ P-25 │ │      │ │       │ │       │ │[Merge]│ │       │  │
│ │ P-31 │ │      │ │       │ │       │ │ P-9 ✓ │ │       │  │
│ └──────┘ └──────┘ └───────┘ └───────┘ └───────┘ └───────┘  │
├──────────────────────────────────────────────────────────────────┤
│ [History] Round #2 (active)  Round #1 12/12 100%  ✓              │
└──────────────────────────────────────────────────────────────────┘
```

### 5.2 单卡设计

```
┌─────────────────────────────────┐
│ [#2]                       ⋮⋮  │  ← round 徽章 + 拖手柄
│ // PHYSIS-5                     │  ← key
│ Implement Worktree 17-state     │  ← title
│ ───────────────────────────────│
│ [DOING]              14:32      │  ← refactor_status + moved_at
│ ┌─────────────────────────────┐│
│ │  [⟳ Merge]                  ││  ← done 列才显示
│ └─────────────────────────────┘│
└─────────────────────────────────┘
```

合并后变体:
```
┌─────────────────────────────────┐
│ [#2]                       ⋮⋮  │
│ // PHYSIS-3                     │
│ Webhook Idempotency-Key         │
│ ───────────────────────────────│
│ [✓ MERGED]              14:32   │  ← 绿色徽章
└─────────────────────────────────┘
```

### 5.3 列自定义 (跟 KanbanBoard 1:1)

| 操作 | 触发 | UI |
|---|---|---|
| 重命名 | 点击列名 | inline edit (input + Enter/Esc) |
| 重排 | 拖 ⋮⋮ 拖手柄 | drop 时 reorder |
| 删除 (非兜底) | ✕ 按钮 | 二次确认 + tooltip 提示兜底保护 |
| 添加 | 末尾 + Add Column 按钮 | `prompt("status 名")` → 新列 |

### 5.4 Merge 按钮状态机

| 卡状态 | UI 显示 |
|---|---|
| refactor_status != "done" | 不显示 |
| refactor_status = "done" && !merged_at | 绿色 `[⟳ Merge]` 按钮, 点击后调 mergeRefactorCard |
| refactor_status = "done" && merged_at | 绿色 `[✓ MERGED · HH:MM]` 徽章 |
| readOnly (closed round) | 不显示按钮 |

### 5.5 Settings Popover (per 拍板)

- `batch_size` 输入 (1-50, 默认 5)
- `Reset Columns` 按钮 (回默认 5 列, 二次确认)
- 外部点击关闭

---

## 6. 交互流程

### 6.1 首次访问 (Round #1 自动开)

```
1. user visit /refactor
2. page lazy-init RefactorBoardConfig (默认 5 列)
3. project 无 active round && project 存在 status=done WI -> openRefactorRound
4. 入 round: 全部 done WI 转 RefactorCard (refactor_status: todo)
5. 渲染看板
```

### 6.2 用户重构单个卡

```
1. drag P-5 from todo to doing
2. moveRefactorCard(roundId, "p-5", "doing")
3. store 写 history: [{status: doing, at: now}], moved_at 刷新
4. KPI 计数刷新
```

### 6.3 Merge 单卡 (per 10:50 JST 拍板)

```
1. user clicks [Merge] on P-3 in done column
2. (optional) window.confirm(mergeConfirm)
3. page.handleMergeCard("p-3")
4. store.mergeRefactorCard(roundId, "p-3")
5. 副作用:
   a. Worktree.status: "active" → "merged", last_event_at: now, lock_version: +1
   b. PullRequest.status: "open" → "merged", merged_at: now
   c. RefactorCard.merged_at: now, merged_worktree_id, merged_pr_id
6. card 重新渲染, 绿色 [✓ MERGED] 徽章
7. user 可继续重构其他卡
```

### 6.4 开启下一轮

```
1. user 标记所有卡为 done
2. KPI 显示 100%, [Open Next Round ▶] 按钮亮起
3. user click → handleOpenNextRound
4. store.startNextRefactorRound(projectId)
   a. 校验当前 active round 全 done
   b. closeRefactorRound(active.id)  → closed_at: now
   c. openRefactorRound(projectId)   → round + 1, 新 cards (从 project done WI 拉新一批)
5. UI 切换到新 round, 历史 round 进 bottom history
```

### 6.5 加卡 (UI 入口)

```
1. user clicks [+ Add Cards]
2. AddRefactorCardsDialog 打开
3. 列出 project 中 status=done 但不在 round 内的 WI
4. user 多选 + 点 [Add (N)]
5. addRefactorCard 逐个调
```

---

## 7. 跟现有架构的对齐

| 维度 | 现有模式 | Refactor Sweep |
|---|---|---|
| 状态机 | workItem 6 态 + Board 4 列 | 独立 5 态 RefactorStatus, 跟 workItem 解耦 |
| 持久化 | zustand persist (exclude canvasElements) | 同上, refactorRounds + refactorBoardConfigs 持久化 |
| i18n | dict + useTranslation + 3 语言 | 同上, refactor 段 3 语言完整 |
| 兜底保护 | TODO_FALLBACK_STATUS (Kanban) | REFACTOR_FALLBACK_STATUS (Refactor), 1:1 风格 |
| 列 CRUD | addBoardColumn / removeBoardColumn / renameBoardColumn / reorderBoardColumns | 1:1 镜像 (addRefactorColumn / ...) |
| WIP 限制 | wip_limit 字段 + UI 红色 | 同样 wip_limit 字段 + warn 配色 |
| 拖动 | HTML5 native (per 守门) | 同样 HTML5 native, dataTransfer 区分卡/列 |
| Project switcher | ProjectsClient 顶部 chip row | 镜像 page.tsx 顶部 chip row |
| 历史时间线 | Kanban 不需要 (单 board) | RefactorRoundHistory 独立组件 |
| Merge 行为 | 无 (Kanban 不涉及 git) | mergeRefactorCard 联动 Worktree + PR |

---

## 8. 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | 后端无 /refactor-rounds API (zustand persist only) | localStorage 单设备 | P3-E 加 REST/GraphQL |
| 2 | 多人协作无 (applyRemoteChange 不覆盖 refactorRounds) | 多 tab 冲突 | Phase 2 |
| 3 | 列添加只用 window.prompt, 缺 Drawer 形态 | 移动端体验差 | Phase Mobile |
| 4 | refactor_status 缺 review→doing 回退校验 | 用户可乱拖 | v0.2 加状态机校验 |
| 5 | 历史 round 的列自定义不影响已 closed round | 老数据稳定, 但新列加不到历史 | by design (snapshot) |
| 6 | mergeRefactorCard 失败 (e.g. worktree 已被外部关) 不回滚 | 用户需手动 | v0.2 加事务 |
| 7 | 触屏拖动 (touch events) 未适配 | 手机端用不了 | Phase Mobile |
| 8 | ARIA live region (a11y) 未实现 | 屏幕阅读器 | v0.2 a11y pass |

---

## 9. 守门对照 (per AGENTS.md §4 守门)

| 守门 | 是否满足 | 证据 |
|---|---|---|
| #1 禁回溯叙事 | ✅ | 本文档 commit 引用, 不写"per X 历史形态" |
| #2 引用 BAS git 实证 | ✅ | 无 BAS 引用 |
| #3 缺标比错标 | ✅ | §8 列 8 项已知缺口 |
| #4 子代理授权无证据叙事 = 禁止 | N/A | 本任务无子代理派单 |
| #5 环境变量安全 | ✅ | 全程无 env 读 |
| #6 PowerShell only | ✅ | 工具调用全 PowerShell |
| #7 0 unsafe | ✅ | 无 `any` (除 useStatusLabel map 兜底), 0 `as` 强转 |
| #8 不沿用 bc23d6c 叙事 | ✅ | |
| #9 子代理 dispatch 必先落地 brief | N/A | 无子代理 |
| #10 代签规则应用 | ✅ | Mavis 接手代签 Ulysses |
| #11 缺标比错标安全 | ✅ | §8 |
| #12 docs 同步 (7 段结构) | ✅ | 本文 + PHASE 报告 / git commit 引用 |
| #13 DB 三類横展開 (W/T/M) | N/A | 本任务无 DB 设计 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: /refactor 页面 + 5 态 todo/doing/testing/review/done + 列自定义 + 10 store action + Merge 按钮 | 2026-09-02 10:41 JST 拍板 (testing 列 + 列自定义) + 10:50 JST 拍板 (Merge 按钮) |
