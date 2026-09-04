# scripts/automation/api/__init__.py
# TMO FastAPI routes (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.5 8 API 端点)
#
# 本子项 TMO-01 只实装 /api/tmo/merge (1 of 8 端点)
# 其余 7 端点待 TMO-02..TMO-07 后续子项:
#   - POST /api/tmo/merge         (TMO-01 ✅)
#   - POST /api/tmo/split         (TMO-02 planned)
#   - POST /api/tmo/dependencies  (TMO-03 planned)
#   - POST /api/tmo/bulk          (TMO-04 planned)
#   - POST /api/tmo/summarize     (TMO-05 planned)
#   - POST /api/tmo/reassign      (TMO-06 planned)
#   - POST /api/tmo/metadata      (TMO-07 planned)
#   - GET  /api/tmo/operations    (TMO-08 planned, 状态查询)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #22: 调试控制台不污染 main (本 routes 走 port 8080 console_server.py)
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess

__version__ = "0.1.0"
__all__ = ["routes_tmo"]
