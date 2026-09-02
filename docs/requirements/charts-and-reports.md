# 图表 & 报告系统 总需求（对标 Jira Cloud 报告中心）v1.0

> **状态**: Draft v1.0 (2026-09-02)
> **对标基线**: Jira Cloud Reports & Dashboards (per atlassian.com/software/jira/reports, 2026-09 取证)
> **触发**: 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira, 各个图表设计要完善, 先把需求、基本设计、详细设计文档补上"
> **拍板记录**:
> - 范围 = A 全对标 (22 图表 + 5 scope) (per ask_user 4 维拍板 scope=A)
> - 文档组织 = A 总 3 份 + 每图表 1 份详细设计 (per ask_user doc-org=A)
> - 技术栈 = A Recharts (per ask_user tech-stack=A)
> - spec 关系 = A 在现有 domain-report-spec / domain-dashboard-spec 扩写 v1.0 (per ask_user spec-relation=A)
> **下游交付**:
> - 总基本设计 → `docs/basic-design/charts-and-reports.md`
> - 22 份详细设计 → `docs/design/charts/{chart-id}.md`
> - spec 升版 → `docs/specs/domain-report-spec.md` v1.0 + `docs/specs/domain-dashboard-spec.md` v1.0
> - 实现 → `crates/domain-report/` + `crates/domain-dashboard/` + `frontend/src/components/charts/`

---

## 0. 文档说明

### 0.1 目标与定位

本文档是 Star 平台**图表 & 报告系统**的总需求基线,目标:

1. **对标 Jira Cloud 报告中心**——覆盖 Jira 22 类核心图表 + 5 大类数据 scope,做到"功能等价、能力可比"
2. **支撑 22 bounded context + 6 supporting crate 的数据可视化**——所有业务域(WorkItem / Sprint / Version / User / Time 等)必须有可视化出口
3. **可被 Dashboard 复用**——每个图表既可单独作为 Report 输出,也可作为 Dashboard Gadget 嵌入
4. **跨 5 类 scope 一致性**——同一图表在 Project / Sprint / Version / Project Hierarchy / Issue 5 个 scope 下行为一致

**不在范围内**(per basic-design §0.1 拆分):
- ECharts / D3 / 自研 SVG 等其他图表栈(本期只走 Recharts,per ask_user tech-stack=A)
- 实时流式图表(本期走 30s polling,WebSocket 流式留 V2)
- 商业 BI 集成(Tableau / Power BI 嵌入留 V2)

### 0.2 文档结构

| 章节 | 内容 | 字数预算 |
|---|---|---|
| §1 | 5 大类 scope 定义 | 80 行 |
| §2 | 22 图表类型总览 | 60 行 |
| §3 | 22 图表详细需求(每图 1 段) | 350 行 |
| §4 | 数据过滤与查询(JQL 风格) | 60 行 |
| §5 | 交互需求(订阅/导出/分享/钻取) | 50 行 |
| §6 | Dashboard 集成 | 40 行 |
| §7 | 性能 NFR | 50 行 |
| §8 | 可访问性 & 国际化 | 30 行 |
| §9 | 与现有模块集成 | 40 行 |
| §10 | 验收标准 | 40 行 |
| §11 | 风险 & 缓解 | 30 行 |
| §12 | 修订历史 | 15 行 |

### 0.3 文档关系

```
docs/requirements.md (总需求, 已有)
    ↓ 引用
docs/requirements/charts-and-reports.md (本文档, 图表系统总需求)
    ↓ 引用
docs/basic-design/charts-and-reports.md (图表系统总基本设计)
    ↓ 引用
docs/design/charts/{chart-id}.md (22 份图表详细设计)
    ↓ 实现
crates/domain-report/ + crates/domain-dashboard/ (Rust 实现)
frontend/src/components/charts/ (React + Recharts 实现)
```

### 0.4 dual-use 警告

per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板:

> domain-report / domain-dashboard 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 5 大类 Scope 定义

