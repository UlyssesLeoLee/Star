#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""P0 H2-EXT #1: domain-comment ActorContext 收敛

per HANDOFF-ST-001 v0.3 §5.1 H2-EXT #1 (0.05M):
- port.rs: 删 `use crate::context::ActorContext;` (dead import, port trait 用 crate::ActorContext 顶层 re-export)
- service.rs: 删 `use crate::context::ActorContext;` (dead import) + 强类型字段转换 (UserId::from(actor.user_id) etc.)
- lib.rs: 单元测试 30+ 处 TenantId(tid) 包装 + UserId 包装 + author_user_id = Some(uuid::Uuid::new_v4()) → Some(UserId::from(...))
- context.rs: 删 (子模块未暴露, 仅 dead file)

约束:
- 守门 #9: 0 子代理调用
- 守门 #12: docs commit-time 同步
"""
import sys
from pathlib import Path

REPO = Path(r"D:/Star")


def fix_file(path: Path, replacements: list) -> int:
    if not path.exists():
        print(f"  [WARN] {path} 不存在, 跳过")
        return 0
    text = path.read_text(encoding="utf-8")
    original = text
    success = 0
    for old, new in replacements:
        if old not in text:
            continue
        text = text.replace(old, new)
        success += 1
    if text != original:
        path.write_text(text, encoding="utf-8")
    return success


def main() -> int:
    print("[H2-EXT #1 domain-comment]")

    # 1. port.rs: 删 dead import
    print("\n[port.rs] 删 dead import")
    n = fix_file(REPO / "crates/domain-comment/src/port.rs", [
        ("use crate::context::ActorContext;\n", ""),
    ])
    print(f"  替换 {n} 处")

    # 2. service.rs: 删 dead import + 类型转换
    print("\n[service.rs] 删 dead import + 类型转换")
    n = fix_file(REPO / "crates/domain-comment/src/service.rs", [
        ("use crate::context::ActorContext;\n", ""),
        # actor.user_id.into_uuid() -> actor.user_id (已经是 Uuid)
        ("actor_user_id: Some(actor.user_id.into_uuid())",
         "actor_user_id: Some(actor.user_id)"),
        # author_user_id: UserId::from(actor.user_id) 已经是 from 形式, OK
        # actor.tenant_id != expected (TenantId) -> expected.0
        ("if actor.tenant_id != expected {",
         "if actor.tenant_id != expected.0 {"),
        # c.author_user_id != actor.user_id (UserId vs Uuid)
        ("if c.author_user_id != actor.user_id && !actor.is_tenant_admin() {",
         "if c.author_user_id != UserId::from(actor.user_id) && !actor.is_tenant_admin() {"),
        # r.user_id != actor.user_id
        ("if r.user_id != actor.user_id && !actor.is_tenant_admin() {",
         "if r.user_id != UserId::from(actor.user_id) && !actor.is_tenant_admin() {"),
        # actor_user_id: UserId::from(actor.user_id) - 已经 OK
    ])
    print(f"  替换 {n} 处")

    # 3. lib.rs 测试 30+ 处类型包装
    # 看实际 err 模式后, 用 Python 精确替换
    print("\n[lib.rs 测试] 类型包装")
    n = fix_file(REPO / "crates/domain-comment/src/lib.rs", [
        # author_user_id: Some(me) where me = uuid::Uuid::new_v4() -> Some(UserId::from(me))
        ("author_user_id: Some(me),",
         "author_user_id: Some(UserId::from(me)),"),
        # author_user_id: Some(uuid::Uuid::new_v4())
        ("author_user_id: Some(uuid::Uuid::new_v4()),",
         "author_user_id: Some(UserId::from(uuid::Uuid::new_v4())),"),
        # actor_user_id: me -> UserId::from(me)
        ("actor_user_id: me,",
         "actor_user_id: UserId::from(me),"),
        # uploader_user_id: me -> UserId::from(me)
        ("uploader_user_id: me,",
         "uploader_user_id: UserId::from(me),"),
        # user_id: me -> UserId::from(me)
        ("user_id: me,",
         "user_id: UserId::from(me),"),
        # cmd.actor_user_id = me (在测试中)
        # make_cmd 调用: make_cmd(tid) where tid = uuid::Uuid::new_v4() but make_cmd(tid: TenantId)
        # -> make_cmd(TenantId(tid))
    ])
    print(f"  替换 {n} 处")

    # 4. 删 context.rs (子模块未暴露, 但 dead file 留没意义)
    ctx = REPO / "crates/domain-comment/src/context.rs"
    if ctx.exists():
        # context.rs 实际不被引用 (lib.rs 没 pub mod context), 删了没事
        # 但保险起见, 先看是否有其它地方引用
        grep_count = 0
        for src in REPO.glob("crates/domain-comment/src/*.rs"):
            if "context" in src.read_text(encoding="utf-8", errors="replace"):
                if "use crate::context" in src.read_text(encoding="utf-8", errors="replace"):
                    grep_count += 1
        if grep_count == 0:
            ctx.unlink()
            print(f"  [DEL] {ctx.relative_to(REPO)} (无引用)")
        else:
            print(f"  [KEEP] {ctx.relative_to(REPO)} (有 {grep_count} 处引用)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
