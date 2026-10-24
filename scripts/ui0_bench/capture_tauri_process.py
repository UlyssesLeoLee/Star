# @cypher schema=1 source_sha256=9caef7b7b98048df927727a3013fd2420c40a86cde5db9d2aaa7789bc12df304
# MERGE (self:File {path:"scripts/ui0_bench/capture_tauri_process.py"})
# MERGE (process_record:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::ProcessRecord", kind:"class"})
# MERGE (capture_error:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::CaptureError", kind:"class"})
# MERGE (capture_options:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::CaptureOptions", kind:"class"})
# MERGE (validate_options:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::validate_options", kind:"function"})
# MERGE (parse_process_snapshot:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::parse_process_snapshot", kind:"function"})
# MERGE (process_tree:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::process_tree", kind:"function"})
# MERGE (cpu_delta_100ns:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::cpu_delta_100ns", kind:"function"})
# MERGE (cpu_percent:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::cpu_percent", kind:"function"})
# MERGE (sample_rows:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::sample_rows", kind:"function"})
# MERGE (powershell_query:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::POWERSHELL_QUERY", kind:"variable"})
# MERGE (take_snapshot:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::take_snapshot", kind:"function"})
# MERGE (write_sample:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::write_sample", kind:"function"})
# MERGE (capture:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::capture", kind:"function"})
# MERGE (build_parser:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::build_parser", kind:"function"})
# MERGE (main:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::main", kind:"function"})
# MERGE (self)-[:DEFINES]->(process_record)
# MERGE (self)-[:DEFINES]->(capture_error)
# MERGE (self)-[:DEFINES]->(capture_options)
# MERGE (self)-[:DEFINES]->(validate_options)
# MERGE (self)-[:DEFINES]->(parse_process_snapshot)
# MERGE (self)-[:DEFINES]->(process_tree)
# MERGE (self)-[:DEFINES]->(cpu_delta_100ns)
# MERGE (self)-[:DEFINES]->(cpu_percent)
# MERGE (self)-[:DEFINES]->(sample_rows)
# MERGE (self)-[:DEFINES]->(powershell_query)
# MERGE (self)-[:DEFINES]->(take_snapshot)
# MERGE (self)-[:DEFINES]->(write_sample)
# MERGE (self)-[:DEFINES]->(capture)
# MERGE (self)-[:DEFINES]->(build_parser)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (validate_options)-[:CALLS]->(CaptureError)
# MERGE (parse_process_snapshot)-[:CALLS]->(CaptureError)
# MERGE (process_tree)-[:CALLS]->(CaptureError)
# MERGE (sample_rows)-[:CALLS]->(cpu_delta_100ns)
# MERGE (sample_rows)-[:CALLS]->(cpu_percent)
# MERGE (take_snapshot)-[:CALLS]->(parse_process_snapshot)
# MERGE (take_snapshot)-[:USES]->(powershell_query)
# MERGE (write_sample)-[:CALLS]->(CaptureError)
# MERGE (capture)-[:CALLS]->(validate_options)
# MERGE (capture)-[:CALLS]->(take_snapshot)
# MERGE (capture)-[:CALLS]->(process_tree)
# MERGE (capture)-[:CALLS]->(sample_rows)
# MERGE (capture)-[:CALLS]->(write_sample)
# MERGE (main)-[:CALLS]->(build_parser)
# MERGE (main)-[:CALLS]->(capture)
# @endcypher
"""Read-only Windows process-tree sampler for UI-0 Tauri measurements.

RSS is the sum of each process' Windows working set; shared pages may be counted
more than once, so this is not unique physical memory for the process tree.
CPU is reported as percent of total logical-processor capacity (0..100). The
first sample omits CPU because it has no prior cumulative-time delta.
"""

from __future__ import annotations

import argparse
import json
import math
import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Callable, TextIO


MAX_SNAPSHOT_PROCESSES = 8_192
MAX_TREE_PROCESSES = 4_096
MAX_SNAPSHOT_OUTPUT_BYTES = 2 * 1_048_576
MAX_SAMPLE_LINE_BYTES = 16_384
MAX_SAMPLE_COUNT = 10_000
MAX_SAMPLE_BYTES = 8 * 1_048_576
MAX_SCENARIO_CHARS = 128
MAX_DURATION_SECONDS = 3_600.0
MIN_INTERVAL_SECONDS = 1.0
MAX_INTERVAL_SECONDS = 60.0
SNAPSHOT_TIMEOUT_SECONDS = 10.0
HUNDRED_NANOSECONDS_PER_SECOND = 10_000_000
SCENARIO_PATTERN = re.compile(r"^[A-Za-z0-9_.:-]{1,128}$")


