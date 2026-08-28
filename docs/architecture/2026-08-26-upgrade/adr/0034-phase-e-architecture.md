# ADR-0034: Phase E 整体架构决策 (Spec 增量 + star-mcp 实装)

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-27
> **制定人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手（per 2026-08-27 19:39 JST 用户授权升级）
> **签批**：🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（per §7 签字栏；Mavis 接手可代签 per ADR-0033 §2.1）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../../plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) · [ADR-0026 STAR AI Compat](0026-star-ai-compat.md) · [ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) · [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) · [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) · [AGENTS.md §0 一句话硬约束](../../../../AGENTS.md)
> **关联**：[arch/03 STAR AI Compat Arch](../arch/03-star-ai-compat-arch.md) · [arch/05 GitGit Compat Arch](../arch/05-gitgit-compat-arch.md) · [arch/06 Threat Model + NFR](../arch/06-threat-model-nfr.md) · [spec/mcp/01-mcp-spec.md](../spec/mcp/01-mcp-spec.md) · [PHASE-D2-CLI-IMPL-REPORT.md](../../../../PHASE-D2-CLI-IMPL-REPORT.md) · [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) · [PHASE-D4-P1-FIX-REPORT.md](../../../../PHASE-D4-P1-FIX-REPORT.md) · [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md)

---

## 0. 一句话硬约束

> **可以代签 Ulysses，不可以编造历史。**
> —— per AGENTS.md §0（2026-08-27 19:39 JST 用户授权升级）

---

## 1. 背景

### 1.1 Phase D 已交付（per 2026-08-27 19:36 JST 现状）

| 阶段 | 交付 | 报告 | 关键 commit |
|---|---|---|---|
| Phase D.2 | `star-cli` 从 stub 升级到实装（worktree / mr / submit / lease / heartbeat） | [PHASE-D2-CLI-IMPL-REPORT.md](../../../../PHASE-D2-CLI-IMPL-REPORT.md) | `8a7427d` (merge 1274725) |
| Phase D.3 | `star-mcp` Transport stdio（16 tools + 6 字段错误模型） | [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) | `0a148b8` (merge) |
| Phase D.4 | 18 Blocker 修复（P1-1 `--json` global / P1-2 mr named args / F-08 F-14 命名约定 / 等等 12 文件） | [PHASE-D4-P1-FIX-REPORT.md](../../../../PHASE-D4-P1-FIX-REPORT.md) | `2a0a68c` |
| Phase D.5+ | `star-mcp` Transport Streamable HTTP + Resources/Prompts **占位** + 5 通道 Fallback Ladder | [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) | `2857e6b` (merge d0ed6d8 → 6624417) |
| 治理 | `AGENTS.md`（188 行） + ADR-0033 代签反转（194 行） | (ADR 0033 §3.1) | `901033a` |

**5 通道 + Fallback Ladder 4 级**（per [arch/03 STAR AI Compat Arch](../arch/03-star-ai-compat-arch.md) + [ADR-0026 STAR AI Compat](0026-star-ai-compat.md)）：
- 通道 1: IDE 集成（per ADR-0027 STAR IDE Gateway）
- 通道 2: MCP Transport stdio + Streamable HTTP（per ADR-0032 + Phase D.5+）
- 通道 3: CLI 直接调用（per Phase D.2 star-cli）
- 通道 4: GitGit REST API（per ADR-0028 GitGit Compat + arch/05）
- 通道 5: VCS Core Git（per ADR-0023）
- Fallback Ladder 4 级：5 通道失败后降级到人类 git 命令（per Phase D.5+）

### 1.2 Phase E 在 Phase D 基础上补的 5 项

