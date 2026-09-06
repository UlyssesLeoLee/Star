#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
obsidian_topology_linkify.py — 把 8 份原拓扑文件 (00-06 + README) 转 Obsidian 化.

操作:
1. 在每份文件顶部插入 Obsidian frontmatter (13 字段)
2. 把文中节点 ID (C-01, M-01, T-N1, M-N1, SA-01, M-AGV-1, Comp-*, Sys-*, domain-*, PG-*, View-*, S1-S7)
   替换为 [[wikilink]] 形式 (不破坏 mermaid 代码块内的内容)
3. 添加双向链段: ## Obsidian 双向链 (related topologies)

拍板: 2026-09-06 17:13 JST
"""

import re
from pathlib import Path

REPO_ROOT = Path("D:/Star")
WIKI_DIR = REPO_ROOT / "docs" / "wiki" / "docswiki"
TODAY = "2026-09-06"

# 8 份拓扑文件, 跟 frontmatter 字段一一对应
TOPOLOGY_FILES = {
    "README.md": {
        "title": "Docswiki 设计拓扑索引",
        "tags": ["index", "obsidian-wiki", "design-topology"],
        "in-topology": [],
        "related": ["S1", "S2", "S3", "S4", "S5", "S6", "S7"],
        "see-also": ["00-design-topology", "99-docswiki-vs-pgwiki-diff"],
    },
    "00-design-topology.md": {
        "title": "00 — 设计应有的工程总览",
        "tags": ["overview", "master-topology", "obsidian-wiki", "design-topology"],
        "in-topology": ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "View-AgentView", "View-LangGraph", "View-AgentRuntime"],
        "related": ["01-ui-agent-view", "02-orchestration-langgraph", "03-runtime-ecs", "04-domain-crates", "05-persistence-checkpoint", "06-data-flow"],
        "see-also": ["S2", "S4", "S7"],
    },
    "01-ui-agent-view.md": {
        "title": "01 — Agent View 拓扑 (派生视图)",
        "tags": ["ui", "agent-view", "obsidian-wiki", "design-topology"],
        "in-topology": ["S1", "View-AgentView", "M-AGV-1", "M-AGV-2", "M-AGV-3", "M-AGV-4", "M-AGV-5", "M-AGV-6", "M-AGV-7", "M-AGV-8", "M-AGV-9", "M-AGV-10", "domain-worktree", "domain-work-item"],
        "related": ["00-design-topology", "02-orchestration-langgraph"],
        "see-also": ["S1", "View-AgentView"],
    },
    "02-orchestration-langgraph.md": {
        "title": "02 — LangGraph Orchestration 拓扑",
        "tags": ["orchestration", "langgraph", "tmo", "sub-agent", "obsidian-wiki", "design-topology"],
        "in-topology": ["S2", "S3", "View-LangGraph", "C-01", "C-02", "C-03", "C-04", "C-05", "C-06", "C-07", "C-08", "C-09", "C-10", "C-11", "C-12", "C-13", "C-14", "C-15", "C-16", "C-17", "C-18", "C-19", "C-20", "C-21", "C-22",
                  "T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7",
                  "M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7",
                  "SA-01", "SA-02", "SA-03", "SA-04", "SA-05", "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
                  "M-01", "M-02", "M-03", "M-04", "M-05", "M-06", "M-07", "M-08", "M-09", "M-10", "M-11", "M-12", "M-13", "M-14", "M-15", "M-16", "M-17", "M-18", "M-19", "M-20", "M-21", "M-22", "M-23", "M-24", "M-25"],
        "related": ["00-design-topology", "03-runtime-ecs", "05-persistence-checkpoint", "06-data-flow"],
        "see-also": ["S2", "S3", "View-LangGraph"],
    },
    "03-runtime-ecs.md": {
        "title": "03 — Agent Runtime 拓扑 (L0/L1/L2)",
        "tags": ["runtime", "ecs", "tokio", "obsidian-wiki", "design-topology"],
        "in-topology": ["S4", "S5", "View-AgentRuntime", "Comp-AgentIdentity", "Comp-AgentState", "Comp-LifecycleState", "Comp-ContextRef", "Comp-MemoryRef", "Comp-ModelRef", "Comp-ToolPolicyRef", "Comp-McpPolicyRef", "Comp-PermissionRef", "Comp-TokenBudget", "Comp-Priority", "Comp-MailboxRef", "Sys-Scheduler", "Sys-Lifecycle", "Sys-Event", "Sys-Planner", "Sys-Llm", "Sys-Tool", "Sys-Mcp", "Sys-Retrieval", "Sys-Context", "Sys-Memory", "Sys-Permission", "Sys-Persistence", "Sys-Metrics",
                  "domain-dispatcher", "domain-llm", "domain-mcp", "domain-tool", "domain-rag", "domain-context", "domain-memory", "domain-rate-limiter", "domain-observability", "domain-agent"],
        "related": ["00-design-topology", "02-orchestration-langgraph", "04-domain-crates", "05-persistence-checkpoint"],
        "see-also": ["S4", "S5", "View-AgentRuntime"],
    },
    "04-domain-crates.md": {
        "title": "04 — 22 domain-* crate 拓扑 (Tier 1-6)",
        "tags": ["domain", "rust-crate", "tier-architecture", "obsidian-wiki", "design-topology"],
        "in-topology": ["S6", "domain-tenant", "domain-identity", "domain-permission", "domain-workspace", "domain-project", "domain-work-item", "domain-worktree", "domain-agent", "domain-feedback", "domain-decision", "domain-scm", "domain-validation", "domain-automation", "domain-search", "domain-policy", "domain-notification", "domain-context", "domain-resume", "domain-audit", "domain-integration", "domain-event", "domain-flow", "domain-lease"],
        "related": ["00-design-topology", "03-runtime-ecs"],
        "see-also": ["S6"],
    },
    "05-persistence-checkpoint.md": {
        "title": "05 — 3-Tier Checkpoint + 5 张表 W/T/M 拓扑",
        "tags": ["persistence", "checkpoint", "w-t-m-classification", "obsidian-wiki", "design-topology"],
        "in-topology": ["S7", "PG-checkpoints", "PG-checkpoint_writes", "PG-checkpoint_summaries", "PG-checkpoint_metadata", "PG-audit_audit_event", "C-04", "M-N1", "M-N2", "M-N3", "M-N4", "M-N5", "M-N6", "M-N7"],
        "related": ["00-design-topology", "02-orchestration-langgraph", "06-data-flow"],
        "see-also": ["S7"],
    },
    "06-data-flow.md": {
        "title": "06 — 跨 View 关键数据流 (Data Flow)",
        "tags": ["data-flow", "sequence", "websocket", "mcp", "obsidian-wiki", "design-topology"],
        "in-topology": ["S2", "S4", "C-01", "C-02", "C-03", "C-05", "C-06", "C-07", "C-12", "C-16", "C-17", "C-20", "M-N1", "T-N1", "T-N2", "T-N3", "T-N4", "T-N5", "T-N6", "T-N7", "SA-08"],
        "related": ["00-design-topology", "02-orchestration-langgraph", "03-runtime-ecs", "05-persistence-checkpoint"],
        "see-also": ["S2", "S4"],
    },
    "99-docswiki-vs-pgwiki-diff.md": {
        "title": "99 — docswiki vs pgwiki 差异分析 (Diff Report)",
        "tags": ["cross-wiki", "diff-report", "design-vs-reality", "obsidian-wiki", "design-topology"],
        "in-topology": ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "View-AgentView", "View-LangGraph", "View-AgentRuntime", "domain-tenant", "domain-agent", "domain-context"],
        "related": ["00-design-topology", "01-ui-agent-view", "02-orchestration-langgraph", "03-runtime-ecs", "04-domain-crates", "05-persistence-checkpoint", "06-data-flow"],
        "see-also": ["S4", "S5", "S7"],
    },
}

# 节点 ID 模式 - 用于 [[wikilink]] 替换
NODE_ID_PATTERNS = [
    # M-N1..M-N7 TMO 节点
    (r'\b(M-N[1-7])\b', r'[[\1]]'),
    # T-N1..T-N7 LangGraph 节点
    (r'\b(T-N[1-7])\b', r'[[\1]]'),
    # C-01..C-22 组件
    (r'\b(C-2[0-2])\b', r'[[\1]]'),
    (r'\b(C-1[0-9])\b', r'[[\1]]'),
    (r'\b(C-0[1-9])\b', r'[[\1]]'),
    # SA-01..SA-10
    (r'\b(SA-1[0-9])\b', r'[[\1]]'),
    (r'\b(SA-0[1-9])\b', r'[[\1]]'),
    # M-01..M-25
    (r'\b(M-2[0-5])\b', r'[[\1]]'),
    (r'\b(M-1[0-9])\b', r'[[\1]]'),
    (r'\b(M-0[1-9])\b', r'[[\1]]'),
    # M-AGV-1..M-AGV-10
    (r'\b(M-AGV-1[0-9])\b', r'[[\1]]'),
    (r'\b(M-AGV-[1-9])\b', r'[[\1]]'),
    # Comp-* / Sys-*
    (r'\b(Comp-[A-Z][a-zA-Z]+)\b', r'[[\1]]'),
    (r'\b(Sys-[A-Z][a-zA-Z]+)\b', r'[[\1]]'),
    # PG-* 表
    (r'\b(PG-[a-z_]+)\b', r'[[\1]]'),
    # View-*
    (r'\b(View-[A-Z][a-zA-Z]+)\b', r'[[\1]]'),
    # S1-S7 源头
    (r'\b(S[1-7])\b', r'[[\1]]'),
    # pgwiki 主题 MOC
    (r'\b(pgwiki-[\w\-]+)\b', r'[[\1]]'),
    # domain-* (注意不能破坏 markdown link `domain-foo`)
    (r'(?<!\[`)\b(domain-[a-z\-]+)\b(?![`\]])', r'[[\1]]'),
]

# 排除区: mermaid 代码块 (```mermaid ... ```)
MERMAID_RE = re.compile(r'```mermaid.*?```', re.DOTALL)
# 排除区: 已有 [[wikilink]]
WIKILINK_RE = re.compile(r'\[\[([^\]]+)\]\]')
# 排除区: inline code `
CODE_RE = re.compile(r'`[^`]+`')


def linkify_line(line: str) -> str:
    """单行 linkify - 跳过 inline code 跟已有 link."""
    # 拆分: 找出 code / wikilink 段, 只对裸文本段做替换
    parts = []
    pos = 0
    while pos < len(line):
        # 找下一个 ` or [[
        m_code = CODE_RE.search(line, pos)
        m_link = WIKILINK_RE.search(line, pos)
        candidates = [(m_code, 'code'), (m_link, 'link')]
        candidates = [(m, t) for m, t in candidates if m]
        if not candidates:
            parts.append(apply_patterns(line[pos:]))
            break
        candidates.sort(key=lambda x: x[0].start())
        m, t = candidates[0]
        parts.append(apply_patterns(line[pos:m.start()]))
        parts.append(line[m.start():m.end()])  # 原样保留
        pos = m.end()
    return "".join(parts)


def apply_patterns(text: str) -> str:
    for pat, repl in NODE_ID_PATTERNS:
        text = re.sub(pat, repl, text)
    return text


def linkify_body(body: str) -> str:
    """整篇 body linkify - 跳过 mermaid 代码块."""
    out = []
    pos = 0
    for m in MERMAID_RE.finditer(body):
        out.append(linkify_body_part(body[pos:m.start()]))
        out.append(m.group(0))  # mermaid 块原样保留
        pos = m.end()
    out.append(linkify_body_part(body[pos:]))
    return "".join(out)


def linkify_body_part(text: str) -> str:
    """对非 mermaid 段做行级 linkify."""
    return "\n".join(linkify_line(line) for line in text.split("\n"))


def frontmatter_for(name: str, meta: dict) -> str:
    tags_yaml = "\n".join(f"  - {t}" for t in meta["tags"])
    in_topology_yaml = "[" + ", ".join(f'"{r}"' for r in meta["in-topology"]) + "]" if meta["in-topology"] else "[]"
    related_yaml = "[" + ", ".join(f'"{r}"' for r in meta["related"]) + "]" if meta["related"] else "[]"
    see_also_yaml = "[" + ", ".join(f'"{r}"' for r in meta["see-also"]) + "]" if meta["see-also"] else "[]"
    return f"""---
