# PHASE — Star Stable W3 release-payload boundary audit

**状态:** **W3 OPEN；当前审计基线不是 stable-release-ready。** 当前 Canvas release workflow 静态上为三项服务建立独立、受 allowlist 控制的 Cargo context；但 workspace license/version 与批准的 MIT / `v1.0.6` 身份不一致，release gate 对当前基线将失败，且没有本轮真实 BuildKit archive、最终 image、SBOM 或 provenance 验收证据。

**工作基线:** `dev@a272e9794e1a1975781720c9b3775c914f47f546`

**只读候选:** `codex/star-stable-w7-k3s-context-20261031@c960b9193cc960ae3d85711472a0d4f31fedd0b3`

**任务 brief:** `docs/briefs/W3-STAR-PAYLOAD-AUDIT-20261101.md`

## §0 目的

按 W3 brief 对 Star 容器/build-context 边界进行只读静态审计，区分 stable Canvas release path、local/development path、mock/test image 和 deprecated deployment configuration，并给出不能关闭 W3 的证据缺口。批准范围是 Desktop、Web/Vercel 与三份 Star Canvas 服务镜像；本报告只审核容器和构建上下文路径。

读取通过 RTK `read` / `rg` / `proxy git` 完成。没有运行 Docker、BuildKit、Cargo build、测试、Multipass、K3s 部署或 registry 查询；没有检查真实 BuildKit context archive、最终 OCI image、attached SBOM 或 provenance。静态配置和候选报告中的自述结果均不冒充本轮制品实证。

## §1 审计矩阵

| 路径 / 对象 | 分类 | 审计结论 |
|---|---|---|
| `crates/canvas-engine/Dockerfile`, `crates/domain-canvas/Dockerfile`, `crates/canvas-realtime/Dockerfile` | 批准的 stable Canvas image build path | Publish workflow 的 matrix 只列三项；每项独立生成 context，检查 workspace/local package closure、拒绝 game/foreign crate，并只复制一项 crate 的 manifest 与 `src`。Dockerfiles 使用 digest-pinned Rust/Debian base，最终 stage 只复制单个 executable 与 LICENSE。以上均为源代码静态结论，不代表 archive/image 验收。证据：`.github/workflows/publish-star-canvas-services.yml:64, 94-128, 206-250, 279-308`; 三份 Canvas Dockerfile:4-14。 |
| `.github/workflows/publish-star-canvas-services.yml` | Stable Canvas publish workflow | 需要手动 dispatch、`APPROVED` 输入及任意 `v*` tag；工作流要求 Cargo workspace license 和 `LICENSE` 首行均为 MIT，但当前 `Cargo.toml` 是 `Apache-2.0`，会在 context assembly 前失败。OCI semver tag 来自 Git ref，但生成 Cargo manifest 仍继承 `0.1.0`。证据：workflow:48, 68-76, 150-158, 272-290；`Cargo.toml:182-188`。 |
| `deploy/star-canvas-services-k3s.yaml` | Stable release render template | 三个 Star image 引用使用必填 digest marker，文件明示实际 release digest 尚未填入；Envoy runtime dependency 则固定为 digest。没有真实 digest / SBOM / provenance 对应关系证据。证据：`deploy/star-canvas-services-k3s.yaml:1-4, 49, 118, 187, 346`。 |
| `deploy/k3s-local/Dockerfile`, `deploy/k3s-local/build-and-deploy.sh` | Local/development `star-api-rest` image path；不是批准的 stable image set | 现行脚本把完整 `crates/` 复制进临时 context，且没有复制根 `.dockerignore`；Dockerfile 对 builder 使用 `COPY crates ./crates`。最终 runtime stage 仅复制 API executable，但 context/builder 输入未限制到依赖闭包。部署 manifest 使用 `star-api-rest:local`。证据：script:18, 21-30; Dockerfile:1-13; `.dockerignore:1-13`; `star-api-rest-deploy.yaml:30-32`。 |
| `deploy/k3s-local/envoy/deployment.yaml`, `deploy/k3s-local/bff-deployment.yaml` | Local development manifests | 引用 Envoy `v1.31.0` / `v1.31-latest` 与 placeholder `bff:local`；未进入 stable Canvas build matrix。证据：各文件 `image` 行。 |
| `tools/star-flash-mock/k3s/envoy-deployment.yaml` | Mock/test infrastructure | 明示用途是 `tools/star-flash-mock`，并引用 mutable `v1.32-latest` Envoy；不是 stable release build path。证据：文件:1-4, 35-38。 |
| `deploy/gitops/kargo/kargo-dev-staging.yaml` | Canvas image dev/staging promotion | Warehouse 追踪三份 Canvas 镜像；dev/staging 晋级不是 image build 或 stable release 证据。证据：文件:1-5, 23-56, 126-135。 |
| `deploy/helm/star` | Deprecated placeholder | `Chart.yaml` 声明该 chart deprecated 且没有 active workload templates；不是当前 Canvas deployment path。证据：`Chart.yaml:1-6`; `values.yaml:1-4`。 |

