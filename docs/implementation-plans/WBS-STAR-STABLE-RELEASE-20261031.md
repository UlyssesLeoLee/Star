# WBS — Star Stable Commercial Release Closure

> **状态：OPEN / 不可发布。** 此 WBS 先闭合真实发行范围和证据。用户已条件授权：全部稳定版门禁通过并完成最终复核后可推送一个稳定版本；在此之前不创建 release/tag、不构建或发布 artifact，也不 push。
>
> 审计快照基线：`dev@f34866b8658ad6a226c648a6f88c745d7195ab74`。当前本地 `dev` 已集成 W16-A（`ea7058e1d5ecf9e41d3aa99e4471a07cc8265d11`）和 W16-B 发行范围/MIT 决策 WBS（`163f715bb68bae1d0605300b705fac03b7dad192`）；没有因此重新运行发行构建或部署审计。
> 审计：[`W16-B-RELEASE-CLOSURE-AUDIT-20261031.md`](../reviews/W16-B-RELEASE-CLOSURE-AUDIT-20261031.md)

## 已确认的发行决策（2026-10-31）

- 发行产品面：Desktop、Web/Vercel、三份 Star Canvas 服务镜像。
- Star 自有项目的统一许可证选择：MIT，与当前根 `LICENSE` 正文一致。该选择不代表第三方依赖闭包或 RustGameServer adapter 权属已审查完成。
- 发行范围外：Multipass guest image、内部开发/CI 工具；它们仍可作为测试/构建依赖接受相应的风险和许可审查。
- Docker/build/deployment payload 必须只含上述 Star 服务及明确列出的运行依赖；`star-game` 与其他遗留 game 内容不得进入产品镜像、构建上下文/缓存或活动 workload。
- 用户允许在全部稳定版验收门闭合后推送一个稳定版本；当前仍不得提前创建 release/tag 或发布任何 artifact。

## 完成定义

只有在发行目标明确、版本和法律文本统一、每个真实交付目标依赖锁定并完成 license/advisory/BOM 审查、CI 对缺证据失败关闭、artifact 可验证/可安装/可回滚、部署只引用 Star 制品的固定摘要，并由目标平台的实际验证记录证明后，才可提交稳定 release candidate 审核。任一必需制品未闭合时，整个产品 release 保持 OPEN。

## 优先级与依赖

