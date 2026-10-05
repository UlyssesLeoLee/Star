# HANDOFF-CD-GITOPS-001: CD 交付栈（Argo CD + Kargo + Rollouts）落地现状与剩余工作

> **状态**: 🟢 主体已落地并实跑闭合 ／ 🟡 剩余 6 项待续做
> **日期**: 2026-10-05
> **拍板**: per `ask_fcd0a32a3d12ee35370fe286` + `ask_4b1bb4a6f677125259512988` — 镜像 namespace 归本账号、推 `deploy/*` 到 origin、给 Kargo 配 GHCR 读凭据（凭据部分因 PAT 权限不足未完成，见 §4.1）
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **基线 commit**: `dev` = `7035ae1b`（已推 `origin/dev`，0 ahead / 0 behind）
> **关联文档**: [ADR-0054](../architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md) ｜ [PHASE-MULTIPASS-K3S-GITOPS-REPORT v0.2](PHASE-MULTIPASS-K3S-GITOPS-REPORT.md) ｜ [HANDOFF-SEC-LICENSE-001](HANDOFF-SEC-LICENSE-001.md) ｜ [`deploy/multipass/README.md`](../../deploy/multipass/README.md)

---

## §1 一句话

**Multipass VM 上 Argo CD v3.5.3 + Kargo v1.12.1 + Argo Rollouts v1.10.0 三件套已实跑跑通（全节点 22 Pod：19 Running + 3 Completed / 0 重启 / 0 非健康），`dev` 已与 `origin/dev` 全量同步；但 Kargo 晋级链路因「GHCR 读凭据未注入」停在产出 Freight 之前 —— 坐标已修正、控制器已在轮询正确仓库，唯独凭据这一环未闭合。**

## §2 事实基础（2026-10-05 全部实测，非引用）

### 2.1 基础设施实跑状态

| 项 | 实测值 |
|---|---|
| Multipass | 1.17.0-rc1.2+g0a043d06-noff.win，`local.driver = hcs` |
| VM | `star-k3s`，Running，`10.97.0.116`，Ubuntu 24.04.5 LTS |
| k3s | v1.36.5+k3s1，节点 Ready，allocatable 26.6 GB |
| Pod | 全节点 22 个：19 Running + 3 Completed（Job 类，正常结束）／ 0 重启 ／ 0 非健康 |
| Pod 分布 | `kargo` 8（5 常驻 + 3 GC Job）、`default` 7（Argo CD）、`cert-manager` 3、`kube-system` 3、`argo-rollouts` 1 |
| cert-manager | v1.18.2，3/3 |
| Argo CD | v3.5.3，7/7（`default` ns） |
| Argo Rollouts | v1.10.0，1/1（`argo-rollouts` ns） |
| Kargo | v1.12.1（Helm chart `kargo-1.12.1`），5/5（`kargo` ns） |
| Kargo 常驻 Pod | 5/5 Running、0 重启、122m 稳定（`kargo-garbage-collector` 为 Job `Completed`，正常） |

### 2.2 宿主可达性（本轮新验证，销掉一条旧 🟡）

| 检查 | 结果 |
|---|---|
| Windows 宿主 → k3s apiserver `10.97.0.116:6443` | **TCP 通** |
| `https://10.97.0.116:6443/version`（无凭据） | **HTTP 401** = 认证挑战，服务存活 |
| Windows 宿主 `kubectl` | **v1.36.1 已在 PATH** |

→ 结论：**网络与工具都已就绪**，缺的只是宿主侧的 kubeconfig（内含 client cert/key，属凭据，按守门 #5 不得入库；见 §4.6）。

### 2.3 本机 git 网络事实（影响一切 push/fetch）

| 通道 | 状态 |
|---|---|
| SSH（`git@github.com:...`，仓库 `origin` 默认） | ❌ 本地 DNS 解析不了 `github.com`，TCP 22 不通 |
| HTTPS + `HTTP(S)_PROXY=127.0.0.1:10808` | ✅ 通 |
| `gh` keyring 凭据 | 已登录 `UlyssesLeoLee`，scopes `gist, project, read:org, repo, workflow`，协议 https |

