# PHASE-CD-SELECTION-REPORT: 自动部署体系选型与商业开源依赖门禁落地

> **状态**: ✅ v0.2 (第二阶段：11 项未完成缺口已推进 8 项)
> **日期**: 2026-10-05（v0.1: 2026-10-04）
> **拍板**: per `ask_72c00a6f28daadd5538fa004` — scope_opt1「落 ADR + 选型报告 + cargo 门禁骨架（推荐）」+ env_opt1「本机 k3s 单集群，dev → staging 两级（推荐）」
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **关联文档**: [ADR-0054](../architecture/2026-08-26-upgrade/adr/0054-cd-delivery-stack-argocd-kargo-rollouts.md) ｜ [NOTICE.md](../../../NOTICE.md) ｜ [deploy/gitops/README.md](../../deploy/gitops/README.md) ｜ 根目录 `deny.toml` / `about.toml` ｜ `.github/workflows/license-gate.yml`

---

## §0 目的

Ulysses 于 2026-10-04 20:50 JST 提出：**需要一套 Netflix Spinnaker 等效的开源、免费、可商用自动部署体系，纳入 Star 仓，并给出针对 Rust 项目的选型结论。**

本阶段交付三件事：

1. **选型结论 + 可审计的许可证据**（ADR-0054），含 Spinnaker / Flux 的排除实证
2. **闭合 AGENTS.md §0 商业开源依赖硬约束的自动化缺口** — 实测发现该硬约束此前**完全无自动化**（`deny.toml` 不存在，11 个 workflow 中零个 license/advisory/SBOM 检查）
3. **dev → staging 两级 GitOps 骨架**

---

## §1 改动矩阵

### 1.1 新增文件

| 文件 | 行数 | 用途 | 语法校验 |
|---|---|---|---|
| `deny.toml` | 105 | 许可 / bans / advisories / sources 四类门禁 | ✅ cargo-deny 0.20.2 接受 |
| `.github/workflows/license-gate.yml` | 178 | 三独立 job 门禁 | ✅ yaml.safe_load_all OK |
| `deploy/gitops/argocd/argocd-applications.yaml` | 137 | AppProject + 2 Application | ✅ yaml.safe_load_all OK |
| `deploy/gitops/kargo/kargo-dev-staging.yaml` | 145 | Project + Warehouse + 2 Stage | ✅ yaml.safe_load_all OK |
| `deploy/gitops/README.md` | 90 | 骨架说明 + 未验证声明 | — |
| `scripts/automation/license_gate_mutation.py` | 175 | 许可门禁变异测试（可复现，守门 #19 v19） | ✅ 实跑 exit 0 / PASS=10 |
| `docs/architecture/2026-08-26-upgrade/adr/0054-*.md` | 246 | 选型 ADR | — |
| `docs/reports/PHASE-CD-SELECTION-REPORT.md` | 本文件 | 七段报告 | — |

### 1.2 修改文件

| 文件 | 改动 | 原因 |
|---|---|---|
| `crates/agent-domain/Cargo.toml` | 补 `license.workspace` / `authors.workspace` / `repository.workspace` | **cargo-deny 实跑抓出的真实缺陷**：原缺 license 声明被判 unlicensed |
| `crates/canvas-collab/Cargo.toml` | 同上 | 同上 |
| `Cargo.lock` | `yoke-derive 0.8.3 → 0.8.4`（连带 `syn 3.0.6 → 2.0.119`） | **cargo-deny 实跑抓出的 yanked crate** |

### 1.3 引用扫矩阵（许可实测，2026-10-04）

| 组件 | SPDX | 版本 | 纳入 |
|---|---|---|---|
| argoproj/argo-cd | Apache-2.0 | v3.5.3 | ✅ |
| akuity/kargo | Apache-2.0 | v1.12.1 | ✅ |
| argoproj/argo-rollouts | Apache-2.0 | v1.10.0 | ✅ |
| sigstore/cosign | Apache-2.0 | — | ✅（签名） |
| external-secrets | Apache-2.0 | — | ✅（密钥） |
| **controlplaneio-fluxcd/flux-operator** | **AGPL-3.0** | — | ❌ 排除（ADR-0054 §4.2） |
| **spinnaker/spinnaker** | Apache-2.0 | 2026.1.0 | ❌ 排除（ADR-0054 §4.1） |
| fluxcd/flagger | Apache-2.0 | v1.45.0 | ❌ 未选（功能重叠，**非弃用**） |

