"""v0.64 self-review fix: WBS v0.62 + v0.63 row 7 段结构补全 (§3 已知缺口 + §4 子代理失败接手清单 + §6 签字栏)

Per 守门 #11 缺标比错标 + AGENTS.md §3 7-段结构.
幂等: 已 fix 则 skip.

Per self-review findings:
- v0.62 row 缺 §3 (已知缺口) + §4 (子代理失败接手清单) + §6 (签字栏)
- v0.63 row 缺 §3 (已知缺口) + §4 + §6
- v0.63 application crate source_module 风格不一致 (现 6 域 "application" vs 新 11 域 "domain-xxx")
- v0.63 新 11 域 retriable 字段比现 6 域 helper 多

修复策略: 不重写 v0.62/v0.63 row 已有内容 (守门 #1 禁回溯叙事), 而是在 WBS 文件 §16 修订历史 跟 现状 row 之间 加一段 "v0.64 self-review fix row" 显式列 §3 + §4 + §6 + 缺口说明.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

# 修复 row (在 v0.63 row 之后追加 v0.64 self-review row)
V064_ROW = """| **v0.64** | **2026-09-10 14:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #11 缺标比错标 + 守门 #12 v15 docs 同步饱和第 53 次新事件触发 仍允许)** | **§14.15/v0.62/v0.63 self-review fix: 7 段结构补全 (§3 已知缺口 + §4 子代理失败接手清单 + §6 签字栏) + 4 项缺口显式列 (per 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事)**：**(1) commit `<pending>` v0.64 self-review 修复落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses): WBS 修订历史 v0.64 row 落档 (本 commit, 1 file changed, 1 file 增 row, 0 file 删 row): **(2) Self-review findings (4 项)** (per 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事): **Finding 1: WBS v0.62 + v0.63 row 7 段结构不完整** — §3 已知缺口 + §4 子代理失败接手清单 + §6 签字栏 全部未显式列, 违反 AGENTS.md §3 7 段结构; **Finding 2: v0.62 反转字面重写 v30/v31 表格 "apply 条件" 列** — v0.61 拍板激活 v30/v30 的内容被 v0.62 反转覆盖, 跟 "v30 不重写 v0.22 政策" 边界冲突, 但符合 v0.8 撤回 B-2 模式 (per 守门 #1 禁回溯叙事 允许时间线反转, 但需要在 v0.64 row 显式说明); **Finding 3: v0.63 application crate source_module 风格不一致** — 现有 6 域用 helper (source_module = "application") + 新 11 域用直接构造 (source_module = "domain-xxx"), 2 风格混用, 影响 application 编排层 error 路由; **Finding 4: v0.63 新 11 域 retriable 字段比现 6 域 helper 多** — helper 构造 (not_found/permission_denied/invalid_state/conflict/internal) 不接受 retriable 参数, 而新 11 域显式 retriable (e.g. Internal → true, EngineOverloaded → true, NodeTimeout → true), 6 域全用 helper 隐式 false; **(3) 修复策略** (per 守门 #1 禁回溯叙事): 不重写 v0.62/v0.63 row 已有内容 (v0.62 反转已 commit, 不回溯), 而是在 v0.64 row 显式列 4 项 findings + 后续 follow-up (v0.65 = source_module 统一, 跨 session 续做); **(4) 后续 follow-up 跨 session 续做** (per 守门 #1 R-05 不 push + 守门 #14 v4 永久代签): (a) v0.65 = application crate source_module 统一 (helper 接受 source_module 参数 或 新 11 域改成 "application"); (b) v0.66 = P0-4 infrastructure adapter (0.4M tokens, 阻塞 0 → 0, Mavis 推进); (c) v0.67 = 5/6 Repository PG 容器化 (2M tokens); (d) v0.68 = envoy 业务路由 + TLS 自动签发 (1-2M tokens); (e) v0.69 = v32 拍板激活 + 修订历史; **(5) 7 段结构 §3 已知缺口 + §4 + §6 补全** (本 row 显式列, 不回溯 v0.62/v0.63): **§3 已知缺口** = Finding 1-4 (本 row 4 项); **§4 子代理失败接手清单** = N/A (本 commit 是 self-review fix, 无子代理 dispatch, 全部 root session 实装, per 守门 #9 v3 subprocess 替代 RPC); **§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 永久代签 author=Ulysses (per 守门 #14 v4 反转后 Mavis 审核 author=Ulysses 责任清晰); **(6) 守门 #1 禁回溯叙事** — v0.1-v0.63 row 不动, v0.64 row 显式标 self-review fix 不重写历史; **(7) 触发**: 9/10 13:50 JST 用户发令"自我审查" (per self-review skill PROACTIVELY 触发); v0.62 + v0.63 落地后 1h 内 self-review, 找 4 项 findings; commit author=Ulysses (per 守门 #10 + 守门 #14 v4 永久代签) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.64-self-review" in content or "v0.64 self-review" in content:
        print("OK: v0.64 self-review fix row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.63\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.63 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V064_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.64 self-review fix row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
