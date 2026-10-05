# ADR-0054: 自动部署体系选型 — Argo CD + Kargo + Argo Rollouts

> **状态**：✅ Accepted v0.3（per 2026-10-05 19:58 JST 实跑验证 + 8 处缺陷修正，守门 #10 + #14 v4 Mavis 审核）
> **日期**：2026-10-04
> **决策人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签字**：✅ Mavis 接手终审 (per 2026-08-27 19:39 + 21:59 JST 用户三次授权"允许你代签" + 2026-09-11 23:11 JST"真人的内容由 agent 决定")
> **拍板范围**：选项 1「落 ADR + 选型报告 + cargo 门禁骨架（推荐）」+ 选项 1「本机 k3s 单集群，dev → staging 两级（推荐）」
> **上游文档**：[AGENTS.md §0 商业开源依赖硬约束](../../../AGENTS.md) [AGENTS.md §4 #1 累积规](../../../AGENTS.md)
> **关联 ADR**：[ADR-0026 STAR AI 兼容](../../../docs/adr/0026-multica-patterns-borrow.md) [ADR-0027 Rust 转向 Agent Game](../../../docs/adr/0027-rust-pivot-agent-game.md)
> **证据**：[PHASE-CD-SELECTION-REPORT.md v0.1](../../../reports/PHASE-CD-SELECTION-REPORT.md) ｜ 根目录 `deny.toml`

---

## 1. 背景

Ulysses 于 2026-10-04 20:50 JST 提出需求：**需要一套 Netflix Spinnaker 等效的开源、免费、可商用的自动部署体系，纳入 Star 仓；并要求给出针对 Rust 项目的选型结论。**

该需求受两条既有硬约束直接支配：

1. **AGENTS.md §0 商业开源依赖硬约束** — 组件必须允许不限用途/行业/席位/用量的商业使用；**不得把 copyleft 等同于禁止商用，也不得仅因其为 GPL/AGPL/LGPL 而一概排除**；排除非商业、field-of-use、source-available 或要求付费订阅/席位才能解锁核心功能的组件；**逐版本审查完整构建/分发闭包中的传递依赖、二进制、安装器、容器与 guest image**；只核对上游仓库主许可证不足以放行。
2. **AGENTS.md §0 商业开源依赖硬约束之社区活跃度条款** — 须以近 12 个月的发布/维护证据和公开维护渠道核验。

### 1.1 Spinnaker 的能力分解

"Spinnaker 等效"不是一个单体能力，而是三件独立的事。厘清这一点是本次选型的关键：

| Spinnaker 能力 | 承载组件 | 开源界现状 |
|---|---|---|
| 多云资源抽象 | Clouddriver | Argo CD / Flux（K8s 范围） |
| 流水线编排 | Orca | Argo Workflows / Kargo |
| **环境晋级（dev→stage→prod）+ 审批 + 审计** | Orca + Fiat | **长期空白**，Kargo 补上 |
| **渐进式发布（canary/蓝绿）+ 自动指标分析** | Kayenta | Argo Rollouts + AnalysisTemplate |
| 事件路由 | Echo | Argo Events |

其中"环境晋级"与"渐进式发布"是 Spinnaker 最不可替代的两项能力，也是选型的真正分水岭。

## 2. 候选实测证据

所有数据于 **2026-10-04 20:50–21:04 JST 实测**，数据源为 `gh api`（GitHub REST）+ 上游仓库原始文件直读，未采用二手博客结论。

### 2.1 许可与维护状态（实测）

| 项目 | SPDX（实测） | 版本 | 最后推送 | stars | 结论 |
|---|---|---|---|---|---|
| `argoproj/argo-cd` | **Apache-2.0** | v3.5.3（v3.6.0-rc1 为 RC） | 2026-10-04T08:45Z | 24,322 | ✅ 采纳 |
| `akuity/kargo` | **Apache-2.0** | v1.12.1（2026-10-02） | 2026-10-04T05:28Z | 3,695 | ✅ 采纳 |
| `argoproj/argo-rollouts` | **Apache-2.0** | v1.10.0（2026-08-27） | 2026-10-02T21:03Z | 3,592 | ✅ 采纳 |
| `sigstore/cosign` | Apache-2.0 | — | 2026-10-02 | 6,345 | ✅ 采纳（签名） |
| `external-secrets/external-secrets` | Apache-2.0 | — | 2026-10-02 | 6,892 | ✅ 采纳（密钥） |
| `tektoncd/pipeline` | Apache-2.0 | — | 2026-10-02 | 9,074 | ⏸ 备用（CI 侧，暂不引入） |
| `argoproj-labs/argocd-image-updater` | Apache-2.0 | chart 1.3.1 | 2026-10-02 | 1,719 | ⚠️ 条件采纳（见 §5.1） |
| `fluxcd/flux2` | Apache-2.0 | v2.9.6（2026-10-01） | 2026-10-03 | 8,436 | ❌ 见 §4.2 |
| `controlplaneio-fluxcd/flux-operator` | **AGPL-3.0** | — | 2026-10-02 | 769 | ❌ 排除（见 §4.2） |
| `fluxcd/flagger` | Apache-2.0 | v1.45.0（2026-09-01） | 2026-09-21 | 5,417 | ❌ 未选（Argo Rollouts 覆盖同类能力） |
| `spinnaker/spinnaker` | Apache-2.0 | 2026.1.0 | 2026-09-25 | — | ❌ 排除（见 §4.1） |

### 2.2 Argo CD 许可闭包实测（回应 AGENTS.md "只核对主许可证不足以放行"）

`argoproj/argo-cd` v3 的主 `LICENSE`（Copyright 2017-2018 The Argo Authors）为纯 Apache-2.0，无附加条款。在此基础上进一步实测其 `go.mod` 全量闭包（约 250 个直接 + 间接模块）：

- **未发现任何 AGPL / SSPL / BUSL / Elastic / Facebook 模块**；
- 闭包内 `github.com/argoproj/notifications-engine`（历史上许可争议点位）实测 `LICENSE` 为 Apache-2.0（Copyright 2017-2021 The Argo Authors）；
- `argoproj/gitops-engine` 独立仓库已 `archived=true`（2026-05-12），其代码已并入 argo-cd monorepo（`go.mod` 内 `replace` 指向 `./gitops-engine`），不构成外部闭包。

> ⚠️ **本 ADR 的闭包核验范围**：覆盖主仓库 `go.mod` 声明的模块集合与两个关键一手许可文件。**容器镜像内 `apk`/`apt` 系统包层未纳入本次核验**，落地时须按 §6 待办补 `syft`/`trivy` 生成的 SPDX SBOM 复验。

### 2.3 Rust 侧依赖闭包核验（2026-10-05 补）

本仓自身 107 个 member 的第三方依赖闭包（1265 crate）由 `deny.toml` + `cargo deny list` 核验，明细见 [NOTICE.md](../../../NOTICE.md) 与 [PHASE-CD-SELECTION-REPORT §2.3](../../../reports/PHASE-CD-SELECTION-REPORT.md)。

核验结论：**无任何** 非商业（NC）、field-of-use 或 source-available 许可；存在 **MPL-2.0 × 5**（`dom_query` / `dirs-sys` 传递引入）与 **LGPL-2.1-or-later × 2**（`r-efi`，仅 UEFI target 编译），二者均**显式列入** `deny.toml` 白名单 —— per AGENTS.md §0「不得把 copyleft 等同于禁止商用，也不得仅因其为 GPL/AGPL/LGPL 而一概排除」。

> **方法学要点**：核验时发现 `cargo deny check` 的 `license-not-encountered` 是**否定式**警告（白名单条目未被用到），**不能反推"该许可不存在"**。必须用 `cargo deny list` 这类枚举式输出做交叉验证。同理，Kargo 官方 SPDX SBOM 的 395 个包 `licenseConcluded` 全为 `NOASSERTION`，说明**上游发布了 SBOM 也不等于许可核验通过**。

## 3. 决策

**采纳 Argo CD 三件套作为 Star 仓自动部署体系**：

| 层 | 组件 | 职责 | 许可 |
|---|---|---|---|
| 1. 构建 | 现有 GitHub Actions（11 个 workflow） | 测试 / 构建 / 推送镜像 / 签名 | — |
| 2. 晋级 | **Kargo** | Warehouse 监听 GHCR → Freight → Stage 晋级 + 审批 + 审计 | Apache-2.0 |
| 3. 调谐 | **Argo CD** | 读 `deploy/` 目录 → reconcile 到 k3s | Apache-2.0 |
| 4. 渐进式 | **Argo Rollouts** | canary / 蓝绿 + Analysis 门禁 + 自动回滚 | Apache-2.0 |
| 5. 签名 | Sigstore cosign | 镜像 + Tauri 产物流水线签名 | Apache-2.0 |
| 6. 密钥 | External Secrets | 从外部密钥源同步 Secret | Apache-2.0 |

**第一阶段落地范围（本机 k3s 单集群，dev → staging 两级）**：per 2026-10-04 21:04 JST 拍板，Kargo 晋级链路最短，可最快验证 Freight/Stage/Promotion 模型跑通；prod 门禁待模型稳定后再加。

**CI 侧维持现有 GitHub Actions 不变**，本阶段不引入 Tekton（避免同时改动 CI 侧带来额外变量）。

## 4. 排除理由

### 4.1 排除 Spinnaker — 安全与架构实证

2026-04-20 Spinnaker 同日爆出两个 CVSS 9.9 严重漏洞：

- `CVE-2026-32613` — Echo 服务 SpEL 上下文无限制 RCE
- `CVE-2026-32604` — Clouddriver gitrepo 命令注入

四条支持线（2025.3.x / 2025.4.x / 2026.0.x / 2026.1.x）**同时中招，版本降级不构成可行缓解**；补丁（2026.1.0 / 2026.0.1 / 2025.4.2 / 2025.3.2）与公开 PoC **同日发布，暴露窗口等于补丁窗口**。

根因是架构性的、而非单点缺陷：Orca 侧 SpEL 解析器早已接入可信类白名单（2019 年一系列 SpEL 注入事件后的修复），但 Echo 的 expected-artifacts 求值路径**遗漏了同一份白名单**，导致 SpEL 拥有完整 JVM 类访问能力（含 `java.lang.Runtime`），且该缺口**潜伏约五年未被发现**。

这一形态直接抵触本仓两项硬约束：

1. **AGENTS.md §0 原生 Hook 强约束** — 安全关键 Hook 必须由 Rust 核心原生执行、**fail-closed**。而 Spinnaker 的失败模式恰恰是"同一安全边界在不同微服务各自实现，其中一处漏掉"——即 **fail-open**。
2. **AGENTS.md §0 多代理并行硬约束** — 要求安全边界单一收敛。Spinnaker 14 个微服务各自的信任边界是分散的。

补充：Halyard 已于 2026-07-29 被彻底移除（`chore(halyard): REMOVE HALYARD FINALLY`），生态处于收缩通道；14 个微服务的资源占用相对 Argo CD 5 个组件亦无优势。

### 4.2 排除 Flux — 许可分叉实证

`fluxcd/flux2` 核心 controllers 仍为 Apache-2.0，但 Flux 官方的 operator 化 / 多集群控制面路线**已迁出原组织**：

- 迁移至 `controlplaneio-fluxcd/flux-operator`，实测 SPDX 为 **AGPL-3.0**；
- 实测该组织**全部 17 个仓库**无一例外为 AGPL-3.0 或 GPL-3.0（`schema-catalog` / `charts` / `distribution` / `d1-apps` / `d2-infra` / `terraform-kubernetes-flux-operator-bootstrap` 等）。

按 AGENTS.md §0，此 AGPL **不构成禁止商用**（真开源、可商用），因此**不以许可本身作为一票否决**；但它带来两项持续成本：

1. **源披露义务的灰色地带** — 控制面若对外提供网络服务且经修改，须提供对应源码，边界判断依赖事实而非许可文本本身；
2. **版本审查成本** — 每次 operator 升级都需重新核验整个 `controlplaneio-fluxcd` 组织的许可构成（本次核验即发现 17 个仓库需逐个确认），与 AGENTS.md §0"逐版本审查完整构建/分发闭包"的要求叠加后负担显著。

叠加否决项：Flux 无内置 UI（官方 Web UI 由上述 AGPL-3.0 的 flux-operator 提供），而本仓多 Agent 协作与 Worktree/Run 可视化需要 UI 层；且 Flux 阵营无对应 Kargo 的晋级层，该能力需自行实现。

> **本条排除不否认 Flux 的技术质量**（CNCF 毕业项目，CLI-first 组合性强）。排除理由是"许可闭包形态 + UI 依赖 AGPL + 晋级层缺失"三者叠加后的运维与审查总成本，而非质量。

### 4.3 未选 Flagger（记录澄清）

Flagger 实测**仍处活跃维护**（v1.45.0，2026-09-01 发布，2026-09-21 有推送，Apache-2.0）。Weaveworks 于 2024 年停止运营后，Flagger 由 fluxcd 组织接管并延续至今。

**未选原因不是维护状态，而是功能重叠**：Argo Rollouts 已覆盖 Flagger 的 canary / 蓝绿 / A-B 能力且与 Argo CD 同一 UI 与同一许可族；同时引入两者会形成两套渐进式发布控制器并存的运维面。**本条写入是为纠正"Flagger 已弃用"的错误认知**——它没有弃用，只是在本选型中不构成增量。

## 4.4 宿主层选型：Multipass 1.17.0-rc1 + hcs 驱动（2026-10-05 追加）

> 本节为 §2 三件套选型的**宿主承载层**决策。三件套装在哪台机器上跑，与三件套本身选谁是正交问题，但同样受 §1 的两条硬约束支配。

WSL 发行版内的 k3s 已不可用（`kubectl` 连 `172.28.176.169:6443` connection refused），用户诉求是**解决 k3s 跑不稳**，方向为真 VM 而非容器化集群（k3d 已排除）。

### 4.4.1 宿主事实（2026-10-05 实测）

| 项 | 实测值 | 取证命令 |
|---|---|---|
| Windows 版本 | `EditionID=CoreCountrySpecific`，build 26200.9550 | `HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion` |
| 完整 Hyper-V 角色 | **未安装**（`vmms` 服务 NOT PRESENT） | `Get-Service vmms` |
| `HypervisorPresent` | `True` | `Get-CimInstance Win32_ComputerSystem` |
| `vmcompute` / `HvHost` | **Running** | `Get-Service vmcompute,HvHost` |
| Hyper-V Administrators 组 | 0 成员 | `net localgroup "Hyper-V Administrators"` |
| 内存 / CPU | 31.8 GB / i7-13700F 16C24T | `Win32_ComputerSystem` / `Win32_Processor` |
| C: 剩余空间 | **35.9 GB** | `Get-PSDrive C` |

### 4.4.2 决策：必须用 1.17 的 hcs 驱动

本机是 **Windows 11 家庭版**，不含完整 Hyper-V 角色，且该排除是版本级的，注册表无法绕过。这直接决定 Multipass 的可用驱动：

| Multipass 版本 | Windows 驱动 | 家庭版可用 | 结论 |
|---|---|---|---|
| 1.16.4（当时最新稳定） | `hyperv` | ❌ 无此角色 | 不可用 |
| 1.16.4 | `virtualbox` | ⚠️ 可用但强制 NEM 慢速模式 | **排除**，见 4.4.3 |
| **1.17.0-rc1** | **`hcs`**（原生 HCS API） | ✅ Canonical release notes 明示 "works on all editions of Windows, **including Windows Home**" | **采纳** |

Multipass 许可为 **GPL-3.0**（1.16 起 Windows/macOS 部分亦完全开源），内部自用**零 copyleft 义务**，仅对外分发才触发；本方案属内部开发环境，不分发，符合 §1 硬约束。

### 4.4.3 为何排除 1.16.4 + VirtualBox

三条**独立**理由，任一成立即足以排除：

1. **性能**：本机 `HypervisorPresent=True`（VBS/WSL2 的 hypervisor 已占住 VT-x），VirtualBox 被强制走 NEM 兼容模式（绿乌龟图标），官方与社区一致记载 10-30%+ 损耗，偶有 `WHvSetupPartition failed` / `VERR_NEM_NOT_AVAILABLE`。本机要承载 k3s + 三件套三个常驻 controller，NEM 损耗会直接压垮该负载。
2. **前向性**：官方文档已声明 VirtualBox 驱动自 1.17 起 **deprecated** 且**不提供迁移路径**（`move-from-virtualbox-to-another-driver` 明示 "no migration is planned"）。走此路等于建在死路上。
3. **许可面更宽**：VirtualBox **基础包**为 GPL-3.0（可商用，per Oracle 官方 Licensing FAQ 自 7.1 起），但 **Extension Pack 是 PUEL**，明文排除任何商业用途，Enterprise 授权 100 席起售。虽不强制要求 Extension Pack，但引入"基础包/扩展包"双许可会徒增 §1 硬约束的合规审查面。

> 同族排除结论：VirtualBox **Extension Pack**（PUEL，非商用）、Vagrant（BUSL，source-available）均已在 §2 结论外排除；本节不重复论证。

### 4.4.4 rc1 的已知风险（显式登记，非隐藏）

| 风险 | 状态 |
|---|---|
| rc1 状态，仅发布 1 天，下载量 1，未过社区验证 | 🟡 接受 — 本地开发/测试集群，非生产 |
| Canonical 宣称的 Home 支持未经第三方交叉验证 | 🟡 实跑 `multipass launch` 后确认 |
| hcs 驱动对中国区 Windows 11 家庭版的实际行为 | 🟡 实跑验证 |
| 磁盘 28 GB 是否够 k3s + 三件套 + 业务镜像 | 🟡 实跑验证（本机仅余 35.9 GB，刻意留 7.9 GB 给宿主） |
| rc1 → GA 升级路径 | 🟢 同版本线平滑，升级后重跑 `multipass set local.driver=hcs` 即可 |

### 4.4.5 落地物与实跑结论（2026-10-05 实跑后更新）

- 安装介质 SHA256 **实测校验通过**：`80d0dbf94f8219b6b9d4d9cdf4ab2020e240772610c62407b544b23a4cd63d87`（78,188,079 bytes，期望值取自 GitHub Releases API `v1.17.0-rc1` asset `digest` 字段）
- cloud-init 引导：`deploy/multipass/cloud-init-k3s-gitops.yaml`（9 阶段，逐段落 `DONE:`/`FAIL:` 标记）+ 修复引导 `cloud-init-fix-gitops.yaml`
- 自动化脚本：`scripts/automation/multipass_k3s_gitops.ps1`（预检/建 VM/mount/取 kubeconfig）、`deploy/multipass/fix-rollouts-ns.sh`、`deploy/multipass/install-kargo.sh`
- 密钥门禁：`scripts/automation/cloudinit_secret_scan.py` + `cloudinit_secret_scan_mutation.py`（6 用例，含对照组与反例守卫）
- 说明文档：`deploy/multipass/README.md`

**✅ 实跑结论（2026-10-05 19:32–19:58 JST）**：

| 项 | 实测 |
|---|---|
| `local.driver` | **`hcs`** — 4.4.2 的推断成立，Canonical 宣称的 Home 支持在 Windows 11 家庭版中国区**确实可用** |
| VM | `star-k3s` Running `10.97.0.116`（Ubuntu 24.04.5 LTS） |
| 集群 | k3s `v1.36.5+k3s1`，节点 Ready，allocatable 26.6 GB |
| 三件套 | cert-manager 3/3、Argo CD 7/7、Argo Rollouts 1/1、Kargo 5/5，**合计 19/19 Running，0 重启**，29 CRD |
| GitOps 资源 | `kubectl apply -f deploy/gitops/{kargo,argocd}/` 均 **exit 0** |

**实跑修正了 4.4 选型阶段无法验证的 3 处假设**（详见 `PHASE-MULTIPASS-K3S-GITOPS-REPORT.md` §2.5）：

1. **Kargo 无 `install.yaml`**（选型阶段写的 URL 实测 404），官方为 OCI Helm chart `oci://ghcr.io/akuity/kargo-charts/kargo`，且 `api.adminAccount.passwordHash` / `tokenSigningKey` 无默认值必须显式提供。
2. **Argo Rollouts 官方 `install.yaml` 不含 namespaced RBAC**，装到非 `argo-rollouts` 的 namespace 会直接 `CrashLoopBackOff`。
3. **Argo CD 官方 `install.yaml` 不带 `namespace` 字段**，资源落在 `default`，而非社区文档常见的 `argocd`。

**🟡 残留缺口**：`Kargo promotionTemplate` 各 step 的 `config` **内部**字段名未经官方文档核对 —— CRD schema 与 admission webhook **都只校验到 steps 数组层**，校验不到 step 内部键名。详见报告 §3.2 缺口 #1。

## 5. Rust 项目特有的两个约束

### 5.1 Tauri 桌面产物不在 k8s 内

本仓 `frontend/`（Tauri 2.0 PoC，见 commit `3ad73ad7`）+ `crates/relationship-engine-wasm` / `query-engine-wasm` / `layout-engine-wasm` 产出的桌面与 wasm 制品**不由 k3s 承载**，Kargo 的 Stage/Freight 晋级模型对其不适用。

因此确立**两条独立发布链，不得混用**：

- **服务链**：Docker 镜像 → Kargo 晋级 → Argo CD → k3s（签名用 cosign）
- **桌面链**：Tauri bundle → `tauri signer`（minisign）→ updater channel（**不经过 k3s**）

### 5.2 商业开源依赖硬约束此前无自动化 — 本次闭合

2026-10-04 实测：仓库根目录**不存在** `deny.toml`，`.github/workflows/` 下 11 个 workflow 中**零个**包含 `cargo-deny` / `cargo-audit` / `cargo-about` / `cargo-cyclonedx` / SBOM / license 任一检查。即 AGENTS.md §0"逐版本审查完整构建/分发闭包"此前**完全依赖人工**。

本次落地 `deny.toml`（详见 §2.2 闭包核验方法），并将 §6 待办中的 SPDX SBOM 生成接入 CI。

## 6. 落地清单与待办

### 6.1 本次已落地

- 根目录 `deny.toml` — 许可 / bans / advisories / sources 四类门禁
- `.github/workflows/license-gate.yml` — cargo-deny + cargo-audit 门禁
- `deploy/gitops/` — Argo CD Application + Kargo Project/Warehouse/Stage 骨架（dev → staging）

### 6.2 待办（须在 prod 晋级前完成）

| # | 待办 | 阻断条件 |
|---|---|---|
| 1 | 用 `syft` 或 `cargo-cyclonedx` 生成容器镜像 SPDX SBOM，复验 §2.2 未覆盖的**系统包层** | prod 环境晋级前 |
| 2 | 复验 `argocd-image-updater` 镜像实际漏洞构成 | 引入该组件前 |
| 3 | `cargo-about` 生成 NOTICE 文本（本机未安装该工具，见报告 §3 缺口 #2） | 首次对外分发前 |
| 4 | staging → prod 三级链路 + Analysis 门禁 | 模型稳定后 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Argo CD 三件套选型 + Spinnaker/Flux 排除实证 + deny.toml 闭合许可门禁缺口 | 2026-10-04 20:50 JST 用户需求 + 21:04 JST `ask_72c00a6f28daadd5538fa004` 拍板 |
| v0.2 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 新增 §4.4 宿主层选型：Multipass 1.17.0-rc1 + hcs 驱动（本机 Win11 家庭版无完整 Hyper-V 角色，vmms 未装）；落 cloud-init 9 阶段引导 + pwsh 自动化脚本 + cloud-init 密钥扫描门禁（6 用例变异测试）。排除 1.16.4+VirtualBox（NEM 慢速 / deprecated 无迁移 / PUEL 双许可面） | 2026-10-05 ask_6da0b2511bcee76deccb5b21 mp_route_opt1 + install_auth_opt1 |
| v0.3 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | §4.4.5 补实跑结论：`local.driver=hcs` 在 Win11 家庭版中国区**已验证可用**；三件套 19/19 Pod Running；GitOps 资源 `kubectl apply` 双 exit 0。同时记录实跑修正的 3 处上游假设（Kargo 无 install.yaml / Rollouts install.yaml 无 namespaced RBAC / Argo CD install.yaml 无 namespace 字段） | 实跑 2026-10-05 19:32–19:58 JST |
