#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
scripts/automation/arg_seed.py — ARG seed data generator (ARG.1 子项)
(per docs/briefs/arg-01-arg-crate-skeleton.md §2.1 C + WBS §14.11 ARG.1)

守门 #1 v19 (P 子项 Python 化) + 守门 #5 (env 安全) + 守门 #3 (5 域 Lead 跨域
边强制 consults):

种子数据 (per brief):
- 5 域 Lead (5 nodes) = Lead{Player,Economy,Match,Social,Admin}
- 9 SA (9 nodes) = SA-01..SA-09
- 10 demo agent (10 nodes)
- 跨域 consults 边: 每对 lead 互相 1 条 = C(5,2) = 10 条 CONSULTS 边
- (per brief 5 域 reports_to 真人 Lead 边 = 5 条, 改用虚拟真人 stub)
合计: 24 节点 + 10+ 5 = 15 边

输出:
- JSON fixture `docs/briefs/arg-01-seed.json` 落档
- 不直接调 Memgraph (P3-C W1 G-1 stub 状态), 写 JSON 给 `arg-bridge` W2 用
- env 走 $env:MEMGRAPH_BOLT_URL 但不打印 (per 守门 #5)

用法:
    python scripts/automation/arg_seed.py
    python scripts/automation/arg_seed.py --output docs/briefs/arg-01-seed.json
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import uuid
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Optional

ROOT_DEFAULT = Path(__file__).resolve().parent.parent.parent
OUTPUT_DEFAULT = ROOT_DEFAULT / "docs" / "briefs" / "arg-01-seed.json"

# 5 域, 跟守门 #3 (5 域独立 Lead) 一致
DOMAINS = ["player", "economy", "match", "social", "admin"]

# 9 SA Archetype (per ADR-0045 + LangGraph 9/3 §6.1)
SA_TYPES = [f"SA-{i:02d}" for i in range(1, 10)]


@dataclass
class SeedAgent:
    id: str
    name: str
    archetype: str
    domain: Optional[str]
    trust_score: float = 0.5
    tenant_id: str = field(default_factory=lambda: str(uuid.UUID("00000000-0000-0000-0000-000000000001")))
    created_by: str = "Ulysses"

    def to_dict(self) -> dict:
        return asdict(self)


@dataclass
class SeedEdge:
    id: str
    from_agent: str
    to_agent: str
    edge_type: str
    weight: float = 0.7
    direction: str = "directed"
    tenant_id: str = field(default_factory=lambda: str(uuid.UUID("00000000-0000-0000-0000-000000000001")))
    created_by: str = "Ulysses"

    def to_dict(self) -> dict:
        return asdict(self)


def build_leads() -> list:
    """5 域 Lead (per AGENTS §5 + 守门 #3 v2)."""
    leads = []
    for d in DOMAINS:
        leads.append(SeedAgent(
            id=str(uuid.uuid4()),
            name=f"lead-{d}",
            archetype=f"LEAD_{d.upper()}",
            domain=d,
            trust_score=0.85,
        ).to_dict())
    return leads


def build_sa_archetypes() -> list:
    """9 SA Archetype (per ADR-0045)."""
    return [
        SeedAgent(
            id=str(uuid.uuid4()),
            name=f"sa-{sa.lower()}",
            archetype=sa,
            domain=None,
            trust_score=0.5,
        ).to_dict()
        for sa in SA_TYPES
    ]


def build_demo_agents() -> list:
    """10 demo agent (per brief 24 - 5 - 9 = 10)."""
    demo_names = [
        ("demo-pm-001", "player"),
        ("demo-pm-002", "player"),
        ("demo-ec-001", "economy"),
        ("demo-mt-001", "match"),
        ("demo-mt-002", "match"),
        ("demo-sc-001", "social"),
        ("demo-sc-002", "social"),
        ("demo-ad-001", "admin"),
        ("demo-ad-002", "admin"),
        ("demo-fl-001", None),  # 自由 agent, 不属于任何域
    ]
    return [
        SeedAgent(
            id=str(uuid.uuid4()),
            name=name,
            archetype="CUSTOM",
            domain=domain,
            trust_score=0.5,
        ).to_dict()
        for name, domain in demo_names
    ]


def build_cross_domain_consults(leads: list) -> list:
    """跨域 lead↔lead consults 边 (per 守门 #3 跨域边强制 consults).

    5 域 Lead round-robin consults: lead[i] → lead[(i+1) % 5] 共 5 条 directed.
    brief 规定 "5 条 consults + 5 域 reports_to 真人 Lead = 10 边" 跟这里一致.
    """
    edges = []
    n = len(leads)
    for i in range(n):
        a = leads[i]
        b = leads[(i + 1) % n]
        edges.append(SeedEdge(
            id=str(uuid.uuid4()),
            from_agent=a["id"],
            to_agent=b["id"],
            edge_type="CONSULTS",
            weight=0.6,
            direction="directed",
        ).to_dict())
    return edges


def build_demo_reports_to(leads: list, demo_agents: list) -> list:
    """5 域 demo agent → 同域 lead REPORTS_TO 边 (per brief '5 域 reports_to 真人 Lead').

    每个域抽 1 个 demo 配 1 个 REPORTS_TO 边, 5 条.
    """
    by_domain_lead = {l["domain"]: l for l in leads}
    edges = []
    # 每个域选 1 个 demo
    seen_domain = set()
    for d in demo_agents:
        d_domain = d.get("domain")
        if d_domain and d_domain in by_domain_lead and d_domain not in seen_domain:
            edges.append(SeedEdge(
                id=str(uuid.uuid4()),
                from_agent=d["id"],
                to_agent=by_domain_lead[d_domain]["id"],
                edge_type="REPORTS_TO",
                weight=0.8,
                direction="directed",
            ).to_dict())
            seen_domain.add(d_domain)
    return edges


def build_fixture(tenant_id: Optional[str] = None) -> dict:
    """Build the full seed fixture."""
    leads = build_leads()
    sas = build_sa_archetypes()
    demo = build_demo_agents()
    consults = build_cross_domain_consults(leads)
    reports = build_demo_reports_to(leads, demo)
    return {
        "fixture_version": 1,
        "tenant_id": tenant_id or str(uuid.UUID("00000000-0000-0000-0000-000000000001")),
        "summary": {
            "agents": len(leads) + len(sas) + len(demo),
            "leads": len(leads),
            "sa_archetypes": len(sas),
            "demo_agents": len(demo),
            "edges": len(consults) + len(reports),
            "consults": len(consults),
            "reports_to": len(reports),
        },
        "agents": leads + sas + demo,
        "edges": consults + reports,
    }


def main():
    parser = argparse.ArgumentParser(description="ARG seed fixture generator (ARG.1)")
    parser.add_argument("--output", type=Path, default=OUTPUT_DEFAULT,
                        help="Output JSON path (default: docs/briefs/arg-01-seed.json)")
    parser.add_argument("--tenant-id", type=str, default=None,
                        help="Override tenant UUID")
    parser.add_argument("--verify-env", action="store_true",
                        help="Check $env:MEMGRAPH_BOLT_URL exists but do not print value (守门 #5)")
    args = parser.parse_args()

    # 守门 #5: env check 不打印值
    if args.verify_env:
        if "MEMGRAPH_BOLT_URL" not in os.environ:
            print("WARN: $env:MEMGRAPH_BOLT_URL not set (will use placeholder)", file=sys.stderr)
        else:
            print("OK: $env:MEMGRAPH_BOLT_URL is set (value not echoed per 守门 #5)")

    fixture = build_fixture(args.tenant_id)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(fixture, ensure_ascii=False, indent=2), encoding="utf-8")

    s = fixture["summary"]
    print(f"OK: ARG seed fixture written to {args.output}")
    print(f"  agents={s['agents']} (leads={s['leads']} + sa={s['sa_archetypes']} + demo={s['demo_agents']})")
    print(f"  edges={s['edges']} (consults={s['consults']} + reports_to={s['reports_to']})")


if __name__ == "__main__":
    main()
