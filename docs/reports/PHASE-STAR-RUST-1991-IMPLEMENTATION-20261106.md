# PHASE-STAR-RUST-1991-IMPLEMENTATION-20261106

> **状态：** 本地实现与 focused verification 完成，等待独立复审及集成。此切片不关闭 W2、WLD 或任何稳定发行门。
> **基线：** `dev@655f675ad6381ba5838fa37f191d5329a7b2a463`
> **工作分支：** `codex/rust-1991-harness-baseline-20261106`

## 0. 目的

按用户 2026-11-06 的决定，将 Rust 1.99.1 设为当前最低版本。仅同步登记的验收脚本、测试夹具、scaffold 输出和 WBS；根 workspace、CI、容器等现有 1.99.1 基线值已存在于本 lane 基线，本次没有修改这些文件。

## 1. 改动矩阵

| 范围 | 改动 | 状态 |
|---|---|---|
| Acceptance runners | phase9f2 与 W16-A Canvas/OAuth 的显式 Cargo selector 改为 1.99.1；Canvas runner 精确校验 rustc 1.99.1 build identity/command；device-revocation runner 的 default 与 minimum 设为 1.99.1 | 已实现 |
| Minimum guard test | 明确断言旧 1.99.0 低于新 minimum；保留 1.99.0 错选 compiler fixture 以验证 exact-version mismatch | 已实现 |
| Test fixtures | k3s-local Docker/toolchain、Canvas Cargo/toolchain fixtures 对齐 1.99.1 | 已实现 |
| Scaffold | 生成 workspace 的 `rust-version` 改为 1.99.1 | 已实现 |
| WBS | 更新当前 WLD/VCD/Rust gates；保留历史 1.99.0 结果、修正稳定版 W2/W7 前瞻门槛并记录规范文档同步缺口 | 已更新 |

本 lane 未修改 Rust crate 源码、根 Cargo metadata、CI workflow、Dockerfile、AGENTS.md、requirements/basic/detail design、main 或 remote。

## 2. 验证摘要

