# Rust 1.99.1 候选分支审查 — WBS-MERGE-RUST-BASELINE-CANDIDATE-20261108

**审查日期：** 2026-11-08
**来源：** `codex/star-rust-1991-baseline-20261105@be154efb6cff5dd6eea6c85f5c5a5ccc00d8ccd6`
**目标：** `dev@6ad1a00f746234d5e296f942cc67f20f917d1609`
**结论：** **HOLD 两个来源提交；不整分支合并，不 cherry-pick。** `c640eb34` 的运行时基线已由当前 dev 后续提交表示；`be154efb` 中少量版本文字可供新文档 lane 精确适配，但不能用其旧文件快照覆盖 dev。

## §0 目的与边界

核对候选分支两个独有提交及其每个登记路径，判断相对当前 dev 的依赖闭包、重复内容、残余价值和验证限制。审查只读：未修改、重置、rebase、cherry-pick、合并、清理或删除候选及其他 worktree；未运行测试、Cargo、容器、Multipass/K3s、hosted CI 或发布任务。

产品约束按当前基线审阅：Rust 1.99.1；Multipass 管理本地/CI Linux guest VM，K3s 在 guest 中运行，不以 Docker Desktop 为前置；容器和构建输入只允许获批 Star 产品内容，游戏/旧载荷不得混入；未知状态不得写成生产或发布通过。

## §1 分支、提交与路径矩阵

### 1.1 Git 事实与依赖关系

| 证据 | 观察 |
|---|---|
| 基线 | `dev` 与本审查 worktree 均为 `6ad1a00f746234d5e296f942cc67f20f917d1609`；候选 merge-base 为 `de29ef04e13ab16dc30842c885207c4c6f602774`。 |
| 分叉 | `git rev-list --left-right --count dev...candidate` 为 `102 2`：候选落后 102 个提交、领先 2 个。 |
| 候选独有提交 | `c640eb3428c7601f8c9ccce5394028b9e504dade`（34 paths，+197/−66）；`be154efb6cff5dd6eea6c85f5c5a5ccc00d8ccd6`（16 paths，+37/−35）。 |
| Patch 等价性 | `git cherry -v dev candidate` 对两提交均为 `+`；提交本身没有被 patch-equivalent 标记，但其路径/内容已有后续 dev 实现或需要局部适配。 |
| 候选状态 | 候选工作树只读状态为 clean。此事实不授权清理或移动它。 |

### 1.2 提交 `c640eb34` — “Align Star with Rust 1.99.1” (34 paths)

