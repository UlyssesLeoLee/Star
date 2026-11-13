# WBS-W14 UI-0：Project / Run / Worktree 导航信息架构审查

- 日期：2026-10-26
- 基线：`dev@af84ba2061c73f824fbb31160565ba89569280ad`
- 范围：UI-0 信息架构决策与 UI-1 流程原型验收准备
- 状态：静态设计审查；未修改产品代码、未运行 UI 或性能验收

## 0. 目的与结论

本阶段先固定导航和状态边界，为后续交互原型、数据接入及桌面性能测试提供同一套可检查的模型。产品主树固定为：

```text
Project
└─ Engineering Run
   └─ Worktree
      └─ Branch（Worktree 当前本地 checkout ref/state）
```

Project 是用户进入工作区的第一个层级。Engineering Run 属于 Project，且可在没有远端宿主身份时创建、授权和管理。Worktree 是 Run 管理的本地 checkout；只有选择并重新确认授权的 Worktree 后，才建立当前 checkout 焦点并展示该 Run 的 App tabs。Branch 是这个 Worktree 的本地分支/checkout 状态叶；远端 provider、repository 和远端 branch 是可选关联元数据，不改变树的父子关系。

信息架构应围绕“当前正在管理哪个项目的哪些 Worktree”组织内容，而非让用户先进入通用 App 列表再找 checkout。当前源码已具有 Project 目录树、Worktree 选择和 Run App tabs 的部分界面；可见控件不代表其底层 provider、授权、终端、协作或数据闭环已经可用。

## 1. 层级与内容归属

| 层级 | 是什么 | 导航呈现 | 归属与交互约束 |
|---|---|---|---|
| Project | Star 原生项目身份及其成员授权。可关联 0..N Repository，并可显示项目级 Worktree Index。 | 左侧第一层；切换项目后显示该项目的 Runs。 | Project owner 是身份、成员授权和 Repository association 的权威来源。Project Index 是跨 Run 汇总/过滤入口，不另造一套 Worktree 真相；选择结果应回到对应 Run 与 Worktree 焦点。 |
| Engineering Run | Project 内独立的工作、Agent、任务、计划与分析边界；远端宿主身份可为空。 | 展开 Project 后列出 Runs。 | Run 可在无远端 repository/branch 时存在。Run 的 App、事件、配置快照和授权通过同一个 RunContext 约束；连接器关联不能成为 Run 创建或授权前提。 |
| Worktree | Run 管理的本地 checkout 与 Agent/CLI 工作焦点。 | Run 下的可选节点；支持选择、状态和受控生命周期动作。 | 选择时重新验证 Run 与 Worktree 权限、generation 和有效绑定。切换焦点要先撤销旧焦点可见数据、取消旧请求，再加载新焦点；无法确认时不展示旧数据。 |
| Branch | Worktree 当前的本地 checkout ref/state。 | Worktree 的末端信息，可显示名称、dirty/同步等本地状态。 | 不表示远端 branch，也不成为 Worktree 的父层。远端 branch 如有，只作为可辨别的 provider metadata 呈现，并明确标注来源。 |
| Project Worktree Index | Project 下跨 Run 的 Worktree 管理汇总视图。 | Project 层可达的 Index 入口，与树共享同一实体和筛选口径。 | 可按 Run、Worktree、Branch 与状态过滤；批量操作必须通过各 owner 的授权命令。不得通过 Index 直接写其他 owner 的数据库或绕过 Worktree lifecycle。 |
| Run Apps | Inbox、Task Cards、Canvas、Workflow / LangGraph、Run BI / Benchmark、Plugin Apps。 | 仅在选中有效 Worktree 后，在右侧作为同一层级 tabs 展示。 | 每个 App 收到同一 RunContext 和当前 Worktree focus；跨 App 引用使用授权的 canonical EntityRef/owner API。尚未接入的 App 必须明确 unavailable/待接入，不以静态数据或演示记录冒充。 |
| Project / Global 管理 | Project 与成员/Repository 管理、全局偏好、连接器和高级设置。 | 作为 Project 层操作或独立的全局设置目的地；不插入 Worktree 树内部。 | Repository association 归 Project；连接器的服务商信息仅在真实连接器设置中显示。Hook 编辑位于“高级设置 → Hooks”，Project/Worktree 策略继承与 Run 快照从此处管理，不增加树节点。 |

### 1.1 Run App 的共同规则

