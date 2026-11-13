# @cypher schema=1 source_sha256=93a98f13c83fbf2bf69559ff785dde50408292e5524f07dc728efb6cb36bd69b
# MERGE (self:File {path:"scripts/ui0_bench/test_generate_fixture.py"})
# MERGE (self)-[:IMPORTS]->(:File {path:"scripts/ui0_bench/generate_fixture.py"})
# MERGE (self)-[:IMPORTS]->(:File {path:"scripts/ui0_bench/run_measurement.py"})
# MERGE (fixture_tests:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests", kind:"class"})
# MERGE (measurement_tests:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests", kind:"class"})
# MERGE (test_determinism:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests.test_generation_is_byte_identical", kind:"test"})
# MERGE (test_seed:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests.test_different_seed_changes_stable_fixture", kind:"test"})
# MERGE (test_validation:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests.test_manifest_rejects_inconsistent_totals_and_unknown_schema", kind:"test"})
# MERGE (test_terminal_chunks:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests.test_terminal_stream_chunks_are_bounded_and_contiguous", kind:"test"})
# MERGE (test_prng:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::FixtureGeneratorTests.test_prng_version_has_fixed_vector", kind:"test"})
# MERGE (test_no_driver:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_no_driver_means_no_synthetic_ui_samples", kind:"test"})
# MERGE (test_inventory:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_host_inventory_is_schema_filtered_and_retained", kind:"test"})
# MERGE (test_measurement:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_real_driver_samples_are_summarized", kind:"test"})
# MERGE (test_empty_driver:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_empty_driver_is_not_reported_as_a_pass", kind:"test"})
# MERGE (test_percentile:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_percentile_uses_interpolated_rank_and_empty_is_unknown", kind:"test"})
# MERGE (test_conflicting_units:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_conflicting_units_for_one_metric_are_rejected", kind:"test"})
# MERGE (test_negative_counter:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_negative_counters_are_rejected", kind:"test"})
# MERGE (test_sample_line_limit:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_oversized_sample_line_is_rejected", kind:"test"})
# MERGE (test_inventory_limit:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_oversized_inventory_is_unknown", kind:"test"})
# MERGE (test_input_byte_limit:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_oversized_blank_input_is_rejected", kind:"test"})
# MERGE (test_nonregular_samples:Symbol {id:"scripts/ui0_bench/test_generate_fixture.py::MeasurementRunnerTests.test_nonregular_samples_leave_no_measurement", kind:"test"})
# MERGE (write_jsonl:Symbol {id:"scripts/ui0_bench/generate_fixture.py::write_jsonl", kind:"function"})
# MERGE (validate_manifest:Symbol {id:"scripts/ui0_bench/generate_fixture.py::validate_manifest", kind:"function"})
# MERGE (terminal_chunks:Symbol {id:"scripts/ui0_bench/generate_fixture.py::_terminal_chunks", kind:"function"})
# MERGE (prng_next:Symbol {id:"scripts/ui0_bench/generate_fixture.py::SplitMix64.next_u64", kind:"method"})
# MERGE (run_measurement:Symbol {id:"scripts/ui0_bench/run_measurement.py::run_measurement", kind:"function"})
# MERGE (summarize_measurements:Symbol {id:"scripts/ui0_bench/run_measurement.py::summarize_measurements", kind:"function"})
# MERGE (self)-[:DEFINES]->(fixture_tests)
# MERGE (self)-[:DEFINES]->(measurement_tests)
# MERGE (self)-[:DEFINES]->(test_determinism)
# MERGE (self)-[:DEFINES]->(test_seed)
# MERGE (self)-[:DEFINES]->(test_validation)
# MERGE (self)-[:DEFINES]->(test_terminal_chunks)
# MERGE (self)-[:DEFINES]->(test_prng)
# MERGE (self)-[:DEFINES]->(test_no_driver)
# MERGE (self)-[:DEFINES]->(test_inventory)
# MERGE (self)-[:DEFINES]->(test_measurement)
# MERGE (self)-[:DEFINES]->(test_empty_driver)
# MERGE (self)-[:DEFINES]->(test_percentile)
# MERGE (self)-[:DEFINES]->(test_conflicting_units)
# MERGE (self)-[:DEFINES]->(test_negative_counter)
# MERGE (self)-[:DEFINES]->(test_sample_line_limit)
# MERGE (self)-[:DEFINES]->(test_inventory_limit)
# MERGE (self)-[:DEFINES]->(test_input_byte_limit)
# MERGE (self)-[:DEFINES]->(test_nonregular_samples)
# MERGE (test_determinism)-[:TESTS]->(write_jsonl)
# MERGE (test_determinism)-[:TESTS]->(validate_manifest)
# MERGE (test_seed)-[:TESTS]->(write_jsonl)
# MERGE (test_validation)-[:TESTS]->(validate_manifest)
# MERGE (test_terminal_chunks)-[:TESTS]->(terminal_chunks)
# MERGE (test_prng)-[:TESTS]->(prng_next)
# MERGE (test_no_driver)-[:TESTS]->(run_measurement)
# MERGE (test_inventory)-[:TESTS]->(run_measurement)
# MERGE (test_measurement)-[:TESTS]->(run_measurement)
# MERGE (test_empty_driver)-[:TESTS]->(run_measurement)
# MERGE (test_percentile)-[:TESTS]->(summarize_measurements)
# MERGE (test_conflicting_units)-[:TESTS]->(run_measurement)
# MERGE (test_negative_counter)-[:TESTS]->(run_measurement)
# MERGE (test_sample_line_limit)-[:TESTS]->(run_measurement)
# MERGE (test_inventory_limit)-[:TESTS]->(run_measurement)
# MERGE (test_input_byte_limit)-[:TESTS]->(run_measurement)
# MERGE (test_nonregular_samples)-[:TESTS]->(run_measurement)
# @endcypher

