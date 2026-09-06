---
title: "pgwiki 总目录"
generated: "2026-09-06T01:19:57Z"
node_type: "moc-root"
---

# pgwiki 总目录

# pgwiki 总目录 (D:/Star Obsidian Vault)

**生成时间**: 2026-09-06T01:19:57Z
**节点数**: 181 (crate + table + ADR + view + 根级)
**crate**: 52
**DB table**: 93
**DB schema**: 25
**ADR**: 27
**架构 view**: 5

## 拓扑分层

- [[10-workspace/MOC]] — 物理代码(48 crate + frontend + tools + scripts)
- [[20-database/MOC]] — 数据库(25 schema × 93 table,per docs/data-design/ipa-detail/)
- [[30-architecture/MOC]] — 架构 view + 27 份 ADR
- [[40-crosscutting/dependencies]] — Crate 依赖图(per Cargo.toml)
- [[40-crosscutting/schema-to-crate]] — Schema ↔ Crate 映射(守门 #13)
- [[40-crosscutting/quality-gates]] — 守门 4 维

## 边类型(全部 frontmatter)

- `parent` / `child` — 层级
- `depends_on` — crate → crate (per Cargo.toml `path = "../x"`)
- `owner_crate` — schema → crate (per 守门 #13 派生)
- `arch_view` — crate → view (per ADR-0044/0045/0046)
- `part_of_view` — ADR → view

## Obsidian 使用

1. 用 Obsidian 打开 `D:/Star/docs/wiki/pgwiki/` (File -> Open vault -> Open folder as vault)
2. 启用 Graph view: 点击左侧栏 Graph 图标
3. 启用 Backlinks: 默认开启
4. 搜索: Ctrl/Cmd + Shift + F
