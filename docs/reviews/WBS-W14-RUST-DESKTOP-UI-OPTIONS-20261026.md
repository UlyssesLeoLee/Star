# WBS-W14：Rust 桌面 UI 方向研究

- **日期：** 2026-10-26
- **范围：** 桌面 UI 信息架构、框架候选与性能验证计划
- **状态：** 研究与规划；没有实施框架切换，也没有性能验收

## 1. 结论摘要

当前最紧迫的问题是 Worktree 运维视图的架构和数据边界，而不是单独更换绘制框架。左侧主导航应固定为 **Project → Engineering Run → Worktree → Branch**；选择 Worktree 后，右侧才呈现该 Run 的 App tabs，并把该 Worktree 设为 CLI checkout 焦点。Branch 表示 Worktree 当前的本地 checkout ref/state，不代表远端宿主 Branch。Repository 是 Project 可关联的 0..N 资源；无宿主身份的 Run 仍然必须可以创建、授权、列出和管理。

当前桌面端是 Tauri 2 加 React/TypeScript/Vite；它不是完整 Rust UI。工作树关系图页面有 mock 节点、JavaScript 布局和全量 SVG 节点生成；看板列表也全量映射。Rust 画布 renderer 有视口过滤代码，但桌面 UI 当前没有接上它。当前源码审计没有发现桌面应用层可验收的有界读取、队列背压、取消/drain 或 RSS/帧性能证据。**没有运行测试、构建、实际桌面应用或性能基准。**

因此建议两条路径并行准备：短期先用现有 Tauri 界面落实层级、真实数据边界、惰性 App 加载和列表/画布裁剪，建立可复现性能基线；同时用同一 Rust ViewModel/API 契约制作小范围 Rust-native 原型，对比 **egui/eframe** 与 **Slint**。若更重视可验证的虚拟行和密集 Worktree 画布，egui 应先进入原型；若更重视声明式业务界面和树/列表复用，Slint 应进入第二候选，但先确认产品分发与 GPLv3/商业许可选择。**不在原型前做全应用迁移决定。**

## 2. 当前实现事实与验证边界

