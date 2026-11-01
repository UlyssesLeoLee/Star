# PHASE — Stable Release W0 Version and Owner Audit

**阶段状态：** 有界证据审计完成；W0 仍 PARTIAL，stable release 仍 OPEN / 不可发布。
**审计基线：** `dev@2a57fa0d329be830735d6657677236e195673362`。
**范围：** 仅核对版本声明、现存本地标签和三类 stable workflow；未修改产品元数据或发行流程。

## §0 目的

核对用户批准的 Desktop、Web/Vercel、三个 Star Canvas 镜像目前如何声明和选择版本，列出 W0 关闭前的具体决策与验收条件。此审计不将历史 Git tag 当作本次发行，不创建 tag/release、构建、发布、部署或 push。

## §1 改动矩阵

| 文件 | 改动 | 结果 |
|---|---|---|
| `docs/briefs/STAR-STABLE-W0-VERSION-OWNER-AUDIT-20261031.md` | 新增只读审计范围与验收条件 | 仅声明 W0 边界 |
| 本阶段报告 | 新增实际版本和 workflow 证据、待决项与缺口 | W0 保持 PARTIAL |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W0-VERSION-OWNER-AUDIT-20261031.jsonl` | 记录 brief 和审计完成事件 | 单行 JSONL 事件可解析 |

## §2 验证摘要

| 检查 | 结果 | 证据与限制 |
|---|---|---|
| 工作基线 | PASS | 审计从 clean 的 `dev@2a57fa0d329be830735d6657677236e195673362` worktree 开始。 |
| 根 Rust workspace | OBSERVED | `Cargo.toml:182-187` 声明 workspace package version `0.1.0`、MSRV `1.99.0`、license `Apache-2.0`。license 与用户确定的 MIT 不一致，属于 W1 阻塞。 |
| BFF 与前端 | OBSERVED | `bff/Cargo.toml:24-28` 为 `0.1.0`；`frontend/package.json:2-3` 与 `crates/star-desktop/frontend/package.json:2-3` 均为 `0.1.0`。 |
| Desktop | OBSERVED | `crates/star-desktop/src-tauri/tauri.conf.json:3-4` 的产品名为 `star-desktop`、版本为 `0.1.0`；Desktop Cargo package 继承根 workspace 版本。 |
| Canvas 三服务 | OBSERVED | `canvas-engine`、`domain-canvas`、`canvas-realtime` 的 `Cargo.toml` 均使用 `version.workspace = true`，因此继承 `0.1.0`。 |
| 本地 Git 标签 | PASS（仅本地 refs） | 当前 tag 列表最高 release-style 标签为 `v1.0.5`；其对象说明日期为 2026-09-22。该标签是本次 audit baseline 的祖先。此检查不查询远端 Releases，也不证明 tag 对应的制品仍可下载或可安装。 |
| Desktop workflow | OBSERVED | `.github/workflows/star-stable-desktop.yml:52` 开始 manual dispatch；attest job 条件（335、371、424、469）要求手动触发、`APPROVED` 输入和 `refs/tags/v*`。定向检查确认该 workflow 未展示 Git tag 与 Tauri/Cargo package version 的一致性断言。 |
| Web/Vercel workflow | OBSERVED | `.github/workflows/star-stable-web.yml:6-10,40-47,142-143` 只允许经批准的版本 tag 生成 evidence bundle，并明确说明该 bundle 不是 Vercel deployment。真正 Vercel deployment/version 与 tag 的关联尚无本审计证据。 |
| Canvas workflow | OBSERVED | `.github/workflows/publish-star-canvas-services.yml:12-15,46-48,262-278` 接受 `v*` tag，但 publish 仍需人工 dispatch 与 `APPROVED`；OCI tags 同时含长 commit SHA 与由 Git tag 派生的 SemVer。输出 digest 是部署引用应使用的不可变身份。 |
| Active Canvas deployment manifest | OBSERVED | `deploy/star-canvas-services-k3s.yaml:49,118,187` 目前仍是三个必替换 digest 占位符；Helm `version/appVersion: 0.1.0`（`deploy/helm/star/Chart.yaml:5-6`）不是已确认的 active release source。 |
| GitHub Releases/远端状态 | NOT VERIFIED | 只检查了本地 tag refs；本次没有查询 GitHub Releases API、registry、Vercel 或 hosted workflow run。 |
| 文档/静态检查 | PASS | `git diff --check`、JSONL parse 与 staged diff 均在提交前执行。构建和产品测试按该只读审计范围未运行。 |

### 版本事实与解释

当前各应用/package 的静态版本集中为 `0.1.0`，而已存在的本地 Git release-style 标签已经到 `v1.0.5`。Canvas publish workflow 把 `v*` tag 转成容器 SemVer 标签；Desktop workflow 用 tag 作 gate，但审计范围内没有发现把该 tag 同步或比对到 Tauri/Cargo/JS 的检查。Web workflow 产出带证明的构建证据包，且明确不等同于 Vercel 部署。由此可确认目前没有单一、机器校验的版本映射闭环；不能据此推定正确的下一版本号。

## §3 已知缺口

- **需要用户决定的两个 W0 项：**
  1. 选定下一次稳定版的 canonical SemVer/tag，并明确现存 `v1.0.5` 是否作为该次版本序列的起点；本审计不擅自推导 `v1.0.6` 或任何其他编号。
  2. 指定本次稳定发行的 accountable release owner，负责冻结 source commit、创建受保护 tag、核对用户批准的发行面及支持制品的版本映射并批准最终候选。旧 tag 的 tagger/历史说明不能自动成为当前 owner 决定。
- W0 在 owner/版本决策后仍需落档：版本来源（Git tag 或显式 release manifest）、应用包映射规则、各平台部署/消费版本与不可变 artifact identity。对未采用统一 package version 的组件，须把差异写成明确政策并自动验证。
- 对已确认的产品面，至少要提供 fail-closed consistency check：canonical tag 与冻结 commit 对齐；Desktop Tauri/Cargo、Web 构建证据和三个 Canvas image 的版本/commit 元数据符合批准映射；attestation/SBOM 绑定同一 source commit；Canvas 部署 manifest 用构建产出的 digest；Vercel 需另有真实部署 ID/消费者验证。
- 本次没有查询 GitHub Releases、GHCR、Vercel、远端 tag protection 或 environment reviewers；不得把本地 ref 状态升级成远端发布证据。
- W1 仍被根 Cargo workspace license `Apache-2.0` 与用户选择的 MIT 冲突阻塞；该问题不由 W0 审计解决。

## §3.1 用户决策与当前状态更新（2026-11-01）

本节更新上述历史审计之后已确认的决策，不改写 §2 对 `dev@2a57fa0d329be830735d6657677236e195673362` 的观察结果：

- 用户已确认 Git tag 为 Star 产品发行版本的唯一来源，目标稳定版为 `v1.0.6`，规范版本字符串为 `1.0.6`。
- 用户已指定 Codex/assistant 为 accountable release owner，并条件授权在全部必需发行门禁通过且最终复核完成后发布一个稳定版本。
- 因此 §3 的两个用户决策项已解决；版本映射、冻结 source commit、tag 保护/不可变性、hosted build、Vercel deployment、Canvas digest/attestation 与 consumer/rollback 证据仍需验证。W0 仍为 PARTIAL，stable release 仍 NO-GO。
- 后续集成的 [W0 版本映射报告](PHASE-STAR-STABLE-W0-VERSION-MAP-20261101.md) 与 [W0 集成同步报告](PHASE-STAR-STABLE-W0-INTEGRATION-SYNC-20261101.md) 已记录当前 dev 的版本 resolver 和 Desktop、Web、Canvas 接线；它们取代本报告中对旧 workflow 状态的当前性判断。本报告只作为 `dev@2a57fa0d` 的历史证据，不代表当前实现状态。

## §4 子代理失败接手清单

本 lane 为单个只读审计 worker；没有继续派生子代理，也没有子代理失败。没有产品代码、版本元数据或 workflow 实施任务交接。本阶段报告和结论需要协调者在纳入 WBS 前独立复核。

## §5 守门规则

- 本报告仅使用本地 `dev@2a57fa0d` 及该 checkout 可见的 tag/ref；没有推断 GitHub release、下载、镜像发布或部署事实。
- `v1.0.5` 是历史 tag，且早于当前 dev；它不是本轮新版本或候选批准。
- 所有测试/构建/发布门仍按稳定版 WBS 原状态保持 OPEN/BLOCKED；W0 仍 PARTIAL。
- 只修改了本 lane 登记的 brief、report、dispatch 三个文件；不改 `D:\\Star`、`main`、产品元数据、活动 D2/W8 worktree 或其他 lane。
- 未创建 tag/release、未构建或发布 artifact、未部署、未 push。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）：本次 W0 证据边界和未决项审阅通过；不代表版本选择或 stable release 批准。 |
| SRE Lead | 本阶段未运行平台、容量或恢复验收；对应 WBS 门保持 OPEN。 |
| 平台 | 本阶段未构建、签名、部署或验证消费者；对应 WBS 门保持 OPEN。 |
| 评审主持 | 审计范围内证据与结论一致；远端发布状态明确标记为未核实。 |
| PM | 用户已确认发行面、唯一 tag 版本源 `v1.0.6`、Codex/assistant accountable release owner 与条件发行授权；这些决策不等于 W0 技术验收完成。 |

**稳定版审批：** 未申请；此阶段文档审阅不构成 release 批准。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust/JS/Desktop/Canvas 当前声明、tag 与 workflow 关联、W0 的用户决策和可自动验证的关闭条件；不改变产品或发布状态。 | Stable release WBS 下一项 W0 version/owner audit |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补记用户已确认的 `v1.0.6` 唯一 tag 版本源、Codex/assistant release owner 与条件授权，并将后续 W0 版本映射报告标为当前 dev 依据；明确本报告仅为历史基线证据。W0 与稳定版门禁保持开放。 | 与当前 W0 文档和用户决策复核时发现原决策状态已过时 |
