---
title: "docswiki vs pgwiki 对照表"
generated: "2026-09-07T12:41:12Z"
node_type: "audit-issue"
---

# docswiki vs pgwiki 对照表

## docswiki vs pgwiki 对照表

**设计原则** (per 2026-09-06 11:47 JST 拍板):
- **docswiki** = DDD 视角的叙事(7 份设计主题)
- **pgwiki** = 程序实际拓扑(212 节点,事实)
- **不**建立 1:1 映射(per 守门 #3 + AGENTS.md §5 disclaimer)
- 对照目的:暴露"叙事与事实"的差异 = **问题**

| docwiki 文件 | 主题 | 提到的 crate 段 | pgwiki 对应层 | 差异/缺口 |
|---|---|---|---|---|
| [[docswiki-00-design-topology]] | 工程拓扑总览(DDD 视角) | [[crate-domain-agent]], [[crate-domain-audit]], [[crate-domain-automation]], [[crate-domain-context]], [[crate-domain-feedback]], [[crate-domain-identity]] | [[30-architecture/MOC]] | 已实装 21 / 设计意图 12 |
|  |  | **设计意图(workspace 未实装)**: `domain-crates`, `domain-decision`, `domain-dispatcher`, `domain-event`, `domain-flow`, `domain-lease` ... (+6) |  |  |
| [[docswiki-01-ui-agent-view]] | UI Agent view | [[crate-domain-tenant]], [[crate-domain-work-item]], [[crate-domain-worktree]] | [[10-workspace/frontend-root]] | 已实装 3 / 设计意图 1 |
|  |  | **设计意图(workspace 未实装)**: `star-store` |  |  |
| [[docswiki-02-orchestration-langgraph]] | LangGraph 编排 | [[crate-domain-tenant]], [[crate-star-mcp]] | [[30-architecture/views/view-2026-09-03-langgraph]] | 已实装 2 / 设计意图 2 |
|  |  | **设计意图(workspace 未实装)**: `domain-dev`, `star-lg` |  |  |
| [[docswiki-03-runtime-ecs]] | ECS Runtime | [[crate-domain-agent]], [[crate-domain-context]], [[crate-domain-identity]], [[crate-domain-llm]], [[crate-domain-mcp]], [[crate-domain-permission]] | [[crate-star-dispatcher]] | 已实装 12 / 设计意图 8 |
|  |  | **设计意图(workspace 未实装)**: `domain-backpressure`, `domain-crates`, `domain-dispatcher`, `domain-memory`, `domain-observability`, `domain-rag` ... (+2) |  |  |
| [[docswiki-04-domain-crates]] | DDD domain crate 划分 | [[crate-domain-agent]], [[crate-domain-audit]], [[crate-domain-automation]], [[crate-domain-context]], [[crate-domain-feedback]], [[crate-domain-identity]] | [[10-workspace/_crates]] | 已实装 35 / 设计意图 23 |
|  |  | **设计意图(workspace 未实装)**: `domain-context-design`, `domain-crates`, `domain-decision`, `domain-dispatcher`, `domain-dispatcher-design`, `domain-event` ... (+17) |  |  |
| [[docswiki-05-persistence-checkpoint]] | 持久化 + checkpoint | [[crate-domain-tenant]] | [[20-database/MOC]] | 已实装 1 / 设计意图 2 |
|  |  | **设计意图(workspace 未实装)**: `domain-lead-referral`, `star-checkpoints` |  |  |
| [[docswiki-06-data-flow]] | 数据流 | [[crate-domain-tenant]], [[crate-star-mcp]] | [[40-crosscutting/dependencies]] | 已实装 2 / 设计意图 0 |

**生成时间**: 2026-09-07T12:41:12Z
**来源**: `pgwiki_audit.py` Issue 7

## Counters

```json
{"docswiki_contrast": true, "topic_count": 7}
```

