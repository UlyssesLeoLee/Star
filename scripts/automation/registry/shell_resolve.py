"""
shell_resolve.py — LoginShellResolver (per DD-MULTICA-RUNTIME-001 §3.5 + Multica agents_probe.go:30-50)

macOS GUI daemon 兜底 PATH 解析, 30min TTL 缓存.
Per 守门 #6 PowerShell only: subprocess.run(shell=False).
"""
import os
import subprocess
import sys
import time
from pathlib import Path
from typing import Dict, Optional


class LoginShellResolver:
    """macOS GUI daemon 兜底 PATH 解析 (per Multica agents_probe.go:30-50)"""

    SHELL_RESOLVE_TTL = 30 * 60  # 30min (per FR-6)

    def __init__(self):
        self._cache: Dict[str, str] = {}
        self._cache_key: str = ""
        self._cached_at: float = 0

    def _env_key(self) -> str:
        """fingerprint env (per FR-7)"""
        return "\x00".join([
            os.environ.get("PATH", ""),
            os.environ.get("SHELL", ""),
            os.environ.get("HOME", ""),
        ])

    def resolve(self, cmd: str) -> Optional[Path]:
        """解析 bare command name → 绝对路径"""
        key = self._env_key()
        now = time.time()
        if self._cache and self._cache_key == key and (now - self._cached_at) < self.SHELL_RESOLVE_TTL:
            if cmd in self._cache:
                return Path(self._cache[cmd])
        resolved = self._resolve_via_shell([cmd])
        if resolved is None:
            resolved = {}
        self._cache = resolved
        self._cache_key = key
        self._cached_at = now
        return Path(self._cache[cmd]) if cmd in self._cache else None

    def _resolve_via_shell(self, cmds: list) -> Optional[Dict[str, str]]:
        """fork 用户 login shell, 跑 which (per FR-5)"""
        # Windows 暂不实现 (per FR-7 平台限制)
        if "win" in sys.platform:
            return None
        try:
            shell = os.environ.get("SHELL", "/bin/sh")
            result = subprocess.run(
                [shell, "-c", "which " + " ".join(cmds)],
                capture_output=True, text=True, timeout=10,
                shell=False,  # per 守门 #6
            )
            if result.returncode != 0:
                return None
            paths = result.stdout.strip().split("\n")
            return {cmd: path for cmd, path in zip(cmds, paths) if path and path != cmd}
        except (subprocess.TimeoutExpired, OSError, FileNotFoundError):
            return None
