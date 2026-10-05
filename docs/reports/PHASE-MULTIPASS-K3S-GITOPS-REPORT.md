# PHASE-MULTIPASS-K3S-GITOPS-REPORT — Multipass 宿主层落地 + Argo CD 三件套集成

> **报告版本**：v0.1
> **落地日期**：2026-10-05
> **拍板**：`ask_6da0b2511bcee76deccb5b21` — `mp_route_opt1`（1.17.0-rc1 + hcs 驱动）+ `install_auth_opt1`（用户手动装 MSI）
> **决策人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **关联 ADR**：[ADR-0054 §4.4](../../../architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md) v0.2
> **自动化脚本**：`scripts/automation/multipass_k3s_gitops.ps1` `scripts/automation/cloudinit_secret_scan.py` `scripts/automation/cloudinit_secret_scan_mutation.py`（per 守门 #19 v19，commit message 已含相对路径）

---

## §0 目的

为 ADR-0054 已选定的 **Argo CD v3.5.3 + Kargo v1.12.1 + Argo Rollouts v1.10.0** 三件套确定并落地**宿主承载层**：WSL 内 k3s 已不可用，需以真 VM 替代（k3d 已排除），且全部组件须满足 `AGENTS.md §0 商业开源依赖硬约束`。

本阶段产出三件东西：**宿主层选型决策（含可复现的排除依据）**、**可一键执行的引导资产**、**把"门禁自身可能是坏的"这件事用变异测试钉死**。

---

## §1 改动矩阵

| # | 改动 | 文件 | 状态 |
|---|---|---|---|
| 1 | cloud-init 引导（k3s + cert-manager + 三件套，9 阶段） | `deploy/multipass/cloud-init-k3s-gitops.yaml` | 新增 6.7 KB |
| 2 | 宿主层说明 + 未验证清单 | `deploy/multipass/README.md` | 新增 9.0 KB |
| 3 | 引导自动化脚本（4 阶段 + 逐段标记核对） | `scripts/automation/multipass_k3s_gitops.ps1` | 新增 12.9 KB |
| 4 | cloud-init 密钥扫描门禁 | `scripts/automation/cloudinit_secret_scan.py` | 新增 3.9 KB |
| 5 | 上述门禁的变异测试（6 用例） | `scripts/automation/cloudinit_secret_scan_mutation.py` | 新增 4.1 KB |
| 6 | ADR 宿主层章节（§4.4.1–4.4.5） | ADR-0054 | +58 行，172→231 |
| 7 | ADR 头部状态 + 修订历史升 v0.2 | ADR-0054 | 2 行 |

**刻意未做**：未改 `deploy/k3s-local/**` 任何 manifest（kustomization 与 envoy 保持原样，守门 #1 禁回溯叙事 0 改）；未动 `deploy/gitops/**`；未改 `deny.toml`（本阶段无新增 Rust 依赖）；未动任何 CI workflow。

---

## §2 验证摘要（全部实测）

### 2.1 安装介质完整性

```
文件  : multipass-1.17.0-rc1.2+g0a043d06-noff.win-win64.msi
大小  : 78,188,079 bytes
实测  : 80d0dbf94f8219b6b9d4d9cdf4ab2020e240772610c62407b544b23a4cd63d87
期望  : 80d0dbf94f8219b6b9d4d9cdf4ab2020e240772610c62407b544b23a4cd63d87
期望来源: GitHub Releases API v1.17.0-rc1 asset digest 字段
结果  : MATCH
```

### 2.2 宿主事实（决定选型的实测）

```
Windows EditionID = CoreCountrySpecific   build 26200.9550   ← 家庭版
Get-Service vmms  → NOT PRESENT                              ← 无完整 Hyper-V 角色
HypervisorPresent = True                                      ← VBS/WSL2 hypervisor 已占 VT-x
vmcompute / HvHost = Running                                  ← HCS 可用（hcs 驱动前提）
net localgroup "Hyper-V Administrators" → 0 成员
内存 31.8 GB / i7-13700F 16C24T / C: 剩余 35.9 GB
Docker 29.8.1 (desktop-linux) 正常运行
```

### 2.3 门禁

