# PHASE-MULTIPASS-K3S-GITOPS-REPORT — Multipass 宿主层落地 + Argo CD 三件套实跑

> **报告版本**：v0.2（**实跑后升版** — 宿主装好、VM 建好、三件套 19/19 Pod 跑通）
> **实跑时间窗**：2026-10-05 19:32–19:58 JST
> **拍板**：`ask_6da0b2511bcee76deccb5b21` — `mp_route_opt1`（1.17.0-rc1 + hcs 驱动）
> **决策人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **关联 ADR**：[ADR-0054 §4.4](../../../architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md) v0.2
> **自动化脚本**：`scripts/automation/multipass_k3s_gitops.ps1`、`deploy/multipass/fix-rollouts-ns.sh`、`deploy/multipass/install-kargo.sh`（per 守门 #19 v19，commit message 已含相对路径）

---

## §0 目的

为 ADR-0054 已选定的 **Argo CD v3.5.3 + Kargo v1.12.1 + Argo Rollouts v1.10.0** 三件套确定并落地**宿主承载层**：WSL 内 k3s 已不可用，需以真 VM 替代（k3d 已排除），且全部组件须满足 `AGENTS.md §0 商业开源依赖硬约束`。

本阶段实际完成：宿主层选型决策 → 装机 → 建 VM → 装三件套 → 校验仓库内 GitOps 声明式资源 → **修正实跑抓出的 8 处真实缺陷**。

---

## §1 改动矩阵

### 1.1 基础设施（未入库，仅本机）

| # | 项 | 状态 |
|---|---|---|
| 1 | Multipass 1.17.0-rc1 MSI（78,188,079 B，SHA256 复验 MATCH） | 已装，服务 Running |
| 2 | VM `star-k3s`（4C / 8G / 28G） | Running，`10.97.0.116` |
| 3 | k3s `v1.36.5+k3s1`（`--disable traefik --disable servicelb`） | 节点 Ready |
| 4 | cert-manager v1.18.2 | 3/3 Running |
| 5 | Argo CD v3.5.3（`default` ns） | 7/7 Running |
| 6 | Argo Rollouts v1.10.0（`argo-rollouts` ns） | 1/1 Running |
| 7 | Kargo v1.12.1（Helm chart `deployed`） | 5/5 Running |

### 1.2 入库改动

| # | 文件 | 变更 | 状态 |
|---|---|---|---|
| 1 | `deploy/gitops/kargo/kargo-dev-staging.yaml` | **修正 4 处字段错误** + 3 处 namespace 语义错误 | M |
| 2 | `deploy/gitops/argocd/argocd-applications.yaml` | **修正 1 处 namespace**（`argocd`→`default`） | M |
| 3 | `deploy/multipass/cloud-init-fix-gitops.yaml` | 修复引导（Helm + Rollouts ns + 超时修正） | A |
| 4 | `deploy/multipass/fix-rollouts-ns.sh` | Rollouts RBAC 修复脚本 | A |
| 5 | `deploy/multipass/install-kargo.sh` | Kargo Helm 安装脚本 | A |

（v0.1 已入库的 7 个文件见 commit `9e25f6b5`，本节不重复。）

**刻意未做**：未改 `deploy/k3s-local/**` 任何 manifest；未改 `deny.toml`（本阶段无新增 Rust 依赖）；未动任何 CI workflow；未删除任何既有资源。

---

## §2 验证摘要（全部实测）

### 2.1 安装与宿主

```
MSI SHA256  80d0dbf94f8219b6b9d4d9cdf4ab2020e240772610c62407b544b23a4cd63d87
            = GitHub asset digest  → MATCH（下载前 + 安装前各验一次）
msiexec     exit 0，21s
multipass   1.17.0-rc1.2+g0a043d06-noff.win
local.driver hcs                      ← v0.1 🟡 缺口 #1 闭合
```

### 2.2 集群健康（19/19）

```
node      star-k3s  Ready  control-plane,gitops  v1.36.5+k3s1
allocatable cpu=4  memory=8131220Ki  ephemeral-storage=26603356140
pods      19 Running / 0 重启 / 0 非健康
CRD       29 个
helm      kargo  kargo  deployed  kargo-1.12.1  v1.12.1
```

### 2.3 GitOps 资源（**这一项闭合了 v0.1 的核心 🟡**）

