#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 16: ENDGAME - 修最后 8 err
- domain-board: 删 use crate::value_object::TenantId (本地 define)
- domain-workspace 582: TenantId::from(actor.tenant_id) → actor.tenant_id != *expected
- domain-integration 91, 313: ActorContext::new(Uuid::new_v4(), tenant_id.0) → (UserId::new(), tenant_id)
- domain-validation 100, 104 (lib.rs): 已改 UserId::new(), tenant_id 但漏 .0 撤销
- domain-scm 966: guard.get → 看实际
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0

    # 1. domain-board: 删 use crate::value_object::TenantId (line 37, 我加错)
    f = WORKSPACE / "domain-board" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    if "use crate::value_object::TenantId;" in text and "define_uuid_id!(TenantId)" in text:
        # 内部已有, 删外部 use
        text = text.replace("use crate::value_object::TenantId;\n", "")
        f.write_text(text, encoding="utf-8")
        print(f"domain-board/lib.rs                         1 patch (remove dup use)")
        total += 1

    # 2. domain-workspace 582: check_tenant body 改成用 *expected
    f = WORKSPACE / "domain-workspace" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    if "if TenantId::from(actor.tenant_id) != expected {" in text:
        text = text.replace(
            "if TenantId::from(actor.tenant_id) != expected {",
            "if actor.tenant_id != *expected {"
        )
        f.write_text(text, encoding="utf-8")
        print(f"domain-workspace/lib.rs                    1 patch (check_tenant)")
        total += 1

    # 3. domain-integration lib.rs: 测试代码 (line 91, 313)
    #    ActorContext::new(Uuid::new_v4(), tenant_id.0) → ActorContext::new(UserId::new(), tenant_id)
    f = WORKSPACE / "domain-integration" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    original = text
    text = re.sub(
        r'ActorContext::new\(Uuid::new_v4\(\),\s*(\w+)\.0\)',
        r'ActorContext::new(UserId::new(), \1)',
        text
    )
    if text != original:
        f.write_text(text, encoding="utf-8")
        n = len(re.findall(r'ActorContext::new\(Uuid::new_v4\(\),\s*\w+\.0\)', original))
        print(f"domain-integration/lib.rs                    {n} patches")
        total += n

    # 4. domain-validation lib.rs: 撤销之前的 .0
    f = WORKSPACE / "domain-validation" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    # 测试代码 ActorContext::new 调用 — 之前改错, 应该用本地子模块强类型
    # context.rs 子模块本地, 期望 (UserId, TenantId)
    # 看 line 100/104
    # 之前脚本改成 ActorContext::new(UserId::new(), tenant_id) — 但 tenant_id 是 TenantId 强类型
    # 应该是对的: UserId::new() 返回 UserId, tenant_id 是 TenantId
    # 那 cargo 之前报 E0616 field 0 of struct Uuid is private — tenant_id.0
    # 让我看现在 line 100/104 是什么
    # 实际我修了 lib.rs 100/104 成 (UserId::new(), tenant_id) — 应该是 OK
    # 但之前 24→32 增加了, 让我看 cargo 报什么
    pass  # 不动

    # 5. domain-scm 966: 看实际
    f = WORKSPACE / "domain-scm" / "src" / "lib.rs"
    text = f.read_text(encoding="utf-8")
    # existing_id 字段 — 暂不动
    pass

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