1. **spec 域增量**：新建 `spec/agents/01-agent-runtime-spec.md`（合并 flows/01+02+03 为单契约）+ `spec/mcp/02-resources-prompts-spec.md` + `spec/mcp/03-error-model-spec.md` + `spec/services/01-service-adapter-spec.md` + `spec/services/02-sse-streaming-spec.md` + `spec/services/03-webhook-adapter-spec.md`（共 6 份 spec）—— 已在其他 worktree commit（per 2026-08-27 19:50 JST 派工），**本 worktree 不 fetch**
2. **真实 Resources + Prompts**：把 Phase D.5+ 的 Resources/Prompts **占位** 升级为 **4 类 Resources + 5 个 Prompts 实装**
3. **完整错误模型**：把 Phase D.3 的 **6 字段错误模型** 升级到 **30 标准错误码**（per `spec/mcp/03-error-model-spec.md`）
4. **服务适配器抽象**：把 `star-mcp` 中 16 tools 的 VCS 依赖抽到 `service-adapter` 层（per `spec/services/01-service-adapter-spec.md`）
5. **SSE + Webhook**：新增 `spec/services/02-sse-streaming-spec.md`（Server-Sent Events 流式响应）+ `spec/services/03-webhook-adapter-spec.md`（出站 Webhook 适配）

**关键不变量**（per 8/27 19:36 JST 用户决策）：
- 3 份 spec 已在其他 worktree commit —— **本 worktree 不 fetch，直接基于 ADR 0033 + ADR 0026/0027/0032 + spec/mcp/01 + arch/03/05/06 写新 ADR**
- Phase E 用 **mock data + TODO 标记**（per §2 D5），真实数据源接入推到 Phase F

---

## 2. 决策

### D1. 新增 `spec/agents/01-agent-runtime-spec.md`（合并 flows/01+02+03 为单契约）

**理由**：
- Phase D 在 `spec/flows/01-flow-mr-lifecycle.md` + `02-flow-lease-heartbeat.md` + `03-flow-context-graph.md` 散落 3 份 spec，但 **agent 运行时契约**是单一抽象（包含 lease + heartbeat + resume + context graph），3 份分散 spec 引发**契约边界模糊**（per INTERFACE-REVIEW-A 🟡 #5）
- 合并后 agent runtime spec 是 1 份契约，对外（IDE / CLI / MCP）暴露 1 个 API surface，内部由 3 个子模块实现
- cross-ref 保留 `spec/flows/01+02+03` 作为**实现参考**，但**不是契约源**

**形式**：
- 文件路径：`docs/architecture/2026-08-26-upgrade/spec/agents/01-agent-runtime-spec.md`
- 章节：§1 状态 / §2 Agent 抽象 / §3 Lease 子协议 / §4 Heartbeat 子协议 / §5 Resume 状态机 / §6 Context Graph / §7 错误模型 / §8 与 MCP/CLI/VCS 边界
- 引用 [ADR-0030 Agent Lease + Heartbeat + Resume](0030-agent-lease-heartbeat-resume.md) + [ADR-0031 Context Graph](0031-context-graph.md) + [arch/06 §3 NFR](../arch/06-threat-model-nfr.md)

### D2. 新增 `spec/mcp/02-resources-prompts-spec.md` + `03-error-model-spec.md`（MVP 必实现）

**理由**：
- `02-resources-prompts-spec.md`：Phase D.5+ 仅在 [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) 写了 Resources/Prompts **占位骨架**（无实际 URI Template / Prompt Template），需要 1 份正式 spec 落地
- `03-error-model-spec.md`：Phase D.3 仅 6 字段错误模型（per [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) §2 错误模型），30 标准错误码（含 6 字段 × 5 严重度 × 4 类 = 30 矩阵）需要正式 spec
- 两者都是 **MVP 必实现**（不进 MVP 退出条件）

**形式**：
- `02-resources-prompts-spec.md`：
  - §1 Resources 4 类：`git://worktree/{id}` / `git://commit/{sha}` / `vcs://merge-request/{id}` / `vcs://branch/{name}`
  - §2 Prompts 5 个：`mr_review_template` / `lease_renew_template` / `conflict_resolve_template` / `context_summary_template` / `submit_message_template`
  - §3 URI Template 规范（RFC 6570）
  - §4 缓存策略（per spec/mcp/01 §1.1 ④ `ttlMs` + `cacheScope`）
- `03-error-model-spec.md`：
  - §1 6 字段错误模型（`code` / `message` / `severity` / `retryable` / `details` / `timestamp`）
  - §2 5 严重度（`info` / `warning` / `error` / `critical` / `fatal`）
  - §3 4 类错误（`validation` / `auth` / `upstream` / `internal`）
  - §4 30 标准错误码（5×4 = 20 + 10 业务错误 = 30）
  - §5 错误响应 JSON Schema

