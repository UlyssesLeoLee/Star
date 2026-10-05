#!/usr/bin/env python3
"""把 arch-live 实测出的门禁纪律落进 AGENTS.md (守门 §4.1 派生规 v30).

为什么要有这个脚本 (守门 #19)
    "agent 跟外部交互" 的三类 (子代理 dispatch / CLI 调用 / 代码改造) 都必须走
    scripts/automation/<purpose>.py 落地, commit message 引用脚本相对路径。
    改 AGENTS.md 本身也是代码改造 —— 尤其讽刺的是, 本脚本要落的那条纪律
    恰恰就是"门禁必须有变异测试 + 断言要能区分'没跑'", 而最不该出现的情况
    就是提出者自己不按规矩来。

幂等
    重复运行不会重复插入: 先检测 v30 是否已在表里, 已存在就报 PASS 并退出 0。
    退出码 0 成功 / 1 已存在但内容与预期不符 / 2 定位失败 (锚点找不到)。

用法
    python scripts/automation/agents_guard_v30.py            # 落地
    python scripts/automation/agents_guard_v30.py --check    # 只校验, 不改
"""

from __future__ import annotations

import argparse
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parents[2]
AGENTS = REPO / "AGENTS.md"

# 插入锚点: §4.1 派生规表最后一行 (v29) 的整行前缀。
# 用前缀而不是整行全文 —— 整行太长且会随其他修订变动, 前缀稳定。
ANCHOR_PREFIX = "| v29 | **守门 #1 v29"
MARKER = "| v30 | **守门 #19 v2 — 门禁必须有变异测试"

V30_ROW = (
    "| v30 | **守门 #19 v2 — 门禁必须有变异测试, 且断言要能区分「被测物没跑」** "
    "(per 2026-10-05 arch-live 文档站实测, 5 个 bug + 8 处变异实证): "
    "本仓已有守门 #19 要求「agent 与外部交互必走 scripts/automation/」, "
    "本条是它的质量配套 —— 脚本写了不等于门禁有牙。实证三类失效: "
    "(a) **报告绿 ≠ 跑过** (per CI semgrep 实证, 见 08-guardrails 第 04 节 ①); "
    "(b) **解析退化的 0 = 未知** (per 表解析器漏一种写法静默抽出 0 个, 与「仓库干净」无法区分, "
    "见 08-guardrails ② —— 每个集合必须配人工核对过的数量下限); "
    "(c) **只比较最终状态的断言测不出「代码没跑」** (per arch-live 本轮实证: "
    "先修「无 JS 时数字全是 0」把 data-count 初始文本改成真实值, 于是 "
    "`textContent === data-count` 在一个 JS 都没执行的页面上同样成立; "
    "几小时后 arch.js 出语法错误、整站交互全失效, 而这条数字断言**依然全绿**)。 "
    "**本条强制三条**: "
    "(1) 任何门禁落库前必做变异测试 —— 注入已知违规看它变红, **且不注入的对照组仍绿** "
    "(缺对照组就无法区分「能发现违规」与「装置永远失败」); 变异清单固化在脚本内, "
    "参考 `scripts/automation/arch_live_interact_check.py --mutate` (实跑: 对照组 28/28 绿 + 8/8 变异全抓到) "
    "与 `arch_live_doc_check.py --self-test` (实跑: 变异=1 / 对照=0 / 全零反例=1 且报 PARSE-FAIL); "
    "(2) 断言退出码必须区分「确实有问题」(1) 与「没测到」(2) —— 0 违规与没解析出来不可混为一谈; "
    "(3) 凡靠 JS/框架产生最终态的断言, 第一句必验「机器真的启动过」的独立证据 "
    "(入口函数加的标记类如 `html.anim`、初始化写入 window 的值、计数器自增), "
    "让「没跑」以可读字符串报出而不是静默通过; 往返类用例要在**中途**取快照并断言中途确实翻转过, "
    "否则「一次都没动过」也会通过。 "
    "**附带**: 共享静态资产必先过语法检查 (`node --check`) 再跑运行时门禁 —— "
    "在 /* */ 注释里原样引用含 `\\s*/` 的行首锚点正则, 其中的星号斜杠序列会**当场结束注释**, "
    "后面整段中文被当成代码解析 → SyntaxError → 整份脚本一行没跑; "
    "参考 `arch_live_doc_check.py` 的 `check_assets()` (实跑: 注入语法错误 exit=1, 正常 exit=0) "
    "| `docs/arch-live/README.md` 「为什么交互自检不能省」bug 表 (5 条, 含「怎么发现的」列) "
    "+ `docs/arch-live/08-guardrails.html` §04 陷阱 ④ + `scripts/automation/arch_live_interact_check.py` `MUTATIONS` (8 条) "
    "+ `scripts/automation/arch_live_doc_check.py` `check_assets()` / `FLOORS` |"
)


