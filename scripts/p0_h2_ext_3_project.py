#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #3: domain-project ActorContext 收敛"""
from pathlib import Path

REPO = Path(r"D:/Star")

# 1. port.rs: 删 dead import
text = (REPO / "crates/domain-project/src/port.rs").read_text(encoding="utf-8")
text = text.replace("use crate::context::ActorContext;\n", "")
(REPO / "crates/domain-project/src/port.rs").write_text(text, encoding="utf-8")
print("[OK] port.rs dead import 删")

# 2. service.rs
text = (REPO / "crates/domain-project/src/service.rs").read_text(encoding="utf-8")
replacements = [
    ("use crate::context::ActorContext;\n", ""),
    # actor.tenant_id != expected
    ("if actor.tenant_id != expected {", "if actor.tenant_id != expected.0 {"),
    # actor.user_id 期望 UserId
    ("actor_user_id: Some(actor.user_id),", "actor_user_id: Some(UserId::from(actor.user_id)),"),
    # viewer.tenant_id
    ("viewer.tenant_id != project.tenant_id {", "viewer.tenant_id != project.tenant_id.0 {"),
]
success = 0
for old, new in replacements:
    if old in text:
        text = text.replace(old, new)
        success += 1
(REPO / "crates/domain-project/src/service.rs").write_text(text, encoding="utf-8")
print(f"[OK] service.rs {success} 处替换")

# 3. 删 context.rs
ctx = REPO / "crates/domain-project/src/context.rs"
if ctx.exists():
    ctx.unlink()
    print(f"[DEL] {ctx.relative_to(REPO)}")
