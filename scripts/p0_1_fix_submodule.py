#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 12: 子模块 (context.rs) 内的 ActorContext::new revert
- 子模块有自己的 ActorContext (强类型 ID, lib.rs 迁移后并存)
- 子模块调用 ActorContext::new(Uuid::new_v4(), IDENT.0) 错
- 改回 ActorContext::new(UserId::new(), IDENT)
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")

    # 子模块文件路径 (每个 domain 的 context.rs)
    sub_files = []
    for crate in WORKSPACE.iterdir():
        if not crate.is_dir() or not crate.name.startswith("domain-"):
            continue
        for sub in ("context.rs", "service.rs"):
            f = crate / "src" / sub
            if f.exists():
                sub_files.append(f)

    total = 0
    for f in sub_files:
        text = f.read_text(encoding="utf-8")
        original = text

        # 子模块内 ActorContext::new(Uuid::new_v4(), IDENT.0) → ActorContext::new(UserId::new(), IDENT)
        # 模式: 上下文里有 pub struct ActorContext (本地版本)
        if "pub struct ActorContext" in text:
            # 是本地版本, revert
            text2 = re.sub(
                r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\.0\)',
                r'ActorContext::new(UserId::new(), \1)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\.0\)', text))
                text = text2
                total += n2
                print(f"{str(f.relative_to(WORKSPACE)):50} {n2} patches (revert submodule)")
        else:
            # 不是本地版本, 用 star_context 但要 import uuid
            if "use uuid::Uuid;" not in text and "Uuid::new_v4" in text:
                m = re.search(r'use\s+', text)
                if m:
                    text = text[:m.start()] + "use uuid::Uuid;\n" + text[m.start():]
                    print(f"{str(f.relative_to(WORKSPACE)):50} 1 patch (add use uuid::Uuid)")
                    total += 1

        if text != original:
            f.write_text(text, encoding="utf-8")

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
