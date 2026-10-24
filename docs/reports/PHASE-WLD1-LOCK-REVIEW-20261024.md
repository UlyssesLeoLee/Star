# PHASE-WLD1 Cargo.lock 依赖边审查报告

- 版本：v0.1
- 日期：2026-10-24
- 阶段：WBS-WLD1-LOCK-REVIEW-20261024
- dev 基线：`d4f26ee965106c5bf70dbe07c0b05cbb5639c06b`
- 候选依赖提交：`48f261011a51d14c44ca0182f99ebdade7a2bce5`（本地 `origin/dependabot/cargo/rust-dependencies-fe3c6ef10d` ref；未 fetch，ref 新鲜度未知）
- 结论：**无需把候选分支的整组 Cargo.lock 更新带入 WLD1。预期最小差异是一条本地包依赖边；Cargo 1.99.0 生成与锁定验证尚未执行，因此 lockfile 集成仍未验收。**

## §0 目的

只读核对 WLD1 将 `domain-scm` 加入 `star-pg-adapter/Cargo.toml` 后所需的最小锁文件变化，并评估本地 Dependabot 候选是否适合作为 lockfile 来源。本报告只依据当前可见的本地 Git refs 和工作文件；不证明 remote-tracking ref 的最新状态，也不把静态推断写成 Cargo 已生成或验证。

## §1 核查与结论矩阵

| 核查项 | 可复核证据 | 结论 |
|---|---|---|
| WLD1 manifest 依赖 | WLD1 工作树 `crates/star-pg-adapter/Cargo.toml:28` 声明 `domain-scm = { path = "../domain-scm" }`。 | Manifest 已声明直接依赖。 |
| lock 中的目标包 | dev `Cargo.lock:2109-2120` 已含 `domain-scm 0.1.0`，依赖闭包列有 `async-trait`、`chrono`、`serde`、`serde_json`、`star-context`、`thiserror 2.0.20`、`tokio`、`uuid`。 | 目标 workspace 包及其当前锁定闭包已存在。 |
| 缺失依赖边 | dev `Cargo.lock:7274-7291` 的 `star-pg-adapter` entry 包含 `domain-automation`、`hex` 等，但没有 `domain-scm`。WLD1 工作树同一 entry 仍未更新。 | 预期的最小锁差异是向 `star-pg-adapter` entry 的依赖数组按字典顺序插入一项 `"domain-scm"`（位于 `chrono` 与 `domain-automation` 之间）。这条边可从 manifest 与已有 lock entry 推断；它尚未由 Cargo 生成器确认。 |
| 依赖候选的基线与范围 | `48f26101` 的父提交是 `5a051bfd8f76fbfbe8ff49a994579c60166d6ec6`，提交主题为 `chore(deps): bump the rust-dependencies group across 1 directory with 15 updates`。相对 dev `d4f26ee`，候选 `Cargo.lock` diff 为 408 行新增、502 行删除；package entry 总数为 885 → 875。 | 该候选不是 WLD1 所需的一行本地依赖边，也不是从当前 dev 生成的 lockfile。不要整份搬入 WLD1。 |
| 候选中观察到的版本替换 | 相对 dev lock，24 个同名、同 registry source package 的锁定版本发生替换，列表见下方。 | 这些升级不由新增 path 依赖本身要求；即使其中部分升级可独立评估，也应在独立升级任务中处理。 |
| 候选中新增/移除的包身份 | 新增：`minicov 0.3.8`、`static_vcruntime 3.0.0`、`wasm-bindgen-test-shared 0.2.128`。候选中不再出现：`console_error_panic_hook 0.1.7`、`scoped-tls 1.0.1`、`unicode-xid 0.1.0`、`utf-8 0.7.6`。此外候选移除旧的并存版本身份：`cfg-if 0.1.10`、`futures 0.1.31`、`nix 0.29.0`、`png 0.17.16`、`proc-macro2 0.4.30`、`quote 0.6.13`、`tokio-tungstenite 0.24.0`、`tungstenite 0.24.0`、`wasm-bindgen-futures 0.3.27`。 | package identity 按 `(name, version, source)` 比较时，候选有 27 个新增身份、37 个移除身份；包总数减少 10。此范围包含其他依赖图变动，超出 WLD1 的 lockfile 需求。 |

**24 项锁定版本替换（dev → 候选）：**

| Package | 版本变化 | Package | 版本变化 |
|---|---|---|---|
| `hecs` | 0.11.1 → 0.11.2 | `ico` | 0.5.0 → 0.6.0 |
| `redis` | 1.7.0 → 1.7.1 | `serde_json` | 1.0.151 → 1.0.154 |
| `serial_test` | 3.5.0 → 4.0.1 | `serial_test_derive` | 3.5.0 → 4.0.1 |
| `sha2` | 0.11.0 → 0.11.1 | `swift-rs` | 1.0.8 → 1.0.9 |
| `tauri` | 2.12.1 → 2.12.4 | `tauri-build` | 2.7.1 → 2.7.4 |
| `tauri-codegen` | 2.7.1 → 2.7.4 | `tauri-macros` | 2.7.1 → 2.7.4 |
| `tauri-runtime` | 2.12.1 → 2.12.2 | `tauri-runtime-wry` | 2.12.1 → 2.12.3 |
| `tauri-utils` | 2.10.1 → 2.10.2 | `thiserror` | 2.0.20 → 2.0.21 |
| `thiserror-impl` | 2.0.20 → 2.0.21 | `tokio` | 1.53.1 → 1.53.3 |
| `tokio-util` | 0.7.19 → 0.7.20 | `tree-sitter` | 0.27.0 → 0.27.1 |
| `tree-sitter-language` | 0.1.8 → 0.1.9 | `uuid` | 1.26.1 → 1.28.0 |
| `wasm-bindgen-test` | 0.2.50 → 0.3.78 | `wasm-bindgen-test-macro` | 0.2.50 → 0.3.78 |

