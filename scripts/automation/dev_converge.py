"""Audit and reconcile frozen local branch tips in an isolated integration lane."""

# @cypher schema=1 source_sha256=80519eddf26b4e1f80aa2eed9f4fe1f802b31109c20e5d18d6010d5220d83167
# MERGE (self:File {path:"scripts/automation/dev_converge.py"})
# MERGE (command:Symbol {id:"scripts/automation/dev_converge.py::git",kind:"function"})
# MERGE (plan:Symbol {id:"scripts/automation/dev_converge.py::plan",kind:"function"})
# MERGE (begin:Symbol {id:"scripts/automation/dev_converge.py::begin",kind:"function"})
# MERGE (resolve:Symbol {id:"scripts/automation/dev_converge.py::resolve",kind:"function"})
# MERGE (finish:Symbol {id:"scripts/automation/dev_converge.py::finish",kind:"function"})
# MERGE (main:Symbol {id:"scripts/automation/dev_converge.py::main",kind:"function"})
# MERGE (process:ExternalService {id:"python.subprocess.run"})
# MERGE (self)-[:DEFINES]->(command)
# MERGE (self)-[:DEFINES]->(plan)
# MERGE (self)-[:DEFINES]->(begin)
# MERGE (self)-[:DEFINES]->(resolve)
# MERGE (self)-[:DEFINES]->(finish)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (command)-[:CALLS]->(process)
# MERGE (plan)-[:CALLS]->(command)
# MERGE (begin)-[:CALLS]->(command)
# MERGE (resolve)-[:CALLS]->(command)
# MERGE (finish)-[:CALLS]->(command)
# MERGE (main)-[:CALLS]->(plan)
# MERGE (main)-[:CALLS]->(begin)
# MERGE (main)-[:CALLS]->(resolve)
# MERGE (main)-[:CALLS]->(finish)
# @endcypher

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
AUDIT = ROOT / "docs/reports/dev-converge-20261004"
CACHE = ROOT / ".cache/dev-converge-20261004"
PLAN = AUDIT / "plan.json"
PENDING = CACHE / "pending.json"


