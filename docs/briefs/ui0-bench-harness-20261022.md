# Brief: ui0-bench-harness-20261022

**Agent**: Codex worker
**Phase**: UI0-BENCH-HARNESS-20261022
**Created**: 2026-10-21 18:10:40

---

## 任务
按 UI-0 WBS 实现确定性 benchmark fixture generator、Windows 设备显示/电源采集和 measurement runner。目标是让后续 Tauri+React、共享 Web 与 Rust-native parity 三条 track 使用同一输入并保存原始样本；不得编造尚未测得的 UI 指标或门槛。

## Owner 与 lane
- Owner: worker subagent
- Branch: codex/ui0-bench-harness-20261022
- Worktree: C:\Users\leo19\.codex\worktrees\ui0-bench-harness-20261022\Star
- Base: local dev at 5a051bfd8f76fbfbe8ff49a994579c60166d6ec6
- 只修改登记的 5 个文件：scripts/ui0_bench/generate_fixture.py、scripts/ui0_bench/test_generate_fixture.py、scripts/ui0_bench/capture_windows_inventory.ps1、scripts/ui0_bench/run_measurement.py、docs/reports/PHASE-UI0-BENCHMARK-HARNESS-20261022.md。不要改 WBS、UI 规划、fixture manifest 或任何 frontend UI 文件。

## 设计输入与已核对证据
- WBS: docs/implementation-plans/WBS-UI-PLAN-NEXT-20261022.md，UI-0 要求 deterministic generator/runner、每次测量记录 display refresh/DPI/power，并保留 renderer/threshold open。
- 规范输入 manifest 位于 parent lane 的只读副本：C:\Users\leo19\.codex\worktrees\ui-ux-next-20261021\Star\docs\implementation-plans\fixtures\ui0-bench-fixture-v0.1.json。该文件固定 UI0-BENCH-001，seed 20261021；4 projects × 5 runs；4,400 worktrees；50 rows/page；picker 200-row cap；4 active runs/32 agents/8 terminal panes；每 pane 1 MiB；Canvas 500/2,000/10,000 nodes；5,000 BI events；50 focus cycles；5 cold/30 warm repetitions；报告 p50/p95。SHA-256: D06D57B7CA0D30FDB9960E9FC8757AA936C991ADE1993F2F59EA0CE9B5118328。不要复制或修改此 manifest。
- UI0-RUST-DESKTOP-ADR-20261022.md 记录 target snapshot：Windows 11 Pro 10.0.26300 x64、i7-13700F 16C/24T、32 GB RAM、RTX 3060 12 GB、driver 610.60。当前 display refresh/DPI/power 尚未收集。
- Playwright 已在 frontend 作为 devDependency；仓库有既存 canvas/terminal/worktree e2e。它们不能作为 Rust-native parity 的证据。没有可用的 CodeGraph index：当前 repo 仅有 codegraph_explore，父级已记录无有效 index；没有 check_index_coverage 工具，故仅基于上述文档和定向源搜索。
- 本地 MSRV 1.99.0、Windows-only inventory；不要引入 Docker Desktop/MultiPass 依赖、额外 Python/npm 依赖或改 lockfile。

## 实施要求
1. Generator 读取 canonical manifest 路径参数并流式输出 stable JSONL；固定算法版本、PRNG 算法、顺序、稳定 ID、timestamp 和 terminal byte stream。相同 manifest/seed 必须产生字节级相同输出与 SHA-256；不能依赖 Python hash()、当前时间或未固定随机算法。显式限制驻留内存；大型 terminal/Canvas 数据不得整体拼在内存中。
2. PowerShell inventory 采集 OS/build、CPU、memory、GPU/driver、每个活动 display 的分辨率/refresh/DPI、active power scheme、采集时间和采集方法；可读性或 API 失败时记录 unknown + 原因，不能静默填默认值。严禁读取/输出环境变量、cookie、token 或 credential。
3. Measurement runner 每个 run 保存 manifest/profile SHA、git commit、track、renderer/runtime/toolchain/build mode、host inventory、cold/warm 状态、raw samples、sample count、p50/p95、RSS/CPU/process-tree/queue/cache/terminal/Canvas counters（缺失即 unknown），以及停止/取消/drain 样本。可接收真实 UI driver 的样本；没有 driver 或 track 时标为 unavailable/not-run，不能用合成时延冒充 UI 交互。
4. 不设未经评审的性能通过阈值；只生成结果/比较数据。不要将 VM/K3s readiness 当成 agent runtime 或 UI benchmark 成功。
5. 为 generator/runner 加 stdlib 测试，覆盖 determinism、不同 seed、schema/参数验证、有界终端流和 p50/p95 统计；在报告中记录实际通过项和未执行的 UI tracks。
6. 新 Python/PowerShell 源文件遵守 CypherGraph Guardian header；运行 guard 前核对每条关系有代码证据。命令经 RTK；测试单 worker、有限资源。
7. 提交前列出 diff、运行 focused tests/type/syntax checks、记录真实主机采集结果和精确限制。不要 push，也不要改 dev/main；报告推荐 cherry-pick/merge 供 root 审核。

## 交付/验收
- 同一 manifest+seed 的两次生成 byte-identical；输出数据规模/哈希可重复。
- inventory 包含当前 monitor DPI/refresh 和 power scheme，或精确记录 unavailable 原因。
- runner 不伪造不存在的 UI adapter 指标；原始数据能追溯到一次运行。
- 测试、CypherGraph Guardian、diff check 结果真实记录；任何缺门禁保持 BLOCKED/OPEN。
- 最终 handoff 提供 branch/worktree/base/HEAD、文件清单、commit、验证、已知缺口和建议集成操作。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
