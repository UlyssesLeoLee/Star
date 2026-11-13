# scripts/automation/task_ops/spawn_subtask/__init__.py
# spawn_subtask 子包 — M-N8 Sub-task Binding 实装 (WBS 6.3)
# per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §1 (M-26..M-33)
# + 02-basic-design.md §2.7.1 (M-N8 spawn_subtask_node 7 步原子化)
#
# 8 模块 (per 03-detailed §1 模块结构):
#   - M-26 spawn_subtask_node: M-N8 节点 (7 步原子化, L0 协调)
#   - M-27 lifecycle_linkage:  C-23 LifecycleLinkageManager (parent/child 联动 ≤ 1s)
#   - M-28 exclusive_guard:    C-24 ExclusiveBindingGuard (1:1 独占守门, 不可绕过)
#   - M-29 quota_registry:     C-25 SubTaskQuotaRegistry (per-parent quota, Master SCD2)
#   - M-30 spawn_validator:    C-26 SpawnSubtaskValidator (spawn 前 5 步校验)
#   - M-31 child_factory:      C-27 ChildSubAgentFactory (child state + 4 binding 字段)
#   - M-32 event_bus:          C-28 ParentChildEventBus (state change 订阅派发)
#   - M-33 token_budget:       C-29 SubTaskTokenBudget (per-sub-task token 预算)
#   + exceptions.py (6 异常类) + models.py (SpawnSubtaskRequest/Result) +
#     cli.py (subprocess 入口, per 守门 #19/#24)
#
# 守门 (per AGENTS.md §4):
#   - 守门 #13 a: L1↔L1 禁止通信 → M-N8 全部 L0 协调
#   - 守门 #13 c/d: W/T/M 3 表 (001/002/003 DDL) + Master SCD Type 2
#   - 守门 #19: 标准库 only, subprocess 入口 (cli.py)
#   - 守门 #12: 缺标比错标 → 失败必填 failure_reason

__version__ = "0.1.0"
__all__ = [
    "spawn_subtask_node",
    "lifecycle_linkage",
    "exclusive_guard",
    "quota_registry",
    "spawn_validator",
    "child_factory",
    "event_bus",
    "token_budget",
    "exceptions",
    "models",
    "cli",
]
