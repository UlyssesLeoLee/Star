# Multipass 宿主层 — k3s + Argo CD 三件套本机集群

> **落地日期**: 2026-10-05
> **拍板**: per `ask_6da0b2511bcee76deccb5b21` — `mp_route_opt1`（1.17.0-rc1 + hcs 驱动）+ `install_auth_opt1`（用户手动装 MSI）
> **选型依据**: [ADR-0054](../../../docs/architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md)（三件套选型）+ 本文 §2（宿主层选型）
> **许可**: Multipass GPL-3.0 / k3s Apache-2.0 / Argo CD Apache-2.0 / Kargo Apache-2.0 / Argo Rollouts Apache-2.0 / cert-manager Apache-2.0

## 1. 为什么换掉 WSL 里的 k3s

WSL 发行版里的 k3s 已不可用（`kubectl` 连 `172.28.176.169:6443` connection refused）。用户诉求是
**解决 k3s 跑不稳**，方向为真 VM 而非容器化集群（k3d 已排除）。

选 Multipass 的直接理由：它是唯一**可商用、免费、开源**且在本机可跑的单机 VM 管理器。

## 2. 宿主层选型：为什么必须 1.17.0-rc1 的 hcs 驱动

本机宿主事实（2026-10-05 实测）：

| 项 | 实测值 | 证据 |
|---|---|---|
| Windows 版本 | `EditionID=CoreCountrySpecific`，build 26200.9550 | `HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion` |
| 完整 Hyper-V 角色 | **未安装**（`vmms` 服务 NOT PRESENT） | `Get-Service vmms` |
| `HypervisorPresent` | `True` | `Get-CimInstance Win32_ComputerSystem` |
| `vmcompute` / `HvHost` | **Running** | `Get-Service vmcompute,HvHost` |
| Hyper-V Administrators 组 | 0 成员 | `net localgroup "Hyper-V Administrators"` |
| 内存 / CPU | 31.8 GB / i7-13700F 16C24T | `Win32_ComputerSystem` / `Win32_Processor` |
| C: 剩余空间 | 35.9 GB | `Get-PSDrive C` |

**这是本方案的决定性约束**：`Windows 11 家庭版` 不含完整 Hyper-V 角色，且这不是可以靠注册表
绕过的版本级排除（微软明确 Home 无 Client Hyper-V）。因此：

| Multipass 版本 | Windows 驱动 | 家庭版可用？ | 本机结论 |
|---|---|---|---|
| **1.16.4**（当前最新稳定） | `hyperv` / `virtualbox` | `hyperv` ❌ 家庭版无此角色 | 只能退 VirtualBox |
| | | `virtualbox` ⚠️ 可用但强制 NEM 慢速模式 | **死路**，见下 |
| **1.17.0-rc1** | **`hcs`**（原生 HCS API） | ✅ Canonical 明确 "works on all editions of Windows, **including Windows Home**" | **采用** |

### 2.1 为什么排除 1.16.4 + VirtualBox

三条**独立**理由，任一成立即足以排除：

1. **性能**：本机 `HypervisorPresent=True`（VBS/WSL2 的 hypervisor 已占住 VT-x），
   VirtualBox 会被强制走 NEM 兼容模式（绿乌龟图标），官方与社区一致记载 10-30%+ 损耗，
   偶有 `WHvSetupPartition failed` / `VERR_NEM_NOT_AVAILABLE`。本机要跑 k3s + 三件套
   （Argo CD / Kargo / Rollouts 均为常驻 controller），NEM 损耗会直接压垮这个负载。
2. **前向性**：Multipass 官方文档已声明 VirtualBox 驱动自 1.17 起 **deprecated**，
   且**不提供迁移路径**（`move-from-virtualbox-to-another-driver` 明确"no migration is planned"）。
   走这条路等于建在死路上，后续必然要重建 VM。
3. **许可边界更紧**：VirtualBox **基础包**是 GPL-3.0（可商用，per Oracle 官方 Licensing FAQ
   自 7.1 起），但 **Extension Pack 是 PUEL**，明确排除任何商业用途，且 Enterprise 授权
   100 席起售。VirtualBox 驱动虽不强制要求 Extension Pack，但引入了一个"基础包/扩展包"
   双许可的额外审查面，徒增 §0 硬约束的合规成本。

> **注**：Multipass 本体是 GPL-3.0（1.16 起 Windows/macOS 部分也完全开源），内部自用
> **零 copyleft 义务**，仅在对外分发时才触发。本方案属内部开发环境，不分发。
> 明确排除项：VirtualBox Extension Pack（PUEL，非商用）、Vagrant（BUSL，source-available）。

### 2.2 1.17.0-rc1 的已知风险

| 风险 | 状态 |
|---|---|
| rc1 状态，仅发布 1 天，下载量 1，未过社区验证 | 🟡 接受。本地开发/测试集群，非生产 |
| Canonical 宣称的 Home 支持未经第三方交叉验证 | 🟡 本机实跑 `multipass launch` 后确认 |
| hcs 驱动对 Windows 11 家庭版中国区的实际行为 | 🟡 实跑验证 |
| 1.17 转正式版时 RC→GA 的升级路径 | 🟢 官方承诺同版本线平滑，升级后重跑 `multipass set local.driver=hcs` 即可 |

## 3. 安装（用户手动，MSI 需管理员）

MSI 已下载并 **SHA256 校验通过**：

| 项 | 值 |
|---|---|
| 路径 | `C:\Users\leo19\Downloads\star-tools\multipass-1.17.0-rc1.2+g0a043d06-noff.win-win64.msi` |
| 大小 | 78,188,079 bytes |
| SHA256 | `80d0dbf94f8219b6b9d4d9cdf4ab2020e240772610c62407b544b23a4cd63d87` |
| 期望值来源 | GitHub Releases API `v1.17.0-rc1` asset `digest` 字段 |
| 校验结果 | **MATCH**（2026-10-05 实测） |

