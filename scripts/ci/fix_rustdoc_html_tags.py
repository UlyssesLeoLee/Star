# @cypher schema=1 source_sha256=75555c0c404859f9f7ca7d5b7e47744c833d35559b2399922694817b4ca4d5e4
# MERGE (self:File {path:"scripts/ci/fix_rustdoc_html_tags.py"})
# MERGE (fix_line:Symbol {id:"scripts/ci/fix_rustdoc_html_tags.py::fix_line",kind:"function"})
# MERGE (parse_spec:Symbol {id:"scripts/ci/fix_rustdoc_html_tags.py::parse_spec",kind:"function"})
# MERGE (main:Symbol {id:"scripts/ci/fix_rustdoc_html_tags.py::main",kind:"function"})
# MERGE (self)-[:DEFINES]->(fix_line)
# MERGE (self)-[:DEFINES]->(parse_spec)
# MERGE (self)-[:DEFINES]->(main)
# @endcypher
#!/usr/bin/env python3
"""Fix rustdoc "unclosed HTML tag" warnings by wrapping bare generics in backticks.

rustdoc parses `<Foo>` in doc comments as an HTML tag, so generic type names
like `Vec<Comment>` must be written as ``Vec<Comment>`` inside a code span.

This script rewrites only the specific token rustdoc flagged, on the specific
line rustdoc flagged, and leaves the surrounding prose untouched. It reports
every edit so the change can be reviewed in the diff.

Usage:
    python3 scripts/ci/fix_rustdoc_html_tags.py --apply <file:line:tag> ...
    python3 scripts/ci/fix_rustdoc_html_tags.py --check <file:line:tag> ...
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path


def parse_spec(spec: str) -> tuple[str, int, str]:
    """Parse ``path:line:tag`` into (path, line_number, tag)."""
    path, line, tag = spec.rsplit(":", 2)
    return path, int(line), tag


def fix_line(line: str, tag: str) -> tuple[str, bool]:
    """Wrap the first bare ``<tag>`` occurrence in backticks.

    A tag that is already inside a code span is left alone — rustdoc does not
    complain about those, so re-wrapping would be a no-op at best.
    """
    needle = f"<{tag}>"
    idx = line.find(needle)
    if idx < 0:
        return line, False

    # Already inside a code span? Count backticks before the tag.
    prefix = line[:idx]
    if prefix.count("`") % 2 == 1:
        return line, False

    return line[:idx] + f"`{needle}`" + line[idx + len(needle):], True


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--apply", action="store_true", help="rewrite the files")
    parser.add_argument("--check", action="store_true", help="only report")
    parser.add_argument("specs", nargs="+", help="file:line:tag entries")
    args = parser.parse_args(argv)

    if not (args.apply or args.check):
        parser.error("choose --apply or --check")

    # Group by file so each file is read and written once.
    by_file: dict[str, list[tuple[int, str]]] = {}
    for spec in args.specs:
        path, line, tag = parse_spec(spec)
        by_file.setdefault(path, []).append((line, tag))

    changed_files = 0
    total_edits = 0
    for rel, entries in sorted(by_file.items()):
        path = Path(rel)
        if not path.is_file():
            print(f"skip (missing): {rel}", file=sys.stderr)
            continue
        lines = path.read_text(encoding="utf-8").split("\n")
        edits = 0
        for line_no, tag in entries:
            index = line_no - 1
            if not (0 <= index < len(lines)):
                print(f"skip (out of range): {rel}:{line_no}", file=sys.stderr)
                continue
            new_line, changed = fix_line(lines[index], tag)
            if changed:
                lines[index] = new_line
                edits += 1
                print(f"  {rel}:{line_no}  <{tag}> -> `<{tag}>`")
            else:
                print(f"  {rel}:{line_no}  <{tag}> already in a code span (no change)")
        if edits:
            changed_files += 1
            total_edits += edits
            if args.apply:
                path.write_text("\n".join(lines), encoding="utf-8")

    print(f"{total_edits} edit(s) in {changed_files} file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
