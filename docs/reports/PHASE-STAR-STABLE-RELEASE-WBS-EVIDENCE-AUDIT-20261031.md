# PHASE-STAR-STABLE-RELEASE-WBS-EVIDENCE-AUDIT-20261031

> **状态：独立文档证据审计完成；稳定版仍 OPEN / 不可发布。**
> **审计基线：** dev@c2b3e5f07691bc657a3e7f67e65be9d8575d141a。
> **范围：** 只读核对稳定发行 WBS 与 Wave 2 已合入内容；不审查远端环境配置，不重新运行构建/测试，不修改主 WBS。

## §0 目的

独立核对 docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md 中与 Wave 2 相关的发行范围、许可证、BOM/证明、Web/Canvas 交付、Agent View 数据、验证结果及清理状态。审计以本地 dev 在 Wave 2 集成后的基线 c2b3e5f07691bc657a3e7f67e65be9d8575d141a 为准。工作流静态配置或本地检查不等于 hosted build、真实制品、部署、消费者验证或发行证明。

Wave 2 四条 lane 已合并到该基线：Desktop 63ec043576603879452824967549d87ab789ee86（合并 71340b96）、Canvas/Web 18469dd1e370fde9ed91c13f18e6a9d8cff7e7c2（合并 ef2fb5d7）、依赖审计 15e60d601afae5ab6556e1060f073538031fa38e（合并 3526a779）、Agent View 1e3938837ac0413c79de6ff8d72f7604d8a53934（合并 c2b3e5f0）。合并只证明源码进入本地 dev，不证明发行门通过。

## §1 审计矩阵

