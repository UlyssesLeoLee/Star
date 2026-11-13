# @cypher schema=1 source_sha256=c9c49a08403246ba516d0e906d0508f1db820bc4c5209b7257d26347e224e246
# MERGE (self:File {path:"scripts/ui0_bench/generate_fixture.py"})
# MERGE (manifest_error:Symbol {id:"scripts/ui0_bench/generate_fixture.py::ManifestError", kind:"class"})
# MERGE (splitmix64:Symbol {id:"scripts/ui0_bench/generate_fixture.py::SplitMix64", kind:"class"})
# MERGE (validate_manifest:Symbol {id:"scripts/ui0_bench/generate_fixture.py::validate_manifest", kind:"function"})
# MERGE (load_manifest:Symbol {id:"scripts/ui0_bench/generate_fixture.py::load_manifest", kind:"function"})
# MERGE (terminal_pattern:Symbol {id:"scripts/ui0_bench/generate_fixture.py::_terminal_pattern", kind:"function"})
# MERGE (terminal_chunks:Symbol {id:"scripts/ui0_bench/generate_fixture.py::_terminal_chunks", kind:"function"})
# MERGE (iter_fixture_records:Symbol {id:"scripts/ui0_bench/generate_fixture.py::iter_fixture_records", kind:"function"})
# MERGE (write_jsonl:Symbol {id:"scripts/ui0_bench/generate_fixture.py::write_jsonl", kind:"function"})
# MERGE (main:Symbol {id:"scripts/ui0_bench/generate_fixture.py::main", kind:"function"})
# MERGE (self)-[:DEFINES]->(manifest_error)
# MERGE (self)-[:DEFINES]->(splitmix64)
# MERGE (self)-[:DEFINES]->(validate_manifest)
# MERGE (self)-[:DEFINES]->(load_manifest)
# MERGE (self)-[:DEFINES]->(terminal_pattern)
# MERGE (self)-[:DEFINES]->(terminal_chunks)
# MERGE (self)-[:DEFINES]->(iter_fixture_records)
# MERGE (self)-[:DEFINES]->(write_jsonl)
# MERGE (self)-[:DEFINES]->(main)
# MERGE (load_manifest)-[:CALLS]->(validate_manifest)
# MERGE (terminal_pattern)-[:USES_TYPE]->(splitmix64)
# MERGE (iter_fixture_records)-[:CALLS]->(validate_manifest)
# MERGE (iter_fixture_records)-[:CALLS]->(terminal_pattern)
# MERGE (iter_fixture_records)-[:CALLS]->(terminal_chunks)
# MERGE (write_jsonl)-[:CALLS]->(iter_fixture_records)
# MERGE (main)-[:CALLS]->(load_manifest)
# MERGE (main)-[:CALLS]->(write_jsonl)
# @endcypher

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import math
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import BinaryIO, Iterator


GENERATOR_VERSION = "1.0.0"
PRNG_ALGORITHM = "splitmix64-v1"
MAX_MANIFEST_BYTES = 1_048_576
MAX_WORKTREES = 100_000
MAX_TERMINAL_BYTES_PER_STREAM = 16 * 1_048_576
MAX_TOTAL_TERMINAL_BYTES = 256 * 1_048_576
MAX_CANVAS_NODES = 20_000
MAX_CANVAS_EDGES = 100_000
MAX_BI_EVENTS = 100_000
MAX_PARENT_SCOPES = 10_000
MAX_INTERACTION_CYCLES = 10_000
MAX_MEASUREMENT_REPETITIONS = 1_000
MAX_RECORDS = 2_000_000
TERMINAL_CHUNK_BYTES = 3_072
MASK_64 = (1 << 64) - 1
SUPPORTED_STREAM_MODES = (
    "utf8_ansi_burst",
    "utf8_ansi_paced",
    "long_line",
    "reconnect_snapshot_with_gap",
    "cancel_and_drain",
)
SUPPORTED_CANVAS_OPERATIONS = (
    "initial_viewport_load",
    "pan",
    "zoom",
    "drag",
    "cancel_viewport_load",
)


class ManifestError(ValueError):
    """Raised when a fixture manifest is unsupported or exceeds safety limits."""


