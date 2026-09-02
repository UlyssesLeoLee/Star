# 2026-09-02 智能合并后未完成开发计划

> **报告版本**: v0.1 (2026-09-02 20:03 JST)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-02 19:01 JST Ulysses 指令"智能合并先进分支到 main, 未完成任务总结成开发计划"
> **拍板决策**: 20:03 JST 合并策略=归档不合并 (per `git tag archive/auto-20260902-17ef4658 1239dc4`); 工作区=git stash 暂存 (per `stash@{0} pre-merge-cleanup-2026-09-02`)
> **依赖**: `STAR-P3-WBS-001.md` v0.2 (60/65 拍板, 55/63 实质收官 87.3%) + `AGENTS.md` v0.11 (24 commits 守门派生 v1-v18) + `docs/automation-design.md` v0.2 (5/5 子代理 RPC 实证 #3) + `STAR-OLU-001.md` (1 SRE·周 = 1.2M)

---

## 0. 报告目的

落地 2026-09-02 19:01-20:03 JST 的智能合并 + 后续开发计划:
- **§1 智能合并结果矩阵**: 10 个本地 wt/feat 分支 + 8 个 origin remote tracking 状态 + 1 个分支归档
- **§2 工作区散落 stash 实证**: 12 modified + 9 untracked → stash@{0}/stash@{1} 2 份暂存
- **§3 main HEAD 实证**: 当前 `c1ae95a`, ahead origin/main 32 commits
- **§4 未完成任务汇总**: 跨 P3-B/C/D/E/F + H2 + DB W/T/M + 5 域 Lead 真人 + 推 origin 等 10 项阻塞
- **§5 归档分支内容清单**: `archive/auto-20260902-17ef4658` 16 commits 的可 cherry-pick 评估表
- **§6 推进建议 / 拍板项**: token-OLU 降序的 8 步推进路径 + 3 项需 Ulysses 拍板
- **§7 守门规则验证**: 4 项守门实证 + 已知缺口

---

## 1. 智能合并结果矩阵 (10 本地 + 8 origin)

### 1.1 10 个本地分支处理结果

| 分支 | 类型 | 与 main 关系 | 处理动作 | 状态 |
|---|---|---|---|---|
| `feat/auto-20260901-abaa40a9` | feat | 已 merged (ahead 0) | 保留 (wt dir 待清理, 跨 session 续) | 🟢 完成 |
| `feat/auto-20260902-1d45d8ad` | feat | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `feat/auto-20260902-c8cfc4ff` | feat | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-h2-strong-type-uuid` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-20260902-p-b5` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-20260902-p-b6` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-20260902-p-c6` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-20260902-p-f6` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| `wt-20260902-p-h2-1` | wt | 已 merged (ahead 0) | 保留 (wt dir 待清理) | 🟢 完成 |
| **`feat/auto-20260902-17ef4658`** | **feat** | **NOT-merged (ahead 16, behind 16)** | **`git tag archive/auto-20260902-17ef4658 1239dc4 -m "..."` + `git branch -D` + `git worktree remove --force` + `git branch -dr origin/feat/auto-20260902-17ef4658`** | 🟡 **归档** |

**归档原因 (per 20:03 JST 拍板)**:
- base `4dd0df1` 落后 main HEAD `c1ae95a` ~50+ commits
- 直接 merge 会反向删除 main 上 32K+ 行后续工作: CHARTS P0/P1 15 图表 / star-api-rest 22 路由 / star-frontend refactor / scripts/automation/ 14 Python 脚本 等
- merge-tree 报告 0 textual conflict (因两边独立发展), 但 file-level 副作用是**大段删除**
- 16 commits 内核心内容 (mock KMS / arch-graph / onboarding / ADR-0041~0044) 已通过 cherry-pick 评估表落档 (见 §5), 跨 session 由人工判断选择性恢复

### 1.2 8 个 origin remote tracking 状态

| remote | hash | ahead of main | behind main | 备注 |
|---|---|---|---|---|
| `origin/feat/auto-20260902-17ef4658` | `1239dc4` | 16 | 16 | **已删除** (本地归档完成, origin 推送 需 Ulysses 决策 per 守门 #1) |
| `origin/feature/ai-ide-compat` | `4ba11f2` | 0 | 0 | 跟 main HEAD 0 差, 保留 |
| `origin/wt-b1-openclaw-http` | `7d85c34` | 0 | 0 | 跟 main HEAD 0 差, 保留 |
| `origin/wt-b3-apikey-storage` | `7d85c34` | 0 | 0 | 同上 |
| `origin/wt-b5-openclaw-mock` | `7d85c34` | 0 | 0 | 同上 |
| `origin/wt-b6-hermes-mock` | `7d85c34` | 0 | 0 | 同上 |
| `origin/wt-b7-api-quota` | `7d85c34` | 0 | 0 | 同上 |
| `origin/wt-push-origin` | `7d85c34` | 0 | 0 | 同上 |

**注**: 6 个 `origin/wt-b*` / `origin/wt-push-origin` 都指向 `7d85c34`, 跟 main HEAD `c1ae95a` 有差异 (main 后续有 32 ahead), 但 ahead/behind 都是 0 表明**这些 remote ref 跟 main 的 merge-base 跟 main HEAD 一致, 0 净差**。可能它们是基于 main @ 7d85c34 之前的某个 hash 创建但已 merge 入 main, remote ref 保持 merge 后的位置。

### 1.3 worktree 目录现状 (11 个 worktree 保留, 跨 session 续清理)

```
D:/Star                                         c1ae95a [main]
D:/Star/.worktrees/feat-auto-20260901-abaa40a9  76019ce [feat/auto-20260901-abaa40a9]
D:/Star/.worktrees/feat-auto-20260902-1d45d8ad  a00dbe7 [feat/auto-20260902-1d45d8ad]
D:/Star/.worktrees/feat-auto-20260902-c8cfc4ff  cde5df9 [feat/auto-20260902-c8cfc4ff]
D:/Star/.worktrees/wt-h2-strong-type-uuid       4dd0df1 [wt-h2-strong-type-uuid]
D:/Star/.worktrees/wt-nav-i18n-a                bd918e4 [wt/star-nav-i18n-a]
D:/Star/.worktrees/wt-nav-shots-b               8c893a9 [wt/star-nav-shots-b]
D:/Star/.worktrees/wt-p-b5                      e0e26e0 [wt-20260902-p-b5]
D:/Star/.worktrees/wt-p-b6                      aa706a5 [wt-20260902-p-b6]
D:/Star/.worktrees/wt-p-c6                      664e675 [wt-20260902-p-c6]
D:/Star/.worktrees/wt-p-f6                      c893f09 [wt-20260902-p-f6]
D:/Star/.worktrees/wt-p-h2-1                    426749d [wt-20260902-p-h2-1]
```

**清理建议** (per 守门 #9 不 commit 散落子代理产出, 跨 session 续):
- 9 个已 merged wt dir: 磁盘可清理 (`git worktree remove --force <dir>`), 但保留可回滚, **建议保留到 DDD Review 阶段**
- `wt/star-nav-i18n-a` / `wt/star-nav-shots-b` 已 merged 入 main, 同上

---

## 2. 工作区散落 stash 实证 (守门 #9 派生规)

### 2.1 stash 暂存 (2 份)

| stash | 触发 | 内容类型 | 实证 |
|---|---|---|---|
| `stash@{0}` | 19:30 JST 智能合并任务启动 | untracked (9 文件) + modified (12 文件) | `git stash push -u -m 'pre-merge-cleanup-2026-09-02'` exit 0, 21 文件入 stash |
| `stash@{1}` | 20:03 JST 残留 Cargo.lock | 1 modified file | `git stash push -m 'pre-merge-cleanup-2026-09-02-cargo-lock-followup' -- Cargo.lock` exit 0, 1 文件入 stash |

### 2.2 暂存内容 (21 + 1 = 22 文件)

**12 modified**:
- `Cargo.lock` (1 文件)
- `docs/briefs/smoke/smoke-dispatcher-001.md` (per smoke test 阶段产物)
- `docs/briefs/smoke/smoke-dispatcher-001.status.json`
- `docs/frontend/screenshots/nav-color-tokens/dark-matrix.png` (4 张 screenshot 重生成)
- `docs/frontend/screenshots/nav-color-tokens/dark-sidebar.png`
- `docs/frontend/screenshots/nav-color-tokens/light-matrix.png`
- `docs/frontend/screenshots/nav-color-tokens/light-sidebar.png`
- `frontend/src/components/UserMenu.tsx` (per D.7 real-mode 状态条)
- `frontend/src/lib/i18n/dictionary.ts` (4 张 i18n 字典)
- `frontend/src/lib/i18n/en.ts`
- `frontend/src/lib/i18n/ja.ts`
- `frontend/src/lib/i18n/zh-CN.ts`

**9 untracked**:
- `docs/architecture/2026-09-02-upgrade/spec/integration/` (per 9/2 升级 spec 集成目录)
- `docs/frontend/screenshots/nav-color-tokens/dark-header-inbox.png.main-untracked` (`.main-untracked` 后缀: 之前某次 wt merge 时残留)
- `docs/phases/` (per kanban-vmodel-jp P1-P9 phase 落档)
- `frontend/src/app/(app)/settings/developer/` (per developer settings 页面)
- `frontend/src/lib/refactor-state-machine.ts` (per /refactor 状态机)
- `scripts/automation/__pycache__/` (Python 编译缓存, 可加 .gitignore)
- `scripts/automation/_verify_i18n.py` (per i18n 验证脚本, 临时)
- `scripts/automation/cli_helper/__pycache__/` (Python 编译缓存)
- `scripts/automation/nav_completion_i18n.py.main-untracked` (`.main-untracked` 后缀, 同上)

**当前 wd 状态**: 0 modified + 0 untracked (干净)

### 2.3 暂存来源追溯 (per 守门 #9 不 commit 散落子代理产出)

来源不确定, 但**形态特征**表明是 9/1-9/2 期间多个 wt merge 时散落:
- `*.main-untracked` 后缀: 守门 #9 实证 #1-#3 提到的"子代理 RPC 不可靠 → 散落产出 → 手动重命名"模式
- 4 张 screenshot 重生成: per `star-nav-completion-001` 子任务 B (commit `8c893a9` + merge `6af1482`)
- i18n 字典 4 文件: per `star-nav-completion-001` 子任务 A (commit `bd918e4` + merge `6bce434`)
- `docs/phases/`: per kanban-vmodel-jp P1-P9 4 行业预设 (13 commits + 13 merge)
- `frontend/src/app/(app)/settings/developer/`: per D.5 settings real-mode
- `frontend/src/lib/refactor-state-machine.ts`: per `/refactor` 页面 (per commit `6b93818`)

**结论**: 暂存内容是**多个 wt merge 后的合理产出** (非散落), 建议**逐个评估 cherry-pick 到 main** (D.7 settings + refactor-state-machine + phases/ + dev/ 实装都对应已完成 commit 的伴随产物)。

---

## 3. main HEAD 实证 (per 当前 `c1ae95a`)

### 3.1 git 实证

| 维度 | 数据 | 证据 |
|---|---|---|
| 当前 branch | `main` | `git status` |
| HEAD | `c1ae95a6654ff18cb1ef0fd1bfb5c8ad4987a0e6` | `git rev-parse HEAD` |
| ahead origin/main | **32 commits** | `git rev-list --count origin/main..HEAD` |
| 守门 #1 守门 | 0 err (cargo check + fmt + clippy + test) | 跨 P3-A 25 守门 + 9 wt merge 后实证 |
| 工作区 | 0 modified + 0 untracked | `git status --porcelain \| wc -l` (本任务完成后) |

### 3.2 32 commits 增量分解 (per `git log --oneline origin/main..main`)

| # | commit | 主题 | 类别 |
|---|---|---|---|
| 1 | `c1ae95a` | docs(automation): star-nav-completion-001 元 commit | docs |
| 2 | `6af1482` | merge: star-nav-completion-001 子任务 B | merge |
| 3 | `6bce434` | merge: star-nav-completion-001 子任务 A | merge |
| 4 | `8c893a9` | feat(visual): HeaderTab 顶栏 4 active x 2 theme 视觉走查全补 | UI |
| 5 | `bd918e4` | fix(i18n): 5 module categoryLabel 同步 + remote entry 新加 | i18n |
| 6 | `227226c` | feat(header): 顶栏 5 tab 走域分色 | UI |
| 7 | `c81a0ab` | feat(header): 顶栏 5 tab 走域分色 (Inbox/Issues/Projects/Agents/Settings) | UI |
| 8 | `f65744a` | feat(subnav): per-item 4 view 域分色 | UI |
| 9-17 | (9 commits) | docs(mobile) IPA v1.0-v1.2.1 + v0.1 spec v0.3-v0.7 (per §6 1.2.1 终版 + v0.7) | docs |
| 18 | `826bc37` | fix(nav): 5 域色 hue 排开 + 6 module 重新分类 + SubNav 染色 | UI |
| 19 | `2ec0c06` | feat(nav): 5 域分色 icon tile (Jira 风格) | UI |
| 20 | `7e252c5` | [CHARTS-P1] P1 阶段 7 图表实装 (C08-C12, C14-C15) | CHARTS |
| 21 | `4cd8652` | docs(agents): AGENTS.md v0.35 落 Phase L star-api-rest 骨架 守门 #12 | docs |
| 22 | `c8f6dc7` | feat(api-rest): Phase L 骨架 crates/star-api-rest (22 路由 stub) | Phase L |
| 23 | `624e972` | [CHARTS-P0-BULK] P0 阶段 2: 7 图表批量实装 | CHARTS |
| 24 | `d6d8631` | [CHARTS-P0] 图表基础设施 + C01 Burndown 完整跑通 22 文件 | CHARTS |
| 25 | `a2ddb17` | feat(ui): 漫画气泡 Tooltip 组件 + i18n (zh-CN/en/ja) + KanbanBoard 接入 | UI |
| 26 | `24de303` | fix(refactor): 补 §8 4 项缺口 #3 #4 #6 #8 | docs |
| 27 | `6b93818` | feat(refactor): /refactor 页面 Jira 风格 5 列 todo/doing/testing/review/done | UI |
| 28 | `c2836a7` | [CHARTS] 图表 & 报告系统 对标 Jira Cloud 报告中心 26 份文档 | docs |
| 29 | `3e0d057` | fix(automation-design): 自审 3 问题 + 守门派生 v22/v23/v24 | docs |
| 30 | `2bdbbdd` | feat(automation-debug): §12 调试控制台 14 Python + 5 unittest 调试 + AI 修改 mock | debug |
| 31-32 | (2 commits) | P3-B/C/D/E/F 5 阶段合并 (per §14.6 累计统计 + AGENTS.md §8 修订) | merges |

---

## 4. 未完成任务汇总 (per `STAR-P3-WBS-001.md` v0.2 + `AGENTS.md` §7 + 守门 #13)

### 4.1 跨 P3 全 5 阶段 8 阻塞项 (per WBS §7)

| # | 阻塞 | 影响阶段 | 需 | 现状 |
|---|---|---|---|---|
| 1 | B.5 OpenClaw 真实集成 | P3-B.5 | endpoint + API key | 🟡 mock 备选 (per `29692a7` 路径) |
| 2 | B.6 Hermes 真实集成 | P3-B.6 | endpoint + API key | 🟡 mock 备选 (per `29692a7` 路径) |
| 3 | E.4 KMS 集成 | P3-E.4 | Vault / AWS KMS 凭证 | 🟡 LocalMockKms mock (per `5ea9611`) |
| 4 | **E.5 / F.1 5 域 Lead 真人到位** | P3-E.5 + P3-F.1 | 5 个真人 (per 8/21 JST 拒绝兼任硬约束) | 🟡 Mavis 代签 临时, 跨 session 续 |
| 5 | D.2 / D.6 CI runner 配置 | P3-D.2 / D.6 | GitHub Actions 真实 runner | 🟡 stub 实装 (per `8ace1d5`) |
| 6 | E.6 Saga 跨域编排 | P3-E.6 | match 域 Lead 真人到位 | 🔴 阻塞 |
| 7 | E.7 DDD 边界验证 | P3-E.7 | 5 域 Lead 真人到位 | 🔴 阻塞 |
| 8 | F.1 DDD Review 阶段 | P3-F.1 | 5 域 Lead + SRE + 平台 + 评审 + PM 5 角色真人 | 🔴 阻塞 |

### 4.2 跨 Phase 10 阻塞项 (per WBS §14.4)

| # | 阻塞项 | 阻塞阶段 | 状态 | 备注 |
|---|---|---|---|---|
| B-1 | **强类型 ID 重构** (DeviceId→Uuid / device_id String→Uuid) | H2-4 → H2-2 → H2-5 | 🟢 **9/1 23:59 JST 选项 1 拍板** (2.5M / 0.4 周) | 9/2 9:00 JST 启 wt |
| B-2 | **5 域 Lead 真人到位** | P3-C.9 / P3-E.5 / P3-F.1 + H2-2 | 🟡 **9/1 23:59 JST 选项 2 拍板**: Mavis 代签临时, 跨 session 续 | 违反 8/21 兼任硬约束, per 19:39 JST 临时授权 |
| B-3 | B.5 OpenClaw 真实 endpoint + API key | P3-B.5 | 凭证 (mock 备选) | wiremock 降级为 🟡 占位 |
| B-4 | B.6 Hermes 真实 endpoint + API key | P3-B.6 | 凭证 (mock 备选) | 同 B-3 |
| B-5 | E.4 KMS 凭证 (Vault / AWS KMS) | P3-E.4 | 凭证 (LocalMockKms mock) | |
| B-6 | D.2 / D.6 GitHub Actions CI runner | P3-D.2 / D.6 | 真实 runner (stub 已实装) | |
| B-7 | 5 tab 命名拍板 (Kanban / Timeline / Backlog / Agents / Worktrees) | UI 端 | DDD Review 拍板 | 拍板问卷 (per `29692a7`) |
| B-8 | **推 origin (R-05 反转已落地)** | final-action | 🟡 **9/1 23:59 JST 选项 1 拍板**: 现在推 main, **9/1 23:59 JST 推失败** | github.com 443 不可达 (Recv failure: Connection was reset, 21s timeout) + 无 PAT/GITHUB_TOKEN |
| B-9 | **4 份报告签字栏 DDD Review 终审** | DDD Review | 4 份签字栏全填 + 修订历史 +1 + 守门 0 违反 | per 9/1 23:59 选项 2, Mavis 代签, 真人到位后追溯 |
| B-10 | **守门 #13 适用边界** (子代理 1 FAIL + 子代理 3 PASS) | DDD Review 7 项 | 🟢 **9/1 23:59 JST 选项 1 拍板**: 仅 Backend PG (INVENTORY 100/100 PASS), task schema 保持现状 | 5 P1-P9 0/147 = 0% 标 结论: 结构性 NOT in scope |

### 4.3 H2 强类型重构 5 子项 (per WBS §14.2)

| # | 子项 | token 预算 | 软参考周 | 状态 | 自动化档 |
|---|---|---|---|---|---|
| H2-1 | star_context 共享 ActorContext 字段扩展 | 0.4M | 0.07 周 | 🟢 **阶段 1 完成** (commit `68ae5ff`) | **[P]** `refactor_template.py` |
| H2-2 | 3 domain port/service 改造 (feedback/validation/integration) | 1.5M | 0.25 周 | 🔴 **阻塞** (revert `8364223`) | **[P]** `refactor_template.py` |
| H2-3 | 5 domain 跨域改造 (comment/identity/project/tenant/work-item) | 0.6M | 0.10 周 | 🟡 **3/5 完成** | **[P]** `refactor_template.py` |
| H2-4 | **强类型 ID 重构** (DeviceId→Uuid / device_id String→Uuid) | 0.8M | 0.13 周 | 🔴 **阻塞** (业务语义不兼容) | **[P]** `refactor_template.py` |
| H2-5 | H2 原 3 domain service.rs 改造 (~150+ call sites) | 0.5M | 0.08 周 | 🔴 **阻塞** (依赖 H2-4) | **[P]** `refactor_template.py` |
| **小计** | | **~3.8M** | **~0.63 周** | **1/5 阶段 1 + 3/5 H2-EXT** | **5/5 [P]** |

### 4.4 DB W/T/M 6 子项 (per WBS §14.3, 守门 #13)

| # | 子项 | 状态 | 引用基线 |
|---|---|---|---|
| CW-1 | W = 物理删除 / タイマー失効 / 短 TTL 明示 retention | 🟢 持续验证 | `00-CLASSIFICATION-W-T-M.md` v0.1 |
| CW-2 | T = 物理删除禁止 + 監査必須 + RLS 13 類必携 | 🟢 持续验证 | `00-CLASSIFICATION-RULES.md` v0.1 |
| CW-3 | M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 | 🟢 持续验证 | 同上 |
| CW-4 | Master 100% RLS / Transaction 100% audit / Work 100% retention_period | 🟢 持续验证 | 同上 |
| CW-5 | 混合分類 (M/T / T/W) 主分類单计 + §已知缺口显式列出 | 🟢 持续验证 | 同上 |
| CW-6 | 其他多分類横展 (status / role / permission / policy / event / tag / category) 按日本 IPA SEC 規則合一禁止, 全部独立列举 | 🟢 持续验证 | 同上 |
| **小计** | | **6/6 持续验证** | 2 引用基线 docs 落档 |

### 4.5 已知散落 (stash 待 cherry-pick 评估, 跨 session 续)

| # | 路径 | 来源 | 建议动作 |
|---|---|---|---|
| 1 | `frontend/src/app/(app)/settings/developer/` | D.5 settings real-mode | 🟢 cherry-pick 评估 (D.7 子项伴随产物) |
| 2 | `frontend/src/lib/refactor-state-machine.ts` | per commit `6b93818` /refactor 页面 | 🟢 cherry-pick 评估 (commit `6b93818` 已 merge, 散落大概率是 wt 残留) |
| 3 | `docs/phases/` | kanban-vmodel-jp P1-P9 | 🟢 cherry-pick 评估 (13 commits 已 merge, 散落是 4 行业预设产物) |
| 4 | `docs/architecture/2026-09-02-upgrade/spec/integration/` | 9/2 升级 spec 集成 | 🟡 待评估 (本任务范围外) |
| 5 | `docs/frontend/screenshots/nav-color-tokens/dark-header-inbox.png.main-untracked` | 之前某次 wt merge 残留 | 🟢 删除 (.main-untracked 后缀 = 散落信号) |
| 6 | `scripts/automation/nav_completion_i18n.py.main-untracked` | 同上 | 🟢 删除 (同上) |
| 7 | `scripts/automation/_verify_i18n.py` | i18n 验证脚本 | 🟡 待评估 (是否归 registry.md) |
| 8 | `scripts/automation/__pycache__/` + `cli_helper/__pycache__/` | Python 编译缓存 | 🟢 加 .gitignore (per 守门 #9 派生产物清理) |
| 9 | 4 张 i18n 字典 + UserMenu.tsx + 4 张 screenshot | `star-nav-completion-001` 子任务 A/B | 🟢 git stash pop 验证 (per merge commit `6bce434` `6af1482`) |

---

## 5. 归档分支内容清单 (per `archive/auto-20260902-17ef4658` = 1239dc4)

### 5.1 16 commits 时间线 (per `git log feat/auto-20260902-17ef4658 --oneline`)

| # | commit | 主题 |
|---|---|---|
| 1 | `1239dc4` | docs(agents): §8 修订历史 v0.34 落档 (Phase 3 mock 5 commit + ADR-0044 v0.2 错判修正 + 15 ahead 实证) |
| 2 | `c4573e3` | test(arch-graph): hop dispatch vitest 7 case + ADR-0044 §5+§6 错判修正 (per 9/2 09:46 JST Ulysses "1" 拍板) |
| 3 | `f44c23b` | feat(mock): KMS unlock/lock + useKms hook (per ADR-0044 §4) |
| 4 | `113a1e4` | feat(mock): api handler stub + 4 必备 LLM × fetch MSW 试响应 (per ADR-0044 §2+§3) |
| 5 | `7b8d5d3` | docs(agents): §8 修订历史 v0.33 落档 (Phase 2 6 commit + 5 维度守门 #12 cascade 实证) |
| 6 | `e4a6e11` | docs(spec): onboarding §6.3 audit endpoint 拡張 (per ADR-0043 v0.1 + 5 commit 实证) |
| 7 | `f14ef0f` | feat(frontend-bridge): retry.ts async backend POST + MSW /api/audit/onboarding-failed handler |
| 8 | `62c18f5` | chore(env): .env.example 增量 (per ADR-0043 §3 + 4 必备 LLM provider) |
| 9 | `fa05464` | feat(audit): domain-audit InMemoryAuditRecorder + 3 onboarding tests (per ADR-0043 §2.2) |
| 10 | `fae5c66` | docs(audit): audit_audit_event.md v0.2 - onboarding event_type 追加 (per ADR-0043 §2.2) |
| 11 | `62bc032` | docs(adr-0043): audit-onboarding-failed 拍板 (既存 audit_audit_event 活用, 新表 0) |
| 12 | `e94c129` | docs(agents): §8 修订历史 v0.32 落档 (4 commit + 5 维度守门 #12 cascade 实证) |
| 13 | `4646d13` | docs(onboarding): §3 段設計 doc (per ADR-0042, 9/2 08:14 JST Ulysses "1" 拍板) |
| 14 | `a54c79d` | feat(onboarding): 首次启动自动识别 LLM API key + 5 重试 + 解决步骤上报 (per ADR-0042) |
| 15 | `cb2475e` | feat(agent-settings): 每个 agent 齿轮按钮 + AgentSettingsModal 弹窗 (per 2026-09-02 02:49 JST 拍板) |
| 16 | `742d377` | feat(arch-graph): Kanban 任务卡架构查看器 (Phase 1, cytoscape + MSW mock + 设计三件套) |

### 5.2 cherry-pick 评估表 (按 main 缺失度 + 跨项目价值排序)

| # | commit | 路径 (相对 `1239dc4`) | main 缺失度 | 跨项目价值 | 建议 | 备注 |
|---|---|---|---|---|---|---|
| 1 | `742d377` | `docs/architecture/2026-08-26-upgrade/adr/0041-arch-agent-graph-viewer.md` | 🟢 **缺失** (main 0 份 arch-agent-graph ADR) | 🟢 高 (跨 arch-graph 视图) | **[P1]** cherry-pick | Phase 2+ 实施 基础 |
| 2 | `742d377` | `docs/architecture/2026-08-26-upgrade/spec/agent-api/arch-agent-graph-viewer.md` | 🟢 缺失 | 🟢 高 | **[P1]** cherry-pick | |
| 3 | `742d377` | `docs/reports/ARCH-AGENT-GRAPH-001-REPORT.md` | 🟢 缺失 | 🟢 中 | **[P2]** cherry-pick | 报告文档 |
| 4 | `742d377` | `frontend/src/app/(app)/agent-windows/page.test.tsx` | 🟢 缺失 (page.tsx 未知是否在 main) | 🟡 中 | **[P2]** cherry-pick (条件) | 先 verify main 有无 page.tsx |
| 5 | `742d377` | `frontend/src/components/board/ArchGraphModal.tsx` | 🟢 缺失 | 🟢 高 (UI 增强) | **[P1]** cherry-pick | |
| 6 | `742d377` | `frontend/src/mocks/data/graph.ts` | 🟢 缺失 | 🟡 中 (mock) | **[P2]** cherry-pick | |
| 7 | `742d377` | `frontend/src/mocks/handlers/graph.ts` | 🟢 缺失 | 🟡 中 (mock) | **[P2]** cherry-pick | |
| 8 | `742d377` | `frontend/src/mocks/__tests__/graph.test.ts` + `graph.handlers.hop.test.ts` | 🟢 缺失 | 🟡 中 (test) | **[P2]** cherry-pick | |
| 9 | `742d377` | `frontend/src/types/cytoscape-ext.d.ts` + `graph.ts` | 🟢 缺失 | 🟡 中 (types) | **[P2]** cherry-pick | |
| 10 | `62bc032` | `docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md` | 🟢 缺失 | 🟢 高 (审计策略) | **[P1]** cherry-pick | 既存 audit 活用, 0 新表 |
| 11 | `fae5c66` | `docs/data-design/ipa-detail/tables/audit_audit_event.md` v0.2 (onboarding event_type 追加) | 🟡 部分 (main v0.1 存在, v0.2 增量) | 🟢 高 (数据设计增量) | **[P1]** cherry-pick (v0.2 diff) | |
| 12 | `fa05464` | `crates/domain-audit/src/lib.rs` (InMemoryAuditRecorder + 3 onboarding tests) | 🟡 部分 (main lib.rs 已有) | 🟡 中 (实装增量) | **[P3]** cherry-pick 评估 (冲突风险) | 需 git diff 验证 |
| 13 | `4646d13` | `docs/architecture/2026-08-26-upgrade/spec/agent-api/onboarding.md` | 🟢 缺失 | 🟢 高 (onboarding 设计) | **[P1]** cherry-pick | |
| 14 | `a54c79d` | `frontend/src/lib/onboarding/*` (scanner / retry / Guide / 3 test) | 🟢 缺失 | 🟢 高 (onboarding 实装) | **[P2]** cherry-pick | |
| 15 | `a54c79d` | `frontend/src/components/OnboardingGuard.tsx` | 🟢 缺失 | 🟢 中 (UI 组件) | **[P2]** cherry-pick | |
| 16 | `f14ef0f` | `frontend/src/mocks/handlers/audit.ts` | 🟢 缺失 | 🟡 中 (mock) | **[P2]** cherry-pick | |
| 17 | `62c18f5` | `.env.example` (4 必备 LLM provider 增量) | 🟡 部分 (main .env.example 已有) | 🟡 中 (env 配置) | **[P3]** cherry-pick 评估 (冲突风险) | 需 git diff 验证 |
| 18 | `cb2475e` | `frontend/src/components/agent-windows/AgentSettingsModal.tsx` | 🟢 缺失 (per main 0 份 agent-windows) | 🟢 中 (UI 组件) | **[P3]** cherry-pick 评估 (依赖 page.tsx) | 需先 verify main 有无 page.tsx |
| 19 | `f44c23b` | `frontend/src/hooks/useKms.ts` | 🟢 缺失 | 🟡 中 (KMS hook) | **[P3]** cherry-pick | |
| 20 | `f44c23b` | `frontend/src/mocks/handlers/kms.ts` | 🟢 缺失 | 🟡 中 (mock) | **[P3]** cherry-pick | |
| 21 | `f44c23b` | `frontend/src/mocks/handlers/providers-llm.ts` | 🟢 缺失 | 🟡 中 (mock) | **[P3]** cherry-pick | |
| 22 | `e4a6e11` | `docs/architecture/2026-08-26-upgrade/spec/agent-api/onboarding.md` §6.3 audit endpoint 拡張 | 🟡 部分 (main onboarding.md 已有) | 🟡 中 (spec 增量) | **[P3]** cherry-pick 评估 (diff) | |
| 23 | `62bc032` | `docs/data-design/ipa-detail/tables/graph_graph_{node,edge,fingerprint}.md` | 🟢 缺失 (3 份新表) | 🟡 中 (数据设计) | **[P3]** cherry-pick 评估 (依赖 graph 实装) | arch-graph 配套 |

**P1 优先级 (建议立刻 cherry-pick, ~0.3M token, 4 子项)**:
- ADR-0041 + spec + REPORT + ArchGraphModal (4 项 arch-graph 基础)
- ADR-0043 + audit_audit_event v0.2 + onboarding spec (3 项 onboarding 基础)

**P2 优先级 (中期 cherry-pick, ~0.5M token, 7 子项)**:
- 全部 frontend 实装 (ArchGraphModal + mocks + tests + types + onboarding + OnboardingGuard + retry)

**P3 优先级 (后期 cherry-pick, ~0.4M token, 8 子项)**:
- 需 git diff 验证冲突的 (domain-audit lib.rs / .env.example / onboarding spec §6.3 / AgentSettingsModal / useKms / KMS mocks / providers-llm mocks / 3 份 graph tables)

**总 cherry-pick 预算**: ~1.2M token (~0.2 周), 19 子项, 跨 session 续 (per 拍板决策)

---

## 6. 推进建议 / 拍板项

### 6.1 token-OLU 降序推进路径 (per `STAR-OLU-001.md` 1 SRE·周 = 1.2M)

| 优先级 | 任务 | token 预算 | 软参考周 | 状态 | 阻塞依赖 |
|---|---|---|---|---|---|
| 1 | **归档分支 P1 cherry-pick** (4 子项, arch-graph + onboarding 基础) | 0.3M | 0.05 周 | 待 Ulysses 拍板 | 无 |
| 2 | **stash pop 验证** (per §4.5 9 项, 评估是否 cherry-pick 或清理) | 0.1M | 0.02 周 | 待 stash pop 评估 | 无 |
| 3 | **H2-2 续做** (3 domain port/service 改造, 1.5M, 跨 session 续) | 1.5M | 0.25 周 | 🔴 阻塞 | 需 H2-4 强类型先做 |
| 4 | **H2-3 续做** (5 domain 跨域 2/5 + 文档 同步) | 0.3M | 0.05 周 | 🟡 3/5 完成 | 续 wt 派子代理 (守门 #9 RPC 实证) |
| 5 | **H2-4 强类型 ID 重构** (DeviceId→Uuid, 拍板已 9/1 23:59) | 0.8M | 0.13 周 | 🔴 阻塞 | 9/2 9:00 启 wt |
| 6 | **H2-5 service.rs 改造** (依赖 H2-4) | 0.5M | 0.08 周 | 🔴 阻塞 | 需 H2-4 完成 |
| 7 | **E.5 / F.1 5 域 Lead 真人寻访** | 0.4M | 0.07 周 | 🟡 阻塞 | 跨 session 续 (per WBS §14.4 B-2) |
| 8 | **推 origin** (守门 #1 反转已落地, 等网络恢复 + PAT) | 0.1M | 0.02 周 | 🟡 阻塞 | 需 Ulysses 提供 PAT (per WBS §14.4 B-8) |

**总推进预算**: ~4.0M token (~0.67 周), 8 步, 3 项需 Ulysses 拍板 (优先级 1/2/8)

### 6.2 3 项需 Ulysses 拍板 (per `2026-09-01 14:58 JST` 拍板决策必须用选项)

1. **P1 cherry-pick 4 子项** (arch-graph + onboarding 基础, 0.3M token): 现在做 / 推迟到 H2 续做后 / 不做
2. **stash pop 处理** (9 项散落, 0.1M token): 逐项 cherry-pick / 全部丢弃 / 留 stash 跨 session 续
3. **推 origin** (守门 #1 反转已落地, 但 443 不可达 + 无 PAT): 现在做 (待网络 + PAT) / 推迟 / 用本地 tag-only 方式

### 6.3 跨 session 续做建议 (H2 续做 vs 归档分支 cherry-pick 二选一)

**现状判断**: H2 强类型重构是**当前 main 演化路径** (per HANDOFF-ST-001 + 9/1 23:59 拍板), 归档分支 cherry-pick 是**侧支实验** (per arch-graph + onboarding 试验性).

**建议**: 优先 H2 续做 (3 domain + 5 domain + 强类型重构), 归档分支 cherry-pick 跨 session 缓做.

---

## 7. 守门规则验证 (per `AGENTS.md` §4 守门 13+ 项)

### 7.1 4 项守门实证

| 守门 | 规则 | 本次任务实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 归档 + stash 0 行 cargo 改动, 不触发 cargo check | ✅ |
| #6 | PowerShell only | 全部 PowerShell 命令 (`$ErrorActionPreference` + `;` + `Select-Object`), 0 bash | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 21 散落 → stash 暂存 0 commit; 归档用 `git tag -a` 0 commit | ✅ |
| #10 | 代签规则应用 | 文档 author + 审批 = Mavis 接手 (per 8/27 19:39 JST 用户授权) | ✅ |

### 7.2 5 维质量门自审 (per `STAR-OLU-001.md` §6)

- **功能完整**: 智能合并任务全部完成 (10 分支处理 + 工作区 stash + main HEAD 验证 + 归档分支 cherry-pick 评估表), 4/4 子项 ✅
- **测试覆盖**: 不适用 (本任务无代码改动, 仅 git 操作)
- **守门 0 违反**: 4 守门实证 (见 §7.1), 0 违反 ✅
- **文档同步**: 本文档 (dev plan) + 归档分支 tag 注解 (含核心内容清单) + §5.2 cherry-pick 评估表 19 子项落档 ✅
- **git 证据**: 归档 tag `archive/auto-20260902-17ef4658` = `1239dc4` 实证可查, stash `{0}/{1}` 实证可查, worktree list 11 个实证可查 ✅

**总分**: **4/5** (测试覆盖 不适用, 0 行代码改动) → 推进门槛 4/5 ≥4 ✅

### 7.3 已知缺口 (per 缺标比错标, 显式列)

1. **cherry-pick 评估表未落地**: §5.2 19 子项 P1/P2/P3 排序 + 路径列表已落档, 但**未实际 cherry-pick**; 需 Ulysses 拍板 (见 §6.2 #1)
2. **worktree dir 11 个未清理**: 已 merged 的 9 个 wt dir 仍占磁盘, 跨 session 续清理 (见 §1.3)
3. **stash 内容 cherry-pick 评估未做**: 21 散落 暂存, 需 Ulysses 拍板处理 (见 §6.2 #2)
4. **AGENTS.md §7 待办** 8 项 (per 守门 #4) + WBS §7 8 项 + WBS §14.4 10 项 仍**未拍板续做**; 本报告仅汇总, 不主动推进
5. **5 域 Lead 真人寻访**: 跨 session 续 (per WBS §14.4 B-2), 不在本次任务范围
6. **推 origin final-action**: 9/1 23:59 JST 推失败, 443 不可达 + 无 PAT, 跨 session 续 (见 §6.2 #3)

### 7.4 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 20:03 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 10 分支智能合并 + 21 散落 stash 暂存 + main HEAD `c1ae95a` 实证 + 跨 P3 + H2 + DB W/T/M + 5 域 Lead 真人 + 推 origin 等 18 项未完成任务汇总 + 归档分支 `archive/auto-20260902-17ef4658` 19 子项 cherry-pick 评估表 + 8 步推进建议 + 3 项 Ulysses 拍板项 | 2026-09-02 19:01 JST Ulysses 指令"智能合并先进分支到 main, 未完成任务总结成开发计划" |
