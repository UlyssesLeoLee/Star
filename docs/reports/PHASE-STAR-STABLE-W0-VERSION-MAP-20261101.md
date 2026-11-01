# PHASE-STAR-STABLE-W0-VERSION-MAP-20261101

> **状态：版本映射审计完成；W0 PARTIAL，稳定发行仍 OPEN / 不可发布。**
> **审计基线：** 本地 dev@2a57fa0d329be830735d6657677236e195673362。
> **用户决策：** Git tag 是 Star 产品发行版本的唯一权威来源；下一稳定版为 `v1.0.6`，规范版本字符串为 `1.0.6`。
> **范围：** 只读盘点 Desktop、Web/Vercel、三份 Star Canvas 服务镜像及相关元数据；未改产品文件、未创建或移动 tag、未构建/发布/部署/push、未刷新远端 refs。

## §0 目的

确定当前发行面是否能从同一个 Git tag 取得产品版本，并给出最小的后续实现与验收路径。当前 tag `v1.0.6` 只作为用户选定的目标版本，不代表该 tag 已存在或已发布。

静态证据来自当前 worktree 文件、PowerShell 定向行读取、Python 3 `tomllib` 对 `crates/**/Cargo.toml` 的只读清点，以及本地 Git tag/history 查询。工作流配置不等于 hosted 执行或真实制品证明；远端 tag、tag protection 和平台部署状态没有刷新或核验。

## §1 发行版本映射矩阵

