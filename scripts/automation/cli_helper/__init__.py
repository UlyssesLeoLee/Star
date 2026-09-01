# scripts/automation/cli_helper/__init__.py
# CLI 调用基类库 (per docs/automation-design.md §3.2 + §6.2)
#
# 模块:
#   - base: CliHelper 基类 (run / cargo / git / wt / with_worktree)
#
# 约束 (per 守门 #1 v1):
#   - 标准库 only: subprocess / pathlib / json / time
#   - 跨平台: Windows / WSL / macOS / Linux
#   - 失败可重试 (默认 1 次, 指数 backoff)
#   - audit_log 必填

__version__ = "0.1.0"
__all__ = ["base", "CliHelper", "CliResult"]
