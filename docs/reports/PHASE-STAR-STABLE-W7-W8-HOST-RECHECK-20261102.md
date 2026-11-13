# PHASE-STAR-STABLE-W7-W8-HOST-RECHECK-20261102

> **文档同步审查：ACCEPT。产品门禁：W7 OPEN / BLOCKED，W8 OPEN，稳定版 v1.0.6 NO-GO。** 本次补充了当前宿主与真实过滤上下文的窄范围证据；没有构建或发布镜像，也没有操作 Multipass 实例。

## §0 目的与范围

固定审查基线为 `dev@001904298ca1f01d889beb5809e67e57853565df`。审查目标是核对 W7 本地 K3s context 与 W8 Multipass/K3s 容量相关的现有报告、WBS 状态及本轮新证据，并为 WBS 更新提供独立结论。当前基线的目标 checkout 在观察时干净。

本 lane 只新增本报告和对应审查文件。没有修改 WBS、源码、`D:\Star`、`main`、其他 worktree 或 Multipass inventory；没有创建、启动、停止、删除、恢复或 purge VM，也没有 build、deploy、push、tag 或 release。

## §1 证据矩阵

| 项目 | 当前证据 | 结论与边界 |
|---|---|---|
| W7 focused tests | `py -3 -B -m unittest -v scripts.automation.test_star_k3s_context`：18 项，17 passed、1 skipped。跳过项因 Windows 拒绝创建 symlink（WinError 1314）。测试使用受控 Cargo metadata fixture。 | 验证过滤规则与失败关闭逻辑；不证明真实 Cargo 闭包、镜像内容或 guest 部署。 |
| W3 Canvas regression | `py -3 -B -m unittest scripts.release.tests.test_star_canvas_context_policy`：35 passed。 | 是独立的 Canvas context policy 覆盖，不是 `deploy/k3s-local` 实测证据。 |
| 锁定 toolchain 的真实 context | 在固定 dev 基线上，generator 使用 Rust/Cargo 1.99.0 时退出 2；Cargo metadata 退出 101，根因输出为 `rustc -vV` 启动失败（`1.99.0-x86_64-pc-windows-msvc` 下的 `rustc.exe` 不适用）。未生成 context 目录。 | 项目声明的 Rust 1.99.0 尚未通过真实 context gate。该结果不是 Cargo.lock 语义失败。 |
| 诊断版本真实 context | 协调者随后在同一 dev 基线以进程级 `RUSTUP_TOOLCHAIN=1.99.1` 实测 generator：exit 0，333 files、6,170,236 bytes；SHA-256 `e8316e07d4a7e5851f2cc94dde9792b2306ca7f6bf4276c4b705cb5b5b012fdc`，`forbidden_paths=[]`，生成 Dockerfile 不含 `COPY crates ./crates`。临时目录已由 `TemporaryDirectory` 自动清理。 | 这是 1.99.1 的真实过滤 context 证据；不替代 1.99.0，不证明 image build、最终 layers 或部署。摘要来自协调者本轮补充，review worker 未独立复跑；临时产物已清理，当前无法重算摘要。 |
| context 生成器与 Dockerfile 静态结构 | `deploy/k3s-local/Dockerfile` 是带 `@star-local-crates` 标记的模板；生成器从 Cargo 元数据选择本地闭包、生成过滤后的 workspace 与 per-crate COPY，并拒绝生成产物中的 broad `COPY crates ./crates` 和 game 标识。 | 静态实现与成功的 1.99.1 context 结果相符；不证明最终 OCI 内容。 |
| Multipass 宿主快照 | 只读查询显示 Multipass / multipassd 为 `1.16.4+win`。inventory 为 1 条：Stopped、Ubuntu 26.04 LTS、无 IPv4；占用户测试上限 2 个名额中的 1 个。本轮最新剩余空间快照：C `61,041,459,200` bytes，D `116,321,267,712` bytes。 | 单一停止 guest 与磁盘快照不构成可用 K3s 集群或双实例容量证明。没有查询该 guest 的 K3s 状态。 |
| W8 启动器静态门禁 | `scripts/automation/multipass_k3s_gitops.ps1` 声明最低 Multipass `1.17.0`、HCS、Ubuntu `24.04` 和测试上限 2；脚本锁涵盖 inventory 检查与 launch；容量统计包括 Stopped/Deleted，Deleted 目标不自动恢复或 purge。 | 当前宿主 `1.16.4+win` 低于脚本要求；该代码检查不是并发/失效注入实测。 |
| W8 文档一致性 | `deploy/multipass/README.md` 仍含 `1.17.0-rc1` 安装与历史验证说明。 | 与脚本的稳定版本门槛尚未统一，保留为开放文档缺口。 |

## §2 验证摘要

