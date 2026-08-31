#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 14: 最后 33 err
- ActorContext::new(Uuid::new_v4(), IDENT) → IDENT.0 (IDENT 是强类型 ID)
- ActorContext::new(IDENT_USER, IDENT_TENANT) → IDENT_USER.0, IDENT_TENANT.0
- ActorContext::new(IDENT_USER, ...) → IDENT_USER.0
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0
    for crate in WORKSPACE.iterdir():
        if not crate.is_dir() or not crate.name.startswith("domain-"):
            continue
        for f in (crate / "src").rglob("*.rs"):
            text = f.read_text(encoding="utf-8")
            original = text
            n = 0
            # 1. ActorContext::new(Uuid::new_v4(), IDENT) → .0 (排除 IDENT 已是 .0/.uuid/Uuid::*)
            text2 = re.sub(
                r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\)(?!\.0)',
                r'ActorContext::new(Uuid::new_v4(), \1.0)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\)(?!\.0)', text))
                n += n2
                text = text2
            # 2. ActorContext::new(IDENT_USER, IDENT_TENANT) → 2 个 .0
            #    模式: ActorContext::new(IDENT1, IDENT2) (2 个标识符, 不是 Uuid::* 或 .0)
            text2 = re.sub(
                r'ActorContext::new\((?!(?:Uuid|UserId|TenantId)\w*\(\))(\w+),\s*(\w+)\)(?!\.0)',
                lambda m: f'ActorContext::new({m.group(1)}.0, {m.group(2)}.0)'
                if m.group(1) not in ('Uuid::new_v4',) and 'Uuid::' not in m.group(1)
                and 'Uuid::' not in m.group(2) and '.' not in m.group(1) and '.' not in m.group(2)
                else m.group(0),
                text
            )
            if text2 != text:
                # 简化计数: 用 diff
                n2 = len(re.findall(r'ActorContext::new\(\w+,\s*\w+\)(?!\.0)', text))
                # 排除已匹配的 (Uuid::new_v4)
                n2 = len(re.findall(r'ActorContext::new\((?!Uuid::new_v4)\w+,\s*\w+\)(?!\.0)', text))
                n += n2
                text = text2
            # 3. domain-workspace check_tenant 缺 TenantId import — 实际上 body 内有 TenantId::from
            # 修: 加 use crate::value_object::TenantId;
            if "fn check_tenant" in text and "TenantId::from(actor.tenant_id)" in text:
                if "use crate::value_object::TenantId" not in text and "use value_object::TenantId" not in text:
                    m = re.search(r'use\s+', text)
                    if m:
                        text = text[:m.start()] + "use crate::value_object::TenantId;\n" + text[m.start():]
                        n += 1
            # 4. domain-feedback context.rs line 26: 函数定义没问题, 是 caller 错
            #    实际是 service.rs 调用 ActorContext::new(Uuid::new_v4(), tenant_id.0) 但 context.rs 期望 TenantId 强类型
            #    修: 撤销 .0
            if f.name == "context.rs" and "pub struct ActorContext" in text and "pub fn new(user_id: UserId" in text:
                # 子模块, 期望强类型, 撤销 .0
                text2 = re.sub(
                    r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\.0\)',
                    r'ActorContext::new(UserId::new(), \1)',
                    text
                )
                if text2 != text:
                    n2 = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\.0\)', text))
                    n += n2
                    text = text2

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{str(f.relative_to(WORKSPACE)):50} {n} patches")
                total += n

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
