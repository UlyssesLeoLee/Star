#!/usr/bin/env python3
"""
scripts/automation/backout_b4_misindent.py v0.1
Phase B.4 sub-session #2 回退: domain-feedback 14 处 call site named arg 误改

修法: 检测 `        \w+: TenantId(\w+),\n\n` 8 space 模式 (call site named arg 误改成 struct),
恢复原状 `                    \w+,` 20 space call site named arg
"""
import re
import sys
from pathlib import Path

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

WORKDIR = Path("D:/Star/.worktrees/feat-auto-20260904-1c260bc7")
target = WORKDIR / "crates/domain-feedback/src/lib.rs"

content = target.read_text(encoding="utf-8")
lines = content.split("\n")
print(f"Total lines: {len(lines)}")

# 检测模式: 8 空格 + field + : + TenantId(var) + , + 空行
#  实际是: "        tenant_id: TenantId(tenant_id),"
#  上一行是 "                make_create_cmd("
#  下一行是 "                    FeedbackTarget::WorkItem {"
# 修法: 把 8 空格版本删掉, 上一行 indent 改成 20 空格 + 字段 shorthand

fixes = 0
i = 0
new_lines = []
while i < len(lines):
    line = lines[i]
    # 检测 "        \w+: TenantId(\w+)," 这种 8 空格误改
    m = re.match(r'^(\s{0,8})(\w+): TenantId\((\w+)\),\s*$', line)
    if m and len(m.group(1)) == 8:
        field_name = m.group(2)
        # 删除当前行
        # 上一行 (new_lines 倒数第 1) 应该是函数调用括号
        if new_lines and re.search(r'^\s*make_\w+\($|^\s*\w+::\w+\($', new_lines[-1]):
            # 把 上一行 append `                    field_name,`
            prev_line = new_lines[-1]
            # 计算 indent: 上一行 indent + 20 space
            prev_indent = re.match(r'^(\s*)', prev_line).group(1)
            new_lines.append(f"{prev_indent}                    {field_name},")
            fixes += 1
            # 不 append 当前误改行, 跳过空行
            i += 1
            if i < len(lines) and lines[i] == "":
                i += 1
            continue
    new_lines.append(line)
    i += 1

new_content = "\n".join(new_lines)
target.write_text(new_content, encoding="utf-8")
print(f"Fixed: {fixes}")
