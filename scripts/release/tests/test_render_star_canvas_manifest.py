"""Focused fail-closed tests for Star Canvas release manifest rendering."""

# @cypher schema=1 source_sha256=bb5bd44e59370031ec8e23d4be1400fe1a7eb2dba445512dbf41f5af2c164b99
# MERGE (self:File {path:"scripts/release/tests/test_render_star_canvas_manifest.py"})
# MERGE (suite:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests",kind:"class"})
# MERGE (set_up:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.setUp",kind:"method"})
# MERGE (valid:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.test_renders_deterministically_from_exact_release_inputs",kind:"method"})
# MERGE (release_reject:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.test_rejects_invalid_resolver_output",kind:"method"})
# MERGE (digest_reject:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.test_rejects_mutable_or_mismatched_digest_records",kind:"method"})
# MERGE (missing:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.test_rejects_missing_or_duplicate_service_records",kind:"method"})
# MERGE (placeholder:Symbol {id:"scripts/release/tests/test_render_star_canvas_manifest.py::RenderManifestTests.test_rejects_template_placeholder_drift",kind:"method"})
# MERGE (self)-[:DEFINES]->(suite)
# MERGE (suite)-[:DEFINES]->(set_up)
# MERGE (suite)-[:DEFINES]->(valid)
# MERGE (suite)-[:DEFINES]->(release_reject)
# MERGE (suite)-[:DEFINES]->(digest_reject)
# MERGE (suite)-[:DEFINES]->(missing)
# MERGE (suite)-[:DEFINES]->(placeholder)
# @endcypher

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import tempfile
import unittest
from pathlib import Path


REPOSITORY = Path(__file__).resolve().parents[3]
MODULE_PATH = REPOSITORY / "scripts" / "release" / "render_star_canvas_manifest.py"
SPEC = importlib.util.spec_from_file_location("render_star_canvas_manifest", MODULE_PATH)
assert SPEC is not None and SPEC.loader is not None
RENDERER = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RENDERER)


