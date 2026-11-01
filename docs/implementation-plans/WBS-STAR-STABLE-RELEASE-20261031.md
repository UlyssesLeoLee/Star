# WBS — Star Stable Commercial Release Closure

> **状态：OPEN / 不可发布。** 此 WBS 先闭合真实发行范围和证据。用户已条件授权：全部稳定版门禁通过并完成最终复核后可推送一个稳定版本；在此之前不创建 release/tag、不构建或发布 artifact，也不 push。
>
> 最新接受证据快照（2026-11-01）：记录该快照时，target dev@bde4e9d8e9dac0e25f7d983814d37fb27bc9313e 尚未包含集成候选 agent/star-stable-followup/integration@cdb66b5b4445aa57c7d1c60d3c5e6acf0f50a9fe 或本 WBS lane；该集成候选基于当时的 dev@bde4e9d8e9dac0e25f7d983814d37fb27bc9313e，包含已接受的 W1 与 W2 lane。W1 来源提交 9083742a63306f310976b820faaa02a400a17ab8、修正提交 ab7b0389ba68422725348bbe5811aff89efd1637，集成提交 393c98ce3b5562e64ef5e82e58dd46e09fe6c3f2。W2 来源提交 eb1e91f47a3b6d19483990942114caa742407b9d，集成提交 cdb66b5b4445aa57c7d1c60d3c5e6acf0f50a9fe。W1 仍为 BLOCKED，W2 仍为 OPEN/BLOCKED；W0–W9 均未关闭，稳定版仍 OPEN。
> 审计：[`W16-B-RELEASE-CLOSURE-AUDIT-20261031.md`](../reviews/W16-B-RELEASE-CLOSURE-AUDIT-20261031.md)

## 已确认的发行决策（2026-10-31）

- 发行产品面：Desktop、Web/Vercel、三份 Star Canvas 服务镜像。
- Star 自有项目的统一许可证选择：MIT，与当前根 `LICENSE` 正文一致。已核实根 `Cargo.toml` 的 `[workspace.package].license` 仍为 `Apache-2.0`，与用户决定冲突；W1 保持阻塞，直到 owner 修正并验证全部元数据。第三方依赖闭包或 RustGameServer adapter 权属也尚未审查完成。
- 发行范围外：Multipass guest image、内部开发/CI 工具；它们仍可作为测试/构建依赖接受相应的风险和许可审查。
- Docker/build/deployment payload 必须只含上述 Star 服务及明确列出的运行依赖；`star-game` 与其他遗留 game 内容不得进入产品镜像、构建上下文/缓存或活动 workload。
- 用户允许在全部稳定版验收门闭合后推送一个稳定版本；当前仍不得提前创建 release/tag 或发布任何 artifact。

## 完成定义

只有在发行目标明确、版本和法律文本统一、每个真实交付目标依赖锁定并完成 license/advisory/BOM 审查、CI 对缺证据失败关闭、artifact 可验证/可安装/可回滚、部署只引用 Star 制品的固定摘要，并由目标平台的实际验证记录证明后，才可提交稳定 release candidate 审核。任一必需制品未闭合时，整个产品 release 保持 OPEN。

## 优先级与依赖