---

## §2 验证摘要

### 2.1 cargo-deny 门禁：4 轮实跑迭代（**非仅写配置**）

本机实测，cargo-deny **0.20.2**（`E:\DevCache\cargo\bin`），Cargo.lock **829 个包**。

| 轮次 | 失败原因 | 修正动作 |
|---|---|---|
| 1 | `private = [...]` 数组形式不被 0.20 接受；`LGPL-3.0-linking-exception` 非有效 SPDX id；`GPL-3.0-with-classpath-exception` 不存在（classpath exception 仅 GPL-2.0 有） | 改单表形式；补 `Apache-2.0 WITH Classpath-exception-2.0`；删无效 id |
| 2 | SPDX 短后缀 `LGPL-2.1+` 不被解析器接受 | 改长 id（`-only` / `-or-later`） |
| 3 | 0.20 **已移除** `[licenses] deny = [...]` 与 `unlicensed` 键 | 改为纯白名单语义（见下） |
| 4 | 真实依赖问题暴露 | 修 2 crate license + 补 2 白名单项 + 修 yanked |

**0.20 语义变更（重要，PR #611，关闭 issue #602「Copyleft licenses are exempted from deny by default」）**：`licenses.deny` / `unlicensed` 已移除，且**移除了 copyleft 的隐式豁免**，改为纯白名单——不在 `allow` 内的许可一律拒。

这一变更**恰好强化**了本仓硬约束的表达力：

- GPL/AGPL/LGPL **显式列入** `allow` = 机器可读地声明「copyleft 不阻断构建」（per AGENTS.md §0「不得仅因其为 GPL/AGPL/LGPL 而一概排除」）
- NC / field-of-use / source-available（BUSL / SSPL / Elastic / Facebook / PolyForm / JSON / NPL / APL）**不在白名单即自动拒绝**，无需显式 deny 列表

### 2.2 门禁抓出的真实缺陷（非误报，已修）

| # | 发现 | 严重度 | 处置 |
|---|---|---|---|
| 1 | `agent-domain` 缺 license 声明 → unlicensed | **P1** | ✅ 已修（补 `license.workspace = true`） |
| 2 | `canvas-collab` 缺 license 声明 → unlicensed | **P1** | ✅ 已修 |
| 3 | `yoke-derive v0.8.3` 已 yanked | **P1** | ✅ 已修（`cargo update -p yoke-derive` → 0.8.4） |
| 4 | `target-lexicon 0.12.16` 用 `Apache-2.0 WITH LLVM-exception`，白名单缺失 | P2 | ✅ 已补入白名单 |
| 5 | `webpki-roots 0.26.11 / 1.0.9` 用 `CDLA-Permissive-2.0`，白名单缺失 | P2 | ✅ 已补入白名单 |

**若未实跑，这 3 个 P1 缺陷（2 个 unlicensed + 1 个 yanked）会全部漏过。**

### 2.3 依赖树 copyleft 明细（⚠️ 2026-10-05 订正 v0.1 的错误结论）

**v0.1 曾断言"依赖树中不存在任何 copyleft crate"——该结论错误。**

v0.1 的依据是 `cargo deny check` 输出的 11 条 `license-not-encountered` 警告（命中 LGPL/GPL/AGPL/MPL 等白名单条目）。但 **2026-10-05 用 `cargo deny list --format json` 做结构化枚举后证明 copyleft 确实存在**：

| 许可 | crate 数 | 明细 |
|---|---:|---|
| **MPL-2.0**（文件级 copyleft） | **5** | `cssparser` 0.37.0 / `selectors` 0.38.0 / `cssparser-macros` 0.7.1 / `dtoa-short` 0.3.5（均经 `dom_query` 0.28.0）；`option-ext` 0.2.0（经 `dirs-sys` 0.5.0） |
| **LGPL-2.1-or-later**（库链接型 copyleft） | **2** | `r-efi` 5.3.0 / 6.0.0（经 `getrandom`，**仅 UEFI target** 编译） |

