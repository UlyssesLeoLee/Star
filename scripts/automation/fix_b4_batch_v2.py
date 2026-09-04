#!/usr/bin/env python3
"""
scripts/automation/fix_b4_batch_v2.py v0.1
Phase B.4 sub-session #2+: 22 lib test crate 通用 type mismatch fixer

跨 22 domain 通用化 fix_b2_batch3.py (commit 40e5fd6):
- 模式 A: assert_eq! 比较 Uuid 跟 TenantId/UserId → wrap
- 模式 B: struct 字段 shorthand `tenant_id,` → `tenant_id: TenantId(tenant_id),`
- 模式 C: ListByUserQuery / 类似 struct 同 B
- 模式 D: domain-permission 特例 `tenant_id: tenant` / `user_id: user` (局部 var) → `tenant_id: TenantId(tenant)` / `user_id: UserId(user)`
- 模式 E: ActorContext::new(user.0, tenant.0) → ActorContext::new(user, tenant)

守门 #5 v2: 强制 UTF-8
"""
import json
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

if len(sys.argv) < 2:
    print("Usage: python fix_b4_batch_v2.py <cargo-json-output-file> [--dry-run]")
    sys.exit(1)

dry_run = "--dry-run" in sys.argv
target_file_idx = sys.argv.index("--dry-run") - 1 if dry_run else 1
target = Path(sys.argv[target_file_idx])

# 解析 cargo --message-format=json
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

# 按 file:line:col 排序
errs.sort(key=lambda e: (e["file"], e["line"], e["col"]))

# 按 file 分组
by_file = {}
for e in errs:
    by_file.setdefault(e["file"], []).append(e)

print(f"Total err: {len(errs)}, files: {len(by_file)}")

# 改写规则
def fix_file(file_path, file_errs, content, lines):
    """改写 lib.rs, 5 种模式"""
    changes = []
    # 按 line 倒序处理 (避免 line offset shift)
    sorted_errs = sorted(file_errs, key=lambda e: -e["line"])

    for e in sorted_errs:
        idx = e["line"] - 1
        if idx < 0 or idx >= len(lines):
            continue
        old = lines[idx]
        new = old

        # 模式 A: assert_eq!(r.tenant_id, tenant_id); → assert_eq!(r.tenant_id, TenantId(tenant_id));
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
        # 模式 B + C: struct 字段 shorthand
        elif re.search(r'\b(tenant_id|user_id|project_id|workspace_id|board_id|actor_id|admin_id|user_2_id|owner_id|target_id|assignee_id|member_id|subject_user_id|resource_id|reviewer_id),\s*$', new):
            # 匹配行末单独字段 shorthand, 改成显式构造
            m = re.search(r'^\s*(\w+),\s*$', new)
            if m:
                field_name = m.group(1)
                # domain-permission 特例: subject_user_id 也是 user_id 模式
                # 其他字段 (board_id / workspace_id 等) 走通用
                if field_name in ("tenant_id", "user_id", "subject_user_id", "actor_id", "admin_id"):
                    type_name = "TenantId" if field_name == "tenant_id" else "UserId"
                    # 简化: tenant/user_id 都按 type 推断
                    if field_name == "tenant_id":
                        new = f"        {field_name}: TenantId({field_name}),\n"
                    elif field_name in ("user_id", "subject_user_id"):
                        new = f"        {field_name}: UserId({field_name}),\n"
                    else:
                        # 跨域 ID 暂跳过, 跨 sub-session 续
                        continue
                else:
                    # 其他 ID 类型 (ProjectId / WorkspaceId / BoardId 等) 暂跳过
                    # 它们的 type 不一定是 ProjectId(需查 macro 定义)
                    continue
        # 模式 D: domain-permission 特例
        elif re.search(r'tenant_id:\s*tenant\b', new) and not "TenantId" in new:
            new = re.sub(r'tenant_id:\s*tenant\b', 'tenant_id: TenantId(tenant)', new)
        elif re.search(r'user_id:\s*user\b', new) and not "UserId" in new:
            new = re.sub(r'user_id:\s*user\b', 'user_id: UserId(user)', new)
        # 模式 E: ActorContext::new(user.0, tenant.0) → new(user, tenant)
        elif re.search(r'ActorContext::new\((\w+)\.0,\s*(\w+)\.0\)', new):
            new = re.sub(
                r'ActorContext::new\((\w+)\.0,\s*(\w+)\.0\)',
                r'ActorContext::new(\1, \2)',
                new
            )
        # 模式 F: helper 调用 (e.g. make_actor_user(tenant_id), make_admin_actor(tenant_id))
        # 修法: 改调用方 wrap TenantId(tenant_id)
        elif re.search(r'(make_\w+_actor\w*|make_\w+_user\w*)\(tenant_id\)', new):
            new = re.sub(
                r'(make_\w+_actor\w*|make_\w+_user\w*)\(tenant_id\)',
                r'\1(TenantId(tenant_id))',
                new
            )
        # 模式 G: 重复 wrap (e.g. tenant_id: TenantId(tenant_id),) → 检测类型
        # 不重复处理, 留给 cargo 报 unknown 走下个 pattern
        # 模式 H: 短变量名 (e.g. tenant_id: tid, admin(tid)) → tenant_id: TenantId(tid), admin(TenantId(tid))
        elif re.search(r'tenant_id:\s*tid\b', new) and "TenantId" not in new:
            new = re.sub(r'tenant_id:\s*tid\b', 'tenant_id: TenantId(tid)', new)
        elif re.search(r'\badmin\(([a-zA-Z_]\w*)\)', new) and "TenantId" not in new and not re.search(r'\badmin\(TenantId', new):
            # 通用 admin(短变量) wrap
            m = re.search(r'\badmin\(([a-zA-Z_]\w*)\)', new)
            if m:
                var = m.group(1)
                # 排除已经是 TenantId(var)
                if var != 'TenantId':
                    new = re.sub(r'\badmin\(' + re.escape(var) + r'\)', f'admin(TenantId({var}))', new)

        if new != old:
            lines[idx] = new
            changes.append((e["line"], old[:60], new[:60]))

    return changes


# 逐文件处理
total_changes = 0
for file_path, file_errs in by_file.items():
    # file_path 是 cargo 报告的相对路径, 转换为绝对路径
    full_path = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7") / file_path.replace("\\", "/")
    if not full_path.exists():
        print(f"SKIP: {file_path} not found")
        continue

    content = full_path.read_text(encoding="utf-8")
    lines = content.split("\n")

    changes = fix_file(file_path, file_errs, content, lines)
    if changes:
        new_content = "\n".join(lines)
        if dry_run:
            print(f"\n=== DRY-RUN {file_path} ({len(changes)} changes) ===")
            for line, old, new in changes[:5]:
                print(f"  L{line}: {old}")
                print(f"     → {new}")
        else:
            full_path.write_text(new_content, encoding="utf-8")
            print(f"FIXED: {file_path} ({len(changes)} changes)")
            total_changes += len(changes)

if dry_run:
    print(f"\nDRY-RUN total: would fix ~{total_changes} errs across {len(by_file)} files")
else:
    print(f"\nFIXED total: {total_changes} errs across {len(by_file)} files")
