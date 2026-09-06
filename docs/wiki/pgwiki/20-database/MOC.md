---
title: "Database MOC"
generated: "2026-09-06T01:19:56Z"
node_type: "moc"
---

# Database MOC

# Database 总览

PostgreSQL 设计全 26 schema × 93 表 (per `docs/data-design/ipa-detail/00-INVENTORY.md` v0.2)。

**说明**: 25 schema 有表文件,另 1 schema (空) 来自 SCHEMA_TO_CRATE 映射(per 守门 #13 派生,crate 存在但 schema 表尚未落地)。

## 守门 #13 W/T/M 分类 (per 2026-09-01 18:30 JST 拍板)

| 分类 | 含义 | 表数 |
|---|---|---|
| M | (per `00-CLASSIFICATION-W-T-M.md` v0.1 启发式推断,DDD Review 阶段需 Lead 复核) | 5 |
| T | (per `00-CLASSIFICATION-W-T-M.md` v0.1 启发式推断,DDD Review 阶段需 Lead 复核) | 86 |
| W | (per `00-CLASSIFICATION-W-T-M.md` v0.1 启发式推断,DDD Review 阶段需 Lead 复核) | 2 |

## Schema 索引
- [[schema-agent]]
- [[schema-audit]]
- [[schema-automation]]
- [[schema-board]]
- [[schema-collaboration]]
- [[schema-comment]]
- [[schema-context]]
- [[schema-development]]
- [[schema-feedback]]
- [[schema-identity]]
- [[schema-integration]]
- [[schema-local]]
- [[schema-notification]]
- [[schema-permission]]
- [[schema-planning]]
- [[schema-project]]
- [[schema-relation]]
- [[schema-scm]]
- [[schema-search]]
- [[schema-tenant]]
- [[schema-validation]]
- [[schema-work]]
- [[schema-workflow]]
- [[schema-workspace]]
- [[schema-worktree]]
- [[schema-kms]]
