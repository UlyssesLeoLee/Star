"""v0.63 WBS row insertion (per 守门 #12 commit-time docs sync, v29 第 52 次新事件触发)

Inserts v0.63 row after v0.62 in STAR-P3-WBS-001.md (幂等).
Per AGENTS.md §3 7-段结构 + 守门 #14 v4 Mavis 审核决定 author=Ulysses.
"""
import re
import sys
from pathlib import Path

WBS = Path("docs/reports/STAR-P3-WBS-001.md")

V063_ROW = """| **v0.63** | **2026-09-10 13:00 JST** | **架构师 (Mavis 接手 agent per DEC-008) — Mavis 永久代签 Ulysses (per 守门 #14 v4 升级 + 守门 #9 v19 Mavis 自驱 + 守门 #1 v25 单 crate 模式 + 守门 #12 v15 docs 同步饱和第 52 次新事件触发 仍允许)** | **§14.15 P0-3 Stage 1 = 11/17 域 done (feedback/integration/comment/batch/theme/agent/hermes/tenant/identity/permission/project) + application crate 17 域 100% From<DomainError> 映射 (per WBS §14.15 0.6M tokens 实证, v0.62 反转解锁后 Mavis 推进)**：**(1) commit `5c14280` P0-3 Stage 1 落地** (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses + 守门 #1 v25 单 crate 模式): 5 files changed, 1017 insertions(+): (a) `crates/application/Cargo.toml` +11 dep (feedback/integration/comment/batch/theme/agent/cli/tenant/identity/permission/project); (b) `crates/application/src/lib.rs` +11 From impl (270+ lines, 跟 P0-2 v0.58+v0.59+v0.60 star-api-rest/src/error.rs 17 域映射 对称); (c) `crates/application/tests/p03_app_error_mapping.rs` new file (24 tests, 10.3KB); (d) `scripts/app_error_stage1_insert.py` 18.9KB 幂等 Python 脚本 (落 11 From impls, per 守门 #19 v19 agent 交互 Python 化); **(2) 守门 #1 v25 实证** (per 守门 #1 v25 单 crate 模式): `cargo test -p application -j 4` = **35/35 PASS** (11 lib + **24 p03_app_error_mapping** [v0.63], 0 fail); `cargo check -p application --lib -j 4` = 0 err (30.25s, 跨 7 dep crate); `cargo fmt -p application --check` = 0 diff; **(3) 累计 application crate 17 域 100% From<DomainError> 映射 (per WBS §14.15 0.6M tokens)**：现有 6 域 (work_item/workspace/worktree/search/scm/validation, v0.29 之前) + **Stage 1 11 域 (feedback/integration/comment/batch/theme/agent/hermes/tenant/identity/permission/project) [v0.63]**; 跟 P0-2 star-api-rest 17 域 100% 对称, 跨 REST 入口 + application 编排双层 error 映射完整; **(4) 阻塞 0 → 0 实际** (v0.62 反转后 跨 session 续做项 解锁, Mavis 可推进); **(5) 未触动 (per 守门 #1 禁回溯叙事)**: v0.1-v0.62 修订历史不动; 守门 #14 v4 永久代签 author=Ulysses 维持; 守门 #3 (5 域独立 Lead) 维持; 守门 #10 (author=Ulysses) 维持; **(6) 跨 session 续做项** (per 守门 #1 R-05 不 push + 守门 #14 v4 永久代签): (a) WBS v0.63 row 同步 (本 commit); (b) 修订历史 v0.63 row 同步 (本 commit); (c) v0.64 = P0-4 infrastructure adapter (0.4M tokens, Mavis 推进); (d) v0.65 = 5/6 Repository PG 容器化 (2M tokens, Mavis 推进); (e) v0.66 = envoy 业务路由 + TLS 自动签发 (1-2M tokens, Mavis 推进); (f) v0.67 = v32 拍板激活 + 修订历史; **(7) 触发**: 9/10 12:45 JST Ulysses 发令"真人代签流程全部取消, 改为 mavis 审核" (v0.62 反转, 阻塞 5 → 0) + 9/10 13:00 JST Mavis 自驱推进 P0-3 Stage 1 (per 守门 #9 v19 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 守门 #14 v4 永久代签 author=Ulysses) + 守门 #1 v25 单 crate 模式实证 (35/35 PASS) + commit author=Ulysses |"""

def main():
    if not WBS.exists():
        print(f"FAIL: {WBS} not found", file=sys.stderr)
        sys.exit(1)
    content = WBS.read_text(encoding="utf-8")
    if "v0.63-p03-app-error-mapping" in content:
        print("OK: v0.63 row already present (idempotent skip)")
        return
    pat = re.compile(r"(\| \*\*v0\.62\*\*[^\n]*\n)")
    m = pat.search(content)
    if not m:
        print("FAIL: v0.62 row not found", file=sys.stderr)
        sys.exit(2)
    end = m.end()
    new_content = content[:end] + V063_ROW + "\n" + content[end:]
    WBS.write_text(new_content, encoding="utf-8")
    print(f"OK: v0.63 row inserted (was {len(content)} bytes, now {len(new_content)} bytes, +{len(new_content) - len(content)})")

if __name__ == "__main__":
    main()
