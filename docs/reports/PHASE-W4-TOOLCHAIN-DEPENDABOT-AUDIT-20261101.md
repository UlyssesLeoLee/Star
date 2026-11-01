# PHASE-W4-TOOLCHAIN-DEPENDABOT-AUDIT-20261101

**结论：HOLD。** 不接受 `f6bede9301572ddd249ffd8c2baa39af4f106c0e`；本阶段只完成固定提交的只读审查与文档记录，不修改 CI，不集成候选，也不关闭 W4。

## §0 目的

按 [brief](../briefs/W4-TOOLCHAIN-DEPENDABOT-AUDIT-20261101.md)，核验 `dev@4ee0a7670dcba4e24b5df23eef18e90fa353e8e0` 上的 Rust toolchain 策略，并审阅本地 remote-tracking ref `origin/dependabot/github_actions/dtolnay/rust-toolchain-1.120.0` 的固定提交 `f6bede9301572ddd249ffd8c2baa39af4f106c0e`。审查结论是候选不符合 Rust 1.99.0 基线；不修改 workflow、Cargo、分支或 main。

## §1 评审矩阵与文件范围

| 项目 | 结论 | 证据 / 后续要求 |
|---|---|---|
| 固定对象 | 已核对 | Baseline 为 `4ee0a7670dcba4e24b5df23eef18e90fa353e8e0`；候选完整 SHA 为 `f6bede9301572ddd249ffd8c2baa39af4f106c0e`，父提交为 `8dcb65f7cc5602d4f7c6b681c368d5097bc12a22`。本报告仅评审该 commit patch；baseline 与 candidate 的整树差异还包含其他分支变化。 |
| Workflow 修改 | 不接受 | 候选将 `.github/workflows/ci.yml` 六处、`coverage.yml` 一处、`license-gate.yml` 三处、`p1e-integration.yml` 两处、`publish-canvas-game.yml` 一处、`spawn-e2e.yml` 两处、`star-desktop-build.yml` 三处、`wasm-build.yml` 一处，共八个 workflow 内 19 个引用从 `@1.99.0` 替换为 `@1.120.0`。 |
| Action 引用含义 | 已核实 | 上游 README 说明 action 的 `@rev` 决定安装的 Rust toolchain；显式 `toolchain` input 应配 `@master`；SHA pin 必须来自 `master` 历史。故此变更并非单纯升级 action 实现。 |
| 项目基线 | 冲突 | `rust-toolchain.toml` 的 `channel` 和根 `Cargo.toml` 的 `rust-version` 均为 `1.99.0`。候选将多数 CI 更新成 1.120.0，会停止在声明 MSRV 上验收；`p1e-integration.yml` 保留 `toolchain: 1.99.0` override，但 action ref 仍不匹配上游推荐的显式 override 用法。 |
| 上游 Rust 发布证据 | 不支持候选 | Rust 官方发布索引截至 2026-11-01 列有 2026-10-15 发布的 1.99.1 与 2026-10-01 发布的 1.99.0；没有 1.120.0 发布证据。 |
| SHA pin | 候选未完成 | 候选仍使用版本字符串 ref，不是完整 SHA。若 W4 要求不可变 action pin，应在单独变更中选取并复核属于 `master` 历史的完整 commit SHA，显式指定 `toolchain: 1.99.0`，再跑对齐门禁。 |
| 推荐 | HOLD | 不合入该候选。只有在正式决定改变 MSRV，并同步 toolchain、Cargo、CI 和 focused gates 后，才可另行评估新版本；本阶段不执行该变更。 |