class CaptureError(ValueError):
    """A bounded, non-sensitive sampler failure reason."""

    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


@dataclass(frozen=True)
class ProcessRecord:
    process_id: int
    parent_process_id: int
    creation_date: datetime
    working_set_bytes: int
    kernel_time_100ns: int
    user_time_100ns: int

    @property
    def cpu_time_100ns(self) -> int:
        return self.kernel_time_100ns + self.user_time_100ns


@dataclass(frozen=True)
class CaptureOptions:
    root_pid: int
    scenario: str
    condition: str
    interval_seconds: float
    duration_seconds: float
    output: Path


POWERSHELL_QUERY = r"""
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
try {
    if (-not (Get-Command Get-CimInstance -ErrorAction SilentlyContinue)) {
        [Console]::Out.Write('STAR_UI0_CIM_UNAVAILABLE')
        exit 20
    }
    $all = @(Get-CimInstance -ClassName Win32_Process -Property ProcessId,ParentProcessId,CreationDate,WorkingSetSize,KernelModeTime,UserModeTime)
    if ($all.Count -gt 8192) {
        [Console]::Out.Write('STAR_UI0_PROCESS_LIMIT')
        exit 21
    }
    $byPid = @{}
    $childrenByParent = @{}
    foreach ($item in $all) {
        $key = [string][uint32]$item.ProcessId
        $byPid[$key] = $item
        $parentKey = [string][uint32]$item.ParentProcessId
        if (-not $childrenByParent.ContainsKey($parentKey)) {
            $childrenByParent[$parentKey] = New-Object 'System.Collections.Generic.List[object]'
        }
        $childrenByParent[$parentKey].Add($item)
    }
    $rootKey = [string][uint32]__ROOT_PID__
    if (-not $byPid.ContainsKey($rootKey)) {
        [Console]::Out.Write('STAR_UI0_ROOT_MISSING')
        exit 22
    }
    $selected = @{}
    $queue = New-Object 'System.Collections.Generic.Queue[object]'
    $queue.Enqueue($byPid[$rootKey])
    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        $currentKey = [string][uint32]$current.ProcessId
        if ($selected.ContainsKey($currentKey)) { continue }
        if ($selected.Count -ge 4096) {
            [Console]::Out.Write('STAR_UI0_TREE_LIMIT')
            exit 23
        }
        $selected[$currentKey] = $current
        if ($childrenByParent.ContainsKey($currentKey)) {
            foreach ($child in $childrenByParent[$currentKey]) {
                if ($child.CreationDate -ge $current.CreationDate) { $queue.Enqueue($child) }
            }
        }
    }
    foreach ($item in $selected.Values) {
        if ($null -eq $item.CreationDate -or $null -eq $item.WorkingSetSize -or
            $null -eq $item.KernelModeTime -or $null -eq $item.UserModeTime) {
            [Console]::Out.Write('STAR_UI0_PROPERTY_UNAVAILABLE')
            exit 24
        }
    }
    $rows = @(
        foreach ($item in ($selected.Values | Sort-Object -Property ProcessId)) {
            [PSCustomObject]@{
                process_id = [uint32]$item.ProcessId
                parent_process_id = [uint32]$item.ParentProcessId
                creation_date_utc = $item.CreationDate.ToUniversalTime().ToString('o',[Globalization.CultureInfo]::InvariantCulture)
                working_set_bytes = [uint64]$item.WorkingSetSize
                kernel_time_100ns = [uint64]$item.KernelModeTime
                user_time_100ns = [uint64]$item.UserModeTime
            }
        }
    )
    [Console]::Out.Write((ConvertTo-Json -InputObject $rows -Compress -Depth 3))
} catch {
    [Console]::Out.Write('STAR_UI0_SNAPSHOT_FAILED')
    exit 24
}
""".strip()


def _integer(value: object, label: str, *, maximum: int = (1 << 63) - 1) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0 or value > maximum:
        raise CaptureError(f"invalid_{label}")
    return value


