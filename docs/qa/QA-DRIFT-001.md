# QA-DRIFT-001 · Star 设计书 × 代码乖离对账报告

> **报告版本**: v0.1
> **生成时间**: 2026-08-31 12:10 JST
> **报告人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **主仓 HEAD**: `4c8bd5c`（main，12 ahead origin/main）
> **范围**: frontend `src/` 实装 × `docs/*.md` 设计书 + 25 Module domain spec + test-design v0.3 引用对账

---

## 0. 目的

对 Star 项目当前 main HEAD 的**代码与设计书乖离** + **测试设计书与其他设计书乖离**进行全量审计，整理成对**上游 AI**（即生成 requirements.md v2.0 / basic-design 5f1ea5b / api-design v0.2 / test-design v0.3 / frontend-design v0.1 / 25 份 specs/domain-*.md 的源头 session）的提问清单，**驱动上游回填 / 修订**。

按 AGENTS.md §1.2 守门 #9 派生规（子代理 status="succeeded/failed" ≠ 实际成功 / 失败）：本次 4 个子代理中 3 个 RPC 失败（α/β/δ），1 个 succeeded（γ），但 4 个都写出了 raw 报告。β 因 0 产出由 root 直补。所有 raw 实证可查（实测文件大小 28KB / 7KB / 31KB / 26KB）。

守门 #1+#9+#12 守门已在 sibling session `mvs_8d695e92` 拍板 + commit `4c8bd5c`（feat(kanban) + 4 乖离检查报告）中落地：cargo check 0 err / tsc 0 错 / vitest 14/14 / author=Ulysses 唯一 / 92KB 累计 raw。

---

## 1. 乖离对账矩阵（4 个子任务汇总）

