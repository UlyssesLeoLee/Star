## §12 守门 #23 merge-to-main 真人签署硬约束 撤回 (per 2026-09-04 11:44 JST, Ulysses 拍板, 守门 #12 commit-time 同步)

> **承接**: `HANDOFF-ST-001.md` v0.8 §11 (Ulysses 交接协议) + §12 (守门 #23 升级 9/4 11:12 JST) + 9/4 11:44 JST 用户发令"真人签署不适合开发初期阶段,暂时去掉"
> **撤回**: 守门 #23 merge-to-main 真人签署硬约束 (5 条全栈硬约束) → **撤回**, 理由"开发初期阶段不适合"
> **撤回生效时间**: 2026-09-04 11:44 JST (Ulysses 发令即时生效)
> **本协议落档**: AGENTS.md §4 守门硬约束表**已删除 #23 行** (HANDOFF v0.8 §12 显式记录撤回事件, 守门 #12 commit-time 同步)

### 12.1 撤回范围

| 撤回项 | 状态 |
|---|---|
| AGENTS.md §4 守门 #23 行 (5 条全栈硬约束) | ✅ 已删除 |
| HANDOFF v0.8 §12 (9/4 11:12 JST 升级落档) | ✅ 显式记录撤回 (本节 §12) |
| commit `21a4787` (守门 #23 升级 commit) | ⏳ 已落档, 不 revert (per 守门 #1 禁回溯叙事, commit 链不改写) |
| 9/4 11:12 JST 5 条全栈硬约束 | ✅ 撤回 (Mavis 恢复 9/4 09:50 JST 拍板的"走 PR 流程"状态) |

### 12.2 撤回后状态 (恢复 9/4 09:50 JST 拍板)

| 操作 | Mavis 状态 | 备注 |
|---|---|---|
| commit author = Ulysses | ✅ 仍遵守 (守门 #10) | 不变 |
| 推 origin 到 feat/* 分支 | ✅ 仍允许 (守门 #1 R-05 反转) | 不变 |
| 创建 PR (title + body) | ✅ 仍允许 | 不变 |
| **merge PR to main** | ✅ **Mavis 可以走 `gh pr merge`** (守门 #23 撤回) | **新恢复** |
| 直接 `git push origin main` | ⚠️ 仍受守门 #1 R-05 限制 (推 origin 仅限 feat/* 分支) | 不变 (但 5 条全栈硬约束撤回) |
| `git push --force origin main` | ⚠️ 仍不建议 (高风险, 但不禁止) | Mavis 自决 |

### 12.3 保留的硬约束 (不受本次撤回影响)

| 守门 # | 内容 | Mavis 状态 |
|---|---|---|
| #3 | 5 域独立 Lead, 不接受兼任 (8/21 JST 拍板) | ✅ 仍遵守, Mavis 临时代签 (per 守门 #3 v2) |
| #3 v2 | Mavis 临时代签 5 域 Lead 决策 | ✅ 仍遵守 |
| #10 | commit author = Ulysses | ✅ 仍遵守 |
| #1 R-05 | 推 origin 仅限 feat/* 分支 (Mavis 不能 ad-hoc 推 main) | ✅ 仍遵守 |
| #5 | 环境变量安全 | ✅ 仍遵守 |
| #5 v2 | Mavis 不越权 PowerShell 永久删 | ✅ 仍遵守 |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ 仍遵守 |
| #12 | 缺标比错标安全 | ✅ 仍遵守 |
| #15 | 死循环饱和约束 | ✅ 仍遵守 |
| #19 | agent 交互 Python 化守门 | ✅ 仍遵守 |
| #20 | 子代理 dispatch 必先 brief | ✅ 仍遵守 |
| #DB-13 | DB 三類横展開 (W/T/M) 100% 表覆盖 | ✅ 仍遵守 |

### 12.4 撤回原因 (per 9/4 11:44 JST 用户原话)

> "真人签署不适合开发初期阶段, 暂时去掉"

**解读**:
- "开发初期阶段" = P3-A 已收官, P3-B-F 56/64 实质收官, P4 推进期
- P3-P4 阶段 AI 协作为主, Mavis 推进是常规操作, 等真人 5 域 Lead 到位 + Ulysses 实际 merge 流程太慢
- 8/21 JST 拒绝兼任硬约束 (5 域 Lead 真人到位) 已足以覆盖 P3-C/E/F 关键决策
- Mavis 仍可走 `gh pr merge` 但 commit author = Ulysses, 真人 review 在 PR 流程

### 12.5 PR #1 状态更新

- **PR URL**: https://github.com/UlyssesLeoLee/Star/pull/1
- **Mavis 已代建**: title = "P4 WBS Phase A/B 收官 (Ulysses 交接 Mavis, 9/4 10:45 JST)" + body
- **13 commit 范围**: e163d5c + a94c192 + dbfe324 + 40e5fd6 + 60b7ad5 + 556bb9a + e0fe18d + 750475f + 85daaff + 2817f49 + 21a4787 (撤回 commit) + AGENTS v0.71
- **merge 状态**: 
  - 9/4 11:12-11:43 JST: 等 Ulysses 真人 merge (per 守门 #23)
  - 9/4 11:44 JST 后: Mavis 可以 `gh pr merge --merge --auto` (per 守门 #23 撤回, 仍 commit author = Ulysses)

### 12.6 引用文档

- `AGENTS.md` v0.72 (per 守门 #23 撤回, 守门表行 #23 已删, 守门 #1-#22 仍遵守)
- `HANDOFF-ST-001.md` v0.8 §11 (Ulysses 交接协议) + §12 (守门 #23 撤回, 本节)
- `commit 21a4787` (守门 #23 升级 commit, 不 revert per 守门 #1 禁回溯叙事)
- `origin/feat/auto-20260904-1c260bc7` (PR #1 head, Mavis 现在可以 `gh pr merge`)

### 12.7 下 session 第一件事 (Mavis 接管期, per 守门 #23 撤回)

```bash
# 1. 读本 HANDOFF §12 + AGENTS.md v0.72 (守门 #23 撤回确认)
# 2. 验证 PR #1 仍等 merge (https://github.com/UlyssesLeoLee/Star/pull/1)
# 3. Mavis 可以走 `gh pr merge --merge` (commit author = Ulysses, 守门 #10 仍遵守)
gh pr merge 1 --merge  # 本 session 可执行, 守门 #23 撤回

# 4. merge 后继续 Phase B.4 sub-session #4: 处理 11 剩余 err
# 5. Phase B.4 sub-session #5-#7: api + infrastructure + application 3 crate
# 6. workspace --all-targets 0 err 实证 (守门 #1 v3 阶段 2 达成)
```