| ID | 优先级 | 工作包 / owner | 依赖 | 交付物与验收标准 | 状态 |
|---|---|---|---|---|---|
| W0 | P0 | **发行矩阵与版本所有权** — Release owner + PM | 无 | 已锁定发行面：Desktop（Windows/macOS/Linux）、Web/Vercel、三份 Star Canvas 服务镜像。API/BFF、Envoy、Helm/GitOps 及实际 build/runtime 闭包作为这些发行面的支持依赖纳入审查，不另算产品面；Multipass guest image 与开发工具不进入公开产品 payload，Multipass/K3s 仅作测试环境。`star-game` 可保留在源码仓，但不得进入发布 artifact、Docker build context/cache、活动 manifest/workload。仍需确定 canonical version、source commit、tag/release owner、各包版本映射和 release procedure。 | PARTIAL：范围已由用户确认；版本与 owner 映射未定。 |
| W1 | P0 | **项目法律身份与 NOTICE** — 权利 owner + OSS/compliance | W0 | Star 自有项目 canonical license 已由用户确定为 MIT，与根 `LICENSE` 正文一致。将根 Cargo workspace、Helm metadata、各 crate/package 与发行 payload 声明统一为 MIT；逐发行目标核验实际源码许可、授权来源、专利/版权/NOTICE/修改/再分发/安装信息义务。重生成同一 commit + lock + features/target 口径的 NOTICE/license inventory，并人工复核 RustGameServer adapter 权属及完整分发依赖闭包。验收为可追溯的用户决策记录、metadata diff、各 artifact payload 清单、notice diff 与依赖/权属审阅证据。 | BLOCKED：已接受的 lane 9083742a63306f310976b820faaa02a400a17ab8 与修正提交 ab7b0389ba68422725348bbe5811aff89efd1637，经 393c98ce3b5562e64ef5e82e58dd46e09fe6c3f2 集成；13 个 Star 自有 Cargo package license 声明已设为 MIT。ACI emitter 属于独立项目，保留双许可证；BFF 源文件 SPDX 仍为 MIT OR Apache-2.0。根 Cargo workspace license 与用户选择的 MIT 仍未解决，因 owner 重叠而未纳入此 lane；仓库根目录没有 NOTICE。第三方授权/归属、专利与归属声明、完整依赖闭包、NOTICE/SBOM，以及 Desktop、Web/Vercel 与三份 Canvas 实际制品的许可/payload 均未闭合。Rust 1.99.0 rustc 不可用，Cargo metadata 校验未运行；本轮 metadata 修改不构成 W1 关闭证据。 |
| W2 | P0 | **Rust/JS 锁与目标依赖闭包** — Build/Runtime owners | W0、W1 | 让实际发行目标使用的 root/BFF/ACI Cargo workspaces 在 locked offline metadata 下通过；修复 BFF lock mismatch；标明 ACI 是 CI/build-only 或发行闭包依赖，并处理其 MSRV 1.77 与 Star 1.99.0 基线的边界。Web 与 Desktop 分别指定唯一 JS package manager/lockfile、维护中的 Node LTS 和 lock-driven install。验收含每目标 cargo metadata/feature graph、JS frozen install、MSRV/target 结果及锁文件审阅。 | OPEN/BLOCKED：已接受的 W2 fixture lane eb1e91f47a3b6d19483990942114caa742407b9d，经 cdb66b5b4445aa57c7d1c60d3c5e6acf0f50a9fe 集成；seed_directory 与 seed_task_binding 在生成 SQL 前都要求具体的远端 Branch UUID，保留远端 Branch fixture 约束。该 lane 的 git diff --check 通过。要求的 Rust 1.99.0 all-targets check 未开始，rustc.exe 报不适用于已安装的 1.99.0 Windows MSVC toolchain。Rust 1.99.1 诊断等待现有 Cargo build lock 60 秒后停止本 lane 自己启动的命令，未得到编译结论；rustfmt 组件未安装。STAR_RUN_OWNER_TEST_DATABASE_URL 未设置，未运行 PostgreSQL 集成验证。root/BFF/ACI locked offline metadata、目标依赖闭包与 MSRV 验收仍开放。 |
| W3 | P0 | **BOM、签名与 provenance** — Supply-chain owner | W0–W2 | 为每个选定发行物（Desktop 安装包、Web/Vercel build、三份 Canvas 镜像）及其实际依赖闭包生成 schema-valid SPDX 或核准 BOM；每份 BOM 绑定 artifact SHA-256/image digest。配置并运行 build provenance/签名/attestation，提供独立 consumer 验证输出。license inventory 仅作补充材料。生成/上传/验证任一步缺失均令 release gate 失败。验收为 artifact、BOM、attestation bundle 与验证记录的同一 release ID。 | OPEN：Desktop/Web 的 SPDX validator 与 workflow、Canvas digest-bound attestation 工作流完成本地静态/控制验证；Web npm SPDX 生成了 225 packages/315 relationships。Rust SBOM hosted 生成、真实 Desktop/Web/Canvas 构建产物、GitHub attestation、artifact/image digest 绑定后的独立 consumer 验证均未运行。 |
| W4 | P0 | **强制 CI 与 Action/工具完整性** — CI owner + GitHub repo owner | W0、W2、W3 | 将 cargo-audit placeholder digest 替换为经上游校验值；对发行闭包运行 license/advisory；确认 required-check 与最小权限；第三方 Actions 固定审核过的完整 SHA；上传/SBOM/attestation 缺失不得 `continue-on-error`。验收为源码 diff、所有目标 workflow 实际成功 runs，以及 branch protection required-check 证据。 | OPEN/BLOCKED：Wave 2 workflows 的 Action SHA、最小权限和 fail-closed 静态检查通过；hosted runs、branch protection/reviewer/environment 实际配置仍未核实。2026-10-31 `cargo-deny 0.20.2` 在 Rust 1.99.1 诊断扫描中发现 `webpki-roots 1.0.9` 命中 `RUSTSEC-2026-0381`；owner 修复及 Rust 1.99.0 基线复验尚缺。 |
| W5 | P0 | **桌面发行闭环** — Desktop owner | W0–W4 | 将 EOL 的 Node 20 换为受维护 LTS；锁定桌面 JS；为 Linux/Windows/macOS 输出稳定命名资产并上传到同一不可变 release；配置 updater 公钥与签名密钥管理（不在仓库内存 secret）；验证安装、更新、签名和回滚。Linux artifact 上传不得注释或 `|| true`。验收为三 OS 可下载签名 assets + BOM/attestation + 真实安装/升级测试原始记录。 | OPEN：stable Desktop workflow 已静态接入 Node 24 Vitest、SPDX 校验和 tag-only gated provenance；本机 Node 22.19 的前端 13 suites/62 项及 Agent View 定向 27 项测试通过。协调者曾报告本机全前端套件为 947 项中 929 项通过、18 项失败，但独立审计未找到全量运行日志；该数字仅作待复现线索，不作为发布验收证据。Node 24 hosted run、Rust/Tauri release package、BOM attestation、签名/公证、真实安装升级/回滚及 CSP 验证仍缺。 |
| W6 | P0 | **Web/Vercel 发布闭环** — Web owner | W0、W2–W4 | 统一 Web package manager/lockfile、Node 与构建命令；记录部署 source commit、build/runtime config、依赖/BOM 与 rollback 证据。验收需实际 Vercel deployment record 与消费侧验证；不得用静态配置推断部署已就绪。 | OPEN：Vercel build contract 和 Wave 2 Web workflow 的锁驱动 build、npm SPDX、证据绑定仅完成静态检查；本地 frontend `typecheck` 与 Next build 通过不等同于 hosted release run。Vercel Root Directory、真实部署、attestation consumer 验证和 rollback 仍未验证。 |
| W7 | P0 | **Star 容器/部署供应链** — Container + Infra owners | W0–W4 | 覆盖三份 Star Canvas 镜像，并审查 Envoy 等实际运行依赖；固定/记录 Rust 与 OS base image digest、OS package snapshot 与 Action SHA；生成每镜像 BOM/provenance/signature。将部署中的 `:dev`、`envoyproxy/envoy:v1.28-latest` 换为审核后的固定摘要/版本。收窄 Docker build context/`COPY` 到目标 Star 服务和已列明共享依赖，证明 `star-game` 等遗留源码不进入 builder context/cache，且 final layer 仅含目标 Star service/runtime dependencies。验收：构建输入 inventory、镜像 digest、独立 provenance/SBOM verify、K3s rendered manifest 指向同一 digest且无 game workload。 | OPEN：Canvas 发布 workflow 静态 allowlist 仅有 canvas-engine、domain-canvas、canvas-realtime；为各服务生成经过筛选的 Cargo context，拒绝非 Canvas 本地 workspace 依赖与含 game 的路径，并配置 SBOM/provenance。三份 Canvas Dockerfile 仅复制对应 crate 及生成的 workspace/lock/license。此为静态证据：Buildx 发布、真实镜像 digest、消费侧验证和 K3s 部署未执行。另一个 deploy/k3s-local/Dockerfile 仍广泛复制 crates/；根 Cargo workspace 列有 star-game，而 .dockerignore 会排除 star-game，因此该 Docker context 的完整性、兼容性及排除效果尚未验证。在确认用途并分类/过滤前，不能声称所有 Docker payload 均为 Star-only。deploy/helm/star 是已弃用且无活动 workload 的占位 Chart；活动 Canvas 部署清单为 deploy/star-canvas-services-k3s.yaml，仍含必须替换的 digest placeholders。依赖许可/NOTICE、受保护 hosted release environment、实际镜像与部署验收仍开放。 |
| W8 | P1 | **Multipass/K3s 测试文档与两实例验收** — Infra owner（与 D2 lane 集成） | W0、W7 | 使用正式版 Multipass 文档；移除 Docker Desktop 假设；核对 guest image、K3s、chart/add-on 的版本、license、摘要、维护和 BOM。测试前检查 inventory 必须 fail-closed 且最多 2 个实例。验收包括更新后的 README、两实例容量和停止/删除状态计数测试，以及集群升级/恢复日志；不得自动 purge。guest image 是测试环境依赖，不作为本次公开发行 payload。 | OPEN：Multipass 两实例的实机创建/容量/恢复验收尚未取得可复核记录；旧 D2/D4 活动 owner 状态需保持原样核实后再集成或清理。 |
| W9 | P0 | **Release candidate 验证与审批门** — Release owner + QA/SRE + compliance | W0–W8 | 对获批 candidate 运行每个入选目标的构建、功能/安装/升级/回滚、license/advisory、BOM/signature/provenance 验证，记录不可变 artifact digest 和远端 run IDs，逐项关闭审计 RC-01—RC-10。用户已条件授权：全部发行门通过且经最终复核后可推送稳定版本；这不授权当前提前创建 tag/release、build、publish、deploy 或 push。缺任一项即保持 OPEN。 | BLOCKED：依赖 W0–W8 全部完成。 |

