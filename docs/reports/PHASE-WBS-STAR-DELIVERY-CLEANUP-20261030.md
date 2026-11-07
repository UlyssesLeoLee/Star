# PHASE-WBS-STAR-DELIVERY-CLEANUP-20261030

> **状态：OPEN / PENDING EVIDENCE**（2026-11-06）。Runtime、Delivery、Docs 三条 lane 与 D2 maintenance lane 均已审查并 rebase/fast-forward 集成到本地 dev。D2 只关闭四个旧 K3s maintenance 入口；未登记的 star-api-rest 部署脚本仍可执行，Rust 1.99.1 构建、镜像构建和 Multipass K3s 运行验收仍未完成。

> **当前工具链目标复核（2026-11-06）**：用户已指定 Rust 1.99.1 作为目标基线；本 lane 起始 checkout 的 `rust-toolchain.toml` 与根 `Cargo.toml` 仍固定 1.99.0，且本 lane 未修改工具链文件。下方 Rust 1.99.0 metadata 记录是历史证据，不代表 1.99.1 已配置或通过；本阶段尚无 1.99.1 metadata/build 结果。

## §0 目的

执行 [WBS-STAR-DELIVERY-CLEANUP-20261030.md](../implementation-plans/WBS-STAR-DELIVERY-CLEANUP-20261030.md)：将活动容器镜像、CI 发布、部署清单和运维入口收敛到 Star 当前支持的服务；退役 Canvas 游戏服务；保留旧决策文档作为显式历史记录。

当前清单：Canvas 后端 `canvas-engine`、`domain-canvas`、`canvas-realtime`；独立 Star API `star-api-rest`；Envoy 是 ingress。Cargo package/bin 数、部署 workload 数、Service object 数与健康检查 endpoint 数分别计数，不得互换。

## §1 改动矩阵与任务状态

| 项 | 负责 | 当前记录 | 验收/状态 |
|---|---|---|---|
| AGENTS 长期约束 | Docs retirement | 已加入 Star-only 容器/部署要求，并记录完成审查后及时集成及清理已合入 lane 的规则；最终 AGENTS diff 已复核 | COMPLETE |
| 基本设计与架构图 | Docs retirement | 基本设计、部署拓扑和 guard 已对齐三个 Canvas 后端、独立 star-api-rest、Envoy 入口及固定路径；4/4 修改的 HTML 可解析，活动文档旧名扫描无命中 | COMPLETE；全站数量漂移另列 |
| 历史规格保留 | Docs retirement | 原 SRS/BD/DD/计划/ADR 内容和路径保留，并在开头明确标记已退役/非当前范围 | COMPLETE |
| Rust runtime cleanup | Runtime lane | 已移除 canvas-game crate、binary、Dockerfile、workspace/lock 引用；Rust 1.99 metadata 在最终 lane tree 复核为 109 packages / 9 bin targets / 3 Canvas backend bins | D1 COMPLETE；workspace check 被 OS error 225 阻断 |
| OCI/CI/K3s cleanup | Delivery lane | 活动 workflow/Kargo/manifest 只包含三个带 star- 的 GHCR 镜像；manifest 有 4 Deployments/4 Services；四个旧 K3s maintenance 入口已 fail-closed，Canvas 应用指南要求已有可达 K3s context。残留 (1) `deploy/k3s-local/build-and-deploy.sh` 仍可执行；残留 (2) `deploy/helm/star/templates/NOTES.txt` 仍报 8 services，但 chart 缺少相应 Deployment/Service templates；两者都未在本 lane 登记 | PARTIAL / OPEN；未做真实 image build/deploy |
| 最终联动与合入 | 协调者 | Delivery 169f236f、Runtime 1868a13c、Docs 84aac92b 已按顺序 rebase/fast-forward 至本地 dev；代码与文档位于同一 checkout | PARTIAL；待 D4 构建/运行验收 |

## §2 验证摘要

| 检查 | 当前结果 |
|---|---|
| 文档静态检查 | git diff --check、4/4 HTML parser、活动交付旧名扫描及三个 YAML 文件解析均通过；lane 报告的 14 个本地链接检查已随交付目标路径集成。D2 maintenance lane 的 PowerShell AST 6/6、batch fail-closed/指南引用 2/2、注册入口旧路径扫描与 CypherGraph 检查均通过；全仓 D2 仍因两项未登记残留开放 | 静态 PASS；arch_live_doc_check 仍报告 12 项全站计数漂移 |
| 活动文档旧服务引用扫描 | 基本设计、arch-live、worktree guard、AGENTS、CI 发布、部署清单和运维脚本无 canvas-game/gameplay 命中；旧名称仅保留在明确标记为历史的文档中 | PASS |
| 历史 cargo +1.99.0 metadata --locked --no-deps | 当时 runtime lane tree：109 packages / 9 binary targets / 3 Canvas backend binaries；不是用户指定目标 Rust 1.99.1 的验收 | 历史 PASS；此 checkout 仍固定 1.99.0，1.99.1 metadata/build 未运行 |
| Container build / image identity | 4 Deployments/4 Services；镜像矩阵与 workflow 三个构建目标一致，三个 GHCR image basename 均为 star- 前缀 | 静态 PASS；未执行镜像构建或发布 |
| Manifest/workflow parse 与 endpoint checks | Workflow、K3s、Kargo YAML parse；Bash 与 PowerShell 解析通过；Canvas guard tests 21 passed | 静态 PASS；K3s 和 endpoint 未运行 |
| D2 maintenance 静态门 | 本次复核重跑 PowerShell AST 6/6、两个 batch fail-closed/指南引用检查、Multipass cap/lock/inventory/no-auto-purge 源码核对、chart NOTES/templates 核对及 `git diff --check`；旧路径扫描与 CypherGraph freshness 结果沿用 D2 lane 原报告，未在本次重跑。没有执行 Multipass、Docker、kubectl、Helm 或 VM 操作 | 本次复核范围 PASS；旧扫描/graph 结果为 lane 原报告；不代表部署或 build 通过 |
| 最终 dev 对照 | D2 lane 的 rebase/fast-forward 集成点为 `1ce184aecde0c94bab386b4435c39852d00e66b2`；本次阶段证据同步在该集成点之后提交。没有修改 main 或 push | 集成完成；D2/D4 仍 OPEN |

