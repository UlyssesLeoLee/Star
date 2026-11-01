# PHASE-STAR-STABLE-W4-CI-GATE-AUDIT-20261031

> **状态：OPEN / BLOCKED。** 本审计不授权或证明创建 tag、release、artifact、镜像或部署。

## §0 目的

按已登记 brief，对 Desktop、Web/Vercel、三份 Star Canvas 镜像的 CI、依赖锁、Actions、工具版本、镜像输入、权限、失败处理及外部保护作只读审查。基线为本地 dev@2a57fa0d329be830735d6657677236e195673362；本 lane 起始提交 2ad2179e4071729028ea8eb77fad31def8701ed2 只含 brief/dispatcher。未修改工作流、manifest、lock、Dockerfile、产品源码或部署文件；未访问凭据、触发构建、部署或发布。W0–W9 与整体稳定版仍 OPEN（WBS:3,5）。

## §1 改动矩阵 / 发行目标审计矩阵

| 范围 | 状态 | 证据与结论 |
|---|---|---|
| 通用 CI | **BLOCKED** | .github/workflows/ci.yml:1-9 对 dev/main 的 push 和 PR 触发；无显式 permissions 或 concurrency。Rust 做 workspace all-targets check，但 test 只跑 star-context --lib（:13-48）。Clippy 名称虽为 advisory，步骤没有 continue-on-error，实际是强制项（:39-43）。E2E 只在 main push（:77-96）。Frontend typecheck/test/build 的失败被 continue-on-error 忽略（:124-158）；doc/bench 同样可忽略失败（:187-230）。Action refs 使用可移动 tag。 |
| License/advisory gate | **BLOCKED** | license-gate.yml:30-31 仅显式授予 contents: read。cargo-deny 0.20.2 有固定 SHA-256（:41-57）；cargo-audit 0.22.2 的 SHA 仍为占位，安装步骤在校验前失败关闭（:95-119），实际 scanner 无法运行。cargo-machete job 可失败而不失败整个 workflow（:127-143）。license inventory 导出/上传可以 continue-on-error，且缺少 inventory 文件为 ignore（:70-86）。该 workflow 没有被 stable artifact workflow 显式设为依赖。 |
| Desktop Linux .deb/AppImage | **PARTIAL；发布 BLOCKED** | stable Desktop workflow 固定 Action SHA、Node 24.22.2 和根 Rust 1.99.0；跑前端 npm ci、Vitest、构建及 SPDX 生成，缺 artifact 会失败（star-stable-desktop.yml:75-170）。Tauri build 未传 --locked（:127-129）；workspace lockfile 因此未被构建命令强制。当前 Cargo.toml workspace license=Apache-2.0，而 release metadata gate 要求 MIT（:333-366 与 Cargo.toml:183-187），当前候选会被拒绝。没有本地基线的 hosted build/attestation。 Artifact names are explicitly unsigned; this workflow provides provenance attestation but no installer signing/notarization (star-stable-desktop.yml:164,244,325). |
| Desktop Windows MSI | **PARTIAL；发布 BLOCKED** | 同一 workflow 的 Windows runner/build、npm ci、前端测试和 SPDX 检查位于 :173-249；Tauri build 未传 --locked（:209-211）；缺 artifact 会失败（:241-249）。MIT metadata gate 和 hosted run 缺口同上。 Artifact names are explicitly unsigned; this workflow provides provenance attestation but no installer signing/notarization (star-stable-desktop.yml:164,244,325). |
| Desktop macOS universal DMG | **PARTIAL；发布 BLOCKED** | macOS runner/build、npm ci、前端测试和 SPDX 检查位于 :252-330；Tauri build 未传 --locked（:290-292）；缺 artifact 会失败（:322-330）。MIT metadata gate和 hosted run缺口同上。Stable release attestation 仅在手动 APPROVED + v* tag 后执行，并依赖相应 build 与 metadata gate（:335-507）。 Artifact names are explicitly unsigned; this workflow provides provenance attestation but no installer signing/notarization (star-stable-desktop.yml:164,244,325). |
| Web/Vercel | **PARTIAL；Vercel 发布 BLOCKED** | star-stable-web.yml:3-4 明示只生成 Web build evidence bundle，不做 Vercel deploy。Node 固定 24.22.2，npm ci 使用 frontend/package-lock.json，生成 npm SBOM，上传缺文件即失败并 attestation（:35-131）。frontend/package-lock.json 与 frontend/pnpm-lock.yaml 同时受跟踪；workflow 只用 npm lock，frontend/package.json 没有 packageManager 字段（package.json:1-6；lockfile header:1-4）。没有真实 Vercel deployment、消费侧验证或 rollback 记录。 |
| Canvas canvas-engine | **PARTIAL；发布 BLOCKED** | publish workflow 的矩阵只允许三个 Canvas crate（:46-60），为每个 crate 生成筛选 workspace 与 Cargo.lock，并拒绝非 Canvas 依赖、game 路径（:75-249）。check/test 使用 --locked（:41-44），Docker build/push 以 digest 校验、SBOM、provenance 和 digest-bound attestation 结束（:271-312）。Dockerfile 的 rust/debian base image 均固定 sha256（crates/canvas-engine/Dockerfile:4,10）。但当前 MIT metadata gate 因根 Cargo.toml 的 Apache-2.0 会失败（workflow:64-73；Cargo.toml:187），也没有 hosted build/consumer verification。 |
| Canvas domain-canvas | **PARTIAL；发布 BLOCKED** | 与 canvas-engine 相同的 allowlist、locked build、digest/SBOM/attestation 静态配置；crates/domain-canvas/Dockerfile:4,10 固定 rust/debian base digest。实际发布、许可 gate 与 consumer verification 未通过/未运行。 |
| Canvas canvas-realtime | **PARTIAL；发布 BLOCKED** | 与 canvas-engine 相同的 allowlist、locked build、digest/SBOM/attestation 静态配置；crates/canvas-realtime/Dockerfile:4,10 固定 rust/debian base digest。实际发布、许可 gate 与 consumer verification 未通过/未运行。 |
| 非目标 deploy/k3s-local image | **需单独分类；不属于三份 Canvas 镜像矩阵** | stable Canvas workflow 只接受 canvas-engine/domain-canvas/canvas-realtime，未调用 deploy/k3s-local/Dockerfile（publish workflow:60,92-98,240-246）。该 Dockerfile 使用浮动 rust:slim-bookworm 与 debian:bookworm-slim、未固定 apt 包快照并 COPY crates ./crates（deploy/k3s-local/Dockerfile:1-10）。这不证明三份 Canvas context 有污染；但用途与产品边界、context 与依赖需另行处理，不能据三份 Canvas 结果宣称仓库所有 Docker payload 都满足 Star-only。 |

