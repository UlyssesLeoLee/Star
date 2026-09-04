"""scripts/automation/task_ops/nodes — TMO 7 节点 (M-N1..M-N7) Python 実装

子代理 owner 划分 (per docs/briefs/tmo-2026-09-04-parallel.md):
    M-N1 merge  -> wt-tmo-01
    M-N2 split  -> wt-tmo-02
    M-N3 reorder-> wt-tmo-03
    M-N4 bulk   -> wt-tmo-04 (本 worktree, bulk_node)
    M-N5 summarize -> wt-tmo-05
    M-N6 reassign  -> wt-tmo-06
    M-N7 metadata  -> wt-tmo-07
"""

__version__ = "0.4.0-tmo04"
__all__ = ["bulk_node"]
