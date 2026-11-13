# W7 Multipass 本地部署路径 Review — 2026-11-02

## 决定

**W7 代码与文档 lane 可作为静态实现候选；W7 发布门保持 OPEN / BLOCKED，v1.0.6 保持 NO-GO。** 本 review 不批准生产发布、Multipass/K3s 部署或发布载荷验收。

## Review 范围

- 基线：`dev@09b4eead99b047759be271ade2f5b06c0612ffb1`
- 代码：`deploy/k3s-local/build-and-deploy.sh`、`scripts/automation/star_k3s_context.py`、`scripts/automation/test_star_k3s_context.py`
- 文档：W7 brief、phase report、dispatch JSONL、此 review、`WBS-STAR-STABLE-RELEASE-20261031.md`
- 约束：显式既有 Multipass guest、无 Docker Desktop/宿主 K3s 依赖、使用 Star-only context、保留最多 2 个实例规则、稳定版发行门禁不误报。

## 发现与处理

| 等级 | 发现 | 处理/状态 |
|---|---|---|
| P2（已修复） | 首轮实现只检查 `kubectl get nodes` 返回非空；存在 NotReady 节点时仍可能开始 build/apply。 | `_require_guest_prerequisites` 现解析 nodes JSON，并要求每个节点的 `Ready` condition 为 `True`；新增 NotReady 在任何 transfer 前失败的回归测试。复跑完整聚焦测试通过。 |
| P2（已修复） | Windows 工作副本 CRLF 导致 Bash `-n` 报 EOF。 | 以 LF 规范化后重跑 `bash -n` 退出码 0；Cypher guard stamp 随后再次检查通过。 |
| — | 独立 standards review 未发现其他有证据的代码缺陷。 | 保留真实 guest、Cargo、image 与 release evidence 的限制；不把静态测试提升为发布证明。 |

## 需求对照

| 需求 | 结论 | 证据边界 |
|---|---|---|
| 必须选定一个已运行 Multipass guest，并在 guest 内完成 build/import/apply | 静态实现符合 | unittest 检查所有 Multipass exec/streaming 命令均指向所选实例；没有真实 guest 执行。 |
| 只使用经过验证的 Star Cargo 闭包，失败时不构建 | 代码路径符合；实际生成未验证 | 测试通过 fixture 投影；Cargo metadata 受本机 Rust 1.99.0 rustc 不可用阻断。 |
| Guest 和 K3s 节点未就绪时 fail closed | 符合 | 测试覆盖 guest prereq 不可用、无/停止实例和节点 NotReady，阻止后续传输。 |
| 不使用 host Docker/K3s、不管理 Multipass 生命周期 | 符合静态检查 | 测试检查部署脚本不包含 host build/deploy 命令；没有 runtime trace。 |
| 对外发布不混入 game/legacy，且最终 image 经过验收 | 未证明 | 当前 helper 仍没有真实 context archive、image layer inventory、digest、SBOM/provenance 或 consumer verification。 |
| 完成 K3s、rollback 和 Star-native infrastructure 闭环 | 未证明 | 没有真实 Multipass/K3s rollout/rollback；没有 Star-native reservations、cluster binding 和 admission。 |

## 检查结果

| 检查 | 结果 |
|---|---|
| `rtk proxy py -3 -m unittest scripts.automation.test_star_k3s_context -v` | PASS — 18 项，17 passed、1 skipped（Windows symlink privilege） |
| `rtk proxy py -3 -m py_compile scripts/automation/star_k3s_context.py scripts/automation/test_star_k3s_context.py` | PASS |
| `rtk proxy bash -n deploy/k3s-local/build-and-deploy.sh` | PASS — final LF working copy |
| `rtk git diff --check` | PASS |
| CypherGraph Guardian `--changed --stamp` 与 `--changed` | PASS — 3 source files |
| CodeRabbit `review --agent -t uncommitted` | NOT AVAILABLE — CLI shim resolves to `/root/.local/bin/coderabbit`, which returned permission denied; independent standards/spec reviews and direct local review were completed. |
| Rust 1.99.0 production Cargo metadata | BLOCKED — rustc.exe 对 `1.99.0-x86_64-pc-windows-msvc` 不适用，Cargo metadata exit 101 |
| Live Multipass/Docker/K3s/image/digest/SBOM/rollback | NOT RUN |

## 剩余风险

- 真实 Cargo 闭包、guest 内 image build、containerd/CRI 可见性、部署、消费者验证和 rollback 均未实证。
- 本地节点 Ready 检查不提供 Star 原生 Infrastructure 预算、owner reservations、cluster generation binding 或任务执行授权。
- `deploy/k3s-local/README.md` 是 D2 owner 的未提交草稿，保留未动；其与当前 helper 的操作说明需要该 owner 后续协调。
- W7 仍 OPEN / BLOCKED；Stable v1.0.6 仍 NO-GO。