Tracked inventory 检出四份 Dockerfile 和一个 `.dockerignore`，未检出 Containerfile 或 Docker Compose build definition。仓库中其他 image references 已列于 review；它们属于 local/dev/mock/runtime dependency，不扩展 stable Star image allowlist。

## §2 验证摘要

所有命令均在分派 worktree `C:\Users\leo19\.codex\worktrees\star-stable-w3-payload-audit-20261101\Star` 中执行。

| 命令 | 结果 |
|---|---|
| `rtk proxy git status --short --branch` | 初始 `## codex/star-stable-w3-payload-audit-20261101`，worktree clean。 |
| `rtk proxy git rev-parse dev` | `a272e9794e1a1975781720c9b3775c914f47f546`，与 brief 基线一致。 |
| `rtk proxy git rev-parse codex/star-stable-w7-k3s-context-20261031` | `c960b9193cc960ae3d85711472a0d4f31fedd0b3`，与 brief 候选一致。 |
| `rtk proxy git diff --name-status a272e9794e1a1975781720c9b3775c914f47f546..c960b9193cc960ae3d85711472a0d4f31fedd0b3` | 全量候选差异为 30 files、1,358 insertions、887 deletions：含两个 workflow、local Dockerfile、context generator/tests、WBS 修改及多个 brief/report/dispatch 删除与替换。没有修改候选。 |
| `rtk proxy git diff --stat a272e9794e1a1975781720c9b3775c914f47f546..c960b9193cc960ae3d85711472a0d4f31fedd0b3` | 同上；不将候选整体变化视为可接受集成。 |
| `rtk proxy git ls-files '*Dockerfile*' '.dockerignore*' '*compose*.yml' '*compose*.yaml' 'Containerfile*'` | `.dockerignore`、三个 Canvas Dockerfile、一个 `deploy/k3s-local/Dockerfile`；无 compose/Containerfile。 |
| `rtk read -n .github/workflows/publish-star-canvas-services.yml` | 静态确认三镜像 allowlist、生成 context、依赖闭包检查、MIT release gate、Buildx context 路径、`sbom: true` / `provenance: mode=max` 与 digest attestation 配置。 |
| `rtk rg -n -C 2 '^version =|^license =' Cargo.toml crates/canvas-engine/Cargo.toml crates/domain-canvas/Cargo.toml crates/canvas-realtime/Cargo.toml` and `rtk read -n --head-lines 2 LICENSE` | Workspace version `0.1.0`、license `Apache-2.0`；LICENSE 首行为 `MIT License`。 |
| `rtk read -n deploy/k3s-local/Dockerfile deploy/k3s-local/build-and-deploy.sh .dockerignore` | 现行 local context 复制整棵 `crates/`；临时目录不包含 `.dockerignore`；Dockerfile broad-copy 所有 crates。 |
| `rtk rg -n 'image:' deploy -g '*.yaml' -g '*.yml'` | 列出三项 digest-placeholder Canvas image、digest-pinned Envoy、local API、local Envoy/BFF，以及 Kargo image subscriptions。 |
| `rtk proxy git tag --list v1.0.6` / `rtk proxy git show-ref --tags v1.0.6` | 未返回本地 `v1.0.6` tag/ref；不据此推断远端 tag 状态。 |