| 检查 | 结果 |
|---|---|
| `test_w16a_device_revocation_http_pg_acceptance` | 16/16 passed |
| `test_w16a_canvas_sse_pg_acceptance` | 25/25 passed |
| `test_w16a_oauth_pg_acceptance` | 5/5 passed |
| `test_phase9f2_binding_http_acceptance` | 11/11 passed |
| `test_star_k3s_context` | 19 passed, 1 skipped；skip 为当前 Windows 进程无创建 symbolic link 权限（WinError 1314） |
| `test_star_canvas_context_policy` | 35/35 passed |
| Python compile check | 8 个变更 Python 文件全部通过 `py_compile` |
| Scaffold generation | 在 lane worktree 内临时目录安全运行 generator，28 个 workspace crates 输出的根 manifest 为 `rust-version = "1.99.1"`；临时目录自动清除 |
| Rust toolchain identity | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` 与 `cargo 1.99.1 (5f94df478 2026-08-27)` 可执行 |
| Legacy toolchain availability | `rustup toolchain list` 显示 1.99.0 安装记录，但 `rustc +1.99.0 -V` 失败：rustc.exe 不适用于该 toolchain；旧 1.99.0 focused Rust gates 无法运行 |
| Diff whitespace | `git diff --check` 通过 |

六个 Python suites 共 112 项：111 passed、1 skipped、0 failed。未运行 Cargo 编译/测试，因为本 lane 只改 Python harness、test fixture 与生成模板，不改 Rust crate；也未运行 PostgreSQL、target DB/RLS、完整 workspace、fmt/Clippy、hosted CI、Vercel、Multipass/K3s 或发布验收。不可运行的 1.99.0 toolchain 不得记作旧基线验证通过。

## 3. 已知缺口

1. 当前用户方向已定为 1.99.1，但受限 manifest 之外仍有规范文档未同步：`docs/requirements.md` AC-INFRA-002-R/S/U/V/W、`docs/basic-design.md` §16.44–§16.47、`docs/design/DD-WORKTREE-GROUP-001.md` AC-INFRA-002-U/V，以及根 `AGENTS.md` 的 1.99.0 要求。本 lane 没有修改这些文件；必须另行登记文档 lane。不得将这些引用一概标作历史结果。
2. 1.99.1 的全 workspace/all-targets/fmt/Clippy 与每个发行目标的依赖/MSRV、PG/RLS 和 runtime gates 仍未验证；W2 与 v1.0.6 保持 OPEN/NO-GO。
3. 第一次执行 `rtk python tools/scaffold_skeleton.py --help` 时，脚本没有 help parser，实际运行了 generator，并按其硬编码 ROOT 报告写入 `D:\Star-worktrees\dev-skeleton`。stdout 列出 28 个 crate 的 Cargo/lib 输出和 workspace Cargo.toml。该目录可能已有数据；本 worker 未继续读取、覆盖、清理或删除该目录。协调者已获知并要求将该路径视作保留的 lane 外数据。后续 scaffold 验证通过只读源码确认 ROOT 可覆盖，再把生成写入注册 worktree 内的临时目录并自动清除。
4. Independent reviewer 的代码复核与 dev 集成尚未完成；本阶段报告不代表独立审批或产品准备就绪。

## 4. 子代理失败接手清单

- 本实现 worker 未派生额外子代理；协调者另有独立 review lane。
- Review lane 预审发现 active minimum tuple 仍为 1.99.0、测试未拒绝 1.99.0。实现已修正 tuple 为 `(1, 99, 1)` 并增加旧基线拒绝断言；保留 wrong-selected-compiler fixture。请独立复核此修复及受限文件清单。
- 没有其他 worker RPC 或执行失败需要接手。

## 5. 守门规则

- 只集成经独立复核、目标基线复核和验证接受的本 lane 提交；不得整分支盲合。
- 保留历史测试结果；不得把 1.99.0 的旧结果伪装成 1.99.1 验证。
- 不得把单测、静态 scaffold 检查或 toolchain version probe 标为 workspace/runtime/生产验收。
- 1.99.0 已安装记录不可启动时，将对应旧 gate 明确记为 unavailable/NOT_RUN。
- WBS、SRS/BD/DD 与 AGENTS 的不一致必须作为开放文档缺口追踪；未登记的路径不在本 lane 修改。
- `D:\Star-worktrees\dev-skeleton` 为本轮意外写入的 lane 外目录，保留，不清理、不挪动。
- 不触碰用户脏工作树、main、远端或其它 lane；本报告无 push、tag、release 或 deploy 结果。

## 6. 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | 本切片仅同步 1.99.1 验收 harness/fixtures/scaffold，不宣称 Rust workspace 或产品闭环完成；待协调者复核 |
| SRE Lead | 目标 Rust build、PG/RLS、恢复、运行和性能 gates 未执行；保持 OPEN |
| Platform | CI/Docker 基线值已在基线中；本 lane 未验证 hosted/Multipass/K3s 环境 |
| 评审主持 | 实现 worker 自查完成；独立 review lane 尚待最终结论 |
| PM | 按用户当前指定 Rust 1.99.1 作为基线；stable release 保持 NO-GO |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 终审发现稳定版 WBS 的 W2/MSRV 边界与 W7 下一步 Cargo 验收仍前瞻引用 1.99.0；已改为 1.99.1，未改写历史 run 结果，并明确 requirements/basic/DD/AGENTS.md 是 manifest 外的规范同步缺口。 | 协调者终审反馈 |
| v0.1 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 1.99.1 harness 对齐、focused 测试、1.99.0 toolchain 失效、规范文档缺口和 scaffold 输出意外；不关闭产品/发布门 | 用户确定 Rust 1.99.1 为当前版本并授权实施 |
