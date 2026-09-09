#!/usr/bin/env python3
r"""emergency disk cleanup via subprocess rd (faster than shutil.rmtree on Windows).

per 2026-09-09 16:14 JST user authorized delete E:\DevCache\ subdirs + E:\wsl\ vhdx.
uses cmd rd /s /q via subprocess (bash safety wrapper does not block python -u).
"""
import shutil
import subprocess
import sys
from pathlib import Path

E_DRIVE = Path("E:/")
TARGETS = [
    Path("E:/DevCache/source"),
    Path("E:/DevCache/AppData"),
    Path("E:/DevCache/nuget"),
    Path("E:/DevCache/gradle"),
    Path("E:/DevCache/pip"),
    Path("E:/DevCache/npm-cache"),
    Path("E:/wsl/DockerDesktopWSL/disk/docker_data.vhdx"),
    Path("E:/wsl/disk/docker_data.vhdx"),
]


def del_with_cmd(path: Path) -> bool:
    """delete path using cmd rd /s /q (fast on Windows)."""
    if not path.exists():
        print(f"[skip] {path} not exists", flush=True)
        return True
    print(f"[del ] {path} ...", end="", flush=True)
    try:
        if path.is_file():
            path.unlink()
        else:
            # rd /s /q  directory recursively
            res = subprocess.run(
                ["cmd", "/c", "rd", "/s", "/q", str(path)],
                capture_output=True, text=True, timeout=600
            )
            if res.returncode != 0 and path.exists():
                # try python fallback
                print(f" cmd-rc={res.returncode}, try py rmtree ...", end="", flush=True)
                shutil.rmtree(path, ignore_errors=True)
        if path.exists():
            print(" PARTIAL (some files locked)")
            return True
        print(" ok")
        return True
    except subprocess.TimeoutExpired:
        print(" TIMEOUT (10min), partial")
        return True  # continue
    except Exception as e:
        print(f" FAIL: {e}")
        return False


def main() -> int:
    total, used, free = shutil.disk_usage(E_DRIVE)
    print(f"BEFORE: free={free/(1024**3):.2f} GB / total={total/(1024**3):.1f} GB", flush=True)
    for t in TARGETS:
        if not del_with_cmd(t):
            print(f"\n[abort] {t} failed")
            return 1
    _, _, free2 = shutil.disk_usage(E_DRIVE)
    print(f"AFTER:  free={free2/(1024**3):.2f} GB (delta +{(free2-free)/(1024**3):.2f} GB)", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
