# scripts/automation/task_ops/__init__.py
# TMO (Task Management Operations) Python 化基类 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6)
#
# 模块:
#   - relationship_graph: TaskRelationshipGraph 4 字段 DAG (parent / merged_from / split_into / superseded_by)
#   - dag_validator: DAGValidator cycle detection O(V+E) (3-color DFS iterative)
#   - nodes.merge_node: M-N1 merge_node (实装在 wt-tmo-01, 此处 re-export)
#   - nodes.reorder_node: M-N3 reorder_node + dep_set 校验
#   - bulk_queue: BulkOperationQueue (实装在 wt-tmo-04)
#   - nodes.bulk_node: M-N4 bulk_node (实装在 wt-tmo-04)
#
# 约束 (per 守门 #1 v1 + 守门 #12 + 守门 #13 a + 守门 #19):
#   - 全部 L0 协调, L1↔L1 禁止通信 (TMO 节点是 L0 派发层, 不进 sub-agent context)
#   - cycle detection 算法复杂度严格 O(V+E) — DFS iterative + 3-color 标记
#   - audit log 必填, 落 docs/reports/tmo.log
#   - 不开 OpenAI / Anthropic API (per 守门 #23)
#   - 不写 .rs (per 守门 #22 调试控制台不污染 main 编译链)
#
# 落档:
#   - v0.1 (2026-09-04 20:04 JST) — TMO-03 初始化 (wt-tmo-03 子代理落档)
#   - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手

__version__ = "0.1.0"
__all__ = [
    "relationship_graph",
    "dag_validator",
    "nodes",
]