from __future__ import annotations

import hashlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

from generate_fixture import (  # noqa: E402
    ManifestError,
    SplitMix64,
    _terminal_chunks,
    validate_manifest,
    write_jsonl,
)
from run_measurement import (  # noqa: E402
    MAX_INVENTORY_BYTES,
    MAX_SAMPLE_INPUT_BYTES,
    MAX_SAMPLE_LINE_BYTES,
    SampleError,
    run_measurement,
    summarize_measurements,
)


def small_manifest() -> dict[str, object]:
    return {
        "schema": "star.ui-benchmark-fixture",
        "version": "0.1.0",
        "profile_id": "TEST-UI0",
        "captured_date": "2026-10-21",
        "seed": 20261021,
        "directory": {
            "projects": 2,
            "runs_per_project": 2,
            "parent_scopes": 4,
            "worktrees_per_run": 3,
            "worktrees_total": 12,
            "branches_per_worktree": 1,
            "page_size": 2,
            "available_pages_per_run": 2,
            "picker_retained_page_limit": 2,
            "parent_cache_boundary": 2,
            "state_flag_rules": {
                "busy_every_nth_worktree": 2,
                "stale_every_nth_worktree": 3,
                "dirty_every_nth_worktree": 2,
                "locked_every_nth_worktree": 3,
                "unknown_every_nth_worktree": 2,
                "flags_overlap": True,
                "index_base": 0,
            },
        },
        "concurrency": {
            "active_runs": 2,
            "agents_per_active_run": 2,
            "agents_total": 4,
            "open_terminal_panes": 1,
        },
        "tasks_and_terminal": {
            "tasks_per_active_run": 1,
            "output_bytes_per_pane": 37,
            "stream_modes": ["utf8_ansi_burst", "long_line"],
        },
        "canvas": {
            "node_counts": [3],
            "edges_per_node": {"sparse": 0.5, "dense": 1},
            "operations": ["initial_viewport_load", "pan", "cancel_viewport_load"],
        },
        "bi": {"events_per_page": 2, "pages": 2, "events_total": 4, "runs_while_ui_is_active": True},
        "interaction_sequence": {
            "focus_switch_cycles": 3,
            "switch_across_parent_scopes": 4,
            "directory_scroll_and_keyboard_navigation": True,
            "overlap_canvas_and_bi_background_work": True,
            "cancel_old_generation_and_wait_for_drain": True,
        },
        "measurement_repetitions": {"cold_start_runs": 2, "warm_interaction_runs": 3, "report_percentiles": [50, 95]},
    }


