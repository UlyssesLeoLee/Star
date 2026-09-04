# scripts/automation/task_ops/nodes/__init__.py
# TMO LangGraph 节点 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6.2)
#
# 节点 (L0 协调层, 守门 #13 a L1↔L1 禁止通信):
#   - merge_node:   M-N1  (实装在 wt-tmo-01)
#   - split_node:   M-N2  (待 TMO-05 实装)
#   - reorder_node: M-N3  (本子代理实装)
#   - bulk_node:    M-N4  (实装在 wt-tmo-04)
#   - summarize_node: M-N5 (待 TMO-06 实装)
#   - reassign_node: M-N6 (待 TMO-06 实装)
#   - metadata_node: M-N7 (待 TMO-07 实装)
#
# 约束 (per 守门 #1 v1 + 守门 #13 a + 守门 #19):
#   - 全部 L0 协调, 不进 L1 sub-agent context
#   - cycle detection + interrupt 协议走 reorder_node 强约束
#   - audit log 必填, 落 docs/reports/tmo.log

__version__ = "0.1.0"
__all__ = ["reorder_node"]
