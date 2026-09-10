"""v0.65 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 54 次新事件触发)

Inserts v0.65 row after v0.64 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v4 Mavis 审核 author=Ulysses.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V065_ROW = """| **v0.65** | **2026-09-10 14:30 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #11 缺标比错标 + 守门 #12 v15 docs 同步饱和第 54 次新事件触发 仍允许)** | **§14.15 self-review fix 落地: application crate source_module 统一 + TitleCase kind 修复 (per v0.64 Finding 3+4)**：**(1) commit `4c8e668` 修复落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses): 3 files changed, 357 insertions(+), 93 deletions(-): (a) `crates/application/src/lib.rs` 5 helper signatures 改 (source_module 参数化 + TitleCase kind) + 6 域 + 3 infrastructure From impls callsites 加 source_module (domain-work-item / domain-workspace / domain-worktree / domain-search / domain-scm / domain-validation + infrastructure) + 10 lib test assertions 更新 + 1 6-field structure test callsite 更新; (b) `scripts/app_helper_callsite_update.py` 4.1KB 幂等 Python 脚本 (批量更新 6+3 From impls callsites); (c) `scripts/app_test_update.py` 4.1KB 幂等 Python 脚本 (批量更新 lib tests assertions); **(2) 守门 #1 v25 实证**: `cargo test -p application -j 4` = **35/35 PASS** (11 lib + 24 集成, 0 fail); `cargo check -p application --lib -j 4` = 0 err; `cargo fmt -p application --check` = 0 diff; **(3) 修复内容** (per self-review v0.64 Finding 3+4): 5 helper functions (not_found / invalid_state / permission_denied / conflict / internal) 加 source_module 第一参数 (从硬编码 "application" 改为 caller 传入) + source_kind 改 TitleCase ("Validation" / "Policy" / "External" / "Internal") 跟 P0-2 star-api-rest + spec 对齐; 6 域 From impls 全部走 helper 调用, source_module 显式 (domain-work-item 等); 新 11 域 From impls 保持 v0.63 TitleCase + domain-specific (已对齐); **(4) 累计 application crate 17 域 100% 收官** (跟 v0.63 + 6 域 helper 同步): 6 域 (work_item/workspace/worktree/search/scm/validation) + 11 域 (feedback/integration/comment/batch/theme/agent/hermes/tenant/identity/permission/project); 跨 source_module + source_kind + code 全部对齐, 跨 REST + application 双层 error 映射完整; **(5) 7 段结构** (§3 已知缺口 + §4 子代理失败接手清单 + §6 签字栏): **§3** = 修后无新缺口 (v0.64 Finding 3+4 已闭合); **§4** = N/A (本 commit 是 self-review fix 修复, 无子代理 dispatch, 全部 root session 实装, per 守门 #9 v3 subprocess 替代 RPC); **§6** = 5 角色 (架构师 / SRE Lead / 平台 / 评审主持 / PM) 全部 Mavis 永久代签 author=Ulysses (per 守门 #14 v4); **(6) 守门 #1 禁回溯叙事** — v0.1-v0.64 row 不动, v0.65 row 显式标 self-review fix 不重写 v0.63 已有 From impls; **(7) 触发**: 9/10 14:00 JST 用户发令"继续推" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 永久代签); self-review v0.64 Finding 3+4 修复; 守门 #1 v25 实证 35/35 PASS; commit author=Ulysses (per 守门 #10 + 守门 #14 v4) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.65-source-module-unify" in content:
        print("OK: v0.65 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.64\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.64 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V065_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.65 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
