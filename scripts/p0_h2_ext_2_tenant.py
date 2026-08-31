#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #2: domain-tenant ActorContext 收敛"""
from pathlib import Path

REPO = Path(r"D:/Star")

# 1. port.rs: 删 dead import
text = (REPO / "crates/domain-tenant/src/port.rs").read_text(encoding="utf-8")
text = text.replace("use crate::context::ActorContext;\n", "")
(REPO / "crates/domain-tenant/src/port.rs").write_text(text, encoding="utf-8")
print("[OK] port.rs dead import 删")

# 2. service.rs: dead import + 类型转换
text = (REPO / "crates/domain-tenant/src/service.rs").read_text(encoding="utf-8")
replacements = [
    ("use crate::context::ActorContext;\n", ""),
    # actor.tenant_id != expected (TenantId) -> expected.0
    ("if actor.tenant_id != expected {", "if actor.tenant_id != expected.0 {"),
    # actor.user_id 是 Uuid, 期望 UserId (强类型) -> UserId::from(actor.user_id)
    ("actor_user_id: Some(actor.user_id),", "actor_user_id: Some(UserId::from(actor.user_id)),"),
    # viewer.tenant_id != id (id 是 TenantId) -> id.0
    ("viewer.tenant_id != id {", "viewer.tenant_id != id.0 {"),
]
success = 0
for old, new in replacements:
    if old in text:
        text = text.replace(old, new)
        success += 1
(REPO / "crates/domain-tenant/src/service.rs").write_text(text, encoding="utf-8")
print(f"[OK] service.rs {success} 处替换")

# 3. 删 context.rs
ctx = REPO / "crates/domain-tenant/src/context.rs"
if ctx.exists():
    ctx.unlink()
    print(f"[DEL] {ctx.relative_to(REPO)}")
