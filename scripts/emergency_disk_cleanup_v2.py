#!/usr/bin/env python3
r"""emergency disk cleanup v2: skip E:\DevCache\source (Git lock), delete others + 2 vhdx.

file deletion does NOT require target file's size in free space (just dir entry).
skip source (5 git repos with active lock files).
"""
import shutil
import subprocess
import sys
from pathlib import Path

TARGETS = [
    Path("E:/DevCache/AppData"),
    Path("E:/DevCache/nuget"),
    Path("E:/DevCache/gradle"),
    Path("E:/DevCache/pip"),
    Path("E:/DevCache/npm-cache"),
    Path("E:/wsl/DockerDesktopWSL/disk/docker_data.vhdx"),
    Path("E:/wsl/disk/docker_data.vhdx"),
]


def del_path(p: Path) -> bool:
    if not p.exists():
        print(f"[skip] {p} not exists", flush=True)
        return True
    print(f"[del ] {p} ...", end="", flush=True)
    try:
        if p.is_file():
            p.unlink()
        else:
            res = subprocess.run(
                ["cmd", "/c", "rd", "/s", "/q", str(p)],
                capture_output=True, text=True, timeout=300
            )
            if p.exists():
                shutil.rmtree(p, ignore_errors=True)
        if p.exists():
            print(" PARTIAL (some files locked)")
        else:
            print(" ok")
        return True
    except subprocess.TimeoutExpired:
        print(" TIMEOUT")
        return True
    except Exception as e:
        print(f" FAIL: {e}")
        return False


def main() -> int:
    total, used, free = shutil.disk_usage("E:/")
    print(f"BEFORE: free={free/(1024**3):.2f} GB", flush=True)
    for t in TARGETS:
        del_path(t)
    _, _, free2 = shutil.disk_usage("E:/")
    print(f"AFTER:  free={free2/(1024**3):.2f} GB (delta +{(free2-free)/(1024**3):.2f} GB)", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