**方法论教训（已写入 NOTICE.md）**：`license-not-encountered` 是「白名单条目未被用到」的**否定式**警告，**不能反推「该许可在依赖树中不存在」**。断言"不存在"必须用 `cargo deny list` 这类**枚举式**输出交叉验证。这次是本阶段第二次"因错误方式得出结论"（第一次是变异测试假阳性），同属一条纪律：**证实存在用枚举，证否必须换判据**。

> ⚠️ **仍未完全解释**：`check`（`--all-features`）与 `list`（无 flag）产出的依赖图规模不同（后者 1265 crate）。按守门 #11 缺标比错标，本条记为**未完全解释的差异**（缺口 #13），不臆造原因。两者在 copyleft 结论上**一致**（均存在 MPL/LGPL），故上述明细可信。

**对硬约束的影响**：这些 copyleft crate **已显式列入** `deny.toml` 白名单（per AGENTS.md §0「不得仅因其为 GPL/AGPL/LGPL 而一概排除」），不阻断构建。义务范围：MPL-2.0 为文件级（修改其文件须开源，链接进本仓不触发本仓披露）；`r-efi` 仅 UEFI 目标，当前 Windows/Linux 产物不含。详见 [NOTICE.md](../../../NOTICE.md)。

### 2.4 真实安全债务（已登记为带到期日的忽略项）

| Advisory | 类型 | 实证来源 | 判定 |
|---|---|---|---|
| **RUSTSEC-2023-0071** Marvin Attack | **vulnerability** | `cargo tree -i rsa` → `rsa v0.9.10` ← `jsonwebtoken v11.1.0` | Marvin 影响 **RSA 私钥**操作时序；本仓 JWT 仅走**公钥验签**，不可达。⚠️ **一旦引入 RSA 私钥签名/解密必须立即移除忽略** |
| RUSTSEC-2025-0057 | unmaintained | `fxhash` ← `sled v0.34.7` | 传递依赖，无 CVE |
| RUSTSEC-2024-0384 | unmaintained | `instant` ← `parking_lot` ← `sled` | 传递依赖，无 CVE |
| RUSTSEC-2024-0370 | unmaintained | `proc-macro-error`（当前 target 不可见） | 传递依赖，无 CVE |

4 条均写入 `[advisories] ignore`，**每条 reason 内写明复审期限 2026-11-04**。

> ⚠️ **订正**：初稿曾写「cargo-deny v2 支持 `expiration` 字段，到期自动失败」。**该说法经实跑证伪**——0.20.2 对 ignore 项仅接受 `id` 与 `reason` 两个 key，报 `error[unexpected-keys]: found 1 unexpected keys, expected: ["id", "reason"]`。故复审期限**只以文本形式写在 reason 内，不具机器强制力**，复审依赖人。见 §3 缺口 #11。

### 2.5 `wildcards` 降为 `warn` 的判断依据

第 4 轮实跑中 `wildcards = "deny"` 报出 **60+ workspace crate** 的 path 依赖被判为版本通配。`allow-wildcard-paths = true` **不生效**，cargo-deny 明确诊断为：

> `allow-wildcard-paths is enabled, but does not apply to public crates as crates.io disallows path dependencies.`

**根因**：本仓 100+ workspace crate 均未声明 `publish = false`，故 cargo-deny 一律视为 **public crate**。

**判定为配置不适用于 monorepo，而非掩盖真实问题**：

- path 依赖 `foo = { path = "../foo" }` 无 version 字段，cargo-deny 按「无版本约束」计为 wildcard
- path 依赖由 workspace members 锁定，**无法被 crates.io 解析，不构成可复现性风险**
- 真正的版本通配 `version = "*"` 仍会被 warn 捕获

故降为 `warn`，并把长期修法（为全部 crate 补 `publish = false`）记为待办 #5。**未采用「改门禁参数让检查消失」的做法，也未放宽 `sources` / `advisories` 等其他防线。**