| 检查 | 命令 | 结果 |
|---|---|---|
| cloud-init YAML 语法 | `yaml.safe_load_all` | **1 doc / 3 write_files / 18 runcmd，解析通过** |
| pwsh 脚本语法 | `[Parser]::ParseFile` AST | **0 errors**（修复后复跑，见 §2.5） |
| 密钥扫描（真实文件） | `cloudinit_secret_scan.py` | **PASS，0 命中** |
| 变异测试（6 用例） | `cloudinit_secret_scan_mutation.py` | **PASS=6 FAIL=0** |

变异测试逐例结果（均断言**诊断文本出现**，不只看退出码）：

```
PASS  control_clean                          exit=0 diag 'PASS'
PASS  mut_pem_key_value                      exit=1 diag 'PEM'
PASS  mut_key_hidden_in_comment              exit=1 diag 'PEM'
PASS  mut_github_token                       exit=1 diag 'github token'
PASS  mut_jwt_private_value                  exit=1 diag 'JWT'
PASS  guard_variable_name_in_comment_allowed  exit=0 diag 'PASS'
RESULT: PASS=6 FAIL=0 TOTAL=6
```

### 2.4 许可

| 组件 | 许可 | 结论 |
|---|---|---|
| Multipass 1.17.0-rc1 | GPL-3.0（1.16 起 Windows 部分亦完全开源） | ✅ 内部自用零 copyleft 义务，不分发 |
| k3s | Apache-2.0 | ✅ |
| Argo CD / Kargo / Argo Rollouts | Apache-2.0（ADR-0054 §2.1 实测） | ✅ |
| cert-manager v1.18.2 | Apache-2.0 | ✅ |
| **VirtualBox Extension Pack** | **PUEL（非商用）** | ❌ **排除，不安装** |
| **Vagrant** | **BUSL（source-available）** | ❌ 排除（沿用 ADR-0054 §4 结论） |

### 2.5 实跑抓出并修复的两个真实缺陷

**P1 — pwsh 脚本实跑必炸（AST 校验抓出）**

`Write-Stage "Stage 2: mount repo -> $InstanceName:/home/ubuntu/star"` 中，PowerShell 把 `$InstanceName:` 解析为**变量名 `InstanceName:`**（含冒号），报"变量引用无效"。修复为 `${InstanceName}:`。
**教训**：只做 YAML 校验就 commit 的话，这个脚本在 Stage 2 必然抛错。语法校验本身是独立门禁，不是可选项。

**P1 — 密钥扫描器存在绕过漏洞（变异测试抓出）**

- 第一版：全量扫描 → 误报文件头"本文件不含 secret"的**注释**里出现的 `JWT_PRIVATE_KEY_PEM` 字样。
- 第二版（修误报）：**跳过所有注释行** → 变异测试注入 `# -----BEGIN RSA PRIVATE KEY-----` 到注释行，扫描器返回 **exit 0 放行**。即"注释"成了绕过密钥检测的藏身处，**比原始误报更严重**。
- 第三版（当前）：按模式分类——含实际密钥材料者（PEM 块 / AKIA / ghp_ / 长 base64）**注释行照扫**，仅"提及变量名"类（JWT 值赋值 / 私钥路径）跳过注释。变异测试 `mut_key_hidden_in_comment` 由 FAIL 转 PASS。

**附带抓到**：变异测试自身在 Windows 中文控制台崩 `UnicodeEncodeError`（GBK 编不了 UTF-8 中文）。门禁自己崩 = 门禁形同虚设、退出码不可信。已在两个脚本内 `sys.stdout.reconfigure(encoding="utf-8")` 修复。

---

## §3 已知缺口（per 缺标比错标）

**全部为 🟡 未实跑**。本阶段只落了资产与选型依据，**未创建任何 VM、未安装 Multipass、未跑过任何 k8s 命令**。

