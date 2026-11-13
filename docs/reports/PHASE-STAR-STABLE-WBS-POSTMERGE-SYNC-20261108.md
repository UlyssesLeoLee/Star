# PHASE-STAR-STABLE-WBS-POSTMERGE-SYNC-20261108

**Phase**: PHASE-WBS-POSTMERGE-STABLE-WBS-20261108
**Agent**: reviewer
**Lane**: `agent/wbs-postmerge-20261108/stable-wbs`
**Baseline**: `dev@131bc9581badcc214add9a29a490ff0241fa2e0f`
**Date**: 2026-11-08
**Brief**: [`docs/briefs/WBS-POSTMERGE-STABLE-WBS-20261108.md`](../briefs/WBS-POSTMERGE-STABLE-WBS-20261108.md)
**Review**: [`docs/reviews/WBS-POSTMERGE-STABLE-WBS-REVIEW-20261108.md`](../reviews/WBS-POSTMERGE-STABLE-WBS-REVIEW-20261108.md)

## §0 结论

**同步完成，门禁状态不变。** 本 lane 把 2026-11-08 三份已集成候选审查的结论同步进 Stable Release WBS，并记录它们暴露的当前 dev 实测事实。**W0–W9 状态全部未变，v1.0.6 保持 NO-GO。** 未摘取或合并任何候选提交。

## §1 输入证据

三份审查已在 dev ancestry（commit 已核验）：

| 来源 | dev 中的提交 | 结论 |
|---|---|---|
| Rust 1.99.1 基线候选 | `9937cb8a009fc6b58b6b6576f619a0ec6b891f2a` | 两个提交 HOLD |
| VCD-0 详细/测试设计候选 | `ce752c4a627ca01f754e3a30b1a3f37b0848cae4` | 五个提交 HOLD |
| W8 source-position 候选 | `4b9756cf0879264a322a9f18866586c580b83656` | 九个提交 HOLD |

## §2 本次同步的实测事实

全部经 `git show` / `git rev-list --left-right --count` 直接复核，非记忆、非转述。

### 2.1 W2 — Rust 基线与 AGENTS.md 规范漂移（仍开放）

| 文件 | 实测值 |
|---|---|
| `dev:rust-toolchain.toml` | `channel = "1.99.1"` |
| `dev:Cargo.toml:185` | `rust-version = "1.99.1"` |
| `dev:AGENTS.md:36` | **`minimum supported Rust version is 1.99.0`** |

workspace/toolchain 基线确已为 1.99.1，但 `AGENTS.md` 仍为 1.99.0。Rust 候选审查称「当前 AGENTS.md 已为 1.99.1」**不成立**；该规范漂移缺口仍然存在，与 `docs/requirements.md` AC-INFRA-002-R/S/U/V/W、`docs/basic-design.md` §16.44–§16.47、`docs/design/DD-WORKTREE-GROUP-001.md` AC-INFRA-002-U/V 的活动 1.99.0 要求同属待同步项。这些文件超出本 lane 登记范围，未改动。

Rust 候选处置：`c640eb34` 的 workspace/CI/Docker 子集已被 dev 表示 → HOLD；`be154efb` 的版本文字同步只在新登记文档 lane 内逐句适配，不整提交摘取 → HOLD。

### 2.2 W6 — VCD-0 候选 ancestry 459/5

```
git rev-list --left-right --count dev...codex/vcd0-canonical-detail-test-20261024
→ 459  5
```

候选五个独有提交全部 HOLD。候选 DD v0.1 / TD v0.1 落后于当前 dev 的 DD v0.3 / TD v0.4；两个 brief 属一次性任务说明。候选未提供 Neon 目标 profile、hostless Run、0..N Repository、解绑前迁移、推送协作草稿/冲突验收的完整映射。W6 维持 ADAPT/OPEN。

### 2.3 W8 — W8 source 候选 ancestry 434/9 + `pull_one` P1 授权问题

```
git rev-list --left-right --count dev...UlyssesLeoLee/w8-source-position-claim-fix-20261025
→ 434  9
```

九个候选提交无一可安全 cherry-pick：首个实现提交同时拥有 source-origin API、adapter、Inbox/quarantine 调用方、两份 migration 与授权变更，后续提交均依赖该契约与共享 migration。

当前 dev 的 API 世代（实测行号）：

| 位置 | 内容 |
|---|---|
| `crates/bi-owner-runtime/src/task_event_inbox.rs:40` | `bi.commit_task_event_v2` |
| `crates/bi-owner-runtime/src/task_event_inbox.rs:353` | `FROM bi.commit_task_event_v2(` |
| `crates/bi-owner-runtime/src/task_event_quarantine.rs:16` | `bi.record_task_event_quarantine_v1` |
| `crates/bi-owner-runtime/src/task_event_quarantine.rs:81` | `FROM bi.record_task_event_quarantine_v1(` |

**新登记 P1 授权问题**：候选 `db/migrations/2026-10-24-platform-native-transport-lease-order.sql` 第 106–111 行：

```sql
ALTER FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    OWNER TO star_platform_transport_owner;
REVOKE ALL ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM PUBLIC, star_platform_transport_runtime;
GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    TO star_platform_transport_runtime;
```

