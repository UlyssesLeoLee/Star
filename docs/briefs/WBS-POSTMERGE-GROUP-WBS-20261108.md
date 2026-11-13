# Brief: WBS-POSTMERGE-GROUP-WBS-20261108

**Agent**: reviewer
**Phase**: PHASE-WBS-POSTMERGE-GROUP-WBS-20261108
**Created**: 2026-11-08
**Lane**: `agent/wbs-postmerge-20261108/group-wbs`
**Worktree**: `C:/Users/leo19/.codex/worktrees/Star-wbs-postmerge-20261108-group-wbs`
**Base**: `dev@872b5c4ad6b6c63e81596c6953fec5274d7782b5`（stable-wbs lane 已合并后的 dev；本 lane 已 fast-forward 到该基线后再实测 ancestry）

---

## §0 目的

把 2026-11-08 三份已集成到 `dev` 的候选审查结论同步进 Group WBS（`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md`），使 W8 / W16 / WLD-7 / VCD-0 的当前状态与当前 dev 真实状态一致。**只同步已验证事实，不关闭任何门禁。**

## §1 输入证据（已在 dev ancestry，commit 已核验）

| 来源 | dev 中的提交 | 报告 |
|---|---|---|
| Rust 1.99.1 基线候选审查 | `9937cb8a009fc6b58b6b6576f619a0ec6b891f2a` | `docs/reviews/WBS-MERGE-RUST-BASELINE-CANDIDATE-20261108.md` |
| VCD-0 候选审查 | `ce752c4a627ca01f754e3a30b1a3f37b0848cae4` | `docs/reviews/WBS-MERGE-VCD0-CANDIDATE-20261108.md` |
| W8 source 候选审查 | `4b9756cf0879264a322a9f18866586c580b83656` | `docs/reviews/WBS-MERGE-W8-SOURCE-CANDIDATE-20261108.md` |

三份审查一致结论：三个候选分支都无可安全摘取的提交（HOLD 2 / 5 / 9 个提交）。

## §2 本 lane 必须同步的内容（逐条，附当前 dev 实测）

### §2.1 新增 §23.22 — 2026-11-08 候选审查结论与 W8/W16 门禁现状

这是本 lane 的主产出。新节记录：

1. **基线与候选边界**：本 lane 基于 `dev@872b5c4a`；三份审查已合入 dev ancestry；三个候选分支分别 HOLD 2 / 5 / 9 个提交，无一可安全 cherry-pick。
2. **W8 候选 ancestry 更新**：当前实测 `git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025` → **435 / 9**（§23.14、§23.17 记录的是更早基线的 428/9 等旧值）。
3. **VCD-0 候选 ancestry 更新**：当前实测 `git rev-list --left-right --count dev...codex/vcd0-canonical-detail-test-20261024` → **460 / 5**。
4. **W8 API 世代实测**：当前 dev 仍用 `bi.commit_task_event_v2`（`crates/bi-owner-runtime/src/task_event_inbox.rs:40,353`）与 `bi.record_task_event_quarantine_v1`（`crates/bi-owner-runtime/src/task_event_quarantine.rs:16,81`）——v3 source-origin / source-position 仲裁未进入 dev。
5. **新增 P1 授权问题**：候选 `db/migrations/2026-10-24-platform-native-transport-lease-order.sql` 中 `platform_transport.pull_one` 是 `SECURITY DEFINER` + 固定 search path；迁移 REVOKE 后又把 EXECUTE 授回 `star_platform_transport_runtime`，调用方可直接提供 `app.platform_deployment_id` / `app.platform_tenant_id` / `app.platform_service_id` 设置绕过认证 wrapper 边界；候选自带测试 `pull_one_execute_is_regranted_only_to_the_runtime_role`（`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:523`）把该不安全授权固化为期望。**该 migration 在当前 dev 中不存在**（实测 `git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` 报不存在），故属新引入路径的风险而非既有 dev 状态。
6. **Rust 基线与 AGENTS.md 规范漂移**：Rust 候选审查称「当前 AGENTS.md 已为 1.99.1」**不成立**——实测 `dev:AGENTS.md:36` 仍写 1.99.0，而 `dev:rust-toolchain.toml` 为 `channel = "1.99.1"`、`dev:Cargo.toml:185` 为 `rust-version = "1.99.1"`。该规范漂移缺口仍然存在。
7. **门禁状态**：W8 保持 OPEN；W16 / WLD-7 保持 OPEN / PARTIAL / NOT_RUN；VCD-0 保持 OPEN；9F4J 保持 OPEN / PARTIAL。

