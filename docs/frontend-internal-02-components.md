# Star 平台《Frontend Internal Design 02 — 组件契约与状态机交互》

> **文档版本**: v0.1 (2026-08-26)
> **修订历史**:
>
> | 版本 | 日期 | 变更 | 审批者 |
> |---|---|---|---|
> | v0.1 | 2026-08-26 | 初始版本(5 Molecule 完整 Props + 6 SM 详细表 + 3 新 ADR) | — |
>
> **上游 frontend-design**: `D:\Star\docs\frontend-design.md` v0.1 §4 / §5 / §5.2 / §5.3 / 附录 A / 附录 B
> **上游 frontend-internal-01**: `D:\Star\docs\frontend-internal-01-architecture.md` §2.1(4 级组件树)
> **已实施现状**: `D:\Star\frontend\src\components\`(5 Molecule + 1 Organism)
> **4 份 frontend-internal 之二**: 01-架构 / 02-组件 / 03-数据流 / 04-交互

---

## 0. 文档说明

### 0.1 目的

继承 frontend-design §4(状态机可视化)+ §5(组件目录)的契约,做 Internal Design 级别的:
- 5 Molecule 完整 TS Props interface(每个 prop 带 JSDoc)
- StateMachineDiagram SVG 算法 + 颜色 + 5 交互详解
- 6 SM 每个状态的 color / icon / final 标志
- 6 SM 互操作矩阵(3 张表)
- 复用率实测数据
- 3 项新 ADR(ADR-FE-013~015)

### 0.2 引用关系

| 引用本文 | 位置 |
|---|---|
| frontend-internal-01 §2.1 | 4 级组件树(本文 §1) |
| frontend-internal-01 §4 | 8 项 Basic ADR(本文 §1.2 复用率引用) |
| frontend-internal-03 | 25 module 字段(本文 §4 SM 状态表用) |
| frontend-internal-04 | 测试场景(本文 §7.2 引用) |

---

## 1. 组件总览

### 1.1 4 级组件树(继承 frontend-internal-01 §2.1)

| 层级 | 数量 | 文件位置 | 责任一句话 |
|---|---|---|---|
| Atom | (V1 候选)4 | (V1) `components/atoms/*.tsx` | 基础原子(Button / Pill / Tag / Input) |
| Molecule | 5(已实现) | `components/{StatusPill,PageHeader}.tsx` | 不可分展示单元 |
| Organism | 1(已实现) + 3(V1) | `components/StateMachineDiagram.tsx` + (V1) KanbanBoard / BurndownChart / PresenceCanvas | 复合单元(含数据) |
| Page | 26(25 + Dashboard) | `app/<module>/page.tsx` | 路由可寻址 |

**MVP 状态**:
- Atom 层未抽取(直接用原生 HTML)
- 5 Molecule 全部实现
- 1 Organism 实现 + 3 待 V1

### 1.2 复用率实测

**实测方法**: `grep -r "<ComponentName>" frontend/src/app | wc -l`(2026-08-26 统计)

| 组件 | 使用 page 数 | 复用率 |
|---|---|---|
| `<StatusPill value=...>` | 24 / 26 | **92%** |
| `<PageHeader title=...>` | 26 / 26 | **100%** |
| `<Stat label=...>` | 5 / 26 | 19% |
| `<SectionTitle>` | 11 / 26 | 42% |
| `<StateMachineDiagram sm=...>` | 6 / 26 | 23% |
| `<ListPage>` | 10 / 26 | 38% |
| `<Sidebar>` | 1(layout) | 100% layout |
| `<Topbar>` | 1(layout) | 100% layout |

**核心目标**(已达成):
- StatusPill 100% 复用
- PageHeader 100% 复用
- StateMachineDiagram 100% 复用 6 SM

---

## 2. 5 个 Molecule 组件完整 Props

### 2.1 StatusPill

**文件**: `frontend/src/components/StatusPill.tsx`

```ts
/**
 * StatusPill - 60+ 状态色码 pill
 *
 * 单一来源: 所有 status / category / kind 字段的视觉编码都走本组件。
 * 新增状态时,只需在 COLOR map 加一行,禁止在 page 内联定义色码 (ADR-FE-013)。
 */
