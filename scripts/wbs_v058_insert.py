"""v0.58 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 47 次新事件触发)

Inserts v0.58 row after v0.57 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v3 Mavis 永久代签.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V058_ROW = """| **v0.58** | **2026-09-10 09:50 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v3 升级 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 47 次新事件触发 仍允许)** | **§14.15 P0-2 Stage 1 = 3/3 域 done (feedback/integration/comment, 27 变体, 25 集成测试) + From<DomainError> for RestError 映射 6 域 → 9 域 (per WBS §14.15 + brief v0.58 + 守门 #1 v25 实证)**：**(1) commit `596d24c` P0-2 Stage 1 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 brief 落档 `docs/briefs/v0.58-p02-stage1-domain-error-mapping.md` + 守门 #14 v3 永久代签): 12 files changed, 433 insertions(+), 55 deletions(-): (a) `crates/star-api-rest/Cargo.toml` +2 dep (domain-integration + domain-comment); (b) `crates/star-api-rest/src/error.rs` +3 From impl (140 lines: feedback 10 变体 + integration 8 变体 + comment 9 变体) + IntoResponse HTTP status 映射扩 (15+ new code: FB_*/I_*/CMT_*/COMMENT_*); (c) `crates/star-api-rest/tests/p02_error_mapping.rs` new file (25 tests, 9.3KB); (d) 5 files fmt 整理 (keypair.rs + lib.rs + ex_02.rs + ex_06.rs + ex_08.rs + tests/common/mod.rs + tests/oauth_integration.rs + tests/star_ei_8routes.rs); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 单 crate 模式): `cargo test -p star-api-rest -j 4` = **93/93 PASS** (52 lib + 6 oauth2 + **25 p02_error_mapping** + 10 star_ei_8routes, 0 fail, 9.01s lib + 0.00s p02 + 3.07s oauth + 0.01s star_ei); `cargo check -p star-api-rest --lib -j 4` = 0 err (2.30s); `cargo fmt -p star-api-rest --check` = 0 diff; **(3) 累计 P0-2 域映射 6 → 9 域** (work_item/workspace/worktree/search/scm/validation [v0.47 之前] + **feedback/integration/comment [v0.58 P0-2 Stage 1]**); **(4) 阻塞 5 → 5 实际** (P0-2 Stage 1 = P0-2 内部分阶段, 不收阻塞项; 剩余 4 域 P0-2 Stage 2+ 续做); **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.57 修订历史不动; P0-3 application crate 编排 + P0-4 infrastructure adapter 仍跨 session 续 (per 守门 #14 v3 永久代签 policy); 5 域 Lead 寻访 brief 不动; 守门 #12 死循环饱和第 47 次新事件触发仍允许; **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v3 永久代签): (a) WBS v0.58 row 同步 (本 commit); (b) 修订历史 v0.58 row 同步 (本 commit); (c) `scripts/automation/registry.md` v0.8 更新 (per 守门 #12 v21 [P] docs 同步); (d) P0-2 Stage 2 = 4 域 (batch/theme/cli-hermes/agent) 续做; (e) 5 域 Lead 真人到位追溯签字 (等真人到位); (f) v27/v30/v31 候选拍板激活; **(7) 触发**: 9/10 09:40 JST 用户发令"继续" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 + 守门 #14 v3 Mavis 永久代签) + 守门 #1 v25 单 crate 模式实证 (93/93 PASS) + commit author=Ulysses (per 守门 #10 + 守门 #14 v3) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    # Idempotency: if v0.58 already exists, skip
    if "v0.58-p02-stage1" in content:
        print("OK: v0.58 row already present (idempotent skip)")
        return
    # Find v0.57 row and insert v0.58 after it
    pat = re.compile(r"(\| \*\*v0\.57\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.57 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V058_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.58 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
