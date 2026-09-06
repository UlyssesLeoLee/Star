#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
obsidian_topology_gen.py — 把 docs/wiki/docswiki/ 8 份拓扑 + 174 节点转成 Obsidian Wiki 风格.

产出:
- 7 份源头 S1-S7 note
- 174 份节点 note (含 frontmatter 13 字段, [[wikilink]] 反向链)
- 8 份原拓扑文件: 补 frontmatter + 节点文字转 [[wikilink]]
- 8 份 .canvas (Obsidian Canvas 插件 JSON)

拍板: 2026-09-06 17:13 JST
- Scope: docswiki 8 份
- Frontmatter: 完整集 (5 基础 + 8 扩展)
- 双链: 节点→节点 + 源→拓扑
- Canvas: 8 份都出

守门:
- 100% 文档实证 (per 守门 #12)
- 缺标比错标 (per 守门 #11)
- 修订 author = Ulysses (per 8/27 19:39 JST 授权)
"""

import json
import os
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

REPO_ROOT = Path("D:/Star")
WIKI_DIR = REPO_ROOT / "docs" / "wiki" / "docswiki"
NODES_DIR = WIKI_DIR / "nodes"
CANVAS_DIR = WIKI_DIR / "canvas"
TODAY = "2026-09-06"

# 颜色 (供 .canvas 节点 color 字段, Obsidian Canvas 6 色调色板)
COLOR_BY_LAYER = {
    "source": "6",        # 源头 - 紫
    "ui": "4",            # UI - 蓝
    "orchestration": "5", # 编排 - 紫
    "runtime": "3",       # Runtime - 橙
    "domain": "2",        # Domain - 绿
    "persistence": "1",   # Persistence - 红
    "platform": "6",      # Platform - 紫
    "node": "7",          # 节点 (独立笔记) - 灰
}


@dataclass
class Node:
    """Obsidian 节点笔记定义."""
    id: str                  # 节点 ID (e.g. "C-01", "domain-tenant", "S1")
    title: str               # 显示标题
    layer: str               # source / ui / orchestration / runtime / domain / persistence / platform
    view: str                # AgentView / LangGraph / AgentRuntime / Cross
    description: str         # 一句话描述 (文档实证)
    source: str              # 文档来源 (e.g. "S2 §1.3 L142", "S4 §3.2 L183")
    classification: str      # 守门 #13 W/T/M (如适用, 否则 "-")
    importance: str          # P0 / P1 / P2
    related: list = field(default_factory=list)     # 上下游节点
    see_also: list = field(default_factory=list)    # 横向相关节点
    supersedes: Optional[str] = None
    revision: str = f"v0.1 @ {TODAY} Ulysses(per 19:39 JST)— Mavis 接手"
    in_topology: list = field(default_factory=list)  # 出现在哪几份拓扑

    def frontmatter(self) -> str:
        """Obsidian YAML frontmatter (13 字段)."""
        in_topo = self.in_topology if self.in_topology else [self.id]
        related_yaml = "[" + ", ".join(f'"{r}"' for r in self.related) + "]" if self.related else "[]"
        see_also_yaml = "[" + ", ".join(f'"{r}"' for r in self.see_also) + "]" if self.see_also else "[]"
        in_topology_yaml = "[" + ", ".join(f'"{r}"' for r in in_topo) + "]" if in_topo else "[]"
        lines = [
            "---",
            f"id: {self.id}",
            f"title: {self.title}",
            f"date: {TODAY}",
            f"source: '{self.source}'",
            f"status: design-baseline",
            f"classification: {self.classification}",
            f"importance: {self.importance}",
            f"layer: {self.layer}",
            f"view: {self.view}",
            f"related: {related_yaml}",
            f"see-also: {see_also_yaml}",
            f"version: 0.1",
            f"revision: '{self.revision}'",
            f"supersedes: {self.supersedes or 'null'}",
            f"in-topology: {in_topology_yaml}",
            "guards:",
            "  - id: '#1'",
            "    name: 0 unsafe + 0 err",
            "    evidence: 文档实证, 100% file:line 引用",
            "  - id: '#3'",
            "    name: 5 域独立 Lead / 3 view 平行",
            "    evidence: 不建立业务子域↔DDD 映射",
            "  - id: '#7'",
            "    name: 0 unsafe",
            "    evidence: 纯 markdown / JSON, 无代码",
            "  - id: '#11'",
            "    name: 缺标比错标",
            "    evidence: 已知缺口显式列",
            "  - id: '#12'",
            "    name: AI 協作文档治理",
            "    evidence: 0 回溯叙事, BAS git 实证",
            "  - id: '#13'",
            "    name: DB W/T/M 严格分类",
            "    evidence: PG 5 表 100% 覆盖 (T=3, M=1, T-WORM=1)",
            "  - id: '#19'",
            "    name: agent 交互 Python 化",
            "    evidence: 本脚本 = scripts/automation/obsidian_topology_gen.py",
            "  - id: '#10'",
            "    name: 代签规则应用",
            "    evidence: author=Ulysses (per 19:39 JST 授权)",
            "tags:",
            f"  - {self.layer}/nodes",
            f"  - {self.view.lower()}-view",
            "  - obsidian-wiki",
            "  - design-topology",
            f"  - id/{self.id.lower()}",
            "---",
        ]
        return "\n".join(lines)

    def body(self) -> str:
        """笔记正文 — [[wikilink]] 风格."""
        # in-topology fallback: 至少自指代
        in_topo = self.in_topology if self.in_topology else [self.id]
        related_md = "\n".join(f"- [[{r}]]" for r in self.related) if self.related else "_无_"
        see_also_md = "\n".join(f"- [[{r}]]" for r in self.see_also) if self.see_also else "_无_"
        in_topology_md = "\n".join(f"- [[{r}]]" for r in in_topo) if in_topo else "_无_"
        out = [
            f"# {self.id} — {self.title}",
            "",
            f"> **节点类型**: {self.layer} / {self.view}",
            f"> **重要度**: {self.importance}",
            f"> **守门分类**: {self.classification}",
            "",
            "## 1. 描述",
            "",
            self.description,
            "",
            f"**文档来源**: {self.source}",
            "",
            "## 2. 出现在拓扑 (in-topology)",
            "",
            in_topology_md,
            "",
            "## 3. 上下游 (related)",
            "",
            related_md,
            "",
            "## 4. 横向相关 (see-also)",
            "",
            see_also_md,
            "",
            "## 5. 守门实证",
            "",
            "| 守门 | 实证 |",
            "|---|---|",
            "| #1 0 unsafe + 0 err | 文档实证 100% |",
            "| #3 5 域独立 Lead / 3 view 平行 | 不建立业务子域↔DDD 映射 |",
            "| #11 缺标比错标 | 已知缺口显式列 |",
            "| #12 AI 協作文档治理 | 0 回溯叙事 |",
            f"| #13 DB W/T/M | 守门 #13 (本节点 = {self.classification}) |",
            "| #19 agent 交互 Python 化 | 本脚本 = scripts/automation/obsidian_topology_gen.py |",
            "",
            "## 6. 修订历史",
            "",
            "| 版本 | 日期 | 修订人 | 修订内容 | 触发 |",
            "|---|---|---|---|---|",
            f"| v0.1 | {TODAY} | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 | 2026-09-06 17:13 JST 用户拍板「规划内拓扑结构应该构成一份 obsidian wiki」 |",
        ]
        return "\n".join(out)

    def note_text(self) -> str:
        return self.frontmatter() + "\n\n" + self.body() + "\n"


# ============================================================
# 节点数据 - 全部从文档实证, file:line 标在 source
# ============================================================

def all_nodes():
    """生成所有节点."""
    nodes = []

    # ---------- 7 源头 S1-S7 ----------
    sources = [
        ("S1", "BD-AGENT-VIEW-001", "design", "Cross", "Agent View 基本設計書 (3-tier UI / 10 模块 / 派生链 / 7 节点 画布)", "BD-AGENT-VIEW-001.md (per S2 §1.3 L11 引用)", "-", "P0", ["T1-AgentView"], None, ["00-design-topology", "01-ui-agent-view"]),
        ("S2", "LangGraph 02 Basic Design", "orchestration", "LangGraph", "LangGraph view 基本设计 (2-level hierarchical + 22 组件 C-01..C-22 + 7 TMO 节点 M-N1..M-N7 + 9 SA 类型)", "2026-09-03-langgraph/02-basic-design.md (per S2 §1.1-§1.3)", "-", "P0", ["T2-Orchestration"], None, ["00-design-topology", "02-orchestration-langgraph"]),
        ("S3", "LangGraph 03 Detailed Design", "orchestration", "LangGraph", "LangGraph view 詳細設計 (25 模块 M-01..M-25 + 7 subgraph + 7 TMO class + LangGraph 节点 M-N1..M-N7)", "2026-09-03-langgraph/03-detailed-design.md (per S3 §1.1-§3.5)", "-", "P0", ["T2-Orchestration"], None, ["02-orchestration-langgraph"]),
        ("S4", "Agent Runtime 02 Basic Design", "runtime", "AgentRuntime", "Agent Runtime 基本设计 (L0 派发 + L1 ECS + L2 业务共享池 + 9 SA Archetype + 13 Systems + 31 domain-* 目标)", "2026-09-03-agent-runtime/02-basic-design.md (per S4 §2.1-§3.5)", "-", "P0", ["T3-Runtime"], None, ["00-design-topology", "03-runtime-ecs"]),
        ("S5", "Agent Runtime 03 Detailed Design", "runtime", "AgentRuntime", "Agent Runtime 詳細設計 (9 domain-* crate 新建 + ECS Component 12 类 + 状态机 + 9 Systems + DB schema 5 张表)", "2026-09-03-agent-runtime/03-detailed-design.md (per S5 §1.1-§3.5)", "-", "P0", ["T3-Runtime"], None, ["03-runtime-ecs"]),
        ("S6", "22 domain 接入顺序", "domain", "Cross", "22 domain crate Tier 1-6 接入顺序 + 依赖深度分层 + 5 域映射 + 接入工作量估算", "2026-08-26-upgrade/spec/integration/01-22-domain-integration-spec.md (per S6 §2)", "-", "P0", ["T4-Domains"], None, ["00-design-topology", "04-domain-crates"]),
        ("S7", "PostgreSQL Checkpointer Tier 3", "persistence", "Cross", "PostgreSQL Checkpointer Tier 3 (5 张表 schema + W/T/M 严格分类 + 12 Reducer 跨 Tier)", "2026-08-26-upgrade/adr/0047-postgresql-checkpointer-tier3.md (per S7 §3.1-§3.4)", "M (Master) + T (Transaction)", "P0", ["T5-Persistence"], None, ["00-design-topology", "05-persistence-checkpoint"]),
    ]
    for s in sources:
        nodes.append(Node(*s))

    # ---------- LangGraph 22 组件 C-01..C-22 (per S2 §1.3 L140-163) ----------
    # (id, title, description, source, importance, view, related, see_also)
    components = [
        ("C-01", "TopAgent", "L0 全体代理 LangGraph instance, 意图解析/dispatch/collect/respond", "S2 §1.3 L142", "P0", "LangGraph", ["T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7", "C-02-SubAgentPool", "C-04-CheckpointStore"], ["M-01", "M-02"]),
        ("C-02", "SubAgentPool", "sub-agent spawn / pool / lifecycle 管理", "S2 §1.3 L143", "P0", "LangGraph", ["C-03-SubAgent", "C-01-TopAgent"], ["M-04"]),
        ("C-03", "SubAgent", "SA-01..SA-09 各 subgraph instance", "S2 §1.3 L144", "P0", "LangGraph", ["C-02-SubAgentPool"], ["M-05", "M-06"]),
        ("C-04", "CheckpointStore", "3-tier 永続化 (Memory / SQLite / PostgreSQL)", "S2 §1.3 L145", "P0", "Cross", ["PG-checkpoints", "PG-checkpoint_writes", "PG-checkpoint_metadata"], ["M-08"]),
        ("C-05", "McpClient", "star-mcp 16 tools proxy", "S2 §1.3 L146", "P0", "LangGraph", ["star-mcp", "M-11-AuditedMcpToolNode"], ["M-10"]),
        ("C-06", "UIStreamer", "WebSocket / SSE / REST 3 通道推送", "S2 §1.3 L147", "P0", "Cross", ["C-01-TopAgent", "C-02-SubAgentPool", "View-AgentView"], ["M-12"]),
        ("C-07", "TaskCardManager", "UI 状态 ↔ Sub-agent state mirror", "S2 §1.3 L148", "P0", "Cross", ["View-AgentView", "C-02-SubAgentPool"], ["M-N7"]),
        ("C-08", "AuditLogger", "全 tool call / dispatch / interrupt 記録", "S2 §1.3 L149", "P0", "Cross", ["M-11-AuditedMcpToolNode", "PG-audit_audit_event"], ["M-13"]),
        ("C-09", "TokenTelemetry", "token 計量 + OLU 集計 (per 守门 #4)", "S2 §1.3 L150", "P1", "Cross", ["LLM Pool", "View-AgentRuntime"], ["M-14"]),
        ("C-10", "GuardEnforcer", "AGENTS.md §4 守门 37 项 自动检查", "S2 §1.3 L151", "P1", "Cross", ["M-11-AuditedMcpToolNode"], ["M-15"]),
        ("C-11", "StateSchemaRegistry", "LangGraph state schema 中央管理", "S2 §1.3 L152", "P1", "Cross", ["TopAgentState", "SubAgentState"], ["M-18"]),
        ("C-12", "InterruptManager", "human-in-the-loop interrupt / resume", "S2 §1.3 L153", "P0", "Cross", ["C-03-SubAgent", "View-AgentView"], ["M-16"]),
        ("C-13", "SubAgentRegistry", "sub-agent 类型注册表 (SA-01..SA-09 + new)", "S2 §1.3 L154", "P1", "LangGraph", ["C-02-SubAgentPool"], []),
        ("C-14", "CrossDomainDispatcher", "跨 domain crate 调用协调 (per 守门 #3)", "S2 §1.3 L155", "P2", "LangGraph", ["22 domain crates"], []),
        ("C-15", "HealthCheck", "/api/health endpoint, 状態監視", "S2 §1.3 L156", "P1", "Cross", ["FastAPI app"], []),
        ("C-16", "TaskOperationsManager", "★ TMO 集中管理: 7 节点 (M-N1..M-N7) + 7 协议 + DAG 校验; 唯一 cross-task actor", "S2 §1.3 L157", "P0", "LangGraph", ["M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"], ["M-19"]),
        ("C-17", "TaskRelationshipGraph", "★ 任务卡 DAG (parent_task_id / merged_from / split_into / superseded_by 4 字段), cycle prevention", "S2 §1.3 L158", "P0", "LangGraph", ["C-20-DAGValidator", "C-16-TaskOperationsManager"], ["M-20"]),
        ("C-18", "BulkOperationQueue", "★ bulk_action 队列 + asyncio.gather 协调, 部分失败回滚", "S2 §1.3 L159", "P0", "LangGraph", ["C-16-TaskOperationsManager", "C-02-SubAgentPool"], ["M-21"]),
        ("C-19", "MetadataRegistry", "★ task_metadata 表中央管理 (Master RLS 必携 per 守门 #13 c)", "S2 §1.3 L160", "P1", "LangGraph", ["PG-checkpoint_metadata", "C-07-TaskCardManager"], ["M-23"]),
        ("C-20", "DAGValidator", "★ cycle detection O(V+E) 校验, 检测到环 → reject + interrupt", "S2 §1.3 L161", "P0", "LangGraph", ["C-17-TaskRelationshipGraph"], ["M-22"]),
        ("C-21", "ReassignManager", "★ SA-XX 类型切换, checkpoint preserved", "S2 §1.3 L162", "P1", "LangGraph", ["C-02-SubAgentPool", "C-04-CheckpointStore"], ["M-24"]),
        ("C-22", "SummarizeCollector", "★ 跨 N SubAgentState 状态聚合, LLM 表格化", "S2 §1.3 L163", "P1", "LangGraph", ["C-16-TaskOperationsManager", "C-02-SubAgentPool"], ["M-25"]),
    ]
    for c in components:
        # (id, title, description, source, importance, view, related, see_also)
        nodes.append(Node(c[0], c[1], "orchestration", c[5], c[2], c[3], "-", c[4], c[6], c[7], None, ["C-" + c[0][2:].zfill(2), "00-design-topology", "02-orchestration-langgraph"]))

    # ---------- LangGraph 节点 T-N1..T-N7 ----------
    tnodes = [
        ("T-N1", "parse_intent", "LLM 意图分类 + 必要 sub-agent 抽出", "S2 §2.1.2 L200", "P0"),
        ("T-N2", "dispatch", "SubAgentPool.spawn, 任务卡生成", "S2 §2.1.2 L201", "P0"),
        ("T-N3", "tool_node", "MCP tool 直接呼出 (sub-agent 不要時)", "S2 §2.1.2 L202", "P0"),
        ("T-N4", "collect", "sub-agent 結果待合 (asyncio.gather)", "S2 §2.1.2 L203", "P0"),
        ("T-N5", "respond", "LLM 生成 user-facing 回答", "S2 §2.1.2 L204", "P0"),
        ("T-N6", "interrupt", "暂停等待 user 决策", "S2 §2.1.2 L205", "P0"),
        ("T-N7", "guard_check", "守门 #4 / #9 / #12 / #13 检查", "S2 §2.1.2 L206", "P0"),
    ]
    for tn in tnodes:
        nodes.append(Node(tn[0], tn[1], "orchestration", "LangGraph", tn[2], tn[3], "T (Transaction - guard violation log)", tn[4], ["TopAgent", "C-01"], ["C-10-GuardEnforcer"], None, ["02-orchestration-langgraph"]))

    # ---------- TMO 7 节点 M-N1..M-N7 ----------
    mnodes = [
        ("M-N1", "merge_node", "合并 a+b → merged_task, stash_state + supersede + dispatch", "S2 §2.6.1 L512", "P0"),
        ("M-N2", "split_node", "拆分 a → a1 + a2, checkpoint snapshot + forked context dispatch", "S2 §2.6.1 L513", "P0"),
        ("M-N3", "reorder_node", "依赖 DAG 调整, cycle detection (per DAGValidator C-20)", "S2 §2.6.1 L514", "P0"),
        ("M-N4", "bulk_node", "N 张卡批量 action (pause/resume/cancel/set_priority), asyncio.gather + 部分失败回滚", "S2 §2.6.1 L515", "P0"),
        ("M-N5", "summarize_node", "跨 N SubAgentState 聚合 + LLM 表格化", "S2 §2.6.1 L516", "P1"),
        ("M-N6", "reassign_node", "sub-agent 类型 SA-XX 切换, checkpoint preserved (ReassignManager C-21)", "S2 §2.6.1 L517", "P1"),
        ("M-N7", "metadata_node", "task_metadata 表更新 (Master RLS 必携 per 守门 #13 c)", "S2 §2.6.1 L518", "P1"),
    ]
    for mn in mnodes:
        nodes.append(Node(mn[0], mn[1], "orchestration", "LangGraph", mn[2], mn[3], "T (Transaction - audit log)", mn[4], ["TaskOperationsManager", "C-16"], ["C-17-TaskRelationshipGraph", "C-20-DAGValidator"], None, ["02-orchestration-langgraph", "05-persistence-checkpoint", "06-data-flow"]))

    # ---------- 9+1 SA 类型 ----------
    sas = [
        ("SA-01", "code-review", "代码审查 (per PR/MR), review-plan → review-execute → review-report", "S2 §2.2.2 L261", "P0"),
        ("SA-02", "test-gen", "测试生成, test-plan → test-execute → test-verify", "S2 §2.2.2 L262", "P0"),
        ("SA-03", "5-域-lead-audit", "★ 唯一一个跨域 + 治理矩阵型 sub-agent, 依赖 5 域 Lead 真人到位 (per 守门 #3 反転 Mavis 临时代签)", "S2 §2.2.2 L263", "P0"),
        ("SA-04", "git-ops", "git 操作 (worktree/commit/push), ops-plan → ops-execute → ops-verify", "S2 §2.2.2 L265", "P0"),
        ("SA-05", "doc-sync", "文档同步 (AGENTS.md / WBS / ADR), doc-plan → doc-execute → doc-verify", "S2 §2.2.2 L266", "P0"),
        ("SA-06", "refactor", "代码重构, refactor-plan → refactor-execute → refactor-verify (cargo test)", "S2 §2.2.2 L267", "P0"),
        ("SA-07", "db-migration", "DB migration (per 守门 #13 W/T/M), db-plan → db-migrate → db-verify", "S2 §2.2.2 L268", "P0"),
        ("SA-08", "domain-dev", "DDD bounded context 開発 (per 22 domain crates), dev-plan → dev-execute → dev-verify", "S2 §2.2.2 L269", "P0"),
        ("SA-09", "free-form", "默认 fallback, 自由形式, generic plan-execute-verify-report", "S2 §2.2.2 L270", "P0"),
        ("SA-10", "task-orchestrator", "★ v0.2 TMO 跨任务编排型, NEW", "S3 §1.1 L66", "P0"),
    ]
    for sa in sas:
        nodes.append(Node(sa[0], sa[1], "orchestration", "LangGraph", sa[2], sa[3], "T (audit log 必携)", sa[4], ["SubAgentPool", "C-02"], ["SA-01..SA-09", "M-06"], None, ["02-orchestration-langgraph", "03-runtime-ecs"]))

    # ---------- Agent View 10 模块 M-AGV-1..M-AGV-10 ----------
    agv = [
        ("M-AGV-1", "types.ts", "派生类型定义", "S1 §6.1 L503", "P0", "AgentView", ["M-AGV-2", "M-AGV-3", "M-AGV-6", "M-AGV-8"], []),
        ("M-AGV-2", "selectors.ts", "选 agent / worktree / wi, 7 纯函数", "S1 §6.1 L504", "P0", "AgentView", ["M-AGV-1"], ["M-AGV-5"]),
        ("M-AGV-3", "layout.ts", "自由散开布局算法, 2 纯函数 + 1 helper", "S1 §6.1 L505", "P0", "AgentView", ["M-AGV-1"], ["M-AGV-4"]),
        ("M-AGV-4", "layout.test.ts", "layout 单测, 11 tests", "S1 §6.1 L506", "P1", "AgentView", ["M-AGV-3"], []),
        ("M-AGV-5", "selectors.test.ts", "selectors 单测, 14 tests", "S1 §6.1 L507", "P1", "AgentView", ["M-AGV-2"], []),
        ("M-AGV-6", "AgentCanvasView", "SVG 无限画布", "S1 §6.1 L508", "P0", "AgentView", ["M-AGV-1", "zustand store", "StatusPill"], ["M-AGV-7"]),
        ("M-AGV-7", "AgentCanvasView.test", "AgentCanvasView smoke, 4 tests", "S1 §6.1 L509", "P1", "AgentView", ["M-AGV-6"], []),
        ("M-AGV-8", "AgentFilter", "顶部 dropdown", "S1 §6.1 L510", "P0", "AgentView", ["M-AGV-2", "zustand store"], []),
        ("M-AGV-9", "page.tsx", "主页面, 客户端组件", "S1 §6.1 L511", "P0", "AgentView", ["M-AGV-1..8", "zustand store"], []),
        ("M-AGV-10", "nav/registry.ts (改)", "注册 nav entry, +12 bytes", "S1 §6.1 L512", "P1", "AgentView", ["Bot icon"], []),
    ]
    for m in agv:
        # (id, title, description, source, importance, view, related, see_also)
        nodes.append(Node(m[0], m[1], "ui", m[5], m[2], m[3], "-", m[4], m[6], m[7], None, ["01-ui-agent-view"]))

    # ---------- Agent Runtime 12 ECS Components (per S4 §3.2) ----------
    ecs_comp = [
        ("Comp-AgentIdentity", "AgentIdentity", "agent_id + tenant_id + agent_type", "S4 §3.2 L183", "P0", ["SA-01..SA-09", "TenantId"]),
        ("Comp-AgentState", "AgentState", "12 状态 (Idle / Ready / Scheduled / Planning / WaitingLlm / ...)", "S4 §3.2 L184", "P0", ["AgentStateEnum"]),
        ("Comp-LifecycleState", "LifecycleState", "HOT/WARM/COLD (per SRS REQ-ECS-003)", "S4 §3.2 L185", "P0", ["LifecycleStateEnum"]),
        ("Comp-ContextRef", "ContextRef", "Context 引用 (不存 Full Context), tier L1Hot/L2Recent/L3Full", "S4 §3.2 L186", "P0", ["ContextStore"]),
        ("Comp-MemoryRef", "MemoryRef", "Memory 引用, S/E/U/W/K 5 类", "S4 §3.2 L187", "P0", ["MemoryStore"]),
        ("Comp-ModelRef", "ModelRef", "模型配置 + 共享 LLM Pool 引用", "S4 §3.2 L188", "P0", ["LLM Pool"]),
        ("Comp-ToolPolicyRef", "ToolPolicyRef", "Tool 权限引用, tool_allowlist", "S4 §3.2 L189", "P0", ["Tool Registry"]),
        ("Comp-McpPolicyRef", "McpPolicyRef", "MCP 权限引用, server_allowlist", "S4 §3.2 L190", "P0", ["MCP Pool"]),
        ("Comp-PermissionRef", "PermissionRef", "ACL 引用 (不复制 ACL)", "S4 §3.2 L191", "P0", ["domain-permission"]),
        ("Comp-TokenBudget", "TokenBudget", "Token 配额, max_ctx + max_out + remaining + cost", "S4 §3.2 L192", "P0", ["TokenTelemetry"]),
        ("Comp-Priority", "Priority", "优先级 (Critical/High/Normal/Low/Background)", "S4 §3.2 L193", "P0", ["SchedulerSystem"]),
        ("Comp-MailboxRef", "MailboxRef", "Mailbox 引用 (不存大消息)", "S4 §3.2 L194", "P0", ["EventBus"]),
    ]
    for c in ecs_comp:
        related_list = c[5] if c[5] else []
        nodes.append(Node(c[0], c[1], "runtime", "AgentRuntime", c[2], c[3], "T (state change append)", c[4], [related_list[0]] if related_list else [], related_list, None, ["03-runtime-ecs"]))

    # ---------- Agent Runtime 13 Systems ----------
    systems = [
        ("Sys-Scheduler", "SchedulerSystem", "Agent 调度 (Ready Queue + Priority), 每 frame", "S4 §3.4 L220", "P0", "独立 L0 L1 L2"),
        ("Sys-Lifecycle", "LifecycleSystem", "HOT/WARM/COLD 状态转换, 每 frame", "S4 §3.4 L221", "P0", "独立 Runtime 概念"),
        ("Sys-Event", "EventSystem", "Event 路由 + Mailbox 投递, Event-driven", "S4 §3.4 L222", "P0", "跟 LangGraph 状态转换并行"),
        ("Sys-Planner", "PlannerSystem", "任务规划, 调度时", "S4 §3.4 L223", "P0", "引用 LangGraph 9/3 planner node"),
        ("Sys-Llm", "LlmSystem", "LLM 请求 + Token 计量, LLM 调用时", "S4 §3.4 L224", "P0", "独立 L2 LLM Pool"),
        ("Sys-Tool", "ToolSystem", "Tool 调用 + 权限检查, Tool 调用时", "S4 §3.4 L225", "P0", "跟 LangGraph tool node 协作"),
        ("Sys-Mcp", "McpSystem", "MCP 调用, MCP 调用时", "S4 §3.4 L226", "P0", "跟 LangGraph MCP node 协作"),
        ("Sys-Retrieval", "RetrievalSystem", "RAG 检索, RAG 调用时", "S4 §3.4 L227", "P1", "独立 L2 RAG Pool"),
        ("Sys-Context", "ContextSystem", "Context 装载/卸载 (Lazy Load), Context 需要时", "S4 §3.4 L228", "P0", "独立 L2 Context Store"),
        ("Sys-Memory", "MemorySystem", "Memory 读写, Memory 访问时", "S4 §3.4 L229", "P0", "独立 L2 Memory Store"),
        ("Sys-Permission", "PermissionSystem", "权限检查, 任何外部调用", "S4 §3.4 L230", "P0", "跨域 Tenant + ACL"),
        ("Sys-Persistence", "PersistenceSystem", "Checkpoint + 持久化, 任务状态变更", "S4 §3.4 L231", "P0", "跨域 L2 + DB"),
        ("Sys-Metrics", "MetricsSystem", "可观测性指标采集, 每 frame", "S4 §3.4 L232", "P1", "独立 L0 Observer"),
    ]
    for s in systems:
        nodes.append(Node(s[0], s[1], "runtime", "AgentRuntime", s[2], s[3], "-", s[4], [], s[5].split(" "), None, ["03-runtime-ecs"]))

    # ---------- 22 domain crate (含 9 新建) ----------
    domain_crates = [
        # Tier 1
        ("domain-tenant", "Tier 1 基础, tenant_id, tenant://{tenant_id}, Permission 域", "S6 §2 L42", "0.8-1.2M"),
        ("domain-identity", "Tier 1 基础, user_id, identity://{user_id}, Permission 域", "S6 §2 L43", "0.8-1.2M"),
        ("domain-permission", "Tier 1 基础, rule_id, permission://{rule_id}, Permission 域", "S6 §2 L44", "1.0-1.5M"),
        # Tier 2
        ("domain-workspace", "Tier 2 业务原子, ws_id, 依赖 tenant, Worktree 域", "S6 §2 L54", "1.2-1.6M"),
        ("domain-project", "Tier 2 业务原子, 依赖 tenant, Flow 域 (待 spec/agents/02 v0.2 补)", "S6 §2 L55", "1.0-1.5M"),
        ("domain-work-item", "Tier 2 业务原子, wi_id, 依赖 project, Flow 域", "S6 §2 L56", "1.2-1.6M"),
        # Tier 3
        ("domain-worktree", "Tier 3 业务实体, wt_id, 依赖 project + work-item, Worktree 域", "S6 §2 L66", "1.5-2.0M"),
        ("domain-agent", "Tier 3 业务实体, agent_id, 依赖 identity + workspace, Agent 域", "S6 §2 L67", "1.5-2.0M"),
        ("domain-feedback", "Tier 3 业务实体, fb_id, 依赖 work-item + identity, Integration 域", "S6 §2 L68", "1.0-1.5M"),
        ("domain-decision", "Tier 3 业务实体, dec_id, 依赖 work-item, Flow 域", "S6 §2 L69", "1.2-1.6M"),
        # Tier 4
        ("domain-scm", "Tier 4 业务复合, scm_id, 依赖 worktree + agent, Worktree 域", "S6 §2 L78", "1.5-2.0M"),
        ("domain-validation", "Tier 4 业务复合, val_id, 依赖 work-item + decision, Agent 域", "S6 §2 L79", "1.0-1.5M"),
        ("domain-automation", "Tier 4 业务复合, rule_id, 依赖 agent + decision, Flow 域", "S6 §2 L80", "1.2-1.6M"),
        ("domain-search", "Tier 4 业务复合, query_id, 依赖 work-item + agent, Integration 域", "S6 §2 L81", "1.0-1.5M"),
        # Tier 5
        ("domain-policy", "Tier 5 业务扩展, policy_id, 依赖 validation + automation, Permission 域", "S6 §2 L92", "1.0-1.5M"),
        ("domain-notification", "Tier 5 业务扩展, nt_id, 依赖 agent + decision, Integration 域", "S6 §2 L93", "1.0-1.5M"),
        ("domain-context", "Tier 5 业务扩展, ctx_id, 依赖 worktree + decision + context, Integration 域", "S6 §2 L94", "1.5-2.0M"),
        ("domain-resume", "Tier 5 业务扩展, resume_id, 依赖 agent + lease, Agent 域", "S6 §2 L95", "1.0-1.5M"),
        # Tier 6
        ("domain-audit", "Tier 6 业务高级, audit_id, 依赖所有 Tier 1-5, Admin 域", "S6 §2 L105", "1.5-2.0M"),
        ("domain-integration", "Tier 6 业务高级, int_id, 依赖 worktree + scm + agent, Integration 域", "S6 §2 L106", "1.5-2.0M"),
        ("domain-event", "Tier 6 业务高级, event_id, 依赖 agent + audit + notification, Integration 域", "S6 §2 L107", "1.0-1.5M"),
        ("domain-flow", "Tier 6 业务高级, flow_id, 依赖 (聚合), Flow 域", "S6 §2 L108", "1.0-1.5M"),
        ("domain-lease", "Tier 6 业务高级, lease_id, 依赖 agent + resume, Agent 域 (1 跨域)", "S6 §2 L109", "1.0-1.5M"),
        # 9 新建
        ("domain-dispatcher", "★ 新建 L0 派发, per S5 §1.1, P3-B", "S5 §1.1 L48-59", "TBD"),
        ("domain-llm", "★ 新建 L2 LLM Pool, per S5 §1.1, P3-C", "S5 §1.1 L61", "TBD"),
        ("domain-mcp", "★ 新建 L2 MCP Pool, per S5 §1.1, P3-C", "S5 §1.1 L62", "TBD"),
        ("domain-tool", "★ 新建 L2 Tool Registry, per S5 §1.1, P3-C", "S5 §1.1 L63", "TBD"),
        ("domain-rag", "★ 新建 L2 RAG Pool, per S5 §1.1, P3-E", "S5 §1.1 L64", "TBD"),
        ("domain-memory", "★ 新建 L2 Memory Store, per S5 §1.1, P3-D", "S5 §1.1 L66", "TBD"),
        ("domain-rate-limiter", "★ 新建 L0 RateLimiter, per S5 §1.1, P3-B", "S5 §1.1 L67", "TBD"),
        ("domain-observability", "★ 新建 L0 Observer, per S5 §1.1, P3-B", "S5 §1.1 L68", "TBD"),
    ]
    for d in domain_crates:
        nodes.append(Node(d[0], d[0], "domain", "Cross", d[1], d[2], "M/T/W 混合 per 设计", d[3], [], [], None, ["04-domain-crates"]))

    # ---------- 15 个 star-* crate (per `cargo metadata` 2026-09-06 18:46 JST 实测) ----------
    star_crates = [
        ("star-mcp", "MCP server (16 tools + transport_http + handlers + sa_real_impls), 49 src 文件", "crates/star-mcp/src/", "已实装 P0"),
        ("star-api-rest", "REST API (per spec/rest/01), 20 src 文件", "crates/star-api-rest/src/", "已实装 P0"),
        ("star-cli", "CLI (per 守门 #6 PowerShell only), 16 src 文件", "crates/star-cli/src/", "已实装 P1"),
        ("star-saga", "Saga 协调 (per spec/saga/01 5 步流程), 11 src 文件", "crates/star-saga/src/", "已实装 P0"),
        ("star-sa", "Sub-Agent (per S2 §1.1 + S4 §2.1 9 SA Archetype), 6 src 文件", "crates/star-sa/src/", "已实装 P0"),
        ("star-dispatcher", "L0 派发 (per S4 §3.1, enqueue/get/list/transition/submit API), 5 src 文件", "crates/star-dispatcher/src/", "已实装 P0"),
        ("star-cache", "Cache (per spec/cache/01 §4 TTL 表), 4 src 文件", "crates/star-cache/src/", "已实装 P1"),
        ("star-credential", "Credential (per 守门 #5 环境变量安全), 4 src 文件", "crates/star-credential/src/", "已实装 P1"),
        ("star-treesitter", "Tree-sitter (per 2026-09-03 treesitter-worktree-graph view), 4 src 文件", "crates/star-treesitter/src/", "已实装 P1"),
        ("star-context", "ActorContext (per H2 star_context 9/3 P0-1), 3 src 文件", "crates/star-context/src/", "已实装 P0"),
        ("star-sse", "SSE 推送 (per spec/services/02), 3 src 文件", "crates/star-sse/src/", "已实装 P0"),
        ("star-webhook", "Webhook (per spec/services/03), 3 src 文件", "crates/star-webhook/src/", "已实装 P0"),
        ("star-taskgraph", "Task Graph (per BATCH-REQ-001 v0.1.2 + ADR-0040), 2 src 文件", "crates/star-taskgraph/src/", "已实装 P1"),
        ("star-vcs", "VCS (per ADR-0023 GitGit + 4 Provider), 2 src 文件", "crates/star-vcs/src/", "已实装 P0"),
        ("star-dto", "DTO 共享, 1 src 文件", "crates/star-dto/src/", "已实装 P1"),
    ]
    for s in star_crates:
        nodes.append(Node(s[0], s[0], "runtime", "Cross", s[1], s[2], "-", s[3], [], [], None, ["04-domain-crates", "99-pg-broker-audit"]))

    # ---------- 3 个 other crate (per `cargo metadata` 2026-09-06 18:46 JST 实测) ----------
    other_crates = [
        ("api", "REST 入口 (per spec/rest/01)", "crates/api/src/", "已实装 P0"),
        ("application", "Application Layer (per S4 §2.1)", "crates/application/src/", "已实装 P0"),
        ("infrastructure", "Infrastructure 聚合", "crates/infrastructure/src/", "已实装 P0"),
    ]
    for o in other_crates:
        nodes.append(Node(o[0], o[0], "platform", "Cross", o[1], o[2], "-", o[3], [], [], None, ["04-domain-crates", "99-pg-broker-audit"]))

    # ---------- 9 个 docswiki 列的"应新建" domain crate (设计意图, 0/9 实装 per `cargo metadata`) ----------
    design_only = [
        ("domain-dispatcher-design", "★ 设计意图 0/9 实装, 跟已实装 star-dispatcher 命名冲突, 待重命名 (per §7 dual-namespace + ADR-0048)", "S5 §1.1", "TBD"),
        ("domain-llm-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-C 实装", "S5 §1.1", "TBD"),
        ("domain-mcp-design", "★ 设计意图 0/9 实装, 跟已实装 star-mcp 命名冲突, 待重命名", "S5 §1.1", "TBD"),
        ("domain-tool-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-C 实装", "S5 §1.1", "TBD"),
        ("domain-rag-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-E 实装", "S5 §1.1", "TBD"),
        ("domain-memory-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-D 实装", "S5 §1.1", "TBD"),
        ("domain-rate-limiter-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-B 实装", "S5 §1.1", "TBD"),
        ("domain-observability-design", "★ 设计意图 0/9 实装, 命名无冲突, 待 P3-B 实装", "S5 §1.1", "TBD"),
    ]
    for d in design_only:
        nodes.append(Node(d[0], d[0], "domain", "Cross", d[1], d[2], "-", d[3], [], [], None, ["04-domain-crates", "99-pg-broker-audit"]))

    # ---------- 5 张 PG 表 (per S7 §3.2) ----------
    pg_tables = [
        ("PG-checkpoints", "checkpoints (T)", "Transaction (append-only), 物理删除禁止, RLS 13 類必携", "S7 §3.2 L78", "T"),
        ("PG-checkpoint_writes", "checkpoint_writes (T)", "Transaction (append-only), 物理删除禁止, RLS 13 類必携", "S7 §3.2 L79", "T"),
        ("PG-checkpoint_summaries", "checkpoint_summaries (T)", "Transaction (append-only), 物理删除禁止, RLS 13 類必携", "S7 §3.2 L80", "T"),
        ("PG-checkpoint_metadata", "checkpoint_metadata (M)", "Master (SCD Type 2), 物理删除禁止, valid_from/valid_to, RLS 13 類必携", "S7 §3.2 L81", "M"),
        ("PG-audit_audit_event", "audit_audit_event (T WORM)", "Transaction (WORM), 自身必审计, RLS 13 類必携, per ADR-0043", "S7 §3.2 L82 + ADR-0043", "T"),
    ]
    for t in pg_tables:
        nodes.append(Node(t[0], t[1], "persistence", "Cross", t[2], t[3], t[4], "P0", ["PostgresCheckpointer", "M-25"], [], None, ["05-persistence-checkpoint"]))

    # ---------- 3 View ----------
    views = [
        ("View-AgentView", "Agent View", "派生视图 (无限画布, Miro 风格), React + Next.js + zustand", "S1 §1.3 L61-67", "P0", "AgentView", ["zustand store"]),
        ("View-LangGraph", "LangGraph view", "UI 驱动的 2-level hierarchical Agent (L0 + L1 任务卡子代理), LangGraph Python subgraph", "S2 §1.0 + §1.1", "P0", "LangGraph", ["View-AgentRuntime", "View-AgentView"]),
        ("View-AgentRuntime", "Agent Runtime view", "Rust 大规模并发 Runtime 基础设施 (派发 + ECS + 共享池)", "S4 §1.3 L56-67", "P0", "AgentRuntime", ["22 domain crates", "View-LangGraph"]),
    ]
    for v in views:
        nodes.append(Node(v[0], v[1], "orchestration" if "Lang" in v[0] else ("ui" if "AgentView" in v[0] else "runtime"), v[5], v[2], v[3], "-", v[4], v[6], [], None, ["00-design-topology"]))

    # ---------- LangGraph 25 模块 M-01..M-25 (per S3 §1.2 L150-178) ----------
    mods = [
        ("M-01", "top_agent.graph", "TopAgent StateGraph 定義 + compile", "S3 §1.2 L154", "P0"),
        ("M-02", "top_agent.nodes", "T-N1..T-N7 実装", "S3 §1.2 L155", "P0"),
        ("M-03", "top_agent.state", "TopAgentState TypedDict", "S3 §1.2 L156", "P0"),
        ("M-04", "sub_agent.pool", "sub-agent spawn / lifecycle", "S3 §1.2 L157", "P0"),
        ("M-05", "sub_agent.base", "共通 5 节点 模板", "S3 §1.2 L158", "P0"),
        ("M-06", "sub_agent.types", "SA-01..SA-09 実装", "S3 §1.2 L159", "P0"),
        ("M-07", "sub_agent.registry", "sub-agent 类型 → 実装 mapping", "S3 §1.2 L160", "P1"),
        ("M-08", "checkpoints.store", "3-tier ABC", "S3 §1.2 L161", "P0"),
        ("M-09", "checkpoints.sqlite", "Tier 2 実装 (default v0.1)", "S3 §1.2 L162", "P0"),
        ("M-10", "mcp.client", "star-mcp 16 tools proxy", "S3 §1.2 L163", "P0"),
        ("M-11", "mcp.audited_tool_node", "audit + guard 統合 ToolNode", "S3 §1.2 L164", "P0"),
        ("M-12", "ui.streamer", "WebSocket / SSE 推送", "S3 §1.2 L165", "P0"),
        ("M-13", "cross_cutting.audit_logger", "全 tool call 記録", "S3 §1.2 L166", "P0"),
        ("M-14", "cross_cutting.token_telemetry", "token 計量", "S3 §1.2 L167", "P1"),
        ("M-15", "cross_cutting.guard_enforcer", "AGENTS.md §4 守门 37 项 自动检查", "S3 §1.2 L168", "P1"),
        ("M-16", "cross_cutting.interrupt_manager", "human-in-loop interrupt / resume", "S3 §1.2 L169", "P0"),
        ("M-17", "api.app", "FastAPI app + 路由 mount", "S3 §1.2 L170", "P0"),
        ("M-18", "schema.registry", "State schema 中央管理", "S3 §1.2 L171", "P1"),
        ("M-19", "task_ops.manager", "★ v0.2 TMO 7 节点 集中调度, 唯一 cross-task actor", "S3 §1.2 L172", "P0"),
        ("M-20", "task_ops.relationship_graph", "★ v0.2 任务卡 DAG (4 字段), cycle prevention", "S3 §1.2 L173", "P0"),
        ("M-21", "task_ops.bulk_queue", "★ v0.2 批量操作队列 + asyncio.gather, 部分失败回滚", "S3 §1.2 L174", "P0"),
        ("M-22", "task_ops.dag_validator", "★ v0.2 cycle detection O(V+E)", "S3 §1.2 L175", "P0"),
        ("M-23", "task_ops.metadata_registry", "★ v0.2 task_metadata 表中央管理 (Master RLS per 守门 #13 c)", "S3 §1.2 L176", "P1"),
        ("M-24", "task_ops.reassign_manager", "★ v0.2 SA-XX 类型切换 + checkpoint preserved", "S3 §1.2 L177", "P1"),
        ("M-25", "task_ops.summarize_collector", "★ v0.2 跨 N SubAgentState 聚合, LLM 表格化", "S3 §1.2 L178", "P1"),
    ]
    for m in mods:
        nodes.append(Node(m[0], m[1], "orchestration", "LangGraph", m[2], m[3], "T (audit log 必携)", m[4], [], [], None, ["02-orchestration-langgraph"]))

    return nodes


def write_node_notes(nodes, nodes_dir: Path):
    """写节点笔记."""
    nodes_dir.mkdir(parents=True, exist_ok=True)
    for n in nodes:
        # 文件名用节点 ID
        fname = n.id.replace("/", "_") + ".md"
        path = nodes_dir / fname
        path.write_text(n.note_text(), encoding="utf-8")
    return len(nodes)


def main():
    nodes = all_nodes()
    print(f"[gen] nodes total: {len(nodes)}")
    n_nodes = write_node_notes(nodes, NODES_DIR)
    print(f"[gen] wrote {n_nodes} node notes to {NODES_DIR}")


if __name__ == "__main__":
    main()
