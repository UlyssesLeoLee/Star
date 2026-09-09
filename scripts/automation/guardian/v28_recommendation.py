"""v28_recommendation: ask_user 推荐项格式校验 + 自动 retry.

拍板激活后, Mavis 终端触发 `ask_user` 调用**必含**:
- 2-4 选项
- 至少 1 个标 "(推荐)" / "（推荐）"
- 推荐项放第一个
- description 写"做这件事的具体后果"

不满足 -> 自动 retry 1 次加推荐项; 不适用 (微决策, Mavis 自驱) = commit 措辞 / 具体改法 / 报告 v3.x 升版 / WipeCluster 跑 / 守门 v3x 候选落地 / 30min 探活.

status: 待 Ulysses 拍板激活.
"""
# SPDX-License-Identifier: MIT OR Apache-2.0

import argparse
import re
import sys


RECOMMENDED_TAG = re.compile(r"\(推荐\)|（推荐\)|\(recommended\)|（recommended\)", re.IGNORECASE)


def _score(options: list[dict]) -> tuple[int, list[str]]:
    """Score an ask_user call. Returns (score, issues). score in {0,1,2}."""
    issues: list[str] = []
    n = len(options)
    if n < 2 or n > 4:
        issues.append(f"option count {n} not in [2,4]")
    if not any(RECOMMENDED_TAG.search(o.get("label", "")) for o in options):
        issues.append("missing (推荐) tag on at least one option")
    if options and not RECOMMENDED_TAG.search(options[0].get("label", "")):
        issues.append("first option is not the recommended one")
    if not all(o.get("description", "").strip() for o in options):
        issues.append("at least one option has empty description")
    return (2 if not issues else 0), issues


def validate(options: list[dict]) -> int:
    score, issues = _score(options)
    if score == 2:
        print("OK: ask_user payload complies with v28 (2-4 options, 推荐 first, descriptions present)")
        return 0
    print("FAIL: v28 violations:")
    for i in issues:
        print(f"  - {i}")
    return 1


def lint() -> int:
    """Lint a JSON file from stdin (one ask_user payload)."""
    import json
    try:
        payload = json.loads(sys.stdin.read() or "{}")
    except json.JSONDecodeError as e:
        print(f"FAIL: invalid JSON: {e}")
        return 1
    options = payload.get("options") or payload.get("choices") or []
    return validate(options)


def main() -> int:
    parser = argparse.ArgumentParser(description="v28 ask_user recommendation validator")
    parser.add_argument("--lint", action="store_true", help="read JSON from stdin and validate")
    args = parser.parse_args()
    if args.lint:
        return lint()
    parser.print_help()
    return 0


if __name__ == "__main__":
    sys.exit(main())