| 范围 | 当前代码证据 | 对 UI 规划的含义 |
|---|---|---|
| Desktop 栈 | Tauri 2 shell 与 React 19、TypeScript、Vite：[`package.json`](../../crates/star-desktop/frontend/package.json#L13-L29)、[`README`](../../crates/star-desktop/README.md#L28-L39) | 当前实际渲染路径依赖 WebView；不能把已有桌面端描述成 Rust-native UI。 |
| Worktree 页面 | mock 节点构造、JS Map 布局、useMemo 与完整 SVG `.map`：[`WorktreeVizPage.tsx`](../../crates/star-desktop/frontend/src/pages/WorktreeVizPage.tsx#L29-L86)、同文件 `#L141-L173` | 实际页面还没有视口裁剪；应先移除演示数据路径并接入受权 Worktree provider。 |
| 看板列表 | 各状态列和无泳道视图均对完整 item 列表映射：[`KanbanBoard.tsx`](../../crates/star-desktop/frontend/src/components/KanbanBoard.tsx#L193-L233)；完整工作项过滤：[`BoardView.tsx`](../../crates/star-desktop/frontend/src/components/BoardView.tsx#L56-L68) | 长列表需要分页、虚拟化或分组窗口；仅换框架不能自动解决后端读取和数据驻留。 |
| Rust 画布绘制 | `canvas-renderer` 有 viewport/LOD 代码，但 Tauri manifest 依赖 `canvas-engine`，未依赖 `canvas-renderer`：[`Cargo.toml`](../../crates/star-desktop/src-tauri/Cargo.toml#L32-L36) | Rust 渲染能力没有证明已经接入产品页面。 |
| Tauri 命令和样例库 | Worktree/Canvas 命令返回无 cursor 的 Vec，demo DB 有固定 fixture：[`lib.rs`](../../crates/star-desktop/src-tauri/src/lib.rs#L154-L193) | 演示 fixture 不是生产页大小、持久化 provider 或授权事实。生产界面缺 provider 时应 fail closed。 |
| 布局引擎 | hook 尝试 WASM、失败时回到 JS：[`useLayoutEngine.ts`](../../crates/star-desktop/frontend/src/hooks/useLayoutEngine.ts#L7-L8)、同文件 `#L57-L94`；Worktree 页面另有 JS 布局 | 不能把 Rust/WASM 布局视为当前生产渲染路径。 |

本表只复述本阶段的定向源码审计证据，不声称审计了全部仓库或完整依赖闭包。测试、构建、安装包、启动时长、CPU、RSS、帧时间与并发行为均为 **NOT_RUN**。供应商宣传的库大小或渲染能力不代替 Star 的整应用测量。

## 3. 候选路线

| 路线 | 已核对的官方能力与优势 | 需要 Star 自己证明的部分 | 适配判断 |
|---|---|---|---|
| **Tauri 2 + React（现有基线）** | Tauri 用系统 WebView 承载 HTML/CSS/JavaScript，可通过 message passing 调用 Rust API；前端框架可替换，官方将 Vite SPA 列为推荐配置。Rust commands、events 与 plugin 接口已有官方契约。核心项目许可证为 MIT 或 Apache-2.0。见 [架构](https://v2.tauri.app/concept/architecture/)、[前端配置](https://v2.tauri.app/start/frontend/)、[Rust command](https://v2.tauri.app/develop/calling-rust/)、[Plugin 开发](https://v2.tauri.app/develop/plugins/)、[许可证](https://v2.tauri.app/concept/architecture/#license)。 | 当前页面是 JS UI；列表/关系图虚拟化、异步 IPC、取消、事件去重/背压和卸载时内存释放均需实现。插件 API 本身也不能证明第三方代码已隔离。WebView 复用/小安装体积不等于本产品的低 RSS。 | **近期开发表面，作为基线保留。** 改进导航和数据合同最快；不能满足“桌面 UI 完全用 Rust”的最终验收。 |
| **Slint** | Rust 声明式 UI 与 `.slint` 标记语言；`ListView` 只实例化可见项，官方将其用于长列表；支持后台线程取数，再经 event loop 更新 UI；官方暴露可访问性角色/标签/动作。见 [ListView](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/)、[后台更新模型](https://github.com/slint-ui/slint/blob/master/docs/development/model-repeater-system.md)、[可访问性](https://docs.slint.dev/latest/docs/slint/reference/common/)、[渲染后端](https://github.com/slint-ui/slint/blob/master/docs/astro/src/content/docs/guide/backends-and-renderers/backends_and_renderers.mdx)。 | `ListView` 的可见行优化不能自动解决多层树筛选、WIP 卡片布局或无限画布空间索引；这些需要用真实交互做 POC。官方渲染器资料显示 software renderer 无 GPU、文本仅限 western scripts；Vello renderer 仍为 experimental，Skia disk footprint 较重，因此中文/多语言桌面不能凭“native Rust”就默认选最轻的 renderer。网页端 Slint Wasm 使用 Canvas/WebGL 且不支持屏幕阅读器，不能直接替代 Vercel 的 DOM UI，见 [Web 限制](https://docs.slint.dev/latest/docs/slint/guide/platforms/web/)。许可路径需先确定：官方列有 GPLv3、royalty-free 与 paid commercial 选项；官方页面将不同许可绑定到不同应用分发方式/资格，需依据 Star 发布方式逐项核对，不能仅凭“开源”二字放行，见 [FAQ](https://slint.dev/faqs)、[Pricing](https://slint.dev/pricing) 与 [1.18.1 release](https://github.com/slint-ui/slint/releases/tag/v1.18.1)。 | **结构化业务 UI 的候选。** 如果 Star 采用 GPLv3 兼容分发，GPL 路径可进一步评估；若要专有分发，需先核实 royalty-free/商业路径对组织资格和嵌入场景的限制。不要从官方 FAQ 的小型 runtime 数字推导整应用占用。 |
| **egui/eframe** | Rust immediate-mode UI；`ScrollArea::show_rows` 只构建可见行，`show_viewport` 可只绘制可视区域；原生 AccessKit 接入有官方说明；eframe 提供 wgpu 或 glow 渲染并支持原生与 WebAssembly。见 [ScrollArea](https://docs.rs/egui/latest/egui/containers/scroll_area/struct.ScrollArea.html)、[可访问性](https://github.com/emilk/egui/blob/main/docs/accessibility.md)、[eframe README](https://github.com/emilk/egui/tree/main/crates/eframe)。官方 Cargo 工作区标记 `MIT OR Apache-2.0`、MSRV 1.95，见 [Cargo.toml](https://github.com/emilk/egui/blob/main/Cargo.toml)。 | UI 代码每帧描述当前界面；GUI thread 不能 `.await` 或阻塞，官方建议通过非阻塞 channel/watch/promise 接收任务结果，见 [egui async FAQ](https://github.com/emilk/egui/blob/main/README.md#how-do-i-use-egui-with-async)。应用数据、缓存、纹理和画布索引仍需有界；`show_rows` 是等高行助手，不等于带排序/过滤/多级层次的整套表格或无限画布方案。复杂表单、文本输入法、键盘/屏幕阅读器等需要实测。 | **优先进入 Rust-native 原型。** 可用同一数据协议快速验证 Worktree 树、长列表和图/画布视口；不能从 immediate-mode 或 crate 许可证推断应用性能。 |
| **iced** | Rust 的 Elm 风格 State/Message/Update/View 架构；任务和订阅适合将异步操作建模为消息，`Task` 支持 abortable；内置 Canvas、Scrollable、Lazy 和 wgpu/tiny-skia 渲染选项。见 [README](https://github.com/iced-rs/iced)、[Task API](https://docs.iced.rs/iced/struct.Task.html)、[Lazy](https://docs.iced.rs/iced/widget/struct.Lazy.html)、[widget 清单](https://docs.iced.rs/iced_widget/index.html)。官方 Cargo manifest 为 MIT、MSRV 1.93，见 [Cargo.toml](https://github.com/iced-rs/iced/blob/master/Cargo.toml)。 | `Lazy` 只在依赖变化时避免重建子树，不等于只保留视口可见行；需自行实现树/列表虚拟化与画布空间裁剪。官方仓库的 [accessibility issue #552](https://github.com/iced-rs/iced/issues/552) 仍为 Open，桌面读屏能力必须先通过原型验收。 | **保留为第三候选。** 任务与取消语义有吸引力；在列表可见行和可访问性要求得到验证前，不作为首个 Rust UI 原型。 |

### 不作为证据的性能数字

Slint FAQ 提到其运行时在资源受限嵌入式场景下的内存目标；该数值不覆盖 Star 的桌面壳、主题、图表、Canvas 数据、Agent 输出、Web/本地数据缓存或第三方依赖，本计划不拿它与其它整套框架比较。Tauri 安装包小、egui immediate-mode 或 iced 单项基准同样不能代替实际功能和用户数据下的整应用结果。

## 4. 维护与发布活动（截至 2026-10-26）

| 项目 | 官方时间证据 | 判断边界 |
|---|---|---|
| Tauri | `tauri-v2.12.5` 于 2026-10-21 发布：[release](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.12.5)。 | 有近 12 个月发布证据；版本、依赖闭包、支持平台需在实际接入时复核。 |
| Slint | `v1.18.1` 于 2026-09-21 发布，前一版 `v1.18.0` 于 2026-09-16：[releases](https://github.com/slint-ui/slint/releases)。 | 有近 12 个月连续发布/修复证据；这不代表 Star 的 UI 需求已由组件覆盖。 |
| egui | `0.37.0` 于 2026-10-15 发布：[release](https://github.com/emilk/egui/releases/tag/0.37.0)。 | 有近 12 个月发布证据；项目也明示新版本可能含 breaking changes，需锁定版本并评估升级策略。 |
| iced | 稳定版 `0.14.0` 发布于 2025-12-07，主分支 Cargo workspace 标记 `0.15.0-dev`：[changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md)、[Cargo.toml](https://github.com/iced-rs/iced/blob/master/Cargo.toml)。官方仓库仍有 2026 年 issue/PR 活动：[Issues](https://github.com/iced-rs/iced/issues)。 | 最近稳定版仍在 12 个月窗口内，但稳定发布节奏比另三项慢；不能仅因维护活动存在就假设 release cadence、兼容性或企业支持符合 Star 需要。 |

这是一项 bounded check：确认了官方 release/changelog 与少量当前仓库 activity 页面，没有估算完整社区人数、issue 首次响应时间、公司支持持续性或依赖供应链风险。

## 5. 面向 Star 的 UI 与运行时设计

1. **单一导航投影。** Project 选择器为左侧第一层；其下展开展示 Runs，再展 Worktrees，最后显示每个本地 Worktree 的 checkout Branch/ref/state。多 Repository 关联留在 Project 配置；远端 provider 仅为可选关联元数据。全局 Project Worktree Index 可以聚合状态，但不替代 Project → Run 主导航。
2. **选中后再装载。** 展开 Project/Run/Worktree 时分别读相应范围；选中 Worktree 才初始化该 Run 的 Inbox、Task Card、Canvas、Board、Workflow、BI/Benchmark、Plugin Apps tabs。切换 Worktree 时取消旧请求/订阅、释放临时渲染资源，并让 CLI 接受由所选 Worktree 明确绑定的工作目录和 RunContext。
3. **数据 owner 与 UI 分离。** Rust API/应用层持有 Project membership、Run scope、Worktree identity、授权和事件契约；UI 只消费版本化分页投影与 event stream。前端不能用本地 seed/mock 代替真实 Run/Worktree。provider/授权/持久化缺失就显示明确不可用状态，不创建假的任务或 checkout。
4. **事件推动状态，不靠高频全量拉取。** 协作更新从持久化 owner Outbox/relay 到客户端事件流；客户端用 cursor 断线补齐，按实体 id/revision 增量更新。冲突时保留自己的草稿，并显示可读取的对端修改；桌面 renderer 不拥有业务事实。连接中断和权限撤销要让旧页面/缓存失效。
5. **在 UI 边界做资源上限。** Worktree 树、Kanban、Inbox、事件列表按需读取、分页和视口虚拟化；无限 Canvas 使用空间索引、viewport culling、LOD、滚动窗口和增量布局。终端输出/日志写入有界内存 ring buffer，较旧输出留在本地持久或按 cursor 读取；大量事件合并刷新频率，但不可丢掉需要审计的持久事实。
6. **并行和取消。** 只在当前激活的 Run/Worktree 建立 UI subscriptions；每个数据流设置容量、背压、重连和取消/drain 契约。工作线程完成后以稳定 revision 投影回主线程；过期的 Worktree/Run 结果直接丢弃，不能覆盖新选择状态。公平调度和执行预算仍由 Star 服务端/Runtime owner 控制。
7. **隔离插件呈现。** UI 框架不应允许 Plugin 通过 arbitrary native code 进入桌面主进程；插件视图走版本化能力清单、受限 IPC 和宿主渲染组件。动态 App tab 只是授权后可见的 UI，不应获得隐藏的文件、shell 或 Worktree 权限。
8. **分离宿主基础设施视图。** Multipass VM、K3s cluster/node 和 PostgreSQL 是不同资源身份；它们以 Infrastructure/Run 状态与容量信息呈现，不插入 Project → Run → Worktree → Branch 树中，也不把虚拟机数量等同 Worktree 数量。

## 6. 建议的分阶段 UI 工作

| 阶段 | 交付物 | 验收门 |
|---|---|---|
| **UI-0：信息架构冻结** | 左侧导航、选中 Worktree 后的 tab 层级、全局/Worktree chat scope、Project 的 0..N Repository 配置、Branch 标签语义与失败/空状态图 | 全部现有导航项逐一映射到 Project/Run/Worktree/Branch 或右侧 App tab；远端 host 缺席也不影响 Run；Canvas 与任务卡为同级 App；不出现假任务/seed 运营数据。 |
| **UI-1：操作流程原型** | 用真实权限范围画出“创建 Run、分配 Worktree 给 Agent、查看 checkout/冲突/资源、打开任务卡 CLI、检查 Canvas/BI”的流程；再做窄屏/长列表布局原型 | 操作者能从 Project 找到被占用/失联/冲突的 Worktree；CLI 与 Worktree/Task/Run 绑定可见；切换 Worktree 后不会看到前一 Worktree 的内容。仅用静态线框作视觉讨论，不把 mock 当可用业务数据。 |
| **UI-2：当前路径性能基线** | 接入真实有权限的数据源；先量测现有 Tauri/React 应用冷启动、空闲/加载/RSS、输入响应、长列表交互、切换/取消和多 Run 窗口 | 数据 API 有 page/cursor/最大字节与取消；长列表创建的 UI 对象受可视区控制；切换时旧请求无法更新新 Worktree；CPU/RSS/帧响应有可重复脚本和机器/配置记录。先做基线，再由产品确定预算，不提前编数字。 |
| **UI-3：Rust-native 对照原型** | 用同一只读 ViewModel/data provider 实现相同的 Project→Run→Worktree 树、状态表、一个 Kanban 列表、一个带视口裁剪的 Canvas 节点图。第一轮对比 egui/eframe；第二候选 Slint 必须先满足许可门。 | UI 事件模型/分页/取消、节点数量、viewport bounds、图标/字体和数据完全一致；检查 target Windows 下输入法、可访问性、键盘导航、多窗口、挂起/恢复、安装尺寸、idle/loaded RSS、CPU、p95 render/input delay；所有指标均写环境和复现方法。对比既有 Tauri 优化基线。 |
| **UI-4：迁移或维持决定** | 基于 UI-2/UI-3 结果、许可清单、团队能力和跨平台支持，作出保留 Tauri、全量迁移或分层混合的决策 | 未达性能阈值、授权/可访问性/取消链不全或许可未核验时，不迁移业务功能；门禁需有可审阅的差异和回滚方案。Vercel 浏览器端保留独立 Web 渲染策略，Rust-native Desktop 不能以失去 DOM/Web accessibility 为代价。 |

### 原型 workload 建议

使用一组固定可复现数据集，包含多个 Project、Runs、Worktrees、Agent 状态、长日志、任务卡与 Canvas 节点；分别记录小、中、大档的数量、内容和硬件。数据集只用于专门的性能评估，不能出现在产品 seed 或作为真实任务；业务闭环用授权 provider。建议至少覆盖静止闲置、快速切换 Worktree、Run 内 tab 多开、Canvas 缩放/拖动、事件涌入、断线重连和长时间运行。每次都记录冷/热启动、进程数、idle/peak/settled RSS、CPU、帧时间与事件到屏延时 p50/p95、队列峰值、被物化 UI item 数、取消/drain 时间、GPU/软件渲染路径和系统版本。

当前没有目标设备规格和产品预算，本阶段不写绝对 MB/ms 阈值。先由 UI-2 固定 Windows 目标机上的 baseline 与 workload，再由产品与工程共同设下限/上限；不能用“感觉流畅”或单个库的 demo 代替验收。

## 7. 下一步决策与风险

- **现在可推进：** UI-0 与 UI-1；并在已授权真实 provider 可用后做 UI-2。左侧 Project→Run→Worktree→Branch 架构不依赖 UI 框架，因此可以先冻结。
- **Rust-native 候选排序：** egui/eframe 先做小原型，重点验 Worktree 长树、表格、画布裁剪、字体/输入法和 AccessKit；Slint 做对照，重点验树模型、可见行复用、复杂表单和运行时授权；iced 在可访问性和长列表虚拟化有实证后再进入对照。
- **许可门：** egui MIT/Apache-2.0；iced MIT；Slint 有 GPLv3 与其他授权选项。不能把 GPLv3 误判为禁止商业使用，但要确认 Star app 的源码/二进制分发方式与 GPL 兼容性；若拟用 Slint 的非 GPL 路径，先以具体套餐、公司资格、功能版本、分发地/用户、设备类型核验是否满足项目“不依赖付费解锁核心功能”的约束。正式依赖前还要检查版本固定的传递依赖、系统 WebView、字体、安装器、NOTICE 与 SPDX/SBOM。
- **迁移风险：** 直接替换全套前端会把导航、授权、协作事件、画布和插件问题一起混到框架迁移中；先固定 API 与 Rust ViewModel，再逐页迁移可降低回归范围。
- **人机可用性风险：** 输入法、多语言/复杂脚本、屏幕阅读器、键盘快捷键、缩放与拖放要在 Windows 上实测；原生自绘不等于自动有完整可访问性。

## 8. 证据来源

上述框架事实与 release 日期均来自截至 2026-10-26 核对的官方项目文档、仓库、release/changelog。主要来源如下：

- Tauri：[架构](https://v2.tauri.app/concept/architecture/)、[前端配置](https://v2.tauri.app/start/frontend/)、[Rust command](https://v2.tauri.app/develop/calling-rust/)、[plugin 开发](https://v2.tauri.app/develop/plugins/)、[v2.12.5 release](https://github.com/tauri-apps/tauri/releases/tag/tauri-v2.12.5)。
- Slint：[ListView](https://docs.slint.dev/latest/docs/slint/reference/std-widgets/views/listview/)、[accessibility 属性](https://docs.slint.dev/latest/docs/slint/reference/common/)、[renderers](https://github.com/slint-ui/slint/blob/master/docs/astro/src/content/docs/guide/backends-and-renderers/backends_and_renderers.mdx)、[Web 限制](https://docs.slint.dev/latest/docs/slint/guide/platforms/web/)、[licensing FAQ](https://slint.dev/faqs)、[pricing/licence options](https://slint.dev/pricing)、[1.18.1 changelog](https://github.com/slint-ui/slint/blob/master/CHANGELOG.md)、[releases](https://github.com/slint-ui/slint/releases)。
- egui：[README 与 async 限制](https://github.com/emilk/egui/blob/main/README.md)、[virtual row/viewport API](https://docs.rs/egui/latest/egui/containers/scroll_area/struct.ScrollArea.html)、[AccessKit](https://github.com/emilk/egui/blob/main/docs/accessibility.md)、[MIT/Apache 与 Rust MSRV](https://github.com/emilk/egui/blob/main/Cargo.toml)、[0.37.0 release](https://github.com/emilk/egui/releases/tag/0.37.0)。
- iced：[README](https://github.com/iced-rs/iced)、[2025-12-07 changelog](https://github.com/iced-rs/iced/blob/master/CHANGELOG.md)、[async Task/取消](https://docs.iced.rs/iced/struct.Task.html)、[Lazy](https://docs.iced.rs/iced/widget/struct.Lazy.html)、[widget 清单](https://docs.iced.rs/iced_widget/index.html)、[MIT/MSRV](https://github.com/iced-rs/iced/blob/master/Cargo.toml)、[accessibility issue #552](https://github.com/iced-rs/iced/issues/552)。

**限制：** 框架文档说明可用 API 和上游自述；不证明 Star 集成后功能完整、RSS 低、UI 一定流畅或可商业交付。Issue/PR 页面只作为活跃状态证据，不等同于社区质量或维护承诺。性能、许可分发闭包、目标主机、国际化/可访问性和完整依赖许可均需后续阶段实证。\n