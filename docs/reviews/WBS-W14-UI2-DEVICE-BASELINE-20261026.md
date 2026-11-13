# WBS-W14 UI-2 目标设备基线协议与证据审查

**日期：** 2026-10-26
**审查基线：** `codex/wbs-w14-ui2-device-baseline-20261026`，`dev@e5a29193af3e75bcd4a34c2e45e2a73fafd3cf56`
**状态：** 协议建议已整理；目标设备与 workload 未冻结；基准测量 `NOT_RUN`。

## 0. 结论与边界

WBS 已把 UI-2 定义为：冻结目标 Windows 设备、数据规模、并发 workload、RSS/CPU/响应指标及采样方法，并测量当前 Tauri+React 桌面路径。现有文档没有给出获批设备、真实产品数据规模、并发容量、测量阈值或运行样本。因此本报告只提供可审阅的协议和证据清单，不代表 UI-2 性能验收完成，也不关闭 W14。

本 lane 只读取 WBS 与既有 W14 文档；没有探索产品源码结构，没有调用可用的 CodeGraph 索引，也没有运行构建、测试、benchmark 或桌面应用。无法从现有文档证明的进程归属和采集能力均标为未知。

## 1. 需求到证据追踪

| UI-2 要求 | 来源 | 已有证据 | 尚缺内容 |
|---|---|---|---|
| 固定目标 Windows 设备、系统与构建身份 | `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` §2 W14；`docs/reviews/WBS-W14-DESKTOP-PERFORMANCE-AUDIT-20261025.md` §3 | WBS 要求目标 Windows 实测；既有审计模板列出 CPU/核数、内存、磁盘、GPU、电源模式、Windows build、Rust/Tauri/frontend build、feature 和 debug/release | 获批的最低/代表性目标设备、GPU/磁盘/显示/电源信息、应用 build 与工具版本尚未冻结或实测 |
| 固定数据集及数据规模 | WBS §2 W14；既有桌面性能审计 §3；`docs/reviews/WBS-W14-RUST-DESKTOP-UI-OPTIONS-20261026.md` §6 “原型 workload 建议” | 文档要求记录 Project/Run/Worktree/Task/Canvas 数量、payload 大小、节点/边、日志与事件速率，并使用相同数据比较路径 | 真实授权 provider、数据清单、各档数量/字节、日志/事件速率及去标识方式尚未确定 |
| 固定并行 workload 与容量扫描 | WBS §1 D-10、§2 W14；既有桌面性能审计 §3–4 | WBS 把 1/2/4/8/16 Agent 明确限定为资源允许时的候选扫描值，非容量承诺或 PT-09 基线；审计要求记录 Agent/Run/Worktree、订阅、慢消费者、突发/稳态负载、draft/conflict | 实际扫描档位、其他并发资源数、事件速率、慢消费者比例、目标 profile 和预算均未冻结 |
| 空闲、启动、交互、峰值、取消/恢复和长时场景 | WBS §2 W14、§16 第 4 项；既有桌面性能审计 §3–4；UI 选项审查 §6 | 文档列出冷/热启动、空闲、稳定/峰值、切换 Worktree、Run tabs、Canvas 平移缩放、事件涌入、断线恢复、取消/drain、长时漂移 | 场景顺序、持续时间、重复次数、输入脚本及“启动完成/操作完成”标记定义尚未冻结 |
| RSS/CPU/响应、采样窗口与统计口径 | WBS §1 D-10、§2 W14、§16 验收表；既有桌面性能审计 §3 | 要求记录 RSS/working set、CPU、IPC/队列延迟分位数、p95 窗口、帧表现、缓存/日志水位、峰值/中位数与长时漂移；阈值为 `TBD-MEASURE` | 采集器与版本、频率、延迟边界、预算、采样完整性判据和原始文件尚未确定 |
| 不把原型、静态代码或 mock 当性能证明 | WBS §16–17；`docs/reviews/WBS-W14-UI1-FLOW-REVIEW-20261026.md` §1、§4–7 | UI-1 是无 API/持久化/Shell 的静态原型；没有生产数据或桌面运行证据；RSS/CPU/帧时间和并行负载均为 `NOT_RUN` | UI-2 必须锁定实际桌面 build 和授权数据路径；若只能跑原型或 mock，结果须单独标为原型测量，不能作为产品基线 |