## 后续工作安排（按依赖顺序）

下一步顺序：1）由 W1 当前 owner 解决 root workspace license 与用户 MIT 决策的冲突，补 root NOTICE，并完成 Desktop、Web/Vercel、Star Canvas 制品的第三方授权、归属、许可证、NOTICE/SBOM 与完整 payload 闭包审查。2）W2 owner 恢复可用的 Rust 1.99.0 工具链、安装匹配的 rustfmt、配置 PostgreSQL 测试端点，再运行 locked all-targets check、fixture tests 和 PostgreSQL 集成测试；不得把 1.99.1 等待 Cargo 锁的尝试当作验证结果。3）W1/W2 有可复核证据后推进 W3/W4 供应链和 required-CI 门，并并行完成 W5/W6/W7 的 Desktop、Web/Vercel、Canvas 真实构建、attestation 与部署验证；先分类或过滤 broad local K3s Docker context。4）W8 仍是独立测试环境门：最多使用两个 Multipass inventory 实例，取得创建/容量/恢复证据。5）仅在 W0–W8 关闭后执行 W9 candidate 验收。本次 WBS 更新不授权创建 tag/release、发布、部署或 push。

### 已登记的首轮并行 lane（stable-hardening set revision 28 集成前快照）

基线为本地 dev@04d8b7a84514dcb454874eb852651778e1452dea；本轮仅做发布闭环硬化，不关闭稳定版 WBS，也不执行 tag、发布、部署或 push。首个宽清单被 MAGOS 拒绝后，按 owner 冲突证据收窄 manifest；没有修改或清理被占用的分支/工作树。状态列对应本次 WBS 同步时的 MAGOS set 状态；最终集成以 set-show 与集成记录为准。

