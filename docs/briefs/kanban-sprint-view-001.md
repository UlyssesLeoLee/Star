# Brief: kanban-vmodel-jp Sprint 视图 — KANBAN-SPRINT-001

> **任务卡 ID**: `KANBAN-SPRINT-001`
> **目的**: 在 `D:\Star\deliverables\kanban-vmodel-jp` 现有 Kanban (WIP + 5 状态 + V9 阶段) 基础上, **不破坏 Kanban 逻辑**, 加 **Sprint 视图** 作为并列模式
> **触发**: 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图" (per ask_user 选项 A)
> **范围**: `deliverables/kanban-vmodel-jp/{index.html, styles.css, app.js, data.js, README.md}` 5 份文件; `docs/kanban-vmodel-jp/` 新增 1 份报告
> **依赖**: 现有 Kanban 完全保留, 5 视图 (`kanban | list | timeline`) 不动, 新增第 4 视图 `sprint` 通过 topbar Tab 切换
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **代签**: per 19:39 JST 用户授权

---

## 0. 目标

让用户在保留 Kanban (WIP) 全部能力的同时, 用 Sprint 模式做时间盒迭代管理:

| 能力 | Kanban (现有) | Sprint (新增) |
|---|---|---|
| 时间盒 | 无 | 固定 (默认 2 周, 1-4 周可配) |
| 任务来源 | 全部 184 task + 4 行业预设 | 从 Backlog 拖入 Sprint |
| 列定义 | 5 状态 (Backlog/ToDo/進行中/レビュー/完了) | 同 5 状态 (仅显示 Sprint 范围内任务) |
| WIP 限制 | ✅ 列级别 | ✅ 列级别 (沿用) |
| 完成定义 | 拖入「完了」 | 拖入「完了」 + Sprint 结束自动 close |
| 度量 | 简单 (5 列) | Velocity (5 sprint) + Burndown (本 sprint) + 历史表 |
| 仪式 | 无 | 启动/站会/评审/回顾 (留 P3) |

---

## 1. WBS — 3 子阶段 / 1.5-2.0M token 总预算

| # | 阶段 | 子项 | 命中维度 | 自动化档 | 落档脚本 | token 估 | 状态 |
|---|---|---|---|---|---|---|---|
| **P1** | Sprint 核心 | (a) Sprint 数据模型 + localStorage 持久化<br>(b) Topbar Tab 切换 (Kanban/Sprint)<br>(c) Sprint CRUD: 创建/启动/完成/取消<br>(d) Sprint Planning UI: Backlog 视图 + 拖入<br>(e) Sprint Board: 仅显示 Sprint 任务, 5 列沿用<br>(f) Sprint Header: 名称/Goal/剩余天数/进度条 | V, S | **[M]** | `automation/kanban_sprint_gen.py` (per P1 单脚本生成器) | 0.5-0.7M | 待开 |
| **P2** | Sprint 度量 | (a) Velocity 图: 最近 5 sprint 完成小时数 (SVG bar)<br>(b) Burndown 图: 当前 sprint 每日剩余小时数 (SVG line)<br>(c) Sprint 历史表: 名称/起止/完成率/Velocity<br>(d) Capacity: 团队规模 × 可用工时配置 | V, S | **[M]** | `automation/kanban_sprint_charts.py` | 0.5-0.7M | 待开 (依赖 P1) |
| **P3** | Sprint 仪式 | (a) Standup notes: 每日 3 问模板 (昨日/今日/障碍)<br>(b) Sprint Review: 完成的 task 列表 + 演示备注<br>(c) Retrospective: 3 列 (好/改进/行动) markdown 板<br>(d) Sprint Goal 横幅 + 启动会模板 | V, A | **[M]** | `automation/kanban_sprint_ceremonies.py` | 0.4-0.6M | 待开 (依赖 P2) |

**总预算**: ~1.5-2.0M token (per 2026-09-03 13:12 JST 拍板估算)

---

## 2. 守门对齐 (per AGENTS.md §4 + §4.1)