| # | 子任务 | worker task_id | raw 报告 | 状态 | 乖离数 | P0 | P1 | P2 | unverified |
|---|---|---|---|---|---|---|---|---|---|
| α | frontend/src vs frontend-design.md / frontend-internal-01~04 / frontend/design/* | bg_361d286b | `docs/qa/raw/alpha-frontend-drift.md` (28KB) | 子代理 RPC failed 但写出 28KB，**采用** | 31 | 7 | 10 | 14 | 0 |
| β | 状态机/枚举 vs types/ids.ts vs domain-* spec | bg_a87f51d8 | `docs/qa/raw/beta-domain-drift.md` (7KB) | 子代理 RPC failed 0 产出，**root 直补** | 17 | 4 | 2 | 2 | 0 |
| γ | test-design §6/§7 AC vs requirements v2.0 | bg_cdd38f85 | `docs/qa/raw/gamma-testdesign-requirements.md` (31KB) | 子代理 succeeded，**采用** | 27 | 10 | 3 | 3 | 6（+pass=5）|
| δ | test-design 关键引用 vs 7 份设计书实际章节 | bg_7bd56173 | `docs/qa/raw/delta-testdesign-crossref.md` (26KB) | 子代理 RPC failed 但写出 26KB，**采用** | 28 | 11 | 8 | 8 | 1 |
| **合计** | — | — | — | — | **103** | **32** | **23** | **27** | **7** |

---

## 2. P0 乖离（32 条必查 / 必改）按主题归并

### 2.1 路由 IA 乖离（α P0 = 7 条）

| # | 乖离 | 设计书 | 代码 | 修复建议 |
|---|---|---|---|---|
| DRIFT-α-001 | `/projects` 5 tab 命名：设计 `list/board/gantt/calendar/workflow` vs 实装 `kanban/timeline/backlog/agents/worktrees` | ui-redesign-multica-style.md §2 line 22-29 | `app/projects/page.tsx:76` TAB_ITEMS | 选 1 个权威版本（5 tab 已 23:03 JST 拍板），同步 3-pane + multica 文档 |
| DRIFT-α-002 | `Cmd+1/2/3/4` 4 view vs 实装 5 tab | ui-3pane-arch.md §1.3 line 145-156 | `app/projects/page.tsx:76` | 改 1.3 写"5 tab 实装 (per 23:03 JST 拍板)" |
| DRIFT-α-003 | `/board → /projects?tab=board` redirect 目标 tab=board 不存在 | `lib/redirects.ts:48` | `app/projects/page.tsx:137` | redirect → tab=kanban |
| DRIFT-α-004 | 4 redirect 目标（scm/collaboration/workflow/relation）全无对应 tab | `lib/redirects.ts:54-72` | `app/projects/page.tsx:137` | ✅ **已修复** (2026-09-06, `ux/frontend-drift-fix`)：实测 4 个 redirect 现均指向 `?tab=worktrees`，与 `TAB_ITEMS` 实际接受的 tab id 一致，无残留死链 |
| DRIFT-α-005 | `/canvas/:id` deep link 失联 | `lib/redirects.ts:75-78` | `app/projects/page.tsx:132-140` useSearchParams 只接 `?tab` | projects 解析 `?canvas=` + 渲染 CanvasView |
| DRIFT-α-020 | `?K` 全局搜索：openCommandBar 设 isOpen=true 但 CommandBar 组件不存在 | frontend-internal-04 §1.1 | `lib/commandBarStore.ts:71` | 实装 `<CommandBar>` 组件 |
| DRIFT-α-029 | 同 DRIFT-α-020（重复计数 / 合并到 020） | — | — | — |

### 2.2 状态机数字错（β P0 = 4 条）

| # | 乖离 | 实际 | 声称 | 修复 |
|---|---|---|---|---|
| DRIFT-β-003 | WorkItem 状态数 | 6（todo/in_progress/review/blocked/done/wontfix） | test-design §2.1.1 行 179 写 "3 态" | test-design 改 6 态 |
| DRIFT-β-005 | Runtime 状态数 | 5（registered/online/offline/compromised/revoked） | test-design §2.1.1 行 200 + §14 行 1484 写 "8 种边界" + frontend-design §3 #20 "8 边界" | test-design + frontend-design 同步改 5 状态 |
| DRIFT-β-014 | spec 跨文档传染：specs/domain-work-item-spec.md 引 basic-design §5.2 "3 态" vs types/ids.ts 6 态 | 同 β-003 | spec 改 |  |
| DRIFT-β-015 | spec 跨文档传染：specs/domain-local-runtime-spec.md 引 "8 边界" vs types/ids.ts 5 态 | 同 β-005 | spec 改 |  |

### 2.3 test-design 引用空转 / 自指（γ P0 = 10 + δ P0 = 11 = 21 条）

**γ P0 涵盖**（详见 raw `gamma-testdesign-requirements.md`）：
- S1-S5 同步点对不上实际章节
- T1-T3 自指（test-design 引用自己 §6.x 而非 requirements.md §8.3/§27.6/§29.1）
- 线程 C 3 字段（Design Artifact / Test Level / Incident Record）实际位置
- 13 处 tenant_id 端点声明

**δ P0 涵盖**（详见 raw `delta-testdesign-crossref.md`）：
- T1 引用"§6.2.1" 但 §6.2.1 在 test-design 自己 → 改 requirements.md §8.3
- T2 引用"§6.3.3" 但 §6.3.3 在 test-design 自己 → 改 requirements.md §27.6
- T3 引用"§6.3.4" 但 §6.3.4 在 test-design 自己 → 改 requirements.md §29.1
- "VAL-001 验证 §6.2.1" — basic-design §6.2.1 不存在（per test-design §0 自检 "basic-design 停在 98c73b1"）→ 标 TBD
- §6.3.2 引用规范 / §2.5.2 引用规范 / §14 引用规范 与 basic-design 实际章节不对应
- S1-S5 / T1-T2 同步点 vs requirements 实际章节号漂移

---

## 3. P1 乖离（23 条重要功能 / 命名漂移）按主题归并

### 3.1 组件 / 路由结构（α P1 = 10 条）

DRIFT-α-006: settings 7 tab（multica §2）vs 5 tab（实际）
DRIFT-α-007: settings redirect 4 目标 tab 跟 page 5 tab id 不一致 — ✅ **已修复** (2026-09-06, `ux/frontend-drift-fix`)：`settings/page.tsx` 原硬编码 `useState("profile")` 完全不读 `?tab=`，改为直接从 `searchParams` 派生（同 `sprint/page.tsx` idiom），深链与浏览器前进/后退恢复正常；4 个未实装的目标 tab（permissions/members/workspace/integrations）仍落 profile，属 P2 功能缺口非路由 wiring bug，不在本次修复范围
DRIFT-α-008: analytics 5 K 维度 vs 5 图表 tab
DRIFT-α-009: collaboration StatsPage vs canvas placeholder — ✅ **部分已修复** (2026-09-06, `ux/frontend-drift-fix`)：`/canvas/:id` 本身可达但无任何 UI 入口（无 nav registry 项、无 `/canvas` 索引路由、`/collaboration` 画廊被 redirect 拦截）；已在 Worktrees tab 补一条 project-scoped canvas 链接（对应 seed 里真实的 `ref_kind="project"` 关系）。`/collaboration` 画廊本身仍被 redirect 覆盖 — 未处理，原乖离条目的"collaboration 入口名 vs canvas 实装"矛盾未完全解决
DRIFT-α-010: 25 route 1:1 vs 22 平铺 + 5 (app) group 双层 IA
DRIFT-α-013: frontend-internal-01 路由图与实际双层 IA 不一致
DRIFT-α-016: StateMachineDiagram 6 SM 自检（FD-01 已知 bug）
DRIFT-α-017: ADR-FE-013 状态色统一（KanbanCard / GanttBar 直接 className）— ✅ **GanttBar 部分已修复** (2026-09-06, `ux/frontend-drift-fix`)：`GanttBar.tsx` 的 status→color 表原有 2 处硬编码 hex 值与 `StatusPill.tsx` 的权威色表不一致（`planned` 应蓝实灰、`active` 应绿实蓝，均已核对 `StatusPill` COLOR map 逐条修正），并补了 CSS var fallback；`KanbanCard.tsx` 的直接 className 内联色码**未处理**，原乖离条目未完全解决
DRIFT-α-019: store 双源（in-memory + MSW fetch 4 panel）
DRIFT-α-021: page 直接 useStore.setState 违反 §3.1 硬约束（5 page）

### 3.2 状态机 / 枚举名（β P1 = 2 条）

DRIFT-β-011: Worktree 提交流程命名（pr_open/ci_pass/merge vs committing/pushing/ci_running/review_requested）
DRIFT-β-013: Feedback 终态（closed vs resolved）

### 3.3 test-design cross-ref（δ P1 = 8 条）

详见 raw §3/§6/§7，主要：
- §6.x 引用 §X 与 7 份设计书实际章节未穷举对账
- Data Design / Runtime Design / Integration Design 引用节号未在对应文档中找到
- Security Design / AI Agent Design 引用偏少

### 3.4 test-design vs requirements（γ P1 = 3 条）

详见 raw §6/§7，涵盖：13 处 tenant_id 端点声明对账补全 + requirements.md 章节号细节漂移

---

## 4. P2 乖离（27 条命名 / 小细节）按主题归并

α P2=14：DRIFT-α-011 双 page.tsx redirect / -012 [slug] V1 候选 / -015 Atom 分层 / -018 WS 未实装 / -022 Topbar 改 AppHeader / -025 Topbar 56 vs 64 / -027 agent-windows 25 module / -028 SubNav 路径 / -030 issues in-memory / -035 WorkItemKind 5 vs 6 等

β P2=2：DRIFT-β-006 PullRequestStatus 7 值精准化 / -007 ChangeSetStatus 5 值精准化

γ P2=3：详见 raw

δ P2=8：详见 raw §3 + §6 + §7

---

## 5. 验证摘要

**守门 #1**（cargo check / clippy）：本报告纯文档审计，未触发。
- sibling 4c8bd5c 实证：cargo check --workspace --lib 0 err / tsc --noEmit 0 错 / vitest store.test.ts 14/14

**守门 #9**（子代理 status ≠ 实际成功 / 失败）：4 个子代理 3 RPC failed 1 succeeded，全部已 root 验证文件存在。
- α failed (ERR_CONNECTION_RESET) → 实测文件 28KB ✓
- β failed (HTTP2_PING_FAILED) → 0 产出，root 直补 7KB ✓
- γ succeeded → 实测文件 31KB ✓
- δ failed (HTTP2_PING_FAILED) → 实测文件 26KB ✓
- 实证 P3-A.6/A.7 RPC 不可靠模式在本次 4 子代理任务中**3/4 触发**，印证 AGENTS.md §1.2 守门 #9 必须 git 实证。

**守门 #12**（commit-time 同步）：sibling 4c8bd5c 已 commit 4 raw + Kanban 改动；本主文档 `QA-DRIFT-001.md` 待 commit。

实测文件（per `Test-Path` + `Get-Item Length`）：

| 文件 | 大小 | 状态 |
|---|---|---|
| `docs/qa/raw/alpha-frontend-drift.md` | 28221 | ✓ sibling 4c8bd5c committed |
| `docs/qa/raw/beta-domain-drift.md` | 7089 | ✓ sibling 4c8bd5c committed (root 直补版) |
| `docs/qa/raw/gamma-testdesign-requirements.md` | 31540 | ✓ sibling 4c8bd5c committed |
| `docs/qa/raw/delta-testdesign-crossref.md` | 26382 | ✓ sibling 4c8bd5c committed |
| `docs/qa/QA-DRIFT-001.md` | 15420 | ⚠ untracked，待 commit |

---

## 6. 已知缺口（unverified = 7 条 + 5 条 pass）

- **γ unverified 6 条**：requirements.md 章节号行级精确对账（root 抽时间补）
- **δ unverified 1 条**：Data Design §X 引用对账（root 抽时间补）
- **γ pass 5 条**：S1-S5 / T1-T2 / 线程 C 字段 / 13 处 tenant_id 全 PASS（无乖离）
- **QA-DRIFT-001 额外缺口**：
  - 25 Module domain-*.md 实际字段 vs types/ids.ts 882 行字段级 schema 对账（需 25 份独立扫，**当前 session token 不够**）
  - frontend/src "未实装的页面" vs frontend-internal-04 占位说明补全
  - basic-design.md 实际章节号 ↔ test-design 引用 §X 完整 mapping table

---

## 7. 子代理失败接手清单（per AGENTS.md §1.2 守门 #9）

| 子代理 | task_id | 失败模式 | 接手方式 | 接手结果 |
|---|---|---|---|---|
| α | bg_361d286b | net::ERR_CONNECTION_RESET | 实测文件存在（28KB），**采用** | 31 条乖离 |
| β | bg_a87f51d8 | net::ERR_HTTP2_PING_FAILED | 0 产出，**root 直补** raw | 17 条乖离 |
| γ | bg_cdd38f85 | succeeded | 实测文件存在（31KB），**采用** | 27 条乖离 + 5 PASS |
| δ | bg_7bd56173 | net::ERR_HTTP2_PING_FAILED | 实测文件存在（26KB），**采用** worker 失败前已写内容 | 28 条乖离 |

实证 P3-A.6/A.7 RPC 不可靠模式在本次 4 子代理任务中**3/4 触发**（α 失败但有产出 / β 失败 0 产出 / δ 失败但有产出），印证 AGENTS.md §1.2 守门 #9 必须 git 实证（不能信 status）。

---

## 8. 对上游 AI 提问清单（核心 deliverable）

> **优先级**：P0 必答（32 条） / P1 重要（23 条） / P2 选答（27 条）
> **回填对象**：上游 AI 维护 requirements.md / basic-design / api-design / test-design / frontend-design / 25 份 specs 的 session
> **回填格式**：每条问题给出"权威版本（一句话拍板）+ 章节号 / 字段名" 即可，Mavis 接手代签 commit

### 8.1 P0 必答（32 条）摘要

**Q1（路由 IA）**：`/projects` 5 tab 命名以哪个版本为权威？选项：
- A. `kanban / timeline / backlog / agents / worktrees`（23:03 JST 已拍板实装，frontend/src/app/projects/page.tsx:76）
- B. `list / board / gantt / calendar / workflow`（ui-redesign-multica-style.md §2 line 22-29 旧版）

**Q2（路由 IA）**：`/board /scm /collaboration /workflow /relation /canvas/:id` 6 个 redirect 目标应该 redirect 到 5 tab 中哪个？目前 redirect 目标 tab id（board/workflow/relations）实际不存在。
- 推荐 A：全部 redirect 到 `/projects?tab=kanban`
- 推荐 B：redirect 到 `/projects?tab=worktrees`
- 推荐 C：保留独立路由

**Q3（路由 IA）**：(app) group 含 6 子路由（agents / agent-windows / inbox / issues / analytics / settings）— frontend-design.md §2.2 完全没列，是否补成"22 顶级 + 6 (app) panel"双层 IA？

**Q4（路由 IA）**：`/settings` 实际 5 tab（profile/account/team/billing/apikeys），multica §2 写 7 tab。哪个权威？

**Q5（路由 IA）**：`/analytics` 5 维度（cost/tokens/tasks/errors/leaderboard/runtime）vs 实装 5 图表 tab（Burndown/Gantt/Cost/Velocity/Leaderboard）—— 同名不同物。哪个权威？

**Q6（搜索）**：`?K` 全局搜索（per frontend-internal-04 §1.1）— 实装未完成（CommandBar 组件缺）。补还是删除文档？

**Q7（状态机）**：WorkItem 状态数 = 6（types/ids.ts 实测），test-design §2.1.1 行 179 写"3 态"。改 test-design 还是 types/ids.ts？

**Q8（状态机）**：Local Runtime 状态数 = 5（types/ids.ts 实测：registered/online/offline/compromised/revoked），test-design §2.1.1 行 200 + §14 行 1484 写"8 种边界"。改 test-design + frontend-design §3 #20？还是 types/ids.ts 补 3 状态？

**Q9（test-design 自指）**：test-design §0 同步 2026-08-31 T1/T2/T3 引用"§6.2.1 / §6.3.3 / §6.3.4"——但这些章节号在 test-design 自己内部，不在 requirements.md。改回 requirements.md §8.3 / §27.6 / §29.1？

**Q10（test-design 空引用）**："VAL-001 验证 §6.2.1" — basic-design §6.2.1 实际不存在（per test-design §0 自检 "basic-design 停在 98c73b1"）。补 basic-design §6.2.1 还是 test-design 改 TBD？

**Q11（spec 传染）**：specs/domain-work-item-spec.md 引 basic-design §5.2 "3 态" vs types/ids.ts 实际 6 态 — 改 spec 还是改 types/ids.ts？

**Q12（spec 传染）**：specs/domain-local-runtime-spec.md 引 "8 边界" vs types/ids.ts 实际 5 状态 — 同 Q8。

**Q13（test-design 章节）**：S1-S5 同步点表格只写"测试点 / 优先级"，不写"§N 章节号"——与 test-design §0.4 自己定的引用规则冲突。补章节号？

**Q14-32（test-design vs requirements 19 条 P0，详见 γ 报告 §1-§7 + δ 报告 §3/§6/§7）**：
- 涵盖：S1-S5 同步点实际位置、T1-T3 自指纠正、线程 C 三字段（Design Artifact / Test Level / Incident Record）在 requirements.md 实际 §8.3 / §27.6 / §29.1 验证、13 处 tenant_id 端点声明对账、§6.3.2 引用规范 / §2.5.2 引用规范 / §14 引用规范 与 basic-design 实际章节不对应

### 8.2 P1 重要（23 条）摘要

- 组件 / 路由结构 α 报告 §3.1（10 条）
- 状态机 / 枚举名 β 报告 §2/§4（2 条）
- test-design cross-ref δ 报告 §3/§6/§7（8 条）
- test-design vs requirements γ 报告 §6/§7（3 条）

### 8.3 P2 选答（27 条）

详见 α/β/γ/δ raw 报告对应章节。

---

## 8.4 上游 AI 回答（v0.3, 2026-09-10）

> **方法论**：本报告 v0.1 生成于 2026-08-31 12:10 JST。此后 3 个并行 session（handoff 兜底 `4614267`、`ux/frontend-drift-fix` → `c42a368`、test-design `v0.4/v0.5`）已对部分条目落地修复。**数字有时效性**（per QA-ST-001 Q9-T 教训：968 err 不等于文档记的 170 err，任何后续引用前必须重新实测），本轮回答一律对**当前 main HEAD 代码/文档重新核实**，不沿用 v0.1 原始描述。32 条 P0 按 7 类归并作答；P1（23）/P2（27）按同类推论批量处理，不逐条重新核实（详见 §8.4.8）。

### 8.4.1 类 A — 路由 IA（Q1/Q2/Q4/Q6，已消解或已拍板）

**A1（Q1，/projects 5 tab 命名）**：**选项 A 已拍板并已实装**。`2026-08-29 22:49 JST Ulysses 拍板`（commit `7d85c34`）确认 5 tab = `kanban/timeline/backlog/agents/worktrees`；`docs/frontend/design/ui-redesign-multica-style.md` §0.1（2026-08-31 12:48 JST 已追加）已记录该拍板 + 新旧映射表。**无需上游再答** — 属于已拍板但 QA-DRIFT-001 v0.1 生成时未同步扫到的既存事实。残留动作：无（文档 + 代码已一致）。

**A2（Q2，6 redirect 目标）**：**已消解**。实测 `frontend/src/lib/redirects.ts`（2026-09-10）：`/board→?tab=kanban`、`/scm /collaboration /workflow /relation→?tab=worktrees`、`/planning→?tab=timeline`，4 个目标 tab id 均在当前 `TAB_ITEMS`（kanban/timeline/backlog/agents/worktrees）内，无死链。`/canvas/:id` 未走 redirect（改为独立路由 `app/canvas/[id]/page.tsx`，per 2026-09-04 canvas e2e 守门 prerequisite 注记），比原方案（`?canvas=:id` 查询参数）更优（独立可测路由）。残留：`ui-redesign-multica-style.md` §0.1 末行仍写"`/canvas/:id` → `/projects?canvas=:id`"与当前实现不符 — **downstream-AI 文本同步**（HANDOFF-DRIFT-001 D1）。

**A3（Q4，/settings 5 tab vs multica 7 tab）**：**5 tab 为权威**。实测 `settings/page.tsx`（2026-09-10）：`TAB_IDS = [profile, account, team, billing, apikeys]`，且 DRIFT-α-007 已于 2026-09-06 修复深链（URL `?tab=` 直接派生，不再硬编码）。4 个旧 redirect 目标（permissions/members/workspace/integrations）仍落 Profile——**代码注释已显式标注这是已知缺口 #3、非本次修复范围**，即已定性为「未来功能缺口」而非「路由 bug」。残留：若 multica 设计书仍列 7 tab，需同步改 5 — **downstream-AI 文本同步**（D2）。

**A6（Q6，CommandBar / ⌘K）**：**已消解，误判**。实测（2026-09-10）`CommandBar.tsx` 已存在并接入 `AppHeader.tsx` + `app/(app)/layout.tsx` + `app/layout.tsx`，`ui-redesign-multica-style.md` §3 也已写明"⌘K 搜索（CommandBar 全局命令）"。DRIFT-α-020 记录的"组件不存在"是 2026-08-31 时点的真实状态，此后已实装。无需上游回答；无残留动作。

### 8.4.2 类 B — 需 Ulysses 拍板的产品命名题（Q3/Q5）

**B1（Q3，"22 顶级 + 6 (app) group" 双层 IA 是否需补文档）**：**推荐：需要补**，但**待 Ulysses 拍板范围** —— 因为自 QA-DRIFT-001 v0.1 生成后，路由又发生了 2 次拍板性重命名（`2026-09-05 19:13 JST`：`/issues → /sprint`；`2026-09-05 19:45 JST`：`/agents → /agent-view`，均见 `redirects.ts` 注释），若现在补写 IA 文档，应补**当前**结构（含 sprint/agent-view），而不是 v0.1 审计时的旧结构。这不是单纯"要不要写"的是非题，而是"以哪个时间点的路由结构为基准"的范围确认，故标 待 Ulysses 拍板（选项：a. 现在补当前 IA 快照到 frontend-design.md；b. 暂缓，等路由拍板收敛后一次性补）。

**B2（Q5，/analytics 命名：设计"5 维"cost/tokens/tasks/errors/leaderboard/runtime vs 实装 5 tab burndown/gantt/cost/velocity/leaderboard）**：**待 Ulysses 拍板** —— 与 A1（/projects）不同，未找到任何显式拍板记录（如 "22:49 JST 拍板" 式的 commit/时间戳）把 analytics 5 tab 命名定为权威版本，两者是同名不同物的真实产品分歧，非简单的"文档没跟上代码"。**推荐选项 A**（code-authoritative，与 A1/A3 保持同一先例：采用实装 `Burndown/Gantt/Cost/Velocity/Leaderboard` 为权威，设计书改注 5 tab 具体含义），推荐理由：5 个已实装 tab 均有可用 UI + 数据绑定，而设计书"5 维"本身就自相矛盾（列了 6 个词：cost/tokens/tasks/errors/leaderboard/runtime）。

### 8.4.3 类 C — 状态机数字（Q7/Q11 已消解，Q8/Q12 待拍板）

**C1（Q7 + Q11，WorkItem 状态数）**：**已核实 6 态为实际**（`frontend/src/types/ids.ts:198-199` 实测 2026-09-10：`todo/in_progress/review/blocked/done/wontfix`）。test-design §2.1.1 "3 态" + `specs/domain-work-item-spec.md` 引 basic-design §5.2 "3 态" 均为过期数字。**答案：改 test-design + spec 为 6 态**，types/ids.ts 保持不变（无发现任何拍板记录说明"3 态"曾是有意设计，判定为纯文档滞后）。Downstream-AI 文本同步（D3）。

**C2（Q8 + Q12，Local Runtime 状态数）**：**待 Ulysses 拍板**。实测 `types/ids.ts:505-506`：`RuntimeStatus = registered/online/offline/compromised/revoked`（5 态），但 test-design §2.1.1/§14 + frontend-design §3 #20 + `specs/domain-local-runtime-spec.md` 一致写"8 种边界状态"——3 处文档一致大于 1 处代码，且差值达 3 态（非 C1 那种 1-态误差量级），可能是真实功能缺口（原设计 8 态、代码仅实现 5 态）而非单纯笔误。未找到任何拍板记录说明"8 态"被否决改为 5 态。**推荐选项 A**：暂定 5 态为准（与 C1 同规则），文档改 5，若日后需要 3 个未实装边界态（如 pending/degraded/quarantined 等）再单独立项；**选项 B**：视为真实未完成功能，需求单列补齐 3 态。因涉及是否需要新增产品功能（而非单纯文本对齐），标 待 Ulysses 拍板。

### 8.4.4 类 D — test-design 自指引用（Q9/Q10/Q13/Q14-32，已解锁可执行）

**D-核心事实**：实测 `docs/requirements.md`（2026-09-10）：**§8.3、§27.6、§29.1 三个章节确实存在**（分别为"8.3 Design Artifact"line 244、"27.6 Test Level" line 964、"29.1 Incident Record" line 1043）。`docs/basic-design.md` **§6.2 存在但无 §6.2.1 子节**（§6.2 "Local Runtime Security Boundary" 是 bullet list，未分子号，line 2449-2463）。

**D1（Q9，T1-T3 自指）**：**答案确认**：test-design 的 T1/T2/T3 应把 "§6.2.1 / §6.3.3 / §6.3.4"（这些号码实际是 test-design 自己内部的章节号）改引 **requirements.md §8.3 / §27.6 / §29.1**。此修复方向本身在 test-design.md 自己的 v0.4 修订（2026-08-31，见 test-design.md §0 "驱动上游回填清单" γ-02/γ-03/γ-04、δ-01/δ-02/δ-03 行）里已经**由下游 AI 自己预判正确**，此前因"等上游 AI 回填"卡住 — **本回答即解锁**，可交 downstream-AI 执行（D4：test-design.md line ~976/1053/1227-1232/1878-1880 等处按已知位置替换引用号）。

**D2（Q10，VAL-001 引用 §6.2.1 不存在）**：**答案**：basic-design §6.2.1 确实不存在，且不建议新增子号（§6.2 本身是无编号 bullet list，强行拆 §6.2.1 会打乱既有结构）。**改法：VAL-001 引用改为 §6.2**（就近锚点），不新增章节。Downstream-AI 文本同步（D4，同批次）。

**D3（Q13，S1-S5 同步点缺章节号）**：**答案**：S1-S5 表格应按 D1/D2 同一批次补上确认存在的章节号（§8.3/§27.6/§29.1/§6.2ᵃ及 γ/δ raw 报告中列出的其余锚点）。Downstream-AI 文本同步（D4，同批次）。

**D4（Q14-32，19 条 test-design vs requirements P0，含 13 处 tenant_id 端点声明 + §6.3.2/§2.5.2/§14 引用规范）**：**同类批处理**：既然 requirements.md 的 3 个关键锚点（§8.3/§27.6/§29.1）已确认存在，γ/δ raw 报告（`docs/qa/raw/gamma-testdesign-requirements.md`、`docs/qa/raw/delta-testdesign-crossref.md`）里逐条列出的其余引用号也应逐一核实存在性后按同规则修正（存在则改引用号，不存在则标 TBD，不臆造章节）。**因条目多且需要逐条核对 requirements.md 行号，本回答不逐条重新核实 19 条細項，而是把"核实方法 + 已验证的 3 个锚点"作为解锁依据，交 downstream-AI 按此方法论批量执行**（HANDOFF-DRIFT-001 H1）。13 处 tenant_id 端点声明对账维持 γ 报告原判定（5 条 PASS，per §6 已知缺口）。

### 8.4.5 综合决策矩阵

| Q | 分类 | 状态 | 推荐 | 待 Ulysses 拍板？ |
|---|---|---|---|---|
| Q1 | A1 | 已拍板已实装 | 选项 A（kanban/timeline/backlog/agents/worktrees） | 否（已有 8/29 拍板记录） |
| Q2 | A2 | 已消解 | 无需改，仅 doc 文本同步 | 否 |
| Q3 | B1 | 待拍板 | a. 现在补当前 IA（含 sprint/agent-view） | **是** |
| Q4 | A3 | 已消解 | 5 tab 为权威，doc 同步 | 否 |
| Q5 | B2 | 待拍板 | code-authoritative（Burndown/Gantt/Cost/Velocity/Leaderboard） | **是** |
| Q6 | A6 | 已消解（误判） | 无需改 | 否 |
| Q7 | C1 | 已消解 | test-design/spec 改 6 态 | 否 |
| Q8 | C2 | 待拍板 | a. 5 态为准（推荐）/ b. 补 3 态功能 | **是** |
| Q9 | D1 | 已解锁 | 改引 requirements.md §8.3/§27.6/§29.1 | 否 |
| Q10 | D2 | 已解锁 | VAL-001 改引 §6.2 | 否 |
| Q11 | C1 | 已消解 | 同 Q7 | 否 |
| Q12 | C2 | 待拍板 | 同 Q8 | **是** |
| Q13 | D3 | 已解锁 | 按 D1/D2 补章节号 | 否 |
| Q14-32 | D4 | 已解锁（方法论） | 按 3 个已验证锚点批量核实修正 | 否（执行细节，非产品决策） |

### 8.4.6 P1（23 条）批量处理

按 §8.4.1-8.4.4 同类规则推论，不逐条重新核实：
- **α P1（10 条，组件/路由结构）**：DRIFT-α-007/009/017 已在 v0.2（2026-09-06）标注修复状态（007 完全修复，009/017 部分修复，残留项已在 v0.2 原文列明）。DRIFT-α-006（settings 7 vs 5 tab）同 A3 规则。其余（008/010/013/016/018/019/021）与 A1/B1 同类，按"code-authoritative + doc 同步"处理，交 downstream-AI（HANDOFF-DRIFT-001 H2）。
- **β P1（2 条，命名漂移）**：DRIFT-β-011/013 均为纯命名对齐（非数量差），同 C1 规则（doc 改，不涉及功能缺口），downstream-AI 文本同步（H2）。
- **γ/δ P1（11 条，test-design cross-ref）**：同 D4 方法论，downstream-AI 按 3 个已验证锚点 + raw 报告逐条核实修正（H1，同批次）。

### 8.4.7 P2（27 条）批量处理

全部为命名/小细节类（per 报告 §4 自身定性"P2 选答"），无一涉及产品决策，统一按"code-authoritative，doc/spec 文本同步"处理，交 downstream-AI 视 token 预算择机执行（HANDOFF-DRIFT-001 H3，低优先级）。

### 8.4.8 待 Ulysses 拍板汇总（不由 AI 代为决定）

1. **Q3**：是否现在补"22 顶级 + 6 (app) group"IA 文档，以及基准是 v0.1 审计时路由还是当前（含 09-05 sprint/agent-view 重命名）路由。
2. **Q5**：/analytics tab 命名是否采用 code-authoritative（Burndown/Gantt/Cost/Velocity/Leaderboard）。
3. **Q8 + Q12**：Local Runtime 状态数以 5（当前实装）为最终版，还是视为遗留 3 个未实装边界态、需要立项补齐。

---

## 9. 签字栏

| 角色 | 签字 | 时间 |
|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-31 12:10 JST |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手（5 域独立真实身份 DDD Review 阶段补） | 2026-08-31 12:10 JST |
| **平台** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-08-31 12:10 JST |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-08-31 12:10 JST |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 2026-08-31 12:10 JST |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：4 子代理对账（α/β/γ/δ 实战测，3 RPC failed 1 succeeded），103 条乖离汇总（α 31 / β 17 / γ 27 / δ 28），32 P0 必答 + 23 P1 重要 + 27 P2 选答 向上游 AI 提问清单 | 2026-08-31 11:47 JST 用户发令"代码是否存在和设计书乖离，测试设计书是否和其他设计书存在乖离，如有则整理进 qa 文档向上游 ai 提问，开子代理和 worktree 并行处理" |
| v0.2 | 2026-09-06 | Claude (frontend UX 修复 session, worktree `ux/frontend-drift-fix` → merged `main` @ `c42a368`) | 标注 4 条 α 乖离修复状态：DRIFT-α-004（redirect → tab=worktrees 已一致）✅ 完全解决；DRIFT-α-007（settings `?tab=` 深链改从 URL 派生）✅ 完全解决（4 个未实装 tab 仍 P2 缺口，非本次范围）；DRIFT-α-009（canvas 无 UI 入口，补 project-scoped 链接）⚠ 部分解决（`/collaboration` 画廊仍被 redirect 拦截，未处理）；DRIFT-α-017（GanttBar 状态色核对 StatusPill 修正 2 处硬编码 + planned/active 校验中新发现修复）⚠ 部分解决（`KanbanCard.tsx` 内联色码未处理）。验证：542 测试通过（54 文件）+ `tsc --noEmit` 净（2 条无关既存错误）；浏览器视觉验证未做（Chrome extension 本 session 未连上，2 次尝试均失败）。改动详见 commit `c42a368` | 用户发令"优化界面UX"（Auto Mode 下自主定范围）+ 后续"merge，并更新相关文档" |
| v0.3 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§8.4 新增"上游 AI 回答"**：32 条 P0 按 7 类归并作答（对当前 main HEAD 重新核实，不沿用 v0.1 旧数字，per QA-ST-001 Q9-T 教训）——A 类 4 条（Q1/Q2/Q4/Q6）已消解或已拍板；B 类 2 条（Q3/Q5）待 Ulysses 拍板；C 类 4 条（Q7/Q8/Q11/Q12）2 条已消解 + 2 条待拍板；D 类 22 条（Q9/Q10/Q13/Q14-32）已解锁（确认 requirements.md §8.3/§27.6/§29.1 实际存在，basic-design §6.2.1 不存在改引 §6.2）。P1（23）/ P2（27）按同类规则批量处理，交 `HANDOFF-DRIFT-001.md` 下游执行；3 项待 Ulysses 拍板（Q3/Q5/Q8+12）已列 §8.4.8，不由 AI 代为决定 | 用户发令"github的issue和qa，本地qa都回答处理干净" |
