"""pgwiki_issues_sync.py — pgwiki audit issue 同步桥 (v2, A3 状态机)

per Ulysses 2026-09-06 13:41 JST 拍板 A3:
A1 计数 + A2 commit 引用,任一触发都 close,reopen 只看 A1 计数。

设计:
- 幂等键: issue title (pgwiki audit: <stem> (<category>))
- 状态机:
  - open → close: counters 全 0 **OR** git log 找到 `Closes #N` / `Fixes #N` 引用
  - closed → reopen: 仅当 counters > 0(A1 计数回升,避免 commit 误 reopen)
  - open → open(无变化): append UPDATE 段
  - closed → closed(无变化): 静默
- 状态变更必写 comment + 时间 + 触发原因
- 守门 #1a: 401 重试 2 次

依赖: gh CLI 已登录 (scope: repo)
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

# counters 中判定"已修"的字段(值 = 0 表示全修完)
FIXED_FIELDS = (
    "orphan_schemas",
    "placeholder_schemas",
    "empty_crates",
    "broker_adr",
    "broker_arch",
    "table_module_broken",
    "fake_deps",
)


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
    text += "\n> **状态机** (per 2026-09-06 13:41 JST 拍板 A3):\n"
    text += "> - 关闭触发: counters 全 0 **或** git log 含 `Closes #N` / `Fixes #N`\n"
    text += "> - 重开触发: 仅 counters > 0\n"
    return text


def parse_counters(file_path: Path) -> dict:
    """从 issue 报告末尾的 `## Counters` 段解析 json。空则 {}"""
    text = file_path.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"##\s+Counters\s*\n+```json\s*\n(.+?)\n```", text, re.DOTALL)
    if not m:
        return {}
    try:
        return json.loads(m.group(1))
    except Exception:
        return {}


def is_counters_zero(counters: dict) -> bool:
    """判定 counters 是否全 0(= 已修完)"""
    if not counters:
        return False
    # 7 号 docswiki_contrast 是对照表,不算"问题数",不算 close
    if "docswiki_contrast" in counters and len(counters) == 1:
        return False
    for field in FIXED_FIELDS:
        if counters.get(field, 0) > 0:
            return False
    return True


def issue_title(stem: str, category: str) -> str:
    return f"pgwiki audit: {stem} ({category})"


def gh_api(method: str, endpoint: str, payload: dict = None, max_retries: int = 2) -> tuple:
    """走 gh api。401 走守门 #1a 重试。返回 (exit, stdout, stderr)"""
    cmd = ["gh", "api", "--method", method, f"/repos/{REPO}{endpoint}"]
    if payload is not None:
        cmd.extend(["--input", "-"])
    last_proc = None
    for attempt in range(max_retries + 1):
        proc = subprocess.run(
            cmd,
            input=json.dumps(payload, ensure_ascii=False) if payload is not None else None,
            capture_output=True, text=True, encoding="utf-8",
        )
        last_proc = proc
        if proc.returncode == 0:
            return proc.returncode, proc.stdout, proc.stderr
        if "401" in proc.stderr or "Authentication" in proc.stderr:
            if attempt < max_retries:
                wait = 2 ** attempt
                print(f"  [401 retry {attempt + 1}/{max_retries}] wait {wait}s ...")
                time.sleep(wait)
                continue
        return proc.returncode, proc.stdout, proc.stderr
    return last_proc.returncode, last_proc.stdout, last_proc.stderr


