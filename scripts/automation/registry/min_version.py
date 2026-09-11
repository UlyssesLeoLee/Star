"""
min_version.py — MinVersionGate (per DD-MULTICA-RUNTIME-001 §3.3 + Multica version.go:13-178)

8 provider 锁最低 semver, 区分 sentinel error (FR-8 ~ FR-11 per SRS).
"""
import re
from dataclasses import dataclass
from typing import Optional, Tuple


@dataclass(frozen=True)
class MinVersionVerdict:
    provider: str
    detected: Optional[str]
    minimum: Optional[str]
    status: str  # "ok" | "too_old" | "missing" | "no_minimum"
    error: Optional[str] = None


class MinVersionGate:
    """8 provider 最低 semver 校验 (per Multica version.go:13-22)"""

    MIN_VERSIONS = {
        "claude": "2.0.0",
        "codex": "0.100.0",
        "copilot": "1.0.0",
        "grok": "0.2.89",
        "qwen": "0.20.0",
        "dim": "0.3.10",
        "mcode": "0.1.2",
        "zeroclaw": "0.8.0",
    }

    DEV_BUILD_RE = re.compile(r"^v?\d+\.\d+\.\d+-\d+-g[0-9a-fA-F]+")
    SEMVER_RE = re.compile(r"v?(\d+)\.(\d+)\.(\d+)")

    def check(self, provider: str, detected_version: str) -> MinVersionVerdict:
        min_ver = self.MIN_VERSIONS.get(provider)
        if min_ver is None:
            return MinVersionVerdict(provider, detected_version, None, "no_minimum")

        parsed = self._parse_semver(detected_version)
        if parsed is None:
            return MinVersionVerdict(
                provider, detected_version, min_ver, "missing",
                error=f"cannot parse version {detected_version!r}",
            )

        if self._is_dev_build(detected_version):
            return MinVersionVerdict(provider, detected_version, min_ver, "ok")

        min_parsed = self._parse_semver(min_ver)
        if min_parsed is None:
            return MinVersionVerdict(
                provider, detected_version, min_ver, "missing",
                error=f"invalid minimum version {min_ver!r}",
            )

        if parsed < min_parsed:
            return MinVersionVerdict(
                provider, detected_version, min_ver, "too_old",
                error=f"{provider} {detected_version} < minimum {min_ver}",
            )
        return MinVersionVerdict(provider, detected_version, min_ver, "ok")

    def _parse_semver(self, raw: str) -> Optional[Tuple[int, int, int]]:
        if not raw:
            return None
        m = self.SEMVER_RE.search(raw)
        if not m:
            return None
        return (int(m.group(1)), int(m.group(2)), int(m.group(3)))

    def _is_dev_build(self, version: str) -> bool:
        if not version:
            return False
        return bool(self.DEV_BUILD_RE.match(version))
