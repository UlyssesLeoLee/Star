"""RF-001 T1.5 rebase conflict resolution — option A (per sub-agent analysis).

For each of 7 crates/star-mcp/src/tools/*.rs files:
1. Take HEAD's content (post-18 T1.5 commits) — done via `git checkout --ours`
2. Remove the `#![warn(missing_docs)]` line + following blank line
3. `git add` each file

Per 守门 #19 自动化 + 守门 #12 0 误删无关文件.
"""
import subprocess
from pathlib import Path

WORKTREE = Path(r"D:\Star\.worktrees\wt-t15-missing-docs")
FILES = [
    "create_worktree.rs",
    "find_references.rs",
    "get_code_context.rs",
    "get_context.rs",
    "get_symbol.rs",
    "search_code.rs",
    "search_issues.rs",
]
AUTHOR = ["-c", "user.name=Ulysses", "-c", "user.email=ulysses@mavis.local"]


def run(cmd, check=True):
    """Run a git command, raise on non-zero exit."""
    result = subprocess.run(cmd, capture_output=True, text=True, cwd=str(WORKTREE))
    if check and result.returncode != 0:
        raise RuntimeError(f"cmd {cmd} failed: {result.stderr}")
    return result


def resolve_one(filename: str) -> None:
    """Resolve one file: git checkout --ours then remove first 2 lines."""
    rel_path = f"crates/star-mcp/src/tools/{filename}"
    full_path = WORKTREE / rel_path

    # Step 1: take HEAD's content (removes conflict markers)
    run(["git", *AUTHOR, "checkout", "--ours", "--", rel_path])

    # Step 2: read content
    original = full_path.read_text(encoding="utf-8")
    lines = original.splitlines(keepends=True)

    # Step 3: remove lines 1-2 (#![warn(missing_docs)] + blank)
    # Verify line 1 is the warn line before removing (defensive)
    if not lines[0].strip().startswith("#![warn(missing_docs)]"):
        raise RuntimeError(
            f"{filename}: line 1 is not '#![warn(missing_docs)]' — got {lines[0]!r}"
        )
    if lines[1].strip() != "":
        raise RuntimeError(
            f"{filename}: line 2 is not blank — got {lines[1]!r}"
        )

    new_content = "".join(lines[2:])

    # Step 4: write back
    full_path.write_text(new_content, encoding="utf-8")

    # Step 5: git add
    run(["git", *AUTHOR, "add", "--", rel_path])

    print(f"[OK] {filename}: removed 2 lines (warn + blank), kept {len(lines) - 2} lines")


def main():
    print("=== RF-001 T1.5 rebase option A resolution ===")
    print(f"worktree: {WORKTREE}")
    print(f"files: {len(FILES)}")
    print()
    for f in FILES:
        resolve_one(f)
    print()
    print("=== status check ===")
    result = run(["git", *AUTHOR, "status", "--short"])
    print(result.stdout or "(clean — no unmerged paths)")


if __name__ == "__main__":
    main()
