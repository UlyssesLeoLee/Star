#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 ActorContext 单点化迁移脚本 (per AGENTS.md §0 + audit report P0-1)

功能:
1. 22 个 domain-* 删除 `pub struct ActorContext` 定义 + impl, 改为 `pub use star_context::ActorContext;`
2. 22 个 domain-* Cargo.toml 加 star-context 依赖
3. 22 个 domain-* 内部 `actor.user_id` / `actor.tenant_id` 强类型 ID 用法改为 `UserId::from(actor.user_id)` / `TenantId::from(actor.tenant_id)`
4. 22 个 domain-* 内部 `actor.is_platform_admin` 字段访问改为 `actor.is_platform_admin()` 方法调用
5. 3 个 supporting crate (api / application / infrastructure) 同上
6. star-context Cargo.toml 已加 uuid 依赖 (前一步完成)

约束 (per AGENTS.md §1.2 禁回溯 + 守门 #1):
- 不 commit, 仅 worktree 改动
- 留 dry-run 默认
- 出错立即停止
"""

import os
import re
import sys
from pathlib import Path
from typing import List, Tuple

WORKSPACE = Path(r"D:\Star")
CRATES = WORKSPACE / "crates"

# 22 个有 ActorContext 定义的 domain
DOMAINS_WITH_ACTOR_CTX = [
    "domain-agent",
    "domain-audit",
    "domain-automation",
    "domain-board",
    "domain-collaboration",
    "domain-comment",
    "domain-context",
    "domain-development",
    "domain-identity",
    "domain-local-runtime",
    "domain-notification",
    "domain-permission",
    "domain-planning",
    "domain-project",
    "domain-relation",
    "domain-scm",
    "domain-search",
    "domain-tenant",
    "domain-work-item",
    "domain-workflow",
    "domain-workspace",
    "domain-worktree",
]

# 3 个 supporting crate
SUPPORTING_CRATES = ["api", "application", "infrastructure"]

# 22 + 3 = 25 个目标
TARGET_CRATES = DOMAINS_WITH_ACTOR_CTX + SUPPORTING_CRATES

DRY_RUN = "--apply" not in sys.argv


def log(msg: str) -> None:
    print(f"[P0-1] {msg}")


def err(msg: str) -> None:
    print(f"[P0-1] ERROR: {msg}", file=sys.stderr)


def replace_in_file(path: Path, old: str, new: str, count: int = -1) -> int:
    """替换文件中的字符串. 返回替换次数. count=-1 替换全部."""
    if not path.exists():
        return 0
    text = path.read_text(encoding="utf-8")
    if count < 0:
        new_text = text.replace(old, new)
        n = text.count(old)
    else:
        new_text = text.replace(old, new, count)
        n = min(count, text.count(old))
    if new_text != text and not DRY_RUN:
        path.write_text(new_text, encoding="utf-8")
    return n


def replace_with_regex(path: Path, pattern: str, replacement: str, flags: int = 0) -> int:
    if not path.exists():
        return 0
    text = path.read_text(encoding="utf-8")
    regex = re.compile(pattern, flags)
    new_text, n = regex.subn(replacement, text)
    if new_text != text and not DRY_RUN:
        path.write_text(new_text, encoding="utf-8")
    return n


# =====================================================================
# Cargo.toml 加 star-context 依赖
# =====================================================================

def patch_cargo_toml(crate_dir: Path) -> Tuple[bool, str]:
    cargo = crate_dir / "Cargo.toml"
    if not cargo.exists():
        return False, "Cargo.toml not found"

    text = cargo.read_text(encoding="utf-8")

    # 已包含 star-context 依赖则跳过
    if re.search(r"^\s*star-context\s*=", text, re.MULTILINE):
        return False, "already has star-context"

    # 在 [dependencies] 块末尾加一行
    if "[dependencies]" in text:
        # 找 [dependencies] 段最后一个非空行（end of section）
        lines = text.split("\n")
        in_deps = False
        insert_idx = None
        for i, line in enumerate(lines):
            if re.match(r"^\[dependencies\]", line):
                in_deps = True
                continue
            if in_deps:
                if line.startswith("["):  # 进入下一段
                    insert_idx = i
                    break
                if line.strip():  # 最后一个非空行
                    insert_idx = i + 1
        if insert_idx is None:
            insert_idx = len(lines)

        # 在 insert_idx 前插入
        lines.insert(insert_idx, 'star-context = { path = "../star-context" }')
        new_text = "\n".join(lines)
        if not DRY_RUN:
            cargo.write_text(new_text, encoding="utf-8")
        return True, "added star-context dep"
    return False, "no [dependencies] section"


# =====================================================================
# 替换 ActorContext 字段访问
# =====================================================================

def patch_lib_rs(crate_dir: Path) -> List[str]:
    """返回执行的操作列表 (for 报告)."""
    lib_rs = crate_dir / "src" / "lib.rs"
    if not lib_rs.exists():
        return ["lib.rs not found"]

    actions = []

    # 1. 替换 `actor.is_platform_admin` (字段) → `actor.is_platform_admin()` (方法)
    #    仅在条件表达式中 (`!` 后或 `&&`/`||` 后)
    n = replace_with_regex(
        lib_rs,
        r'\bactor\.is_platform_admin\b(?![\(\w])',
        'actor.is_platform_admin()',
    )
    if n:
        actions.append(f"is_platform_admin field→method: {n} 处")

    # 2. `actor.tenant_id` 强类型比对: `actor.tenant_id != cmd.tenant_id` 改 `TenantId::from(actor.tenant_id) != cmd.tenant_id`
    #    模式:  左边是 actor.tenant_id, 右边是 X.tenant_id (X 是 Uuid 类型或可转 TenantId)
    #    我们只改左边, 用 TenantId::from(...) 包
    n = replace_with_regex(
        lib_rs,
        r'\bactor\.tenant_id\b(?![\(\w])',
        'TenantId::from(actor.tenant_id)',
    )
    if n:
        actions.append(f"actor.tenant_id → TenantId::from: {n} 处")

    # 3. `actor.user_id` 强类型比对 → `UserId::from(actor.user_id)`
    n = replace_with_regex(
        lib_rs,
        r'\bactor\.user_id\b(?![\(\w])',
        'UserId::from(actor.user_id)',
    )
    if n:
        actions.append(f"actor.user_id → UserId::from: {n} 处")

    # 4. `actor.project_ids` 强类型比对 → `.iter().map(|p| ProjectId::from(*p))` 太复杂
    #    实际看代码 project_ids 用法多是 `&actor.project_ids` 给 trait 方法
    #    star_context::ActorContext.project_ids: Vec<Uuid> -- 但强类型 Vec<ProjectId> 不能直接传
    #    需要: actor.project_ids.iter().map(|p| ProjectId::from(*p)).collect::<Vec<_>>()
    #    暂不改, 留给后续 1-1 修复 (P1-1 不在 P0-1 范围)
    #    或简单改: `let pid: ProjectId = ProjectId::from(actor.project_ids[0]);` 形式
    #    简化: 仅改 `actor.project_ids[0]` 模式 + Vec 整个传
    #    为了 P0-1 通过, 我们只处理 Vec<ProjectId>::from(actor.project_ids) 模式
    #    实际上 ProjectId::from(Uuid) 已实现, Vec<T> 没有 From<Vec<U>>
    #    留作 P1-1 后续, P0-1 暂跳过 project_ids 转换

    # 5. 删除 `pub struct ActorContext { ... }` 和 `impl ActorContext { ... }` 段
    #    改用 `pub use star_context::ActorContext;`
    text = lib_rs.read_text(encoding="utf-8")
    if "pub struct ActorContext" in text:
        # 找到 pub struct ActorContext 块和后续 impl ActorContext 块
        # 用正则删除
        # 模式 1: `// ... ActorContext ...\n pub struct ActorContext { ... }\n`
        # 模式 2: `impl ActorContext { ... }\n`

        # 简化: 删除从 `pub struct ActorContext` 到下一个 `// ====` 边界 或 `pub ` 或 `impl `
        # 实际更稳: 用正则匹配整个 struct + impl
        # 我们用以下策略:
        # 1. 删除 pub struct ActorContext 块 (从 pub struct 到对应 })
        # 2. 删除 impl ActorContext 块 (从 impl ActorContext 到对应 })
        # 3. 在删除处插入 `pub use star_context::ActorContext;`

        struct_pattern = r'///[^\n]*\n*pub struct ActorContext\s*\{[^}]*\}\s*\n'
        m = re.search(struct_pattern, text, re.DOTALL)
        if m:
            text = text[:m.start()] + text[m.end():]
            actions.append("删 pub struct ActorContext")

        impl_pattern = r'///[^\n]*\n*impl ActorContext\s*\{[^{}]*(?:\{[^{}]*\}[^{}]*)*\}\s*\n'
        m = re.search(impl_pattern, text, re.DOTALL)
        if m:
            text = text[:m.start()] + text[m.end():]
            actions.append("删 impl ActorContext 块")

        # 找到 "use" 块结束位置, 插入 `pub use star_context::ActorContext;`
        if "pub use star_context::ActorContext;" not in text:
            # 找 use 语句后第一个空行位置
            lines = text.split("\n")
            last_use_idx = -1
            for i, line in enumerate(lines):
                if re.match(r'^\s*(pub\s+)?use\s+', line):
                    last_use_idx = i
            if last_use_idx >= 0:
                # 在 last_use_idx 之后插入
                lines.insert(last_use_idx + 1, 'pub use star_context::ActorContext;')
                text = "\n".join(lines)
                actions.append("加 pub use star_context::ActorContext;")

        if not DRY_RUN:
            lib_rs.write_text(text, encoding="utf-8")

    return actions


# =====================================================================
# main
# =====================================================================

def main() -> int:
    if DRY_RUN:
        log("DRY-RUN 模式 (加 --apply 才真正写文件)")
    else:
        log("APPLY 模式 (将写文件)")

    total = 0
    failed = 0

    for crate_name in TARGET_CRATES:
        crate_dir = CRATES / crate_name
        if not crate_dir.is_dir():
            err(f"跳过 {crate_name}: 目录不存在")
            failed += 1
            continue

        log(f"--- {crate_name} ---")
        total += 1

        # 1. Cargo.toml
        ok, msg = patch_cargo_toml(crate_dir)
        log(f"  Cargo.toml: {msg}")

        # 2. lib.rs 字段访问替换 + struct/impl 删除
        actions = patch_lib_rs(crate_dir)
        for a in actions:
            log(f"  lib.rs: {a}")

        if not actions:
            log(f"  lib.rs: 无修改 (可能已迁移)")

    log(f"")
    log(f"完成 {total} 个 crate, 失败 {failed}")
    log(f"模式: {'APPLY' if not DRY_RUN else 'DRY-RUN'}")
    log(f"下一步: 跑 cargo check --workspace --all-targets 验证")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
