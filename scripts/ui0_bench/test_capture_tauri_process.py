# @cypher schema=1 source_sha256=fedb505ababb664bfee21a361ad645acdcd5188a40efd0ea1a248b3851e6ab10
# MERGE (self:File {path:"scripts/ui0_bench/test_capture_tauri_process.py"})
# MERGE (capture_tests:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests", kind:"class"})
# MERGE (test_tree_aggregation:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_tree_aggregation_and_child_changes", kind:"test"})
# MERGE (test_parent_pid_reuse:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_parent_pid_reuse_is_not_a_child_edge", kind:"test"})
# MERGE (test_first_cpu_sample:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_first_sample_omits_cpu_delta", kind:"test"})
# MERGE (test_cpu_normalization:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_cpu_is_normalized_to_total_logical_capacity", kind:"test"})
# MERGE (test_child_pid_reuse:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_reused_child_pid_has_no_cross_identity_delta", kind:"test"})
# MERGE (test_root_pid_reuse:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_reused_root_pid_fails_closed", kind:"test"})
# MERGE (test_snapshot_strict:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_snapshot_schema_rejects_extra_fields_and_invalid_counters", kind:"test"})
# MERGE (test_fixed_powershell:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_subprocess_uses_fixed_powershell_and_no_shell", kind:"test"})
# MERGE (test_pid_input_validation:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_root_pid_query_input_cannot_be_command_fragment", kind:"test"})
# MERGE (test_cim_error:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_cim_unavailable_is_structured_and_does_not_leak_stderr", kind:"test"})
# MERGE (test_property_unavailable:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_missing_process_properties_fail_closed", kind:"test"})
# MERGE (test_non_windows:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_non_windows_fails_before_creating_output", kind:"test"})
# MERGE (test_existing_output:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_existing_output_is_never_overwritten", kind:"test"})
# MERGE (test_midrun_exit:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_root_exit_midrun_removes_partial_output", kind:"test"})
# MERGE (test_sample_bounds:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_sample_limits_are_enforced", kind:"test"})
# MERGE (test_interval_floor:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_subsecond_interval_is_rejected", kind:"test"})
# MERGE (test_runner_contract:Symbol {id:"scripts/ui0_bench/test_capture_tauri_process.py::CaptureTests.test_samples_match_existing_runner_contract", kind:"test"})
# MERGE (self)-[:DEFINES]->(capture_tests)
# MERGE (self)-[:DEFINES]->(test_tree_aggregation)
# MERGE (self)-[:DEFINES]->(test_parent_pid_reuse)
# MERGE (self)-[:DEFINES]->(test_first_cpu_sample)
# MERGE (self)-[:DEFINES]->(test_cpu_normalization)
# MERGE (self)-[:DEFINES]->(test_child_pid_reuse)
# MERGE (self)-[:DEFINES]->(test_root_pid_reuse)
# MERGE (self)-[:DEFINES]->(test_snapshot_strict)
# MERGE (self)-[:DEFINES]->(test_fixed_powershell)
# MERGE (self)-[:DEFINES]->(test_pid_input_validation)
# MERGE (self)-[:DEFINES]->(test_cim_error)
# MERGE (self)-[:DEFINES]->(test_property_unavailable)
# MERGE (self)-[:DEFINES]->(test_non_windows)
# MERGE (self)-[:DEFINES]->(test_existing_output)
# MERGE (self)-[:DEFINES]->(test_midrun_exit)
# MERGE (self)-[:DEFINES]->(test_sample_bounds)
# MERGE (self)-[:DEFINES]->(test_interval_floor)
# MERGE (self)-[:DEFINES]->(test_runner_contract)
# MERGE (test_tree_aggregation)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::process_tree", kind:"function"})
# MERGE (test_tree_aggregation)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::sample_rows", kind:"function"})
# MERGE (test_parent_pid_reuse)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::process_tree", kind:"function"})
# MERGE (test_first_cpu_sample)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::sample_rows", kind:"function"})
# MERGE (test_cpu_normalization)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::cpu_percent", kind:"function"})
# MERGE (test_child_pid_reuse)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::cpu_delta_100ns", kind:"function"})
# MERGE (test_root_pid_reuse)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::cpu_delta_100ns", kind:"function"})
# MERGE (test_snapshot_strict)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::parse_process_snapshot", kind:"function"})
# MERGE (test_fixed_powershell)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::take_snapshot", kind:"function"})
# MERGE (test_pid_input_validation)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::take_snapshot", kind:"function"})
# MERGE (test_cim_error)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::take_snapshot", kind:"function"})
# MERGE (test_property_unavailable)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::parse_process_snapshot", kind:"function"})
# MERGE (test_non_windows)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::capture", kind:"function"})
# MERGE (test_existing_output)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::capture", kind:"function"})
# MERGE (test_midrun_exit)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::capture", kind:"function"})
# MERGE (test_sample_bounds)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::write_sample", kind:"function"})
# MERGE (test_interval_floor)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::validate_options", kind:"function"})
# MERGE (test_runner_contract)-[:TESTS]->(:Symbol {id:"scripts/ui0_bench/capture_tauri_process.py::write_sample", kind:"function"})
# MERGE (test_runner_contract)-[:CALLS]->(:Symbol {id:"scripts/ui0_bench/run_measurement.py::read_driver_samples", kind:"function"})
# MERGE (self)-[:IMPORTS]->(:File {path:"scripts/ui0_bench/run_measurement.py"})
# @endcypher
from __future__ import annotations

