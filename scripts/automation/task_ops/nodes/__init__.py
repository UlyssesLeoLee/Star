# scripts/automation/task_ops/nodes/__init__.py
# TMO 7 节点 (M-N1..M-N7) 実装 (per 03-detailed-design.md v0.2 §3.2.1.1)
#
# 节点清单 (合并 wt-tmo-01 + wt-tmo-02 + wt-tmo-03 + wt-tmo-04 + feat-tmo-05-06-07 实装):
#   - M-N1: merge_node      (TMO-01 ✅ wt-tmo-01-merge, 22/22 tests, async fn)
#   - M-N2: split_node      (TMO-02 ✅ wt-tmo-02-split, 132/132 tests, async fn)
#   - M-N3: reorder_node    (TMO-03 ✅ wt-tmo-03-dag, 70/70 tests, class factory 模式)
#   - M-N4: bulk_node       (TMO-04 ✅ wt-tmo-04-bulk, 49/49 tests, make_bulk_node factory)
#   - M-N5: summarize_node  (TMO-05 ✅ feat-tmo-05-06-07, async fn, mock LLM per 守门 #5+#23)
#   - M-N6: reassign_node   (TMO-06 ✅ feat-tmo-05-06-07, async fn, 跨 SA 类型切换)
#   - M-N7: metadata_node   (TMO-07 ✅ feat-tmo-05-06-07, async fn, Master RLS + SCD Type 2 per 守门 #13 c)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → 全部 L0 协调 (TaskOperationsManager C-16)
#   - 守门 #19: Python 化, 标准库 only, 不写 .rs
#   - 守门 #22: 调试控制台 (port 8080) 不污染 main 编译链
#   - 守门 #23: AI 修改 mock 模式, 不开 OpenAI/Anthropic API

__version__ = "0.3.0"
__all__ = [
    "merge_node",
    "split_node",
    "reorder_node",
    "bulk_node",
    "summarize_node",
    "reassign_node",
    "metadata_node",
]
