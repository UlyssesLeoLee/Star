# 06. Threat Model & Non-functional Requirements

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **依赖**：[ADR-0021 ~ 0025](../adr/) · [Protocol Survey](../../../ecosystem-survey/protocol-survey.md)

---

## 1. Threat Model（per §43）

### 1.1 资产

- Git 仓库内容（commit / branch / tag / worktree）
- Issue / Task / MR 数据
- Agent 身份 + lease
- 用户身份 + permission
- API token（AI provider / Git remote / SSH key）
- Audit trail

### 1.2 威胁 + 缓解

| 威胁 | 攻击面 | 缓解 |
|---|---|---|
| **T-01: Tool Poisoning**（恶意 MCP tool 描述） | MCP server | Tool 描述视为 untrusted；Host 必取得 user 同意才能 invoke；清单版本化 |
| **T-02: Prompt Injection**（恶意文档/网页夹带 prompt） | Repository 任何文件 | 文件内容视作 data 不是 instruction；AGENTS.md / system prompt 必明示 |
| **T-03: 越权 PR 创建** | API token | Permission Discovery（per §27）;Least Privilege;Rate Limit |
| **T-04: worktree 注入** | Git 协议 | receive-pack 必 fail-closed（已 per RGS-OPEN-QA ISS-117） |
| **T-05: AGENTS.md 篡改** | Git 写入 | 仓库内 AGENTS.md 由 reviewer 控制；STAR bootstrap 不依赖其内容做特权操作 |
| **T-06: Vendor 突然断供** | 任何 vendor 集成 | Zero Vendor Cooperation Principle（per ADR-0021）;Fallback Ladder 4 级 |
| **T-07: Agent 永久占用** | Agent Lease | Heartbeat + Lease Timeout + Resume 协议（per §30-§31） |
| **T-08: 敏感凭证泄露** | AGENTS.md / file | 凭证不入仓；用 Vault 抽象（GitGit V0 T6 task 负责） |
| **T-09: 多 Agent 文件冲突** | Multi-Agent | 物理隔离 worktree + AST-level conflict detection（per §32） |
| **T-10: Audit trail 篡改** | Audit 存储 | 不可变 append-only log；HMAC chain |

### 1.3 不可接受风险

- ❌ Agent 绕过 user approval 做 production 操作
- ❌ Agent 修改 production 分支 without explicit user
- ❌ 任何 vendor 适配器获得 STAR Core 决策权
- ❌ 任何"per X 历史形态"等回溯叙事（per user.md 2026-08-26 强证据）

## 2. Non-functional Requirements

| NFR | 指标 | 测量方式 |
|---|---|---|
| **NFR-AI-001** Universal Submit 协议必须被所有 AI Agent 100% 支持 | 7 款主流 Agent 中 4 款实测通过 | Phase D Unknown Agent Test |
| **NFR-AI-002** 任何"零 STAR 训练数据"Agent 能完成闭环 | Unknown Agent Test pass | Phase D |
| **NFR-IDE-001** 任何"零 STAR 专用 plugin"IDE 能接入 | Unknown IDE Test pass | Phase D |
| **NFR-PERF-001** `star` CLI 命令响应 | P95 < 200ms (本地)；< 2s (REST) | benchmark |
| **NFR-PERF-002** MCP tool invoke 响应 | P95 < 500ms | benchmark |
| **NFR-PERF-003** Git Provider 操作 | 跟 GitHub/GitLab 持平（不慢于 1.5x） | benchmark |
| **NFR-REL-001** Core 100% vendor-neutral | grep 测试 | CI |
| **NFR-REL-002** 删除 Optional Adapter 后 Core 100% 完整 | build + test pass | CI |
| **NFR-REL-003** Fallback Ladder 4 级全部可工作 | 4 级分别跑通 | Phase D |
| **NFR-SEC-001** Tool Poisoning 防护 | 全部 tool 描述带签名 | 验证脚本 |
| **NFR-SEC-002** Audit trail 不可篡改 | HMAC 链 | 验证脚本 |
| **NFR-OP-001** 单 SRE 集中模式 OLU | ≤ 2 SRE·周/周 | per RGS-TS-001 §6.2 token-OLU |
| **NFR-COMPAT-001** 任何 Git 客户端可 clone GitGit | 跑 git clone + git push + git worktree | CI |
| **NFR-COMPAT-002** MCP 2026-07-28 规范兼容 | 跑官方 MCP Inspector | Phase D |
| **NFR-COMPAT-003** OpenAPI 3.1 spec 有效 | swagger-cli validate | CI |

## 3. Performance Requirements

| 性能 | 目标 |
|---|---|
| `star` CLI 启动 | < 100ms |
| `star agent capabilities --json` | < 50ms |
| `star task current --json` | < 200ms |
| `star code search` | < 1s (P95) |
| `star submit` 端到端 | < 5s (typical) |
| MCP tool list | < 1s (with cache) |
| REST API P95 | < 500ms |

## 4. Token Efficiency Requirements（per §23）

| 指标 | 目标 |
|---|---|
| 禁止发送整个 Issue 历史到 LLM | ✅ 强制 |
| 禁止发送整个 Repository | ✅ 强制 |
| 禁止发送所有文档 | ✅ 强制 |
| 禁止发送所有 Pipeline 日志 | ✅ 强制 |
| 禁止发送所有代码文件 | ✅ 强制 |
| 强制 Progressive Disclosure | ✅ |
| 强制 Graph-based Context | ✅ |
| 强制 Semantic Search | ✅ |
| 强制 Symbol-level Retrieval | ✅ |
| 强制 Incremental Diff | ✅ |
| 强制 Context Cache | ✅ |
| 强制 Context Snapshot | ✅ |
| 强制 Task-aware Retrieval | ✅ |
| 强制 IDE Viewport Context（per IDE Session 状态） | Phase 2 |

## 5. 与其他 spec 的关系

| Spec | 引用本文 |
|---|---|
| [arch/03 STAR AI Compat Arch](03-star-ai-compat-arch.md) | 引用 NFR-AI-001/002 |
| [arch/04 STAR IDE Gateway](04-star-ide-gateway-arch.md) | 引用 NFR-IDE-001 |
| [arch/05 GitGit Compat Arch](05-gitgit-compat-arch.md) | 引用 NFR-COMPAT-001 |
| [spec/acceptance/*](../spec/acceptance/) | 引用 NFR-* 作为验收门 |
| [spec/flows/audit-model](../spec/flows/) | 引用 T-10 / NFR-SEC-002 |

## 6. 签字栏 / 修订历史

per [arch/01](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
