# WBS-STAR-DELIVERY-CLEANUP-20261030

> **状态：OPEN**（2026-10-30）。本 WBS 是 Star 交付资产清理的执行基线；文档 lane 的本地静态检查已通过；并行实现、构建、部署、运行时与最终集成验收仍未确认，不将其标记为完成。
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
| D0 | 固定当前支持清单、资源计数口径与活动路径 | 协调者 | 无 | 文档统一区分 package/bin/workload/Service object/health endpoint；服务身份与固定路径一致 | OPEN |
| D1 | 移除退役 `canvas-game` Rust package/binary 及其仅游戏专属的活动构建依赖 | Rust/runtime cleanup | D0 | `cargo +1.99.0 metadata --no-deps --format-version 1` 确认 workspace 为 109 packages、9 binaries、3 Canvas binaries；不得把该数量解释为已部署服务数 | OPEN |
| D2 | 清理活动 OCI 镜像、workflow、K3s manifest、启动和验收脚本 | Delivery cleanup | D0 | 文件名为 `.github/workflows/publish-star-canvas-services.yml`、`deploy/star-canvas-services-k3s.yaml`、`scripts/start-star-backend.ps1`、`scripts/verify-star-services.sh`、`docs/deployment/star-canvas-services-deploy.md`；只定义三个 Canvas 后端与 Envoy 入口；镜像为 Star 服务身份；不包含游戏 Deployment、Service、HPA、PDB、路由、配置或 secret | OPEN |
| D3 | 同步活动设计/拓扑、将旧游戏规格归档为历史记录、建立 WBS 和阶段报告 | Docs retirement | D0、D1、D2 的服务清单 | 基本设计、架构图、性能指南和 guard 文档使用同一服务清单与路径；本 lane 的 HTML 解析、diff whitespace 与活动旧引用扫描已通过；旧 SRS/BD/DD/计划/ADR 保留原文并在开头标记“已退役/非当前范围”；新链接指向活动路径，待最终跨 lane 核对 | IN PROGRESS |
| D4 | 静态及构建验收 | Runtime + Delivery + Docs | D1–D3 | 对活动交付范围做 `canvas-game` 引用扫描；新 manifest/workflow 可解析；镜像名、image-build target 与服务身份一致；Rust 1.99 metadata/build evidence 与最终包数一致；文档链接有效、活动文档不把历史五服务清单描述为现状 | OPEN |
| D5 | 集成并复核最终版本 | 协调者 | D4 | 只集成已审查、通过其 lane 验收的提交到本地 `dev`；确认最终 `dev` 文档与代码一致；本 WBS 与 phase report 记录 SHA、验证命令/结果和未关闭缺口 | OPEN |

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
4. 未取得 Rust 1.99.0 metadata/build、manifest/workflow 解析及 endpoint 验证结果前，phase 状态保持 OPEN。
5. 不将 Envoy ingress 计为 Canvas backend，也不将 K8s Service 对象数量混作应用服务数。

## 5. 修订历史

| 版本 | 日期 | 修订人 | 变更 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 Star-only delivery retirement WBS、口径、lane 与验收门；所有并行实施和验证初始标为 OPEN | 用户要求活动容器/部署只含 Star 支持服务，并清理旧游戏遗留项 |
