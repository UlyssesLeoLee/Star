---
title: "Schema → Crate 提议映射(待 DDD Review 拍板)"
generated: "2026-09-06T02:52:23Z"
node_type: "cross-schema-crate"
---

# Schema → Crate 提议映射(待 DDD Review 拍板)

# Schema -> Crate 提议映射(待 DDD Review 拍板,非权威)

> **⚠️ 重要 disclaimer** (per 2026-09-06 11:47 JST Ulysses 拍板 + AGENTS.md §5 仓库拓扑 disclaimer + 守门 #3 拒绝兼任):
>
> 本表是**程序实际拓扑**的观察:`scripts/automation/pgwiki_index.py` 的 `SCHEMA_TO_CRATE` 常量直接来自历史 schema name ↔ crate name 命名重合。**不等于** DDD bounded context 划分。
>
> **真实情况**:Star 当前 workspace 有 35 个 `domain-*` crate + 17 个其它 crate,docs/architecture 描述了 DDD 设计意图。两者**不**建立 1:1 映射。
>
> **行动**:DDD Review 阶段由 Lead 复核本表,确认是否调整 SCHEMA_TO_CRATE 常量 / 修订 docswiki 叙事。

**依据**: 守门 #13 W/T/M + AGENTS.md §6.1 命名解读 disclaimer + ADR-0044

| Schema | 提议主责 crate | 表数 |
|---|---|---|
