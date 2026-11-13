"""Tests for scripts/ci/detect_changes.py, gate.py and release_preflight.py.

Run: python -m unittest discover -s scripts/ci/tests -v
"""

# @cypher schema=1 source_sha256=13de3f6818cae838d736b551b74e594444b296ffd9213217820f6cff123bba31
# MERGE (self:File {path:"scripts/ci/tests/test_ci_scripts.py"})
# MERGE (detect:File {path:"scripts/ci/detect_changes.py"})
# MERGE (gate:File {path:"scripts/ci/gate.py"})
# MERGE (preflight:File {path:"scripts/ci/release_preflight.py"})
# MERGE (preflight_case:Symbol {id:"scripts/ci/tests/test_ci_scripts.py::ReleasePreflightTests",kind:"class"})
# MERGE (self)-[:DEFINES]->(preflight_case)
# MERGE (preflight_case)-[:CALLS]->(preflight)
# MERGE (detect_case:Symbol {id:"scripts/ci/tests/test_ci_scripts.py::DetectChangesTests",kind:"class"})
# MERGE (gate_case:Symbol {id:"scripts/ci/tests/test_ci_scripts.py::GateTests",kind:"class"})
# MERGE (self)-[:DEFINES]->(detect_case)
# MERGE (self)-[:DEFINES]->(gate_case)
# MERGE (detect_case)-[:CALLS]->(detect)
# MERGE (gate_case)-[:CALLS]->(gate)
# @endcypher

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

# repo 根 (scripts/ci/tests → scripts/ci → scripts → repo root)
ROOT = Path(__file__).resolve().parents[3]

import detect_changes  # noqa: E402
import gate  # noqa: E402
import release_preflight  # noqa: E402