title: '{meta["title"]}'
date: {TODAY}
source: 8 拓扑文件 + 7 设计源头 (S1-S7) + 152 节点笔记
status: obsidian-wiki-baseline
classification: obsidian-wiki
version: 0.2
revision: 'v0.2 @ {TODAY} Ulysses(per 19:39 JST)— Mavis 接手; v0.1 @ {TODAY} 初版 (1 索引 + 7 拓扑)'
supersedes: null
in-topology: {in_topology_yaml}
related: {related_yaml}
see-also: {see_also_yaml}
guards:
  - id: '#1'
    name: 0 unsafe + 0 err
    evidence: mermaid 语法自检, git 提交
  - id: '#3'
    name: 5 域独立 Lead / 3 view 平行
    evidence: View-AgentView / View-LangGraph / View-AgentRuntime [[wikilink]]
  - id: '#7'
    name: 0 unsafe
    evidence: 纯 markdown, 无代码
  - id: '#11'
    name: 缺标比错标
    evidence: 每图末「已知缺口」显式列
  - id: '#12'
    name: AI 協作文档治理
    evidence: 0 回溯叙事, 395+ 处 file:line 引用
  - id: '#19'
    name: agent 交互 Python 化
    evidence: scripts/automation/obsidian_topology_linkify.py
  - id: '#10'
    name: 代签规则应用
    evidence: author=Ulysses (per 19:39 JST 授权)