| 守门 | 应用方式 |
|---|---|
| #1 v19+ (Python 化) | P1/P2/P3 走 `scripts/automation/kanban_sprint_*.py`, commit message 含脚本相对路径 |
| #9 (不 commit 散落子代理产出) | 全部 Mavis 主上下文 + 自动化脚本生成, 不派 worker |
| #11 (缺标比错标) | 已知缺口在报告 §3 显式列 |
| #12 (AI 协作文档治理) | 不沿用历史叙事, sprint 行为按当前代码实现, 不写 "per X 历史形态" |
| #20 v20 (subagent brief 必先落地) | 本 brief 已落 `docs/briefs/kanban-sprint-view-001.md` |
| #21 v21 ([P] 子项 docs 同步) | 自动化档更新 `docs/automation-design.md` §4.7 + `scripts/automation/registry.md` |

---

## 3. 验收标准 (per stage)

### P1 收官 (commit 后)
- ✅ Topbar 新增 Sprint Tab, 点击切换 Sprint 视图
- ✅ Sprint 视图默认显示 "无活跃 Sprint" 空状态, 提供 "创建 Sprint" 按钮
- ✅ 创建 Sprint: 名称 + 持续时间 (1-4 周) + 起始日 + Goal → localStorage `vmodel-sprints-v1`
- ✅ **Jira 設計: Sprint 計画 Backlog 优先** (per 2026-09-03 13:55 JST Ulysses 反馈) — Sprint Plan modal 仅显示 `status === 'backlog'` 的 task; 非 backlog task 无法拖入/添加, 弹错误 toast
- ✅ **Jira 設計: Sprint 外した task 自动回 Backlog** — `removeFromSprint` 调 `t.status = 'backlog'`
- ✅ **Jira 設計: 中止 Sprint 全件回 Backlog** — `cancelSprint` 调 `returnSprintTasksToBacklog(s)`
- ✅ **Jira 設計: 完了 Sprint 未完了 task 回 Backlog** — `completeSprint` 调 `returnSprintTasksToBacklog(s, {onlyIncomplete: true})`
- ✅ **Jira 設計: 削除 Sprint 全件回 Backlog** — sprintEdit deleteBtn 调 `returnSprintTasksToBacklog(draft)`
- ✅ Sprint Planning modal 显示 Jira 設計 hint + 计划済非 backlog 警告
- ✅ Sprint Board: 5 列 (沿用), 仅显示 sprint 范围内 task
- ✅ Sprint Header: 名称 / Goal / 剩余天数 / 进度条 (已完成/总估算)
- ✅ Start/Complete/Cancel 生命周期正常, Kanban 视图不受影响
- ✅ 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P1-REPORT.md` v0.1 落档 (per STAR-7 段结构)
- ✅ 守门 #1 v19: `scripts/automation/kanban_sprint_gen.py` 已落档, commit message 含脚本相对路径
- ✅ cargo 不适用 (本仓 vanilla JS), 守门 #1 改 "浏览器 console 0 err + 184 task 渲染不破坏"

### P2 收官
- ✅ Velocity 图: SVG bar, 5 sprint × 完成小时数, 鼠标悬停 tooltip
- ✅ Burndown 图: SVG line, 当前 sprint 每日剩余小时数 (含理想线对比)
- ✅ Sprint 历史表: 名称 / 起止 / 完成率 / Velocity / Goal 达成
- ✅ Capacity: 团队规模 × 每周可工时 → 自动算 sprint capacity vs commitment
- ✅ 报告 `SPRINT-VIEW-P2-REPORT.md` v0.1 落档

### P3 收官
- ✅ Standup notes: Sprint 详情面板新增 "Daily Standup" Tab, 每日 3 问输入框 (昨日/今日/障害) + 過去履歴 (最新 7 天, 今日分除外)
- ✅ Sprint Review: 完了 task 列表 (checkbox 选 Demo 候補) + Review Notes (markdown textarea) + Demo 数实时更新
- ✅ Retrospective: 3 列 KPT 板 (✅ Keep 绿 / ⚠️ Problem 黄 / 🎯 Try 蓝) + Markdown 导出 (含 Velocity/Capacity 元数据)
- ✅ Sprint Goal: 緑色横幅 + 編集モード (textarea 切替) + 启动会テンプレ (`<pre>` block) + 保存
- ✅ 报告 `SPRINT-VIEW-P3-REPORT.md` v0.1 落档

---

## 4. 已知缺口 (per 守门 #11 缺标比错标)

- [P1] Sprint 范围**不跨多个 Sprint** — 同一 task 不能同时属于两个 active sprint (1 task 1 sprint)
- [P1] 没有 Story Points 概念, **用 `estimate` 字段 (小时) 当点数** — 后续如需 Fibonacci 1/2/3/5/8/13 单独 P4
- [P1] **Sprint 中途增删 task 不更新 velocity** — velocity 仅在 Sprint Complete 时统计
- [P1] **Sprint Plan modal backlog 空时无 CTA** — 需手动跳回 Kanban Board 把 task 拖到「バックログ」列, 提示文案已加 (per 2026-09-03 13:55 JST)
- [P2] Burndown 每日数据**手动**录入, 不接外部时间跟踪 (Toggl/Jira 时间日志)
- [P2] Velocity **不区分角色** (dev/qa/devops), 单值展示
- [P3] Standup notes **不推送**到 Slack/Teams, 仅本地
- [P3] Retrospective **不导出 PDF**, 仅 localStorage + JSON 导出
- [ALL] **无多人协作** — 当前 localStorage 单机, 多人需服务端 (out of scope)
- [ALL] **无 SRE Lead / DDD Review 拍板** — Mavis 代签, 5 域真人到位后追溯 (per 守门 #3)
- [P1 v0.2 新增] **既存数据无 backlog 校验** — 旧用户 localStorage 里的 task 若 status !== 'backlog' 且不在 sprint, Sprint Plan 不会显示但也不会被清理 (per 守门 #11 缺标)
- [P1 v0.2 新增] **手动把 sprint 内 task 状态从 todo 改 doing 时, 不会触发 snapshot** — 拖拽改状态才触发, 任务详情 modal 改 status 不会
- [P3] **Standup 不区分个人** — Jira 实际每个团队成员各填 1 份, 本实现是团队级 1 份 (per 守门 #11 缺标, 团队规模小时可接受)
- [P3] **Standup 履歴 无全文搜索** — 仅按日期倒序展示
- [P3] **Sprint Goal 启动会模板不可編輯** — 写死在 `<pre>` 块, 用户不能个性化
- [P3] **Sprint Review Notes 不支持 Markdown 预览** — 纯 textarea, 需手动外部预览
- [P3] **Sprint Review Demo 候補 不能拖拽排序** — 按 task ID 字母序, 不能调整演示顺序
- [P3] **Retrospective 不能跨 Sprint 模板** — 每个 Sprint 独立, 不可复用上次 KPT
- [P3] **未提供 Standup 自动提醒** — 需用户主动打开 ceremonies 面板
- [P3] **Ceremonies 不能推送通知** — 无 Slack/Teams/Email 集成. 已知缺口 (per Jira 实际, task 状态变更通常在 board 上发生)

---

## 8. Jira 設計参考

per 2026-09-03 13:55 JST Ulysses 反馈 "参考jira设计":

| 行为 | Jira 設計 | 本实现 (P1 v0.2) |
|---|---|---|
| Backlog 是独立状态 | ✅ Backlog 是 pre-sprint 区, 不属于任何 sprint | ✅ `status === 'backlog'` 任务在 Kanban 板"バックログ"列 |
| Sprint 只能从 Backlog 拉 | ✅ Drag from Backlog to Sprint | ✅ `addToSprint` 校验 `status === 'backlog'` |
| Sprint 内 task 有独立 status (To Do/In Progress/Done) | ✅ Sprint board 是独立视图 | ✅ Sprint board 5 列 (Backlog/ToDo/Doing/Review/Done), 沿用 Kanban 列定义 |
| 完了 Sprint 未完了 task 回 Backlog | ✅ 自动回流 | ✅ `returnSprintTasksToBacklog(s, {onlyIncomplete: true})` |
| 中止 Sprint 全 task 回 Backlog | ✅ 取消时回流 | ✅ `returnSprintTasksToBacklog(s)` |
| 削除 Sprint 全 task 回 Backlog | ✅ (Jira 实际: 删除是罕见操作, 通常取消即可) | ✅ sprintEdit modal deleteBtn |
| 移出 Sprint 单 task 回 Backlog | ✅ Drag from Sprint Board to Backlog (跨视图) | ✅ `removeFromSprint` 重置 status='backlog' |
| Sprint velocity 自动算 | ✅ Completed story points 聚合 | ✅ `sprintDoneHours(s)` (sum of done estimate) |

**Jira 限制 (本实现超越的部分)**:
- ✅ Sprint Goal 字段 — Jira Scrum 板有
- ✅ 起動/Review/Retrospective 仪式 — Jira Scrum 板有, 计划 P3
- ❌ Story Points (1/2/3/5/8/13 Fibonacci) — Jira 标配, 本实现用 hours (estimate) 当 points (per 守门 #11 缺标)
- ❌ 多人协作 / 服务端同步 — Jira 标配, 本实现 out of scope (localStorage 单机)

---

## 5. 报告结构 (per STAR 7 段结构 + AGENTS.md §3)

每 P 报告必含:

1. §0 目的
2. §1 改动矩阵 (新增/修改/删除文件清单 + 行数)
3. §2 验证摘要 (浏览器 console 0 err + task 渲染完整 + 关键交互 e2e)
4. §3 已知缺口 (per §4 缺口清单勾选)
5. §4 子代理失败接手清单 (本任务无 subagent, 此节标注 "无 subagent 派发, 全部 Mavis 主上下文 + 自动化脚本")
6. §5 守门规则 (15-17 项核对)
7. §6 签字栏 (5 角色: 架构/SRE Lead/平台/评审主持/PM, per AGENTS.md §6.2)
8. §7 修订历史

---

## 6. 不在范围 (out of scope)

- ❌ 改 Kanban 现有逻辑 (5 状态 / WIP 限制 / V9 阶段)
- ❌ 改 data.js 行业预设任务 (P1-P9 4 行业已落档)
- ❌ 改 4 视图现有 3 视图 (kanban/list/timeline)
- ❌ 多人协作 / 服务端 / 实时同步
- ❌ 改 STAR 仓其他 deliverable (gm-backend / gm-console / 前端项目)
- ❌ 推 origin (per 守门 #1 R-05 反转但仍需 Ulysses 拍板)
- ❌ 撤销现有 13 个 kanban-vmodel-jp commit

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 3 阶段 WBS (P1 核心 / P2 度量 / P3 仪式), 1.5-2.0M token 总预算, 守门对齐, 已知缺口 9 项 | 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图" |
| v0.2 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **Jira 設計 Backlog 优先** 增量: 4 处数据流修改 (addToSprint 校验 / removeFromSprint 重置 / completeSprint 未完了回流 / cancelSprint + 削除 全件回流) + 新增 `returnSprintTasksToBacklog()` ヘルパー + Sprint Plan modal Jira 設計 hint + 非 backlog 警告 + 已知缺口 +2; §3 验收 + §8 Jira 設計参考新增; 自动化档 `kanban_sprint_gen.py` 校验项 43 → 54 (+11) | 2026-09-03 13:55 JST Ulysses 反馈 "进入sprint前应该在backlog, 删除sprint列时, 里面的内容也应该进入backlog, 参考jira设计。所有文档要更新好" |
| v0.3 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **P3 收官** 增量: §3 验收 P3 详情 + 已知缺口 +8 (Standup 不区分个人 / Standup 履歴无搜索 / Goal 模板不可編輯 / Review Notes 无 preview / Demo 候補 不能排序 / Retro 不能复用 / Standup 无提醒 / Ceremonies 无推送); 自动化档 54 → 93 (+38); 总 token 估 ~1.4M (P1 0.5M + P2 0.4M + P3 0.4M + 文档 0.1M) / 预算 1.5-2.0M | 2026-09-03 14:05 JST Ulysses 拍板 "开 P3 仪式" + 14:20 JST Mavis 推进 P3 收官 |