def validate_options(options: CaptureOptions) -> int:
    _integer(options.root_pid, "root_pid", maximum=(1 << 32) - 1)
    if options.root_pid == 0:
        raise CaptureError("invalid_root_pid")
    if not isinstance(options.scenario, str) or len(options.scenario) > MAX_SCENARIO_CHARS or not SCENARIO_PATTERN.fullmatch(options.scenario):
        raise CaptureError("invalid_scenario")
    if options.condition not in ("cold", "warm"):
        raise CaptureError("invalid_condition")
    for label, value, minimum, maximum in (
        ("interval", options.interval_seconds, MIN_INTERVAL_SECONDS, MAX_INTERVAL_SECONDS),
        ("duration", options.duration_seconds, MIN_INTERVAL_SECONDS, MAX_DURATION_SECONDS),
    ):
        if isinstance(value, bool) or not isinstance(value, (float, int)) or not math.isfinite(float(value)):
            raise CaptureError(f"invalid_{label}")
        if value < minimum or value > maximum:
            raise CaptureError(f"invalid_{label}")
    maximum_ticks = math.floor(options.duration_seconds / options.interval_seconds) + 1
    maximum_lines = maximum_ticks * 3
    if maximum_lines > MAX_SAMPLE_COUNT:
        raise CaptureError("sample_count_limit")
    if not isinstance(options.output, Path):
        raise CaptureError("invalid_output_path")
    return maximum_lines


def _parse_creation_date(value: object) -> datetime:
    if not isinstance(value, str) or len(value) > 40:
        raise CaptureError("invalid_process_snapshot")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise CaptureError("invalid_process_snapshot") from exc
    if parsed.tzinfo is None:
        raise CaptureError("invalid_process_snapshot")
    return parsed


def parse_process_snapshot(stdout: str) -> tuple[ProcessRecord, ...]:
    encoded_size = len(stdout.encode("utf-8", errors="replace"))
    if encoded_size > MAX_SNAPSHOT_OUTPUT_BYTES:
        raise CaptureError("snapshot_output_limit")
    if stdout in {
        "STAR_UI0_CIM_UNAVAILABLE",
        "STAR_UI0_PROCESS_LIMIT",
        "STAR_UI0_ROOT_MISSING",
        "STAR_UI0_TREE_LIMIT",
        "STAR_UI0_PROPERTY_UNAVAILABLE",
        "STAR_UI0_SNAPSHOT_FAILED",
    }:
        reason = {
            "STAR_UI0_CIM_UNAVAILABLE": "cim_unavailable",
            "STAR_UI0_PROCESS_LIMIT": "process_count_limit",
            "STAR_UI0_ROOT_MISSING": "root_process_missing",
            "STAR_UI0_TREE_LIMIT": "process_tree_limit",
            "STAR_UI0_PROPERTY_UNAVAILABLE": "process_property_unavailable",
            "STAR_UI0_SNAPSHOT_FAILED": "snapshot_failed",
        }[stdout]
        raise CaptureError(reason)
    try:
        raw = json.loads(stdout)
    except json.JSONDecodeError as exc:
        raise CaptureError("invalid_process_snapshot") from exc
    if not isinstance(raw, list) or not raw or len(raw) > MAX_TREE_PROCESSES:
        raise CaptureError("invalid_process_snapshot")
    records: list[ProcessRecord] = []
    seen: set[int] = set()
    for item in raw:
        if not isinstance(item, dict) or set(item) != {
            "process_id", "parent_process_id", "creation_date_utc", "working_set_bytes",
            "kernel_time_100ns", "user_time_100ns",
        }:
            raise CaptureError("invalid_process_snapshot")
        record = ProcessRecord(
            process_id=_integer(item["process_id"], "process_id", maximum=(1 << 32) - 1),
            parent_process_id=_integer(item["parent_process_id"], "parent_process_id", maximum=(1 << 32) - 1),
            creation_date=_parse_creation_date(item["creation_date_utc"]),
            working_set_bytes=_integer(item["working_set_bytes"], "working_set_bytes"),
            kernel_time_100ns=_integer(item["kernel_time_100ns"], "kernel_time_100ns"),
            user_time_100ns=_integer(item["user_time_100ns"], "user_time_100ns"),
        )
        if record.process_id == 0 or record.process_id in seen:
            raise CaptureError("invalid_process_snapshot")
        seen.add(record.process_id)
        records.append(record)
    return tuple(records)


def process_tree(records: tuple[ProcessRecord, ...], root_pid: int) -> dict[int, ProcessRecord]:
    by_pid = {record.process_id: record for record in records}
    root = by_pid.get(root_pid)
    if root is None:
        raise CaptureError("root_process_missing")
    children: dict[int, list[ProcessRecord]] = {}
    for record in records:
        parent = by_pid.get(record.parent_process_id)
        if parent is not None and record.process_id != root_pid and record.creation_date >= parent.creation_date:
            children.setdefault(parent.process_id, []).append(record)
    selected: dict[int, ProcessRecord] = {}
    pending = [root]
    while pending:
        current = pending.pop()
        if current.process_id in selected:
            continue
        if len(selected) >= MAX_TREE_PROCESSES:
            raise CaptureError("process_tree_limit")
        selected[current.process_id] = current
        pending.extend(children.get(current.process_id, ()))
    return selected


