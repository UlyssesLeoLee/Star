---
title: "Broker ADR Refs(ADR 引用但 crate 不存在)"
generated: "2026-09-06T02:52:23Z"
node_type: "audit-issue"
---

# Broker ADR Refs(ADR 引用但 crate 不存在)

## Broker ADR Refs(ADR 引用但 workspace 无)

**含义**: 27 份 ADR 文档中提到这些 crate 段,但 `Cargo.toml` 没注册。
**风险**: ADR 是历史决策,可能(1)crate 已重命名(2)crate 被废弃(3)ADR 描述超前(4)措辞非 crate 名。
**行动**: 逐条 git log --follow 实证,或在 DDD Review 阶段删 ADR 引用 / 补 crate。

| 引用名 | 出现 ADR | 实证片段(前 80 字符) |
|---|---|---|
| `api-key` |  | `(无)` |
| `domain-service` | 0040-domain-batch.md | `- 节点类型 `domain-service` 调用 33 `domain-*` crate service (F-050, per star_context 端口)` |
| `domain-team` | 0034-jira-ification.md | `4. **22 DDD bounded context 解耦** (per AGENTS.md §5 + Q1-D 拍板): `domain-identity` / `domain-team` 实体应放对应 crate, W1 跟 `dom` |
| `star-lsp-proxy` | 0027-star-ide-gateway.md | `- LSP Proxy（`star-lsp-proxy`）MVP 阶段不实现；Phase 2 再加（per arch/04 §6）` |
| `star-optional` | 0025-vendor-adapter-anti-contamination.md | `- workspace 多了一层（`star-optional`）` |

**生成时间**: 2026-09-06T02:52:23Z
**来源**: `pgwiki_audit.py` Issue 4