官方证据：[`dtolnay/rust-toolchain` README](https://github.com/dtolnay/rust-toolchain/blob/master/README.md)、[Rust 官方 release index](https://blog.rust-lang.org/releases/)、[Rust 1.99.1 发布公告](https://blog.rust-lang.org/2026/10/15/Rust-1.99.1/)、[Rust 1.99.0 发布公告](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/)。本地已有 SHA 引用先例：`.github/workflows/publish-star-canvas-services.yml` 使用 `ff7f6a3374ce205788da40198ec918bd5d67aea7`，注释标记 `1.99.0`。

## §2 验证摘要

- 读取 brief、固定 HEAD、候选 commit 元数据、parent、merge-base 与 commit patch；确认该 commit 修改 8 个 workflow，共 19 个替换。
- 定向核验 `rust-toolchain.toml`、根 `Cargo.toml` 和当前 workflows 中的 toolchain 引用。
- 读取上游 action README，并核对 Rust 官方发布索引和 1.99.0/1.99.1 官方公告。
- 对候选未运行 workflow、Cargo 构建或测试；未 fetch、push、改动源 workflow、切换分支或操作 main。文档完成后运行 `rtk proxy git diff --check`；暂存后再运行 staged diff whitespace 检查。
- 此为有限配置审查；未验证上游 candidate ref 是否实际存在，也未检查 live PR 或 hosted CI。

## §3 已知缺口

1. 没有 fetch；本地 remote-tracking ref 可能陈旧，不能推断当前 upstream PR 状态。
2. 没有运行 GitHub Actions；`@1.120.0` 的远端 action ref 是否解析成功未实测。官方 Rust release index 仅足以说明该 Rust 版本没有对应官方发布公告证据。
3. W4 的完整行动项（包含所有工作流的 action SHA pin 决策、workflow 成功证据和保护规则确认）仍需协调者按稳定版 WBS 继续审查；本 review 不关闭 W4。
4. 该 lane 不修改 workflow、Cargo baseline 或 candidate ref，因此 HOLD 结论只是审查决定，不等于拒绝/关闭上游 PR。

## §4 子代理失败接手清单

- 本 lane 未派生子代理；无失败子代理、RPC 失败或散落产出需接手。
- 上游资料通过官方页面直接核验；无其他 lane 文件需要移入本 lane。

## §5 守门规则

1. 只依据 brief 固定的 baseline 与候选完整 SHA，不以 branch 名称替代提交证据。
2. 只审阅候选 commit patch；不将 baseline 至候选的整树差异误认为本 commit 的改动。
3. 不改 workflows、Cargo 文件、分支 refs、source worktree 或 main。
4. 不 fetch、push、cherry-pick、merge、rebase、tag、release 或部署。
5. `dtolnay/rust-toolchain@rev` 的版本 ref 会选择 Rust toolchain；不可把它误当作独立的 action release 版本。
6. 显式 `toolchain` override 与 action revision 分别记录；上游建议显式输入时使用 `@master` revision。
7. `rust-toolchain.toml`、Cargo `rust-version` 与 CI 的 MSRV 检查必须保持一致；升级前先决策并运行 Rust 1.99 baseline gates。
8. 只用官方 Rust release evidence 判定实际已发布版本；不从候选分支名称推断 Rust release 已存在。
9. 如采用 action SHA pin，必须核验其完整 40 位 SHA 且位于上游 `master` 历史。
10. 不将静态审查等同于 hosted workflow 成功、PR 合并或 W4 关闭。
11. remote-tracking ref 可能过期；无 fetch 时不得声称 live 远端状态。
12. 只集成本 lane 审查文档；不得覆盖或回滚其他 worker 的工作。

## §6 签字栏

| 角色 | 审批者与评语 |
|---|---|
| 架构 | 架构师（Mavis 接手审核）— 候选破坏 1.99.0 MSRV CI 信号，保持 HOLD。 |
| SRE Lead | 架构师（Mavis 接手审核）— 未运行 hosted workflow；不视为 CI 或 release gate 通过。 |
| 平台 | 架构师（Mavis 接手审核）— `@rev` 的 toolchain 语义与显式 override 均已记录，仍需单独 SHA pin 决策。 |
| 评审主持 | 架构师（Mavis 接手审核）— 范围固定于候选 commit patch；没有扩展到 source workflow 或其他分支。 |
| PM | 架构师（Mavis 接手审核）— 不接受当前候选；W4 保持 OPEN。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 W4 toolchain candidate 固定提交审查；结论 HOLD，记录八个 workflow、官方 action/Rust release 证据和未执行项。 | `docs/briefs/W4-TOOLCHAIN-DEPENDABOT-AUDIT-20261101.md` |
