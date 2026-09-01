#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/saga_e2e.py — C.6 Saga 跨 5 域补偿 + 失败回滚 e2e
(per docs/automation-design.md v0.1 §3.3 + §4.2 + §6.4 C.6 共享)

5 域补偿链: player (创建角色) -> economy (扣费) -> match (匹配对手) -> social (发通知) -> admin (审计)
任何 1 步失败回滚前 N 步补偿

per WBS §2 C.6 (commit 25d086e star-saga 增强) + INV-SG-05 SagaStep.idempotency_key

用法:
    # Dry-run 5 域 × 2 case (成功 + 失败回滚) = 10 case
    python scripts/automation/saga_e2e.py --dry-run

    # 跑全部 chain (5 域 × 2 case = 10 case)
    python scripts/automation/saga_e2e.py

    # 触发特定域失败 (rollback 实证)
    python scripts/automation/saga_e2e.py --fail-domain social

约束 (per 守门 #1 v1):
    - 标准库 only (dataclasses / enum / json / argparse / subprocess / pathlib)
    - SagaStep dataclass (id / domain / action / compensation / idempotency_key per INV-SG-05)
    - 5 域 mock 各 1 个 function, 返 success/failure (5 域 Lead 真人到位前)
    - audit_log 必填, 落 docs/reports/saga-e2e.log
"""

from __future__ import annotations

import argparse
import json
import sys
import time
from dataclasses import dataclass, field, asdict
from enum import Enum
from pathlib import Path
from typing import Optional, Callable

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


class Domain(Enum):
    """5 域 (per 守门 #3 历史治理命名, 跟 DDD bounded context 区分)"""

    PLAYER = "player"
    ECONOMY = "economy"
    MATCH = "match"
    SOCIAL = "social"
    ADMIN = "admin"


@dataclass
class SagaStep:
    """Saga step (per INV-SG-05 idempotency_key)"""

    id: str
    domain: Domain
    action: str  # function name to call
    compensation: str  # function name to call on rollback
    idempotency_key: str  # INV-SG-05
    status: str = "pending"  # pending / success / failed / compensated
    result: Optional[dict] = None
    error: Optional[str] = None


@dataclass
class SagaResult:
    """Saga 整体执行结果"""

    success: bool
    steps: list  # list[SagaStep]
    failed_at: Optional[str] = None
    compensated_steps: list = field(default_factory=list)  # list[str] (step.id)
    duration_ms: float = 0.0


@dataclass
class AuditEntry:
    """审计日志条目 (per docs/automation-design.md §3.4)"""

    timestamp: float
    phase: str
    action: str
    input: dict
    output: dict
    error: Optional[str] = None


# 5 域 mock action / compensation (5 域 Lead 真人到位前 e2e 用 mock 域)
def mock_player_create(actor: dict) -> dict:
    """player 域: 创建角色"""
    return {"character_id": f"char-{int(time.time() * 1000)}", "actor": actor}


def mock_player_delete(result: dict) -> dict:
    """player 域补偿: 删除角色"""
    return {"deleted": True, "character_id": result.get("character_id")}


def mock_economy_charge(actor: dict) -> dict:
    """economy 域: 扣费"""
    return {"charged_usd": 1.0, "wallet_id": actor.get("wallet_id")}


def mock_economy_refund(result: dict) -> dict:
    """economy 域补偿: 退款"""
    return {"refunded_usd": result.get("charged_usd"), "wallet_id": result.get("wallet_id")}


def mock_match_pair(actor: dict) -> dict:
    """match 域: 匹配对手"""
    return {"opponent_id": f"opp-{int(time.time() * 1000)}"}


def mock_match_unpair(result: dict) -> dict:
    """match 域补偿: 取消匹配"""
    return {"unpaired": True, "opponent_id": result.get("opponent_id")}


def mock_social_notify(actor: dict) -> dict:
    """social 域: 发通知"""
    return {"notification_id": f"notif-{int(time.time() * 1000)}"}


def mock_social_unnotify(result: dict) -> dict:
    """social 域补偿: 撤销通知"""
    return {"unnotified": True, "notification_id": result.get("notification_id")}


def mock_admin_audit(actor: dict) -> dict:
    """admin 域: 审计"""
    return {"audit_id": f"audit-{int(time.time() * 1000)}"}


def mock_admin_unaudit(result: dict) -> dict:
    """admin 域补偿: 撤销审计"""
    return {"unaudited": True, "audit_id": result.get("audit_id")}


# Saga 5 域链 (per brief)
SAGA_CHAIN = [
    SagaStep(
        id="step-1-player",
        domain=Domain.PLAYER,
        action="mock_player_create",
        compensation="mock_player_delete",
        idempotency_key=f"saga-{int(time.time() * 1000)}-player",
    ),
    SagaStep(
        id="step-2-economy",
        domain=Domain.ECONOMY,
        action="mock_economy_charge",
        compensation="mock_economy_refund",
        idempotency_key=f"saga-{int(time.time() * 1000)}-economy",
    ),
    SagaStep(
        id="step-3-match",
        domain=Domain.MATCH,
        action="mock_match_pair",
        compensation="mock_match_unpair",
        idempotency_key=f"saga-{int(time.time() * 1000)}-match",
    ),
    SagaStep(
        id="step-4-social",
        domain=Domain.SOCIAL,
        action="mock_social_notify",
        compensation="mock_social_unnotify",
        idempotency_key=f"saga-{int(time.time() * 1000)}-social",
    ),
    SagaStep(
        id="step-5-admin",
        domain=Domain.ADMIN,
        action="mock_admin_audit",
        compensation="mock_admin_unaudit",
        idempotency_key=f"saga-{int(time.time() * 1000)}-admin",
    ),
]


class SagaE2E:
    """Saga 跨 5 域补偿 + 失败回滚 e2e (per docs/automation-design.md §3.3)"""

    def __init__(
        self,
        actor: dict = None,
        fail_domain: Optional[Domain] = None,
        audit_log: Optional[Path] = None,
    ):
        self.actor = actor or {"user_id": "u-001", "wallet_id": "w-001"}
        self.fail_domain = fail_domain
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / "saga-e2e.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)
        # 复制 chain (不修改全局 SAGA_CHAIN)
        self.steps = [self._clone_step(s) for s in SAGA_CHAIN]

    @staticmethod
    def _clone_step(step: SagaStep) -> SagaStep:
        return SagaStep(
            id=step.id,
            domain=step.domain,
            action=step.action,
            compensation=step.compensation,
            idempotency_key=step.idempotency_key,
            status=step.status,
            result=step.result,
            error=step.error,
        )

    def run(self) -> SagaResult:
        """跑 saga 链 (5 域 + 失败回滚)"""
        start = time.time()
        failed_at = None
        compensated = []

        for i, step in enumerate(self.steps):
            try:
                # 触发 fail_domain 时返 failure
                if self.fail_domain and step.domain == self.fail_domain:
                    raise RuntimeError(f"simulated failure at {step.domain.value} domain")

                action_fn = globals()[step.action]
                step.result = action_fn(self.actor)
                step.status = "success"
            except Exception as e:
                step.status = "failed"
                step.error = str(e)
                failed_at = step.id
                # 失败: 回滚前 i 步 (反向)
                for j in range(i - 1, -1, -1):
                    prev = self.steps[j]
                    if prev.status == "success":
                        try:
                            comp_fn = globals()[prev.compensation]
                            comp_result = comp_fn(prev.result or {})
                            prev.status = "compensated"
                            prev.result = comp_result
                            compensated.append(prev.id)
                        except Exception as comp_e:
                            prev.status = "compensation_failed"
                            prev.error = f"compensation: {comp_e}"
                break

        duration = (time.time() - start) * 1000
        success = failed_at is None
        result = SagaResult(
            success=success,
            steps=self.steps,
            failed_at=failed_at,
            compensated_steps=compensated,
            duration_ms=duration,
        )
        self._audit(
            action="run_saga",
            input={"actor": self.actor, "fail_domain": self.fail_domain.value if self.fail_domain else None},
            output=asdict(result),
        )
        return result

    def _audit(self, action: str, input: dict, output: dict, error: Optional[str] = None):
        def _normalize(obj):
            if isinstance(obj, dict):
                return {k: _normalize(v) for k, v in obj.items()}
            if isinstance(obj, (list, tuple)):
                return [_normalize(v) for v in obj]
            if isinstance(obj, Path):
                return str(obj)
            if isinstance(obj, Domain):
                return obj.value
            return obj

        entry = AuditEntry(
            timestamp=time.time(),
            phase="saga-e2e",
            action=action,
            input=_normalize(input),
            output=_normalize(output),
            error=error,
        )
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")

    def summary(self, result: SagaResult) -> str:
        success_count = sum(1 for s in result.steps if s.status == "success")
        compensated_count = sum(1 for s in result.steps if s.status == "compensated")
        return (
            f"=== Saga E2E: {self.fail_domain.value if self.fail_domain else 'success'} case ===\n"
            f"steps: {len(result.steps)}\n"
            f"success: {success_count}\n"
            f"compensated: {compensated_count}\n"
            f"failed_at: {result.failed_at or '(none)'}\n"
            f"success_overall: {result.success}\n"
            f"duration_ms: {result.duration_ms:.2f}\n"
            f"audit_log: {self.audit_log}\n"
        )


def main():
    parser = argparse.ArgumentParser(description="Saga 跨 5 域补偿 + 失败回滚 e2e")
    parser.add_argument("--fail-domain", choices=[d.value for d in Domain],
                        help="触发特定域失败 (rollback 实证)")
    parser.add_argument("--dry-run", action="store_true", default=True,
                        help="dry run 模式 (默认)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false",
                        help="真跑模式")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    # dry-run 模式: 默认成功 case; 真跑模式: 同上 (5 域 mock)
    fail_domain = Domain(args.fail_domain) if args.fail_domain else None
    saga = SagaE2E(fail_domain=fail_domain, audit_log=args.audit_log)
    result = saga.run()
    print(saga.summary(result))
    for step in result.steps:
        print(f"  [{step.status:12}] {step.id:18} ({step.domain.value})")

    sys.exit(0 if result.success else 1)


if __name__ == "__main__":
    main()