### D3. 新增 `spec/services/01-service-adapter-spec.md` + `02-sse-streaming-spec.md` + `03-webhook-adapter-spec.md`（不进 MVP 退出条件）

**理由**：
- `01-service-adapter-spec.md`：star-mcp 16 tools 当前直接调 VCS Core（GitGit REST + git CLI），缺乏**服务适配器抽象层**（per [arch/05 §2 GitGit Compat Arch](../arch/05-gitgit-compat-arch.md) 仅定义了 VCS 边界，未定义 service-adapter 抽象）
- `02-sse-streaming-spec.md`：Phase D.5+ Streamable HTTP 实现了**入站** HTTP 请求，但 **SSE 流式响应**（server-push）未实现（per [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) §5 待办 #2）
- `03-webhook-adapter-spec.md`：出站 Webhook（MR 创建 / Lease 过期 / 冲突检测）**未实现**（per AGENTS.md §7 待办 #4 16 tool 真实数据源接入 的派生需求）
- 三者**不进 MVP 退出条件**（per §3 D3 行），仅做 spec + 骨架，**实装推到 Phase F**

**形式**：
- `01-service-adapter-spec.md`：
  - §1 ServiceAdapter trait 定义（`fn execute` / `fn health` / `fn capabilities`）
  - §2 GitGitServiceAdapter / GitServiceAdapter / MockServiceAdapter 3 实现
  - §3 capability 协商协议
  - §4 health check 协议
- `02-sse-streaming-spec.md`：
  - §1 SSE 协议（text/event-stream，per HTML5 spec）
  - §2 Event Type 枚举（`progress` / `log` / `result` / `error` / `heartbeat`）
  - §3 Last-Event-ID 续传（per MCP 2026-07-28 关键变更 ⑦ 未实现项）
  - §4 心跳策略（15s keepalive）
- `03-webhook-adapter-spec.md`：
  - §1 出站 Webhook 触发源（MR 创建 / Lease 过期 / 冲突 / Submit 完成 / Context Graph 变更）
  - §2 Webhook Payload 格式（JSON Schema）
  - §3 重试策略（指数退避 + dead-letter queue）
  - §4 签名验证（HMAC-SHA256）

### D4. star-mcp 实装 Resources（4 类）+ Prompts（5 个）+ 6 字段错误模型（30 标准错误码）

**理由**：
- 4 类 Resources + 5 个 Prompts + 30 错误码 = Phase E star-mcp 实装的 **3 个核心交付**
- 4 文件（`resources.rs` / `prompts.rs` / `error.rs`（扩展）/ `error_codes.rs`）+ 12 测试（4 类 × 2 测试 + 5 个 × 1 测试 + 30 错误码 × 0.1 = 12）

**形式**：
- `crates/star-mcp/src/resources.rs`（+200 行）：
  - 4 个 `ResourceProvider` 实现：`WorktreeResourceProvider` / `CommitResourceProvider` / `MergeRequestResourceProvider` / `BranchResourceProvider`
  - 每个 Provider 实现 `list` / `read` / `subscribe` 3 个方法
- `crates/star-mcp/src/prompts.rs`（+150 行）：
  - 5 个 `PromptTemplate` 实现
  - 每个 Template 实现 `name` / `description` / `arguments` / `render` 4 个方法
- `crates/star-mcp/src/error.rs`（扩展 +50 行，从 6 字段升级到 6 字段 + 5 严重度 + 4 类）：
  - 保留 Phase D.3 的 6 字段（`code` / `message` / `severity` / `retryable` / `details` / `timestamp`）
  - 新增 `severity` 枚举（5 值）+ `category` 字段（4 类）
- `crates/star-mcp/src/error_codes.rs`（+100 行，30 错误码表）：
  - 5 严重度 × 4 类 = 20 矩阵 + 10 业务错误（MR_NOT_FOUND / LEASE_EXPIRED / CONFLICT_DETECTED / etc.）
