#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""H2-EXT #1 fix5: domain-comment lib.rs 测试 Some(me) -> Some(UserId::from(me))"""
from pathlib import Path

REPO = Path(r"D:/Star/crates/domain-comment/src/lib.rs")

text = REPO.read_text(encoding="utf-8")

replacements = [
    # Some(me) -> Some(UserId::from(me))
    ("Some(me),", "Some(UserId::from(me)),"),
    # cmd.author_user_id = Some(me);  ->  Some(UserId::from(me));
    ("Some(me);", "Some(UserId::from(me));"),
    # tenant_id: tid, in EditCommentCommand etc.
    # 测试里 EditCommentCommand { tenant_id: tid, ... }
    # 但 tid 是 Uuid, tenant_id 字段是 TenantId 强类型, 需要 TenantId(tid)
    # 这是 EditCommentCommand / DeleteCommentCommand / AddReactionCommand
    ("tenant_id: tid,", "tenant_id: TenantId(tid),"),
    # tenant_id: tid 结尾 (没逗号, 在大括号末尾)
    ("tenant_id: tid\n", "tenant_id: TenantId(tid)\n"),
    # actor_user_id: me,  -> UserId::from(me),  注意 EditCommentCommand 等
    # EditCommentCommand { actor_user_id: me, ... } - actor_user_id 是 UserId 强类型
    # EditCommentCommand { actor_user_id: me } -> { actor_user_id: UserId::from(me) }
    ("actor_user_id: me,", "actor_user_id: UserId::from(me),"),
    ("actor_user_id: me\n", "actor_user_id: UserId::from(me)\n"),
    # user_id: me,  (in AddReactionCommand)
    ("user_id: me,", "user_id: UserId::from(me),"),
    ("user_id: me\n", "user_id: UserId::from(me)\n"),
    # 其他 me 直接传
    # 但 .0 私有没有了 (上面 fix3 已修)
]

success = 0
for old, new in replacements:
    count = text.count(old)
    if count > 0:
        text = text.replace(old, new)
        success += count
        print(f"  [OK] {old[:50]} x{count}")

REPO.write_text(text, encoding="utf-8")
print(f"\n替换 {success} 处")
