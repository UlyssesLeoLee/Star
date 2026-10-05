#!/usr/bin/env python3
"""把 arch-live 文档站的 4 个脚本登记进 registry.md + docs/automation-design.md.

为什么需要 (守门 #1 v21)
    "[P] 档子项落档后必更新 docs/automation-design.md §4 任务卡表 +
    scripts/automation/registry.md 索引; commit 引用 automation-design.md §N.M 章节号"
    —— 本次是 [P] 档 (跨 stage 累计远超 10K token), 4 个新脚本必须登记。

幂等 + 复读校验 (守门 v30 第 3 条)
    写入动作本身也要有独立证据: 写完重新读回来, 不靠"我写过了"当通过。
    版本号动态取现有 max+1 —— 硬编码是本轮实测踩过的坑 (见 agents_guard_v30.py 注释):
    修订历史表早已到 v0.38+, 硬编码 v0.47 会直接制造重号。

退出码
    0 落库成功 (或幂等跳过)  /  1 落库后复读校验失败  /  2 锚点定位失败
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]
REGISTRY = REPO / "scripts" / "automation" / "registry.md"
DESIGN = REPO / "docs" / "automation-design.md"

TASK_ID = "ARCH-LIVE"
SIG = "arch_live_doc_check.py"          # 内容指纹, 用于幂等判定
SCRIPT_ROWS = [
    ("scripts/automation/arch_live_doc_check.py",
     "arch-live 文档站数字漂移 + 资产静态门禁: 重扫仓库比对页面 data-fact 声明 (23 条) "
     "与 data-count 种子值 (29 处, 无 JS 时即显示该值); 运行前 node --check assets/arch.js; "
     "--self-test 变异+对照+全零反例, --sync-seed / --rebase-baseline 为修复入口; "
     "基线 commit 过期只发 NOTICE 不改退出码, 扫不出基线串才 exit 2",
     "ARCH-LIVE；守门 #1 v19 / v21 / v30；automation-design §4.34.7",
     "🟢 完成: PASS 23 data-fact + 29 种子值 + node --check; 解析退化 exit 2 实测有牙"),
    ("scripts/automation/arch_live_interact_check.py",
     "arch-live 页面交互自检: Playwright 真点按钮 + 断言 DOM, 覆盖 9 页 28 项 "
     "(步进器往返/钳位/自动播放/键盘、标签页、crate 过滤与计数器联动、主题切换、"
     "复制按钮、无 JS 与 prefers-reduced-motion 两条降级路径); "
     "--mutate 注入 8 处已知 bug 到站点副本, 断言对应用例变红且对照组全绿",
     "ARCH-LIVE；守门 #1 v19 / v21 / v30；automation-design §4.34.7",
     "🟢 完成: 28/28 PASS + --mutate 对照组 28/28 绿、8/8 变异抓到; 由此揪出 4 个交互 bug"),
    ("scripts/automation/arch_live_shot.py",
     "arch-live 渲染门禁: 用仓库自带 Playwright (@playwright/test) 对 9 页各生成整页 PNG "
     "到 docs/arch-live/.preview/ (已 .gitignore, 可重建); "
     "fail-closed: 页面数为 0 / 产物缺失或过小均 exit 2",
     "ARCH-LIVE；守门 #1 v19 / v21；automation-design §4.34.7",
     "🟢 完成: 9/9 页 exit 0"),
    ("scripts/automation/agents_guard_v30.py",
     "把守门 #1 §4.1 派生规 v30 (门禁必须有变异测试, 且断言要能区分「被测物没跑」) "
     "幂等落进 AGENTS.md 修订表 + 修订历史; 动态取版本号 max+1 防重号; "
     "落库后复读校验 + v30 列数与相邻行一致",
     "守门 #1 v19 / v30；AGENTS.md §4.1 v30 + 修订历史 v0.82；automation-design §4.34.7",
     "🟢 完成: v30 + v0.82 已落库, --check exit 0"),
    ("scripts/automation/arch_live_registry_sync.py",
     "ARCH-LIVE [P] 档登记入口 (守门 #1 v21): 把 4 个 arch-live 脚本幂等登记进本 registry "
     "索引表 + 修订历史, 并在 docs/automation-design.md 追加 §4.34.7 任务卡; "
     "版本号取「文档版本」与「修订历史」两条序列的共同 max+1; "
     "落点为末个 §4 子节之后 (本文件 §5 之后仍有 §4 内容, 按第一个 '## 5.' 插入会错位); "
     "本脚本自身也在登记之列 (先有鸡先有蛋), --check 幂等",
     "ARCH-LIVE；守门 #1 v19 / v21 / v30；automation-design §4.34.7",
     "🟢 完成: 5 行登记 + §4.34.7 任务卡, 幂等 3 连跑 exit 0"),
]

REV_SIG = "ARCH-LIVE 文档站 9 页 + 4 个 fail-closed 门禁脚本"
# 与同级小节一致用 '### 4.34.7', 不带 § 前缀 (第一版写成 '### §4.34.7', 与 4.34.x 兄弟节点不齐)
TASK_CARD_SIG = "4.34.7 ARCH-LIVE"


# ---------------------------------------------------------------- registry.md
def sync_registry() -> tuple[str, list[str]]:
    notes: list[str] = []
    lines = REGISTRY.read_text(encoding="utf-8").splitlines(keepends=True)
    hdr = next((i for i, l in enumerate(lines) if l.startswith("| 脚本路径")), -1)
    if hdr < 0:
        raise LookupError("registry.md: 找不到 '| 脚本路径' 表头")
    end = hdr + 1
    while end < len(lines) and lines[end].lstrip().startswith("|"):
        end += 1

    body = "".join(lines)
    new_rows = [p for p, *_ in SCRIPT_ROWS if f"`{p}`" not in body]
    changed = bool(new_rows)
    if new_rows:
        add = "".join(
            "| `%s` | %s | %s | 本次提交（见Git） | %s |\n" % r
            for r in SCRIPT_ROWS if r[0] in new_rows
        )
        lines[end:end] = [add]
        notes.append("索引表新增 %d 行 (line %d)" % (len(new_rows), end + 1))
    else:
        notes.append("索引表 4 行已在, 幂等跳过")

    if REV_SIG not in body:
        changed = True
        # §0 背景说明: 在该段末尾追加一行
        for i, l in enumerate(lines):
            if l.startswith("## 0."):
                j = i + 1
                while j < len(lines) and not lines[j].startswith("##"):
                    j += 1
                while j > i and not lines[j - 1].strip():
                    j -= 1
                lines.insert(j, (
                    "\n新增 **%s**（[P] `scripts/automation/arch_live_doc_check.py`、"
                    "`arch_live_interact_check.py`、`arch_live_shot.py`、`agents_guard_v30.py`）"
                    "——`docs/arch-live/` 9 页动态架构文档站（零外部依赖、file:// 可开）"
                    "及其 fail-closed 门禁，automation-design §4.34.7；"
                    "同时激活守门 #1 §4.1 派生规 v30（门禁必须有变异测试）\n" % TASK_ID))
                notes.append("§0 背景说明 +1 段")
                break

        # 修订历史: 版本号取"文档版本"与"修订历史"两条序列的共同 max+1。
        # 本文件的修订历史停在 v0.38 而文档版本已到 v0.47 — 两条序列本来就不同步;
        # 只按其中一条取 max 会插出"倒退"的版本号, 宁可取两条的共同上界。
        rev_idx = [i for i, l in enumerate(lines) if l.startswith("| **v0.")]
        if not rev_idx:
            raise LookupError("registry.md: 找不到 '| **v0.NN |' 修订历史行")
        rev_vers = [int(re.search(r"v0\.(\d+)", lines[i]).group(1)) for i in rev_idx]
        doc_vers = [int(m.group(1)) for l in lines[:12]
                    for m in [re.search(r"\*\*文档版本\*\*: v0\.(\d+)", l)] if m]
        if not doc_vers:
            raise LookupError("registry.md: 找不到 '**文档版本**: v0.NN' 头")
        base = max(max(rev_vers), max(doc_vers))
        nxt = base + 1
        lines.insert(rev_idx[-1] + 1, (
            "| **v0.%d** | **2026-10-05 JST** | "
            "Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | %s | "
            "Ulysses 2026-10-05 10:31 JST ask_user 拍板「写进 AGENTS.md 编号 v30」(推荐项) + "
            "`docs/arch-live/` 9 页文档站落库；守门 #1 v21 [P] 档登记要求；"
            "commit 引用 `scripts/automation/agents_guard_v30.py` + 本脚本 |\n"
            % (nxt, REV_SIG)))
        notes.append("修订历史 v0.%d 已插入 (rev序列max=%d, 文档版本=%d)"
                     % (nxt, max(rev_vers), max(doc_vers)))
    else:
        notes.append("修订历史已在, 幂等跳过")

    # 文档版本头: 只在本次真的改了内容时才 bump, 否则重复运行会把版本号一直往上推。
    # (第一版把 bump 放在幂等分支之外, 第二次运行就把 v0.48 推成了 v0.49 — 实测踩过。)
    if changed:
        for i, l in enumerate(lines):
            m = re.search(r"\*\*文档版本\*\*: v0\.(\d+) \(([^)]*)\)", l)
            if m:
                lines[i] = ("**文档版本**: v0.%d (%s)" % (int(m.group(1)) + 1, m.group(2))
                            + l[m.end():])
                notes.append("文档版本 -> v0.%d" % (int(m.group(1)) + 1))
                break
    else:
        notes.append("无内容变更, 文档版本不动 (幂等)")

    REGISTRY.write_text("".join(lines), encoding="utf-8", newline="")
    return "", notes


# ------------------------------------------------- docs/automation-design.md
TASK_CARD = """
### {sig} docs/arch-live/ 9 页动态架构文档站 + 4 个 fail-closed 门禁脚本（per 2026-10-05 arch-live 实测）
新增 **9 页纯静态 HTML 文档站**（`docs/arch-live/index.html` + `01..08-*.html`），零外部依赖、
无 ES module / 无 fetch，`file://` 双击可开、离线可用；内容全部对着仓库实测数字写成
（105 crate / 40 domain / 31 star-* / 43 migration / 150 CREATE TABLE / 525 RLS / 50 REST 路由 /
16 MCP tool / 57 page.tsx / 11 CI workflow），每个数字带 `data-fact` 声明交由脚本核对，
每页底部有 `证据` 折叠块列到 `file:line`；`已实装` 与 `仅骨架 / no-op stub` 显式分标。

