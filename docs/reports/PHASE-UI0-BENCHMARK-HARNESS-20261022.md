# PHASE-UI0-BENCHMARK-HARNESS-20261022

| 项目 | 内容 |
|---|---|
| 版本 / 状态 | v0.1.2 / harness 已实现并通过父级自审；三条 UI track 尚未运行 |
| 日期 | 2026-10-22 |
| 分支 / 基线 | `codex/ui0-bench-harness-20261022` / `dev` `5a051bfd8f76fbfbe8ff49a994579c60166d6ec6` |
| 范围 | 仅 UI-0 benchmark generator、Windows inventory、measurement runner、标准库测试和本报告 |
| 规范输入 | `ui0-bench-fixture-v0.1.json` SHA-256 `d06d57b7ca0d30fdb9960e9fc8757aa936c991ade1993f2f59ea0ce9b5118328` |

## §0 目的

为 UI-0 提供可重复的 JSONL fixture、Windows 主机显示/刷新率/DPI/电源清单和接收真实 UI driver 样本的测量 runner。对应输入是 WBS UI-0 与 `UI0-RUST-DESKTOP-ADR-20261022.md`；规范 manifest 保持只读。该工具链提供测量输入和结果记录，不代表任何 UI renderer 已完成比较或达标。

## §1 改动矩阵

| 已登记路径 | 实施内容 | 状态 |
|---|---|---|
| `scripts/ui0_bench/generate_fixture.py` | 以 manifest/seed 固定顺序、ID、时间戳和 SplitMix64 输出 JSONL；终端字节以有限大小 chunk 流式输出并记录 SHA-256；manifest、记录数、worktree、terminal、canvas、BI、作用域和 repetitions 均有显式上限。 | 已实现；规范输入两次输出字节一致。 |
| `scripts/ui0_bench/test_generate_fixture.py` | Python 标准库测试覆盖确定性、不同 seed、manifest 验证、终端 chunk、PRNG 固定向量、无 driver 状态、真实样本汇总、raw hash、inventory 筛选及大小上限、driver 行/总字节上限、全空行超限、非普通文件、负计数器、单位冲突和 p50/p95。 | 16 项通过。 |
| `scripts/ui0_bench/capture_windows_inventory.ps1` | 收集 Windows/CPU/内存/GPU 与 driver、活动显示分辨率/刷新率/DPI、电源计划；API 失败记录 `unknown` 与原因。 | 实机采集成功；4 个显示器的刷新率和 DPI 均可读。 |
| `scripts/ui0_bench/run_measurement.py` | 仅接受普通文件形式的 driver 输入；按字节预算流式读取 JSONL，单行最多 16 KiB、总输入另设独立 8 MiB 上限（包括空行；避免 16 KiB × 10,000 条样本的理论约 160 MiB 读取量）、最多 10,000 个样本；保留筛选后的原始样本，汇总 p50/p95 和计数器/生命周期时间；记录 manifest SHA、Git HEAD、track、renderer/runtime/toolchain/build mode、inventory。缺少样本时不合成 UI 时延。 | 单测通过；读取失败只保留原始输出文件，不写 `measurement.json`；没有 UI driver 时状态为 `unavailable` 或 `not_run`。 |
| `docs/reports/PHASE-UI0-BENCHMARK-HARNESS-20261022.md` | 记录实现矩阵、验证证据、实机采集、缺口和守门条件。 | 本报告。 |

本 lane 仅交付 UI-0 harness 与本报告，没有改动 WBS、UI 规划、规范 manifest 或前端文件。父级对脚本、边界、失败状态和验证结果完成自审；该报告只记录 harness 验收，不表示真实 UI track 或 renderer 已通过。分支集成由父级在单独复核目标 owner、HEAD 和工作区后执行；本任务未 push。

## §2 验证摘要