- 测试（12 个）：
  - `tests/test_resources.rs`：4 类各 2 测试（list + read）= 8
  - `tests/test_prompts.rs`：5 个各 1 测试（render）= 5（合并到 4 测试）
  - `tests/test_error_codes.rs`：30 错误码 × 1/3 = 10 测试（合并到 12 总数）

### D5. Phase F 接入真实数据源（22 domain crate），Phase E 用 mock data + TODO 标记

**理由**：
- Phase D 25 domain crate（per `crates/` 目录扫描 2026-08-27 19:50 JST）是 **stub**（per AGENTS.md §7 待办 #7）
- Phase E 目标**不是**接入真实数据源，而是**完成 spec + 错误模型 + Resources/Prompts 骨架**
- Phase F 才推 22 domain crate 真实数据接入（per §5 token-OLU 估算）
- Phase E 用 mock data + TODO 标记（**显式列已知缺口** per §7）

**形式**：
- `crates/star-mcp/src/mock/` 新建目录，6 文件（4 Resources + 5 Prompts 各 1 mock provider）
- 每个 mock provider 加 `// TODO(phase-f): 接入 22 domain crate 真实数据` 注释
- mock data 用 `serde_json::json!` 内联，不引入新外部依赖（per 守门 0 unsafe / 0 新外部依赖）

---

## 3. 跨 spec 关系表

| 新 spec | 上游契约 | 下游实现 | cross-ref |
|---|---|---|---|
| `spec/agents/01-agent-runtime-spec.md`（D1） | ADR-0030 + ADR-0031 + arch/06 §3 NFR | `crates/star-mcp/src/agent_runtime.rs`（Phase E 新增） + `crates/star-cli/src/agent/` | spec/flows/01+02+03 实现参考（不是契约源） |
| `spec/mcp/02-resources-prompts-spec.md`（D2） | spec/mcp/01 §1.1 ④（缓存）+ MCP 2026-07-28 §Resources/Prompts | `crates/star-mcp/src/resources.rs` + `prompts.rs`（D4） | [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) §占位骨架 |
| `spec/mcp/03-error-model-spec.md`（D2） | spec/mcp/01 §错误模型 + spec/agent-api/01-schema.md §3.15 | `crates/star-mcp/src/error.rs`（扩展）+ `error_codes.rs`（D4） | [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) §2 6 字段错误模型 |
| `spec/services/01-service-adapter-spec.md`（D3） | arch/05 §2 GitGit Compat Arch + ADR-0023 VCS Core | Phase F 接入（不进 MVP 退出条件） | arch/05 §2（已有 VCS 边界定义） |
| `spec/services/02-sse-streaming-spec.md`（D3） | spec/mcp/01 §1.1 + MCP 2026-07-28 §Streamable HTTP §SSE | Phase F 接入（不进 MVP 退出条件） | [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) §5 待办 #2 |
| `spec/services/03-webhook-adapter-spec.md`（D3） | arch/05 §4 GitGit Webhook + ADR-0028 GitGit Compat | Phase F 接入（不进 MVP 退出条件） | AGENTS.md §7 待办 #4 派生需求 |
| `crates/star-mcp/src/agent_runtime.rs`（D1 + D4 整合） | spec/agents/01 + spec/mcp/02 + 03 | IDE / CLI / MCP 入口 | arch/03 §3 5 通道 + ADR-0026 STAR AI Compat |
| `agent-api/v1`（Phase D 已存在） | spec/agent-api/01-schema.md | star-mcp + star-cli + star-ide 客户端 | spec/mcp/01 §2 16 tools |

**关键边界**：
- `spec/agents/01` 是 **契约**（面向 IDE/CLI/MCP 客户端）
- `spec/mcp/02+03` 是 **MCP 实现 spec**（仅 star-mcp 仓）
- `spec/services/01+02+03` 是 **服务适配器 spec**（22 domain crate 接入的抽象层）
- `arch/03+05+06` 是 **架构总纲**（不变量 + 边界 + NFR）
- `agent-api/v1` 是 **JSON-RPC 协议**（跨进程）

---

## 4. 5 域独立 Lead 映射（per 8/21 JST 用户偏好）

per 5 域独立 Lead，不接受兼任（per AGENTS.md §4 #3 守门 + [RGS-TS-001](https://github.com/UlyssesLeoLee/RustGameServer) 8/21 JST 用户拍板）：

