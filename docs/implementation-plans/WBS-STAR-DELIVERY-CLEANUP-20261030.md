# WBS-STAR-DELIVERY-CLEANUP-20261030

> **状态：OPEN**（2026-10-30）。本 WBS 是 Star 交付资产清理的执行基线；文档 lane 的本地静态检查已通过；并行实现、构建、部署、运行时与最终集成验收仍未确认，不将其标记为完成。
>
> **当前工具链目标复核（2026-11-06）**：用户已指定 Rust 1.99.1 作为目标基线；本 lane 起始 checkout 的 `rust-toolchain.toml` 与根 `Cargo.toml` 仍固定为 1.99.0。下方 `+1.99.0 metadata` 是当时实际取得的历史结果，不代表 1.99.1 metadata 或构建验收；本 lane 不含工具链升版，D4 须在升版后以 1.99.1 重新验证。
>
> **目标**：发布物只包含 Star 当前支持的服务身份，退役的 Canvas 游戏服务不进入活动容器镜像、CI 发布、部署/GitOps 或运行脚本。旧规格和决策保留为明确标注的历史记录。

## 1. 范围与服务计数口径

当前 Canvas 后端服务：`canvas-engine`、`domain-canvas`、`canvas-realtime`。`star-api-rest` 是独立部署的 Star API。Envoy 是边缘入口，不属于 Canvas 应用服务。

| 口径 | 预期数量 | 解释 |
|---|---:|---|
| Rust workspace packages | 109（待最终 Cargo metadata 证据） | 代码包数量，不是部署服务数 |
| Rust binary targets | 9（待最终 Cargo metadata 证据） | workspace 可执行 target 数量 |
| Canvas backend binary targets | 3（待最终 Cargo metadata 证据） | 仅上述三个后端 |
| Canvas deployment workload | 3 Deployments + 3 ClusterIP Services | 来自 Canvas manifest |
| Canvas manifest 中的 ingress | 1 Envoy Deployment + 1 LoadBalancer Service | 入口资源，不计入应用服务数 |
| Canvas manifest K8s 资源计数 | 4 Deployments + 4 Service objects | 资源对象数，不是应用服务数 |
| 联通性检查的应用 endpoint | 4 | 三个 Canvas 后端 + 独立 `star-api-rest`；Envoy 不计入 |

Delivery lane 提供的目标镜像引用为 `ghcr.io/ulyssesleolee/star-canvas-engine`、`ghcr.io/ulyssesleolee/star-domain-canvas` 和 `ghcr.io/ulyssesleolee/star-canvas-realtime`；最终 manifest/workflow 仍须逐项核对，以上引用本身不代表已构建或推送镜像。

`star-api-rest` 不得被加入 `deploy/star-canvas-services-k3s.yaml`；当前是否已在目标环境部署须由独立部署证据确认。容器镜像引用必须体现 Star 服务身份（镜像名包含 `star-` 或等价、可审计的 Star 命名空间），不得发布退役游戏服务镜像。

## 2. 执行分解

| WBS | 工作 | 负责 lane | 依赖 | 验收标准 | 状态 |
|---|---|---|---|---|---|
| D0 | 固定当前支持清单、资源计数口径与活动路径 | 协调者 | 无 | 文档统一区分 package/bin/workload/Service object/health endpoint；服务身份与固定路径一致 | COMPLETE |
| D1 | 移除退役 Canvas 游戏 Rust package/binary 及其仅游戏专属的活动构建依赖 | Rust/runtime cleanup | D0 | 历史 cargo +1.99.0 metadata 记录 workspace 为 109 packages、9 binaries、3 Canvas binaries；不得把该数量解释为已部署服务数 | COMPLETE（历史 metadata 证据）；构建门禁在当前最低版本 1.99.1 下由 D4 单独跟踪 |
| D2 | 清理活动 OCI 镜像、workflow、K3s manifest、启动和验收脚本 | Delivery cleanup | D0 | 固定 Star 路径、4 Deployments/4 Services、3 个 GHCR 镜像均以 star- 开头、workflow/manifest/Kargo 一致；旧维护入口 fail-closed；活动路径不含未受支持的旧部署命令 | PARTIAL / OPEN；四个旧 K3s maintenance 入口已退役并 fail-closed，前端提示转向 Canvas 部署指南；`star-api-rest` 旧部署 README 明确标为非操作手册。待办分别为：(1) 未登记的 `deploy/k3s-local/build-and-deploy.sh` 仍可直接调用 Docker、sudo k3s/containerd 与 kubectl；(2) `deploy/helm/star/templates/NOTES.txt` 仍报 8 个服务，但当前 chart 没有相应 Deployment/Service templates。若稳定发行审计确认影响交付，两项须分别登记处理后才能关闭 D2 |
| D3 | 同步活动设计/拓扑、将旧游戏规格归档为历史记录、建立 WBS 和阶段报告 | Docs retirement | D0、D1、D2 的服务清单 | 基本设计、架构图、性能指南和 guard 文档使用同一服务清单与路径；HTML 解析和活动文档旧名称扫描通过；旧规格/计划/ADR 保留并标为退役历史 | COMPLETE；全站 arch-live 计数漂移单列跟踪 |
| D4 | 静态及构建验收 | Runtime + Delivery + Docs | D1–D3 | 活动交付扫描、YAML 解析、镜像矩阵和脚本语法检查通过；Rust workspace build、镜像 build 与 K3s endpoint 仍需证据 | PARTIAL / OPEN；本 lane 对四个旧 K3s maintenance 入口、变更 PowerShell AST、batch 引用及指南进行了静态核对；未登记的 star-api-rest 脚本仍可执行，且 Rust 1.99.1 metadata/build、镜像 build、已运行 Multipass K3s 的部署与 endpoint 仍 OPEN。当前 checkout 已随 rebase 使用 Rust 1.99.1，但本 lane 未运行 workspace gate |
| D5 | 集成并复核最终版本 | 协调者 | D4 | Runtime、Delivery、Docs 三条已审查 lane 已 rebase 并 fast-forward 合入本地 dev；D2 maintenance lane 经父级审查后 rebase/fast-forward 合入；最终 build/部署证据补齐后关闭 | PARTIAL；D2 lane 已集成到本地 dev，API 旧部署入口、Rust 1.99.1 构建、镜像与 Multipass K3s endpoint 验收仍开放 |