> ⚠️ 仓库 `origin` 的 URL 仍是 SSH。**在本环境下任何走 `origin` 的 push/fetch 都会失败**。本轮所有推送均用显式 HTTPS URL + `gh auth setup-git` 的 credential helper 完成，未改动 `origin` 配置（该配置存于共享 `.git/config`，会影响全部 14 个 worktree，超出单次授权范围）。

### 2.4 Kargo Warehouse 当前状态

```
kubectl apply -f deploy/gitops/kargo/   → APPLY_EXIT=0   (live admission webhook 接受)
仓库内 4 个 subscription repoURL（集群内已确认）:
  ghcr.io/ulyssesleolee/star-canvas-engine
  ghcr.io/ulyssesleolee/star-domain-canvas
  ghcr.io/ulyssesleolee/star-canvas-realtime
  ghcr.io/ulyssesleolee/star-canvas-game
控制器最新错误 (12:49:56Z):
  error listing tags for repo URL ghcr.io/ulyssesleolee/star-canvas-engine
```

**坐标正确、控制器在轮询、因缺凭据而失败** —— 属预期未闭合状态，非回归。

## §3 本轮决策（2026-10-05）

| # | 决策 | 拍板来源 | 结果 |
|---|---|---|---|
| 1 | 领先分支全部并入 `dev` | 用户指令 | ✅ 完成，`git branch --no-merged dev` 为空 |
| 2 | 只推 `deploy/dev` + `deploy/staging`（不动 201 个 `dev` 提交） | `ask_fcd0a32a3d12ee35370fe286` | ✅ 已推，后续用户另行授权推 `dev` |
| 3 | 镜像 namespace 归本账号 `UlyssesLeoLee` | `ask_4b1bb4a6f677125259512988` | ✅ CI / k3s 清单 / Kargo 三处已统一 |
| 4 | 给 Kargo 配 GHCR 读凭据（而非把镜像转公开） | `ask_fcd0a32a3d12ee35370fe286` | 🟡 **未完成**，因 PAT 缺 `read:packages`，见 §4.1 |
| 5 | 不注入一把已证明读不到目标的凭据 | Mavis 判断（守门 #11 缺标比错标） | ✅ 未创建任何 Secret |

## §4 剩余工作（续做清单，按优先级）

### 4.1 🔴 P1 — Kargo GHCR 读凭据（当前唯一阻塞晋级链路）

**现状**：本机 `GHCR_PAT` 是有效 fine-grained PAT，但缺 Packages 读权限。实测：

| 目标 | 结果 | 含义 |
|---|---|---|
| `fluxcd/source-controller`（公开对照） | **OK, 100 tags** | 凭据本身有效，方法正确 |
| `ulyssesleolee/star-canvas-*`（4 个） | **403** | 已认证但未授权 |
| `ulyssesleolee/star` / `ulyssesleolee/bff` | **404** | 仓库不存在 |

**判据**：`403` 与 `404` 语义不同 —— 403 = 仓库存在但无权限，404 = 不存在。

**续做步骤**：

1. 在 GitHub 侧给该 PAT（或新建）授予 **`read:packages`**（fine-grained PAT 需在 *Packages* 权限下勾选 *Read*；classic PAT 需 `read:packages` scope）
2. 确认 4 个 `star-*` 镜像**是否已实际推送**（若 CI 从未成功跑过，包不存在，则无论权限如何都是 404）
3. 跑注入脚本（**会先验权，不通过则 fail closed 中止，不创建 Secret**）：
   ```powershell
   $env:KARGO_VM = 'star-k3s'
   pwsh -NoProfile -File scripts/automation/kargo_image_credential.ps1 `
     -RepoUrl 'ghcr.io/ulyssesleolee/star-canvas-engine'
   ```
   多仓库可用 `-RepoUrl` 传数组 + `-UseRegex`（Secret 内写 `repoURLIsRegex: "true"`）
4. **验收判据（不可只看 apply 返回码）**：读控制器日志
   ```bash
   sudo k3s kubectl logs -n kargo deploy/kargo-controller --tail=100 | grep -i warehouse
   ```
   出现 Freight 记录、`Warehouse.status` 有 `phase` / `freightCount` 才算生效。**`status` 全空 = 还没 reconcile 过；`apply` exit 0 ≠ 生效**（本轮已实证过这条，见 §6.2）。

**Secret 形态**（Kargo 按约定发现，**不走 CRD 字段**）：

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: kargo-image-ghcr          # 名字任意
  namespace: star                # 必须放 Project 自己的 namespace
  labels:
    kargo.akuity.io/cred-type: image
stringData:
  repoURL: ghcr.io/ulyssesleolee/star-canvas-engine
  username: UlyssesLeoLee
  password: <PAT>
```

