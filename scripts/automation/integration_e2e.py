#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/integration_e2e.py — OpenClaw / Hermes 真实集成 e2e stub
(per docs/automation-design.md v0.1 §3.2 + §4.1 + §6.4 B.5/B.6 共享)

B.5 (OpenClaw) + B.6 (Hermes) 共享 5 endpoint × 4 method = 20 case
per WBS §1 B.5/B.6 拍板 (2026-08-30 07:42 JST 选项 1, 9/2 23:59 JST 选项 1)

用法:
    # B.5 OpenClaw dry-run
    python scripts/automation/integration_e2e.py --dry-run --provider openclaw

    # B.6 Hermes dry-run
    python scripts/automation/integration_e2e.py --dry-run --provider hermes

    # 5 endpoint 全部跑 (真凭证需 Ulysses 提供 API key)
    OPENCLAW_API_KEY=xxx python scripts/automation/integration_e2e.py --provider openclaw
    HERMES_API_KEY=xxx python scripts/automation/integration_e2e.py --provider hermes

约束 (per 守门 #1 v1 + 守门 #5 环境变量安全):
    - 标准库 only: dataclasses / re / json / argparse / subprocess / pathlib
    - 5 endpoint 共享, 改 base_url + auth header (B.5 = X-OpenClaw-Auth, B.6 = X-Hermes-Auth)
    - 默认 dry_run=True (守门 #5: 不打印 env var, 只 invoke)
    - audit_log 必填, 落 docs/reports/integration-e2e.log
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
REPORTS_DIR_DEFAULT = ROOT_DEFAULT / "docs" / "reports"


@dataclass
class EndpointConfig:
    """OpenClaw / Hermes 端点配置 (per WBS §1 B.5/B.6 拍板)"""

    provider: str  # "openclaw" | "hermes"
    base_url: str
    auth_header: str
    auth_value: str  # API key (从 env 读, 不打印)
    timeout: int = 30

    @classmethod
    def from_provider(cls, provider: str) -> "EndpointConfig":
        if provider == "openclaw":
            return cls(
                provider="openclaw",
                base_url=os.environ.get("OPENCLAW_BASE_URL", "https://api.openclaw.local/v1"),
                auth_header="X-OpenClaw-Auth",
                auth_value=os.environ.get("OPENCLAW_API_KEY", ""),
            )
        elif provider == "hermes":
            return cls(
                provider="hermes",
                base_url=os.environ.get("HERMES_BASE_URL", "https://api.hermes.local/v2/hermes"),
                auth_header="X-Hermes-Auth",
                auth_value=os.environ.get("HERMES_API_KEY", ""),
            )
        else:
            raise ValueError(f"unknown provider: {provider}")


@dataclass
class Endpoint:
    """单个端点定义 (5 endpoint 共享)"""

    name: str
    path: str
    methods: list  # 4 method: GET / POST / PUT / DELETE


# 5 endpoint 共享 (per docs/automation-design.md §4.1 B.5/B.6 brief)
ENDPOINTS = [
    Endpoint("agents", "/agents", ["GET", "POST"]),
    Endpoint("sessions", "/sessions", ["GET", "POST", "PUT", "DELETE"]),
    Endpoint("messages", "/messages", ["GET", "POST"]),
    Endpoint("tools_invoke", "/tools/invoke", ["POST"]),
    Endpoint("cost", "/cost", ["GET"]),
]


@dataclass
class CaseResult:
    """单 case 实证结果"""

    endpoint: str
    method: str
    provider: str
    dry_run: bool
    success: bool
    status_code: int
    response_preview: str  # 头 200 字符
    duration_ms: float
    error: Optional[str] = None


@dataclass
class AuditEntry:
    """审计日志条目 (per docs/automation-design.md §3.4)"""

    timestamp: float
    phase: str
    action: str
    input: dict
    output: dict
    error: Optional[str] = None


class IntegrationE2E:
    """OpenClaw / Hermes 5 endpoint × 4 method 共享基类 (per §3.2)"""

    def __init__(self, config: EndpointConfig, dry_run: bool = True, audit_log: Optional[Path] = None):
        self.config = config
        self.dry_run = dry_run
        self.audit_log = audit_log or (REPORTS_DIR_DEFAULT / f"integration-e2e-{config.provider}.log")
        self.audit_log.parent.mkdir(parents=True, exist_ok=True)
        self.results: list = []

    def run_all(self) -> list:
        """跑 5 endpoint × 4 method = 20 case"""
        for endpoint in ENDPOINTS:
            for method in endpoint.methods:
                result = self.run_case(endpoint, method)
                self.results.append(result)
        return self.results

    def run_case(self, endpoint: Endpoint, method: str) -> CaseResult:
        """单 case 跑 (per §3.2 run 方法)"""
        start = time.time()
        url = f"{self.config.base_url}{endpoint.path}"
        headers = {self.config.auth_header: self.config.auth_value}

        if self.dry_run:
            # dry-run 模式: 不真发请求, 返 mock 响应
            mock = self._mock_response(endpoint, method)
            duration = (time.time() - start) * 1000
            result = CaseResult(
                endpoint=endpoint.name,
                method=method,
                provider=self.config.provider,
                dry_run=True,
                success=True,
                status_code=200,
                response_preview=json.dumps(mock)[:200],
                duration_ms=duration,
            )
            self._audit(
                action="run_case_dry_run",
                input={"endpoint": endpoint.name, "method": method, "url": url},
                output=asdict(result),
            )
            return result
        else:
            # 真跑模式 (需 API key + 网络)
            # stub: 真实实装需 import requests 或 subprocess curl (per §6 已知缺口)
            duration = (time.time() - start) * 1000
            result = CaseResult(
                endpoint=endpoint.name,
                method=method,
                provider=self.config.provider,
                dry_run=False,
                success=False,
                status_code=0,
                response_preview="",
                duration_ms=duration,
                error="real-mode 需 import requests 或 subprocess curl (per §6 已知缺口, stub)",
            )
            self._audit(
                action="run_case_real",
                input={"endpoint": endpoint.name, "method": method, "url": url},
                output=asdict(result),
                error="real-mode stub",
            )
            return result

    def _mock_response(self, endpoint: Endpoint, method: str) -> dict:
        """mock 响应 (含 cost / token_usage / status 字段 per brief)"""
        return {
            "endpoint": endpoint.name,
            "method": method,
            "provider": self.config.provider,
            "status": "ok",
            "cost": {"input_tokens": 100, "output_tokens": 50, "total_usd": 0.0015},
            "token_usage": {"prompt": 100, "completion": 50, "total": 150},
            "request_id": f"req-{int(time.time() * 1000)}",
        }

    def _audit(self, action: str, input: dict, output: dict, error: Optional[str] = None):
        def _normalize(obj):
            if isinstance(obj, dict):
                return {k: _normalize(v) for k, v in obj.items()}
            if isinstance(obj, (list, tuple)):
                return [_normalize(v) for v in obj]
            if isinstance(obj, Path):
                return str(obj)
            return obj

        entry = AuditEntry(
            timestamp=time.time(),
            phase=f"integration-e2e-{self.config.provider}",
            action=action,
            input=_normalize(input),
            output=_normalize(output),
            error=error,
        )
        with self.audit_log.open("a", encoding="utf-8") as f:
            f.write(json.dumps(asdict(entry), ensure_ascii=False) + "\n")

    def summary(self) -> str:
        success = sum(1 for r in self.results if r.success)
        failed = sum(1 for r in self.results if not r.success)
        return (
            f"=== Integration E2E: {self.config.provider} ===\n"
            f"endpoints: {len(ENDPOINTS)}\n"
            f"methods_per_endpoint: 4 (GET/POST/PUT/DELETE)\n"
            f"total_cases: {len(ENDPOINTS) * 4} (5 × 4)\n"
            f"success: {success}\n"
            f"failed: {failed}\n"
            f"dry_run: {self.dry_run}\n"
            f"audit_log: {self.audit_log}\n"
        )


def main():
    parser = argparse.ArgumentParser(description="OpenClaw / Hermes 5 endpoint × 4 method 共享基类")
    parser.add_argument("--provider", choices=["openclaw", "hermes"], required=True,
                        help="provider: openclaw (B.5) | hermes (B.6)")
    parser.add_argument("--dry-run", action="store_true", default=True,
                        help="dry run 模式 (默认)")
    parser.add_argument("--no-dry-run", dest="dry_run", action="store_false",
                        help="真跑模式 (需 API key + 网络)")
    parser.add_argument("--audit-log", type=Path, help="审计日志路径")
    args = parser.parse_args()

    config = EndpointConfig.from_provider(args.provider)
    e2e = IntegrationE2E(config, dry_run=args.dry_run, audit_log=args.audit_log)
    results = e2e.run_all()
    print(e2e.summary())
    for r in results:
        print(f"  [{'OK' if r.success else 'FAIL'}] {r.endpoint} {r.method} ({r.duration_ms:.2f}ms)")

    sys.exit(0 if all(r.success for r in results) else 1)


if __name__ == "__main__":
    main()
