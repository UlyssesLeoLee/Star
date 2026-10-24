# Brief: UI0-TAURI-PROCESS-SAMPLER-20261023

**Agent**: ui0_tauri_process_sampler_worker
**Phase**: WBS-UI0-TAURI-PROCESS-SAMPLER
**Created**: 2026-10-23 18:23:15

---

# UI-0 Tauri 进程树采样器

## 任务与基线

- 任务：补上现有 UI-0 JSONL runner 缺少的 Windows Tauri 进程树 RSS/CPU 采集入口。
- 基线：dev@194f7b89464432472707164f3126cc4bff9af0d1；当前本地 dev 比记录的 origin/dev ahead 32。不要 fetch、push 或修改 dev checkout。
- 此切片不选择 renderer、不设置性能门槛，也不声称已有 Tauri/Rust-native benchmark 通过。

## 现有契约与证据

- scripts/ui0_bench/run_measurement.py 的 COUNTER_NAMES 已包含 process_tree_rss_bytes 与 process_tree_cpu_percent；样本格式为 kind=sample、scenario、condition、metric、value、unit，可由现有 runner 汇总 p50/p95。
- 现有 track 值是 tauri-react、shared-web、rust-native。新增采样器应产生该 runner 可读的原始 JSONL，不修改现有 runner 或其已注册 lane 文件。
- WBS-UI-PLAN-NEXT-20261022.md 与 PHASE-UI0-BENCHMARK-HARNESS-20261022.md 明确：浏览器样本不等于 Tauri/Rust-native 证据，native process-tree RSS/CPU、queue/cache、terminal/Canvas 和 cancel/drain 仍未验证。
- 目标设备已知是 Windows；不要将本工具采集的 Tauri+React 数据标成 Rust-native renderer 数据。

## 范围

只拥有 manifest 中的 5 个文件。新增 scripts/ui0_bench/capture_tauri_process.py：Windows-only、只采样、不启动/终止进程；操作者显式给 Tauri 根 PID、scenario、cold/warm、采样间隔与时长。以 Win32_Process 父子关系找进程树，处理 PID 重用/进程退出/子进程变化；输出进程树 RSS、CPU 和进程数量样本，raw file 有界、创建时防止覆盖、输出不可包含进程名或其他无关宿主信息。CPU 数值要说明核数归一化方式；不伪造起始区间 CPU 值。

采样要受现有 runner 的 16 KiB 单行、8 MiB 总输入、10,000 条样本限制约束。使用固定的 PowerShell 查询和 subprocess 参数列表，无 shell、无任意命令输入；缺少 Windows/CIM/目标 PID 或权限时输出结构化失败，不能退化成手工样本。新增单测通过 mock snapshot 子进程覆盖树聚合、RSS、CPU delta、进程退出/PID 重用、限制与失败路径。可做一次对当前普通进程的短采集 smoke，但必须标为工具链 smoke，不能计入 Tauri benchmark。

## 文档与验收

- 更新 WBS-UI-PLAN-NEXT-20261022.md 的 UI-0 状态与下一步，准确记录新增采集器、聚焦验证和仅 process-tree 两类资源指标的边界；保持 UI-0/renderer/阈值和真实 Tauri/Rust-native运行门 OPEN。
- 新增阶段报告，使用仓库规定的 7 段结构。记录测试、实际 smoke（若执行）、机器与构建的缺失字段；不得把 fixture 或 mock 测试描述为产品性能结果。
- 仅 Python 标准库，不增加依赖。为新增 Python 源文件添加证据充分的 CypherGraph 头；遵循 CypherGraph Guardian 与 cypher-header，并运行 cgg_guard.py --changed --stamp。运行此 lane 的 focused unittest、Python AST 和 diff 检查。
- 不修改 scripts/ui0_bench/run_measurement.py、generate_fixture.py、capture_windows_inventory.ps1、已存在三次 Web 样本或 WLD-0 文件。
- UI-0 其余 queue/cache/terminal/Canvas、生命周期、真实 Tauri 与 Rust-native renderer 可比运行、target-device evidence、无障碍和选型决策仍须保留在 WBS。

## 协作边界

你负责实现、聚焦验证和提交此 lane。不要操作其他 worktree，不要 rebase、merge、push、删分支或清理 worktree。提交前确认只改 manifest 文件，commit author 使用 Ulysses <ulysses@mavis.local>。完成后报告 commit SHA、文件、验证命令/结果和所有开放门禁。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
