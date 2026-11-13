# PHASE-STAR-STABLE-W0-VERSION-MAP-20261101

> **状态：版本接线已集成；W0 PARTIAL，稳定发行 NO-GO。**
> **当前 dev 基线：** 66610de4ce7ae2c56241e9af06c32d6074c21666。
> **用户决策：** Git tag 是 Star 产品发行版本唯一权威来源；目标稳定版为 v1.0.6，规范版本字符串为 1.0.6；Codex/assistant is accountable release owner。用户已条件授权在全部必需发行门禁通过且最终复核完成后发布一个稳定版本。
> **范围：** 汇总当前 dev 已集成的 W0 resolver、Web、Desktop、Canvas 版本绑定和 W6 候选文档审阅。以下均为代码/文档静态接线与 lane 报告中的定向验证，不代表 hosted run、真实稳定 tag 构建、Vercel 部署、镜像发布或 consumer proof。远端 refs 未刷新；本地没有 v1.0.6 tag。用户的条件授权已给出，但门禁与最终复核条件尚未满足；当前不得执行 release。

## §0 目的

把已接受并存在当前 dev 的 W0 版本绑定与 W6 候选评审结果同步到单一映射记录，明确哪些版本契约已经静态接线，哪些 release gates 仍未运行。目标发行面为 Desktop（Windows/macOS/Linux）、Web/Vercel 和三份 Star Canvas 服务镜像；稳定版产品版本只来自经校验的 Git tag。

当前基线 HEAD 是 66610de4ce7ae2c56241e9af06c32d6074c21666。已集成 W0 证据包括 resolver commit 9090942e240137cea15aff097be1800f33e2a621、Web 172b92abd59262ba4d0352acd8dc5978953ec132、Desktop 55386e5eb532fc1c5b0b66ed1cc5c91d0aab8224、Canvas fc7cbe3e5b002328910a54b713d7a0996f5dc7f8，以及 Canvas 阶段复核 85ef753d6d20d2d156569597f114bb3974be3864。W6 文档候选审阅 6f7cbc88f670e2764269d6b0ec1d00b652513b94 在当前 dev；其当前基线说明由 66610de4 更新。

## §1 发行版本映射矩阵

| 发行面 | 当前集成状态 | tag 绑定与证据边界 |
|---|---|---|
| Git tag / 责任人 | 目标版本 v1.0.6；Git tag 是唯一版本源；Codex/assistant is accountable release owner。用户已条件授权在全部必需发行门禁通过且最终复核完成后发布一个稳定版本。当前本地没有 v1.0.6，远端 tag 状态未知。条件授权已给出，但其执行条件尚未满足。 | resolver 要求 canonical vMAJOR.MINOR.PATCH tag、精确 tag ref、peeled commit 等于 HEAD，以及 tracked/untracked/submodule 工作树干净；输出 tag、版本与 source SHA。ignored files 不在 resolver clean 检查范围，须由 packaging 输入约束处理。resolver commit 9090942e 集成；其 lane 报告记录 10 项 focused tests 通过。 |
| Web / Vercel | stable Web workflow 已使用共享 resolver，将 tag、规范版本和 source SHA 绑定到 build-info、证据目录、压缩包、artifact 与 attestation 名称。npm package metadata 仍为 0.1.0，按 lane 范围不作为 Star release 版本。 | Web lane commit 172b92ab 集成；报告记录 YAML/静态契约检查通过。该 workflow 只生成 build-evidence artifact，不执行 Vercel deployment。没有 hosted workflow、production deployment、运行版本读取或 consumer/rollback 证据。 |
| Desktop | stable Linux、Windows、macOS build 在 mutation/build 前运行 resolver；stable Tauri version、Rust runtime version、artifact name 和身份 JSON 使用同一 resolver 输出；开发构建仍用 crate 版本回退。 | Desktop lane commit 55386e5e 集成并保留 --locked 与审批/tag 门禁。报告记录 fmt、YAML/静态检查通过；精确 Rust 1.99.0 test 未能启动，因为本机安装的 toolchain 缺可用 rustc.exe；actionlint 不可用。没有真实 tagged build、安装包、签名、公证或安装/更新/rollback 验收。 |
| 三份 Canvas OCI 镜像 | release-only filtered Cargo context 将 resolver 版本写入临时 workspace/lock root；OCI version/source tag、labels、digest records 和最终 K3s manifest renderer 共用 tag/version/source SHA。普通 workspace manifests 未改，开发 crate 版本仍为 0.1.0。 | Canvas lane commit fc7cbe3e 集成，复核 commit 85ef753d。lane 报告记录 renderer 8 项 tests、resolver 10 项 tests、过滤上下文仿真和静态 YAML 检查通过。精确 Rust 1.99.0 all-targets 在编译前被不可用 rustc.exe 阻断；actionlint 不可用。没有 hosted Buildx、真实 OCI digest、镜像发布、运行时版本读取或部署证据。 |
| 活动 K3s manifest | 三份镜像使用必须替换的 digest placeholders；六处版本 label 使用 release render 值。renderer 在三个 digest 与 resolver identity 不一致时应 fail closed。 | 目前没有已发布镜像 digest 或生成的 release manifest artifact。manifest 仍是不可直接部署模板；无实际 K3s rollout/consumer 验证。 |
| 独立本地 K3s Dockerfile | deploy/k3s-local/Dockerfile 仍有宽泛 COPY crates ./crates；根 .dockerignore 只排除 star-game 与 game-legacy。此 context 不属于三份 Canvas allowlist。 | 这是独立 payload-closure blocker，须完成构建输入过滤或经批准的 payload 分类/退役说明。Canvas filtered-context 成果不能证明该 Docker context 合格。W7 候选尚未 rebase 并同步到当前 dev，不能据其结果关闭此项。 |
| W6 Vercel 候选评审 | 当前 dev 中的固定提交审阅结论为 ADAPT；评审的是候选设计文档，不是已接受实现。候选需要适配 current DD/TD，并补 Project 0..N repositories、hostless Run/provider optionality、Neon 与 Owner 映射、owner-commit push collaboration 等契约。 | W6 review commit 6f7cbc88；基线注记由当前 commit 66610de4 刷新。没有 Neon/Owner/RLS/pooler/migration runtime proof、Vercel production deployment、push collaboration、consumer verification 或 rollback；W6 保持 OPEN/ADAPT。 |