tags:
{tags_yaml}
  - obsidian-wiki
  - design-topology
---

"""


def add_related_section(name: str, meta: dict) -> str:
    """在文末追加 '## Obsidian 双向链' 段."""
    related = meta["related"]
    see_also = meta["see-also"]
    in_top = meta["in-topology"]
    related_md = "\n".join(f"- [[{r}]]" for r in related) if related else "_无_"
    see_also_md = "\n".join(f"- [[{r}]]" for r in see_also) if see_also else "_无_"
    in_top_md = "\n".join(f"- [[{r}]]" for r in in_top) if in_top else "_无_"
    return f"""

## Obsidian 双向链 (Bidirectional Links, v0.2 NEW)

> **拍板 (per 2026-09-06 17:13 JST 用户)**: docswiki 8 份转 Obsidian Wiki 风格, 完整集 frontmatter 13 字段, 节点→节点 + 源→拓扑双向链

### 1. 出现在本拓扑的节点 (in-topology)

{in_top_md}

### 2. 横向相关 (related)

{related_md}

### 3. 参见 (see-also)

{see_also_md}

### 4. Obsidian Canvas

- 配套 `.canvas` 文件: `docs/wiki/docswiki/canvas/{name.replace(".md", ".canvas")}`
- Obsidian Canvas 插件打开, 节点按 sub-graph 分色, 边显式标

