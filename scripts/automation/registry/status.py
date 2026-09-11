"""
status.py — StatusClassifier (per DD-MULTICA-RUNTIME-001 §3.4 + ADR-0026 v0.2 §1.1 抽象)

3 档 status 分类 (active / stale / poisoned) + missing 4 档.
Per 守门 #11 缺标比错标: poisoned 不删标.
"""
from enum import Enum
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from probe import RuntimeEntry
    from min_version import MinVersionVerdict


class StatusVerdict(str, Enum):
    """3 档 status 枚举 (per SRS-MULTICA-RUNTIME-001 §2 用语)"""
    ACTIVE = "🟢"      # 探测 + auth 双过
    STALE = "🟡"        # 探测过但 auth 失效或版本过低
    POISONED = "🔴"     # 探测到但 unusable, 不删标 (per 守门 #11)
    MISSING = "⚫"       # 探测失败 (binary 不在 PATH)


class StatusClassifier:
    """3 档 status 分类 (per FR-12 ~ FR-15)

    阶段 1 实装: auth_status 默认 "ok" (per scan.py 显式注入);
                  auth probe 阶段 2 实装, 届时会有 "expired" 检测
    """

    def classify(
        self,
        entry: "RuntimeEntry",
        min_verdict: "MinVersionVerdict",
        auth_status: str = "ok",  # "ok" | "expired"
    ) -> StatusVerdict:
        # missing: 探测失败
        if entry.path is None:
            return StatusVerdict.MISSING

        # poisoned: 探测到但 unusable (永久 too_old)
        if min_verdict.status == "too_old":
            return StatusVerdict.POISONED

        # stale: 探测过但 auth 失效
        if auth_status == "expired":
            return StatusVerdict.STALE

        # active: 探测 + auth (未失效) 双过
        return StatusVerdict.ACTIVE