`pull_one` 是 `SECURITY DEFINER` 且固定 search path。REVOKE 后又把 EXECUTE 授回 runtime role，使调用方可直接提供 `app.platform_deployment_id` / `app.platform_tenant_id` / `app.platform_service_id` 设置，绕过认证 wrapper 边界。候选自带测试 `pull_one_execute_is_regranted_only_to_the_runtime_role`（`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs:523`）把该不安全授权固化为期望。

**该 migration 在当前 dev 中不存在**（`git show dev:db/migrations/2026-10-24-platform-native-transport-lease-order.sql` → `does not exist in 'dev'`），故属新引入路径的风险，不是既有 dev 状态。WBS 已要求后续 W8 实现 lane 必须：移除直接 runtime grant、只保留 wrapper-only grant、以「直接调用失败 / wrapper 调用成功」正负测试证明。W8 保持 OPEN。

### 2.4 W1 — 根 Cargo metadata 与 MIT 决策冲突（仍存在）

| 文件 | 实测值 |
|---|---|
| `dev:Cargo.toml:187` | `license = "Apache-2.0"` |
| `dev:LICENSE` | `MIT License`（Copyright (c) 2026 Ulysses Leo Lee） |

用户已选 MIT，根 workspace 身份冲突未解 → W1 维持 BLOCKED。本次只复核该事实，未改写 metadata。

## §3 产物

| 文件 | 说明 |
|---|---|
| `docs/briefs/WBS-POSTMERGE-STABLE-WBS-20261108.md` | brief（守门 #20：先于 WBS 改动落盘） |
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` | 顶部同步快照段 + W1/W2/W6/W8 四行状态列追加 + 修订历史 v1.59 |
| `docs/reviews/WBS-POSTMERGE-STABLE-WBS-REVIEW-20261108.md` | 独立复核（12/12 实测命中） |
| `docs/reports/PHASE-STAR-STABLE-WBS-POSTMERGE-SYNC-20261108.md` | 本文件 |
| `docs/reports/WBS-DISPATCH-POSTMERGE-STABLE-WBS-20261108.jsonl` | dispatcher 落档 |

## §4 验证摘要

| 检查 | 结果 |
|---|---|
| 事实复核 | 12 项 `file:line` / ancestry / 存在性断言全部由 `git show` / `git rev-list` 直接实测命中（复核表见 review §2） |
| 表格结构 | W0–W9 每行 7 个分隔竖线（与改动前一致）；修订历史每行 6 个（与表头一致） |
| `git diff --check` | clean |
| 改动范围 | 仅 5 个登记路径；`AGENTS.md`、`requirements.md`、`basic-design.md`、`DD-WORKTREE-GROUP-001.md`、`main`、任何候选分支 / worktree 均未改动 |
| 未运行的验证 | **Cargo、CI、PostgreSQL、Multipass/K3s、Vercel/Neon、hosted workflow、build、artifact、release、deployment 全部未运行。** 只读检查不是执行验证 |

## §5 已知缺口

1. `AGENTS.md` 的 1.99.0 规范漂移缺口**仍未关闭**，修复需单独授权的 lane。
2. `docs/requirements.md` / `docs/basic-design.md` / `docs/design/DD-WORKTREE-GROUP-001.md` 的活动 1.99.0 要求同样未关闭。
3. `pull_one` P1 问题的运行时严重度未评估（未在 PostgreSQL 执行 grant / 直接调用负例）。
4. W8 的 PostgreSQL/RLS/concurrency/durable-ack/recovery 门仍 OPEN。
5. 远端新鲜度未知：本轮未 fetch，v1.0.6 tag/release 的远端状态仍是历史快照。
6. ancestry 数字是 Git metadata，不是内容等价证明。

## §6 守门规则遵守

- 守门 #11（缺标比错标）：纠正来源审查中「AGENTS.md 已为 1.99.1」的不成立陈述，并显式标注该纠正与来源不一致。
- 守门 #12（docs 同步）：brief / review / report / dispatch 同步落档。
- 守门 #13（记录覆盖口径矛盾，未代为改写分类）：只新增当前快照，不改写既有分类数字。
- 守门 #14（Mavis 接手审核）：签字栏由 Mavis 接手审核。
- 守门 #20（brief 先落盘）：brief 先于 WBS 改动落盘。
- 候选保护：`codex/star-rust-1991-baseline-20261105`、`codex/vcd0-canonical-detail-test-20261024`、`UlyssesLeoLee/w8-source-position-claim-fix-20261025` 的分支、worktree 与文件保持原样，未读取复制其内容、未摘取其提交。
- `D:\Star` 用户脏文件未触碰；`main` 未改；未 fetch、push、tag、release、build、deploy。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-11-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 2026-11-08 三份候选审查结论的 WBS 同步：W2 的 AGENTS.md 1.99.0 规范漂移（纠正来源审查陈述）、W6 的 VCD-0 ancestry 459/5、W8 的 source ancestry 434/9 与 `pull_one` 直接 runtime grant P1 问题、W1 的根 metadata Apache-2.0 与 LICENSE MIT 冲突。门禁状态全部不变，v1.0.6 保持 NO-GO。 |