图表的**数据作用域** (scope) 决定图表从哪些实体聚合数据。Jira 报告中心按 scope 划分为 5 类,Star 完整对标:

| Scope ID | 名称 | 数据范围 | 典型图表 | 权限边界 |
|---|---|---|---|---|
| **S1** | **Project** (项目级) | 单个项目内的所有 issue | Burndown / CFD / Velocity | project_view / project_admin |
| **S2** | **Project Hierarchy** (项目组合级) | 跨项目聚合,按 project tree 路径 | 跨项目 Velocity / Workload | portfolio_view (新) |
| **S3** | **Sprint** (Sprint 级) | 单个 Sprint 内的 issue + 时间窗 | Burndown / Sprint Report / Sprint Burnup | sprint_view / project_admin |
| **S4** | **Version** (版本级) | 单个 Version (release) 内的 issue | Version Workload / Release Burndown | version_view / project_admin |
| **S5** | **Issue** (Issue 级 / 过滤器驱动) | JQL 风格过滤器返回的 issue 集 | Created vs Resolved / Cycle Time / Throughput | filter_view / filter_share |

**派生约束**:
- 任何图表必须**显式声明 scope**(在 `ReportDefinition.scope` 字段)
- 同一图表 type 在不同 scope 下**复用 series 渲染逻辑**,但**数据查询路径不同**(per basic-design §3.2)
- 跨 scope 图表(如 S2)必须**显式聚合**(per INV-REPORT-04)

---

## 2. 22 图表类型总览

按"敏捷分析全链路"组织,6 大类 22 图:

### 2.1 Agile & Sprint 类(5)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C01 | Burndown Chart | S3 Sprint | X=日期, Y=剩余 Story Points / Issue 数 | Burndown Chart |
| C02 | Burnup Chart | S3 Sprint | X=日期, Y=累积完成 / 总范围 | Burnup Chart |
| C03 | Velocity Chart | S1 Project | X=Sprint, Y=承诺 / 完成 SP | Velocity Chart |
| C04 | Sprint Report | S3 Sprint | 表(本期 / 上期 / 未完成分组) | Sprint Report |
| C05 | Cumulative Flow Diagram (CFD) | S1 Project | X=日期, Y=各状态 issue 数(堆叠) | Cumulative Flow Diagram |

### 2.2 Cycle Time & Forecast 类(4)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C06 | Control Chart | S5 Issue (filter) | X=完成日期, Y=周期时间(天), ±3σ 控制线 | Control Chart |
| C07 | Cycle Time Report | S5 Issue (filter) | 直方图 + 50/85/95 分位 | Cycle Time Report |
| C08 | Throughput Report | S5 Issue (filter) | X=周/月, Y=完成 issue 数 | Throughput Report |
| C09 | Forecast Chart | S3 Sprint / S1 Project | X=日期, Y=剩余范围 + 预测完成线 | Forecast (新) |

### 2.3 Time & SLA 类(3)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C10 | Time Tracking Report | S5 Issue (filter) | X=用户/项目, Y=估时/已记录/剩余 | Time Tracking Report |
| C11 | Resolution Time Report | S5 Issue (filter) | X=优先级/类型, Y=平均/中位解决时间 | Resolution Time Report |
| C12 | SLA Compliance | S1 Project | X=日期, Y=SLA 命中率(%) | SLA Report (新) |

### 2.4 Distribution & Workload 类(5)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C13 | Created vs Resolved | S1 Project | X=日期, Y=新建/解决双线 | Created vs Resolved Chart |
| C14 | Issue Type Distribution | S5 Issue (filter) | Pie, 切片=type | Pie Chart |
| C15 | Priority Distribution | S5 Issue (filter) | Pie, 切片=priority | Pie Chart |
| C16 | Assignee Workload | S5 Issue (filter) | Bar, X=用户, Y=open issue 数 | Assignee Workload |
| C17 | Workload by Component | S5 Issue (filter) | Bar 横向, X=component, Y=open issue 数 | Version Workload (component 变体) |

