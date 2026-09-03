# RF-001 T1.1 根目录报告文件归档 — 完成报告

> **状态**: ✅ 完成
> **对应 WBS**: `docs/refactor/WBS-001-refactor.md` §0 T1.1, §1 T1.1 详情 (line 42-45)

---

## 0. 结论

根目录 71 个报告类 `.md` 文件 `git mv` 至 `docs/reports/`, 0 行内容改动 (纯路径移动), 全仓精确正则扫描确认 0 处 markdown 链接语法 (`](...)`）引用这些文件, 无需修正链接。

---

## 1. 范围核实 (WBS 估计 vs 实测)

WBS 原文写"98 个 md"。本次 `git ls-files -- "*.md" | grep -v "/"` 实测根目录 tracked `.md` 文件共 **74** 个，扣除 `README.md`/`CHANGELOG.md`/`AGENTS.md` 3 项例外（WBS 步骤 4 隐含这 3 个文件留在根目录，仅同步其中的链接），剩余 **71** 个。"98" 与实测不符（可能是 WBS 撰写时的估算，或此前 session 已零散清理过一部分），**以本次 `git ls-files` 实测为准**，此处明确订正记录，不掩盖数字差异。

---

## 2. `HANDOFF-ST-001.md` 特例条款核查

WBS 步骤 2 对 `HANDOFF-ST-001.md` 有特例：若仍被跨 session 活跃引用，移动前需先跟 Ulysses 确认。

实测：`git ls-files | grep -i HANDOFF-ST-001` → 命中 `docs/reports/HANDOFF-ST-001.md`，**该文件已不在根目录**，说明在更早的 session 中已完成移动。本次 T1.1 执行前根目录 74 文件列表中本就不含它，因此该特例条款在本次执行范围内 **moot**（不适用），无需额外确认或单独 commit。

---

## 3. 模式匹配与范围决策

WBS 步骤 2 命令: `git mv PHASE-*.md STAR-*.md QA-*.md ..\..\reports\HANDOFF-ST-001.md DDD-LEAD-REVIEW-PROCESS.md REQUIREMENTS-THREAD-C-HANDOFF.md docs/reports/`

对 71 个候选文件按上述命名 pattern（`PHASE-*`/`STAR-*`/`QA-*`/`DDD-LEAD-REVIEW-PROCESS.md`/`REQUIREMENTS-THREAD-C-HANDOFF.md`）匹配，**67 个匹配，4 个不匹配**：

| 文件 | 内容核实 | 决策 |
|---|---|---|
| `P3-C-D-SELECTION-RESULT.md` | P3-C + P3-D 拍板结果文档 | ✅ 一并归档 |
| `P3-E-F-SELECTION-RESULT.md` | P3-E + P3-F 拍板结果文档 | ✅ 一并归档 |
| `RGS-LEAD-ROSTER.md` | 已废弃 (deprecated) 的域 Lead roster 文档 | ✅ 一并归档 |
| `remediation-plan.md` | 基本設計書改善計画文档 | ✅ 一并归档 |

**决策依据**: T1.1 任务标题为"根目录报告文件归档"(通用范围)，WBS 步骤 2 命令中枚举的 pattern 是最常见的命名族，非穷举白名单；上述 4 个文件经内容核实均属报告/拍板类文档，与其余 67 个文件性质一致，若不归档则与 T1.1 任务意图相悖。本决策及依据在此明确记录（守门 #11 "缺标比错标安全" — 明确写出决策与依据，而非静默扩大范围）。

---

## 4. 内部链接扫描与修正

按 WBS 步骤 3 规定命令的思路，对全仓 `*.md` 文件执行两轮扫描：

1. **粗筛** (WBS 原始正则 `\](\.\./)*\(PHASE\|STAR\|QA\|HANDOFF\)...`) — 0 命中。
2. **精筛**（构造 71 个文件名的精确 alternation 正则，匹配 `](...<精确文件名>)` markdown 链接语法，排除 `.worktrees/` 下的独立 worktree 副本）：

```bash
pattern=$(paste -sd'|' 71-file-list.txt | sed 's/\./\\./g')
grep -rnoE "\]\([^)]*($pattern)\)" --include=*.md . | grep -v "/\.worktrees/"
```

结果：**0 命中**。全仓（含 `README.md`/`CHANGELOG.md`/`AGENTS.md`/`docs/refactor/*.md` 等留在根目录或引用这些报告的文件）均未使用 markdown 相对路径链接语法引用这 71 个文件 — 现存引用全部是纯文本文件名提及（如"per XXX.md"），不构成可点击的相对路径链接，因此移动后不会产生死链接。

另检查 `README.md`/`CHANGELOG.md`/`AGENTS.md` 中是否有"根目录下的 XXX.md"这类隐含路径假设的措辞 — 0 命中，无需修正。

**结论**: 步骤 3/4（修正内部链接引用）在本仓库当前实际状态下无需任何改动，非因为跳过验证，而是因为验证后确认 0 处真实链接引用。

---

## 5. 改动清单

- `git mv` 71 个根目录 `.md` 文件 → `docs/reports/<同名>.md`（0 命名冲突，逐一核实 `docs/reports/` 原有 65 个文件与新增 71 个无重名）
- `docs/refactor/WBS-001-refactor.md` §0 状态表: T1.1 行 ⚪ → ✅，小计行更新为 "3/12 完成 + 1/12 部分完成"

---

## 6. 守门实证

| 守门 | 规则 | 本报告实证 | 通过 |
|---|---|---|---|
| #1 | 改动前后 `git status` 核对 | 移动前后仅 71 处 rename + 1 处 WBS 编辑, 无意外改动 | ✅ |
| #9 | 不 commit 散落子代理产出 | 0 子代理 dispatch, 本 session 亲自执行 `git mv` + grep 验证 | ✅ |
| #11 | 缺标比错标 | 明确记录 WBS "98" 与实测"71"的数字差异, 明确记录 4 个不匹配 pattern 文件的归档决策依据, 不掩盖 | ✅ |
| #12 | commit-time docs 同步 | 本报告 + `WBS-001-refactor.md` §0 T1.1/小计行同 commit | ✅ (待 commit) |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | T1.1 根目录报告文件归档: 71 个文件 `git mv` 至 `docs/reports/`, 全仓链接扫描确认 0 处需修正, WBS 状态表同步更新 | 用户"继续推进 RF-001" (Auto Mode) |
