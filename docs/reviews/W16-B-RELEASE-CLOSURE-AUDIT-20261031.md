# W16-B 商业发行闭包审计刷新

> **结论：OPEN；本次不批准稳定商业发行。** 当前证据仍不能证明每一种发行制品的完整依赖、法律文本、SBOM、构建来源和签名都闭合。
>
> **核对基线：** `dev@f34866b8658ad6a226c648a6f88c745d7195ab74`（2026-10-31）
> **历史对照：** [2026-10-28 审计](W16-B-RELEASE-CLOSURE-AUDIT-20261028.md)，基线 `9e037f9e191dc7499e9e8d039e9966b348f83262`
> **范围：** 只读源审计、锁定离线 Cargo metadata、官方一手资料核对；未构建、测试、发布、部署、创建 tag 或访问远端 CI/发布设置。

## §0 目的与证据边界

本次刷新核对当前 `dev` 的项目许可身份、Rust/JavaScript 工作区锁定、桌面交付、Web 发布可追溯性、Star Canvas 容器、SBOM/NOTICE、供应链证明、Multipass/K3s 文档和版本/tag 一致性。用户的商业发行要求是允许不受用途、行业、席位或用量限制的商业使用；按每个真实制品逐版本核对全部传递依赖、二进制、安装器、操作系统包、容器/guest image、部署 chart 和维护状态；履行源代码、版权、NOTICE、专利及适用的安装信息义务；为制品提供 SPDX/SBOM 与可验证的构建来源。

### 0.1 用户决策更新（2026-10-31）

- 用户已锁定发行面：Desktop（Windows/macOS/Linux）、Web/Vercel、三份 Star Canvas 服务镜像。API/BFF、Envoy、Helm/GitOps 与实际构建/运行依赖纳入支持闭包审查；Multipass guest image 和内部开发工具不作为公开产品 payload。
- Star 自有项目 canonical license 已选择 MIT，与根 `LICENSE` 正文一致。根 Cargo workspace 和 Helm metadata 当前仍声明 Apache-2.0，因此元数据同步及第三方/adapter 权利、NOTICE、专利和分发闭包仍是 P0 未关闭项。
- 用户允许在全部稳定版验收门通过并完成最终复核后推送一个稳定版本；本轮不构建、发布、部署、创建 tag 或 push。
- 下文中“若纳入 stable”以及“范围待用户选择”的旧审计措辞，现由上述决策更新：Desktop、Web/Vercel、Canvas 镜像均为本轮发行范围内的 P0 门禁。审计记录的静态发现仍有效。

证据分为三类：

1. **静态源码证据：** 当前 checkout 中的 manifests、lockfiles、Dockerfiles、工作流、部署 YAML 和文档。它只能证明被审计的源码内容，不能证明远端 workflow 或实际制品状态。
2. **本地命令证据：** 本报告 §2 列出实际执行的只读命令及输出；没有运行构建或测试。
3. **外部资料证据：** 仅引用官方项目文档、正式发布页和标准页面。外部说明不证明 Star 已启用相应能力。

GitHub required checks、远端 release/assets、Vercel 项目配置/部署、GHCR 现有镜像、机器上的 Multipass inventory、集群实际运行镜像、供应商协议与权利人授权材料均未在本 lane 查询或验证。任何这些项目都不以仓库 YAML 或历史报告代替。

## §1 当前发行面与关闭矩阵