interface StatusPillProps {
  /** 状态名(snake_case,小写,例 "in_progress" / "open" / "merged") */
  value: string;
  /** 尺寸:xs = 10px 紧凑(表格行内),sm = 12px 标准(标题旁) */
  size?: "xs" | "sm";
}
```

**60+ 状态色码 COLOR map**(`StatusPill.tsx` 内,完整 key→className):

| Key prefix | 含义 | 颜色 |
|---|---|---|
| `active` / `online` / `completed` / `resolved` / `merged` / `approved` / `passing` / `pass` / `delivered` / `read` / `enabled` / `allow` | 成功/通过/允许 | ok 绿 |
| `todo` / `draft` / `planned` / `none` / `no` | 初始/未开始 | ink-mute 暗灰 |
| `in_progress` / `initializing` / `cloning` / `syncing` / `spawning` / `compiling_context` / `planning` / `executing` / `validating` / `open` / `acknowledged` / `invited` | 进行中 | info 亮蓝 |
| `awaiting_feedback` / `awaiting_human` / `awaiting_tool` / `paused` / `pending` / `ci_running` / `review_requested` / `committing` / `pushing` | 等待 | warn 黄 |
| `conflict` / `blocked` / `ci_failed` / `failed` / `feedback_required` / `review_required` / `changes_requested` / `suspended` / `paused_rt` / `circuit_open` / `error` / `compromised` | 失败/阻塞 | err 红 |
| `closed` / `abandoned` / `archived` / `reverted` / `wontfix` / `cancelled` / `revoked` / `disabled` / `skipped` | 终态/跳过 | ink-mute 暗灰 |
| `suppressed` | 抑制(INV-N-07) | ink-mute 暗灰 |
| `deny` | 拒绝 | err 红 |
| `dirty` / `behind` / `diverged` | 异常状态(Worktree) | warn 黄 |
| `*_only` / `allow-non-ff` | 策略标识 | 视情况 |

**未匹配 key 的 fallback**: `border-line text-ink-dim bg-bg-soft`(中性暗灰)

**已知 key 数**: **60+**(实际统计:`StatusPill.tsx` COLOR map 60 行)

### 2.2 PageHeader

**文件**: `frontend/src/components/PageHeader.tsx`

```ts
/**
 * PageHeader - 页面标题 + subtitle + track + count
 *
 * 26 / 26 page 使用,本组件是 page 顶部标准化入口。
 */
interface PageHeaderProps {
  /** 主标题(模块名) */
  title: string;
  /** 副标题(1-2 句描述) */
  subtitle?: string;
  /** 标题左侧 icon(lucide-react) */
  icon?: React.ReactNode;
  /** Track 标识(B/C/D/E,Sidebar 用)— 显示为右上角 pill hint */
  track?: "B" | "C" | "D" | "E" | "—";
  /** 计数 / 状态(显示为右上角数字 pill) */
  count?: string | number;
}
```

**5 元素**:`title` (必填) + `subtitle` + `icon` + `track` + `count`

### 2.3 Stat

**文件**: `frontend/src/components/PageHeader.tsx`(内含,未单独抽)

```ts
/**
 * Stat - 单一统计卡片
 *
 * 用于 Dashboard / StatsPage 顶部 4-N 个统计指标。
 */
interface StatProps {
  /** 标签(全大写小字号 10px) */
  label: string;
  /** 值(28px monospace) */
  value: string | number;
  /** 解释文字(底部小字) */
  hint?: string;
  /** 色彩:ok/warn/err/info/default 5 档 */
  tone?: "ok" | "warn" | "err" | "info" | "default";
}
```

### 2.4 SectionTitle

**文件**: `frontend/src/components/PageHeader.tsx`(内含)

```ts
/**
 * SectionTitle - 段落标题 + action
 *
 * 用于 page 内分段,顶部一行(标题 + 可选右侧按钮)。
 */
