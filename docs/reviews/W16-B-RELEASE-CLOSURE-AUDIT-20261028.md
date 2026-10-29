# W16-B 发布闭包审计

> **状态：OPEN；当前证据不足以批准商业发行。** 本文是有界只读审计，不代表任何发行制品已构建、签署或完成许可证放行。
>
> 基线：`dev@9e037f9e191dc7499e9e8d039e9966b348f83262`
> 审计分支：`codex/w16-b-release-closure-20261028`
> 日期：2026-10-28

## §0 目的与范围

核对仓库当前可见的发行入口、依赖闭包来源、Rust 基线、许可证与 NOTICE、SBOM 证据、桌面和容器制品，以及 Multipass/K3s 测试约束。用户设定的发行政策是：无限制商业用途、逐版本核查整个实际分发闭包；遵守各许可证的源码、版权、NOTICE、专利和安装信息义务；发布 SPDX/SBOM；核验每个外部组件近 12 个月的维护证据；测试阶段最多保留两个 Multipass inventory 实例；运行路径不依赖 Docker Desktop。

本审计只读取当前 worktree 与公开上游主来源。没有构建或测试，没有安装工具，没有访问环境变量或凭据，也没有检查远端 CI 运行结果、分支保护设置、实际发布文件、客户安装包、容器镜像层或集群内实际镜像清单。审计不修改实现、锁文件、工作流、WBS 或用户的 `D:\Star` 主 worktree。

## §1 审计矩阵与结论