| ID | 优先级 | 工作包 / owner | 依赖 | 交付物与验收标准 | 状态 |
|---|---|---|---|---|---|
| W0 | P0 | **发行矩阵与版本所有权** — Release owner + PM | 无 | 已锁定发行面：Desktop（Windows/macOS/Linux）、Web/Vercel、三份 Star Canvas 服务镜像。API/BFF、Envoy、Helm/GitOps 及实际 build/runtime 闭包作为这些发行面的支持依赖纳入审查，不另算产品面；Multipass guest image 与开发工具不进入公开产品 payload，Multipass/K3s 仅作测试环境。`star-game` 可保留在源码仓，但不得进入发布 artifact、Docker build context/cache、活动 manifest/workload。仍需确定 canonical version、source commit、tag/release owner、各包版本映射和 release procedure。 | PARTIAL：范围已由用户确认；版本与 owner 映射未定。 |
| W1 | P0 | **项目法律身份与 NOTICE** — 权利 owner + OSS/compliance | W0 | Star 自有项目 canonical license 已由用户确定为 MIT，与根 `LICENSE` 正文一致。将根 Cargo workspace、Helm metadata、各 crate/package 与发行 payload 声明统一为 MIT；逐发行目标核验实际源码许可、授权来源、专利/版权/NOTICE/修改/再分发/安装信息义务。重生成同一 commit + lock + features/target 口径的 NOTICE/license inventory，并人工复核 RustGameServer adapter 权属及完整分发依赖闭包。验收为可追溯的用户决策记录、metadata diff、各 artifact payload 清单、notice diff 与依赖/权属审阅证据。 | PARTIAL：Chart.yaml 与 deny.toml 已调整；根 Cargo metadata、license-gate、各制品权属及 NOTICE 仍未验证。 |
| W2 | P0 | **Rust/JS 锁与目标依赖闭包** — Build/Runtime owners | W0、W1 | 让实际发行目标使用的 root/BFF/ACI Cargo workspaces 在 locked offline metadata 下通过；修复 BFF lock mismatch；标明 ACI 是 CI/build-only 或发行闭包依赖，并处理其 MSRV 1.77 与 Star 1.99.0 基线的边界。Web 与 Desktop 分别指定唯一 JS package manager/lockfile、维护中的 Node LTS 和 lock-driven install。验收含每目标 cargo metadata/feature graph、JS frozen install、MSRV/target 结果及锁文件审阅。 | OPEN：桌面隔离前端 lock 与 `npm ci`/build 已在 lane 验证；Web 根 lock 与 Rust workspace/feature/MSRV 闭包仍未验证。 |
| W3 | P0 | **BOM、签名与 provenance** — Supply-chain owner | W0–W2 | 为每个选定发行物（Desktop 安装包、Web/Vercel build、三份 Canvas 镜像）及其实际依赖闭包生成 schema-valid SPDX 或核准 BOM；每份 BOM 绑定 artifact SHA-256/image digest。配置并运行 build provenance/签名/attestation，提供独立 consumer 验证输出。license inventory 仅作补充材料。生成/上传/验证任一步缺失均令 release gate 失败。验收为 artifact、BOM、attestation bundle 与验证记录的同一 release ID。 | OPEN |
| W4 | P0 | **强制 CI 与 Action/工具完整性** — CI owner + GitHub repo owner | W0、W2、W3 | 将 cargo-audit placeholder digest 替换为经上游校验值；对发行闭包运行 license/advisory；确认 required-check 与最小权限；第三方 Actions 固定审核过的完整 SHA；上传/SBOM/attestation 缺失不得 `continue-on-error`。验收为源码 diff、所有目标 workflow 实际成功 runs，以及 branch protection required-check 证据。 | OPEN：2026-10-31 本地 cargo-deny 诊断另发现 `webpki-roots 1.0.9` 命中 `RUSTSEC-2026-0381`；需 owner 核准锁文件修复并在 Rust 1.99.0 基线上复验。 |
| W5 | P0 | **桌面发行闭环** — Desktop owner | W0–W4 | 将 EOL 的 Node 20 换为受维护 LTS；锁定桌面 JS；为 Linux/Windows/macOS 输出稳定命名资产并上传到同一不可变 release；配置 updater 公钥与签名密钥管理（不在仓库内存 secret）；验证安装、更新、签名和回滚。Linux artifact 上传不得注释或 `|| true`。验收为三 OS 可下载签名 assets + BOM/attestation + 真实安装/升级测试原始记录。 | OPEN：新独立构建流程与前端修复已实现；本地 13 个 suite/62 项通过，但运行环境为 Node 22.19（低于产品要求 Node 24），且 hosted CI、Rust/Tauri 包、签名、公证、安装升级/回滚和 CSP 未验证。 |
| W6 | P0 | **Web/Vercel 发布闭环** — Web owner | W0、W2–W4 | 统一 Web package manager/lockfile、Node 与构建命令；记录部署 source commit、build/runtime config、依赖/BOM 与 rollback 证据。验收需实际 Vercel deployment record 与消费侧验证；不得用静态配置推断部署已就绪。 | OPEN：Vercel build contract 静态配置已通过；项目 Root Directory、真实部署、消费侧验证和回滚仍未验证。 |
| W7 | P0 | **Star 容器/部署供应链** — Container + Infra owners | W0–W4 | 覆盖三份 Star Canvas 镜像，并审查 Envoy 等实际运行依赖；固定/记录 Rust 与 OS base image digest、OS package snapshot 与 Action SHA；生成每镜像 BOM/provenance/signature。将部署中的 `:dev`、`envoyproxy/envoy:v1.28-latest` 换为审核后的固定摘要/版本。收窄 Docker build context/`COPY` 到目标 Star 服务和已列明共享依赖，证明 `star-game` 等遗留源码不进入 builder context/cache，且 final layer 仅含目标 Star service/runtime dependencies。验收：构建输入 inventory、镜像 digest、独立 provenance/SBOM verify、K3s rendered manifest 指向同一 digest且无 game workload。 | OPEN：三个镜像的最小构建上下文与 YAML 静态检查通过；缺 Rust 1.99 编译、实际 CI/OCI 构建、发布审批、依赖许可证/NOTICE、真实镜像 digest 和 Multipass/K3s readiness。 |
| W8 | P1 | **Multipass/K3s 测试文档与两实例验收** — Infra owner（与 D2 lane 集成） | W0、W7 | 使用正式版 Multipass 文档；移除 Docker Desktop 假设；核对 guest image、K3s、chart/add-on 的版本、license、摘要、维护和 BOM。测试前检查 inventory 必须 fail-closed 且最多 2 个实例。验收包括更新后的 README、两实例容量和停止/删除状态计数测试，以及集群升级/恢复日志；不得自动 purge。guest image 是测试环境依赖，不作为本次公开发行 payload。 | OPEN：文档 lane 尚未集成；真实两实例验收缺失。 |
| W9 | P0 | **Release candidate 验证与审批门** — Release owner + QA/SRE + compliance | W0–W8 | 对获批 candidate 运行每个入选目标的构建、功能/安装/升级/回滚、license/advisory、BOM/signature/provenance 验证，记录不可变 artifact digest 和远端 run IDs，逐项关闭审计 RC-01—RC-10。用户已条件授权：全部发行门通过且经最终复核后可推送稳定版本；这不授权当前提前创建 tag/release、build、publish、deploy 或 push。缺任一项即保持 OPEN。 | BLOCKED：依赖 W0–W8 全部完成。 |