class SplitMix64:
    """Small, explicitly versioned deterministic PRNG used by this fixture format."""

    def __init__(self, seed: int) -> None:
        self.state = seed & MASK_64

    def next_u64(self) -> int:
        self.state = (self.state + 0x9E3779B97F4A7C15) & MASK_64
        value = self.state
        value = ((value ^ (value >> 30)) * 0xBF58476D1CE4E5B9) & MASK_64
        value = ((value ^ (value >> 27)) * 0x94D049BB133111EB) & MASK_64
        return value ^ (value >> 31)


def _integer(value: object, name: str, *, minimum: int = 0, maximum: int) -> int:
    if isinstance(value, bool) or not isinstance(value, int):
        raise ManifestError(f"{name} must be an integer")
    if value < minimum or value > maximum:
        raise ManifestError(f"{name} must be between {minimum} and {maximum}")
    return value


def _mapping(value: object, name: str) -> dict[str, object]:
    if not isinstance(value, dict) or not all(isinstance(key, str) for key in value):
        raise ManifestError(f"{name} must be an object with string keys")
    return value


def _required_mapping(parent: dict[str, object], key: str) -> dict[str, object]:
    if key not in parent:
        raise ManifestError(f"missing {key}")
    return _mapping(parent[key], key)


def validate_manifest(manifest: object) -> dict[str, object]:
    root = _mapping(manifest, "manifest")
    if root.get("schema") != "star.ui-benchmark-fixture":
        raise ManifestError("unsupported manifest schema")
    if root.get("version") != "0.1.0":
        raise ManifestError("only fixture manifest version 0.1.0 is supported")
    if not isinstance(root.get("profile_id"), str) or not re.fullmatch(r"[A-Za-z0-9._-]{1,64}", root["profile_id"]):
        raise ManifestError("profile_id must use the bounded identifier format")
    try:
        datetime.strptime(str(root.get("captured_date")), "%Y-%m-%d")
    except ValueError as exc:
        raise ManifestError("captured_date must use YYYY-MM-DD") from exc
    _integer(root.get("seed"), "seed", maximum=MASK_64)

    directory = _required_mapping(root, "directory")
    projects = _integer(directory.get("projects"), "directory.projects", minimum=1, maximum=1_000)
    runs_per_project = _integer(
        directory.get("runs_per_project"), "directory.runs_per_project", minimum=1, maximum=1_000
    )
    worktrees_per_run = _integer(
        directory.get("worktrees_per_run"), "directory.worktrees_per_run", maximum=MAX_WORKTREES
    )
    total_worktrees = projects * runs_per_project * worktrees_per_run
    if total_worktrees > MAX_WORKTREES or directory.get("worktrees_total") != total_worktrees:
        raise ManifestError("directory.worktrees_total must match the bounded project/run/worktree product")
    if directory.get("parent_scopes") != projects * runs_per_project:
        raise ManifestError("directory.parent_scopes must match projects times runs_per_project")
    for field in ("page_size", "available_pages_per_run", "picker_retained_page_limit", "parent_cache_boundary"):
        _integer(directory.get(field), f"directory.{field}", minimum=1, maximum=100_000)
    flags = _required_mapping(directory, "state_flag_rules")
    if isinstance(flags.get("index_base"), bool) or flags.get("index_base") not in (0, 1):
        raise ManifestError("directory.state_flag_rules.index_base must be 0 or 1")
    if not isinstance(flags.get("flags_overlap"), bool):
        raise ManifestError("directory.state_flag_rules.flags_overlap must be boolean")
    for field in ("busy_every_nth_worktree", "stale_every_nth_worktree", "dirty_every_nth_worktree",
                  "locked_every_nth_worktree", "unknown_every_nth_worktree"):
        _integer(flags.get(field), f"directory.state_flag_rules.{field}", minimum=1, maximum=MAX_WORKTREES)

    concurrency = _required_mapping(root, "concurrency")
    total_runs = projects * runs_per_project
    if total_runs > MAX_PARENT_SCOPES:
        raise ManifestError(f"total parent scopes exceed the explicit bound of {MAX_PARENT_SCOPES}")
    active_runs = _integer(concurrency.get("active_runs"), "concurrency.active_runs", maximum=total_runs)
    agents_per_run = _integer(
        concurrency.get("agents_per_active_run"), "concurrency.agents_per_active_run", maximum=10_000
    )
    _integer(concurrency.get("open_terminal_panes"), "concurrency.open_terminal_panes", maximum=10_000)
    _integer(concurrency.get("agents_total"), "concurrency.agents_total", maximum=100_000)
    if concurrency.get("agents_total") != active_runs * agents_per_run:
        raise ManifestError("concurrency.agents_total must match active_runs times agents_per_active_run")

    terminal = _required_mapping(root, "tasks_and_terminal")
    _integer(terminal.get("tasks_per_active_run"), "tasks_and_terminal.tasks_per_active_run", maximum=100_000)
    stream_bytes = _integer(
        terminal.get("output_bytes_per_pane"), "tasks_and_terminal.output_bytes_per_pane",
        maximum=MAX_TERMINAL_BYTES_PER_STREAM,
    )
    modes = terminal.get("stream_modes")
    if not isinstance(modes, list) or not modes or any(mode not in SUPPORTED_STREAM_MODES for mode in modes):
        raise ManifestError("tasks_and_terminal.stream_modes contains an unsupported mode")
    total_terminal_bytes = stream_bytes * concurrency["open_terminal_panes"] * len(modes)
    if total_terminal_bytes > MAX_TOTAL_TERMINAL_BYTES:
        raise ManifestError("total terminal fixture bytes exceed the explicit output bound")

    canvas = _required_mapping(root, "canvas")
    node_counts = canvas.get("node_counts")
    if not isinstance(node_counts, list) or not node_counts:
        raise ManifestError("canvas.node_counts must be a non-empty array")
    total_nodes = 0
    for index, value in enumerate(node_counts):
        total_nodes += _integer(value, f"canvas.node_counts[{index}]", minimum=1, maximum=MAX_CANVAS_NODES)
    if total_nodes > MAX_CANVAS_NODES * 3:
        raise ManifestError("total canvas nodes exceed the explicit output bound")
    edge_rates = _required_mapping(canvas, "edges_per_node")
    total_edges = 0
    for density in ("sparse", "dense"):
        ratio = edge_rates.get(density)
        if isinstance(ratio, bool) or not isinstance(ratio, (int, float)) or not math.isfinite(ratio):
            raise ManifestError(f"canvas.edges_per_node.{density} must be a finite number")
        if ratio < 0 or ratio > 4:
            raise ManifestError(f"canvas.edges_per_node.{density} must be between 0 and 4")
        total_edges += sum(int(count * ratio) for count in node_counts)
    if total_edges > MAX_CANVAS_EDGES:
        raise ManifestError("total canvas edges exceed the explicit output bound")
    operations = canvas.get("operations")
    if not isinstance(operations, list) or any(item not in SUPPORTED_CANVAS_OPERATIONS for item in operations):
        raise ManifestError("canvas.operations contains an unsupported operation")

    bi = _required_mapping(root, "bi")
    events_per_page = _integer(bi.get("events_per_page"), "bi.events_per_page", maximum=MAX_BI_EVENTS)
    pages = _integer(bi.get("pages"), "bi.pages", maximum=MAX_BI_EVENTS)
    events_total = _integer(bi.get("events_total"), "bi.events_total", maximum=MAX_BI_EVENTS)
    if events_per_page * pages != events_total:
        raise ManifestError("bi.events_total must match events_per_page times pages")
    if not isinstance(bi.get("runs_while_ui_is_active"), bool):
        raise ManifestError("bi.runs_while_ui_is_active must be boolean")

    sequence = _required_mapping(root, "interaction_sequence")
    _integer(sequence.get("focus_switch_cycles"), "interaction_sequence.focus_switch_cycles",
             maximum=MAX_INTERACTION_CYCLES)
    _integer(sequence.get("switch_across_parent_scopes"), "interaction_sequence.switch_across_parent_scopes",
             maximum=total_runs)
    for field in (
        "directory_scroll_and_keyboard_navigation",
        "overlap_canvas_and_bi_background_work",
        "cancel_old_generation_and_wait_for_drain",
    ):
        if not isinstance(sequence.get(field), bool):
            raise ManifestError(f"interaction_sequence.{field} must be boolean")
    repetitions = _required_mapping(root, "measurement_repetitions")
    _integer(repetitions.get("cold_start_runs"), "measurement_repetitions.cold_start_runs",
             maximum=MAX_MEASUREMENT_REPETITIONS)
    _integer(repetitions.get("warm_interaction_runs"), "measurement_repetitions.warm_interaction_runs",
             maximum=MAX_MEASUREMENT_REPETITIONS)
    percentiles = repetitions.get("report_percentiles")
    if percentiles != [50, 95]:
        raise ManifestError("measurement_repetitions.report_percentiles must be [50, 95]")

    terminal_stream_count = concurrency["open_terminal_panes"] * len(modes)
    terminal_chunks = (stream_bytes + TERMINAL_CHUNK_BYTES - 1) // TERMINAL_CHUNK_BYTES
    interaction_extras = sum(
        bool(sequence.get(field))
        for field in (
            "directory_scroll_and_keyboard_navigation",
            "overlap_canvas_and_bi_background_work",
            "cancel_old_generation_and_wait_for_drain",
        )
    )
    estimated_records = (
        1 + projects + total_runs + total_worktrees + active_runs * terminal["tasks_per_active_run"]
        + concurrency["agents_total"] + terminal_stream_count * (terminal_chunks + 2)
        + len(canvas["node_counts"]) * 3 + total_nodes + total_edges + events_total
        + sequence["focus_switch_cycles"] + interaction_extras + 1
    )
    if estimated_records > MAX_RECORDS:
        raise ManifestError("estimated JSONL record count exceeds the explicit output bound")
    return root