class DetectChangesTests(unittest.TestCase):
    def test_docs_only_change_skips_heavy_jobs(self) -> None:
        r = detect_changes.classify(["docs/architecture/foo.md", "README.md"])
        self.assertTrue(r["docs"])
        self.assertFalse(r["rust"])
        self.assertFalse(r["bff"])
        self.assertFalse(r["frontend"])
        self.assertFalse(r["desktop"])

    def test_workflow_change_triggers_workflow_lint_only_with_ci_scripts(self) -> None:
        r = detect_changes.classify([".github/workflows/license-gate.yml"])
        self.assertTrue(r["workflows"])
        self.assertFalse(r["rust"])
        self.assertFalse(r["frontend"])

    def test_html_under_docs_does_not_trigger_anything(self) -> None:
        r = detect_changes.classify(["docs/arch-live/index.html"])
        self.assertEqual(r, {a: False for a in detect_changes.AREAS})

    def test_rust_crate_change(self) -> None:
        r = detect_changes.classify(["crates/domain-agent/src/lib.rs"])
        self.assertTrue(r["rust"])
        self.assertTrue(r["bff"])  # bff path-depends on crates/*
        self.assertFalse(r["frontend"])
        self.assertFalse(r["desktop"])

    def test_desktop_crate_change_triggers_desktop_and_rust(self) -> None:
        r = detect_changes.classify(["crates/star-desktop/src-tauri/tauri.conf.json"])
        self.assertTrue(r["desktop"])
        self.assertTrue(r["rust"])

    def test_frontend_change(self) -> None:
        r = detect_changes.classify(["frontend/src/app/page.tsx"])
        self.assertTrue(r["frontend"])
        self.assertFalse(r["rust"])

    def test_lockfile_and_toolchain_trigger_rust(self) -> None:
        for path in ("Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "deny.toml"):
            with self.subTest(path=path):
                self.assertTrue(detect_changes.classify([path])["rust"])

    def test_ci_definition_change_runs_everything(self) -> None:
        r = detect_changes.classify([".github/workflows/ci.yml"])
        self.assertTrue(all(r.values()), r)

    def test_windows_separators_are_normalised(self) -> None:
        r = detect_changes.classify(["crates\\domain-agent\\src\\lib.rs"])
        self.assertTrue(r["rust"])

    def test_empty_change_list_is_all_false(self) -> None:
        self.assertEqual(
            detect_changes.classify([]), {a: False for a in detect_changes.AREAS}
        )

    def test_non_push_event_fails_open_to_everything(self) -> None:
        with mock.patch.dict(os.environ, {"EVENT_NAME": "workflow_dispatch"}, clear=False):
            with mock.patch.object(detect_changes, "_emit") as emit:
                self.assertEqual(detect_changes.main([]), 0)
        self.assertTrue(all(emit.call_args.args[0].values()))

    def test_all_zero_base_fails_open(self) -> None:
        env = {"EVENT_NAME": "push", "BASE_SHA": "0" * 40, "HEAD_SHA": "a" * 40}
        with mock.patch.dict(os.environ, env, clear=False):
            with mock.patch.object(detect_changes, "_emit") as emit:
                detect_changes.main([])
        self.assertTrue(all(emit.call_args.args[0].values()))

    def test_undeterminable_diff_fails_open(self) -> None:
        env = {"EVENT_NAME": "pull_request", "BASE_SHA": "b" * 40, "HEAD_SHA": "c" * 40}
        with mock.patch.dict(os.environ, env, clear=False):
            with mock.patch.object(detect_changes, "_git_changed_files", return_value=None):
                with mock.patch.object(detect_changes, "_emit") as emit:
                    detect_changes.main([])
        self.assertTrue(all(emit.call_args.args[0].values()))


class GateTests(unittest.TestCase):
    def test_all_success_passes(self) -> None:
        ok, _ = gate.evaluate({"a": {"result": "success"}, "b": {"result": "success"}})
        self.assertTrue(ok)

    def test_skipped_is_not_a_failure(self) -> None:
        ok, _ = gate.evaluate({"a": {"result": "success"}, "b": {"result": "skipped"}})
        self.assertTrue(ok)

    def test_failure_blocks(self) -> None:
        ok, lines = gate.evaluate({"a": {"result": "success"}, "b": {"result": "failure"}})
        self.assertFalse(ok)
        self.assertTrue(any(line.startswith("FAIL b") for line in lines))

    def test_cancelled_blocks(self) -> None:
        ok, _ = gate.evaluate({"a": {"result": "cancelled"}})
        self.assertFalse(ok)

    def test_empty_gate_fails_closed(self) -> None:
        ok, _ = gate.evaluate({})
        self.assertFalse(ok)

    def test_missing_result_fails_closed(self) -> None:
        ok, _ = gate.evaluate({"a": {}})
        self.assertFalse(ok)

    def test_main_rejects_invalid_json(self) -> None:
        with mock.patch.dict(os.environ, {"NEEDS_JSON": "not json"}, clear=False):
            self.assertEqual(gate.main(), 2)


def _run(name: str, conclusion: str | None, run_id: int, status: str = "completed") -> dict:
    return {"name": name, "conclusion": conclusion, "id": run_id, "status": status}


class ReleasePreflightTests(unittest.TestCase):
    GATE = "CI gate (required)"
    LIC = "License (cargo-deny)"

    def test_tag_grammar(self) -> None:
        for good in ("v0.0.0", "v1.2.3", "v10.20.30"):
            with self.subTest(tag=good):
                self.assertTrue(release_preflight.validate_tag(good))
        for bad in ("1.2.3", "v1.2", "v01.2.3", "v1.2.3-rc1", "v1.2.3+b", "v1.2.3 ", "../x", "v1.2.3" + chr(10)):
            with self.subTest(tag=bad):
                self.assertFalse(release_preflight.validate_tag(bad))

    def test_all_required_green_passes(self) -> None:
        page = {"check_runs": [_run(self.GATE, "success", 2), _run(self.LIC, "success", 3)]}
        ok, _ = release_preflight.evaluate_check_runs(page)
        self.assertTrue(ok)

    def test_missing_required_check_fails_closed(self) -> None:
        ok, lines = release_preflight.evaluate_check_runs({"check_runs": [_run(self.GATE, "success", 2)]})
        self.assertFalse(ok)
        self.assertTrue(any("no such check run" in line for line in lines))

    def test_failed_required_check_fails(self) -> None:
        page = {"check_runs": [_run(self.GATE, "failure", 2), _run(self.LIC, "success", 3)]}
        ok, _ = release_preflight.evaluate_check_runs(page)
        self.assertFalse(ok)

    def test_newest_rerun_wins(self) -> None:
        page = {"check_runs": [_run(self.GATE, "failure", 2), _run(self.GATE, "success", 9), _run(self.LIC, "success", 3)]}
        ok, _ = release_preflight.evaluate_check_runs(page)
        self.assertTrue(ok)
        page = {"check_runs": [_run(self.GATE, "success", 2), _run(self.GATE, "failure", 9), _run(self.LIC, "success", 3)]}
        ok, _ = release_preflight.evaluate_check_runs(page)
        self.assertFalse(ok)

    def test_in_progress_check_is_not_success(self) -> None:
        page = {"check_runs": [_run(self.GATE, None, 2, "in_progress"), _run(self.LIC, "success", 3)]}
        ok, _ = release_preflight.evaluate_check_runs(page)
        self.assertFalse(ok)

    def test_paginated_payload_is_merged(self) -> None:
        pages = [{"check_runs": [_run(self.GATE, "success", 2)]}, {"check_runs": [_run(self.LIC, "success", 3)]}]
        ok, _ = release_preflight.evaluate_check_runs(pages)
        self.assertTrue(ok)

    def test_empty_payload_fails_closed(self) -> None:
        ok, _ = release_preflight.evaluate_check_runs({"check_runs": []})
        self.assertFalse(ok)




class SbomValidatorTests(unittest.TestCase):
    """WBS 10.3.4: validate_sbom.js 的 cargo-sbom 等价形态接受 + fail-closed 变异。"""

    SBOM_JS = "scripts/ci/validate_sbom.js"

    def _write_sboms(self, tmp: str, rust: dict, frontend: dict) -> tuple[str, str]:
        import json
        rp = os.path.join(tmp, "rust.spdx.json")
        fp = os.path.join(tmp, "fe.spdx.json")
        json.dump(rust, open(rp, "w", encoding="utf-8"))
        json.dump(frontend, open(fp, "w", encoding="utf-8"))
        return rp, fp

    def _cargo_style(self) -> dict:
        return {
            "spdxVersion": "SPDX-2.3",
            "packages": [{"name": "star-desktop", "SPDXID": "SPDXRef-Package-star-desktop"}],
            "files": [{"fileName": "./star-desktop", "SPDXID": "SPDXRef-File-star-desktop"}],
            "relationships": [
                {"spdxElementId": "SPDXRef-DOCUMENT", "relationshipType": "DESCRIBES", "relatedSpdxElement": "SPDXRef-File-star-desktop"},
                {"spdxElementId": "SPDXRef-File-star-desktop", "relationshipType": "GENERATED_FROM", "relatedSpdxElement": "SPDXRef-Package-star-desktop"},
            ],
        }

    def _npm_style(self) -> dict:
        return {
            "spdxVersion": "SPDX-2.3",
            "packages": [{"name": "star-desktop-frontend" if i == 0 else f"p{i}", "SPDXID": f"SPDXRef-Package-{i}"} for i in range(225)],
            "relationships": [
                {"spdxElementId": "SPDXRef-DOCUMENT", "relationshipType": "DESCRIBES", "relatedSpdxElement": "SPDXRef-Package-0"},
            ],
        }

    def test_cargo_sbom_equivalent_form_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            rp, fp = self._write_sboms(tmp, self._cargo_style(), self._npm_style())
            r = subprocess.run(["node", self.SBOM_JS, rp, fp], cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(r.returncode, 0, r.stderr)
            self.assertIn("PASS", r.stdout)

    def test_describes_without_generated_from_rejected(self) -> None:
        cargo = self._cargo_style()
        cargo["relationships"] = cargo["relationships"][:1]  # 只留 DESCRIBES, 去掉 GENERATED_FROM
        with tempfile.TemporaryDirectory() as tmp:
            rp, fp = self._write_sboms(tmp, cargo, self._npm_style())
            r = subprocess.run(["node", self.SBOM_JS, rp, fp], cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("DESCRIBES", r.stderr)

    def test_retired_game_package_rejected(self) -> None:
        cargo = self._cargo_style()
        cargo["packages"].append({"name": "star-game", "SPDXID": "SPDXRef-Package-star-game"})
        with tempfile.TemporaryDirectory() as tmp:
            rp, fp = self._write_sboms(tmp, cargo, self._npm_style())
            r = subprocess.run(["node", self.SBOM_JS, rp, fp], cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(r.returncode, 0)
            self.assertIn("retired game", r.stderr)

    def test_missing_args_fails(self) -> None:
        r = subprocess.run(["node", self.SBOM_JS], cwd=ROOT, capture_output=True, text=True)
        self.assertNotEqual(r.returncode, 0)


if __name__ == "__main__":
    unittest.main()

