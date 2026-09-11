"""
scripts/automation/registry/ — Multica Runtime Registry 域

@deprecated v0.legacy — Python 实装阶段 (阶段 1 commit `fadff8f`).
                R2 阶段 (本 commit): 8 文件重命名为 *_v0_legacy.py + 头部加 @deprecated.
                R3 阶段 (per ADR-0027 R3): 新 Rust `star-registry` crate 实装,
                此 Python package 走 legacy fallback (per 守门 #6 兼容).

Per SRS-MULTICA-RUNTIME-001 v0.1 + DD-MULTICA-RUNTIME-001 v0.1 (评审 7 修复后).

阶段 1 (2026-09-11 JST): scan.py 主体 + 5 provider 验证
阶段 2: 25 provider 全适配 + console 扩展 (废弃, 跳过, 走 R3 Rust)
阶段 3: Rust star-registry crate 替换 (per ADR-0027)

R2 阶段 (本 commit):
- 8 文件 git mv 到 *_v0_legacy.py
- 头部加 @deprecated docstring
- 5 文件 import 更新 (probe/reporter/scan/status/validate)
- providers/__init__.py 同样重命名
- Python __init__.py 保留作 package marker + deprecation notice

Per 守门 #11 缺标比错标: 不删, 永久留档.
Per 守门 #14 v3 Mavis 永久代签: 5 域 Lead 真人内容由 Mavis 决定 (per 9/11 23:11 JST 强化).
"""

__version__ = "0.1.0-legacy"
__deprecated__ = True
__replacement__ = "Rust star-registry crate (per ADR-0027 R3)"