```
kubectl apply -f deploy/gitops/kargo/    → exit 0
  project.kargo.akuity.io/star created
  warehouse.kargo.akuity.io/star-services created
  stage.kargo.akuity.io/dev created
  stage.kargo.akuity.io/staging created

kubectl apply -f deploy/gitops/argocd/   → exit 0
  namespace/star-gitops created
  appproject.argoproj.io/star created
  application.argoproj.io/star-dev created
  application.argoproj.io/star-staging created
```

预期内的非健康状态（非缺陷）：

| 资源 | 状态 | 原因 |
|---|---|---|
| `stage/dev`, `stage/staging` | `0/1 Fulfilled` `Stage has no current Freight` | 镜像坐标为推测值，仓库尚无镜像 |
| `application/star-dev` | `Sync=Unknown` / `Healthy` | `targetRevision: dev` 分支在 GitHub 上尚不存在 |

### 2.4 许可

| 组件 | 许可 | 结论 |
|---|---|---|
| Multipass 1.17.0-rc1 | GPL-3.0（1.16 起 Windows 部分亦完全开源） | ✅ 内部自用零 copyleft 义务，不分发 |
| k3s / Argo CD / Kargo / Rollouts / cert-manager | Apache-2.0 | ✅ |
| **VirtualBox Extension Pack** | **PUEL（非商用）** | ❌ 未安装 |
| **Vagrant** | **BUSL（source-available）** | ❌ 排除 |

### 2.5 实跑抓出并修复的 8 处真实缺陷

| # | 缺陷 | 根因 | 修法 |
|---|---|---|---|
| 1 | **Argo Rollouts CrashLoopBackOff** | `install.yaml` 装到 `default` ns，namespaced RBAC 全不生效 → `configmaps "argo-rollouts-config" is forbidden: User "system:serviceaccount:default:argo-rollouts" cannot get resource "configmaps"` | 装到官方 `argo-rollouts` ns + 显式补 `Role`/`RoleBinding` 读 configmaps/secrets |
| 2 | **Kargo 安装 404** | 我写的 `github.com/akuity/kargo/releases/download/v1.12.1/install.yaml` **不存在** | 改用官方 OCI Helm chart `oci://ghcr.io/akuity/kargo-charts/kargo`（须显式给 `passwordHash` + `tokenSigningKey`） |
| 3 | **误报 4 个 FAIL** | `kubectl wait --timeout 300s` 超时就写 `FAIL:`，实际 cert-manager/Argo CD 2–5 分钟就绪 | 超时提到 600s；**判 FAIL 前先看 Pod 状态** |
| 4 | `subscriptions` 结构错 | 写成 `image: <字符串>` + `tagSelection` | 改 `- image: { repoURL, constraint, imageSelectionStrategy }`（image 是**对象**） |
| 5 | `allowTagsRegexes` 类型/名错 | 写成 `allowTags: '<字符串>'` | 改 `allowTagsRegexes: ['^\d+\.\d+\.\d+$']`；且 v1.x 字段名是 `constraint` 非 `semverConstraint` |
| 6 | **promotion step 用 `with` 而非 `config`** | 按"同类组件惯例"写的，从未被任何 schema 校验抓到 | live CRD 实测 `steps.items.properties` = `as/config/continueOnError/if/retry/task/uses/vars` → 改 `config` |
| 7 | **Kargo namespace 语义全错** | 手写 `Namespace: star-kargo` + **臆造**标签 `kargo.akuity.io/project-name` | 官方：`Project`（Cluster-scoped）reconcile **自动创建同名 ns**。删掉手写 Namespace，资源放 `star` ns。实证 Kargo 自动打了 `kargo.akuity.io/project=true` |
| 8 | Argo CD `namespace: argocd` | 官方 `manifests/install.yaml` **不带 namespace 字段**，资源落在 `default` | 改 3 处为 `default` |

---

## §3 已知缺口（per 缺标比错标）

### 3.1 已闭合（v0.1 🟡 → v0.2 ✅）

| v0.1 缺口 | v0.2 结论 |
|---|---|
| #1 1.17.0-rc1 hcs 在 Win11 家庭版中国区行为 | ✅ 已验证 — `local.driver = hcs`，VM 出 IP 并跑通 k3s + 三件套 |
| #2 hcs 网络能力 | ✅ VM 内 `10.97.0.116` 可达，SSH/k3s 全通 |
| #3 cloud-init 9 阶段 | ✅ 完成；4 个 FAIL 中 3 个是超时误报，1 个真错（Rollouts）已修 |
| #4 磁盘 28G 够不够 | ✅ allocatable 26.6 GB |
| #5 Kargo CRD 字段 | ✅ exit 0（过程中修正 4 处字段错误） |
| #6 Argo CD AppProject/Application | ✅ exit 0（修正 1 处 namespace） |