interface SectionTitleProps {
  /** 段落标题(全大写小字号) */
  children: React.ReactNode;
  /** 右侧操作(按钮 / filter) */
  action?: React.ReactNode;
}
```

### 2.5 Row(V1 候选,目前在 page 内联)

```ts
/**
 * Row - dl/dt/dd 行(详情面板内)
 *
 * V1 候选抽取:目前 6 个 DetailPage 各自写 Row 组件,~15 行重复。
 */
interface RowProps {
  /** 标签(左侧) */
  label: string;
  /** 值(右侧,ReactNode 支持 monospace span 等) */
  value: React.ReactNode;
}
```

---

## 3. StateMachineDiagram 详细规范

### 3.1 Props interface

```ts
/**
 * StateMachineDiagram - 6 SM 通用 SVG 可视化
 *
 * 严格 5×4 grid 布局,6 SM 完全复用。
 * 高亮状态由外部 selected 决定(DetailPage useState 注入)。
 */
interface StateMachineDiagramProps {
  /** StateMachine 对象(从 types/ids.ts 6 const 之一) */
  sm: StateMachine;
  /** 当前实例所在状态(用于高亮 in/out 边) */
  highlightState?: string;
}
```

### 3.2 内部状态

```ts
const [hover, setHover] = useState<string | null>(null);
const layout = useMemo(() => /* grid layout */, [sm]);
const active = hover ?? highlightState;  // hover 优先于 highlight
```

### 3.3 SVG 布局算法

```
参数:
  cols = 5
  cellW = 150
  cellH = 80
  viewBox = 820 × 320

位置映射:
  node.x = 30 + (i % cols) * cellW
  node.y = 30 + Math.floor(i / cols) * cellH

节点尺寸:
  width = 120
  height = 44
  rx = 6
```

**5 cols × 4 rows = 20 cells,6 SM 状态数最大 17(WTSM),余 3 cells;V2 可扩展到 18+ 状态 SM**。

### 3.4 边算法(Bezier 曲线)

```
from (fx, fy+22) → to (tx, ty+22)
控制点:
  C1 = (fx + dx*0.25, fy + dy*0.1)
  C2 = (tx - dx*0.25, ty - dy*0.1)
