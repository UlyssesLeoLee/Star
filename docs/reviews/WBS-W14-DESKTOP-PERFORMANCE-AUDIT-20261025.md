# WBS-W14 桌面内存与并行验收证据审计

**日期：** 2026-10-25
**基线：** dev `e92a88616adbfa027e55b0d06cf556ef87f37740`（报告编写时的 lane 基线）
**范围：** 只读检查 Star Desktop、Canvas renderer、相关测试设计与 WBS W14；没有改源码、WBS 或设计基线。

## 1. 目的与方法

本报告核对 WBS §2 的 W14：缓存、虚拟化、日志、IPC、Tokio 队列、公平调度、背压、取消/drain、RSS 与 p95 benchmark。W14 依赖 W3、W7、W11、W13、W16；这些依赖或目标设备条件尚未全部闭合，故本报告提供现状证据与下一批可执行切片，不作为性能验收通过声明。

CodeGraph MCP 调用返回 `Transport closed`；本 lane 的 `.codegraph` 目录不是可用索引，`codegraph explore` 报告没有 `.codegraph/` index。因此按仓库指引改用定向源码检索和行号核对。遵照 brief，本次未运行测试、构建、桌面程序或基准。

## 2. 证据矩阵

| 验收面 | 状态 | 证据与结论 |
|---|---|---|
| 桌面前端技术栈 | **缺失：Rust 原生 UI** | 当前桌面是 Tauri API + React 19/TypeScript/Vite：`crates/star-desktop/frontend/package.json:13-29`；README 列出的页面与运行方式见 `crates/star-desktop/README.md:28-39`。这不能描述为 Rust 原生 UI。 |
| Worktree 图的真实数据与布局 | **缺失：生产接线** | `WorktreeVizPage` 用确定性 `makeMockNodes` 造数据：`crates/star-desktop/frontend/src/pages/WorktreeVizPage.tsx:29-43`；布局把节点分组并保存在 `Map`：`:45-70`，组件默认保留 120 个节点并对完整数组计算布局：`:73-86`。该页面是演示路径。 |
| Worktree 图 UI 虚拟化 | **缺失：实际 UI** | 页面以 `nodes.map` 为每个节点创建 SVG group、circle 与 text，没有按 viewport 裁剪：`WorktreeVizPage.tsx:141-173`。Vitest 测试设计明确断言 120 个节点都出现在 DOM：`crates/star-desktop/frontend/tests/WorktreeVizPage.test.tsx:36-47`；本次未运行。 |
| 看板长列表虚拟化 | **缺失** | `KanbanBoard` 在泳道和无泳道模式分别通过 `items.map` 渲染全部卡片：`crates/star-desktop/frontend/src/components/KanbanBoard.tsx:193-205,220-233`。现有实现没有证明存在分页或视口虚拟化。 |
| Rust Canvas renderer 视口过滤 | **部分实现，未证明接入 Desktop** | `canvas-renderer` 提供 200px margin viewport 判定：`crates/canvas-renderer/src/viewport.rs:79-109`，renderer 可据此跳过视口外节点：`crates/canvas-renderer/src/renderer.rs:121-148`。但 Tauri manifest 声明 `canvas-engine` 而未声明 `canvas-renderer`：`crates/star-desktop/src-tauri/Cargo.toml:32-36`；Desktop 图页面也直接用 SVG。现有 Rust 算法不能作为 Desktop 大数据渲染完成的证据。 |
| “1000 节点”虚拟化证据 | **部分：合成计数，不是性能 benchmark** | `simulate_1000_wt` 生成固定坐标并统计视口内节点：`crates/canvas-renderer/src/viewport.rs:122-173`；没有真实 UI 渲染、计时、CPU 或内存采样。注释提出 `<30%` 预期：`:122-125`，同文件测试断言却为 `<60%`：`:188-198`。该目标冲突需在后续验收设计中消解；本报告不选择阈值，也未运行测试。 |
| Task provider fail-closed | **已实现：缺 provider 时禁用** | Tauri 的 `list_work_items` 在 Run-scoped provider 未配置时返回错误：`crates/star-desktop/src-tauri/src/lib.rs:105-109`；前端 hook 调用 provider 并处理返回：`crates/star-desktop/frontend/src/hooks/useTauriWorkItems.ts:21-45`。此项不证明 provider、分页或任务执行闭环已接入。 |
| Worktree / Canvas IPC 与数据上界 | **部分：固定 demo fixture；生产边界缺失** | `list_worktree_groups` 和 `list_canvas_entities` 从 `MockDb` 读取完整 `Vec`：`src-tauri/src/lib.rs:154-185`。README 标注每类当前 4 条 fixture、没有生产 Project/Run/provider 接入：`crates/star-desktop/README.md:59-61,81-86`。有限 fixture 不是 cursor、页大小、响应字节或取消契约。 |
| Tokio 队列、公平调度、背压、取消与 drain | **未证明 / OPEN** | 本次检查的 Desktop 命令与 renderer 路径没有给出业务级有界队列容量、慢消费者背压、公平调度、取消/drain 的运行证据；Tauri async 命令或 renderer 单测不能替代这些验收。具体上限及跨 App 预算仍为 **unknown**。 |
| 缓存、事件与日志驻留上界 | **未证明 / OPEN** | `useMemo` 和布局 `Map` 是页面派生数据结构，不是带容量、淘汰与内存预算的缓存策略：`WorktreeVizPage.tsx:45-86`。本次核对范围未发现可验收的日志保留/裁剪策略或 RSS 采样结果；数值上限为 **unknown**。 |
| Layout WASM / JavaScript fallback | **部分实现** | `useLayoutEngine` 仍以 JavaScript 实现并将 WASM 集成留作后续：`crates/star-desktop/frontend/src/hooks/useLayoutEngine.ts:7-8,57-66`；加载失败会 fallback 至 JS：`:71-98`。这不证明当前页面的布局在 Rust/WASM 中运行。 |
| RSS、CPU、p95、并行容量 | **未知 / NOT_RUN** | 本次未运行 benchmark；所检查的 Desktop 源码与 README 没有目标设备 RSS、CPU、p95 或长时漂移实测。WBS 中 1/2/4/8/16 Agent 只是资源允许时的候选扫描序列，不是容量承诺或 PT-09 基线：`docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:40,63`。README 仍列出桌面运行、bundle 与 Windows build 验证未完成：`crates/star-desktop/README.md:81-86`。 |