| Lane | 当前状态 | 分支 / 工作树 | 文件范围 | 目标与验收 |
|---|---|---|---|---|
| license-policy | active（WBS 最终同步） | agent/stable-hardening/license-policy / D:/orcaWork/Star/star-stable-hardening-license-policy | Chart.yaml、deny.toml、本 WBS、lane brief/report/audit | 目标：首方 Helm 元数据设为 MIT；保留完整依赖权利/NOTICE 审阅缺口。 |
| desktop-packaging | active（workflow follow-up） | agent/stable-hardening/desktop-packaging / D:/orcaWork/Star/star-stable-hardening-desktop-packaging | 独立稳定桌面 workflow、Tauri 前端 package/lock/config/build script、修复与 lane brief/report/audit | 目标：锁驱动安装、前端构建和测试；跨平台 hosted build、签名密钥、真实安装升级另验。 |
| web-vercel-config | ready（待集成） | agent/stable-hardening/web-vercel-config / D:/orcaWork/Star/star-stable-hardening-web-vercel-config | frontend/vercel.json、lane brief/report/audit | 显式 Vercel 构建合同；不改活跃依赖 lock owner 文件；真实 deployment 另验。 |
| canvas-oci | ready（待集成） | agent/stable-hardening/canvas-oci / D:/orcaWork/Star/star-stable-hardening-canvas-oci | 三份 Canvas Dockerfile、发布 workflow、K3s 镜像 manifest、.dockerignore、lane brief/report/audit | 目标：只含 Star Canvas 服务依赖，builder/context/cache 排除 star-game，部署固定 digest；OCI/K3s 实验另验。 |

