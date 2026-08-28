# Star Frontend — 三栏自适应信息架构 v0.1

> **状态**: Draft v0.1
> **日期**: 2026-08-29
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **触发**: Ulysses 2026-08-29 04:02 JST 拍板 "Star 自创（推荐）" + "补齐 P1-P3 全部 33 项"
> **关联**: `docs/frontend/design/dynamic-interaction-design.md` (DnD/协同基线)
> **关联**: `docs/frontend/design/ui-detailed-design.md` (像素级 wireframe)

---

## §0 目的

定义 Star frontend 在 **补齐 P1-P3 全部 33 项 Jira 缺失功能** 时的统一信息架构, 避免子代理各自为政导致 UI 风格碎片化. 所有 wt (w6-w15) 必须遵守本规范, 不得在子代理 brief 中重新发明 UI 布局.

**核心问题**: Jira 把所有内容挤在中栏, 用户认知负荷高. Star 解决方案 = 把 **项目 / 任务 / 上下文** 三个第一公民显式分栏.

---

## §1 三栏信息架构

### §1.1 整体布局 (24-grid)

```
┌─────────────────────────────────────────────────────────────────┐
│  TopBar  (56px, sticky) — 面包屑 / 搜索 (Rovo Cmd+K) / 通知 / 用户 │
├──────────┬────────────────────────────────────┬──────────────────┤
│          │                                    │                  │
│ SideBar  │  MainWorkArea  (自适应, 1fr)       │  ContextPanel    │
│ (240px)  │                                    │  (320px,可关)    │
│          │                                    │                  │
│ 项目树   │  当前选中对象的主视图                │  当前对象的      │
│ 视图     │  (Board / List / Timeline / Gantt  │  上下文:         │
│ 筛选     │   / Calendar / Forms / Dashboard)  │  - 属性          │
│ 状态     │                                    │  - 评论          │
│          │                                    │  - 关联项        │
│          │                                    │  - AI 建议       │
│          │                                    │  - 操作历史      │
│          │                                    │                  │
│          │  Cmd+\ 折叠 → 仅 TopBar + Main      │  Cmd+. 折叠      │
│          │                                    │                  │
│          │  Cmd+1/2/3 切换 Main 视图预设       │                  │
│          │  (Compact / Comfortable / Focus)    │                  │
└──────────┴────────────────────────────────────┴──────────────────┘
```

**自适应规则**:
- 视口 ≥ 1440px: 三栏全展开 (240 + 1fr + 320)
- 视口 1024-1439px: ContextPanel 默认折叠, 按需唤出
- 视口 < 1024px: SideBar 改为图标列 (60px), ContextPanel 全屏覆盖
- 视口 < 768px: 移动端布局 (走 PWA 离线模式)

### §1.2 SideBar (左侧 240px)

**5 个分组, 最多 2 层展开, 避免认知爆炸**:

```
🏠 Home
   - 我的工作 (assigned to me)
   - 提及我 (@me)
   - 已关注 (watching)
   - 草稿 (drafts, 3)

📂 项目 (5)
   - 折叠/展开, 显示 Pinned 3 个
   - + 新建项目

👁 视图 (3)
   - 看板 / 时间线 / 列表 / 日历 / 概览
   - 视图预设 (我保存的 3 个)

🔍 筛选 (4)
   - 我创建的筛选
   - 团队共享筛选
   - 全局默认筛选 (未分配 / 高优 / 即将到期)

⚙ 管理 (1)
   - 自动化规则 (12)
   - 通知设置
   - 集成 (8)
```

**交互原则**:
- 折叠 = 记忆, 不重置
- Pinned 项 = 顶置, 上限 3 个
- 数字徽章 = 真实计数, 不模糊 (区别于 Notion 的 "9+")
- 键盘导航: `j/k` 上下, `Enter` 进入, `Space` 多选, `Cmd+B` 折叠整栏

### §1.3 MainWorkArea (中间, 核心)

**核心规则**: **每个对象类型对应一个"视图族"**, 用户进入对象 → 看到该对象类型的默认视图 → 可通过 `Cmd+1/2/3/4` 切换.

