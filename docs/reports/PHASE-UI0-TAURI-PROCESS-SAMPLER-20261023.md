# PHASE-UI0-TAURI-PROCESS-SAMPLER-20261023

| 项目 | 内容 |
|---|---|
| 版本 / 状态 | v0.1.0 / 采样器与聚焦验证已完成；真实 Tauri、Rust-native 与性能决策仍 OPEN |
| 日期 | 2026-10-23 |
| 分支 / 基线 | `codex/ui0-tauri-process-sampler` / brief 基线 `f70c8b8d`，lane 来源为本地 `dev@194f7b89` |
| 范围 | 只读 Windows `Win32_Process` 进程树 RSS/CPU 采样、进程计数、聚焦测试、UI WBS 与本报告；不修改现有 runner、生成器、inventory 或 Web evidence |

## §0 目的

为已有 UI-0 JSONL runner 补充真实 Windows 进程数据采集入口，允许操作者显式指定 Tauri 根 PID、场景、cold/warm 条件、采样间隔、时长和新输出文件。该切片只补采样基础设施；不会据此选择 renderer、设定发布阈值或声称 UI 性能验收完成。

## §1 改动矩阵

| 路径 | 改动 | 状态 |
|---|---|---|
| `scripts/ui0_bench/capture_tauri_process.py` | 固定 PowerShell 查询本机 `Win32_Process` 的 PID、父 PID、创建时间、working set 与累计 kernel/user CPU 时间；沿父子关系选树，拒绝缺失/超限数据；输出受行数、行长和总字节限制的 RSS、进程数与 CPU JSONL。 | 完成；只读、不启动或终止进程。 |
| `scripts/ui0_bench/test_capture_tauri_process.py` | 覆盖树聚合/变化、父 PID 重用、root/child PID 重用、首样本不输出 CPU、CPU 归一化、schema/property 失败、固定 PowerShell 调用、权限/CIM 错误、Windows 限制、输出不覆盖、root 中途退出清理、输出限制及现有 runner schema。 | 17 项聚焦测试。 |
| `docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md` | 同步 UI-0 采样器状态、后续真实 Tauri 采样步骤和开放门禁。 | 更新至 v1.9；UI-0 保持 OPEN。 |
| 本报告 | 记录实现、验证、工具链 smoke 和未完成验收。 | 完成。 |

实现边界：scenario 仅接受有限 ASCII 标识；PowerShell 命令是固定查询，PID 在校验为 uint32 后以十进制写入固定脚本；不使用 shell、不接受任意命令、不把进程名、命令行、路径、错误原文或原始 stderr 写入 JSONL。每个 snapshot 都启动受限 PowerShell helper，最小采样间隔设为 1 秒以限制启动频率；该 collector 对被测应用的观察者开销尚未量化，真实性能对比前必须校准。RSS 按进程树汇总每个 `WorkingSetSize`，共享页可能在多个进程的工作集中重复出现，因此它不能代表进程树的唯一物理内存。文件以独占创建方式打开，已存在输出不会被覆盖。采样中断时删除本次创建的不完整文件，并通过稳定错误码报告失败。

CPU 百分比为进程树累计 kernel/user CPU 时间增量 ÷ 实测采样墙钟间隔 ÷ 逻辑处理器数 × 100，表示整机逻辑处理器总容量中的百分比，封顶 100%。Windows `Win32_Process` 的 PID 可重用，官方类文档建议用创建时间识别父进程生命周期；累计 CPU 字段单位标注为 100 纳秒，working set 单位为 bytes。[Microsoft Win32_Process 文档](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-process)

## §2 验证摘要

| 验证 | 命令 / 操作 | 结果 |
|---|---|---|
| 聚焦标准库测试 | `rtk python -B -m unittest discover -s scripts/ui0_bench -p test_capture_tauri_process.py -v` | **17/17 通过**。包含现有 runner `read_driver_samples` 对采样 JSONL 的 schema 接收检查。 |
| Python AST | `rtk python -B -c "import ast,pathlib; [ast.parse(pathlib.Path(p).read_text(encoding='utf-8'), filename=p) for p in ('scripts/ui0_bench/capture_tauri_process.py','scripts/ui0_bench/test_capture_tauri_process.py')]"` | 通过；两份源码均由 Python 3.13.7 解析。 |
| Windows CIM 工具链 smoke | `rtk python -B -c <bounded inline smoke>`；Python 测试宿主 PID 为 root，间隔 `1s`、时长 `2.2s`，写入临时目录中的 JSONL 并由临时目录自动清理 | 采集完成并产生 5 条记录：两次 RSS、两次 process count 和一次后续 CPU delta。首样本未伪造 CPU 值。该 root 是短命 Python 工具宿主，不是 Tauri；该 smoke 只证明本机 PowerShell/CIM/JSONL 工具链可运行，不是产品或性能证据。未采集该次 smoke 的硬件 inventory。 |
| 输出 schema | 聚焦测试将样本送入 `run_measurement.read_driver_samples` | 接受 3 个记录，并识别 RSS counter 与 process-count metric；该测试使用 fixtures，不是实际 Tauri benchmark。 |
| CypherGraph Guardian | `rtk python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --root . --changed --stamp` | **通过**；2 个变更源码文件的 Cypher 头与指纹通过校验。 |
| 独立代码审查 CLI | `rtk proxy coderabbit --version` 与 `rtk proxy coderabbit auth status` | 未能启动；本机 WSL shim 对 `/root/.local/bin/coderabbit` 返回 `Permission denied`。不据此声称 CodeRabbit review 已执行。 |
| Diff 检查 | `rtk git diff --check` | 提交前复核；不包含未审阅路径。 |
| CodeGraph | 当前 worktree 无索引；会话提供的工具中无 `index_repository` / `check_index_coverage` | 未初始化索引；本 lane 只审阅精确范围内源码和文档，不作全仓结构结论。 |