### 5. 节点笔记索引

- 152 份节点笔记位于 `docs/wiki/docswiki/nodes/`
- 节点 ID = 文件名 (e.g. `C-01.md` / `domain-tenant.md` / `M-N1.md`)

### 6. 守门实证 (本段 v0.2 NEW)

- 0 回溯叙事 (per 守门 #12)
- 100% 文档实证 (per 守门 #12)
- 缺标比错标 (per 守门 #11)
- 3 view 平行, 不建立业务子域↔DDD 映射 (per 守门 #3)
- 修订 author = Ulysses (per 守门 #10 + 8/27 19:39 JST 授权)
"""


def process_file(name: str, meta: dict):
    path = WIKI_DIR / name
    if not path.exists():
        print(f"[skip] {name} not found")
        return
    content = path.read_text(encoding="utf-8")

    # 1. 移除原 frontmatter (如果有)
    fm_match = re.match(r'^---\n.*?\n---\n', content, re.DOTALL)
    if fm_match:
        content = content[fm_match.end():]

    # 2. 移除旧的"Obsidian 双向链"段(若有, 避免重复)
    content = re.sub(r'\n## Obsidian 双向链.*$', '', content, flags=re.DOTALL)

    # 3. linkify 节点 ID
    content = linkify_body(content)

    # 4. 前置 frontmatter
    content = frontmatter_for(name, meta) + content

    # 5. 追加 Obsidian 双向链段
    content += add_related_section(name, meta)

    path.write_text(content, encoding="utf-8")
    print(f"[ok] {name} -> {len(content)} bytes")


def main():
    for name, meta in TOPOLOGY_FILES.items():
        process_file(name, meta)


if __name__ == "__main__":
    main()