## 2. 已知、可采集与待决事实

### 2.1 仓库文档已知

- WBS §2 的 W14 当前为 OPEN：UI-0 已集成，UI-1 静态原型已交付且独立复核待补；UI-2 仍是下一步的目标设备基线，之后才是 UI-3 同数据对照和 UI-4 迁移/维持决策。
- WBS §2 将 Tauri+React 保留为当前可测基线；候选并行度 1/2/4/8/16 只是资源允许时的扫描序列。
- WBS §16 第 4 项要求保留原始采样和复现说明。§16 验收表及 §17.3 明确此前没有桌面运行、设备基线、RSS、CPU、帧时间或并发容量结论。
- UI 选项审查 §3 描述 Tauri 使用系统 WebView，并可通过 message passing 调用 Rust API；这只能说明架构路径，不能证明该 build 的实际 PID 树或各子进程的资源归属。
- UI 选项审查 §6 的负载建议涵盖多个 Project/Run/Worktree、Agent 状态、长日志、任务卡和 Canvas 节点；这些是场景建议，不是已确认的生产数据规模。

### 2.2 当前工作站候选（不是已确认的目标设备）

协调者于 2026-10-26 提供了本机只读设备快照：Dell Inspiron 3020、Intel Core i7-13700F、物理内存 `34,104,651,776` bytes（约 `31.76 GiB`）、Windows 11 Pro `10.0.26300` / build `26300`。该快照只用于标识一个可复核候选，不代表用户确认的最低配置、目标部署机或产品性能档位，也不是本 lane 独立复采的基准证据。

快照未包含 GPU/显存、磁盘型号与可用空间、显示器分辨率/缩放/刷新率、电源计划、当前后台负载、WebView2 版本、Star build/依赖版本或运行 PID 树。后续采集须记录这些实际值和采集时间；缺失字段填 `UNKNOWN`，不能从型号推导或猜测。

### 2.3 必须在实测前决定

1. 目标设备选择：本机候选、最低支持设备或另一个代表性 Windows 设备；如需多设备，各自建立独立 profile。
2. 真实数据与各 workload 档：由经授权、去除敏感信息的 provider 事实确定实体数、payload 字节、图节点/边、日志长度、事件速率与并发数；没有数据前不填造规模。
3. Agent/Run/Worktree 的候选扫描档、是否包括多窗口及慢消费者比例；WBS 的候选序列不自动构成要全部执行的要求。
4. 资源与交互门槛：绝对/增量内存、CPU、启动、输入到呈现、帧时间、取消/drain、恢复、长期漂移预算及批准人。当前一律 `TBD-MEASURE`。
5. 采样工具/版本、阶段时长、重复次数、环境隔离方式和最终基线 profile owner。

## 3. 测量边界与进程归属

本报告没有源码结构证据来确认 Tauri 主进程、WebView 子进程、sidecar、Agent Runtime 或任何外部执行器在此 build 中如何启动、嵌套或归属 Worktree。CodeGraph 索引不可用，且本 lane brief 禁止源码结构探索。因此当前结论是：**桌面进程分组 `UNKNOWN`；sidecar/Agent 是否属于 UI 测量进程组 `UNKNOWN`。** 不得把没有识别到的进程记为零，也不得仅凭进程名称推断 owner。

实测时按 PID 建立时间戳化清单：记录进程名、PID、父 PID、可执行映像身份、出现/退出时间，并把映射证据和 Run/Worktree 执行关联起来；不采集可能含令牌的完整命令行。分别呈现桌面主进程、可确认的 WebView 进程、可确认的 sidecar/Agent 进程及未分类进程。无法证明归属时单列 `UNATTRIBUTED`，同时报告分组内可归属数据和整机总量。对共享/多实例 WebView 记录 PID 去重规则，避免重复相加。