def cpu_delta_100ns(previous: dict[int, ProcessRecord], current: dict[int, ProcessRecord], root_pid: int) -> int:
    previous_root = previous.get(root_pid)
    current_root = current.get(root_pid)
    if previous_root is None or current_root is None:
        raise CaptureError("root_process_missing")
    if previous_root.creation_date != current_root.creation_date:
        raise CaptureError("root_pid_reused")
    delta = 0
    for process_id, current_record in current.items():
        old_record = previous.get(process_id)
        if old_record is None or old_record.creation_date != current_record.creation_date:
            continue
        process_delta = current_record.cpu_time_100ns - old_record.cpu_time_100ns
        if process_delta < 0:
            raise CaptureError("cpu_counter_regressed")
        delta += process_delta
    return delta


def cpu_percent(cpu_delta: int, elapsed_seconds: float, logical_processors: int) -> float:
    if (
        isinstance(logical_processors, bool)
        or not isinstance(logical_processors, int)
        or logical_processors <= 0
        or elapsed_seconds <= 0
        or not math.isfinite(elapsed_seconds)
    ):
        raise CaptureError("cpu_interval_unavailable")
    total_capacity = elapsed_seconds * logical_processors * HUNDRED_NANOSECONDS_PER_SECOND
    return min(100.0, max(0.0, cpu_delta * 100.0 / total_capacity))


def sample_rows(
    options: CaptureOptions,
    current: dict[int, ProcessRecord],
    previous: dict[int, ProcessRecord] | None,
    elapsed_seconds: float | None,
    logical_processors: int,
) -> list[dict[str, object]]:
    root = current.get(options.root_pid)
    if root is None:
        raise CaptureError("root_process_missing")
    rss_total = sum(record.working_set_bytes for record in current.values())
    rows: list[dict[str, object]] = [
        {
            "kind": "sample", "scenario": options.scenario, "condition": options.condition,
            "metric": "process_tree_rss_bytes",
            "value": rss_total, "unit": "bytes", "counters": {"process_tree_rss_bytes": rss_total},
        },
        {
            "kind": "sample", "scenario": options.scenario, "condition": options.condition,
            "metric": "process_tree_process_count", "value": len(current), "unit": "count",
        },
    ]
    if previous is not None and elapsed_seconds is not None:
        delta = cpu_delta_100ns(previous, current, options.root_pid)
        measured_cpu_percent = cpu_percent(delta, elapsed_seconds, logical_processors)
        rows.append({
            "kind": "sample", "scenario": options.scenario, "condition": options.condition,
            "metric": "process_tree_cpu_percent",
            "value": measured_cpu_percent, "unit": "percent",
            "counters": {"process_tree_cpu_percent": measured_cpu_percent},
        })
    return rows


def take_snapshot(root_pid: int, *, runner: Callable[..., subprocess.CompletedProcess[str]] | None = None) -> tuple[ProcessRecord, ...]:
    root_pid = _integer(root_pid, "root_pid", maximum=(1 << 32) - 1)
    if root_pid == 0:
        raise CaptureError("invalid_root_pid")
    if not sys.platform.startswith("win"):
        raise CaptureError("windows_required")
    system_root = os.environ.get("SystemRoot")
    if not system_root or not Path(system_root).is_absolute():
        raise CaptureError("powershell_unavailable")
    powershell = Path(system_root) / "System32" / "WindowsPowerShell" / "v1.0" / "powershell.exe"
    script = POWERSHELL_QUERY.replace("__ROOT_PID__", str(root_pid))
    command = [
        str(powershell), "-NoLogo", "-NoProfile", "-NonInteractive", "-Command", script,
    ]
    execute = runner or subprocess.run
    try:
        result = execute(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=SNAPSHOT_TIMEOUT_SECONDS,
            check=False,
            shell=False,
        )
    except subprocess.TimeoutExpired as exc:
        raise CaptureError("snapshot_timeout") from exc
    except (OSError, subprocess.SubprocessError) as exc:
        raise CaptureError("powershell_unavailable") from exc
    if result.returncode != 0:
        try:
            return parse_process_snapshot(result.stdout)
        except CaptureError as exc:
            if exc.reason in {
                "cim_unavailable", "process_count_limit", "root_process_missing",
                "process_tree_limit", "process_property_unavailable", "snapshot_failed",
            }:
                raise
            raise CaptureError("snapshot_failed") from exc
    return parse_process_snapshot(result.stdout)