| 核对事项 | 结论 | 证据与边界 |
|---|---|---|
| 用户批准的发行产品面 | **SUPPORTED** | 用户在本对话确认 Desktop、Web/Vercel、三份 Star Canvas 服务镜像；决策记录在主 WBS v0.2，历史提交 163f715bb68bae1d0605300b705fac03b7dad192。Canvas/Web 报告也按三镜像与 Web build 设定范围：docs/reports/PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md，提交 18469dd1e370fde9ed91c13f18e6a9d8cff7e7c2。 |
| Star 自有项目选 MIT | **SUPPORTED AS USER DECISION** | 用户明确选择 MIT；根 LICENSE:1 为 MIT License。这仅确认项目决策和根文本，不等于整个分发闭包已完成权属/NOTICE 审核。 |
| MIT 元数据已一致 | **STALE / CONTRADICTED** | 当前 Cargo.toml:187 仍为 license = Apache-2.0；deploy/helm/star/Chart.yaml:13 为 MIT；根 LICENSE 为 MIT。Desktop 阶段报告已执行 release metadata gate 并确认 Apache/MIT 不一致会失败：docs/reports/PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md（提交 63ec043576603879452824967549d87ab789ee86）。主 WBS 对根 Cargo 元数据写“仍未验证”已过时；实际已发现明确冲突，W1 仍须 owner 修复并完成全包许可审查。 |
| Desktop SPDX / provenance | **INCOMPLETE** | Desktop workflow 的 YAML/静态条件、22 个完整 Action SHA、npm SPDX 2.3（225 个包）及 validator mutation/control 在本地通过；Rust SBOM 未在本机生成，Hosted CI、实际包、attestation、独立消费者验证未运行。来源：同上 Desktop 阶段报告及 .github/workflows/star-stable-desktop.yml，提交 63ec043576603879452824967549d87ab789ee86；不是实际 Desktop release evidence。 |
| Web/Vercel 证明 | **INCOMPLETE** | Web workflow 静态结构、锁驱动安装、build evidence archive、SPDX 与 attestation subject 绑定检查通过；报告明确没有 hosted run、Vercel deployment、外部 environment reviewer 配置或消费者验证。静态 archive 不是可部署 Vercel 产物或部署记录。来源：docs/reports/PHASE-STABLE-RELEASE-W2-CANVAS-WEB-ATTEST-20261031.md 与 .github/workflows/star-stable-web.yml，提交 18469dd1e370fde9ed91c13f18e6a9d8cff7e7c2。 |
| 三份 Canvas 镜像及真实证明 | **INCOMPLETE** | workflow 的 YAML/Bash/静态 digest 和权限断言通过，并配置 Buildx SBOM/provenance 与 digest-bound attestation；报告明确未产生、发布或独立验证任何镜像，未运行 K3s/consumer 验证，也未核实 GitHub environment reviewer/ref 保护。来源：同上 Canvas/Web 阶段报告、.github/workflows/publish-star-canvas-services.yml、crates/canvas-realtime/Dockerfile、crates/canvas-engine/Dockerfile、crates/domain-canvas/Dockerfile，提交 18469dd1e370fde9ed91c13f18e6a9d8cff7e7c2。真实 OCI/K3s 制品仍无证据。 |
| 依赖 owner PR 审核 | **SUPPORTED AS REVIEW-ONLY; NOT INTEGRATED** | 依赖报告记录 PR #296、#292 均未合入：#296 有锁文件与 CI/许可门失败及 package-manager 决策缺口；#292 把 Rust toolchain 设为日志中不存在的 1.120.0 并收到 404。报告是 2026-10-31 只读快照，提交 15e60d601afae5ab6556e1060f073538031fa38e；没有当前远端刷新，不能推断 PR 今日状态。 |
| Agent View 已摆脱旧游戏机制 | **SUPPORTED WITH NARROW SCOPE** | a3365edd 移除 Agent View 游戏循环和专属模块，1e393883 补充 seed 数据边界；阶段报告称生产路由构建成功并扫描 12 个引用 chunk，未发现被移除的游戏实现标识。证据只覆盖 Agent View 路由/chunks，不覆盖整个 Web payload、镜像或源码仓。来源：docs/reports/PHASE-STABLE-RELEASE-W2-AGENT-VIEW-CORE-20261031.md（1e3938837ac0413c79de6ff8d72f7604d8a53934）。 |
| Agent View 使用生产数据/Agent 执行 | **CONTRADICTED** | 当前 frontend/src/app/(app)/agent-view/page.tsx:85 从 store 读取 agentSessions；frontend/src/lib/store.ts:363 将其初始化为 seed.agentSessions；frontend/src/lib/seed.ts:203 定义演示会话。阶段报告也明确实时授权 provider、凭据存储、runtime、CLI 执行、取消/恢复与独立验证未接入。不能把 UI 或 seed 视为生产 Agent 闭环。 |
| Agent View 定向测试、typecheck/build | **SUPPORTED** | 阶段报告记录 2 个测试文件/27 项通过、typecheck 通过、Next build 52/52 静态页面成功，且 Agent View 路由存在；同时报告 Node 22.19 低于部分锁定依赖 engine 要求 22.22.2，CodeRabbit 未能运行。来源为上项报告，提交 1e3938837ac0413c79de6ff8d72f7604d8a53934。 |
| 全前端 929/947（18 失败） | **INCOMPLETE / NOT TRACEABLE IN REPO** | 本次 Wave 2 阶段报告与主 WBS 均未找到该总数的持久化日志或报告。它不能由 Agent View 的 27 项定向测试替代，也不能作为全前端 suite 通过证据。本审计未重跑该 suite；若主 WBS保留该数量，应同时记录命令、环境、完整摘要和 18 项失败分类。 |
| BFF Rust 检查/88 tests | **INCOMPLETE / COORDINATOR-REPORTED** | 上游协调摘要报告 BFF cargo check --all-targets --all-features -j 4 无编译错误（219 warnings）与 88 tests 通过，但 Wave 2 已提交报告没有保存对应命令输出。本 lane 未重跑，因此保留为协调者运行结果而非独立、可复现的 repo evidence。 |
| Root Rust workspace 检查 | **SOURCE DEFECT SUPPORTED; RUN LOG NOT PERSISTED HERE** | crates/run-owner-runtime/src/schedule_admission.rs:638 在 revalidate_task_binding 中调用 .bind(branch_id)，当前方法只定义 command 和 target，没有局部 branch_id。git diff 4e6336391f1eec8c6425b2eea9f8543a4fce50da..c2b3e5f07691bc657a3e7f67e65be9d8575d141a -- crates/run-owner-runtime/src/schedule_admission.rs 为空，说明 Wave 2 未触碰该源文件。协调者报告 Rust 1.99.1 workspace all-targets 在此失败；本审计未复跑 compiler，目标 Rust 1.99.0 本机不可用的结果也没有在本 lane 内复现。W2/RC 不能标通过。 |
| 发布、tag、push、部署 | **SUPPORTED FOR THIS WAVE, WITH REMOTE LIMIT** | 四份 Wave 2 phase report 均明确未创建新 tag、发布/部署或真实制品；本地已有历史 tag v0.79.0–v1.0.5，但没有 tag 指向被审计 HEAD；本地 dev 状态为 ahead 210。没有 refresh remote，也未查阅 GitHub release/deployment 页面，所以本结论仅限 Wave 2 与当前本地 checkout，不能证明远端不存在其他发行活动。 |
| Wave 2 lane 清理 | **INCOMPLETE; NOT CLEANED** | Git 分支/工作树只读清单仍列出四个源分支及工作树：agent/stable-release-wave2/desktop-attest、canvas-web-attest、dependency-audit、agent-view-core 与 D:/orcaWork/Star/agent-stable-release-wave2-*；四个 worktree 当前 Git status 干净。集成分支也仍存在。干净不等于 owner 已结束；Orca 运行时状态不能确认，所以不得称已清理或删除。 |