### §1.1 Actions、toolchain、runner 和 base image 固定情况

- **稳定 Desktop actions**：star-stable-desktop.yml:82,180,259,341 固定 checkout SHA 3d3c42e5aac5ba805825da76410c181273ba90b1；:87,185,264 固定 setup-node SHA 820762786026740c76f36085b0efc47a31fe5020；:162,242,323 固定 upload-artifact SHA cf430e030ddbb5b0abf93d22962f4752f3646cd9；:382,435,480 固定 download-artifact SHA 9000827ccba6bdab643e8b6fd33ac0654aef8333；:400,407,415,451,456,462,496,501,507 固定 attest SHA 1e69f48acb82d1966a394da916b4c1698aa569d6。均为完整 commit SHA；本审计未核验 SHA 与上游 release/tag 的对应关系或审查 action 源码。
- **稳定 Web actions**：star-stable-web.yml:36,56,115,126,131 使用相同 checkout/setup-node/upload-artifact/attest 完整 SHA。
- **Canvas actions**：publish-star-canvas-services.yml:36,62,253,260,264,273,298 分别固定 checkout 11d5960a326750d5838078e36cf38b85af677262、Rust toolchain action ff7f6a3374ce205788da40198ec918bd5d67aea7、docker/login c94ce9fb468520275223c153574b00df6fe4bcc9、setup-buildx 8d2750c68a42422c14e847fe6c8ac0403b4cbd6f、metadata c299e40c65443455700f0fdfc63efafe5b349051、build-push ca052bb54ab0790a636c9b5f226502c73d547a25、attest 1e69f48acb82d1966a394da916b4c1698aa569d6。setup-buildx 没有配置 version 输入（:258-263）；action 源码固定，但 Buildx 版本未由此 workflow 固定。
- **通用 CI / license gate**：ci.yml:17,20,25,53,56,61,83,86,89,106,109,112,130,133,171,174,192,195,198,217,220,225 使用 checkout@v4、dtolnay/rust-toolchain@1.99.0、rust-cache@v2、setup-node@v7、markdownlint action@v19 等移动 tag；license-gate.yml:44,48,82,98,101,134,137 也使用 v4/1.99.0 tag，未固定为完整 SHA。
- **Legacy desktop workflow**：star-desktop-build.yml:34-179 使用移动的 checkout/setup-node/upload-artifact/rust-cache/toolchain tags；Node 版本写为 20（:97-99,156-158），MSI/DMG 上传找不到文件时只 warn（:125-130,174-179）。该工作流会在 dev/main 触发并保存临时 Actions artifact，不是 stable Desktop tag-only attestation 工作流，不能代替稳定发行证明。
- **锁和工具**：rust-toolchain.toml:4-6 将 Rust channel 固定为 1.99.0，Cargo.toml:185 的 MSRV 同值；stable Desktop/Web 固定 Node 24.22.2。Desktop cargo-sbom 固定 0.10.0 并使用 --locked（star-stable-desktop.yml:112-113 等），但没有显式二进制 SHA。cargo-deny 固定版本和 SHA-256；cargo-audit SHA 仍为占位；cargo-machete 0.9.2 --locked 但失败可忽略。根 Cargo.lock 被 Canvas context builder 筛选使用。Web root 同时存在 npm 与 pnpm lock，且 package.json 未声明 packageManager。
- **Hosted runners**：Desktop 使用 ubuntu-24.04、windows-2025、macos-15（desktop workflow:75,173,252），Web 用 ubuntu-24.04（web:25），Canvas 用 ubuntu-latest（canvas:34,50）。runner 标签不是 VM image digest；runner 上的预装工具、Docker Engine/Buildx、Python/npm、apt 索引等未固定到镜像摘要。
- **入选 Canvas base images**：三个 Dockerfile 使用 rust:1.99.0-bookworm@sha256:114c7a4425406451c2866b6aafe69fe29b1b298832db1277d411ac73c82d04d6 与 debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3（各 Dockerfile:4,10）。context builder 复制 allowlisted crate、生成 manifest、筛选 lock、toolchain 和 LICENSE（publish workflow:95-112,163-246）。
- **k3s-local image**：deploy/k3s-local/Dockerfile:1,8 的 rust:slim-bookworm/debian:bookworm-slim 未带 digest；:3,9 apt-get 没有固定 package snapshot；:5 复制整个 crates 目录。它不在已批准 Canvas image 矩阵内。