`thiserror`、`sha2` 等版本替换虽与 adapter 当前依赖名称相关，但 dev lock 已为它们选定满足当前 manifest 的版本；添加 `domain-scm` 依赖边不要求把这些版本升级。候选还基于 main 的父提交，不能将其 lock 差异等同于以当前 dev 为输入重新解析的结果。

## §2 验证摘要

- 经 RTK 只读检查了 dev `d4f26ee` 与本地候选 `48f26101` 的 `Cargo.lock` package entries、diff numstat，以及 WLD1 工作树 manifest/lock 条目。
- WLD1 工作树在本轮开始时已有实现文件未提交改动；审计未修改这些文件，也未触碰 `Cargo.toml` 或 `Cargo.lock`。
- `cargo` / `rustup` 在当前执行环境不可用（由协调者本轮环境检查确认）；本审计未运行 `cargo metadata`、`cargo generate-lockfile`、`cargo check` 或测试。因此一行最小 delta 只是静态可推断结果，**不是 Cargo 生成或验证通过的证据**。
- 报告为未跟踪新文件，因此用 `rtk proxy git diff --no-index --check -- NUL <报告路径>` 检查；无空白诊断输出。该命令 exit 1 表示文件与 NUL 存在差异，是 `--no-index` 的正常差异状态，不是空白错误。

## §3 已知缺口与建议集成步骤

1. 当前没有 Cargo 1.99.0 实测，无法证明 resolver 对 WLD1 目标 manifest 的规范 lockfile 输出恰为一行，也无法证明没有其他必需解析变化。
2. 将 WLD1 实施 lane rebase 到最新 dev 后，在 Rust 1.99.0 环境以该工作树完整 manifest 为输入更新 lockfile；优先使用保留既有选定版本的普通解析流程，随后核对 Cargo.lock diff。预期变更仅在 `star-pg-adapter` entry 增加 `"domain-scm"`；若 Cargo 生成更广差异，应逐项解释而不是直接接受。
3. 对生成结果运行 `cargo check --workspace --lib --locked -j 4`；再按协调者定义的 WLD1 全量门禁运行测试与 PostgreSQL 验收。任何锁定验证失败都先解决，再集成。
4. 不 cherry-pick 或复制候选 `48f26101` 的整组升级。若未来要评估这些升级，需以当前 dev 为基线建立独立 manifest、验证和许可证/供应链审查范围。
5. `origin/dependabot/...` ref 未 fetch，不能据本审计确认 GitHub PR 状态、远端是否有更新或该提交是否仍代表最新候选。

## §4 子代理失败接手清单

本审计 lane 未派发子代理，没有待接手子代理产出或失败项。审查范围只读 WLD1 的 manifest/lock 目标条目和指定 dev/候选 refs；WLD1 实施修复由其 owner lane 负责。

## §5 守门规则核对

| # | 守门项 | 本阶段结果 |
|---|---|---|
| 1 | 只写 brief manifest 指定的报告文件 | 通过；本报告为唯一新增交付文件 |
| 2 | 不编辑 Cargo.toml、Cargo.lock、源码或其他 lane | 通过；均只读 |
| 3 | 所有 Git 操作经 RTK | 通过 |
| 4 | 不 fetch，不宣称 remote-tracking ref 新鲜 | 通过；明确标记新鲜度未知 |
| 5 | 不 commit、rebase、cherry-pick、merge 或 push | 通过；本 lane 仅审计 |
| 6 | 不把候选大范围依赖升级带入 WLD1 | 通过；建议以 dev lock 为基线生成本地最小差异 |
| 7 | 不将静态推断说成 Cargo 已生成或验证 | 通过；明确 Cargo 验证未运行 |
| 8 | Rust toolchain 基线为 1.99.0 | 设计为后续验证门；本环境缺 Cargo，未验证 |
| 9 | 报告差异含 package 与锁定版本证据 | 通过；列明 24 项替换及新增/移除 package identities |
| 10 | 报告-only diff-check | 写入后用 no-index 检查；无空白诊断输出，exit 1 是新文件差异状态 |

## §6 签字栏

| 角色 | 审阅结论 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）：建议仅在当前 dev 基线上生成 WLD1 所需 lock 变更；不接纳整组旧基线升级。 |
| SRE Lead | Mavis 接手审核：没有 Cargo/Rust 编译或数据库证据；保留为未验证。 |
| 平台 | Mavis 接手审核：`domain-scm` 包已在锁文件；直接依赖边尚需 Cargo 1.99.0 生成并锁定验证。 |
| 评审主持 | Mavis 接手审核：候选 refs 是本地只读证据；remote 状态未知。 |
| PM | Mavis 接手审核：lockfile 集成门未关闭，不据本报告将 WLD1 标为完成。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容与触发 |
|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 WBS-WLD1 lock-review brief 比较当前 dev、WLD1 目标条目与本地 Dependabot 候选；确认可推断的一行依赖边，拒绝把候选的大范围锁版本变化作为该边的替代；Cargo 生成与验证保持开放。 |