### 3.2 仍为 🟡

| # | 缺口 | 影响 | 首次落地须做 |
|---|---|---|---|
| 1 | **promotion step `config` 内部字段名未经官方文档核对** | Kargo 真正执行晋级时可能因字段名错而失败 | 查 Kargo v1.12 promotion step 参考表（`git-clone`/`git-commit`/`git-push` 的 config 键名） |
| 2 | `deploy/dev` / `deploy/staging` 分支尚未创建 | `git-push` step 必失败 | `git branch deploy/dev deploy/staging` |
| 3 | 镜像坐标 `ghcr.io/ulyssesleolee/star` 是推测值 | 不准则 Warehouse 永不产出 Freight（静默失效） | 与 CI 实际推送目标核对 |
| 4 | Windows 侧 `kubectl` 直连 6443 未验证 | 当前只能用 `multipass exec` 进 VM 操作 | 配 kubeconfig 或 port-forward |
| 5 | `multipass mount` 未启用 | 仓库同步走 `transfer -r`，需 `local.privileged-mounts=true`（管理员） | 管理员跑一次该设置 |
| 6 | 密钥扫描门禁未接 CI | 本地能跑，CI 无门禁 | 挂 `.github/workflows/` |
| 7 | rc1 → GA 升级路径 | 1.17 正式版未发布 | 正式版发布后重跑驱动设置 |
| 8 | `multipass exec`/`transfer` 的 shell 引号陷阱 | 已在脚本层规避，但仍是易错点 | 见 §5 教训四 |
| 9 | 旧 WSL k3s 数据**未迁移** | 两者不共享状态 | 如需保留须先导出 |

---

## §4 子代理失败接手清单

本阶段**未派任何子代理**（守门 #20：子代理 RPC 不可靠；本任务路径明确，Mavis 自驱完成）。

**未 commit 的非本 session 产出**（per 守门 #9）：`docs/arch-live/` + 3 个 `arch_live_*.py`。

**未入库的一次性探测脚本**（安全策略阻止 `rm`，留在工作区待你处置）：
`deploy/multipass/dump-kargo-crd-schema.sh`、`probe-project-ns.sh`、`probe-project-registration.sh`、`probe-projectconfig.sh`、`probe-subscription-shape.sh`、`star-fix-run.sh`。

---

## §5 守门规则遵守情况

| 守门 | 要求 | 本阶段执行 |
|---|---|---|
| #1 v2/v19 | `cargo check --workspace --all-targets -j 4` 0 err | **不适用** — 本阶段无 Rust 代码改动 |
| #1 v15 | docs 同步须有新事件触发 | ✅ 触发源 = 实跑（8 处缺陷修复） |
| #1 禁回溯叙事 | 禁"原本是/历史形态" | ✅ 修正史只写"v0.x 写成了什么 → 实测报什么 → 改成什么"，每条附实测报错原文 |
| #5 v2 | 密钥禁入 log / 入仓 | ✅ Kargo admin 凭据 VM 内现场生成（`openssl rand` + `htpasswd`），落 `/root/kargo-admin-pass.txt`（0600），**未写入仓库或 cloud-init**；并落机器门禁 |
| #6 | PowerShell only | ✅ 全部命令 pwsh |
| #7 | 0 unsafe | ✅ 无 Rust 代码 |
| #9 | 不 commit 非本 session 产出 | ✅ arch-live 4 项 + 6 个一次性探测脚本已排除 |
| #10 / #14 v4 | 代签 + Mavis 审核 | ✅ author=Ulysses |
| #11 | 缺标比错标 | ✅ §3.2 全部 9 项标 🟡；**预期内的非健康状态也显式区分于缺陷** |
| #19 v19 | 自动化脚本落 `scripts/automation/` | ✅ 3 个脚本，commit message 引用相对路径 |
| #0 商业开源依赖 | 可商用 + 逐版本审闭包 | ✅ §2.4；显式排除 PUEL / BUSL |

### 5.1 方法学教训（本轮最有价值的产出）

