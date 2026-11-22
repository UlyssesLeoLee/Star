/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/architecture/2026-11-15-lane-workspace/WBS-LANE-WORKSPACE-001.md"})
@endcypher
*/
# WBS-LANE-WORKSPACE-001 — Lane 工作空间收敛与授权上下文统一

日期: 2026-11-15 · 触发: 用户 8 大部分需求 (产品导航/Lane 上下文、工作空间布局、看板收敛、
Canvas 关系图、聊天/状态/Team、会话授权、状态可读性、实施验收)

## 0. 调查结论 (实际实现与数据, 2026-11-15 实测)

| 项 | 现状 | 差距 |
|---|---|---|
| 导航层级 | ProjectRunTree 渲染 Project→(fallback 默认LANE)→Worktree→Branch 四层; Sidebar 注释仍写 "Engineering Run" | Lane 未统一命名; 无 Lane 数据模型 |
| 数据类型 | `Worktree` 无 `lane_id`; 无 `Lane` interface; store 无 `lanes` 集合 | 需补 Lane 本地实体 + Worktree.lane_id |
| 导航状态 | navStore 仅 `selectedProjectId` | 缺 selectedLaneId/selectedWorktreeId/selectedBranch |
| Worktree 页 | 1279 行: AuthorizedProjectSelector(授权目录) + GitBindingPanel + BranchWorkspace + 17 状态机 | 授权上下文已有雏形, 需接 Lane 选择 |
| Branch 中央应用 | BranchWorkspace 5 tabs: tasks/agents/validation/nema/canvas | 需求 = Lane 级: Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins |
| 任务卡入口 | /work-item/[id] 子页面 + WorkItemDetailDrawer 两套并存 | 需求 = 同一个可关闭子窗口 |
| 看板 | 独立 /kanban 页 + BranchWorkspace tasks 双套; 列内创建/拖动/增删/重命名已接线 (2026-11-15) | 需收敛为一套 + 数据来源标识 |
| Canvas | /canvas/[id] 3 标签 + /canvas/worktree-graph 5 视图 (2026-11-15 已互链) | 需经 Worktree 上下文访问 + 任务卡链接 |
| 聊天 | RightChatRail 全局折叠栏(默认展开=聊天); SideChatPanel task/global 双 scope | 底栏需 WORKTREE/GLOBAL 范围选择 + 当前范围标识 |
| 会话/授权 | AuthorizedProjectSelector + AuthorizedProjectDirectoryState 存在; 401/403 区分未验证 | 需按需求六核对 |
| mock 数据 | seed.ts deterministic mock (产品约束: preview/live 双模式, 本地 DB 基本为空) | 正式视图需标识数据来源; 未授权操作禁用 |
| 实施状态 | Phase 1 ✅ 462260e8 · Phase 2.1/2.2/2.4/2.5(部分) ✅ cfbfe4c5+304f3ce5 · Phase 3.2 ✅ cf48593b · Phase 4.1/4.3 ✅ 304f3ce5 · Phase 5.2/5.3/5.4 ✅ 304f3ce5 · Agent并入Canvas ✅ 7b129089 · IM1.0 改造 ✅ 95afa636 · 1b 创建栏 ✅ 9187f43d | 剩余: 2.3(部分)/2.5(部分)/3.1/3.3/3.4/4.2/4.4/5.1/5.5/6.x/7.x/8.2 |

## 1. 分阶段计划

### Phase 1 — Lane 命名与导航上下文 (P0, 本 session 实施)
- [x] 1.1 Sidebar/i18n 注释与文案 "Engineering Run" → "Lane" (runDirectory 段落保留 fallbackLane 语义)
- [x] 1.2 store 加 `lanes` 集合 + `Lane` 类型 (本地可建/授权/列出, 无远端身份); Worktree 补 `lane_id?`
- [x] 1.3 navStore 加 selectedLaneId/selectedWorktreeId/selectedBranch + 深链同步
- [x] 1.4 ProjectRunTree 四层渲染 Lane 真实数据 (fallback 目录可点选)
- [x] 1.5 选择 Worktree 后统一授权上下文: 面包屑/中央应用/任务详情/CLI checkout 焦点同一上下文; 绑定信息缺失显式显示"未绑定"状态

### Phase 2 — 工作空间布局整合 (P0/P1)
- [x] 2.1 BranchWorkspace tabs → Lane 级中央应用: Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins
- [x] 2.2 Project 左栏共用顶部目录 / AI 更新页签和底部状态标签；Group 投影的 Branch 任务使用相同状态筛选。右边缘提供可配置的应用快捷导航。
- [ ] 2.2a 将 AI 更新页签接到授权事件流。当前显示未接入状态。
  - 2026-11-20 回归：侧栏、移动目录、目录树、导航状态和任务卡的 6 个 Vitest 文件通过 61 项测试。覆盖页签保留、共享筛选、悬停进入快捷菜单、钉选与移除及触屏打开。
- [x] 2.3 任务卡统一可关闭子窗口 (3cc5b6b7: kanban 4 处 push → Drawer; /work-item 路由残留不再作入口)
- [x] 2.4 Project Worktree Index 定位文案完成 (304f3ce5, wt-finish lane: 跨 Lane/Worktree/Branch 汇总入口语义)
- [x] 2.5 closeTaskCard 验收通过 — 切换 Worktree 上下文验证仍缺 (backlog)

