#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v5.py v0.1
Phase B.4 sub-session #6 改进版: 12-15 call site + 多参数 wrap + deref

v0.5 改进:
- 模式 B'': 12-15 空格 call site named arg (排除 helper 内部 13+ 空格)
- 模式 F.5: (var, project) / (var_a, project_a) multi-arg helper 第一参数 TenantId wrap
- 模式 F.6: make_admin_actor(var_a, project_id) 第一参数 TenantId wrap
- 模式 E.3: ActorContext::new(Uuid::new_v4(), var.0) 第二参数 wrap + 删 .0
- 模式 K: .with_project(project_id) → .with_project(*project_id.as_uuid()) (24+ 空格)
- 模式 L: .list_by_tenant(ListByTenantQuery { tenant_id }, &actor) (一行内 wrap)
"""
import json
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

if len(sys.argv) < 2:
    print("Usage: python fix_b4_batch_v5.py <cargo-json-output-file> [--dry-run]")
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

    sorted_errs = sorted(file_errs, key=lambda e: -e["line"])

    for e in sorted_errs:
        idx = e["line"] - 1
        if idx < 0 or idx >= len(lines):
            continue
        old = lines[idx]
        new = old

        # 模式 B'': 12-15 空格 call site named arg (在 make_xxx() 内部)
        # 条件: 上一行以 make_xxx( 开头
        if re.match(r'^(\s{12,15})(\w+),\s*$', new):
            # 检查上一行
            if idx > 0:
                prev_line = lines[idx - 1]
                if re.search(r'\b(make_\w+|\w+::\w+|sample_index_cmd|submit_result|with_project|create_feedback|create_worktree|create_change_set|create_sprint|list_by_tenant|list_consumed_events|create_integration|get_integration|get_sync_state|mark_applied|mark_verified|record_consumed|create_with|delete_feedback|create_cmd|register_cmd|basic_cmd|three_state_workflow|approve|reject|transition|upsert_index|add_relevant_item|add_item|create_command|create_query|create_actor)\($', prev_line):
                    # call site named arg 跳过
                    pass
                else:
                    m = re.match(r'^(\s{12,15})(\w+),\s*$', new)
                    indent = m.group(1)
                    field_name = m.group(2)
                    if field_name in ("tenant_id", "tenant"):
                        new = f"{indent}{field_name}: TenantId({field_name}),\n".rstrip("\n")
                    elif field_name in ("user_id", "user", "subject_user_id"):
                        new = f"{indent}{field_name}: UserId({field_name}),\n".rstrip("\n")
        # 模式 E.3: ActorContext::new(Uuid::new_v4(), var.0) → new(Uuid::new_v4(), TenantId(var))
        elif re.search(r'ActorContext::new\(([a-zA-Z_]\w*)\.0\)', new):
            new = re.sub(
                r'ActorContext::new\(([a-zA-Z_]\w*)\.0\)',
                r'ActorContext::new(TenantId(\1))',
                new
            )
        # 模式 F.5: make_admin(tenant, project) / make_developer(tenant, project)
        elif re.search(r'^\s+let\s+(\w+)\s*=\s*make_(admin|developer)(_actor)?\(([a-zA-Z_]\w*),\s*(\w+)\);?\s*$', new):
            m = re.search(r'^\s+let\s+(\w+)\s*=\s*make_(admin|developer)(_actor)?\(([a-zA-Z_]\w*),\s*(\w+)\);?\s*$', new)
            if m:
                var = m.group(4)  # tenant / tenant_a / tenant_b
                # 推断: 第一个参数 (var) 是 TenantId wrap
                if "tenant" in var.lower() and f"TenantId({var})" not in new:
                    new = new.replace(f"({var},", f"(TenantId({var}),", 1)
        # 模式 F.6: make_admin_actor(tenant_a, project_id) / make_actor(actor_t)
        elif re.search(r'^\s+let\s+\w+\s*=\s*make_\w+\(([a-zA-Z_]\w*)\);?\s*$', new):
            m = re.search(r'^\s+let\s+\w+\s*=\s*make_\w+\(([a-zA-Z_]\w*)\);?\s*$', new)
            if m:
                var = m.group(1)
                if "tenant" in var.lower() and f"TenantId({var})" not in new:
                    new = new.replace(f"({var})", f"(TenantId({var}))")
                elif "user" in var.lower() and f"UserId({var})" not in new:
                    new = new.replace(f"({var})", f"(UserId({var}))")
        # 模式 F.7: make_create_cmd(tenant_a, ...) / basic_cmd(tid) 多参数 + 第一参数 wrap
        elif re.search(r'\b(make_create_cmd|make_register_cmd|make_three_state_workflow|make_create_sprint_cmd|basic_cmd|sample_index_cmd|make_cmd|projector_actor)\(([a-zA-Z_]\w*)\b', new):
            m = re.search(r'\b(make_create_cmd|make_register_cmd|make_three_state_workflow|make_create_sprint_cmd|basic_cmd|sample_index_cmd|make_cmd|projector_actor)\(([a-zA-Z_]\w*)\b', new)
            if m:
                func = m.group(1)
                var = m.group(2)
                if "tenant" in var.lower() and f"TenantId({var})" not in new:
                    new = new.replace(f"{func}({var}", f"{func}(TenantId({var})", 1)
        # 模式 K: .with_project(project_id) → .with_project(*project_id.as_uuid())
        elif re.search(r'\.with_project\((\w+)\)', new):
            new = re.sub(
                r'\.with_project\((\w+)\)',
                r'.with_project(*\1.as_uuid())',
                new
            )
        # 模式 L: ListByTenantQuery { tenant_id } → ListByTenantQuery { tenant_id: TenantId(tenant_id) }
        elif re.search(r'\{(\s+)tenant_id(\s+)\}', new):
            new = re.sub(
                r'\{(\s+)tenant_id(\s+)\}',
                r'{\1tenant_id: TenantId(tenant_id)\2}',
                new
            )

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
