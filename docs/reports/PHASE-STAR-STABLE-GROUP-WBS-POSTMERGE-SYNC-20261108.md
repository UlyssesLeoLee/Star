# PHASE-STAR-STABLE-GROUP-WBS-POSTMERGE-SYNC-20261108

**Phase**: PHASE-WBS-POSTMERGE-GROUP-WBS-20261108
**Agent**: reviewer
**Lane**: `agent/wbs-postmerge-20261108/group-wbs`
**Baseline**: `dev@872b5c4ad6b6c63e81596c6953fec5274d7782b5`（stable-wbs lane 已合并后的 dev）
**Date**: 2026-11-08
**Brief**: [`docs/briefs/WBS-POSTMERGE-GROUP-WBS-20261108.md`](../briefs/WBS-POSTMERGE-GROUP-WBS-20261108.md)
**Review**: [`docs/reviews/WBS-POSTMERGE-GROUP-WBS-REVIEW-20261108.md`](../reviews/WBS-POSTMERGE-GROUP-WBS-REVIEW-20261108.md)

## §0 结论

**同步完成，门禁状态不变。** 本 lane 把 2026-11-08 三份已集成候选审查的结论同步进 Group WBS（新增 §23.22），并更新 §2 表 W8 / W16 两行的状态列。**W8 / W16 / WLD-7 / VCD-0 / 9F4J 状态全部未变。** 未摘取或合并任何候选提交。

## §1 输入证据

三份审查已在 dev ancestry（commit 已核验）：

| 来源 | dev 中的提交 | 结论 |
|---|---|---|
| Rust 1.99.1 基线候选 | `9937cb8a009fc6b58b6b6576f619a0ec6b891f2a` | 两个提交 HOLD |
| VCD-0 详细/测试设计候选 | `ce752c4a627ca01f754e3a30b1a3f37b0848cae4` | 五个提交 HOLD |
| W8 source-position 候选 | `4b9756cf0879264a322a9f18866586c580b83656` | 九个提交 HOLD |

## §2 同步的实测事实

全部经 `git show` / `git rev-list --left-right --count` 直接复核，非记忆、非转述。

### 2.1 候选 ancestry

| 候选 | 实测命令 | 结果 |
|---|---|---|
| `UlyssesLeoLee/w8-source-position-claim-fix-20261025` | `git rev-list --left-right --count dev...<cand>` | **435 / 9** |
| `codex/vcd0-canonical-detail-test-20261024` | 同上 | **460 / 5** |
| `codex/star-rust-1991-baseline-20261105` | 同上 | 108 / 2（见 §2.4） |

**基线漂移自查**：本 lane 起草时实测 W8 为 434/9、VCD-0 为 459/5；随后 stable-wbs lane 合并进 dev，dev 前移一个提交，实测值变为 435/9 与 460/5。本 lane 先 `git merge --ff-only dev` 前进到 `dev@872b5c4a`，再重测并据新值改写 §23.22 与 brief，**未沿用旧数字**。

### 2.2 W8 API 世代（当前 dev 实测行号）

| 位置 | 内容 |
|---|---|
| `crates/bi-owner-runtime/src/task_event_inbox.rs:40` | `bi.commit_task_event_v2` |
| `crates/bi-owner-runtime/src/task_event_inbox.rs:353` | `FROM bi.commit_task_event_v2(` |
| `crates/bi-owner-runtime/src/task_event_quarantine.rs:16` | `bi.record_task_event_quarantine_v1` |
| `crates/bi-owner-runtime/src/task_event_quarantine.rs:81` | `FROM bi.record_task_event_quarantine_v1(` |

v3 source-origin 返回 schema 与共享 source-position 仲裁**均未进入当前 dev**。

### 2.3 新登记 P1 授权问题：`pull_one`

候选 `db/migrations/2026-10-24-platform-native-transport-lease-order.sql` 第 106–111 行：

```sql
ALTER FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    OWNER TO star_platform_transport_owner;
REVOKE ALL ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM PUBLIC, star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    TO star_platform_transport_runtime;
```

`pull_one` 是 `SECURITY DEFINER` 且固定 search path。REVOKE 后又把 EXECUTE 授回 runtime role，使调用方可直接提供 `app.platform_deployment_id` / `app.platform_tenant_id` / `app.platform_service_id` 设置，绕过认证 wrapper 边界。候选自带测试 `pull_one_execute_is_regranted_only_to_the_runtime_role`（`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:523`）把该不安全授权固化为期望。

**该 migration 在当前 dev 中不存在**（`git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` → `does not exist in 'dev'`），故属**新引入路径的风险**，不是既有 dev 状态。

后续 W8 实现 lane 的强制要求（已写入 §23.22）：移除直接 runtime grant、只保留 wrapper-only grant、以「直接调用失败 / wrapper 调用成功」正负测试证明。

### 2.4 Rust 基线与 AGENTS.md 规范漂移（仍开放）

| 文件 | 实测值 |
|---|---|
| `dev:rust-toolchain.toml` | `channel = "1.99.1"` |
| `dev:Cargo.toml:185` | `rust-version = "1.99.1"` |
| `dev:AGENTS.md:36` | **`minimum supported Rust version is 1.99.0`** |

