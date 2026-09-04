# scripts/automation/api/__init__.py
# TMO FastAPI routes (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6.4 8 API 端点)
#
# 8 端点 (合并 wt-tmo-01 + wt-tmo-03 + wt-tmo-04 实装):
#   - POST /api/tmo/merge         (TMO-01 ✅ wt-tmo-01-merge)        合并任务卡 (M-N1 + SA-10)
#   - POST /api/tmo/split         (TMO-02 planned)                   拆分任务卡 (M-N2)
#   - POST /api/tmo/dependencies  (TMO-03 ✅ wt-tmo-03-dag)           依赖边管理 (M-N3 + DAGValidator)
#   - POST /api/tmo/bulk          (TMO-04 ✅ wt-tmo-04-bulk)           批量操作 (M-N4 + BulkOperationQueue)
#   - POST /api/tmo/summarize     (TMO-05 planned)                   跨任务汇总 (M-N5)
#   - POST /api/tmo/reassign      (TMO-06 planned)                   重新分配 (M-N6)
#   - POST /api/tmo/metadata      (TMO-07 planned)                   元数据编辑 (M-N7, Master RLS)
#   - GET  /api/tmo/operations    (TMO-08 stub ✅ wt-tmo-01-merge)    状态查询 (TaskOperationsManager snapshot)
#   - GET  /api/tmo/relationships (TMO-09 stub ✅ wt-tmo-03-dag)      DAG 关系查询
#
# 路由 (L0 协调层, 守门 #13 a):
#   全部 L0 协调, 不持有 L1 sub-agent state
#   FastAPI router, 由 console_server.py 统一 mount (per 守门 #24 v3)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → TMO 8 端点全部 L0 协调
#   - 守门 #19: Python 化, 不写 .rs
#   - 守门 #20: 子代理 dispatch 必先 brief 落档
#   - 守门 #22: 调试控制台不污染 main (本 routes 走 port 8080 console_server.py)
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 → subprocess

__version__ = "0.1.0"
__all__ = ["routes_tmo"]