| 对象类型 | Cmd+1 | Cmd+2 | Cmd+3 | Cmd+4 |
|---|---|---|---|---|
| **项目 (Space)** | 看板 | 时间线 | 列表 | 概览 |
| **工作项 (WorkItem)** | 详情 | 子项树 | 历史 | 关联 |
| **Sprint** | 看板 | 燃尽 | 容量 | 回顾 |
| **Epic** | 时间线 | 子 Epic | 依赖图 | 进度 |
| **计划 (Plan)** | 时间线 | 容量 | 依赖 | What-if |
| **仪表板 (Dashboard)** | 主仪表板 | Wallboard | 嵌入视图 | 导出 |

**视图切换器**: TopBar 右侧第二组按钮, 永远显示当前对象类型的 4 个视图, 鼠标悬停显示键盘提示.

**视图密度预设** (Cmd+Shift+D 循环):
- **Compact** (默认): 14px 字体, 紧凑行高, 信息密度高 (适合 power user)
- **Comfortable**: 16px 字体, 标准行高, 平衡 (推荐)
- **Focus**: 18px 字体, 宽松行高, 当前项高亮 (适合演示 / 长时阅读)

**视图内筛选条**: 永远 sticky 在 MainWorkArea 顶部 (56px), 不跟随滚动. 包含: 搜索框 / 字段筛选 / 排序 / 分组 / 保存为视图.

### §1.4 ContextPanel (右侧 320px)

**核心定位**: 展示当前对象的"上下文", 不是"另一个视图". 用户不需要切走就能完成任务.

**5 个 Tab (永远显示, 不超过 5 个, 避免选择瘫痪)**:

| Tab | 用途 | 内容来源 |
|---|---|---|
| **属性** | 字段编辑 / 状态 / 分配 | `domain-work-item` |
| **评论** | 公开 + 内部 / @ 提及 / 反应 | `domain-comment` |
| **关联** | 链接 / 子项 / 依赖 / 阻塞 | `domain-relation` |
| **活动** | 状态变更 / 字段修改 / 通知已读 | `domain-audit` |
| **AI 助手** | Rovo 上下文建议 / 相似项 / 风险 | `star-context` ContextGraph |

**Tab 排序规则**:
- 默认按"使用频率 + 当前对象类型最优 Tab" 自动排序 (per 8/29 spec/MVP 决策)
- 用户可固定 (pin) 单个 Tab 顺序
- 永远有一个 Tab 高亮 (sticky), 切换不消失

**键盘**: `Cmd+[` / `Cmd+]` 切换 Tab, `Cmd+.` 全折叠

### §1.5 TopBar (顶部 56px, sticky)

**6 个区域, 严格从左到右**:

```
[☰ 折叠] [面包屑 /browse/... › 项目A › 看板 › 卡片#123] [Rovo Cmd+K 搜索框] [+] [🔔 3] [👤]
   1        2                                              3                4   5    6
```

- **1 折叠**: 折叠 SideBar (Cmd+B)
- **2 面包屑**: 可点击, 不超过 4 级, 超长省略
- **3 搜索框**: Rovo 入口, 跨数据源语义搜索 (per ADR 0031)
- **4 + 按钮**: 上下文创建 (项目内 = 工作项, 顶部 = 新项目)
- **5 通知**: 实时未读数 + 下拉 (走 SSE star-sse)
- **6 用户**: 头像 + 角色 + 5 域 Lead 标识

---

## §2 视觉语言 (避免认知负荷)

### §2.1 色彩 (Star 调色板, 非 Jira 蓝)

| 用途 | 颜色 | Hex | 用途 |
|---|---|---|---|
| Primary | Star Indigo | `#5B5BD6` | 主操作, 链接, 高亮 |
| Success | Moss Green | `#3D8B5F` | 已完成, 已通过 |
| Warning | Amber | `#C77B30` | 风险, 即将到期 |
| Danger | Rust Red | `#B53D3D` | 阻塞, 失败 |
| Neutral | Slate | `#475569` | 文本, 边框 |
| Surface | Pearl | `#F8FAFC` | 背景 |
| Surface-2 | Mist | `#EEF2F7` | 卡片 / 面板 |