> 已用 live CRD 确认：`Warehouse.spec` / `ProjectConfig.spec` / `Project` **全无 credential 字段**（`grep -oE '"[a-zA-Z]*[Cc]redential[a-zA-Z]*"'` 零命中），所以只能走 Secret 约定。

### 4.2 🟡 P2 — `imagePullSecrets: ghcr-pull-secret` 在集群中不存在

`deploy/canvas-game-k3s.yaml` 4 个 Deployment 都引用了 `imagePullSecrets: - name: ghcr-pull-secret`，但**该 Secret 在集群任何 namespace 都不存在**（`kubectl get secret -A | grep ghcr` → 无命中）。

私有镜像在 kubelet 拉取时同样需要凭据，**与 Kargo 的轮询凭据是两回事**，即使 §4.1 通了，Pod 仍会 `ErrImagePull`。需另建 dockerconfigjson 类型的 pull secret。

### 4.3 🟡 P3 — promotionTemplate step 内部字段名未核对

`steps.items` 内部 `config` 的**具体键名**未经官方 promotion step 参考表逐项核对。CRD schema 只校验到 steps 数组层（live CRD：`as/config/continueOnError/if/retry/task/uses/vars`），**server dry-run 同样校验不到 step 内部** —— 这是 `kargo-dev-staging.yaml` 唯一残留的结构不确定面。晋级真正跑起来前需对照官方文档逐项确认。

### 4.4 🟡 P4 — cloud-init 密钥扫描门禁未接 CI

`scripts/automation/cloudinit_secret_scan.py` + `cloudinit_secret_scan_mutation.py` 已落库且实跑通过，但**未挂进 `.github/workflows/` 任何一条**（grep `cloudinit_secret_scan` 零命中）。缺 CI 强制 = 门禁只在本地生效。

本轮复核实测（双向）：

```
# 真实文件
python scripts/automation/cloudinit_secret_scan.py \
  deploy/multipass/cloud-init-k3s-gitops.yaml deploy/multipass/cloud-init-fix-gitops.yaml
→ PASS: 2 file(s) scanned, 0 secret material found     exit 0

# 变异对照组（注入伪造 PEM 私钥块）
→ FAIL: [PEM 私钥块]  BEGIN: -----BEGIN RSA PRIVATE KEY-----   exit 1

python scripts/automation/cloudinit_secret_scan_mutation.py
→ RESULT: PASS=6 FAIL=0 TOTAL=6                          exit 0
```

> 该脚本需**显式传入文件参数**，无参运行会打 usage 并 exit 2 —— 那是用法错误，不是扫描通过。

### 4.5 🟡 P5 — `origin` 仍是 SSH，在本环境必失败

见 §2.3。若确定长期在本环境开发，建议把 `origin` 改为 HTTPS（影响全部 14 个 worktree 的共享 `.git/config`，需明确授权后再动）。否则每次推送都得走显式 URL。

### 4.6 🟡 P6 — 宿主侧 kubeconfig 未落地

网络与 `kubectl` 都已就绪（§2.2），只差把 VM 内 `/etc/rancher/k3s/k3s.yaml`（`--write-kubeconfig-mode 644`）取到宿主。**该文件含 client cert/key，属凭据，按守门 #5 须放仓库外路径并确认不入库。**

### 4.7 🟡 P7 — `multipass mount` 未启用

`local.privileged-mounts = false`。仓库同步目前走 `multipass transfer -r`（每次传完必须做字节数 parity 校验 —— `transfer -r` 到已存在目录**不覆盖子文件**）。启用需管理员 `multipass set local.privileged-mounts=true`。

### 4.8 🟡 P8 — `registry_check.py` 197 条历史 WARN