## 当前可以并行的工作包

W1 法律/NOTICE、W2 锁与构建闭包、W3 SBOM/provenance、W5 桌面闭环、W6 Web/Vercel、W7 容器供应链可在隔离 lane 并行推进。发行面已固定；W0 剩余的 canonical version/tag/commit 映射必须在生成发行制品前闭合。任何 lane 当前都不得提前创建 release/tag 或发布 artifact。变更 rebase/merge 进入 dev 后重新运行相应验收。W8 文档由 D2 lane 负责；D4 正在活动的 Multipass owner 文件不得被并行修改。

### 已登记的首轮并行 lane（stable-hardening set revision 24 快照）

基线为本地 dev@04d8b7a84514dcb454874eb852651778e1452dea；本轮仅做发布闭环硬化，不关闭稳定版 WBS，也不执行 tag、发布、部署或 push。首个宽清单被 MAGOS 拒绝后，按 owner 冲突证据收窄 manifest；没有修改或清理被占用的分支/工作树。状态列对应本次 WBS 同步时的 MAGOS set 状态；最终集成以 set-show 与集成记录为准。

| Lane | 当前状态 | 分支 / 工作树 | 文件范围 | 目标与验收 |
|---|---|---|---|---|
| license-policy | active（WBS 同步） | agent/stable-hardening/license-policy / D:/orcaWork/Star/star-stable-hardening-license-policy | Chart.yaml、deny.toml、本 WBS、lane brief/report/audit | 目标：首方 Helm 元数据设为 MIT；保留完整依赖权利/NOTICE 审阅缺口。 |
| desktop-packaging | active（workflow follow-up） | agent/stable-hardening/desktop-packaging / D:/orcaWork/Star/star-stable-hardening-desktop-packaging | 独立稳定桌面 workflow、Tauri 前端 package/lock/config/build script、修复与 lane brief/report/audit | 目标：锁驱动安装、前端构建和测试；跨平台 hosted build、签名密钥、真实安装升级另验。 |
| web-vercel-config | ready（待集成） | agent/stable-hardening/web-vercel-config / D:/orcaWork/Star/star-stable-hardening-web-vercel-config | frontend/vercel.json、lane brief/report/audit | 显式 Vercel 构建合同；不改活跃依赖 lock owner 文件；真实 deployment 另验。 |
| canvas-oci | active（release gate follow-up） | agent/stable-hardening/canvas-oci / D:/orcaWork/Star/star-stable-hardening-canvas-oci | 三份 Canvas Dockerfile、发布 workflow、K3s 镜像 manifest、.dockerignore、lane brief/report/audit | 目标：只含 Star Canvas 服务依赖，builder/context/cache 排除 star-game，部署固定 digest；OCI/K3s 实验另验。 |

