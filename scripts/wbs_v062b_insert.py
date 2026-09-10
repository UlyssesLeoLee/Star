"""v0.62b WBS row insertion (re-attempt after main advanced to f57aed4 with ARG.8 commit)

Inserts v0.62b row after v0.59 (or wherever the last v5x/v6x row is) in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v4 Mavis 审核决定 author=Ulysses.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V062_ROW = """| **v0.62** | **2026-09-10 12:45 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v3 → v4 反转升级 + 守门 #9 v19 Mavis 自驱 + 守门 #12 v15 docs 同步饱和第 51 次新事件触发 仍允许)** | **§4 守门 #14 v3 → v4 反转 + v30/v31 政策取消 + v32 候选新增 (per 2026-09-10 12:45 JST Ulysses 发令"真人代签流程全部取消, 改为 mavis 审核")**：**(1) commit `<pending>` v0.62 反转落地** (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19 第 6 次强化 Mavis 全权代理 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v3 → v4 反转): 6 files changed: (a) `AGENTS.md` §4 守门 #14 表格行内容反转 (到位 timeline = 不适用 / Mavis 边界 = 审核决定 author=Ulysses, 替代 v0.61 的"全部代签"边界); (b) `AGENTS.md` §4.1.1 v30 + v31 2 行 status → 🟡 **v0.62 反转: 政策已取消**, 新增 v32 候选 "Mavis 审核决定 author=Ulysses 政策" (🟡 待拍板激活); (c) `docs/guardian/v30_signature_boundary.md` status → 🟡 v0.62 反转 + 修订历史 v0.2 row; (d) `docs/guardian/v31_lead_traceback.md` status → 🟡 v0.62 反转 + 修订历史 v0.2 row; (e) `docs/recruitment/5-business-domain-lead-referral.md` v0.3 反转 (真人代签流程全部取消, 本 brief 仍保留作为参考但不强制); (f) `docs/reports/STAR-P3-WBS-001.md` v0.62 row (本 commit); **(2) 守门 #14 v3 → v4 反转差异** (per 守门 #1 禁回溯叙事 + 守门 #14 v4 升级): v3 = Mavis 永久代签全部签字栏 (实际签字是 Mavis, 责任也是 Mavis); v4 = Mavis 审核决定 author=Ulysses (Mavis 审核 + author=Ulysses, 责任更清晰: Mavis 承担审核责任, Ulysses 是名义 author); **(3) 累计 v3x 5 候选 → 6 候选**: v27/v28/v29 🟢 active + v30/v31 🟡 反转 (v0.62) + v32 🟡 新增候选 (v0.62); **(4) 阻塞 5 → 0 实际** (v0.62 反转后): 5 域 Lead 真人寻访 流程**全部作废** (per v31 取消), 跨 session 续做项 解锁 (P0-3 application crate + P0-4 infrastructure adapter + 5/6 Repository PG 容器化 + envoy 业务路由 + TLS 自动签发) 不再依赖真人到位, Mavis 可直接推进; **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.61 修订历史不动; v30/v31 placeholder 文档保留不删 (作为历史); 守门 #3 (5 域独立 Lead) 维持 (跨域 Lead 决策结构本身不变, 只是 Mavis 角色从代签→审核); 守门 #10 (author=Ulysses) 维持; **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v4 Mavis 审核决定): (a) WBS v0.62 row 同步 (本 commit); (b) 修订历史 v0.62 row 同步 (本 commit); (c) v0.63 = P0-3 application crate 编排 (0.6M tokens, Mavis 推进); (d) v0.64 = P0-4 infrastructure adapter (0.4M tokens, Mavis 推进); (e) v0.65 = 5/6 Repository PG 容器化 (2M tokens, Mavis 推进); (f) v0.66 = envoy 业务路由 + TLS 自动签发 (1-2M tokens, Mavis 推进); (g) v32 候选拍板激活 (待 v0.62 docs 同步后); **(7) 触发**: 9/10 12:45 JST 用户发令"真人代签流程全部取消, 改为 mavis 审核" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:19/15:29 强化 + 守门 #14 v3 → v4 反转 + 守门 #32 新候选) + 守门 #1 禁回溯叙事 (placeholder 不删) + commit author=Ulysses (per 守门 #10 + 守门 #14 v4) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.62-v14v4-mavis-audit" in content or "v0.62b-v14v4-mavis-audit" in content:
        print("OK: v0.62 row already present (idempotent skip)")
        return
    # Find v0.61 row (most recent v0.6x)
    pat = re.compile(r"(\| \*\*v0\.61\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.61 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V062_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.62 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