`actual_scripts=232, warnings=197, errors=0`。全为历史存量脚本（`__init__.py` / `__tests__/` / `wbs_*` 等）未登记进 `scripts/automation/registry.md`。本轮已用「合并只 M 不 A `.py` 文件」做对照组，确认非新引入。清理属技术债，不阻塞。

## §5 续做触发条件

满足任一即应启动 §4.1：

1. 需要 Kargo **自动产出 Freight**（当前晋级链路为空转）
2. 需要 Argo CD 应用真正 `Synced`（依赖 §4.1 + §4.2 双通）
3. 4 个 `star-*` 镜像首次带 `v*` tag 推送（`allowTagsRegexes: ^\d+\.\d+\.\d+$` 只收三段式 semver）

§4.2 可独立于 §4.1 触发：任何一次 `kubectl apply -f deploy/canvas-game-k3s.yaml` 都会因缺 pull secret 而 `ErrImagePull`。

## §6 方法学留档（本轮实证，续做时直接复用）

### 6.1 「真实凭据与伪造垃圾返回完全相同的结果」= 量具坏了

诊断 GHCR 时我把 PAT 直接当 Bearer 塞进 `/v2/<repo>/tags/list`，得到「所有仓库一律 403」，据此写下「`gho_` OAuth token 不被 GHCR 接受，需要 PAT」—— **方向性错误结论**。正确流程是 `Basic(user:PAT)` 换 registry token 再作 Bearer。

揪出它的不是报错，是这个对照：

```
Bearer <真实 fine-grained PAT>  -> 403
Bearer <字面量 "ghp_garbage">    -> 403    <- 一模一样
Basic base64("u:p") 垃圾         -> 401
完全不带 Authorization           -> 401
```

一把**有效**的 PAT 与一把**垃圾串**返回逐字节相同的结果 → 凭据压根没被当作凭据评估。

> **规律**：诊断的第一判据不是「报什么错」，是「**换掉被测物结果会不会变**」。结果不变 → 问题在量具/方法；结果变 → 才轮到讨论被测物。这个对照成本一行代码，能挡掉一整类方向性错误结论。

### 6.2 `kubectl apply` exit 0 ≠ 配置生效

Kargo Warehouse 的 `repoURL` 长期写错（按 `ghcr.io/<owner>/<repo>` 惯例**推断**），而：

```
kubectl apply --dry-run=server  → exit 0
kubectl apply                   → exit 0
Project.status                  → True / "Project is synced and ready for use"
```

**但控制器从 11:00Z 起每个 reconcile 周期都在失败**，`Warehouse.status` 全空。

**判「配置生效」要三层，返回码只算第 0 层**：
1. apply 返回码 —— 只证明「请求被 API server 接受」
2. `.status` 的 `phase` / `conditions` / `lastPollTime` —— 证明控制器**reconcile 过**
3. **控制器日志** —— 证明 reconcile 的**结果**

只读 2 层会漏（status 可能还没被填）。根因是 `subscriptions.items` 标了 `x-kubernetes-preserve-unknown-fields`，CRD 层完全不校验该结构。

### 6.3 「同一文件三处三个值」的注释漂移没人会报错

`publish-canvas-game.yml` 头注释长期写 `starghorg`，env 写 `SayAtelier`，真实归属是第三个值。**注释不参与执行，没有任何门禁会为它报错**，读注释的人无法发现。已改为以 `env.IMAGE_NAMESPACE` 为唯一事实源并逐字对齐。

同类：本仓 `IMAGE_OWNER` 原本同时兼任**登录名**（需 camelCase）与**镜像路径**（需全小写）两个**方向相反**的语义，共用一个变量意味着改一边必然破坏另一边。已拆成两个变量。

### 6.4 变异测试必须能区分「被测物没跑」

本轮修了 `scripts/automation/license_gate_mutation.py` 的状态泄漏（漏还原 `Cargo.lock` 留下幽灵 member；`Cargo.toml` 文本模式往返把 LF 全量转 CRLF，被本仓 `core.autocrlf=true` 恰好掩盖）。改法是 bytes 级备份/还原 + 后置断言。

变异测试证明断言有牙：

