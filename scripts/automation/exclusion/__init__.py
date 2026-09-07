"""Star 排他与幂等架构 view (Star-EI) 实施包.

per docs/architecture/2026-09-07-exclusion-idempotency/01-requirements.md
per docs/reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md v0.1 §1.1 EX-03
per ask_2e8740e6779ac6a8d854c590 拍板 (4 阶段 8 wt 串行, EX-03 阶段 2 wt 1)

守门合规:
- 守门 #13 a: L2 SubAgent 不直接调 L3 Domain 互抢, 必须经 L0 (本包是 L0 层)
- 守门 #5: 不打印 DATABASE_URL, 仅引用
- 守门 #19 v19: 走 scripts/automation/exclusion/ (强制 Python 化)
"""

__version__ = "0.1.0"
__author__ = "Ulysses (per DEC-008) - Mavis 接手"
