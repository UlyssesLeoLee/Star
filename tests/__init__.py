# tests/__init__.py
# Star 仓 tests 根目录 (TMO 4 子项合并: wt-tmo-01 + wt-tmo-03 + wt-tmo-04 + wt-explore-deps)
# per docs/architecture/2026-09-03-langgraph/03-detailed-design.md v0.2
# §8 UT/IT/E2E/PT 设计
#
# 套件 (合并 wt-tmo-01 + wt-tmo-03 + wt-tmo-04 实装):
#   - unit/test_task_ops/test_merge_node.py    (UT-20 M-N1, TMO-01 ✅)
#   - unit/test_task_ops/test_dag_validator.py (DAGValidator cycle detection, 守门 #13 a, TMO-03 ✅)
#   - unit/test_task_ops/test_reorder_node.py  (UT-22 M-N3 reorder_node, TMO-03 ✅)
#   - unit/test_task_ops/test_bulk_node.py     (UT-23 M-N4 bulk_node, NFR-TMO-03, TMO-04 ✅)
#   - integration/test_tmo_merge.py            (IT-10 M-N1 + SA-10 整合, TMO-01 ✅)
#   - integration/test_tmo_bulk_dag.py         (IT-12 partial, 跨 subgraph DAG 校验, TMO-03 + TMO-04 ✅)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #1: 0 .rs 改动, cargo check 0 err (per 守门 #1 v22)
#   - 守门 #9: 子代理 status 必 git log 实证 (4 子代理 ca9ed98 / 8fef058 / 0983523 / e394ed9)
#   - 守门 #19: Python 化, pytest 9.x
#   - 守门 #22: 调试控制台不污染 main 编译链

__version__ = "0.1.0"
