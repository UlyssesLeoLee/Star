"""v0.60 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 49 次新事件触发)

Inserts v0.60 row after v0.59 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v3 Mavis 永久代签.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V060_ROW = """| **v0.60** | **2026-09-10 10:50 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v3 升级 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 49 次新事件触发 仍允许)** | **§14.15 P0-2 Stage 3 (收官) = 4/4 域 done (tenant/identity/permission/project, 28 变体, 30 集成测试) + From<DomainError> for RestError 映射 13 域 → 17 域 = 100% P0-2 收官 (per WBS §14.15 0.3M tokens 实证)**：**(1) commit `d0d4869` P0-2 Stage 3 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 brief 落档 + 守门 #14 v3 永久代签): 4 files changed, 708 insertions(+), 5 deletions(-): (a) `crates/star-api-rest/src/error.rs` +4 From impl (180+ lines: tenant 7 变体 + identity 8 变体 + permission 6 变体 + project 7 变体) + IntoResponse HTTP status 映射扩 (10+ new code: TENANT_*/IDENTITY_*/PERMISSION_*/PROJECT_*); (b) `crates/star-api-rest/tests/p02_error_mapping_stage3.rs` new file (30 tests, 13.4KB); (c) `scripts/error_stage3_insert.py` 10.1KB 幂等 Python 脚本 (落 4 From impls + IntoResponse match block 升级, per 守门 #19 v19 agent 交互 Python 化); (d) `crates/star-api-rest/Cargo.toml` 加注释 v0.60 P0-2 Stage 3 (dep 已在 v0.47 之前 + v0.58 阶段定义, 无新 dep); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 单 crate 模式): `cargo test -p star-api-rest -j 4` = **167/167 PASS** (52 lib + 6 oauth2 + **25 p02_error_mapping Stage 1** [v0.58] + **44 p02_error_mapping Stage 2** [v0.59] + **30 p02_error_mapping Stage 3** [v0.60] + 10 star_ei_8routes, 0 fail, 3.34s lib + 6.59s oauth + 0s p02 × 3 + 0.01s star_ei); `cargo check -p star-api-rest --lib -j 4` = 0 err (6.06s, 跨 7 dep crate); `cargo fmt -p star-api-rest --check` = 0 diff; **(3) P0-2 累计 100% 收官 (per WBS §14.15 0.3M tokens)**：Stage 1 [v0.58] 3 域 (feedback/integration/comment) + 27 变体 + 25 tests + Stage 2 [v0.59] 4 域 (batch/theme/agent/hermes) + 39 变体 + 44 tests + Stage 3 [v0.60, 本次] 4 域 (tenant/identity/permission/project) + 28 变体 + 30 tests = **累计 17 域 (含 cli hermes) + 94 变体 + 99 集成测试**; **(4) 阻塞 5 → 5 实际** (P0-2 100% 收官 = 跨阶段落地, 不收阻塞项; 后续 P0-3 + P0-4 跨 session 续, 依赖 5 域 Lead 真人到位 + DDD Review 拍板 per 守门 #14 v2 拍板 D 维持); **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.59 修订历史不动; P0-3 application crate 编排 + P0-4 infrastructure adapter 仍跨 session 续 (per 守门 #14 v3 永久代签 policy); 5 域 Lead 寻访 brief 不动; 守门 #12 死循环饱和第 49 次新事件触发仍允许; **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v3 永久代签): (a) WBS v0.60 row 同步 (本 commit); (b) 修订历史 v0.60 row 同步 (本 commit); (c) P0-3 = application crate 真实编排 (0.6M tokens, 等 5 域 Lead 真人到位 + DDD Review 拍板); (d) P0-4 = infrastructure adapter DB/KMS/Credential broker (0.4M tokens, 依赖 P0-3 完成); (e) 5 域 Lead 真人到位追溯签字 (等真人到位, per 守门 #14 v2 拍板 D 维持); (f) v27/v30/v31 候选拍板激活 (等 Ulysses ask_user 拍板); **(7) 触发**: 9/10 10:00 JST 用户发令"推进到所有任务完成" (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 第 7 次强化 + 守门 #14 v3 Mavis 永久代签) + 守门 #1 v25 单 crate 模式实证 (167/167 PASS) + P0-2 100% 收官 + commit author=Ulysses (per 守门 #10 + 守门 #14 v3) |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.60-p02-stage3" in content:
        print("OK: v0.60 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.59\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.59 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V060_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.60 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