### Phase 3 — Backlog/Sprint/看板收敛 (P1)
- [ ] 3.1 独立 /kanban 与 BranchWorkspace Backlog 合并为一套 (保留列内创建/拖动/列增删重命名)
- [x] 3.2 kanban 预览数据标识 (cf48593b) + 聊天未接入禁用写入 (591915cb) — kanban 写操作授权禁用仍缺 (backlog)
- [ ] 3.3 创建/移动/刷新/状态变更身份与上下文一致性验证
- [ ] 3.4 失败提示 + 草稿保留; 授权失败不重放写操作

### Phase 4 — Canvas 上下文与关系图 (P1)
- [x] 4.1 Canvas ?worktree= 上下文条 + 数据隔离诚实标注 (591915cb) — 任务卡链接未做 (backlog)
- [ ] 4.2 独立 Canvas/Group Canvas/Worktree 关系图对象身份与持久化边界核对
- [x] 4.3 5 视图四态 (空/未接入/未授权/正常), 未授权接真实 401 信号 (56442775)
- [ ] 4.4 Canvas↔任务双向跳转/深链/保存失败草稿保留/切换 Worktree 授权隔离验证

### Phase 5 — 聊天/Worktree 状态/Team (P1)
- [◐] 5.1 group 页右栏 chat/worktree(默认折叠)/team 已有 — Team 管理动作未接后端
- [x] 5.2 chatStore chatScope + RightChatRail 切换器 + 当前范围 label (304f3ce5, chat lane)
- [x] 5.3 GLOBAL 授权目标下拉 + 未选禁用 + 提交重校验 + 失败保草稿 (304f3ce5, chat lane)
- [x] 5.4 chat rail/panel 未接入徽标 + 不生成示例消息 (304f3ce5 + group 页既有)
- [ ] 5.5 加载/断线/恢复/错误/慢消费者状态; 实时协作 owner 事实/授权范围推送/冲突保护/有界资源

### Phase 6 — 会话与授权一致性 (P1)
- [ ] 6.1 统一宿主会话与授权边界 (Lane 目录/Group/Backlog/Canvas 同一会话)
- [ ] 6.2 401 关闭能力; 403 ≠ 会话失效 (UI 区分)
- [◐] 6.3 groupProjection 401 代际守卫强化 (55728d52) — 写操作重放防护未做
- [ ] 6.4 不凭前端状态推断认证/执行权限

### Phase 7 — 状态/可读性/性能 (P2)
- [ ] 7.1 统一加载/空/未连接/无权限/失败/恢复中/成功提示 + 可执行下一步
- [ ] 7.2 默认精简技术说明; 诊断/策略版本/执行证据可展开
- [ ] 7.3 桌面尺寸/窄窗口/缩放/字号/对比度/键盘/焦点/无障碍检查
- [◐] 7.4 ProjectRunTree 已虚拟化 (rows.slice + VIEWPORT_HEIGHT/OVERSCAN) — 聊天/事件/日志未做有界
- [ ] 7.5 Hooks 保持"高级设置→Hooks"; Worktree/Lane Detail/BI 展示有效策略并深链

### Phase 8 — 实施验收 (贯穿)
- [x] 8.1 全 session 每批提交均过 tsc 0 / vitest 128·1064 / cgg stamp / 浏览器实测 (自审报告 2026-11-15)
- [ ] 8.2 文档同步 (本 WBS 打钩 + SRS/BD 增量)
- [x] 8.3 自审报告 + 移交清单已给出 (2026-11-15)

## 2. 关键决策 (实施中记录)

- Lane 本地实体先行: `Lane { id, tenant_id, project_id, name, git_remote_url?, git_provider?, default_branch? }` —
  无远端绑定信息时 Lane 依然可创建/授权/列出/管理 (需求一.2); Worktree.lane_id 可选引用。
- 内部 Run 命名 (RunWorkspaceShell/RunBiPanel/engineering_run 路由/ID) **保留不动** — 界面改名不代表
  内部迁移完成 (需求一.5); 兼容映射: 前端 Lane 显示层 ↔ 内部 engineering_run 契约, 迁移列入 backlog。
- seed/live 双模式保留 (产品约束, 本地 DB 基本为空) — 但 UI 必须标识数据来源 (preview 徽标),
  未授权写操作禁用 (需求三.2/3.3)。

## 3. 修订历史

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 初版: 调查结论 + 8 阶段分阶段计划 + 关键决策 (Phase 1 打钩) | 462260e8 |
| 2026-11-15 | v1.1 | Phase 2.1/2.2 打钩: BranchWorkspace → Lane 级中央应用, Group Workspace 双页签 | cfbfe4c5 |
| 2026-11-15 | v1.2 | §0 状态表补实施状态行 (Phase 1/2.1/2.2 已完成 commit 指认) + 修订历史表 | 本提交 (见 git log) |
| 2026-11-15 | v1.3 | 4 lane 并行 fan-out (docs/chat/canvas/wt, delegate_task deepseek probe 通过) rebase+merge dev; 2.4/2.5/4.1/4.3/5.2/5.3/5.4 打钩; Agent 中心并入 Canvas; IM1.0 chat 域模型; 1b 创建栏 6 控件 | 304f3ce5→9187f43d |
| 2026-11-15 | v1.4 | 实际状态全量同步: ◐ 半完成项标注意外; 移交 Codex (MAGOS disarm 命令交付用户终端) | 本提交 |
| 2026-11-15 | v1.5 | 结构定版: Agent 中心拆分 (90111428) + workflow 回归 canvas「流程」标签 (e804a570) + rail 顶格/agent 设置 (d05c6e3c) + 任务卡子窗口全站统一 (3cc5b6b7); WBS 状态回写 | 本提交 |
