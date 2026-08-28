#!/usr/bin/env python3
"""一次性修 JqlError::Parse 构造: 9 处全部用 JqlError::Parse { pos, message } 形式"""
import re
from pathlib import Path

p = Path(r"D:/Star/crates/domain-search/src/jql.rs")
src = p.read_text(encoding="utf-8")

# 用正则匹配所有剩余 JqlError::Parse 构造
# 旧形式: JqlError::Parse(<expr>) 其中 <expr> 是字符串字面量或 format! 表达式
# 新形式: JqlError::Parse { pos: <int_or_default>, message: <expr> }

# 简单字符串字面量
patterns = [
    # JqlError::Parse("xxx".into())
    (r'JqlError::Parse\("([^"]+)"\.into\(\)\)', r'JqlError::Parse { pos: 0, message: "\1".into() }'),
    # JqlError::Parse(format!("xxx {} yyy", var))
    (r'JqlError::Parse\(format!\("([^"]+)"', r'JqlError::Parse { pos: 0, message: format!("\1"'),
]

count = 0
for pat, repl in patterns:
    new_src, n = re.subn(pat, repl, src)
    if n > 0:
        count += n
        src = new_src
        print(f"replaced {n} occurrences of pattern: {pat[:40]}...")

# 补全 format! 后面缺失的 })
# 上一条替换可能留 "})" 形式, 需手动检查 (此处未做, 让 cargo build 报错后查)
# 实际更安全: 把所有 JqlError::Parse { pos: 0, message: format!(...)} 后面多余 ), 改 },  }
# 但只对"明确是 format! 的"做
src = re.sub(
    r'JqlError::Parse \{ pos: 0, message: format!\("([^"]+"\s*,\s*[^)]+)\)\}',
    r'JqlError::Parse { pos: 0, message: format!("\1") }',
    src
)

p.write_text(src, encoding="utf-8")
print(f"total replaced: {count}")
