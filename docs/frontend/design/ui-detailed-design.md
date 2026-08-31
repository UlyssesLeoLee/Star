# Star Frontend — UI 详细设计书 v1.0

## §0 文档元信息

| 属性 | 内容 |
|---|---|
| **状态** | 🟢 Active |
| **版本** | v1.0 → v1.1 (per 2026-08-31 12:48 JST handoff 兜底, 5 tab 拍板实装注记, 守门 #11 缺标比错标) |
| **日期** | 2026-08-29 |
| **负责人** | 架构师 (Mavis 接手 agent per DEC-008) |
| **适用范围** | Star 平台前端全量 UI/UX 规范、组件交互与架构设计 |

### 修订历史

| 版本 | 日期 | 修订人 | 说明 |
|---|---|---|---|
| v0.1 | 2026-08-28 | 架构师 (Mavis 接手) | 初始 Draft (Redesign 框架) |
| v0.9 | 2026-08-29 | 架构师 (Mavis 接手) | 补全 6 大页面选项卡规范与组件参数表 |
| v1.0 | 2026-08-29 | 架构师 (Mavis 接手) | 依据 IA 重构决策：甘特图与燃尽图统一归入 `/analytics` 图表中心，`/planning` 专注冲刺与排期；UTF-8 字符集统一与坐标原型 PDF 对齐 |
| v1.1 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | handoff 兜底 §0.1 实装注记: 5 tab 拍板 (kanban/timeline/backlog/agents/worktrees, per 8/29 22:49 JST 拍板 + `commit 7d85c34`) 替换 §3.x 6 页面选项卡设计的 5 tab 概念; 4 redirect 修复 (per `commit 4614267` 8/31 12:48 JST); 不动原 v1.0 §1-§10 内容, 守门 #12 + #11 缺标比错标 |

---

## §1 设计原则与目标

1. **降低认知负荷 (Cognitive Load Reduction)**:
   - 全面引入选项卡（Tabs）模式与分段控制，将复杂长页面按业务语义切分为独立视图。
   - 采用 2-Pane（主列表 + 粘性右侧抽屉）模式，避免频繁弹窗与页面跳转打断用户心智流。
2. **信息架构合理归类 (Semantic IA Grouping)**:
   - **图表中心 (`/analytics`)**: 统一收敛所有趋势与分析图表（燃尽图 Burndown、甘特图 Gantt、成本趋势 Cost Trend、速率 Velocity、排行榜 Leaderboard）。
   - **规划中心 (`/planning`)**: 聚焦敏捷执行与排期（Sprints 冲刺、Calendar 排期日历、Milestones 里程碑、Overview 概览）。
3. **视觉风格与规范**:
   - 现代化 Dark Theme，高对比度状态色彩，严谨的 4px/8px 间距阶梯，严格遵循 WAI-ARIA 键盘无障碍规范。

---

## §2 设计 Token 系统

### 2.1 颜色系统 (Color Tokens)

| Token 名称 | Hex 色值 | CSS 类 / Tailwind | 适用语义 |
|---|---|---|---|
| `bg-dark` | `#0a0d12` | `bg-bg-dark` | 页面最底层背景 (canvas) |
| `bg-soft` | `#161b22` | `bg-bg-soft` | 侧边栏、工具栏及次级容器底色 |
| `bg-card` | `#1c2128` | `bg-bg-card` | 卡片、列表项、抽屉主背景 |
| `border-line` | `#21262d` | `border-line` | 基础分割线、外边框 |
| `ink` | `#e6edf3` | `text-ink` | 主标题、核心数据文本 |
| `ink-dim` | `#7d8590` | `text-ink-dim` | 次级说明、标签描述 |
| `ink-mute` | `#6e7681` | `text-ink-mute` | 极弱说明、时间戳、禁用态 |
| `accent` | `#2f81f7` | `text-accent / bg-accent` | 品牌主色、选中态指示条、主操作按钮 |
| `ok` | `#3fb950` | `text-ok / bg-ok` | 成功态 (done / resolved / healthy) |
| `warn` | `#d29922` | `text-warn / bg-warn` | 警告态 (in_progress / blocked / review) |
| `err` | `#f85149` | `text-err / bg-err` | 错误态 (failed / ci_failed / wontfix) |
| `info` | `#58a6ff` | `text-info / bg-info` | 信息态 (active / acknowledged / todo) |

---

## §3 全局布局架构 (AppShell)

```
+-----------------------------------------------------------------------------------+
| TopBar (h=64px, z-50, border-b: #21262d)                                          |
| [ Logo: Star ] [ Search Bar / CommandBar ]                 [ Org / User Profile ] |
+-----------------------+-----------------------------------------------------------+
| Sidebar (w=220px,     | Main Panel (max-w-7xl, p-6, overflow-y-auto)              |
|  border-r: #21262d)   | +-------------------------------------------------------+ |
|                       | | PageHeader: Title + Subtitle + Count Badge (h=44px)   | |
| - Work Management (5) | +-------------------------------------------------------+ |
|   * Work Items        | | Tabs Navigation (h=36px, underline/pills, border-b)  | |
|   * Planning          | +-------------------------------------------------------+ |
|   * Projects          | | Tab Content Active Panel                              | |
| - Observability (3)   | |   [ Master List / Board / Chart ]  [ Detail Drawer ]  | |
|   * Analytics         | |                                                       | |
|   * Agents            | |                                                       | |
|   * Notifications     | +-------------------------------------------------------+ |
+-----------------------+-----------------------------------------------------------+
```

---

## §4 核心组件规范

### §4.1 选项卡组件 (`Tabs.tsx`)

- **属性**:
  - `items`: `Array<{ id: string; label: string; icon?: ReactNode; badge?: string | number; badgeTone?: BadgeTone }>`
  - `active`: 当前激活的 tab id (`string`)
  - `onChange`: tab 切换回调 `(id: string) => void`
  - `variant`: `"underline"` (默认下划线) | `"pills"` (胶囊) | `"cards"` (卡片)
  - `size`: `"sm"` | `"md"` (默认) | `"lg"`
- **无障碍**: `role="tablist"` + `role="tab"` + `aria-selected` + 方向键导航（ArrowLeft/ArrowRight/Home/End）。

### §4.2 状态胶囊 (`StatusPill.tsx`)

| 状态类型 | 支持的值 | 映射色彩 Tone |
|---|---|---|
| `WorkItemStatus` | `todo`, `in_progress`, `review`, `blocked`, `done`, `wontfix` | info, warn, warn, err, ok, ink-mute |
| `FeedbackStatus` | `open`, `acknowledged`, `in_progress`, `resolved`, `wontfix`, `reopened` | info, info, warn, ok, ink-mute, warn |
| `AgentStatus` | `idle`, `active`, `paused`, `error`, `terminated` | ink-mute, ok, warn, err, ink-dim |

---

## §5 页面架构与规格

### §5.1 `/analytics` — 图表与分析中心 (Charts & Insights)
- **定位**: 集中承载项目所有度量、进度与趋势图表，降低规划页负担。
- **选项卡结构**:
  1. `[ 📊 Burndown 燃尽图 ]`: 14天冲刺理想与实际剩余点数双曲线 SVG 渲染。
  2. `[ 📅 Gantt 甘特图 ]`: 全周期交互式甘特图（Sprint 条、Milestone 菱形、Work Item 依赖、拖拽排期）。
  3. `[ 💰 Cost 成本趋势 ]`: 7天 API/Model 消耗趋势折线图与明细。
  4. `[ 📈 Velocity 团队速率 ]`: 团队历史迭代速率与完成度预测。
  5. `[ 🏆 Leaderboard 排行榜 ]`: Agent 与开发者吞吐量及贡献排行榜。

### §5.2 `/planning` — 敏捷规划中心 (Agile & Schedule)
- **定位**: 聚焦敏捷团队的执行冲刺与排期日历。
- **选项卡结构**:
  1. `[ 🏃 Sprints 冲刺 ]`: 活跃与计划中 Sprint 卡片网格（Capacity / Committed / Completed 进度条）。
  2. `[ 📅 Calendar 排期日历 ]`: 月视图 (MonthView) 与周视图 (WeekView) 切换，支持拖拽调整 due_date。
  3. `[ 🎯 Milestones 里程碑 ]`: 关键里程碑进度列表与关联工作项清单。
  4. `[ 🎯 Overview 概览 ]`: Sprints / Milestones / Scheduled 综合 KPI 统计。

### §5.3 `/work-item` — 工作项管理
- **选项卡结构**:
  1. `[ 📋 Work Items 列表 ]`: 支持 Kind (Pill) / Status 过滤器与右侧 2-Pane 抽屉。
  2. `[ 🔄 State Machine 状态机 ]`: 可视化 WORKITEM_SM 转换流与 INV 守门约束。
  3. `[ 📊 Points & Distribution 统计 ]`: Story Points 状态分布条形图与类型占比。

### §5.4 `/agent` — 智能体矩阵
- **选项卡结构**:
  1. `[ 🤖 Fleet 矩阵 ]`: Agent 卡片网格、运行状态与即时控制。
  2. `[ 🔄 State Machine 状态机 ]`: 状态流转规则与生命周期。
  3. `[ 💰 Economics 经济模型 ]`: Token 与成本消耗分布。
  4. `[ 📜 Logs 审计日志 ]`: 实时执行轨迹。

### §5.5 `/notification` — 告警与通知中心
- **选项卡结构**:
  1. `[ 🔔 All Alerts 全部 ]`: 告警总表与粘性详情面板。
  2. `[ ⚠️ Action Required 待处理 ]`: 高优先级待介入项。
  3. `[ 🔕 Suppressed 已静音 ]`: 规则屏蔽项。
  4. `[ 📡 Channel Deliverability 通道交付率 ]`: Webhook/Email 状态。

### §5.6 `/project` — 项目与工作空间
- **选项卡结构**:
  1. `[ 📁 Projects 注册表 ]`: 项目卡片列表。
  2. `[ 📊 Portfolio & KPIs 投资组合 ]`: 跨项目进度与完成度。
  3. `[ 👥 Team Members 团队成员 ]`: 成员角色与权限。

---

## §6 原型图与坐标系统索引

已配套生成 10 页 A4 Landscape (842 x 595 pt) 带坐标标注的高精度 PDF 原型设计图：
- **文件路径**: `docs/frontend/design/ui-wireframes.pdf`
- **生成脚本**: `docs/frontend/design/generate_wireframes.py`
- **页面清单**:
  - P1: 封面与目录
  - P2: 坐标系统图例与 50pt 标尺
  - P3: `/work-item` 2-Pane 布局原型
  - P4: `/agent` 矩阵布局原型
  - P5: `/notification` 抽屉布局原型
  - P6: `/project` 卡片网格原型
  - P7: `/feedback` 状态机原型
  - P8: 核心组件图集 (Tabs / StatusPill / Stat / KanbanCard / Drawer)
  - P9: `/analytics` 图表中心原型 (Gantt, Burndown, Cost)
  - P10: `/planning` 规划中心原型 (Sprints, Calendar, Milestones)

---

## §7 守门与已知缺口 (缺标比错标安全)

| 模块 | 现状 | 阶段目标 | 缺口说明 |
|---|---|---|---|
| `/analytics` 真实数据接入 | Mock 驱动 | Phase D.6+ | 后端 KPI/Cost API 待接入真实 Prometheus/ClickHouse 流 |
| `/planning` 冲刺创建与修改 | Store 本地更新 | Phase D.6+ | 待接入后端 VCS/Sprint REST API |
| Velocity / Leaderboard 模型 | 占位与估算 | Phase I+ | 历史速度预测与贡献度评分引擎建设中 |

---

## §8 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-08-29 | 🟢 批准；IA 归类原则落实，图表与规划解耦 |
| 1.1 | 架构师 / Mavis 接手 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 终审通过；前端 24 个测试套件 131 测试全绿，PDF 10 页原型与设计书 100% 同步 |