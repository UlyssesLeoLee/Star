"""pgwiki_issues_sync.py — pgwiki audit issue 同步桥 (per 2026-09-06 13:20 JST 拍板 A)

设计:
- 用 issue title 做幂等键(per 50-issues/<filename>.md → 固定 title)
- 流程: 列表 GitHub issues → 按 title 匹配 → 有则 PATCH body 追加 UPDATE 段,无则 POST 新建
- 失败重试: 401 走守门 #1a 重试细则 (max 2 retries,非 401 直接 fail)
- 输出 diff summary: "X created, Y updated, Z failed"
- 同步结束返回 0(全成功)/1(部分失败)

依赖: gh CLI 已登录(scope: repo)
"""
import json
import re
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

PGWIKI_ISSUES = Path(r"D:/Star/docs/wiki/pgwiki/50-issues")
REPO = "UlyssesLeoLee/Star"
LABEL_OWNER = "pgwiki"
LABEL_AUDIT = "audit"

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


def issue_title(stem: str, category: str) -> str:
    return f"pgwiki audit: {stem} ({category})"


def gh_api(method: str, endpoint: str, payload: dict = None, max_retries: int = 2) -> tuple:
    """走 gh api,返回 (exit_code, stdout, stderr)。401 走 max_retries 重试(per 守门 #1a)。"""
    cmd = ["gh", "api", "--method", method, f"/repos/{REPO}{endpoint}"]
    if payload is not None:
        cmd.extend(["--input", "-"])
    for attempt in range(max_retries + 1):
        proc = subprocess.run(
            cmd,
            input=json.dumps(payload, ensure_ascii=False) if payload is not None else None,
            capture_output=True, text=True, encoding="utf-8",
        )
        if proc.returncode == 0:
            return proc.returncode, proc.stdout, proc.stderr
        # 401 重试细则(per 守门 #1a:401 不算 timeout,跨 session 续,这次只重试有限次数)
        if "401" in proc.stderr or "Authentication" in proc.stderr:
            if attempt < max_retries:
                wait = 2 ** attempt
                print(f"  [401 retry {attempt + 1}/{max_retries}] wait {wait}s ...")
                time.sleep(wait)
                continue
        return proc.returncode, proc.stdout, proc.stderr
    return proc.returncode, proc.stdout, proc.stderr


def list_existing_issues() -> dict:
    """list repo 所有 open+closed issue with label 'pgwiki',返回 {title: {number, state, body}}"""
    title_map = {}
    # paginate,每页 100
    for page in range(1, 5):
        rc, out, err = gh_api("GET", f"/issues?state=all&labels={LABEL_OWNER},{LABEL_AUDIT}&per_page=100&page={page}")
        if rc != 0:
            print(f"[ERR] list issues page {page}: {err[:200]}")
            break
        issues = json.loads(out) if out.strip() else []
        if not issues:
            break
        for iss in issues:
            t = iss.get("title", "")
            title_map[t] = {
                "number": iss.get("number"),
                "state": iss.get("state"),
                "body": iss.get("body", ""),
            }
        if len(issues) < 100:
            break
    return title_map


def create_issue(title: str, body: str, labels: list) -> tuple:
    payload = {
        "title": title,
        "body": body,
        "labels": labels,
        "assignees": ["UlyssesLeoLee"],
    }
    rc, out, err = gh_api("POST", "/issues", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def update_issue_body(number: int, old_body: str, new_body: str) -> tuple:
    """edit issue body,追加 UPDATE 段(不重写全部,保留历史可读)"""
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    # 拼: 旧 body + 分隔 + 新 body + UPDATE stamp
    merged = old_body.rstrip() + "\n\n---\n\n## 🔄 UPDATE " + now + "\n\n" + new_body
    payload = {"body": merged}
    rc, out, err = gh_api("PATCH", f"/issues/{number}", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def main():
    # 1. 列出现有 issues
    print("[sync] 拉取现有 pgwiki audit issues ...")
    existing = list_existing_issues()
    print(f"[sync] 已存在 {len(existing)} 个")
    # 2. 逐个同步
    created = []
    updated = []
    failed = []
    for fname, category in ISSUES:
        fpath = PGWIKI_ISSUES / fname
        if not fpath.exists():
            print(f"[skip] {fname} 不存在")
            continue
        stem = fpath.stem
        title = issue_title(stem, category)
        body = md_to_issue_body(fpath)
        labels = [LABEL_OWNER, LABEL_AUDIT, category]
        if title in existing:
            # 更新
            iss = existing[title]
            print(f"[update #{iss['number']}] {title} (state={iss['state']})")
            ok, info = update_issue_body(iss["number"], iss["body"], body)
            if ok:
                updated.append({"file": fname, "title": title, "url": info, "number": iss["number"]})
                print(f"  -> {info}")
            else:
                failed.append({"file": fname, "title": title, "error": info})
                print(f"  [FAIL] {info}")
        else:
            # 创建
            print(f"[create] {title}")
            ok, info = create_issue(title, body, labels)
            if ok:
                created.append({"file": fname, "title": title, "url": info})
                print(f"  -> {info}")
            else:
                failed.append({"file": fname, "title": title, "error": info})
                print(f"  [FAIL] {info}")
    # 3. 汇总
    print("\n" + "=" * 60)
    print("=== Sync 汇总 ===")
    print(f"  Created: {len(created)}")
    for c in created:
        print(f"    [{c['file']}] {c['title']}\n      {c['url']}")
    print(f"  Updated: {len(updated)}")
    for u in updated:
        print(f"    [{u['file']}] #{u['number']} {u['title']}\n      {u['url']}")
    if failed:
        print(f"  Failed: {len(failed)}")
        for f in failed:
            print(f"    [{f['file']}] {f['title']}\n      {f['error']}")
        sys.exit(1)
    print(f"\n[OK] 同步完成: {len(created)} created, {len(updated)} updated, 0 failed")
    sys.exit(0)


if __name__ == "__main__":
    main()