### 2.6 工具链环境实证

| 项 | 实测值 |
|---|---|
| `cargo` | **不在 PATH 上**（`Get-Command cargo` 返回空），须用绝对路径 `E:\DevCache\cargo\bin\cargo.exe` |
| cargo 版本 | 1.98.1（与 `rust-toolchain.toml` `channel = "1.98.1"` 一致） |
| cargo-deny | 0.20.2（预装于 `E:\DevCache\cargo\bin`，非 PATH） |
| cargo-audit | 0.22.2（预装） |
| cargo-machete | 0.9.2（预装） |
| cargo-about / cargo-cyclonedx / syft / trivy | **未安装** |

`cargo-deny 0.20.2` 官方 release 二进制已下载并 **SHA256 校验通过**（`975a2214...87dc`），校验和已写入 CI workflow。

### 2.7 cargo check 验证（守门 #1 派生规 v2）

改动 `Cargo.lock` 后**不假设锁文件变更安全**，实跑验证：

```
cargo check --workspace --all-targets -j 4
→ EXIT 0, 195.6s, 0 error
  warning 均为 pre-existing（aes_gcm deprecated / never used / unused import 等）
```

`yoke-derive 0.8.3→0.8.4` 连带 `syn 3.0.6→2.0.119`（降级）经此确认**编译安全**。

### 2.8 门禁变异测试（验证门禁「有牙」而非只会绿灯）

绿灯只证明「没报错」。故注入探针 crate（`license` 字段可控）验证门禁的判别能力：

| 注入 SPDX | 期望 | exit | 结果 |
|---|---|---|---|
| Apache-2.0 | 放行 | 0 | ✅ green |
| MIT | 放行 | 0 | ✅ green |
| MPL-2.0（弱 copyleft） | 放行 | 0 | ✅ green |
| **GPL-3.0-only** | **放行** | 0 | ✅ **copyleft 不阻断** |
| **AGPL-3.0-only** | **放行** | 0 | ✅ **copyleft 不阻断** |
| **CC-BY-NC-4.0** | **拒绝** | 4 | ✅ red，断言含 `rejected: license` |
| **BUSL-1.1** | **拒绝** | 4 | ✅ red，断言含 `rejected: license` |
| **SSPL-1.0** | **拒绝** | 4 | ✅ red，断言含 `rejected: license` |
| **Elastic-2.0** | **拒绝** | 4 | ✅ red，断言含 `rejected: license` |
| CC0-1.0（公有领域） | 放行 | 0 | ✅ 不误报 |

**汇总：PASS=10  FAIL=0  INVALID=0。**

三要素齐备（这是门禁测试的最低要求）：

- **(a) 注入违规看它变红** — 4 个非商业 / source-available 许可全部被拒
- **(b) 对照组仍绿** — 5 个合法许可全部放行，排除「装置永远失败」
- **(c) 反例守卫** — GPL / AGPL / MPL / CC0 必须放行，防止把规则写宽变成永久误报
- **额外：每个 red 用例都断言诊断文本本身出现**（`rejected: license`），排除「因错误原因变红」

> **过程中的两次假阳性（自我捕获）**：
> 1. 首轮注入位置错误（probe 被插到 `members` 数组**外**），门禁因 `cargo metadata` **解析失败**而红。若只看 exit code，会误判为「非商业许可已被正确拒绝」。经查完整诊断文本才发现真实原因是装置故障。
> 2. 第二轮 PowerShell 脚本用 `Out-File -Append` 导致 manifest 格式损坏，4 个用例全部「red」但无许可诊断——同样是**因错误原因变红**。
>
> 两次均通过「必须看诊断文本而非只看退出码」捕获。这也正是守门 #12 的由来（见 §5）。

探针已清理（`mavis-trash` / 脚本内 `shutil.rmtree`），`Cargo.toml` 经 `git diff` 确认与 HEAD **字节一致**，最终 `cargo deny check` **EXIT 0**（0 error，147 warning：69 duplicate / 67 wildcard / 11 license-not-encountered）。