### §1.2 门禁、权限、失败行为与并发

- Desktop 的 release metadata/attestation 要求 manual dispatch、APPROVED、v* tag（star-stable-desktop.yml:335-375）；三个 attestation job 需要各自构建和 metadata gate，并指定 star-stable-release environment。上传缺少输入文件会失败（:162-170,242-249,323-330）。当前 Cargo metadata=Apache-2.0，而用户已选择 MIT，Desktop 与 Canvas 的 release metadata gate 会失败（desktop:357-366；canvas:68-73；Cargo.toml:187）。
- Web 仅 workflow_dispatch，shell 检查 tag 和 APPROVED（star-stable-web.yml:5-20,35-52）；生成 artifact/SBOM 和 attest，不部署 Vercel（:74-145）。缺 artifact 失败（:114-123）。
- Canvas 在 push/tag 时运行测试；publish job 只能由 APPROVED 版本 tag 的 workflow_dispatch 触发且 needs test（publish-star-canvas-services.yml:9-20,31-56）。之后校验 MIT、image allowlist 和过滤 lock/context，并检查镜像 digest 格式，生成 SBOM/max provenance 和 digest-bound attestation（:64-73,75-249,271-312）。无 concurrency 配置，同 tag 重复 dispatch 可并行；标签写入有竞态风险。
- CI/license workflow 无 concurrency；Desktop 对同 ref cancel-in-progress=true（star-stable-desktop.yml:66-68），Web cancel-in-progress=false（star-stable-web.yml:18-20）。并发策略不能替代 required checks。
- APPROVED 只是人工输入，YAML 没有校验 W0–W8 是否已关闭，也没有将 license/advisory workflow 的成功状态作为 stable artifact jobs 的依赖。仓库 workflow 中的 environment 声明本身也不能证明外部 reviewer 或 ref 限制。