| 验证 | 命令 / 操作 | 结果 |
|---|---|---|
| 标准库单测 | `rtk proxy python -B -m unittest discover -s scripts/ui0_bench -p test_generate_fixture.py -v` | 父级复跑 16/16 通过，3.319 秒。 |
| Python 语法 | `python -B -c "import ast, pathlib; paths = [pathlib.Path('scripts/ui0_bench/generate_fixture.py'), pathlib.Path('scripts/ui0_bench/run_measurement.py'), pathlib.Path('scripts/ui0_bench/test_generate_fixture.py')]; [ast.parse(p.read_text(encoding='utf-8'), filename=str(p)) for p in paths]"` | 3 个 Python 文件通过。 |
| PowerShell 语法 | `System.Management.Automation.Language.Parser.ParseFile` 对 `capture_windows_inventory.ps1` 解析并检查 parser errors | 父级复跑通过。 |
| 规范 fixture | 对规范 manifest 分别运行 `generate_fixture.py --manifest <canonical> --output <new-jsonl>` 两次 | 每次 92,078 records、73,610,591 bytes；SHA-256 均为 `51cd0d1a016e7c72505d30bcd6afd58c8e4ee4558622720e46b46d905b2e0c91`；文件大小与 SHA-256 一致。 |
| 实机 inventory + runner | `capture_windows_inventory.ps1` 输出 JSON，runner 使用该 JSON 与规范 manifest 生成一次无 track 的记录 | inventory `available`；测量状态 `unavailable`，原因 `track_not_selected`，UI 样本数 0。不是一次 UI benchmark 成功。 |
| CypherGraph Guardian | `rtk proxy python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | 父级复跑 PASS，4 个源文件已核对并刷新 stamp。 |
| CodeGraph | 本 worktree 没有可用 index，且无 `check_index_coverage` 工具 | 未初始化 index；依照任务 brief 仅做定向文档和源文件检查。 |
| CodeRabbit | `coderabbit review --agent -t uncommitted` | 未能运行：Windows shim 转到 WSL 后，`/root/.local/bin/coderabbit` 返回 `Permission denied`。因此不视为独立审阅通过；已做本地自审。 |
| Diff 检查 | `rtk proxy git diff --check` + 对 5 个交付路径逐行扫描尾随空白 | Git 检查无错误；另对新增的 5 个交付路径扫描并确认无尾随空格/制表符。 |

实际主机采集记录（2026-10-22 UTC）：Windows 11 Pro `10.0.26300` build `26300`，x64；Intel i7-13700F，16 cores / 24 logical processors；物理内存 `34,104,651,776` bytes。WMI 报告三项显示适配器：Virtual Desktop Monitor (`13.50.53.699`)、MuMu Virtual Display Adapter (`20.36.41.498`) 和 NVIDIA GeForce RTX 3060 (`32.0.16.1060`)。NVIDIA 的 `AdapterRAM` 仅记录为 CIM 报告的 `4,293,918,720` bytes，不将其解释成已验证的物理 VRAM 容量。活动电源计划为“高性能”，GUID `8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c`。

| 显示设备 | 分辨率 | 刷新率 | DPI |
|---|---:|---:|---:|
| `DISPLAY2`（主显示器） | 1080 × 1920 | 60 Hz | 96 × 96 |
| `DISPLAY1` | 1440 × 2560 | 59 Hz | 96 × 96 |
| `DISPLAY3` | 2560 × 1440 | 200 Hz | 96 × 96 |
| `DISPLAY4` | 1200 × 1920 | 59 Hz | 96 × 96 |

## §3 已知缺口

1. `tauri-react`、`shared-web`、`rust-native` 均未提供 UI driver 样本；没有真实交互延迟、RSS/CPU、queue/cache、terminal/Canvas 或 stop/cancel/drain 性能结论。
2. renderer 选择与性能通过门槛保持 **OPEN**，等待同一规范输入下三条 track 的可比真实运行。
3. `Win32_VideoController.AdapterRAM` 对 RTX 3060 返回的 CIM 字段不是经独立核验的显存容量；本报告不与 ADR 的硬件目标快照作容量等值判断。
4. CodeGraph index/coverage 在该 worktree 不可用；没有初始化索引。当前图结构结论限于 CypherGraph Guardian 的源文件关系检查和直接文件审阅，不声称图索引覆盖完整。
5. CodeRabbit 因 WSL CLI 权限错误未完成。本报告保留该限制，不将本地审阅称为独立 reviewer 通过。
6. 本 lane 未运行 Next.js build、Rust build 或 UI runtime benchmark；这些不能由 fixture 测试替代。

## §4 子代理失败接手清单

本 lane 没有再派发子代理；无子代理失败或待接手事项。CodeRabbit CLI 是本地审阅工具执行失败，已在 §2 记录，不属于子代理结果。

## §5 守门规则

1. 使用注册的 canonical manifest 和 SHA-256；该输入只读，不把 lane 输出写回 manifest。
2. 相同 manifest/seed/算法版本必须输出相同 JSONL 字节；PRNG 固定为 `splitmix64-v1`。
3. 生成顺序、ID 和时间戳由 manifest/seed 决定，不依赖当前时间或 Python `hash()`。
4. manifest、记录数、worktree、scope、terminal bytes、Canvas、BI 和 repetition 必须受明确上限约束。
5. JSONL 逐条写出；terminal stream 按不超过 3,072 bytes 的 chunk 逐段编码和哈希。
6. Windows inventory 记录采集方法和时间，并采集显示分辨率、刷新率、DPI、电源方案及 OS/CPU/内存/GPU 信息。
7. inventory 字段采集失败时写 `unknown` 和原因，不填默认成功值。
8. runner 只接受普通文件 driver 输入；逐行按剩余字节预算读取。单行最多 16 KiB、另设独立整份输入 8 MiB 上限（包括空行和空白行；单行与样本数上限仍允许理论约 160 MiB 读取量）、样本最多 10,000 条；任一上限超出即拒绝。
9. 没有 driver 或 track 时只写 `unavailable` / `not_run` 及原因，不合成 UI 时延。
10. 报告 raw sample、样本数、p50/p95 与明确单位；单位冲突拒绝汇总。
11. 缺少的资源计数器和 lifecycle 事件保持 `unknown`；计数值不能用 fixture 值伪装成运行观测。
12. 每次 runner 输出记录 manifest SHA、Git HEAD、运行 track 元数据和 host inventory。
13. renderer 决策及性能阈值保持 **OPEN**，直到收到可比实测证据和评审结论。
14. VM/K3s readiness、fixture 生成通过或 harness 单测通过均不等同于 UI benchmark 成功。
15. 仅输出 inventory schema 的字段白名单；不读取或输出环境变量、cookie、token 或 credential。
16. 不新增 Python/Node 依赖或 lockfile，不依赖 Docker Desktop 或 Multipass。
17. driver 读取失败或输入类型不符时不写 `measurement.json`；可保留 `raw_samples.jsonl` 作为失败证据。提交集成前复核精确 changed paths、CypherGraph header、Focused tests 和 `git diff --check`；本 lane 不 push/merge。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 父 lane 终审待完成 |
| SRE Lead | 父 lane 终审待完成 |
| 平台 | 父 lane 终审待完成 |
| 评审主持 | 父 lane 终审待完成 |
| PM | 父 lane 终审待完成 |

**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。父 lane 终审尚未完成；本栏不表示 UI track 或性能验收获批。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1.0 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 UI-0 benchmark harness、标准库验证、实机 inventory 与未执行的 UI track。 | UI0-BENCH-HARNESS-20261022 实施任务。 |
| v0.1.2 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 父级复跑 16/16 标准库测试、3 个 Python AST、PowerShell parser、4 个源码关系检查和 diff 检查；自审确认没有引入 UI 样本或把 unavailable/not_run 描述为性能通过。 | 按 UI WBS 推进 UI-0 并完成集成前复核。 |
| v0.1.1 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 driver 普通文件限制、独立 8 MiB 总输入预算和失败无 measurement.json 的回归测试；同步 16 项验证结果。 | 父 lane review 指出空行可绕过样本上限。 |