| # | 域 | 角色 | Lead | 决策范围 | Phase E 责任 |
|---|---|---|---|---|---|
| 1 | 架构域 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 6 份 spec 终审 + ADR 0034 终审 | spec/agents/01 + spec/mcp/02+03 + spec/services/01+02+03 6 文件终审 + ADR 0034 commit 签字 |
| 2 | SRE 域 | SRE Lead | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任） | star-mcp 部署 + 监控 + SLA | star-mcp 4 文件（resources.rs / prompts.rs / error.rs / error_codes.rs）SLO 定义 + 12 测试 CI 集成 |
| 3 | 平台域 | 平台工程师 | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任） | crate 依赖 + toolchain + workspace | crates/star-mcp 依赖升级（保留 0 新外部依赖 per 守门）+ workspace.toml 同步 |
| 4 | 评审域 | 评审主持 | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任） | DDD Review 主持 | Phase E 6 spec + 4 文件 + 12 测试 DDD Review 主持 + sign-off 表 |
| 5 | PM 域 | PM | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任） | 进度跟踪 + 风险升级 + R-05 决策 | Phase E 进度跟踪（1-2 人·周）+ Phase F 22 domain crate 接入计划 |

**5 域责任矩阵**（per 8/21 JST 用户偏好 + ADR-0026 + ADR-0027 + ADR-0032）：

| 决策类型 | 架构 | SRE | 平台 | 评审 | PM |
|---|---|---|---|---|---|
| spec 终审 | 🟢 主 | 🟡 咨询 | 🟡 咨询 | 🟢 签字 | 🟡 知会 |
| 实装 commit | 🟢 终审 | 🟡 部署审核 | 🟢 依赖决策 | 🟡 DDD Review | 🟡 进度 |
| 错误模型 | 🟢 主 | 🟡 SLO | 🟡 监控集成 | 🟢 sign-off | 🟡 风险 |
| 跨域冲突（如 Webhook 触发源 = MR 创建） | 🟢 架构边界 | 🟢 SLA 承诺 | 🟡 工具链 | 🟡 Review | 🟡 风险升级 |

---

## 5. token-OLU 估算（per 8/21 JST token-OLU 框架，1 人·周 ≈ 1M tokens）

per AGENTS.md §4 #4 守门（AI 协作 token-OLU 而非人天）+ [RGS-TS-001](https://github.com/UlyssesLeoLee/RustGameServer) §6.2 token-OLU 框架：

### 5.1 Phase E spec 写作

| 任务 | 估算 tokens | 说明 |
|---|---|---|
| `spec/agents/01-agent-runtime-spec.md`（合并 3 flows） | 0.8-1.2M | 8 章节 + cross-ref 3 flows + ADR-0030/0031 引用 |
| `spec/mcp/02-resources-prompts-spec.md` | 0.5-0.8M | 4 类 Resources + 5 Prompts + URI Template + 缓存策略 |
| `spec/mcp/03-error-model-spec.md` | 0.4-0.6M | 6 字段 + 5 严重度 + 4 类 + 30 错误码矩阵 + JSON Schema |
| `spec/services/01-service-adapter-spec.md` | 0.3-0.5M | trait 定义 + 3 实现 + capability 协商 + health check |
| `spec/services/02-sse-streaming-spec.md` | 0.3-0.5M | SSE 协议 + Event Type + Last-Event-ID + 心跳 |
| `spec/services/03-webhook-adapter-spec.md` | 0.3-0.5M | 触发源 + Payload + 重试 + 签名 |
| ADR 0034（本文件） | 0.4-0.6M | 9 章节 + 5 决策 + 跨 spec 关系表 + 5 域映射 + token-OLU |
| **小计** | **3.0-4.7M** | 6 spec + 1 ADR |

### 5.2 Phase E star-mcp 实装

