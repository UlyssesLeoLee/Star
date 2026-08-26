# 协议层事实基线（MCP / AGENTS.md / OpenAPI / LSP / tree-sitter / rust-analyzer）

> **调查日期**：2026-08-26
> **范围**：4 套 + 2 套工具 = STAR × GitGit 协议选型的关键事实
> **目的**：给 AI Compatibility Layer / IDE Compatibility Layer / Code Intelligence 协议选型提供事实基线

---

## 1. Model Context Protocol (MCP) — 2026-07-28 当前规范

| 维度 | 状态（2026-08-26） |
|---|---|
| **最新规范版本** | 2026-07-28（per `modelcontextprotocol.io/specification/draft`） |
| **传输** | stdio（本地）+ Streamable HTTP（远程） + OAuth 2.1 |
| **核心原语** | Tools / Resources / Prompts（client 端新增 Elicitation） |
| **核心变更（2026-07-28）** | ① Stateless core（无 session） ② Multi Round-Trip Requests (MRTR) ③ Header-based routing（`Mcp-Method` / `Mcp-Name`） ④ 可缓存 list 结果（`ttlMs` / `cacheScope`） ⑤ Authorization hardening（RFC 9207 issuer validation） ⑥ 正式 Feature Lifecycle（Active / Deprecated / Removed） |
| **已弃用（12 个月内移除）** | Roots · Sampling · Logging · 旧 HTTP+SSE transport · Dynamic Client Registration (DCR) |
| **采用数据** | 9,700 万次 SDK 下载/月（Python + TS）；9,652 个官方 Registry server；41% 企业生产环境采用（Stacklok 调研） |
| **支持方** | Claude · OpenAI Agents SDK/Responses API · Gemini SDK · Microsoft Copilot Studio · Vercel AI SDK · GitHub MCP Server |
| **SDK 状态** | TS / Python / Go / C# 已 GA，**Rust 仍在 beta**（per powerdrill.ai 2026-08-05） |
| **参考实现** | Anthropic TypeScript SDK + `createMcpHandler`（stateless mode，2025-11 GA） |

**资料**：
- 官方 spec：https://modelcontextprotocol.io/specification/draft
- 2026-07-28 解读：https://cfl.re/4w8Yrlu（Cloudflare 博客）· https://powerdrill.ai/blog/what-is-mcp · https://www.getmaxim.ai/articles/what-is-model-context-protocol-mcp-a-2026-guide
- 中文完整教程：https://most.tw/posts/blog/mcp-complete-guide-2026
- 旧 spec 迁移窗口：12 个月（Roots / Sampling / Logging / HTTP+SSE）

**对 STAR 的推论**：
- MCP 2026-07-28 是必支持基线
- Rust SDK 仍在 beta — STAR CLI 用 Rust 实现时需要选 stdio transport（避免 Streamable HTTP 的实现风险）
- 弃用功能：不要在 STAR MCP Server 实现 Sampling / Logging（用 stderr 或 OTel）
- Tool list 必须按 deterministic order 排序 + 支持 ttlMs 缓存

---

## 2. AGENTS.md — 事实标准

| 维度 | 状态 |
|---|---|
| **来源** | Agentic AI Foundation（Linux Foundation 旗下），非任何单一厂商 |
| **格式** | 纯 Markdown，无 schema，无必填字段 |
| **层级解析** | 最近的 AGENTS.md 优先（monorepo 可用 per-package override） |
| **覆盖工具** | OpenAI Codex · Google Jules · GitHub Copilot Coding Agent · Aider · goose · opencode · Factory · Devin · Amp · RooCode · Cursor · VS Code · Zed · JetBrains Junie · Warp · Windsurf · Augment Code · Gemini CLI（via config switch） 等 20+ |
| **使用量** | 60,000+ 仓库已采用 · GitHub 23.7k stars · 88 个文件（OpenAI 自己的仓库） |
| **章节推荐** | overview · build & test · code style · testing · security · commit message · PR rules · deployment |
| **常被对比的 vendor-specific 文件** | `CLAUDE.md`（Claude）· `GEMINI.md`（Gemini）· `.cursorrules`（Cursor）· `.github/copilot-instructions.md`（Copilot）· `.junie/guidelines.md`（Junie） |