**4 个门禁脚本**（守门 #1 v19：agent 与外部交互必走 `scripts/automation/`）：

| 脚本 | 档 | 守门 | 实跑结果 |
|---|---|---|---|
| `arch_live_doc_check.py` | [P] | #1 v19 / v21 / v30 | PASS（23 data-fact + 29 种子值 + `node --check`）；`--self-test` 变异=1 / 对照=0 / 全零反例=1 且报 PARSE-FAIL |
| `arch_live_interact_check.py` | [P] | #1 v19 / v21 / v30 | 28/28 PASS；`--mutate` 对照组 28/28 绿 + 8/8 变异抓到 |
| `arch_live_shot.py` | [P] | #1 v19 / v21 | 9/9 页 exit 0 |
| `agents_guard_v30.py` | [P] | #1 v19 / v30 | v30 + 修订历史 v0.82 落库，`--check` exit 0 |

**门禁自身抓出的 5 个真 bug**（截图永远拍不到，全靠交互自检暴露）：
① crate 计数器 `[data-shown]` 是墙的兄弟节点，`wall.querySelector` 永远拿不到；
② 标签页选择器落在 `[data-tabgroup]` 包裹层（它没有 `data-tab`），`null !== 'l2'` 恒真 →
整个容器被藏起来，子面板一个没切；③ `data-count` 初始文本写死 `0`，无 JS 时数字墙全是 0；
④ 复制按钮 `appendChild` 到末尾而清理正则按行首锚定，复制出的代码尾部多一行「复制」；
⑤ 块注释里原样引用行首锚点正则，星号斜杠序列当场结束注释 → 整份 `arch.js` SyntaxError →
全站交互静默失效。

