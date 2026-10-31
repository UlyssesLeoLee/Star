# PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031

> **版本**：v0.4 · **日期**：2026-10-31 · **分支**：`agent/stable-release-wave2/desktop-attest` · **基线**：`319026f8`（前一 Desktop lane commit） · **范围**：仅 Desktop stable workflow、此 lane brief、此阶段报告及 dispatcher JSONL。
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

## §0 目的

为 Linux `.deb`/AppImage、Windows MSI 和 macOS universal DMG 补充 SPDX 依赖清单及 GitHub 构建证明。构建与测试任务继续只获得只读仓库权限；签发 OIDC 和 attestation 权限仅在手动 `v*` 标签 dispatch、`release_confirmation` 明确选择 `APPROVED` 且共享 MIT license metadata gate 通过后，进入 `star-stable-release` 环境的隔离任务中。输出仍是 unsigned CI artifacts，不创建标签、不发布、不部署。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `.github/workflows/star-stable-desktop.yml` | Linux/Windows/macOS 各生成 package-scoped Rust SPDX 2.3 与 frontend SPDX 2.3；校验 JSON/schema 基本字段及空输入，拒绝 SBOM 中出现 retired `star-game` 标记；上传 unsigned bundles + SBOM；仅在 `workflow_dispatch`、精确确认 `APPROVED` 且 ref 为 `v*` tag 时运行共享 MIT license metadata gate，随后每个平台的 `star-stable-release` environment job 才能建立 provenance 与两份语言级 SBOM predicate。 | 已实施；本地静态检查通过 |
| `docs/briefs/STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md` | 记录 SBOM 按生态系统拆分、权限隔离、显式批准/命名环境双门，以及当前许可证元数据不一致会阻断证明的事实。 | 已更新 |
| `docs/reports/PHASE-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.md` | 本报告：差异、验证结果、外部阻塞与发布门。 | 已新增 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W2-DESKTOP-ATTEST-20261031.jsonl` | 通过 dispatcher 追加静态验证完成记录。 | 已追加 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| YAML 解析与工作流静态断言（PyYAML 6.0.3） | PASS：7 个 job；dispatch choice 必选、默认 `NO`、仅含 `NO`/`APPROVED`；3 个构建 job 只读；共享许可证 gate 只在 `APPROVED` + `v*` tag 时运行；3 个证明 job 均依赖许可证 gate、引用 `star-stable-release`，并校验对应平台的实际 subject/SBOM 路径。 |
| Action SHA 格式 | PASS：22 处 `uses` 均使用 40 位完整 SHA。 |
| 上游 tag 到 SHA 核对 | PASS：`actions/attest@v4.2.2` → `1e69f48acb82d1966a394da916b4c1698aa569d6`；`actions/download-artifact@v8.0.2` → `9000827ccba6bdab643e8b6fd33ac0654aef8333`；`actions/upload-artifact@v7.0.2` → `cf430e030ddbb5b0abf93d22962f4752f3646cd9`；`actions/checkout@v7.0.1` → `3d3c42e5aac5ba805825da76410c181273ba90b1`；`actions/setup-node@v7.0.0` → `820762786026740c76f36085b0efc47a31fe5020`。使用 `rtk git ls-remote` 对各自官方上游仓库 tag 逐项核验。 |
| `git diff --check` | PASS：无空白/补丁格式错误。 |
| Frontend SPDX generation command | PASS locally with npm 10.9.3: generated SPDX 2.3 JSON from the committed lockfile with 225 packages and no retired `star-game` package. The configured hosted runtime is Node.js 24.22.2, so this is command-level evidence, not hosted-version parity. |
| Release license metadata gate | PASS fail-closed behavior: executed the exact Python heredoc extracted from workflow; root `[workspace.package].license` is `Apache-2.0`, while `LICENSE` begins `MIT License`; script exits nonzero on the selected MIT mismatch. |
| Cypher relationship header | PASS: source digest matches `599cff6ca6747565b8d05b9a31ce6be003fff5b31b49c08c3a7042f8e8989f67`; `cgg_guard --changed --stamp` reported 0 code files because workflow YAML is excluded, so Guardian `source_digest` was checked directly. Cypher references were parsed and all 5 pinned action identities plus 3 new gate configs matched the workflow. |
| Dispatcher validation record | PASS: final lane validation is appended through `SubagentDispatcher._audit`; hosted CI, actual attestations, and consumer verification remain explicitly unrun. |
| Dispatcher final validation record | PASS: appended through `SubagentDispatcher._audit`; records 7 jobs, 22 pinned actions, tag checks, subject scope, expected license block, and hosted-only gaps. |
| Hosted CI / 实际产物 / 消费者验证 | 未执行；不得标记通过。 |
| `actionlint` | 本机未安装；以 PyYAML 语法解析及定向结构断言替代，仍需 hosted run 补足 GitHub workflow 语义验证。 |
| Rust SBOM 生成器 / Cargo build | 未在本机运行：Cargo 1.99.0 CLI 存在，但所选 toolchain 缺少 `rustc` component，且 `cargo-sbom` 未安装。CI 会安装固定版本 `cargo-sbom 0.10.0` 并在 hosted runner 实际执行。 |

核对资料：GitHub [Artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations)、[workflow_dispatch inputs](https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow)、[environment protection rules](https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments)、官方 [`actions/attest` v4.2.2](https://github.com/actions/attest/releases/tag/v4.2.2)、官方 [`actions/download-artifact` v8.0.2](https://github.com/actions/download-artifact/releases/tag/v8.0.2)、[npm SBOM CLI](https://docs.npmjs.com/cli/commands/npm-sbom/)、[`cargo-sbom` 0.10.0 README](https://docs.rs/crate/cargo-sbom/0.10.0/source/README.md) 与[上游仓库](https://github.com/psastras/sbom-rs)。cargo-sbom 文档允许按 `--cargo-package star-desktop` 选定 Cargo workspace 成员，默认 SPDX JSON 2.3；上游使用 MIT 许可证，0.10.0 于 2025-06-17 发布。GitHub 的 [Rust BOM topic 索引](https://github.com/topics/bom?l=rust&o=desc&s=forks)显示上游项目 2026-09-18 更新，满足本轮 12 个月活动核验窗口。Context7 的 GitHub Actions 文档用于核对 dispatch choice 输入、job if 条件及环境审批机制；Cargo 文档用于核对外部子命令传参。Context7 未提供 cargo-sbom 专用条目，具体参数以该 crate 上游 README 为准。

## §3 已知缺口

1. 尚无 GitHub hosted run，因此 SBOM 生成、上传、下载与 attest action 的实际执行都未验证。下游需在候选 `v*` tag 上以 `APPROVED` 手动运行并保存 run URL 与结论。
2. 尚无真实 attestation；发布消费者的 `gh attestation verify` 尚未执行。
3. Attestation 是构建来源证明与 SBOM predicate，不是安装包代码签名，也不提供 Apple notarization。三类桌面产物仍显示 unsigned；签名/notarization 是单独发布门。
4. SPDX 文件按 Cargo 与 npm 生态分别生成；npm 使用锁文件完整图并包含 dev/build 依赖。它不代表对安装包中系统库、bundled runtime 或工具链的完整逐项盘点。许可证、NOTICE、专利/再分发义务必须结合实际 artifact 闭包另行审核；SBOM 本身不是许可证放行结论。
5. `star-game` 静态字符串拒绝规则用于防止遗留业务域被纳入两份 SBOM，但只有 hosted SBOM 检查完成后才可作为实际产物证据。
6. Artifact 保留 7 天用于同一 workflow run 的隔离 attestation job；这不是稳定版本下载或归档渠道。稳定版仍未关闭发布门。
7. 未执行实际 cargo build、真实安装包的 binary hash 核对或跨平台 consumer 验证。Frontend SPDX 命令仅在本机 npm 10.9.3 上验证，未在配置的 Node.js 24.22.2 hosted runner 上复现。
8. 当前 lane 只能在工作流中引用 `star-stable-release`；仓库 Settings 中该 environment 是否已配置 Required reviewers / Prevent self-review 未核验。Stable-release gate 关闭前必须由仓库管理员确认 required reviewer protection 生效，否则 GitHub environment 名称本身不会暂停 job。
9. 当前许可证元数据冲突已实证：用户选择 MIT；`Cargo.toml` 的 `[workspace.package].license` 是 `Apache-2.0`，`LICENSE` 首行是 `MIT License`，且 metadata 由其他 owner 持有。本 lane 不修改共享许可证字段；共享 tag gate 会在证明 job 前 fail closed，直至责任 owner 解决冲突并两项均匹配 MIT。当前稳定版不得生成 artifact attestation。

## §4 子代理失败接手清单

本 lane 未派发子代理；没有待接手的子代理失败产出。主代理只在登记 worktree 和文件范围内实施。

## §5 守门规则

- 只登记/提交上述四个 manifest 文件；不修改其他 workflow、manifest、lockfile、WBS、main、remote 或其他 worktree。
- 不创建标签、不发布、不部署、不 push、不执行 release；用户已确认 stable release 仍须由父级整体门禁决定。
- 所有 Git 操作经 RTK；Action 使用完整 SHA，且对照官方上游 release/tag。
- 构建 job 仅继承 `contents: read`；OIDC 与 `attestations: write` 仅赋给手动 `v*` 标签的证明 job。
- 证明 job 还必须匹配显式 `APPROVED` dispatch 输入并声明 `environment: star-stable-release`；不得将环境名称引用误报为实际 reviewer approval 已发生。
- `release_metadata_gate` 必须在证明 job 之前确认根 Cargo workspace metadata 与 `LICENSE` heading 均匹配 MIT；当前 Apache-2.0/MIT 冲突应失败且不得发出任何 attestation。
- 每个 bundle 的 subject 路径必须直接指向该平台非空文件；SBOM 缺失、为空、JSON 不合法或 SPDX 核心字段缺失时 fail closed。
- 不将 attestation 描述成签名或 notarization，不声称 hosted CI 或 consumer verification 成功。
- 在父级审核确认前，这一提交留在 lane 分支，不由本 lane merge、push 或清理。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) — 静态守门自审通过 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 静态守门自审通过 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 静态守门自审通过 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 静态守门自审通过 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 静态守门自审通过 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 首次记录 Desktop SBOM、tag-only provenance jobs、验证证据与发布缺口。 | STAR-STABLE-W2-DESKTOP-ATTEST-20261031 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 加入默认 NO 的 dispatch choice，要求精确 APPROVED；所有证明任务引用 star-stable-release environment，并记录 Required reviewers 外部配置门。 | 父级独立评审反馈 |
| v0.3 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 增加共享许可证门；当前 Cargo workspace 仍为 Apache-2.0，而 LICENSE heading 为 MIT，证明任务按预期阻止。 | 父级许可证门评审反馈 |
| v0.4 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 记录最终上游 SHA/subject 复核、Cypher YAML digest 的手动核验边界及最终 dispatcher audit；保持许可证冲突和 hosted-only gate 明确未关闭。 | 本轮最终复核 |
