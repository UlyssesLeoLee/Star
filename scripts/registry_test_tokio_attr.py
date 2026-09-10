"""v0.68 self-review fix Finding #3: registry 测试用 #[tokio::test] attribute 替代 Runtime::new().unwrap().block_on()

Per 守门 #19 v19 agent 交互 Python 化 + 守门 #1 禁回溯叙事 (不动 functional code).
幂等: 已修 跳过.

策略: 用 Python 脚本批量替换 9 callsites, 加 #[tokio::test] attribute 在 fn 上面.
"""
import re
import sys
from pathlib import Path

REG = Path("crates/infrastructure/src/registry.rs")

def main():
    if not REG.exists():
        print(f"FAIL: {REG} not found", file=sys.stderr)
        sys.exit(1)
    content = REG.read_text(encoding="utf-8")
    if "#[tokio::test]" in content:
        print("OK: 已有 #[tokio::test] (idempotent skip)")
        return
    # Find test mod
    m = re.search(r"^#\[cfg\(test\)\]\nmod tests \{", content, re.MULTILINE)
    if not m:
        print("FAIL: tests mod not found", file=sys.stderr)
        sys.exit(2)
    start = m.end()
    # Find end of tests mod (next } at column 0)
    end = content.find("\n}\n", start)
    if end < 0:
        end = len(content)
    block = content[start:end]
    new_block = block
    # Replace `tokio::runtime::Runtime::new().unwrap().block_on(...)` with `...` (no wrapping)
    new_block = re.sub(
        r"tokio::runtime::Runtime::new\(\)\s*\.unwrap\(\)\s*\.block_on\(",
        "",
        new_block,
    )
    # Add `#[tokio::test]` attribute to each `#[test]` fn that uses block_on
    # Simpler: just change every `#[test]` after a block_on to `#[tokio::test]`
    # But we removed block_on, so we need to detect
    # Strategy: each #[test] fn that previously called .block_on is now a regular sync fn
    # So we should NOT change to #[tokio::test] but keep #[test] (the methods now return immediately)
    # Actually wait, the methods are async (return Future). After removing block_on, they return Future
    # So #[test] would fail (Future is not ()). We need #[tokio::test]
    # Replace ALL `    #\[test\]\n` with `    #[tokio::test]\n` in the test mod
    new_block = re.sub(
        r"    #\[test\]\n",
        "    #[tokio::test]\n",
        new_block,
    )
    if new_block == block:
        print("WARN: no changes applied (pattern mismatch?)")
        return
    new_content = content[:start] + new_block + content[end:]
    REG.write_text(new_content, encoding="utf-8")
    print(f"OK: registry tests refactored to #[tokio::test] (was {len(content)} bytes, now {len(new_content)} bytes)")


if __name__ == "__main__":
    main()
