# PHASE-INFRA-2A-MULTIPASS-INVENTORY-REPORT

## 0. 目的

落地 INFRA-2 的首个 Rust 产品代码切片：通过固定、只读的 Multipass CLI inventory 观察接口，为后续可信 Host Infrastructure Manager 提供受限 adapter 起点。此阶段不启动或修改 Multipass VM/K3s，也不宣称 provider、Manager 或执行闭环完成。

## 1. 改动矩阵与验收

| 范围 | 实施结果 | 证据/边界 |
|---|---|---|
| Rust crate | 新增 [`multipass-adapter`](../../crates/multipass-adapter/src/lib.rs) 与 workspace registration。绝对 executable path、固定 `list --format json` argv、stdin closed、stdout 128 KiB / stderr 4 KiB、5 秒 deadline、single-flight。 | 只提供 `list_instances`；无 shell、任意 argv、lifecycle/guest command。 |
| 解析和资源界限 | 最多 512 行；接受 name-keyed state-record map 与非空 `list: [{name,state}]` row-list compatibility shape；拒绝空/超长/控制字符字段、重复名称/JSON key、空记录数组/row list、JSON 错误及超限；保留未知 state 为 `Other`。`registered_count` 包含全部行，不过滤 Stopped/Deleted。取消时 child 使用 `kill_on_drop`。 | 错误不保存或返回 stdout/stderr 原文。snapshot 不是 owner identity 或 readiness 证据。 |
| 测试 | 12 项 fixture unit tests 覆盖官方映射格式、row-list 兼容格式、重复 JSON key、跨 probe 对象的进程级 single-flight、Deleted 计数、坏 schema/字段、重复名、行上限、未知状态、stdout/stderr bounds、exact byte bound、相对 executable path。 | 使用 fixture/in-memory pipe，无 Multipass CLI/VM。 |
| 文档 | 同步 requirements v6.00 §50.8R / AC-INFRA-002-R，basic design v5.97 §16.42，Infrastructure DD v0.26 §7.12，Group DD v4.66 §8.16，implementation plan v6.29 §6.102。 | 没有 DB/schema/W-T-M 变更；同步两实例测试 cap 与 fail-closed 边界。 |

## 2. 验证摘要

| 命令/检查 | 结果 |
|---|---|
| Rust toolchain | `rustc 1.99.0 (b940084d7 2026-09-28)`；与本阶段明确选定的 1.99.0 一致。 |
| 初次 `cargo test -p multipass-adapter --lib --offline -j 4` 与 Cargo Clippy | parser 修正前 9/9 与 warnings-as-errors 通过；这两条旧证据不代表当前 parser。 |
| Rust 1.99.0 direct rustc test harness | 当前源码 12/12 通过，链接既有 workspace `serde`/`serde_json`/`thiserror`/`tokio` artifacts；未运行 Multipass。 |
| Rust 1.99.0 `clippy-driver --test -D warnings` | 当前源码通过，0 warning。 |
| Rust 1.99.0 `rustfmt --edition 2021 --check` | 当前源码通过。 |
| Cargo 当前源码重跑 | 未通过到编译阶段：离线 registry 缺 `async-stream`；在线 crates.io index 多次 TLS `close_notify`，手动中止重试。 |
| `cgg_guard.py --changed --stamp` | 通过，1 个 Rust source checked；新文件 Cypher source hash 已生成。 |
| `git diff --check` | 通过。 |
| 首次链接尝试 | MSVC `LNK1104` 无法打开长 target 路径下的 test exe；保留为失败记录。短路径 `C:\t\star-infra2a` 中已有依赖产物可用于当前 direct rustc harness。 |
| workspace-wide check/test、RSS/performance | 本阶段未运行。既有 dev 基线的 workspace check 不作为本新增 crate 的全 workspace 验证。 |
| live Multipass / VM | 本阶段未调用 Multipass CLI、未启动服务、未创建/改动实例。测试 inventory cap 仍为最多 2 条已登记实例（含 Stopped/Deleted）。 |
| CodeRabbit | 未能执行：Windows `coderabbit.cmd` shim 转调 `/root/.local/bin/coderabbit`，shell 返回 `Permission denied`。这不是 clean review 结果。 |

## 3. 已知缺口