Rust 候选审查称「当前 AGENTS.md 已为 1.99.1」**不成立**。该规范漂移缺口仍然存在，与 `docs/requirements.md` AC-INFRA-002-R/S/U/V/W、`docs/basic-design.md` §16.44–§16.47、`docs/design/DD-WORKTREE-GROUP-001.md` AC-INFRA-002-U/V 的活动 1.99.0 要求同属待同步项——这些文件超出本 lane 登记范围，未改动。

Rust 候选处置：`c640eb34` 的 workspace/CI/Docker 子集已被 dev 表示 → HOLD；`be154efb` 的版本文字同步只在新登记文档 lane 内逐句适配，不整提交摘取 → HOLD。

### 2.5 门禁状态（全部未变）

| 门 | 状态 |
|---|---|
| W8 | **OPEN** — PostgreSQL / RLS / concurrency / durable-ack / recovery 仍未运行 |
| W16 | **OPEN / PARTIAL / NOT_RUN** — 双边草稿 / 冲突、participant 读取授权、跨实例 delivery / recovery、性能、PT-09 未闭合 |
| WLD-7 | **OPEN / PARTIAL / NOT_RUN** |
| VCD-0 | **OPEN** |
| 9F4J | **OPEN / PARTIAL** |

W8 候选 HOLD 不改变 W16 / WLD-7 已记录的 owner 边界结论（Identity/Auth 拥有撤销事实与其自有 Outbox；Collaboration 经自有 Inbox 消费并可发出仅含元数据的自有 Outbox；无跨 owner 写表、无共享事务）。

## §3 产物

| 文件 | 说明 |
|---|---|
| `docs/briefs/WBS-POSTMERGE-GROUP-WBS-20261108.md` | brief（守门 #20：先于 WBS 改动落盘） |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` | 新增 §23.22 + §2 表 W8 / W16 两行状态列追加 |
| `docs/reviews/WBS-POSTMERGE-GROUP-WBS-REVIEW-20261108.md` | 独立复核（10/10 实测命中） |
| `docs/reports/PHASE-STAR-STABLE-GROUP-WBS-POSTMERGE-SYNC-20261108.md` | 本文件 |
| `docs/reports/WBS-DISPATCH-POSTMERGE-GROUP-WBS-20261108.jsonl` | dispatcher 落档 |

## §4 验证摘要

| 检查 | 结果 |
|---|---|
| 事实复核 | 10 项 `file:line` / ancestry / 存在性断言全部由 `git show` / `git rev-list` 直接实测命中（复核表见 review §2） |
| 基线一致性 | 本 lane 已 ff 到 `dev@872b5c4a` 后重测；§23.22 与 brief 中的数字与该基线一致 |
| 历史节完整性 | §14.1 / §20.1 / §23.13 / §23.14 / §23.21 文本未改动；新增内容仅在 §23.22 与 §2 表两行状态列末尾 |
| 表格结构 | 全文表格列数一致性扫描 **0 处错配**；W8 / W16 行分隔竖线数与同表其它行一致 |
| 格式 | 围栏配对正确；尾随空格 0；`git diff --check` clean |
| 改动范围 | 仅 5 个登记路径；`AGENTS.md`、`requirements.md`、`basic-design.md`、`DD-WORKTREE-GROUP-001.md`、`main`、任何候选分支 / worktree 均未改动 |
| 未运行的验证 | **Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release、deployment 全部未运行。** 只读检查不是执行验证 |

## §5 已知缺口

1. `AGENTS.md` 与三份规范文档的 1.99.0 规范漂移缺口**仍未关闭**，修复需单独授权的 lane。
2. `pull_one` P1 问题的运行时严重度未评估（未在 PostgreSQL 执行 grant / 直接调用负例）。
3. W8 的 PostgreSQL / RLS / concurrency / durable-ack / recovery 门仍 OPEN。
4. W16 / WLD-7 的规范引用漂移仍在（spec 与 data-design 仍引用 test-design v1.7；`domain-collaboration-spec.md:303` 三处断链），本 lane 未改这些源文档。
5. 远端新鲜度未知：本轮未 fetch，v1.0.6 tag/release 的远端状态仍是历史快照。
6. ancestry 数字是 Git metadata，不是内容等价证明。

## §6 守门规则遵守

- 守门 #11（缺标比错标）：纠正来源审查中「AGENTS.md 已为 1.99.1」的不成立陈述，并显式标注该纠正与来源不一致。
- 守门 #12（docs 同步）：brief / review / report / dispatch 同步落档。
- 守门 #13（记录覆盖口径矛盾，未代为改写分类）：只新增当前快照，不改写历史节正文。
- 守门 #14（Mavis 接手审核）：签字栏由 Mavis 接手审核。
- 守门 #20（brief 先落盘）：brief 先于 WBS 改动落盘。
- 候选保护：`codex/star-rust-1991-baseline-20261105`、`codex/vcd0-canonical-detail-test-20261024`、`UlyssesLeoLee/w8-source-position-claim-fix-20261025` 的分支、worktree 与文件保持原样，未读取复制其内容、未摘取其提交。
- `D:\Star` 用户脏文件未触碰；`main` 未改；未 fetch、push、tag、release、build、deploy。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 2026-11-08 三份候选审查结论的 Group WBS 同步：新增 §23.22，W8 ancestry 435/9、VCD-0 460/5、dev Inbox v2 / quarantine v1 实测、新登记 `pull_one` 直接 runtime grant P1 问题、AGENTS.md 1.99.0 规范漂移仍未关闭。W8 / W16 / WLD-7 / VCD-0 / 9F4J 门禁状态全部不变。 |