- Canvas 与 Task Cards 同级。Canvas 可关联/创建任务卡，但关联经 owner API、canonical EntityRef 和授权检查完成；画布节点不是第二份任务真相。
- CLI 从 Task Card 内启动。启动请求携带当前 Run、Worktree、Task、授权/Hook profile 版本及资源预算；CLI checkout 焦点固定到该 Worktree。取消/恢复、输出、验证证据和结果回写均回到相同任务与 RunContext。
- Plugin Apps 只能以版本化 capability 接入 Run 边界；热插拔只能更新应用注册/卸载状态，不得把任意 native code 装入桌面主进程，也不能扩大 Run 权限。
- Run BI / Benchmark 展示由可信 owner 事件和已验证投影产生的数据；客户端局部状态、fixture 或一次性预览不是业务事实。
- 有效 Hook 策略与执行结果可在 Run/Worktree 处显示摘要并深链到“高级设置 → Hooks”；最终策略编辑、审批和审计仍由高级设置承载。

## 2. 底栏聊天与当前焦点

底栏提供显式 `GLOBAL` / `WORKTREE` scope：

| Scope | 必需上下文 | 切换规则 |
|---|---|---|
| `GLOBAL` | 当前用户身份；不含隐式 Worktree 写入范围。 | 在未选择有效 Worktree 时是唯一可选范围。消息、检索和工具调用不得借用之前选中 Worktree 的旧授权。 |
| `WORKTREE` | 当前 Project、Run、Worktree generation 和实时授权。 | 仅在焦点仍有效时启用。Worktree 切换、撤权、Run 变更、session 失效或 checkout generation 变化时立即退出旧 scope，取消在途工作、丢弃旧上下文缓存，再验证后重新开放。 |

scope 应在输入区、消息列表和操作确认处持续可见；不依赖用户记住先前点击的 Worktree。发送前由服务端再授权，客户端 scope 选择不构成授权事实。Global 与 Worktree 会话之间不得静默复制消息、附件、草稿或工具上下文。断线恢复必须从 owner 的持久 cursor/receipt 读取；不能以固定周期轮询取代事件触发的更新。

## 3. 选择、授权与页面状态

下表是导航和 App shell 的 UI 状态合同。每次直接路由进入、从浏览器历史恢复或切换目标都执行相同授权流程。

| 状态 | 可见内容 | 禁止行为 / 下一步 |
|---|---|---|
| 未选 Project / 无可访问 Project | Project 选择入口或解释性空状态；全局设置仍可按自身权限访问。 | 不展示旧 Project 的 Run、Worktree、聊天或 App 数据。不能用 seed/mocked Project 补足目录。 |
| Project 已选，Runs 正在加载 | Project 名称与树 loading 状态；可继续访问明确的 Project/global 设置。 | 不将上一 Project 的 Runs 暂时挂在当前 Project 下；同一请求只允许一个有界加载状态，可取消。 |
| Project 已选，无可访问 Run | Project 空状态与受授权的创建入口。 | 不能假定该 Project 已绑定远端 repository；hostless Run 可正常创建。 |
| Run 已选，Worktree 未选 | Run 页面显示“选择 Worktree”的引导、该 Run 授权 Worktree 列表与加载/空/错误状态；不显示 Run App tabs。 | 不把最近访问的其他 Run/Worktree 自动当焦点。创建/导入 Worktree 必须使用受控 owner command。 |
| Worktree 已选，授权通过 | 更新左侧选中态；右侧显示该 Run 的同级 App tabs，并以 Worktree focus 初始化工作区。 | 在授权上下文、focus 和 generation 均匹配前，不挂载 App 数据组件或启用写操作。App 切换不能更改 Run owner。 |
| Worktree 页面正在授权/加载 | 只显示当前路由的 loading 状态；当前页面不闪现先前焦点的内容。 | 所有列表、Canvas、聊天和任务请求都应有取消句柄；路由 key 不匹配的晚到响应必须被丢弃。 |
| Session 过期、撤权、绑定不一致 | 清空受保护内容与 WORKTREE scope，显示可理解的授权/绑定错误及安全重试/重新选择入口。 | 401 或身份代际失效时不能重放写请求；无完整授权事实、授权目录不可用或 revision 冲突时 fail closed。 |
| 深链指向未知或越权实体 | 显示不可用/无权限状态，不返回受保护对象的名称、计数或内容。 | 不用 URL 中的 Project/Run/Worktree ID 作为授权凭据；服务端重新检查所有 scope。 |
| 分页、断线或 App 加载失败 | 保留仍有效的导航上下文，显示局部错误、cursor 状态和显式重试。 | 重试只重复幂等读取或同 idempotency key 的受控命令；不能跨 Run/Worktree 复用 cursor 或重复副作用。 |

