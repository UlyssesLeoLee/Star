#!/usr/bin/env python3
# _generate_45_agent_runtime_fixtures.py - Agent Runtime G-1~G-18 落地 fixture generator
# Author: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权)
# 触发: 2026-09-05 07:18 JST user 拍板 "完成剩余轮次" + P3-D Agent Runtime G-1~G-18
# 守门: SRS-001 G-1~G-18 + agent-runtime 02-basic-design v0.1

from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[3]
L1_DIR = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "agent-runtime" / "l1-ecs"
L2_DIR = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "agent-runtime" / "l2-pools"
GUARD_DIR = REPO_ROOT / "tools" / "star-flash-mock" / "mock_data" / "agent-runtime" / "guards"

# 8 Archetype (SA-02..SA-09) ECS
ARCHETYPES = [
    {"id": "SA-02", "name": "test-gen", "description": "test generation archetype"},
    {"id": "SA-03", "name": "5-domain-lead-audit", "description": "5 域 Lead audit archetype"},
    {"id": "SA-04", "name": "git-ops", "description": "git ops archetype (per 守门 #1 R-05)"},
    {"id": "SA-05", "name": "doc-sync", "description": "doc sync archetype (per 守门 #12)"},
    {"id": "SA-06", "name": "refactor", "description": "refactor archetype"},
    {"id": "SA-07", "name": "db-migration", "description": "db migration archetype (per 守门 #13 W/T/M)"},
    {"id": "SA-08", "name": "domain-dev", "description": "domain dev archetype (per 守门 #7 0 unsafe)"},
    {"id": "SA-09", "name": "free-form", "description": "free form archetype (LLM 分解子任务)"},
]

# 13 Systems (per SRS-001 §29-§42)
SYSTEMS = [
    "scheduler", "lifecycle", "event", "planner", "llm",
    "tool", "mcp", "retrieval", "context", "memory",
    "permission", "persistence", "metrics",
]

# 6 L2 Pool (缺 6: HTTP / Tool Reg / RAG / Tokenizer / Rate / CB)
POOLS = [
    {"name": "http-pool", "description": "HTTP connection pool"},
    {"name": "tool-registry", "description": "Tool registry pool"},
    {"name": "rag-pool", "description": "RAG retrieval pool"},
    {"name": "tokenizer", "description": "Tokenizer pool"},
    {"name": "rate-limiter", "description": "Rate limiter pool"},
    {"name": "circuit-breaker", "description": "Circuit breaker pool"},
]

# 18 G-* 守门 (per SRS-001)
GUARDS = [
    {"id": "G-1", "name": "task_queue_no_persistence_gap", "category": "L0", "description": "任务队列无持久化缺口"},
    {"id": "G-2", "name": "ecs_archetype_selection", "category": "L1", "description": "ECS 选型 (bevy_ecs / flecs)"},
    {"id": "G-3", "name": "lightweight_to_ecs_hysteresis", "category": "L0", "description": "Lightweight < 10 → ECS ≥ 12 迟滞区"},
    {"id": "G-4", "name": "agent_lifecycle_hot_warm_cold", "category": "L1", "description": "Agent HOT/WARM/COLD 生命周期"},
    {"id": "G-5", "name": "event_driven_mailbox", "category": "L1", "description": "Event Driven + Mailbox + PayloadRef"},
    {"id": "G-6", "name": "rate_control", "category": "L0", "description": "L0 速率控制"},
    {"id": "G-7", "name": "backpressure_throttle", "category": "L0", "description": "L0 Backpressure 限流 (queue > 1000 触发 429)"},
    {"id": "G-8", "name": "system_scheduler", "category": "L1", "description": "13 Systems 调度器"},
    {"id": "G-9", "name": "system_lifecycle", "category": "L1", "description": "HOT/WARM/COLD 转换 System"},
    {"id": "G-10", "name": "system_event", "category": "L1", "description": "Event System (Mailbox)"},
    {"id": "G-11", "name": "system_planner", "category": "L1", "description": "Planner System (LLM 计划分解)"},
    {"id": "G-12", "name": "system_llm", "category": "L1", "description": "LLM System (per 守门 #4 token-OLU)"},
    {"id": "G-13", "name": "system_tool", "category": "L1", "description": "Tool System (per 16 MCP tool)"},
    {"id": "G-14", "name": "llm_pool", "category": "L2", "description": "LLM Pool (8 providers)"},
    {"id": "G-15", "name": "mcp_pool", "category": "L2", "description": "MCP Pool (16 tool)"},
    {"id": "G-16", "name": "circuit_breaker", "category": "L2", "description": "Circuit Breaker 熔断 (per 守门 #24)"},
    {"id": "G-17", "name": "guard_enforcer", "category": "Cross", "description": "AGENTS.md §4 守门 13 main + 24 派生规 = 37 项自动检查"},
    {"id": "G-18", "name": "known_gap", "category": "Cross", "description": "G-1~G-17 已知缺口跟踪 (per 守门 #11)"},
]