| 候选路径 | 当前 dev 表示 / 依赖 | 建议 |
|---|---|---|
| `.github/workflows/ci.yml`; `.github/workflows/coverage.yml`; `.github/workflows/license-gate.yml`; `.github/workflows/p1e-integration.yml`; `.github/workflows/publish-star-canvas-services.yml`; `.github/workflows/spawn-e2e.yml`; `.github/workflows/star-desktop-build.yml`; `.github/workflows/star-stable-desktop.yml`; `.github/workflows/wasm-build.yml` | 当前 workflow/toolchain pins 已在 1.99.1 基线；所列路径在 dev 与候选相同。当前 dev 对相关 CI 内容另有后续提交。 | **HOLD：** 重复；不摘取。 |
| `Cargo.toml`; `bff/Cargo.toml`; `crates/platform-messaging-proto/Cargo.toml`; `rust-toolchain.toml` | 当前 `Cargo.toml:185`、BFF/proto manifest 和 `rust-toolchain.toml:4` 均设为 1.99.1；路径内容与候选相同。 | **HOLD：** 重复；不摘取。 |
| `crates/canvas-engine/Dockerfile`; `crates/canvas-realtime/Dockerfile`; `crates/domain-canvas/Dockerfile`; `deploy/k3s-local/Dockerfile` | Dockerfile 路径与候选相同，基础版本已是 Rust 1.99.1。当前 k3s-local 文件是带 `@star-local-crates-begin/end` 标记的模板，由 Star Cargo 闭包生成器选择显式 crate 输入；不能把模板检查当成真实 build context 或最终镜像验收。 | **HOLD：** 重复；图像、context、SBOM/provenance 仍未由本审查验证。 |
| `AGENTS.md` | 当前 dev 已把最低版本设为 1.99.1，并包含容器 digest 与历史证据等更完整边界；候选的改写较旧。 | **HOLD：** 保留当前版本，不用候选两行覆盖。 |
| `docs/automation-design.md`; `docs/basic-design.md`; `docs/design/DD-WORKTREE-GROUP-001.md`; `docs/requirements.md` | 候选增加/更新 1.99.1 基线文字，但文件已随当前 dev 后续工作演进。当前 requirements 的 AC-INFRA-002-R/S/U/V/W、basic-design 的对应 §16 门及 Group DD 的 U/V 仍残留活动验收文字 `1.99.0`；当前 Rust 实施报告也记录了文档同步缺口。候选 `c640` 未完整修正这些门。 | **ADAPT（新文档 lane）：** 仅校准活动 MSRV/未来门；保留历史 1.99.0 as-run 结果。不得 cherry-pick 旧整文件。 |
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` | 当前 dev 的稳定版 WBS 已追加后续 W0/W4/W5/W6/W7/W8 证据和状态，仍为 v1.0.6 NO-GO。候选较旧快照不能涵盖当前门禁。 | **HOLD：** 当前 WBS 作为权威，拒绝旧版覆盖。 |
| `docs/briefs/STAR-RUST-1991-BASELINE-20261105.md`; `docs/reports/PHASE-STAR-RUST-1991-BASELINE-20261105.md` | 这是候选独有的旧 brief/report。dev 已有 2026-11-06 的 `PHASE-STAR-RUST-1991-IMPLEMENTATION-20261106.md`、自审/独立审查与 WBS 同步。候选报告的阶段测试记录属于报告作者当时的陈述，本审查没有重跑。 | **HOLD：** 不引入旧阶段快照；如需历史留档，先由协调者确认其不重复、不误导。 |
| `scripts/automation/phase9f2_binding_http_acceptance.py`; `scripts/automation/test_star_k3s_context.py`; `scripts/automation/test_w16a_device_revocation_http_pg_acceptance.py`; `scripts/automation/w16a_canvas_sse_pg_acceptance.py`; `scripts/automation/w16a_device_revocation_http_pg_acceptance.py`; `scripts/automation/w16a_oauth_pg_acceptance.py`; `scripts/release/tests/test_star_canvas_context_policy.py`; `tools/scaffold_skeleton.py` | 当前 dev 的 `fac360a9` 已单独更新这组 Rust 1.99.1 harness、fixture、context-policy tests 与 scaffold；候选文件是早期版本，不能替代其当前内容。 | **HOLD：** 不回退到候选脚本。当前阶段报告记录 111 passed/1 skipped；本审查未独立运行这些测试。 |
| `scripts/automation/registry.md` | 与上列脚本和 1.99.1 状态相关，但候选后续 `be154` 又改过此文件；当前 registry 已有 Rust 1.99.1 基线提示，另有旧 runner 行需逐条校准。 | **ADAPT（新文档 lane）：** 见 §1.3；保留 as-run 历史。 |

### 1.3 提交 `be154efb` — “Sync active docs to Rust 1.99.1 baseline” (16 paths)

此提交的核心是把活动工具链/验收文本从 1.99.0 改为 1.99.1。它不包含代码依赖，适合逐句同步，不适合作为 16 路径整体提交。下面逐项记录：

| 候选路径 | 当前 dev 中的状态 | 建议 |
|---|---|---|
| `docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md` | VCD-2 的活动目标仍引用 1.99.0；Rust 执行未通过 VCD-2 前仍关闭。 | **ADAPT：** 仅更新活动基线句，保持隔离账号、实际构建/RSS/回滚门和关闭状态。 |
| `docs/architecture/2026-09-30-upgrade/rust-wasm-build-integration-guide.md` | 系统要求仍写 1.99.0。 | **ADAPT：** 只改当前工具链要求，不碰历史命令/报告。 |
| `docs/automation-design.md` | 有 1.99.1 已通过的 current-run 证据，但一些活动 gate/default 文句仍写 1.99.0；提交中也有执行记录周边改动。 | **ADAPT：** 只更新工具链要求、未来 gate 与 runner 默认；历史执行结果/失败原文保留为历史。 |
| `docs/basic-design/vercel/BD-STAR-VERCEL-DEPLOY-001.md` | 当前 §6.2/§6.3 已使用 1.99.1。 | **HOLD：** 已表示。 |
| `docs/design/DD-LOCAL-INFRASTRUCTURE-001.md` | §7.15.3 的活动最低版本门仍写 1.99.0；其余多个 1.99.0 是具体 as-run 结果。 | **ADAPT：** 仅改当前最低版本门，不重写历史测试记录。 |
| `docs/detailed-design/vercel/DD-STAR-VERCEL-DEPLOY-001.md` | 当前 VCD-2 活动验证/开放门仍有 1.99.0。 | **ADAPT：** 只改活动 MSRV 文字，保留 `NOT_RUN` 和其余 VCD-0/2 边界。 |
| `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` | 当前工具链依赖说明仍见 1.99.0。 | **ADAPT：** 更新当前依赖基线；不推导 Canvas/发布门已通过。 |
| `docs/implementation-plans/UI-PLAN-RUN-WORKTREE-NEXT-20261021.md` | UI-7 发布质量与 Rust 桌面验收门仍有 1.99.0。 | **ADAPT：** 更新前瞻门槛，不改变禁用/未验收状态。 |
| `docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md` | 当前 VCD-2 前瞻门已为 1.99.1；其他 1.99.0 多为旧阶段实际结果。 | **HOLD 文件快照：** 只在后续复核发现活动门残留时局部更新；不得把历史 run 版本改写为 1.99.1。 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | 当前 Group WBS 已有更新的版本与 9F4J/W16 状态；早期 1.99.0 出现于历史证据/修订记录。 | **HOLD：** 候选 WBS 快照已落后；不覆盖当前 WBS。 |
| `docs/reports/PHASE-STAR-RUST-1991-BASELINE-20261105.md` | 候选自有阶段报告，dev 当前已有 11-06 实施报告与独立复核记录。 | **HOLD：** 不再导入旧阶段报告；其中的旧工具链测试结果不由本次审查背书。 |
| `docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md` | 当前文件顶端已经声明 Rust 1.99.1；出现的 1.99.0 主要是 2026-10 的本地 as-run 证据。 | **HOLD：** 保留历史证据；若发现活动要求另行逐句审查。 |
| `docs/requirements/SRS-STAR-VERCEL-DEPLOY-001.md` | 仓库工具链、VCD-2、NFR-VC-08 与 GAP-03 仍明确写 1.99.0。 | **ADAPT：** 仅同步活动基线到 1.99.1；GAP-03/实际 Vercel 构建仍 NOT_RUN。 |
| `docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md` | T-028、NFR-VC-08、GAP-03 与 known-gaps 仍写 1.99.0。 | **ADAPT：** 更新测试目标版本，保持 T-028/GAP-03 NOT_RUN。 |
| `scripts/automation/registry.md` | 文件顶端说明新/重跑默认 1.99.1，但个别 runner 行仍描述默认 1.99.0；其历史结果另有 1.99.1 run。 | **ADAPT：** 只校准当前 runner 默认/执行说明；保存原始 as-run 版本和失败记录。 |
| `docs/briefs/STAR-RUST-1991-BASELINE-20261105.md` | 候选修改其自有 brief 的版本文本；dev 已由较新阶段/WBS 说明基线决策。 | **HOLD：** 仅为候选工作记录，不作为当前规范或验证证据。 |

### 1.4 依赖闭包与选择性方案

- `c640eb34` 的 Rust pin/workflow/Docker 子集已经等于当前 dev。其余实现路径由当前 dev 的 `ebdde739`（runtime baseline）和 `fac360a9`（验收脚本/夹具/scaffold）等拆分提交更新；之后 `67e4beb8`、`00dbd300` 又修订未来 Rust gate 与报告。整提交 cherry-pick 会重复或回退 dev 当前内容。
- `be154efb` 的版本文字变更不存在代码依赖，但其中多个文件已有独立更新、部分旧版本是必须保留的历史测试结果。安全摘取单位是“当前文件中的活动要求/未来门句子”，不是整个文件或该 commit。
- **建议：** 后续单独登记当前 dev 的最小文档同步 lane，核对 §1.3 的 ADAPT 路径，并和 current WBS 一起更新；明确保留所有 as-run 1.99.0 结果。特别复核 `docs/requirements.md` AC-INFRA-002-R/S/U/V/W、`docs/basic-design.md` 对应活动 gate、`docs/design/DD-WORKTREE-GROUP-001.md` U/V；候选 `c640` 没有关闭这些缺口。当前 `AGENTS.md` 已为 1.99.1，因此不应沿用旧实施报告中“AGENTS 仍为 1.99.0”的陈述。
- 此建议不授权直接改动规范文件，也不表示来源分支已可删除；本任务只给候选集成结论。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| Git ancestry / 独有提交 / 文件清单 | 已用只读 Git 查询核验：102 behind、2 ahead；两提交均为 `+`；34 与 16 个文件。 |
| 当前 Rust 基线静态核对 | `rust-toolchain.toml`、workspace/BFF/proto manifest、Rust workflows 与 1.99.1 基础镜像值和候选基线相符；当前稳定版 WBS 仍标记 NO-GO。 |
| Multipass / K3s / 容器静态核对 | 当前 DD 明确 Multipass 管理本地/CI Linux guest、guest 中运行 K3s，且没有 Docker Desktop 前置要求。当前 k3s-local Dockerfile 是生成器模板；稳定 WBS 明确真实 filtered context、最终镜像、digest、SBOM/provenance 与 K3s rollout 仍开放。 |
| 测试、编译和运行 | **本审查均未运行。** 候选 11-05 报告及 dev 11-06 报告中的 focused test 数值只作为文件内陈述引用，不是本轮独立验证。 |
| Review whitespace | 将在提交前仅对三个登记文件执行 `rtk git diff --check`。 |

## §3 已知缺口与风险

1. 当前规范文档仍有活动 Rust 1.99.0 gate 文句（见 §1.4），不能把所有残留版本字串都归为历史；需要有范围的文档同步。
2. 候选阶段报告的测试、Rust compiler probe 和 context fixture 记录没有在本轮重新执行；不能扩大为 workspace、hosted CI、目标数据库/RLS、Multipass/K3s、Vercel、镜像或稳定版验收。
3. 当前 Docker 发布 workflow 的静态 allowlist/context 防护不等于实际构建输入和最终镜像已证明无游戏/遗留文件。当前 stable WBS 仍要求实际构建上下文、层、digest、SBOM/provenance 与 rollout 证据。Multipass guest image 属测试/开发基础设施；本审查不把它加入公开 Star 发布 payload。
4. 用户要求产品面避免竞品名称。本候选提交没有改前端或用户可见 UI 文案；这次只读文件范围不足以对完整产品文案作全局结论。
5. stable v1.0.6 仍为 NO-GO；无 tag、release、push 或部署证据。

## §4 子代理/执行失败接手

本审查未派生其他子代理，未运行测试或命令式产品操作。没有执行失败需要接手。Git 只读查询曾因 PowerShell 将短哈希参数误拆而返回无效 revision；未产生任何写操作，随后使用完整提交 ID 完成所需核验。

## §5 守门规则

- 保持候选及所有其他 worktree/branch 原样；本结论不批准删除、归档、改写或合并来源。
- 不整提交 cherry-pick；仅在新登记的隔离 lane 对当前文件中的活动版本句做适配。
- 1.99.0 的历史执行证据必须按真实 as-run 版本保留；禁止把旧结果改写为 1.99.1 成功。
- 只读检查不是验证；没有执行的 Cargo/CI/PG/VM/container/hosted gate 保持未运行/开放。
- Docker 发布制品需以真实过滤后的 context、最终镜像/层和 SBOM/provenance 证明仅含批准的 Star 组件；`.dockerignore`、Dockerfile 模板或静态配置不足以放行。
- Multipass 是本地/CI Linux guest 的首选管理方式，K3s 在 guest 中运行；本地流程不要求 Docker Desktop。最多两个 Multipass inventory 记录的测试上限继续生效。
- 用户面不得出现竞品对标/仿制文案；真实连接器名称例外仅适用于对应连接器信息。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | 审查建议 HOLD 两个候选提交；候选实现不替代当前 Worktree/Run、Multipass/K3s 和 Star-only artifact 边界。 |
| SRE Lead | 未运行任何当前运行/部署 gate；保持 OPEN。 |
| Platform | 当前 Rust 配置基线已由 dev 表示；真实镜像/hosted/K3s 制品门未关闭。 |
| 评审主持 | 仅接受未来隔离文档 lane 的精确适配，不接受整提交覆盖。 |
| PM | v1.0.6 保持 NO-GO；本报告不授权 tag/release/deploy。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.1 候选的拓扑、两提交路径矩阵、dev 表示、可适配文档句、未运行验证和 HOLD 结论。 | WBS 请求逐项审查既存候选 worktree |
