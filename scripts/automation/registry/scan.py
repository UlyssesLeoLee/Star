"""
scan.py — RuntimeRegistry 主入口 (per DD-MULTICA-RUNTIME-001 §3.1)

调用: python scripts/automation/registry/scan.py
"""
import argparse
import json
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

# 允许从 scripts/automation/registry/ 目录运行
sys.path.insert(0, str(Path(__file__).parent))

# Windows GBK 兼容: reconfigure stdout to utf-8 (per 阶段 1 实装 fix)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(encoding="utf-8")

from probe import RuntimeProbe
from min_version import MinVersionGate
from status import StatusClassifier
from reporter import RuntimeStatusReporter


def main():
    parser = argparse.ArgumentParser(description="Multica Runtime Registry scan (v32 阶段 1)")
    parser.add_argument("--json", action="store_true", help="output JSON format")
    parser.add_argument("--log-dir", type=Path, default=Path("docs/reports/runtime-scan"),
                        help="audit log dir (per FR-23)")
    args = parser.parse_args()

    scan_id = str(uuid.uuid4())[:8]
    started_at = datetime.now(timezone.utc)

    probe = RuntimeProbe()
    gate = MinVersionGate()
    classifier = StatusClassifier()
    reporter = RuntimeStatusReporter(log_dir=args.log_dir)

    entries = probe.probe_all()

    # classify + min version check
    results = []
    for e in entries:
        verdict = gate.check(e.provider, e.version or "")
        status = classifier.classify(e, verdict)
        results.append({
            "provider": e.provider,
            "path": str(e.path) if e.path else None,
            "version": e.version,
            "min_version": verdict.minimum,
            "min_version_status": verdict.status,
            "min_version_error": verdict.error,
            "probe_method": e.probe_method,
            "status": status.value,
        })

    finished_at = datetime.now(timezone.utc)
    reporter.write_log(scan_id, started_at, finished_at, entries)

    summary = {
        "scan_id": scan_id,
        "started_at": started_at.isoformat() + "Z",
        "finished_at": finished_at.isoformat() + "Z",
        "results": results,
        "summary": {
            "active": sum(1 for r in results if r["status"] == "🟢"),
            "stale": sum(1 for r in results if r["status"] == "🟡"),
            "poisoned": sum(1 for r in results if r["status"] == "🔴"),
            "missing": sum(1 for r in results if r["status"] == "⚫"),
        },
    }

    if args.json:
        print(json.dumps(summary, ensure_ascii=False, indent=2))
    else:
        print(f"=== Multica Runtime Registry scan {scan_id} ===")
        print(f"started: {started_at.isoformat()}Z")
        print(f"finished: {finished_at.isoformat()}Z")
        print()
        print(f"{'PROVIDER':<12} {'STATUS':<4} {'VERSION':<24} {'MIN':<10} {'PATH':<40}")
        print("-" * 92)
        for r in results:
            path_disp = r["path"] or "-"
            if len(path_disp) > 38:
                path_disp = "..." + path_disp[-35:]
            ver_disp = (r["version"] or "-")[:22]
            min_disp = r["min_version"] or "-"
            print(f"{r['provider']:<12} {r['status']:<4} {ver_disp:<24} {min_disp:<10} {path_disp:<40}")
        print()
        s = summary["summary"]
        print(f"summary: 🟢 active={s['active']}  🟡 stale={s['stale']}  🔴 poisoned={s['poisoned']}  ⚫ missing={s['missing']}")


if __name__ == "__main__":
    main()