class RenderManifestTests(unittest.TestCase):
    def setUp(self) -> None:
        self.release = {
            "tag": "v1.0.6",
            "version": "1.0.6",
            "source_sha": "a" * 40,
        }
        self.template = "".join(
            f"---\nkind: Deployment\nmetadata:\n  labels:\n    version: __STAR_RELEASE_TAG_REQUIRED__\nspec:\n  template:\n    metadata:\n      labels:\n        version: __STAR_RELEASE_TAG_REQUIRED__\n    spec:\n      containers:\n      - name: {service}\n        image: {repository}@sha256:{RENDERER.DIGEST_MARKERS[service]}\n"
            for service, repository in RENDERER.SERVICES.items()
        )
        self.records = [self.digest_record(service) for service in RENDERER.SERVICES]

    def digest_record(self, service: str) -> dict[str, str]:
        repository = RENDERER.SERVICES[service]
        digest = "sha256:" + (str(len(service) % 10) * 64)
        return {
            "service": service,
            **self.release,
            "version_ref": f"{repository}:{self.release['version']}",
            "source_ref": f"{repository}:{self.release['source_sha']}",
            "image_ref": f"{repository}@{digest}",
            "digest": digest,
        }

    def test_renders_deterministically_from_exact_release_inputs(self) -> None:
        rendered = RENDERER.render_manifest(self.template, self.release, self.records)
        self.assertEqual(rendered, RENDERER.render_manifest(self.template, self.release, self.records))
        self.assertEqual(rendered.count("version: v1.0.6"), 6)
        self.assertNotIn("__", rendered)
        for record in self.records:
            self.assertEqual(rendered.count(f"image: {record['image_ref']}"), 1)

    def test_renders_the_active_k3s_template_with_all_three_pins(self) -> None:
        template = (REPOSITORY / "deploy" / "star-canvas-services-k3s.yaml").read_text(
            encoding="utf-8"
        )
        rendered = RENDERER.render_manifest(template, self.release, self.records)
        self.assertEqual(rendered.count("version: v1.0.6"), 6)
        self.assertNotIn("v0.1", rendered)
        for record in self.records:
            self.assertIn(f"image: {record['image_ref']}", rendered)

    def test_cli_writes_the_same_deterministic_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            release_path = root / "release.json"
            template_path = root / "template.yaml"
            output_path = root / "rendered" / "manifest.yaml"
            release_path.write_text(json.dumps(self.release), encoding="utf-8")
            template_path.write_text(self.template, encoding="utf-8")
            digest_paths = []
            for index, record in enumerate(self.records):
                digest_path = root / f"digest-{index}.json"
                digest_path.write_text(json.dumps(record), encoding="utf-8")
                digest_paths.extend(["--digest-record", str(digest_path)])

            with contextlib.redirect_stdout(io.StringIO()):
                exit_code = RENDERER.main(
                    [
                        "--release-json",
                        str(release_path),
                        *digest_paths,
                        "--template",
                        str(template_path),
                        "--output",
                        str(output_path),
                    ]
                )

            self.assertEqual(exit_code, 0)
            self.assertEqual(
                output_path.read_text(encoding="utf-8"),
                RENDERER.render_manifest(self.template, self.release, self.records),
            )

    def test_rejects_invalid_resolver_output(self) -> None:
        for tag, version in (("v01.0.6", "01.0.6"), ("v1.0.6", "1.0.5"), ("1.0.6", "1.0.6")):
            with self.subTest(tag=tag, version=version):
                value = {**self.release, "tag": tag, "version": version}
                with self.assertRaises(RENDERER.RenderError):
                    RENDERER.render_manifest(self.template, value, self.records)

    def test_rejects_mutable_or_mismatched_digest_records(self) -> None:
        mutable = [dict(record) for record in self.records]
        mutable[0]["image_ref"] = mutable[0]["image_ref"].split("@", 1)[0] + ":latest"
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, mutable)

        mismatched_source = [dict(record) for record in self.records]
        mismatched_source[1]["source_sha"] = "b" * 40
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, mismatched_source)

        mismatched_tag = [dict(record) for record in self.records]
        mismatched_tag[0]["version_ref"] = mismatched_tag[0]["version_ref"].replace(
            ":1.0.6", ":1.0.5"
        )
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, mismatched_tag)

        malformed_digest = [dict(record) for record in self.records]
        malformed_digest[2]["digest"] = "sha256:not-a-digest"
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, malformed_digest)

    def test_rejects_missing_or_duplicate_service_records(self) -> None:
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, self.records[:2])
        duplicate = [*self.records[:2], dict(self.records[0])]
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(self.template, self.release, duplicate)

    def test_rejects_template_placeholder_drift(self) -> None:
        bad_version_count = self.template.replace("__STAR_RELEASE_TAG_REQUIRED__", "", 1)
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(bad_version_count, self.release, self.records)

        wrong_repository = self.template.replace(
            "ghcr.io/ulyssesleolee/star-canvas-engine", "example.invalid/star-canvas-engine"
        )
        with self.assertRaises(RENDERER.RenderError):
            RENDERER.render_manifest(wrong_repository, self.release, self.records)

    def test_workflow_keeps_release_controls_and_uses_resolver_outputs(self) -> None:
        workflow = (REPOSITORY / ".github" / "workflows" / "publish-star-canvas-services.yml").read_text(
            encoding="utf-8"
        )
        for required in (
            "github.event.inputs.release_gate_confirmation == 'APPROVED'",
            "startsWith(github.ref, 'refs/tags/v')",
            "queue: max",
            "cancel-in-progress: false",
            "environment: star-stable-release",
            "docker/setup-buildx-action@8d2750c68a42422c14e847fe6c8ac0403b4cbd6f",
            "fromJSON('[\"canvas-engine\",\"domain-canvas\",\"canvas-realtime\"]')",
            "sbom: true",
            "provenance: mode=max",
            'allowed = {"canvas-engine", "domain-canvas", "canvas-realtime"}',
            "resolve_star_version.py",
            '"version": version',
            '"source_sha": source_sha',
            '"workspace": True',
            '"version": release_version',
            'release_roots[0]["version"] != release_version',
            "org.opencontainers.image.version=${{ needs.resolve_release.outputs.version }}",
            "org.opencontainers.image.revision=${{ needs.resolve_release.outputs.source_sha }}",
            "render_star_canvas_manifest.py",
            "Upload digest-pinned Canvas release manifest",
        ):
            with self.subTest(required=required):
                self.assertIn(required, workflow)


if __name__ == "__main__":
    unittest.main()
