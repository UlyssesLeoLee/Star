# PHASE-STABLE-HARDENING-CANVAS-OCI — 阶段报告

**阶段**：STABLE-HARDENING / Canvas OCI
**基线**：`dev@04d8b7a84514dcb454874eb852651778e1452dea`
**范围**：三个 Star Canvas 服务镜像及其发布工作流、K3s 镜像引用模板
**状态**：实现已完成；Cargo/OCI/K3s 运行时门禁未完成，不可据此发布稳定版

## §0 目的

将 `canvas-engine`、`domain-canvas`、`canvas-realtime` 的发布输入限定到各自 crate 与 Cargo 依赖闭包，阻止游戏遗留和其他 workspace 产品进入 BuildKit 上下文或缓存；固定构建基础镜像、限制发布权限，并让部署引用只接受实际发行生成的不可变 digest。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `.github/workflows/publish-star-canvas-services.yml` | main/dev 与 tag push 只运行测试；手动 workflow_dispatch 需从 `refs/tags/v*` 运行且将必填确认选为 `APPROVED` 才可请求发布；发布 job 绑定 `star-stable-release` environment；其余最小 context、lock 闭包、独立缓存、SHA 固定 Actions、provenance/SBOM、commit+semver 标签与 MIT 元数据检查保持不变 | 完成静态更新，CI 和 environment protection 未验证 |
| `crates/canvas-engine/Dockerfile`、`crates/domain-canvas/Dockerfile`、`crates/canvas-realtime/Dockerfile` | 分别只复制本服务 crate；固定 Rust 1.99.0 与 Debian Bookworm 基础镜像 digest；使用 `--locked`；随镜像包含仓库 MIT `LICENSE` 并设置 OCI license label；以 UID/GID 10001 运行 | 完成，未运行 Buildx |
| `.dockerignore` | 对 `.git`、target、node_modules、`star-game` 与 game-legacy 路径设置防御性排除 | 完成 |
| `deploy/star-canvas-services-k3s.yaml` | 三个 Canvas image 设置显式 digest 输入位；固定 Envoy 到 registry manifest digest；将 GHCR `imagePullSecrets` 移至 Pod spec | 完成模板；缺实际 Canvas digest，不可直接部署 |
| 本 brief 与本报告 | 记录本 lane 范围、结果、证据与缺口 | 完成 |
| 本 lane dispatcher JSONL | 追加静态验证、阻断与完成记录 | 完成 |

## §2 验证摘要

| 检查 | 结果 | 证据/限制 |
|---|---|---|
| Workflow YAML 与 K3s YAML 解析 | 通过 | Python PyYAML 6.0.3；K3s 共 4 个 Deployment，全部采用 digest 形式；前三个是明确的发行 digest 输入位 |
| context 生成器本地执行 | 通过 | `canvas-engine` 119 个锁定包、`domain-canvas` 119 个、`canvas-realtime` 136 个；每个上下文仅 9 个文件（含 MIT LICENSE），均无 `star-game`、其他 crate 源码或非本服务目录 |
| Cargo.lock 闭包静态核对 | 通过 | 从当前 lockfile 按包依赖递归过滤；闭包中的本地 package 只有当前服务；发现 path dependency 或无法唯一解析的 lock 引用会直接失败 |
| Dockerfile digest / COPY 边界检查 | 通过 | 三份 Dockerfile 的 Rust 与 Debian base 均匹配 64 位 registry digest；每份仅复制对应服务 crate，并配置非 root 用户 |
| Actions 权限、缓存及引用检查 | 通过 | `packages: write` 仅授予发布 job；取消 cargo 全 workspace cache；三个 BuildKit cache scope 分服务隔离且使用新版本前缀；Actions 固定到 commit SHA |
| `rtk git diff --check` | 通过 | 无空白错误 |
| `cargo check` / `cargo test` | 未验证 | 本地执行 `rtk cargo check -p canvas-engine -p domain-canvas -p canvas-realtime --all-targets --locked -j 4` 在 `rustc -vV` 阶段失败：当前安装的 `rustc.exe` 不适用于 `1.99.0-x86_64-pc-windows-msvc` 工具链；Cargo 报告 `0 crates`。本 lane 未将其记为通过；CI focused gates 尚未运行 |
| OCI 构建、推送与 K3s 部署 | 未运行 | 本 lane 不执行 publish/push/tag/deploy；未使用 Docker Desktop；无实际 GitHub Actions Buildx 输出或远端 Multipass/K3s 验证 |

基础镜像 digest 通过 Docker Hub Registry v2 manifest 只读响应核实：Rust `1.99.0-bookworm` 为 `sha256:114c7a4425406451c2866b6aafe69fe29b1b298832db1277d411ac73c82d04d6`，Debian `bookworm-slim` 为 `sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3`。现有 Envoy `v1.28-latest` manifest 的 registry digest 是 `sha256:2ff39d93964ddadfc74e7db0ec62bf433d91dcafa6c3542a6754715a60ba6b3c`，部署中只保留该 digest，不保留可变 tag。