| 发行面 | 当前值与源 | 是否由 Git tag 派生 | 最小改动方向 / 结论 |
|---|---|---|---|
| Git 历史与目标版本 | 本地 tag 有 `v1.0.0`–`v1.0.5`；`v1.0.5` 指向 `1369c4e8e5cb697ef00ab599a85d8940a8f9d86d`，该 commit 是当前 dev 的祖先。当前本地没有 `v1.0.6`。 | 用户已指定将来以 `v1.0.6` 为唯一发行源；本地没有该 tag。远端状态未知。 | 仅在最终发行源 commit 和所有 release gates 关闭后，由 release owner 创建指向该 commit 的不可变 tag；不得提前预创建或复用 tag。 |
| Root Cargo 与产品 crate | 根 workspace `Cargo.toml:182-183` 的 `workspace.package.version` 是 `0.1.0`。`crates/**` 盘点为 110 个 package：96 个继承 workspace version，14 个有独立固定 package version。三份 Canvas crate 与 Desktop crate 均继承根 workspace version（`crates/canvas-engine/Cargo.toml:7`、`crates/domain-canvas/Cargo.toml:7`、`crates/canvas-realtime/Cargo.toml:7`、`crates/star-desktop/src-tauri/Cargo.toml:7`）。 | 否；编译及 SBOM 元数据仍显示 `0.1.0`。14 个显式 package version 也不是 tag 派生值，例如 `crates/agent-domain/Cargo.toml:7`、`crates/star-credential/Cargo.toml:7`。 | 建一个共享的 release-version resolver，从已校验的 tag 导出 `STAR_RELEASE_TAG=v1.0.6`、`STAR_RELEASE_VERSION=1.0.6`。只对发行根 package 生成临时 release metadata，使 Desktop 与三份 Canvas image 的产品根版本来自 tag；普通内部 crate semver 作为库/API 身份分类，不冒充 Star 产品发行号。若某显式 crate 以后成为独立发行根，必须接入同一 resolver。 |
| BFF 与 ACI emitter | `bff/Cargo.toml:24-25` 固定 `0.1.0`；`tools/aci-emitter/Cargo.toml:5-6` 固定 `0.1.0`。 | 否。 | 当前批准发行范围没有 BFF 或 ACI emitter 制品；BFF 是独立服务候选，ACI emitter 是工具/CI helper。不得把这两个值当 Star 总版本。若 BFF 进入发行清单，其产品根元数据需由同一 tag resolver 生成；ACI 维持工具包版本，除非正式纳入分发。 |
| Web / Vercel | `frontend/package.json:2-4` 为 `star-frontend@0.1.0`；npm lockfile 根元数据 `frontend/package-lock.json:3,9` 同为 `0.1.0`。稳定 Web workflow 用锁文件安装（`.github/workflows/star-stable-web.yml:59-69`）。 | 否；workflow 仅检查 tag 前缀及人工 `APPROVED`，没有把 tag 版本注入 Next.js build。 | 将 resolver 输出作为唯一 `STAR_RELEASE_VERSION` build input，并进入 Web 的可读 build manifest/诊断信息和 provenance。发布证据应同时记录 tag、commit SHA、archive digest。当前 workflow 明确只产出 build evidence，不部署 Vercel（`.github/workflows/star-stable-web.yml:3-4`）；Vercel 部署和消费者/回滚验证仍是独立缺口。 |
| Desktop | `crates/star-desktop/frontend/package.json:2-4` 与对应 lockfile 根 package 都是 `0.1.0`；Tauri bundle 的 `productName` 为 `star-desktop`，版本是 `0.1.0`，bundle 启用，updater artifacts 关闭（`crates/star-desktop/src-tauri/tauri.conf.json:3-5,30-32`）。 | 否；tag 目前只是 gate 条件，实际 Tauri/Cargo build 仍读取 checkout 中固定 metadata。 | 在临时构建 metadata 中把 tag 派生的 `1.0.6` 写入 Tauri app/bundle 版本和 `star-desktop` 的 Cargo package metadata；构建后检查三个 OS 包内版本与同一 tag 一致。当前上传容器名固定为 `star-desktop-linux-unsigned`、`star-desktop-windows-unsigned`、`star-desktop-macos-universal-unsigned`（`.github/workflows/star-stable-desktop.yml:161-165,242-244,322-325`），应至少将正式 release record/artifact 标识关联到 tag + commit SHA。 |
| Canvas OCI | 发布 workflow 监听 `v*` tag，并通过 `docker/metadata-action` 配置 SHA tag 与 semver tag `type=semver,pattern={{version}}`，并把 metadata tags 和 labels 传给 Buildx（`.github/workflows/publish-star-canvas-services.yml:9-20,262-279`）。该 action 将 `v1.0.6` 的 `{{version}}` 渲染为 `1.0.6`，默认 `org.opencontainers.image.version` label 取计算出的主版本；配置依据见 [Docker Metadata Action semver 文档](https://github.com/docker/metadata-action#typesemver) 和 [默认 OCI labels](https://github.com/docker/metadata-action#outputs)。workflow 没有单独的、严格的稳定版 tag parser，release job 只检查 `refs/tags/v` 和人工批准（`:48,64-65`）；无真实镜像运行证据。 | **部分派生**：OCI semver tag 和默认 OCI version label 由 tag/ref metadata 计算；Kubernetes `version` labels 仍是固定值。实际镜像 digest 是不可变部署身份。 | 三个 image 的 tag 必须由共享 resolver 的精确 semver 解析生成；保留 SHA tag 作为来源线索，发布记录保存 `v1.0.6`、commit、三个 digest、provenance 和 SBOM。验收以 digest 为准，不以可移动的 semver tag 代替部署身份。 |
| Canvas K3s render template | 当前活动模板 `deploy/star-canvas-services-k3s.yaml:1-4` 要求在部署前替换 digest placeholder；三份镜像使用 digest 引用（`:49,118,187`），但六个 Kubernetes `version` label 固定为 `v0.1`（`:31,42,100,111,169,180`）。 | 镜像 digest 不带语义版本；labels 未跟随 tag。 | 渲染时由同一 resolver 把 `version: v1.0.6` 写入六处 label，并注入由成功发布 job 产生的精确 image digest。保留 digest pin 和 placeholder fail-closed 检查。 |
| 已弃用 Helm 占位 | `deploy/helm/star/Chart.yaml:3` 明确说明它是无活动 workload、未用于部署的 deprecated placeholder；`version`/`appVersion` 为 `0.1.0`（`:5-6`）。 | 否。 | 不计入当前产品 release payload，不把该 placeholder 当真实版本入口。移除/恢复 Helm 部署时，再要求其渲染值由同一 resolver 生成。 |
| Workflow gates 与实际交付 | Web dispatch 输入 `release_gate_confirmation`（默认 NO）并要求 `refs/tags/v*`（`.github/workflows/star-stable-web.yml:5-13,40-52`）；Desktop dispatch 输入 `release_confirmation`，attestation jobs 要求 APPROVED + `refs/tags/v`（`.github/workflows/star-stable-desktop.yml:52-61,335,371,424,469`）；Canvas 也要求人工 dispatch + APPROVED。三者都没有单独输入版本号，属于正确方向，但当前只按前缀 gate。 | tag 是选择 ref 的来源；当前不保证 tag 语法严格、checkout commit 一致或不可变。 | 所有 stable workflow 调用同一 parser/validator：拒绝非稳定 semver tag、校验 tag peeled commit 等于批准的发行源 SHA，并只消费 resolver 输出；不允许用户再输入第二个版本字符串。为 tag 命名空间配置创建/更新/删除限制。 |

上述 14 个显式 Rust crate package version 当前固定为 `0.1.0` 的 11 个包和 `0.0.1` 的 3 个包；具体 manifests：`crates/agent-domain/Cargo.toml:7`、`crates/canvas-collab/Cargo.toml:7`、`crates/domain-agent-windows/Cargo.toml:7`、`crates/domain-ai/Cargo.toml:7`、`crates/domain-cli/Cargo.toml:7`、`crates/domain-dashboard/Cargo.toml:7`、`crates/domain-form/Cargo.toml:7`、`crates/domain-kms/Cargo.toml:3`、`crates/domain-report/Cargo.toml:7`、`crates/domain-theme/Cargo.toml:7`、`crates/platform-messaging-proto/Cargo.toml:3`、`crates/star-credential/Cargo.toml:7`、`crates/star-taskgraph/Cargo.toml:7`、`crates/star-treesitter/Cargo.toml:7`。其是否属于独立 crate 发布要由 W1 的发行制品清单确认；当前不应仅因其出现在 workspace 就擅自把内部 semver 与产品号混为一谈。

## §2 验证摘要

| 核验 | 结果 | 证据边界 |
|---|---|---|
| 基线与本地 tag | 通过只读核验 | 当前 HEAD 为 `2a57fa0d329be830735d6657677236e195673362`；`v1.0.5` peeled commit 为 `1369c4e8e5cb697ef00ab599a85d8940a8f9d86d` 且为 HEAD 祖先；本地 `v1.0.6` 不存在。使用 `rtk proxy git tag --list 'v1.0.*'`、`rtk proxy git show -s ... v1.0.5`、`rtk proxy git merge-base --is-ancestor v1.0.5 HEAD`。未 fetch，远端 tag 状态未知。 |
| Rust metadata 盘点 | 通过只读清点 | `rtk proxy py -3` + Python `tomllib` 读取 `crates/**/Cargo.toml`，统计 110 package manifests、96 个 workspace version inheritance、14 个固定 package version；没有 Cargo build。 |
| Web/Desktop/Canvas/K3s 表面 | 完成静态行核验 | 使用 PowerShell `Select-String` 和精确行窗口读取 manifests、稳定 workflow、Tauri config、Canvas digest template、Helm placeholder；未构建应用或镜像。 |
| 版本源单一性 | **不通过 / W0 PARTIAL** | tag gates 仅作审批条件，产品版本仍是 `0.1.0`/`v0.1`；Web tag build 没有 Vercel 部署；没有验证远端 tag immutability。 |
| 构建、hosted、部署与消费侧验证 | 未执行 | 未运行 Cargo、npm、Tauri、Docker/Buildx、Vercel、K3s 或 GitHub Actions hosted jobs；未生成 artifact、SBOM、image digest 或 attestations。 |
| 文档完整性 | 通过 | PHASE §0–§7 检查、注册 dispatch JSONL 逐行 JSON parse、`rtk proxy git diff --cached --check` 均通过；此报告不代表产品版本映射已实现。 |
| 独立 CodeRabbit 审阅 | 未能启动 | `coderabbit` 命令入口存在，但 RTK proxy 返回 `/root/.local/bin/coderabbit: Permission denied`；因此未收到独立审阅结果，不能视作通过。已完成本地逐项人工复核。 |

## §3 已知缺口

1. 单一 tag resolver、精确 `vMAJOR.MINOR.PATCH` 校验和 tag-to-commit 绑定尚未实现；stable workflow 只校验 tag 前缀。
2. Tauri、Cargo 产品根、npm、Web build manifest、Canvas OCI metadata、K3s `version` labels 尚未共享一个 tag-derived 版本值。
3. Desktop 当前上传容器名无 tag；没有在本地证据中发现正式 GitHub Release 创建步骤。稳定 workflow 配置与真实的可下载 release 不等价。
4. Web stable workflow 只生成和 attests build evidence，不部署 Vercel；Vercel project/root directory、部署 id、production URL、version metadata、rollback/consumer evidence 未核验。
5. Canvas OCI workflow 可从 semver tag 生成版本 tag，但没有严格 tag parser；K3s 模板仍有 digest placeholders 和 `v0.1` labels，尚无真实发布 digest/部署消费者证据。
6. 当前本地 tag 缺 `v1.0.6`；未刷新远端，不能断言远端尚未创建。GitHub tag ruleset、谁可创建、是否禁止更新/删除均未核验。
7. BFF 独立 `0.1.0` 与 ACI emitter `0.1.0` 不在当前批准制品范围。W1 必须维护明确 product-root/utility 清单，避免未来把它们的独立 metadata 误认作产品版本或漏接 tag resolver。
8. 本阶段为静态审计，无 Rust/MSRV、Node/npm、Tauri、Buildx、Vercel 或 K3s 执行证据。W0 不得关闭。

## §4 子代理失败接手清单

- 本 lane 未再派生子代理；没有失败子代理或遗留源码改动。
- Coordinator 维护本 lane brief；本报告和 dispatch JSONL 由本 lane 更新。其它 W0/W1 lane、主 WBS 与工作树均不在本 lane 写入范围。

## §5 守门规则

1. 只修改 MAGOS 登记的阶段报告与 dispatch JSONL；brief 由 coordinator 持有。
2. Git tag 是唯一产品发行版本来源；`1.0.6` 只能由 `v1.0.6` resolver 输出，任何 workflow 不接收第二个版本字符串。
3. stable parser 必须拒绝 `v` 前缀以外格式、预发布版本或非 SemVer tag；若要发布预发布版，需单独的版本策略和 channel，不得放过稳定 gate。
4. 发布前校验 annotated tag 的目标 commit 与最终审阅、验证的 source commit 一致；tag 创建/更新/删除权限应由 GitHub ruleset 限制。该平台设置本次未核验。
5. Desktop/Web/Canvas 所有制品、SBOM、provenance、K3s render 记录同一 tag、source SHA 与产物 SHA/digest；部署只引用已验证的不可变 digest。
6. 运行时与包内版本须经真实构建后读取验证；静态配置检查不能关闭版本一致性验收。
7. Cargo 内部库/API package version 与 Star 产品发行版本须在 W1 制品清单中明确分类；没有独立分发的内部 crate 不应成为第二个产品版本源。
8. Desktop artifact 上传、Vercel production deploy、Canvas push、K3s 部署和 consumer verification 是不同门；不能以 workflow YAML 或 tag 存在替代 hosted/实际证据。
9. `deploy/helm/star/Chart.yaml` 当前为弃用空占位，不列为发行 payload；若恢复部署，必须重新审查。
10. 本阶段未创建/移动 tag、未发布 artifact/image、未部署、未 push、未修改 main 或其他 worktree。
11. W0 保持 PARTIAL，直到 resolver 与 build-time mappings 落地、静态与真实构建验收通过、来源 commit 固定且消费者可验证。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 建议产品发行版本只由 Git tag 导出；内部 crate semver 单独分类 |
| SRE Lead | 架构师（Mavis 接手审核）— tag immutability、hosted 执行和部署证据待核验 |
| 平台 | 架构师（Mavis 接手审核）— Desktop/Web/Canvas/K3s 版本 surface 已静态映射；未实施 |
| 评审主持 | 架构师（Mavis 接手审核）— 审计范围与未验证事实已标明 |
| PM | 架构师（Mavis 接手审核）— v1.0.6 作为目标版本；当前不可发布 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 审核本地 dev@2a57fa0；静态映射 Git tag、Cargo、npm/Tauri、stable workflows、Canvas OCI/K3s labels 与弃用 Helm placeholder；定义 tag-derived resolver、不可变 tag 与最终发行验收；保持 W0 PARTIAL。 | 用户确定 Git tag 为唯一发行版本源，下一稳定版 v1.0.6 |