REV_MARKER = "| v0.75 | 2026-10-05 10:31 JST |"

# 修订历史表 5 列: 版本 | 日期 | 修订人 | 修订内容 | 触发
# 版本号必须动态取现有最大值 +1 —— 硬编码是本次实测踩的坑: 我只看了文件末尾一屏,
# 硬编码 v0.75 落库, 结果该版本号早已存在(实际表已到 v0.81), 直接制造重号。
# AGENTS.md §4.1.1 早就把"重号"记成待修问题, 不该由我这条再加一个。
REV_ROW_TMPL = (
    "| {ver} | 2026-10-05 10:31 JST | "
    "Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | "
    "**§4.1 派生规新增 v30: 门禁必须有变异测试, 且断言要能区分「被测物没跑」** | "
    "per arch-live 文档站 (docs/arch-live, 9 页) 实测: 交互自检一次跑出 5 个 bug "
    "(其中「整站 JS 静默失效」由 arch.js 块注释里的行首锚点正则引发 —— 行首锚点里的 "
    "反斜杠-s-星号-斜杠 序列当场结束注释), 且暴露出「修复把已有断言的鉴别力磨掉」: "
    "修「无 JS 数字全是 0」时把 data-count 初始文本改成真实值, 于是 "
    "`textContent === data-count` 在一个 JS 都没跑的页面上同样成立, "
    "后来 JS 整体挂掉时这条断言依然全绿。 "
    "Ulysses 2026-10-05 10:31 JST ask_user 拍板选项 1「写进 AGENTS.md, 编号 v30」(推荐项)。 "
    "落地脚本 `scripts/automation/agents_guard_v30.py` (幂等 + 复读校验 + 列数一致性) |"
)

REV_SIG = "§4.1 派生规新增 v30"   # 与版本号无关的内容指纹, 用于幂等与纠错


def existing_versions(src: str) -> list[float]:
    return [float(m) for m in re.findall(r"^\| v0\.(\d+) \|", src, re.M)]


def append_rev(src: str) -> tuple[str, list[str]]:
    """在修订历史表末行之后追加一行. 幂等, 且版本号动态取 max+1."""
    notes: list[str] = []
    if REV_SIG in src:
        notes.append("修订历史已含 v30 记录, 幂等跳过")
        return src, notes
    vers = existing_versions(src)
    if not vers:
        print("FAIL: 修订历史表未找到 '| v0.NN |' 行, 结构可能已变, 需人工确认", file=sys.stderr)
        raise SystemExit(2)
    nxt = f"v0.{int(max(vers)) + 1}"
    dupes = sorted({v for v in vers if vers.count(v) > 1})
    if dupes:
        notes.append(f"提示: 表内已存在的重号版本 {', '.join('v0.%d' % d for d in dupes)} "
                     f"(本次不改动, per 守门 #1 禁回溯叙事); 新行取 {nxt}")
    lines = src.splitlines(keepends=True)
    hit = max(i for i, l in enumerate(lines) if l.startswith("| v0."))
    lines.insert(hit + 1, REV_ROW_TMPL.format(ver=nxt) + "\n")
    notes.append(f"修订历史 {nxt} 已插入 (line {hit + 2}, 动态取 max+1)")
    return "".join(lines), notes