**教训一：离线 JSON Schema 校验对这些字段完全无效。**
`Warehouse.spec.subscriptions.items` 标了 `x-kubernetes-preserve-unknown-fields: true`，
`Project` CRD 根本没有 `spec` 字段 —— **CRD 层不管这些结构，只有 admission webhook 管**。
⇒ v0.1 我建的 `scripts/automation/validate_gitops_crds.py`，对缺陷 4/5/7 **一个都抓不到**。
**结论：结构正确性必须由 `kubectl apply --dry-run=server` + admission webhook 判定。**
（该脚本本身没错——它对 CRD 有 schema 的资源有效——但**不能**被当作 GitOps 资源的门禁。）

**教训二：`multipass transfer -r` 到已存在目录不覆盖子文件。**
改完 manifest 重新 transfer，VM 里仍是 7209 字节的**旧文件**（本地已 8429），
dry-run 报出一字未变的旧错误，差点让我以为"修正无效、官方文档在骗我"。
⇒ 传文件后必须做**字节数 parity 校验**（`stat -c %s` vs `Get-Item .Length`），
不一致就 `rm -rf` 后重传。**"命令返回 0"≠"文件已更新"。**

**教训三：单次读数不能判定流程已终止。**
读 `/var/log/star-cloud-init.log` 见 160 字节就断言"runcmd 停了"并开始猜自己写错 ——
真相是一切正常。`exec >>` 重定向下各 runcmd 是**独立进程、各自缓冲**，
我读到的是未 flush 的中间态。
⇒ 判"是否已终止"要用哨兵文件 / 进程状态 / 二次读数是否增长。
**我明明在同一份 cloud-init 里写了 `.complete` 哨兵和逐段标记，却绕过它们直接读文件大小。**

**教训四：`multipass exec` 不能传整串 shell 命令。**
`multipass exec vm -- 'a; b; c'` 会把整串当**单个文件名**，报 `No such file or directory`；
`... | k3s kubectl ...` 这类管道则被 PowerShell 试图在宿主解析。
⇒ 复杂逻辑写成 `.sh` 文件 `transfer` 进去再 `bash`，且**在 VM 内跑 `bash -n`** 做语法门禁
（Windows 侧 bash 看不到 Windows 路径，检查了也白查）。

**教训五：超时只说明"没等到"，不说明"失败了"。**
首版 `wait --timeout 300s` 超时就写 `FAIL:`，4 个 FAIL 里 3 个是误报 ——
真正失败的只有 Argo Rollouts 一个。**门禁在"不确定"时必须去看被测物的实际状态，
而不是把"没等到"记成"失败"** —— 后者会让人去修没坏的组件。

---

## §6 签字栏

| 角色 | 签字 | 说明 |
|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 选型依据成立且经实跑验证（hcs 在 Home 版可用） |
| SRE Lead | — | 🟡 真人未到位（守门 #14 v2） |
| 平台 | 平台 (Mavis 接手 agent per DEC-008) | 🟢 19/19 Pod 健康，GitOps 资源 real-apply exit 0 |
| 评审主持 | 评审主持 (Mavis 接手 agent per DEC-008) | 🟢 8 处缺陷的根因与修法均有实测报错原文佐证 |
| PM | — | 🟡 真人未到位 |

**本报告不宣称**：GitOps 晋级链路已端到端跑通（`git-push` 分支不存在）、镜像订阅已产出 Freight（坐标是推测值）、Argo CD 已成功 sync（GitHub 分支不存在）。**只宣称：宿主层与三件套已就绪并健康，仓库内声明式资源已通过 server-side 校验。**

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 宿主层选型（Multipass 1.17.0-rc1 + hcs）+ 排除 1.16.4/VirtualBox 三条独立依据 + cloud-init/脚本/密钥门禁落地 | 2026-10-05 `ask_6da0b2511bcee76deccb5b21` mp_route_opt1 |
| v0.2 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **实跑补记**：装机 + 建 VM + 三件套 19/19 Pod 跑通；闭合 v0.1 全部 6 项核心 🟡；实跑抓出并修复 **8 处真实缺陷**（Rollouts RBAC 缺 / Kargo URL 404 / wait 超时误报 / subscriptions 结构 / allowTagsRegexes 类型 / step `with`→`config` / Kargo namespace 语义 / Argo CD namespace）；新增 5 条方法学教训 | 实跑（19:32–19:58 JST） |