import io
import json
import subprocess
import sys
import tempfile
import unittest
from datetime import datetime
from pathlib import Path
from unittest.mock import patch

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import capture_tauri_process as sampler
import run_measurement


def record(
    process_id: int,
    parent_id: int,
    *,
    created: str = "2026-10-23T12:00:00+00:00",
    rss: int = 100,
    kernel: int = 10_000_000,
    user: int = 0,
) -> sampler.ProcessRecord:
    return sampler.ProcessRecord(
        process_id=process_id,
        parent_process_id=parent_id,
        creation_date=datetime.fromisoformat(created),
        working_set_bytes=rss,
        kernel_time_100ns=kernel,
        user_time_100ns=user,
    )


def options(output: Path, *, duration: float = 1.0, interval: float = 1.0) -> sampler.CaptureOptions:
    return sampler.CaptureOptions(4100, "launch", "warm", interval, duration, output)


class CaptureTests(unittest.TestCase):
    def test_tree_aggregation_and_child_changes(self) -> None:
        root = record(4100, 99, rss=1000)
        child = record(4101, 4100, rss=2000, kernel=20_000_000)
        grandchild = record(4102, 4101, rss=3000, kernel=30_000_000)
        unrelated = record(5100, 99, rss=9_000_000)
        first = sampler.process_tree((root, child, grandchild, unrelated), 4100)
        self.assertEqual(set(first), {4100, 4101, 4102})
        first_rows = sampler.sample_rows(options(Path("unused")), first, None, None, 8)
        self.assertEqual(first_rows[0]["value"], 6000)
        self.assertEqual(first_rows[1]["value"], 3)
        next_tree = sampler.process_tree((root, record(4101, 4100, rss=5000, kernel=25_000_000), unrelated), 4100)
        next_rows = sampler.sample_rows(options(Path("unused")), next_tree, first, 1.0, 8)
        self.assertEqual(next_rows[0]["value"], 6000)
        self.assertEqual(next_rows[1]["value"], 2)
        self.assertIn("process_tree_cpu_percent", {row["metric"] for row in next_rows})

    def test_parent_pid_reuse_is_not_a_child_edge(self) -> None:
        root = record(4100, 99, created="2026-10-23T12:00:02+00:00")
        stale_child = record(4101, 4100, created="2026-10-23T12:00:01+00:00", rss=5000)
        self.assertEqual(set(sampler.process_tree((root, stale_child), 4100)), {4100})

    def test_first_sample_omits_cpu_delta(self) -> None:
        tree = sampler.process_tree((record(4100, 99),), 4100)
        rows = sampler.sample_rows(options(Path("unused")), tree, None, None, 8)
        self.assertEqual([row["metric"] for row in rows], ["process_tree_rss_bytes", "process_tree_process_count"])

    def test_cpu_is_normalized_to_total_logical_capacity(self) -> None:
        # 2 logical CPUs × 1 second = 2 CPU-seconds of total capacity.
        self.assertEqual(sampler.cpu_percent(20_000_000, 1.0, 2), 100.0)
        self.assertEqual(sampler.cpu_percent(10_000_000, 1.0, 2), 50.0)

    def test_reused_child_pid_has_no_cross_identity_delta(self) -> None:
        old = {4100: record(4100, 99), 4101: record(4101, 4100, kernel=80_000_000)}
        current = {4100: record(4100, 99), 4101: record(4101, 4100, created="2026-10-23T12:00:01+00:00", kernel=1)}
        self.assertEqual(sampler.cpu_delta_100ns(old, current, 4100), 0)

    def test_reused_root_pid_fails_closed(self) -> None:
        old = {4100: record(4100, 99)}
        current = {4100: record(4100, 99, created="2026-10-23T12:00:01+00:00")}
        with self.assertRaisesRegex(sampler.CaptureError, "root_pid_reused"):
            sampler.cpu_delta_100ns(old, current, 4100)

    def test_snapshot_schema_rejects_extra_fields_and_invalid_counters(self) -> None:
        item = {
            "process_id": 4100, "parent_process_id": 99,
            "creation_date_utc": "2026-10-23T12:00:00.0000000Z",
            "working_set_bytes": 100, "kernel_time_100ns": 10, "user_time_100ns": 0,
        }
        self.assertEqual(sampler.parse_process_snapshot(json.dumps([item]))[0].process_id, 4100)
        with self.assertRaisesRegex(sampler.CaptureError, "invalid_process_snapshot"):
            sampler.parse_process_snapshot(json.dumps([{**item, "process_name": "private.exe"}]))
        with self.assertRaisesRegex(sampler.CaptureError, "invalid_kernel_time_100ns"):
            sampler.parse_process_snapshot(json.dumps([{**item, "kernel_time_100ns": -1}]))

    def test_missing_process_properties_fail_closed(self) -> None:
        with self.assertRaisesRegex(sampler.CaptureError, "process_property_unavailable"):
            sampler.parse_process_snapshot("STAR_UI0_PROPERTY_UNAVAILABLE")

    def test_subprocess_uses_fixed_powershell_and_no_shell(self) -> None:
        payload = json.dumps([{
            "process_id": 4100, "parent_process_id": 99,
            "creation_date_utc": "2026-10-23T12:00:00.0000000Z",
            "working_set_bytes": 100, "kernel_time_100ns": 10, "user_time_100ns": 0,
        }])
        completed = subprocess.CompletedProcess(args=[], returncode=0, stdout=payload, stderr="sensitive-host-path")
        with patch.object(sampler.sys, "platform", "win32"), patch.dict("os.environ", {"SystemRoot": r"C:\Windows"}):
            def runner(command: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                self.assertEqual(command[0], r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe")
                self.assertEqual(command[1:5], ["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
                self.assertIn("4100", command[5])
                self.assertNotIn("launch", command[5])
                self.assertFalse(kwargs["shell"])
                self.assertEqual(kwargs["stderr"], subprocess.DEVNULL)
                self.assertEqual(kwargs["timeout"], sampler.SNAPSHOT_TIMEOUT_SECONDS)
                return completed

            records = sampler.take_snapshot(4100, runner=runner)
        self.assertEqual(records[0].process_id, 4100)
        self.assertNotIn("sensitive-host-path", json.dumps(records, default=str))

    def test_root_pid_query_input_cannot_be_command_fragment(self) -> None:
        with patch.object(sampler.sys, "platform", "win32"), patch.dict("os.environ", {"SystemRoot": r"C:\Windows"}):
            with self.assertRaisesRegex(sampler.CaptureError, "invalid_root_pid"):
                sampler.take_snapshot("4100; Write-Output private")  # type: ignore[arg-type]

    def test_cim_unavailable_is_structured_and_does_not_leak_stderr(self) -> None:
        completed = subprocess.CompletedProcess(args=[], returncode=20, stdout="STAR_UI0_CIM_UNAVAILABLE", stderr="private host detail")
        with patch.object(sampler.sys, "platform", "win32"), patch.dict("os.environ", {"SystemRoot": r"C:\Windows"}):
            with self.assertRaisesRegex(sampler.CaptureError, "cim_unavailable") as caught:
                sampler.take_snapshot(4100, runner=lambda *_args, **_kwargs: completed)
        self.assertNotIn("private host detail", str(caught.exception))

    def test_non_windows_fails_before_creating_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "samples.jsonl"
            with self.assertRaisesRegex(sampler.CaptureError, "windows_required"):
                sampler.capture(options(output), is_windows=False)
            self.assertFalse(output.exists())

    def test_existing_output_is_never_overwritten(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "samples.jsonl"
            output.write_text("keep\n", encoding="utf-8")
            with self.assertRaisesRegex(sampler.CaptureError, "output_already_exists"):
                sampler.capture(options(output), snapshot_provider=lambda _pid: (record(4100, 99),), is_windows=True)
            self.assertEqual(output.read_text(encoding="utf-8"), "keep\n")

    def test_root_exit_midrun_removes_partial_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "samples.jsonl"
            current_time = [0.0]
            snapshots = iter(((record(4100, 99),), ()))

            def sleep(seconds: float) -> None:
                current_time[0] += seconds

            def clock() -> float:
                return current_time[0]

            def provider(_pid: int) -> tuple[sampler.ProcessRecord, ...]:
                return next(snapshots)

            with self.assertRaisesRegex(sampler.CaptureError, "root_process_missing"):
                sampler.capture(
                    options(output, duration=2.2, interval=1.0),
                    snapshot_provider=provider, clock=clock, sleeper=sleep, is_windows=True,
                )
            self.assertFalse(output.exists())

    def test_sample_limits_are_enforced(self) -> None:
        with self.assertRaisesRegex(sampler.CaptureError, "sample_count_limit"):
            sampler.validate_options(options(Path("unused"), duration=3600.0, interval=1.0))
        row = {
            "kind": "sample", "scenario": "x", "condition": "warm",
            "metric": "process_tree_rss_bytes", "value": 1, "unit": "bytes",
        }
        with self.assertRaisesRegex(sampler.CaptureError, "sample_output_limit"):
            sampler.write_sample(io.StringIO(), row, sampler.MAX_SAMPLE_COUNT, 0)
        with self.assertRaisesRegex(sampler.CaptureError, "sample_output_limit"):
            sampler.write_sample(io.StringIO(), row, 0, sampler.MAX_SAMPLE_BYTES)

    def test_subsecond_interval_is_rejected(self) -> None:
        with self.assertRaisesRegex(sampler.CaptureError, "invalid_interval"):
            sampler.validate_options(options(Path("unused"), duration=1.0, interval=0.5))

    def test_samples_match_existing_runner_contract(self) -> None:
        tree = sampler.process_tree((record(4100, 99, rss=4096),), 4100)
        generated = sampler.sample_rows(options(Path("unused")), tree, None, None, 4)
        second = sampler.sample_rows(options(Path("unused")), tree, tree, 1.0, 4)
        generated.extend(row for row in second if row["metric"] == "process_tree_cpu_percent")
        buffer = io.StringIO()
        count = 0
        size = 0
        for row in generated:
            count, size = sampler.write_sample(buffer, row, count, size)
        summarized = io.StringIO()
        result = run_measurement.read_driver_samples(io.BytesIO(buffer.getvalue().encode("utf-8")), summarized)
        self.assertEqual(result["sample_count"], 3)
        self.assertEqual(result["counters"]["process_tree_rss_bytes"]["status"], "available")
        self.assertEqual(result["counters"]["process_tree_cpu_percent"]["status"], "available")
        self.assertEqual(result["metrics"]["warm:process_tree_process_count"]["unit"], "count")


if __name__ == "__main__":
    unittest.main()
