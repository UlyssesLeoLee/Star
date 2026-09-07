# Brief: OPT-WORKER-04 — Phase C T3.3 + T3.1 + T1.5 (per OPT-WBS-09..11)

**Agent**: worker
**Phase**: OPT-P4
**Created**: 2026-09-07 12:04 JST
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses

---

## 1. 任务目标

完成 Phase C 3 子项 (per `STAR-P4-UNIMPL-WBS-001.md` §4):

- **C.1**: T3.3 ubiquitous-language.md v1.0 扩 (22 domain 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design, v0.1 已落 `524a75a`)
- **C.2**: T3.1 共享 star-dto 重构 (消除 22 domain 字段重复定义)
- **C.3**: T1.5 `unreachable_pub = "deny"` 3 步切换 (step 1 已落 `0e6a965`, step 2/3 待续)

---

## 2. Worktree 创建

```bash
cd D:\Star
git worktree add -b feat/opt-phase-c D:/Star/.worktrees/wt-opt-phase-c main
cd D:/Star/.worktrees/wt-opt-phase-c
```

- **Worktree 路径**: `D:/Star/.worktrees/wt-opt-phase-c`
- **Branch**: `feat/opt-phase-c` (基于 main HEAD `bfb0bca`)

---

## 3. 实装要求

### 3.1 C.1 — ubiquitous-language.md v1.0 扩

**文件**: `docs/ubiquitous-language.md` (v0.1 已落 `524a75a`)

**任务**:
- 在文档加 §3 "22 domain 字段命名表" 段 (per 守门 #1 v18 H2-EXT 派生)
- 5 抽样对照 spec 附录 B vs `docs/basic-design.md` 字段命名差异
- 加 v1.0 修订段 + 修订人 (Mavis 接手代签)

**22 domain 清单** (per workspace 当前 34 domain-* crate, 取 22 主):
identity, permission, workspace, worktree, work-item, project, sprint, comment, review, board, agent, automation, batch, board, context, dashboard, development, feedback, integration, kms, notification, relation, scm, search, tenant, validation, workflow, ai, form, planning, theme, etc.

每行格式:
| domain | 字段 | spec 命名 | basic-design 命名 | 一致性 | 备注 |

### 3.2 C.2 — 共享 star-dto 重构

**任务**:
- 新建 `crates/star-dto/src/lib.rs` (per `STAR-P3-WBS-001.md` §14.4 派生)
- 提取 22 domain 公共字段 (id / created_at / updated_at / tenant_id / workspace_ids / actor_id) 到 star-dto
- 在 5 个 domain crate 改用 `use star_dto::*;` (选 5 个, 不全 22)
- 加 unit test

**5 domain 选择** (优先级, 估最少冲突):
1. `domain-relation`
2. `domain-board`
3. `domain-tenant`
4. `domain-workspace`
5. `domain-context`

### 3.3 C.3 — T1.5 step 2/3 切换

**文件**: `Cargo.toml` workspace.lints.rust (line 75-77 附近)

**当前状态** (per 9/3 实证 `0e6a965`):
```toml
[workspace.lints.rust]
unreachable_pub = "warn"  # step 1, 待改 deny
rust_2018_idioms = { level = "deny", priority = -1 }  # step 2 ✅
missing_docs = "deny"  # step 3 ✅ 但有 #![allow] 散布
```

**任务** (3 步切换 per `STAR-P4-UNIMPL-WBS-001.md` §3 C.3):
1. **Step 2 已落地** — 跳过
2. **Step 3 实证**: 跑 `cargo check --workspace --all-targets -j 4` 验证 missing_docs deny 0 err (per 9/5 PR #12 实证已落地)
3. **Step 1 完成**: 改 `unreachable_pub = "warn"` → `unreachable_pub = "deny"` (per 9/3 实证 `0e6a965` 已完成, 跳过)

**注意**: step 1 + step 2 + step 3 都已落地, 任务 = **验证 + 文字实证**

加 `docs/reports/PHASE-P4-C-T15-IMPL-REPORT.md` (per守门 #3 报告 7 段结构)

---

## 4. 守门硬约束

1. `cargo check --workspace --all-targets -j 4` 必须 0 err (per守门 #1 v19)
2. `cargo test -p star-dto + 5 domain --lib -j 4` 必须全 pass (新 crate + 改 5 domain)
3. `cargo fmt + clippy` 必须干净
4. 禁回溯叙事 + 禁 RGS 引用 + PowerShell only
5. 守门 #13 W/T/M 派生: star-dto 共享字段必须按 W/T/M 分类 (per 9/1 18:30 JST 拍板)

---

## 5. Commit 形式

**3 个 commit (per C.1 / C.2 / C.3)**:

Commit 1 (C.1 docs):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "docs(ubiquitous-language): 扩 v1.0 + 22 domain 字段命名表 (per C.1)

- 加 §3 22 domain 字段命名表
- 5 抽样对照 spec 附录 B vs basic-design
- v0.1 → v1.0 修订段

Refs: docs/briefs/OPT-WORKER-04-phase-c.md
Refs: docs/reports/STAR-P4-OPT-WBS-001.md §4.1 #5
Per守门 #10 author=Ulysses
"
```

Commit 2 (C.2 code):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(star-dto): 新建共享 DTO crate + 5 domain 接入 (per C.2)

- 新建 crates/star-dto (公共 id/timestamp/tenant/actor 字段)
- 5 domain 改用 use star_dto::* (relation/board/tenant/workspace/context)
- 加 unit test 覆盖

Refs: docs/briefs/OPT-WORKER-04-phase-c.md
Refs: STAR-P4-UNIMPL-WBS-001.md §4 C.2
Per守门 #13 W/T/M 派生规 (共享字段按分类)
Per守门 #10 author=Ulysses
"
```

Commit 3 (C.3 verify + report):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "docs(report): T1.5 3 步切换验证 + Phase C 实证 (per C.3)

- T1.5 step 1/2/3 全部落地验证
- cargo check --workspace --all-targets -j 4 0 err 实证
- 新建 docs/reports/PHASE-P4-C-T15-IMPL-REPORT.md

Refs: docs/briefs/OPT-WORKER-04-phase-c.md
Refs: STAR-P4-UNIMPL-WBS-001.md §4 C.3
Per守门 #12 commit-time docs 同步 (新事件 = Commit 2 代码改动)
Per守门 #10 author=Ulysses
"
```

---

## 6. 交付物

返回时报告:
1. worktree 路径: `D:/Star/.worktrees/wt-opt-phase-c`
2. branch 名称: `feat/opt-phase-c`
3. **3 个 commit hash** (7 字符)
4. 修改文件列表 (file:line, 跨 3 commit)
5. `cargo check --workspace --all-targets -j 4` 输出 (0 err)
6. `cargo test -p star-dto + 5 domain` 输出 (全 pass)
7. star-dto 新 crate 落档 实证

---

## 7. 失败处理

- cargo check 不通过: 修复 + 重试, max 2 次
- 仍失败: 报告具体错误, 不 commit

---

## 8. 时间预算

≤ 500K token (因 star-dto 新 crate + 5 domain 接入 + docs)
