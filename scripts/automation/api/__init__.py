# scripts/automation/api/__init__.py
# TMO API 路由 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md §2.6.4)
#
# 路由 (L0 协调层, 守门 #13 a):
#   - /api/tmo/dependencies  — 依赖边管理 (本子代理 wt-tmo-03 落档)
#   - /api/tmo/reorder       — 触发 M-N3 reorder_node (本子代理 wt-tmo-03 落档)
#   - /api/tmo/merge         — 触发 M-N1 merge_node (实装在 wt-tmo-01)
#   - /api/tmo/bulk          — 触发 M-N4 bulk_node (实装在 wt-tmo-04)
#
# 约束 (per 守门 #1 v1 + 守门 #13 a + 守门 #22 + 守门 #24):
#   - 全部 L0 协调, 不持有 L1 sub-agent state
#   - FastAPI router, 由 console_server.py 统一 mount (per §12.3)
#   - audit log 落 docs/reports/tmo.log

__version__ = "0.1.0"
__all__ = ["routes_tmo"]
