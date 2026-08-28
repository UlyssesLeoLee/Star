#!/usr/bin/env python3
"""把 visualize.rs 所有 r#"..."# 升级到 r##"..."## (避免 #475569 颜色值冲突)"""
from pathlib import Path

p = Path(r"D:/Star/crates/domain-workflow/src/visualize.rs")
src = p.read_text(encoding="utf-8")
# r#"..."#  -> r##"..."##
# 注意: 已经是 r##" 的不能再升, 但正则可以保证幂等
import re
new_src = re.sub(r'r#"(.*?)"#', r'r##"\1"##', src, flags=re.DOTALL)
p.write_text(new_src, encoding="utf-8")
print("done", src != new_src)
