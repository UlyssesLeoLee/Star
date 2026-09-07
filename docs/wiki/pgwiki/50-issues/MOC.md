---
title: "Issues MOC"
generated: "2026-09-06T09:12:11Z"
node_type: "moc"
---

# Issues 总览

**生成时间**: 2026-09-06T09:12:11Z

**对照目的** (per 2026-09-06 11:47 JST Ulysses 拍板):
- pgwiki = 程序实际拓扑(事实)
- docswiki = DDD 视角叙事
- 本目录 = 两者对照暴露的"模块间问题"

## 统计

| 维度 | 数 |
|---|---|
| Cargo workspace members | 52 |
| DB schema (有表) | 25 |
| Orphan schema (有表无映射) | 1 |
| Placeholder schema (无表) | 1 |
| Empty crate (src 空) | 0 |
| Broker ADR refs | 5 |
| Broker Arch refs | 32 |
| Table Module broken | 0 |
| Cargo fake deps | 0 |
| docswiki 文件 | 8 |

## Issue 索引

- [[00-orphan-schemas]] — Orphan Schemas(有表无 crate 映射)
- [[01-placeholder-schemas]] — Placeholder Schemas(crate 在,schema 表未落地)
- [[03-broker-adr-refs]] — Broker ADR Refs(ADR 引用但 crate 不存在)
- [[04-broker-arch-refs]] — Broker Arch View Refs
- [[07-docswiki-vs-pgwiki]] — docswiki vs pgwiki 对照表