| | 结果 |
|---|---|
| 对照组（修复后） | `PASS=10 FAIL=0 INVALID=0 LEAK=0` exit 0，`Cargo.toml`/`Cargo.lock` 跑前跑后 SHA256 逐字节一致 |
| 变异体（拆掉 lock 还原） | exit **2** + `[LEAK] Cargo.lock not restored to pre-run bytes` |
| 变异体下的 10 个许可用例 | **仍全 PASS** → 证明 LEAK 通道与许可判定**独立**，清理坏掉不会伪造出绿的许可门 |

### 6.5 探针超时 ≠ 被测对象失败

本轮一个后台任务（`kubectl logs --tail=100000` 串 grep）撞 3600s 墙钟上限被标 `failed`。用有界查询复查：5 个 Kargo 常驻 Pod 全 1/1、0 重启、122m 稳定。**超时只说明「没等到」，不代表系统失败**；同理单次读数不能判定流程终止。

## §7 相关文件索引

| 文件 | 作用 |
|---|---|
| `deploy/multipass/cloud-init-k3s-gitops.yaml` | VM 首次引导（k3s + 三件套） |
| `deploy/multipass/cloud-init-fix-gitops.yaml` | 修复引导（Helm + Rollouts ns + 600s 超时） |
| `deploy/multipass/fix-rollouts-ns.sh` | Rollouts RBAC 修复（装到官方 `argo-rollouts` ns + 显式补 Role/RoleBinding） |
| `deploy/multipass/install-kargo.sh` | Kargo Helm 安装（OCI chart，须显式给 `api.adminAccount.passwordHash` + `tokenSigningKey`，两者无默认值） |
| `deploy/multipass/README.md` | 实跑结果 + 残留缺口 + 10 条故障排查表 |
| `deploy/gitops/kargo/kargo-dev-staging.yaml` | Project / Warehouse / 2×Stage；**修正史 7 处**全部记录在案 |
| `deploy/gitops/argocd/argocd-applications.yaml` | Argo CD Applications（namespace `default`） |
| `deploy/canvas-game-k3s.yaml` | 4 个 canvas-game Deployment + envoy，**引用尚不存在的 `ghcr-pull-secret`**（§4.2） |
| `.github/workflows/publish-canvas-game.yml` | 4 镜像构建推送；`IMAGE_OWNER` / `IMAGE_NAMESPACE` 已拆分 |
| `scripts/automation/kargo_image_credential.ps1` | Kargo 镜像凭据注入（建前强制验权、fail closed、守门 #5 不打印） |
| `scripts/automation/license_gate_mutation.py` | 许可门禁变异测试（已修状态泄漏 + 字节级还原） |
| `scripts/automation/cloudinit_secret_scan.py` | cloud-init 密钥扫描门禁（**未接 CI**，§4.4） |

## §8 守门约束（续做时适用）

- **守门 #1 派生规 v2**：`cargo check --workspace --all-targets -j 4` 须 0 err（本轮基线：exit 0 / 2m14s）
- **守门 #5 v2**：PAT / kubeconfig / 密钥一律走 env 或运行时注入，**禁入 log、禁入仓**；命令行不接受明文 PAT 参数（会进 shell history）
- **守门 #7**：0 unsafe
- **守门 #11 缺标比错标**：未验证项必须标 🟡，不得以 ✅ 冒充；须显式区分「预期内的非绿色」与「真实故障」
- **守门 #19 v19**：自动化脚本落 `scripts/automation/`，commit message 引用相对路径
- **守门 #30**（AGENTS.md §4.1 派生规）：任何门禁落库前必做变异测试 + 对照组，且断言退出码须区分「确实有问题」(1) 与「没测到」(2)
- **AGENTS.md §0 商业开源依赖硬约束**：三件套均为 Apache-2.0 实测；新增组件须逐版本审查完整构建/分发闭包，仅核对上游主许可证**不足以放行**
- **提交信息**：branch 相关内容用 Star 版本号（v0.X）+ 5 角色；RGS 仓内容用 v3.X + 5 域 Lead

## §9 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：CD 交付栈实跑现状 + 8 项剩余工作清单 + 5 条方法学留档 | 用户指令「剩余内容记入 handoff」 |