**禁用**: 不用纯黑 `#000`, 不用纯白 `#FFF`, 不用饱和度过高的红/绿. 全部走 `bg-{name}-{50..900}` Tailwind 调色板.

### §2.2 字体 (双栈, 不依赖网络字体)

```
UI:    Inter, "Helvetica Neue", system-ui, sans-serif
代码:  "JetBrains Mono", "Fira Code", monospace
中文:  "PingFang SC", "Microsoft YaHei", sans-serif
```

字号 (5 档, 不引入第 6 档):
- 12px (caption)
- 14px (body small)
- 16px (body, 默认)
- 20px (h3)
- 24px (h2)
- 32px (h1)

行高: 紧凑 1.4 / 标准 1.6 / 宽松 1.8 (三档固定)

### §2.3 间距 (4px 基础栅格)

- 4 / 8 / 12 / 16 / 24 / 32 / 48 / 64
- 卡片内边距: 16px (Comfortable) / 12px (Compact)
- 组件间距: 24px (默认) / 16px (密集)
- 区块间距: 48px

### §2.4 圆角 (3 档, 不超过 4 档)

- `rounded-sm` (4px): 标签, 徽章
- `rounded-md` (8px): 按钮, 输入框, 卡片
- `rounded-lg` (12px): 模态框, 大卡片

### §2.5 阴影 (2 档, 不用 3+ 层叠加)

- `shadow-sm`: hover 提升
- `shadow-lg`: 模态 / 弹出菜单

---

## §3 认知负荷防御规则 (12 条硬约束)