| 任务 | 估算 tokens | 说明 |
|---|---|---|
| `crates/star-mcp/src/resources.rs`（+200 行） | 1.0-1.5M | 4 个 ResourceProvider 实现 + mock data + TODO 标记 |
| `crates/star-mcp/src/prompts.rs`（+150 行） | 0.7-1.0M | 5 个 PromptTemplate 实现 + render 逻辑 |
| `crates/star-mcp/src/error.rs`（扩展 +50 行） | 0.3-0.5M | 6 字段 + 5 严重度 + 4 类 |
| `crates/star-mcp/src/error_codes.rs`（+100 行） | 0.5-0.8M | 30 错误码表 + JSON Schema |
| `crates/star-mcp/src/mock/`（6 文件） | 0.5-0.8M | 4 Resources + 5 Prompts mock provider |
| 12 测试（4 Resources × 2 + 5 Prompts × 1 + 30 错误码 × 0.1 = 12） | 1.0-1.5M | list/read/render/序列化测试 |
| 集成 + 守门验证 | 0.5-0.8M | cargo test + clippy + 0 unsafe + 0 新外部依赖 |
| **小计** | **4.0-6.9M** | 4 文件 + 12 测试 + 集成 |

### 5.3 总计

| 项 | tokens | 人·周 |
|---|---|---|
| Phase E spec 写作 | 3.0-4.7M | 0.4-0.6 |
| Phase E star-mcp 实装 | 4.0-6.9M | 0.5-0.8 |
| **总计** | **7.0-11.6M ≈ 8-13M** | **0.9-1.4 ≈ 1-2 人·周** |

**对比 Phase D**（per [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) §守门）：
- Phase D.5+ ≈ 1 人·周 ≈ 1M tokens（含 Streamable HTTP + Resources/Prompts 占位）
- Phase E ≈ 1-2 人·周 ≈ 8-13M tokens（**8-13 倍 Phase D.5+**，因为加了 6 spec + 完整 Resources/Prompts + 30 错误码 + mock 6 文件 + 12 测试）

**OLU 守门**（per AGENTS.md §4 #4 + RGS-TS-001 §6.2）：
- 1 SRE 上限 = 1 人·周 ≈ 1M tokens
- Phase E 8-13M tokens = 1 个架构师 + 1 个 SRE 协作 1-2 周
- 不需要申请额外 SRE 编制（per AGENTS.md §4 #3 5 域独立 Lead，架构 + SRE 已是 2 域，不需扩展）

---

## 6. 与上游 ADR 引用

| 上游 ADR | 引用方式 | 应用于 Phase E |
|---|---|---|
| [ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) | 全局不变量 | Phase E 6 spec 继续不引入厂商适配器（per D1+D2+D3 决策） |
| [ADR-0022 IDE Placement](0022-ide-placement.md) | 边界 | IDE 归 STAR（不归 MCP），star-mcp 仅 IDE 后端 |
| [ADR-0023 VCS Core = GitGit](0023-version-control-provider.md) | 上游契约 | star-mcp 16 tools 上游 = GitGit REST（per arch/05） |
| [ADR-0024 IDE Session Identity](0024-ide-session-identity.md) | session 边界 | Phase E `spec/agents/01` 引用 session identity 字段 |
| [ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md) | 反污染 | Phase E `spec/services/01` ServiceAdapter trait 隔离厂商适配 |
| [ADR-0026 STAR AI Compat](0026-star-ai-compat.md) | 5 通道 + Fallback Ladder | Phase E 6 spec 落 5 通道契约 |
| [ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) | 3 通道 + Gateway 责任矩阵 | Phase E `spec/agents/01` 引用 Gateway 边界 |
| [ADR-0028 GitGit Compat](0028-gitgit-compat.md) | REST 12+2 endpoints | Phase E `spec/mcp/03` 错误码 + `spec/services/03` Webhook 用 GitGit 12+2 |
| [ADR-0029 Universal Submit](0029-universal-submit.md) | 12 步 + 6 字段错误 | Phase E `spec/agents/01` 引用 Universal Submit 子流程 |
| [ADR-0030 Agent Lease + Heartbeat + Resume](0030-agent-lease-heartbeat-resume.md) | 11 字段 + 跨 Agent Handoff | Phase E `spec/agents/01` 合并 Lease + Heartbeat + Resume 为单契约 |
| [ADR-0031 Context Graph](0031-context-graph.md) | MVP 4 节点 + 5 关系 | Phase E `spec/agents/01` §6 Context Graph 引用 |
| [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) | 16 tools + 6 字段错误 | Phase E `spec/mcp/03` 升级 6 字段到 30 错误码 |
| [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) | 代签规则 | 本 ADR commit author = Ulysses (per 19:39 JST 升级) |