def load_manifest(path: Path, seed_override: int | None = None) -> tuple[dict[str, object], str, int]:
    with path.open("rb") as source:
        raw = source.read(MAX_MANIFEST_BYTES + 1)
    if len(raw) > MAX_MANIFEST_BYTES:
        raise ManifestError(f"manifest exceeds {MAX_MANIFEST_BYTES} bytes")
    try:
        manifest = json.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise ManifestError("manifest must be valid UTF-8 JSON") from exc
    manifest = validate_manifest(manifest)
    seed = manifest["seed"] if seed_override is None else _integer(seed_override, "seed override", maximum=MASK_64)
    return manifest, hashlib.sha256(raw).hexdigest(), seed


def _timestamp(captured_date: str, offset_ms: int) -> str:
    start = datetime.strptime(captured_date, "%Y-%m-%d").replace(tzinfo=timezone.utc)
    return (start + timedelta(milliseconds=offset_ms)).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def _scope_id(seed: int, kind: str, first: int, second: int = 0, third: int = 0) -> str:
    return f"ui0-{seed:016x}-{kind}-{first:04d}-{second:04d}-{third:05d}"


def _terminal_pattern(mode: str, pane_index: int, mode_index: int, seed: int) -> bytes:
    prng = SplitMix64(seed ^ ((pane_index + 1) << 32) ^ (mode_index + 1))
    tag = f"{prng.next_u64():016x}"
    prefix = f"pane={pane_index + 1} mode={mode} seed_tag={tag} ".encode("utf-8")
    content = {
        "utf8_ansi_burst": "\x1b[32m✓ burst output λ\x1b[0m\r\n",
        "utf8_ansi_paced": "\x1b[36m✓ paced output 漢字\x1b[0m\r\n",
        "long_line": "long-line-without-newline-",
        "reconnect_snapshot_with_gap": "snapshot=fixture sequence-gap=fixture resumed=fixture\r\n",
        "cancel_and_drain": "cancel-requested=fixture drain-record=fixture\r\n",
    }[mode].encode("utf-8")
    return prefix + content