def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--check", action="store_true", help="只校验 v30 是否已落库, 不改文件")
    args = ap.parse_args()

    if not AGENTS.exists():
        print(f"FAIL: AGENTS.md not found: {AGENTS}", file=sys.stderr)
        return 2
    src = AGENTS.read_text(encoding="utf-8")

    already = MARKER in src
    if args.check:
        ok = already and REV_SIG in src
        print(("PASS: v30 + 修订历史均已落库" if ok else "MISS: v30 或修订历史未落库"),
              file=sys.stderr if not ok else sys.stdout)
        return 0 if ok else 1

    if already:
        print("PASS: v30 已在表内, 幂等跳过 §4.1 插入")
    else:
        # 定位 v29 行末 (表里派生规的末行), 在其后插入
        lines = src.splitlines(keepends=True)
        hit = -1
        for i, line in enumerate(lines):
            if line.startswith(ANCHOR_PREFIX):
                hit = i
        if hit < 0:
            print(f"FAIL: 锚点行未找到 (前缀 {ANCHOR_PREFIX!r}), 表格结构可能已变, 需人工确认",
                  file=sys.stderr)
            return 2
        if any(l.startswith(ANCHOR_PREFIX) for l in lines[hit + 1:]):
            print("FAIL: v29 不唯一, 锚点有歧义, 拒绝盲插", file=sys.stderr)
            return 2
        lines.insert(hit + 1, V30_ROW + "\n")
        src = "".join(lines)
        print(f"PASS: v30 已插入 §4.1 派生规表 (line {hit + 2})")

    src, notes = append_rev(src)
    for n in notes:
        print("     " + n)

    AGENTS.write_text(src, encoding="utf-8", newline="")

    # 落库后自检: 重新读回来确认真的写进去了, 不靠"我写过了"当证据。
    # 这正是 v30 第 (3) 条的精神: 写入动作本身也要有独立证据。
    after = AGENTS.read_text(encoding="utf-8")
    problems = []
    if after.count(MARKER) != 1:
        problems.append(f"FAIL: v30 出现 {after.count(MARKER)} 次, 应恰好 1 次")
    if after.count(REV_SIG) != 1:
        problems.append(f"FAIL: v30 修订历史记录出现 {after.count(REV_SIG)} 次, 应恰好 1 次")
    # 版本号重号自检: 新增一行后, 不应引入新的重号
    vers = existing_versions(after)
    dupes = {v for v in vers if vers.count(v) > 1}
    problems.extend(f"FAIL: 版本号 v0.{int(d)} 重号 (本次新增导致)" for d in sorted(dupes)
                    if int(d) >= 82)
    if problems:
        for p in problems:
            print(p, file=sys.stderr)
        return 1
    # 表格列数一致性: v30 必须与相邻行同列数, 否则 markdown 表会散
    alines = after.splitlines()
    v29 = next((l for l in alines if l.startswith(ANCHOR_PREFIX)), "")
    v30 = next((l for l in alines if l.startswith("| v30 |")), "")
    if v29.count("|") != v30.count("|"):
        print(f"FAIL: v30 列数({v30.count('|')}) 与 v29({v29.count('|')}) 不一致, 表格会散",
              file=sys.stderr)
        return 1
    print(f"PASS: 复读校验 1 处命中 + 列数一致 (v29/v30 均 {v29.count('|')} 个竖线)")
    print("      commit 时引用本脚本路径 scripts/automation/agents_guard_v30.py")
    return 0



if __name__ == "__main__":
    raise SystemExit(main())
