# @cypher schema=1 source_sha256=8da72ea3e28782dff3df9f269f969f8194677d41652e47f3896ccc28b526e7af
# MERGE (self:File {path:"scripts/ui0_bench/run_measurement.py"})
# MERGE (self)-[:IMPORTS]->(:File {path:"scripts/ui0_bench/generate_fixture.py"})
# MERGE (sample_error:Symbol {id:"scripts/ui0_bench/run_measurement.py::SampleError", kind:"class"})
# MERGE (validate_sample:Symbol {id:"scripts/ui0_bench/run_measurement.py::_validate_sample", kind:"function"})
# MERGE (run_measurement:Symbol {id:"scripts/ui0_bench/run_measurement.py::run_measurement", kind:"function"})
# MERGE (read_driver_samples:Symbol {id:"scripts/ui0_bench/run_measurement.py::read_driver_samples", kind:"function"})
# MERGE (inventory:Symbol {id:"scripts/ui0_bench/run_measurement.py::_inventory", kind:"function"})
# MERGE (git_commit:Symbol {id:"scripts/ui0_bench/run_measurement.py::_git_commit", kind:"function"})
# MERGE (summarize_measurements:Symbol {id:"scripts/ui0_bench/run_measurement.py::summarize_measurements", kind:"function"})
# MERGE (sha256_file:Symbol {id:"scripts/ui0_bench/run_measurement.py::_sha256_file", kind:"function"})
# MERGE (open_regular_sample_file:Symbol {id:"scripts/ui0_bench/run_measurement.py::_open_regular_sample_file", kind:"function"})
# MERGE (load_manifest:Symbol {id:"scripts/ui0_bench/generate_fixture.py::load_manifest", kind:"function"})
# MERGE (main:Symbol {id:"scripts/ui0_bench/run_measurement.py::main", kind:"function"})
# MERGE (self)-[:DEFINES]->(sample_error)
# MERGE (self)-[:DEFINES]->(validate_sample)
# MERGE (self)-[:DEFINES]->(run_measurement)
# MERGE (self)-[:DEFINES]->(read_driver_samples)
# MERGE (self)-[:DEFINES]->(inventory)
# MERGE (self)-[:DEFINES]->(git_commit)
# MERGE (self)-[:DEFINES]->(summarize_measurements)
# MERGE (self)-[:DEFINES]->(sha256_file)
# MERGE (self)-[:DEFINES]->(open_regular_sample_file)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (read_driver_samples)-[:CALLS]->(validate_sample)
# MERGE (read_driver_samples)-[:CALLS]->(summarize_measurements)
# MERGE (run_measurement)-[:CALLS]->(read_driver_samples)
# MERGE (run_measurement)-[:CALLS]->(inventory)
# MERGE (run_measurement)-[:CALLS]->(git_commit)
# MERGE (run_measurement)-[:CALLS]->(load_manifest)
# MERGE (run_measurement)-[:CALLS]->(sha256_file)
# MERGE (run_measurement)-[:CALLS]->(open_regular_sample_file)
# MERGE (main)-[:CALLS]->(run_measurement)
# @endcypher

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import re
import stat
import subprocess
import sys
import uuid
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path
from typing import BinaryIO, TextIO

from generate_fixture import ManifestError, load_manifest


