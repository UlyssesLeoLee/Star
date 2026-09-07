---
title: "Placeholder Schemas(crate 在,schema 表未落地)"
generated: "2026-09-07T11:39:18Z"
node_type: "audit-issue"
---

# Placeholder Schemas(crate 在,schema 表未落地)

## Placeholder Schemas(占位:无表 + SCHEMA_TO_CRATE 列出)

**含义**: crate 已注册在 workspace,`SCHEMA_TO_CRATE` 显式列出 schema 名,但 `docs/data-design/ipa-detail/tables/` 没表文件。
**风险**: 该 crate 的持久化层未落地 / schema 设计在另一份文档,可能跟 docs/architecture 描述的 EFS 不一致。

| Schema | 占位 crate | 实际表数 | 建议 |
|---|---|---|---|

**生成时间**: 2026-09-07T11:39:18Z
**来源**: `pgwiki_audit.py` Issue 2

## Counters

```json
{"placeholder_schemas": 0, "placeholder_list": []}
```

