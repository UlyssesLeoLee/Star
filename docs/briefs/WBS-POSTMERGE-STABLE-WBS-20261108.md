# Brief: WBS-POSTMERGE-STABLE-WBS-20261108

**Agent**: reviewer
**Phase**: PHASE-WBS-POSTMERGE-STABLE-WBS-20261108
**Created**: 2026-11-08
**Lane**: `agent/wbs-postmerge-20261108/stable-wbs`
**Worktree**: `C:/Users/leo19/.codex/worktrees/Star-wbs-postmerge-20261108-stable-wbs`
**Base**: `dev@131bc9581badcc214add9a29a490ff0241fa2e0f`

---

## §0 目的

把 2026-11-08 三份已集成到 `dev` 的候选审查结论同步进 Stable Release WBS，使 WBS 的当前快照与当前 dev 真实状态一致。**只同步已验证事实，不关闭任何发行门禁。**

## §1 输入证据（已在 dev ancestry，commit 已核验）

| 来源 | dev 中的提交 | 报告 |
|---|---|---|
| Rust 1.99.1 候选审查 | `9937cb8a009fc6b58b6b6576f619a0ec6b891f2a` | `docs/reviews/WBS-MERGE-RUST-BASELINE-CANDIDATE-20261108.md` |
| VCD-0 候选审查 | `ce752c4a627ca01f754e3a30b1a3f37b0848cae4` | `docs/reviews/WBS-MERGE-VCD0-CANDIDATE-20261108.md` |
| W8 source 候选审查 | `4b9756cf0879264a322a9f18866586c580b83656` | `docs/reviews/WBS-MERGE-W8-SOURCE-CANDIDATE-20261108.md` |

三份审查的结论一致：**三个候选分支都没有可直接摘取的提交**（HOLD）。但审查同时暴露了 WBS 当前快照中的若干陈述已过时或与 dev 实际内容不符。

## §2 本 lane 必须同步的内容（逐条，附当前 dev 实测）

### W2 行 — Rust 基线与 AGENTS.md 事实更正

审查 §1.4 与 §3 第 1 项指出「当前 `AGENTS.md` 已为 1.99.1」。**实测该陈述不成立**：

```
git show dev:AGENTS.md | sed -n '36p'
→ **Rust toolchain baseline**：STAR workspace minimum supported Rust version is 1.99.0. ...
```

`dev:AGENTS.md:36` 仍写 **1.99.0**，而 `dev:rust-toolchain.toml` 为 `channel = "1.99.1"`、`dev:Cargo.toml:185` 为 `rust-version = "1.99.1"`。这是**规范漂移缺口**，不是历史记录。WBS 必须记录该缺口仍然存在，不得写成已解决。

同时：审查 §1.4 记录的 Rust 基线事实（workspace/toolchain/CI 已是 1.99.1）经复核成立；`dev` 中 `docs/requirements.md`、`docs/basic-design.md`、`docs/design/DD-WORKTREE-GROUP-001.md` 的活动 1.99.0 规范性要求仍存在。

### W6 行 — VCD-0 候选 ancestry 数字更新

WBS v1.58 记录 VCD-0 基线为 `dev@6634a455`，Git metadata 392/5。当前 dev 实测：

```
git rev-list --left-right --count dev...codex/vcd0-canonical-detail-test-20261024
→ 459  5
```

候选仍为 5 个独有提交，仍不接受。W6 维持 ADAPT/OPEN。

### W8 行 — W8 source 候选 ancestry 数字更新 + P1 授权问题登记

WBS v1.58 与 Group WBS §23.14/§23.17 记录 W8 候选基线数字。当前 dev 实测：

```
git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025
→ 434  9
```

候选仍为 9 个独有提交，仍无可安全摘取项。

**新增 P1 授权问题（审查 §3.3，此前 WBS 未登记）**：候选的 `db/migrations/2026-10-24-platform-native-transport-lease-order.sql` 中 `platform_transport.pull_one` 是 `SECURITY DEFINER` 函数且固定 search path；迁移先 REVOKE 后又把 EXECUTE 授回 `star_platform_transport_runtime`，使调用方可直接提供 `app.platform_deployment_id` / `app.platform_tenant_id` / `app.platform_service_id` 设置，绕过认证 wrapper 边界。候选自带测试 `pull_one_execute_is_regranted_only_to_the_runtime_role` 把该不安全授权固化为期望。