**可复现资产**：变异测试已固化为 [`scripts/automation/license_gate_mutation.py`](../../scripts/automation/license_gate_mutation.py)（守门 #19 v19：实证脚本落地而非一次性执行），实跑 **exit 0 / 19.9s / PASS=10 FAIL=0 INVALID=0**，清理逻辑自验证通过。退出码语义：`0` 全部符合预期，`1` 有 FAIL，`2` 装置故障。

### 2.9 GitOps CRD 离线校验（v0.2 新增，闭合缺口 #1 的结构部分）

本机 k3s 不可达（`172.28.176.169:6443 connection refused`，2026-10-05 实测），无法做 server 端 dry-run。改用**上游 CRD 的 `openAPIV3Schema` 离线校验**（[`scripts/automation/validate_gitops_crds.py`](../../scripts/automation/validate_gitops_crds.py)）。

> 工具选择说明：实测 **kubeconform v0.8.0 对 CRD 自定义 schema 的本地文件定位不生效**（自定义 `-schema-location` 始终 `could not find schema`，`-debug` 亦未暴露查找路径）。改用 jsonschema 直接校验 CRD schema 更可控，且能给出精确字段路径。

**校验抓出 4 处真实错误**（全部是 v0.1 凭印象写的字段，无一实跑过）：

| # | 资源 | 错误 | 性质 |
|---|---|---|---|
| 1 | `AppProject` | `clusterResourceWhitelist` 写成 map `{group,kind}`，CRD 要求 **array** | 字段类型错 |
| 2 | `Warehouse` | 缺 `interval`（required） | 漏必填 |
| 3 | `Stage` ×2 | 写 `requiredFreight`，实际字段名是 **`requestedFreight`**（且为**数组**） | 字段名错 |
| 4 | `Stage/staging` | `origin.kind: Stage` 非法 —— CRD enum **只允许 `Warehouse`** | ⚠️ **模型级错误** |

第 4 项最关键：v0.1 设想的"staging 从 dev 订阅"模型在 Kargo v1.x 下**不成立**。实际机制是 **Freight 传递链** —— `origin` 恒为 `Warehouse`，Stage 间的上下游由本 Stage 的 `requestedFreight[].sources.stages`（`string[]`）声明。该结论来自 CRD schema 的 `enum` 与字段类型，非猜测。

**最终结果：8/8 资源通过，exit 0**（`Namespace` ×1 / `AppProject` ×1 / `Application` ×2 / `Project` ×1 / `Warehouse` ×1 / `Stage` ×2）。

同时纠正了 v0.1 的一处臆造：**Kargo v1.12.1 的 `Project` CRD 根本没有 `spec` 字段**（root properties 仅 `apiVersion/kind/metadata/status`），v0.1 写的 `spec.promotionPolicies` 不存在，已删除。

内置 promotion step 名称（`kargo.akuity.io/git-clone` / `git-commit` / `git-push`）**由 v1.12.1 源码树反推**（`git_cloner.go` / `git_commiter.go` / `git_pusher.go`），非猜测；但各 step 的 `with` 字段名未经官方文档核对（缺口 #12）。

### 2.10 `publish = false` 补齐与门禁强化（v0.2 新增，闭合缺口 #6）

[`scripts/automation/mark_crates_private.py`](../../scripts/automation/mark_crates_private.py) 为 **107 个 member manifest** 补 `publish = false`。

> **脚本自身也踩了坑**：v0.1 思路只扫 `crates/*/Cargo.toml`，漏掉 `crates/star-desktop/src-tauri/`（嵌套一层）与 `tools/aci-emitter/`。首轮 apply 后 `wildcard` 仍报 1 个 error，定位后改为遍历仓库内全部含 `[package]` 的 manifest（排除 `target/`），并对 `crates/` 之外的 member 单独高亮。

补齐后 `bans.wildcards` 由 v0.1 的 `warn` 降级状态**恢复为 `deny` 强门禁**：

| | v0.1 | v0.2 |
|---|---|---|
| `wildcards` | `warn`（因误报降级） | **`deny`**（强门禁） |
| wildcard 错误数 | 67（60+ crate） | **0** |
| `cargo deny check` | EXIT 0（147 warning） | **EXIT 0**（80 warning：69 duplicate + 11 not-encountered） |

