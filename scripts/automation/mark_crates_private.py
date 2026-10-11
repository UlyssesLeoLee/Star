#!/usr/bin/env python3
"""为 workspace crate 补 `publish = false` (dry-run 默认).

为什么需要:
    cargo-deny 依据 `publish` 字段判定 crate 是否 private。`allow-wildcard-paths`
    只对 **private** crate 生效, 而本仓 100+ crate 均未声明 publish, 故一律被
    cargo-deny 视为 public, 导致:
      1. `bans.wildcards` 把 `foo = { path = "../foo" }` (无 version 字段) 判为版本通配
      2. `allow-wildcard-paths = true` 对其不生效
    实测 (2026-10-04): 60+ crate 因此报 error, 迫使 wildcards 降级为 warn。
    补齐 publish = false 后可恢复 wildcards = "deny" 的强门禁。

安全性:
    仅在 crate 未声明 publish 时插入, 不覆盖既有值。
    Star 全部 crate 均为仓内 domain-*/star-* 内部 crate, 无 crates.io 发布意图
    (per workspace.package 未配置 publish, 且无任何 crate 声明 publish)。

用法:
    python scripts/automation/mark_crates_private.py            # dry-run, 只报告
    python scripts/automation/mark_crates_private.py --apply    # 实际写入
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]

# 已有 publish 声明 (任意形式) 则跳过
_HAS_PUBLISH = re.compile(r"(?m)^\s*publish\s*=")
# [package] 段起始
_PACKAGE_HDR = re.compile(r"(?m)^\[package\]\s*$")

NOTE = (
    "# 2026-10-05 (per cargo-deny 实跑): 标记为 private, 使 cargo-deny 的\n"
    "# allow-wildcard-paths 生效, 并允许恢复 bans.wildcards = \"deny\"。\n"
    "# 本仓 crate 无 crates.io 发布意图。详见 PHASE-CD-SELECTION-REPORT.md §3 缺口 #6。\n"
    "publish = false\n"
)


def find_member_manifests() -> list[Path]:
    """遍历仓库内全部 member manifest.

    不只扫 crates/ —— 实测存在 crates/star-desktop/src-tauri/ (嵌套) 以及
    bff/、tools/aci-emitter/ 等 crates/ 之外的 member, 只扫 crates/*/Cargo.toml
    会漏掉它们, 导致 cargo-deny 仍判定其为 public crate。
    """
    out: list[Path] = []
    root_manifest = REPO_ROOT / "Cargo.toml"
    for manifest in REPO_ROOT.rglob("Cargo.toml"):
        if manifest == root_manifest:
            continue
        parts = set(manifest.relative_to(REPO_ROOT).parts)
        if "target" in parts or "node_modules" in parts:
            continue
        text = manifest.read_text(encoding="utf-8", errors="replace")
        if _PACKAGE_HDR.search(text):
            out.append(manifest)
    return sorted(out)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--apply", action="store_true", help="实际写入 (默认 dry-run)")
    args = ap.parse_args()

    if not (REPO_ROOT / "Cargo.toml").is_file():
        sys.stderr.write(f"workspace root not found: {REPO_ROOT}\n")
        return 2

    manifests = find_member_manifests()
    changed: list[str] = []
    skipped_existing: list[str] = []

    for manifest in manifests:
        text = manifest.read_text(encoding="utf-8")
        if _HAS_PUBLISH.search(text):
            skipped_existing.append(manifest.parent.name)
            continue
        m = _PACKAGE_HDR.search(text)
        if not m:
            continue
        insert_at = m.end() + 1
        new_text = text[:insert_at] + NOTE + text[insert_at:]
        changed.append(str(manifest.relative_to(REPO_ROOT)))
        if args.apply:
            nl = "\r\n" if "\r\n" in text else "\n"
            manifest.write_text(
                new_text.replace("\r\n", "\n").replace("\n", nl), encoding="utf-8"
            )

    mode = "APPLY" if args.apply else "DRY-RUN"
    print(f"=== mark_crates_private.py [{mode}] ===")
    print(f"member manifests found : {len(manifests)}")
    print(f"to add publish         : {len(changed)}")
    print(f"already has it         : {len(skipped_existing)}")
    if changed:
        # 高亮 crates/ 之外的 member —— 这是上一版脚本的盲区
        outside = [c for c in changed if not c.startswith("crates" + "/") and "/crates/" not in c]
        for c in outside:
            print(f"  [outside crates/] {c}")
        print(f"sample: {', '.join(changed[:6])}{' ...' if len(changed) > 6 else ''}")
    if not args.apply and changed:
        print("\n(dry-run: 未写入。确认无误后加 --apply)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