保留边界：根 Cargo.toml 与 license-gate.yml 保持其活动 owner 所有权；Web 根 package-lock 仍属于活跃依赖 owner。桌面 lane 只更新隔离的桌面 package lock 并新增独立稳定构建 workflow，不覆盖旧桌面 workflow/owner 文件。Canvas tag 发布审批、第三方依赖许可证/NOTICE 和真实 OCI/K3s readiness 仍未验证；W3 SBOM/provenance、W4 required-check 与 Action pin 证据、W8 D2/D4 的 Multipass 两实例真实验收、W9 candidate 验收仍 OPEN/BLOCKED。D4 活动终端及所有 W8、D2 文件保持原样。

## 外部约束依据（截至 2026-10-31）

- Rust 官方 notes 显示 1.99.1 于 2026-10-15 发布；Star baseline 1.99.0仍是用户声明的最低版本，但是否升 patch 需在 W2 决策。[Rust Release Notes](https://doc.rust-lang.org/releases.html)
- Node 官方状态表显示 v20 于 2026-03-24 EOL；生产应使用 Active/Maintenance LTS。桌面 CI 仍固定 v20。[Node.js releases](https://nodejs.org/en/about/previous-releases)
- Canonical 官方 release list 显示 Multipass 1.17.0 于 2026-10-19 GA；当前 README 仍是 1.17.0-rc1。[Multipass releases](https://github.com/canonical/multipass/releases)
- GitHub 可对 binary/container artifact 生成 provenance/SBOM attestation，但要求把实际 artifact path 或 image digest 与 BOM 绑定；这只说明平台支持，不证明 Star 已实现。[GitHub how-to](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)

## 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | W16-B 子代理（Mavis 接手审核待协调者复核） | 按 W16-B 审计发现制定稳定发行 WBS；所有构建、发布和部署证据保持 OPEN。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录用户确认的 Desktop + Web/Vercel + 三份 Star Canvas 镜像发行面及 MIT canonical license；把 W5–W7 标为无条件 P0、W8 的两实例运行验收保持 OPEN，并记录仅在所有门禁通过后推送稳定版的条件授权。 | 用户完成发行范围与许可证决策 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | W16-B 审计/WBS 已经父级审阅并快进集成到本地 `dev@163f715`；稳定版门禁仍为 OPEN。 | W16-B 文档 lane 完成集成 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 登记并启动 stable-hardening 首轮四个隔离 lane；按 MAGOS 实际 owner 冲突证据收窄清单，完整标明未解除的 Cargo/license workflow 与 Dependabot 锁文件阻塞；没有关闭稳定版门禁。 | 用户确认发行范围与 MIT，并要求整理后发布稳定版 |
| v0.5 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 stable-hardening manifest revision 3 与当前执行状态：Web/Vercel 静态配置 lane 已 ready、待集成；license、desktop、Canvas OCI 仍进行中；release gates 不变且仍 OPEN/BLOCKED。 | Web lane 完成独立静态复核 |
| v0.6 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 `cargo-deny 0.20.2` 在 Rust 1.99.1 诊断扫描中发现 `webpki-roots 1.0.9` / `RUSTSEC-2026-0381`；声明该结果不替代 Rust 1.99.0 基线验收，且锁文件修复需要 owner 审查。 | 本轮 W1 policy 复核发现现有 advisory 阻塞 |
| v0.7 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 stable-hardening revision 24 的实际切片：Chart/deny 为 MIT，桌面隔离 lock/build/frontend tests 已实现，Vercel 配置通过首轮审查；桌面 CI 与 Canvas 发布审批 follow-up 正在进行。按证据更新 W1/W2/W5/W6/W7；所有 hosted CI、法律/NOTICE、签名、部署、OCI digest 和两实例验收仍开放。 | 用户确认范围与许可证后，四条并行 lane 已完成首轮 |
