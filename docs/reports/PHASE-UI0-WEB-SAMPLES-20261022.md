# PHASE-UI0-WEB-SAMPLES-20261022

| 项目 | 内容 |
|---|---|
| 版本 / 状态 | v0.1.0 / 三次既存 shared-Web 样本已审计；UI-0 整体仍未完成 |
| 日期 | 2026-10-22 |
| Lane | branch codex/ui-plan-v0-7-20261022；base dev df1c22f66d8d0c3b48baac7e348494ba3466569b |
| 依据 | docs/briefs/UI0-WEB-SAMPLES-20261022.md 登记的文件清单 |
| 规范输入 | ui0-bench-fixture-v0.1.json SHA-256 d06d57b7ca0d30fdb9960e9fc8757aa936c991ade1993f2f59ea0ce9b5118328 |

## §0 目的

审计已经存在的三次 shared-Web Playwright 样本，核对样本 schema、run metadata、数量、时间、fixture/commit 身份和 raw 输出哈希，并为 UI-0 harness 报告补充有界的浏览器测量证据。此 lane 没有再次运行 Playwright，没有探测、启动、停止、重启或修改本地服务进程；这份报告不表示 UI-0 完成，也不选择 renderer 或性能门槛。

## §1 改动矩阵

| 登记路径 / 证据 | 内容与结果 |
|---|---|
| scripts/ui0_bench/playwright_driver.mjs | 源审查确认只接受 http/https 的 localhost、127.0.0.1、::1；固定访问 /projects 与 /worktree；cold 各一次、warm 默认五次且最多十次，导航 timeout 30 秒；仅保留有限的浏览器指标；输出限 8 MiB，并以 wx 创建文件以拒绝覆盖。此 lane 未修改 driver。 |
| docs/reports/ui0-web-driver-samples-20261022.jsonl | R1：52 条，6,780 bytes，SHA-256 13af00ae216f43a22302e2a0d4b516e6b90bd8ab76180cbbdda4b3006e888ad6。 |
| docs/reports/ui0-web-driver-samples-20261022-r2.jsonl | R2：60 条，7,730 bytes，SHA-256 28da9f59522a49b26a8323a81db71846dc39425e0686d793a1b174acd17c0a20。 |
| docs/reports/ui0-web-driver-samples-20261022-r3.jsonl | R3：60 条，7,746 bytes，SHA-256 c2c9f235cdabbb805ec704ca2733ec774abde535ee399e4ab00e3dacfdd9dd6c。 |
| 三个 ui0-web-measurement-run 目录及其 measurement.json / raw_samples.jsonl | 每个 measurement 的 sample_count 与源 JSONL、raw_samples 行数相符；逐流计算的 raw SHA-256 均与 measurement.json 相符。三个 run 的 measurement 均为 shared-web / available，记录的 Git commit 与本 lane base 相同，manifest SHA 与规范输入相同。 |
| docs/reports/ui0-web-host-inventory-20261022.json | SHA-256 24f1cd1c5e7a8abedfb7f8f42fc003a0460b6f73878356a5087341906949269e；R3 的 host_inventory.source_sha256 与此文件一致。R1/R2 的 host inventory 为 unknown / not_supplied。 |
| docs/reports/ui0-web-dispatch-20261022.jsonl | 仅有一次 dispatcher brief 记录；未保存 Playwright invocation、实际 base URL 或 stdout。 |
| docs/reports/PHASE-UI0-BENCHMARK-HARNESS-20261022.md | 更新旧的“没有真实 UI 样本”表述，保留 Rust/Tauri、renderer 和阈值等 OPEN 状态。 |
| docs/reports/PHASE-UI0-WEB-SAMPLES-20261022.md | 本次审计结果、逐 run 指标、来源哈希和限制。 |
| docs/briefs/UI0-WEB-SAMPLES-20261022.md | dispatcher brief，原样保留。 |

三个 run 的身份：

