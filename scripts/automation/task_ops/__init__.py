# scripts/automation/task_ops/__init__.py
# TMO (Task Management Operations) Python 化基类 (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6)
#
# 节点 (M-N1..M-N7):
#   - M-N1 merge_node:   合并 a + b → merged_task (实装 wt-tmo-01)
#   - M-N2 split_node:   拆分 a → a1 + a2 (planned)
#   - M-N3 reorder_node: 依赖 DAG 调整, cycle detection (实装 wt-tmo-03)
#   - M-N4 bulk_node:    N 张卡批量 action (实装 wt-tmo-04)
#   - M-N5 summarize_node: 跨任务汇总 (planned)
#   - M-N6 reassign_node:  sub-agent 类型 SA-XX 切换 (planned)
#   - M-N7 metadata_node:  task_metadata 表更新 (Master RLS 必携 per 守门 #13 c, planned)
#
# 模块 (合并 wt-tmo-01 + wt-tmo-03 + wt-tmo-04 实装):
#   - manager:            TaskOperationsManager 集中调度 (wt-tmo-01)
#   - protocols:          7 协议 TypedDict (wt-tmo-01)
#   - relationship_graph: TaskRelationshipGraph 4 字段 DAG (wt-tmo-03)
#   - dag_validator:      DAGValidator cycle detection O(V+E) (wt-tmo-03, 守门 #13 a 强约束)
#   - bulk_queue:         BulkOperationQueue asyncio.gather + partial failure rollback (wt-tmo-04)
#   - nodes:              7 节点 子包
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → TMO 7 节点全部 L0 协调
#   - 守门 #13 c: Master RLS 必携 (task_metadata)
#   - 守门 #13 d: task card 状态 = Work (短 TTL), checkpoint history = Transaction (append-only), metadata = Master (SCD Type 2)
#   - 守门 #19: 走 Python 化, 不写 .rs
#   - 守门 #20: 子代理 dispatch 必先 brief 落档
#   - 守门 #22: 调试控制台不污染 main (Python 进程 port 8080)
#   - 守门 #23: AI 修改 mock, 不开 OpenAI/Anthropic API
#   - 守门 #24: 浏览器 → Next.js → FastAPI 8080 console_server.py → subprocess

__version__ = "0.1.0"
__all__ = [
    "manager",
    "protocols",
    "relationship_graph",
    "dag_validator",
    "bulk_queue",
    "nodes",
]
