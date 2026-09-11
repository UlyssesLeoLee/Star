"""
probe.py — RuntimeProbe (per DD-MULTICA-RUNTIME-001 §3.2 + Multica agents_probe.go:155-306)

25 provider 探测, 3 层 fallback (PATH → shell → app bundle).
Per 守门 #5 env 安全: env 只 invoke 不读.
Per 守门 #6: subprocess.run(shell=False).
"""
import re
import subprocess
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import List, Optional

from shell_resolve import LoginShellResolver


@dataclass(frozen=True)
class RuntimeEntry:
    """单 provider 探测结果 (per Multica AgentEntry)"""
    provider: str
    path: Optional[Path]
    version: Optional[str]
    version_parsed: Optional[tuple]
    probe_method: str  # "path" | "shell" | "app_bundle" | "missing"
    detected_at: datetime = field(default_factory=lambda: datetime.now(timezone.utc))
    error: Optional[str] = None


class RuntimeProbe:
    """25 provider 探测 (per FR-1, 阶段 1 仅 5 provider)"""

    # 阶段 1: 5 named provider (后续 v32 阶段 2 扩到 25)
    DEFAULT_CMDS = {
        "claude": "claude",
        "codex": "codex",
        "mcode": "mcode",
        "copilot": "copilot",
        "grok": "grok",
    }

    def __init__(self, shell_resolver: Optional[LoginShellResolver] = None):
        self._shell_resolver = shell_resolver or LoginShellResolver()

    def probe_all(self) -> List[RuntimeEntry]:
        """probe 阶段 1 的 5 provider"""
        return [self.probe_one(p) for p in self.DEFAULT_CMDS]

    def probe_one(self, provider: str) -> RuntimeEntry:
        """单 provider 探测, 3 层 fallback"""
        default_cmd = self.DEFAULT_CMDS[provider]
        # 1. LookPath / where
        path = self._resolve_executable(default_cmd)
        if path:
            version = self._get_version(path)
            return RuntimeEntry(
                provider=provider, path=path, version=version,
                version_parsed=self._parse_semver(version),
                probe_method="path",
            )
        # 2. 绝对路径不 fallback (per Multica agents_probe.go:129-131)
        if "/" in default_cmd or "\\" in default_cmd:
            return RuntimeEntry(provider=provider, path=None, version=None, version_parsed=None, probe_method="missing")
        # 3. login shell 兜底
        shell_path = self._shell_resolver.resolve(default_cmd)
        if shell_path:
            version = self._get_version(shell_path)
            return RuntimeEntry(
                provider=provider, path=shell_path, version=version,
                version_parsed=self._parse_semver(version),
                probe_method="shell",
            )
        return RuntimeEntry(
            provider=provider, path=None, version=None, version_parsed=None, probe_method="missing",
        )

    def _resolve_executable(self, cmd: str) -> Optional[Path]:
        """LookPath 解析, 跨平台 (Windows where / macOS which)"""
        if "win" in sys.platform:
            return self._resolve_windows(cmd)
        return self._resolve_unix(cmd)

    def _resolve_unix(self, cmd: str) -> Optional[Path]:
        import shutil
        found = shutil.which(cmd)
        return Path(found) if found else None

    def _resolve_windows(self, cmd: str) -> Optional[Path]:
        """Windows where"""
        try:
            result = subprocess.run(
                ["where", cmd],
                capture_output=True, text=True, timeout=10,
                shell=False,  # per 守门 #6
            )
            if result.returncode == 0:
                first = result.stdout.strip().split("\n")[0]
                return Path(first) if first else None
        except (subprocess.TimeoutExpired, OSError, FileNotFoundError):
            pass
        return None

    def _get_version(self, path: Path) -> Optional[str]:
        """跑 <cmd> --version 拿版本字符串"""
        try:
            result = subprocess.run(
                [str(path), "--version"],
                capture_output=True, text=True, timeout=10,
                shell=False,  # per 守门 #6
            )
            if result.returncode == 0:
                return (result.stdout or result.stderr or "").strip().split("\n")[0]
        except (subprocess.TimeoutExpired, OSError, FileNotFoundError, PermissionError):
            pass
        return None

    SEMVER_RE = re.compile(r"v?(\d+)\.(\d+)\.(\d+)")

    def _parse_semver(self, raw: Optional[str]) -> Optional[tuple]:
        if not raw:
            return None
        m = self.SEMVER_RE.search(raw)
        if not m:
            return None
        return (int(m.group(1)), int(m.group(2)), int(m.group(3)))