## 3. 可复现 benchmark workload/profile 记录模板

每次记录一个不可变 profile/version 与一次运行，不在测量前填写性能阈值。并发扫描按目标设备资源选择；1/2/4/8/16 仅为候选值。

| 字段 | 每次运行需记录 |
|---|---|
| Profile 身份 | profile ID、版本、配置 digest、WBS/代码 commit、执行日期与操作者 |
| 设备与系统 | CPU 型号/核数、物理内存、磁盘、GPU（如适用）、Windows 版本/构建、电源模式、后台负载说明 |
| 软件构建 | Rust toolchain、Tauri 版本、前端 lockfile/build ID、启用的 feature、debug/release、renderer/provider 版本 |
| 数据集 | fixture 或脱敏真实数据来源；Project/Run/Worktree/Task/Canvas 数量；单条 payload 与总 payload 字节；图节点/边分布；日志与事件速率 |
| 并行 workload | Agent/Run/Worktree 数、UI 订阅数、慢消费者比例、突发与稳态写入率、协作 draft/冲突负载、候选并发档位及选择理由 |
| 执行阶段 | 冷启动、空闲、稳定负载、峰值、取消/drain、恢复/重连；每阶段时长、预热时长、重复次数 |
| 采样方法 | RSS/working set 的来源与频率、CPU 采样器、IPC/队列延迟分位数定义、p95 窗口、帧表现采样、日志/缓存水位；计时器与采样开销说明 |
| 结果 | 各阶段 RSS 峰值/中位数、CPU、IPC p50/p95/p99、队列等待 p95、drop/backpressure 数、取消/drain 时间、恢复时间、帧表现、错误/拒绝数、长时漂移 |
| 可信度与限制 | 原始数据路径/摘要、运行是否完整、遗漏指标、机器干扰、数据是否为 fixture、可重复性与结论置信度；未采集字段填 `NOT_RUN` |

**目标阈值：** 在目标设备、可运行桌面路径和 profile 确认前均为 `TBD-MEASURE`。不得用模拟节点数、测试断言或未采样的单次截图替代 benchmark。

## 4. 后续实现切片与验收

1. **W14-A：有界数据与 IPC 契约**（依赖 W3/W7/W11 的真实 provider 与授权边界）。实现 cursor 分页、最大响应记录数/字节、过期 cursor 语义、请求取消；验收真实 provider 路径在上限边缘分页无重复/漏项、超限 fail closed、Worktree 切换与窗口关闭后请求可取消，并记录 payload 与内存水位。
2. **W14-B：桌面 UI 虚拟化与 Rust 路线接线**（依赖 W13 的真实导航/Run 数据接线）。先明确并实现 Rust 原生桌面 UI 目标路径，再让 Worktree、Kanban、Canvas 使用同一 viewport/window 边界；验收视口外元素不进入 UI 树、分页/滚动不重复或漏卡、切换 Run/Worktree 后缓存释放，且大数据路径不是 fixture。
3. **W14-C：目标设备资源与并行 benchmark**（依赖 W14-A/B、W16 durable collaboration、W11 执行资源边界）。按 §3 固定 profile 测量冷启动、稳态、峰值、慢消费者、取消/drain、恢复及长时漂移；在真实样本和多次运行后再冻结 RSS、p95、队列、缓存和并行预算。所有阈值需有目标设备实测和可重放原始数据支持。

## 5. 完成度与限制

- W14 的有界性能指标、目标设备基线、队列/缓存预算、跨 App 并行采样和最终阈值仍 **OPEN**；本审计没有关闭任何运行验收门。
- 已见到的 Rust viewport 逻辑、静态测试、demo fixtures 和前端占位不等同于生产接线或性能实证。
- 没有运行测试、编译、桌面程序、PostgreSQL、benchmark 或负载工具。
- 推荐下一步先确认可运行的目标桌面路径与真实 provider，再推进 W14-A/B；W16 与 W11 的 durable delivery / 资源预约门未通过前，不应宣传可承载的 Agent 并发规模。
