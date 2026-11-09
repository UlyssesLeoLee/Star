# WBS-POSTMERGE-GROUP-WBS-REVIEW-20261108

**审查基线：** 本地 `dev@872b5c4ad6b6c63e81596c6953fec5274d7782b5`（stable-wbs lane 已合并）
**Lane：** `agent/wbs-postmerge-20261108/group-wbs`
**Worktree：** `C:/Users/leo19/.codex/worktrees/Star-wbs-postmerge-20261108-group-wbs`
**审查对象：** 本 lane 对 `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` 的 2026-11-08 同步改动
**结论：** **ACCEPT — 改动内容与当前 dev 实测一致；不关闭任何门禁。**

## §0 目的与边界

独立复核本 lane 对 Group WBS 的同步改动是否 (a) 属实反映当前 dev 的真实状态、(b) 保留了候选分支「HOLD、不合并」的处置、(c) 未把任何未验证门禁写成通过、(d) 未改写历史快照节、(e) 未越界改动 lane 登记范围外的文件。

本审查只读核对 Git 对象与文件内容。**未运行 Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release 或 deployment。** 未 fetch、未 push、未改 `main`、未改任何候选分支或 worktree。

## §1 改动清单

| 文件 | 改动 |
|---|---|
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | 新增 §23.22；§2 表 W8 行与 W16 行状态列追加 2026-11-08 事实 |
| `docs/briefs/WBS-POSTMERGE-GROUP-WBS-20261108.md` | 本 lane 的 brief（守门 #20：先于 WBS 改动落盘） |
| `docs/reviews/WBS-POSTMERGE-GROUP-WBS-REVIEW-20261108.md` | 本文件 |
| `docs/reports/PHASE-STAR-STABLE-GROUP-WBS-POSTMERGE-SYNC-20261108.md` | 阶段报告 |
| `docs/reports/WBS-DISPATCH-POSTMERGE-GROUP-WBS-20261108.jsonl` | dispatcher 落档 |

## §2 独立复核清单（逐条实测）

| # | 待证事实 | 复核命令 | 结果 |
|---|---|---|---|
| 1 | W8 source ancestry = 435/9 | `git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025` | ✅ `435  9` |
| 2 | VCD-0 ancestry = 460/5 | `git rev-list --left-right --count dev...codex/vcd0-canonical-detail-test-20261024` | ✅ `460  5` |
| 3 | dev Inbox 仍用 v2 | `git show dev:crates/bi-owner-runtime/src/task_event_inbox.rs \| sed -n '40p;353p'` | ✅ 行 40 = `bi.commit_task_event_v2`；行 353 = `FROM bi.commit_task_event_v2(` |
| 4 | dev quarantine 仍用 v1 | `git show dev:crates/bi-owner-runtime/src/task_event_quarantine.rs \| sed -n '16p;81p'` | ✅ 行 16 = `bi.record_task_event_quarantine_v1`；行 81 = `FROM bi.record_task_event_quarantine_v1(` |
| 5 | 该 lease-order migration 不在 dev | `git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` | ✅ `does not exist in 'dev'` |
| 6 | 候选 `pull_one` REVOKE 后又 GRANT 回 runtime | `git show <candidate>:db/migrations/2026-10-24-platform-native-transport-lease-order.sql \| grep -n 'REVOKE ALL ON FUNCTION platform_transport.pull_one' -A 3` | ✅ 行 108–109 `REVOKE ALL ... FROM PUBLIC, star_platform_transport_runtime;`；行 110–111 `GRANT EXECUTE ... TO star_platform_transport_runtime;` |
| 7 | 候选测试把该授权固化为期望 | `git show <candidate>:crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs \| sed -n '523p'` | ✅ `fn pull_one_execute_is_regranted_only_to_the_runtime_role() {` |
| 8 | `dev:AGENTS.md:36` 仍写 1.99.0 | `git show dev:AGENTS.md \| sed -n '36p'` | ✅ "minimum supported Rust version is 1.99.0" |
| 9 | `dev:rust-toolchain.toml` = 1.99.1 | `git show dev:rust-toolchain.toml \| grep channel` | ✅ `channel = "1.99.1"` |
| 10 | `dev:Cargo.toml:185` = 1.99.1 | `git show dev:Cargo.toml \| sed -n '185p'` | ✅ `rust-version = "1.99.1"` |

**10/10 命中。**

## §3 基线漂移处理（本 lane 的一处自查）

本 lane 起草时以 `dev@131bc958` 实测得 W8 ancestry 434/9、VCD-0 460/5 之前的 459/5。随后 stable-wbs lane 合并进 dev，dev 前移一个提交，实测值变为 **435/9** 与 **460/5**。

处置：本 lane 先 `git merge --ff-only dev` 前进到 `dev@872b5c4a`，再**重测**并据新值改写 §23.22 与 brief；未沿用旧数字。复核清单 #1/#2 是按前进后的基线复算的。

