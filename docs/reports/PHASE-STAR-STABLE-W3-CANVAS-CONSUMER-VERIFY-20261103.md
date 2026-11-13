# PHASE-STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103

## §0 目的

为 Canvas 稳定发布链增加独立消费者验证：manifest job 在下载本次 workflow run 的 resolver 身份和三个不可变 OCI digest 记录后，逐镜像验证 provenance attestation，全部成功后才渲染并上传 digest 固定的 K3s manifest。

本改动实现可执行的 consumer verification 步骤，不代表任何 hosted workflow、镜像构建、GHCR 发布、Multipass/K3s 部署或消费者验收已经运行。W3 继续 OPEN/BLOCKED，稳定版 v1.0.6 继续 NO-GO。

## §1 改动矩阵

| 文件 | 改动 |
|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | 在 manifest-rendering job 增加只读权限：`contents: read`、`attestations: read`、`packages: read`。下载 resolver 身份和三个 digest records 后，先通过现有 `render_star_canvas_manifest.py` 的验证函数核对 record 完整性及 tag/version/source SHA，再调用 `scripts/release/verify_attested_release.py` 分别验证三个 `oci://` digest subjects，随后才渲染和上传 manifest。将工作流当前关系记录到最小 Cypher header，并刷新 source digest。 |
| `docs/briefs/STAR-STABLE-W3-CANVAS-CONSUMER-VERIFY-20261103.md` | 本 lane 的协调 brief。 |
| 本阶段报告 | 记录当前实现、验证结果和未关闭门禁。 |

该 workflow 原来没有 Cypher block。本 lane 新增了最小关系 header，边只覆盖 YAML 明确读取的 run artifacts、显式调用的验证/渲染脚本，以及 job 的权限配置。

实现依据：[GitHub CLI attestation verify](https://cli.github.com/manual/gh_attestation_verify)、[GitHub Actions workflow permissions](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax) 与 [GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)。OCI verification 使用 `oci://` subject；消费 job 只授予读取 attestations 和 package 的权限，不配置 `id-token` 或写权限。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy git diff --check` | 通过。 |
| 工作流/YAML 解析器 | 未运行。 |
| 产品测试或自动化测试 | 未运行；本 lane 只改工作流配置和文档。 |
| GitHub-hosted workflow、GHCR artifact 与 `gh attestation verify` | 未运行；当前没有本次发布的镜像/attestation。 |
| Multipass/K3s rollout、consumer deploy/rollback | 未运行。 |

实现通过同一 workflow run 上传的 resolver/digest 记录绑定 tag、version、source SHA、服务名、镜像仓库与 digest。验证脚本进一步校验 signer workflow、tag source ref、source digest、SLSA predicate 与 OCI subject digest。任何记录缺失或校验失败都会终止 manifest job。

## §3 已知缺口

1. 未运行 GitHub-hosted workflow，因此没有实际 attestations API 查询或镜像消费者验证证据。
2. 没有真实 Canvas build、GHCR digest、最终 OCI image inspection、SBOM/provenance 独立消费结果或 K3s rollout 证据。
3. 这项验证不覆盖 Multipass inventory、Infrastructure reservation、Run binding、guest runtime readiness、部署、回滚或负载验证。
4. W1 Cargo workspace license 与用户选择的 MIT 仍不一致；其他稳定发行门禁也未关闭。v1.0.6 保持 NO-GO。
5. 本 job 仅独立验证 GitHub SLSA provenance；Buildx SBOM attestation 未在此处独立验证，仍是未关闭的稳定发行证据缺口。

## §4 子代理失败接手清单

本 lane 未派生下级代理，无 RPC 失败、未接收代码或需接手的工作。

## §5 守门规则

1. 只验证来自本次批准 tag workflow run 的 resolver 身份和 per-service immutable digest record。
2. 消费验证 job 仅有 `contents: read`、`attestations: read`、`packages: read`；`GH_TOKEN` 只提供给验证 step。
3. 在全部三个镜像的消费者验证成功前，不进入 manifest rendering/upload。
4. 验证 attestation 不等同于真实部署或运行时验收；不得据此关闭 W3/W7 或稳定版本门禁。
5. 本 lane 未修改 central WBS、main、远端或其他 worktree；未 push、创建 tag/release、发布镜像或部署。

## §6 签字栏

| 角色 | 状态 | 说明 |
|---|---|---|
| 架构 | 待协调者独立审阅 | 只在 digest-bound consumer job 中调用现有 verifier。 |
| SRE Lead | 待协调者独立审阅 | Hosted workflow、真实镜像和 rollout 证据仍缺。 |
| 平台 | 待协调者独立审阅 | Consumer job 使用显式只读权限。 |
| 评审主持 | 待协调者独立审阅 | 审阅记录由协调者负责。 |
| PM / Release Owner | Codex/assistant is accountable release owner | 用户已条件授权稳定发行；全部门禁与最终复核尚未满足。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-11-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Canvas OCI attestations 的独立消费者验证实现、只读权限、静态差异检查和未关闭的 hosted/runtime 门禁。 |