## §3 已知缺口

1. 历史 Rust workspace check 未通过：Cargo 启动 rustc 时被 Windows 安全软件以 OS error 225 拦截；该记录未固定 toolchain 版本，未执行 Canvas crate tests。该结果不能归因于源码错误，也不能记为通过；当前 Rust 1.99.1 workspace gate 仍未运行。
2. arch_live_doc_check.py --verbose 在最终 runtime tree 报告 12 项全站数据计数漂移，涉及数据库表/策略/schema/migration、REST route、Rust 文件数和 LOC；尚未确认这些漂移的来源。本次活动服务文案与路径检查独立通过。
3. 尚无三个 Star Canvas 镜像的真实构建/GHCR 发布证据，也未在 Multipass 管理的现有 K3s 集群中运行部署或四个应用 endpoint 健康检查；未创建或更改 VM/集群。
4. star-desktop 的两个生产 IPC 查询仍使用 MockDb 演示行，另列 WBS 跟进；本次只移除退役 Canvas 游戏 crate 与交付资产，不能据此宣称所有 mock 数据已清除。
5. 本地 dev 仍领先 origin/dev；本次没有 push，也没有改动 main。其余未完成 lane 和分支均未清理。
6. 未登记的 `deploy/k3s-local/build-and-deploy.sh` 仍可执行：源码实际调用 `docker build`、`docker save`、`sudo /usr/local/bin/k3s ctr -n k8s.io images import`、`crictl images`、`kubectl apply`、rollout 与 pod 查询，用于 star-api-rest。README 已将其标记为退役并禁止运行，但本 lane 不得修改该脚本；需另行登记 lane 决定退役或重构。此处描述的是 Docker CLI 与本机 k3s/containerd 调用，不是 Docker Desktop 前置条件。
7. `deploy/helm/star/templates/NOTES.txt` 仍声明“8 services deployed”并列出旧服务名；当前 chart 只有 `Chart.yaml`、values、helpers、secret 与 NOTES，没有相应 Deployment/Service templates。该 NOTES 内容与当前 chart 不符，且本 lane 未运行 Helm/K3s 验证。若稳定发行审计确认其影响交付，须另行登记处理。

## §4 子代理失败 / 接手清单

- Runtime、Delivery、Docs lanes 的差异已分别审阅；runtime 发现的旧 CI/K3s 引用由 Delivery lane 原子替换后，三条提交按依赖顺序集成。
- D2 maintenance lane 的四个旧 K3s 入口已由父级独立审阅并合入 dev；超出 manifest 的 star-api-rest build/deploy 脚本仍列为独立待决项。
- Cargo 构建阻断、全站 arch-live 数量漂移、镜像/集群验收和桌面 MockDb 跟进已列入 §3。

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
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Runtime/Delivery/Docs 三条 lane rebase 并 fast-forward 到本地 dev、Star-only 静态验收及 Cargo OS error 225 阻断；阶段保持 OPEN，列出未完成构建、Multipass K3s endpoint、arch-live 计数漂移和桌面 MockDb 跟进 | 完成三条 lane 集成后的验收与 WBS 同步 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D2 maintenance 隔离 lane 的四个旧 K3s 入口 fail-closed 处理；分别列明 (1) 未登记、仍可执行并调用 Docker CLI/本机 k3s/containerd/kubectl 的 API 脚本，(2) 缺少 Deployment/Service templates 但 NOTES 仍误报 8 服务的旧 Chart 文案；两项若影响稳定发行须另行登记。D2 保持 PARTIAL / OPEN；当前 lane 尚未集成，build、发布及真实 K3s endpoint 门保持 OPEN | D2 maintenance 源码审查与活动入口扫描 |
| v0.5 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Rust 1.99.1 是用户指定的目标基线，同时注明该 lane 起始 checkout 仍固定 1.99.0；将 1.99.0 metadata 与未记录 toolchain 版本的 OS error 225 标为历史证据；当前目标版本 workspace build、镜像、K3s endpoint 与 D2 未登记入口仍开放 | 当前工具链目标与 D2 WBS 证据复核 |
| v0.6 | 2026-11-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D2 lane 已经父级审查并 rebase/fast-forward 集成到本地 dev；验证范围仅含脚本语法、fail-closed 文本行为、Multipass 静态约束和差异检查。D2/D4 保持 OPEN，未登记 API 脚本、Rust 1.99.1 build、镜像和 K3s endpoint 仍未关闭 | 完成 D2 lane 本地集成与 WBS/阶段报告同步 |