**由此激活守门 #1 §4.1 派生规 v30**（Ulysses 2026-10-05 10:31 JST ask_user 拍板选项 1）：
门禁必须有变异测试（含对照组）、退出码区分「确实有问题」与「没测到」、
只比较最终状态的断言必须配「机器真的启动过」的独立证据。

**已知缺口（如实记录）**：Mavis 内置浏览器（FilePanel，1280×720）里首屏以下的控件点不动
（顶部 `#themeBtn` 正常），页面有白色合成层覆盖块且滚动失效；判定为内置浏览器局限而非页面缺陷，
依据是 Playwright 真实点击 28/28 通过 + 视口外点击的对照现象，未在内置浏览器内复现交互。

---
"""


def sync_design() -> tuple[str, list[str]]:
    notes: list[str] = []
    if not DESIGN.exists():
        return "MISSING", notes
    body = DESIGN.read_text(encoding="utf-8")
    if TASK_CARD_SIG in body:
        return "", ["automation-design §4.34.7 已在, 幂等跳过"]
    lines = body.splitlines(keepends=True)
    # 落点: 追加到**最后一个** §4 子节之后 (= 文件末尾)。
    # 不能"插到第一个 '## 5.' 之前": 本文件是追加式乱序的 ——
    # '## 5. 守门基线' 在 line 1206, 而 '### 4.34.6' 还在 line 2520,
    # 也就是 §5 之后仍有 §4 的内容。按第一个 '## 5.' 插入会落进 §4 表格中间
    # (实测踩过: 任务卡被插到 line 1171, 落在 Schedule 路由表的表格里)。
    last4 = max((i for i, l in enumerate(lines) if l.startswith("### 4.")), default=None)
    end = len(lines)
    if last4 is not None:
        j = last4 + 1
        while j < len(lines) and not re.match(r"^#{1,3} ", lines[j]):
            j += 1
        end = j
    lines.insert(end, TASK_CARD.format(sig=TASK_CARD_SIG))
    DESIGN.write_text("".join(lines), encoding="utf-8", newline="")
    return "", [f"automation-design {TASK_CARD_SIG} 已插入 (line {end + 1}, 末个 §4 子节之后)"]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只校验, 不改文件")
    args = ap.parse_args()

    if not REGISTRY.exists():
        print(f"FAIL: {REGISTRY} not found", file=sys.stderr)
        return 2

    if args.check:
        rb = REGISTRY.read_text(encoding="utf-8")
        db = DESIGN.read_text(encoding="utf-8") if DESIGN.exists() else ""
        miss = [p for p, *_ in SCRIPT_ROWS if f"`{p}`" not in rb]
        ok = not miss and TASK_CARD_SIG in db and REV_SIG in rb
        print((f"PASS: registry {len(SCRIPT_ROWS)} 行 + §4.34.7 任务卡 + 修订历史均已落库" if ok
               else f"MISS: 缺 {miss or []} / 任务卡 {TASK_CARD_SIG not in db}"),
              file=sys.stderr if not ok else sys.stdout)
        return 0 if ok else 1

    try:
        _, n1 = sync_registry()
        _, n2 = sync_design()
    except LookupError as e:
        print(f"FAIL: {e}", file=sys.stderr)
        return 2

    # 落库后复读校验 (守门 v30 第 3 条)
    rb = REGISTRY.read_text(encoding="utf-8")
    db = DESIGN.read_text(encoding="utf-8")
    problems = []
    for p, *_ in SCRIPT_ROWS:
        if f"`{p}`" not in rb:
            problems.append(f"FAIL: {p} 复读后不在 registry.md")
    if TASK_CARD_SIG not in db:
        problems.append(f"FAIL: {TASK_CARD_SIG} 复读后不在 automation-design.md")
    if REV_SIG not in rb:
        problems.append("FAIL: 修订历史行复读不到")
    # 列数一致性: 新增的索引行必须与表头同列数
    hdr = next((l for l in rb.splitlines() if l.startswith("| 脚本路径")), "")
    ncol = hdr.count("|")
    for p, *_ in SCRIPT_ROWS:
        row = next((l for l in rb.splitlines() if l.startswith(f"| `{p}`")), "")
        if row and row.count("|") != ncol:
            problems.append(f"FAIL: {p} 列数 {row.count('|')} != 表头 {ncol}, 表格会散")
    if problems:
        for p in problems:
            print(p, file=sys.stderr)
        return 1

    for n in n1 + n2:
        print("  " + n)
    print(f"PASS: {len(SCRIPT_ROWS)} 个脚本已登记 + §4.34.7 任务卡已落库, "
          f"复读校验通过 (列数 {ncol})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