### 2.5 Version & Release 类(3)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C18 | Version Workload | S4 Version | Bar 横向, X=完成状态(完成/进行中/未开始), Y=版本 | Version Workload Report |
| C19 | Release Burndown | S4 Version | X=日期, Y=剩余 issue 数 | Release Burndown |
| C20 | Time in Status | S5 Issue (filter) | Bar 横向, X=状态, Y=平均停留天数 | Time Since Status Report |

### 2.6 Custom & Advanced 类(2)

| ID | 名称 | Scope 默认 | 关键轴 | Jira 对应 |
|---|---|---|---|---|
| C21 | Heatmap (活跃度) | S5 Issue (filter) | X=周, Y=小时, value=新建/解决 issue 数 | Heat Map |
| C22 | Recently Created | S5 Issue (filter) | 列表(不算严格图表,但 Report 中心含) | Recently Created Issues |

---

## 3. 22 图表详细需求

> **约定**: 每图表给出 (a) 业务定义 (b) 关键轴 & series (c) 必需交互 (d) 数据源 (e) 验收标准 (f) 异常/边界

### 3.1 C01 — Burndown Chart

**业务定义**: Sprint 期内,剩余 Story Points (或剩余 issue 数)随时间的下降趋势。理想线 vs 实际线对比。

**关键轴 & series**:
- X 轴: Sprint 日期 (Sprint.start_date → Sprint.end_date)
- Y 轴: 剩余 SP (或 issue 数), 范围 0 → Sprint.total_sp
- Series:
  - 理想线 (Ideal): 线性下降 (Sprint.start 时 = total_sp, end 时 = 0)
  - 实际线 (Actual): 每日累计完成 SP 的反向 = total_sp - Σ(day_i 完成 SP)
  - Scope Change Marker (可选): sprint 范围调整事件用垂直虚线标记

**必需交互**:
- 悬停 tooltip: 显示当日日期 / 剩余 SP / 当日完成 SP / scope change (如有)
- Y 轴切换: SP ↔ issue count
- 时间窗切换: Sprint 内 ↔ 自定义 (zoom)
- 导出: PNG / CSV

**数据源**:
- `Sprint { sprint_id, start_date, end_date, total_sp, scope_change_log[] }`
- `WorkItem { sprint_id, completed_at, story_points }` 投影到 daily buckets
- 数据延迟: 5min TTL (per INV-REPORT-02)

**验收标准**:
- Sprint.start_date 之前 X 轴不显示
- Sprint.end_date 之后 X 轴冻结在 end_date
- Scope change 触发后 actual 线有断点
- 0 SP sprint 显示 "无数据" 提示

**异常/边界**:
- Sprint 未开始: 仅显示 ideal 线
- Sprint 结束超过 30 天: 显示"历史"标签, 默认折叠
- Scope change > 10 次: 仅显示最近 10 次

### 3.2 C02 — Burnup Chart

**业务定义**: Sprint 期内,累积完成的 SP (或 issue 数)上升趋势 + Sprint 范围调整线。

**关键轴 & series**:
- X 轴: Sprint 日期
- Y 轴: 累积完成 SP
- Series:
  - 实际完成线 (Actual)
  - Sprint 范围线 (Scope): 阶梯式,Sprint 范围调整时跳跃
  - 理想完成线 (Ideal, 可选)

**必需交互**:
- 悬停 tooltip: 日期 / 完成 SP / 当前范围 / 范围变化量
- 切换 Y 轴单位
- 切换 series 显示

**数据源**: 同 C01

**验收标准**:
- Scope 线在范围变更时立即跳跃(垂直线段)
- 实际线在范围变更后保持原值

**异常/边界**: 同 C01

### 3.3 C03 — Velocity Chart

**业务定义**: 跨多个 Sprint,团队承诺 SP 与完成 SP 的对比。