| 领域 | 当前 `dev` 静态证据 | 结论 |
|---|---|---|
| Star-only 服务容器 | [Canvas 发布工作流](../../.github/workflows/publish-star-canvas-services.yml) 的矩阵只有 `canvas-engine`、`domain-canvas`、`canvas-realtime`；[K3s 清单](../../deploy/star-canvas-services-k3s.yaml) 使用这三个 `star-*` 镜像，另有 Envoy edge 镜像。三份 Dockerfile 的最终阶段只复制对应 Star 服务二进制；但 workflow 的 context 是仓库根，未发现 root `.dockerignore`，且每份 Dockerfile 在 builder stage 执行 `COPY crates ./crates`，因此整个 crates 源树（包括仍在 workspace 的 `star-game`）会进入构建阶段/构建缓存。工作树近期提交记录包含 `chore(runtime): retire canvas-game service` 与 Star-only 部署对齐。当前维护的三镜像发布矩阵/K3s Canvas 清单没有游戏镜像或 game workload。 | **运行镜像已对齐 Star-only，构建输入仍有收敛项。** 用户要求 Docker 等只含 Star 时，W7 需将 build context/复制范围收窄到目标 Star 服务和明确共享依赖，并证明 game crate 不进入 builder context/cache；最终运行层仍只复制目标服务二进制。K3s 配置现在仍使用 `:dev`，Envoy 使用 `v1.28-latest`，不能作为不可变生产版本。 |
| 项目自身许可证 | 根 [Cargo.toml](../../Cargo.toml) 的 `workspace.package.license` 为 `Apache-2.0`；根 [LICENSE](../../LICENSE) 首行是 `MIT License`；[Helm Chart.yaml](../../deploy/helm/star/Chart.yaml) annotation 为 Apache-2.0；Chart 自述为 deprecated placeholder、无 active workload templates。用户已选择 MIT 作为 Star 自有项目 canonical license。 | **P0 阻断：** 按用户选择统一 Cargo workspace、Helm metadata、各 crate/package 和真实发行 payload 中的项目 license expression；另核实源代码归属、adapter 授权、第三方依赖及 NOTICE/专利/源码/安装信息义务。当前差异是元数据未同步，不是“禁止商用”的结论。 |
| Rust workspace / lockfiles | 离线锁定 metadata：根 workspace `109` 个 workspace member、`858` 个 package/resolved node；独立 `tools/aci-emitter` workspace `1` 个 member、`72` 个 package/resolved node；独立 [BFF workspace](../../bff/Cargo.toml) 在 `--locked --offline` 下失败，Cargo 明确报告 `bff/Cargo.lock` 需要更新。ACI 仍声明 `rust-version = "1.77"`，与 Star 1.99.0 基线不同。统计是 metadata 解析计数，不能等同制品清单、平台 feature 闭包或 BOM。 | **P0 阻断：** 修复并审阅 BFF lock；明确 ACI 是发布、构建辅助还是排除出发行；逐实际目标解析 root/BFF/ACI 的 locked feature/target 图。Cargo 官方说明 lockfile 用于锁定精确依赖版本以支持确定性构建；不能将缺失/过期锁文件当作可复现证据。[Cargo FAQ](https://doc.rust-lang.org/cargo/faq.html#why-have-cargolock-in-version-control) |
| Rust toolchain | 仓库要求 Rust 1.99.0；本机 `+stable` 为 1.99.1。Rust 官方 release notes 已列出 1.99.1（2026-10-15），故 1.99.0 不是当日最新 patch。 | **需发行 owner 决定是否升到 1.99.1。** 如果维持 1.99.0，必须提供在精确 pin 上的 release 目标验证证据；本 lane 没有做构建。 [Rust 官方 release notes](https://doc.rust-lang.org/releases.html) |
| 许可证盘点 / NOTICE | 根 [NOTICE.md](../../NOTICE.md) 标记生成于 2026-10-05，列出“1265 个 crate”；其自身注明 RustGameServer 适配的构建/分发依赖闭包审计未完成。上一审计的根 metadata 是 859 个节点，本轮是 858 个节点，时间/工具/计数对象不同，不能据差值推断删包或合规。`deny.toml` 是许可表达式 allowlist。cargo-deny 官方定义 `list` 为列出各 crate license 信息；其 license check 依赖 crate metadata/许可文本推断，并明确不穷尽扫描所有 crate 源码，也不保证 metadata 与实际 LICENSE 文本一致。 | **P0 阻断：** 重生成同一 toolchain、lockfile、features、target 和输出制品口径的 license inventory 与 NOTICE，并人工核实 source license 与本仓 RustGameServer 授权材料。当前 NOTICE 日期和数量不能代表本次 `dev`。 [cargo-deny list](https://embarkstudios.github.io/cargo-deny/cli/list.html), [cargo-deny license limitations](https://embarkstudios.github.io/cargo-deny/checks/licenses/index.html) |
| SBOM / 构建证明 | [license-gate.yml](../../.github/workflows/license-gate.yml) 将 `cargo-deny list --format json` 称为“SPDX evidence”，该导出及 artifact upload 都 `continue-on-error: true`；这是 license inventory，不是 SPDX BOM。当前检查到的 desktop 与 GHCR 发布 workflow 没有与制品摘要绑定的 SBOM/provenance/attestation/signature 步骤。SPDX 当前规范页列 3.0。GitHub 官方文档要求对 binaries/images 生成 provenance attestation，并要求 SBOM 文件路径或 image subject digest 明确关联。 | **P0 阻断：** 对每个实际发布 artifact 生成 schema-valid SPDX 或核准 BOM，并将 BOM、NOTICE、制品 SHA-256、构建 commit/workflow 和 attestation 绑定；缺失必须阻断发布。cargo-deny 清单可以作为附加输入，不能替代 BOM。 [SPDX specification](https://spdx.dev/use/specifications/), [GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations) |
| 安全/许可门禁 | cargo-audit workflow 中 `CARGO_AUDIT_SHA256` 是 `REPLACE_WITH_UPSTREAM_SHA256_FOR_0.22.2`；step 检查占位并 `exit 2`，所以静态配置表示该 job 当前必然失败，直到有经核验的摘要替换。license export 和 unused-dependency job 为可失败后继续。 | **P0 阻断 / 远端状态未知：** 修复并本地核实工具归档摘要，实际运行所有 gate；由 repo owner 确认所需 check 已设为 required。没有读取远端 CI，因此不能说 job 实际执行状态或分支保护已配置。 |
| Desktop | [desktop workflow](../../.github/workflows/star-desktop-build.yml) 在 Windows/macOS 安装 Node 20；Node 官方状态表显示 Node 20 于 2026-03-24 EOL。桌面前端目录只有 `package.json`，无 `package-lock`/`pnpm-lock`；[build-desktop.sh](../../scripts/build-desktop.sh) 使用不带 lock 的 `npm install`，并在已有 `node_modules` 时跳过安装。Linux `.deb` 只复制到临时目录，upload 仍注释且 `cp` 可失败后继续；Windows MSI 与 macOS DMG 作为 30 天 Actions artifacts 上传，不是 GitHub Release assets。Tauri updater 启用但 public key 仍为 `REPLACE_WITH_BASE64_PUBLIC_KEY`。 | **P0 阻断：** Desktop 已纳入稳定发行范围。锁定桌面前端依赖/维护中的 LTS Node、上传 Linux 输出、将所有平台产物绑定不可变 release ID，配齐签名、公钥、升级/回滚验证与长期留存。Node 官方说明生产应使用仍受支持 LTS；20 已 EOL。[Node.js release schedule](https://nodejs.org/en/about/previous-releases) |
| Web/Vercel | `frontend/` 同时保留 `package-lock.json` 与 `pnpm-lock.yaml`；常规 CI 执行 `npm ci`，P1E workflow 使用 pnpm 10 与 frozen lockfile。未找到版本控制中的 Vercel deployment result 或可核验的外部项目设置。本审计未访问 Vercel。 | **P0 阻断：** Web/Vercel 已纳入稳定发行范围。统一包管理器/lockfile；提供该 commit 的实际 build/deploy provenance、运行时依赖与 SBOM。不要由配置文件推断已有生产部署。 |
| Container / K3s | 三个 Star Canvas Dockerfile 使用 tag-only `rust:slim-bookworm` 与 `debian:bookworm-slim`，运行 `apt-get update`；publisher 使用 `docker/build-push-action@v5`、推送 branch/commit/semver/latest tag，但没有 workflow 内 SBOM/attest/signature；K3s 清单固定 `:dev` 与 `envoyproxy/envoy:v1.28-latest`。builder 的 `COPY crates ./crates` 会把仍在 workspace 的 `star-game` 源码带入构建输入。 | **P0 阻断：** 三份 Star Canvas 镜像已纳入稳定发行范围。pin/记录基础镜像和外部 action 的审核 SHA、OS package snapshot；将部署引用换为 digest，生成 image BOM 与 provenance/signature，并验证消费侧 digest。Docker context/cache 和活动 workload 必须排除 game/非 Star 遗留内容。 [GitHub action SHA guidance](https://docs.github.com/en/actions/reference/security/secure-use), [Docker digest guidance](https://docs.docker.com/build/building/best-practices/#pin-base-image-versions) |
| Multipass / K3s | [Multipass README](../../deploy/multipass/README.md) 仍要求 1.17.0-rc1、保存本机特定安装路径及“Windows + Docker Desktop”磁盘余量。Canonical 官方 releases 显示 Multipass 1.17.0 于 2026-10-19 正式发布，且新版 Windows 默认 HCS driver；这与 README 叙述不同。此 lane 未运行 Multipass/K3s，测试阶段两实例上限没有在本机重新核验。 | **文档/运行闭包阻断待 D2 lane 合并验证：** 移除 Docker Desktop 假设、用正式版与可校验 installer/image/chart 证据更新说明；保留最多 2 个 inventory instances、锁和 fail-closed 规则；在安全授权的独立验收后才验证真实两实例运行。 [Multipass releases](https://github.com/canonical/multipass/releases), [Windows HCS migration](https://canonical.com/multipass/docs/latest/how-to-guides/customise-multipass/migrate-from-hyperv-to-hcs-on-windows/) |
| 版本与 stable 发布入口 | 本地 git refs 最新 semver 风格 tag 为 `v1.0.5`；root Cargo workspace 和 Tauri config 为 `0.1.0`；Helm chart 是已废弃 placeholder `0.1.0`。当前三个 Canvas 镜像 workflow 会响应 `v*` 并产出 semver tag，但它本身没有创建 GitHub Release、桌面版本、artifact set、immutable release 或签名验证闭环。用户已确定产品范围，但本审计仅核对本地 refs/源码，不核验远端 releases。 | **P0 阻断：** canonical version、tag owner、release source commit 与各平台制品版本映射仍未决定；需将它们绑定到同一不可变发布记录。用户允许在所有门禁通过并最终复核后推送稳定版。 |

### 1.1 相对上一轮的实质变化

- 上轮将 game/canvas 镜像作为容器闭包之一；当前 `dev` 已有 `canvas-game` 服务退役与 Star 服务部署对齐提交。当前维护的 Canvas publisher 与 K3s manifest 是三份 Star 服务镜像加 Envoy edge；没有游戏镜像发布入口。`star-game` 仍属于源码 workspace，因此准确表述应为“未进入当前受审的容器/运行清单”，不能说仓库已没有 game 代码。
- 根 metadata 的结果由上轮记录 859 resolved packages 变为本轮 858；本轮 root metadata 命令成功，但只表明当前 lockfile 可解析的默认图。`bff` locked metadata 本轮失败；不能将统计差异解释为依赖清理完成。
- Multipass 1.17.0 已正式发布，之前“当前最新稳定 1.16.4、必须 RC1”的 README 现在过时；相关 README 正由独立 D2 lane 处理，本 lane 保留，不越界修改。

## §2 本次验证摘要

### 静态源码核对

已查看根/BFF/ACI manifests 与 lockfiles、根 LICENSE/NOTICE、`deny.toml`/`about.toml`、license/security gate、Desktop workflow/Tauri updater/build script、Canvas publisher 与 3 个 Dockerfile、K3s manifest、deprecated Helm chart、Multipass README，以及当前本地 tag refs。

### 实际执行的本地只读命令

| 命令/检查 | 结果 | 限定 |
|---|---|---|
| `rustc +stable --version`; `cargo +stable --version` | rustc/cargo `1.99.1` | 不是精确项目 pin `1.99.0`。 |
| `cargo +stable metadata --locked --offline --format-version 1` | exit 0；109 workspace members、858 packages/resolved nodes | metadata only；没有构建、测试或完整平台/feature artifact closure。 |
| `cargo +stable metadata --locked --offline --format-version 1 --manifest-path bff/Cargo.toml` | exit 1；`bff/Cargo.lock` 需要更新，`--locked` 禁止更新 | 没有改动 lockfile；需要 owner 后续修复并审查。 |
| 同一 locked/offline metadata 命令，`tools/aci-emitter/Cargo.toml` | exit 0；1 member、72 packages/resolved nodes | ACI workspace 仍有不同 MSRV 1.77。 |
| `git tag --list --sort=-v:refname` | 本地最新 tag `v1.0.5` | 只代表本 checkout 的 local refs，不代表远端 release 状态。 |
| Markdown whitespace scan | 通过 | 三份本轮 Markdown 文件独立扫描，尾随空白 `0` 行。它们仍是未跟踪文件，所以 `git diff --check` 不会校验这些新文件。 |

没有执行 `cargo test`、编译、`cargo-deny`、`cargo-audit`、前端 install/build、Docker/OCI build、Multipass 或 Kubernetes 命令；没有查询远端 CI、Vercel、GHCR、分支保护或 secret。

## §3 阻断与已知缺口

| ID / 优先级 | 缺口 | 责任 owner / 前置 | 关闭证据 |
|---|---|---|---|
| RC-01 / P0 | 发行面已由用户确定为 Desktop（Windows/macOS/Linux）、Web/Vercel、三份 Star Canvas 镜像；canonical version、source commit、tag owner 和各包映射未定。 | Product + Release owner | 签署版本/制品映射和 release procedure；API/BFF、Envoy、Helm/GitOps 作为支持依赖纳入闭包；guest image/内部工具不作为公开产品 payload。 |
| RC-02 / P0 | Star 自有项目 canonical license 已选 MIT，但根 Cargo workspace 与 Helm metadata 当前仍标 Apache-2.0；第三方/adapter 权属和 NOTICE 闭包未核实。 | Release owner + OSS/compliance | 将项目元数据统一为 MIT；逐真实制品审核版权、adapter 授权、依赖许可、NOTICE/专利/源码/安装信息材料并留存审阅证据。 |
| RC-03 / P0 | Root `NOTICE.md` 生成数据过期且含 RustGameServer adapter 尚未闭包审计的明示限制；许可 inventory 被误叫 SPDX evidence。 | Legal/OSS + Platform | 每个发布目标重新生成/核对完整 notice；人工复核源许可与授权；具体制品携带并归档。 |
| RC-04 / P0 | BFF locked metadata 失败；ACI 与 Rust baseline 1.99.0 分离。 | Rust build owner；依赖目标矩阵 | 每个实际发布 workspace lock 命令成功且未隐式更改；ACI 纳入/排除决定及 MSRV 例外/对齐证明。 |
| RC-05 / P0 | Stable artifact 没有与制品关联的 SPDX BOM + attestation/provenance/signature/verifier 记录。 | Supply-chain owner；RC-01 + RC-04 + RC-03 | 对每个平台 installer/容器等逐制品 SHA 生成、validate 和 verify BOM/attestation；缺失则阻断。 |
| RC-06 / P0 Desktop | Node 20 已 EOL、桌面 JS 无 lockfile、Linux artifact 未上传、更新公钥占位，且无实测签名/升级回滚路径。 | Desktop owner；已纳入发行面 | 受维护 Node LTS + lockfile；三平台 release assets 可下载；配置 updater/signing 并提供安装、更新、回滚的实际记录。 |
| RC-07 / P0 Container | 发布流程推可变 tags，Dockerfile 用浮动基础 tag/apt latest；Action tag 未锁到审核 SHA；无 SBOM/attestation；builder context 包含 `star-game` 源码。 | Container owner；RC-01/05 | 不可变基础镜像/Action/OS 依赖，三份 Star Canvas image 的 BOM/provenance/signature/verify；Docker context/cache 与运行清单排除 game/遗留内容，部署消费同一 digest。 |
| RC-08 / P0 | `cargo-audit` workflow 仍有硬编码 placeholder SHA；required-check、远端 workflow result 不可见。 | CI owner + GitHub repo owner | 核验上游归档摘要并更新、运行 CI；owner 提供 branch protection required check evidence。 |
| RC-09 / P0 Web | `frontend/` 同时保留 npm 与 pnpm lock；常规 CI 与另一 workflow 使用不同包管理器；没有本审计可见的 Vercel 部署证据。 | Web owner；已纳入发行面 | 选定唯一包管理器/锁文件/Node LTS；提供实际部署 commit、runtime config、dependency/BOM 和 rollback evidence。 |
| RC-10 / P1 Infra | Multipass/K3s 测试文档仍有旧版本/环境描述；guest/K3s/chart/add-on 的许可、版本、摘要与两实例恢复验收未闭合。 | Infra owner；D2 文档 lane、两实例验收 | 更新 README；核对测试依赖 BOM/license/maintenance；最多两实例且 fail-closed 的容量与恢复原始记录。guest image 不是此次公开产品 payload。 |

## §4 子代理失败接手清单

本 lane 没有再派生 worker；本次审计独立于 D2 Multipass 文档维护和 W16 revoke lane。没有子代理失败产出待接手。D2 lane 需要在其完成后重新审查其文档变更，不能以本报告中的发现冒充 D2 运行验收。

## §5 发行守门（未满足前不得标记稳定发行就绪）

1. 发行负责人签署单一 release target matrix 与 canonical version/tag/commit mapping。
2. 产品权利人选择并统一 root/LICENSE/Cargo/Chart/package 对应 license expression 与文本。
3. 每个交付物按实际法律文本检查版权、专利、NOTICE、源码/安装信息义务；不以 cargo metadata 取代人工审核。
4. Root、BFF、ACI 作为独立 Cargo 工作区逐个 locked 解析；不自动重写 lockfile。
5. 每个发布平台/feature/target 均有依赖闭包与 Rust MSRV evidence；区分发行物和开发/CI-only tooling。
6. Web 与 Desktop 确认唯一 JS package manager、lockfile、Node LTS 与 locked install。
7. 第三方组件维护状态和商业许可逐版本核验；采用 copyleft 不等于拒绝商用，但必须履约。
8. NOTICE/license inventory 由与发布制品相同依赖图生成，记工具、版本、参数、lockfile 与 commit。
9. 每个安装包、容器和 guest/chart artifact 均生成 schema-valid SPDX/CycloneDX BOM，并绑定确切 SHA/digest。
10. 每个实际制品生成 build provenance/signature/attestation，并从干净消费环境验证身份、提交和摘要。
11. 许可证、advisory、SBOM、signature、provenance、desktop artifact 和 image publish gates 设置为真实 required checks；上传缺失必须失败。
12. 容器 base image、外部 Action、OS package snapshot 与部署镜像 digest 可复查，不使用 `:dev`/mutable `latest` 作为 stable 引用。
13. Desktop Windows/macOS/Linux 安装包都可在 release pipeline 获取；验证 signing/notarization/updater key/升级和回滚路径。
14. Web/Vercel 只有被目标矩阵纳入后才以有效部署记录、runtime config 与依赖清单作为发行对象；不宣称未验证环境已发布。
15. Multipass/K3s README 使用当前正式版材料且符合 Windows HCS/两实例/无 Docker Desktop 条件；不运行未经批准的 inventory/destructive 操作。
16. stable candidate 的功能测试、平台安装测试、容器 digest/部署验证、两实例容量与恢复演练全有可复核原始输出。
17. 最终发行前检查远端 branch protection/release permissions、不可变 tag/release、制品保留时间与消费者验证说明；远端状态须使用授权账户证据。

## §6 审核与发行决议

| 角色 | 审核范围 | 当前状态 |
|---|---|---|
| 架构 | 产品交付矩阵与版本身份 | 发行面已由用户确定；canonical version、source commit 和 owner 映射仍未闭合。 |
| SRE Lead | CI、artifact retention、部署与运行验证 | 未闭合；未查看远端 job/部署结果。 |
| 平台 | Cargo/JS/OS/容器/guest 依赖与供应链证明 | 未闭合；BFF lock 已被本地命令证明不匹配。 |
| 评审主持 | 许可证、NOTICE、BOM 与可验证证据 | 未闭合；root license mismatch、NOTICE 过期。 |
| PM | stable 范围、版本号与交付承诺 | 发行面已确定；版本/tag/commit 映射仍不一致。用户已条件授权完整门禁通过后的稳定版推送。 |

**本轮审计结果：** 仅允许作为 release closure 的发现清单和后续 WBS 使用；**不批准商业发行，不授权 push/tag/build/publish/deploy**。`cargo-deny list` 只能提供许可证信息，不是 SPDX BOM；官方 SPDX 规范当前为 3.0，而 GitHub 提供 artifact/SBOM attestations 机制，但 Star 尚无源码证据显示已为受审发行物实现这些绑定。[cargo-deny list](https://embarkstudios.github.io/cargo-deny/cli/list.html), [SPDX specifications](https://spdx.dev/use/specifications/), [GitHub artifact attestation how-to](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | W16-B 子代理（Mavis 接手审核待协调者复核） | 基于 `dev@f34866b8` 刷新 W16-B release closure，吸收 Star-only Canvas image/deployment 变化，增加锁定 metadata/BFF lock 错误、Desktop/容器/BOM/签名与商业发行 WBS。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新用户确认的发行面与 MIT canonical license；将 Desktop/Web/Canvas image 均设为不可条件跳过的 P0 门，并记录仅在全部门禁通过后的稳定版条件推送授权。保持未验证项 OPEN。 |
