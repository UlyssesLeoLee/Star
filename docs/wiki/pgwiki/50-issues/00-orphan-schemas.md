---
title: "Orphan Schemas(有表无 crate 映射)"
generated: "2026-09-06T02:52:23Z"
node_type: "audit-issue"
---

# Orphan Schemas(有表无 crate 映射)

## Orphan Schemas(有表但 SCHEMA_TO_CRATE 缺失)

**含义**: DB 里有表,但 `scripts/automation/pgwiki_index.py` 的 `SCHEMA_TO_CRATE` 映射表没列。
**风险**: pgwiki 不会给这些 schema 建 schema 节点 / crate 关联边,docwiki 对照时会缺一段。

| Schema | DB 文件数 | pgwiki 节点 | 应主责 crate(需 DDD Review 拍板)|
|---|---|---|---|
| `work` | 5 | 无 | ?(DDD Review 阶段拍)|

**生成时间**: 2026-09-06T02:52:23Z
**来源**: `pgwiki_audit.py` Issue 1