**关键轴 & series**:
- X 轴: Sprint 名称 (按时间倒序/正序)
- Y 轴: SP 数
- Series:
  - 承诺 SP (Committed): 柱状
  - 完成 SP (Completed): 柱状(叠加或并列)
  - 平均完成线 (Average): 虚线水平

**必需交互**:
- 悬停 tooltip
- 切换: 显示承诺 / 显示完成 / 都显示
- 时间窗: 最近 N 个 Sprint (默认 6)
- 导出 CSV

**数据源**:
- `Sprint { sprint_id, name, committed_sp, completed_sp }`

**验收标准**:
- 已结束 Sprint 显示 committed + completed
- 进行中 Sprint 仅显示 committed (completed 实时)
- 平均线 = Avg(completed_sp) of 全部显示 Sprint

**异常/边界**:
- < 2 个已完成 Sprint: 不画平均线,显示提示
- 0 committed Sprint: 仍显示, label "未规划"

### 3.4 C04 — Sprint Report

**业务定义**: 当前 Sprint 完成度,分"本期完成 / 上期完成(延期) / 未完成"三组。

**关键轴 & series**:
- 不算图表,是表格 + 摘要
- 三列: Issue Key / 标题 / 状态分类
- 摘要区: 完成数 / 延期数 / 未完成数 / 完成 SP

**必需交互**:
- 点击 issue key → 跳转 issue detail
- 切换 Sprint

**数据源**: `WorkItem { sprint_id, completed_at, status }`

**验收标准**:
- 本期完成 = completed_at 在本 Sprint 内
- 延期 = completed_at 在本 Sprint 内 但 sprint_id 来自上一 Sprint
- 未完成 = 当前 Sprint 范围 但 未完成

### 3.5 C05 — Cumulative Flow Diagram (CFD)

**业务定义**: 项目内每天各状态 (To Do / In Progress / Done) 的 issue 数量堆叠面积图。

**关键轴 & series**:
- X 轴: 日期
- Y 轴: issue 数 (堆叠)
- Series (按 status 类别堆叠):
  - To Do (最底)
  - In Progress
  - In Review
  - Done (最顶)

**必需交互**:
- 悬停 tooltip
- 切换 status 类别显示
- 时间窗: 最近 N 天 (默认 30 / 90 / 365)

**数据源**:
- `WorkItem { status, status_changed_at }` → 每日快照

**验收标准**:
- CFD 总和恒等于 issue 总数(不变)
- Done 区域只能上升
- 状态类别变化时(workflow 改动)显示断点

**异常/边界**:
- 自定义 workflow: 取当前 active 状态作为 series
- 0 issue: 仅画 X 轴

### 3.6 C06 — Control Chart

**业务定义**: 每个 issue 完成时的"周期时间 (cycle time)"散点图,叠加 ±3σ 控制线检测异常。

**关键轴 & series**:
- X 轴: 完成日期 (按时间)
- Y 轴: 周期时间 (天,对数刻度可选)
- Series:
  - 散点 (每个 issue 1 点)
  - 中位线 (Median)
  - 70% / 85% / 95% 分位线
  - ±3σ 控制线

**必需交互**:
- 悬停: issue key / 周期 / 状态
- 点击点: 跳转 issue
- 切换 log/linear 刻度
- 异常高亮 (超 3σ)

**数据源**:
- `WorkItem { completed_at, cycle_time (天) }`
- 周期时间 = completed_at - first_in_progress_at

**验收标准**:
- 至少 10 个完成 issue 才画控制线
- 异常点红色高亮 + 文字"⚠ 异常"
- 鼠标悬停异常点显示 z-score

### 3.7 C07 — Cycle Time Report

**业务定义**: 周期时间的分布(直方图)+ 50/85/95 百分位。

**关键轴 & series**:
- X 轴: 周期时间桶 (0-1d, 1-3d, 3-7d, 7-14d, 14-30d, 30d+)
- Y 轴: issue 数
- 文本区: 50% / 85% / 95% 百分位数值