- `multipass-adapter` 尚未接入独立 Rust Host Infrastructure Manager、typed IPC、审计/Outbox 或 UI。
- CLI 清单不是 `HostEnvironment`/instance/cluster ownership ledger；本 crate 不验证 Host/cluster generation、profile、资源余量、租约或 test cap 策略。
- 本 crate 不探测 Multipass daemon、driver、CPU/memory/disk、K3s API/node/CRI、Agent Runtime sandbox 或 Worktree bridge readiness。
- 没有 provision/start/stop/drain/upgrade/delete 操作，VM 生命周期、owner reservations、target DB/RLS、BI/Benchmark 和 Schedule/Task CLI execution 均未完成并保持 fail-closed。
- 当前 host 的 Multipass Stage-0 先决检查此前报告 daemon 未就绪；本阶段没有尝试启动服务。没有 live CLI 结果或桌面 RSS/并行公平性数据。
- Phase 9F4D-3D6 raw payload 留存策略仍等用户选择（加密 24 小时或 metadata+digest）；它与本只读 adapter 相互独立。

## 4. 子代理失败接手清单

- 本阶段没有派发子代理；改动由主 agent 在 Codex worktree 内完成并直接复核。
- 无子代理输出、commit 或 worktree 需要接手。

## 5. 守门规则

1. Rust workspace/toolchain 最低版本固定为用户选择的 1.99.0；Cargo、CI 与依赖要求保持一致。
2. Adapter 必须由未来可信 Host Manager 调用；桌面 UI、Plugin、Agent 与 Task CLI 不直接持有 Multipass 会话。
3. executable path 必须由可信配置解析为绝对路径；调用者不可传入命令参数。
4. 当前唯一允许的 argv 顺序为 `list --format json`，不经 shell 拼接；parser 支持 name-keyed mapping 与显式 row-list compatibility shape。
5. stdin 必须关闭；stdout 上限 128 KiB，stderr 上限 4 KiB。
6. 每个 probe deadline 固定为 5 秒；timeout 要终止子进程并 fail closed。
7. 同一进程内的所有 probe 对象共用一个 Semaphore；任一 probe 在途时其他对象立即返回 Busy，不积累无界队列。
8. future 取消时必须触发 child cleanup；不得因 UI/调用者取消而留下持续 CLI 进程。
9. spawn、read、wait、非零退出、JSON/字段错误和任何限制超限都不能被解释成空 inventory 或 Ready。
10. 任何 diagnostic 不得保留 Multipass 原始 stdout/stderr。
11. Stopped 与 Deleted 条目都必须保留在计数中；不能按“正在运行”过滤磁盘仍占用的实例。
12. 测试阶段所有已登记实例总数最多 2 条，含 Stopped/Deleted；这个临时测试 cap 不能变成客户拓扑上限。
13. 清单行名称只是 provider display value，不构成稳定 ID、owner proof、Host/cluster binding 或授权依据。
14. 未知 state 只能显示为未分类，不得映射为 Running、Ready 或 execution-capable。
15. daemon、VM、guest、K3s、Runtime sandbox 与 Worktree bridge readiness 必须作为独立证据层。
16. 本 crate 不得新增 launch/exec/mount/stop/delete/purge API；VM 生命周期仍需独立 owner ledger、Hook、授权和幂等操作门。
17. 本阶段测试只能使用 fixture/in-memory pipe；live test 若需要 VM 必须走带 host-local 独占锁的 bootstrap guard，且不自动 purge。

## 6. 签批栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis review — 已核对 Multipass 只读 adapter 与 Manager/owner/readiness 分层。 |
| SRE Lead | Mavis review — 已核对固定 deadline、single-flight、bounded IO 与取消清理；live daemon/VM 未验收。 |
| Platform | Mavis review — Rust 1.99.0、12/12 direct rustc tests、clippy-driver `-D warnings`、rustfmt 与 CGG 结果已记录；Cargo 最新代码重跑受 registry/network 阻断。 |
| 评审主持 | Mavis review — CodeRabbit 未运行、workspace-wide check 未运行及 LNK1104 首次失败均如实披露。 |
| PM | Mavis review — INFRA-2A 只增加 observation slice；host manager、执行闭环和其他 phase 仍开放。 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录固定只读 inventory adapter、bounded parser/process、9/9 focused tests、文档同步、CodeRabbit/workspace/live VM 未验证边界和两实例测试 cap。 | INFRA-2A implementation 与 final self-review。 |
| v0.2 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据 Multipass 官方 machine-readable name→record-array 示例修正 parser，增加官方 map 与重复 JSON key fixtures；当前源码 11/11 direct rustc tests、clippy-driver 与 rustfmt 通过。明确 Cargo 当前重跑受离线缓存缺包和在线 TLS 失败阻断，旧 9/9 Cargo 结果只适用于修正前代码。 | self-review 对照 Multipass CLI reference 发现 JSON mapping compatibility defect。 |
| v0.3 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 semaphore 从每 probe 对象提升为进程共享，拒绝多对象并发启动 CLI；增加无子进程的全局 single-flight fixture，当前源码 12/12 direct rustc tests、clippy-driver 与 rustfmt 通过。 | self-review 对照文档发现对象级 semaphore 无法保证进程级单飞。 |
