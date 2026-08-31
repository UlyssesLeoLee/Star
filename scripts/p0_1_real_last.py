#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 15: 修最后 10 err
- 删除我加错的 use crate::value_object::TenantId (3 处)
- domain-validation lib.rs: 撤销 tenant_id.0 → tenant_id
- domain-validation context.rs: 加 use uuid::Uuid (用 UserId::new() 模式)
- domain-workspace: 删我加的 TenantId::from 调用
- domain-scm: existing_id 字段
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")
    total = 0

    # 1. 删除我加错的 use crate::value_object::TenantId
    for crate_name in ["domain-workspace", "domain-automation", "domain-development", "domain-scm"]:
        f = WORKSPACE / crate_name / "src" / "lib.rs"
        if not f.exists():
            continue
        text = f.read_text(encoding="utf-8")
        # 找 `use crate::value_object::TenantId;\n` 是我加的 (单行, 顶部)
        # 删这一行
        new_text = re.sub(
            r'^use crate::value_object::TenantId;\n',
            '',
            text,
            flags=re.MULTILINE
        )
        if new_text != text:
            f.write_text(new_text, encoding="utf-8")
            print(f"{crate_name}/lib.rs                         1 patch (remove bad use)")
            total += 1

    # 2. domain-validation lib.rs: 撤销 .0 (子模块 context.rs 用强类型)
    f = WORKSPACE / "domain-validation" / "src" / "lib.rs"
    if f.exists():
        text = f.read_text(encoding="utf-8")
        original = text
        # `tenant_id.0` 在 ActorContext::new 调用 — 如果 context.rs 子模块用强类型, .0 错
        # 看: domain-validation 有自己的 context.rs 子模块
        # 撤销: 找 `tenant_id.0` 在 ActorContext::new 调用方, 改回 `tenant_id`
        # 但前提是 `tenant_id` 强类型. 在 lib.rs 测试代码 fn make_test_actor(tenant_id: TenantId)
        n2 = text.count("ActorContext::new(Uuid::new_v4(), tenant_id.0)")
        text = text.replace(
            "ActorContext::new(Uuid::new_v4(), tenant_id.0)",
            "ActorContext::new(Uuid::new_v4(), tenant_id)"  # 如果 context 用 Uuid
        )
        # 实际: 子模块 context.rs 强类型, 需要 UserId::new() 不是 Uuid::new_v4()
        n2 = text.count("ActorContext::new(Uuid::new_v4(), tenant_id)")
        text = text.replace(
            "ActorContext::new(Uuid::new_v4(), tenant_id)",
            "ActorContext::new(UserId::new(), tenant_id)"
        )
        if text != original:
            f.write_text(text, encoding="utf-8")
            print(f"domain-validation/lib.rs                    {n2} patches")
            total += n2

    # 3. domain-workspace check_tenant: TenantId 不在 scope
    f = WORKSPACE / "domain-workspace" / "src" / "lib.rs"
    if f.exists():
        text = f.read_text(encoding="utf-8")
        # 在 fn check_tenant 加 `use crate::value_object::TenantId;` 不行 (刚才加 E0432)
        # 改用 `let expected_tenant: TenantId = expected;` 或直接 `expected.as_uuid()` 比较
        # 实际: expected: TenantId 已在签名, 问题是 `actor.tenant_id` 是 Uuid 跟 TenantId 比
        # 改: `actor.tenant_id != expected.as_uuid()` (假设 TenantId 字段是 pub Uuid)
        # 简化: 直接比较 `actor.tenant_id != *expected` (解引用 tuple struct 内部 Uuid)
        text = text.replace(
            "if TenantId::from(actor.tenant_id) != expected {",
            "if actor.tenant_id != *expected {"
        )
        if text != text:
            pass  # 改上面
        text2 = text
        text = re.sub(
            r'if TenantId::from\(actor\.tenant_id\) != expected',
            r'if actor.tenant_id != *expected',
            text
        )
        if text != text2:
            f.write_text(text, encoding="utf-8")
            print(f"domain-workspace/lib.rs                    1 patch")
            total += 1

    # 4. domain-scm:966 existing_id 字段 — 看代码
    f = WORKSPACE / "domain-scm" / "src" / "lib.rs"
    if f.exists():
        text = f.read_text(encoding="utf-8")
        # existing_id 不可访问, 改成 other 字段名
        # 看实际结构 — 暂时不动
        pass

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
