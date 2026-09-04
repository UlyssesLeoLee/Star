#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v3.py v0.1
Phase B.4 sub-session #2 改进版: 22 lib test crate 通用 type mismatch fixer

v0.2 → v0.3 改进: 区分 struct shorthand (16+ 空格) vs call site named arg (20+ 空格在 `make_xxx(` 内)
- v0.2 把 call site named arg 误改成 struct (8 空格), 导致 parser err
- v0.3 只改 16+ 空格的 struct shorthand, 跳过 20+ 空格的 call site named arg (保持原状)
"""
import json
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

if len(sys.argv) < 2:
    print("Usage: python fix_b4_batch_v3.py <cargo-json-output-file> [--dry-run]")
    sys.exit(1)

dry_run = "--dry-run" in sys.argv
target = Path(sys.argv[1])

errs = []
with open(target, "r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        try:
            msg = json.loads(line)
        except json.JSONDecodeError:
            continue
        if msg.get("reason") != "compiler-message":
            continue
        m = msg.get("message", {})
        if m.get("level") != "error":
            continue
        for span in m.get("spans", []):
            if not span.get("is_primary"):
                continue
            errs.append({
                "file": span.get("file_name", ""),
                "line": span.get("line_start"),
                "col": span.get("column_start"),
                "text": (span.get("text", [{}])[0].get("text", "") if span.get("text") else "").strip(),
            })

# 按 (file, line) 分组
by_file = {}
for e in errs:
    by_file.setdefault(e["file"], []).append(e)

print(f"Total err: {len(errs)}, files: {len(by_file)}")

def fix_file(file_path, file_errs):
    full_path = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7") / file_path.replace("\\", "/")
    if not full_path.exists():
        return 0

    content = full_path.read_text(encoding="utf-8")
    lines = content.split("\n")
    changes = 0

    # 按 line 倒序
    sorted_errs = sorted(file_errs, key=lambda e: -e["line"])

    for e in sorted_errs:
        idx = e["line"] - 1
        if idx < 0 or idx >= len(lines):
            continue
        old = lines[idx]
        new = old

        # 模式 A: assert_eq!(r.tenant_id, tenant_id); → wrap
        if re.search(r'assert_eq!\(r\.tenant_id,\s*tenant_id\);', new):
            new = re.sub(
                r'assert_eq!\(r\.tenant_id,\s*tenant_id\);',
                'assert_eq!(r.tenant_id, TenantId(tenant_id));',
                new
            )
        elif re.search(r'assert_eq!\(r\.user_id,\s*user_id\);', new):
            new = re.sub(
                r'assert_eq!\(r\.user_id,\s*user_id\);',
                'assert_eq!(r.user_id, UserId(user_id));',
                new
            )
        elif re.search(r'assert_eq!\(s\.tenant_id,\s*tenant\);', new):
            new = re.sub(
                r'assert_eq!\(s\.tenant_id,\s*tenant\);',
                'assert_eq!(s.tenant_id, TenantId(tenant));',
                new
            )
        elif re.search(r'assert_eq!\(wt\.tenant_id,\s*tenant_id\);', new):
            new = re.sub(
                r'assert_eq!\(wt\.tenant_id,\s*tenant_id\);',
                'assert_eq!(wt.tenant_id, TenantId(tenant_id));',
                new
            )
        # 模式 B: struct shorthand 16-19 空格 (20+ 几乎都是 call site named arg)
        elif re.match(r'^(\s{16,19})(\w+),\s*$', new):
            m = re.match(r'^(\s{16,19})(\w+),\s*$', new)
            indent = m.group(1)
            field_name = m.group(2)
            # 真实 struct 构造
            if field_name in ("tenant_id", "tenant"):
                new = f"{indent}{field_name}: TenantId({field_name}),\n".rstrip("\n")
            elif field_name in ("user_id", "user", "subject_user_id"):
                new = f"{indent}{field_name}: UserId({field_name}),\n".rstrip("\n")
            # actor 字段是 ActorContext 类型, 不是 TenantId, 跳过
            elif field_name == "actor":
                pass
            else:
                # 其他 ID 字段 (project_id / workspace_id 等) 跨 session 续
                pass
        # 模式 D: 局部 var (tenant_id: tenant, user_id: user) → wrap
        elif re.search(r'tenant_id:\s*tenant\b', new) and "TenantId" not in new:
            new = re.sub(r'tenant_id:\s*tenant\b', 'tenant_id: TenantId(tenant)', new)
        elif re.search(r'user_id:\s*user\b', new) and "UserId" not in new:
            new = re.sub(r'user_id:\s*user\b', 'user_id: UserId(user)', new)
        # 模式 F: helper 调用 (make_actor_user(tenant_id), make_admin_actor(tenant_id))
        # 通用化: 任何 make_\w+ 调 tenant_id
        elif re.search(r'\b(make_\w+)\((.*tenant_id.*)\)', new):
            # 多参数版本: make_xxx(tenant_id, ...) → make_xxx(TenantId(tenant_id), ...)
            m = re.search(r'\b(make_\w+)\((.*tenant_id.*)\)', new)
            if m:
                # 只 wrap 第一个 tenant_id 参数
                func_name = m.group(1)
                args = m.group(2)
                # 找第一个 tenant_id (在参数列表中)
                if "tenant_id" in args and "TenantId(tenant_id)" not in args:
                    new_args = re.sub(r'\btenant_id\b', 'TenantId(tenant_id)', args, count=1)
                    new = new.replace(m.group(0), f"{func_name}({new_args})")
        # 模式 F.2: 短 helper name (admin_ctx / dev_ctx / dev) 调 short_var
        # 通用化: 任何 helper_xxx(var) 其中 var 是局部 Uuid
        elif re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*(\w+)\((\w+)\)\s*;?$', new):
            m = re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*(\w+)\((\w+)\)\s*;?$', new)
            if m:
                func_name = m.group(3)
                var = m.group(4)
                # 排除已经是 TenantId(var) / make_xxx (模式 F 已处理)
                if var != 'TenantId' and not func_name.startswith('make_') and not func_name.startswith('User'):
                    # 检查 var 是不是 Uuid-like (从上下文推断)
                    # 简化: 只改 tenant_id 模式
                    if "tenant" in func_name.lower() or "admin" in func_name.lower() or "dev" in func_name.lower():
                        new = new.replace(m.group(0), m.group(0).replace(f"{func_name}({var})", f"{func_name}(TenantId({var}))"))
        elif re.search(r'\b(make_\w+_actor\w*|make_\w+_user\w*)\(tenant_id\)', new):
            new = re.sub(
                r'\b(make_\w+_actor\w*|make_\w+_user\w*)\(tenant_id\)',
                r'\1(TenantId(tenant_id))',
                new
            )
        # 模式 H: 短变量 (tenant_id: tid, admin(tid))
        elif re.search(r'tenant_id:\s*tid\b', new) and "TenantId" not in new:
            new = re.sub(r'tenant_id:\s*tid\b', 'tenant_id: TenantId(tid)', new)
        elif re.search(r'\badmin\(([a-zA-Z_]\w*)\)', new) and "TenantId" not in new and not re.search(r'\badmin\(TenantId', new):
            m = re.search(r'\badmin\(([a-zA-Z_]\w*)\)', new)
            if m:
                var = m.group(1)
                if var != 'TenantId':
                    new = re.sub(r'\badmin\(' + re.escape(var) + r'\)', f'admin(TenantId({var}))', new)
        # 模式 I: 跨域 ID (project_id: project, workspace_id: workspace) — 跨 session 续
        # 其他模式 (e.g. UserId.new()) — 跨 session 续

        if new != old:
            lines[idx] = new
            changes += 1

    if changes and not dry_run:
        new_content = "\n".join(lines)
        full_path.write_text(new_content, encoding="utf-8")
    return changes


total = 0
for file_path, file_errs in by_file.items():
    c = fix_file(file_path, file_errs)
    if c:
        total += c
        action = "DRY-RUN" if dry_run else "FIXED"
        print(f"{action}: {file_path} ({c} changes)")

print(f"\n{'DRY-RUN' if dry_run else 'FIXED'} total: {total} errs")