def write_sample(stream: TextIO, row: dict[str, object], sample_count: int, byte_count: int) -> tuple[int, int]:
    encoded = json.dumps(row, sort_keys=True, separators=(",", ":"), ensure_ascii=True) + "\n"
    line_bytes = len(encoded.encode("utf-8"))
    if line_bytes > MAX_SAMPLE_LINE_BYTES:
        raise CaptureError("sample_line_limit")
    if sample_count >= MAX_SAMPLE_COUNT or byte_count + line_bytes > MAX_SAMPLE_BYTES:
        raise CaptureError("sample_output_limit")
    stream.write(encoded)
    stream.flush()
    return sample_count + 1, byte_count + line_bytes


def capture(
    options: CaptureOptions,
    *,
    snapshot_provider: Callable[[int], tuple[ProcessRecord, ...]] | None = None,
    clock: Callable[[], float] = time.monotonic,
    sleeper: Callable[[float], None] = time.sleep,
    is_windows: bool | None = None,
    logical_processors: int | None = None,
) -> int:
    maximum_lines = validate_options(options)
    windows = sys.platform.startswith("win") if is_windows is None else is_windows
    if not windows:
        raise CaptureError("windows_required")
    processor_count = os.cpu_count() if logical_processors is None else logical_processors
    if isinstance(processor_count, bool) or not isinstance(processor_count, int) or processor_count <= 0:
        raise CaptureError("logical_processor_count_unavailable")
    provider = snapshot_provider or take_snapshot
    if not options.output.parent.is_dir():
        raise CaptureError("output_directory_missing")
    created_file = False
    complete = False
    sample_count = 0
    byte_count = 0
    try:
        with options.output.open("x", encoding="utf-8", newline="\n") as stream:
            created_file = True
            started = clock()
            sample_index = 0
            previous: dict[int, ProcessRecord] | None = None
            previous_at: float | None = None
            while True:
                due = started + sample_index * options.interval_seconds
                deadline = started + options.duration_seconds
                if due > deadline:
                    break
                delay = due - clock()
                if delay > 0:
                    sleeper(delay)
                if clock() > deadline:
                    break
                records = provider(options.root_pid)
                current = process_tree(records, options.root_pid)
                observed_at = clock()
                elapsed = observed_at - previous_at if previous_at is not None else None
                for row in sample_rows(options, current, previous, elapsed, processor_count):
                    sample_count, byte_count = write_sample(stream, row, sample_count, byte_count)
                if sample_count > maximum_lines:
                    raise CaptureError("sample_count_limit")
                previous = current
                previous_at = observed_at
                elapsed_from_start = max(0.0, observed_at - started)
                next_index = math.ceil(elapsed_from_start / options.interval_seconds)
                sample_index = max(sample_index + 1, next_index)
            if sample_count == 0:
                raise CaptureError("no_samples_collected")
        complete = True
        return sample_count
    except FileExistsError as exc:
        raise CaptureError("output_already_exists") from exc
    except OSError as exc:
        raise CaptureError("output_write_failed") from exc
    finally:
        if created_file and not complete:
            try:
                options.output.unlink(missing_ok=True)
            except OSError:
                pass


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Read-only Windows Tauri process-tree RSS/CPU sampler")
    parser.add_argument("--root-pid", required=True, type=int, help="operator-selected Tauri root PID")
    parser.add_argument("--scenario", required=True, help="bounded scenario identifier")
    parser.add_argument("--condition", required=True, choices=("cold", "warm"))
    parser.add_argument("--interval-seconds", required=True, type=float, help="sampling interval in seconds (minimum 1.0)")
    parser.add_argument("--duration-seconds", required=True, type=float, help="sampling duration in seconds (1 through 3600)")
    parser.add_argument("--output", required=True, type=Path, help="new JSONL file; existing paths are never overwritten")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    options = CaptureOptions(
        root_pid=args.root_pid,
        scenario=args.scenario,
        condition=args.condition,
        interval_seconds=args.interval_seconds,
        duration_seconds=args.duration_seconds,
        output=args.output,
    )
    try:
        count = capture(options)
    except CaptureError as exc:
        print(json.dumps({"status": "error", "reason": exc.reason}, separators=(",", ":")), file=sys.stderr)
        return 2
    print(json.dumps({"status": "completed", "samples_written": count}, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