Windows 的 Working Set 是驻留内存观测值，不应无条件称为 Linux RSS。每次报告至少保留进程级 Working Set 和 Private Bytes/commit 指标；若采集器支持 Private Working Set，则单独列出采集器字段名。汇总值须说明是否包含共享页和是否按 PID 去重。CPU 需记录累计 CPU time 及归一化公式；建议报告“进程组 CPU 秒 / 墙钟秒 / 逻辑处理器数”对应的全机百分比，同时保留原始值。

## 4. 可复现测量协议（建议冻结后执行）

以下数值是**测量协议建议参数**，用于让后续结果可复现；不是 Star 的性能预算、容量承诺或已批准验收阈值。工程 owner 可在 UI-2 启动前修订并将最终值写进不可变 profile。

### 4.1 冻结 profile 和数据

为一次完整基线生成 `profile_id`、版本、配置 digest 和唯一 `run_id`。记录 WBS/代码 commit、安装包/build ID、Rust toolchain、Tauri/WebView2、frontend lock/build、release/debug、启用 feature、采集器及版本、设备/系统快照、电源计划和环境负载。使用真实授权 provider；数据 manifest 列出各实体数量、单条/总 payload 字节、节点/边分布、日志和事件速率。去标识数据或固定生成数据必须保存 seed/digest，并标明它不是生产事实。UI-2 与以后 UI-3 使用同一 manifest、事件脚本、窗口尺寸、字体和交互定义。

小/典型/高负载档由产品代表数据和已确认目标 workload 选出；在批准前只记录档名为 `TBD`，不填假定数量。Agent 并发仅从 WBS 候选序列中按设备资源选择，并写明选择理由；不将候选序列宣传为产品支持规模。

### 4.2 阶段、重复和采样（建议值）

| 阶段 | 建议执行方式 | 采集重点 |
|---|---|---|
| 环境与空闲基线 | 固定设备、电源和 build；关闭前次 app 及其已确认子进程；等待 60 秒稳定后连续采样 300 秒；每 profile 做 5 次独立重复 | 进程清单、CPU、Working Set/Private Bytes、整机负载；记录运行前后台干扰 |
| 冷/热启动 | “冷”定义为 Star 进程组已退出后的进程冷启动，不承诺清空 OS 文件缓存；先固定一个预热启动，再分别记录 10 次进程冷启动和 10 次热启动 | 首个可交互时刻、各 PID 出现时间、初始空闲内存；启动分布以原始各次时长和中位数呈现，不用小样本伪装 p95 |
| 稳态交互 | 固定导航树、任务列表、Canvas 等数据 manifest；每个动作类别至少记录 100 个事件，并在同一 profile 重复 5 次 | Worktree 选择/切换、同级 App tab、长列表滚动、Canvas 缩放/拖动、聊天输入；每个动作的输入到可见呈现延迟与错误 |
| 并行/峰值 | 仅运行已冻结且设备资源允许的并发档；先稳定 60 秒，再测量 300 秒；包含事件突发、慢消费者和 UI 交互并发 | Agent/Run/Worktree/订阅实际数、队列与缓存水位、CPU/内存、事件到屏、丢弃/背压/错误数 |
| 取消、恢复与长时 | 按固定脚本触发取消/drain、断线重连与恢复；长时段长度由 profile owner 预先冻结 | 发出取消到完成 drain、恢复用时、重连后的内容正确性、阶段内峰值及结束相对空闲基线的漂移 |

建议以 1 秒间隔采集进程资源 counters；交互与事件到屏需逐事件打点，不以 1 秒资源采样推算 UI p95。帧/呈现采集器需在 profile 中指定名称、版本、配置和 overhead 校验方法。上述频率、时长、重复次数是建议的测量方法，开始前必须冻结；若目标设备负载不足以运行候选并发档，应记为未执行及原因，不自动降档后宣称通过。

### 4.3 统计、空闲扣除与无效运行判定