## §2 验证摘要

- 审计基线 Git HEAD 为 c2b3e5f07691bc657a3e7f67e65be9d8575d141a，正处于 Wave 2 四个 lane 的合并 DAG 顶端。合并依次可见于 71340b96、ef2fb5d7、3526a779、c2b3e5f0。
- 阅读四份 Wave 2 阶段报告、当前稳定版 WBS、Desktop/Web/Canvas workflows、root Cargo metadata/LICENSE、Agent View route/store/seed 源码；读取四个 Wave 2 源分支与 worktree 的 Git 状态。
- rtk git diff --check：通过（本审计 lane 写入前）；审计报告只依据上述文档/源码静态证据，不运行构建、测试、hosted CI 或部署。
- dev 本地状态显示 ahead 210；remote-tracking 状态没有刷新。没有 tag 指向被审计 HEAD；不能把本地状态扩展成远端“绝无发布”结论。
- 当前主 WBS 的基线仍写 dev@f34866b8，文件最新历史停在 4e633639；Wave 2 四个 lane、验证结果及其仍存在的工作树尚未纳入 WBS。

## §3 已知缺口

1. Desktop/Web/Canvas 工作流没有 GitHub hosted run、真实构建与签名/attestation 验证者输出；Canvas 没有真实 OCI digest 和 Multipass/K3s 部署验证，Web 没有真实 Vercel deployment 与消费侧/rollback 验证。
2. MIT 项目决定已清楚，但 Cargo.toml 仍宣称 Apache-2.0；第三方来源、完整发布依赖闭包、NOTICE、专利/再分发义务与各 artifact payload 清单未闭合。
3. 所有 929/947 和 BFF 88-test 总数都缺少本仓可追溯运行日志；需要记录完整 command、toolchain/runtime、exit code、summary 与失败分类。Rust 1.99.0 精确基线没有验证。
4. Root workspace branch_id source defect 尚未修复；失败属于当前基线阻塞，不是 Wave 2 改动。
5. Agent View 虽已移除旧游戏循环，但页面仍基于 seed store；生产授权范围 provider、真实空状态、Agent runtime/CLI 与独立验证未完成。
6. Wave 2 源 worktree 干净但仍存在；Orca owner 无法核实，清理必须等 owner 可验证后进行。
7. Web 与 Canvas evidence workflows 的 GitHub environment reviewers、allowed refs 与 branch protection 实际设置未核实；配置中出现 environment 名称不证明保护规则生效。
8. WBS 全局快照/lane 状态需要更新至 c2b3e5f，并记录静态与实际执行证据的区分；W0–W9 及 stable release 均必须保持 OPEN/BLOCKED。

## §4 子代理失败接手清单

- 本审计 lane 未再派生子代理；无失败接手项。
- 本报告是独立核对结果，不替代 WBS owner 的修改与父级最终审查。

## §5 守门规则

1. 只写本 lane 登记的 brief、阶段报告和 dispatcher JSONL。
2. 不修改主 WBS、Cargo/lock/workflow、PR、main、remote 或其他 worktree。
3. 不将静态 workflow checks、seed UI、本地局部测试或配置声明升级为实际发行证明。
4. MIT 项目选择与 Cargo metadata 冲突继续阻止 W1/attestation。
5. 任何必要 artifact、hosted run、消费验证、部署、回滚或权属证据缺失时，release 保持 OPEN。
6. 四个 Wave 2 源分支/worktree 在 Git inventory 仍存在；owner 未验证前不声称清理，不删除。
7. 不创建 tag/release，不发布 artifact，不部署，不 push，不操作 PR。
8. 审计结论限定于本地 dev@c2b3e5f 和本报告列明的资料；未刷新远端。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 数据与执行边界按 source evidence 标记 |
| SRE Lead | 架构师（Mavis 接手审核）— hosted/platform 证据缺失，发行保持 OPEN |
| 平台 | 架构师（Mavis 接手审核）— 本次只读静态核对，未宣称发布验证 |
| 评审主持 | 架构师（Mavis 接手审核）— 主 WBS 同步待父级合并审阅 |
| PM | 架构师（Mavis 接手审核）— 用户批准的发行范围/MIT 已确认，stable release 不可发布 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于 dev@c2b3e5f 独立核对发行 WBS 与 Wave 2 四条 lane；记录 MIT 元数据冲突、静态与真实证明边界、Agent View seed 数据、测试日志缺口、root Rust 源缺陷和未清理工作树。所有发行门保持 OPEN/BLOCKED。 | Wave 2 合入本地 dev 后的 WBS 独立审计 |
