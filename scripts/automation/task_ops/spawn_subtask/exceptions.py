# @cypher schema=1 source_sha256=e82cce15e16e5f019328fa41e9bdf176b4ebe8ed2b5e12a992f3faf3defe1195
# MERGE (self:File {path:"scripts/automation/task_ops/spawn_subtask/exceptions.py"})
# @endcypher
# scripts/automation/task_ops/spawn_subtask/exceptions.py
# spawn_subtask 6 异常类 (per docs/architecture/2026-09-09-subtask-binding/
# 03-detailed-design.md §3.3.1.1 + 02-basic-design.md §2.7.1 失败模式)
#
# 失败模式 (per 02-basic §2.7.1):
#   (a) InvalidParentState       (b) InvalidSAType         (c) SubTaskQuotaExhausted
#   (d) SelfSpawnProhibited      (e) ExclusiveBindingViolation
#   增量: SubTaskTokenBudgetExhausted (per NFR-SB-04 / UT-32)
#
# 守门 #12 缺标比错标: 失败必填 failure_reason (type name + message 由调用方组装).

from __future__ import annotations


class InvalidParentState(Exception):
    """父 task 状态 ∈ {superseded, completed, failed, cancelling} (per F-26 互斥表)."""


class InvalidSAType(Exception):
    """SA type 非法 (e.g. SA-10 自嵌套, per F-30 + NFR-SB-02)."""


class SubTaskQuotaExhausted(Exception):
    """父 task sub-task quota 耗尽 (max=2, per F-31 默认值)."""


class SelfSpawnProhibited(Exception):
    """parent_task_id == child_task_id 自身派生 (per F-30 派生约束)."""


class ExclusiveBindingViolation(Exception):
    """C-24 ExclusiveBindingGuard 拒绝 (per NFR-SB-02 不可绕过)."""


class SubTaskTokenBudgetExhausted(Exception):
    """per-sub-task token 预算耗尽 (per NFR-SB-04 + F-31 TokenTelemetry 集成)."""