- 每个 `run_id` 保留原始时间序列；按阶段报告绝对内存中位数、峰值、CPU 中位数/峰值、各动作 p50/p95/p99、启动逐次结果、队列/缓存水位、丢弃/背压、取消/drain、恢复、错误数及长时漂移。
- 同一设备、build、profile 下，以该次运行前空闲样本的中位数作为配对 idle baseline；报告每个阶段的绝对值及“阶段统计值减 idle 中位数”的增量。内存增量不能替代绝对占用或峰值。多次运行先分别计算各自统计，再报告跨运行中位数与范围；保留原始分布，不只报一个总平均。
- CPU 同时保留 process CPU time 与采样器换算的 host-normalized percent。输入响应从用户动作事件的时间戳起算，结束点是对应界面更新首次呈现在屏幕的时间戳；若无法获得这两个端点或帧采样缺失，该指标记 `NOT_RUN`，不得以 API 返回时间冒充可见响应。
- build/profile/设备不一致、授权 provider 不可用、落入 mock/静态原型路径、数据 manifest 不一致、场景未完成、关键采样缺失、进程归属无法解释、系统遭明显干扰或原始文件缺失时，该次运行标为无效/不完整并说明原因；不能填补或合并成通过证据。
- 产品通过/失败阈值暂为 `TBD-MEASURE`。在产品/工程冻结预算前，只能比较数据与描述测量结果，不作 SLA、容量或“低内存已达标”结论。

### 4.4 原始证据和复现包

每个 profile 保存只读目录：`metadata.json`（设备、系统、构建、profile、数据 digest、工具版本）、`scenario.json`（阶段、次数、脚本/事件序列版本）、`processes.csv`（时间、PID、父 PID、映像身份、分类）、`resources.csv`（时间、PID、Working Set、Private Bytes、CPU time、队列/缓存水位）、`actions.csv`（动作名、起止时间、结果）、`run-summary.json`（分位数、峰值、遗漏字段和有效性）及采集器日志。所有文件以 `run_id` 关联并生成摘要；使用脱敏数据，不归档凭证、完整命令行或工作区私密内容。

设备身份可用只读 PowerShell 查询复核并将脱敏后的输出附在 metadata 中：

```powershell
Get-CimInstance Win32_ComputerSystem | Select-Object Manufacturer, Model, TotalPhysicalMemory
Get-CimInstance Win32_Processor | Select-Object Name, NumberOfCores, NumberOfLogicalProcessors
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber
Get-CimInstance Win32_VideoController | Select-Object Name, AdapterRAM, DriverVersion
```

同时记录磁盘型号/可用空间、显示分辨率/缩放/刷新率、电源计划、后台负载、WebView2 与实际 Star build；没有可靠采集值时填 `UNKNOWN`。复现者只需使用相同设备/profile、build、manifest 和脚本执行上述阶段，原始数据与 summary 一起归档；不得先运行 UI-3 后再回填 UI-2 的基线。

## 5. 状态、限制与下一步

- **NOT_RUN：** 本 lane 未运行构建、测试、benchmark、桌面应用或任何性能采样；未证明当前工作站就是目标部署设备；未验证 PID 树、真实 provider 或数据 workload。
- **当前不能通过：** 设备候选不等同获批设备；UI-1 静态原型/模拟数据不能替代生产路径；文档和采样协议不构成 RSS、CPU、延迟、并发、可靠性或 W14 完成证据。
- **解除实测前最小步骤：** 由产品/工程 owner 确认目标 Windows 设备和目标 workload；提供可运行的当前 Tauri+React build 与真实授权 provider；冻结去标识数据 manifest、候选并发档、上述测量参数和预算责任人；在目标机保存一次 pilot 的 metadata/原始样本，检查进程映射及采集 overhead，随后按冻结 profile 重复完整场景。
- 完成 UI-2 后，先审阅原始证据和遗漏字段，再更新 WBS 状态；只有 UI-2 baseline 可重放后才推进 UI-3 同数据对比和 UI-4 迁移/维持决定。W14 的 W3/W7/W11/W13/W16 依赖、取消/背压与资源服务等级仍须分别验收。

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-26 | WBS-W14 UI-2 evidence review lane | 汇总 UI-2 的文档证据、当前工作站候选、未知项、进程归属限制、可复现测量协议与 NOT_RUN 门；没有改变性能阈值或声称实测通过。 | WBS-W14 UI-2 设备基线审查任务 |
