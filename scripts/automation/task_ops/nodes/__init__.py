# scripts/automation/task_ops/nodes/__init__.py
# TMO 7 节点 (M-N1..M-N7) 実装 (per 03-detailed-design.md v0.2 §3.2.1.1)
#
# 本子项 TMO-01 只实装 M-N1 merge_node; 其余 6 节点待 TMO-02..TMO-07 后续子项
#
# 节点清单:
#   - M-N1: merge_node         (TMO-01 已实装)
#   - M-N2: split_node         (TMO-02 planned)
#   - M-N3: reorder_node       (TMO-03 planned, 强约束守门 #13 a)
#   - M-N4: bulk_node          (TMO-04 planned, NFR-TMO-03 ≥ 80% success)
#   - M-N5: summarize_node     (TMO-05 planned)
#   - M-N6: reassign_node      (TMO-06 planned)
#   - M-N7: metadata_node      (TMO-07 planned, Master RLS 必携 per 守门 #13 c)

__version__ = "0.1.0"
__all__ = [
    "merge_node",
]