在**管理员** PowerShell 中执行：

```powershell
Start-Process msiexec.exe -Verb RunAs -ArgumentList '/i','"C:\Users\leo19\Downloads\star-tools\multipass-1.17.0-rc1.2+g0a043d06-noff.win-win64.msi"','/qn','/norestart' -Wait
```

装完确认驱动（**必须是 `hcs`**）：

```powershell
multipass version
multipass get local.driver      # 期望 hcs
```

## 4. 一键引导

```powershell
# Stage 0: 只做预检, 不改任何东西
pwsh -File scripts/automation/multipass_k3s_gitops.ps1 -Stage 0

# 全流程: 建 VM + cloud-init + mount 仓库 + 取回 kubeconfig
pwsh -File scripts/automation/multipass_k3s_gitops.ps1 -Stage all
```

**VM 规格与磁盘权衡**（per 守门 #3 显式权衡）：

| 项 | 值 | 依据 |
|---|---|---|
| vCPU | 4 | 本机 24 线程，留足余量给 Windows + Docker Desktop |
| 内存 | 8 GB | k3s ~1.5G + 三件套 controller ~3G + 业务 ~1G，留 3G buffer |
| 磁盘 | **28 GB** | 本机 C: 仅剩 **35.9 GB**。刻意留 7.9 GB 给 Windows 自身，避免把宿主盘撑爆导致系统不稳定 |

## 5. cloud-init 装了什么

`cloud-init-k3s-gitops.yaml` 分 9 阶段，**每阶段落 `DONE:`/`FAIL:` 标记**到
`/var/log/star-cloud-init.log`：

| 阶段 | 内容 |
|---|---|
| 1 | k3s server（`--disable traefik --disable servicelb`） |
| 2 | 等 apiserver `/readyz` |
| 3 | 打 `node-role.kubernetes.io/gitops` 标签 |
| 4 | cert-manager **v1.18.2**（envoy/certificate.yaml 依赖，锁 minor，不用 latest） |
| 5 | Argo Rollouts **v1.10.0** |
| 6 | Argo CD **v3.5.3** |
| 7 | Kargo **v1.12.1** |
| 8 | 应用仓库内 `deploy/gitops/` + `deploy/k3s-local/` |
| 9 | 最终节点/Pod 快照 |

**为何禁 traefik/servicelb**：envoy 按 2026-09-01 13:05 JST 偏好走**独立 deployment**
（`deploy/k3s-local/envoy/`，ClusterIP），不需要 ingress controller；servicelb 会与
`star-api-rest` 的 NodePort 30081 语义混淆。metrics-server 保留（HPA / `kubectl top` 需要）。

**为何要 DONE/FAIL 标记**：cloud-init `runcmd` 全部"尝试过"不等于全部成功。引导脚本
逐段统计 `FAIL:` 标记而非只看退出码 —— 这与 2026-10-04「CI 报绿 ≠ 门禁跑过」是同族教训。

## 6. ⚠️ 未实跑验证声明

per 守门 #11「缺标比错标安全」——本目录**尚未实跑**，以下全部为 🟡：

| 项 | 状态 | 首次落地须做 |
|---|---|---|
| Multipass 1.17.0-rc1 hcs 在 Win11 家庭版中国区的实际行为 | 🟡 未实跑 | `multipass launch` 后核对 |
| hcs 驱动的网络/端口转发能力 | 🟡 未实跑 | 验证 `kubectl get nodes` 从 Windows 侧可达 |
| cloud-init 9 阶段全部通过 | 🟡 未实跑 | 核对 `sudo cat /var/log/star-cloud-init.log` 无 `FAIL:` |
| Kargo CRD 字段（`kargo.akuity.io/v1alpha1`） | 🟡 未实跑 | `kubectl apply --dry-run=server -f deploy/gitops/kargo/` |
| Argo CD AppProject / Application 字段 | 🟡 未实跑 | `argocd app diff` |
| envoy 独立 deployment 能否正常调谐 | 🟡 未实跑 | `kubectl -n star-system get deploy envoy` |
| Kargo promotion step 的 `with` 字段名 | 🟡 未核对官方文档 | 见 `HANDOFF-SEC-LICENSE-001.md` |
| 磁盘 28G 是否够 k3s + 三件套 + 业务镜像 | 🟡 未实跑 | `kubectl get nodes -o jsonpath='{.items[*].status.allocatable}'` |

YAML **语法**已通过校验（2026-10-05，`yaml.safe_load_all` 通过）。

## 7. 故障排查

| 症状 | 处置 |
|---|---|
| `multipass set local.driver=hcs` 报错 | hcs 驱动是 1.17 才有，确认 `multipass version` |
| VM 起来了但 cloud-init 卡住 | `multipass exec star-k3s -- sudo cat /var/log/star-cloud-init.log` |
| `mount` 失败 | Windows 上 mount 默认关，需管理员跑 `multipass set local.privileged-mounts=true` |
| Windows 侧 `kubectl` 连不上 6443 | hcs 驱动端口转发能力待验证（§6 🟡） |

## 8. 与 WSL k3s 的关系

- WSL 内的 k3s **已停用**（不可用），本方案为替代品，**不并存**。
- 两者不共享任何状态：VM 内是独立 k3s + 独立 containerd + 独立数据目录。
- 旧 WSL k3s 的数据如需保留，迁移前先 `multipass exec` 导出；本方案**未迁移任何数据**。
