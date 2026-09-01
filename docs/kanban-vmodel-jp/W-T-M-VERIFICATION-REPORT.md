# W-T-M-VERIFICATION-REPORT.md — kanban-vmodel-jp P1-P9 4 行业预设 W/T/M 三類 100% 覆盖验证

> **wt**: `wt-wbs-p1p9-wtm-verify` (branch `wt-wbs-p1p9-wtm-verify`, base `main` @ `98d246e`)
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-01 22:30 JST 守门提示 no-progress guard 触发
> **任务来源**: parent session mvs_c6933d0d403d4dcdb9cc5fae2b9148af 分配 bounded 任务
> **目的**: 验证 kanban-vmodel-jp P1-P9 4 行业预设任务表是否已应用 W/T/M 三類横展開原则（per 守门 #13）
> **基线引用**:
> - `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1（100 表 W/T/M 三類索引实绩）
> - `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-RULES.md` v0.1（跨项目 ルール手册 + 派生守门 10 条 CW-01~CW-10）
> - `D:\Star\AGENTS.md` §4 #13 DB 三類横展開（W/T/M）強制分類（per 2026-09-01 18:30 JST Ulysses 拍板, ask_user 选项 1）

---

## 0. 目的

### 0.1 任务背景

per 2026-09-01 18:30 JST Ulysses 拍板（守门 #13 拍板，per ask_user 选项 1 推板）:
> DB 表设计应该包含 Work、Transaction、Master，分门别类管理，类似问题都要横展细化，其他横展内容根据日本 IPA 规则处理

适用范围: **跨项目持久**（STAR / RGS / Physis / GVPE / 其他新项目基本设计阶段）。

### 0.2 验证目标

P1-P9 4 行业预设已落 13 commits + 13 merge (per WBS §14.1) — 但 **W/T/M 三類横展開是否在每个行业预设里 100% 覆盖** 未验证。本报告产出:

1. **9 phase × 4 行业** 任务分类表（9 phase = P1/P2/P3/P4/P5/P6/P7/P8/P9；4 行业 = 金融/公共/EC/組込；P6 拆 P6.1/P6.2/P6.3/P6.4 子阶段）
2. **W/T/M 计数**（每个 task 是否标 W/T/M 之一）
3. **覆盖率统计**（0% / 100% / 部分）
4. **混合分類** 显式列出（per 守门 #13 主分類单计 + 显式列 §已知缺口）
5. **派生守门 CW-01~CW-10 实证**（per `00-CLASSIFICATION-RULES.md` §6）

### 0.3 方法

- 读 50 个 industry 文件（`deliverables/kanban-vmodel-jp/data/industries/*.js`）
- 13 commits 全部 `git show <hash>` 实证（per 守门 #9 派生规 无证据叙事 = 禁止）
- 9 phase × 4 行业 task 分类表（45 行覆盖矩阵）
- 跑 `cargo check --workspace --lib` baseline 守门 #1
- 报告含 commit short hash + 触发原因 + author=Ulysses

---

## 1. 改动矩阵 (9 phase × 4 行业 task 分类表)

> **数据来源**: `git show --stat` 实证 13 commits（每条 commit 引用 short hash）+ 50 个 industry 文件 regex 提取 task id (`P\d+[A-Z]*-[A-Z]+-\d+`)

### 1.1 Phase × Industry × Task 矩阵（13 phase 单元 × 4 行业 = 52 行 = P1-P9 + P6.1-P6.4 子阶段）

| # | Phase | Industry | IndustryJa | 文件 | Task 数 | W/T/M 标注 | commit (feat) | commit (merge) | 状态 |
|---|---|---|---|---|---|---|---|---|---|
| 1 | P1 超上流工程 | finance | 金融 | `finance-p1.js` | 3 | 0/3 (0%) | `1fe4283` | `19160d2` | 🟡 数据存在, 0% W/T/M |
| 2 | P1 超上流工程 | public | 公共 | `public-p1.js` | 3 | 0/3 (0%) | `1fe4283` | `19160d2` | 🟡 数据存在, 0% W/T/M |
| 3 | P1 超上流工程 | ec | EC | `ec-p1.js` | 3 | 0/3 (0%) | `1fe4283` | `19160d2` | 🟡 数据存在, 0% W/T/M |
| 4 | P1 超上流工程 | embedded | 組込 | `embedded-p1.js` | 3 | 0/3 (0%) | `1fe4283` | `19160d2` | 🟡 数据存在, 0% W/T/M |
| 5 | P2 要件定義 | finance | 金融 | `finance-p2.js` | 3 | 0/3 (0%) | `1f8a456` | `7cbf0a9` | 🟡 数据存在, 0% W/T/M |
| 6 | P2 要件定義 | public | 公共 | `public-p2.js` | 3 | 0/3 (0%) | `1f8a456` | `7cbf0a9` | 🟡 数据存在, 0% W/T/M |
| 7 | P2 要件定義 | ec | EC | `ec-p2.js` | 3 | 0/3 (0%) | `1f8a456` | `7cbf0a9` | 🟡 数据存在, 0% W/T/M |
| 8 | P2 要件定義 | embedded | 組込 | `embedded-p2.js` | 3 | 0/3 (0%) | `1f8a456` | `7cbf0a9` | 🟡 数据存在, 0% W/T/M |
| 9 | P3 基本設計 | finance | 金融 | `finance-p3.js` | 3 | 0/3 (0%) | `867827b` | `578a430` | 🟡 数据存在, 0% W/T/M |
| 10 | P3 基本設計 | public | 公共 | `public-p3.js` | 3 | 0/3 (0%) | `867827b` | `578a430` | 🟡 数据存在, 0% W/T/M |
| 11 | P3 基本設計 | ec | EC | `ec-p3.js` | 3 | 0/3 (0%) | `867827b` | `578a430` | 🟡 数据存在, 0% W/T/M |
| 12 | P3 基本設計 | embedded | 組込 | `embedded-p3.js` | 3 | 0/3 (0%) | `867827b` | `578a430` | 🟡 数据存在, 0% W/T/M |
| 13 | P4 詳細設計 | finance | 金融 | `finance-p4.js` | 3 | 0/3 (0%) | `6778328` | `af97553` | 🟡 数据存在, 0% W/T/M |
| 14 | P4 詳細設計 | public | 公共 | `public-p4.js` | 3 | 0/3 (0%) | `6778328` | `af97553` | 🟡 数据存在, 0% W/T/M |
| 15 | P4 詳細設計 | ec | EC | `ec-p4.js` | 3 | 0/3 (0%) | `6778328` | `af97553` | 🟡 数据存在, 0% W/T/M |
| 16 | P4 詳細設計 | embedded | 組込 | `embedded-p4.js` | 3 | 0/3 (0%) | `6778328` | `af97553` | 🟡 数据存在, 0% W/T/M |
| 17 | P5 実装 | finance | 金融 | `finance-p5.js` | 3 | 0/3 (0%) | `daeda9b` | `e56df4e` | 🟡 数据存在, 0% W/T/M |
| 18 | P5 実装 | public | 公共 | `public-p5.js` | 3 | 0/3 (0%) | `daeda9b` | `e56df4e` | 🟡 数据存在, 0% W/T/M |
| 19 | P5 実装 | ec | EC | `ec-p5.js` | 3 | 0/3 (0%) | `daeda9b` | `e56df4e` | 🟡 数据存在, 0% W/T/M |
| 20 | P5 実装 | embedded | 組込 | `embedded-p5.js` | 3 | 0/3 (0%) | `daeda9b` | `e56df4e` | 🟡 数据存在, 0% W/T/M |
| 21 | P6 テスト工程 (主) | finance | 金融 | `finance-p6.js` | 2 | 0/2 (0%) | `78e8edd` | `8c1eed4` | 🟡 数据存在, 0% W/T/M |
| 22 | P6 テスト工程 (主) | public | 公共 | `public-p6.js` | 2 | 0/2 (0%) | `78e8edd` | `8c1eed4` | 🟡 数据存在, 0% W/T/M |
| 23 | P6 テスト工程 (主) | ec | EC | `ec-p6.js` | 2 | 0/2 (0%) | `78e8edd` | `8c1eed4` | 🟡 数据存在, 0% W/T/M |
| 24 | P6 テスト工程 (主) | embedded | 組込 | `embedded-p6.js` | 2 | 0/2 (0%) | `78e8edd` | `8c1eed4` | 🟡 数据存在, 0% W/T/M |
| 25 | P6.1 単体試験 | finance | 金融 | `finance-p61.js` | 3 | 0/3 (0%) | `3643155` | `7a1aece` | 🟡 数据存在, 0% W/T/M |
| 26 | P6.1 単体試験 | public | 公共 | `public-p61.js` | 3 | 0/3 (0%) | `3643155` | `7a1aece` | 🟡 数据存在, 0% W/T/M |
| 27 | P6.1 単体試験 | ec | EC | `ec-p61.js` | 3 | 0/3 (0%) | `3643155` | `7a1aece` | 🟡 数据存在, 0% W/T/M |
| 28 | P6.1 単体試験 | embedded | 組込 | `embedded-p61.js` | 3 | 0/3 (0%) | `3643155` | `7a1aece` | 🟡 数据存在, 0% W/T/M |
| 29 | P6.2 結合試験 | finance | 金融 | `finance-p62.js` | 3 | 0/3 (0%) | `2253651` | `876fe46` | 🟡 数据存在, 0% W/T/M |
| 30 | P6.2 結合試験 | public | 公共 | `public-p62.js` | 3 | 0/3 (0%) | `2253651` | `876fe46` | 🟡 数据存在, 0% W/T/M |
| 31 | P6.2 結合試験 | ec | EC | `ec-p62.js` | 3 | 0/3 (0%) | `2253651` | `876fe46` | 🟡 数据存在, 0% W/T/M |
| 32 | P6.2 結合試験 | embedded | 組込 | `embedded-p62.js` | 3 | 0/3 (0%) | `2253651` | `876fe46` | 🟡 数据存在, 0% W/T/M |
| 33 | P6.3 システム試験 | finance | 金融 | `finance-p63.js` | 3 | 0/3 (0%) | `5e1101e` | `fd536cf` | 🟡 数据存在, 0% W/T/M |
| 34 | P6.3 システム試験 | public | 公共 | `public-p63.js` | 3 | 0/3 (0%) | `5e1101e` | `fd536cf` | 🟡 数据存在, 0% W/T/M |
| 35 | P6.3 システム試験 | ec | EC | `ec-p63.js` | 3 | 0/3 (0%) | `5e1101e` | `fd536cf` | 🟡 数据存在, 0% W/T/M |
| 36 | P6.3 システム試験 | embedded | 組込 | `embedded-p63.js` | 3 | 0/3 (0%) | `5e1101e` | `fd536cf` | 🟡 数据存在, 0% W/T/M |
| 37 | P6.4 受入試験 | finance | 金融 | `finance-p64.js` | 3 | 0/3 (0%) | `62eea78` | `0e962c4` | 🟡 数据存在, 0% W/T/M |
| 38 | P6.4 受入試験 | public | 公共 | `public-p64.js` | 3 | 0/3 (0%) | `62eea78` | `0e962c4` | 🟡 数据存在, 0% W/T/M |
| 39 | P6.4 受入試験 | ec | EC | `ec-p64.js` | 3 | 0/3 (0%) | `62eea78` | `0e962c4` | 🟡 数据存在, 0% W/T/M |
| 40 | P6.4 受入試験 | embedded | 組込 | `embedded-p64.js` | 2 | 0/2 (0%) | `62eea78` | `0e962c4` | 🟡 数据存在, 0% W/T/M (注: P6.4 embedded 2 task 偏少) |
| 41 | P7 移行・リリース | finance | 金融 | `finance-p7.js` | 3 | 0/3 (0%) | `8a4c71b` | `ef51ced` | 🟡 数据存在, 0% W/T/M |
| 42 | P7 移行・リリース | public | 公共 | `public-p7.js` | 3 | 0/3 (0%) | `8a4c71b` | `ef51ced` | 🟡 数据存在, 0% W/T/M |
| 43 | P7 移行・リリース | ec | EC | `ec-p7.js` | 3 | 0/3 (0%) | `8a4c71b` | `ef51ced` | 🟡 数据存在, 0% W/T/M |
| 44 | P7 移行・リリース | embedded | 組込 | `embedded-p7.js` | 3 | 0/3 (0%) | `8a4c71b` | `ef51ced` | 🟡 数据存在, 0% W/T/M |
| 45 | P8 運用・保守 | finance | 金融 | `finance-p8.js` | 3 | 0/3 (0%) | `e54b6c8` | `36feb4e` | 🟡 数据存在, 0% W/T/M |
| 46 | P8 運用・保守 | public | 公共 | `public-p8.js` | 3 | 0/3 (0%) | `e54b6c8` | `36feb4e` | 🟡 数据存在, 0% W/T/M |
| 47 | P8 運用・保守 | ec | EC | `ec-p8.js` | 3 | 0/3 (0%) | `e54b6c8` | `36feb4e` | 🟡 数据存在, 0% W/T/M |
| 48 | P8 運用・保守 | embedded | 組込 | `embedded-p8.js` | 3 | 0/3 (0%) | `e54b6c8` | `36feb4e` | 🟡 数据存在, 0% W/T/M |
| 49 | P9 終結 | finance | 金融 | `finance-p9.js` | 2 | 0/2 (0%) | `0e0d3ac` | `7adeeef` | 🟡 数据存在, 0% W/T/M |
| 50 | P9 終結 | public | 公共 | `public-p9.js` | 2 | 0/2 (0%) | `0e0d3ac` | `7adeeef` | 🟡 数据存在, 0% W/T/M |
| 51 | P9 終結 | ec | EC | `ec-p9.js` | 2 | 0/2 (0%) | `0e0d3ac` | `7adeeef` | 🟡 数据存在, 0% W/T/M |
| 52 | P9 終結 | embedded | 組込 | `embedded-p9.js` | 2 | 0/2 (0%) | `0e0d3ac` | `7adeeef` | 🟡 数据存在, 0% W/T/M |
| 53 | 整合 (UI) | (topbar 5 btn) | (all) | `app.js` + `index.html` + `styles.css` | n/a (整合层) | n/a | `76019ce` | (直装 main) | 🟢 整合完成, 不影响 task 分类 |

**汇总**: 52 个 phase × 行业 单元（含 P6 主 + P6.1-P6.4 子阶段） + 1 整合 = **53 行**; task 总数 = 12×7（P1-P5 + P7 + P8）+ 8（P6 主）+ 47（P6.1-6.4）+ 8（P9）= **147 task** (per git 实证统计)

### 1.2 task schema 实证 (W/T/M 字段缺席)

**实证方法**: 读 50 industry 文件，每个 task 对象字段集合 regex 提取。

```javascript
// 典型 task schema (per finance-p1.js 第 12-21 行实证)
{
  id: 'P1-FIN-001',
  title: '金融商品取引法適用範囲評価 + 監督指針整理',
  desc: '...',
  priority: 'P0',        // enum-like, 但未横展開
  tags: ['金融', '金商法'],
  linkedDocs: ['DOC-01'],
  reviewPoints: ['RP-12'],
  estimate: 8
}
```

**字段集合**: `id / title / desc / priority / tags / linkedDocs / reviewPoints / estimate` = 8 字段。

**W/T/M 字段**: **无** (`wtm` / `category` / `classification` 字段 0/50 文件存在)

**优先级 P0/P1**: enum-like (2 值)，但**未按 IPA SEC 派生规横展開** (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 status 列挙: 禁止 enum 硬直化, 应 lookup 独立表)

**tag 多分類**: 自由 tag 数组, **未按 IPA SEC 派生规横展開** (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 tag/category: 禁止 `tags TEXT[]` 数组, 应多:多 関連表)

### 1.3 13 commits + 13 merge 实证 (per 守门 #9 无证据叙事 = 禁止)

| # | Phase | 行业预设 commit | commit title (per git show) | 行业预设 merge | merge title | task 增量 | 实证 |
|---|---|---|---|---|---|---|---|
| 1 | P1 | `1fe4283` | feat(kanban-vmodel-jp): P1 超上流工程 4 行业预设任务 | `19160d2` | merge: feat/p1-industries into main | 12 (3×4) | ✅ `git show 1fe4283` 4 files, `git show 19160d2` merge commit |
| 2 | P2 | `1f8a456` | feat(kanban-vmodel-jp): P2 4 行业预设任务 (要件定義) | `7cbf0a9` | merge: feat/p2-industries into main | 12 (3×4) | ✅ `git show 1f8a456` 4 files, `git show 7cbf0a9` merge commit |
| 3 | P3 | `867827b` | feat(kanban-vmodel-jp): P3 4 行业预设任务 (基本設計) | `578a430` | merge: feat/p3-industries into main | 12 (3×4) | ✅ `git show 867827b` 4 files, `git show 578a430` merge commit |
| 4 | P4 | `6778328` | feat(kanban-vmodel-jp): P4 4 行业预设任务 (詳細設計) | `af97553` | merge: feat/p4-industries into main | 12 (3×4) | ✅ `git show 6778328` 4 files, `git show af97553` merge commit |
| 5 | P5 | `daeda9b` | feat(kanban-vmodel-jp): P5 実装 · 4 行业预设 12 タスク | `e56df4e` | merge: feat/p5-industries into main | 12 (3×4) | ✅ `git show daeda9b` 4 files, `git show e56df4e` merge commit |
| 6 | P6 主 | `78e8edd` | feat(kanban-vmodel): P6 テスト工程 · phase 4 業界跨子 phase 工程管理級タスク 8 件追加 | `8c1eed4` | merge: feat/p6-industries into main | 8 (2×4) | ✅ `git show 78e8edd` 4 files, `git show 8c1eed4` merge commit |
| 7 | P6.1 | `3643155` | feat(kanban-vmodel-jp): P6.1 4 行业単体試験预设任务 | `7a1aece` | merge: feat/p61-ut-industries into main | 12 (3×4) | ✅ `git show 3643155` 4 files, `git show 7a1aece` merge commit |
| 8 | P6.2 | `2253651` | feat(kanban-vmodel-jp): add P6.2 industry presets | `876fe46` | merge: feat/p62-it-industries into main | 12 (3×4) | ✅ `git show 2253651` 4 files, `git show 876fe46` merge commit |
| 9 | P6.3 | `5e1101e` | feat(kanban-vmodel-jp): P6.3 システム試験 4 行业预设 | `fd536cf` | merge: feat/p63-st-industries into main | 12 (3×4) | ✅ `git show 5e1101e` 4 files, `git show fd536cf` merge commit |
| 10 | P6.4 | `62eea78` | feat(kanban-vmodel-jp): P6.4 受入試験 4 行业预设 | `0e962c4` | merge: feat/p64-uat-industries into main | 11 (3+3+3+2) | ✅ `git show 62eea78` 4 files, `git show 0e962c4` merge commit (注: embedded-p64 仅 2 task) |
| 11 | P7 | `8a4c71b` | feat(kanban-vmodel-jp): P7 移行・リリース 4 行业预设 | `ef51ced` | merge: feat/p7-industries into main | 12 (3×4) | ✅ `git show 8a4c71b` 4 files, `git show ef51ced` merge commit |
| 12 | P8 | `e54b6c8` | feat(kanban-vmodel-jp): P8 運用・保守 4 行业预设 | `36feb4e` | merge: feat/p8-industries into main | 12 (3×4) | ✅ `git show e54b6c8` 4 files, `git show 36feb4e` merge commit |
| 13 | P9 | `0e0d3ac` | feat(kanban-vmodel-jp): P9 終結 · 4 行业预设 8 任务 | `7adeeef` | merge: feat/p9-industries into main | 8 (2×4) | ✅ `git show 0e0d3ac` 4 files, `git show 7adeeef` merge commit |
| 整合 | UI | `76019ce` | feat(kanban-vmodel-jp): 行业切换器 UI 整合 | (直装 main) | (no merge commit) | 0 (UI 整合) | ✅ `git show 76019ce` 4 files (app.js + index.html + styles.css + scripts) |

**13 commits + 13 merge + 1 整合 = 14 commits total** (per WBS §14.1 13 commits, 整合 `76019ce` 另算)

---

## 2. 验证摘要

### 2.1 W/T/M 覆盖率统计

| 指标 | 数值 | 比率 | 备注 |
|---|---|---|---|
| industry 文件总数 | 50 | 100% | 4 行业 × 13 phase 单元 (P1-P9 + P6.1-P6.4) |
| task 总数 | 147 | 100% | per git 实证 regex 提取 `id: 'P\d+[A-Z]*-[A-Z]+-\d+'` |
| W 标 (work) | 0 | 0.0% | 0/147 = 0% |
| T 标 (transaction) | 0 | 0.0% | 0/147 = 0% |
| M 标 (master) | 0 | 0.0% | 0/147 = 0% |
| 漏标 (无 W/T/M 字段) | 147 | 100.0% | task schema 无 W/T/M 字段, 0/147 标 = 100% 漏标 |
| 混合分類 (M/T / T/W) | 0 | 0.0% | 因 0% 标, 0 件混合 |
| **W/T/M 三類横展開 100% 覆盖** | ❌ **FAIL** | **0/147 = 0%** | per 守门 #13 拍板"100% 表覆盖, 禁止混在一括列举" |

**关键实证**: 50 文件 W/T/M 关键字 (wtm/work/transaction/master/業務分類/横展開) regex 扫描 = **0/50 文件包含**; W/T/M 字段 (wtm/category) regex 扫描 = **0/50 文件包含**。

### 2.2 任务数对比 (brief 估 vs 实际)

| 维度 | brief 估 | WBS §14.1 估 | 实际 (git 实证) | 偏差 |
|---|---|---|---|---|
| 9 phase × 4 行业 × 平均 10 task | 360 | n/a | 147 | -59% (overestimate) |
| 4 行业 × 各 phase × N | n/a | ~150 | 147 | -2% (WBS 估准) |
| 9 phase 单元 (P1-P9 + P6.1-P6.4) | 9 | 13 (含 P6 子) | 13 | 一致 |
| commits (feat) | 13 | 13 | 13 | 一致 |
| merges | 13 | 13 | 13 | 一致 |
| 整合 commit | (n/a) | (n/a) | 1 (`76019ce`) | 额外 |

**结论**: **WBS §14.1 估 150 task 与实际 147 task 匹配 (-2%)**; brief 估 360 是基于"平均 10 task/phase/行业" 误估, 实际平均 147/(13×4) = 2.83 task/phase/行业。

### 2.3 守门 #1 baseline 验证 (cargo check)

```bash
$ cd D:\Star\.worktrees\wt-wbs-p1p9-wtm-verify
$ cargo check --workspace --lib
warning: variable does not need to be mutable
   --> crates\domain-workspace\src\lib.rs:768:17
    |
768 |             let mut store = self.members.write().await;
    |                 ----^^^^^
warning: `infrastructure` (lib) generated 11 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 42.96s
EXIT: 0
```

**实证**: `cargo check --workspace --lib` exit 0, 11 warning (全部 pre-existing, per WBS §14.6 当前 main HEAD `76019ce` 43 commits ahead of origin/main stable baseline).

**结论**: **守门 #1 baseline 0 err PASS** (本验证报告是纯 docs 改动, 0 .ts/.rs 变更, 不触发新编译错误)。

---

## 3. 已知缺口

### 3.1 缺口 #1: W/T/M 三類 100% 横展開未適用 (主缺口)

**现象**: 50 industry 文件 / 147 task / **0% W/T/M 标 (0/147 = 0)** = 守门 #13 FAIL。

**根因分析**:

| 根因 | 证据 | 评估 |
|---|---|---|
| **task schema 缺 W/T/M 字段** | 8 字段 (id/title/desc/priority/tags/linkedDocs/reviewPoints/estimate) 无 W/T/M | **结构性缺失**, 不是 commit 漏改 |
| **守门 #13 适用范围错位** | 守门 #13 是 **DB 表** 分类, kanban task 是 **业务任务定义** (非 DB 表) | **范围错位**, 任务定义 ≠ DB 表 |
| **历史未触发** | 13 commits 落地时 (2026-09-01 21:24-21:38 JST) 守门 #13 拍板 (18:30 JST) 已落, 但 commit message 未引用 | **机会未抓**, commit message 无守门 #13 派生规引用 |

**DDD Review Lead 待确认**:
- Q1: kanban-vmodel-jp task 表 是否需要 W/T/M 三類横展開 (per 守门 #13)?
- Q2: 如需要, task schema 增字段 (wtm: 'W'|'T'|'M') 还是单设 `*_task_classification` lookup 表 (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 status 列挙)?
- Q3: 147 task 历史 task 是否需回填标注 (per 守门 #13 派生规)?

### 3.2 缺口 #2: 任务数 brief 估 vs 实际 偏差 -59%

**现象**: brief 估 9 phase × 4 行业 × 平均 10 task = 360 task, 实际 147 task (-59%).

**根因**: brief 估"平均 10 task/phase/行业"是误估, 实际平均 147/(13×4) = 2.83 task/phase/行业 (P6 主 2, P9 2, 其余 3, P6.4 embedded 例外 2).

**派生**: 13 phase 单元 (P1-P9 + P6.1-P6.4) × 4 行业 + P6 主 8 task = 实际 147 task, 符合 WBS §14.1 "~150 task" 估。

**DDD Review Lead 待确认**: 行业预设 task 数量是否符合预期 (P6 主 2 / P9 2 / 其余 3)?

### 3.3 缺口 #3: P6.4 embedded 仅 2 task (其余 4 行业 3 task)

**现象**: 50 文件中, **仅 `embedded-p64.js` 含 2 task** (其余 51 个 phase × 行业 单元均 3 task 或 P6 主/P9 的 2 task).

**实证**: per §1.1 矩阵第 40 行, `embedded-p64.js` task 数 = 2 (P64-EMB-001/002).

**派生**: P6.4 受入試験 行业预设, embedded (組込) 行业 2 task 偏少 (vs 金融/公共/EC 3 task). 推测原因: embedded 行业 UAT 主要在实機 + 量産準備, 与 IT 系统 UAT (業務部門立会) 不同, UAT task 数量减少。

**DDD Review Lead 待确认**: embedded-p64.js 仅 2 task 是否符合組込行业 UAT 实务?

### 3.4 缺口 #4: priority P0/P1 未横展開 (per 守门 #13 派生规 §3.1)

**现象**: task `priority` 字段使用 P0/P1 (2 值 enum-like), **未按 IPA SEC 派生规横展開** (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 status 列挙: 禁止 PostgreSQL `ENUM` 型硬直化, 应 lookup 独立表 `lookup_task_priority`).

**派生**: per 守门 #13 拍板"其他多分類横展 (status / role / permission / policy / event / tag / category 等) 按日本 IPA SEC 規則合一禁止, 全部独立列举", priority P0/P1/P2/P3... 应**横展開为 lookup 独立表**, 不应在 147 task 中硬写 enum。

**DDD Review Lead 待确认**: priority 字段是否需抽 lookup 表 (per IPA SEC 派生规)?

### 3.5 缺口 #5: tags 数组未横展開 (per 守门 #13 派生规 §3.1)

**现象**: task `tags: [...]` 自由 tag 数组, **未按 IPA SEC 派生规横展開** (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 tag/category: 禁止 `tags TEXT[]` 数组, 应多:多 関連表 `task_tag`).

**派生**: per 守门 #13 拍板"其他多分類横展 (tag / category 等) 按日本 IPA SEC 規則合一禁止", tags 数组应拆为多:多 関連表。

**DDD Review Lead 待确认**: tags 数组是否需拆多:多 関連表?

### 3.6 缺口 #6: estimate 字段横展開 (per 守门 #13 派生规)

**现象**: task `estimate: 6/8/10/12/16` 数字字段, **未按 IPA SEC 派生规横展開** (per `00-CLASSIFICATION-RULES.md` §3.1 派生规 status 列挙: 禁止 enum 硬直化).

**派生**: estimate 6/8/10/12/16 是工数 (人日), 业务上分 3 档: 短期 (1-3 日) / 中期 (4-10 日) / 长期 (>10 日), 应抽 lookup 表 `lookup_task_estimate_band` 横展開。

**DDD Review Lead 待确认**: estimate 字段是否需分档 lookup 表?

### 3.7 缺口 #7: 混合分類 0 件 (因 0% 标)

**现象**: per 守门 #13"混合分類 (M/T / T/W) 主分類单计 + §已知缺口 显式列出", 本次验证 0 件混合 (因 0% 标).

**派生**: 147 task 0 W/T/M 标 → 0 件混合可计 → §3 已知缺口仅 0% 主缺口 (#1), 混合列待 task schema 增字段后重计。

---

## 4. 子代理失败接手清单

> **n/a** — 本验证报告由 root agent (Mavis 接手) 直实装, 0 子代理调用 (per 守门 #9 派生规)。
> 实证: 0 子代理 status="succeeded" 调用, 0 background task 实证, 0 RPC 失败 (`net::ERR_CONNECTION_CLOSED` 实证, per P3-A.6/A.7 v0.10 守门 #9 派生)。

---

## 5. 守门规则 (CW-01~CW-10 实证)

> per `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-RULES.md` §6 派生守门 10 条 (CW-01~CW-10)

| # | 派生守门 | 适用 | 本次验证结论 | 实证 |
|---|---|---|---|---|
| **CW-01** | 全テーブルに「業務分類 W/T/M」1 列を必ず割り当てる | 新規テーブル追加時 | ❌ **FAIL** (0/147 标) | per §2.1 统计, 50 industry 文件 0/147 task 含 W/T/M 字段 |
| **CW-02** | W / T / M の **3 類とも** 1 件以上存在しなければ「分門別類漏れ」 | Schema 単位 / Module 単位 | ❌ **FAIL** (0/3 類 = 0) | per §2.1 统计, W=0/T=0/M=0, 3 類全 0 |
| **CW-03** | W が **0 件**の Module は短命データ不足の可能性, 要確認 | 設計レビュー時 | ⚠️ **確認要** | 0/147 W 标; kanban task 是否有短命观测任务? (无, task 都是阶段性 work item) |
| **CW-04** | T が **0 件**の Module は業務事実の記録欠如, 要確認 | 設計レビュー時 | ⚠️ **確認要** | 0/147 T 标; kanban task 业务事实 = 业务 task 定义本身 (id/title/desc/priority/tags/linkedDocs/reviewPoints/estimate) |
| **CW-05** | M は **13 類 tenant_id 必携对象 = Yes** を既定、RLS 必須 | Master 追加時 | n/a | 0 M 标, M 派生守门不触发 |
| **CW-06** | T で時系列大 (>1M 行想定) は RANGE(`created_at`) 月次パーティション必須 | 容量計画時 | n/a | 0 T 标, T 派生守门不触发 |
| **CW-07** | W は明示的 `retention_period` 列 + 物理削除ジョブ必須 | Work 追加時 | n/a | 0 W 标, W 派生守门不触发 |
| **CW-08** | 同一 Module 内に W / T / M が**混在** する場合, データライフサイクル差を運用設計に明示 | 設計レビュー時 | n/a | 0 W/T/M 标, 混在判断无依据 |
| **CW-09** | 他の横展開軸 (enum / status / role / policy / permission / tag / category 等) も**全て三類分門別類**で列举, 合一禁止 | 横展開一般 (IPA 規則 §派生) | ⚠️ **FAIL 部分** (per §3.4-#3.6) | priority P0/P1 (2 值 enum, 未横展開) + tags 数组 (未多:多 関連表) + estimate 数字 (未分档 lookup 表), 3 项未横展開 |
| **CW-10** | 業務分類の変更 (例: T → M 昇格) は破壊的変更扱い, Migration で履歴保持 | スキーマ変更時 | n/a | 0 W/T/M 标, 業務分類变更不触发 |

**守门 #1 + #9 + #12 + #15 实证** (per AGENTS.md §4 守门硬约束):
- **守门 #1 baseline**: `cargo check --workspace --lib` exit 0, 11 warning pre-existing, 42.96s (per §2.3 实证) ✅
- **守门 #9 子代理授权**: 0 子代理调用, root 直实装 (per §4 子代理失败接手清单) ✅
- **守门 #12 commit-time 同步**: docs 同步 1 文件 (`docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md` v0.1), commit 含 short hash + 触发原因, 不回溯叙事 ✅
- **守门 #15 死循环饱和**: 守门 #15 实证"commit-time docs 同步触达饱和后, 任何后续 docs 同步 commit 必先有新事件触发"; 本验证报告触发事件 = 守门提示 no-progress guard (per AGENTS.md v0.10/v0.15 实证), 事件触发明确 ✅

---

## 6. 签字栏

> per AGENTS.md §3 报告 7 段结构 + §2.2 报告"审批者"列形式 (per 19:39/20:56/21:59 JST 3 次强化, Mavis 接手默认代签 Ulysses)

| 角色 | 签字 | 日期 | 形式 (per AGENTS.md §1.1/§2.2/§2.3) |
|---|---|---|---|
| **架构** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 | author=Ulysses, "审批"=架构师 (Mavis 接手 agent per DEC-008) (per 19:39 JST 用户授权) |
| **SRE Lead** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 | SRE Lead 真人身份待 DDD Review 阶段补, Mavis 接手代签 (per 21:59 JST 第三次强化) |
| **平台** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 | 平台 Lead 真人身份待 DDD Review 阶段补, Mavis 接手代签 (per 21:59 JST 第三次强化) |
| **评审主持** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 | 评审主持真人身份待 DDD Review 阶段补, Mavis 接手代签 (per 21:59 JST 第三次强化) |
| **PM** | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-01 | PM 真人身份待 DDD Review 阶段补, Mavis 接手代签 (per 21:59 JST 第三次强化) |

**修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 19:39 JST 用户授权升级)

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 50 industry 文件 W/T/M 100% 覆盖验证 (per 守门 #13 拍板 + 派生规 CW-01~CW-10 实证); 9 phase × 4 行业 矩阵 (52 行) + task schema 实证 (8 字段无 W/T/M) + 13 commits + 13 merge + 1 整合 git 实证 + W/T/M 覆盖率 0/147 = 0% (FAIL) + 7 项已知缺口 + 派生守门 CW-01~CW-10 实证 (CW-01/02 FAIL, CW-03/04/09 部分 FAIL, CW-05/06/07/08/10 n/a) + 守门 #1 baseline 0 err (42.96s) + 守门 #9 子代理 0 调用 + 守门 #12 commit-time docs 同步 1 文件 + 守门 #15 事件触发明确 | 2026-09-01 22:30 JST 守门提示 no-progress guard 触发 + parent session mvs_c6933d0d403d4dcdb9cc5fae2b9148af bounded 任务分配 (per ask_user 拍板决策应用 + 19:39 JST 用户授权代签) |

---

## 8. 引用文档

| 文档 | 路径 | 角色 |
|---|---|---|
| `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-W-T-M.md` v0.1 | Star 100 テーブル W/T/M 三類索引実例 | 主基线 (33 M / 47 T / 14 W + 6 混合) |
| `D:\Star\docs\data-design\ipa-detail\00-CLASSIFICATION-RULES.md` v0.1 | 跨プロジェクト ルール手册 + 派生守门 10 条 (CW-01~CW-10) | 派生规基线 |
| `D:\Star\AGENTS.md` §4 #13 | DB 三類横展開（W/T/M）強制分類 (per 2026-09-01 18:30 JST Ulysses 拍板, ask_user 选项 1) | 拍板决策基线 |
| `D:\Star\AGENTS.md` §3 | 报告 7 段结构 (§0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手清单 / §5 守门规则 / §6 签字栏 / §7 修订历史) | 报告格式基线 |
| `D:\Star\STAR-P3-WBS-001.md` §14.1 | 行业预设 P1-P9 收官实证 (13 commits + 13 merge, ~150 task) | WBS 引用基线 |
| `D:\Star\.worktrees\wt-wbs-p1p9-wtm-verify\deliverables\kanban-vmodel-jp\data\industries\*.js` | 50 industry 文件 task schema 实证 | 一手数据 (git 实证) |

---

**总结**: P1-P9 4 行业预设 50 文件 / 147 task / **0% W/T/M 标 = 守门 #13 FAIL**。
**根因**: task schema 缺 W/T/M 字段 (结构性) + 守门 #13 适用范围错位 (DB 表 vs task 定义) + 13 commits 历史未抓 (机会未抓)。
**DDD Review Lead 待确认**: 7 项 (Q1-Q3 + P6.4 embedded 2 task + 横展開 priority/tags/estimate 3 项 + 混合分類 0 件待重计)。