def encode_fixture(manifest: dict[str, object], seed: int) -> tuple[bytes, tuple[int, int, str]]:
    sink = io.BytesIO()
    result = write_jsonl(sink, validate_manifest(manifest), "a" * 64, seed)
    return sink.getvalue(), result


class FixtureGeneratorTests(unittest.TestCase):
    def test_generation_is_byte_identical(self) -> None:
        manifest = small_manifest()
        first, first_result = encode_fixture(manifest, manifest["seed"])
        second, second_result = encode_fixture(manifest, manifest["seed"])
        self.assertEqual(first, second)
        self.assertEqual(first_result, second_result)
        self.assertEqual(first_result[2], hashlib.sha256(first).hexdigest())

    def test_different_seed_changes_stable_fixture(self) -> None:
        manifest = small_manifest()
        first, _ = encode_fixture(manifest, 1)
        second, _ = encode_fixture(manifest, 2)
        self.assertNotEqual(first, second)

    def test_manifest_rejects_inconsistent_totals_and_unknown_schema(self) -> None:
        invalid_total = small_manifest()
        invalid_total["directory"]["worktrees_total"] += 1
        with self.assertRaises(ManifestError):
            validate_manifest(invalid_total)
        invalid_schema = small_manifest()
        invalid_schema["schema"] = "other"
        with self.assertRaises(ManifestError):
            validate_manifest(invalid_schema)

    def test_terminal_stream_chunks_are_bounded_and_contiguous(self) -> None:
        chunks = list(_terminal_chunks(b"pattern", 10_000))
        self.assertEqual(chunks[0][0], 0)
        self.assertTrue(all(len(payload) <= 3_072 for _, payload in chunks))
        self.assertEqual([offset for offset, _ in chunks], [sum(len(data) for _, data in chunks[:i]) for i in range(len(chunks))])
        self.assertEqual(sum(len(payload) for _, payload in chunks), 10_000)

    def test_prng_version_has_fixed_vector(self) -> None:
        prng = SplitMix64(0)
        self.assertEqual(prng.next_u64(), 0xE220A8397B1DCDAF)
        self.assertEqual(prng.next_u64(), 0x6E789E6AA1B965F4)


