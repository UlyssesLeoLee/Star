# scripts/automation/task_ops/__init__.py
# TMO (Task Management Operations) 7-node cross-task management module
# (per docs/architecture/2026-09-03-langgraph/02-basic-design.md v0.2 §2.6
#  + 03-detailed-design.md v0.2 §3.2.1.1
#  + ADR-0046 LangGraph TMO 任务卡管理操作
#  + PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1 §1 TMO-01)
#
# 节点 (M-N1..M-N7):
#   - M-N1 merge_node: 合并 a + b → merged_task (本子项 TMO-01 实证)
#   - M-N2 split_node: 拆分 a → a1 + a2
#   - M-N3 reorder_node: 依赖 DAG 调整, cycle detection
#   - M-N4 bulk_node: N 张卡批量 action
#   - M-N5 summarize_node: 跨任务汇总
#   - M-N6 reassign_node: sub-agent 类型 SA-XX 切换
#   - M-N7 metadata_node: task_metadata 表更新 (Master RLS 必携 per 守门 #13 c)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → TMO 7 节点全部 L0 协调
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
    "nodes",
]