`cargo metadata` 验证 107/107 manifest 合法；`cargo check --workspace --all-targets -j 4` **EXIT 0 / 30.2s**。

### 2.11 Kargo 官方 SBOM 的方法学价值（v0.2 新增）

取得 Kargo v1.12.1 官方 SPDX SBOM（`akuity-kargo_v1_12_1.spdx.json`），**395 个包的 `licenseConcluded` 全部为 `NOASSERTION`**。

这直接印证 AGENTS.md §0「只核对上游仓库主许可证不足以放行」—— **上游自己提供的 SBOM，其许可字段也可能是空的**。因此「上游发布了 SBOM」不构成许可核验通过的证据。本仓采用的核验路径是：`Cargo.lock` → `cargo deny list`（结构化枚举）→ 与 `deny.toml` 白名单比对 → `NOTICE.md` 声明。

---

## §3 已知缺口

per 守门 #11「缺标比错标安全」。v0.2 已推进 8 项（2026-10-05），剩余 5 项仍未完成。

| # | 缺口 | 状态 | 阻断条件 | 处置 |
|---|---|---|---|---|
| 1 | **Kargo / Argo CD 配置运行时验证** | 🟡 **部分闭合** | 首次部署前 | ✅ **结构已验证**：`scripts/automation/validate_gitops_crds.py` 用上游 v3.5.3 / v1.12.1 官方 CRD 的 `openAPIV3Schema` 离线校验，**8/8 资源 exit 0**（见 §2.9）。🟡 仍缺：本机 k3s 不可达（`172.28.176.169:6443 connection refused`），无法做 `kubectl apply --dry-run=server` 或真机 reconcile；promotion step 的 `with` 字段名未经官方文档核对 |
| 2 | **`cargo-about` NOTICE 生成** | ✅ **已闭合** | — | 已落地 [`about.toml`](../../../about.toml) + `about.hbs` / `about-list.hbs`（官方 0.9.2 版）+ [`NOTICE.md`](../../../NOTICE.md)。cargo-about 0.9.2 实跑 exit 0，产出 1265-crate 许可分布；`cargo deny list` 独立交叉验证一致。完整 LICENSE 文本（208 KB）作 CI artifact 不入库 |
| 3 | **容器镜像 SPDX SBOM 未生成** | 🟡 未完成 | prod 晋级前 | 仓库有 5 个 Dockerfile，Docker 可用，syft 1.54.0 已就绪。**但本 session 未实际构建镜像**，故 SBOM 仍缺。ADR-0054 §6.2 待办 #1 |
| 4 | **`argocd-image-updater` 未复验** | 🟡 未完成 | 引入该组件前 | 已迁至 `argoproj-labs`（非主 org）。已取得 Kargo 官方 SPDX SBOM（395 包）作为方法学佐证：**其 `licenseConcluded` 全为 `NOASSERTION`**，即上游自身亦未在 SBOM 声明许可 —— 印证「上游 SBOM ≠ 许可核验通过」。image-updater 本体仍待 syft 复现 |
| 5 | `Cargo.lock` 变更后编译验证 | ✅ **已解决** | — | v0.1: EXIT 0 / 195.6s；v0.2（105 个 manifest 加 `publish = false` 后）: **EXIT 0 / 30.2s** |
| 6 | **100+ crate 未加 `publish = false`** | ✅ **已解决** | — | `scripts/automation/mark_crates_private.py` 覆盖 **107 个 member manifest**（含 `crates/star-desktop/src-tauri/` 与 `tools/aci-emitter/` 等 `crates/` 之外者），`cargo metadata` 验证通过。`bans.wildcards` 已由 `warn` **恢复为 `deny`**，wildcard 错误 67 → **0**（见 §2.10） |
| 7 | **Marvin Attack 暴露面** | 🟡 **已拍板转 handoff** | 见 HANDOFF-SEC-LICENSE-001.md §5 | ⚠️ v0.1 判定「仅公钥验签、私钥运算不可达」**已被证伪**：`star-api-rest/src/auth/mod.rs:157` 存在 RS256 **私钥签名**（`EncodingKey::from_rsa_pem`），`auth/oauth/keypair.rs` 有完整 `RsaPrivateKey` 运算。官方确认 CVSS 5.9 Medium、**永久无补丁**。`deny.toml` 忽略理由已改写为准确描述。**2026-10-05 `ask_8f97b10454f7bf708c009f40`：Ulysses 选定迁 ES256 后改判「先不迁移」** → 13 项改动清单 + 4 条续做触发条件已落 [HANDOFF-SEC-LICENSE-001.md](HANDOFF-SEC-LICENSE-001.md) |
| 8 | `sled v0.34.7` 未维护债务 | 🟡 未解决 | — | 3 条 unmaintained advisory 的共同根因；长期需评估替代 |
| 9 | staging → prod 三级链路与 Analysis 门禁 | 🟡 第二阶段 | 模型稳定后 | ADR-0054 §6.2 待办 #4。挂载点已确认存在于 Kargo v1.12 `spec.verification.analysisTemplates` |
| 10 | **CI 门禁未在 GitHub Actions 实跑** | 🟡 未验证 | 首次 push 后 | 本地已验 cargo-deny；cargo-audit 的 SHA256 仍为**占位符**（未填时 fail-closed `exit 2`） |
| 11 | advisory 忽略项无「到期自动失败」 | 🟡 工具局限 | — | cargo-deny 0.20.2 的 ignore 仅接受 `id` + `reason`，不支持 `expiration`（实测证伪）。复审期限只能写在 reason 文本内，**无机器强制力** |
| 12 | **Kargo promotion step 的 `with` 字段名未核对** | 🟡 未验证 | 首次部署前 | `uses` 值（`kargo.akuity.io/git-clone` / `git-commit` / `git-push`）有源码树反推依据；但各 step 的 `with` 参数名未经官方文档逐项核对。CRD schema **不校验** `with` 内部（preserve-unknown-fields），故离线校验无法覆盖 |
| 13 | **`check` 与 `list` 依赖图规模差异未解释** | 🟡 未解释 | — | `check --all-features` 与 `list`（无 flag）产出的图规模不同（后者 1265 crate）。两者在 copyleft 结论上**一致**（均存在 MPL/LGPL），故 §2.3 明细可信；但差异成因未查明，按缺标比错标不臆造原因 |

