"""Run reproducible focused gates for the 9F4D infrastructure contract slice."""

# @cypher schema=1 source_sha256=b02eed9bb906e478037c268cf00f5f79ad5a0f96db0baf8767fc1fde968ea942
# MERGE (self:File {path:"scripts/automation/phase9f4d_infrastructure_contract.py"})
# MERGE (main:Symbol {id:"scripts/automation/phase9f4d_infrastructure_contract.py::main",kind:"function"})
# MERGE (run_step:Symbol {id:"scripts/automation/phase9f4d_infrastructure_contract.py::run_step",kind:"function"})
# MERGE (write_text:Symbol {id:"scripts/automation/phase9f4d_infrastructure_contract.py::write_text",kind:"function"})
# MERGE (source_path:Variable {id:"scripts/automation/phase9f4d_infrastructure_contract.py::SOURCE",kind:"constant"})
# MERGE (output_dir:Variable {id:"scripts/automation/phase9f4d_infrastructure_contract.py::OUTPUT_DIR",kind:"constant"})
# MERGE (cargo_target_dir:Config {id:"scripts/automation/phase9f4d_infrastructure_contract.py::DEFAULT_CARGO_TARGET_DIR",kind:"path"})
# MERGE (step_timeout:Variable {id:"scripts/automation/phase9f4d_infrastructure_contract.py::MAX_STEP_SECONDS",kind:"constant"})
# MERGE (repo:ExternalService {id:"git.repository",kind:"directory"})
# MERGE (cargo:ExternalService {id:"cargo.fmt_check_compile_clippy_test",kind:"command"})
# MERGE (cargo_clippy:ExternalService {id:"cargo-clippy.subcommand",kind:"command"})
# MERGE (rustfmt:ExternalService {id:"rustfmt.check",kind:"command"})
# MERGE (subprocess:ExternalService {id:"python.subprocess.run",kind:"function"})
# MERGE (audit:Config {id:"scripts/automation/phase9f4d_infrastructure_contract.py::audit.jsonl"})
# MERGE (source:File {path:"crates/domain-infrastructure/src/lib.rs"})
# MERGE (readiness_source:File {path:"crates/domain-infrastructure/src/readiness.rs"})
# MERGE (self)-[:DEFINES]->(main)
# MERGE (self)-[:DEFINES]->(run_step)
# MERGE (self)-[:DEFINES]->(write_text)
# MERGE (self)-[:DEFINES]->(source_path)
# MERGE (self)-[:DEFINES]->(output_dir)
# MERGE (self)-[:DEFINES]->(cargo_target_dir)
# MERGE (self)-[:DEFINES]->(step_timeout)
# MERGE (main)-[:USES]->(repo)
# MERGE (main)-[:CALLS]->(run_step)
# MERGE (main)-[:USES]->(source_path)
# MERGE (main)-[:USES]->(output_dir)
# MERGE (main)-[:CONFIGURES]->(cargo_target_dir)
# MERGE (main)-[:CONFIGURES]->(source)
# MERGE (run_step)-[:CALLS]->(subprocess)
# MERGE (run_step)-[:CALLS]->(write_text)
# MERGE (run_step)-[:USES]->(step_timeout)
# MERGE (run_step)-[:CONFIGURES]->(cargo)
# MERGE (run_step)-[:CONFIGURES]->(cargo_clippy)
# MERGE (run_step)-[:CONFIGURES]->(rustfmt)
# MERGE (main)-[:CONFIGURES]->(readiness_source)
# MERGE (run_step)-[:WRITES]->(audit)
# @endcypher

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Sequence

ROOT = Path(__file__).resolve().parents[2]
SOURCE = Path("crates/domain-infrastructure/src/lib.rs")
READINESS_SOURCE = Path("crates/domain-infrastructure/src/readiness.rs")
OUTPUT_DIR = ROOT / "target" / "phase9f4d-infrastructure-contract"
DEFAULT_CARGO_TARGET_DIR = ROOT / "target" / "9f4d-cargo-target"
MAX_STEP_SECONDS = 900


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


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
        stdout = error.stdout.decode("utf-8", errors="replace") if isinstance(error.stdout, bytes) else error.stdout or ""
        stderr = error.stderr.decode("utf-8", errors="replace") if isinstance(error.stderr, bytes) else error.stderr or ""
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
        "stdout_log": str(stdout_path.relative_to(ROOT)),
        "stderr_log": str(stderr_path.relative_to(ROOT)),
    }
    with (output_dir / "audit.jsonl").open("a", encoding="utf-8") as audit_log:
        audit_log.write(json.dumps(event, sort_keys=True) + "\n")
    passed = exit_code == 0 and not timed_out
    print(json.dumps({"step": name, "passed": passed, "exit_code": exit_code, "timed_out": timed_out}))
    return passed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cargo", default="cargo", help="Cargo executable or command name")
    parser.add_argument("--rustfmt", default="rustfmt", help="rustfmt executable or command name")
    parser.add_argument("--toolchain-bin", type=Path, help="Optional Rust toolchain bin directory")
    parser.add_argument("--target-dir", type=Path, help="Optional Cargo target directory")
    arguments = parser.parse_args()

    output_dir = OUTPUT_DIR
    output_dir.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    if arguments.toolchain_bin:
        toolchain_bin = arguments.toolchain_bin.resolve()
        environment["PATH"] = str(toolchain_bin) + os.pathsep + environment.get("PATH", "")
        environment["RUSTC"] = str(toolchain_bin / ("rustc.exe" if os.name == "nt" else "rustc"))
        cargo = str(toolchain_bin / ("cargo.exe" if os.name == "nt" else "cargo"))
        cargo_clippy = str(toolchain_bin / ("cargo-clippy.exe" if os.name == "nt" else "cargo-clippy"))
        rustfmt = str(toolchain_bin / ("rustfmt.exe" if os.name == "nt" else "rustfmt"))
    else:
        cargo = arguments.cargo
        cargo_clippy = cargo
        rustfmt = arguments.rustfmt

    target_dir = (arguments.target_dir or DEFAULT_CARGO_TARGET_DIR).resolve()
    environment["CARGO_TARGET_DIR"] = str(target_dir)
    commands = (
        ("format", [rustfmt, "--edition", "2021", "--check", str(SOURCE), str(READINESS_SOURCE)]),
        ("check", [cargo, "check", "--locked", "-p", "domain-infrastructure", "--all-targets", "-j", "4"]),
        ("clippy", [cargo_clippy, "clippy", "--locked", "-p", "domain-infrastructure", "--all-targets", "-j", "4", "--", "-D", "warnings"]),
        ("test", [cargo, "test", "--locked", "-p", "domain-infrastructure", "--all-targets", "-j", "4"]),
    )
    (output_dir / "audit.jsonl").write_text("", encoding="utf-8")
    results = [run_step(name, command, environment, output_dir) for name, command in commands]
    print(json.dumps({"result": "passed" if all(results) else "failed", "steps": len(results)}))
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