**必需交互**:
- 切换桶大小
- 切换: 显示所有 / 仅已完成 / 仅未完成

**验收标准**:
- 桶选择自适应: 数据 < 50 时用 1 天桶, ≥ 50 时用 3 天桶
- 百分位基于线性插值(per numpy.percentile 方法)

### 3.8 C08 — Throughput Report

**业务定义**: 单位时间内完成的 issue 数(条形/折线)。

**关键轴 & series**:
- X 轴: 周或月 (可切换)
- Y 轴: 完成 issue 数
- Series: 单线 (完成数)

**必需交互**:
- 时间粒度切换
- 时间窗选择
- 移动平均 (3 周/3 月)叠加

**数据源**: `WorkItem { completed_at }` 按周/月分桶

### 3.9 C09 — Forecast Chart

**业务定义**: 基于历史 Velocity / Throughput,预测 Sprint 或项目完成日期。

**关键轴 & series**:
- X 轴: 日期 (含未来)
- Y 轴: 累积完成 SP / issue 数
- Series:
  - 历史实际线
  - 预测线 (基于历史 3-6 个 Sprint 速度平均)
  - 预测区间 (80% / 95% confidence band)

**必需交互**:
- 切换预测方法: 平均速度 / 滚动平均 / 线性回归
- 显示预测完成日期

**数据源**:
- 历史 `Sprint { committed_sp, completed_sp }`
- 当前 `Sprint { remaining_sp, end_date }`

**验收标准**:
- 至少 3 个已完成 Sprint 才预测
- 预测区间宽度基于历史速度标准差

### 3.10 C10 — Time Tracking Report

**业务定义**: 每个用户/项目/issue 的估时 vs 已记录 vs 剩余时间。

**关键轴 & series**:
- 行: issue (或用户/项目, 可切换)
- 列: Original Estimate / Time Spent / Remaining / 进度(%)
- 表格 + 摘要图 (按用户聚合的 Bar)

**必需交互**:
- 行/列维度切换
- 时间窗

**数据源**:
- `WorkItem { original_estimate, time_spent, remaining_estimate }`
- `WorkLog { worklog_id, issue_id, time_spent_seconds, author_id, started_at }`

### 3.11 C11 — Resolution Time Report

**业务定义**: 解决时间 (resolution time) 按优先级/类型/经办人分组的平均/中位。

**关键轴 & series**:
- X 轴: 优先级或类型 (切换)
- Y 轴: 平均/中位解决时间 (天)
- Series: Bar (分组: Avg / Median)

**必需交互**: 切换分组维度

**数据源**:
- `WorkItem { priority, issue_type, resolved_at, created_at }`
- resolution time = resolved_at - created_at

### 3.12 C12 — SLA Compliance

**业务定义**: 项目/优先级维度,SLA 命中率(%)随时间变化。

**关键轴 & series**:
- X 轴: 日期
- Y 轴: SLA 命中率 (%)
- Series: 折线 (按优先级叠加)

**必需交互**: 切换 SLA 定义(项目/优先级各自)

**数据源**:
- `SLA { sla_id, target_resolution_hours, priority, project_id }`
- `WorkItem { resolved_at, priority, project_id }` 命中判定

### 3.13 C13 — Created vs Resolved Chart

**业务定义**: 每天新建 issue 数 vs 解决 issue 数,两条线对比。

**关键轴 & series**:
- X 轴: 日期
- Y 轴: issue 数
- Series: Created / Resolved (双线)

**必需交互**: 时间粒度(天/周/月)

**数据源**: `WorkItem { created_at, resolved_at }`

### 3.14 C14 — Issue Type Distribution

**业务定义**: 按 issue type 分组的占比 Pie。

**关键轴 & series**:
- Pie, 切片 = type
- 中心: 总数 + 标签

**必需交互**: 悬停百分比 / 计数;点击切片 → filter

### 3.15 C15 — Priority Distribution

**业务定义**: 按 priority 分组的占比 Pie。

