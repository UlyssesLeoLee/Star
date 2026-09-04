# scripts/automation/sub_agent/types/__init__.py
# Sub-agent types registry (SA-01..SA-10)
#
# 已实装:
#   - SA-10 task-orchestrator (per TMO-01 本子项)
#
# Planned (待后续 phase):
#   - SA-01 code-review
#   - SA-02 test-gen
#   - SA-03 5-域-lead-audit (跨域 + 治理矩阵, 5 域 Lead 真人到位后追溯签字)
#   - SA-04 git-ops
#   - SA-05 doc-sync
#   - SA-06 refactor
#   - SA-07 db-migration
#   - SA-08 domain-dev
#   - SA-09 free-form

__version__ = "0.1.0"
__all__ = [
    "sa_10_task_orchestrator",
]