## 4. 当前实现与目标差距

以下是本次定向源码查询得到的正向证据；它不构成对全仓或生产部署的穷尽审计。

| 区域 | 已观察到的实现 | 状态判定 | UI-0/UI-1 后续门 |
|---|---|---|---|
| 左侧导航入口 | Sidebar 调用 `ProjectRunTree` 和 `ProjectWorktreeIndexLink`；Project 树组件的说明文字使用 Project → Engineering Run → Worktree → Branch。 | 局部实现 | 原型确认 Project 是第一个可见层级；Index 明确属于 Project 并回到树中 canonical Run/Worktree。检查窄侧栏、展开态和小窗口的信息层次。 |
| 树浏览性能 | `ProjectRunTree.tsx` 使用固定行高、固定 viewport、高度窗口切片和 overscan，并实现上下/左右/Home/End 键盘导航。 | 已有局部虚拟化与键盘处理；尚无目标机可访问性或内存测量 | 原型与 UI-2 复测超长 Project/Run/Worktree 目录、键盘焦点跨窗口化行、screen reader 语义、缩放和国际化长文本。 |
| Run 工作区授权 | `RunWorkspaceShell` 加载 RunContext；无 Worktree route 时读取授权 Worktree 供选择；路由和授权 context 不一致时错误退出。 | 已有客户端授权上下文检查；不等于真实生产身份、完整授权目录或撤权 E2E | UI-1 覆盖直接深链、session 代际变更、撤权、旧响应迟到及错误重试；生产授权和数据库门仍独立开放。 |
| Run tabs | 组件声明 Inbox、Task Cards、Canvas、Workflow / LangGraph、Run BI / Benchmark、Plugin Apps；这些 tabs 只在有效 Worktree focus 下显示。现有代码实际装载 Task Cards 和 Run BI 面板，其他项 disabled/待迁移。 | 部分应用容器与占位 UI | 原型展示所有同级位置，但明确 unavailable；实现按需挂载 active app，未启用 App 不分配隐藏缓存或后台请求。 |
| Worktree focus | 路由含 Project/Run/可选 Worktree；页面无 focus 时要求选择该 Run Worktree；选择后显示本地 Branch 状态。 | 局部实现 | 规范 local branch 为 Worktree 叶状态，验证切换时旧焦点数据先失效；不能让 remote branch 进入父级。 |
| 任务卡 CLI | 页面声明“CLI 从任务卡内打开”，同时明确 Runtime、取消/恢复、独立验证和结果回写未装配；执行准入可被服务端能力位关闭。 | UI 说明存在，CLI 执行闭环未接入 | UI-1 仅原型化入口及禁用/准入状态；实际启用须同时满足 Run/Worktree/Task 绑定、sandbox、Hook、预算、授权再验、取消恢复、独立验证和结果回写。 |
| Chat scope | 页面显示 `WORKTREE/GLOBAL` scope；WORKTREE 仅在已加载有效焦点时可见，但当前聊天输入和发送按钮 disabled。 | scope 控件原型，chat runtime 未装配 | 原型覆盖切换、撤权、focus reset 和草稿隔离；生产聊天需要服务端授权、push/回执和可恢复持久状态。 |
| Canvas、Workflow、Plugins、Inbox | 在同级 tab 集合中有名称，但当前组件没有为它们装载业务面板。 | 占位 | 不由 tab 名称推定功能可用；先定义 Run owner/API/capability，再接入 UI。 |
| 桌面性能路线 | 框架研究建议先完善现有 Tauri UI 与数据边界，再用同一 ViewModel/workload 做 Rust-native 小原型；未决定全量迁移。 | 计划阶段 | UI-1 不作框架选择；UI-2 固定现有路径 baseline 后，UI-3 对比同一数据/交互，再以 UI-4 决策迁移范围。 |

## 5. UI-0 信息架构决策清单

下列事项是原型与交付的冻结输入，不代表当前代码已全部完成：

