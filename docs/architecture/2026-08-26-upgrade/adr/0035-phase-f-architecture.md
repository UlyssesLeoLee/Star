# ADR-0035: Phase F 真实数据源接入架构

> **状态**：🟢 Active v0.1（Draft 提交即激活 per AGENTS.md §1.1）
> **日期**：2026-08-27
> **制定人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手（per 2026-08-27 21:59 JST 用户授权第三次强化 + 19:39/20:56 JST 连续发令"允许你代签"）
> **签批**：架构师（Mavis 接手 agent per DEC-008）— Mavis 接手代签（per §9 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../../plan/2026-08-26-upgrade-plan.md)
> **依赖**：[ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) · [ADR-0034 Phase E Architecture](0034-phase-e-architecture.md) · [AGENTS.md §0 一句话硬约束](../../../../AGENTS.md)
> **关联**：[spec/services/01-service-adapter-spec.md §1-§3](../spec/services/01-service-adapter-spec.md) · [spec/services/02-sse-streaming-spec.md §3](../spec/services/02-sse-streaming-spec.md) · [spec/services/03-webhook-adapter-spec.md §2-§5](../spec/services/03-webhook-adapter-spec.md) · [spec/agents/01-agent-runtime-spec.md §2 Lease 协议](../spec/agents/01-agent-runtime-spec.md) · [arch/05 §2 GitGit Compat Arch](../arch/05-gitgit-compat-arch.md) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md §5 待办 #2](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) · [PHASE-D3-MCP-TRANSPORT-REPORT.md §2 6 字段错误模型](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md)

---

## 0. 一句话硬约束
> **可以代签 Ulysses，不可以编造历史。**
> — per AGENTS.md §0 + 2026-08-27 19:39 JST 用户授权升级 + 21:59 JST 第三次强化
---

## 1. 背景

### 1.1 Phase E 已交付（per 2026-08-27 19:36-19:59 JST + base commit 938e9ab）

| 阶段 | 交付 | 关键 commit | 引用 |
|---|---|---|---|
| Phase E ADR | [adr/0034-phase-e-architecture.md](0034-phase-e-architecture.md) (357 行) | `b3472c3` (per `git log -p --follow` Phase F base) | ADR-0034 §1.1 |
| Phase E spec/services | 3 份 spec 616 行 = SA + SSE + Webhook | `6701917` (merge feat/phase-e-spec-services) | spec/services/01-03 |
| Phase E spec/agents | 1 份 spec 18 KB = agent runtime | `9aaf014` (merge feat/phase-e-adr) | spec/agents/01 |
| Phase E star-mcp 实装 | 4 文件 (resources/prompts/error/error_codes) + 16 配套 + 49 测试 | `6a3a7f9` (merge feat/phase-e-mcp-impl) | ADR-0034 §2 D4 |

