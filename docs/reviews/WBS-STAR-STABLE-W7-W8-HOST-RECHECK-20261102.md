# WBS-STAR-STABLE-W7-W8-HOST-RECHECK-20261102 — 独立审查

## 审查决定

**ACCEPT：接受本次 WBS 证据同步。** 接受范围仅包括把新证据和仍开放的缺口记录到 WBS；不代表接受 W7/W8 的产品验收，也不授权稳定版发布。W7 应维持 **OPEN / BLOCKED**，W8 维持 **OPEN**，稳定版 v1.0.6 维持 **NO-GO**。

## 固定基线与范围

- 审查基线：`dev@001904298ca1f01d889beb5809e67e57853565df`；目标 checkout 观察时干净。
- brief 声明的 1.99.0 真实 context 失败与本轮后续 1.99.1 context 成功并不互相替代：前者未产出 context；后者是不同 toolchain 的诊断结果。
- 本审查只改本 lane 的阶段报告和审查文件。WBS 由协调者更新；没有操作源代码、VM、main、`D:\Star` 或其他 worktree。

## 核验结论

| 事项 | 审查判断 | WBS 同步要求 |
|---|---|---|
| W7 单元测试 | 18 项中 17 通过、1 项因 Windows symlink 权限跳过。fixture 使用受控 Cargo metadata。 | 可记录为聚焦策略测试通过；不可写成真实 Cargo 闭包或构建通过。 |
| Canvas policy 回归 | 35 项通过，但测试覆盖的是 Canvas context policy。 | 单独记录，禁止作为 k3s-local 验收证据。 |
| Rust 1.99.0 | Cargo metadata 因 `rustc -vV` 启动失败而退出；未生成 filtered context。 | W7 真实基准仍 BLOCKED；不要将失败归因为锁文件语义错误。 |
| Rust 1.99.1 | 协调者补充了真实 generator exit 0、333 files、6,170,236 bytes、SHA-256 `e8316e07d4a7e5851f2cc94dde9792b2306ca7f6bf4276c4b705cb5b5b012fdc`、无禁用路径且无 broad COPY 的证据；临时目录已清理。review worker 未独立复跑。 | 加入为 1.99.1 诊断证据，并标注来源；不得替代 Rust 1.99.0、镜像 build 或最终载荷验证。 |
| Multipass 宿主 | 当前只读快照为 1.16.4+win、一条 Stopped Ubuntu 26.04 记录；测试上限为 2。 | 不能写成 K3s ready 或双实例容量通过；该主机版本低于脚本 1.17.0 门槛。 |
| 启动器门禁 | 脚本静态声明 1.17.0/HCS、Ubuntu 24.04、双实例上限与独占锁，并计数 Stopped/Deleted。 | 只能记静态实现；并发锁、坏 inventory、配额与恢复仍待实测。 |
| Multipass README | 仍保留 1.17.0-rc1 安装/验证文案。 | 保留为文档差异；协调者不要因本次只读审查直接改该文件。 |

## 应保留的未完成项

1. Rust 1.99.0 能在目标 Windows 环境执行 `rustc -vV`，并成功生成已审计的 filtered context。
2. 对准确生成 context 执行 build，核查镜像内容、digest/layers、SBOM、provenance、签名与独立 consumer。
3. 满足 Multipass 版本和 HCS 要求，验证 Ubuntu 24.04 guest、K3s API、节点/CRI readiness。
4. 实测两实例配额、Stopped/Deleted 统计、坏 inventory fail-closed、并发锁及磁盘容量；保留用户最多 2 个实例的限制，不自动 purge。
5. 完成真实 guest rollout、rollback、恢复与容量证据，并保持宿主 GitOps helper 不冒充 Star 原生 Host/cluster reservation、Run binding 或 Agent sandbox 验收。
6. 由持有者协调 `deploy/multipass/README.md` 的 RC 文案与稳定版本策略。

## 最终判定

- 文档同步：**ACCEPT**，但 WBS 只能增加证据和明确未完成项。
- W7：**HOLD / OPEN / BLOCKED**。
- W8：**HOLD / OPEN**。
- 稳定版 v1.0.6：**BLOCK / NO-GO**，直到全部适用发行门通过且 release owner 批准。

更多逐项证据见 [`PHASE-STAR-STABLE-W7-W8-HOST-RECHECK-20261102.md`](../reports/PHASE-STAR-STABLE-W7-W8-HOST-RECHECK-20261102.md)。
