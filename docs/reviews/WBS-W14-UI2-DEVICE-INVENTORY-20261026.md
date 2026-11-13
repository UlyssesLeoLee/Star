# WBS-W14 UI-2 本机设备候选清单

**采集日期：** 2026-10-26 10:50（本机时区 UTC−07:00）
**采集范围：** 只读 Windows 主机能力快照
**状态：** 候选清单；目标设备未批准；性能测量 `NOT_RUN`

## 0. 结论与边界

本次采集补充 [UI-2 目标设备基线协议与证据审查](WBS-W14-UI2-DEVICE-BASELINE-20261026.md) 中尚未采集的本机字段。它只能说明本次查询可见的主机信息，**不能**将本机指定为 UI-2 目标设备，也不批准 workload、数据清单、采样 profile、资源预算或 SLA。

采集只读 WMI、少量与显示比例/WebView2/已安装应用相关的注册表元数据及磁盘容量信息；没有读取环境变量、凭据、序列号、卷 ID、账户标识或完整个人路径。未启动 Star，未运行 benchmark、build 或 test，未安装软件或修改配置。

## 1. 可复核观察

| 项目 | 观察结果 | 证据来源与限制 |
|---|---|---|
| CPU | 13th Gen Intel(R) Core(TM) i7-13700F；16 个物理核心、24 个逻辑处理器 | `Win32_Processor` 的 Name、NumberOfCores、NumberOfLogicalProcessors |
| 已安装内存 | 34,104,651,776 bytes，约 31.76 GiB | `Win32_ComputerSystem.TotalPhysicalMemory`；不是 Star 可用内存或运行时占用 |
| GPU | WMI 返回 NVIDIA GeForce RTX 3060；另有两个虚拟显示适配器条目 | `Win32_VideoController`；未采集 PNP 标识或驱动版本 |
| GPU 显存字段 | NVIDIA 条目的 `AdapterRAM` 为 4,293,918,720 bytes，约 4.00 GiB | 这是 WMI 报告值，不等同于已核实的专用 VRAM 容量；**专用 VRAM：UNKNOWN** |
| Windows | Microsoft Windows 11 专业版；版本 10.0.26300；build 26300 | `Win32_OperatingSystem` 的 Caption、Version、BuildNumber |
| 显示模式 | NVIDIA 适配器报告 1920×1080、60 Hz | `Win32_VideoController` 的当前模式字段；不证明所有显示器或 Star 窗口的实际呈现模式 |
| 显示缩放 | **UNKNOWN** | WMI 通用显示条目不提供有效分辨率；注册表中有 per-monitor DPI 原始值，但本次未将其编码解释为比例 |
| 当前测量 Worktree 磁盘余量 | C: 可用 64,806,879,232 bytes，约 60.36 GiB | 只记录盘符和容量，不记录卷标、卷 ID 或路径 |
| 主项目目录磁盘余量 | D: 可用 101,922,725,888 bytes，约 94.92 GiB | 同上；容量是采集时快照，不代表后续 workload 的可用预算 |
| WebView2 Runtime | 155.0.4283.62 | EdgeUpdate 的 WebView2 Runtime 产品元数据；未启动 Star，不能证明 Star 实际选择或加载的 Runtime |
| 当前 Star 桌面构建 | **UNKNOWN** | 查询标准 Windows 卸载注册表项和当前用户 AppX 清单，未发现可明确识别的 Star 桌面版本；此查询不是对所有安装位置的穷尽证明 |

## 2. 观察与推断

### 观察

- 当前隔离测量 Worktree 位于 C: 卷；主项目目录位于 D: 卷。本表仅列盘符与自由空间。
- CPU、内存、GPU/显示模式、OS 和 WebView2 版本来自本机只读查询。
- 设备信息未提供 Star 进程树、实际 build、生产数据、后台负载或性能测量样本。

### 推断与限制

- 该主机可作为 UI-2 的**候选设备**供产品/工程 owner 审阅；本清单不批准其为最低配置、代表性设备或正式目标。
- WMI 的 `AdapterRAM` 不足以确认该 GPU 的专用显存；需要在获批设备上通过可信只读来源复核。
- 单次磁盘自由空间和已安装的 WebView2 Runtime 版本不构成资源预算、Star 运行时选择或性能结论。
- 本快照不定义数据规模、并发量、场景、采样频率、重复次数、指标阈值、profile owner 或 SLA；这些仍需在实测前独立冻结。

## 3. 未读取或仍未知

- 专用 GPU VRAM 的准确容量、GPU 驱动版本。
- 当前 Star 桌面安装包/build ID、实际进程分组、Star 所用 WebView2 版本。
- 显示缩放比例；完整多显示器分辨率与刷新率。
- 电源计划、采集时后台负载、CPU/GPU 温度与功耗。
- 真实授权数据 provider、数据 manifest、并发 workload、采样 profile、资源预算与 SLA。
- 已批准的 UI-2 目标设备及其批准人。

以上项目保持 `UNKNOWN` 或未批准状态；不得从硬件型号、当前磁盘空间或协议建议参数推导补齐。

## 4. 复核方法

本次只读查询覆盖：

- `Win32_OperatingSystem`、`Win32_Processor`、`Win32_ComputerSystem`；
- `Win32_VideoController`、`Win32_DesktopMonitor`、逻辑磁盘容量；
- EdgeUpdate 下 WebView2 Runtime 的产品名/版本元数据；
- 当前用户 DPI 相关键的有限数值元数据，以及标准卸载应用项和 AppX 名称/版本。

显示缩放注册表原始值没有被解释为用户比例；应用清单查询没有输出安装位置。没有查询环境变量。

## 5. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-26 | WBS-W14 UI-2 device inventory lane | 添加经脱敏的本机候选设备快照；观察与推断分开；未知项维持 UNKNOWN；未更改验收阈值或宣称性能通过。 | WBS-W14 UI-2 只读设备清单任务 |
