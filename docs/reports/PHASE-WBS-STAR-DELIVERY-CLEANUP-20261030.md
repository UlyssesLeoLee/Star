# PHASE-WBS-STAR-DELIVERY-CLEANUP-20261030

> **状态：OPEN / PENDING EVIDENCE**（2026-10-30）。本报告记录当前 docs lane 草案；并行 Runtime 与 Delivery lane、最终静态审计、构建/部署证据和 `dev` 集成都尚未完成确认。

## §0 目的

执行 [WBS-STAR-DELIVERY-CLEANUP-20261030.md](../implementation-plans/WBS-STAR-DELIVERY-CLEANUP-20261030.md)：将活动容器镜像、CI 发布、部署清单和运维入口收敛到 Star 当前支持的服务；退役 Canvas 游戏服务；保留旧决策文档作为显式历史记录。

当前清单：Canvas 后端 `canvas-engine`、`domain-canvas`、`canvas-realtime`；独立 Star API `star-api-rest`；Envoy 是 ingress。Cargo package/bin 数、部署 workload 数、Service object 数与健康检查 endpoint 数分别计数，不得互换。

## §1 改动矩阵与任务状态

| 项 | 负责 | 当前记录 | 验收/状态 |
|---|---|---|---|
| AGENTS 长期约束 | Docs retirement | 已新增 Star 交付资产仅用当前 Star 服务身份、已退役演示/游戏服务不得进入活动构建/部署的规则；`git diff --check` 已通过 | 待 diff 和最终跨 lane 审查；OPEN |
| 基本设计与架构图 | Docs retirement | 已更新基本设计目录和部署拓扑；arch-live Canvas inventory 指向三个后端及固定活动路径；4 个修改后的 HTML 页面已成功解析，固定交付路径在 Delivery lane 中均存在 | 待链接、命名、资源数量与最终 manifest 对照；OPEN |
| 历史规格保留 | Docs retirement | 旧 SRS/BD/DD/实施计划/ADR/plan 均加退役历史说明，原文和路径保留 | 待链接与内容保留复核；OPEN |
| Rust runtime cleanup | Runtime lane | 目标是移除退役游戏 crate / bin；lane 已报告 Rust 1.99.0 metadata 为 109 packages / 9 bin targets / 3 Canvas backend bins | 报告证据已收到，仍待最终变更、lock/build 检查及提交 SHA；OPEN |
| OCI/CI/K3s cleanup | Delivery lane | 已确认固定活动路径、三服务+Envoy清单口径，并报出三个带 `star-` 的目标 GHCR image refs | 实际 manifest/workflow/script 与 image build/validation 结果待 lane 报告；OPEN |
| 最终联动与合入 | 协调者 | 尚未开始 | 需所有来源完成复核后集成本地 `dev`；OPEN |

## §2 验证摘要

| 检查 | 当前结果 |
|---|---|
| 文档静态检查 | `git diff --check` 通过；4/4 修改后的 arch-live HTML 页面解析通过；14 个新增/修改的 Markdown 与 HTML 本地引用均可解析（跨 lane 部署指南在 Delivery lane 解析）；5 个固定路径及独立 API manifest 路径存在 |
| 活动文档旧服务引用扫描 | 基本设计、arch-live、worktree guard 与 AGENTS 未发现旧服务引用；旧名称只在两个顶部标明历史快照的架构记录及六份顶部标明退役的历史规格/计划/ADR 中保留 |
| `cargo +1.99.0 metadata --locked --no-deps` | Runtime lane 已报告通过：109 packages / 9 bin targets / 3 Canvas backend bins；最终 commit/build 证据仍待交付 |
| Container build / image identity | Delivery lane 提供目标引用 `ghcr.io/ulyssesleolee/star-canvas-engine`、`ghcr.io/ulyssesleolee/star-domain-canvas`、`ghcr.io/ulyssesleolee/star-canvas-realtime`；尚无独立检查或构建成功证据 |
| Manifest/workflow parse 与 endpoint checks | Delivery lane 最终证据待交付；不宣称 K3s 部署或 4 endpoint 健康检查通过 |
| 最终 `dev` 对照 | 尚未进行 |

## §3 已知缺口

1. 文档 lane 本地 diff/HTML/活动引用检查已通过；`05-worktree-canvas.html` 的一个部署指南链接在 docs lane 单独检出时不在本地，已确认目标存在于 Delivery lane，需在集成后复跑链接检查。
2. 尚未取得 Rust lane 最终 workspace metadata、build gate、移除文件清单和提交 SHA。
3. 尚未取得 Delivery lane 实际镜像引用、K3s manifest 资源数、workflow/script 验证和提交 SHA。
4. Cargo metadata 计数（109 packages、9 bins、3 Canvas backend bins）已由 Runtime lane 报告；仍需在最终提交和构建状态下复核。它们与部署计数是不同口径。
5. 当前 endpoint 检查设计为三个 Canvas 后端加独立 `star-api-rest`；Envoy ingress 不在这四个 endpoint 中。是否完成运行时检查仍待证据。
6. 尚未复核全部分支提交是否只包含授权文件，也未合入 `dev`。

## §4 子代理失败 / 接手清单

- 文档 lane 已完成本地静态复核；HTML/目标路径检查结果已记录。当前没有本 lane 子代理失败需要接手。
- Runtime lane 与 Delivery lane 正在并行实施。若任一 lane 无法提供最终产物或验证证据，协调者应接手其未完成清单并保持本 phase OPEN；不得根据“已派发”或计划数据填写完成状态。
- 现有沟通确认了活动路径，但尚不足以证明代码、镜像、容器或部署已验收。

## §5 守门规则

1. 活动镜像、发布、部署、GitOps 与运行脚本只使用 Star 当前支持的服务身份。
2. 退役游戏服务不得进入活动构建、CI 或部署资源；仅显式标记为历史的文档可保留旧名称。
3. 三个 Canvas 后端、独立 Star API、Envoy ingress、Cargo package、binary target、K8s Deployment/Service 与验证 endpoint 使用各自计数口径。
4. `star-api-rest` 不并入 Canvas deployment manifest；Envoy 不计为 Canvas 应用服务。
5. 未获得可复查的 Cargo metadata、构建、镜像、manifest/workflow 和 endpoint 证据时不关闭 phase。
6. 只合入审查通过的 lane 内容；不修改 `main`、不 push、不清理此 phase 以外的分支或工作树。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构师 | 待最终服务清单和运行时实现对照 |
| SRE Lead | 待 manifest、startup/verify script 与集群证据 |
| 平台 | 待 Rust metadata/build 与镜像发布证据 |
| 评审主持 | 待静态审查、文件范围和 lane diff 审核 |
| PM | 待各 WBS 项有可审计验收结果 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 OPEN 阶段报告骨架；列明当前清单、并行 lane 状态、验证证据缺口；未声明构建、部署或合入完成 | Star-only delivery cleanup phase 启动 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录文档 lane 静态检查结果：4/4 arch-live HTML 解析通过，活动旧服务引用扫描通过，固定交付路径存在；WBS 保持 OPEN，最终验收仍待跨 lane 和 dev 集成证据 | 完成本 lane 文档审查与静态检查 |
