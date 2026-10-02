# DD-LOCAL-INFRASTRUCTURE-001 — Rust 本地基础设施与 k3s

> v0.2 · 2026-10-02 · Draft / 设计候选；provider 未安装或实现，兼容性与 RSS 均未验证。
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 上游：requirements v5.45 / basic-design v5.42 §16.21 / Group DD v4.29 §8.8；实施计划 §6.69。

## §0 目的与边界

为渡口增加 provider-neutral 的 Rust Host Infrastructure Manager，管理本地或远端 Linux 计算环境、k3s readiness、资源预算与有界操作。平台路线候选为：Windows 使用 OS 提供的 WSL2 adapter 做 PoC；macOS 评估外部安装的 Lima；Linux 评估外部安装的 Incus；已有/远端 Linux k3s 可作为减少本机 RSS 的路线假设；k3d 仅复用既有 Docker 做开发/测试；Cloud Hypervisor 留作 Linux host 后续 PoC。以上均是拟议路线，不表示已支持或验证。K3s 承载 PostgreSQL/NATS/后端 App 和受控开发服务；Agent CLI 的 admission、native Hook、sandbox、取消、独立验证与结果写回仍属于 Runtime owner，安装集群不会自动满足这些门。

Infrastructure Environment 是 Host/Project 可绑定的计算环境，不成为 Project → Branch → Run → Worktree 主树的新层级。Run 固定 environment/profile/version/digest、namespace 和预算；WorktreeFocus 提供服务端验证的 checkout/workspace 映射。配置放 Advanced Settings 的 Infrastructure 内容 tab，Run 只展示授权后的有效环境、状态与资源，深链回配置。

## §1 选型、许可证与上游维护门

Star 默认支持、默认安装或分发、打包进产品及加入核心依赖的组件，只允许 Apache-2.0、MIT、BSD-2-Clause、BSD-3-Clause、ISC、Zlib 等宽松且无使用限制的许可。必须保留各许可证要求的版权/许可声明；Apache-2.0 的 NOTICE（如适用）与专利许可/条件也须保留。项目根目录的许可证只说明该项目相应范围，不能代表其完整依赖、发行包或镜像。

每个 Star 发布版本均须生成并审核完整 SPDX/SBOM，覆盖直接与传递依赖，以及实际随包、由 Star 默认下载/安装/缓存/转运的可执行文件、第三方工具、容器层与镜像、guest image、kernel、guest userspace packages、firmware 等，并将组件版本和内容 digest 绑定到证据。任何未知许可证或不在许可白名单内的出货项均阻断默认分发；不能只审核上游主仓库 LICENSE。仅外部安装且不由 Star 分发的 provider，不据此豁免集成版本/来源记录；其外部安装物只有进入 Star 管理或分发范围时才按对应 artifact 清单判定。

默认支持资格还须有可复核的上游活跃度证据：在评估日之前 12 个月内至少一项正式 release/维护记录，并且至少一个公开 issue、discussion、forum 或 support channel 有 12 个月内带日期的活动记录。每次支持/发行评审必须保存版本、日期和具体 permalink；channel 首页链接不能代替活动证据。以下是截至 2026-10-02 的官方快照示例，仅证明上游发布/公开协作信号，不证明 Star 集成或服务等级。