| Run | run_id | measurement observed_at (UTC) | 样本数 | renderer 记录 | host inventory |
|---|---|---|---:|---|---|
| R1 | 8dced262-fbdb-42e9-b3da-3e4ed3dae212 | 2026-10-22T14:09:54.339Z | 52 | Chromium via Playwright；精确 Chromium 版本未记录 | unknown / not_supplied |
| R2 | 22ef0752-40ce-4da6-aa42-61a5cfb672b5 | 2026-10-22T14:11:30.385Z | 60 | Chromium 153.0.8010.12 via Playwright | unknown / not_supplied |
| R3 | a7472124-64d0-4d54-ac5c-5df3274c0e4f | 2026-10-22T14:13:13.028Z | 60 | Chromium 153.0.8010.12 via Playwright | available；关联上方 inventory 文件 |

三份样本均逐行通过现有 runner 的 _validate_sample；最大行长 137 bytes，文件大小远低于 8 MiB。三个 measurement 的 track 值均为 shared-web，Git commit 为 df1c22f66d8d0c3b48baac7e348494ba3466569b，canonical manifest SHA-256 为 d06d57b7ca0d30fdb9960e9fc8757aa936c991ade1993f2f59ea0ce9b5118328。run_id 均可解析为 UUID，observed_at 为有效 UTC 时间且依次递增。measurement.raw_samples.count、源记录数和磁盘 raw_samples 行数一致；raw_samples SHA-256 均重新计算匹配。原始样本、raw 输出、measurement 和 inventory 文件均未修改。

driver 固定两条 route：projects_route 和 worktree_route。runner 的 metric_summaries 按 condition 与 metric 汇总两条 route 的记录，不按 route 分组。下表保留每个 run，未跨 run 合并；p50/p95 使用 runner 的插值分位数方法，显示到小数点后三位。

### R1 指标

| 条件 | 指标 | n | p50 | p95 | 单位 |
|---|---|---:|---:|---:|---|
| cold | navigation_load_ms | 2 | 359.600 | 473.270 | ms |
| cold | dom_content_loaded_ms | 2 | 215.500 | 351.040 | ms |
| cold | first_contentful_paint_ms | 2 | 286.000 | 406.600 | ms |
| cold | dom_element_count | 2 | 456.500 | 644.150 | count |
| cold | javascript_heap_used_bytes | 2 | 6301796.000 | 6594396.800 | bytes |
| warm | navigation_load_ms | 10 | 145.400 | 221.380 | ms |
| warm | dom_content_loaded_ms | 10 | 117.800 | 165.100 | ms |
| warm | first_contentful_paint_ms | 2 | 82.000 | 83.800 | ms |
| warm | dom_element_count | 10 | 456.500 | 665.000 | count |
| warm | javascript_heap_used_bytes | 10 | 12814948.000 | 20083077.200 | bytes |

### R2 指标

| 条件 | 指标 | n | p50 | p95 | 单位 |
|---|---|---:|---:|---:|---|
| cold | navigation_load_ms | 2 | 292.550 | 351.815 | ms |
| cold | dom_content_loaded_ms | 2 | 80.000 | 93.230 | ms |
| cold | first_contentful_paint_ms | 2 | 232.000 | 304.000 | ms |
| cold | dom_element_count | 2 | 456.500 | 644.150 | count |
| cold | javascript_heap_used_bytes | 2 | 6289684.000 | 6597700.000 | bytes |
| warm | navigation_load_ms | 10 | 310.800 | 425.010 | ms |
| warm | dom_content_loaded_ms | 10 | 98.200 | 200.350 | ms |
| warm | first_contentful_paint_ms | 10 | 254.000 | 379.000 | ms |
| warm | dom_element_count | 10 | 456.500 | 665.000 | count |
| warm | javascript_heap_used_bytes | 10 | 6346962.000 | 6553861.400 | bytes |

### R3 指标

