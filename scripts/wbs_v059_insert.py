"""v0.59 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 48 次新事件触发)

Inserts v0.59 row after v0.58 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v3 Mavis 永久代签.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V059_ROW = """| **v0.59** | **2026-09-10 10:25 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v3 升级 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 48 次新事件触发 仍允许)** | **§14.15 P0-2 Stage 2 = 4/4 域 done (batch/theme/agent + cli-hermes, 39 变体, 44 集成测试) + From<DomainError> for RestError 映射 9 域 → 13 域 (per WBS §14.15 + brief v0.59 + 守门 #1 v25 实证)**：**(1) commit `308ae32` P0-2 Stage 2 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 brief 落档 `docs/briefs/v0.59-p02-stage2-domain-error-mapping.md` + 守门 #14 v3 永久代签): 5 files changed, 899 insertions(+), 4 deletions(-): (a) `crates/star-api-rest/Cargo.toml` +3 dep (domain-batch + domain-theme + domain-cli; domain-agent 已在 Phase B.2.6 line 61); (b) `crates/star-api-rest/src/error.rs` +4 From impl (180+ lines: batch 19 变体 + theme 8 变体 + agent 9 变体 + hermes 4 变体) + IntoResponse HTTP status 映射扩 (15+ new code: BATCH_*/AGENT_*/THEME_*/HERMES_* + 401/408/503 status 加); (c) `crates/star-api-rest/tests/p02_error_mapping_stage2.rs` new file (44 tests, 14.7KB); (d) `scripts/error_stage2_insert.py` 12.5KB 幂等 Python 脚本 (落 4 From impls + IntoResponse match block 升级, per 守门 #19 v19 agent 交互 Python 化); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 单 crate 模式): `cargo test -p star-api-rest -j 4` = **137/137 PASS** (52 lib + 6 oauth2 + **25 p02_error_mapping Stage 1** [v0.58] + **44 p02_error_mapping Stage 2** [v0.59] + 10 star_ei_8routes, 0 fail, 7.87s lib + 3.47s oauth + 0s p02_stage1 + 0s p02_stage2 + 0.01s star_ei); `cargo check -p star-api-rest --lib -j 4` = 0 err (1m 5s, 跨 7 dep crate); `cargo fmt -p star-api-rest --check` = 0 diff; **(3) 累计 P0-2 域映射 9 → 13 域** (work_item/workspace/worktree/search/scm/validation [v0.47 之前] + feedback/integration/comment [v0.58] + **batch/theme/agent + cli(hermes) [v0.59]**); **(4) 阻塞 5 → 5 实际** (P0-2 Stage 2 = P0-2 内部分阶段, 不收阻塞项; 剩余 P0-2 Stage 3 = 4 域 tenant/identity/permission/project 续做 + P0-3 + P0-4 跨 session 续); **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.58 修订历史不动; P0-3 application crate 编排 + P0-4 infrastructure adapter 仍跨 session 续 (per 守门 #14 v3 永久代签 policy); 5 域 Lead 寻访 brief 不动; 守门 #12 死循环饱和第 48 次新事件触发仍允许; **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v3 永久代签): (a) WBS v0.59 row 同步 (本 commit); (b) 修订历史 v0.59 row 同步 (本 commit); (c) P0-2 Stage 3 = 4 域 (tenant/identity/permission/project) 续做; (d) 5 域 Lead 真人到位追溯签字 (等真人到位); (e) v27/v30/v31 候选拍板激活; **(7) 触发**: 9/10 10:00 JST 用户发令"推进到所有任务完成" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 + 守门 #14 v3 Mavis 永久代签) + 守门 #1 v25 单 crate 模式实证 (137/137 PASS) + commit author=Ulysses (per 守门 #10 + 守门 #14 v3) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.59-p02-stage2" in content:
        print("OK: v0.59 row already present (idempotent skip)")
        return
    # Find v0.58 row and insert v0.59 after it
    pat = re.compile(r"(\| \*\*v0\.58\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.58 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V059_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.59 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