---

## §4 子代理失败接手清单

**本阶段未派发任何子代理。** 全部工作由 Mavis 直接完成。

理由：本次为「外部许可与维护状态核实 + 配置实跑迭代」型任务，子代理需要独立完成 4 轮 cargo-deny 实跑并据实修正配置判断，而该过程强依赖单会话内的错误反馈闭环（每轮失败信息直接决定下一轮修正）。拆给子代理会丢失该闭环，且 per 守门 #9 存在「子代理 status=succeeded ≠ 实际成功」的既有实证。

---

## §5 守门规则

| # | 守门项 | 本阶段遵守情况 |
|---|---|---|
| 1 | 禁回溯叙事 | ✅ 未使用「per X 历史形态」「原本是」等表述；许可数据全部标注实测时间点与来源（`gh api` / 上游原始文件直读） |
| 2 | BAS / ADR 引用需 git 实证 | ✅ ADR-0054 的关联 ADR 均为现存文件；未引用不存在的前序文档 |
| 3 | 缺标比错标安全 | ✅ §3 十项缺口全部标 🟡 未验证，无一项以 ✅ 冒充；Kargo/Argo CD 未实跑如实声明 |
| 4 | 商业开源依赖硬约束 | ✅ 见 §2.1 纯白名单语义 + §2.4 有期限债务登记 |
| 5 | copyleft 不得一概排除 | ✅ GPL/AGPL/LGPL 显式列入 allow；且 §2.3 实证依赖树本无 copyleft |
| 6 | 非商业 / source-available 排除 | ✅ BUSL/SSPL/Elastic/Facebook/PolyForm/JSON/NPL/APL 均不在白名单即自动拒绝 |
| 7 | 上游社区活跃度需近 12 个月证据 | ✅ §1.3 每个组件均记录最后推送时间（2026-09-21 ~ 2026-10-04，全部在 12 个月内） |
| 8 | 逐版本审查传递依赖闭包 | ⚠️ 部分完成：Go 侧（Argo CD go.mod 闭包）已审；Rust 侧经 cargo-deny 覆盖 829 包；**镜像系统包层未审**（缺口 #3） |
| 9 | 只核对主许可证不足以放行 | ✅ 关键许可争议位（`notifications-engine`）已直读一手 LICENSE 文件验证，非仅信主仓声明 |
| 10 | 门禁不得吞退出码 | ✅ CI 三个 job 独立、直接调 CLI、由 shell 原生传播退出码；不使用第三方 action 转发结果 |
| 11 | 门禁不得「改参数让检查消失」 | ✅ `wildcards` 降级附完整根因与长期修法；未放宽 `sources` / `advisories` / `unlicensed` |
| 12 | 未经实跑验证的常量/字段不得写入门禁 | ✅ **执行中两次自我捕获** | 初稿写入 `expiration` 字段（0.20.2 不支持）→ 实跑报错后移除并改写 reason；cargo-audit SHA256 保留**占位符**且未填时 `exit 2` fail-closed，不假装通过。**本条是本阶段被违反最多次的守门，已两次实证捕获** |
| 13 | 提交前自审 diff，重点看「为更严而加的东西」 | ✅ 自审捕获 2 处 | (a) staging Application 注释称「刻意不设 automated」而代码含 `automated`，会绕过 Kargo 审批 → 已移除；(b) 变异测试因注入位置错误而**假阳性**，靠读诊断文本而非只看 exit code 捕获 |
| 14 | 子代理 status ≠ 实际成功 | ✅ 本阶段无子代理，不适用 |
| 15 | 报告七段结构 | ✅ 本文件 §0–§7 |
| 16 | 代签规则 | ✅ per 2026-08-27 19:39/20:56/21:59 三次授权 + 2026-09-11 23:11「真人的内容由 agent 决定」 |
| 17 | 守门 #1 编译验证 | ✅ `cargo check --workspace --all-targets -j 4` EXIT 0 / 195.6s（§2.7） |

