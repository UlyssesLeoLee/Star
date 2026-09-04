# scripts/automation/task_ops/nodes/__init__.py
# TMO 7 节点 (M-N1..M-N7) 実装 (per 03-detailed-design.md v0.2 §3.2.1.1)
#
# 节点清单 (合并 wt-tmo-01 + wt-tmo-03 + wt-tmo-04 实装):
#   - M-N1: merge_node      (TMO-01 ✅ wt-tmo-01-merge)
#   - M-N2: split_node      (TMO-02 planned)
#   - M-N3: reorder_node    (TMO-03 ✅ wt-tmo-03-dag, 守门 #13 a 强约束 cycle prevention)
#   - M-N4: bulk_node       (TMO-04 ✅ wt-tmo-04-bulk, NFR-TMO-03 partial failure rollback)
#   - M-N5: summarize_node  (TMO-05 planned)
#   - M-N6: reassign_node   (TMO-06 planned)
#   - M-N7: metadata_node   (TMO-07 planned, Master RLS 必携 per 守门 #13 c)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → 全部 L0 协调

__version__ = "0.1.0"
__all__ = [
    "merge_node",
    "reorder_node",
    "bulk_node",
]