| 验证类型 | 命令或检查 | 结果 |
|---|---|---|
| Focused tests | `py -3 -B -m unittest -v scripts.automation.test_star_k3s_context` | 18 项；17 passed、1 skipped（Windows symlink 权限）。 |
| Canvas policy tests | `py -3 -B -m unittest scripts.release.tests.test_star_canvas_context_policy` | 35 passed；仅适用于 Canvas context policy。 |
| 真实 filtered context，锁定版本 | `star_k3s_context.py --repo <dev> --output <unique temp>`，Rust/Cargo 1.99.0 | exit 2；Cargo metadata 101；未生成 context。 |
| 真实 filtered context，诊断版本 | 同一 generator，进程级 `RUSTUP_TOOLCHAIN=1.99.1` | 协调者记录 exit 0、333 files、6,170,236 bytes 和上述摘要；不是 1.99.0 或 image build 验收。 |
| 宿主只读检查 | `multipass version`、`multipass list --format json`、`Get-Volume` | Multipass 1.16.4+win；一条 Stopped Ubuntu 26.04 inventory；记录空间快照。无 VM 生命周期操作。 |
| 静态检查 | Dockerfile template、context generator、Multipass launcher、`deploy/multipass/README.md` | 找到 per-crate 过滤路径、2-instance cap 静态实现和 README RC 版本差异。 |

未运行 Rust 1.99.0 成功 context、Cargo build/test、Docker build、image archive/layer 检查、Multipass guest/K3s readiness、双实例并发与容量测试、rollout/rollback、SBOM/provenance、签名或独立 consumer 验证。

## §3 已知缺口

1. 项目基准 Rust 1.99.0 在本机无法启动 rustc，真实 context gate 仍未通过；1.99.1 诊断结果不能替代固定基准。
2. 没有 Docker build 或最终 image digest/layers 证据。成功生成过滤 context 不能单独证明所有发行制品只含 Star 载荷。
3. 宿主 Multipass 1.16.4+win 低于脚本最低版本 1.17.0，HCS driver 未在本轮验证。
4. inventory 只有一个 Stopped Ubuntu 26.04 记录；脚本新建目标为 Ubuntu 24.04，K3s/CRI 状态未知。
5. 两实例上限、Stopped/Deleted 计数与锁当前只有静态实现证据，没有并发、异常 inventory 或容量实测。
6. `deploy/multipass/README.md` 的 1.17.0-rc1 内容未与稳定版本门槛同步。
7. 尚无实际 rollout/rollback、SBOM/provenance、独立 consumer、Star 原生 Host/cluster reservations 与 Run binding 的验收证据。

因此建议 WBS 仅追加本轮证据并保持 W7 OPEN / BLOCKED、W8 OPEN、v1.0.6 NO-GO。该文档 lane 不关闭任何产品发布门。

## §4 子代理失败接手清单

- 本 lane 没有派生子代理或待接手产出。
- 1.99.1 成功 context 的运行摘要由协调者补充；本 review worker 没有保留或独立重算临时目录摘要。WBS 应注明证据来源，不能把它写成 1.99.0 结果。
- 没有接管或更改 Multipass guest、D2 owner README、WBS 或其他 worktree。

## §5 守门规则

1. `dev@001904298ca1f01d889beb5809e67e57853565df` 是本轮固定基线；不得把新鲜度未验证的来源描述为当前状态。
2. Rust 1.99.0 是项目声明的验证基准，1.99.1 仅标为诊断版本。
3. 1.99.0 context 未生成；不得写成通过或 Cargo.lock 内容错误。
4. 1.99.1 的成功 context 必须保留文件数、字节数、摘要和其诊断性质。
5. 临时 context 已清理；不得声称当前还可检查其文件或 OCI 镜像层。
6. fixture 测试不得等价为真实 Cargo 闭包或生产构建证据。
7. Canvas policy 测试不得等价为 k3s-local 路径验收。
8. Dockerfile 是生成模板；不得把模板本身说成已经 build 的镜像。
9. 最终 image layers、digest、SBOM、provenance 和 consumer 证据仍为缺口。
10. Multipass 版本与 inventory 是只读宿主快照，不是 guest 或 K3s health 结果。
11. 当前单个 Stopped guest 计入用户最多保留 2 个实例的测试上限。
12. 未验证 HCS driver、Ubuntu 24.04 guest、CRI 或 K3s readiness。
13. 双实例 cap 与文件锁只有静态实现检查，没有并发压力或异常路径测试。
14. Stopped/Deleted 记录计数规则必须保留，且不得自动 purge。
15. `deploy/multipass/README.md` 的 RC 文案仍为待解决差异。
16. 本轮没有执行 VM 生命周期操作、build、deploy、push、tag 或 release。
17. W7 保持 OPEN / BLOCKED，W8 保持 OPEN，稳定版 v1.0.6 保持 NO-GO。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | Mavis 接手审核：宿主、guest、K3s 与 Star-native owner 边界保持分离；没有把 VM Ready 当作 Run admission。 |
| SRE Lead | Mavis 接手审核：真实 1.99.0 context、镜像、K3s、容量、rollout/rollback 证据仍缺。 |
| 平台 | Mavis 接手审核：接受将 1.99.1 实测作为诊断证据；必须继续单独记录 1.99.0 失败与 README RC 差异。 |
| 评审主持 | Mavis 接手审核：接受本次文档同步；W7/W8 产品门保持开放。 |
| PM / Release owner | 用户是 accountable release owner；本报告不批准 tag、release 或发布。当前 v1.0.6 NO-GO。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 合并核对 W7/W8 的测试、1.99.0 失败、1.99.1 诊断 context、Multipass 快照与 README 版本差异；保留开放门禁。 | 稳定版 WBS 宿主/context 复核 brief 与协调者补充实测结果 |
