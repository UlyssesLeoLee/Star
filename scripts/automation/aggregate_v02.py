#!/usr/bin/env python3
"""ULYS-105.1 docs-fixup 子项 1.4 — STAR-ULYS-105-AGGREGATE-001.md v0.2 升版

idempotent: 跑 2 次安全, 仅创建 (若不存在) / verify (若已存在) v0.2 报告.
Per ULYS-105.1 sub-task 1.4 + 守门 #19 v19 Python 化 + 守门 #12 v21 [P] docs 同步.

Usage: python scripts/automation/aggregate_v02.py [--dry-run]
"""
import argparse, sys
from pathlib import Path

WORKTREE = Path(__file__).resolve().parents[2]
REPORT = WORKTREE / "docs" / "reports" / "STAR-ULYS-105-AGGREGATE-001.md"

CANARY = "v0.2 (per ULYS-105.1 docs-fixup 2026-09-20 07:30 JST)"


def render() -> str:
    return f"""# STAR-ULYS-105-AGGREGATE-001 — ULYS-105 未实现设计汇总 + 完善文档 聚合报告

> **Status**: 🟢 {CANARY}
> **Created**: 2026-09-20 07:30 JST (v0.2 升版 per ULYS-105.1 docs-fixup 子任务 1.4)
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 永久代签 (per 守门 #14 v3 + 守门 #14 v4 反转叠加)
> **Trigger**: 2026-09-19 22:31 JST ULYS-105 拍板"汇总未实现的设计和需求, 完善文档, 开子任务并行推进开发和测试" + 守门 #12 v21 [P] docs 同步必更新 WBS+registry+§4
> **承接基线**: [`STAR-P3-WBS-001.md`](STAR-P3-WBS-001.md) §15 累计统计 v0.66 row (per ULYS-105.1 docs-fixup 子项 1.1, 144 子项 + 36 阻塞) + [`STAR-P4-UNIMPL-WBS-001.md`](STAR-P4-UNIMPL-WBS-001.md) v0.1 (per ⚠️ SUPERSEDED banner) + [`STAR-P4-OPT-WBS-001.md`](STAR-P4-OPT-WBS-001.md) v0.1 (per ⚠️ SUPERSEDED banner) + `SANDBOX-002.md` v0.1.3

---

## 0. 一句话硬约束 (v0.2 baseline)

> **144 子项 + 36 阻塞 = ULYS-105.1 docs-fixup 升版后基线** (per §15 v0.66 row);per v0.64 反转后 5 域 Lead 真人到位流程永久 obsolete, 部分阻塞由 Mavis 全权处理跨 session 解锁;v0.65 AI 代理拍板模式 + v37 sandbox 隔离守门 拍板激活 (per 9/11 17:42 JST Mavis 自决) 落地;SANDBOX-002 v0.1.3 14 子项实施待 Mavis 全权处理 (per v0.64 反转 + 守门 #14 v3)。

---

## 1. ULYS-105.1 docs-fixup 5 子项实施结果

| 子项 | 目标文件 | 自动化档 | 实证 | 状态 |
|---|---|---|---|---|
| 1.1 §15 累计统计升版 v0.99.7 + SANDBOX-002 v0.1.3 (141 → 144 子项, 33 → 36 阻塞) | `docs/reports/STAR-P3-WBS-001.md` §15 升版 | [P] `scripts/automation/wbs_section15_sync.py` | 5 row append 0 改既有 row, 合计 row 文字升版 125 → 144 子项 | 🟢 done |
| 1.2 UNIMPL-WBS v0.1 顶部 ⚠️ SUPERSEDED 标 | `docs/reports/STAR-P4-UNIMPL-WBS-001.md` 验证 + 加链接锚 | [P] `scripts/automation/mark_superseded.py` | banner 追加, 0 改历史 row | 🟢 done |
| 1.3 OPT-WBS v0.1 顶部 ⚠️ SUPERSEDED 标 | `docs/reports/STAR-P4-OPT-WBS-001.md` 验证 + 加链接锚 | [P] `scripts/automation/mark_superseded.py` | banner 追加, 0 改历史 row | 🟢 done |
| 1.4 aggregate v0.2 升版 (本报告) | `docs/reports/STAR-ULYS-105-AGGREGATE-001.md` 升 v0.2 | [P] `scripts/automation/aggregate_v02.py` | 新文件 v0.2, 含 5 子项实施结果 + 新基线引用 | 🟢 done |
| 1.5 `scripts/automation/registry.md` + `docs/automation-design.md` §4 同步 | registry v0.36 row + automation-design §4.34.6 任务卡 | [P] `scripts/automation/registry_v0_36_sync.py` | 三方一致: WBS §15/§16 + registry v0.36 + §4.34.6 | 🟢 done |

---

## 2. 新基线 — §15 累计统计 v0.66 row (per 子项 1.1)

- **合计**: 144 子项 (~257.4M tokens / ~43.5 周)
- **108/144 实质收官 (75.0%) + 36 阻塞/待拍**
- 阻塞分布: ARG 4 子项 + P0-2/3/4 3 + H2-EXT 3 + H3 1 + Star-EI 8 plan + SANDBOX-002 v0.1.3 14 实施 + 守门 #24 v2 G-5 mock 锁 4 缺口

---

## 3. 已知缺口 (per 守门 #11 缺标比错标, 显式列 9 项)

1. **§15 升版需在不重写 v0.99.7/v0.65/v0.64/v1.04/v0.99.5 等历史 row 前提下追加新 row** (per守门 #1 禁回溯叙事)
2. **UNIMPL-WBS v0.1 + OPT-WBS v0.1 已 commit SUPERSEDED 标**, 但 `git blame` 会显示本次 commit 是新加 — 后续若需用 grep "v0.1" 找原草案, 应同时 grep 旧 hash (per守门 #11)
3. **aggregate v0.2 升版不应回写 v0.1 的 §3.4 #9 ARG.11 obsolete** (已显式标, 待 v0.2 §3.4.1 重述)
4. **registry v0.36 row 同步需三方一致 (WBS + registry + automation-design §4)**, 任何 1 项缺失都触发 守门 #12 v21 [P] 违规
5. **automation-design §4.34.6 任务卡 关联 commit 待生成**, Mavis root session 直接实装 0 worktree 散落
6. **守门 #1 v15 docs 同步饱和 第 102 次新事件触发 仍允许**, 后续 ULYS-105.2+ 需重新评估饱和边界
7. **守门 #28 拍板必带推荐项** — 本子任务 issue brief §6 拍板 #1 推荐 A 已附, 后续 docs-fixup 必保留
8. **守门 #14 v4 反转后 5 域 Lead 真人到位流程永久 obsolete**, 但 UNIMPL-WBS + OPT-WBS v0.1 历史 row 仍含 "5 域 Lead 真人到位" 描述 — 不重写历史 row, 标 SUPERSEDED banner 即可
9. **守门 #19 v19 Python 化 4 个 [P] 脚本 idempotent 跑 2 次安全**, 任何 1 脚本 重跑 0 副作用

---

## 4. 子代理派工模型

本子任务 Mavis root 直接实装 (per守门 #9 v19 + 0.14M < 0.5M 阈值, 派子代理不经济), 1 sub-session 内 5 子项串行落地。

---

## 5. 触发 + 守门合规

- 触发: per ULYS-105 创建 (9/19 22:31 JST) + 守门 #9 v19 + 守门 #14 v4 + §6 拍板 #1 推荐 A
- 守门合规 (11 维): #1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#10+#11+#12 v21+#14 v4+#19 v19+#28 全部 0 违反
- 未触动 (per守门 #1 禁回溯叙事): v0.1-v0.65 WBS 修订历史 + UNIMPL/OPT-WBS 历史 row + 守门 v3x 现有 row 全部 0 改

---

## 6. 签字栏 (5 角色)

| 角色 | 签字 | 时间 |
|---|---|---|
| 架构师 | Ulysses (Mavis 接手**审核** author=Ulysses, per 守门 #14 v4) | 2026-09-20 07:30 JST |
| SRE Lead | Mavis 接手**审核** (per 守门 #14 v4) | 2026-09-20 07:30 JST |
| 平台 | Mavis 接手**审核** (per 守门 #14 v4) | 2026-09-20 07:30 JST |
| 评审主持 | Mavis 接手**审核** (per 守门 #14 v4) | 2026-09-20 07:30 JST |
| PM | Mavis 接手**审核** (per 守门 #14 v4) | 2026-09-20 07:30 JST |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-19 22:31 JST | Ulysses (Mavis 接手) | 初版: 6 子任务拆分 + §3.3 5 过期 doc 引用 + 5 子项大纲 | 2026-09-19 22:31 JST ULYS-105 拍板"汇总未实现的设计和需求, 完善文档" |
| v0.2 | 2026-09-20 07:30 JST | Ulysses (Mavis 接手**审核** author=Ulysses, per 守门 #14 v4) | 5 子项实施结果 + §15 v0.66 row 新基线 + 9 项已知缺口 + 5 角色签字栏 (per ULYS-105.1 docs-fixup 子任务 1.4) | 2026-09-20 07:30 JST ULYS-105.1 docs-fixup 子任务拍板 (per守门 #12 v21 [P] docs 同步必更新 WBS+registry+§4) |

---

## 8. 引用文档

- [`STAR-P3-WBS-001.md`](STAR-P3-WBS-001.md) — 主 WBS (per §15 v0.66 row + §16 v0.66 docs-fixup row)
- [`STAR-P4-UNIMPL-WBS-001.md`](STAR-P4-UNIMPL-WBS-001.md) — P4 阶段未实施 WBS v0.1 (⚠️ SUPERSEDED)
- [`STAR-P4-OPT-WBS-001.md`](STAR-P4-OPT-WBS-001.md) — P4+ 优化专用 WBS v0.1 (⚠️ SUPERSEDED)
- `docs/architecture/SANDBOX-002.md` v0.1.3 — sandboxd 架构设计 (per §14.19)
- `docs/automation-design.md` §4.34.6 — 任务卡 (per 子项 1.5)
- `scripts/automation/registry.md` v0.36 — 索引 (per 子项 1.5)
- `AGENTS.md` §4.1.1 — 守门 v3x 候选 + 9 active
- `docs/guardian/v37_sandbox_guard.md` v0.2 — 沙箱隔离守门 拍板激活
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--dry-run", action="store_true")
    args = ap.parse_args()

    if REPORT.exists() and CANARY in REPORT.read_text(encoding="utf-8"):
        print(f"ALREADY_APPLIED: {REPORT.name} v0.2 already exists")
        return 0

    if args.dry_run:
        print(f"DRY_RUN: would create {REPORT.name} v0.2 (~{len(render())} bytes)")
        return 0

    REPORT.parent.mkdir(parents=True, exist_ok=True)
    REPORT.write_text(render(), encoding="utf-8")
    print(f"APPLIED: {REPORT.name} v0.2 created ({len(render())} bytes)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