1. **每屏 ≤ 7 ± 2 个信息块** (Miller's Law 严格遵守)
2. **导航深度 ≤ 3 级** (首页 → 对象类型 → 对象)
3. **每对象类型视图 ≤ 4 种** (主 + 3 备, 走 Cmd+数字切换)
4. **不出现"模态套模态"** (二级确认走 inline expansion)
5. **错误状态必须给"下一步"建议**, 不只说"出错"
6. **空状态必须给"开始"动作**, 不只说"暂无数据"
7. **loading 必须有 shape**, 不出现布局抖动 (用 Skeleton, 不用 spinner)
8. **快捷键全部可发现** (按 `?` 显示所有快捷键)
9. **所有数字用真实计数**, 不用 "99+", 不用 "若干"
10. **所有"全部"按钮必须给"显示更多"** (避免一次性渲染 1000+ 项)
11. **所有删除/破坏性操作走 inline 确认**, 不用 confirm dialog
12. **所有时间显示 2 种** (相对 + 绝对), 不只用一种

---

## §4 子代理 UI 交付基线

**所有 wt-w6 到 wt-w15 子代理交付时, 必须**:

1. **不发明新色**: 用 §2.1 调色板
2. **不发明新字号**: 用 §2.2 五档
3. **不发明新圆角**: 用 §2.4 三档
4. **不发明新间距**: 用 §2.3 八档
5. **新视图必须挂入 MainWorkArea §1.3 表格**: 不在 MainWorkArea 外另开新区域
6. **新对象类型必须挂入 ContextPanel §1.4**: 不在 ContextPanel 外另开新面板
7. **新交互必须有键盘快捷键**: 不只用鼠标
8. **新组件必须有 Storybook story**: 在 `frontend/.storybook/` 下, 跑 `pnpm storybook` 可看
9. **新页面必须有 Skeleton 状态**: 不出现 spinner 抖动
10. **新页面必须有 EmptyState**: 不出现"暂无数据"白屏

---

## §5 wt 拓扑与子代理分工 (per 8/29 04:02 JST 拍板)

### §5.1 wt 列表 (10 个, 全部从 main HEAD e7dfb30 拉分支)

| # | wt branch | 子代理 | 模块 | 目标 |
|---|---|---|---|---|
| 1 | `feat/w6-search` | worker-1 | JQL 解析 + 执行 | domain-search 8 层实装 + 单元测试 |
| 2 | `feat/w7-workflow` | worker-2 | 工作流引擎 | domain-workflow 8 层实装 + 状态机 |
| 3 | `feat/w8-automation` | worker-3 | 自动化引擎 | domain-automation 8 层实装 + 治理 |
| 4 | `feat/w9-board` | worker-4 | 看板 + WIP + 泳道 | domain-board 8 层 + 协同 (合并 w1 已完成) |
| 5 | `feat/w10-planning` | worker-5 | 路线图 + Plans + 容量 + What-if | domain-planning 8 层 (整合 w2 Gantt) |
| 6 | `feat/w11-report` | worker-6 | 报告引擎 | 新 crate `domain-report` 8 层 |
| 7 | `feat/w12-dashboard` | worker-7 | 仪表板 + Wallboard | 新 crate `domain-dashboard` 8 层 |
| 8 | `feat/w13-forms` | worker-8 | 拖拽表单 | 新 crate `domain-form` 8 层 + 前端表单构建器 |
| 9 | `feat/w14-ai` | worker-9 | AI Workflow Builder / Work Readiness / 报告洞察 / JQL AI | star-context 扩展 + 新 crate `domain-ai` |
| 10 | `feat/w15-integration` | worker-10 | Confluence / Slack / Teams 集成 + SSO/SCIM/SSO 完善 | domain-integration 扩展 + domain-identity 完善 |

### §5.2 与已有 wt 关系

- **w1-w5** (Kanban DnD / Gantt / Calendar / Workflow / Store) 保留, 不重做
- 子代理在新 wt 中**复用 w1-w5 已完成的前端实现**, 整合到三栏架构的 MainWorkArea
- w9-board 整合 w1-kanban / w3-calendar
- w10-planning 整合 w2-gantt
- w7-workflow 整合 w4-workflow-editor

### §5.3 merge 顺序 (避免冲突)

1. 先 merge w5-store (基础层, 无业务依赖)
2. 再 merge w6-search (被 w8-automation 依赖)
3. 再 merge w7-workflow (被 w8-automation / w9-board / w10-planning 依赖)
4. 再 merge w8-automation (被 w11-report 依赖)
5. 再 merge w9-board + w10-planning (可并行, 互不依赖)
6. 再 merge w11-report (依赖 w6-search + w7-workflow)
7. 再 merge w12-dashboard (依赖 w11-report)
8. 再 merge w13-forms (独立, 任何时候)
9. 再 merge w14-ai (依赖 w6-search + w7-workflow)
10. 再 merge w15-integration (独立, 最后)

### §5.4 子代理并行策略

- 全部 10 个子代理 **background 模式并行启动**
- 每个子代理独占 1 个 wt, 独立 commit
- 子代理互不通信, 通过本规范文档 (§1-§4) 协同 UI
- 子代理失败由 verifier 子代理接手, 不互相抢

---

## §6 验证基线 (per 8/29 04:02 JST 用户拍板)

每个子代理交付必须满足:

1. `cargo test -p <crate>` 全绿
2. `cargo clippy -p <crate> -- -D warnings` 0 警告
3. `pnpm test` (前端相关) 全绿
4. `pnpm storybook` (新组件) 可跑
5. 7 段 PHASE 报告 (per AGENTS.md §3) 已写
6. 8 项守门 (per AGENTS.md §4) 已自审
7. 签字栏 5 角色 (per AGENTS.md §9) Mavis 接手代签
8. commit author = `Ulysses <ulysses@mavis.local>` (per AGENTS.md §2.1)

---

## §7 已知缺口 (per 缺标比错标安全)

- 移动 App 暂未启动 (P3-15, wt-w16 后续)
- Feature Flag 集成 (P3-19, 暂未排 wt)
- GraphQL API (P3-20, 暂未排 wt)
- Marketplace 插件平台 (P2-13, wt-w17 后续)
- 多语种门户 (P3-21, 暂未排 wt)
- 访客角色完整实装 (P3-22, 暂未排 wt)
- 沙盒 What-if 规划 (P2-16) — w10-planning 包含 MVP
- 状态页 Statuspage (P3-30) — 定位外
- CSAT 满意度调查 (P3-29) — JSM 定位外

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 初版: 三栏自适应 + 12 条认知负荷防御 + 10 个 wt 拓扑 | 2026-08-29 04:02 JST Ulysses 拍板"Star 自创（推荐）" + "补齐 P1-P3 全部" |
