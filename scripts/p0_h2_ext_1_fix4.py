#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #1 fix4: domain-comment lib.rs 测试 Some(Uuid) -> Some(UserId::from(...))"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-comment/src/lib.rs")

text = REPO.read_text(encoding="utf-8")

# 模式: Some(uuid::Uuid::new_v4()) -> Some(UserId::from(uuid::Uuid::new_v4()))
# 但只针对 author_user_id / user_id 字段, 不改 actor_user_id (那个直接 me)
replacements = [
    # cmd.author_user_id = Some(uuid::Uuid::new_v4()) -> Some(UserId::from(uuid::Uuid::new_v4()))
    ("Some(uuid::Uuid::new_v4())",
     "Some(UserId::from(uuid::Uuid::new_v4()))"),
]

success = 0
for old, new in replacements:
    count = text.count(old)
    if count > 0:
        text = text.replace(old, new)
        success += count

REPO.write_text(text, encoding="utf-8")
print(f"替换 {success} 处")