---

## 7. 已知缺口（per 8/26 04:30 缺标比错标安全规则）

**显式列未确定项**（per AGENTS.md §4 #11 守门 + DDD Review 必查）：

| # | 缺口 | 影响 | 决策时点 |
|---|---|---|---|
| 1 | 6 份 spec 已在其他 worktree commit，**本 worktree 不 fetch** | 跨 spec 引用用 cross-ref，不引用具体行（per 任务说明） | 2026-08-27 19:50 JST（已派工） |
| 2 | `spec/agents/01` 合并 flows/01+02+03，**3 份 flows 保留** 还是 **删除**？ | 若保留 = cross-ref 双向维护成本；若删除 = 历史丢失 | Phase E DDD Review 阶段决策 |
| 3 | 30 错误码与 6 字段错误模型**向后兼容**：Phase D.3 已实装的 6 字段是否要保留？ | 若保留 = 旧客户端继续工作；若移除 = 强制升级 | Phase E 6 spec 终审决策 |
| 4 | `spec/services/01` ServiceAdapter trait **mock 6 文件** 在 `crates/star-mcp/src/mock/` 还是 **独立 crate** `crates/star-mock/`？ | 若同仓 = 编译时必编译；若独立 = 测试时可关闭 | Phase E 平台域决策 |
| 5 | SSE `Last-Event-ID` 续传是否在 Phase E 实装？**当前规划推到 Phase F** | 若 Phase E 实装 = +0.5M tokens；若推 Phase F = Phase F OLU 上升 | Phase E 评审域决策 |
| 6 | Webhook 签名验证 `HMAC-SHA256` 是否需要 **per-tenant 密钥** 还是 **全局密钥**？ | 若 per-tenant = 复杂密钥管理；若全局 = 简化 | Phase E 架构域决策（建议 per-tenant per arch/06 §3 NFR-OP-015） |
| 7 | `spec/agents/01` 是否要包含 **tool approval** 子协议？ | 若包含 = 5 子协议；若不包含 = 4 子协议 | Phase E 评审域决策（建议推迟到 Phase F） |
| 8 | 22 domain crate 接入 Phase F **优先级排序**（哪些先接入）？ | 影响 Phase F OLU 估算 | Phase F 启动时决策（不在本 ADR 范围） |
| 9 | 12 测试是否覆盖 **e2e MCP stdio + Streamable HTTP**？ | 若仅单元测试 = 集成风险；若 e2e = +1M tokens | Phase E 评审域决策（建议先单元 + 集成，e2e 推 Phase F） |
| 10 | `agent-api/v1` 协议是否需要扩展支持 **Resources/Prompts**？ | 若扩展 = 协议版本 bump 到 v1.1；若不扩展 = star-mcp 内部实现 | Phase E 架构域决策（建议不扩展，保持 v1 稳定） |

**守门约束**（per 8/26 04:30）：
- 缺标比错标安全 —— 列在 §7 即视为已暴露风险，DDD Review 必查
- 禁止"per X 历史形态"等回溯叙事（本 ADR 不引用未 git 实证的 DTL/BAS）
- BAS 引用必须 `git log -p --follow` 实证（本 ADR 不引用 BAS，仅引用 ADR/spec/arch/PHASE 报告）

---

## 8. 后果（Phase E 完成后 Phase F 的方向）

### 8.1 Phase F 接入真实数据源（22 domain crate）

per [arch/05 §2 GitGit Compat Arch](../arch/05-gitgit-compat-arch.md) + AGENTS.md §7 待办 #7：

| 任务 | 估算 tokens | 说明 |
|---|---|---|
| 22 domain crate 真实数据接入 | 30-50M | 每 crate 1-2M（含 trait 实现 + 集成测试） |
| SSE + Webhook 实装 | 5-8M | per `spec/services/02+03`（Phase E 仅 spec） |
| e2e MCP stdio + Streamable HTTP | 3-5M | 12 测试补 e2e 覆盖 |
| **Phase F 总计** | **38-63M ≈ 5-8 人·周** | 需 1 架构 + 2-3 SRE 协作 5-8 周 |

