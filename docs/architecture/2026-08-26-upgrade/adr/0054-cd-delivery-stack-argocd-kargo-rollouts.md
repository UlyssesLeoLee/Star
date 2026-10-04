# ADR-0054: 自动部署体系选型 — Argo CD + Kargo + Argo Rollouts

> **状态**：✅ Accepted v0.1 (per 2026-10-04 21:04 JST `ask_72c00a6f28daadd5538fa004` scope_opt1 拍板, 守门 #10 + #14 v4 Mavis 审核)
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