(同 C14,维度 = priority)

### 3.16 C16 — Assignee Workload

**业务定义**: 每个 assignee 当前 open issue 数 (Bar 横向)。

**关键轴 & series**:
- Y 轴: assignee (top N 默认 20)
- X 轴: open issue 数
- Series: Bar (可分状态堆叠: To Do / In Progress / In Review)

**必需交互**: 切换堆叠模式;点击用户 → 跳转 user detail

### 3.17 C17 — Workload by Component

**业务定义**: 按 component 分组的 open issue 数(Bar 横向)。

(同 C16,维度 = component)

### 3.18 C18 — Version Workload

**关键轴 & series**:
- Y 轴: Version 名称
- X 轴: issue 数
- Series: Bar 堆叠 (Done / In Progress / To Do / 未分类)

**验收标准**: 包含已发布/未发布两个分组

### 3.19 C19 — Release Burndown

**业务定义**: Version 发布前的剩余 issue 数随时间下降。

**关键轴 & series**: (同 C01,但 scope = Version 而非 Sprint)

**差异**: X 轴 = release_due_date 而非 Sprint.end_date

### 3.20 C20 — Time in Status

**业务定义**: 每个状态平均停留时间(Bar 横向)。

**关键轴 & series**:
- Y 轴: 状态名称
- X 轴: 平均停留天数

**数据源**:
- `WorkItemStatusHistory { workitem_id, status, entered_at, exited_at }`

**验收标准**: 仅统计已 exit 状态;current 状态不计入(否则偏差)

### 3.21 C21 — Heatmap (活跃度)

**业务定义**: 周 × 小时 矩阵,值 = 新建/解决 issue 数,色阶展示。

**关键轴 & series**:
- X 轴: 小时 (0-23)
- Y 轴: 周一-周日
- value: 计数,色阶 (浅→深)

**必需交互**:
- 切换 value = 新建 / 解决
- 时区切换(用户时区 vs UTC)

### 3.22 C22 — Recently Created

**业务定义**: 最近创建的 issue 列表(分页)。

**关键轴 & series**: 不算图表,表格。列: Key / 标题 / 类型 / 优先级 / 创建人 / 创建时间

**必需交互**: 分页 / 排序 / 跳转

---

## 4. 数据过滤与查询 (JQL 风格)

### 4.1 过滤表达式语法

Star 平台采用类 JQL (Jira Query Language) 的过滤表达式,支持:

```
<field> <operator> <value> [AND|OR <field> <operator> <value>]*
```

**支持字段** (per requirements §8):
- `project`, `sprint`, `fixVersion`, `component`, `assignee`, `reporter`
- `type`, `priority`, `status`, `label`
- `created`, `updated`, `resolved`, `due` (含 `>=`, `<=`, `>`, `<`, `between`)
- `story_points`, `original_estimate`, `time_spent`

**支持操作符**:
- `=`, `!=`, `IN`, `NOT IN`, `~`(包含)
- `>=`, `<=`, `>`, `<`, `BETWEEN`
- `IS EMPTY`, `IS NOT EMPTY`

### 4.2 过滤器保存与分享

- **命名过滤器**: `Filter { filter_id, owner_id, jql, shared_with[] }`
- **跨用户分享**: `FilterShare { filter_id, user_id/group_id, permission (view/edit) }`
- **默认过滤器**: 系统提供 5 个默认 filter (My Open Issues / Reported by Me / Recently Updated / All Open / Done This Sprint)

### 4.3 过滤表达式对图表的影响

- 任何 scope = S5 (Issue) 的图表**必须**绑定一个 filter
- 过滤器变更触发图表重新计算
- 过滤器保存在 ReportDefinition.filter_id 字段

---

## 5. 交互需求

### 5.1 订阅

- **订阅触发**: 用户订阅 Report 后,Report 周期生成快照
- **周期选项**: Daily / Weekly / Monthly / On Change
- **触发通道**: Email / In-App Notification (per REQ-NOTIF-002)
- **降噪策略** (per REQ-NOTIF-002): 同类订阅 24h 内合并