def _terminal_chunks(pattern: bytes, byte_count: int, chunk_bytes: int = TERMINAL_CHUNK_BYTES) -> Iterator[tuple[int, bytes]]:
    if not pattern:
        raise ValueError("terminal pattern cannot be empty")
    if chunk_bytes <= 0 or chunk_bytes > TERMINAL_CHUNK_BYTES:
        raise ValueError(f"chunk_bytes must be between 1 and {TERMINAL_CHUNK_BYTES}")
    offset = 0
    while offset < byte_count:
        size = min(chunk_bytes, byte_count - offset)
        start = offset % len(pattern)
        repeats = (start + size + len(pattern) - 1) // len(pattern) + 1
        payload = (pattern * repeats)[start : start + size]
        yield offset, payload
        offset += size


def iter_fixture_records(manifest: dict[str, object], manifest_sha256: str, seed: int) -> Iterator[dict[str, object]]:
    root = validate_manifest(manifest)
    seed = _integer(seed, "seed", maximum=MASK_64)
    captured_date = root["captured_date"]
    directory = root["directory"]
    projects = directory["projects"]
    runs_per_project = directory["runs_per_project"]
    worktrees_per_run = directory["worktrees_per_run"]
    concurrency = root["concurrency"]
    terminal = root["tasks_and_terminal"]
    canvas = root["canvas"]
    edge_rates = canvas["edges_per_node"]
    bi = root["bi"]
    sequence = root["interaction_sequence"]
    flag_rules = directory["state_flag_rules"]
    index_base = flag_rules["index_base"]
    rng = SplitMix64(seed)

    yield {
        "type": "fixture",
        "schema": root["schema"],
        "version": root["version"],
        "profile_id": root["profile_id"],
        "generator_version": GENERATOR_VERSION,
        "prng_algorithm": PRNG_ALGORITHM,
        "seed": seed,
        "manifest_sha256": manifest_sha256,
        "captured_at": _timestamp(captured_date, 0),
        "measurement_only": True,
        "authorization_evidence": False,
        "production_data": False,
        "profile_parameters": {
            "directory": {
                "projects": directory["projects"],
                "runs_per_project": directory["runs_per_project"],
                "parent_scopes": directory["parent_scopes"],
                "worktrees_per_run": directory["worktrees_per_run"],
                "page_size": directory["page_size"],
                "available_pages_per_run": directory["available_pages_per_run"],
                "picker_retained_page_limit": directory["picker_retained_page_limit"],
                "parent_cache_boundary": directory["parent_cache_boundary"],
                "state_flag_rules": {
                    "busy_every_nth_worktree": flag_rules["busy_every_nth_worktree"],
                    "stale_every_nth_worktree": flag_rules["stale_every_nth_worktree"],
                    "dirty_every_nth_worktree": flag_rules["dirty_every_nth_worktree"],
                    "locked_every_nth_worktree": flag_rules["locked_every_nth_worktree"],
                    "unknown_every_nth_worktree": flag_rules["unknown_every_nth_worktree"],
                    "flags_overlap": flag_rules["flags_overlap"],
                    "index_base": index_base,
                },
            },
            "concurrency": {
                "active_runs": concurrency["active_runs"],
                "agents_per_active_run": concurrency["agents_per_active_run"],
                "agents_total": concurrency["agents_total"],
                "open_terminal_panes": concurrency["open_terminal_panes"],
            },
            "tasks_and_terminal": {
                "tasks_per_active_run": terminal["tasks_per_active_run"],
                "output_bytes_per_pane": terminal["output_bytes_per_pane"],
                "stream_modes": terminal["stream_modes"],
            },
            "canvas": {
                "node_counts": canvas["node_counts"],
                "edges_per_node": {
                    "sparse": edge_rates["sparse"],
                    "dense": edge_rates["dense"],
                },
                "operations": canvas["operations"],
            },
            "bi": {
                "events_per_page": bi["events_per_page"],
                "pages": bi["pages"],
                "events_total": bi["events_total"],
                "runs_while_ui_is_active": bi["runs_while_ui_is_active"],
            },
            "interaction_sequence": {
                "focus_switch_cycles": sequence["focus_switch_cycles"],
                "switch_across_parent_scopes": sequence["switch_across_parent_scopes"],
                "directory_scroll_and_keyboard_navigation": sequence["directory_scroll_and_keyboard_navigation"],
                "overlap_canvas_and_bi_background_work": sequence["overlap_canvas_and_bi_background_work"],
                "cancel_old_generation_and_wait_for_drain": sequence["cancel_old_generation_and_wait_for_drain"],
            },
            "measurement_repetitions": {
                "cold_start_runs": root["measurement_repetitions"]["cold_start_runs"],
                "warm_interaction_runs": root["measurement_repetitions"]["warm_interaction_runs"],
                "report_percentiles": root["measurement_repetitions"]["report_percentiles"],
            },
        },
    }

    run_ids: list[str] = []
    ordinal = 0
    for project_index in range(1, projects + 1):
        project_id = _scope_id(seed, "project", project_index)
        yield {
            "type": "project",
            "project_id": project_id,
            "ordinal": project_index - 1,
            "display_name": f"Fixture Project {project_index:04d}",
            "generated_value": f"{rng.next_u64():016x}",
            "created_at": _timestamp(captured_date, project_index),
        }
        for run_index in range(1, runs_per_project + 1):
            run_id = _scope_id(seed, "run", project_index, run_index)
            run_ids.append(run_id)
            yield {
                "type": "run",
                "run_id": run_id,
                "project_id": project_id,
                "ordinal": len(run_ids) - 1,
                "display_name": f"Fixture Run {project_index:04d}-{run_index:04d}",
                "generated_value": f"{rng.next_u64():016x}",
                "created_at": _timestamp(captured_date, len(run_ids) + projects),
            }
            for worktree_index in range(worktrees_per_run):
                logical_index = worktree_index + index_base
                flags = {
                    name: logical_index % flag_rules[f"{name}_every_nth_worktree"] == 0
                    for name in ("busy", "stale", "dirty", "locked", "unknown")
                }
                yield {
                    "type": "worktree",
                    "worktree_id": _scope_id(seed, "worktree", project_index, run_index, worktree_index + 1),
                    "project_id": project_id,
                    "run_id": run_id,
                    "ordinal": ordinal,
                    "local_branch_id": f"local-ref-{project_index:04d}-{run_index:04d}-{worktree_index + 1:05d}",
                    "page_index": worktree_index // directory["page_size"],
                    "flags": flags,
                    "generated_value": f"{rng.next_u64():016x}",
                    "created_at": _timestamp(captured_date, ordinal + projects + len(run_ids)),
                }
                ordinal += 1

    active_run_ids = run_ids[: concurrency["active_runs"]]
    for run_index, run_id in enumerate(active_run_ids):
        for task_index in range(1, terminal["tasks_per_active_run"] + 1):
            yield {
                "type": "task",
                "task_id": _scope_id(seed, "task", run_index + 1, task_index),
                "run_id": run_id,
                "ordinal": task_index - 1,
                "status": "fixture_input",
                "generated_value": f"{rng.next_u64():016x}",
            }

    agents_per_run = concurrency["agents_per_active_run"]
    for agent_index in range(concurrency["agents_total"]):
        active_run_index = agent_index // agents_per_run if agents_per_run else 0
        yield {
            "type": "agent",
            "agent_id": _scope_id(seed, "agent", agent_index + 1),
            "run_id": active_run_ids[active_run_index],
            "ordinal": agent_index,
            "status": "fixture_input",
            "generated_value": f"{rng.next_u64():016x}",
        }

    stream_bytes = terminal["output_bytes_per_pane"]
    stream_modes = terminal["stream_modes"]
    for pane_index in range(concurrency["open_terminal_panes"]):
        run_id = active_run_ids[pane_index % len(active_run_ids)] if active_run_ids else None
        for mode_index, mode in enumerate(stream_modes):
            pattern = _terminal_pattern(mode, pane_index, mode_index, seed)
            stream_id = _scope_id(seed, "terminal", pane_index + 1, mode_index + 1)
            yield {
                "type": "terminal_stream_start",
                "stream_id": stream_id,
                "run_id": run_id,
                "pane_index": pane_index,
                "mode": mode,
                "byte_count": stream_bytes,
                "chunk_bytes_max": TERMINAL_CHUNK_BYTES,
                "pattern_sha256": hashlib.sha256(pattern).hexdigest(),
                "fixture_data_only": True,
            }
            stream_hash = hashlib.sha256()
            for offset, chunk in _terminal_chunks(pattern, stream_bytes):
                stream_hash.update(chunk)
                yield {
                    "type": "terminal_chunk",
                    "stream_id": stream_id,
                    "offset": offset,
                    "byte_count": len(chunk),
                    "data_base64": base64.b64encode(chunk).decode("ascii"),
                }
            yield {
                "type": "terminal_stream_end",
                "stream_id": stream_id,
                "byte_count": stream_bytes,
                "raw_sha256": stream_hash.hexdigest(),
            }

    for canvas_index, node_count in enumerate(canvas["node_counts"], start=1):
        canvas_id = _scope_id(seed, "canvas", canvas_index)
        yield {
            "type": "canvas_case",
            "canvas_id": canvas_id,
            "node_count": node_count,
            "operations": canvas["operations"],
            "measurement_only": True,
        }
        for node_index in range(node_count):
            yield {
                "type": "canvas_node",
                "canvas_id": canvas_id,
                "node_id": _scope_id(seed, "node", canvas_index, node_index + 1),
                "ordinal": node_index,
                "x": (rng.next_u64() % 20_001) - 10_000,
                "y": (rng.next_u64() % 20_001) - 10_000,
            }
        for density in ("sparse", "dense"):
            edge_count = int(node_count * canvas["edges_per_node"][density])
            edge_case_id = f"{canvas_id}-{density}"
            yield {
                "type": "canvas_edge_case",
                "canvas_id": canvas_id,
                "edge_case_id": edge_case_id,
                "density": density,
                "edge_count": edge_count,
            }
            for edge_index in range(edge_count):
                source = edge_index % node_count
                target = (edge_index * 37 + 1) % node_count
                if target == source:
                    target = (target + 1) % node_count
                yield {
                    "type": "canvas_edge",
                    "edge_case_id": edge_case_id,
                    "edge_id": _scope_id(seed, "edge", canvas_index, 1 if density == "sparse" else 2,
                                         edge_index + 1),
                    "source_ordinal": source,
                    "target_ordinal": target,
                }

    event_names = ("directory_observed", "task_observed", "canvas_viewport_observed", "terminal_chunk_observed")
    for event_index in range(bi["events_total"]):
        run_id = run_ids[event_index % len(run_ids)] if run_ids else None
        yield {
            "type": "bi_event",
            "event_id": _scope_id(seed, "bi", event_index + 1),
            "run_id": run_id,
            "ordinal": event_index,
            "event_name": event_names[event_index % len(event_names)],
            "occurred_at": _timestamp(captured_date, event_index + 1),
            "fixture_data_only": True,
        }

    parent_scope_count = len(run_ids)
    for cycle in range(sequence["focus_switch_cycles"]):
        from_scope = cycle % parent_scope_count if parent_scope_count else None
        to_scope = (cycle + 1) % parent_scope_count if parent_scope_count else None
        yield {
            "type": "interaction_step",
            "step_id": _scope_id(seed, "interaction", cycle + 1),
            "ordinal": cycle,
            "operation": "focus_switch",
            "from_run_id": run_ids[from_scope] if from_scope is not None else None,
            "to_run_id": run_ids[to_scope] if to_scope is not None else None,
            "expected_measurement": "driver_required",
        }
    for index, operation in enumerate(("directory_scroll_and_keyboard_navigation", "overlap_canvas_and_bi_background_work",
                                       "cancel_old_generation_and_wait_for_drain"), start=1):
        if sequence.get(operation):
            yield {
                "type": "interaction_step",
                "step_id": _scope_id(seed, "interaction-extra", index),
                "ordinal": sequence["focus_switch_cycles"] + index - 1,
                "operation": operation,
                "expected_measurement": "driver_required",
            }

    repetitions = root["measurement_repetitions"]
    yield {
        "type": "measurement_plan",
        "cold_start_runs": repetitions["cold_start_runs"],
        "warm_interaction_runs": repetitions["warm_interaction_runs"],
        "percentiles": repetitions["report_percentiles"],
        "performance_thresholds": "OPEN",
        "renderer_choice": "OPEN",
    }