def build_archetype(arch: dict) -> dict:
    return {
        "fixture_version": "v1", "module": "l1-ecs", "type": "archetype", "archetype_id": arch["id"],
        "method": "POST", "scenario": f"ecs-spawn-{arch['name']}",
        "description": f"L1 ECS Archetype: {arch['id']} {arch['name']} (per docs/architecture/2026-09-03-agent-runtime/02-basic-design.md v0.1 + LangGraph 9/3 §6.1)",
        "request": {"archetype_id": arch["id"], "components": ["agent_id", "task_ref", "context_buffer", "guard_violation_state"]},
        "response_200": {
            "entity_id": f"entity-uuid-{arch['id'].lower()}-001", "components_attached": 4, "world": "main",
            "ecs_query_path": f"Query<&mut {arch['id'].replace('-', '')}Component>",
        },
        "fixture_assertion": {"archetype_spawn_ok": True, "components_4_attached": True},
    }


def build_system(sys_name: str) -> dict:
    return {
        "fixture_version": "v1", "module": "l1-ecs", "type": "system", "system_name": sys_name,
        "method": "POST", "scenario": f"system-{sys_name}-exec",
        "description": f"L1 ECS System: {sys_name} (per SRS-001 §29-§42 + agent-runtime 02)",
        "request": {"system_name": sys_name, "params": {"trigger": "scheduled"}},
        "response_200": {
            "system_id": f"system-{sys_name}-001", "execution_status": "success",
            "elapsed_ms": 12, "entities_processed": 50,
        },
        "fixture_assertion": {"system_exec_ok": True, "elapsed_under_50ms": True},
    }


def build_pool(pool: dict) -> dict:
    return {
        "fixture_version": "v1", "module": "l2-pools", "pool": pool["name"], "method": "GET",
        "scenario": f"{pool['name']}-status",
        "description": f"L2 {pool['name']} (per docs/architecture/2026-09-03-agent-runtime/02-basic-design.md v0.1 §2.1 共享池)",
        "request": {},
        "response_200": {
            "pool": pool["name"], "status": "active",
            "active_connections": 8, "max_connections": 64, "circuit_breaker": "closed",
        },
        "fixture_assertion": {"pool_active": True, "circuit_breaker_closed": True},
    }


def build_guard(guard: dict) -> dict:
    return {
        "fixture_version": "v1", "module": "agent-runtime", "type": "guard", "guard_id": guard["id"],
        "guard_name": guard["name"], "category": guard["category"], "method": "GET",
        "scenario": f"guard-{guard['id']}-check",
        "description": f"{guard['id']} {guard['name']} (per SRS-001 G-1~G-18 + agent-runtime 02 + docs/test-design.md v0.7 §21)",
        "request": {},
        "response_200": {
            "guard_id": guard["id"], "guard_name": guard["name"], "category": guard["category"],
            "check_status": "pass", "elapsed_ms": 5,
        },
        "fixture_assertion": {"guard_check_pass": True, "elapsed_under_10ms": True},
    }


def main():
    L1_DIR.mkdir(parents=True, exist_ok=True)
    L2_DIR.mkdir(parents=True, exist_ok=True)
    GUARD_DIR.mkdir(parents=True, exist_ok=True)
    created = 0
    skipped = 0
    # 8 Archetype
    for arch in ARCHETYPES:
        path = L1_DIR / f"v1--l1--ecs--archetype--{arch['id'].lower()}.json"
        if path.exists():
            skipped += 1
        else:
            path.write_text(json.dumps(build_archetype(arch), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            created += 1
    # 13 Systems
    for sys_name in SYSTEMS:
        path = L1_DIR / f"v1--l1--ecs--system--{sys_name}.json"
        if path.exists():
            skipped += 1
        else:
            path.write_text(json.dumps(build_system(sys_name), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            created += 1
    # 6 Pool
    for pool in POOLS:
        path = L2_DIR / f"v1--l2--pool--{pool['name']}.json"
        if path.exists():
            skipped += 1
        else:
            path.write_text(json.dumps(build_pool(pool), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            created += 1
    # 18 G-* 守门
    for guard in GUARDS:
        path = GUARD_DIR / f"v1--guard--{guard['id'].lower()}.json"
        if path.exists():
            skipped += 1
        else:
            path.write_text(json.dumps(build_guard(guard), ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
            created += 1
    print(f"created: {created}, skipped: {skipped}, total: 8+13+6+18 = 45")


if __name__ == "__main__":
    main()