| 条件 | 指标 | n | p50 | p95 | 单位 |
|---|---|---:|---:|---:|---|
| cold | navigation_load_ms | 2 | 352.800 | 411.300 | ms |
| cold | dom_content_loaded_ms | 2 | 101.000 | 108.830 | ms |
| cold | first_contentful_paint_ms | 2 | 286.000 | 363.400 | ms |
| cold | dom_element_count | 2 | 456.500 | 644.150 | count |
| cold | javascript_heap_used_bytes | 2 | 6561730.000 | 6593739.400 | bytes |
| warm | navigation_load_ms | 10 | 271.750 | 385.290 | ms |
| warm | dom_content_loaded_ms | 10 | 67.500 | 290.350 | ms |
| warm | first_contentful_paint_ms | 10 | 210.000 | 337.000 | ms |
| warm | dom_element_count | 10 | 456.500 | 666.650 | count |
| warm | javascript_heap_used_bytes | 10 | 6170700.000 | 6474099.400 | bytes |

R1 比 R2/R3 少 8 条。逐 route/condition/metric 核对后，这 8 条全部是 warm first_contentful_paint_ms：projects_route 缺 5 条，worktree_route 缺 3 条；其他指标点均存在。driver 只输出有限且非负的数值，paint entry 缺失时将其作为 null 过滤，不会写入样本。记录没有显示 run 失败，但也没有证据确定浏览器为何没有提供这 8 个 FCP 值。R2/R3 的 warm FCP 各有 10 条。没有重跑或用其他值补齐 R1。

R3 host inventory 的 captured_at 为 2026-10-22T14:12:15.833Z，measurement observed_at 为 14:13:13.028Z；该 inventory SHA-256 与 R3 measurement 引用一致。inventory 报告 Windows 10.0.26300 build 26300、64-bit，Intel i7-13700F（16 cores / 24 logical processors），物理内存 34,104,651,776 bytes。它记录三个显示适配器：Virtual Desktop Monitor 13.50.53.699、MuMu Virtual Display Adapter 20.36.41.498、NVIDIA GeForce RTX 3060 32.0.16.1060。CIM 的 AdapterRAM 仅作为 CIM 原始报告值，不当作已验证的物理显存。

| 显示设备 | 分辨率 | DPI | 刷新率 |
|---|---:|---:|---|
| DISPLAY2（主显示器） | 1080 × 1920 | 96 × 96 | unknown / collection_failed |
| DISPLAY1 | 1440 × 2560 | 96 × 96 | unknown / collection_failed |
| DISPLAY3 | 2560 × 1440 | 96 × 96 | unknown / collection_failed |
| DISPLAY4 | 1200 × 1920 | 96 × 96 | unknown / collection_failed |

电源方案 GUID 为 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c；保存的显示名称含替换字符，因此不声明其可读名称。R1/R2 没有提供 host inventory，不能借 R3 的清单回填。

## §2 验证摘要

| 验证 | 命令 / 操作 | 结果 |
|---|---|---|
| 样本 schema 与来源 | 使用现有 runner 的 _validate_sample 逐行验证三个输入，检查 8 MiB/16 KiB 限制、记录数、run UUID、UTC 时间、track、Git、manifest SHA、raw 文件行数和 SHA-256 | 3/3 通过；52、60、60 条；所有 raw hashes 与 measurement 匹配；fixture 和 commit 一致。 |
| Shared-Web 汇总 | 分别读取三个 measurement.json 的 cold/warm metric_summaries，不合并 run | 每个摘要的 count、p50、p95 已记录于 §1；performance_thresholds 和 renderer_choice 均为 OPEN。 |
| Host inventory 引用 | SHA-256(ui0-web-host-inventory-20261022.json) 与 R3 host_inventory.source_sha256 比较 | 匹配；R1/R2 明确为 not_supplied。 |
| Playwright driver 语法 | rtk proxy node --check scripts/ui0_bench/playwright_driver.mjs | 通过。 |
| 标准库测试 | rtk proxy python -B -m unittest discover -s scripts/ui0_bench -p test_generate_fixture.py -v | 16/16 通过，3.084 秒。 |
| Python AST / PowerShell parser | 现有 generator、runner、test 的 AST parse；capture_windows_inventory.ps1 的 PowerShell Parser | Python 3/3 通过；PowerShell parse 通过。 |
| CypherGraph Guardian | rtk proxy python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp | PASS；1 个源文件已核对。 |
| whitespace / diff | git diff --check；登记文件逐行尾随空白检查 | PASS after report updates; all 15 manifest paths had zero trailing whitespace. |
| CodeGraph MCP | 对本 worktree 查询时返回没有可用 .codegraph 索引；没有调用 init | 不创建索引，直接源文件和样本审计。 |

