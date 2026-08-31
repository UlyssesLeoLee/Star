#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #2 fix2: domain-tenant lib.rs 3 test err 修

- 2 处: TenantPolicy::default_for(Uuid) / SecurityPolicy::default_for(Uuid)
  -> TenantPolicy::default_for(TenantId(uuid))
- 1 处: other_t.0 私有字段 (other_t 是 TenantId) -> other_t.as_uuid()
"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-tenant/src/lib.rs")

text = REPO.read_text(encoding="utf-8")
original = text
success = 0

replacements = [
    # default_for 改接受 TenantId
    ("TenantPolicy::default_for(uuid::Uuid::new_v4())",
     "TenantPolicy::default_for(TenantId(uuid::Uuid::new_v4()))"),
    ("SecurityPolicy::default_for(uuid::Uuid::new_v4())",
     "SecurityPolicy::default_for(TenantId(uuid::Uuid::new_v4()))"),
    # other_t.0 私有 -> other_t.as_uuid() (强类型 ID 的 as_uuid 返回 Uuid Copy)
    ("other_t.0", "other_t.as_uuid()"),
]

for old, new in replacements:
    count = text.count(old)
    if count > 0:
        text = text.replace(old, new)
        success += count
        print(f"  [OK] {old[:50]} x{count}")

REPO.write_text(text, encoding="utf-8")
print(f"\n替换 {success} 处")