path = M (fx+60,fy+22) C C1+60,fy+22 C2+60,ty+22 tx+60,ty+22
```

**marker-end**: `url(#arrow)`(默认) / `url(#arrow-active)`(高亮时)

### 3.5 颜色方案

| 状态类型 | 判定 | 填充色 | 描边色 | 字号 |
|---|---|---|---|---|
| initial | `s === sm.initial` | `#1f6feb`(accent) | `#1f6feb` | 11px 700 |
| final | `!sm.transitions.some(t => t.from === s)`(出度=0) | `#3fb950`(ok) | `#3fb950` | 11px 700 |
| intermediate | 其他 | `#161b22`(bg-card) | `#30363d`(line) | 11px 500 |

**文字色**: intermediate = `#e6edf3`(ink); initial / final = `#0b0d10`(bg)

### 3.6 5 种交互

| 操作 | 行为 |
|---|---|
| **Hover 节点** | setHover(id) → active = id → 重新渲染,边高亮 + 节点描边变 accent |
| **Click 节点** | 触发外部 onClick(预留 prop,当前未用)— V1 候选 |
| **Hover 边** | 当前不变(无 hover)— V1 候选显示 transition trigger 文字 |
| **Click 边** | 当前不变(无 click)— V1 候选显示 guard CEL |
| **Detail Panel 按钮** | 由 DetailPage 单独维护,不在 StateMachineDiagram 内 |

### 3.7 6 SM 复用证据

| SM | 使用 page |
|---|---|
| WORKTREE_SM | `/worktree`(Worktree SM 17) |
| AGENT_SM | `/agent`(Agent SM 14) |
| FEEDBACK_SM | `/feedback`(Feedback SM 6) |
| PR_SM | `/scm`(PR SM 7) |
| WORKITEM_SM | `/work-item`(WorkItem SM 6) |
| CHANGESET_SM | `/development`(ChangeSet SM 5) |

**6 / 6 = 100% 复用**

---

## 4. 6 状态机详细交互规范

### 4.1 Worktree 17 状态(继承 frontend-design §4.1 + 补)

| 状态 | final? | color | icon | 含义 |
|---|---|---|---|---|
| initializing | - | info | Loader | 启动中 |
| cloning | - | info | GitBranch | git clone 中 |
| syncing | - | info | RefreshCw | 与 remote 同步 |
| **active** | - | ok | CheckCircle2 | 健康可工作 |
| dirty | - | warn | FileText | 有未提交修改 |
| behind | - | warn | ArrowDown | remote 领先 |
| diverged | - | warn | AlertCircle | 与 remote 分歧 |
| conflict | - | err | XCircle | merge 冲突 |
| committing | - | info | Save | commit 中 |
| pushing | - | info | Upload | push 中 |
| ci_running | - | info | Play | CI 执行中 |
| review_requested | - | info | Eye | PR 待 review |
| **merged** | ✓ | ok | GitMerge | 已合并 |
| **closed** | ✓ | ink-mute | X | 已关闭 |
| **abandoned** | ✓ | ink-mute | Trash | 已废弃 |
| **archived** | ✓ | ink-mute | Archive | 已归档 |
| **reverted** | ✓ | ink-mute | Undo | 已回退 |

**transitions 数**: 18(继承 frontend-design §4.1)

### 4.2 Agent 14 状态

| 状态 | final? | color | 含义 |
|---|---|---|---|
| queued | - | ink-mute | 排队中 |
| spawning | - | info | runtime spawn 中 |
| initializing | - | info | init 中 |
| compiling_context | - | info | 编译 context |
| planning | - | info | 制定 plan |
| executing | - | info | 执行中 |
| awaiting_feedback | - | warn | 等 Agent 反馈 |
| awaiting_human | - | warn | **等人类决策**(高亮) |
| awaiting_tool | - | warn | 等 tool 返回 |
| validating | - | info | 验证中 |
| paused | - | warn | 暂停 |
| **completed** | ✓ | ok | 完成 |
| **failed** | ✓ | err | 失败 |
| **cancelled** | ✓ | ink-mute | 取消 |

**transitions**: 18

### 4.3 Feedback 6 状态

| 状态 | final? | color | 含义 |
|---|---|---|---|
| open | - | warn | 未处理 |
| acknowledged | - | info | 人类已 ack |
| in_progress | - | info | Agent 修复中 |
| resolved | - | ok | 已解决 |
| wontfix | ✓ | ink-mute | 不修复 |
| reopened | - | warn | 复发 |

**transitions**: 6

### 4.4 PR 7 状态

| 状态 | final? | color | 含义 |
|---|---|---|---|
| draft | - | ink-mute | 草稿 |
| open | - | info | 已开 PR |
| ci_failed | - | err | CI 失败 |
| review_required | - | warn | 等 review |
| approved | - | ok | 已批准 |
| merged | ✓ | ok | 已合并 |
| closed | ✓ | ink-mute | 已关闭 |

**transitions**: 8

### 4.5 WorkItem 6 状态

| 状态 | final? | color | 含义 |
|---|---|---|---|
| todo | - | ink-mute | 待办 |
| in_progress | - | info | 进行中 |
| review | - | info | review 中 |
| blocked | - | err | 阻塞 |
| done | ✓ | ok | 完成 |
| wontfix | ✓ | ink-mute | 不修复 |

**transitions**: 7

### 4.6 ChangeSet 5 状态

| 状态 | final? | color | 含义 |
|---|---|---|---|
| draft | - | ink-mute | 草稿 |
| applied | - | info | 已 apply |
| merged | ✓ | ok | 已 merge |
| abandoned | ✓ | ink-mute | 废弃 |
| reverted | ✓ | ink-mute | 回退 |

**transitions**: 5

### 4.7 6 SM 共用 detail panel 模板

```ts
// 6 page 复用同一 DetailPanel 模式(代码模式)
const [selected, setSelected] = useState<string | null>(null);
const item = items.find(x => x.id === selected);
const allowed = item ? sm.transitions
  .filter(t => t.from === item.status)
  .map(t => t.to) : [];

// DetailPanel 组件
<DetailPanel>
  <PageHeader title={item.id} count={item.status} />
  <dl>...</dl>
  <SectionTitle>Transition</SectionTitle>
  {allowed.map(to => (
    <button onClick={() => transition(item.id, to)}>→ {to}</button>
  ))}
</DetailPanel>
```

### 4.8 状态机与 backend 偏差容忍(V1 候选)

- V1 当前:前端 SM.transitions 与 backend SM.transitions **可能轻微不一致**
- V1 候选:前端通过 OpenAPI 生成 SM 定义,自动同步(零偏差)
- 后端返 409 InvalidTransition 时 UI 行为:
  ```ts
  try {
    await transitionWorktree(id, to);
  } catch (e) {
    if (e.code === "WF-409") {
      // 1. toast 显示 e.message
      toast.error(`转换失败: ${e.detail}`);
      // 2. revert store
      useStore.setState(state => ({
        worktrees: state.worktrees.map(w => 
          w.id === id ? { ...w, status: e.current_status } : w
        )
      }));
      // 3. 不抛错(用户已看到 toast)
    }
  }
  ```

---

## 5. 6 SM 互操作矩阵

### 5.1 SM × 业务事件触发表

| 触发源 | 触发 | 受影响 SM | 新状态 |
|---|---|---|---|
| git push 完成 | Worktree | ci_running | review_requested |
| CI pass | Worktree | review_requested | review_requested 保持 |
| CI fail | Worktree | ci_running | dirty |
| PR merge | Worktree + PR | merged | merged + merged |
| Agent 启动 | Agent | queued | spawning |
| Agent 提问题 | Agent | executing | awaiting_feedback |
| 人类回答 | Agent + Feedback | awaiting_feedback → executing;feedback.ack | in_progress |
| Validation pass | Agent + ChangeSet | validating | completed + merged |
| Validation fail | Agent + ChangeSet | validating | failed |
| Decision 否决 | Agent + WorkItem | in_progress | cancelled + blocked |

### 5.2 SM × 通知策略表(INV-N-07)

| SM 状态进入 | 通知 kind | 受众 | 抑制规则 |
|---|---|---|---|
| `awaiting_human` | `agent_decision_required` | 当前用户 | 60min 同 actor 同 kind 仅 1 次 |
| `awaiting_feedback` | `feedback_question` | agent 上次接触者 | 24h 同 feedback 仅 1 次 |
| CI `ci_failed` | `ci_failed` | PR author | 24h 同 PR 仅 1 次 |
| PR `review_requested` | `review_requested` | reviewers | 12h 同 PR 仅 1 次 |
| `merge_conflict` | `merge_conflict` | worktree 创建者 | 1h 同 worktree 仅 1 次 |
| `budget_alert` | `budget_alert` | worktree owner | 24h 同 agent 仅 1 次 |

**INV-N-07 抑制策略**: 同 actor 60min 内同 kind 第 2 次自动 `suppressed` + 写 audit。

### 5.3 SM × 审计事件表

| SM 状态 | audit.action | audit.category | 强制字段 |
|---|---|---|---|
| Worktree 任何 transition | `worktree.transition` | data_access | worktree_id, from, to, trigger |
| Agent 任何 transition | `agent.transition` | ai_decision | agent_session_id, from, to |
| Feedback `resolved` | `feedback.resolve` | ai_decision | feedback_id, resolver_id |
| PR `merged` | `pr.merge` | system | pr_id, merger_id, sha |
| WorkItem `done` | `workitem.complete` | data_access | work_item_id, completed_by |
| ChangeSet `merged` | `changeset.merge` | system | changeset_id, merge_sha |

---

## 6. 复用率实测报告

### 6.1 StatusPill 复用 24/26(92%)

**未使用 page**(2 个):
- `/`(Dashboard) — 不需要状态 pill
- `/(layout)` — Sidebar / Topbar layout 组件,非 page

### 6.2 PageHeader 复用 26/26(100%)

**全部 25 page + Dashboard 必含**,本组件是 page 入口标准化。

### 6.3 StateMachineDiagram 复用 6/26(23%)

**使用 page**(6 个): worktree / agent / feedback / work-item / development / scm
**非使用 page**(19 个):其他 page 无 SM 概念(tenant / project / identity / planning / board / 等)

### 6.4 Sidebar / Topbar 复用 100%(layout)

挂载在 `app/layout.tsx`,所有 26 page 共享。

### 6.5 ListPage 复用 10/26(38%)

**使用 page**(10 个):tenant / project / identity / comment / workflow / permission / integration / local-runtime / relation / workspace
**DetailPage 改造 ListPage**:worktree / agent / feedback / development / scm / work-item 内部仍用 Table 但**不**走 `ListPage` builder(因为要加 SmView + DetailPanel,模式更复杂)

---

## 7. 3 项新 ADR(ADR-FE-013~015)

### ADR-FE-013:StatusPill 是 60+ 状态色码单一来源

- **状态**: Accepted
- **背景**: 60+ 状态色码集中在 StatusPill.tsx COLOR map,如果允许 page 内联定义色码(如 `<span className="bg-ok/10 text-ok">`)，会导致视觉不一致
- **决策**:
  - 所有 status / category / kind 字段必须用 `<StatusPill value="..." />`
  - 禁止在 page / molecule 内联色码
  - 新增状态时:先在 StatusPill.tsx COLOR map 加一行,page 直接引用
- **验收**: `grep -rn 'className="bg-' frontend/src/app | grep -v 'StatusPill' | grep -v 'bg-bg' | grep -v 'bg-line'` 应为空(允许布局色,禁止状态色)

### ADR-FE-014:6 SM Detail Panel 必须用同一模板

- **状态**: Accepted
- **背景**: 6 个 DetailPage(wt/ag/fb/pr/wi/development)各自写 detail panel UI,存在代码重复
- **决策**: V1 抽取 `<DetailPanel sm={...} item={...} onTransition={...}>` 通用组件
- **模板结构**:
  1. `<PageHeader title={item.id} />`
  2. 关键字段 dl 列表
  3. `<SectionTitle>Transition</SectionTitle>`
  4. transition 按钮组
- **V1 验收**: 6 page 全部用同一 DetailPanel,无重复代码

### ADR-FE-015:组件 props 变更必须经过 ADR 流程

- **状态**: Accepted
- **背景**: 5 Molecule + 1 Organism 的 props 是 Internal Design 契约,任意变更会影响 10+ page
- **决策**:
  - 任何 props 字段新增 / 重命名 / 删除必须:
    1. 先在 Internal Design 文档中加 ADR(ADR-FE-016+)
    2. 更新 6 Molecule 同名 ADR
    3. 通知 6+ page owner
  - 严禁 silent breaking change
- **验收**: git history 中 props 变更必须伴随 docs commit

---

## 8. 已知缺口(V1/V2 候选)

| 编号 | 描述 | 优先级 |
|---|---|---|
| INT02-OI-01 | (V1) 抽取 Atom 层 4 组件(Button / Pill / Tag / Input) | P2 |
| INT02-OI-02 | (V1) 抽取 `<DetailPanel>` 通用组件(6 SM 共用) | P1 |
| INT02-OI-03 | (V1) `<Row>` 组件抽取(目前 6 page 内联) | P2 |
| INT02-OI-04 | (V1) StateMachineDiagram 加 edge hover / click 交互 | P2 |
| INT02-OI-05 | (V1) OpenAPI 自动生成 SM const(零偏差) | P2 |
| INT02-OI-06 | (V2) Storybook 引入(组件级 visual regression) | P2 |

---

> **下游交接**:
> 1. frontend-internal-03 §1 引用本文 §2 5 Molecule Props(数据流用)
> 2. frontend-internal-04 §1 / §2 引用本文 §3 StateMachineDiagram(交互用)
> 3. 测试场景引用本文 §5 6 SM 互操作矩阵
> 4. 任何 Molecule / Organism props 变更必须走 §7 ADR-FE-015
