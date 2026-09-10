"""v0.70 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 59 次新事件触发)"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V070_ROW = """| **v0.70** | **2026-09-10 14:55 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事 + 守门 #12 v15 docs 同步饱和第 59 次新事件触发 仍允许)** | **§14.19 守门 v3x v32 候选拍板激活 (per 9/10 14:55 JST Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses)**：**(1) commit `cc4c04f` 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses + 9/8 15:19/15:29 强化): 2 files changed, 51 insertions(+), 1 deletion(-): (a) `docs/guardian/v32_audit_boundary.md` 5.6KB (新文件, 跟 v30/v31 模式对称): 5 类审核决策表 (commit / 报告 / WBS row / 守门 v3x 候选 / docs sync 饱和) + 6 适用边界 (a-f, 跟 v30 类似) + 跟 v30 反转关系 (v32 = v30 替代政策) + 已知缺口 3 项 (v30/v31 反转标提交历史, host 状态改变判定, 整体方向大转弯定义未明) + 修订历史 v0.1 row; (b) `AGENTS.md` §4.1.1 v32 row status 🟡 待拍板激活 → 🟢 **active (v0.70 拍板激活)**; **(2) 守门 #14 v4 实证**: Mavis 审核 author=Ulysses (per 9/10 12:45 v0.62 反转 + 9/10 14:55 v0.70 拍板激活); Mavis 承担**审核责任** (不是代签责任, 责任更清晰); **(3) 累计 v3x 6 候选状态**: v27/v28/v29 🟢 active (v0.56-v0.61) + v30/v31 🟡 反转 (v0.62) + **v32 🟢 active (v0.70 拍板激活)**; **(4) 7 段结构** (§3 + §4 + §6): **§3 已知缺口** = (a) v30/v31 placeholder 文档保留 (per 守门 #1 禁回溯叙事), v32 替代 v30; (b) v32 适用边界 (b) "整体方向大转弯" 定义未明, 需 v0.71+ follow-up 显式列 (e.g. 5 域模型改 / 守门 #14 政策再反转 / 大规模 CRUD 切换); (c) v32 适用边界 (b) "host 状态永久改变" 的具体判定: 5 域 Lead 真人寻访流程已取消 (per v0.62), host 状态改变主要涉及 k3s cluster / 数据库 / 域名 / OAuth client_secret; **§4 子代理失败接手清单** = N/A (本 commit 是 root session 直接实装, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC); **§6 签字栏** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 永久代签 author=Ulysses (per 守门 #14 v4 审核); **(5) 阻塞 0 → 0 实际** (v0.62 反转后跨 session 续做项 解锁, Mavis 可推进); **(6) 守门 #1 禁回溯叙事** — v0.1-v0.69 row 不动, v0.70 row 显式标 v32 拍板激活; **(7) 触发**: 9/10 14:50 JST 用户发令"继续" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 永久代签); commit author=Ulysses (per 守门 #10 + 守门 #14 v4) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.70-v32-activate" in content:
        print("OK: v0.70 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.69\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.69 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V070_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.70 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