Rust package/workspace metadata 的普通开发版本仍可保留为 crate metadata，但不能成为 Star 产品 release 版本的第二来源。现有文档仍记录 root Cargo workspace license metadata 与 MIT 用户决策不一致；这是 W1 独立阻断，本报告未改 LICENSE、Cargo metadata 或法律文件。

## §2 验证摘要

| 核验 | 结果 | 证据边界 |
|---|---|---|
| 当前基线与集成提交 | 当前基线为 dev@66610de4；resolver、Web、Desktop、Canvas、Canvas 复核及 W6 review 提交均为当前 HEAD 祖先。 | Git ancestry 查询通过；只证明这些提交已在当前本地 dev 历史中，不证明远端同步。 |
| Git tag | 本地 v1.0.6 不存在；未创建 tag。 | 只查询当前本地 refs；远端状态未刷新。 |
| resolver / Canvas focused tests | lane 报告分别记录 resolver 10 项与 renderer 8 项通过。 | 来自已集成 lane 报告，不在本次文档同步中重跑。 |
| Web / Desktop / Canvas 静态检查 | lane 报告记录 Web workflow YAML/静态断言通过；Desktop fmt、YAML/静态断言通过；Canvas YAML、renderer 和过滤上下文仿真通过。 | 静态配置不替代 GitHub-hosted workflow 或真实产品制品。 |
| Rust 1.99.0 gate | Desktop 测试与 Canvas all-targets 编译均未能启动或完成，因已安装 1.99.0 Windows toolchain 的 rustc.exe 不可用。 | 不以其他 Rust 版本替代；exact 1.99.0 gates 仍 OPEN。 |
| Vercel / Neon / collaboration | W6 候选 review 仅为固定提交的文档审阅，结论 ADAPT。 | 没有 production deployment、Neon/Owner runtime、push-collaboration、consumer 或 rollback 执行证据。 |
| 本次文档一致性 | WBS、当前版本映射、阶段报告和 dispatch JSONL 均按当前 dev 状态同步。 | 本次验证记录在 PHASE-STAR-STABLE-W0-INTEGRATION-SYNC-20261101；不包含应用构建、部署或发布。 |

## §3 已知缺口