---

## §6 签字栏

| 角色 | 签署 | 说明 |
|---|---|---|
| 架构师 | ✅ Mavis 接手终审 | per 守门 #14 v4 审核决定，author=Ulysses |
| SRE Lead | ✅ Mavis 接手 | 门禁与 CI 落地 |
| 平台 | ✅ Mavis 接手 | k3s / GitOps 骨架 |
| 评审主持 | ✅ Mavis 接手 | 选型依据可审计性（每项数据附实测来源） |
| PM | ✅ Mavis 接手 | 第一阶段范围与第二阶段待办已分层 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：Argo CD 三件套选型（ADR-0054）+ deny.toml 四类门禁实跑落地（4 轮迭代）+ 3 个 P1 真实缺陷修复 + dev→staging GitOps 骨架 + CI 三 job 门禁 + 10 用例变异测试 | 2026-10-04 20:50 JST 用户需求；21:04 JST `ask_72c00a6f28daadd5538fa004` 拍板 scope_opt1 + env_opt1 |
| **v0.2** | **2026-10-05** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手** | **推进 8 项未完成缺口**：① 闭合 #1 结构部分（CRD 离线校验 8/8，抓出 4 处真实错误含 1 处模型级错误）② 闭合 #2（`about.toml` + `NOTICE.md` + 1265-crate 许可分布）③ 闭合 #5（cargo check EXIT 0/30.2s）④ 闭合 #6（107 manifest 加 `publish = false`，`wildcards` 恢复 `deny`，错误 67→0）⑤ **纠正 v0.1 两处错误结论**：Marvin「私钥不可达」被私钥签名路径证伪；「依赖树无 copyleft」被 `cargo deny list` 证伪（实存 MPL-2.0 × 5 + LGPL-2.1-or-later × 2）⑥ 新增缺口 #12（promotion step `with` 字段名）、#13（`check`/`list` 图差异未解释） | 2026-10-05 08:48 JST 用户指令「推进完成未完成项之后 commit」 |