| # | 缺口 | 首次落地须做 |
|---|---|---|
| 1 | Multipass 1.17.0-rc1 hcs 在 Win11 家庭版**中国区**的实际行为 | `multipass launch` 后核对 `multipass info` |
| 2 | hcs 驱动的网络/端口转发能力（Windows 侧 `kubectl` 能否直连 6443） | `kubectl --kubeconfig ... get nodes` |
| 3 | cloud-init 9 阶段是否全部通过 | `sudo cat /var/log/star-cloud-init.log` 确认无 `FAIL:` |
| 4 | 磁盘 28 GB 是否够 k3s + 三件套 + 业务镜像（本机仅余 35.9 GB） | `kubectl get nodes -o jsonpath='{.items[*].status.allocatable}'` |
| 5 | Kargo CRD 字段（`kargo.akuity.io/v1alpha1`）server dry-run | `kubectl apply --dry-run=server -f deploy/gitops/kargo/` |
| 6 | Argo CD AppProject / Application 字段 | `argocd app diff` |
| 7 | envoy 独立 deployment 能否正常调谐 | `kubectl -n star-system get deploy envoy` |
| 8 | rc1 → GA 升级路径 | 1.17 正式版发布后重跑驱动设置 |
| 9 | 密钥扫描门禁**尚未接入 CI**（本阶段只落脚本） | 挂到 `.github/workflows/license-gate.yml` 或新 job |
| 10 | 沿用 `HANDOFF-SEC-LICENSE-001.md`：Kargo promotion step 的 `with` 字段名未核对官方文档 | 查 Kargo v1.12.1 promotion step 官方文档 |

**未做的数据迁移**：旧 WSL k3s 数据**未迁移**到 VM，两者不共享任何状态。

---

## §4 子代理失败接手清单

本阶段**未派任何子代理**（守门 #20 相关 RPC 不可靠的规避：任务量小且路径明确，Mavis 自驱完成）。

**工作区 4 项 untracked 非本 session 产出，已按守门 #9 排除、未 commit**：
`docs/arch-live/`、`scripts/automation/arch_live_doc_check.py`、`arch_live_interact_check.py`、`arch_live_shot.py`。

---

## §5 守门规则遵守情况

| 守门 | 要求 | 本阶段执行 |
|---|---|---|
| #1 v2/v19 | `cargo check --workspace --all-targets -j 4` 0 err | **不适用** — 本阶段无 Rust 代码改动 |
| #1 v15 | docs 同步须有新事件触发 | ✅ 触发源 = 宿主层选型拍板 `ask_6da0b...` |
| #1 禁回溯叙事 | 禁"原本是/历史形态" | ✅ 全文无回溯叙事；排除依据均附实测命令 |
| #5 v2 | 密钥禁入 log / 入仓 | ✅ cloud-init 零密钥，并落**机器门禁**（非仅靠自觉） |
| #6 | PowerShell only | ✅ 全部命令 pwsh，无 bash 混用 |
| #7 | 0 unsafe | ✅ 无 Rust 代码 |
| #9 | 不 commit 非本 session 产出 | ✅ 4 项 arch-live 产物已排除 |
| #10 / #14 v4 | 代签 + Mavis 审核 | ✅ author=Ulysses，状态行标 Mavis 接手终审 |
| #11 | 缺标比错标 | ✅ §3 全部 10 项标 🟡，无一项以 ✅ 冒充 |
| #19 v19 | 自动化脚本落 `scripts/automation/` | ✅ 3 个脚本均落该目录，commit message 引用相对路径 |
| #0 商业开源依赖 | 可商用 + 逐版本审闭包 | ✅ 见 §2.4；显式排除 PUEL / BUSL |
| 新增 | 门禁必须有变异测试 + 对照组 | ✅ 6 用例；**并因此抓到门禁自身的绕过漏洞** |

---

## §6 签字栏

| 角色 | 签字 | 说明 |
|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 宿主层选型依据完整，rc1 风险已显式登记 |
| SRE Lead | — | 🟡 真人未到位（5 域 Lead per 守门 #14 v2） |
| 平台 | 平台 (Mavis 接手 agent per DEC-008) | 🟡 脚本未在本机实跑（待用户装 MSI） |
| 评审主持 | 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 排除依据三条独立、可复现 |
| PM | — | 🟡 真人未到位 |

> 🟡 项 = 未验证或待真人到位（守门 #11 缺标比错标）。**本报告不宣称宿主层已验证可用**，只宣称选型依据成立、资产已就绪、门禁已自证。

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：宿主层选型（Multipass 1.17.0-rc1 + hcs）+ 排除 1.16.4/VirtualBox 三条独立依据 + cloud-init/脚本/密钥门禁落地 + 两个 P1 缺陷修复记录 | 2026-10-05 `ask_6da0b2511bcee76deccb5b21` mp_route_opt1 + install_auth_opt1 |
