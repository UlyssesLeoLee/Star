# Brief: WBS-W14-UI0-NAVIGATION-IA-20261026

**Agent**: explorer
**Phase**: W14-UI0
**Created**: 2026-10-26 07:33:39

---

在此分支只新增 docs/reviews/WBS-W14-NAVIGATION-IA-20261026.md。不要修改产品代码、规范主文档或 WBS；不要运行测试。产出是 UI-0 信息架构评审：逐项映射 Project → Engineering Run → Worktree → Branch 的主树、Project Worktree Index、选择 Worktree 后才呈现的 Run Apps tabs、Task Card 内 CLI、Canvas 与 Task Card 同级、WORKTREE/GLOBAL 聊天范围、全局设置与 Project/Run 作用域。覆盖未选 Project、Run 无 Worktree、已选 Worktree、授权过期/撤权、加载/空/错误/重试/深链状态；分别标明已实现、仅 UI/占位、未实现，不把 mock/fixture 当产品能力，不提及竞品品牌。提出 UI-0 决策清单和 UI-1 流程原型验收；性能建议须有有界缓存/队列、按需加载、树虚拟化、取消/drain、权限切换时清空旧 scope、可访问性与复现指标，不凭空设内存或时延阈值。

依据：本 lane 的基线是 dev af84ba2061c73f824fbb31160565ba89569280ad。docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md 的 W14 行与 §15；docs/reviews/WBS-W14-RUST-DESKTOP-UI-OPTIONS-20261026.md。CodeGraph 项目 D:\Star 的 status 为 up-to-date（1907 files/37350 nodes/117596 edges），source query 已读取 frontend/src/components/Sidebar.tsx、frontend/src/components/run/ProjectRunTree.tsx 和 frontend/src/components/run/RunWorkspaceShell.tsx：左侧已有 ProjectRunTree、ProjectWorktreeIndexLink；目录树 36px rows/288px viewport/3 overscan 并有键盘导航；RunWorkspaceShell 的 6 个同级 tabs 只在有授权 Worktree focus 后显示，但当前仅 Task Cards 和 Run BI 面板装载，其他 tabs 显示待迁移；CLI 与聊天发送 runtime 未装配，WORKTREE/GLOBAL scope 当前只在焦点有效时可选。不要把这些局部 UI 证据写成生产闭环。

CodeGraph MCP 不提供 check_index_coverage 工具；只作正面源证据，不做全仓穷尽/否定性结论，并在报告限制处说明。遵守用户已确认的导航、owner、hostless Run、低内存和原生 Hook 原则。完成后自审只读差异，报告精确文件、证据和未验证事项。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
