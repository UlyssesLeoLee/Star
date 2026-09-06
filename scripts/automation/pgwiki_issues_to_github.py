"""pgwiki_issues_to_github.py — 把 50-issues/*.md 转成 GitHub issues 批量创建。

per Ulysses 2026-09-06 12:27 JST 拍板。

设计:
- 每个 issue 的 title = `pgwiki audit: <issue name> (<category>)`
- body = 50-issues/<file>.md 的 H1 之后内容(去掉 frontmatter)
- labels = ['pgwiki', 'audit', '<category>']
- 走 gh api POST /repos/.../issues(gh CLI 2.89 没 --batch)
- 创建完后输出 issue URL 列表到 stdout

依赖: gh CLI 已登录(scope: repo)
"""
import json
import re
import subprocess
import sys
from pathlib import Path

PGWIKI_ISSUES = Path(r"D:/Star/docs/wiki/pgwiki/50-issues")
REPO = "UlyssesLeoLee/Star"

ISSUES = [
    ("00-orphan-schemas.md", "db-mapping"),
    ("01-placeholder-schemas.md", "db-mapping"),
    ("03-broker-adr-refs.md", "adr-broker"),
    ("04-broker-arch-refs.md", "arch-broker"),
    ("07-docswiki-vs-pgwiki.md", "docswiki-contrast"),
]


def md_to_issue_body(file_path: Path) -> str:
    text = file_path.read_text(encoding="utf-8", errors="replace")
    if text.startswith("---"):
        end = text.find("---", 3)
        if end > 0:
            text = text[end + 3:].lstrip("\n")
    text = re.sub(r"^#\s+.+?\n", "", text, count=1).lstrip()
    text += "\n\n---\n\n"
    text += "**生成器**: `scripts/automation/pgwiki_audit.py`\n"
    text += f"**源文档**: `D:/Star/docs/wiki/pgwiki/50-issues/{file_path.name}`\n"
    text += "**触发**: re-run `python scripts/automation/pgwiki_audit.py` 自动重生成\n\n"
    text += "**对照目的** (per 2026-09-06 11:47 JST Ulysses 拍板):\n"
    text += "- pgwiki = 程序实际拓扑(事实)\n"
    text += "- docswiki = DDD 视角叙事\n"
    text += "- 本 issue = 两者对照暴露的\"模块间问题\"\n"
    return text


def create_one_issue(title: str, body: str, labels: list) -> str:
    """走 gh api POST /repos/.../issues,返回 issue URL"""
    payload = {
        "title": title,
        "body": body,
        "labels": labels,
        "assignees": ["UlyssesLeoLee"],
    }
    proc = subprocess.run(
        [
            "gh", "api",
            "--method", "POST",
            f"/repos/{REPO}/issues",
            "--input", "-",  # stdin
        ],
        input=json.dumps(payload, ensure_ascii=False),
        capture_output=True, text=True, encoding="utf-8",
    )
    if proc.returncode != 0:
        print(f"[ERR] {title}\nstderr: {proc.stderr[:500]}")
        return None
    try:
        obj = json.loads(proc.stdout)
        return obj.get("html_url", "?")
    except Exception as e:
        print(f"[parse ERR] {title}: {e}")
        print(proc.stdout[:500])
        return None


def main():
    created = []
    for fname, category in ISSUES:
        fpath = PGWIKI_ISSUES / fname
        if not fpath.exists():
            print(f"[skip] {fname} 不存在")
            continue
        title = f"pgwiki audit: {fpath.stem} ({category})"
        body = md_to_issue_body(fpath)
        labels = ["pgwiki", "audit", category]
        print(f"[create] {title}")
        url = create_one_issue(title, body, labels)
        if url:
            created.append({"file": fname, "title": title, "url": url})
            print(f"  -> {url}")
    # 汇总
    print("\n=== 汇总 ===")
    for c in created:
        print(f"  [{c['file']}] {c['title']}\n    {c['url']}")
    print(f"\n[OK] {len(created)} 个 issue 创建成功")
    if len(created) != len(ISSUES):
        print(f"[WARN] 期望 {len(ISSUES)} 实际 {len(created)}")


if __name__ == "__main__":
    main()
