#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v4.py v0.1
Phase B.4 sub-session #5 改进版: 多参数 + 20-23 空格 + UserId::new()

v0.4 改进:
- 模式 B': 20-23 空格 struct shorthand wrap (per domain-workflow / domain-collaboration 实证)
- 模式 F.3: make_actor(user, tenant, project) 多参数 + 第一参数 wrap UserId
- 模式 F.4: make_actor(t1, t2) 多参数 + 第一参数 wrap TenantId
- 模式 J: UserId.new() / TenantId.new() / ProjectId.new() → UserId::new() (关联函数)
"""
import json
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

if len(sys.argv) < 2:
    print("Usage: python fix_b4_batch_v4.py <cargo-json-output-file> [--dry-run]")
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

        # 模式 A: assert_eq! 比较 wrap
        if re.search(r'assert_eq!\((\w+)\.(\w+),\s*(\w+)\);', new):
            m = re.search(r'assert_eq!\((\w+)\.(\w+),\s*(\w+)\);', new)
            struct_name, field_name, var = m.group(1), m.group(2), m.group(3)
            type_name = field_name.replace('_id', '').title()
            # 简化: 强制 TenantId/UserId 推断
            if 'tenant' in field_name.lower():
                new = re.sub(
                    rf'assert_eq!\({struct_name}\.{field_name},\s*{var}\);',
                    f'assert_eq!({struct_name}.{field_name}, TenantId({var}));',
                    new
                )
            elif 'user' in field_name.lower():
                new = re.sub(
                    rf'assert_eq!\({struct_name}\.{field_name},\s*{var}\);',
                    f'assert_eq!({struct_name}.{field_name}, UserId({var}));',
                    new
                )
        # 模式 B': 20-25 空格 struct shorthand wrap (26+ 跳过避免 call site named arg)
        elif re.match(r'^(\s{20,25})(\w+),\s*$', new):
            m = re.match(r'^(\s{20,25})(\w+),\s*$', new)
            indent = m.group(1)
            field_name = m.group(2)
            if field_name in ("tenant_id", "tenant"):
                new = f"{indent}{field_name}: TenantId({field_name}),\n".rstrip("\n")
            elif field_name in ("user_id", "user", "subject_user_id"):
                new = f"{indent}{field_name}: UserId({field_name}),\n".rstrip("\n")
        # 模式 B: 16-19 空格 (跟 v0.3)
        elif re.match(r'^(\s{16,19})(\w+),\s*$', new):
            m = re.match(r'^(\s{16,19})(\w+),\s*$', new)
            indent = m.group(1)
            field_name = m.group(2)
            if field_name in ("tenant_id", "tenant"):
                new = f"{indent}{field_name}: TenantId({field_name}),\n".rstrip("\n")
            elif field_name in ("user_id", "user", "subject_user_id"):
                new = f"{indent}{field_name}: UserId({field_name}),\n".rstrip("\n")
        # 模式 F.3: make_actor(user, tenant, project) 多参数 (第一参数 wrap UserId)
        elif re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*make_actor\((.*)\);?\s*$', new):
            m = re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*make_actor\((.*)\);?\s*$', new)
            if m:
                args = m.group(3)
                # 拆 args 找第一个 TenantId / UserId 位置
                # 简化: 把 (user, ...) → (UserId(user), ...) (假设 user 是 UserId 位置)
                # 但实际 user 通常是 Uuid, 需要 UserId(user)
                # 推断: 第一个参数是 user (UserId), 第二个是 tenant (TenantId)
                arg_list = [a.strip() for a in args.split(",")]
                if len(arg_list) >= 2:
                    # Wrap: user → UserId(user), tenant → TenantId(tenant), ...
                    new_arg_list = []
                    wrap_map = {
                        "user": "UserId",
                        "tenant": "TenantId",
                        "project": "ProjectId",
                        "t1": "TenantId",
                        "t2": "TenantId",
                        "tenant_a": "TenantId",
                        "tenant_b": "TenantId",
                        "actor_t": "TenantId",
                        "cmd_t": "TenantId",
                    }
                    for a in arg_list:
                        # 跳过已经 wrap 的
                        if a.startswith("TenantId(") or a.startswith("UserId(") or a.startswith("ProjectId("):
                            new_arg_list.append(a)
                        elif "::" in a or "." in a or a.startswith("Uuid::"):
                            new_arg_list.append(a)
                        else:
                            # 推断 type
                            for prefix, type_name in wrap_map.items():
                                if a == prefix or a.startswith(prefix + "_"):
                                    new_arg_list.append(f"{type_name}({a})")
                                    break
                            else:
                                new_arg_list.append(a)
                    new_args = ", ".join(new_arg_list)
                    old_match = f"make_actor({args})"
                    new_match = f"make_actor({new_args})"
                    if old_match in new:
                        new = new.replace(old_match, new_match)
        # 模式 F.4: make_three_state_workflow(tenant) → make_three_state_workflow(TenantId(tenant))
        elif re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*(make_\w+_workflow|make_create_cmd|make_register_cmd|make_admin_actor|make_developer_actor|make_\w+_user|make_\w+)\(([a-zA-Z_]\w*)\);?\s*$', new):
            m = re.search(r'^\s+(let|return)\s+(\w+)\s*=\s*(\w+)\(([a-zA-Z_]\w*)\);?\s*$', new)
            if m:
                func_name = m.group(3)
                var = m.group(4)
                if var not in ('TenantId', 'UserId', 'ProjectId'):
                    # 推断 type (根据 func_name)
                    type_name = "TenantId"
                    if "user" in func_name.lower() or "user" in var:
                        type_name = "UserId"
                    elif "admin" in func_name.lower() or "dev" in func_name.lower():
                        # admin_actor / dev / developer_actor 接受 TenantId
                        type_name = "TenantId"
                    elif "workflow" in func_name.lower() or "register" in func_name.lower():
                        type_name = "TenantId"
                    old_call = f"{func_name}({var})"
                    new_call = f"{func_name}({type_name}({var}))"
                    if old_call in new and not f"{type_name}({var})" in new:
                        new = new.replace(old_call, new_call)
        # 模式 J: UserId.new() / TenantId.new() / ProjectId.new() → ::new()
        elif re.search(r'\b(UserId|TenantId|ProjectId|RoleId|WorktreeId|AgentSessionId|AgentId|BoardId|WorkspaceId|WhiteboardId|FeedbackId|IterationId|ValidationId|NotificationId|ApprovalId|TopicId|MessageId|CommentId|AttachmentId)\.new\(\)', new):
            new = re.sub(
                r'\b(UserId|TenantId|ProjectId|RoleId|WorktreeId|AgentSessionId|AgentId|BoardId|WorkspaceId|WhiteboardId|FeedbackId|IterationId|ValidationId|NotificationId|ApprovalId|TopicId|MessageId|CommentId|AttachmentId)\.new\(\)',
                r'\1::new()',
                new
            )
        # 模式 D: 局部 var (tenant_id: tenant, user_id: user) → wrap
        elif re.search(r'tenant_id:\s*tenant\b', new) and "TenantId" not in new:
            new = re.sub(r'tenant_id:\s*tenant\b', 'tenant_id: TenantId(tenant)', new)
        elif re.search(r'user_id:\s*user\b', new) and "UserId" not in new:
            new = re.sub(r'user_id:\s*user\b', 'user_id: UserId(user)', new)
        # 模式 E.2: &ActorContext::new(user.0, tenant.0) → new(user, tenant) (有 & 前缀)
        elif re.search(r'ActorContext::new\((\w+)\.0,\s*(\w+)\.0\)', new):
            new = re.sub(
                r'ActorContext::new\((\w+)\.0,\s*(\w+)\.0\)',
                r'ActorContext::new(\1, \2)',
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