### 5.2 导出

- **格式**: CSV / Excel (XLSX) / PNG / PDF
- **CSV 导出**: 必须含原始数据点(可二次分析)
- **PNG 导出**: 当前图表视图(分辨率 ≥ 2x)
- **PDF 导出**: 标题 + 图表 + 数据表(三件套)
- **导出范围**: 当前过滤 / 全部数据 (默认当前过滤)

### 5.3 分享

- **链接分享**: 生成 token URL, 含权限 (view / edit)
- **嵌入分享**: 提供 iframe embed code (per Dashboard 集成)

### 5.4 钻取 (Drill-down)

- **点击图表元素 → 跳转**:
  - 图表数据点 (Bar slice / Pie slice / Scatter point) → 跳到对应 issue 列表(filter)
  - 图例切换 → 切换 series 显示
  - 跨图表钻取: 图表 A 的数据点 → 触发图表 B 重新查询(同 scope)

---

## 6. Dashboard 集成

### 6.1 Gadget 类型与图表对应

每个图表 type 可作为 Gadget 嵌入 Dashboard:

| Gadget Type | 对应图表 | 特殊配置 |
|---|---|---|
| `chart-{ID}` | 对应 22 图表 | chart_type + filter_id + scope |
| `text` | (无) | markdown 文本 |
| `activity` | 活动流 | project_id / filter_id |
| `assigned_to_me` | 个人待办 | user_id |
| `filter_results` | 过滤结果列表 | filter_id |
| `wallboard-clock` | 数字时钟 | timezone |
| `wallboard-sla` | 大字号 SLA 数字 | project_id, priority |

### 6.2 12-Grid 布局

- Dashboard 12-grid (Tailwind 标准)
- Gadget 拖拽 / 调整大小
- Gadget 不重叠 (per INV-DASH-02)

### 6.3 Wallboard 模式

- 全屏展示,无编辑权限 (per INV-DASH-03)
- 30s auto-refresh

---

## 7. 性能 NFR

### 7.1 数据查询性能

| 图表类型 | 数据量 | P95 响应时间 | 缓存策略 |
|---|---|---|---|
| C01-C05 (Sprint 敏捷) | < 5K issue / sprint | < 2s | 5min TTL |
| C06-C08 (Cycle/Throughput) | < 50K issue | < 3s | 5min TTL |
| C13-C17 (Distribution) | < 100K issue | < 2s | 5min TTL |
| C18-C20 (Version) | < 10K issue / version | < 2s | 5min TTL |
| C21 (Heatmap) | < 100K issue | < 3s | 5min TTL |

### 7.2 渲染性能

- 首次渲染 (FCP): < 1.5s (Recharts 客户端渲染)
- 切换 filter 重渲染: < 500ms
- 大数据点 (>10K) 启用数据采样 + 提示

### 7.3 并发与吞吐

- 单 Project 同时打开 Report 数: ≤ 50
- 单 Dashboard 加载 Gadget 数: ≤ 12 (12-grid 上限)
- 后台批量生成 Report: ≤ 100 / min

---

## 8. 可访问性 & 国际化

### 8.1 可访问性 (a11y)

- 图表必须有 `aria-label` + `role="img"`
- 颜色对比度 WCAG 2.1 AA (4.5:1)
- 键盘导航: Tab 切换图表 / 焦点状态可见
- 屏幕阅读器: 关键数据点文本描述(如"中位周期时间 3.5 天")
- 不依赖颜色单独传达信息(用 icon / 文本双通道)

### 8.2 国际化 (i18n)

- 所有文本走 i18n 资源 (`frontend/src/i18n/{locale}.json`)
- 支持 zh-CN / en-US / ja-JP (3 语,per Star 国际化基线)
- 数字/日期格式: 跟随 locale (per `date-fns/locale`)
- 图表轴标签本地化

