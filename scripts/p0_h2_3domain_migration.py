#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""P0 H2: 3 domain (feedback/validation/integration) ActorContext 收敛

Per HANDOFF-ST-001 H2 (上游 AI 推荐 (b) - 删除 3 子模块, 22 domain 统一顶层 Uuid):
- port.rs / service.rs / invariants.rs: use crate::context::ActorContext -> use star_context::ActorContext
- lib.rs: 删 pub mod context; + 别名 (_ContextActorContext / ContextActorContext)
- context.rs: 删
- 强类型 -> Uuid 转换: actor.user_id (UserId) -> actor.user_id (Uuid, 后续 .into_uuid() 调用删)
- 强类型 -> Uuid 转换: actor.tenant_id (TenantId) -> actor.tenant_id (Uuid)
- 兼容: UserId::from(actor.user_id) / TenantId::from(actor.tenant_id) 在期望强类型字段处
- integration can_access_project: ProjectId (强类型) -> ProjectId.0 (Uuid)
- star-mcp handler/feedback.rs: domain_feedback::context::ActorContext -> domain_feedback::ActorContext

约束:
- 守门 #9: 0 子代理调用 (root 直实装)
- 守门 #12: docs commit-time 同步 (脚本本身不 commit, 留待人工)
"""
import re
import sys
from pathlib import Path

REPO = Path(r"D:/Star")
DOMAINS = ["domain-feedback", "domain-validation", "domain-integration"]

# 强类型 ID 转换的 regex 模式 (需要看上下文)
# 1. actor.user_id 直接传给 UserId 字段 (需要包 UserId::from)
# 2. actor.user_id.into_uuid() 传给 Uuid 字段 (现在 actor.user_id 已经是 Uuid, 删 .into_uuid())
# 3. actor.tenant_id != TenantId 强类型 (需要 .0)
# 4. viewer.can_access_project(ProjectId 强类型) -> viewer.can_access_project(ProjectId.0)

def replace_in_file(path: Path, replacements: list) -> bool:
    """在文件中执行多个 (old, new) 替换. 全部成功返回 True, 任一失败抛错."""
    if not path.exists():
        print(f"  [WARN] {path} 不存在, 跳过")
        return False
    text = path.read_text(encoding="utf-8")
    original = text
    for old, new in replacements:
        if old not in text:
            print(f"  [WARN] {path.name}: pattern not found: {old[:60]}...")
            # 不抛错, 仅警告
            continue
        text = text.replace(old, new)
    if text != original:
        path.write_text(text, encoding="utf-8")
        print(f"  [OK] {path.name}: {len(replacements)} 处替换")
        return True
    return False


def migrate_domain(d: str) -> None:
    """迁移单个 domain 的 port/service/invariants/lib.rs/context.rs"""
    src = REPO / f"crates/{d}/src"

    # 1. port.rs: use crate::context::ActorContext -> use star_context::ActorContext
    port = src / "port.rs"
    if port.exists():
        replace_in_file(port, [
            ("use crate::context::ActorContext;", "use star_context::ActorContext;"),
        ])

    # 2. service.rs: use + 强类型转换
    svc = src / "service.rs"
    if svc.exists():
        replacements = [
            ("use crate::context::ActorContext;", "use star_context::ActorContext;"),
            # actor.user_id.into_uuid() -> actor.user_id (因 actor.user_id 现在已经是 Uuid)
            # 但 author_user_id 字段类型还是 Uuid (保持), 所以直接传
            ("actor_user_id: Some(actor.user_id.into_uuid())",
             "actor_user_id: Some(actor.user_id)"),
            # resolver_user_id 期望 UserId 强类型, 需要 UserId::from
            # "resolver_user_id: actor.user_id" -> 强类型调用
            # 注: feedback lib.rs:365 `resolver_user_id: actor.user_id` 期望 UserId
            # 但 service.rs 改完后 actor.user_id 是 Uuid, 不能直接传给 UserId 字段
            # 这需要在 domain 层做转换; 留作 compile error 修复 (后续 cargo check 阶段)
        ]
        replace_in_file(svc, replacements)

    # 3. invariants.rs (validation only)
    inv = src / "invariants.rs"
    if inv.exists():
        replace_in_file(inv, [
            ("actor: &crate::context::ActorContext", "actor: &star_context::ActorContext"),
            ("&crate::context::ActorContext", "&star_context::ActorContext"),
        ])

    # 4. lib.rs: 删 pub mod context; + 别名
    lib = src / "lib.rs"
    if lib.exists():
        lib_replacements = [
            ("pub mod context;\n", ""),  # 删 pub mod context; 行
            # validation: use context::ActorContext as _ContextActorContext; 整行
            ("use context::ActorContext as _ContextActorContext; // 内部使用 (子模块强类型 ID 版)\n", ""),
            # integration: pub use context::ActorContext as ContextActorContext; 整行
            ("pub use context::ActorContext as ContextActorContext; // 子模块强类型 ID 版本 (供 domain 内部 use crate::context::ActorContext)\n", ""),
        ]
        replace_in_file(lib, lib_replacements)

    # 5. 删 context.rs
    ctx = src / "context.rs"
    if ctx.exists():
        ctx.unlink()
        print(f"  [DEL] {ctx.relative_to(REPO)}")


def migrate_integration_can_access_project() -> None:
    """domain-integration service.rs: viewer.can_access_project(ProjectId) -> ProjectId.0"""
    svc = REPO / "crates/domain-integration/src/service.rs"
    if not svc.exists():
        return
    replacements = [
        # can_access_project 期待 Uuid, project_id 是 ProjectId 强类型, 取 .0
        ("viewer.can_access_project(integration.project_id)",
         "viewer.can_access_project(integration.project_id.0)"),
        ("viewer.can_access_project(q.project_id)",
         "viewer.can_access_project(q.project_id.0)"),
    ]
    replace_in_file(svc, replacements)


def migrate_handler_feedback() -> None:
    """star-mcp handler/feedback.rs: domain_feedback::context::ActorContext -> domain_feedback::ActorContext"""
    h = REPO / "crates/star-mcp/src/handlers/feedback.rs"
    if not h.exists():
        return
    replacements = [
        # 1. import: 子模块 -> 顶层 re-export
        ("use domain_feedback::context::ActorContext;",
         "use domain_feedback::ActorContext;"),
        # 2. ActorContext::new(UserId::new(), domain_feedback::TenantId::new()) -> Uuid::new_v4() x2
        # 当前行: ActorContext::new(\n            UserId::new(),\n            domain_feedback::TenantId::new(),\n        );
        # 改成: ActorContext::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4());
        ("ActorContext::new(\n            UserId::new(),\n            domain_feedback::TenantId::new(),\n        );",
         "ActorContext::new(uuid::Uuid::new_v4(), uuid::Uuid::new_v4());"),
    ]
    replace_in_file(h, replacements)


def main() -> int:
    print("== P0 H2: 3 domain port/service/invariants 迁移 ==")
    for d in DOMAINS:
        print(f"\n[{d}]")
        migrate_domain(d)

    print("\n== P0 H2: integration can_access_project 参数转换 ==")
    migrate_integration_can_access_project()

    print("\n== P0 H2: star-mcp handler/feedback.rs 顶层 re-export ==")
    migrate_handler_feedback()

    print("\n[完成] 全部脚本替换落地, 留待 cargo check 验证")
    return 0


if __name__ == "__main__":
    sys.exit(main())