实测确认：该 migration **在当前 dev 中不存在**（`git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` → `does not exist in 'dev'`），因此这是**新引入路径的风险**，不是既有 dev 状态。当前 dev 的 Inbox 仍用 `bi.commit_task_event_v2`、quarantine 仍用 `bi.record_task_event_quarantine_v1`（`crates/bi-owner-runtime/src/task_event_inbox.rs:40,353`、`task_event_quarantine.rs:16,81`）。

### W1 行 — 根 Cargo metadata 与 MIT 决策的冲突复核

实测：`dev:Cargo.toml:187` 仍为 `license = "Apache-2.0"`，`dev:LICENSE` 正文为 `MIT License`（Copyright (c) 2026 Ulysses Leo Lee）。用户已选 MIT，根 workspace 身份冲突未解 → W1 维持 BLOCKED。此为既有事实复核，不新增结论。

## §3 明确不做的事

- ❌ 不改 `AGENTS.md`、`docs/requirements.md`、`docs/basic-design.md`、`docs/design/DD-WORKTREE-GROUP-001.md`（超出本 lane 登记范围，且各自有 owner）
- ❌ 不 cherry-pick / 不合并任何候选分支（`codex/star-rust-1991-baseline-20261105`、`codex/vcd0-canonical-detail-test-20261024`、`UlyssesLeoLee/w8-source-position-claim-fix-20261025`）
- ❌ 不读写候选 worktree 的任何文件
- ❌ 不改 `main`、不 push、不 fetch、不创建 tag/release、不 build/deploy
- ❌ 不把任何 gate 写成通过；W0–W9 与 v1.0.6 维持 NO-GO
- ❌ 不改 `D:\Star` 用户脏文件

## §4 验收标准

1. WBS 顶部新增一条 2026-11-08 同步快照段，声明基于 `dev@131bc958`。
2. W2 行记录 `AGENTS.md` 仍为 1.99.0 的规范漂移缺口（含 `dev:AGENTS.md:36` 定位）。
3. W6 行记录 VCD-0 ancestry 459/5 与「仍不接受候选提交」。
4. W8 行记录 W8 source ancestry 434/9、`pull_one` 直接授权 P1 问题、该 migration 在 dev 中不存在、Inbox 仍 v2 / quarantine 仍 v1。
5. W1 行记录根 Cargo metadata Apache-2.0 与 LICENSE MIT 的冲突仍存在。
6. 修订历史新增 v1.59 行。
7. 全部 W0–W9 状态与 v1.0.6 NO-GO 不变。
8. `git diff --check` 通过；markdownlint 0 issue（若工具可用）。

## §5 守门规则

- 守门 #11（缺标比错标）：未经实测的断言不写；AGENTS.md 事实以 `git show dev:AGENTS.md` 为准。
- 守门 #13（记录覆盖口径矛盾，未代为改写分类）：不改写既有分类数字，只新增当前快照。
- 守门 #20（brief 先落盘）：本 brief 先于 WBS 改动落盘，commit message 引用本路径。
- 守门 #12（docs 同步）：本 lane 的 brief/review/report 同步落档。
- 守门 #14（Mavis 接手审核）：签字栏由 Mavis 接手审核。
- 只读检查不是验证：未运行的 Cargo/CI/PG/VM/hosted gate 保持未运行/开放。

## §6 独立复核清单（review 阶段逐条核对）

1. 4 条 `file:line` 引用（`dev:AGENTS.md:36`、`dev:Cargo.toml:187`、`crates/bi-owner-runtime/src/task_event_inbox.rs:40`、`task_event_quarantine.rs:16`）逐条 `git show dev:<path>` 复核
2. 2 条 ancestry 数字（459/5、434/9）逐条 `git rev-list --left-right --count` 复算
3. 1 条 migration 不存在性断言逐条 `git show dev:<path>` 复核
4. WBS 全部 W0–W9 状态行未被改动
5. `git diff --check` 通过

## §7 Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
