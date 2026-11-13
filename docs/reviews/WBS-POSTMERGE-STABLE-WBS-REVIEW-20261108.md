# WBS-POSTMERGE-STABLE-WBS-REVIEW-20261108

**审查基线：** 本地 `dev@131bc9581badcc214add9a29a490ff0241fa2e0f`
**Lane：** `agent/wbs-postmerge-20261108/stable-wbs`
**Worktree：** `C:/Users/leo19/.codex/worktrees/Star-wbs-postmerge-20261108-stable-wbs`
**审查对象：** 本 lane 对 `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` 的 2026-11-08 同步改动
**结论：** **ACCEPT — 改动内容与当前 dev 实测一致；不关闭任何发行门禁。**

## §0 目的与边界

独立复核本 lane 对 Stable Release WBS 的同步改动是否 (a) 属实反映当前 dev 的真实状态、(b) 保留了候选分支「HOLD、不合并」的处置、(c) 未把任何未验证门禁写成通过、(d) 未越界改动 lane 登记范围外的文件。

本审查只读核对 Git 对象与文件内容。**未运行 Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release 或 deployment。** 未 fetch、未 push、未改 `main`、未改任何候选分支或 worktree。

## §1 改动清单

| 文件 | 改动 |
|---|---|
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` | 新增 2026-11-08 顶部同步快照段；W1 / W2 / W6 / W8 四行状态列追加实测事实；修订历史新增 v1.59 行 |
| `docs/briefs/WBS-POSTMERGE-STABLE-WBS-20261108.md` | 本 lane 的 brief（守门 #20：先于 WBS 改动落盘） |
| `docs/reviews/WBS-POSTMERGE-STABLE-WBS-REVIEW-20261108.md` | 本文件 |
| `docs/reports/PHASE-STAR-STABLE-WBS-POSTMERGE-SYNC-20261108.md` | 阶段报告 |
| `docs/reports/WBS-DISPATCH-POSTMERGE-STABLE-WBS-20261108.jsonl` | dispatcher 落档 |

未改动任何其他文件；`git status --short` 在提交前仅列出上述路径。

## §2 独立复核清单（逐条实测）

| # | 待证事实 | 复核命令 | 结果 |
|---|---|---|---|
| 1 | `dev:AGENTS.md:36` 仍写 1.99.0 | `git show dev:AGENTS.md \| sed -n '36p'` | ✅ 逐字命中 "minimum supported Rust version is 1.99.0" |
| 2 | `dev:Cargo.toml:185` 为 1.99.1 | `git show dev:Cargo.toml \| sed -n '185p'` | ✅ `rust-version = "1.99.1"` |
| 3 | `dev:Cargo.toml:187` 为 Apache-2.0 | `git show dev:Cargo.toml \| sed -n '187p'` | ✅ `license = "Apache-2.0"` |
| 4 | `dev:rust-toolchain.toml` 为 1.99.1 | `git show dev:rust-toolchain.toml \| grep channel` | ✅ `channel = "1.99.1"` |
| 5 | `dev:LICENSE` 正文为 MIT | `git show dev:LICENSE \| head -1` | ✅ `MIT License` |
| 6 | VCD-0 ancestry = 459/5 | `git rev-list --left-right --count dev...codex/vcd0-canonical-detail-test-20261024` | ✅ `459  5` |
| 7 | W8 source ancestry = 434/9 | `git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025` | ✅ `434  9` |
| 8 | 该 lease-order migration 不在 dev | `git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` | ✅ `does not exist in 'dev'` |
| 9 | dev Inbox 仍用 v2 | `git show dev:crates/bi-owner-runtime/src/task_event_inbox.rs \| sed -n '40p;353p'` | ✅ 行 40 = `bi.commit_task_event_v2`；行 353 = `FROM bi.commit_task_event_v2(` |
| 10 | dev quarantine 仍用 v1 | `git show dev:crates/bi-owner-runtime/src/task_event_quarantine.rs \| sed -n '16p;81p'` | ✅ 行 16 = `bi.record_task_event_quarantine_v1`；行 81 = `FROM bi.record_task_event_quarantine_v1(` |
| 11 | 候选 `pull_one` REVOKE 后又 GRANT 回 runtime | `git show UlyssesLeoLee/...:db/migrations/2026-10-24-platform-native-transport-lease-order.sql \| sed -n '104,113p'` | ✅ `REVOKE ALL ... FROM PUBLIC, star_platform_transport_runtime;` 紧接 `GRANT EXECUTE ... TO star_platform_transport_runtime;` |
| 12 | 候选测试把该授权固化为期望 | `git grep -n pull_one_execute_is_regranted_only_to_the_runtime_role <candidate>` | ✅ `crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:523` |

**12/12 命中。** 无一条断言依赖记忆或转述。

## §3 关键纠正：Rust 候选审查中的事实错误

Rust 基线候选审查（`docs/reviews/WBS-MERGE-RUST-BASELINE-CANDIDATE-20261108.md`）§1.4 与 §3 第 1 项称：

> 「当前 `AGENTS.md` 已为 1.99.1，因此不应沿用旧实施报告中『AGENTS 仍为 1.99.0』的陈述。」

**该陈述不成立。** 实测 `dev:AGENTS.md:36` 仍为 1.99.0（复核 #1）。因此：

- 本 WBS **不能**写成「AGENTS.md 已是 1.99.1」，否则会把一个未关闭的规范漂移缺口写成已解决；
- 正确表述是：workspace/toolchain 基线已为 1.99.1，但 `AGENTS.md` 仍为 1.99.0，**两者构成仍存在的规范漂移缺口**，与 `docs/requirements.md` AC-INFRA-002-R/S/U/V/W、`docs/basic-design.md` §16.44–§16.47、`docs/design/DD-WORKTREE-GROUP-001.md` AC-INFRA-002-U/V 的活动 1.99.0 要求同属待同步项。

本 lane 采纳纠正后的表述，并**明确标注该纠正与来源审查不一致**（守门 #11 缺标比错标）。

## §4 处置一致性核对

| 检查项 | 结果 |
|---|---|
| 是否摘取或合并了任何候选提交？ | ❌ 无。W2/W6/W8 三行均记录 HOLD，且明确候选分支、worktree、文件保持原样 |
| 是否把候选的 `be154efb` 版本文字同步当成已完成？ | ❌ 无。W2 行明确写「只在新登记的文档 lane 内逐句适配，不整提交摘取」 |
| 是否把 ancestry 数字当作内容差异？ | ✅ 数字仅作定位，且同时记录了处置结论 |
| 是否新增了任何 W0–W9 门禁状态变化？ | ❌ 无。四行状态词未改（W1 BLOCKED / W2 OPEN-BLOCKED / W6 ADAPT-OPEN / W8 OPEN） |
| 是否把 `pull_one` P1 问题写成 dev 既有缺陷？ | ✅ 明确写「在当前 dev 中不存在」「属新引入路径的风险而非既有 dev 状态」 |
| 是否声称任何未运行的验证已通过？ | ❌ 无。W8 行的 PostgreSQL/RLS/concurrency/durable-ack/recovery 门仍写 OPEN |
| 是否越界改动 lane 登记范围外的文件？ | ❌ 无。`AGENTS.md`、`requirements.md`、`basic-design.md`、`DD-WORKTREE-GROUP-001.md` 均未改动 |
| 是否改 `main` / push / tag / build / deploy？ | ❌ 无 |

## §5 表格与格式完整性

- W0–W9 十行：**每行 7 个 `|`**，与改动前一致 → 表格列数未错配。
- 修订历史行：**每行 6 个 `|`**，与表头 `| 版本 | 日期 | 修订人 | 内容 | 触发 |` 的 6 个一致。
- 顶部新增快照段：作为 blockquote 段落插入 `## 已确认的发行决策` 之前，未插入表格内部。
- `git diff --check`：**clean**（无尾随空格 / 空白错误）。

## §6 已知缺口与限制

1. **只读复核不是执行验证。** 本 lane 未运行 Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release、deployment。WBS 中相关门禁状态保持未运行/开放。
2. **ancestry 数字是 Git metadata，不是内容差异证明。** 459/5 与 434/9 说明候选相对当前 dev 的分叉规模，不能证明候选内容是否已等价进入 dev；结论另由三份审查的逐提交依赖闭包分析给出。
3. **远端新鲜度未知。** 本轮未 fetch；`origin/dev`、`origin/main` 与 GitHub 上 v1.0.6 tag/release 状态仍是既有记录中的历史快照。
4. **`pull_one` P1 问题的严重度未做运行时评估。** 本 lane 只核对了 migration 文本与测试名，未在 PostgreSQL 上执行 grant / 直接调用负例。该验证属于后续 W8 实现 lane 的门。
5. **`AGENTS.md` 1.99.0 缺口仍未关闭。** 修复它需要改动 `AGENTS.md`，超出本 lane 登记范围；必须由单独授权的 lane 处理。
6. **`docs/requirements.md` / `docs/basic-design.md` / `docs/design/DD-WORKTREE-GROUP-001.md` 的活动 1.99.0 要求同样未关闭**，同上。

## §7 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | ACCEPT。本 lane 同步的三份审查结论与当前 dev 实测一致，四个状态行的门禁状态未变。 |
| SRE Lead | 未运行任何构建 / 部署 / runtime gate；相关门保持 OPEN。 |
| Platform | `AGENTS.md` 1.99.0 漂移缺口已如实登记为未关闭项；`pull_one` P1 授权问题登记为新引入路径风险。 |
| 评审主持 | 12/12 独立复核项实测命中，表格结构与 diff check 通过，接受本 lane。 |
| PM | W0–W9 与 v1.0.6 维持 NO-GO；本报告不授权 tag/release/deploy。 |

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对本 lane 的 Stable Release WBS 2026-11-08 同步改动做独立只读复核：12 项事实逐条 `git show` / `git rev-list` 实测，纠正来源审查中「AGENTS.md 已为 1.99.1」的不成立陈述，确认无候选提交被摘取、无门禁被写成通过、无登记范围外改动；结论 ACCEPT。 |