RUNNER_VERSION = "1.0.0"
MAX_INVENTORY_BYTES = 1_048_576
MAX_SAMPLE_LINE_BYTES = 16_384
MAX_SAMPLE_COUNT = 10_000
MAX_SAMPLE_INPUT_BYTES = 8 * 1_048_576
SCENARIO_PATTERN = re.compile(r"^[A-Za-z0-9_.:-]{1,128}$")
METRIC_PATTERN = re.compile(r"^[a-z][a-z0-9_]{0,63}$")
UNIT_PATTERN = re.compile(r"^[A-Za-z0-9_/%.-]{1,24}$")
COUNTER_NAMES = (
    "process_tree_rss_bytes",
    "process_tree_cpu_percent",
    "cpu_percent",
    "queue_items",
    "queue_bytes",
    "queue_age_ms",
    "cache_items",
    "cache_bytes",
    "terminal_buffer_bytes",
    "canvas_visible_nodes",
    "canvas_visible_edges",
    "cancel_to_drain_ms",
    "stop_to_drain_ms",
    "orphaned_handles",
)
LIFECYCLE_EVENTS = ("stop_requested", "cancel_requested", "drain_started", "drain_completed")
TRACKS = ("tauri-react", "shared-web", "rust-native")
INVENTORY_METHODS = {
    "Get-CimInstance Win32_OperatingSystem",
    "Get-CimInstance Win32_Processor",
    "Get-CimInstance Win32_ComputerSystem",
    "Get-CimInstance Win32_VideoController",
    "powercfg.exe /getactivescheme",
    "System.Windows.Forms + Win32 display APIs",
    "System.Windows.Forms.Screen.AllScreens",
    "user32.EnumDisplaySettings",
    "Shcore.GetDpiForMonitor",
    "System.Windows.Forms.Screen.Bounds",
}
INVENTORY_REASONS = {"collection_failed", "windows_only_api", "no_active_displays", "display_api_unavailable"}


class SampleError(ValueError):
    """Raised when a UI driver sample does not match the bounded input contract."""


def percentile(values: list[float], p: float) -> float | None:
    if not values:
        return None
    if not math.isfinite(p) or p < 0 or p > 100:
        raise ValueError("percentile must be between 0 and 100")
    ordered = sorted(values)
    position = (len(ordered) - 1) * (p / 100.0)
    lower = math.floor(position)
    upper = math.ceil(position)
    if lower == upper:
        return ordered[lower]
    weight = position - lower
    return ordered[lower] * (1.0 - weight) + ordered[upper] * weight


def summarize_measurements(values: list[float]) -> dict[str, float | int | None]:
    return {
        "count": len(values),
        "p50": percentile(values, 50),
        "p95": percentile(values, 95),
    }


def _finite_number(value: object, label: str) -> float:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        raise SampleError(f"{label} must be a finite number")
    try:
        number = float(value)
    except (OverflowError, ValueError) as exc:
        raise SampleError(f"{label} must be a finite number") from exc
    if not math.isfinite(number):
        raise SampleError(f"{label} must be a finite number")
    return number


def _validate_sample(sample: object) -> dict[str, object]:
    if not isinstance(sample, dict) or not all(isinstance(key, str) for key in sample):
        raise SampleError("sample must be a JSON object")
    kind = sample.get("kind")
    if kind == "sample":
        allowed = {"kind", "scenario", "condition", "metric", "value", "unit", "counters"}
        if set(sample) - allowed or not allowed - {"counters"} <= set(sample):
            raise SampleError("measurement sample has missing or unsupported fields")
        if sample["condition"] not in ("cold", "warm"):
            raise SampleError("condition must be cold or warm")
        if not isinstance(sample["scenario"], str) or not SCENARIO_PATTERN.fullmatch(sample["scenario"]):
            raise SampleError("scenario must use the bounded identifier format")
        if not isinstance(sample["metric"], str) or not METRIC_PATTERN.fullmatch(sample["metric"]):
            raise SampleError("metric must use the bounded identifier format")
        if not isinstance(sample["unit"], str) or not UNIT_PATTERN.fullmatch(sample["unit"]):
            raise SampleError("unit must use the bounded identifier format")
        normalized: dict[str, object] = {
            "kind": "sample",
            "scenario": sample["scenario"],
            "condition": sample["condition"],
            "metric": sample["metric"],
            "value": _finite_number(sample["value"], "value"),
            "unit": sample["unit"],
        }
        if "counters" in sample:
            counters = sample["counters"]
            if not isinstance(counters, dict) or set(counters) - set(COUNTER_NAMES):
                raise SampleError("counters contains unsupported names")
            cleaned_counters: dict[str, float] = {}
            for name, value in sorted(counters.items()):
                number = _finite_number(value, f"counter {name}")
                if number < 0:
                    raise SampleError(f"counter {name} must be non-negative")
                cleaned_counters[name] = number
            normalized["counters"] = cleaned_counters
        return normalized
    if kind == "lifecycle":
        allowed = {"kind", "scenario", "event", "elapsed_ms"}
        if set(sample) - allowed or not {"kind", "scenario", "event"} <= set(sample):
            raise SampleError("lifecycle sample has missing or unsupported fields")
        if not isinstance(sample["scenario"], str) or not SCENARIO_PATTERN.fullmatch(sample["scenario"]):
            raise SampleError("scenario must use the bounded identifier format")
        if sample["event"] not in LIFECYCLE_EVENTS:
            raise SampleError("event must be a stop/cancel/drain lifecycle event")
        normalized = {"kind": "lifecycle", "scenario": sample["scenario"], "event": sample["event"]}
        if "elapsed_ms" in sample:
            normalized["elapsed_ms"] = _finite_number(sample["elapsed_ms"], "elapsed_ms")
        return normalized
    raise SampleError("kind must be sample or lifecycle")