W7 候选 phase report 自述其工具曾在 Rust 1.99.1 诊断环境生成 333-file context 并通过输出 lock 的 locked metadata；其自述 10 tests passed、1 skipped。该记录不是本轮实测，也不能替代 pinned Rust 1.99.0、Multipass guest image build 或最终 image/SBOM/provenance 证据。见 `c960b9193cc960ae3d85711472a0d4f31fedd0b3:docs/reports/PHASE-STAR-STABLE-W7-K3S-CONTEXT-FILTER-20261031.md:3, 9, 24-40`。**本 lane 未执行测试或构建。**

## §3 已知缺口

1. **MIT 身份不一致，当前 stable Canvas release gate 静态上必失败。** `Cargo.toml:187` 为 `Apache-2.0`，而 workflow:75 要求 `MIT`，LICENSE:1 是 `MIT License`；三份 Canvas Dockerfile:14 也声明 `MIT`。需先对齐源 package metadata、许可证文本、OCI label 与 SBOM，再验证精确 tag 的 gate 运行结果。
2. **运行时版本未随 Git tag 更新。** Cargo workspace version 是 `0.1.0`；workflow 为生成 manifest 复制 workspace.package 字段，三服务从 `CARGO_PKG_VERSION` 暴露版本。OCI semver tag 源于 Git ref，不能证明二进制内部版本为 `v1.0.6`。当前工作树没有本地该 tag，无法核验对应源 commit。
3. **稳定发布条件只接受任意 `refs/tags/v*`。** 对本次批准的 `v1.0.6` 没有精确 ref 比较；实际 dispatch ref 与批准版号未取证。
4. **没有本轮真实 Canvas BuildKit context archive 或逐文件 hash manifest。** Workflow 检查文件集合但仅打印文件/package 数量；Docker build 消费的实际归档内容未被留档或复核。
5. **没有最终三份 OCI image 内容检查、image digest、attached SBOM 或 provenance。** 配置了 `sbom: true`、最大 provenance 和 attestation action，不代表已有成功运行结果。
6. **Envoy 是 pinned runtime dependency，但其 SBOM/license/provenance 未在本轮取证。** 需按明确的第三方 runtime dependency 单独闭合。
7. **`deploy/k3s-local` 状态未决。** 当前 dev path 的 context 过宽且超出 stable image set。W7 候选新增了过滤工具、context digest 与 guest build 路径，但旧 `build-and-deploy.sh` 和 README 仍走直接构建模板的旧入口。候选 report 另自述 D2 活动 worktree 中有未提交 README 将该 API image/build/deploy 标为 retired/unsupported；此冲突未在本 lane 独立验证，且未触碰该 worktree。按 brief，不把 W7 local path 自动当成已批准产品能力或 release evidence。
8. **候选大范围删除未做集成审查。** W7 与当前 dev 的 full diff 为 30 files；除本 audit 范围的 payload files 外，包含 desktop workflow、WBS、多个阶段报告/brief/dispatch 的变更或删除。不可整体合入，需按当前 WBS 与 owner 状态选择性复核。

## §4 子代理失败接手清单

本 W3 lane 未派发子代理；没有子代理失败产物需要接手。W7 候选自身的 phase report 记录其前序 dispatch brief 时序偏差以及后续独立复核，属于候选 lane 自述，不是本 lane 的执行历史；本报告保留该范围，不替候选背书。

