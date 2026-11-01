# PHASE — STAR Stable W3 Artifact Evidence Audit

> **状态：** 审计完成；W3 仍 OPEN / BLOCKED，稳定发行不可发布。
> **工作基线：** `dev@2a57fa0d329be830735d6657677236e195673362`。
> **范围：** 仅审查已确认的 Desktop、Web/Vercel 和三份 Star Canvas 服务镜像的工作流与已接受阶段报告；不运行或触发 hosted release 工作流。

## §0 目的

核对当前静态配置是否为 W3 所要求的实际发行制品和依赖闭包建立 BOM、摘要绑定、provenance/attestation 与独立消费验证路径，并区分配置证据和真实执行证据。本阶段不修改 workflow、制品、WBS 或外部发布设置。

## §1 改动矩阵 / 证据矩阵

| 证据面 | 源文件与定位 | 当前证据 | 审计结论 |
|---|---|---|---|
| Desktop Linux / Windows / macOS | `.github/workflows/star-stable-desktop.yml:141-161,221-241,302-322,399-420,450-465,495-510`；`docs/reports/PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md:23-53` | 三平台分别生成 Rust 与 npm SPDX 文件、运行字段/非空校验、上传 bundle；tag dispatch 的证明 job 将平台制品路径和 SBOM 作为 attestation subject。阶段报告记载 YAML/静态断言与 npm SBOM 控制验证，不记载 hosted 执行。 | **静态路径已配置，执行证据缺失。** Rust SBOM 的本地生成和传递依赖闭包未验证；证明 job 还受 MIT metadata gate 阻断。 |
| Web / Vercel | `.github/workflows/star-stable-web.yml:5-26,71-85,114-144`；`docs/reports/PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md:16-22,28-44` | 手动工作流生成 npm SPDX、构建证据归档、SHA-256 文件，并为归档配置 provenance 和 SBOM attestation；摘要中提供消费者验证命令。 | **这是 Web build-evidence bundle，不是 Vercel 部署包或部署记录。** 没有 hosted run、实际 Vercel 部署或消费者验证。 |
| 三份 Canvas 服务镜像 | `.github/workflows/publish-star-canvas-services.yml:9-22,46-56,273-312`；同一 W2 报告 `:16,22,30-44` | 显式批准的版本标签手动 dispatch job 配置 Buildx `sbom: true`、`provenance: mode=max`，校验 Buildx 返回的 SHA-256 image digest，并将该 digest 交给 attestation action；摘要中给出 OCI consumer 验证命令。 | **镜像 digest 的静态绑定路径已配置，真实镜像与真实证明未生成。** 没有 Buildx 发布、实际 OCI digest、K3s 消费或 `gh attestation verify` 输出。 |
| W3 完成定义 | `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md:27` | 逐发行面及其实际依赖闭包要求 schema-valid BOM、artifact SHA-256/image digest、build provenance/签名/attestation 和独立 consumer 验证；同一 release ID 要能串起制品、BOM、attestation bundle 与验证记录。 | 当前报告与工作流没有组成上述端到端证据链，W3 不可关闭。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 审计基线 | PASS：本 worktree HEAD 为 `2a57fa0d329be830735d6657677236e195673362`。 |
| Desktop workflow 与已接受阶段报告 | PASS：确认了三平台静态 SBOM/attestation 路径，并与报告中的本地和未执行边界逐项对照。 |
| Web/Canvas workflow 与已接受阶段报告 | PASS：确认 artifact/image subject 的静态绑定以及明确的消费侧验证指令；报告明确记载 hosted run、部署及验证均未运行。 |
| 真实运行证据 | NOT AVAILABLE：本审计输入和本地仓库报告中没有可核验的 workflow run URL/ID、实际 release artifact、image digest、attestation bundle 或 consumer 验证输出。本阶段未查询 GitHub 远端。 |
| 本阶段运行构建、SBOM 生成、发布、部署或测试 | NOT RUN：此 lane 是只读证据审计，不把先前报告的静态检查冒充为本阶段重跑结果。 |

## §3 已知缺口

1. **Desktop：** hosted Linux/Windows/macOS build、打包制品、SBOM 生成/下载、provenance 与 SBOM attestation、消费者验证均没有运行证据。阶段报告说明 Rust 依赖枚举被缺失的可用 `rustc` 阻断，且 validator 的 Rust package 下限仅能证明根包存在，不能证明完整依赖闭包（Desktop 阶段报告 §2-§4，尤其 §4 第 10 项）。签名/公证及实际安装包 hash 也未验证。
2. **Web/Vercel：** npm SBOM 覆盖范围不是实际部署闭包的完整证明；归档明确是 build-evidence bundle。没有真实 Vercel project build/deployment、部署产物 digest、rollback 或消费侧验证。外部 `star-stable-release` reviewer/ref 规则未核验。
3. **Canvas：** 三个镜像的 Buildx SBOM/provenance 与 digest-bound attestation 只存在于静态工作流配置。没有发布出的镜像、不可变 digest、独立 OCI attestation 验证或 Multipass/K3s 部署证据。
4. **法务与依赖闭包仍阻断证明：** W1/W2 尚未关闭。Desktop workflow 的 MIT metadata gate 会在根 Cargo metadata 与 MIT 选择冲突时拒绝证明；全量依赖、NOTICE 和各实际制品 payload 的权属/许可审查不能由 SPDX/SBOM 取代。
5. **证明语义：** 当前工作流配置的是 GitHub provenance/SBOM attestations；本审计没有证明任何制品已被签发，也不把 attestation 等同于代码签名、公证、安装包签名或已通过消费者验证。

## §4 子代理失败接手清单

本 lane 为单人只读审计，没有派发子代理；本阶段没有子代理失败或接手产物。W3 的后续接手者须等 W0-W2 所需的版本、法律与依赖闭包证据落实后，运行经批准的目标工作流，保留每个平台的不可变制品摘要、SBOM、attestation bundle、run URL/ID 和消费者验证输出。

## §5 守门规则

1. 仅静态审查三个当前 workflow 和已接受的 lane 报告；未检查/未记录的 GitHub 设置保持未知。
2. Desktop、Web/Vercel 与三份 Canvas 镜像仍是本轮发行面；Multipass guest image 不是公开制品。
3. workflow 中存在 attest action、environment 名称、Buildx SBOM/provenance 开关或验证命令，不代表 hosted 运行、保护规则生效、制品存在或消费者验证成功。
4. W1/W2 的许可证、NOTICE 与实际依赖闭包未过门时，不运行证明型 release job。
5. 本审计不触发构建、签名、attestation、发布、部署，不创建 tag/release，也不 push。
6. W3 与整体稳定发行保持 OPEN / 不可发布；本报告不代替最终 release 审批。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）：静态证据和真实执行证据已分开；W3 不关闭。 |
| SRE Lead | 本阶段不适用：未执行目标平台部署、恢复或运行验收。 |
| 平台 | 本阶段不适用：未运行 hosted workflow、产物构建或签名/attestation。 |
| 评审主持 | 文档范围审阅：证明 subject 需与每个发行制品摘要绑定，consumer 输出仍缺。 |
| PM | 用户批准的发行范围保持不变；稳定发行仍 OPEN，未批准发布。 |

**发布审批：** 未申请；本阶段文档审核不代表 stable release 批准。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W3 bounded evidence audit；记录三发行面的静态 BOM/attestation 路径、实际 hosted/consumer evidence 缺口，并保持 W3 OPEN。 | 用户要求按 WBS 并行推进稳定发行，且禁止将配置冒充可发布证据。 |