## 3. 活动交付接口

| 用途 | 活动路径 |
|---|---|
| Canvas container build/publish | `.github/workflows/publish-star-canvas-services.yml` |
| Canvas K3s workloads + Envoy ingress | `deploy/star-canvas-services-k3s.yaml` |
| Star backend startup | `scripts/start-star-backend.ps1` |
| App endpoint checks | `scripts/verify-star-services.sh` |
| Deployment operation guide | `docs/deployment/star-canvas-services-deploy.md` |
| Separate Star API deployment | `deploy/k3s-local/star-api-rest-deploy.yaml` (subject to delivery-lane confirmation) |

## 4. 集成门

1. `D1` 和 `D2` 可并行；`D3` 可先按固定接口文档化，但最终计数和镜像名需等两个实施 lane 证据复核。
2. 不因工作树或分支存在就整分支合并；逐项检查变更范围、当前设计相容性及验证证据。
3. 不删除历史规格文件；它们仅保留为审计记录，不可继续作为产品/发布范围来源。
4. 未取得 Rust 1.99.1 metadata/build、manifest/workflow 解析及 endpoint 验证结果前，phase 状态保持 OPEN；历史 Rust 1.99.0 metadata 不能替代当前版本的证据。当前 Multipass provider 只有只读预检和创建新 guest 的受控路径；本次未创建 VM，也未声称存在启动/恢复既有 guest 的 Star-managed 操作。
5. 不将 Envoy ingress 计为 Canvas backend，也不将 K8s Service 对象数量混作应用服务数。

## 5. 修订历史

| 版本 | 日期 | 修订人 | 变更 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 Star-only delivery retirement WBS、口径、lane 与验收门；所有并行实施和验证初始标为 OPEN | 用户要求活动容器/部署只含 Star 支持服务，并清理旧游戏遗留项 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | D4 静态核查通过 YAML、三镜像身份矩阵及 Bash/PowerShell 语法门；修复 Helm 占位 Chart YAML 并统一 registry；因活动旧 updater 及其调用链缺少 manifest，D4 标为 PARTIAL / BLOCKED；构建、镜像发布及 K3s 运行门保持 OPEN | D4 静态 lane 实测及活动调用链范围复核 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更正 D2 为 PARTIAL / OPEN：受控 Star 镜像交付面静态核对通过，但活动旧 updater 仍含旧 registry/旧调用链，未满足无退役游戏引用的验收；D4 保持 PARTIAL / BLOCKED | 活动维护调用链复核否定 D2 原 COMPLETE 状态 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 四个旧 K3s maintenance 入口 fail-closed 退役，前端提示改指向 Canvas 指南，`star-api-rest` 旧 README 标为非操作说明；分别记录仍能直接调用 Docker/k3s/containerd/kubectl 的 API 脚本，以及缺少 Deployment/Service templates 但 NOTES 仍报 8 服务的旧 Chart 文案。若稳定发行审计确认影响交付，需分别登记处理，D2 保持 PARTIAL / OPEN。D4 解除旧 maintenance updater 范围阻断但保持 OPEN；build、镜像和真实 Multipass K3s endpoint 证据未补齐 | D2 Multipass maintenance 静态审查及范围外遗留发现 |
| v0.5 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Rust 1.99.1 记录为用户指定的目标基线，并说明本 lane 起始 checkout 仍固定 1.99.0；历史 1.99.0 metadata 不替代目标基线验收，D2/D4 的遗留部署入口、镜像和现有 Multipass K3s endpoint 缺口仍开放 | 用户指定 Rust 1.99.1；避免把目标版本或历史 metadata 误写成已完成升版与验收 |
| v0.6 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D2 lane 独立审查、两笔提交 rebase 和 fast-forward 到本地 dev；当前 checkout 已用 Rust 1.99.1，但 D2 lane 未运行 workspace gate；未登记的 API 部署脚本、Rust 1.99.1 workspace gate、镜像构建和 Multipass K3s endpoint 证据继续开放 | 完成 D2 隔离 lane 的审查、本地集成与工具链状态复核 |