| 候选 | 平台与用途 | 上游许可证/近期证据（截至 2026-10-02） | 决策边界 |
|---|---|---|---|
| Incus | Linux host；外部安装 provider 候选 | 项目声明 Apache-2.0；[7.5.1 release，2026-09-25](https://github.com/lxc/incus/releases/)；公开 [issue #4102，2026-10-01](https://github.com/lxc/incus/issues/4102)，另有 [社区支持/发行公告](https://discuss.linuxcontainers.org/) | Linux 优先评估；不由 Star 静默安装或随包分发。仅候选，尚无 Star adapter/兼容性验证。 |
| Lima | macOS host；外部安装 provider 候选 | 项目 [Apache-2.0](https://github.com/lima-vm/lima/blob/master/LICENSE)；[2.2.0 release，2026-07-21](https://github.com/lima-vm/lima/releases/)；公开 [issue #5552，2026-09-30](https://github.com/lima-vm/lima/issues/5552)。官方安装页列 macOS/Linux 为支持 host、Windows 为 untested；另有 [k3s 示例模板](https://lima-vm.io/docs/examples/containers/kubernetes/) | macOS 优先评估，外部安装；Windows 不列为支持路线。模板存在不表示 Star 集成或已测试。 |
| 已有/远端 Linux K3s | Linux guest/node 或远端用户集群；候选用于降低本机驻留资源 | K3s 项目 [Apache-2.0](https://github.com/k3s-io/k3s/blob/main/LICENSE)；[v1.37.0+k3s1 release，2026-09-14](https://github.com/k3s-io/k3s/releases/)；公开 [issue #14723，2026-09-30](https://github.com/k3s-io/k3s/issues/14723)；[Windows FAQ](https://docs.k3s.io/faq) | 仅连接明确授权且归用户管理的集群。远端可以减少 Star 本机负载是设计推论，具体 RSS/网络代价未测。K3s 本身不原生支持 Windows；WSL2 PoC 中它运行在 Linux guest 内。镜像与发行 artifact 需单独 SBOM。 |
| WSL2 | Windows OS 提供的 Linux adapter | [Microsoft WSL 配置文档](https://learn.microsoft.com/en-us/windows/wsl/wsl-config)；WSL GitHub 的 MIT 只适用于该仓库代码，不代表 Windows、kernel、发行版或镜像的许可证 | 仅 Windows PoC，不作为 Star 分发的 provider。不得将 WSL 仓库许可证外推至其余 OS/guest 内容；全局资源与停机操作按 §2、§5。 |
| k3d | 复用用户已有 Docker engine | [MIT](https://github.com/k3d-io/k3d)；[5.8.3 stable release，2026-02-15](https://github.com/k3d-io/k3d/releases/)，另有 [5.9.0-rc.0 pre-release，2026-10-01](https://github.com/k3d-io/k3d/releases/)；公开 [issue #1696，2026-08-06](https://github.com/k3d-io/k3d/issues/1696) | 仅既有 Docker 上的开发/测试；不得自动安装 Docker 或作为生产环境推荐。k3d license 不覆盖 Docker、K3s 与镜像内容。 |
| Rancher Desktop | macOS/Windows/Linux 桌面运行时，比较项 | 根 LICENSE 为 [Apache-2.0](https://github.com/rancher-sandbox/rancher-desktop/blob/main/LICENSE)；[1.24.0 release，2026-07-29](https://github.com/rancher-sandbox/rancher-desktop/releases/)；公开 [issue #11071，2026-10-01](https://github.com/rancher-sandbox/rancher-desktop/issues/11071) | 比较项，不作为默认依赖/分发。其发行包包含多种工具，必须对每项 bundled utility、依赖与镜像做完整 SBOM/SPDX 审核，根 Apache-2.0 不足以放行。 |
| Cloud Hypervisor | Rust VMM；仅列 Linux host 后续 PoC | [v53.0 release，2026-07-12](https://github.com/cloud-hypervisor/cloud-hypervisor/releases/latest)；公开 [issue #8977，2026-09-30](https://github.com/cloud-hypervisor/cloud-hypervisor/issues/8977)；源码 per-file SPDX 示例为 [Apache-2.0 OR BSD-3-Clause](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/hypervisor/src/lib.rs) 与 [Apache-2.0 AND BSD-3-Clause](https://github.com/cloud-hypervisor/cloud-hypervisor/blob/main/vm-allocator/src/system.rs) | 后续 Linux PoC；不得概括为单一 Apache-2.0，须逐文件 SPDX 并审核所有出货 artifact/依赖。支持 Windows guest 不等于支持 Windows host。 |
| Multipass | Ubuntu VM manager | [GPLv3 LICENSE](https://github.com/canonical/multipass/blob/main/LICENSE)；[GNU FAQ](https://www.gnu.org/licenses/gpl-faq.en.html#DoesTheGPLAllowMoney) 明确 GPL 不禁止收费/商业分发，但分发者必须遵守相应 GPL 条件 | 因产品要求排除 copyleft/source-disclosure 义务，明确排除 Star 默认支持、默认安装、bundle 和核心依赖；此项政策选择不是声称 GPL 禁止商用。 |

默认支持/分发审查必须再次核实当时的正式 release/维护记录和有日期的公开渠道活动；此处的上游快照不是 Star 适配器验证、性能结果或支持 SLA。若未来发现 artifact 不在白名单或 12 个月活跃度证据缺失，应暂停该 provider 的默认支持/出货，直至审计通过或替换路线。

## §2 共享与隔离、资源公平

默认按 Host/已授权 environment 复用一套按需启动的基础设施。Project/Run 通过 explicit namespace binding、ServiceAccount/RBAC、ResourceQuota/LimitRange、默认拒绝 NetworkPolicy、受限 Pod Security、受控 PVC 和端口分配使用资源。禁止按每个 Worktree/Agent 重复启动完整 VM/控制面。Host admission 必须扣除 OS/base reserve、明确 host safety reserve、已承诺的 managed reservations 和可观测的 unmanaged load；按 Project 再到 Run/Agent 执行有界、公平、可取消的分配，避免单一 Project/Agent 占满共享池。资源读数或硬限制不足以证明 headroom 时不得假定有空闲预算。

WSL 中非渡口管理的 distro/workload 若没有可靠的可观测用量与可执行硬上限，其资源预算一律视为 unknown，WSL 可安全分配余量也视为 unknown。仍须保留明确 host reserve；遇到 unknown 时拒绝新的 WSL allocation/admission，并提示用户释放或显式管理资源，不能用瞬时空闲量或总内存猜测可承载能力。

Namespace 是组织与权限范围，不能独自证明恶意 Agent code 的强隔离。执行不可信代码须由已验证 Runtime sandbox/VM backend 提供实际隔离；没有该 capability 时禁用对应执行，不把集群 admin kubeconfig、privileged Pod、宿主 root/socket 或任意 hostPath 交给 Agent/Plugin。InfrastructureBackend 与 AgentRuntimeBackend 是独立 provider，可按部署需求组合。

K3s 基线复用其 containerd；增加 Docker/第二个 containerd/第二个 broker 需要明确用途、端点 owner 和预算。readiness 必须验证 CRI/节点/目标服务可用与期限，不能由 process 存在或端口打开推导就绪。PostgreSQL SoR、owner API/存储过程与 Outbox/Inbox 的现有边界保持一致。

## §3 版本化 Rust 契约

InfrastructureBackendV1 提供 typed discover/probe/plan/create/start/stop/inspect/delete，以及可选 suspend/resume、image import/export 能力。每次返回 capability/version/digest；未知或不支持的操作明确禁用。Executable 与 argv 由 verified profile 构造，路径/镜像/端口由受权 registry 解析；UI 不能提交任意 shell、宿主路径或 kube context。

InfrastructureProfileV1 分开固定 host_executable_os/arch 与 guest_os/arch，另固定 backend/version、guest image/kernel/k3s/container runtime digest、所需 capabilities、CPU/memory/disk/IO/network budgets、mount/network/retention policy。宿主可执行文件必须匹配 Host OS/architecture；Guest command/payload 由已验证的 guest agent/SSH/Kubernetes API 在匹配 Guest OS/architecture 的环境执行。host_worktree_path → guest_workdir 必须经 Worktree owner 校验并版本化；cwd 使用 guest 内规范路径，不能假定 host 与 guest 路径相同。target、映射或 backend capability 不符时拒绝。

RunInfrastructureBindingV1 固定 tenant/Project/Branch/Run/environment、profile revision、namespace、runtime identity 与资源租约。命令携带 actor/current grants、command/correlation ID、expected revision 与 deadline；operation receipt 只返回脱敏 ID/state/reason。Runtime readiness 等待期间释放 DB row locks，最终短事务重新授权/CAS；基础设施 operation 不授予 Task/CLI 权限。Kube/VM 凭据由 credential owner 保管，UI/日志/BI 不返回 Secret。

## §4 生命周期、Hook、Loop 与 BI

状态目标为 absent → provisioning → stopped → starting → ready → draining → stopped，错误进入带 reason 的 degraded/failed；suspend 只对声明并验收 capability 的 backend 开放。创建/升级/网络/mount/删除均经 native Hook、plan、授权和最终复核；插件只能建议，不可绕过安全门。

按需启动与 idle stop 受 durable lease、Agent/Task/Loop/DB/Outbox 活动和 checkpoint/备份状态控制。停止前拒绝新准入并 drain 当前操作；活跃或 unknown 不能自动销毁。Schedule Loop 使用唯一 occurrence/租约机制，不另建 cron owner；重启恢复幂等 command 并重验 scope/fence。Upgrade 固定版本、迁移/备份与 rollback receipt；不能假定跨 hypervisor 版本 snapshot 可恢复。

事件固定 environment/Run/profile/version/digest、operation、资源/延迟摘要、Hook decision、failure/drain reason；脱敏 append-only Audit 与 Outbox 联动 BI/Benchmark。BI 建议经已生效 native policy 采纳；不能直接扩大内存、权限或重启用户环境。

## §5 低内存与操作所有权

K3s 官方要求 server 最低 2 cores / 2 GB RAM，且基线不含 workload；参见[官方 requirements](https://docs.k3s.io/installation/requirements)。这不是渡口整套服务的实测预算。2 vCPU / 4 GiB 仅为 PoC 的暂定测试 profile，用于测量与验证 admission，不代表产品最低配置、可行性结论或已测 RSS。测试须纳入 PostgreSQL/NATS/真实 Rust 服务及 1/2/4/8 Agent 固定负载，测峰值、p95/取消、冷启动与长运行漂移，并扣除 OS/宿主保留量；结果出炉前预算保持 provisional。

WSL2 .wslconfig 的 memory/processors、autoMemoryReclaim 等是[影响所有 WSL2 distro 的全局设置](https://learn.microsoft.com/en-us/windows/wsl/wsl-config)。wsl --shutdown 会立即终止所有运行中的 distro 和 WSL2 utility VM（[官方说明](https://learn.microsoft.com/en-us/windows/wsl/basic-commands#shutdown)）。修改 .wslconfig 或执行全局 shutdown 前必须在产品界面明确说明影响范围并获得用户显式授权；禁止静默修改或把全局操作当作常规回收。只对明确属于渡口的 distro/VM/namespace/PVC 执行其余生命周期操作。非渡口 WSL workload 的 cap 不可证实时按 §2 视为 unknown、保留 host reserve 并拒绝准入；不得声称资源隔离已达硬限。

Rust manager 使用共享 Tokio executor、有界 per-host operation queue、公平 admission、deadline/cancel/drain、分页状态窗口与有界 log cache；耗时 hypervisor/Kube/IO 在 UI 线程外执行。Image cache 按 digest 共享且有上限。Suspend/stop 释放宿主内存须实测，磁盘/数据库持久性须有明确 retention 与备份；无主操作须有 reconciler，失败不会留下静默活跃进程。

## §6 持久化分类与后续 schema 门

以下为**拟分类与设计要求**，尚未创建表或宣称 RLS 已实现：

| 拟分类对象 | 初步分类 | 设计要求 |
|---|---|---|
| Profile/backend trust/image catalog、environment 与 Project/Run binding | Master | SCD2/version/digest、no physical delete、独立 role/policy/capability 分类；目标库完整 RLS 与 audit |
| Operation/Hook/resource/upgrade/backup receipts、Audit/Outbox events | Transaction | append-only、scope/audit/retention 与完整 RLS；evidence 不保存凭据 |
| Operation lease/queue、health observation/log window、bootstrap cache | Work | 显式短 TTL/retention、bounded cache/queue、可清理；不得成为授权来源 |

启动目标 PostgreSQL 之前的 bootstrap trust/inventory/audit 如何持久化仍需单独 schema/安全设计：不能循环依赖未启动 DB，也不能把本地 cache 当 Master authority。SQLite 没有 PostgreSQL 原生 RLS，若采用本地持久 substrate，必须明确逻辑 scope enforcement/加密/不可变证据及与仓库完整 RLS 规则的差距；未关闭前不启用持久 writer。下一阶段对实际每张表完成 W/T/M 100% 分类、13类 scope 评审和运行角色验收。

## §7 Phase 与验收

1. INFRA-1：冻结 provider/profile/binding schema、版本/许可与 SPDX/SBOM 清单、维护活跃度证据门和 operation/安全/资源能力矩阵；实现真实 discover/probe/readiness。
2. INFRA-2：Windows WSL2 PoC（K3s 只运行在 Linux guest 内）；覆盖 create/start/stop/health、systemd/cgroup/network/readiness 与自有资源 ownership；任何全局配置/停机先经过显式授权门。
3. INFRA-3：Project/Run namespace、least-privilege、quotas/mount/network/credential owner；接 Runtime readiness 与生命周期，不借此宣称 CLI sandbox。
4. INFRA-4：评估 Linux 外部安装 Incus、macOS 外部安装 Lima 与已有/远端 Linux K3s；评估已有 Docker 上的 k3d 开发/测试路径；Cloud Hypervisor 单独做 Linux host PoC 和逐文件 SPDX 审核；均不得写成已集成。
5. INFRA-5：固定设备/workload 的 RSS/CPU/IO/p95、1–8 Agent 并行公平、崩溃/撤权/取消/断网/磁盘满/恢复验收，SBOM/source/notice 与商业分发审查。

真实环境可创建/复用→绑定 Run→服务就绪→受控任务实际执行→取消/恢复→独立验证回写，且不会影响其他 user resources，才满足闭环。所有步骤目前仍开放；本次只修订设计与实施依赖，不安装宿主功能或改变已有集群。

## §8 审阅与修订历史

| 版本 | 日期 | 审核/修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 用户要求适配 Rust 商业项目的 Multipass 类 k3s 配套；provider 路线、资源/Hook/Run 契约与真实未实现门 |
| v0.2 | 2026-10-02 | 架构师（Mavis 接手 agent per DEC-008）；Ulysses（一人公司 12 角色）— Mavis 接手审核 | 按商业 OSS 限制收敛许可证白名单与 provider 路线；补齐发行/社区证据门、WSL 全局资源授权、未知预算拒绝准入、Host/Guest target 区分及未验证状态；复核官方来源 |