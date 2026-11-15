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

## 1. 分阶段计划

### Phase 1 — Lane 命名与导航上下文 (P0, 本 session 实施)
- [x] 1.1 Sidebar/i18n 注释与文案 "Engineering Run" → "Lane" (runDirectory 段落保留 fallbackLane 语义)
- [x] 1.2 store 加 `lanes` 集合 + `Lane` 类型 (本地可建/授权/列出, 无远端身份); Worktree 补 `lane_id?`
- [x] 1.3 navStore 加 selectedLaneId/selectedWorktreeId/selectedBranch + 深链同步
- [x] 1.4 ProjectRunTree 四层渲染 Lane 真实数据 (fallback 目录可点选)
- [x] 1.5 选择 Worktree 后统一授权上下文: 面包屑/中央应用/任务详情/CLI checkout 焦点同一上下文; 绑定信息缺失显式显示"未绑定"状态

### Phase 2 — 工作空间布局整合 (P0/P1)
- [x] 2.1 BranchWorkspace tabs → Lane 级中央应用: Backlog/Sprint、Canvas、Workflow/LangGraph、BI/Benchmark、Plugins
- [x] 2.2 Group Workspace 左侧双页签: 导航树 / AI 更新 (Branch 任务按工作流状态分组; 底部状态筛选同步两页签)
- [ ] 2.3 任务卡统一为可关闭子窗口 (Drawer 保留 /work-item 子页面择一, 全站一致)
- [ ] 2.4 Project Worktree Index 保留为汇总入口 (不冒充生产管理)
- [ ] 2.5 深链/前进后退/关闭任务窗/切换 Worktree 验证

### Phase 3 — Backlog/Sprint/看板收敛 (P1)
- [ ] 3.1 独立 /kanban 与 BranchWorkspace Backlog 合并为一套 (保留列内创建/拖动/列增删重命名)
- [ ] 3.2 正式视图数据来源标识 (authorized/server vs preview/seed); 未授权命令禁用 + 原因说明
- [ ] 3.3 创建/移动/刷新/状态变更身份与上下文一致性验证
- [ ] 3.4 失败提示 + 草稿保留; 授权失败不重放写操作

### Phase 4 — Canvas 上下文与关系图 (P1)
- [ ] 4.1 Canvas 作为 Lane 应用入口, 经当前 Worktree 上下文访问; 可链接授权范围内任务卡
- [ ] 4.2 独立 Canvas/Group Canvas/Worktree 关系图对象身份与持久化边界核对
- [ ] 4.3 关系图 5 视图只展示有权读取数据; 移除固定示例; 区分空/未授权/未接入/不可用
- [ ] 4.4 Canvas↔任务双向跳转/深链/保存失败草稿保留/切换 Worktree 授权隔离验证

### Phase 5 — 聊天/Worktree 状态/Team (P1)
- [ ] 5.1 右侧默认聊天; Worktree 状态默认折叠; Team 统一 Runtime/Daemon 与 Worker/Router 小队
- [ ] 5.2 底栏聊天 WORKTREE/GLOBAL 范围选择 + 当前范围标识
- [ ] 5.3 GLOBAL 操作选择授权目标 + 提交重校验 (选择目标本身不授予权限)
- [ ] 5.4 未接入能力显式禁用, 不生成示例消息冒充
- [ ] 5.5 加载/断线/恢复/错误/慢消费者状态; 实时协作 owner 事实/授权范围推送/冲突保护/有界资源

### Phase 6 — 会话与授权一致性 (P1)
- [ ] 6.1 统一宿主会话与授权边界 (Lane 目录/Group/Backlog/Canvas 同一会话)
- [ ] 6.2 401 关闭能力; 403 ≠ 会话失效 (UI 区分)
- [ ] 6.3 旧会话结果不覆盖当前会话; 授权失败不重放写操作
- [ ] 6.4 不凭前端状态推断认证/执行权限

### Phase 7 — 状态/可读性/性能 (P2)
- [ ] 7.1 统一加载/空/未连接/无权限/失败/恢复中/成功提示 + 可执行下一步
- [ ] 7.2 默认精简技术说明; 诊断/策略版本/执行证据可展开
- [ ] 7.3 桌面尺寸/窄窗口/缩放/字号/对比度/键盘/焦点/无障碍检查
- [ ] 7.4 列表/事件/聊天/图数据分页或虚拟化; 无界日志防驻留
- [ ] 7.5 Hooks 保持"高级设置→Hooks"; Worktree/Lane Detail/BI 展示有效策略并深链

### Phase 8 — 实施验收 (贯穿)
- [ ] 8.1 每 phase: tsc 0 + vitest 全绿 + cgg stamp + 浏览器实测关键界面
- [ ] 8.2 文档同步 (本 WBS 打钩 + SRS/BD 增量)
- [ ] 8.3 最终报告: 已完成+验证证据 / 未接通+后续依赖 (按优先级)

## 2. 关键决策 (实施中记录)

- Lane 本地实体先行: `Lane { id, tenant_id, project_id, name, git_remote_url?, git_provider?, default_branch? }` —
  无远端绑定信息时 Lane 依然可创建/授权/列出/管理 (需求一.2); Worktree.lane_id 可选引用。
- 内部 Run 命名 (RunWorkspaceShell/RunBiPanel/engineering_run 路由/ID) **保留不动** — 界面改名不代表
  内部迁移完成 (需求一.5); 兼容映射: 前端 Lane 显示层 ↔ 内部 engineering_run 契约, 迁移列入 backlog。
- seed/live 双模式保留 (产品约束, 本地 DB 基本为空) — 但 UI 必须标识数据来源 (preview 徽标),
  未授权写操作禁用 (需求三.2/3.3)。