| 范围 | 观察到的事实 | 结论 |
|---|---|---|
| Rust 基线 | 根 workspace 声明 Rust `1.99.0`，BFF 独立 workspace 也声明 `1.99.0`；`tools/aci-emitter` 独立 workspace 仍声明 `1.77`。根基线可在 [Cargo.toml](../../Cargo.toml#L184-L190)、[bff/Cargo.toml](../../bff/Cargo.toml#L21-L28)、[tools/aci-emitter/Cargo.toml](../../tools/aci-emitter/Cargo.toml#L1-L9) 复核。 | **阻断：** ACI workspace 与用户要求的 workspace Rust 基线不一致。当前未验证其真实交付边界及依赖 MSRV。 |
| Rust 锁定依赖规模 | 只读 `cargo +1.99.0 metadata --locked --offline --format-version 1` 输出了 110 个 workspace packages、859 个 resolved packages、749 个 registry packages。此为根 workspace 的默认 feature metadata，不是按制品/平台展开并签收的发行闭包。最后的辅助 Python 输出器在生成 workspace 路径时 `IndexError`；计数已输出，但该流水线最终退出码为 1。 | **未放行：** 不把这些计数当作完整依赖审计或可复现发行证明；BFF 与 ACI 的独立锁文件未分别解析。 |
| 商用许可策略 | [deny.toml](../../deny.toml#L21-L85) 是白名单策略，包含 GPL/AGPL/LGPL 和 MPL；这符合“copyleft 不等于禁止商用”的政策。根 CI 的 [license-gate.yml](../../.github/workflows/license-gate.yml#L37-L67) 固定 cargo-deny 版本及摘要并对根 workspace 执行全 feature 检查。 | **部分通过：** 策略方向合理，但通过白名单不等于逐制品法律审查，也不能代替实际 license 文本、源代码、链接/捆绑方式和履约材料核验。cargo-deny 文档也明确说明它不会穷尽扫描每个 crate 的全部源码，并指出 metadata 与实际许可证文本可能不一致。[cargo-deny license 文档](https://embarkstudios.github.io/cargo-deny/checks/licenses/index.html) |
| 项目自身许可证标识 | 根 `workspace.package` 声明 `Apache-2.0`；仓库根 [LICENSE](../../LICENSE#L1) 标题为 MIT License；受限文件清单中只看到根 `LICENSE` 与 `tools/aci-emitter/LICENSE`，没有根 `LICENSE-APACHE`。Helm [Chart.yaml](../../deploy/helm/star/Chart.yaml#L11-L13) 也标为 Apache-2.0。 | **阻断：** 项目/发行制品应使用的法律文本与 manifest/chart 标识需要由权利人明确并统一；这不是对侵权或无权发行的判定。 |
| 许可证清单与 NOTICE | [NOTICE.md](../../NOTICE.md#L1-L16) 自称由 `cargo-about` 生成，日期为 2026-10-05，并列出“1265 个 crate”；[about.toml](../../about.toml#L11-L18) 留有 2026-10-04 的“829 包闭包”注释。当前根 workspace metadata 输出 859 个 resolved package。三组统计来自不同命令/时间/计数单位，现有材料无法证明彼此可比或已对齐。 | **阻断：** 需要用相同锁文件、workspace、feature、目标平台和工具版本重新生成并解释差异；同时证明每种实际制品包含相应 NOTICE/LICENSE 文本。 |
| cargo-deny 导出性质 | CI 的 `cargo-deny list --format json` 被标为“SPDX evidence”，但该步骤与上传均 `continue-on-error: true`，文件名是 `licenses.json`。[cargo-deny 官方说明 `list` 输出的是各 crate 的许可证信息，支持 human/json/tsv](https://embarkstudios.github.io/cargo-deny/cli/list.html)，因此这项清单不是 SPDX SBOM。SPDX 自身是 ISO/IEC 5962:2021 国际标准。[SPDX 规范](https://spdx.dev/use/specifications/) | **阻断：** 在所检查的发布工作流中未发现与实际发行制品绑定、校验和固定的 SPDX/CycloneDX BOM。必须对每个制品生成并验证 SBOM；许可证清单可保留为补充证据。 |
| 供应链门禁 | [license-gate.yml](../../.github/workflows/license-gate.yml#L69-L85) 中许可证清单导出与 artifact 上传非阻断；同一工作流把 cargo-audit SHA 保留为 `REPLACE_WITH_UPSTREAM_SHA256_FOR_0.22.2`，并按脚本配置在未填写时失败关闭（见 `:91-124`）；cargo-machete job 是 advisory（`:127-143`）。 | **未验证：** 源码显示 advisory job 会失败关闭，但未查远端运行状态与分支保护 required-check；不得声称线上门禁已经通过或阻断发行。 |
| 桌面发行 | [star-desktop-build.yml](../../.github/workflows/star-desktop-build.yml#L65-L180) 生成 Linux `.deb`、Windows `.msi`、macOS `.dmg`。Linux 只复制到临时目录，上传步骤仍注释未启用；Windows/macOS 上传保留 30 天。构建脚本在 `crates/star-desktop/frontend` 执行 `npm install`，其目录没有 npm/pnpm/yarn 锁文件；Tauri 配置则要求执行 `npm run build`（[脚本](../../scripts/build-desktop.sh#L95-L120)、[Tauri 配置](../../crates/star-desktop/src-tauri/tauri.conf.json#L7-L12)）。Windows/macOS 构建固定 Node 20。 | **阻断：** 桌面前端安装不是 lockfile 驱动的可重现安装；Linux 制品没有已启用的 CI artifact 上传；三个平台都没有本审计确认的制品级 SBOM/签名。Node 20 于 2026-03-24 到达 EOL，官方说明 EOL 版本不再获得安全修复；当前桌面工作流仍使用 Node 20。[Node 官方版本状态](https://nodejs.org/en/about/previous-releases)、[官方 EOL 说明](https://nodejs.org/en/about/eol) |
| Web 前端 | 仓库同时保留 `frontend/package-lock.json` 与 `frontend/pnpm-lock.yaml`；常规 CI 使用 Node 22 + `npm ci`，集成工作流另用 pnpm frozen lockfile。未找到 `frontend/package.json` 的 `packageManager` 字段。 | **需收敛：** 明确 Web 发布目标、唯一权威锁文件/包管理器，并验证两个锁文件是否可再现且在发布前同步。Vercel 实际构建配置、发布产物、运行时配置和对应 SBOM 未在此轮确认。 |
| 容器与 Helm | [publish-canvas-game.yml](../../.github/workflows/publish-canvas-game.yml#L83-L130) 发布四个 GHCR 镜像；引用 `docker/build-push-action@v5`，配置 tag/labels/push，所检查 step 没有 SBOM/provenance 输出。 [本地 API Dockerfile](../../deploy/k3s-local/Dockerfile#L1-L10) 使用可变的 `rust:slim-bookworm` 与 `debian:bookworm-slim`，并通过 apt 安装 `pkg-config/libssl-dev/ca-certificates`。 | **阻断：** 容器运行库、操作系统包、镜像 digest、构建来源/签名及 BOM 没有与应用制品绑定的证据。Dockerfile 中的 `docker` 仅是容器格式/构建步骤的一部分；观察到的 GitHub Actions Buildx 不构成 Docker Desktop 前置要求。 |
| Multipass/K3s | 受控脚本将 `$MaxTestPhaseInstances = 2`，在 inventory 检查和启动期间持跨进程锁，计数包括已停止/已删除记录；无法验证 inventory 或超过上限会拒绝启动，并且不自动 purge。[脚本约束](../../scripts/automation/multipass_k3s_gitops.ps1#L168-L168)、[预留与检查](../../scripts/automation/multipass_k3s_gitops.ps1#L302-L373)。但 README 仍以“给 Windows + Docker Desktop 留余量”描述资源预算（[README](../../deploy/multipass/README.md#L104-L108)）。README 与脚本材料仍记录 Multipass 1.17.0-rc1；上游 1.17.0 已于 2026-10-19 发布。[Multipass 官方 releases](https://github.com/canonical/multipass/releases)；K3s 官方 releases 页面显示 2026-09-30 的制品及校验和。[K3s 官方 releases](https://github.com/k3s-io/k3s/releases) | **配额实现已由源码支持；版本和文档需收敛：** 两个测试实例上限满足当前约束。更新安装基线到已发布且校验过的 Multipass 版本，移除过时 Docker Desktop 资源假设；为 Multipass/K3s/Guest OS/Helm charts 与所有 add-on 分别核验许可证、摘要、维护状态和 BOM。 |
| 独立 Rust workspaces | 根 workspace、`bff/`、`tools/aci-emitter/` 各有独立 Cargo.lock；根 `license-gate.yml` 执行的是仓库根目录命令，当前没有该 workflow 对 BFF/ACI 独立 workspace 执行相同闭包门禁的步骤。 | **阻断：** 需要把实际独立构建/部署单元登记进目标矩阵，并对各自 lockfile 和所有发行目标单独执行依赖/许可证/漏洞/SBOM 盘点。 |

cargo-deny 的逐 crate 许可证列表是有价值的审查输入，但不是软件物料清单；cargo-about 的上游说明也将其描述为“依赖许可证清单生成器”，而非制品 SBOM 生成器。[cargo-about 项目说明](https://github.com/EmbarkStudios/cargo-about/blob/main/README.md)。可选择 SPDX 或另一个符合产品政策的 SBOM 标准，但必须明确 schema 版本、工具版本、实际平台/feature closure、制品摘要和验证结果，不能只生成 workspace 报表后当作镜像/安装包 SBOM。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| Worktree/分支 | 起始状态为指定分支 `codex/w16-b-release-closure-20261028`，HEAD 与目标基线一致：`9e037f9e191dc7499e9e8d039e9966b348f83262`。 |
| Cargo metadata | 只读 offline locked metadata 在约 12 秒返回；导出 110 workspace packages / 859 resolved packages / 749 registry packages。辅助 Python 在可选路径展示部分发生 `IndexError`，最终命令 exit 1；因此只记录已打印计数，不作完整闭包验证结论。未对 BFF/ACI 分别运行 metadata。 |
| 构建与测试 | 未运行；符合本 lane 限定。 |
| 发行流程 | 未启动任何 CI、发布、镜像 build/push、安装包签署或集群部署。 |
| 外部主来源 | 查阅 SPDX、cargo-deny、cargo-about、Node.js、Multipass、K3s 官方项目页面。Multipass 与 K3s 上述来源证明这两项近期有发布活动；它们不能代替对其余每个组件的 12 个月维护证据。 |

## §3 已知缺口

1. 尚无经产品权利人确认的发行清单：至少需按 Desktop 各平台、Web/Vercel、API/BFF、四个 GHCR 服务镜像、Helm/GitOps、WASM 以及其它真正交付的二进制/guest image 区分“发布”“仅 CI”与“开发工具”。
2. 没有逐目标平台、feature set、构建 profile、Rust/JS lockfile、OS 包、基础镜像、Helm chart/依赖、Multipass guest 与 K3s/插件镜像组成的端到端依赖闭包证据。
3. 没有针对确切发行制品、并以制品 SHA-256 关联的 SPDX/CycloneDX SBOM、签名、来源证明或最终 artifact inventory。已提交的 NOTICE 与 licenses JSON 不能证明发行物实际携带了相同依赖或法律文本。
4. 没有逐版本 license-file 与 source review、Apache 专利/NOTICE 履约检查、MPL/LGPL/GPL 传递义务分析、第三方二进制许可证清单或安装/修改信息核验；cargo-deny 单靠 metadata 无法发现所有源文件级差异。
5. 没有完整外部组件近 12 个月维护矩阵。此轮只验证 Multipass、K3s 与 Node 状态的官方记录；未审查其余 859 个根 Cargo package、JS 依赖、独立 workspace、OS 包及容器/Helm 镜像的维护活动、许可证或商业用途条款。
6. 未验证线上 cargo-audit、cargo-machete、桌面/容器/Web 工作流状态、required checks、发布访问权限、签名密钥托管或灾难恢复流程。
7. 根 workspace metadata 的 859 个 resolved packages 是默认 feature 视图；不是 `deny.toml` 全 feature 视图，更不是每个平台的实际制品清单。统计不能代替上述依赖闭包确认。

## §4 子代理失败接手清单

此审计 lane 未派生子代理；W16-A 由协调者并行推进。没有外部研究或验证子任务失败需要接手。

## §5 关闭门槛与发行守门

1. 建立经负责人确认的发行目标矩阵，为每个目标固定 lockfile、平台、features、基础镜像/OS 包、第三方镜像与打包资源；把仅内部工具排除或纳入的理由写清楚。
2. 统一根项目的许可证声明与发行附带许可证文本；确认仓库权利主体、每个 crate/服务的许可证表达式，以及 Helm chart 标识的一致性。
3. 将 `tools/aci-emitter` 的 MSRV/锁定工具链与 Rust `1.99.0` 基线对齐或由产品所有者正式明确例外边界；不得把 `1.77` 作为未披露的工作区基线。
4. 为 root、BFF、ACI、Web、Desktop 前端及实际 OS/container/guest 依赖分别做 locked、目标相关 license/advisory/notice 检查；锁文件更改必须重新生成并审阅完整 SBOM。
5. 以确切制品构建结果生成 machine-readable BOM，validate schema；将每个 BOM、LICENSE/NOTICE 汇总、制品文件、镜像/chart digest 和 provenance 绑定到同一个不可变 release id。让 BOM 生成和上传成为阻断条件，缺失时不得发行。
6. 把桌面前端改为 lockfile 驱动安装并使用仍受维护的 Node LTS；确保 Linux、Windows、macOS 产物都能被 release pipeline 获取、校验、签名并追溯，解决 Linux 上传缺失与短期 artifact retention 的运营方案。
7. 为容器固定可验证的 base image digest、OS package repository snapshot/version 与工具 action SHA；为 GHCR 所有镜像开启制品级 SBOM、provenance、签名与 digest 记录。镜像不可因 tag 更新而静默改变运行内容。
8. 重新生成 NOTICE/license attribution，并以受支持的比较流程解释 829/1265/859 计数的来源和口径；人工复核 cargo-deny 未能自动识别的 LICENSE 文件、来源及传递义务。
9. 对每一个随发行闭包交付的外部依赖记录准确版本、用途、license expression/text、商业使用条件、修改/链接/捆绑形态、履约项、近 12 个月 release/维护活动与上游公开维护渠道。当前 Multipass/K3s 活动证据仅是两个样例。
10. 把 Multipass 基线从 RC 记录更新为验证过的正式版；保留脚本已有两实例、全 inventory 计数、独占锁、fail-closed 和不自动 purge 行为。清理 README 中 Docker Desktop 的资源假设；部署说明需明确可用构建/导入替代路径，而不将桌面版容器产品列为前置。
11. 在发行环境实际复核上述每个平台的构建、SBOM/NOTICE 内容、镜像 digest、签名、install/uninstall/upgrade 与最多两个受测 VM 的磁盘预算；W16-B 审计本身不执行这些验证。

## §6 审批与当前状态

| 项 | 结果 |
|---|---|
| 发布闭包审计 | 🔴 未关闭 |
| 商业发行审批 | 不批准；必要闭包和法律材料未被证明 |
| 复核者 | 架构师（Mavis 接手审核） |
| 后续责任 | 发行 owner 确定目标矩阵和产品许可；各交付 owner 产出制品级 BOM、NOTICE/许可材料和维护证据；CI owner 修复强制门禁与签署链 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次建立 W16-B 发行闭包审计：记录发行入口、许可证/Notice/SBOM 缺口、Rust 与 Node 基线差异、Multipass 两实例守门及精确未验证范围。 |