Python 版本为本机 `3.13.7`。Smoke 未记录 Tauri 产品版本、构建 SHA、renderer、Rust 工具链、benchmark workload 或目标设备 inventory：这些字段对 PowerShell tooling smoke 不适用/未采集，不能从中推导产品性能结论。

## §3 已知缺口

1. 当前没有对真实 Tauri 根 PID 运行过产品采样；未记录 Tauri build SHA、track、renderer 或实际 UI workload。上述 PowerShell smoke 不得进入 benchmark 结果。
2. 没有 Rust-native renderer 可比运行。UI-0 renderer 选择、性能阈值和 release gate 保持 OPEN。
3. 本切片只采集 RSS、CPU 与进程数量；queue/cache、terminal buffer、Canvas visible nodes/edges、取消/停止到 drain、孤儿句柄等计数器与生命周期事件仍未知。
4. 一次短 tooling smoke 只覆盖 PowerShell 进程的两次 RSS/process-count 样本与一次 CPU 差分；未采集该次 smoke 的硬件/显示/电源 inventory。
5. 进程树以 WMI snapshot 的 PID/父 PID/创建时间构建；采样间隔内新建后又退出的子进程，以及退出进程的最后 CPU 区间可能不出现在相邻快照差分中。root 缺失或身份改变时整次采样 fail closed。
6. CodeGraph index 和 coverage 不可用，未做全仓 graph 审计；依照工作 brief 不初始化索引。
7. 每秒启动一次 PowerShell/CIM snapshot 的 collector overhead 未测量；产品比较前须量化它对目标进程 CPU、响应时延与 RSS 的扰动，或改为低扰动的持久采集器。

## §4 子代理失败接手清单

本 lane 未派发子代理，没有子代理失败或待接手成果。父级集成复核仍需确认 manifest 路径、commit 与 `dev` checkout 状态；本 lane 不 rebase、merge、push 或清理 worktree。

## §5 守门规则

1. 只支持 Windows；非 Windows 环境失败关闭。
2. root PID 必须由操作者显式提供，并固定为 1..uint32 上限。
3. scenario、condition、interval、duration 和输出路径都是必填；scenario 仅接受有限字符集，interval 不低于 1 秒。
4. 仅使用固定 PowerShell 查询本机 `Win32_Process`，以参数数组运行，`shell=False`；不接受任意命令输入。
5. 查询只取 PID、父 PID、创建时间、working set、kernel/user 累计 CPU；禁止查询或序列化进程名、命令行和可执行路径。
6. 输出同时受 8,192 个主机进程、4,096 个进程树成员和 2 MiB snapshot 文本上限约束；越界 fail closed。
7. 每个目标进程的创建时间、working set 与累计 CPU 字段必须可用；未知值不能伪造成零值样本。
8. 用父 PID + 创建时间过滤 PID 重用造成的伪父子边；root 创建时间变化时中止采样。
9. 每个 CPU delta 只跨相同 PID、相同创建时间的进程求和；重用的子 PID 不继承旧进程 CPU 时间。
10. 第一快照只输出 RSS 和进程数，不将 CPU 写成零；首个有效 CPU 百分比必须来自后续实测累计时间差。
11. CPU 百分比分母固定为实际间隔 × 主机逻辑处理器数；语义是整机总逻辑 CPU 容量的 0–100%。
12. 输出文件使用独占创建，禁止覆盖；失败时清除本次写入的不完整样本文件。
13. JSONL 每行 ≤16 KiB、总数据 ≤8 MiB、样本记录 ≤10,000；检查在写入前执行。
14. 目标 PID 不存在、权限/CIM/PowerShell 失败、进程属性缺失、root 退出或身份变化必须 fail closed，并只返回非敏感稳定原因码。
15. 通过现有 runner 的样本校验，不扩展或修改 runner、fixture、inventory 与现有 Web evidence。
16. 单测、AST、Diff 与 Cypher header 验证必须分别记录实际结果；fixture、mock 与 tooling smoke 均不作产品 benchmark 通过证明。
17. renderer、性能阈值、Rust-native 可比性、collector overhead 校准与 UI-0 发布验收继续 OPEN，直至拥有带 provenance 的真实目标设备证据与独立评审。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 父级复核待完成 |
| SRE Lead | 父级复核待完成 |
| 平台 | 父级复核待完成 |
| 评审主持 | 父级复核待完成 |
| PM | 父级复核待完成 |

**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。上述签字状态不是性能验收批准。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1.0 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增只读 Windows Tauri 进程树 sampler 阶段报告，记录聚焦验证、tooling smoke 与仍开放的 UI-0 性能门。 | WBS-UI0-TAURI-PROCESS-SAMPLER-20261023 brief。 |
