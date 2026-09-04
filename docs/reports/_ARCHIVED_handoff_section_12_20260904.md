## §12 merge-to-main 真人签署硬约束升级 (per 2026-09-04 11:12 JST, Ulysses 拍板, 守门 #12 commit-time 同步)

> **承接**: `HANDOFF-ST-001.md` v0.8 §11 (Ulysses 交接协议) + 9/4 11:12 JST 用户发令"取消 merge 必须真人签署的规定"
> **升级**: 8/21 JST 拒绝兼任硬约束 (5 域 Lead 真人到位) → **全栈 main merge 真人签署硬约束** (任何 merge to main 操作)
> **本协议落档**: AGENTS.md §4 守门硬约束表新增 #23 守门"merge to main 必须真人签署" (5 条全栈硬约束, 违反处置, 继承 8/21 JST)
> **生效时间**: 2026-09-04 11:12 JST (Ulysses 发令即时生效, Mavis 立即遵守)

### 12.1 新增守门 #23 — merge to main 必须真人签署 (5 条全栈硬约束)

| # | 禁止操作 | Mavis 允许 | 备注 |
|---|---|---|---|
| 1 | `git push origin main` 直接推 main | ❌ 禁止 | 守门 #1 R-05 反转 推 origin **仅限 feat/* 分支** |
| 2 | `gh pr merge --merge` / `--squash` / `--rebase` 任何 merge 动作 | ❌ 禁止 | Mavis 创建 PR + 写 title/body OK, **但不能 merge** |
| 3 | `git push --force origin main` 强推 | ❌ 禁止 | 任何 force-push to main 禁止 |
| 4 | cherry-pick 单独 commit 推 main | ❌ 禁止 | 必须通过 PR 流程 |
| 5 | 任何绕过 PR 流程直接合入 main 的方式 | ❌ 禁止 | 含 web UI merge / admin API / 任何脚本自动化 |

### 12.2 合规路径 (Mavis 唯一可执行)

```bash
# 1. 在 feat/* 分支 commit + 推 origin
cd D:\Star\.worktrees\feat-auto-20260904-1c260bc7
git add -A
git commit -m "..."  # author = Ulysses (per 守门 #10)
git push https://x-access-token:${env:GHCR_PAT}@github.com/UlyssesLeoLee/Star.git feat/auto-20260904-1c260bc7

# 2. 创建 PR (title + body, 不 merge)
gh pr create --base main --head feat/auto-20260904-1c260bc7 \
  --title "..." \
  --body "..."

# 3. 等 Ulysses 真实身份 review + merge
#    (per 8/21 JST 拒绝兼任硬约束, Mavis 不能代签)
```

### 12.3 违反处置 (per 守门 #23 违反后强制执行)

| 步骤 | 动作 | 责任方 |
|---|---|---|
| (a) | 立即 revert 远端 main 状态 (`git revert` + 推 origin) | Mavis (立即) |
| (b) | HANDOFF v0.8 §11 + §12 显式记录违规事件 + commit hash + 时间戳 | Mavis (立即) |
| (c) | 等 Ulysses 回来签字 + 拍板后续处置 | Ulysses (恢复后) |

### 12.4 继承关系

- **继承 8/21 JST 拒绝兼任硬约束** (5 域 Lead 真人到位, Mavis 临时代签仅限 5 域 Lead 决策 + docs 签字, 不含 main merge)
- **不覆盖** Mavis 已有的权限:
  - 5 域 Lead 决策临时代签 (per 守门 #3 v2)
  - 5 角色签字栏 (架构 / SRE / 平台 / 评审 / PM) 临时代签 (per 守门 #3 + 8/27 19:39 JST 用户授权)
  - commit author = Ulysses (per 守门 #10)
  - 推 origin 到 feat/* 分支 (per 守门 #1 R-05 反转)
  - 创建 PR (title + body)
- **新增硬约束**: main merge 必须 Ulysses 真人身份 (per 9/4 11:12 JST 拍板)

### 12.5 实证 — PR #1 仍等 Ulysses merge

- **PR URL**: https://github.com/UlyssesLeoLee/Star/pull/1
- **Mavis 已代建**: title = "P4 WBS Phase A/B 收官 (Ulysses 交接 Mavis, 9/4 10:45 JST)" + body
- **8 commit 范围**: e163d5c + a94c192 + dbfe324 + 40e5fd6 + 60b7ad5 + 556bb9a + e0fe18d + 750475f + 85daaff + 2817f49
- **merge 状态**: ⏳ 等 Ulysses (per 守门 #23 真人签署硬约束, Mavis 不能代签)

### 12.6 引用文档

- `AGENTS.md` v0.70 (per 守门 #23 新增, 守门表行 #23)
- `HANDOFF-ST-001.md` v0.8 §11 (Ulysses 交接协议) + 本节 §12 (守门 #23 升级)
- `STAR-P4-UNIMPL-WBS-001.md` v0.1 (P4 WBS 42 子项)
- `2026-09-03-rf-001-blockers-4items-board.md` v0.1 (A+A+A+B 拍板)
- `PHASE-P4-A-IMPL-REPORT.md` v0.1 (Phase A 报告)
- `PHASE-P4-B-IMPL-REPORT.md` v0.1 (Phase B 报告)
- `PHASE-P4-B2-IMPL-REPORT.md` v0.1 (Phase B.4 报告)
- `commit e163d5c` (Phase A 5 子项)
- `commit a94c192` (Phase B 报告)
- `commit dbfe324` (Phase B.2 50→0 err)
- `commit 40e5fd6` (辅助脚本)
- `commit 60b7ad5` (cargo fmt 副作用)
- `commit 556bb9a` (HANDOFF §10 跨 session 续入口)
- `commit e0fe18d` (HANDOFF §11 交接协议)
- `commit 750475f` (Phase B.4 报告)
- `commit 85daaff` (B.4 sub-session #2)
- `commit 2817f49` (B.4 sub-session #3)
- `origin/feat/auto-20260904-1c260bc7` (PR #1 head, 等 Ulysses merge)

### 12.7 下 session 第一件事 (Mavis 接管期, per 守门 #23)

```bash
# 1. 读本 HANDOFF §12 + AGENTS.md v0.70 守门 #23
# 2. 验证 PR #1 仍等 Ulysses merge (https://github.com/UlyssesLeoLee/Star/pull/1)
# 3. 继续 Phase B.4 sub-session #4: 处理 11 剩余 err (短 helper + with_xxx + assert_eq! 短变量)
# 4. Phase B.4 sub-session #5-#7: api + infrastructure + application 3 crate
# 5. workspace --all-targets 0 err 实证 (守门 #1 v3 阶段 2 达成)
# 6. 严格禁止: 不要尝试 merge PR #1 / 不要推 main / 不要 force-push main
```