保留边界：根 Cargo.toml 与 license-gate.yml 保持其活动 owner 所有权；Web 根 package-lock 仍属于活跃依赖 owner。桌面 lane 只更新隔离的桌面 package lock 并新增独立稳定构建 workflow，不覆盖旧桌面 workflow/owner 文件。Canvas 发布现需显式手动确认并绑定 star-stable-release environment，但仓库侧审查人/分支限制尚未验证；第三方依赖许可证/NOTICE 与真实 OCI/K3s readiness 仍未验证。W3 SBOM/provenance、W4 required-check 与 Action pin 证据、W8 D2/D4 的 Multipass 两实例真实验收、W9 candidate 验收仍 OPEN/BLOCKED。D4 活动终端及所有 W8、D2 文件保持原样。

## 最新集成状态（2026-10-31）

stable-hardening 首轮四个已审查 lane 已合入本地 `dev`：license-policy（`785155322e0f003c5cc6c5340d609d9e2673a680`）、desktop-packaging（`08a7b4fdb9db0d9c4ed0c0ff773d8833ccf8290e`）、web-vercel-config（`3880dc943775e68c40b5ee8553444135f0e64048`）、canvas-oci（`00404e38173801fd1b143c213eadbef2958093db`）。四个来源先以非快进合并进入 `agent/stable-hardening/integration`，再从原基线 `04d8b7a84514dcb454874eb852651778e1452dea` 快进到 `dev@782a7309997fe4fff7e522bb74639968431a51f6`；MAGOS GitPlan set 已释放。

集成后的 `git diff --check` 通过，容器构建/部署输入静态扫描仅发现用于排除 `star-game` 的断言和 `.dockerignore` 规则。相应 lane 的本地构建、测试与静态审查见各阶段报告；本次没有重新运行完整发行构建、hosted CI、签名/安装、真实 Vercel 部署或 OCI/K3s 验收。W0–W9 状态不变，稳定产品发行仍为 OPEN；没有创建 tag/release、构建或发布制品、部署或 push。

四个 lane 工作树及其四个来源分支、集成分支均已在确认干净、无人使用且提交已包含于 `dev` 后清理；其他工作树与分支未触碰。

## Wave 2 更新与验证快照（2026-10-31）

Wave 2 四个 lane 已按 MAGOS 计划合入本地 `dev@c2b3e5f07691bc657a3e7f67e65be9d8575d141a`，来源提交为：Desktop/SBOM validator `63ec043576603879452824967549d87ab789ee86`、Canvas/Web evidence `18469dd1e370fde9ed91c13f18e6a9d8cff7e7c2`、dependency audit `15e60d601afae5ab6556e1060f073538031fa38e`、Agent View cleanup `1e3938837ac0413c79de6ff8d72f7604d8a53934`。集成合并提交为 `71340b96458e31d6387fc980f477d5ca8fde5e68`、`ef2fb5d702e6eff8340f27fabe1b38a93f6f3e27`、`3526a779534dd1b0f712395a05bddc84b8303465`、`c2b3e5f07691bc657a3e7f67e65be9d8575d141a`。本地 `dev` 快进完成；本轮未 push、创建新 release/tag、发布 artifact 或部署。历史标签 `v0.79.0`–`v1.0.5` 存在且未触碰。