---

## 9. 与现有模块集成

### 9.1 数据源对接

| 数据源 | 图表 | 集成方式 |
|---|---|---|
| `domain-work-item` Projection | C01-C05, C10-C20 | Port: `WorkItemQueryPort.list(filter)` |
| `domain-planning` Projection | C01-C03, C09, C18-C19 | Port: `SprintQueryPort`, `VersionQueryPort` |
| `domain-identity` | C10, C16 | Port: `UserQueryPort` |
| `domain-permission` | 所有 (scope 权限) | Port: `PermissionPort.check(user, scope, action)` |
| `domain-audit` | (报告自身审计) | Port: `AuditRecorderPort.record(event)` |
| `domain-notification` | 订阅触发 | Port: `NotificationPort.send(target, event)` |

### 9.2 不持事实原则 (per INV-REPORT-01)

Report / Dashboard **不得持有 SoR 业务事实**,仅持有:
- `ReportDefinition` 元数据 (配置)
- `ReportSnapshot` 缓存(5min TTL)
- 订阅关系 (User ↔ Report)

事实永远在 `domain-work-item` / `domain-planning` 等 SoR 域。

### 9.3 跨域接触面 (与 existing spec 对齐)

per basic-design v0.16 §3.1 解耦机制 8 种,Report / Dashboard 接触面:
- `report` 读 work-item → Customer-Supplier (Open Host Service)
- `report` 写 audit → Conformist (AuditRecorder)
- `dashboard` 嵌入 report → Shared Kernel (ReportDefinition)
- `report` 触发通知 → Separate Ways (异步)

---

## 10. 验收标准

### 10.1 功能完整

- 22 图表全部上线,每图表对应 Jira 报告中心同名功能
- 5 大类 scope 全部支持
- 10 Dashboard Gadget 全部可配置
- 5 种导出格式 (CSV/XLSX/PNG/PDF) 全实现

### 10.2 性能

- per §7.1 表 P95 全达标
- per §7.2 渲染 FCP < 1.5s

### 10.3 可访问性

- WCAG 2.1 AA 合规 (per §8.1)
- 键盘 100% 可达

### 10.4 测试覆盖

- 单元测试: 每图表 ≥ 5 case (含边界)
- 集成测试: 22 图表 × 5 scope = 110 组合抽样 30%
- E2E: 5 个核心 user flow (创建 Report / 订阅 / 嵌入 Dashboard / 导出 / 钻取)

### 10.5 文档同步

- per AGENTS.md §3 报告 7 段结构
- 22 详细设计全部 git 实证 commit

---

## 11. 风险 & 缓解

| Risk | 影响 | 缓解 |
|---|---|---|
| RISK-CHART-01: 22 图表全部上线周期长 | 单 sprint 难消化 | 分 3 批: P0(8 个敏捷/cycle) / P1(7 个 distribution/time) / P2(7 个 version/advanced) |
| RISK-CHART-02: Recharts 大数据性能 | > 10K 点卡顿 | 数据采样 + 虚拟滚动 + 提示用户细化 filter |
| RISK-CHART-03: Jira 行为差异 | 用户切换困惑 | docs/jira-vs-star-compat.md 列出差异 |
| RISK-CHART-04: 导出性能 | PDF 50 页慢 | 异步生成 + 邮件通知 |
| RISK-CHART-05: Dashboard 嵌套查询风暴 | 1 Dashboard 12 Gadget 同时查 | 合并查询 + 共享 5min 缓存 |
| RISK-CHART-06: 订阅通知噪音 | 大量邮件 | per REQ-NOTIF-002 降噪 + digest |
| RISK-CHART-07: JQL 兼容度 | 用户迁移成本 | 文档化 StarQL 与 JQL 差异 |

---

## 12. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 scope + 22 图表 + 10 Gadget + NFR + 验收 + 风险 (per 2026-09-02 10:04 JST Ulysses 拍板) | 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira" |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