### 8.2 Phase F 风险

- **22 domain crate 接入** 跨域冲突风险（per 5 域独立 Lead 守门）—— 需 PM 域风险升级
- **SSE + Webhook** 引入新外部依赖（SSE library + Webhook 客户端），守门 0 新外部依赖 需反转（per Phase D.5+ 例外，显式反转）
- **e2e 测试** 引入 CI runner 资源开销（SRE 域 SLO 决策）

### 8.3 Phase E → Phase F 触发条件

- Phase E 6 spec 终审通过（架构 + 评审签字）
- Phase E 4 文件 + 12 测试 CI 全绿
- Phase E ADR 0034 DDD Review sign-off
- Phase F 22 domain crate 接入计划 PM 域批准

---

## 9. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-27 | 🟡 Draft v0.1（待 §10 6 spec 终审 + 评审域 DDD Review） |
| 2 | SRE Lead | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任 per 8/21 JST） | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) |
| 3 | 平台工程师 | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任 per 8/21 JST） | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) |
| 4 | 评审主持 | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任 per 8/21 JST） | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) |
| 5 | 项目负责人（PM） | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5)（5 域独立 Lead，不接受兼任 per 8/21 JST） | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) | 🟢 Mavis 接手代签 (per 8/27 19:39/21:59 JST 三次强化 + 12-domain-lead-roster §5) |

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手（per 19:39 JST 用户授权升级） | 初版：Phase E 整体架构决策（5 项 D1-D5）+ 跨 spec 关系表 + 5 域独立 Lead 映射 + token-OLU 估算 8-13M tokens + 与上游 ADR 0021-0033 引用 + 已知缺口 10 项 + Phase F 方向 | 2026-08-27 19:50 JST 派工新建 ADR 0034 |

---

## 11. 引用文档

- [AGENTS.md §0 一句话硬约束](../../../../AGENTS.md) — 代签授权升级
- [AGENTS.md §4 #3 5 域独立 Lead](../../../../AGENTS.md) — 5 域责任矩阵
- [AGENTS.md §4 #4 token-OLU](../../../../AGENTS.md) — AI 协作 token-OLU 框架
- [AGENTS.md §4 #11 缺标比错标](../../../../AGENTS.md) — §7 已知缺口硬约束
- [ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 全局不变量
- [ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — 5 通道 + Fallback Ladder
- [ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) — 3 通道 + Gateway 责任
- [ADR-0030 Agent Lease + Heartbeat + Resume](0030-agent-lease-heartbeat-resume.md) — 11 字段
- [ADR-0031 Context Graph](0031-context-graph.md) — 4 节点 + 5 关系
- [ADR-0032 MCP Transport stdio](0032-mcp-transport-stdio.md) — 16 tools + 6 字段错误
- [ADR-0033 Agent Co-Signing Policy](0033-agent-co-signing-policy.md) — 代签规则
- [arch/03 STAR AI Compat Arch](../arch/03-star-ai-compat-arch.md) — 5 通道架构
- [arch/05 GitGit Compat Arch](../arch/05-gitgit-compat-arch.md) — VCS 边界
- [arch/06 Threat Model + NFR](../arch/06-threat-model-nfr.md) — §3 NFR
- [spec/mcp/01-mcp-spec.md](../spec/mcp/01-mcp-spec.md) — MCP MVP 规范
- [PHASE-D2-CLI-IMPL-REPORT.md](../../../../PHASE-D2-CLI-IMPL-REPORT.md) — Phase D.2 CLI
- [PHASE-D3-MCP-TRANSPORT-REPORT.md](../../../../PHASE-D3-MCP-TRANSPORT-REPORT.md) — Phase D.3 MCP stdio
- [PHASE-D4-P1-FIX-REPORT.md](../../../../PHASE-D4-P1-FIX-REPORT.md) — Phase D.4 18 Blocker 修复
- [PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md](../../../../PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md) — Phase D.5+ Streamable HTTP + Resources/Prompts 占位
