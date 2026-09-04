# tests/__init__.py
# TMO Python 化测试 root (per docs/architecture/2026-09-03-langgraph/03-detailed-design.md §3.5 UT/IT/E2E/PT)
#
# 套件:
#   - unit/test_task_ops/test_dag_validator.py  (cycle detection O(V+E) 实证, 守门 #13 a)
#   - unit/test_task_ops/test_reorder_node.py   (UT-22 M-N3 reorder_node)
#   - integration/test_tmo_bulk_dag.py          (IT-12 partial, 跨 subgraph DAG 校验)
#
# 约束 (per 守门 #1 v1 + 守门 #13 a + 守门 #19):
#   - pytest 9.x 跑
#   - 100% pass 门槛 (守门 #9 v20 + 守门 #12 v21)
#   - 不依赖外部网络 / 第三方 LLM API
