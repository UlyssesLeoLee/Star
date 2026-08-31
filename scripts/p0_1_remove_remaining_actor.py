#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
P0-1 fix 2: 清理所有 domain 的 `pub struct ActorContext` 和 `impl ActorContext` 残留
- bug: 第一次脚本的 struct 正则要求 /// 文档注释, 部分 crate 没注释所以没删
- fix: 改用更宽松的 regex, 删除所有 pub struct ActorContext {...} + impl ActorContext {...}
- 适用: 所有 22 domain + 3 supporting
"""
import re
import sys
from pathlib import Path

WORKSPACE = Path(r"D:\Star\crates")

DOMAINS_WITH_ACTOR_CTX = [
    "domain-agent", "domain-audit", "domain-automation", "domain-board",
    "domain-collaboration", "domain-comment", "domain-context", "domain-development",
    "domain-identity", "domain-local-runtime", "domain-notification", "domain-permission",
    "domain-planning", "domain-project", "domain-relation", "domain-scm", "domain-search",
    "domain-tenant", "domain-work-item", "domain-workflow", "domain-workspace", "domain-worktree",
]
SUPPORTING_CRATES = ["api", "application", "infrastructure"]
TARGETS = DOMAINS_WITH_ACTOR_CTX + SUPPORTING_CRATES

DRY_RUN = "--apply" not in sys.argv


def remove_actor_struct_and_impl(crate_dir: Path) -> tuple[int, int]:
    """删除 pub struct ActorContext {...} 和 impl ActorContext {...} 块.
    返回 (struct_removed, impl_removed).
    """
    lib_rs = crate_dir / "src" / "lib.rs"
    if not lib_rs.exists():
        return (0, 0)

    text = lib_rs.read_text(encoding="utf-8")
    original = text

    # 1. 删除 pub struct ActorContext { ... } 块
    # 模式: 可选 /// 文档注释 + 可选 #[derive(...)] + pub struct ActorContext { ... }
    # 改进: 用 stack-based brace matching
    struct_removed = 0

    while True:
        # 找 pub struct ActorContext 的位置
        m = re.search(r'(?:#\[derive[^\]]*\]\s*)?pub struct ActorContext\s*\{', text)
        if not m:
            # 试无 derive 的简化模式
            m = re.search(r'pub struct ActorContext\s*\{', text)
        if not m:
            break

        # 找匹配的 } (brace matching)
        start = m.start()
        # 跳过 // ===== 行 + /// 文档注释
        # 找到 { 的位置
        brace_open = text.find('{', m.end() - 1)
        depth = 1
        i = brace_open + 1
        while i < len(text) and depth > 0:
            c = text[i]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
            i += 1
        # i 指向 } 之后

        # 跳过 } 后的换行
        end = i
        if end < len(text) and text[end] == '\n':
            end += 1

        # 找 struct 块前的 /// 注释和 // ===== 行
        # 向前找连续 /// 注释和 // ===== 行
        back_start = start
        # 跳过空行
        while back_start > 0 and text[back_start - 1] in ' \t':
            back_start -= 1
        # 检查 back_start - 1 是否是 \n
        if back_start > 0 and text[back_start - 1] == '\n':
            back_start -= 1
        # 向前看 // 行
        while back_start > 0:
            line_start = text.rfind('\n', 0, back_start) + 1
            line = text[line_start:back_start + 1].rstrip()
            if line.startswith('//') or line == '':
                back_start = line_start - 1
            else:
                break

        # 删除 [back_start+1, end)
        text = text[:back_start + 1] + text[end:]
        struct_removed += 1

    # 2. 删除 impl ActorContext { ... } 块 (同样 stack-based)
    impl_removed = 0
    while True:
        m = re.search(r'impl ActorContext\s*\{', text)
        if not m:
            break

        brace_open = text.find('{', m.end() - 1)
        depth = 1
        i = brace_open + 1
        while i < len(text) and depth > 0:
            c = text[i]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
            i += 1
        end = i
        if end < len(text) and text[end] == '\n':
            end += 1

        # 向前找 /// 注释
        back_start = m.start()
        while back_start > 0 and text[back_start - 1] in ' \t':
            back_start -= 1
        if back_start > 0 and text[back_start - 1] == '\n':
            back_start -= 1
        while back_start > 0:
            line_start = text.rfind('\n', 0, back_start) + 1
            line = text[line_start:back_start + 1].rstrip()
            if line.startswith('//') or line == '':
                back_start = line_start - 1
            else:
                break

        text = text[:back_start + 1] + text[end:]
        impl_removed += 1

    # 3. 重复的 pub use star_context::ActorContext; (如果多次插入, 只留一个)
    use_lines = re.findall(r'pub use star_context::ActorContext;', text)
    if len(use_lines) > 1:
        # 删到只剩 1 个 (保留第一个)
        first_pos = text.find('pub use star_context::ActorContext;')
        rest = text[first_pos + len('pub use star_context::ActorContext;'):]
        rest = rest.replace('pub use star_context::ActorContext;', '')
        text = text[:first_pos + len('pub use star_context::ActorContext;')] + rest

    if not DRY_RUN and text != original:
        lib_rs.write_text(text, encoding="utf-8")

    return (struct_removed, impl_removed)


def main() -> int:
    if DRY_RUN:
        print("DRY-RUN")
    else:
        print("APPLY")

    total_struct = 0
    total_impl = 0
    for crate in TARGETS:
        d = WORKSPACE / crate
        s, i = remove_actor_struct_and_impl(d)
        total_struct += s
        total_impl += i
        print(f"{crate:30} struct={s} impl={i}")

    print(f"\n总删除: struct={total_struct}, impl={total_impl}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