## §3 已知缺口

1. measurement 记录将三次 run 标记为 shared-web，driver 的静态 host allowlist 也只准许 loopback；但现有样本和 dispatcher audit 没有保留每次实际的 base URL 或完整 driver 命令，因此实际 endpoint provenance 仍为 UNKNOWN。
2. R1 的 Chromium 精确版本未记录；R1/R2 没有 host inventory。仅 R2/R3 有同一 Chromium 153.0.8010.12 的 renderer 字符串，但样本仍按 run 分开报告。
3. R1 有 8 个 warm FCP 值缺失；缺失值的浏览器侧原因 UNKNOWN。R2/R3 完整记录 FCP，不回填 R1。
4. R3 inventory 的 4 个显示器刷新率均为 unknown / collection_failed；保存的 power scheme name 含替换字符。R3 inventory 不代表 R1/R2 的采集环境。
5. 浏览器导航时间、DOM count 和 JavaScript heap 不等于 native UI 的进程树 RSS/CPU，也不是 Rust desktop 的内存或 CPU 数据。没有 queue/cache、terminal/Canvas 计数器、stop/cancel/drain 生命周期观测。
6. 本报告只覆盖记录为 shared-web 的 Chromium 页面样本；没有 Tauri+React 或 rust-native 可比 run，也不代表二者性能相等或已达发布标准。
7. renderer 选择、性能阈值和 UI-0 完整验收保持 OPEN。此 lane 未检查当前 localhost server 状态，因为已有 R2/R3 完整有效，不需要重跑；没有声明 server 当前在线。
8. CodeGraph MCP 没有可用索引；CodeRabbit 本 lane 未运行，先前报告记录其 WSL shim Permission denied 限制。

## §4 子代理失败接手清单

本 lane 未派发子代理；没有子代理失败或待接手工作。

## §5 守门规则

1. 保留三份已有 source JSONL 与对应 measurement/raw 文件的字节内容；审计只读，不回填或重写样本。
2. 只接受 shared-Web driver schema；行长和总字节上限通过现有 runner 验证。
3. driver 仅访问 localhost、127.0.0.1、::1 和固定的 /projects、/worktree routes；没有授权时不得改用其他 host。
4. driver 输出最大 8 MiB，文件以 wx 创建；不得覆盖现有证据。
5. cold/warm 分组、count、p50/p95 分 run 报告；不将 R1/R2/R3 混合成单一值。
6. 缺失值、未提供 inventory、未记录的版本和 endpoint 均标记 unknown，不填默认值。
7. JavaScript heap 不是 native process RSS；不把浏览器指标宣称为 Rust/Tauri 性能证据。
8. 不因 R2/R3 具有完整 FCP 而把其值补写到 R1。
9. 不因样本存在或 harness 测试通过而宣称 UI-0 完成；renderer/threshold 保持 OPEN。
10. 不启动、停止、重启或修改本地 server 或其他进程；本 lane 没有执行新的浏览器采集。
11. 不读取或输出环境变量、cookie、token 或 credential；不依赖 Docker Desktop 或 Multipass。
12. 只提交 brief 登记的 15 个路径，不修改 WBS、UI plan、canonical fixture 或 frontend。
13. 合入前重新核对 lane HEAD、manifest 路径和每个样本 SHA；本 lane 不 rebase、不 merge、不 push。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 父 lane review 待完成 |
| SRE Lead | 父 lane review 待完成 |
| 平台 | 父 lane review 待完成 |
| 评审主持 | 父 lane review 待完成 |
| PM | 父 lane review 待完成 |

修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。此栏不代表 renderer、性能阈值或 UI-0 已获批。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1.0 | 2026-10-22 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 审计并分别记录三次 shared-Web driver 样本、host inventory、measurement 汇总及现存限制。 | UI0-WEB-SAMPLES-20261022 lane。 |