1. **树的真相唯一。** Project → Engineering Run → Worktree → Branch 是唯一主导航顺序。Project Index 是同实体的汇总视图，远端 Repository/Branch 仅为可选 metadata。
2. **Project 状态不由 Worktree 反向拥有。** 首层切换、权限、Project 成员和 0..N Repository association 归 Project owner；不要把 Project 放进 Worktree 右侧详情里。
3. **App shell 需要一个有效 Worktree focus。** Run 的业务 scope 即使 hostless 也存在；在未选择 Worktree 时可查看 Run 基本信息和 Worktree 选择状态，但 Run App 内容 tabs 不开放。
4. **Run App 同级。** Inbox、Task Cards、Canvas、Workflow / LangGraph、Run BI / Benchmark、Plugin Apps 是同层应用；App 内部可以有局部导航，但不得把这些提升成 Project/Worktree 的兄弟主树。
5. **Worktree 和 Branch 名称准确。** Worktree 表示本地 checkout；Branch 只表示当前本地 checkout ref/state。远端 provider branch 需单独标注来源，不能伪装成本地 checkout。
6. **CLI 入口随任务走。** CLI 必须从 Task Card 打开并继承该卡的 Task/Run/Worktree authorization context；禁止通用任意 shell 成为无约束回退。执行不完整时以明确禁用原因呈现。
7. **Chat scope 显式且短暂。** GLOBAL 与 WORKTREE 是不同授权上下文；Worktree 切换/撤权先清理旧上下文再请求新上下文，不能以界面按钮状态代替授权。
8. **Project Index 不能成为第二套控制面。** 它可以聚合跨 Run 状态、提供过滤和跳转；编辑/迁移/解绑通过各 owner 的版本化 API 与审计事件完成。
9. **设置放在设置。** 全局连接器、插件管理和 Hooks 位于高级设置；Run 内只显示有效配置摘要和深链，不把配置页变成 Run Tree 子节点。
10. **错误状态也服从 scope。** 401/会话代际变化清空旧用户数据；403 保留非敏感的当前页面边界并显示权限提示；找不到对象与无权访问不得泄露对象存在性，服从服务端契约。
11. **框架选择后置。** 先冻结导航、授权和 provider 接口，再用相同数据验证当前 Tauri 路径与 Rust-native 原型；不得以框架 demo/单个控件 benchmark 推断整应用性能。

## 6. UI-1 流程原型与验收标准

### 原型流程

1. 用户打开 Project 入口，选择一个有访问权的 Project；树显示其可访问 Runs 和 Project Worktree Index 入口。
2. 用户展开一个 Run，看见其本地 Worktrees 与各自 Branch 状态；远端关联在详情中以可选来源显示。
3. 用户选择一个 Worktree。原型先验证 scope 状态，再显示此 Run 的同级 App tabs 和当前 Worktree 焦点标识。
4. 用户在 Task Cards 中打开任务。CLI 入口位于任务上下文内；当 Runtime/策略/授权能力未就绪，入口明确禁用并解释缺失条件，不弹出任意 Shell。
5. 用户打开 Canvas，在画布中选择关联任务卡。原型只演示 EntityRef 关系和导航，不复制/模拟第二份任务事实。
6. 用户在底栏切换 WORKTREE 与 GLOBAL；切换说明当前共享上下文变化。切换到另一个 Worktree、撤权或会话过期时，旧的 task/chat/Canvas 内容被立即遮蔽/清理，之后再加载新 scope。
7. 用户通过 Worktree 深链直接进入。授权状态未确认期间无 App 数据可见；若无权、已归档或绑定失效，显示相应安全状态并支持受控返回或重试。
8. 用户观察 Run BI / Benchmark、Inbox、Workflow 和 Plugin Apps 的可用/未接入状态；原型必须区分连接成功事实与 UI 演示态。

### 验收清单