**资料**：
- 官方站点：https://agents.md
- 协议解读：https://www.beri.net/learning/agents-md-spec · https://www.startuphub.ai/ai-news/ai-tools/2026/what-is-agents-md-the-ai-coding-instruction-file-explained
- 实践：https://yoo.be/agents-md-ai-context-file-copilot-rework · https://dev.to/huangyongshan46a11y/agentsmd-the-file-every-github-repo-should-have-in-2026-fdg
- Ted Neward 注解：https://research.tedneward.com/ai/llm/specs/agentsmd.html

**对 STAR 的推论**：
- AGENTS.md 是 STAR 必生成 + 必维护的 bootstrap 文件
- 必须保持**薄**（per §14 任务原文），不要塞企业知识
- 必须含 3 个最小可用命令：`star agent capabilities` · `star task current --json` · `star submit`
- AGENTS.md 是 "Bootstrap" 不是 "Knowledge Base"

---

## 3. OpenAPI — 3.1 / 3.2

| 维度 | 状态 |
|---|---|
| **当前推荐** | OpenAPI 3.1（新项目）或 3.2（最新） |
| **3.1 vs 3.0 关键差异** | 完整对齐 JSON Schema Draft 2020-12 · 支持 `webhooks` · `info.summary` · `info.license.identifier` · 破坏性变更：`nullable: true` → `type: [string, "null"]` · exclusive bound 用 boolean modifier |
| **工具链** | Redocly CLI · Stoplight Studio · Swagger UI 5.x · openapi-generator（3.1 支持进展中） · Spectral |
| **3.2** | 3.1 的增量升级，主要延续 JSON Schema 对齐 |
| **最佳实践** | 机器可读 · 稳定 · 权限感知 · 可审计 · 版本化 |

**资料**：
- 官方 3.0 vs 3.1 差异：https://openapispec.com/docs/what/what-are-the-key-differences-between-openapi-3-0-and-3-1
- 实际案例：https://proxycheck.io/BLOG/post/new-unified-changelog-interface-and-other-news

**对 STAR 的推论**：
- STAR REST API 使用 OpenAPI 3.1（不是 3.0）
- Agent / IDE 客户端可以从 OpenAPI 自动生成类型化 client
- 与 MCP 共享 Domain API（同源），不重复实现

---

## 4. Language Server Protocol (LSP)

| 维度 | 状态 |
|---|---|
| **当前** | 协议稳定，已被所有主流 IDE 支持 |
| **核心能力** | goToDefinition · findReferences · documentSymbol · callHierarchy/incoming+outgoing · hover · diagnostics · completion · codeAction · semanticTokens · workspace/symbol · inlayHint |
| **Agent 集成里程碑** | Claude Code + Kiro CLI 在 **2025-12** 首次引入 native LSP（per bizarro.dev.to 评测） |
| **Rust 生态** | rust-analyzer 官方支持 LSP |
| **Python 生态** | ty（Astral 新语言服务器）· python-lsp-server · ruff (format) |
| **Protobuf 生态** | buf CLI 1.72.0+ 内置 `buf lsp serve`（用 query-driven compiler frontend，2026-01 GA） |
| **LSP 盲点** | 反射 / 字符串路径 / 配置 / SQL / dynamic dispatch（必须用 grep 兜底） |

**资料**：
- LSP 评测：https://bizarro.dev.to/empiree/lsp-for-ai-coding-agents-the-protocol-your-agent-isnt-using-yet-1l33
- ty reference：https://docs.astral.sh/ty/features/language-server
- Autohand 集成：https://www.autohand.ai/docs/guides/lsp-code-intelligence.html
- Protobuf LSP：https://tools.cooconsbit.com/en/articles/protobuf-lsp-setup-guide-en