| Wave 2 lane | 阶段报告 | 已完成内容 | 验证边界 / 剩余缺口 |
|---|---|---|---|
| Desktop attestation (`63ec043`) | [`PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md`](../reports/PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md) | 增加发行许可 fail-closed gate、tag-only provenance/attestation job、SPDX 2.3 Rust/npm 校验器、root 与包数下限控制；本地 npm SBOM 为 225 packages/315 relationships，空数组和 224 包 mutation 被拒绝。 | hosted workflow 未运行；Cargo/Rust SBOM hosted 生成、实际安装包、attestation、consumer verification、签名/公证与外部 environment reviewers 均未验证；根 Cargo `Apache-2.0` / LICENSE MIT 冲突已验证。 |
| Canvas/Web evidence (`18469dd`) | [`PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md`](../reports/PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md) | 增加 Web build/SPDX 与三个 Canvas image digest-bound attestation workflow，要求人工 release input 与 protected environment。 | YAML、脚本语法和静态断言通过；hosted CI、Buildx/镜像发布、Vercel deployment、环境 reviewer/分支配置和 `gh attestation verify` 均未运行。 |
| Dependency audit (`15e60d6`) | [`PHASE-STAR-STABLE-W2-DEPENDENCY-AUDIT-20261031.md`](../reports/PHASE-STAR-STABLE-W2-DEPENDENCY-AUDIT-20261031.md) | 对 owner PR 与依赖边界形成只读审查记录。 | PR owner 改动没有在本 lane 合并；依赖锁/advisory 和许可证审查仍未闭合。 |
| Agent View core (`1e39388`) | [`PHASE-STABLE-RELEASE-W2-AGENT-VIEW-CORE-20261031.md`](../reports/PHASE-STABLE-RELEASE-W2-AGENT-VIEW-CORE-20261031.md) | Web Agent View 移除旧游戏机制与专属前端模块，保留中性的 Agent/Worktree/Canvas/任务关系展示；focused tests 27 项、typecheck、build（52/52 静态页）通过。 | 当前 Agent Session 仍来自 seed，未接入经授权的实时 provider/生产数据；不代表 Agent CLI、runtime、授权、取消/恢复或独立验证已可用。 |

协调者报告合并后的本机全前端 Vitest 为 947 项中 929 项通过、18 项失败，BFF 检查/88 项测试通过，以及 root Rust workspace check 在 schedule_admission.rs:638 失败；独立审计未找到这三项结果的持久化原始命令、完整输出和环境摘要，因此均标记为待复现/不可作为验收证据。审计静态确认 branch_id 源缺陷存在且未被 Wave 2 修改。本机 Rust 1.99.0 不可用及全仓 fmt 既存差异也是协调者报告，尚需补可追溯记录。Agent View lane 的 27 项定向测试、typecheck 与 52/52 页面构建有阶段报告记录；这些局部证据不替代完整全前端或发行验证。W0–W9 仍未闭合。

独立核对见 [PHASE-STAR-STABLE-RELEASE-WBS-EVIDENCE-AUDIT-20261031.md](../reports/PHASE-STAR-STABLE-RELEASE-WBS-EVIDENCE-AUDIT-20261031.md)。审计确认 MIT 元数据冲突与 Agent View seed 数据边界，并指出全前端/BFF/root compiler 协调者运行结果缺少本仓持久化输出；需补存完整命令、工具链、退出码、摘要与失败分类后才能用于验收判断。

四个 Wave 2 来源分支及对应 worktree 在本地 Git 中仍可见：`agent/stable-release-wave2/desktop-attest`、`canvas-web-attest`、`dependency-audit`、`agent-view-core`。Orca owner 查询返回 `runtime_access_denied (EPERM)`，因此无法确认是否无人使用；本轮未删除或清理这些 lane。该状态不影响已合入的本地 `dev`，但须在取得可靠 owner 状态后单独复核清理。

## 外部约束依据（截至 2026-10-31）

