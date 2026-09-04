"""scripts/automation/task_ops — Star LangGraph TMO 7 节点 (per 02 §2.6 v0.2)

模块:
    - bulk_queue: BulkOperationQueue (C-18) asyncio.gather 协调 + 部分失败回滚
    - nodes.bulk_node: M-N4 N 张卡批量 action (pause/resume/cancel/set_priority)
    - nodes.<other>: M-N1/M-N2/M-N3/M-N5/M-N6/M-N7 兄弟 worktree 实装
      (wt-tmo-01/02/03/05/06/07 各自 owner)

约束 (per 守门 #1 v1 + 守门 #13 a/d + 守门 #19):
    - 纯 Python (asyncio + dataclasses + typing, 标准库优先)
    - 7 节点全部 L0 协调 (per 守门 #13 a, L1↔L1 禁止通信)
    - task card = Work (短 TTL), checkpoint = Transaction (append-only),
      metadata = Master (SCD Type 2) (per 守门 #13 c/d)
    - bulk partial failure rollback 走 NFR-TMO-03 (≥80% success 视为 partial
      success, 失败 > 20% 全部 rollback)
    - 跨 stage 累计 ≥ 5K token 升档 [M], ≥ 10K 升档 [P] (per 守门 #19)

子代理 owner 划分 (per docs/briefs/tmo-2026-09-04-parallel.md):
    TMO-01 merge  -> wt-tmo-01
    TMO-02 split  -> wt-tmo-02
    TMO-03 reorder-> wt-tmo-03
    TMO-04 bulk   -> wt-tmo-04 (本 worktree)
    TMO-05 summarize -> wt-tmo-05
    TMO-06 reassign  -> wt-tmo-06
    TMO-07 metadata  -> wt-tmo-07
"""

__version__ = "0.4.0-tmo04"
__all__ = [
    "bulk_queue",
    "nodes",
]
