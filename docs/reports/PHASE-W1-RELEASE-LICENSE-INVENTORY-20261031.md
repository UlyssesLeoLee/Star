# PHASE-W1-RELEASE-LICENSE-INVENTORY-20261031

> 版本 v0.1 · 日期 2026-10-31 · lane license-inventory
> 分支 agent/star-stable-1031/license-inventory · 基线 a1fed3437660be80d75c2741e62837ab1bbd2501
> 范围：只读证据盘点；仅改本 lane brief、阶段报告、dispatcher audit。未修改源码、Cargo/JS manifest、lock、Dockerfile、workflow 或其他 lane。

## §0 目的与结论

按用户确认的发行范围盘点 Desktop、Web/Vercel、三个 Star Canvas 服务镜像及支持依赖。该报告是仓库证据快照，不是法律意见、权属确认或商业发行批准。分类：**已核实**只表示观察到当前文件/输出；**不一致**表示声明冲突；**未知**表示缺少真实制品、目标闭包或授权证据，未知不等于无许可或禁止商用。

**W1 保持 BLOCKED。** 用户选定 MIT 为 Star 首方统一标识，LICENSE:1、3 显示 MIT License 与版权行；Cargo.toml:187 的 workspace license 仍是 Apache-2.0。Canvas/Desktop Rust crate 继承该字段，Canvas Docker 标签却写 MIT。没有从这些仓库文本推断源代码所有权、外部授权链或第三方再分发资格。Cargo 的 license 字段按 SPDX 声明 package 许可，不是权属证明：[Cargo manifest license fields](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields)。

## §1 改动矩阵 / 发行面证据