## §5 守门规则

| # | 守门 | 结果 | 证据 / 缺口 |
|---:|---|---|---|
| 1 | Current dev 与 brief commit 一致 | PASS | `dev` resolves to `a272e9794e1a1975781720c9b3775c914f47f546`. |
| 2 | W7 candidate commit 身份核对 | PASS | Candidate resolves to `c960b9193cc960ae3d85711472a0d4f31fedd0b3`; read-only. |
| 3 | Stable Canvas image set 精确 allowlist | PASS (static) | Workflow matrix contains only the three Canvas service names; no successful release run observed. |
| 4 | Per-service context 不直接使用 monorepo root | PASS (static) | Workflow constructs a separate temp directory and passes that directory to Buildx. |
| 5 | Canvas direct/workspace path dependencies fail closed | PASS (static) | Context script checks crate dependency groups and inherited workspace dependencies. |
| 6 | Canvas local package closure excludes unrelated workspace packages | PASS (static) | Cargo.lock traversal rejects unapproved local package identities. |
| 7 | Canvas context rejects game/foreign crate paths | PASS (static) | File list check rejects `game`, `.git`, `target`, and crate paths outside the current service. |
| 8 | Canvas Dockerfiles pin build/runtime base digests | PASS (static) | All three service Dockerfiles use digest-pinned Rust and Debian bases. |
| 9 | Canvas runtime stage copies only executable and LICENSE | PASS (static) | Dockerfiles copy one binary plus root LICENSE into final stage; final image not inspected. |
| 10 | `.dockerignore` is not used as sole payload proof | PASS (static for Canvas) | Generated Canvas context is explicitly assembled and checked; actual BuildKit archive still open. |
| 11 | Approved MIT identity consistent across Cargo, LICENSE, image labels | FAIL | Workspace says `Apache-2.0`; workflow requires `MIT`; LICENSE and image labels say MIT. |
| 12 | Runtime application version derives from approved Git tag | FAIL | Cargo package is `0.1.0`; services expose `CARGO_PKG_VERSION`; OCI tag is separate metadata. |
| 13 | Exact `v1.0.6` ref and source commit verified | OPEN | No local tag ref; remote and actual CI checkout not checked. |
| 14 | Local `k3s-local` path excluded or formally reclassified | OPEN | Current context is broad; W7/D2 support-scope conflict is reported but not independently resolved. |
| 15 | Actual Canvas context archive and per-file hashes retained | OPEN | Not built or archived in this lane; workflow logs counts only. |
| 16 | Final OCI image, exact digest, SBOM reviewed | OPEN | No registry/build artifact inspection. |
| 17 | Provenance and pinned Envoy dependency evidence verified | OPEN | Workflow configuration exists; actual attestations and Envoy dependency evidence were not retrieved. |

## §6 签字栏

| 角色 | 审批者 / 状态 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — 本报告结论：W3 OPEN。 |
| SRE Lead | SRE Lead (Mavis 接手 agent per DEC-008) — 未运行 guest image / rollout 验收。 |
| 平台 | 平台 (Mavis 接手 agent per DEC-008) — stable release gate 有静态阻断；实际制品未验。 |
| 评审主持 | 评审主持 (Mavis 接手 agent per DEC-008) — static audit complete; evidence gates remain open. |
| PM | PM (Mavis 接手 agent per DEC-008) — 未批准版本、发布或 W3 关闭。 |

**修订人:** `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`

**发布审批:** 未申请；本报告不批准发布，也不关闭 W3。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次完成 W3 静态 release-payload audit；记录三项 Canvas context 的源码级过滤条件、MIT/version 阻断、local K3s 范围问题、W7 候选差异与实际 image/SBOM/provenance 未验事实。 | `docs/briefs/W3-STAR-PAYLOAD-AUDIT-20261101.md` dispatcher brief。 |