### §1.3 GitHub 外部配置与 hosted runs

查询快照：2026-11-01 05:58 UTC，通过未认证公开 REST GET。

- Rulesets API 返回 StarMain (22405373) 与 StarMainD (22405411)，均 enforcement=disabled；详情只有 deletion/non_fast_forward 规则，没有 required status checks：[StarMain](https://api.github.com/repos/UlyssesLeoLee/Star/rulesets/22405373)、[StarMainD](https://api.github.com/repos/UlyssesLeoLee/Star/rulesets/22405411)。
- GET /repos/UlyssesLeoLee/Star/environments 返回 HTTP 200、total_count=0；当时未配置 star-stable-release environment/reviewer rules。仅在 YAML 引用环境名不等于设置了审批保护。
- GET /branches/dev/protection 与 /branches/main/protection 返回 HTTP 401。无法从未认证请求判定 legacy branch protection；不记作“无保护”或“已保护”。
- 远端 dev ref 为 40a84c1aaf804267ee25e02384e99276c3151906；本地基线 2a57fa0d... 相对 origin/dev ahead 229 / behind 0。Actions API 对基线和 brief commit 2ad2179e... 均返回 0 runs。
- 远端 dev 最新运行在 2026-10-24，基于旧 SHA 40a84c1a。CI [40142045777](https://github.com/UlyssesLeoLee/Star/actions/runs/40142045777) 失败：cargo check、Markdown lint、Ubuntu smoke、BFF cargo fmt 失败；Windows/macOS smoke、Frontend、Rust doc、bench 成功，E2E skipped。License gate [40142045790](https://github.com/UlyssesLeoLee/Star/actions/runs/40142045790) 的 cargo-audit/cargo-deny fetch 和 cargo-machete job 失败。旧 [Publish canvas-game 40142045772](https://github.com/UlyssesLeoLee/Star/actions/runs/40142045772) 是历史不同 workflow，不证明当前三份 Canvas image 构建。
- 远端数据仅为本次公开 REST 快照；branch protection 不可见，组织级设置未由这些 endpoint 覆盖。

## §2 验证摘要

审计命令和数据读取：
- rtk proxy git status --short --branch；rtk proxy git rev-parse HEAD；rtk proxy git rev-parse origin/dev；rtk proxy git rev-list --left-right --count 2a57fa0d329be830735d6657677236e195673362...origin/dev。
- rtk proxy rg --files .github/workflows；rtk proxy rg --files；rtk proxy git ls-files，用于定位 workflows、Cargo/npm lock 和 Dockerfiles。
- rtk proxy rg -n 针对 uses、permissions、concurrency、continue-on-error、if-no-files-found、runs-on、APPROVED、cargo install、npm ci、--locked、FROM、COPY、sha256 与发布条件，复核表中路径和行号。
- rtk proxy py -3 只读读取 brief、manifest/package-lock 头、toolchain；并 GET GitHub REST rulesets（列表/详情）、branch protection、environments、Actions runs/check-runs/status/jobs。
- 未运行 cargo/npm/Docker build/test；未请求 GitHub 写入、触发 workflow、创建 tag/release、发布镜像、部署或 push。

本报告是静态审查加公开状态快照，不是构建或测试通过证据。当前本地候选没有 hosted run。

## §3 已知缺口与下一步工作

| 优先级 | 缺口 | 验收标准 |
|---|---|---|
| P0 blocker | cargo-audit digest 占位且 release jobs 不依赖 license/advisory 成功。 | 可信上游校验固定二进制；用 Rust 1.99.0 扫描实际发行闭包；让成功结果成为 release required gate；留存真实 run 与结果。 |
| P0 blocker | CI/license actions 使用 tag、CI 没有最小 permissions、frontend/doc/bench 失败可被忽略。 | 所有第三方 action 使用经审核完整 SHA；每 workflow/job 显式最小权限；将 release 必需任务失败设置为阻断。 |
| P0 blocker | Active ruleset disabled、environment endpoint 当前为空、branch protection 未能读取。 | GitHub owner 配置 required checks、stable release environment reviewer/ref rules；用有权限身份复核并保存有效配置证据。 |
| P0 blocker | Desktop build 未强制锁 Cargo；Web 根有 npm+pnpm 两份锁且 package manager 未声明。 | Tauri build 加锁定模式；明确唯一 package manager/lockfile 并解决其他锁文件；按锁安装和审阅完整依赖图。 |
| P0 blocker | MIT 决策与根 workspace Apache-2.0 冲突，stable Desktop/Canvas 的 metadata gate 必失败。 | 同步 Cargo metadata/法律文本与已选 MIT；重跑 package metadata 与全部发行 payload 检查。 |
| P0 blocker | 当前本地 source commit 没有 hosted CI/release runs。 | 固定 candidate source，对每个目标执行受保护 hosted workflow，记录成功 checks、artifact/image digest、BOM/provenance 与消费侧验证。 |
| P0 blocker | Web workflow 只是 evidence bundle；k3s-local image/context 范围待确认。 | 提供实际 Vercel deployment/rollback evidence；对 k3s-local 明确仅测试/开发用途或纳入产品时收窄 context、固定 base/OS 包并核验 payload。 |
| P1 hardening | Buildx 版本/runner 镜像未固定，Canvas publish 没有并发控制；cargo-sbom 仅版本锁；diagnostic artifact 可缺省。 | 明确 runner/buildx 更新策略并留再现证据；按 image digest 设计同 tag 并发/幂等；发布必需 SBOM/provenance 缺失要失败。 |
| P1 hardening | Stable Desktop 构建没有 Desktop Rust unit test；通用 Rust test 仅测 star-context；Canvas 为三个 crate 测试。 | 为每目标定义适当集成测试矩阵，关联每个 artifact 的真实执行记录。 |

W4 关闭还需要七组发布目标（Desktop 三平台组、Vercel、Canvas 三镜像）的同一 source hosted evidence；此审计不关闭 W0–W9。

## §4 子代理失败接手清单

没有派生子任务或子代理；本 lane 只审查既定清单和外部只读 GET。

## §5 守门规则

1. 不把 YAML 步骤当作 GitHub required checks 或环境审核配置。
2. 不用旧 SHA/旧 Canvas 游戏 workflow、静态检查或移动 Action tag 代替当前候选 hosted 证据。
3. 不把 Web evidence bundle 描述成 Vercel deployment；不把 k3s-local 作为三份 Canvas 镜像之一。
4. Action SHA、lockfile、base image digest 各自不等于完整 provenance、签名或消费侧验证。
5. License/advisory、metadata、SBOM、上传或 attestation 缺失均须阻断发布。
6. W4/稳定版保持 OPEN；此 lane 不发布、不部署、不推送、不创建 tag/release。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 仅审查配置与外部快照，不批准 release。 |
| SRE Lead | SRE Lead（Mavis 接手审核）— 候选 hosted run 缺失；需补 required checks/environment。 |
| 平台 | 平台（Mavis 接手审核）— 稳定 workflow Action SHA/Canvas base digest 有静态 pin；通用 CI、Buildx、runner 与 k3s-local 有可变输入。 |
| 评审主持 | 评审主持（Mavis 接手审核）— branch protection 401 未知；environment endpoint 快照为空；W4 OPEN/BLOCKED。 |
| PM | PM（Mavis 接手审核）— 不授权提前发布。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复核通用 CI、license gate、稳定 Desktop/Web/Canvas workflows、三份 Canvas Dockerfile 与 k3s-local；记录 action/tool/base-image pins、各 artifact gate、GitHub ruleset/environment/hosted-run 限制，保持 W4 与稳定版 OPEN。 | STAR-STABLE-W4-CI-GATE-AUDIT-20261031 brief |
