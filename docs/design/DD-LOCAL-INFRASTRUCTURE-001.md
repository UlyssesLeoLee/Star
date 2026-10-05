# DD-LOCAL-INFRASTRUCTURE-001 — Rust 本地基础设施与 k3s

> v0.8 · 2026-10-05 · Draft / 已有一组宿主 Multipass/HCS + Linux guest K3s/GitOps 实跑；Star 原生 Multipass adapter、Run 绑定、跨平台能力与 RSS 仍未验收。
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 上游：requirements v5.72 §50.8G-50.8H / basic-design v5.69 §8.1、§16.21-16.22 / 实施计划 v5.95 §6.83。

## §0 目的与边界

为渡口增加 provider-neutral 的 Rust Host Infrastructure Manager。**Multipass 是本地/CI Linux VM 的首选管理 provider，K3s 运行在由它管理的 Linux guest 中**；这是后续实现基线，不再是并列候选。Podman machine、Linux Incus、macOS/Linux Lima 按实际 capability 作为替换/备用 provider；用户自有或远端 Linux K3s 是独立集群路径，生产环境不依赖 Multipass。K3s 始终在 Linux host/guest/node 运行，不在 Windows 原生运行。Multipass 上游将产品定位为开发、测试与本地 VM 工具；拥有 Multipass daemon 权限可控制实例及高权限 host mount（[官方安全说明](https://canonical.com/multipass/docs/latest/explanation/about-security/)、[mount 安全说明](https://canonical.com/multipass/docs/stable/explanation/mount/)），因此 adapter 不能成为不可信 Agent 边界。Runtime sandbox 与 Hook 单独验收。K3d 仅复用已有容器引擎做开发/测试；Cloud Hypervisor 保留为 Linux 后续 PoC。已有宿主级实跑见 [`PHASE-MULTIPASS-K3S-GITOPS-REPORT.md`](../reports/PHASE-MULTIPASS-K3S-GITOPS-REPORT.md)：Windows 11 Home + Multipass 1.17.0-rc1/HCS 创建 Ubuntu 24.04.5 guest，运行 K3s v1.36.5+k3s1，并将 cert-manager、Argo CD、Argo Rollouts、Kargo 共 19 个 Pod 验证为 Running。该证据关闭了单机开发环境的宿主/K3s/GitOps PoC，不代表 Star 产品已实现通用 Rust provider adapter、Run namespace/授权绑定、Worktree bridge、不可信 Agent sandbox、跨 OS driver support 或目标 RSS；这些仍按本 DD 的 AC-INFRA-002 与阶段门独立验收。

Infrastructure Environment 是 Host/Project 可绑定的计算环境，不成为 Project → Branch → Run → Worktree 主树的新层级。Run 固定 environment/profile/version/digest、namespace 和预算；WorktreeFocus 提供服务端验证的 checkout/workspace 映射。配置放 Advanced Settings 的 Infrastructure 内容 tab，Run 只展示授权后的有效环境、状态与资源，深链回配置。

## §1 选型、许可证与上游维护门

商业用途、行业、部署规模、席位及使用量均不得因组件或 Star 付费 tier 而设产品限制；选择的开源组件须允许不限上述范围的商业使用。GPL/AGPL/LGPL 允许商业使用和销售，copyleft 本身不是用途限制；修改、链接、bundling、installer、独立可执行文件交付与再分发按具体组合和交付形态履行对应源码、许可证、NOTICE、安装信息等义务。交付合规是发行义务，不得转化为功能、客户或付费限制；不得只因 copyleft 而要求用户必须自行安装。排除非商业、field-of-use、source-available 与实际禁止商业使用的许可。Apache-2.0、MIT、BSD、ISC、Zlib 可作为低互惠负担候选，但不是唯一商业方案。

每个 Star 发布版本均须生成并审核完整 SPDX/SBOM，覆盖直接与传递依赖，以及实际随包、由 Star 默认下载/安装/缓存/转运的可执行文件、第三方工具、容器层与镜像、guest image、kernel、guest userspace packages、firmware 等，并将组件版本和内容 digest 绑定到证据。未知许可或无法满足该交付形态所需义务的出货项阻断该打包形态，不能只审核上游主仓库 LICENSE。独立外部 provider 仍需记录版本、来源、许可与 capability，不因此免除集成审查；此许可机制不用于给 Star 核心功能添加席位/用量门。

默认支持资格还须有可复核的上游活跃度证据：在评估日之前 12 个月内至少一项正式 release/维护记录，并且至少一个公开 issue、discussion、forum 或 support channel 有 12 个月内带日期的活动记录。每次支持/发行评审必须保存版本、日期和具体 permalink；channel 首页链接不能代替活动证据。表格所列活动仅作为上游信号，不代表 Star 集成或服务等级。

| 候选 | 平台与用途 | 上游许可证/近期证据（截至 2026-10-02） | 决策边界 |
|---|---|---|---|
| Incus | Linux host/shared VM 与 system container provider 候选 | 项目声明 Apache-2.0；[7.5.1 release，2026-09-25](https://github.com/lxc/incus/releases/tag/v7.5.1)；公开 [issue #4102，2026-10-01](https://github.com/lxc/incus/issues/4102)，另有 [社区支持/发行公告](https://discuss.linuxcontainers.org/) | Linux server 优先评估；Windows/macOS client 可管理远端 Linux daemon。支持发现既有服务、安装向导或经 SBOM/许可审查的受管安装包；尚无 Star adapter/兼容性验证。 |
| Lima | macOS/Linux 可替换 VM provider 候选 | 项目 [Apache-2.0](https://github.com/lima-vm/lima/blob/master/LICENSE)；[2.2.0 release，2026-07-21](https://github.com/lima-vm/lima/releases/tag/v2.2.0)；公开 [issue #5552，2026-09-30](https://github.com/lima-vm/lima/issues/5552)。官方安装页列 macOS/Linux 为支持 host、Windows 为 untested；另有 [k3s 示例模板](https://lima-vm.io/docs/examples/containers/kubernetes/) | macOS/Linux 补充选项，支持发现既有安装、安装向导或合规受管交付；Windows 按 capability probe 验收。模板存在不表示 Star 集成或已测试。 |
| 已有/远端 Linux K3s | Linux guest/node 或远端用户集群；生产/共享 provider 独立于 Multipass 管理 | K3s 项目 [Apache-2.0](https://github.com/k3s-io/k3s/blob/main/LICENSE)；stable channel 于 2026-10-05 复核为 [v1.36.5+k3s1](https://github.com/k3s-io/k3s/blob/main/channel.yaml)；[近期公开 issue #14601，2026-09-08](https://github.com/k3s-io/k3s/issues/14601)；[Windows FAQ](https://docs.k3s.io/faq) | 仅连接明确授权且归用户管理的 Linux 集群。远端可以减少 Star 本机负载是设计推论，具体 RSS/网络代价未测。K3s 不原生支持 Windows；Multipass/Podman/WSL guest 或远端 Linux 仅是宿主路线。镜像与发行 artifact 需单独 SBOM。 |
| WSL2 | Windows OS 提供的可选 Linux guest adapter | [Microsoft WSL 配置文档](https://learn.microsoft.com/en-us/windows/wsl/wsl-config)；WSL GitHub 的 MIT 只适用于该仓库代码，不代表 Windows、kernel、发行版或镜像的许可证 | 可作为 Windows fallback，不是唯一支持路径；全局资源与停机操作按 §2、§5。不得将 WSL 仓库许可证外推至其余 OS/guest 内容。 |
| k3d | 复用用户已有 Docker engine | [MIT](https://github.com/k3d-io/k3d)；[5.8.3 stable release，2026-02-15](https://github.com/k3d-io/k3d/releases/)，另有 [5.9.0-rc.0 pre-release，2026-10-01](https://github.com/k3d-io/k3d/releases/)；公开 [issue #1696，2026-08-06](https://github.com/k3d-io/k3d/issues/1696) | 仅既有 Docker 上的开发/测试；不得自动安装 Docker 或作为生产环境推荐。k3d license 不覆盖 Docker、K3s 与镜像内容。 |
| Rancher Desktop | macOS/Windows/Linux 桌面运行时，比较项 | 根 LICENSE 为 [Apache-2.0](https://github.com/rancher-sandbox/rancher-desktop/blob/main/LICENSE)；[1.24.0 release，2026-07-29](https://github.com/rancher-sandbox/rancher-desktop/releases/)；公开 [issue #11071，2026-10-01](https://github.com/rancher-sandbox/rancher-desktop/issues/11071) | 比较项，不作为默认依赖/分发。其发行包包含多种工具，必须对每项 bundled utility、依赖与镜像做完整 SBOM/SPDX 审核，根 Apache-2.0 不足以放行。 |
| Cloud Hypervisor | Rust VMM；仅列 Linux host 后续 PoC | [v53.0 release，2026-07-12](https://github.com/cloud-hypervisor/cloud-hypervisor/releases/latest)；公开 [issue #8977，2026-09-30](https://github.com/cloud-hypervisor/cloud-hypervisor/issues/8977)；源码 per-file SPDX 示例为 [Apache-2.0 OR BSD-3-Clause](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/hypervisor/src/lib.rs) 与 [Apache-2.0 AND BSD-3-Clause](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/vm-allocator/src/system.rs) | 后续 Linux PoC；不得概括为单一 Apache-2.0，须逐文件 SPDX 并审核所有出货 artifact/依赖。支持 Windows guest 不等于支持 Windows host。 |
| Multipass | Windows/macOS/Linux 本地 VM provider，支持用 cloud-init 建 Linux guest | [GPL-3.0 LICENSE](https://github.com/canonical/multipass/blob/main/LICENSE)；GPL 明确允许商业销售/使用但再分发按 [GNU FAQ](https://www.gnu.org/licenses/gpl-faq.en.html#DoesTheGPLAllowMoney) 履约；当前 GitHub [latest stable 1.16.4，2026-09-08](https://github.com/canonical/multipass/releases/tag/v1.16.4)；上游 [PR #5202 merged，2026-09-04](https://github.com/canonical/multipass/pull/5202)；公开 [Discourse](https://discourse.ubuntu.com/c/multipass/13) 与 [Matrix](https://matrix.to/#/#multipass:ubuntu.com) 渠道 | **首选本地/CI VM provider**：由 Rust Infrastructure Manager 探测既有安装并创建/管理 Linux guest；K3s 在 guest 中运行。允许符合发行义务的受管安装/捆绑。通过受限 argv adapter 集成；`list --format json` 仅作为受界限 inventory 输入。上游安全文档将其定位为 dev/test/local，daemon/mount 权限属于宿主高权限面，不能替代 Agent sandbox。Windows/macOS/Linux 按已安装稳定版本和 driver capability 分别验收。 |
| Podman machine | Windows/macOS/Linux 本地 container/VM 管理候选；各 OS 使用各自 backend | [Podman Apache-2.0](https://github.com/podman-container-tools/podman)；[v6.1.3 release，2026-09-29](https://github.com/podman-container-tools/podman/releases/tag/v6.1.3)；官方列 Linux QEMU、macOS libkrun/AppleHV、Windows WSL/Hyper-V providers；[community meetings](https://podman.io/community) 有 2026-08-04 记录 | 产品可探测、引导安装，或履行逐项发行审查后采用受管安装/捆绑；Windows WSL/Hyper-V、macOS AppleHV 依赖宿主 OS facility。Podman machine 管理的默认 VM 并未证明 K3s readiness 或 Star Runtime sandbox，须单独 PoC。 |

推荐组合：采用 Rust-native、版本化 InfrastructureProvider contract，不设用途、行业、席位、用量或商业 tier 限制。用户工作站和 CI 的首选路径是 Multipass 管理 Linux guest、guest 内运行 K3s；Linux 多 Run/shared host 可经容量与 trust-domain 评审选择 Incus 或既有/远端 K3s，Podman machine 与 Lima 作为可切换 backend。Multipass 的首选地位是渡口产品架构决策，不代表所有 OS driver 都已验收，也不代表 Multipass 适合生产集群控制面。按 operation/capability 公开 provider 差异并支持用户自有 provider。产品应提供 provider 探测、安装/升级引导和多种合规交付形态；GPL 允许不限用途商业使用，产品分发/修改/组合仍须按实际情形履行 GPLv3 义务，不得把许可履约变成功能或付费门。

### §1.1 Multipass 首选路径决策边界

| 决策项 | 设计基线 | 验收/限制 |
|---|---|---|
| 产品范围 | 本地开发、测试与 CI 的 Linux guest 生命周期由 Multipass 管理；guest 中运行 K3s | 生产控制面以独立远端/用户自有 provider 为主，除非另有可靠性与支持级别设计，不用 Multipass 代替生产虚拟化/集群管理 |
| 共享拓扑 | 默认一个 Host/environment 共用一套按需 VM 与单 server K3s；多个 Project/Run 使用独立 namespace、身份、配额和策略 | 不为每个 Worktree/Agent 复制 VM、控制面、daemon 或镜像缓存；多节点仅由已批准 profile 和 host admission 决定，本地 profile 不承诺 HA |
| 首选不等于普适 | Multipass adapter 是实现顺序第一的本地 VM adapter | 具体安装版本、driver、网络、磁盘、暂停/恢复能力均以 `probe` 结果为准；capability 不满足时给出可解释 fallback，不将 unsupported 平台标为已支持 |
| 管理权限 | GUI/主进程保持普通用户权限；最小 Rust Infrastructure Manager 经 typed IPC 运行 provider command allowlist，实际 daemon 权限由 OS/Multipass 管理 | Agent/Plugin/Task CLI/guest workload 不接触 Multipass CLI、daemon socket/credential、任意 host shell 或拥有者不明的 VM |
| 工作区传输 | Run/Worktree path 由当前授权 resolver 解析，提供受控 guest workspace bridge | 默认不使用可写 host mount；若 bridge 不可用或信任要求超出 provider capability，则禁用执行，不挂载用户 home/repository 根目录 |
| 安全与运行时 | K3s 管理编排资源与 readiness | K3s namespace、RBAC、NetworkPolicy 不单独证明 hostile-code sandbox；OS/VM Runtime sandbox、原生 Hook、取消/恢复、独立验证仍是 CLI enable gate |

Multipass 的 CLI 支持机器可读 JSON inventory（[官方 `list` 文档](https://canonical.com/multipass/docs/stable-docs/reference/command-line-interface/list/)），并支持基于 cloud-init 初始化 VM（[官方 cloud-init 指南](https://canonical.com/multipass/docs/stable/how-to-guides/manage-instances/launch-customized-instances-with-multipass-and-cloud-init/)）。Manager 只解析有大小/时间上限的 JSON，cloud-init 与引用资源必须本地固定版本/digest；不把任意 URL、shell、用户输入参数透传为 privileged operation。

Multipass driver 能力按平台不同（[官方 driver 矩阵](https://canonical.com/multipass/docs/latest/explanation/driver/)）：Linux 使用 QEMU；macOS 支持 QEMU 与 Apple Virtualization，VirtualBox 已 deprecated；Windows 主要使用 Host Compute System，Hyper-V/VirtualBox 已 deprecated。Manager 不能用“Multipass 已安装”推断功能相同；每次 probe 应记录 OS、Multipass 版本、active driver、mount/network/suspend/resume/architecture 等 capability，并由具体 profile 决定可用性。
默认支持资格还须有可复核的上游活跃度证据：在评估日之前 12 个月内至少一项正式 release/维护记录，并且至少一个公开 issue、discussion、forum 或 support channel 有 12 个月内带日期的活动记录。每次支持/发行评审必须保存版本、日期和具体 permalink；channel 首页链接不能代替活动证据。

## §2 共享与隔离、资源公平

默认按 Host/已授权 environment 复用一套按需启动的基础设施。Project/Run 通过 explicit namespace binding、ServiceAccount/RBAC、ResourceQuota/LimitRange、默认拒绝 NetworkPolicy、受限 Pod Security、受控 PVC 和端口分配使用资源。禁止按每个 Worktree/Agent 重复启动完整 VM/控制面。Host admission 必须扣除 OS/base reserve、明确 host safety reserve、已承诺的 managed reservations 和可观测的 unmanaged load；按 Project 再到 Run/Agent 执行有界、公平、可取消的分配，避免单一 Project/Agent 占满共享池。资源读数或硬限制不足以证明 headroom 时不得假定有空闲预算。

WSL 中非渡口管理的 distro/workload 若没有可靠的可观测用量与可执行硬上限，其资源预算一律视为 unknown，WSL 可安全分配余量也视为 unknown。仍须保留明确 host reserve；遇到 unknown 时拒绝新的 WSL allocation/admission，并提示用户释放或显式管理资源，不能用瞬时空闲量或总内存猜测可承载能力。

Namespace 是组织与权限范围，不能独自证明恶意 Agent code 的强隔离。执行不可信代码须由已验证 Runtime sandbox/VM backend 提供实际隔离；没有该 capability 时禁用对应执行，不把集群 admin kubeconfig、privileged Pod、宿主 root/socket 或任意 hostPath 交给 Agent/Plugin。InfrastructureBackend 与 AgentRuntimeBackend 是独立 provider，可按部署需求组合。

#### Multipass-managed guest 与 Run/Worktree 共用模型

`HostEnvironment` 管理 Multipass instance 与 K3s control plane；`RunInfrastructureBinding` 将 Project/Branch/Run 绑定到该环境的 namespace、ServiceAccount、配额、NetworkPolicy、Runtime Profile 与资源 reservation。VM identity 只由 Manager 创建的 `environment_id + generation` owner record 决定，不从名称前缀单独推断归属。Provider inventory 发现未知实例时只显示为 external/unmanaged，不允许 Star 自动 stop/delete。

WorktreeFocus 由可信目录 owner 解析；workspace bridge 绑定 `(tenant, project, branch, engineering_run, worktree, binding_revision, task_execution_run)`，操作前后重验当前 ACL 与 revision。优先实现经过 digest 校验的有限文件/变更同步或 guest 内 checkout；把宿主 checkout 作为只读输入、guest 侧 overlay/独立 checkout 作为写入目标。若后续选择 Multipass mount，必须先验证 Windows/macOS/Linux 驱动的权限、只读/卸载语义和路径边界，并由原生 Hook 显式授权；不得假设 driver 提供一致 mount 安全语义。

默认单 server guest 用于低成本开发/测试，K3s 单 server 可采用上游支持的 embedded datastore profile；需要多节点时通过同一 versioned profile 增加 agent/server，按其 HA 拓扑与容量重新审核。K3s server、agent 与 workload 都占资源；单纯创建 namespace 不会降低 guest 内存消耗，Run quota 必须同时限制 pod CPU/memory/ephemeral storage、并发与日志/镜像保留。CI runner 是另一个明确 owner 的 HostEnvironment，按 ephemeral lifetime 清理其自身实例，不能复用用户工作站的无主全局清理权限。

K3s 基线复用其 containerd；增加 Docker/第二个 containerd/第二个 broker 需要明确用途、端点 owner 和预算。readiness 必须验证 CRI/节点/目标服务可用与期限，不能由 process 存在或端口打开推导就绪。PostgreSQL SoR、owner API/存储过程与 Outbox/Inbox 的现有边界保持一致。

## §3 版本化 Rust 契约

InfrastructureBackendV1 提供 typed discover/probe/plan/create/start/stop/inspect/delete，以及可选 suspend/resume、image import/export 能力。每次返回 capability/version/digest；未知或不支持的操作明确禁用。Executable 与 argv 由 verified profile 构造，路径/镜像/端口由受权 registry 解析；UI 不能提交任意 shell、宿主路径或 kube context。

`MultipassAdapterV1` 为该契约的首个本地实现，结构化输入 `environment_id`、期望 VM state revision、profile revision/digest、guest image alias+digest、cloud-init digest、资源 reservation、operation/idempotency/correlation IDs、actor 与 deadline；不接受裸命令字符串。子进程只能以 argv 数组启动并复核可执行文件路径/版本；stdout/stderr bytes、运行时长、并发 operation 与 result retention 均有界，超时/取消终止子进程并写入终态 receipt。`list --format json` 必须做 schema/version/size/state 校验。命令 allowlist 仅覆盖由已安装版本 capability probe 确认的 `version/list/info/launch/start/stop/wait-ready`；Multipass 的 [`wait-ready`](https://canonical.com/multipass/docs/latest/reference/command-line-interface/wait-ready/) 只等待 daemon 初始化并可接受请求，不代表 VM 或 K3s readiness。单实例永久回收仅允许 `delete --purge <exact-owned-instance>`，必须与 owner ledger 精确匹配且通过 drain、lease release、retention、Hook 和二次授权；无参数全局 `purge` 与 `delete --all` 永不允许（见官方 [`delete`](https://canonical.com/multipass/docs/latest/reference/command-line-interface/delete/)）。VM Running 由单实例 `info/list` 核验；guest/K3s readiness 通过有版本和 digest 的固定 profile probe 检查 API endpoint、node Ready、CRI 与必要 system workload。若支持 guest probe 的版本提供 `multipass exec`，它只可调用预置固定 executable `/usr/local/libexec/star/guest-ready`，其 argv/env/cwd/输出上限均由 profile 固定；任意用户/Task/Agent 命令不得借用 Infrastructure Adapter 的 exec 权限，实际 Task 命令只经独立 AgentRuntimeProvider 与 sandbox。`shell`、任意 `mount`、任意 `exec`、全局 purge、未授权实例操作与更改全局 driver/settings 默认拒绝。必要 guest setup 命令由固定 cloud-init payload 完成，不由 Agent 注入。

每个 create/upgrade `plan` 显示宿主/VM 资源增量、image/guest/K3s digest、网络/数据面变化、workspace access 和 rollback/删除范围；native Hook 根据 Project/Worktree effective policy 与 User confirmation gate 给出 Allow/Deny/RequireHuman。长操作使用短事务创建 operation lease，锁外等待 Multipass/K3s readiness，完成后短事务重授权/CAS 与写 Audit/Outbox；不在 DB transaction 中等待 VM 启动。Readiness 分为 Multipass daemon、单一 VM、K3s control plane/node/CRI、Agent Runtime sandbox 四个带时间戳的独立事实，不得由 daemon `wait-ready` 或 VM Running 推断更高层就绪。Manager 持久化 operation owner 与 generation，掉电恢复时 reconcile 只对账/接管自己拥有的实例。

InfrastructureProfileV1 分开固定 host_executable_os/arch 与 guest_os/arch，另固定 backend/version、guest image/kernel/k3s/container runtime digest、所需 capabilities、CPU/memory/disk/IO/network budgets、mount/network/retention policy。宿主可执行文件必须匹配 Host OS/architecture；Guest command/payload 由已验证的 guest agent/SSH/Kubernetes API 在匹配 Guest OS/architecture 的环境执行。host_worktree_path → guest_workdir 必须经 Worktree owner 校验并版本化；cwd 使用 guest 内规范路径，不能假定 host 与 guest 路径相同。target、映射或 backend capability 不符时拒绝。

RunInfrastructureBindingV1 固定 tenant/Project/Branch/Run/environment、profile revision、namespace、runtime identity 与资源租约。命令携带 actor/current grants、command/correlation ID、expected revision 与 deadline；operation receipt 只返回脱敏 ID/state/reason。Runtime readiness 等待期间释放 DB row locks，最终短事务重新授权/CAS；基础设施 operation 不授予 Task/CLI 权限。Kube/VM 凭据由 credential owner 保管，UI/日志/BI 不返回 Secret。

## §4 生命周期、Hook、Loop 与 BI

状态目标为 absent → provisioning → stopped → starting → ready → draining → stopped → deleting → absent，错误进入带 reason 的 degraded/failed；永久删除仅在独立确认/保留门后触发，suspend 只对声明并验收 capability 的 backend 开放。创建/升级/网络/mount/删除均经 native Hook、plan、授权和最终复核；插件只能建议，不可绕过安全门。

按需启动与 idle stop 受 durable lease、Agent/Task/Loop/DB/Outbox 活动和 checkpoint/备份状态控制。停止前拒绝新准入并 drain 当前操作；活跃或 unknown 不能自动销毁。Schedule Loop 使用唯一 occurrence/租约机制，不另建 cron owner；重启恢复幂等 command 并重验 scope/fence。Upgrade 固定版本、迁移/备份与 rollback receipt；不能假定跨 hypervisor 版本 snapshot 可恢复。

事件固定 environment/Run/profile/version/digest、operation、资源/延迟摘要、Hook decision、failure/drain reason；脱敏 append-only Audit 与 Outbox 联动 BI/Benchmark。BI 建议经已生效 native policy 采纳；不能直接扩大内存、权限或重启用户环境。

## §5 低内存与操作所有权

K3s 官方要求 server 最低 2 cores / 2 GB RAM，且基线不含 workload；参见[官方 requirements](https://docs.k3s.io/installation/requirements)。现有单机 PoC 使用 4 vCPU / 8 GiB / 28 GiB guest，在 31.8 GB 主机上实际运行 K3s 与 19 个 GitOps/基础设施 Pod；详见实跑报告。它只证明该固定 host/profile 可启动并达到当前服务 readiness，不提供 Star 桌面进程树、Agent workload、并行 fairness 或峰值 RSS 基准。2 vCPU / 4 GiB 仍只是另一个待测 PoC profile，不代表产品最低配置或已测可行性。测试须纳入 PostgreSQL/NATS/真实 Rust 服务及 1/2/4/8 Agent 固定负载，测峰值、p95/取消、冷启动与长运行漂移，并扣除 OS/宿主保留量；结果出炉前预算保持 provisional。

WSL2 .wslconfig 的 memory/processors、autoMemoryReclaim 等是[影响所有 WSL2 distro 的全局设置](https://learn.microsoft.com/en-us/windows/wsl/wsl-config)。wsl --shutdown 会立即终止所有运行中的 distro 和 WSL2 utility VM（[官方说明](https://learn.microsoft.com/en-us/windows/wsl/basic-commands#shutdown)）。修改 .wslconfig 或执行全局 shutdown 前必须在产品界面明确说明影响范围并获得用户显式授权；禁止静默修改或把全局操作当作常规回收。只对明确属于渡口的 distro/VM/namespace/PVC 执行其余生命周期操作。非渡口 WSL workload 的 cap 不可证实时按 §2 视为 unknown、保留 host reserve 并拒绝准入；不得声称资源隔离已达硬限。

Rust desktop UI 不直接持有 Multipass CLI session、VM logs 或 Kubernetes watch 全量历史。一个按需运行的 Host Infrastructure Manager 使用普通用户权限和窄 typed IPC 操作 Multipass CLI；若平台要求额外授权，由 OS/Multipass 明示授权，不提升整个 GUI、Agent 或 Plugin 权限。Manager 使用共享 Tokio executor、有界 per-host operation queue、公平 admission、deadline/cancel/drain、分页状态窗口与有界 log cache；耗时 hypervisor/Kube/IO 在 UI 线程外执行。每个 Host/environment 共享一份按 digest 缓存的 guest image/K3s layers，并设容量与 TTL；不按 Run/Worktree 重复下载全镜像。Host RAM 预算必须同时计算桌面进程树、base OS、Multipass VM 分配、K3s/containerd、guest workloads 与未托管用量；unknown load 作为 unknown capacity，不过量承诺。Suspend/stop 是否释放内存需实测，磁盘/数据库持久性须有明确 retention 与备份；无主操作须有 reconciler，失败不会留下静默活跃进程。

## §6 持久化分类与后续 schema 门

以下为**拟分类与设计要求**，尚未创建表或宣称 RLS 已实现：

| 拟分类对象 | 初步分类 | 设计要求 |
|---|---|---|
| Profile/backend trust、VM image/cloud-init/K3s catalog、HostEnvironment、Multipass owned-instance ledger 与 Project/Run binding | Master | SCD2/version/digest、no physical delete、instance owner/generation 明确、独立 role/policy/capability 分类；目标库完整 RLS 与 audit |
| Provision/upgrade/resource reservation、workspace bridge transfer、Hook/backup receipts、Audit/Outbox events | Transaction | append-only 或版本化 CAS、scope/audit/retention 与完整 RLS；evidence 不保存凭据/源文件内容 |
| Operation/lease/queue、health observation/log window、provider inventory cache、bootstrap cache | Work | 显式短 TTL/retention、bounded cache/queue、可清理；不得成为授权来源；inventory unknown 不触发生命周期副作用 |

启动目标 PostgreSQL 之前的 bootstrap trust/inventory/audit 如何持久化仍需单独 schema/安全设计：不能循环依赖未启动 DB，也不能把本地 cache 当 Master authority。SQLite 没有 PostgreSQL 原生 RLS，若采用本地持久 substrate，必须明确逻辑 scope enforcement/加密/不可变证据及与仓库完整 RLS 规则的差距；未关闭前不启用持久 writer。下一阶段对实际每张表完成 W/T/M 100% 分类、13类 scope 评审和运行角色验收。

## §7 Phase 与验收

1. INFRA-1：冻结 provider/profile/binding schema、HostEnvironment/owner/generation lifecycle、operation/license-obligation record、完整 SPDX/SBOM 与近 12 个月社区活跃证据门；定义 capability negotiation 与真实 discover/probe/readiness contract。
2. INFRA-2（首选实现）：将现有宿主级 Multipass/HCS 实跑当作平台 PoC/回归基线，核对可复用的 bootstrap 与 GitOps 声明；另行实现 Star 的 Multipass inventory/capability probe 与 allowlisted subprocess adapter、VM ownership ledger、固定 cloud-init/image/K3s digest、single-server guest provision/readiness、operation lease/audit/outbox、显式 install/license flow 与产品用户可见状态。现有 PowerShell/bootstrap 路径不是 Rust provider contract 或 Run-bound Infrastructure Manager。先验收本地/CI 选定 OS/driver，不将单个平台测试外推为全平台支持。K3s 只运行于 Linux guest；不得因 GPL 要求产品只允许手工自装。
3. INFRA-3：实现 Host-shared VM 与多 Run namespace/quota/fair admission、workspace bridge/路径边界、stop/drain/recovery、owner reconciliation 与 credentials scope；确定何时添加 K3s agent VM 的容量准入。未经验证的 mount/Runtime sandbox capability 默认禁用执行。
4. INFRA-4：接入 Podman machine、Linux Incus、Lima 和 existing/remote Linux K3s fallback；k3d 若评估须复用已安装容器引擎；Cloud Hypervisor 单独做 Linux host PoC 和逐文件 SPDX 审核。每项均需 capability probe，不得写成未验证即已集成。
5. INFRA-5：固定设备/workload 的桌面进程树、VM/K3s/Agent peak RSS、CPU/IO/p95、1–8 Agent 并行公平、冷启动、崩溃/撤权/取消/断网/磁盘满/恢复验收，SBOM/source/notice 与商业分发审查。

真实环境可创建/复用→绑定 Run→服务就绪→受控任务实际执行→取消/恢复→独立验证回写，且不会影响其他 user resources，才满足闭环。所有步骤目前仍开放；本次只修订设计与实施依赖，不安装宿主功能或改变已有集群。

## §8 审阅与修订历史

| 版本 | 日期 | 审核/修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 用户要求适配 Rust 商业项目的 Multipass 类 k3s 配套；provider 路线、资源/Hook/Run 契约与真实未实现门 |
| v0.2 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 按商业 OSS 限制收敛许可证白名单与 provider 路线；补齐发行/社区证据门、WSL 全局资源授权、未知预算拒绝准入、Host/Guest target 区分及未验证状态；复核官方来源 |
| v0.3 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 依用户反馈移除“copyleft=不商用/一概排除”限制；将 Multipass 作为跨平台本地 VM 首选候选、补充 Podman machine 和 provider fallback；更新 GPLv3 履约形态、按版本 capability gate、Multipass 安全定位与活跃社区证据；provider 仍未安装/验收 | 用户要求不受商业用途限制且社区活跃的开源方案 |
| v0.4 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 明确 unlimited commercial use 不得转成 paid tier/use-case gate；删除 copyleft provider 只能手工自装的隐性限制；支持发现、引导及合规受管安装/捆绑；纠正 K3s stable channel 到 v1.36.4 并补充近期 release/社区活动证据 | 用户再次明确拒绝商业使用限制并要求活跃社区 |
| v0.5 | 2026-10-05 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 将 Multipass 固定为本地/CI Linux VM 与 guest K3s 的首选管理路径，区分生产/远端 provider；增加共享 HostEnvironment、restricted Rust adapter/CLI allowlist、Worktree guest workspace bridge、资源准入和真实验收分期；引用 Multipass/K3s 官方安全、driver、cloud-init、资源与拓扑资料；复核 K3s stable channel 为 v1.36.5+k3s1；所有功能仍未实现或验收 | 用户明确后续会用 Multipass 管理虚拟机中的 K3s，要求将重大架构变化纳入设计 |
| v0.6 | 2026-10-05 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 对账 dev 已有 Windows 11 Home + Multipass 1.17.0-rc1/HCS + Ubuntu guest K3s v1.36.5 + 19/19 GitOps Pod 运行证据；将其定位为宿主层 PoC，不外推为 Star Rust Manager/Run binding/sandbox/RSS 已实现；补充真实 VM 配置与 INFRA-2 复用/集成门 | 将 dev 上已提交的宿主层实跑报告纳入本地基础设施详细设计 |
| v0.7 | 2026-10-05 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 依据 Canonical CLI reference 澄清 wait-ready 只代表 daemon ready；将 VM/K3s/Runtime sandbox 列为独立 readiness 层；只允许固定 profile guest probe 走 Multipass exec，并将永久删除限定为精确 owner 的 delete --purge，禁止全局 purge/delete --all | 官方 CLI 核验发现 daemon readiness、guest/K3s readiness 与实例回收有不同语义 |
| v0.8 | 2026-10-05 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 同步 requirements v5.72、basic design v5.69 和实施计划 v5.95 的上游版本指针；不改变 provider 行为、生命周期契约或验收结论 | 文档版本一致性复核 |