这是本次审查确认的唯一一处数字漂移，已在交付前闭环。若保留旧值，WBS 会记录一个已被自己的合并动作推翻的数字。

## §4 关键纠正：Rust 候选审查中的事实错误

Rust 基线候选审查（`docs/reviews/WBS-MERGE-RUST-BASELINE-CANDIDATE-20261108.md`）称「当前 `AGENTS.md` 已为 1.99.1」。**该陈述不成立**（复核 #8/#9/#10）。

Group WBS §23.15 已记录过「root `AGENTS.md` also states 1.99.0」这一事实；来源审查与 §23.15 相互矛盾。本 lane 采纳 §23.15 的实测结论，并在 §23.22 中显式标注该纠正与来源审查不一致（守门 #11 缺标比错标）。

## §5 处置一致性核对

| 检查项 | 结果 |
|---|---|
| 是否摘取或合并了任何候选提交？ | ❌ 无。§23.22 明确记录三个候选分别 HOLD 2/5/9 个提交，「no commit safe to cherry-pick」 |
| 是否改写了历史快照节？ | ✅ 无。§14.1、§20.1、§23.13、§23.14、§23.21 文本未改动；新增内容全部在 §23.22 与 §2 表两行的状态列末尾 |
| 是否改写了 §23.14 / §23.17 的 ancestry 数字？ | ✅ 无。§23.22 明确写「supersedes only the ancestry figures in §23.14 and §23.17；其记录的理由与引用的历史快照保持不变」——即只声明新基线的实测值优先，不改旧节正文 |
| 是否把 `pull_one` P1 问题写成 dev 既有缺陷？ | ✅ 明确写「This migration does not exist in current dev」「a risk of a newly introduced path, not a pre-existing dev condition」 |
| 是否声称任何未运行的验证已通过？ | ❌ 无。PostgreSQL/RLS/concurrency/durable-ack/recovery、mTLS、commit-before-ACK、Neon/Vercel、Multipass/K3s、性能、PT-09 全部写为仍缺 |
| 是否新增或改变了任何门禁状态？ | ❌ 无。W8 OPEN；W16/WLD-7 OPEN/PARTIAL/NOT_RUN；VCD-0 OPEN；9F4J OPEN/PARTIAL——与改动前一致 |
| 是否越界改动 lane 登记范围外的文件？ | ❌ 无。`AGENTS.md`、`requirements.md`、`basic-design.md`、`DD-WORKTREE-GROUP-001.md`、`main`、任何候选分支 / worktree 均未改动 |
| 是否改 `main` / push / tag / build / deploy？ | ❌ 无 |

## §6 结构完整性

- §2 表：W0–W16 各行分隔竖线数与改动前一致；W8 行 7 个、W16 行 7 个，与同表其它行一致 → 列数未错配。
- 全文表格列数一致性扫描：**0 处错配**。
- 围栏代码块配对：**正确**。
- 尾随空格：**0**。
- `git diff --check`：**clean**。

## §7 已知缺口与限制

1. **只读复核不是执行验证。** 本 lane 未运行 Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release、deployment。相关门禁保持未运行/开放。
2. **ancestry 数字是 Git metadata，不是内容等价证明。** 435/9 与 460/5 说明候选相对当前 dev 的分叉规模；结论另由三份审查的逐提交依赖闭包分析给出。
3. **远端新鲜度未知。** 本轮未 fetch；`origin/dev`、`origin/main` 与 v1.0.6 tag/release 的远端状态仍是历史快照。
4. **`pull_one` P1 问题的运行时严重度未评估。** 只核对了 migration 文本与测试名，未在 PostgreSQL 上执行 grant / 直接调用负例。
5. **`AGENTS.md` 与三份规范文档的 1.99.0 缺口仍未关闭**，修复需单独授权的 lane。
6. **W16 / WLD-7 的规范引用漂移仍在**（spec 与 data-design 仍引用 test-design v1.7；`domain-collaboration-spec.md:303` 三处断链），本 lane 未改这些源文档。

## §8 签字栏

| 角色 | 结论 |
|---|---|
| 架构 | ACCEPT。§23.22 的实测事实与当前 dev 一致，门禁状态未变。 |
| SRE Lead | 未运行任何构建 / 部署 / runtime gate；相关门保持 OPEN。 |
| Platform | W8 `pull_one` P1 授权问题已登记为新引入路径风险；AGENTS.md 1.99.0 漂移缺口已如实登记为未关闭项。 |
| 评审主持 | 10/10 复核项实测命中；基线漂移已自查闭环；历史节未被改写；接受本 lane。 |
| PM | W8 / W16 / WLD-7 / VCD-0 门禁状态全部不变；本报告不授权 tag/release/deploy。 |

## §9 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对本 lane 的 Group WBS 2026-11-08 同步改动做独立只读复核：10 项事实逐条 `git show` / `git rev-list` 实测，纠正来源审查中「AGENTS.md 已为 1.99.1」的不成立陈述，确认历史节未改写、无候选提交被摘取、无门禁被写成通过；结论 ACCEPT。 |
