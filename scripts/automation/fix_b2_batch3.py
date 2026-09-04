#!/usr/bin/env python3
"""
scripts/automation/fix_b2_batch3.py v0.1
Phase B.2 batch 3: 精准修改 lib.rs:1111-1497 17 unique errs

- 2 个 assert_eq!: tenant_id → TenantId(tenant_id), user_id → UserId(user_id)
- 12 个 struct shorthand: tenant_id, → tenant_id: TenantId(tenant_id),
- 3 个 ListByUserQuery: 同上 (tenant_id: TenantId(...), user_id: UserId(...))

守门 #5 v2: 强制 UTF-8
"""
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

WORKDIR = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7")
target = WORKDIR / "crates/domain-local-runtime/src/lib.rs"

content = target.read_text(encoding="utf-8")
lines = content.split("\n")
print(f"Total lines: {len(lines)}")

# err line 列表 (1-indexed)
err_lines = [1111, 1112, 1155, 1221, 1250, 1262, 1289, 1301, 1327, 1358,
             1376, 1408, 1416, 1424, 1462, 1477, 1497]

# err 类型:
# 1111: assert_eq!(r.tenant_id, tenant_id);  → TenantId(tenant_id)
# 1112: assert_eq!(r.user_id, user_id);  → UserId(user_id)
# 1155-1497: tenant_id, / user_id, / struct shorthand

# 策略: 按行处理 17 个 err,每个 err 行做精准字符串替换
# 1. assert_eq! 改 wrap
# 2. struct shorthand 改 (按上下文:仅当行是 `tenant_id,` 或 `user_id,` 单独成行)

fixed = 0
for ln in err_lines:
    idx = ln - 1  # 0-indexed
    if idx >= len(lines):
        print(f"WARN: line {ln} out of range")
        continue
    old = lines[idx]
    new = old

    if ln == 1111:
        # assert_eq!(r.tenant_id, tenant_id);
        new = old.replace("assert_eq!(r.tenant_id, tenant_id);",
                          "assert_eq!(r.tenant_id, TenantId(tenant_id));")
    elif ln == 1112:
        # assert_eq!(r.user_id, user_id);
        new = old.replace("assert_eq!(r.user_id, user_id);",
                          "assert_eq!(r.user_id, UserId(user_id));")
    elif ln in (1408, 1416, 1424):
        # ListByUserQuery { tenant_id, user_id }
        # 整行替换
        new = old.replace("tenant_id, user_id",
                          "tenant_id: TenantId(tenant_id), user_id: UserId(user_id)")
    else:
        # 12 个 struct shorthand: `tenant_id,` 单独成行 (1155, 1221, 1250, 1262, 1289, 1301, 1327, 1358, 1376, 1462, 1477, 1497)
        # 改: tenant_id, → tenant_id: TenantId(tenant_id),
        # 注意: 不能影响 `tenant_id: TenantId(tenant_id),` 已改过的行
        if "tenant_id," in new and "TenantId(tenant_id)" not in new:
            new = new.replace("tenant_id,", "tenant_id: TenantId(tenant_id),", 1)
        # user_id 同样
        if "user_id," in new and "UserId(user_id)" not in new:
            new = new.replace("user_id,", "user_id: UserId(user_id),", 1)

    if new != old:
        lines[idx] = new
        fixed += 1
        print(f"L{ln}: FIXED")
    else:
        print(f"L{ln}: NO CHANGE - {old[:80]}")

print(f"\nFixed: {fixed}/{len(err_lines)}")

# 写回
new_content = "\n".join(lines)
target.write_text(new_content, encoding="utf-8")
print(f"Written: {target}")
