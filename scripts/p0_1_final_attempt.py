#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 13: 子模块 ActorContext 调用最后修
- 子模块的 ActorContext::new 接受强类型 ID
- 撤销所有 'IDENT.0' 改回 'IDENT' (IDENT 是 Uuid 弱类型, .0 错)
- 如果 IDENT 是 TenantId/UserId 强类型, 反过来用 .0
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0

    sub_files = []
    for crate in WORKSPACE.iterdir():
        if not crate.is_dir() or not crate.name.startswith("domain-"):
            continue
        for sub in ("context.rs", "service.rs", "lib.rs"):
            f = crate / "src" / sub
            if f.exists():
                sub_files.append(f)

    for f in sub_files:
        text = f.read_text(encoding="utf-8")
        original = text
        n = 0

        # 1. 撤销 .0 (如果 cargo 报 "field 0 of struct Uuid is private", 说明 IDENT 已经是 Uuid)
        #    IDENT.0 → IDENT
        # 2. 但如果 IDENT 是强类型 (TenantId), .0 是对的
        #    区分: file 是 context.rs 子模块 → IDENT 强类型, 保留 .0
        #          file 是 service.rs/lib.rs → IDENT 看 actor 来源

        is_submodule = f.name == "context.rs" and "pub struct ActorContext" in text

        if is_submodule:
            # 子模块 ActorContext 强类型, .0 正确, Uuid::new_v4() 错
            # 撤销 Uuid::new_v4() → UserId::new() (UserId 在 context.rs 已 import)
            n2 = text.count("ActorContext::new(Uuid::new_v4(),")
            text = text.replace(
                "ActorContext::new(Uuid::new_v4(),",
                "ActorContext::new(UserId::new(),"
            )
            n += n2
        else:
            # lib.rs/service.rs → star_context ActorContext, IDENT 应该是 Uuid
            # 撤销 IDENT.0 → IDENT
            n2 = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\.0\)', text))
            text = re.sub(
                r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\.0\)',
                r'ActorContext::new(Uuid::new_v4(), \1)',
                text
            )
            n += n2

        if not DRY_RUN and text != original:
            f.write_text(text, encoding="utf-8")
        if n > 0:
            print(f"{str(f.relative_to(WORKSPACE)):50} {n} patches")
            total += n

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