def git(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    """Use argument lists and bounded execution; never invoke a shell."""
    result = subprocess.run(
        ["git", "-C", str(ROOT), *args], capture_output=True,
        text=True, encoding="utf-8", errors="replace", timeout=120,
    )
    if check and result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    return result


def save(path: Path, data: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def plan() -> None:
    """Freeze refs and project each source without changing an index or ref."""
    if PLAN.exists():
        raise RuntimeError("Plan already exists; never replace its accepted source tips")
    candidate = git("branch", "--show-current").stdout.strip()
    target = git("rev-parse", "refs/heads/dev").stdout.strip()
    sources = []
    for line in git("for-each-ref", "--format=%(refname:short)|%(objectname)", "refs/heads").stdout.splitlines():
        name, sha = line.split("|")
        if name in ("dev", candidate):
            continue
        contained = git("merge-base", "--is-ancestor", sha, target, check=False).returncode == 0
        source = {"branch": name, "sha": sha, "status": "CONTAINED" if contained else "MERGE"}
        source["unique_count"] = int(git("rev-list", "--count", f"{target}..{sha}").stdout)
        source["commits"] = git("log", "--format=%H %s", f"{target}..{sha}").stdout.splitlines()
        if not contained:
            base = git("merge-base", target, sha).stdout.strip()
            source["merge_base"] = base
            source["source_paths"] = git("diff", "--name-only", base, sha).stdout.splitlines()
            projection = git("merge-tree", "--write-tree", target, sha, check=False)
            if projection.returncode not in (0, 1):
                raise RuntimeError(projection.stderr)
            lines = projection.stdout.splitlines()
            source["predicted_conflict"] = projection.returncode == 1
            source["projected_tree"] = lines[0]
            source["projected_paths"] = git("diff", "--name-only", target, lines[0]).stdout.splitlines()
        sources.append(source)
    sources.sort(key=lambda row: (row["branch"] != "main", -row["unique_count"], row["branch"]))
    save(PLAN, {"target": "dev", "target_start": target, "candidate": candidate, "sources": sources})
    for source in sources:
        print(source["branch"], source["status"], source["unique_count"], len(source.get("projected_paths", [])))


def begin(branch: str) -> None:
    """Start an ordinary merge; conflicts stay in this isolated candidate for review."""
    if PENDING.exists():
        raise RuntimeError("A merge is already pending")
    source = next(row for row in json.loads(PLAN.read_text(encoding="utf-8"))["sources"] if row["branch"] == branch)
    if git("rev-parse", f"refs/heads/{branch}").stdout.strip() != source["sha"]:
        raise RuntimeError("Source tip changed after the recorded plan")
    if git("status", "--porcelain=v1", "--untracked-files=all").stdout.strip():
        raise RuntimeError("Commit the candidate's current changes before starting another source")
    if git("merge-base", "--is-ancestor", source["sha"], "HEAD", check=False).returncode == 0:
        print("Already contained:", branch)
        return
    before = git("rev-parse", "HEAD").stdout.strip()
    result = git("merge", "--no-ff", "--no-commit", source["sha"], check=False)
    if result.returncode not in (0, 1):
        raise RuntimeError(result.stdout + result.stderr)
    merge_head = Path(git("rev-parse", "--git-path", "MERGE_HEAD").stdout.strip())
    if not merge_head.exists():
        raise RuntimeError("Merge did not start: " + result.stdout + result.stderr)
    paths = sorted(set(git("diff", "--name-only", before).stdout.splitlines()))
    conflicts = git("diff", "--name-only", "--diff-filter=U").stdout.splitlines()
    save(PENDING, {"source": source, "before": before, "paths": paths, "conflicts": conflicts})
    print(json.dumps({"source": branch, "paths": paths, "conflicts": conflicts}, ensure_ascii=False, indent=2))


def resolve(manifest: Path) -> None:
    """Apply explicitly audited path choices; every changed path needs evidence."""
    pending = json.loads(PENDING.read_text(encoding="utf-8"))
    decisions = json.loads(manifest.read_text(encoding="utf-8"))
    if set(decisions) != set(pending["paths"]):
        raise RuntimeError("Decisions must cover exactly every projected changed path")
    for path, decision in decisions.items():
        if not decision.get("reason") or not decision.get("evidence"):
            raise RuntimeError("Missing semantic rationale or evidence: " + path)
        if decision["action"] == "current":
            git("restore", f"--source={pending['before']}", "--staged", "--worktree", "--", path)
        elif decision["action"] == "merged":
            if path in pending["conflicts"]:
                raise RuntimeError("Conflict requires a manually inspected edit: " + path)
            git("add", "--", path)
        elif decision["action"] == "manual":
            git("add", "--", path)
        else:
            raise RuntimeError("Unknown decision action: " + path)
        file = ROOT / path
        decision["result_sha256"] = hashlib.sha256(file.read_bytes()).hexdigest() if file.is_file() else None
    audit_path = AUDIT / (pending["source"]["sha"][:12] + ".json")
    save(audit_path, {**pending, "decisions": decisions})
    git("add", "--", audit_path.relative_to(ROOT).as_posix())
    print(audit_path)


def finish() -> None:
    """Commit the reviewed real merge after conflict and whitespace checks."""
    pending = json.loads(PENDING.read_text(encoding="utf-8"))
    if git("diff", "--name-only", "--diff-filter=U").stdout.strip():
        raise RuntimeError("Unresolved merge paths remain")
    audit = AUDIT / (pending["source"]["sha"][:12] + ".json")
    if not audit.exists():
        raise RuntimeError("Per-path review evidence is missing")
    git("diff", "--cached", "--check")
    message = f"Merge branch '{pending['source']['branch']}' into dev candidate\n\nResolve against current Codex Run architecture.\nEvidence: {audit.relative_to(ROOT).as_posix()}\nAutomation: scripts/automation/dev_converge.py"
    git("-c", "user.name=Ulysses", "-c", "user.email=ulysses@mavis.local", "commit", "-m", message)
    PENDING.unlink()
    print(git("rev-parse", "HEAD").stdout.strip())


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("plan")
    start = commands.add_parser("begin")
    start.add_argument("branch")
    resolve_parser = commands.add_parser("resolve")
    resolve_parser.add_argument("manifest", type=Path)
    commands.add_parser("finish")
    args = parser.parse_args()
    if args.command == "plan":
        plan()
    elif args.command == "begin":
        begin(args.branch)
    elif args.command == "resolve":
        resolve(args.manifest)
    else:
        finish()


if __name__ == "__main__":
    main()
