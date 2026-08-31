#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #3 fix: domain-project lib.rs 测试 type wrap"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-project/src/lib.rs")

text = REPO.read_text(encoding="utf-8")
original = text
success = 0

# ProjectPolicy::default_for(ProjectId::new(), uuid::Uuid::new_v4())
# -> ProjectPolicy::default_for(ProjectId::new(), TenantId(uuid::Uuid::new_v4()))
replacements = [
    ("ProjectPolicy::default_for(ProjectId::new(), uuid::Uuid::new_v4())",
     "ProjectPolicy::default_for(ProjectId::new(), TenantId(uuid::Uuid::new_v4()))"),
    # 强类型 .0 私有字段 (Project.tenant_id 等) - 需要 as_uuid()
]

for old, new in replacements:
    count = text.count(old)
    if count > 0:
        text = text.replace(old, new)
        success += count
        print(f"  [OK] {old[:60]} x{count}")

REPO.write_text(text, encoding="utf-8")
print(f"\n替换 {success} 处")
