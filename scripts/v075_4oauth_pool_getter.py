#!/usr/bin/env python3
"""v0.75 batch insert pub fn pool() getter to 4 oauth_* Repository.

Per v0.74 已知缺口 (a): 4 个 oauth_* Repository (access_tokens / authorization_codes /
clients / refresh_tokens) 跟 ops 同样需求, 批量加 `pub fn pool(&self) -> &PgPool` 跟
v0.73 ops_metrics_config.rs / v0.74 5 ops Repository 一致.

Pattern identical to v0.74 script, only TARGETS list changes.
"""
import re
import sys
from pathlib import Path

STAR_ROOT = Path("D:/Star/.worktrees/wt-v075-4oauth-pool")
REPO_DIR = STAR_ROOT / "crates" / "star-pg-adapter" / "src" / "repository"

TARGETS = [
    "oauth_access_tokens.rs",
    "oauth_authorization_codes.rs",
    "oauth_clients.rs",
    "oauth_refresh_tokens.rs",
]

GETTER_BLOCK = '''\n    /// 拿共享 PgPool 引用 (v0.75 P0-4 Stage 2.1.2 扩展: 4 oauth_* Repository wire-up)\n    ///\n    /// 用于 application crate 跨域编排时多个 Repository 共享同一 pool\n    /// (per sqlx::PgPool = Arc 内部, clone 廉价).\n    pub fn pool(&self) -> &PgPool {\n        &self.pool\n    }\n'''


def insert_getter(path: Path) -> tuple[str, int]:
    text = path.read_text(encoding="utf-8")
    if "pub fn pool(&self) -> &PgPool" in text:
        return ("skip", 0)
    pattern = re.compile(
        r"(    pub fn new\(pool: PgPool\) -> Self \{\n        Self \{ pool \}\n    \})",
        re.MULTILINE,
    )
    new_text, count = pattern.subn(r"\1" + GETTER_BLOCK, text, count=1)
    if count == 0:
        return ("fail", 0)
    path.write_text(new_text, encoding="utf-8")
    return ("ok", len(new_text) - len(text))


def main() -> int:
    results = []
    for name in TARGETS:
        p = REPO_DIR / name
        if not p.exists():
            results.append((name, "missing", 0))
            continue
        status, delta = insert_getter(p)
        results.append((name, status, delta))
    for name, status, delta in results:
        print(f"  {status:6s}  {name:36s}  +{delta} bytes")
    failed = [r for r in results if r[1] not in ("ok", "skip")]
    if failed:
        print(f"[fail] {len(failed)} files failed", file=sys.stderr)
        return 1
    print(f"[ok] {sum(1 for r in results if r[1] == 'ok')} files updated, "
          f"{sum(1 for r in results if r[1] == 'skip')} skipped")
    return 0


if __name__ == "__main__":
    sys.exit(main())