| 维度 | 通过条件 |
|---|---|
| 层级理解 | 不依赖讲解，用户能从 Project 开始找到 Run、Worktree，并理解 Branch 是 Worktree 的本地末端状态；远端关联不改变顺序。 |
| 焦点建立 | Run 无 Worktree 时只出现选择/空状态，不出现 Run Apps；有效选择后才显示同级 tabs 与焦点标记。 |
| 授权安全 | 直接深链重新验证 Project/Run/Worktree；授权失败、撤权、身份切换和过期响应不会闪现或保留旧 scope 数据。 |
| 跨 App | Task Card 与 Canvas 同级；Canvas 的任务引用回到 canonical Task Card；CLI 入口位于任务内部并固定 checkout 焦点。 |
| Chat scope | GLOBAL 和 WORKTREE 清晰可辨；不存在焦点时不能选择 WORKTREE；切换/撤权时取消旧任务并隔离草稿和缓存。 |
| 可访问性 | 树有正确层级角色/展开/选中/焦点语义；键盘可浏览窗口外项目；读屏和高对比度状态可辨；输入法、缩放、长翻译文本不遮挡关键动作。 |
| 低内存 / 并发 | 树和列表只物化当前窗口与有限 overscan；分页和 Canvas viewport 数据按需加载；高频事件走有界队列、合并/背压和取消/drain；inactive app 不保留无限日志或隐藏订阅。 |
| 性能复现 | UI-1 记录场景、数据集定义、设备/系统、窗口尺寸、并发事件速率、活动订阅、采样方式和进程内存/CPU/输入/绘制延迟；UI-2 冻结目标设备及阈值后，方可评定数值是否达标。 |
| 边界清晰 | 页面、tooltip、错误和原型数据均不把未接入 App 表述为生产可用；演示数据如必须用于交互，仅标为原型场景数据并与产品 provider/业务事实分离。 |

无目标 Windows 设备规格及性能预算，本报告不设绝对 RSS、CPU、帧时长、网络延迟或并发用户阈值。UI-2 先确定目标设备、典型/压力 workload 和采样重复方法，再由产品/工程冻结门槛。

## 7. 低内存与并行实现建议

- 左侧目录用 server cursor 分页，并按当前 Project/Run 展开范围增量读取；固定 row key 和 generation，折叠或切换项目时释放子树缓存。
- 目录和 App 数据共享归一化 entity cache，但限制大小/TTL；缓存键至少含 principal generation、Project、Run、Worktree generation 和 resource version。撤权、账户切换、Run 迁移和 Worktree 归档立即失效相关 entry。
- 只挂载 active Run App；inactive tabs 不启动轮询、不保留全量日志、不保持隐式订阅。激活时按 scope 获取数据，失活时 abort 请求并 drain event handlers。
- Canvas 使用 viewport 空间查询与层级细节选择；节点/边只加载并物化可见区域及必要关系，跨画布引用使用 ID 和受权查询，不反序列化整项目历史。
- Agent/CLI 输出保留有界可视 tail 与 durable cursor；完整日志落到受控存储，UI 滚动时按 cursor 拉取。慢消费者触发背压/可恢复提示，不让每个 Agent 无限占用 renderer memory。
- 桌面 Rust 侧采用有界 worker queue、明确并发预算、可取消 job、每 Project/Run 公平调度和关闭时 drain；UI thread 只消费小批状态快照，不执行同步磁盘扫描、Git 操作或长数据库查询。
- Project/Worktree 切换是明确生命周期边界：取消旧查询与工具调用、收敛旧事件订阅、清空敏感 UI store，再启动新授权 generation；晚到结果需被 generation guard 丢弃。

上述是测量前的实现要求，不代表已拥有相应运行时或已达成内存/并发指标。资源预算、缓存上限、队列容量与目标档位由版本化 profile 和 UI-2 实测确定。

## 8. 限制与后续阶段

- 本评审使用已索引 CodeGraph 的定向正向源证据与两份设计计划；当前工具清单未提供 `check_index_coverage`，因此没有声称覆盖所有调用点或所有导航条目。
- 本阶段没有修改产品 UI/服务代码，没有执行测试、编译、打包、启动桌面程序、数据库/授权 E2E、屏幕阅读器检查、RSS/CPU/帧时间采样或 Rust-native 原型。
- UI-1 产出可点击流程原型和以上状态转换；UI-2 以当前实现和真实 provider 建立目标设备性能 baseline；UI-3 使用同一数据合同比较候选 Rust-native UI；UI-4 结合完整依赖许可、可访问性和性能证据决定保留/迁移范围。不得把 UI-0/1 文档完成标记为 W14 完成。

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-26 | WBS-W14 UI-0 review lane | 初版：固定 Project → Engineering Run → Worktree → Branch 信息架构，梳理 App/chat scope、状态合同、现状差距和 UI-1 验收；性能与运行证据保持 NOT_RUN。 | WBS-W14 研究结论及本轮用户确认的 Worktree-first 导航要求 |
