#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 7: 修剩余 70 err
- actor_user_id: X.user_id / granted_by: X.user_id → 包 UserId::from
- actor.is_project_admin() → has_role+is_platform_admin
- 测试 ActorContext 缺字段
- TenantId::from 在 supporting crate 不可用
- HttpClient::new 类型错
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DRY_RUN = "--apply" not in sys.argv


def main() -> int:
    print("APPLY" if not DRY_RUN else "DRY-RUN")

    targets = [
        "domain-permission", "domain-planning", "domain-work-item",
        "application", "infrastructure", "api",
        "domain-workspace", "domain-local-runtime",
    ]

    total = 0
    for crate in targets:
        src_dir = WORKSPACE / crate / "src"
        if not src_dir.exists():
            continue
        # 找所有 .rs 文件
        for f in src_dir.rglob("*.rs"):
            text = f.read_text(encoding="utf-8")
            original = text
            n = 0

            # 1. actor_user_id: IDENT.user_id → UserId::from(IDENT.user_id)
            # 排除已经包过的
            text2 = re.sub(
                r'actor_user_id:\s*(?!UserId::from\()(\w+)\.user_id\b',
                r'actor_user_id: UserId::from(\1.user_id)',
                text
            )
            if text2 != text:
                n += text.count("actor_user_id: ") - text2.count("actor_user_id: UserId::from(")
                # 重新数 (可能不准, 用 before/after diff)
                n2 = len(re.findall(r'actor_user_id:\s*(?!UserId::from\()\w+\.user_id', text))
                n = n2
                text = text2

            # 2. granted_by: IDENT.user_id → UserId::from(IDENT.user_id)
            text2 = re.sub(
                r'granted_by:\s*(?!UserId::from\()(\w+)\.user_id\b',
                r'granted_by: UserId::from(\1.user_id)',
                text
            )
            if text2 != text:
                n2 = len(re.findall(r'granted_by:\s*(?!UserId::from\()\w+\.user_id', text))
                n += n2
                text = text2

            # 3. actor.is_project_admin() → has_role || is_platform_admin
            n2 = text.count("actor.is_project_admin()")
            text = text.replace(
                "actor.is_project_admin()",
                "(actor.has_role(\"project_admin\") || actor.is_platform_admin)"
            )
            n += n2

            # 4. ActorContext { ... } 测试代码补字段
            # 模式: ActorContext { 后某行有 user_id: ..., tenant_id: ...
            # 找 user_id: 后的 } 之前插入 is_local_runtime + is_platform_admin
            # 简化: 在 device_id: None, 后面加
            n2 = 0
            # 在 supporting crate (api / application / infrastructure) 测试代码中
            if crate in ("api", "application", "infrastructure"):
                # 找 ActorContext { user_id, tenant_id, device_id: None, project_ids: ..., roles: ... }
                # 在 device_id: None 之后加 is_local_runtime + is_platform_admin
                if "device_id: None," in text and "is_local_runtime: false" not in text:
                    text = text.replace(
                        "device_id: None,",
                        "device_id: None, is_local_runtime: false, is_platform_admin: false,",
                    )
                    n2 += 1
                # 替换 TenantId::from(actor.tenant_id) → 直接 actor.tenant_id.is_nil() 检查
                if "TenantId::from(actor.tenant_id).is_nil()" in text:
                    text = text.replace(
                        "TenantId::from(actor.tenant_id).is_nil()",
                        "actor.tenant_id.is_nil()"
                    )
                    n2 += 1
            n += n2

            if not DRY_RUN and text != original:
                f.write_text(text, encoding="utf-8")
            if n > 0:
                print(f"{crate+'/'+f.name:50} {n} patches")
                total += n

    print(f"\nTotal: {total}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
