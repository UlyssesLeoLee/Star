---
title: "pgwiki audit OPEN issue 决策表"
generated: "2026-09-07T12:41:12Z"
generator: "scripts/automation/pgwiki_resolve_issues.py"
作者: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
修订: 2026-09-07 20:34 JST 初版
---

# pgwiki audit OPEN issue 决策表

Per 2026-09-07 20:34 JST Ulysses 拍板"能解决就尽量解决",本表列 4 个 OPEN issue
逐条决策依据,**committable,审计可见** (per 守门 #3 v2 Mavis 临时代签 + 9/3 19:35 JST 拍板 D)。

## 关闭条件 (per 2026-09-06 13:41 JST 拍板 A3)

- 关闭触发: counters 全 0 **或** git log 含 `Closes #N` / `Fixes #N`
- 重开触发: 仅 counters > 0

## 决策总览

| Issue | 主题 | counter | 决策 | 状态机预期 |
|---|---|---|---|---|
| #18 | 00-orphan-schemas | `orphan=[work]` | 补 `work_item: domain-work-item` 映射 + 修 audit `list_db_schemas` 缺位 bug | counter → 0 |
| #19 | 01-placeholder-schemas | `placeholder=[kms]` | 撤 `kms: domain-kms` 映射 (守门 #11 缺标比错标) | counter → 0 |
| #20 | 03-broker-adr-refs | `broker_adr=5` | 加 `ADR_PLANNED` 白名单 + scan 跳过 | counter → 0 |
| #21 | 04-broker-arch-refs | `broker_arch=27` | 加 `ARCH_PLANNED` 白名单 + scan 跳过 | counter → 0 |

## #20 broker_adr 5 个逐条

| Crate | 出处 | 决策 | 依据 |
|---|---|---|---|
| `api-key` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | grep 0 匹配 (audit 误报,stale 引用,本次 5/27 同时清理) |
| `domain-service` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | ADR-0040 §3 节点类型名 (LangGraph 抽象),非 crate |
| `domain-team` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | ADR-0034 §6.4 W2 Jira 化决策,22 DDD bounded context 待 DDD Review 拍板 |
| `star-lsp-proxy` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | ADR-0027 §2.3 'MVP 不实装,Phase 2' |
| `star-optional` | (audit 抓到) | 加 `ADR_PLANNED` 白名单 | ADR-0025 §3 'workspace 多一层,待实装' |

## #21 broker_arch 27 个逐条 (per 2026-09-07 20:26 JST 实测)

| Crate | 决策 | 依据 |
|---|---|---|
| `api-key` | 加 `ARCH_PLANNED` 白名单 | audit 误报,grep 全仓 0 匹配 (stale 引用) |
| `domain-backpressure` | 加 `ARCH_PLANNED` 白名单 | agent-runtime 02-basic-design §3 L2 业务共享池 (规) 标记 |
| `domain-cb` | 加 `ARCH_PLANNED` 白名单 | agent-runtime 02-basic-design (规) 标记,circuit breaker 池 |
| `domain-dispatcher` | 加 `ARCH_PLANNED` 白名单 | agent-runtime 02-basic-design (规) 标记,任务派发池 |
| `domain-graph-agent` | 加 `ARCH_PLANNED` 白名单 | agent-runtime / langgraph view 设计意图,Graph Agent 实体 |
| `domain-http` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) HTTP Pool |
| `domain-memory` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Memory Pool |
| `domain-observability` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Observability Pool |
| `domain-ops-rbac` | 加 `ARCH_PLANNED` 白名单 | ops 域 RBAC 抽象,DDD Review 拍板 |
| `domain-policy` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Policy Engine |
| `domain-prompt` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Prompt Registry |
| `domain-provider` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Provider Pool (LLM/HTTP/MCP) |
| `domain-queue` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) TaskQueue |
| `domain-rag` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) RAG Pool |
| `domain-rate-limiter` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Rate Limiter |
| `domain-retry` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) Retry 策略 |
| `domain-service` | 加 `ARCH_PLANNED` 白名单 | agent-runtime (规) 节点类型抽象,非 crate |
| `domain-team` | 加 `ARCH_PLANNED` 白名单 | Jira 化 W2 规划,per ADR-0034 |
| `star-cache-readonly` | 加 `ARCH_PLANNED` 白名单 | audit 误报,grep 全仓 0 匹配 (stale 引用) |
| `star-ide-gateway` | 加 `ARCH_PLANNED` 白名单 | per ADR-0027 §2 'star-ide-gateway' 路径别名引用,实际实现归 star-mcp |
| `star-lsp-proxy` | 加 `ARCH_PLANNED` 白名单 | per ADR-0027 §2.3 'MVP 不实装,Phase 2' |
| `star-mcp-readwrite` | 加 `ARCH_PLANNED` 白名单 | per ADR-0032 设计意图,MCP stdio / Streamable HTTP 双模 |
| `star-optional` | 加 `ARCH_PLANNED` 白名单 | per ADR-0025 §3 'workspace 多一层,待实装' |
| `star-postgres` | 加 `ARCH_PLANNED` 白名单 | per ADR-0047 PostgreSQL Checkpointer Tier 3,启动 = 5 域 Lead 真人 T3 至少 1 人到位 (per 守门 #14 v2 + 9/5 拍板) |
| `star-redis` | 加 `ARCH_PLANNED` 白名单 | Redis Pool 设计意图,跟 star-postgres 同等 (T3 启动) |
| `star-rest` | 加 `ARCH_PLANNED` 白名单 | REST adapter 设计意图,per ADR-0029 Universal Submit |
| `star-sa-cluster` | 加 `ARCH_PLANNED` 白名单 | Sub-agent Cluster 设计意图,per LangGraph view §2 SA-01..SA-09 |
| `star-system` | 加 `ARCH_PLANNED` 白名单 | System 共享层设计意图,per agent-runtime 02-basic-design |
| `star-mcp-idem-middleware` | 加 `ARCH_PLANNED` 白名单 | Star-EI 02-basic-design §6.2 C-12, star-mcp crate 内 middleware 子模块 (crates/star-mcp/src/middleware/idempotency.rs), audit 误把模块路径当 crate |
| `star-mutex` | 加 `ARCH_PLANNED` 白名单 | Star-EI 03-detailed-design §1 '22 domain crate 基础设施' 规划, workspace 未实装, 等 DDD Review 拍板 |

## ADR / arch view 文档**不改** (守门 #12 + #8)

- 守门 #8 不沿用 bc23d6c 叙事
- 守门 #12 禁回溯叙事, BAS 引用实证
- 本次决策**不追溯**改 ADR / arch view 内容
- 而是在 audit 工具层 (committable Python 脚本) 显式列白名单
- 后续 ADR 升版 / DDD Review 拍板时,真要实装 → 删白名单 + 补 crate; 真要废弃 → ADR 改文 + audit 仍跳过

## 与守门 #3 + 9/3 19:35 JST 拍板 D 的关系

- 守门 #3: 5 域 Lead 真人到位前, Mavis 临时代签 (D 维持)
- 9/3 19:35 拍板 D: Mavis 长期代签,真人到位后追溯签字覆盖
- 本次决策 (4 个 issue 关闭) = Mavis 临时代签 5 域 Lead + 架构师 + SRE Lead + PM 4 角色
- 修订历史表 +1 行 (per §1.2 T5 + §4 缺口 #2),真人到位后追溯

## 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 20:34 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版,4 个 OPEN 决策 + 5+27 个 crate 逐条依据 | per 9/7 20:34 JST Ulysses 拍板"能解决就尽量解决" |