def write_jsonl(stream: BinaryIO, manifest: dict[str, object], manifest_sha256: str, seed: int) -> tuple[int, int, str]:
    digest = hashlib.sha256()
    bytes_written = 0
    record_count = 0
    for record in iter_fixture_records(manifest, manifest_sha256, seed):
        line = json.dumps(record, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8") + b"\n"
        stream.write(line)
        digest.update(line)
        bytes_written += len(line)
        record_count += 1
        if record_count > MAX_RECORDS:
            raise ManifestError("generated JSONL record count exceeded the explicit bound")
    return record_count, bytes_written, digest.hexdigest()


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Generate a deterministic, bounded UI-0 fixture as JSONL.")
    parser.add_argument("--manifest", required=True, type=Path, help="canonical JSON fixture manifest")
    parser.add_argument("--output", required=True, type=Path, help="new JSONL output file; existing files are not overwritten")
    parser.add_argument("--seed", type=int, help="optional deterministic seed override")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        manifest, manifest_sha256, seed = load_manifest(args.manifest, args.seed)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        with args.output.open("xb") as output:
            record_count, bytes_written, fixture_sha256 = write_jsonl(output, manifest, manifest_sha256, seed)
    except (OSError, ManifestError) as exc:
        print(f"ui0 fixture generation failed: {exc}", file=sys.stderr)
        return 2
    summary = {
        "output": str(args.output),
        "record_count": record_count,
        "bytes_written": bytes_written,
        "manifest_sha256": manifest_sha256,
        "fixture_sha256": fixture_sha256,
        "generator_version": GENERATOR_VERSION,
        "seed": seed,
    }
    print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