### §2.2 §2 表 W8 行状态列追加

当前 W8 行写「当前 dev 使用 v2 Inbox API；2026-10-30 候选审计确认…」。需追加 2026-11-08 的实测 ancestry 与 `pull_one` P1 问题，且**不改写** 2026-10-30 的历史快照（守门 #13）。

### §2.3 §2 表 W16 行状态列追加

当前 W16 行已记录 2026-11-06 owner-contract review 与 2026-10-30 的局部 PG 证据。需追加 2026-11-08 复核：W8 候选 HOLD 不改变 W16 的 owner 边界结论；W16/WLD-7 仍 OPEN/PARTIAL/NOT_RUN。

## §3 明确不做的事

- ❌ 不改 `docs/requirements.md`、`docs/basic-design.md`、`docs/design/DD-WORKTREE-GROUP-001.md`、`AGENTS.md`（超出本 lane 登记范围）
- ❌ 不 cherry-pick / 不合并任何候选分支；不读写候选 worktree 的文件
- ❌ 不改 `main`、不 push、不 fetch、不创建 tag/release、不 build/deploy
- ❌ 不把任何门禁写成通过；W8 / W16 / WLD-7 / VCD-0 状态不变
- ❌ 不改写 §14.1、§20.1、§23.13、§23.14 等历史快照节
- ❌ 不改 `D:\Star` 用户脏文件

## §4 验收标准

1. 新增 §23.22 节，含上述 7 项实测事实，每项带可复算的 `git` 命令或 `file:line`。
2. §2 表 W8 行状态列追加 2026-11-08 事实，原 2026-10-30 快照文字保持不变。
3. §2 表 W16 行状态列追加 2026-11-08 复核结论。
4. 表格列数不错配；围栏配对正确；`git diff --check` clean。
5. 全部 WBS 门禁状态未变。

## §5 守门规则

- 守门 #11（缺标比错标）：未经实测的断言不写；纠正来源审查的不成立陈述并显式标注差异。
- 守门 #13（记录覆盖口径矛盾，未代为改写分类）：只新增当前快照，不改写历史节。
- 守门 #20（brief 先落盘）：本 brief 先于 WBS 改动落盘，commit message 引用本路径。
- 守门 #12（docs 同步）：brief / review / report / dispatch 同步落档。
- 守门 #14（Mavis 接手审核）：签字栏由 Mavis 接手审核。
- 只读检查不是验证：未运行的 Cargo/CI/PG/VM/hosted gate 保持未运行/开放。

## §6 独立复核清单（review 阶段逐条核对）

1. 2 条 ancestry 数字（434/9、459/5）逐条 `git rev-list --left-right --count` 复算
2. 4 条 `file:line` 引用（`task_event_inbox.rs:40,353`、`task_event_quarantine.rs:16,81`、候选 `native_origin_verification_pg.rs:523`、`dev:AGENTS.md:36`）逐条 `git show` 复核
3. 1 条 migration 不存在性断言逐条 `git show dev:<path>` 复核
4. 1 条 candidate `pull_one` REVOKE→GRANT 模式复核（`git show <candidate>:<path>`）
5. 历史节（§14.1 / §20.1 / §23.13 / §23.14）文本未被改动
6. W8 / W16 / WLD-7 / VCD-0 状态词未被改动
7. `git diff --check` 通过；表格列数一致

## §7 Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