def read_driver_samples(source: BinaryIO, raw_output: TextIO) -> dict[str, object]:
    sample_hash = hashlib.sha256()
    input_bytes = 0
    sample_count = 0
    lifecycle_count = 0
    metric_values: dict[tuple[str, str, str], list[float]] = defaultdict(list)
    counter_values: dict[tuple[str, str], list[float]] = defaultdict(list)
    counter_units: dict[str, str] = {}
    metric_units: dict[tuple[str, str], str] = {}
    lifecycle_counts = {event: 0 for event in LIFECYCLE_EVENTS}
    lifecycle_elapsed: dict[str, list[float]] = defaultdict(list)

    while True:
        remaining_bytes = MAX_SAMPLE_INPUT_BYTES - input_bytes
        read_limit = min(MAX_SAMPLE_LINE_BYTES + 1, remaining_bytes + 1)
        line = source.readline(read_limit)
        if not line:
            break
        input_bytes += len(line)
        if input_bytes > MAX_SAMPLE_INPUT_BYTES:
            raise SampleError(f"driver sample input exceeds {MAX_SAMPLE_INPUT_BYTES} bytes")
        if len(line) > MAX_SAMPLE_LINE_BYTES:
            raise SampleError(f"driver sample line exceeds {MAX_SAMPLE_LINE_BYTES} bytes")
        if not line.strip():
            continue
        try:
            parsed = json.loads(line.decode("utf-8"))
        except (UnicodeDecodeError, json.JSONDecodeError) as exc:
            raise SampleError("driver samples must be UTF-8 JSONL") from exc
        sample = _validate_sample(parsed)
        if sample["kind"] == "sample":
            sample_count += 1
            metric_key = (sample["condition"], sample["metric"])
            previous_metric_unit = metric_units.setdefault(metric_key, sample["unit"])
            if sample["unit"] != previous_metric_unit:
                raise SampleError(f"metric {sample['metric']} has inconsistent units")
        else:
            lifecycle_count += 1
            if sample.get("elapsed_ms") is not None:
                if sample["elapsed_ms"] < 0:
                    raise SampleError("elapsed_ms must be non-negative")
                lifecycle_elapsed[sample["event"]].append(sample["elapsed_ms"])
        if sample_count + lifecycle_count > MAX_SAMPLE_COUNT:
            raise SampleError(f"driver sample count exceeds {MAX_SAMPLE_COUNT}")
        encoded = json.dumps(sample, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
        raw_output.write(encoded)
        sample_hash.update(encoded.encode("utf-8"))
        if sample["kind"] == "sample":
            group = (sample["condition"], sample["metric"], sample["unit"])
            metric_values[group].append(sample["value"])
            for name, value in sample.get("counters", {}).items():
                unit = (
                    "bytes" if name.endswith("_bytes") else
                    "percent" if name.endswith("_percent") else
                    "ms" if name.endswith("_ms") else
                    "count"
                )
                previous_unit = counter_units.setdefault(name, unit)
                if unit != previous_unit:
                    raise SampleError(f"counter {name} has inconsistent units")
                counter_values[(sample["condition"], name)].append(value)
        else:
            lifecycle_counts[sample["event"]] += 1

    metrics = {
        f"{condition}:{metric}": {
            "condition": condition,
            "metric": metric,
            "unit": unit,
            **summarize_measurements(values),
        }
        for (condition, metric, unit), values in sorted(metric_values.items())
    }
    counters: dict[str, object] = {}
    for name in COUNTER_NAMES:
        observed = {
            condition: values
            for (condition, counter_name), values in counter_values.items()
            if counter_name == name
        }
        counters[name] = {
            "status": "available" if observed else "unknown",
            "reason": None if observed else "not_supplied_by_driver",
            "unit": counter_units.get(name),
            "conditions": {condition: summarize_measurements(values) for condition, values in sorted(observed.items())},
        }
    lifecycle = {
        event: {
            "status": "available" if count else "unknown",
            "reason": None if count else "not_supplied_by_driver",
            "count": count,
            "elapsed_ms": summarize_measurements(lifecycle_elapsed[event]) if lifecycle_elapsed[event] else None,
            "elapsed_ms_status": "available" if lifecycle_elapsed[event] else "unknown",
        }
        for event, count in lifecycle_counts.items()
    }
    return {
        "input_bytes": input_bytes,
        "sample_count": sample_count,
        "lifecycle_sample_count": lifecycle_count,
        "metrics": metrics,
        "counters": counters,
        "lifecycle": lifecycle,
        "raw_samples_sha256": sample_hash.hexdigest(),
    }


def _unknown(reason: str) -> dict[str, object]:
    return {"status": "unknown", "value": None, "reason": reason}


def _safe_text(value: object, maximum: int = 256) -> str | None:
    if not isinstance(value, str) or len(value) > maximum or any(ord(char) < 32 for char in value):
        return None
    return value


def _safe_integer(value: object, *, minimum: int = 0, maximum: int = (1 << 63) - 1) -> int | None:
    if isinstance(value, bool) or not isinstance(value, int) or value < minimum or value > maximum:
        return None
    return value


def _inventory_reason(value: object) -> str | None:
    if value is None:
        return None
    return value if isinstance(value, str) and value in INVENTORY_REASONS else "unrecognized_reason"


def _inventory_field(raw: object, method: str, clean_value) -> dict[str, object]:
    if not isinstance(raw, dict) or raw.get("status") not in ("available", "unknown"):
        return {"status": "unknown", "value": None, "method": method, "reason": "field_shape_unrecognized"}
    if raw.get("method") != method:
        return {"status": "unknown", "value": None, "method": method, "reason": "collection_method_unrecognized"}
    if raw["status"] == "unknown":
        return {"status": "unknown", "value": None, "method": method,
                "reason": _inventory_reason(raw.get("reason")) or "collection_failed"}
    cleaned = clean_value(raw.get("value"))
    if cleaned is None:
        return {"status": "unknown", "value": None, "method": method, "reason": "field_value_unrecognized"}
    return {"status": "available", "value": cleaned, "method": method, "reason": None}


def _sanitize_inventory(value: object) -> dict[str, object] | None:
    if not isinstance(value, dict) or value.get("schema") != "star.ui-benchmark-host-inventory" or value.get("version") != "0.1.0":
        return None
    captured_at = _safe_text(value.get("captured_at"), 40)
    if captured_at is None or not re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z", captured_at):
        return None
    methods = value.get("collection_method")
    if not isinstance(methods, list) or any(method not in INVENTORY_METHODS for method in methods):
        return None

    def clean_os(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        fields = {name: _safe_text(raw.get(name), 256) for name in ("caption", "version", "build", "architecture")}
        return fields if all(item is not None for item in fields.values()) else None

    def clean_cpu_item(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        name = _safe_text(raw.get("name"), 256)
        cores = _safe_integer(raw.get("cores"), maximum=100_000)
        logical = _safe_integer(raw.get("logical_processors"), maximum=100_000)
        return {"name": name, "cores": cores, "logical_processors": logical} if name and cores is not None and logical is not None else None

    def clean_cpu(raw: object) -> object:
        items = raw if isinstance(raw, list) else [raw]
        cleaned = [clean_cpu_item(item) for item in items]
        return cleaned if cleaned and all(item is not None for item in cleaned) else None

    def clean_memory(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        amount = _safe_integer(raw.get("physical_bytes"), minimum=1)
        return {"physical_bytes": amount} if amount is not None else None

    def clean_gpu_item(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        name = _safe_text(raw.get("name"), 256)
        driver = _safe_text(raw.get("driver_version"), 64)
        adapter_ram = _safe_integer(raw.get("adapter_ram_cim_reported_bytes"), maximum=(1 << 32) - 1)
        if name is None or driver is None or adapter_ram is None:
            return None
        return {"name": name, "driver_version": driver, "adapter_ram_cim_reported_bytes": adapter_ram}

    def clean_gpu(raw: object) -> object:
        items = raw if isinstance(raw, list) else [raw]
        cleaned = [clean_gpu_item(item) for item in items]
        return cleaned if cleaned and all(item is not None for item in cleaned) else None

    def clean_power(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        guid = _safe_text(raw.get("guid"), 36)
        name = _safe_text(raw.get("name"), 128)
        if guid is None or name is None or not re.fullmatch(r"[0-9a-fA-F-]{36}", guid):
            return None
        return {"guid": guid.lower(), "name": name}

    def clean_refresh(raw: object) -> dict[str, object]:
        return _inventory_field(raw, "user32.EnumDisplaySettings", lambda number: _safe_integer(number, minimum=1, maximum=1_000))

    def clean_dpi_value(raw: object) -> object:
        if not isinstance(raw, dict):
            return None
        x = _safe_integer(raw.get("x"), minimum=1, maximum=1_000)
        y = _safe_integer(raw.get("y"), minimum=1, maximum=1_000)
        return {"x": x, "y": y} if x is not None and y is not None else None

    def clean_dpi(raw: object) -> dict[str, object]:
        return _inventory_field(raw, "Shcore.GetDpiForMonitor", clean_dpi_value)

    displays_raw = value.get("displays")
    if not isinstance(displays_raw, dict) or displays_raw.get("method") != "System.Windows.Forms + Win32 display APIs":
        return None
    display_status = displays_raw.get("status")
    display_reason = _inventory_reason(displays_raw.get("reason"))
    clean_displays: list[dict[str, object]] = []
    if display_status == "available":
        raw_items = displays_raw.get("displays")
        if not isinstance(raw_items, list):
            return None
        for item in raw_items:
            if not isinstance(item, dict):
                return None
            device = _safe_text(item.get("device_name"), 64)
            if device is None or not re.fullmatch(r"\\\\\.\\DISPLAY\d+", device) or not isinstance(item.get("primary"), bool):
                return None
            resolution = item.get("resolution")
            if not isinstance(resolution, dict) or resolution.get("method") != "System.Windows.Forms.Screen.Bounds":
                return None
            width = _safe_integer(resolution.get("width"), minimum=1, maximum=100_000)
            height = _safe_integer(resolution.get("height"), minimum=1, maximum=100_000)
            if width is None or height is None:
                return None
            clean_displays.append({
                "device_name": device,
                "primary": item["primary"],
                "resolution": {"width": width, "height": height, "method": "System.Windows.Forms.Screen.Bounds"},
                "refresh_hz": clean_refresh(item.get("refresh_hz")),
                "dpi": clean_dpi(item.get("dpi")),
            })
        if not clean_displays:
            return None
    elif display_status != "unknown":
        return None
    elif display_reason is None:
        display_reason = "display_api_unavailable"
    safe_displays = {
        "status": display_status,
        "method": "System.Windows.Forms + Win32 display APIs",
        "reason": display_reason,
        "displays": clean_displays,
    }

    os_info = _inventory_field(value.get("os"), "Get-CimInstance Win32_OperatingSystem", clean_os)
    cpu_info = _inventory_field(value.get("cpu"), "Get-CimInstance Win32_Processor", clean_cpu)
    memory_info = _inventory_field(value.get("memory"), "Get-CimInstance Win32_ComputerSystem", clean_memory)
    gpu_info = _inventory_field(value.get("gpu"), "Get-CimInstance Win32_VideoController", clean_gpu)
    power_info = _inventory_field(value.get("active_power_scheme"), "powercfg.exe /getactivescheme", clean_power)
    return {
        "schema": "star.ui-benchmark-host-inventory",
        "version": "0.1.0",
        "captured_at": captured_at,
        "collection_method": methods,
        "os": os_info,
        "cpu": cpu_info,
        "memory": memory_info,
        "gpu": gpu_info,
        "active_power_scheme": power_info,
        "displays": safe_displays,
    }


def _inventory(path: Path | None) -> dict[str, object]:
    if path is None:
        return _unknown("not_supplied")
    try:
        with path.open("rb") as source:
            raw = source.read(MAX_INVENTORY_BYTES + 1)
    except FileNotFoundError:
        return _unknown("inventory_file_not_found")
    except OSError:
        return _unknown("inventory_file_unreadable")
    if len(raw) > MAX_INVENTORY_BYTES:
        return _unknown("inventory_file_exceeds_size_limit")
    try:
        value = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return _unknown("inventory_file_invalid_json")
    sanitized = _sanitize_inventory(value)
    if sanitized is None:
        return _unknown("inventory_schema_unrecognized")
    return {"status": "available", "value": sanitized, "source_sha256": hashlib.sha256(raw).hexdigest(), "reason": None}


def _sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        while chunk := source.read(64 * 1024):
            digest.update(chunk)
    return digest.hexdigest()


def _open_regular_sample_file(path: Path) -> BinaryIO:
    try:
        metadata = path.stat()
    except OSError:
        raise
    if not stat.S_ISREG(metadata.st_mode):
        raise SampleError("driver samples path must refer to a regular file")

    flags = os.O_RDONLY | getattr(os, "O_BINARY", 0) | getattr(os, "O_NONBLOCK", 0)
    descriptor = os.open(path, flags)
    try:
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise SampleError("driver samples path must refer to a regular file")
        return os.fdopen(descriptor, "rb")
    except BaseException:
        os.close(descriptor)
        raise


def _metadata(value: str | None) -> dict[str, object]:
    if value is None:
        return _unknown("not_provided")
    return {"status": "available", "value": value, "reason": None}


def _git_commit(repo_root: Path) -> dict[str, object]:
    try:
        result = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=repo_root, check=True, capture_output=True,
            text=True, timeout=5,
        )
    except (OSError, subprocess.SubprocessError):
        return _unknown("git_commit_unavailable")
    value = result.stdout.strip()
    if not re.fullmatch(r"[0-9a-fA-F]{40,64}", value):
        return _unknown("git_commit_output_unrecognized")
    return {"status": "available", "value": value.lower(), "reason": None}


def run_measurement(
    manifest_path: Path,
    output_dir: Path,
    *,
    track: str | None = None,
    renderer: str | None = None,
    runtime: str | None = None,
    toolchain: str | None = None,
    build_mode: str | None = None,
    samples_path: Path | None = None,
    inventory_path: Path | None = None,
    repo_root: Path | None = None,
    run_id: str | None = None,
    observed_at: str | None = None,
) -> dict[str, object]:
    if track is not None and track not in TRACKS:
        raise ValueError(f"track must be one of: {', '.join(TRACKS)}")
    if samples_path is not None and track is None:
        raise ValueError("--track is required when --samples is supplied")
    manifest, manifest_sha256, seed = load_manifest(manifest_path)
    output_dir.mkdir(parents=True, exist_ok=False)
    raw_sample_path = output_dir / "raw_samples.jsonl"
    with raw_sample_path.open("x", encoding="utf-8", newline="\n") as raw_output:
        if track is None:
            sample_summary = _empty_sample_summary("track_not_selected")
        elif samples_path is None:
            sample_summary = _empty_sample_summary("driver_samples_not_supplied")
        else:
            try:
                with _open_regular_sample_file(samples_path) as source:
                    sample_summary = read_driver_samples(source, raw_output)
            except FileNotFoundError:
                sample_summary = _empty_sample_summary("driver_sample_file_not_found")
            except OSError:
                sample_summary = _empty_sample_summary("driver_sample_file_unreadable")
    sample_summary["raw_samples_file"] = raw_sample_path.name
    sample_summary["raw_samples_file_sha256"] = _sha256_file(raw_sample_path)

    if track is None:
        measurement_status = "unavailable"
        status_reason = "track_not_selected"
    elif samples_path is None or sample_summary["sample_count"] == 0:
        measurement_status = "not_run"
        status_reason = sample_summary.get("reason", "no_driver_measurement_samples")
    else:
        measurement_status = "available"
        status_reason = None
    if observed_at is None:
        observed_at = datetime.now(timezone.utc).isoformat(timespec="milliseconds").replace("+00:00", "Z")
    if run_id is None:
        run_id = str(uuid.uuid4())

    result: dict[str, object] = {
        "schema": "star.ui-benchmark-measurement",
        "version": "0.1.0",
        "runner_version": RUNNER_VERSION,
        "run_id": run_id,
        "observed_at": observed_at,
        "measurement_status": measurement_status,
        "measurement_status_reason": status_reason,
        "profile_id": manifest["profile_id"],
        "manifest_sha256": manifest_sha256,
        "seed": seed,
        "git_commit": _git_commit(repo_root or Path(__file__).resolve().parents[2]),
        "track": _metadata(track),
        "renderer": _metadata(renderer),
        "runtime": _metadata(runtime),
        "toolchain": _metadata(toolchain),
        "build_mode": _metadata(build_mode),
        "host_inventory": _inventory(inventory_path),
        "cold_warm_state": {"status": "available" if sample_summary["sample_count"] else "unknown",
                             "value": "recorded_per_sample" if sample_summary["sample_count"] else None,
                             "reason": None if sample_summary["sample_count"] else "no_driver_measurement_samples"},
        "sample_count": sample_summary["sample_count"],
        "driver_input_bytes": sample_summary["input_bytes"],
        "raw_samples": {
            "file": sample_summary["raw_samples_file"],
            "count": sample_summary["sample_count"] + sample_summary["lifecycle_sample_count"],
            "sha256": sample_summary["raw_samples_file_sha256"],
        },
        "metric_summaries": sample_summary["metrics"],
        "counters": sample_summary["counters"],
        "lifecycle_samples": {
            "count": sample_summary["lifecycle_sample_count"],
            "events": sample_summary["lifecycle"],
        },
        "performance_thresholds": "OPEN",
        "renderer_choice": "OPEN",
    }
    result_path = output_dir / "measurement.json"
    encoded_result = json.dumps(result, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    result_path.write_text(encoded_result, encoding="utf-8", newline="\n")
    return result


def _empty_sample_summary(reason: str) -> dict[str, object]:
    return {
        "reason": reason,
        "input_bytes": 0,
        "sample_count": 0,
        "lifecycle_sample_count": 0,
        "metrics": {},
        "counters": {
            name: {"status": "unknown", "reason": reason, "unit": None, "conditions": {}}
            for name in COUNTER_NAMES
        },
        "lifecycle": {
            event: {"status": "unknown", "reason": reason, "count": 0,
                    "elapsed_ms": None, "elapsed_ms_status": "unknown"}
            for event in LIFECYCLE_EVENTS
        },
        "raw_samples_sha256": hashlib.sha256(b"").hexdigest(),
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Record one UI-0 measurement run without fabricating driver samples.")
    parser.add_argument("--manifest", required=True, type=Path)
    parser.add_argument("--output-dir", required=True, type=Path)
    parser.add_argument("--track", choices=TRACKS)
    parser.add_argument("--renderer")
    parser.add_argument("--runtime")
    parser.add_argument("--toolchain")
    parser.add_argument("--build-mode", choices=("debug", "release"))
    parser.add_argument("--samples", type=Path, help="JSONL output from a real UI driver")
    parser.add_argument("--inventory-json", type=Path, help="output from capture_windows_inventory.ps1")
    parser.add_argument("--repo-root", type=Path, default=Path(__file__).resolve().parents[2])
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        result = run_measurement(
            args.manifest,
            args.output_dir,
            track=args.track,
            renderer=args.renderer,
            runtime=args.runtime,
            toolchain=args.toolchain,
            build_mode=args.build_mode,
            samples_path=args.samples,
            inventory_path=args.inventory_json,
            repo_root=args.repo_root,
        )
    except (OSError, ManifestError, SampleError, ValueError) as exc:
        print(f"ui0 measurement failed: {exc}", file=sys.stderr)
        return 2
    print(json.dumps({"run_id": result["run_id"], "measurement_status": result["measurement_status"],
                      "output_dir": str(args.output_dir), "sample_count": result["sample_count"]},
                     sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
