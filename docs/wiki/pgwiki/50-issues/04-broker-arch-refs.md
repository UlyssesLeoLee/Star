---
title: "Broker Arch View Refs"
generated: "2026-09-07T11:36:27Z"
node_type: "audit-issue"
---

# Broker Arch View Refs

## Broker Arch View Refs(arch view 引用但 workspace 无)

**含义**: 5 大架构 view 文档提到这些 crate 段,但 `Cargo.toml` 没注册。
**行动**: DDD Review / arch view 文档升版时复核。

| 引用名 |
|---|
| `api-key` |
| `domain-backpressure` |
| `domain-cb` |
| `domain-dispatcher` |
| `domain-graph-agent` |
| `domain-http` |
| `domain-memory` |
| `domain-observability` |
| `domain-ops-rbac` |
| `domain-policy` |
| `domain-prompt` |
| `domain-provider` |
| `domain-queue` |
| `domain-rag` |
| `domain-rate-limiter` |
| `domain-retry` |
| `domain-service` |
| `domain-team` |
| `star-cache-readonly` |
| `star-ide-gateway` |
| `star-lsp-proxy` |
| `star-mcp-readwrite` |
| `star-optional` |
| `star-postgres` |
| `star-redis` |
| `star-rest` |
| `star-sa-cluster` |
| `star-system` |

**生成时间**: 2026-09-07T11:36:27Z
**来源**: `pgwiki_audit.py` Issue 5

## Counters

```json
{"broker_arch": 28, "broker_arch_list": ["api-key", "domain-backpressure", "domain-cb", "domain-dispatcher", "domain-graph-agent", "domain-http", "domain-memory", "domain-observability", "domain-ops-rbac", "domain-policy", "domain-prompt", "domain-provider", "domain-queue", "domain-rag", "domain-rate-limiter", "domain-retry", "domain-service", "domain-team", "star-cache-readonly", "star-ide-gateway", "star-lsp-proxy", "star-mcp-readwrite", "star-optional", "star-postgres", "star-redis", "star-rest", "star-sa-cluster", "star-system"]}
```

