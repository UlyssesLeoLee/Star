"""Run focused, reproducible gates for the 9F4C-D Schedule owner-message contract."""

# @cypher schema=1 source_sha256=a2f6ef16e367deb4e664d9a5ab3ed8b0037c9fddfd4bbae767cafda25665303d
# MERGE (self:File {path:"scripts/automation/phase9f4c_d_owner_protocol.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4c_d_owner_protocol.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f4c_d_owner_protocol.py::run_step",kind:"function"})
# MERGE (write_text:Symbol {id:"scripts/automation/phase9f4c_d_owner_protocol.py::write_text",kind:"function"})
# MERGE (root:Variable {id:"scripts/automation/phase9f4c_d_owner_protocol.py::ROOT",kind:"constant"})
# MERGE (output_root:Variable {id:"scripts/automation/phase9f4c_d_owner_protocol.py::OUTPUT_ROOT",kind:"constant"})
# MERGE (display_path:Symbol {id:"scripts/automation/phase9f4c_d_owner_protocol.py::display_path",kind:"function"})
# MERGE (cargo_target:Config {id:"scripts/automation/phase9f4c_d_owner_protocol.py::CARGO_TARGET_DIR",kind:"path"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (cargo:ExternalService {id:"cargo.domain_automation_gates",kind:"command"})
# MERGE (git:ExternalService {id:"git.diff_check",kind:"command"})
# MERGE (audit:Config {id:"scripts/automation/phase9f4c_d_owner_protocol.py::audit.jsonl",kind:"file"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(write_text)
# MERGE (self)-[:DEFINES]->(root)
# MERGE (self)-[:DEFINES]->(output_root)
# MERGE (self)-[:DEFINES]->(display_path)
# MERGE (self)-[:DEFINES]->(cargo_target)
# MERGE (main)-[:USES]->(root)
# MERGE (main)-[:USES]->(output_root)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:CONFIGURES]->(cargo_target)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (run_step)-[:CALLS]->(write_text)
# MERGE (run_step)-[:CALLS]->(display_path)
# MERGE (run_step)-[:CONFIGURES]->(cargo)
# MERGE (run_step)-[:CONFIGURES]->(git)
# MERGE (run_step)-[:WRITES]->(audit)
# @endcypher

from __future__ import annotations

import argparse
import json
import os
import subprocess
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
MANIFEST = Path("crates/domain-automation/Cargo.toml")
OUTPUT_ROOT = ROOT / "target" / "phase9f4c-d-owner-protocol"
DEFAULT_CARGO_TARGET_DIR = Path(
    os.environ.get("CARGO_TARGET_DIR", str(ROOT / "target" / "9f4c-cargo-target"))
)
MAX_STEP_SECONDS = 900
BASELINE_CLIPPY_ALLOW = [
    "clippy::derivable_impls",
    "clippy::unnecessary_map_or",
    "clippy::unnecessary_sort_by",
    "clippy::bool_assert_comparison",
]


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def display_path(path: Path) -> str:
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(ROOT))
    except ValueError:
        return str(resolved)


def run_step(
    name: str,
    command: Sequence[str],
    environment: dict[str, str],
    output_dir: Path,
) -> bool:
    try:
        result = subprocess.run(
            list(command),
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=MAX_STEP_SECONDS,
            check=False,
        )
        stdout = result.stdout or ""
        stderr = result.stderr or ""
        exit_code = result.returncode
        timed_out = False
    except subprocess.TimeoutExpired as error:
        stdout = (
            error.stdout.decode("utf-8", errors="replace")
            if isinstance(error.stdout, bytes)
            else error.stdout or ""
        )
        stderr = (
            error.stderr.decode("utf-8", errors="replace")
            if isinstance(error.stderr, bytes)
            else error.stderr or ""
        )
        exit_code = None
        timed_out = True
    except OSError as error:
        stdout = ""
        stderr = f"could not execute configured tool: {error}"
        exit_code = None
        timed_out = False

    stdout_path = output_dir / f"{name}.stdout.log"
    stderr_path = output_dir / f"{name}.stderr.log"
    write_text(stdout_path, stdout)
    write_text(stderr_path, stderr)
    event = {
        "step": name,
        "command": list(command),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "stdout_log": display_path(stdout_path),
        "stderr_log": display_path(stderr_path),
    }
    with (output_dir / "audit.jsonl").open("a", encoding="utf-8") as audit_log:
        audit_log.write(json.dumps(event, sort_keys=True) + "\n")
    passed = exit_code == 0 and not timed_out
    print(
        json.dumps(
            {"step": name, "passed": passed, "exit_code": exit_code, "timed_out": timed_out}
        )
    )
    return passed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", default="cargo", help="Cargo executable or command name")
    parser.add_argument("--git", default="git", help="Git executable or command name")
    parser.add_argument("--toolchain-bin", type=Path, help="Optional Rust toolchain bin directory")
    parser.add_argument("--target-dir", type=Path, help="Optional Cargo target directory")
    parser.add_argument("--output-dir", type=Path, help="Optional log root")
    arguments = parser.parse_args()

    environment = os.environ.copy()
    if arguments.toolchain_bin:
        environment["PATH"] = str(arguments.toolchain_bin) + os.pathsep + environment.get("PATH", "")
    cargo_target_dir = arguments.target_dir or DEFAULT_CARGO_TARGET_DIR
    environment["CARGO_TARGET_DIR"] = str(cargo_target_dir)
    run_id = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ") + "-" + uuid.uuid4().hex[:8]
    output_dir = (arguments.output_dir or OUTPUT_ROOT) / run_id
    output_dir.mkdir(parents=True, exist_ok=False)

    clippy = [
        arguments.cargo,
        "clippy",
        "--manifest-path",
        str(MANIFEST),
        "--all-targets",
        "--no-deps",
        "-j",
        "4",
        "--",
        "-D",
        "warnings",
    ]
    for lint in BASELINE_CLIPPY_ALLOW:
        clippy.extend(["-A", lint])

    steps = [
        (
            "fmt_check",
            [
                arguments.cargo,
                "fmt",
                "--manifest-path",
                str(MANIFEST),
                "--",
                "--check",
            ],
        ),
        (
            "all_targets_check",
            [
                arguments.cargo,
                "check",
                "--manifest-path",
                str(MANIFEST),
                "--all-targets",
                "-j",
                "4",
            ],
        ),
        (
            "admission_tests",
            [
                arguments.cargo,
                "test",
                "--manifest-path",
                str(MANIFEST),
                "schedule::tests::admission_",
                "-j",
                "4",
            ],
        ),
        (
            "package_tests",
            [
                arguments.cargo,
                "test",
                "--manifest-path",
                str(MANIFEST),
                "-j",
                "4",
            ],
        ),
        ("clippy_with_baseline_exceptions", clippy),
        ("diff_check", [arguments.git, "diff", "--check"]),
    ]
    passed = [
        run_step(name, command, environment, output_dir)
        for name, command in steps
    ]
    summary = {
        "run_id": run_id,
        "passed": all(passed),
        "steps": [name for name, _ in steps],
        "output_dir": display_path(output_dir),
        "cargo_target_dir": display_path(cargo_target_dir),
        "baseline_clippy_allow": BASELINE_CLIPPY_ALLOW,
    }
    write_text(output_dir / "summary.json", json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, sort_keys=True))
    return 0 if all(passed) else 1


if __name__ == "__main__":
    raise SystemExit(main())