**关键 Phase E 决定**（per [ADR-0034 §1.2 关键不变项](0034-phase-e-architecture.md) + §2 D5）：
- Phase E 只完成 **spec + 错误模型 + Resources/Prompts 骨架**，**真实数据源接入推 Phase F**
- Phase E 用 **mock data + `// TODO(phase-f): 接入 22 domain crate 真实数据` 注释**（per ADR-0034 §2 D5 L146）
- `spec/services/01+02+03` 是不进 MVP 退出条件的服务适配器 spec（per ADR-0034 §2 D3 L94），**实装推到 Phase F**
- 25 domain-* crate 当前是 **stub**（per [AGENTS.md §7 待办 #7](../../../../AGENTS.md)）

### 1.2 Phase F 范围

Phase F 在 Phase E 基础上接 **4 Git Provider + 22 核心 domain crate + SSE 推送 + Webhook 接收**：

1. **4 Git Provider 真实接入**（per `spec/services/01-service-adapter-spec.md` §1-§3 L17-103 + §6 G-01 已知缺口）：GitHub / GitLab / Bitbucket / **Gitea**（注：实际为 Gitea 而非 Local，per §7 #9 已知缺口）
2. **22 核心 domain crate 数据源契约**（per ADR-0034 §1.1 "25 domain crate 是 stub" + §2 D5 L141）：从 stub 升级到真实数据接入
3. **SSE 推送**（per `spec/services/02-sse-streaming-spec.md` §3 L89-108）：事件路由 + SSE endpoint + heartbeat 30s（L55 SSE comment 格式）
4. **Webhook 接收**（per `spec/services/03-webhook-adapter-spec.md` §2-§5 L65-163）：HMAC-SHA256 签名验证（L88）+ 幂等表（L112-135）+ 死信队列（L179-192）

---

## 2. 决策（5 项 D6-D10）

### D6. 新增 `spec/vcs/05-real-providers-spec.md` — 4 Git Provider 真实接入规范

**理由**：
- 现有 `spec/vcs/01-04` 是**抽象层**（per [spec/services/01 §1 L15](../spec/services/01-service-adapter-spec.md) "SA 是 VCS Provider 适配器"）：`01-version-control-provider.md` / `02-gitgit-provider.md` / `03-github-gitlab-compat.md` / `04-fallback-strategy.md`
- 缺 **4 provider 真实接入的运行规约**（OAuth / rate limit / pagination / error mapping / capability 协商）
- Phase E 派生需求（per [AGENTS.md §7 待办 #4](../../../../AGENTS.md) "16 tool 真实数据源接入"）

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/vcs/05-real-providers-spec.md`
- 章节：
  - §1 4 provider 配置 schema（per `spec/services/01 §3 配置 schema` L103-142 扩展）
  - §2 OAuth / Token 认证流程
  - §3 Rate Limit 处理（per `spec/services/01 §4 健康检查 + 容量管理` L143-162）
  - §4 Pagination 协议（cursor-based, 跨 provider 统一）
  - §5 错误映射（vendor 错误码 → spec/mcp/03 30 标准错误码，per [ADR-0034 §2 D2 L102-118](0034-phase-e-architecture.md)）
  - §6 Capability 协商协议（per `spec/services/01 §1` L17-75 `ServiceCapabilities`）
  - §7 已知缺口（per §7）

### D7. 新增 `spec/agents/02-data-sources-spec.md` — 22 domain crate 数据源契约

**理由**：
- 22 核心 domain crate 当前是 **stub**（per [AGENTS.md §7 #7](../../../../AGENTS.md)）
- Phase E mock data 注释 `// TODO(phase-f): 接入 22 domain crate 真实数据`（per ADR-0034 §2 D5 L146）需被 spec 替换
- 25 候选 = 22 核心 + 3 非核心（domain-collaboration / domain-comment / domain-board 推 Phase F+，per §7 #1）
- Agent 运行时（spec/agents/01）需知道 22 crate 的数据访问契约（trait + 健康检查 + 缓存策略）

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/agents/02-data-sources-spec.md`
- 章节：
  - §1 22 核心 domain crate 列表（来自 `crates/` 目录扫描 2026-08-27 22:45 JST 实际 25 个减去 3 非核心）
  - §2 DataSource trait 定义（`fn fetch` / `fn subscribe` / `fn health` / `fn cache_invalidate`）
  - §3 与 spec/agents/01 §2 Lease 协议（L104-141）的协作（lease 续期触发 cache invalidate）
  - §4 缓存策略（TTL + scope + invalidation，per [spec/mcp/01 §1.1 ④ `ttlMs` + `cacheScope`](../spec/mcp/01-mcp-spec.md)）
  - §5 健康检查 + NFR（per [arch/06 §3 NFR](../arch/06-threat-model-nfr.md)）
  - §6 已知缺口

### D8. 新建 crate `star-sa` — 4 provider 实现 + Provider trait + 8 测试

**理由**：
- `spec/services/01 §1 SA 接口定义`（L17-75）定义 trait，需实装
- 4 provider = GitHub / GitLab / Bitbucket / Gitea（per `spec/services/01 §2 协议转换` L78-93，**非 Local**，per §7 #9）
- 8 测试 = 4 provider × 2（health + capability 协商）
- 与 star-mcp 解耦（per ADR-0034 §3 关系表 L158-160 "spec/services/01 ... 22 domain crate 接入的抽象层"）

**形式**：
- 路径：`crates/star-sa/`
- 文件：
  - `src/lib.rs`（~80 行，Provider trait re-export + crate 入口）
  - `src/provider.rs`（~150 行，trait 定义 + capability 协商）
  - `src/github.rs`（~200 行，GitHub v3 REST + v4 GraphQL partial 适配）
  - `src/gitlab.rs`（~180 行，GitLab v4 API 适配）
  - `src/bitbucket.rs`（~150 行，Bitbucket Cloud + Server 适配，per `spec/services/01 §6 G-01` L190）
  - `src/gitea.rs`（~150 行，Gitea v1 API 适配）
  - `src/health.rs`（~80 行，4 provider 健康检查 endpoint，per `spec/services/01 §4` L143-162）
  - `src/error_mapping.rs`（~100 行，vendor 错误 → 30 标准错误码映射，per [ADR-0034 §2 D2 30 错误码](0034-phase-e-architecture.md)）
  - `tests/integration.rs`（8 测试 = 4 × 2）

### D9. 新建 crate `star-sse` — 事件路由 + SSE endpoint + heartbeat 30s

**理由**：
- `spec/services/02-sse-streaming-spec.md §3 与 MCP Streamable HTTP 边界`（L89-108）未实装
- `spec/services/02 §2 连接管理`（L45-88）定义事件类型 `progress` / `log` / `result` / `error` / `heartbeat`
- heartbeat 30s（per `spec/services/02 §2.2 L55` "WHATWG HTML Living Standard SSE spec, proxy 默认 60s idle timeout 取 2x 倒数"）
- 取代 [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md §5 待办 #2](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) "SSE 流式响应（server-push）未实现"

**形式**：
- 路径：`crates/star-sse/`
- 文件：
  - `src/lib.rs`（~60 行）
  - `src/event.rs`（~120 行，5 Event Type 枚举 + JSON 序列化）
  - `src/router.rs`（~150 行，事件路由 = 22 domain crate event → SSE channel）
  - `src/endpoint.rs`（~180 行，SSE HTTP endpoint，text/event-stream + Bearer token 鉴权，per `spec/services/02 §2.1 L49`）
  - `src/heartbeat.rs`（~80 行，30s SSE comment `:heartbeat\n\n`，per `spec/services/02 §2.2 L56`）
  - `src/replay.rs`（~150 行，Last-Event-ID 续传，per §7 #5 已知缺口，多 node 部署待 Phase G）
  - `tests/event_router.rs`（6 测试 = 5 Event Type × 1 + 心跳 × 1）
  - `tests/replay.rs`（3 测试）

### D10. 新建 crate `star-webhook` — HMAC-SHA256 签名验证 + 幂等表 + 死信队列

**理由**：
- `spec/services/03-webhook-adapter-spec.md §2-§5`（L65-163）定义：§2 签名验证（L65-107）/ §3 事件去重（L108-138）/ §4 事件路由（L139-164）/ §5 失败重试（L165-198）
- HMAC-SHA256 验证（per `spec/services/03 §2.2 L88` `hmac_sha256(secret, body)` 配 hex）
- 幂等键 = `(provider, delivery_id)`（per `spec/services/03 §3.1 L112`）
- 死信队列 `webhook_dead_letter` 表（per `spec/services/03 §5 L179-192`，含 `idx_dl_provider_unresolved` 部分索引）
- Bitbucket Cloud HMAC-SHA1 deprecated 待迁移 SHA256（per `spec/services/03 §6 G-02 L205`，per §7 #7 已知缺口）

**形式**：
- 路径：`crates/star-webhook/`
- 文件：
  - `src/lib.rs`（~80 行）
  - `src/verify.rs`（~150 行，4 provider HMAC-SHA256 验签 + GitLab token 模式，per `spec/services/03 §2 L77-80`）
  - `src/idempotent.rs`（~120 行，`(provider, delivery_id)` 幂等表 + DB 事务）
  - `src/router.rs`（~200 行，vendor 事件 → 内部事件映射，per `spec/services/03 §4` L139-164）
  - `src/retry.rs`（~100 行，指数退避 + 死信队列投递，per `spec/services/03 §5` L165-198）
  - `src/dead_letter.rs`（~120 行，`webhook_dead_letter` 表 CRUD + 部分索引）
  - `src/state.rs`（~80 行，in-memory 持久化，per §7 #6 已知缺口，DB 后端 Phase F+）
  - `tests/verify.rs`（6 测试 = 4 provider × 1 + 篡改测试 + 过期测试）
  - `tests/idempotent.rs`（4 测试）
  - `tests/retry.rs`（3 测试）

---

## 3. 跨 spec/crate 关系表

| 关系 | 上游契约 | 下游实现 | cross-ref |
|---|---|---|---|
| `spec/vcs/05`（D6）↔ `spec/services/01 §1-§3` | spec/services/01 §1 SA 接口（L17-75）+ §2 协议转换（L76-101）+ §3 配置 schema（L103-142） | `crates/star-sa`（D8）4 provider 实现 | spec/services/01 §6 G-01/G-03/G-04 已知缺口（L190-193） |
| `spec/agents/02`（D7）↔ `spec/agents/01 §2 Lease 协议` | spec/agents/01 §2 Lease 协议 L104-141（30s heartbeat / 300s TTL） | `crates/star-mcp/src/agent_runtime.rs` cache invalidate hook | spec/agents/01 §6 已知缺口 L216-228 |
| `crates/star-sa`（D8）↔ `spec/services/01-03` | spec/services/01 全 7 节 + spec/services/03 §2 签名验证 L77-80 | 4 provider 实现 + 8 测试 | spec/services/01 §6 已知缺口 L186-196 |
| `crates/star-sse`（D9）↔ `spec/services/02 §3` | spec/services/02 §3 与 MCP Streamable HTTP 边界 L89-108 | SSE endpoint + heartbeat 30s + 6+3 测试 | [PHASE-D5 §5 待办 #2](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) |
| `crates/star-webhook`（D10）↔ `spec/services/03 §2-§5` | spec/services/03 §2 L65-107 + §3 L108-138 + §4 L139-164 + §5 L165-198 | HMAC + 幂等 + 路由 + 重试 + 死信 + 13 测试 | spec/services/03 §6 G-01/G-02/G-03/G-08 已知缺口 L200-211 |
| 25 domain crate ↔ `spec/agents/02 §1` | spec/agents/02 §1 22 核心 + 3 非核心列表（D7） | `crates/star-mcp/src/agent_runtime.rs` + `crates/star-sa` 适配 | [AGENTS.md §7 #7](../../../../AGENTS.md) "25 domain-* crate 真实数据接入" |

**关键边界**（per [ADR-0034 §3 关键边界](0034-phase-e-architecture.md) 扩展）：
- `spec/vcs/05` 是 **VCS Provider 真实接入规约**（契约层）
- `spec/agents/02` 是 **22 domain 数据源契约**（契约层）
- `crates/star-sa/sse/webhook` 是 **3 个新服务适配器**（实装层）
- `spec/services/01-03` 是 **服务适配器 spec**（已存在，Phase F 升级为实装）
- `arch/03+05+06` 是 **架构总纲**（不变量 + 边界 + NFR，不变）

---

## 4. 5 域独立 Lead 责任矩阵

per 8/21 JST 用户偏好（5 域独立 Lead，不接受兼任）+ 8/27 21:59 JST 第三次强化"你可以代签"：

| # | 域 | 角色 | Lead | Phase F 责任 | 决策范围 |
|---|---|---|---|---|---|
| 1 | 架构 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | ADR 0035 commit + 2 spec 终审 + 3 crate 接口终审 | spec/vcs/05 + spec/agents/02 + 3 crate API |
| 2 | SRE | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | star-sa + star-sse + star-webhook 部署 + SLO + 监控 | 3 crate SLO 定义 + 8+9+13 测试 CI 集成 |
| 3 | 平台 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 3 crate 依赖 + toolchain + workspace | 保 0 新外部依赖（除 wiremock/per §7 #7）+ workspace.toml 同步 |
| 4 | 评审 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | DDD Review 主持 | Phase F 2 spec + 3 crate + 30 测试 DDD Review 主持 + sign-off |
| 5 | PM | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)，5 域独立 Lead，不接受兼任) | 进度跟踪 + 22 domain 接入优先级 + 风险升级 | Phase F 4-6 人·周 OLU 校准 + 22 crate 接入顺序（per ADR-0034 §6 已知缺口 #8 L278） |

**5 域责任矩阵**（per ADR-0034 §4 矩阵 L184-198 扩展）：

| 决策类型 | 架构 | SRE | 平台 | 评审 | PM |
|---|---|---|---|---|---|
| 2 新 spec 终审 | 🟢 签 | 🟡 咨询 | 🟡 咨询 | 🟢 签 | 🟡 知会 |
| 3 新 crate 接口 | 🟢 签 | 🟡 咨询 | 🟢 签（依赖） | 🟢 签 | 🟡 知会 |
| 22 domain 接入顺序 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟢 签 |
| 30 测试 CI 集成 | 🟡 咨询 | 🟢 签 | 🟡 咨询 | 🟢 签 | 🟡 知会 |
| R-05 push 决策 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟡 咨询 | 🟢 签 |

---

## 5. token-OLU 估算

per 8/21 JST token-OLU 框架（1 人·周 ≈ 1M tokens）+ [ADR-0034 §5 token-OLU 估算](0034-phase-e-architecture.md) + §8.1 Phase F 方向 L291-300：

| 阶段 | 范围 | 估算 | 单价依据 |
|---|---|---|---|
| Phase F spec 写作 | 2 新 spec（vcs/05 + agents/02）+ 1 ADR（本文） | 2-3M tokens | 每 spec 0.8-1.2M + ADR 0.4M |
| crates/star-sa 实装 | 4 provider 实现 + Provider trait + 8 测试 | 3-5M tokens | 每 provider 0.6-1.0M + 8 测试 0.3M |
| crates/star-sse 实装 | 事件路由 + SSE endpoint + heartbeat + replay | 2-3M tokens | 6 文件 + 6+3 测试 |
| crates/star-webhook 实装 | HMAC + 幂等 + 路由 + 重试 + 死信 | 3-4M tokens | 7 文件 + 13 测试 |
| 22 核心 domain crate 数据接入 | 替换 stub 为真实数据源 | 25-40M tokens | 每 crate 1-2M（trait 实施 + 集成测试） |
| **Phase F 总计** | — | **35-55M tokens ≈ 4-6 人·周** | vs [ADR-0034 §8.1 L297](0034-phase-e-architecture.md) "30-50M / 38-63M 5-8 人·周"（per PM 校准下调 5-10M） |

**vs ADR-0034 §8.1 估算差异**：
- ADR-0034 §8.1 L297 估 "22 domain crate 真实数据接入 30-50M（每 crate 1-2M）"
- 本文 §5 估 "25-40M"
- 差异：PM 校准 22 crate 接入可分批（先 8 个核心 + 14 个延后），节省 5-10M
- 待 PM 终审确认

---

## 6. 与上游 ADR 引用

- [ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 零厂商合作原则（影响 D6 OAuth 流程不能 vendor 锁定）
- [ADR-0022 IDE Placement](0022-ide-placement.md) — IDE 归 STAR（D9 SSE 推送 + D10 Webhook 接收需 IDE gateway 接入）
- [ADR-0023 Version Control Provider Abstraction](0023-version-control-provider.md) — VCS Core 抽象（spec/vcs/01 基础，spec/vcs/05 扩展）
- [ADR-0026 STAR AI Compatibility](0026-star-ai-compat.md) — STAR AI 5 通道 + Fallback Ladder 4 级（star-sa/sse/webhook 接入通道 2 MCP）
- [ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) — IDE 网关（star-sse endpoint 走通道 1 IDE 集成）
- [ADR-0028 GitGit Compatibility](0028-gitgit-compat.md) — GitGit 100% 标准 Git + REST 12+2 endpoints（spec/vcs/02 基础）
- [ADR-0029 Universal Submit](0029-universal-submit.md) — Universal Submit 12 步 + 6 字段错误模型（spec/services/03 §4 事件路由对接）
- [ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — Lease + Heartbeat + Resume 11 字段（spec/agents/01 §2 基础）
- [ADR-0031 Context Graph](0031-context-graph.md) — Context Graph MVP 4 节点 + 5 关系（spec/agents/02 §1 22 domain 含 context）
- [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) — MCP Transport stdio 16 tools + 6 字段错误模型（star-sa error_mapping 对接）
- [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) — 代签规则（本文 commit author = Ulysses per 21:59 JST 第三次强化）
- [ADR-0034 Phase E Architecture](0034-phase-e-architecture.md) — Phase E 整体架构（本文 §1.1 + §3 关系表 + §5 token-OLU 引用）

---

## 7. 已知缺口

per 8/26 04:30 "缺标比错标安全" + 8/27 21:59 JST Mavis 接手代签（不动 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)的 SRE/平台/评审/PM）：

| # | 缺口 | 影响 | 状态 |
|---|---|---|---|
| 1 | `domain-collaboration` / `domain-comment` / `domain-board` 3 非核心 domain crate 接入待 Phase F+ | 25 候选 = 22 核心 + 3 非核心；Phase F 只接 22 核心 | 显式列出，Phase F+ 排期 |
| 2 | 真实 OAuth 流程未涉及（仅 token 认证） | spec/vcs/05 §2 仅支持 static token + env var；OAuth flow 推 Phase G | spec/vcs/05 §2 标注 |
| 3 | 跨域 Saga 协调待 Phase G | 22 domain crate 跨域事务（如 MR 创建触发 notification + audit）需 saga 协调 | Phase G 方向 |
| 4 | Phase F+ cache layer 性能预算未量化 | 22 crate 接入后 cache hit rate / latency 预算待 SRE Lead 校准 | Phase F+ SRE SLO |
| 5 | SSE 多 node 部署 + Last-Event-ID 跨节点 replay | star-sse/src/replay.rs 仅单 node 内存续传；多 node 需 Redis stream | Phase F+ 部署层 |
| 6 | Webhook 接收端持久化（当前 in-memory） | star-webhook/src/state.rs 用 in-memory 状态；Phase G 换 DB 后端 | Phase F+ DB 接入 |
| 7 | `crates/star-sa` 真实网络测试需要 wiremock | 8 集成测试当前是 mock；wiremock-rs 是 Phase D.5+ 例外依赖，per ADR-0034 §2 D4 | Phase D.5+ 显式反转 |
| 8 | `crates/star-sse` 鉴权 token 短期化 + 刷新机制 | spec/services/02 §2.1 L49 Bearer token 当前无 TTL；需 Phase F+ 加短期 token | Phase F+ |
| 9 | 4 Git Provider 中 Gitea vs Local 待 PM 拍板 | spec/services/01 §1 L23 列 6 provider (含 jira)；§2 L78 表格列 4 (GitHub/GitLab/Bitbucket/自建 Git)；§2 L91 列 4 (含 Gitea)；本文 D6/D8 取 Gitea（per spec/services/01 §2 L91），需 PM 终审 | PM 拍板 |
| 10 | 22 domain crate 接入优先级排期 | per [ADR-0034 §6 已知缺口 #8 L278](0034-phase-e-architecture.md) "22 domain crate 接入 Phase F 优先级排序（哪些先接入）" 仍未决 | PM 拍板 |

---

## 8. 后果

### 8.1 Phase F 交付（per §5 token-OLU 35-55M / 4-6 人·周）
- 2 新 spec（vcs/05 + agents/02）
- 1 新 ADR（本文）
- 3 新 crate（star-sa / star-sse / star-webhook）
- 22 核心 domain crate 从 stub 升级到真实数据
- 30+ 新测试（8 + 9 + 13 = 30 + 22 × 1 集成 = 52 总测试）
- 0 新外部依赖（除 wiremock-rs Phase D.5+ 例外）

### 8.2 Phase G 方向
- **data layer 优化**：3 非核心 domain crate（collaboration/comment/board）接入
- **缓存层**：Redis stream 支撑 star-sse 多 node + star-webhook 持久化
- **性能预算**：5 域 NFR 收敛（SRE Lead 校准）
- **跨域 Saga 协调**：22 domain 跨域事务
- **OAuth 流程**：spec/vcs/05 §2 升级

### 8.3 Phase E → Phase F 不变量
- 守门 0 unsafe / 0 新外部依赖（除 wiremock-rs 例外）/ R-05 不 push
- bc23d6c 保留 / 5 域独立 Lead（拒绝兼任）
- token-OLU 框架（1 人·周 ≈ 1M tokens）
- 环境变量安全（per 8/27 11:06 JST hard ban）
- 代签规则应用（author = Ulysses，审批 = 架构师（Mavis 接手 agent per DEC-008））
- 缺标比错标安全（§7 已知缺口 10 项显式列）

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟢 Mavis 接手代签（per 2026-08-27 19:39 JST + 20:56 JST + 21:59 JST 用户授权三次强化 + 8/27 07:16 JST 代签规则反转授权）；本文 5 决策 D6-D10 + 2 新 spec + 3 新 crate 终审 |
| 2 | SRE Lead | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead（拒绝兼任 per 8/21 JST 硬约束），真实身份签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持 | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补 |
| 5 | PM | (🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)) | — | — 5 域独立 Lead，真实身份签字请 DDD Review 阶段补；§5 token-OLU 35-55M / §7 #9 Gitea vs Local / §7 #10 22 domain 优先级待 PM 终审 |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签（per 19:39/20:56/21:59 JST 三次强化） | 初版：5 决策 D6-D10（spec/vcs/05 + spec/agents/02 + star-sa + star-sse + star-webhook）+ 5 域 Lead 责任矩阵 + token-OLU 35-55M 估算 + 与 12 上游 ADR 引用 + 10 项已知缺口 + Phase G 方向 | 2026-08-27 22:45 JST 用户派工"新建 1 份 ADR 0035 Phase F 整体架构"，per 8/27 21:59 JST 第三次强化"继续, 你可以代签" |

---

## 11. 引用文档

- [adr/0034-phase-e-architecture.md](0034-phase-e-architecture.md) — Phase E 整体架构（base 938e9ab 引用 §1.1/§2/§3/§4/§5/§6/§8.1）
- [adr/0033-agent-co-signing-policy.md](0033-agent-co-signing-policy.md) — 代签规则反转 + 19:39 JST 升级 + 21:59 JST 第三次强化
- [spec/services/01-service-adapter-spec.md](../spec/services/01-service-adapter-spec.md) — SA 抽象层（§1-§3 trait + §6 G-01/G-03/G-04 缺口）
- [spec/services/02-sse-streaming-spec.md](../spec/services/02-sse-streaming-spec.md) — SSE 流式（§2 heartbeat 30s + §3 MCP 边界）
- [spec/services/03-webhook-adapter-spec.md](../spec/services/03-webhook-adapter-spec.md) — Webhook 适配（§2 HMAC + §3 幂等 + §4 路由 + §5 死信 + §6 G-02 Bitbucket 迁移）
- [spec/agents/01-agent-runtime-spec.md](../spec/agents/01-agent-runtime-spec.md) — Agent 运行时（§2 Lease 协议 30s heartbeat / 300s TTL）
- [AGENTS.md §0 一句话硬约束 + §4 守门硬约束 + §7 待办清单](../../../../AGENTS.md)
- [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md §5 待办 #2](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) — SSE 未实装（star-sse 取代）
- [PHASE-D3-MCP-TRANSPORT-REPORT.md §2 6 字段错误模型](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) — 错误模型基础（star-sa error_mapping 升级到 30 错误码）
