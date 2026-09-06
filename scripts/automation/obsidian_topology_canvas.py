#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
obsidian_topology_canvas.py — 给 8 份拓扑文件生成对应 .canvas (Obsidian Canvas 插件 JSON).

Obsidian Canvas 1.0 格式 (per Obsidian Canvas 1.0 spec):
{
  "nodes": [
    {"id": "...", "type": "text", "text": "...", "x": 0, "y": 0, "width": 200, "height": 60, "color": "1"}
  ],
  "edges": [
    {"id": "...", "fromNode": "...", "fromSide": "right", "toNode": "...", "toSide": "left", "label": "..."}
  ]
}

本脚本: 为每份拓扑生成一份 .canvas, 节点 = 拓扑中出现的关键节点, 边 = 显式关系.
颜色按 layer (per obsidian_topology_gen.py COLOR_BY_LAYER).
"""

import json
from pathlib import Path
from typing import Any

REPO_ROOT = Path("D:/Star")
WIKI_DIR = REPO_ROOT / "docs" / "wiki" / "docswiki"
CANVAS_DIR = WIKI_DIR / "canvas"
TODAY = "2026-09-06"

# 颜色按 layer (Obsidian Canvas 1-6 palette)
LAYER_COLOR = {
    "source": "6",         # 紫
    "ui": "4",             # 蓝
    "orchestration": "5",  # 紫
    "runtime": "3",        # 橙
    "domain": "2",         # 绿
    "persistence": "1",    # 红
    "platform": "6",       # 紫
}


def node(id_: str, text: str, x: int, y: int, w: int = 240, h: int = 80, color: str = "5", layer: str = "node") -> dict:
    return {
        "id": id_,
        "type": "text",
        "text": text,
        "x": x, "y": y, "width": w, "height": h,
        "color": color,
        "label": f"layer={layer}",
    }


def edge(id_: str, from_node: str, to_node: str, label: str = "", from_side: str = "right", to_side: str = "left") -> dict:
    e = {
        "id": id_,
        "fromNode": from_node,
        "fromSide": from_side,
        "toNode": to_node,
        "toSide": to_side,
    }
    if label:
        e["label"] = label
    return e


# ============================================================
# 8 份 canvas 定义
# ============================================================

def canvas_00_overview() -> dict:
    """00 — 设计应有的工程总览 (主图)."""
    nodes = []
    edges = []
    # 第 1 行: 7 源头
    sources = ["S1", "S2", "S3", "S4", "S5", "S6", "S7"]
    for i, s in enumerate(sources):
        nodes.append(node(s, s, x=i * 260, y=-400, color=LAYER_COLOR["source"]))
    # 第 2 行: 3 View
    nodes.append(node("View-AgentView", "View-AgentView\n(派生视图)", x=0, y=-200, w=300, h=100, color=LAYER_COLOR["ui"]))
    nodes.append(node("View-LangGraph", "View-LangGraph\n(2-level hierarchical)", x=400, y=-200, w=300, h=100, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("View-AgentRuntime", "View-AgentRuntime\n(L0/L1/L2)", x=800, y=-200, w=300, h=100, color=LAYER_COLOR["runtime"]))
    # 第 3 行: 主要组件
    nodes.append(node("M-AGV-9", "M-AGV-9\npage.tsx", x=-200, y=0, color=LAYER_COLOR["ui"]))
    nodes.append(node("C-01", "C-01\nTopAgent", x=200, y=0, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-16", "C-16\nTaskOpsMgr", x=400, y=0, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-04", "C-04\nCheckpoint", x=600, y=0, color=LAYER_COLOR["persistence"]))
    nodes.append(node("C-05", "C-05\nMcpClient", x=800, y=0, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("Comp-AgentIdentity", "Comp-AgentIdentity", x=1000, y=0, color=LAYER_COLOR["runtime"]))
    # 第 4 行: 数据层
    nodes.append(node("domain-tenant", "domain-tenant", x=200, y=200, color=LAYER_COLOR["domain"]))
    nodes.append(node("domain-agent", "domain-agent", x=400, y=200, color=LAYER_COLOR["domain"]))
    nodes.append(node("domain-worktree", "domain-worktree", x=600, y=200, color=LAYER_COLOR["domain"]))
    nodes.append(node("PG-checkpoints", "PG-checkpoints (T)", x=800, y=200, color=LAYER_COLOR["persistence"]))
    nodes.append(node("PG-checkpoint_metadata", "PG-checkpoint_metadata (M)", x=1000, y=200, color=LAYER_COLOR["persistence"]))
    # 第 5 行: 平台
    nodes.append(node("Envoy", "Envoy\n(独立 deployment)", x=800, y=400, w=260, h=80, color=LAYER_COLOR["platform"]))
    nodes.append(node("k3s", "k3s", x=1080, y=400, w=180, h=80, color=LAYER_COLOR["platform"]))

    # 边
    edges.append(edge("e-1", "S1", "View-AgentView", "per 守门 #3"))
    edges.append(edge("e-2", "S2", "View-LangGraph", "per 守门 #3"))
    edges.append(edge("e-3", "S3", "View-LangGraph", "per 守门 #3"))
    edges.append(edge("e-4", "S4", "View-AgentRuntime", "per 守门 #3"))
    edges.append(edge("e-5", "S5", "View-AgentRuntime", "per 守门 #3"))
    edges.append(edge("e-6", "S6", "domain-tenant", "Tier 1"))
    edges.append(edge("e-7", "S7", "PG-checkpoints", "5 表 W/T/M"))
    edges.append(edge("e-8", "View-AgentView", "M-AGV-9", "派生视图"))
    edges.append(edge("e-9", "View-LangGraph", "C-01", "L0 整体"))
    edges.append(edge("e-10", "View-LangGraph", "C-16", "TMO 7 节点"))
    edges.append(edge("e-11", "View-LangGraph", "C-04", "3-tier checkpoint"))
    edges.append(edge("e-12", "View-LangGraph", "C-05", "MCP 16 tools"))
    edges.append(edge("e-13", "View-AgentRuntime", "Comp-AgentIdentity", "L1 ECS"))
    edges.append(edge("e-14", "C-01", "C-16", "TMO 协调"))
    edges.append(edge("e-15", "C-04", "PG-checkpoints", "Memory→SQLite→PG"))
    edges.append(edge("e-16", "C-04", "PG-checkpoint_metadata", "SCD Type 2"))
    edges.append(edge("e-17", "Comp-AgentIdentity", "domain-agent", "L1 跟 22 domain 映射"))
    edges.append(edge("e-18", "domain-tenant", "domain-agent", "依赖 Tier 1→3"))
    edges.append(edge("e-19", "domain-worktree", "domain-agent", "依赖 Tier 2→3"))
    edges.append(edge("e-20", "PG-checkpoints", "Envoy", "mTLS fronting"))
    edges.append(edge("e-21", "Envoy", "k3s", "deploy"))
    return {"nodes": nodes, "edges": edges}


def canvas_01_agent_view() -> dict:
    """01 — Agent View 拓扑 (3-tier + 10 模块)."""
    nodes = []
    edges = []
    # UI Tier
    nodes.append(node("S1", "S1 BD-AGENT-VIEW-001", x=0, y=-300, color=LAYER_COLOR["source"]))
    nodes.append(node("M-AGV-9", "M-AGV-9\npage.tsx\n(root)", x=0, y=-100, w=260, h=100, color=LAYER_COLOR["ui"]))
    nodes.append(node("M-AGV-6", "M-AGV-6\nAgentCanvasView\n(SVG)", x=-300, y=100, w=240, h=100, color=LAYER_COLOR["ui"]))
    nodes.append(node("M-AGV-8", "M-AGV-8\nAgentFilter\n(dropdown)", x=300, y=100, w=240, h=100, color=LAYER_COLOR["ui"]))
    nodes.append(node("M-AGV-2", "M-AGV-2\nselectors.ts\n(7 pure fns)", x=-200, y=300, color=LAYER_COLOR["ui"]))
    nodes.append(node("M-AGV-3", "M-AGV-3\nlayout.ts\n(2 pure fns)", x=200, y=300, color=LAYER_COLOR["ui"]))
    nodes.append(node("M-AGV-1", "M-AGV-1\ntypes.ts\n(leaf)", x=0, y=450, color=LAYER_COLOR["ui"]))
    # Store
    nodes.append(node("Store", "zustand store\nagentSessions/worktrees/workItems", x=0, y=600, w=300, h=100, color=LAYER_COLOR["ui"]))
    # Domain 引用
    nodes.append(node("domain-worktree", "domain-worktree\n(1:1 关联)", x=-400, y=600, color=LAYER_COLOR["domain"]))
    nodes.append(node("domain-work-item", "domain-work-item\n(1:N via wt_id)", x=400, y=600, color=LAYER_COLOR["domain"]))
    # 边
    edges.append(edge("e-s1", "S1", "M-AGV-9", "defines"))
    edges.append(edge("e-1", "M-AGV-9", "M-AGV-6", "render"))
    edges.append(edge("e-2", "M-AGV-9", "M-AGV-8", "render"))
    edges.append(edge("e-3", "M-AGV-9", "M-AGV-2", "useMemo"))
    edges.append(edge("e-4", "M-AGV-9", "M-AGV-3", "useMemo"))
    edges.append(edge("e-5", "M-AGV-9", "M-AGV-1", "types"))
    edges.append(edge("e-6", "M-AGV-9", "Store", "useStore 订阅"))
    edges.append(edge("e-7", "M-AGV-6", "M-AGV-1", "types"))
    edges.append(edge("e-8", "M-AGV-8", "M-AGV-2", "selected"))
    edges.append(edge("e-9", "M-AGV-2", "M-AGV-1", "deps"))
    edges.append(edge("e-10", "M-AGV-3", "M-AGV-1", "deps"))
    edges.append(edge("e-11", "Store", "domain-worktree", "reads"))
    edges.append(edge("e-12", "Store", "domain-work-item", "reads"))
    return {"nodes": nodes, "edges": edges}


def canvas_02_langgraph() -> dict:
    """02 — LangGraph Orchestration (L0 + TMO + L1 + Cross)."""
    nodes = []
    edges = []
    # 来源
    nodes.append(node("S2", "S2 LangGraph 02", x=-600, y=-400, color=LAYER_COLOR["source"]))
    nodes.append(node("S3", "S3 LangGraph 03", x=-400, y=-400, color=LAYER_COLOR["source"]))
    # L0
    nodes.append(node("C-01", "C-01\nTopAgent", x=-400, y=-200, w=240, h=80, color=LAYER_COLOR["orchestration"]))
    # T-N1..T-N7
    for i, n in enumerate(["T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7"]):
        nodes.append(node(n, n, x=-700 + i * 90, y=-50, w=80, h=50, color=LAYER_COLOR["orchestration"]))
    # TMO C-16 + M-N1..M-N7
    nodes.append(node("C-16", "C-16\nTaskOpsManager", x=200, y=-200, w=260, h=80, color=LAYER_COLOR["orchestration"]))
    tmo_names = ["M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"]
    for i, m in enumerate(tmo_names):
        nodes.append(node(m, m, x=80 + i * 80, y=-50, w=70, h=50, color=LAYER_COLOR["orchestration"]))
    # L1 SA
    nodes.append(node("C-02", "C-02\nSubAgentPool", x=600, y=-200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    sa_names = ["SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09", "SA-10"]
    for i, sa in enumerate(sa_names):
        nodes.append(node(sa, sa, x=400 + i * 70, y=20, w=60, h=50, color=LAYER_COLOR["orchestration"]))
    # Cross-Cutting
    nodes.append(node("C-04", "C-04\nCheckpointStore", x=-600, y=200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-05", "C-05\nMcpClient", x=-300, y=200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-08", "C-08\nAuditLogger", x=0, y=200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-10", "C-10\nGuardEnforcer", x=300, y=200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-12", "C-12\nInterruptManager", x=600, y=200, w=240, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-06", "C-06\nUIStreamer", x=900, y=200, w=220, h=80, color=LAYER_COLOR["orchestration"]))
    # 边
    for n in ["T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7"]:
        edges.append(edge(f"e-{n}-C01", n, "C-01", "L0 node"))
    edges.append(edge("e-S2-C01", "S2", "C-01", "defines"))
    edges.append(edge("e-S3-C01", "S3", "C-01", "refines"))
    edges.append(edge("e-C01-C16", "C-01", "C-16", "TMO 协调"))
    for m in tmo_names:
        edges.append(edge(f"e-{m}-C16", m, "C-16", "TMO 7 节点"))
    edges.append(edge("e-C01-C02", "C-01", "C-02", "spawn"))
    edges.append(edge("e-C02-SA", "C-02", "C-02", "spawn"))  # dummy
    for sa in sa_names:
        edges.append(edge(f"e-C02-{sa}", "C-02", sa, "spawn"))
    edges.append(edge("e-SA-C05", "SA-01", "C-05", "MCP tool"))
    edges.append(edge("e-C10-C05", "C-10", "C-05", "guard"))
    edges.append(edge("e-C08-C05", "C-08", "C-05", "audit"))
    edges.append(edge("e-C12-C02", "C-12", "C-02", "interrupt"))
    edges.append(edge("e-C04-S2", "S2", "C-04", "defines"))
    edges.append(edge("e-C06-C01", "C-06", "C-01", "stream"))
    return {"nodes": nodes, "edges": edges}


def canvas_03_runtime() -> dict:
    """03 — Agent Runtime 拓扑 (L0/L1/L2 + 双模式)."""
    nodes = []
    edges = []
    # 来源
    nodes.append(node("S4", "S4 Runtime 02", x=0, y=-500, color=LAYER_COLOR["source"]))
    nodes.append(node("S5", "S5 Runtime 03", x=200, y=-500, color=LAYER_COLOR["source"]))
    # 模式
    nodes.append(node("Lightweight", "Lightweight\n< 10 Agent", x=-200, y=-300, w=200, h=80, color=LAYER_COLOR["runtime"]))
    nodes.append(node("Hysteresis", "Hysteresis\n10-11", x=0, y=-300, w=200, h=80, color=LAYER_COLOR["runtime"]))
    nodes.append(node("ECS-Mode", "ECS Mode\n≥ 12 Agent", x=200, y=-300, w=200, h=80, color=LAYER_COLOR["runtime"]))
    # L0
    nodes.append(node("L0-Dispatch", "L0 派发层\n7 组件", x=-400, y=-100, w=260, h=100, color=LAYER_COLOR["runtime"]))
    # L1
    nodes.append(node("L1-ECS", "L1 ECS\n9 Archetype + 12 Comp + 13 Sys", x=0, y=-100, w=320, h=100, color=LAYER_COLOR["runtime"]))
    # L2
    nodes.append(node("L2-Pool", "L2 业务共享池\nLLM/MCP/HTTP/Tool/RAG/...", x=400, y=-100, w=320, h=100, color=LAYER_COLOR["runtime"]))
    # 9 新建 domain
    new_doms = ["domain-dispatcher", "domain-llm", "domain-mcp", "domain-tool", "domain-rag", "domain-context", "domain-memory", "domain-rate-limiter", "domain-observability", "domain-agent"]
    for i, d in enumerate(new_doms):
        nodes.append(node(d, d, x=-500 + (i % 5) * 200, y=150 + (i // 5) * 100, w=180, h=70, color=LAYER_COLOR["domain"]))
    # 9 SA Archetype
    for i, sa in enumerate(["SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09"]):
        nodes.append(node(sa, sa, x=-200 + (i % 5) * 100, y=400 + (i // 5) * 100, w=90, h=60, color=LAYER_COLOR["orchestration"]))
    # 边
    edges.append(edge("e-S4-RT", "S4", "L0-Dispatch", "defines"))
    edges.append(edge("e-S4-L1", "S4", "L1-ECS", "defines"))
    edges.append(edge("e-S4-L2", "S4", "L2-Pool", "defines"))
    edges.append(edge("e-S5-L1", "S5", "L1-ECS", "refines"))
    edges.append(edge("e-Mode-L0", "Lightweight", "L0-Dispatch", "Tokio async"))
    edges.append(edge("e-Mode-L1", "ECS-Mode", "L1-ECS", "9 Archetype"))
    edges.append(edge("e-Mode-H", "Lightweight", "Hysteresis", "10-11"))
    edges.append(edge("e-H-Mode", "Hysteresis", "ECS-Mode", "12 持续 30s"))
    edges.append(edge("e-L0-L1", "L0-Dispatch", "L1-ECS", "Tokio task 派发"))
    edges.append(edge("e-L1-L2", "L1-ECS", "L2-Pool", "业务请求"))
    for d in new_doms:
        edges.append(edge(f"e-L1-{d}", "L1-ECS", d, "映射"))
    for sa in ["SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09"]:
        edges.append(edge(f"e-L1-{sa}", "L1-ECS", sa, "Archetype"))
    return {"nodes": nodes, "edges": edges}


def canvas_04_domain() -> dict:
    """04 — 22 domain crate Tier 1-6."""
    nodes = []
    edges = []
    nodes.append(node("S6", "S6 22 domain 接入", x=0, y=-400, color=LAYER_COLOR["source"]))
    # 6 Tier
    tiers = {
        "Tier 1": ["domain-tenant", "domain-identity", "domain-permission"],
        "Tier 2": ["domain-workspace", "domain-project", "domain-work-item"],
        "Tier 3": ["domain-worktree", "domain-agent", "domain-feedback", "domain-decision"],
        "Tier 4": ["domain-scm", "domain-validation", "domain-automation", "domain-search"],
        "Tier 5": ["domain-policy", "domain-notification", "domain-context", "domain-resume"],
        "Tier 6": ["domain-audit", "domain-integration", "domain-event", "domain-flow", "domain-lease"],
    }
    y = -100
    for tier, crates in tiers.items():
        nodes.append(node(f"T-{tier.replace(' ', '')}", tier, x=-400, y=y, w=140, h=80, color=LAYER_COLOR["domain"]))
        for i, c in enumerate(crates):
            x = -200 + i * 200
            nodes.append(node(c, c, x=x, y=y, w=180, h=70, color=LAYER_COLOR["domain"]))
            edges.append(edge(f"e-S6-{c}", "S6", c, f"per {tier}"))
        y += 100
    # Tier 1→2→3→4→5→6 依赖
    for i in range(1, 6):
        edges.append(edge(f"e-T{i}-T{i+1}", f"T-Tier{i}", f"T-Tier{i+1}", "依赖"))
    # 9 新建
    new9 = ["domain-dispatcher", "domain-llm", "domain-mcp", "domain-tool", "domain-rag", "domain-context", "domain-memory", "domain-rate-limiter", "domain-observability"]
    for i, d in enumerate(new9):
        x = 800 + (i % 3) * 200
        y2 = -200 + (i // 3) * 100
        nodes.append(node(d, d + "\n★ 新建", x=x, y=y2, w=180, h=70, color=LAYER_COLOR["domain"]))
    return {"nodes": nodes, "edges": edges}


def canvas_05_persistence() -> dict:
    """05 — 3-tier Checkpoint + 5 张表 W/T/M."""
    nodes = []
    edges = []
    nodes.append(node("S7", "S7 PG Checkpointer Tier 3", x=0, y=-400, color=LAYER_COLOR["source"]))
    nodes.append(node("C-04", "C-04 CheckpointStore\n(3-tier ABC)", x=0, y=-200, w=280, h=80, color=LAYER_COLOR["orchestration"]))
    # 3 tier
    nodes.append(node("Tier-1-Memory", "Tier 1 Memory\n(MemorySaver)", x=-300, y=0, w=200, h=80, color=LAYER_COLOR["persistence"]))
    nodes.append(node("Tier-2-SQLite", "Tier 2 SQLite\n(SqliteSaver, v0.1 default)", x=0, y=0, w=200, h=80, color=LAYER_COLOR["persistence"]))
    nodes.append(node("Tier-3-PG", "Tier 3 PostgreSQL\n(PostgresSaver, v0.2 production)", x=300, y=0, w=240, h=80, color=LAYER_COLOR["persistence"]))
    # 5 张表
    tables = [
        ("PG-checkpoints", "checkpoints (T)", "1"),
        ("PG-checkpoint_writes", "checkpoint_writes (T)", "1"),
        ("PG-checkpoint_summaries", "checkpoint_summaries (T)", "1"),
        ("PG-checkpoint_metadata", "checkpoint_metadata (M)\nSCD Type 2", "2"),
        ("PG-audit_audit_event", "audit_audit_event (T WORM)\nper ADR-0043", "1"),
    ]
    for i, (id_, label, color_idx) in enumerate(tables):
        nodes.append(node(id_, label, x=-400 + i * 200, y=200, w=180, h=80, color=color_idx))
        edges.append(edge(f"e-S7-{id_}", "S7", id_, "per §3.2"))
        edges.append(edge(f"e-T3-{id_}", "Tier-3-PG", id_, "writes"))
    edges.append(edge("e-C04-T1", "C-04", "Tier-1-Memory", "default"))
    edges.append(edge("e-C04-T2", "C-04", "Tier-2-SQLite", "default"))
    edges.append(edge("e-C04-T3", "C-04", "Tier-3-PG", "T3 启动"))
    edges.append(edge("e-T1-T2", "Tier-1-Memory", "Tier-2-SQLite", "async flush"))
    edges.append(edge("e-T2-T3", "Tier-2-SQLite", "Tier-3-PG", "5 域 Lead T3 触发"))
    # TMO 7 节点
    tmo_names = ["M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"]
    for i, m in enumerate(tmo_names):
        nodes.append(node(m, m, x=-400 + i * 130, y=400, w=120, h=60, color=LAYER_COLOR["orchestration"]))
        edges.append(edge(f"e-{m}-T3", m, "Tier-3-PG", "writes"))
    return {"nodes": nodes, "edges": edges}


def canvas_06_dataflow() -> dict:
    """06 — 跨 View 关键数据流."""
    nodes = []
    edges = []
    # 7 个数据流的简化拓扑
    nodes.append(node("S2", "S2 LangGraph 02", x=-400, y=-400, color=LAYER_COLOR["source"]))
    nodes.append(node("S4", "S4 Runtime 02", x=400, y=-400, color=LAYER_COLOR["source"]))
    # User → ChatBar → API → Top → Pool → SA → MCP → DB
    nodes.append(node("User", "User", x=-600, y=-100, w=80, h=60, color="5"))
    nodes.append(node("ChatBar", "Chat Bar\n(UI)", x=-400, y=-100, w=160, h=80, color=LAYER_COLOR["ui"]))
    nodes.append(node("API", "Backend API\n(FastAPI)", x=-200, y=-100, w=180, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-01", "C-01 TopAgent\n(L0)", x=0, y=-100, w=200, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-02", "C-02 SubAgentPool", x=200, y=-100, w=200, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("SA-08", "SA-08\ndomain-dev", x=400, y=-100, w=160, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-05", "C-05 McpClient", x=600, y=-100, w=200, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("star-mcp", "star-mcp\n16 tools", x=800, y=-100, w=160, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("C-04", "C-04 Checkpoint", x=200, y=200, w=200, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("WS", "WebSocket\nstream", x=-200, y=200, w=180, h=80, color=LAYER_COLOR["ui"]))
    nodes.append(node("Card", "Task Card Modal", x=-400, y=200, w=160, h=80, color=LAYER_COLOR["ui"]))
    # TMO
    nodes.append(node("C-16", "C-16 TaskOpsManager", x=400, y=200, w=240, h=80, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("M-N1", "M-N1 merge", x=400, y=350, w=140, h=60, color=LAYER_COLOR["orchestration"]))
    nodes.append(node("PG-checkpoints", "PG-checkpoints (T)", x=600, y=350, w=200, h=60, color=LAYER_COLOR["persistence"]))
    # Runtime
    nodes.append(node("L1-ECS", "L1 ECS\n(Agent Runtime)", x=800, y=200, w=200, h=80, color=LAYER_COLOR["runtime"]))
    # 边
    edges.append(edge("e-1", "User", "ChatBar", "input"))
    edges.append(edge("e-2", "ChatBar", "API", "POST /api/top-agent/dispatch"))
    edges.append(edge("e-3", "API", "C-01", "invoke"))
    edges.append(edge("e-4", "C-01", "C-02", "spawn"))
    edges.append(edge("e-5", "C-02", "SA-08", "sub-agent"))
    edges.append(edge("e-6", "SA-08", "C-05", "AuditedMcpToolNode"))
    edges.append(edge("e-7", "C-05", "star-mcp", "stdio/Streamable"))
    edges.append(edge("e-8", "C-01", "C-04", "save"))
    edges.append(edge("e-9", "C-04", "PG-checkpoints", "Tier 3"))
    edges.append(edge("e-10", "C-01", "WS", "push"))
    edges.append(edge("e-11", "WS", "Card", "stream"))
    edges.append(edge("e-12", "User", "Card", "click"))
    edges.append(edge("e-13", "C-01", "C-16", "TMO"))
    edges.append(edge("e-14", "C-16", "M-N1", "merge"))
    edges.append(edge("e-15", "M-N1", "PG-checkpoints", "write"))
    edges.append(edge("e-16", "C-02", "L1-ECS", "9 Archetype"))
    return {"nodes": nodes, "edges": edges}


def canvas_99_broker() -> dict:
    """99 — pgwiki broker 审计 (dual-namespace 拆解)."""
    nodes = []
    edges = []
    # 中心: cargo metadata
    nodes.append(node("cargo-metadata", "cargo metadata\n52 crate 实测\n(per 守门 #1 v19 + §4.2)", x=0, y=-300, w=300, h=100, color="6"))
    # domain-* 34 (绿)
    domain_list = ["domain-tenant", "domain-identity", "domain-permission", "domain-workspace", "domain-project", "domain-work-item", "domain-worktree", "domain-agent", "domain-feedback", "domain-decision", "domain-scm", "domain-validation", "domain-automation", "domain-search", "domain-policy", "domain-notification", "domain-context", "domain-resume", "domain-audit", "domain-integration", "domain-event", "domain-flow", "domain-ai", "domain-batch", "domain-board", "domain-cli", "domain-collaboration", "domain-comment", "domain-dashboard", "domain-development", "domain-form", "domain-kms", "domain-local-runtime", "domain-planning", "domain-relation", "domain-report", "domain-theme", "domain-workflow"]
    for i, d in enumerate(domain_list[:20]):  # 限 20 避免过载
        nodes.append(node(d, d, x=-800 + (i % 10) * 150, y=0, w=140, h=50, color="2"))
    nodes.append(node("domain-more-14", f".. +{len(domain_list)-20} more", x=600, y=0, w=160, h=50, color="2"))
    # star-* 15 (橙)
    star_list = ["star-mcp", "star-api-rest", "star-cli", "star-saga", "star-sa", "star-dispatcher", "star-cache", "star-credential", "star-treesitter", "star-context", "star-sse", "star-webhook", "star-taskgraph", "star-vcs", "star-dto"]
    for i, s in enumerate(star_list):
        nodes.append(node(s, f"{s}\n({['49','20','16','11','6','5','4','4','4','3','3','3','2','2','1'][i]} src)", x=-800 + (i % 5) * 320, y=200 + (i // 5) * 100, w=300, h=70, color="3"))
    # other 3 (紫)
    nodes.append(node("api", "api\n(REST 入口)", x=-400, y=600, w=200, h=80, color="6"))
    nodes.append(node("application", "application\n(Application Layer)", x=-100, y=600, w=200, h=80, color="6"))
    nodes.append(node("infrastructure", "infrastructure\n(Infra 聚合)", x=200, y=600, w=200, h=80, color="6"))
    # 设计意图 9 (红 - 警示)
    design_only = ["domain-dispatcher-design", "domain-llm-design", "domain-mcp-design", "domain-tool-design", "domain-rag-design", "domain-memory-design", "domain-rate-limiter-design", "domain-observability-design"]
    for i, d in enumerate(design_only):
        nodes.append(node(d, d, x=600 + (i % 4) * 200, y=0 + (i // 4) * 80, w=180, h=60, color="1"))
    # broker 32 (灰)
    nodes.append(node("pgwiki-broker-32", "pgwiki 50-issues\nbroker-arch 32\n(全正确)", x=600, y=300, w=240, h=80, color="7"))
    # 边
    for d in domain_list[:20]:
        edges.append(edge(f"e-cargo-{d}", "cargo-metadata", d, "domain-*"))
    edges.append(edge("e-cargo-domain-more", "cargo-metadata", "domain-more-14", "+14"))
    for s in star_list:
        edges.append(edge(f"e-cargo-{s}", "cargo-metadata", s, "star-*"))
    for o in ["api", "application", "infrastructure"]:
        edges.append(edge(f"e-cargo-{o}", "cargo-metadata", o, "other"))
    for d in design_only:
        edges.append(edge(f"e-{d}-broker", d, "pgwiki-broker-32", "in broker-arch"))
    # 命名冲突
    edges.append(edge("e-conflict-dispatcher", "domain-dispatcher-design", "star-dispatcher", "命名冲突"))
    edges.append(edge("e-conflict-mcp", "domain-mcp-design", "star-mcp", "命名冲突"))
    edges.append(edge("e-conflict-context", "star-context", "domain-context", "命名冲突"))
    return {"nodes": nodes, "edges": edges}


def canvas_99_diff() -> dict:
    """99 — docswiki vs pgwiki 差异 (主色蓝/绿, 共同紫)."""
    nodes = []
    edges = []
    # docswiki 8 份主题 (蓝)
    dw_files = ["00-design-topology", "01-ui-agent-view", "02-orchestration-langgraph", "03-runtime-ecs", "04-domain-crates", "05-persistence-checkpoint", "06-data-flow", "README"]
    dw_titles = {
        "00-design-topology": "00 主图\n(设计应有)",
        "01-ui-agent-view": "01 AgentView\n(派生视图)",
        "02-orchestration-langgraph": "02 LangGraph\n(应有)",
        "03-runtime-ecs": "03 Runtime\n(应有)",
        "04-domain-crates": "04 Domain\n22+9 目标",
        "05-persistence-checkpoint": "05 Persistence\n5 PG 表",
        "06-data-flow": "06 DataFlow\n7 流",
        "README": "README 索引",
    }
    for i, f in enumerate(dw_files):
        nodes.append(node(f, f"\n{dw_titles[f]}", x=(i % 4) * 320, y=0, w=280, h=100, color="4"))
    # pgwiki 5 主题 (绿)
    pw_themes = [("pgwiki-10-workspace-MOC", "10-workspace\n52 crate", 0), ("pgwiki-20-database-MOC", "20-database\n25 schema/93 表", 1), ("pgwiki-30-architecture-MOC", "30-architecture\n28 ADR", 2), ("pgwiki-40-crosscutting-dependencies", "40-crosscutting\ndeps + gates", 3), ("pgwiki-50-issues-MOC", "50-issues\n6 问题", 4)]
    for label, text, i in pw_themes:
        nodes.append(node(label, label + "\n" + text, x=(i % 5) * 280, y=200, w=240, h=100, color="2"))
    # 共同 28 ADR (紫)
    nodes.append(node("ADR-0021..0047", "28 ADR\n0021..0047\n(共同)", x=0, y=400, w=280, h=100, color="6"))
    # 量化对比 (红 - 警示)
    nodes.append(node("cargo-52", "cargo members\n52 (vs 22+9=31 设计)", x=400, y=400, w=240, h=100, color="1"))
    nodes.append(node("schema-93", "DB tables\n93 (vs 5 设计 Tier 3)", x=700, y=400, w=240, h=100, color="1"))
    nodes.append(node("broker-32", "broker arch\n32 (未实装)", x=1000, y=400, w=240, h=100, color="1"))
    # 来源
    nodes.append(node("S1..S7", "7 源头\n(共同)", x=600, y=-100, w=280, h=80, color="6"))
    # 边
    for f in dw_files:
        edges.append(edge(f"e-{f}-S17", f, "S1..S7", "defines"))
        edges.append(edge(f"e-{f}-ADR", f, "ADR-0021..0047", "cites"))
    for label, text, i in pw_themes:
        edges.append(edge(f"e-{label}-ADR", label, "ADR-0021..0047", "cites"))
    edges.append(edge("e-cargo-ADR", "cargo-52", "ADR-0021..0047", "fact"))
    edges.append(edge("e-schema-ADR", "schema-93", "ADR-0021..0047", "fact"))
    edges.append(edge("e-broker-ADR", "broker-32", "ADR-0021..0047", "broker"))
    edges.append(edge("e-cargo-issues", "cargo-52", "pgwiki-50-issues-MOC", "exposes"))
    edges.append(edge("e-schema-issues", "schema-93", "pgwiki-50-issues-MOC", "exposes"))
    edges.append(edge("e-broker-issues", "broker-32", "pgwiki-50-issues-MOC", "exposes"))
    return {"nodes": nodes, "edges": edges}


def canvas_readme() -> dict:
    """README — 索引图."""
    nodes = []
    edges = []
    # 7 源头
    sources = ["S1", "S2", "S3", "S4", "S5", "S6", "S7"]
    for i, s in enumerate(sources):
        nodes.append(node(s, s, x=i * 220, y=-400, w=180, h=60, color=LAYER_COLOR["source"]))
    # 8 拓扑文件
    files = ["00-design-topology", "01-ui-agent-view", "02-orchestration-langgraph", "03-runtime-ecs", "04-domain-crates", "05-persistence-checkpoint", "06-data-flow", "README"]
    file_titles = {
        "00-design-topology": "00 主图",
        "01-ui-agent-view": "01 AgentView",
        "02-orchestration-langgraph": "02 LangGraph",
        "03-runtime-ecs": "03 Runtime",
        "04-domain-crates": "04 Domain",
        "05-persistence-checkpoint": "05 Persistence",
        "06-data-flow": "06 DataFlow",
        "README": "README 索引",
    }
    for i, f in enumerate(files):
        nodes.append(node(f, f"\n{file_titles[f]}", x=(i % 4) * 320, y=(i // 4) * 150, w=280, h=100, color="4"))
    # 152 节点笔记
    nodes.append(node("nodes", "152 节点笔记\ndocs/wiki/docswiki/nodes/", x=400, y=200, w=300, h=100, color="7"))
    # 8 canvas
    nodes.append(node("canvases", "8 .canvas 文件\ndocs/wiki/docswiki/canvas/", x=400, y=350, w=300, h=100, color="7"))
    # 边
    for s in sources:
        edges.append(edge(f"e-{s}-00", s, "00-design-topology", "→ 拓扑"))
    for f in files:
        edges.append(edge(f"e-{f}-nodes", f, "nodes", "refers to"))
        edges.append(edge(f"e-{f}-canvases", f, "canvases", "paired"))
    edges.append(edge("e-canvases-nodes", "canvases", "nodes", "drag"))
    return {"nodes": nodes, "edges": edges}


CANVAS_FUNCS = {
    "00-design-topology.canvas": canvas_00_overview,
    "01-ui-agent-view.canvas": canvas_01_agent_view,
    "02-orchestration-langgraph.canvas": canvas_02_langgraph,
    "03-runtime-ecs.canvas": canvas_03_runtime,
    "04-domain-crates.canvas": canvas_04_domain,
    "05-persistence-checkpoint.canvas": canvas_05_persistence,
    "06-data-flow.canvas": canvas_06_dataflow,
    "99-docswiki-vs-pgwiki-diff.canvas": canvas_99_diff,
    "99-pg-broker-audit.canvas": canvas_99_broker,
    "README.canvas": canvas_readme,
}


def main():
    CANVAS_DIR.mkdir(parents=True, exist_ok=True)
    for filename, fn in CANVAS_FUNCS.items():
        data = fn()
        # Obsidian Canvas 1.0 格式
        canvas = {
            "nodes": data["nodes"],
            "edges": data["edges"],
        }
        path = CANVAS_DIR / filename
        path.write_text(json.dumps(canvas, ensure_ascii=False, indent=2), encoding="utf-8")
        print(f"[ok] {filename}: {len(data['nodes'])} nodes, {len(data['edges'])} edges")


if __name__ == "__main__":
    main()