| 项 | 当前仓库证据 | 分类 / 限制 |
|---|---|---|
| 首方元数据 | LICENSE:1,3 为 MIT；Cargo.toml:187 为 Apache-2.0。crates/canvas-engine/Cargo.toml:10、crates/domain-canvas/Cargo.toml:10、crates/canvas-realtime/Cargo.toml:10、crates/star-api-rest/Cargo.toml:10、crates/star-desktop/src-tauri/Cargo.toml:11 使用 license.workspace=true。deploy/helm/star/Chart.yaml:13 写 MIT，但 description 说明这是废弃占位且不用于部署。 | **不一致/P0。** Chart 声明不能关闭根 Cargo、crate 或制品层问题。 |
| Desktop | .github/workflows/star-stable-desktop.yml 配置 Linux .deb/AppImage、Windows MSI、macOS universal DMG、Rust/npm SPDX 生成与 MIT metadata gate。crates/star-desktop/src-tauri/tauri.conf.json:29-38 配 targets=all、updater artifacts=false、icons；未见 licenseFile/NOTICE 映射。crates/star-desktop/frontend/package.json 声明 private=true、无 license。PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md 记录此前本机 npm SPDX 2.3 SBOM 为 225 packages/315 relationships，Rust SBOM 因 rustc 不可用未生成。 | **不一致/未知。** release gate 当前应失败。无真实安装包或 hosted run；静态配置不能证明 LICENSE/NOTICE 在实际安装包中存在或缺失。既有 W2 数值不是本 lane 重跑或真实制品闭包证明。 |
| Web/Vercel | frontend/package.json:4 为 private=true、无 license；frontend 同有 package-lock.json 与 pnpm-lock.yaml。star-stable-web.yml 用 Node 24.22.2、npm ci、npm sbom，build-evidence archive 清单含 .next、public、package.json、package-lock.json、next.config.js、vercel.json、build-info.txt，未复制 LICENSE/NOTICE。workflow 明确该 archive 不是 Vercel deployment。 | **元数据与 package manager 未闭合；实际部署未知。** npm 文档说明 package.json license 用于指示包许可/限制，private=true 主要防止误发布：[npm package.json license](https://docs.npmjs.com/cli/v8/configuring-npm/package-json/#license)。不能由证据包清单推断真实 Vercel 内容；root directory、真实输出、环境与消费者验证未知。双 lockfile 需由 W2 owner 收敛。 |
| 三个 Canvas images | .github/workflows/publish-star-canvas-services.yml:60 列 canvas-engine、domain-canvas、canvas-realtime。三份 Dockerfile 使用 digest 固定的 Rust 1.99.0 Bookworm builder / Debian slim runtime，以 --locked 构建，复制目标 binary、根 LICENSE，并声明 org.opencontainers.image.licenses=MIT。crate license.workspace=true 实际继承 Apache。workflow:64-72 在发布前要求 root metadata 与 LICENSE 均为 MIT；:90-246 静态构建每服务隔离 Cargo context、校验锁文件闭包、拒绝 star-game/未许可本地 crate；:283-284 配置 Buildx sbom/provenance。 | **首方声明不一致；第三方闭包未知。** OCI 规范将此 annotation 定义为所含软件许可证的 SPDX expression，但它是元数据而非制品审计：[OCI image annotations](https://github.com/opencontainers/image-spec/blob/main/annotations.md)。没有 hosted build、真实 image digest/SBOM/provenance 或消费侧验证；Dockerfile 仅复制根 LICENSE。 |
| 支持依赖 | deploy/star-canvas-services-k3s.yaml:49,118,187 的 Star image 为 digest 占位符，并引用 Envoy SHA-256 digest（:346）。deploy/k3s-local/bff-deployment.yaml 使用 bff:local 与 envoyproxy/envoy:v1.31-latest，是本地配置，不能证明稳定发行引用。bff/Cargo.toml:28 声明 Apache-2.0。WBS-STAR-STABLE-RELEASE-20261031.md §W0 将 API/BFF、Envoy、Helm/GitOps 与实际 build/runtime 闭包列为支持依赖。 | **支持闭包未绑定。** Envoy 镜像 license/BOM 未审；最终部署是否包含 BFF/API 和对应镜像需 deployment graph 证实。本地可变 tag 不是稳定发行证据。 |
| NOTICE / 第三方闭包 | NOTICE.md:3-5 记 cargo-about 0.9.2、cargo-deny 0.20.2、生成于 2026-10-05；:11 报 1,265 crates，列 MPL-2.0×5、LGPL-2.1-or-later×2。Cargo.lock 在 10/06 至 10/31 多次变化。NOTICE.md:77-79 记录 RustGameServer 来源/适配说法，并明示尚未完成 build/distribution dependency-closure audit。deny.toml/about.toml 是策略/生成输入，不是每个目标的当前 BOM。license-gate.yml:69-86 的 cargo-deny list 导出/上传均 continue-on-error。 | **过期/未知。** 旧 workspace 计数不代表当前 lock 或各发行目标。MPL/LGPL 不自动禁止商用，但实际修改、链接、捆绑、安装、再分发义务必须逐项审查。 |
| RustGameServer 来源/权利 | NOTICE.md:77 记录来源 commit/path/hash、Apache-2.0 及授权范围声明；:79 明示适配闭包审查未完成。 | **权属/授权未知。** 本 lane 未核对独立授权文件、来源提交内容、贡献者权利或制品依赖链。NOTICE 自述不是本报告对授权有效性或发布资格的确认。 |

目标级闭合至少需分别保存：每个平台 Desktop 实际文件清单、Rust/JS BOM 与随包 notices；Web 实际部署 commit/config/输出与 BOM；每个 Canvas image 的 digest、feature/target Cargo BOM、基础镜像和 OS package 闭包、LICENSE/NOTICE、provenance/消费侧验证；以及最终 deployment graph 中 API/BFF/Envoy 的准确参与关系和相同级别证明。未知项必须阻断相关制品，不能以默认 LICENSE 或 OCI 标签推断。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 基线/边界 | 分支与基线如页首。初始仅有本 lane brief 和 dispatcher JSONL 未跟踪。 |
| 静态证据 | 检索 LICENSE、Cargo/npm manifests、Dockerfiles、Tauri config、稳定发布/license workflows、NOTICE/deny/about、部署清单和 WBS。只证明文件与配置。 |
| cargo-deny | cargo-deny --version 为 0.20.2。cargo-deny --all-features list --format json 失败：cargo metadata 启动 rustc -vV 时报告 Rust 1.99.0-x86_64-pc-windows-msvc 的 rustc.exe 不适用于该 toolchain。**枚举未成功，非通过。** 未换 toolchain 绕行。 |
| 锁定 metadata | cargo metadata --locked --all-features --format-version 1 同因 rustc 组件错误失败。 |
| NOTICE 时效 | git log --date=short --format="%h %ad %s" --since=2026-10-05 -- Cargo.lock 显示 10/06–10/31 锁文件后续变更，旧 NOTICE 不能代表当前 lock。 |
| 实物/远端 | 未跑 hosted workflow、Desktop build、Vercel deployment、Buildx/GHCR 或消费者验证；无真实 artifact/digest/BOM。W2 报告仅作为前轮记录引用。 |
| 测试/改动 | 本 lane 只写证据报告，不运行产品测试。仅更新本 brief、报告、audit；未改 manifest、lock、源码、Docker、workflow 或其他 lane/worktree。 |

复核命令（只读）：

    rtk proxy git status --short --branch
    rtk proxy rg -n "MIT License|Apache-2.0|license.workspace|NOTICE|RustGameServer" LICENSE Cargo.toml NOTICE.md crates deploy
    rtk proxy rg -n "license|LICENSE|NOTICE|sbom|workflow_dispatch|APPROVED" .github/workflows/star-stable-desktop.yml .github/workflows/star-stable-web.yml .github/workflows/publish-star-canvas-services.yml .github/workflows/license-gate.yml
    rtk proxy cargo-deny --version
    rtk proxy cargo-deny --all-features list --format json
    rtk proxy cargo metadata --locked --all-features --format-version 1
    rtk proxy git log --date=short --format="%h %ad %s" --since=2026-10-05 -- Cargo.lock

toolchain 修复后须按实际 target/feature 对每个 release package 生成并存档 BOM/NOTICE、工具版本和输出 hash，再人工审义务；workspace 默认图不能冒充 artifact BOM。

## §3 已知缺口

1. 用户 MIT 决策与 root Cargo Apache 元数据冲突；共享 metadata owner 需统一并验证，本 lane 未更改。
2. NOTICE 早于多次 Cargo.lock 变更，需按精确 commit/lock/feature/target/tool 版本重生成及人工复核。
3. 当前 cargo metadata、cargo-deny 枚举被 rustc 组件错误阻断。
4. RustGameServer 独立授权、贡献者权利、来源内容和 adapter 的分发闭包未知。
5. 无实际 Desktop 安装包、Vercel deployment、三 Canvas image digest、artifact-bound BOM/NOTICE、签名或消费侧证明。
6. Web 有 npm 与 pnpm lock，实际 Vercel package manager 未证实。
7. 第三方 notices、版权/专利/source/install 义务需从实际 payload/closure 判断；SBOM 配置不证明完成。
8. API/BFF/Envoy 是否进入最终公共部署、基础镜像/OS license 闭包未绑定。
9. host OS、Rust/Node 工具链、安装器、runtime image 和默认下载工具没有完整逐目标清单。
10. W1/W2/W3/W5/W6/W7 的后续 owner 验收仍 OPEN；本报告不将其他 phase 标为完成。

## §4 子代理失败接手清单

本 lane 未派发子代理或接收子代理产出。cargo-deny list 与 locked metadata 因所选 Rust 1.99.0 rustc 组件不适用而失败；未绕过或伪报通过。

## §5 守门规则

1. Cargo/npm/Chart/OCI 字段、LICENSE、NOTICE 和 SBOM 是声明或证据输入，单独不证明所有权、授权链或完整分发义务。
2. Cargo metadata 与用户选定 MIT 冲突期间，相关 stable release 保持 fail closed。
3. 按真实修改、链接、捆绑、安装、再分发方式审查义务；不得把 copyleft 自动写成禁止商用。
4. NOTICE 必须对应精确 source/lock/features/target/generator；旧 workspace 数量不能替代每个 artifact BOM。
5. OCI MIT 标签不替代镜像 digest、Cargo/OS 闭包、NOTICE、provenance 和消费者验证。
6. Web evidence archive 不得称作 Vercel deployment；真实输出和环境未验前 W6 不关闭。
7. 未检查实际 payload 不断言其中含/不含 LICENSE 或 NOTICE；只陈述静态配置。
8. RustGameServer 权利资料及目标闭包未核实前，受影响发行保持阻断。
9. 保留本 lane 以外所有 owner 分支、worktree 与用户文件；本任务不清理或改写它们。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | Lane owner 完成证据分类；所有权/授权链未判定，待协调者复核。 |
| SRE Lead | 无 hosted build、Vercel、registry 或 production payload 证据，不批准关闭。 |
| 平台 | 静态配置已盘点；真实构建、部署与镜像闭包待验证。 |
| 评审主持 | 父级独立审阅待完成；W1 未验收通过。 |
| 产品/PM | 发行面沿用用户确认范围；不构成发行批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 盘点首方声明、构建输入、支持依赖、NOTICE 与来源权利缺口；记录 toolchain 阻断并维持 W1 BLOCKED。 | W1-RELEASE-LICENSE-INVENTORY-20261031 |
