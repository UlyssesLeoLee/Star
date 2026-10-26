# WBS-W14 UI-1：导航与工作区流程原型

## 目标

用可点击原型验证用户能否从 Project 进入 Engineering Run、Worktree，再识别该 checkout 的本地 Branch；比较三种明显不同的工作区布局，同时保持相同的授权、App 与聊天状态规则。

## 基线与产物

- 基线：本地 `dev` `5841d31c4e5b8b1266d8c1aded747e07387b763e`。
- 原型：`docs/prototypes/w14-ui1-flow-20261026/index.html`，单文件、无依赖、无 API、无 shell、无持久化。
- 布局：A「项目树工作台」、B「Project Worktree Index」、C「聚焦式控制台」。由原型底部切换条和 `?variant=A|B|C` 分享布局。
- 流程：选 Project/Run/Worktree；显示本地 Branch；从 Project Worktree Index 返回 canonical Worktree；Task Cards 与 Canvas 同级；Task Card CLI 显示安全准入条件；WORKTREE/GLOBAL 草稿隔离；模拟深链授权、授权撤销和重新授权。

## 原型约束

所有目录、任务与分支名称均是标记清晰的演示场景数据，不代表产品事实。原型不调用服务端、不保存用户输入、不执行命令；CLI 操作保持禁用，并展示需要的 Runtime、sandbox、Hook、预算与实时授权门。未授权或授权检查中的状态不得显示 Worktree 应用内容。

## 验收标准

1. 三种布局都保持 Project → Engineering Run → Worktree → Branch 次序，远端宿主信息不改变层级。
2. 没有有效 Worktree 焦点时不显示 Run App tabs；Worktree Index 仍作为 Project 范围入口存在。
3. 任务卡与 Canvas 是同级 App；Canvas 引用能回到 canonical Task Card。
4. CLI 不启动任意终端；通过禁用态清楚说明完整运行闭环仍未接入。
5. GLOBAL 与 WORKTREE 可见且上下文、草稿分开；焦点变化、深链和撤权状态有可点击演示。
6. 页面明确标注是本地原型和演示场景；没有生产连接、性能或业务闭环声明。

## 当前状态

原型和同一实现者的静态复核已完成，等待目标 `dev` 集成；独立人员复核仍待补。复核修正了撤权/切换时残留 Branch 显示、Worktree 草稿跨 checkout 混用、重新选择绕过授权确认、切换 scope 后草稿未恢复、聊天 scope 操作引用了局部变量五处交互问题。布局胜出项尚未由用户选择；原型只供比较，不代表最终产品视觉决定。未启动浏览器、产品服务或 UI 测试，运行表现仍未验证。