**对 STAR 的推论**：
- STAR 可选提供 LSP 端点（不强制，但加分）
- Code Intelligence 服务在 STAR 内，但 LSP 协议层可放在适配层
- 关键能力：definition / references / symbol / callHierarchy / hover / diagnostics
- **LSP 不替代 grep**：STAR Agent 仍要保留 grep 兜底能力

---

## 5. tree-sitter

| 维度 | 状态 |
|---|---|
| **GitHub stars** | 19,000+ |
| **License** | MIT |
| **使用方** | Cursor（code indexing）· Neovim（默认）· Helix · GitHub（code search + syntax highlighting） |
| **支持语言** | 100+（社区维护 grammar 列表） |
| **绑定** | C core + Rust/JS/Python/Go bindings |
| **特点** | 增量解析（sub-millisecond），AST + 错误恢复 |
| **AST Query** | S-expression 模式匹配 |

**资料**：
- Grammar 列表：https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers
- 介绍：https://aicoolies.com/tools/tree-sitter
- 实战（MCP + tree-sitter + 27 tools）：https://dev.to/uwe_c_39d9ab7d16ff8dfe67e/how-i-cut-ai-context-usage-by-50x-with-a-tree-sitter-code-index-plm

**对 STAR 的推论**：
- tree-sitter 是 Code Intelligence 服务的**可选底层**（不是唯一）
- MVP 阶段不必引入；Phase 2 / 3 引入可大幅降低 token 消耗（per 50x 案例）
- Rust binding 成熟，可直接 `tree-sitter-rs`

---

## 6. rust-analyzer

| 维度 | 状态 |
|---|---|
| **位置** | rustup component add rust-analyzer |
| **License** | MIT / Apache 2.0 |
| **能力** | 完整 LSP server + inlay hints · semantic tokens · call hierarchy · runnables |
| **Cargo 项目集成** | 自动发现 Cargo.toml，提供 `cargo check` 即时诊断 |
| **GitGit / STAR 关系** | GitGit 自身（Rust 实现）天然用 rust-analyzer 做 IDE 体验；STAR 复用其 LSP 输出 |

**资料**：广泛引用，未深查单一来源（生态共识）

**对 STAR 的推论**：
- STAR Code Intelligence 服务的 MVP 阶段可**直接用 rust-analyzer LSP server** 做 Rust 代码智能
- 其他语言通过 LSP 客户端抽象（per §4）
- AST 完整分析留给 Phase 2

---

## 7. 综合推论：STAR × GitGit 协议栈选型

| 层 | 协议 / 工具 | 必选 / 可选 | 理由 |
|---|---|---|---|
| **Repository bootstrap** | AGENTS.md | **必选** | 20+ 工具读，vendor-neutral |
| **AI 工具** | MCP 2026-07-28 | 必选 | 增强层 + 6/7 主流 Agent 客户端 |
| **AI 兜底** | Shell + Git CLI | **必选** | 7/7 工具都支持，是 Universal Submit 协议基础 |
| **REST API** | OpenAPI 3.1 | 必选 | 机器可读、跨 IDE 通用 |
| **Code Intelligence** | LSP | 可选 | 增强层（Claude Code / Junie 已支持） |
| **Code Intelligence 底层** | rust-analyzer / tree-sitter | Phase 2+ | MVP 用 ripgrep + 简单 symbol 索引 |
| **机器可读 CLI** | --json + 稳定 schema | **必选** | 4/7 工具 headless JSON mode |
| **Project Instructions 分层** | Bootstrap / Context / Policy / Task | **必选** | 4 类不能混在一起（per §15） |

---

## 8. 已知缺口

- ⚠️ MCP Rust SDK 仍在 beta — 实际行为未实测
- ⚠️ LSP 在 Claude Code / Kiro CLI 之外的 Agent 集成（Codex / Gemini / Cursor）的稳定性未实测
- ⚠️ OpenAPI 3.2 实际差异未深查
- ⚠️ rust-analyzer 2026 最新功能（如 inlay hints 性能改进）未单独查
- ⚠️ tree-sitter 在大型 monorepo（>10k 文件）的真实索引时间未实测（dev.to 评测最大 500 文件）