def list_existing_issues() -> dict:
    """list repo 所有 open+closed issue with label 'pgwiki'。返回 {title: {number, state, body}}"""
    title_map = {}
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
    now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    merged = old_body.rstrip() + "\n\n---\n\n## 🔄 UPDATE " + now + "\n\n" + new_body
    payload = {"body": merged}
    rc, out, err = gh_api("PATCH", f"/issues/{number}", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def close_issue(number: int) -> tuple:
    payload = {"state": "closed", "state_reason": "completed"}
    rc, out, err = gh_api("PATCH", f"/issues/{number}", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def reopen_issue(number: int) -> tuple:
    payload = {"state": "open"}
    rc, out, err = gh_api("PATCH", f"/issues/{number}", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def add_comment(number: int, body: str) -> tuple:
    payload = {"body": body}
    rc, out, err = gh_api("POST", f"/issues/{number}/comments", payload)
    if rc != 0:
        return False, err[:200]
    obj = json.loads(out)
    return True, obj.get("html_url")


def git_log_closes_issue(number: int) -> str:
    """扫 git log 找 commit 含 Closes #N / Fixes #N。返回 commit hash + msg 片段,没找到空字符串。"""
    # 用 errors="replace" 兜底(Windows console 默认 GBK,git log 输出可能含 UTF-8 多字节)
    proc = subprocess.run(
        ["git", "log", "--all", "--pretty=format:%H %s", "-i", "--grep", r"(Closes|Fixes|Resolves) #" + str(number) + r"\b"],
        capture_output=True, text=True, encoding="utf-8", errors="replace", cwd=r"D:/Star",
    )
    line = proc.stdout.strip().splitlines()[0] if proc.stdout.strip() else ""
    return line


def main():
    print("[sync] 拉取现有 pgwiki audit issues ...")
    existing = list_existing_issues()
    print(f"[sync] 已存在 {len(existing)} 个")
    now_str = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    created = []
    updated = []
    closed = []
    reopened = []
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
        counters = parse_counters(fpath)
        counters_zero = is_counters_zero(counters)
        print(f"\n--- {title} ---")
        print(f"  counters: {counters}, zero={counters_zero}")
        # ---------- 分支: 新建 ----------
        if title not in existing:
            print(f"  [create]")
            ok, info = create_issue(title, body, labels)
            if ok:
                created.append({"file": fname, "title": title, "url": info})
                print(f"    -> {info}")
            else:
                failed.append({"file": fname, "title": title, "error": info})
                print(f"    [FAIL] {info}")
            continue
        # ---------- 已存在: A3 状态机 ----------
        iss = existing[title]
        n = iss["number"]
        st = iss["state"]
        # 检测 commit 引用
        commit_ref = git_log_closes_issue(n)
        has_commit_close = bool(commit_ref)
        print(f"  existing: #{n} state={st}, commit_close={has_commit_close}")
        # 决策:
        # 1) counters_zero OR has_commit_close → close(open → close)
        # 2) state=closed AND counters>0 → reopen(A1 计数回升,不复用 close)
        # 3) 否则 → open 状态写 UPDATE 段
        if st == "open":
            if counters_zero or has_commit_close:
                # 关闭
                reason = []
                if counters_zero:
                    reason.append(f"counters 全 0: {counters}")
                if has_commit_close:
                    reason.append(f"commit 引用: `{commit_ref}`")
                reason_str = " + ".join(reason)
                print(f"  [close] {reason_str}")
                ok, info = close_issue(n)
                if ok:
                    # 写 trace comment
                    add_comment(n, f"✅ **自动关闭** (per A3 状态机, {now_str})\n\n触发: {reason_str}\n\n状态由 open → closed,rerun `python scripts/automation/pgwiki_issues_sync.py` 仍会保持 closed(除非 counters 回升)。")
                    closed.append({"file": fname, "title": title, "url": info, "number": n})
                    print(f"    -> {info}")
                else:
                    failed.append({"file": fname, "title": title, "error": info})
                    print(f"    [FAIL] {info}")
            else:
                # 保持 open,append UPDATE
                print(f"  [update body]")
                ok, info = update_issue_body(n, iss["body"], body)
                if ok:
                    updated.append({"file": fname, "title": title, "url": info, "number": n})
                    print(f"    -> {info}")
                else:
                    failed.append({"file": fname, "title": title, "error": info})
                    print(f"    [FAIL] {info}")
        elif st == "closed":
            # 只看 A1: counters 回升 → reopen
            if not counters_zero:
                print(f"  [reopen] counters 回升: {counters}")
                ok, info = reopen_issue(n)
                if ok:
                    add_comment(n, f"♻️ **自动重开** (per A3 状态机, {now_str})\n\n触发: counters > 0 = {counters}\n\n状态由 closed → open,问题重现。")
                    reopened.append({"file": fname, "title": title, "url": info, "number": n})
                    print(f"    -> {info}")
                else:
                    failed.append({"file": fname, "title": title, "error": info})
                    print(f"    [FAIL] {info}")
            else:
                # 仍 closed,静默(不浪费 API 调用)
                print(f"  [skip] closed 且 counters 仍 0")
    # ---- 汇总 ----
    print("\n" + "=" * 60)
    print("=== Sync 汇总 (A3 状态机) ===")
    print(f"  Created: {len(created)}")
    for c in created:
        print(f"    [{c['file']}] {c['title']}\n      {c['url']}")
    print(f"  Updated: {len(updated)}")
    for u in updated:
        print(f"    [{u['file']}] #{u['number']} {u['title']}\n      {u['url']}")
    print(f"  Closed: {len(closed)}")
    for c in closed:
        print(f"    [{c['file']}] #{c['number']} {c['title']}\n      {c['url']}")
    print(f"  Reopened: {len(reopened)}")
    for r in reopened:
        print(f"    [{r['file']}] #{r['number']} {r['title']}\n      {r['url']}")
    if failed:
        print(f"  Failed: {len(failed)}")
        for f in failed:
            print(f"    [{f['file']}] {f['title']}\n      {f['error']}")
        sys.exit(1)
    print(f"\n[OK] 同步完成: {len(created)} created, {len(updated)} updated, {len(closed)} closed, {len(reopened)} reopened, 0 failed")
    sys.exit(0)


if __name__ == "__main__":
    main()
