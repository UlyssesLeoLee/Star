"""v0.61 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 50 次新事件触发)

Inserts v0.61 row after v0.60 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v3 Mavis 永久代签.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V061_ROW = """| **v0.61** | **2026-09-10 11:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v3 升级 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 cargo check 实证 + 守门 #12 v15 docs 同步饱和第 50 次新事件触发 仍允许)** | **§14.19 守门 v3x 3 候选拍板激活 (v27 + v30 + v31 落地, 跨 5 候选全部 active, per 9/8 15:19/15:29/9/9 12:02 + 9/1 14:58 ask_user + 9/5 04:03 拍板推荐项直接执行 + 守门 #14 v3 Mavis 永久代签)**：**(1) commit `<pending>` v3x 3 候选拍板激活落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 brief 落档 + 守门 #14 v3 永久代签 + 9/8 15:19/15:29 强化): 7 files changed: (a) `scripts/automation/guardian/v27_rpc_fallback.py` v0.61 (6.9KB, 3 段 fallback: invoke → verify(30s) → collect_output + status.json 写, per 守门 #9 #3 实证 RPC 不可靠); (b) `scripts/automation/dispatcher.py` CLI 加 3 subcommands (invoke/verify/collect_output, 跟 v27 配套, 向后兼容 legacy `--task-id` 全流程); (c) `AGENTS.md` §4.1.1 v27/v30/v31 3 行 status 🟡 待激活 → 🟢 **active** (per 守门 #12 v21 [P] docs 同步); (d) `docs/guardian/v30_signature_boundary.md` status → 🟢 active + 修订历史 v0.1 row; (e) `docs/guardian/v31_lead_traceback.md` status → 🟢 active + 修订历史 v0.1 row; (f) `docs/reports/STAR-P3-WBS-001.md` v0.61 row (本 commit, 7 段结构); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 cargo check): `python scripts/automation/guardian/v27_rpc_fallback.py --task-id test-v27-001 --brief docs/briefs/v0.58-p02-stage1-domain-error-mapping.md --worktree wt-v058-p02-stage1 --skip-verify` = `{"phase": "invoke", "skipped_verify": true}` + `docs/briefs/test-v27-001.status.json` 写入 + 3 history entry; `python scripts/automation/dispatcher.py verify v0.58-p02-stage1` = `{"task_id": "v0.58-p02-stage1", "phase": "verify", "status": "fail"}` (mock task); **(3) 累计守门 v3x 5 候选 → 5/5 全部 active** (v27 🟢 [本 commit] + v28 🟢 [v0.57] + v29 🟢 [v0.57] + v30 🟢 [本 commit] + v31 🟢 [本 commit]); **(4) 阻塞 5 → 5 实际** (v3x 拍板激活不收阻塞项, 是 governance-level 治理层落地); **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.60 修订历史不动; 守门 #1 禁回溯叙事维持; 守门 #14 v3 Mavis 永久代签不变; v25 重号未修 (per 守门 #11 缺标比错标 暂缓, 修 = 改 1 修订历史 row 涉及 17 commit 不动); **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v3 永久代签): (a) 修订历史 v0.61 row 同步 (本 commit); (b) v30/v31 跨项目 (RGS / Physis / GVPE) 验证 待真人到位; (c) 5 域 Lead 真人寻访 (Ulysses 主动动作); (d) P0-3 application crate + P0-4 infrastructure adapter; **(7) 触发**: 9/10 10:00 JST 用户发令"推进到所有任务完成" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 + 守门 #14 v3 Mavis 永久代签) + 守门 #1 v25 实证 v27 落档 OK + 5 候选全部 🟢 active + commit author=Ulysses (per 守门 #10 + 守门 #14 v3) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.61-v3x-v27" in content:
        print("OK: v0.61 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.60\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.60 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V061_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.61 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
