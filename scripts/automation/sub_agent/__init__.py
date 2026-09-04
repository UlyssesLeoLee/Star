# scripts/automation/sub_agent/__init__.py
# Sub-agent types registry (SA-01..SA-10, per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.2.2)
#
# v0.2 新增 SA-10 task-orchestrator (per 02 §2.2.2 + 03 §3.5)
# 9 个旧 SA (SA-01..SA-09) 在 v0.1 文档落档, 实装待后续 phase
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L0 唯一入口, L1↔L1 禁止通信
#   - 守门 #19: Python 化
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic

__version__ = "0.1.0"
__all__ = [
    "types",
]