## §3 已知缺口

1. **稳定发布仍被许可证元数据门禁阻止。** 当前 `Cargo.toml` 的 `[workspace.package].license` 仍为 `Apache-2.0`，根 `LICENSE` 为 `MIT License`，与用户确认的 Star 统一 MIT 身份不一致。依照 owner 边界，本 lane 未改 `Cargo.toml`。所有分支、tag push 和未批准的手动触发只运行测试；只有从 `refs/tags/v*` 手动运行且将必填 `release_gate_confirmation` 选为 `APPROVED`，才会尝试进入绑定 `star-stable-release` environment 的发布 job。该 environment 的必需审查人、分支规则及 W0-W9 证据核验尚未在 GitHub 配置/验证，必须由仓库 owner 设置并实测；即使流程条件满足，MIT 元数据检查仍会在 GHCR 登录前 fail closed。
2. **三个 Star Canvas image digest 尚不存在于本 lane 可读取证据中。** GHCR package-version 查询返回 403（当前凭据缺少 `read:packages`），并且本 lane 禁止实际 publish。K3s 清单的三个占位符必须在获准发行后替换为对应 Buildx 输出的 `sha256` digest；当前文件是渲染模板，不能直接部署。没有以伪造 digest 或 mutable tag 代替。
3. **Cargo 编译、OCI 镜像构建、K3s readiness 与多架构运行未验证。** 工作流设定 `linux/amd64`；当前环境没有可用的允许运行证据。需要在 GitHub Actions 与目标 Multipass/K3s 环境实测。
4. Envoy 的 digest 固定了当前原清单指向的 `v1.28-latest` 对应内容；本阶段仅完成不可变引用，没有评估 Envoy 版本生命周期或升级兼容性。
5. **第三方许可证与 NOTICE 未随镜像闭合。** 三份 Dockerfile 只包含 Star 根 `LICENSE`；Cargo SBOM/provenance 不代表第三方依赖许可证清单、NOTICE、版权/专利声明或再分发义务已满足。需针对实际过滤后的依赖闭包生成并复核合规材料，完成前不能发布。

## §4 子代理失败接手清单

初始 Canvas OCI lane 未派发子代理。本次 STABLE-CANVAS-RELEASE-GATE-20261031 后续审查已由协调者派发给独立 worker；本报告记录其完成结果，没有未接手的子代理输出。CodeRabbit 二次审查未能运行：本机入口最终调用 `/root/.local/bin/coderabbit` 并返回 `Permission denied`；不将其记录为通过。已由协调 agent 直接检查 diff，并执行本地静态验证。

## §5 守门规则

1. BuildKit context 由三项 allowlist 生成；只复制该 service 的 `Cargo.toml` 与 `src/`、最小 workspace metadata、对应 lock 闭包、MIT `LICENSE` 和 Dockerfile。上下文生成器拒绝未知镜像名、本地 path dependency、额外本地 workspace package、`star-game`、不完整 lock closure 及 game 路径。
2. 不调用 workspace 全量 Cargo 构建，不保留 Swatinem cargo artifact cache；BuildKit GHA cache 按服务隔离，新的 scope 不读取旧全仓构建缓存。
3. 镜像基础层固定 digest；K3s 不使用 `dev`、`latest` 或版本 tag 部署 Star Canvas 镜像。
4. GHCR 写权限只存在于发布 job；MIT 元数据检查在 GHCR 登录前运行。Buildx provenance 与 SBOM 开启，但它们不等于第三方依赖许可证/NOTICE 清单或再分发义务审查。
5. main/dev 与版本 tag push 以及普通 workflow_dispatch 只执行测试；只有从版本 tag 手动 workflow_dispatch 且必填确认值为 `APPROVED` 才能请求发布，并要求 `star-stable-release` environment 审查。GitHub environment 的必需审查人和限制仍需外部配置并验证。
6. 缺少真实发行 digest 时，K3s 文件明确保留未解析占位符，不宣称已可部署或已发布。
7. 没有执行 Docker Desktop、镜像推送、tag 创建、部署或远端 K3s 操作。

## §6 审核栏

| 角色 | 状态 |
|---|---|
| 架构审核 | 待主控 agent 审查本 lane 差异 |
| SRE/发布审核 | 待主控 agent 审查 stable release 门禁与 digest 补齐流程 |
| 平台审核 | 待确认 GitHub Actions 真实结果 |
| 评审主持 | 待审查；CodeRabbit 不可用已如实记录 |
| 产品/范围审核 | 本 lane 仅覆盖三个 Star Canvas 服务；等待主控 agent 确认 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版：隔离 Cargo/OCI 上下文、基础镜像 digest、最小缓存范围、非 root 镜像、digest-input 模板和稳定许可证发布门禁。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 取消版本 tag push 的自动 GHCR 发布；改为需要 tag ref 上手动确认并绑定外部保护 environment 的发布请求，同时澄清环境审查人配置、第三方 license/NOTICE 义务和所有远端发布证据仍待核验。触发原因：独立审查发现 tag push 可直接触发发布。 |