1. 尚未在用户批准的最终 source commit 上创建或核验 v1.0.6 tag；远端 tag immutability 与权限未查。
2. 共享 resolver 尚未由真实 stable tag hosted job 执行；GitHub-hosted CI、Desktop/Web/Canvas 实际 tag builds 与 build artifacts 均未验证。
3. Desktop 与 Canvas 精确 Rust 1.99.0 gates 尚未通过；Rust toolchain 故障不能由其他版本或静态检查替代。
4. 没有真实 Vercel production deployment、Neon/Owner placement、API/DB/RLS/pooler/migration runtime 或协作事件路径证明。
5. W6 固定候选审阅结果是 ADAPT；候选不是当前 DD/TD 的可直接接受实现。
6. 没有三份 Canvas 实际 image digest、SBOM/provenance consumer verification、K3s rollout、Web/Desktop consumer verification 或 rollback 记录。
7. deploy/k3s-local/Dockerfile 的宽泛 crates context 仍是独立 payload-closure blocker，需过滤或完成明确分类。
8. 根 Cargo workspace MIT metadata 差异及完整第三方 license/NOTICE/SBOM 审阅仍属 W1 开放项。
9. W0/W6/W7 与其他发行 gates 均未闭合；稳定发行维持 NO-GO，用户的条件授权已给出，但门禁与最终复核条件尚未满足；当前不得执行 release。

## §4 子代理失败接手清单

本次同步未派生子代理，没有 worker RPC 失败或遗留代码需要接手。W7 lane 由协调者在独立 worktree 审查；本报告只同步 parent 提供并确认属于 WBS 的当前 W7 状态，不修改该 lane 文件或候选分支。

## §5 守门规则

1. Git tag 是唯一产品发行版本源；v1.0.6 只是目标 tag，未创建即不得声称版本已发布。
2. stable resolver 必须检查 canonical tag、exact ref、peeled commit 与 HEAD 一致，以及受约束的干净构建输入。
3. Web、Desktop、Canvas 的产品可见版本、source SHA、artifact identity 与 provenance 均须来自同一 resolver 输出。
4. Cargo/npm 内部 package metadata 不能成为第二个 Star 产品版本源；临时 release metadata 的范围须清楚且可追溯。
5. Web build evidence 不等于 Vercel production deployment；静态配置不等于 hosted CI、实际 tag build 或已交付制品。
6. 精确 Rust 1.99.0 检查未通过前，不以其他工具链结果代替稳定版验收。
7. Canvas digest placeholder、模板渲染和仿真不能替代真实 image digest、consumer verification 或 K3s rollout。
8. deploy/k3s-local/Dockerfile 宽泛 COPY crates ./crates 需独立做 payload filtering/classification；Canvas allowlist 不覆盖此路径。
9. W6 ADAPT 不等于候选实现获接受；Neon/Owner、协作、Vercel、消费者及 rollback 事实需各自验证。
10. release 状态保持 NO-GO，直到全部 W0–W9 gates 闭合且满足用户已经给出的发行条件授权：全部必需门禁通过并完成最终复核。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— tag 唯一版本源与产品面保持一致；payload 闭包未完全核实 |
| SRE Lead | 架构师（Mavis 接手审核）— hosted CI、精确 Rust 1.99.0、Vercel/Neon 与 rollback 证据开放 |
| 平台 | 架构师（Mavis 接手审核）— W0 版本接线已集成，真实 tag build 与制品验证开放 |
| 评审主持 | 架构师（Mavis 接手审核）— W6 ADAPT 与 W7 payload blocker 按当前证据保留 |
| PM / Release Owner | Codex/assistant is accountable release owner；用户已条件授权门禁通过且最终复核完成后发布一个稳定版本；当前条件未满足 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 以 dev@2a57fa0d 为基线静态盘点 Git tag、Cargo/npm/Tauri、stable workflows、Canvas OCI/K3s 与 Helm placeholder；记录版本 resolver 方向并保持 W0 PARTIAL。 | 用户确定 Git tag 是唯一版本源，目标为 v1.0.6 |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步当前 dev@66610de4 中 resolver、Web、Desktop、Canvas 版本绑定和 W6 ADAPT 文档审阅；明确 hosted/Rust 1.99.0/Vercel-Neon/consumer/rollback 与 conditional authorization 条件尚未满足，并将 deploy/k3s-local 宽泛 COPY 记为 payload-closure blocker。 | 当前 dev 已接受 W0 lanes 与 W6 review |
| v0.3 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更正 accountable release owner 为 Codex/assistant，并记录用户已条件授权在全部必需发行门禁通过且最终复核完成后发布一个稳定版本；授权条件未满足前仍保持 NO-GO，不要求二次授权。 | 协调者依据用户既有发行责任与条件授权答复更正本 lane 表述 |