class MeasurementRunnerTests(unittest.TestCase):
    def _manifest_file(self, root: Path) -> Path:
        path = root / "manifest.json"
        path.write_text(json.dumps(small_manifest()), encoding="utf-8")
        return path

    def test_no_driver_means_no_synthetic_ui_samples(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            result = run_measurement(
                self._manifest_file(root), root / "run", observed_at="2026-10-22T00:00:00.000Z", run_id="test-run"
            )
            self.assertEqual(result["measurement_status"], "unavailable")
            self.assertEqual(result["sample_count"], 0)
            self.assertEqual(result["metric_summaries"], {})
            self.assertEqual(result["cold_warm_state"]["status"], "unknown")
            self.assertTrue((root / "run" / "raw_samples.jsonl").is_file())
            self.assertTrue((root / "run" / "measurement.json").is_file())

    def test_real_driver_samples_are_summarized(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self._manifest_file(root)
            sample_path = root / "driver.jsonl"
            sample_path.write_text(
                "\n".join(
                    [
                        json.dumps({"kind": "sample", "scenario": "focus", "condition": "warm", "metric": "focus_to_visible_ms", "value": value, "unit": "ms", "counters": {"queue_items": value}})
                        for value in (1, 3, 5, 7)
                    ]
                    + [
                        json.dumps({"kind": "lifecycle", "scenario": "focus", "event": "cancel_requested"}),
                        json.dumps({"kind": "lifecycle", "scenario": "focus", "event": "drain_completed", "elapsed_ms": 8}),
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            result = run_measurement(
                manifest,
                root / "run",
                track="shared-web",
                renderer="fixture-test-driver",
                samples_path=sample_path,
                observed_at="2026-10-22T00:00:00.000Z",
                run_id="test-run",
            )
            metric = result["metric_summaries"]["warm:focus_to_visible_ms"]
            self.assertEqual(result["measurement_status"], "available")
            self.assertEqual(result["sample_count"], 4)
            self.assertEqual(metric["p50"], 4.0)
            self.assertAlmostEqual(metric["p95"], 6.7)
            self.assertEqual(result["counters"]["queue_items"]["conditions"]["warm"]["p50"], 4.0)
            self.assertEqual(result["counters"]["queue_items"]["unit"], "count")
            self.assertEqual(result["counters"]["process_tree_rss_bytes"]["status"], "unknown")
            self.assertEqual(result["lifecycle_samples"]["events"]["cancel_requested"]["status"], "available")
            self.assertEqual(result["lifecycle_samples"]["events"]["drain_completed"]["count"], 1)
            self.assertEqual(result["lifecycle_samples"]["events"]["drain_completed"]["elapsed_ms"]["p50"], 8.0)
            raw_path = root / "run" / result["raw_samples"]["file"]
            self.assertEqual(result["raw_samples"]["sha256"], hashlib.sha256(raw_path.read_bytes()).hexdigest())

    def test_conflicting_units_for_one_metric_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self._manifest_file(root)
            samples = root / "driver.jsonl"
            samples.write_text(
                "\n".join(
                    json.dumps({"kind": "sample", "scenario": "focus", "condition": "warm",
                                "metric": "focus_to_visible", "value": value, "unit": unit})
                    for value, unit in ((1, "ms"), (2, "seconds"))
                ) + "\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(SampleError, "inconsistent units"):
                run_measurement(manifest, root / "run", track="shared-web", samples_path=samples,
                                observed_at="2026-10-22T00:00:00.000Z", run_id="test-run")

    def test_negative_counters_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self._manifest_file(root)
            samples = root / "driver.jsonl"
            samples.write_text(
                json.dumps({"kind": "sample", "scenario": "queue", "condition": "warm",
                            "metric": "queue_latency", "value": 1, "unit": "ms",
                            "counters": {"queue_items": -1}}) + "\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(SampleError, "must be non-negative"):
                run_measurement(manifest, root / "run", track="shared-web", samples_path=samples,
                                observed_at="2026-10-22T00:00:00.000Z", run_id="test-run")

    def test_oversized_sample_line_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            samples = root / "driver.jsonl"
            samples.write_bytes(b"x" * (MAX_SAMPLE_LINE_BYTES + 1) + b"\n")
            with self.assertRaisesRegex(SampleError, "line exceeds"):
                run_measurement(self._manifest_file(root), root / "run", track="shared-web",
                                samples_path=samples, observed_at="2026-10-22T00:00:00.000Z",
                                run_id="test-run")

    def test_oversized_inventory_is_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            inventory = root / "inventory.json"
            inventory.write_bytes(b"x" * (MAX_INVENTORY_BYTES + 1))
            result = run_measurement(self._manifest_file(root), root / "run", inventory_path=inventory,
                                     observed_at="2026-10-22T00:00:00.000Z", run_id="test-run")
            self.assertEqual(result["host_inventory"]["status"], "unknown")
            self.assertEqual(result["host_inventory"]["reason"], "inventory_file_exceeds_size_limit")

    def test_oversized_blank_input_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            samples = root / "blank-lines.jsonl"
            samples.write_bytes(b"\n" * (MAX_SAMPLE_INPUT_BYTES + 1))
            output = root / "run"
            with self.assertRaisesRegex(SampleError, "input exceeds"):
                run_measurement(self._manifest_file(root), output, track="shared-web",
                                samples_path=samples, observed_at="2026-10-22T00:00:00.000Z",
                                run_id="test-run")
            self.assertTrue((output / "raw_samples.jsonl").is_file())
            self.assertFalse((output / "measurement.json").exists())

    def test_nonregular_samples_leave_no_measurement(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            samples_directory = root / "samples-directory"
            samples_directory.mkdir()
            output = root / "run"
            with self.assertRaisesRegex(SampleError, "regular file"):
                run_measurement(self._manifest_file(root), output, track="shared-web",
                                samples_path=samples_directory, observed_at="2026-10-22T00:00:00.000Z",
                                run_id="test-run")
            self.assertTrue((output / "raw_samples.jsonl").is_file())
            self.assertFalse((output / "measurement.json").exists())

    def test_empty_driver_is_not_reported_as_a_pass(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self._manifest_file(root)
            sample_path = root / "empty.jsonl"
            sample_path.write_text("", encoding="utf-8")
            result = run_measurement(
                manifest, root / "run", track="rust-native", samples_path=sample_path,
                observed_at="2026-10-22T00:00:00.000Z", run_id="test-run",
            )
            self.assertEqual(result["measurement_status"], "not_run")
            self.assertEqual(result["sample_count"], 0)
            self.assertEqual(result["metric_summaries"], {})

    def test_host_inventory_is_schema_filtered_and_retained(self) -> None:
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = self._manifest_file(root)
            inventory_path = root / "inventory.json"
            inventory = {
                "schema": "star.ui-benchmark-host-inventory",
                "version": "0.1.0",
                "captured_at": "2026-10-22T12:00:00.000Z",
                "collection_method": ["Get-CimInstance Win32_OperatingSystem", "System.Windows.Forms + Win32 display APIs"],
                "os": {"status": "available", "method": "Get-CimInstance Win32_OperatingSystem", "reason": None,
                       "value": {"caption": "Windows", "version": "10.0", "build": "26300", "architecture": "64 bit", "private_value": "drop-me"}},
                "cpu": {"status": "unknown", "method": "Get-CimInstance Win32_Processor", "reason": "collection_failed", "value": None},
                "memory": {"status": "unknown", "method": "Get-CimInstance Win32_ComputerSystem", "reason": "collection_failed", "value": None},
                "gpu": {"status": "unknown", "method": "Get-CimInstance Win32_VideoController", "reason": "collection_failed", "value": None},
                "active_power_scheme": {"status": "unknown", "method": "powercfg.exe /getactivescheme", "reason": "collection_failed", "value": None},
                "displays": {"status": "available", "reason": None, "method": "System.Windows.Forms + Win32 display APIs", "displays": [
                    {"device_name": "\\\\.\\DISPLAY1", "primary": True,
                     "resolution": {"width": 1920, "height": 1080, "method": "System.Windows.Forms.Screen.Bounds"},
                     "refresh_hz": {"status": "available", "value": 60, "method": "user32.EnumDisplaySettings", "reason": None},
                     "dpi": {"status": "available", "value": {"x": 96, "y": 96}, "method": "Shcore.GetDpiForMonitor", "reason": None}},
                ]},
            }
            inventory_path.write_text(json.dumps(inventory), encoding="utf-8")
            sample_path = root / "driver.jsonl"
            sample_path.write_text(json.dumps({"kind": "sample", "scenario": "focus", "condition": "warm",
                                               "metric": "focus_to_visible_ms", "value": 1, "unit": "ms"}) + "\n",
                                   encoding="utf-8")
            result = run_measurement(
                manifest, root / "run", track="shared-web", samples_path=sample_path,
                inventory_path=inventory_path, observed_at="2026-10-22T12:00:00.000Z", run_id="test-run",
            )
            self.assertEqual(result["host_inventory"]["status"], "available")
            saved_os = result["host_inventory"]["value"]["os"]["value"]
            self.assertNotIn("private_value", saved_os)
            self.assertEqual(result["host_inventory"]["value"]["displays"]["displays"][0]["refresh_hz"]["value"], 60)

    def test_percentile_uses_interpolated_rank_and_empty_is_unknown(self) -> None:
        summary = summarize_measurements([1.0, 2.0, 3.0, 4.0])
        self.assertEqual(summary["p50"], 2.5)
        self.assertAlmostEqual(summary["p95"], 3.85)
        self.assertIsNone(summarize_measurements([])["p50"])


if __name__ == "__main__":
    unittest.main()