- Rust 官方 notes 显示 1.99.1 于 2026-10-15 发布；Star baseline 1.99.0仍是用户声明的最低版本，但是否升 patch 需在 W2 决策。[Rust Release Notes](https://doc.rust-lang.org/releases.html)
- Node 官方状态表显示 v20 于 2026-03-24 EOL；稳定版 Desktop/Web workflow 已配置 Node 24.22.2，但旧的 `star-desktop-build.yml` 仍有 Node 20 job；所有目标还需在 hosted runner 上验证。[Node.js releases](https://nodejs.org/en/about/previous-releases)
- Canonical 官方 release list 显示 Multipass 1.17.0 于 2026-10-19 GA；当前 README 仍是 1.17.0-rc1。[Multipass releases](https://github.com/canonical/multipass/releases)
- GitHub 可对 binary/container artifact 生成 provenance/SBOM attestation，但要求把实际 artifact path 或 image digest 与 BOM 绑定；这只说明平台支持，不证明 Star 已实现。[GitHub how-to](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)

## 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | W16-B 子代理（Mavis 接手审核待协调者复核） | 按 W16-B 审计发现制定稳定发行 WBS；所有构建、发布和部署证据保持 OPEN。 | W16-B audit |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录用户确认的 Desktop + Web/Vercel + 三份 Star Canvas 镜像发行面及 MIT canonical license；把 W5–W7 标为无条件 P0、W8 的两实例运行验收保持 OPEN，并记录仅在所有门禁通过后推送稳定版的条件授权。 | 用户完成发行范围与许可证决策 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | W16-B 审计/WBS 已经父级审阅并快进集成到本地 `dev@163f715`；稳定版门禁仍为 OPEN。 | W16-B 文档 lane 完成集成 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 登记并启动 stable-hardening 首轮四个隔离 lane；按 MAGOS 实际 owner 冲突证据收窄清单，完整标明未解除的 Cargo/license workflow 与 Dependabot 锁文件阻塞；没有关闭稳定版门禁。 | 用户确认发行范围与 MIT，并要求整理后发布稳定版 |
| v0.5 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 stable-hardening manifest revision 3 与当前执行状态：Web/Vercel 静态配置 lane 已 ready、待集成；license、desktop、Canvas OCI 仍进行中；release gates 不变且仍 OPEN/BLOCKED。 | Web lane 完成独立静态复核 |
| v0.6 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 `cargo-deny 0.20.2` 在 Rust 1.99.1 诊断扫描中发现 `webpki-roots 1.0.9` / `RUSTSEC-2026-0381`；声明该结果不替代 Rust 1.99.0 基线验收，且锁文件修复需要 owner 审查。 | 本轮 W1 policy 复核发现现有 advisory 阻塞 |
| v0.7 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 stable-hardening revision 24 的实际切片：Chart/deny 为 MIT，桌面隔离 lock/build/frontend tests 已实现，Vercel 配置通过首轮审查；桌面 CI 与 Canvas 发布审批 follow-up 正在进行。按证据更新 W1/W2/W5/W6/W7；所有 hosted CI、法律/NOTICE、签名、部署、OCI digest 和两实例验收仍开放。 | 用户确认范围与许可证后，四条并行 lane 已完成首轮 |
| v0.8 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 stable-hardening revision 28 的独立审查结果：桌面三平台 workflow 在 Node 24 构建前运行 Vitest 并关闭 checkout 凭据持久化；Canvas tag push 不再自动发布，只有版本 tag 的明确批准 dispatch 可请求发布并绑定外部保护 environment。工作流静态检查通过，hosted CI 与外部 environment 配置仍未核实；WBS 将 W5/W7 缺口和 lane 状态更新为集成前快照。 | 独立 lane 审查发现发布 workflow 缺少相应门禁 |
| v0.9 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 stable-hardening 四个已审查 lane 按序进入集成分支并快进到本地 dev@782a730；MAGOS GitPlan set 已释放。记录集成差异检查及 `star-game` 排除扫描结果，同时保持 W0–W9 未闭合项与稳定发行 OPEN；未创建 tag、发布、部署或 push。 | 四个 stable-hardening lane 完成 dev 集成 |
| v1.0 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录四个 lane 工作树及五个已合入本地 `dev` 的 lane/集成分支已按清理门禁移除；其余 worktree、`main` 和远端均未修改。 | 四个 stable-hardening lane 完成合并清理 |
| v1.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 以 Wave 2 集成后的 `dev@c2b3e5f` 为快照同步 W0–W9，记录四个 Wave 2 lane 与本地验证边界；明确 MIT 与根 Cargo metadata 的实际冲突、seed 数据和 hosted/发行制品验收缺口；Wave 2 branch/worktree 因 Orca owner 状态不可查而保留。稳定发行仍 OPEN/BLOCKED，未 push 或发布。 | Wave 2 已合并 dev，需把真实实现与验证结果同步回主 WBS |
| v1.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按接受的 integration@cdb66b5b 同步 W1 metadata 与 W2 Schedule fixture 证据；记录该证据快照时的目标基线为 dev@bde4e9d8，尚未包含 integration 与本 WBS lane。W1 因 root license/NOTICE、第三方授权与各目标制品闭包未完成而 BLOCKED；W2 因 Rust 1.99.0、rustfmt、PostgreSQL 端点与实际验证未完成而 OPEN/BLOCKED。补充 Canvas 专用筛选 context 与 broad local K3s Docker context 的证据边界，保留全部 W0–W9 未关闭。main 独有的五个提交均与 dev patch-equivalent，没有可安全摘取的唯一内容。 | W1/W2 lane 已接受，需更新 stable release WBS 与 Docker payload 状态 |
