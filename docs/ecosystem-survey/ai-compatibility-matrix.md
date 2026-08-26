# AI Coding Agent 兼容性矩阵（事实基线）

> **调查日期**：2026-08-26
> **范围**：7 款主流 Coding Agent 的公开能力 + 关键生态事件
> **目的**：给 STAR × GitGit AI/IDE 零厂商适配架构升级提供事实基线
> **原则**：只记录厂商已公开、无需厂商为 STAR 适配的能力。**不依赖**训练数据假设

---

## 1. 7 款 Coding Agent 横向能力

| 能力 | OpenAI Codex CLI | Claude Code | Gemini CLI (个人账户已停) | GitHub Copilot CLI | Cursor Agent | VS Code Agent | JetBrains Junie CLI |
|---|---|---|---|---|---|---|---|
| **Shell 执行** | ✅ npm i -g @openai/codex | ✅ npm i -g @anthropic-ai/claude-code | ✅ npm i -g @google/gemini-cli（个人账户 2026-06-18 停止） | ✅ brew install copilot-cli | ✅ cursor-agent CLI | ✅ 集成终端 + Run in Terminal tool | ✅ junie CLI |
| **Git CLI 调用** | ✅ 通过 shell 调 git | ✅ Claude infers standard git workflows | ✅ gemini 自动调 git | ✅ 集成 gh CLI | ✅ Cursor 调 git worktree | ✅ GitHub PR / git CLI | ✅ 集成 git worktree / GitHub Action |
| **AGENTS.md 读取** | ✅ 启动自动读（per official best practices） | ❌ 默认读 CLAUDE.md（可在 settings 切换） | ⚠️ 默认读 GEMINI.md（一行配置可切到 AGENTS.md） | ⚠️ 读 .github/copilot-instructions.md | ✅ 同时支持 .cursorrules / AGENTS.md | ✅ .github/instructions/ 或 AGENTS.md | ✅ .junie/guidelines.md（可 import） |
| **CLAUDE.md 读取** | ❌ | ✅ 默认 | ❌ | ❌ | ❌ | ❌ | ⚠️ 可 import |
| **GEMINI.md 读取** | ❌ | ❌ | ✅ 默认 | ❌ | ❌ | ❌ | ❌ |
| **MCP Server 集成** | ✅ STDIO + HTTP（v0.146 多文件夹） | ✅ 原生（最大生态） | ✅ 同 MCP | ✅ GitHub MCP Server 官方 | ✅ .cursor/mcp.json | ✅ MCP GA in 1.102 (2025-07-14) | ✅ /mcp 命令 |
| **文件系统访问** | ✅ read/write/edit | ✅ Read/Edit/Write tools | ✅ ReadFile/WriteFile/Edit | ✅ workspace access | ✅ full file ops | ✅ VS Code workspace API | ✅ 全 IDE 项目视图 |
| **MCP Tools 数量上限** | n/a | n/a | n/a | n/a | 1 tools/request cap 软约束 | ⚠️ 128 tools/req 硬上限 | n/a |
| **LSP 集成** | ❌ 暂无 | ✅ 2025-12 起 native LSP | ❌ | ⚠️ 通过 Copilot LSP | ⚠️ 通过 TS/JS LSP | ✅ 全 LSP | ✅ 借用 IDE 语义索引（Junie 强项） |
| **OpenAPI 调用** | 通过 shell 调 curl | 通过 shell / MCP | 通过 shell | 通过 shell + GitHub MCP | 通过 MCP / shell | 通过 shell + extensions | 通过 shell |
| **自定义 Instructions** | AGENTS.md / codex.md | CLAUDE.md / .claude/rules/ | GEMINI.md / settings | copilot-instructions.md | .cursorrules | .github/instructions/ | .junie/guidelines.md |
| **Worktree 隔离** | ✅ --worktree 并行（实验性） | ✅ 手 git worktree + 多 Claude 实例 | ⚠️ 通过 shell | ✅ 每个 background agent 独立 worktree | ✅ 最多 8 个并行 agent 各自 worktree | ✅ vscode.dev/agents tunnel | ✅ /worktree 命令 |
| **Plan Mode** | ✅ /plan toggle | ✅ Shift+Tab 循环 | ✅ v0.34+ 默认开 | ✅ Plan agent | ✅ Plan toggle | ✅ Plan mode | ✅ /plan toggle |
| **Headless Mode** | ✅ codex exec | ✅ claude -p | ✅ gemini -p --output-format json | ✅ copilot -p | n/a (IDE-only) | ✅ copilot -p | ✅ junie --headless |
| **JSON Output** | ✅ --output-format json | ⚠️ 部分命令支持 | ✅ --output-format json/stream-json | ✅ --output-format json | n/a | n/a | ⚠️ /usage 显示 |
| **Approval modes** | suggest / auto-edit / full-auto | default / acceptEdits / plan / auto / bypassPermissions | default / auto_edit / yolo | suggest / auto | Auto-approve / manual | Manual / Auto-approve / per-tool | Ask / Code / Brave + Action Allowlist |
| **License** | Apache 2.0（开源） | Proprietary | Apache 2.0（个人已停，企业仍可用） | Proprietary | Proprietary | MIT (core) | Proprietary |

---

## 2. 关键生态事件（直接影响 STAR 设计选择）

| 时间 | 事件 | 影响 |
|---|---|---|
| 2025-04 | GitHub Copilot Agent Mode 在 VS Code Stable 推出 | Agent 模式正式成为 IDE 一等公民 |
| 2025-09 | Gitpod 改名 Ona，从 IDE-centric 转向 "agent mission control" | Cloud IDE 厂商主动拥抱 agent-first |
| 2025-10-15 | Gitpod Classic pay-as-you-go 关停 | 依赖 Gitpod SaaS 的用户必须迁到 Ona / Codespaces / DevPod |
| 2025-12 | Claude Code + Kiro CLI 引入 native LSP | Agent 第一次能"读懂"代码语义（不仅是 grep） |
| 2026-02-24 | Cursor Cloud Agents 上线（VM 跑 PR） | Coding Agent 真正做到 async + 端到端 PR |
| 2026-04 | GitHub Copilot Coding Agent GA（异步 PR Agent） | Agent 能在 cloud 容器里完整跑测试 + 开 PR |
| 2026-06-17 | JetBrains Junie GA；Copilot 桌面 App GA；Copilot 用量计费改 AI Credits | 桌面 Agent 进入主流 |
| 2026-06-18 | **Gemini CLI 个人账户停止服务**（迁 Antigravity `agy`） | 任何依赖"免费个人 Coding Agent"的方案都有"厂商关停"风险 → 强化 §3 零厂商适配原则 |
| 2026-07-28 | **MCP 2026-07-28 大版本**：stateless core、Multi Round-Trip Requests、Header-based routing、DCR 弃用 | 任何 MCP Server 至少要支持 2026-07-28 + 旧版至少 12 个月迁移窗口 |

---

## 3. 资料来源（per 2026-08-26 调查）

| 主题 | 资料 |
|---|---|
| Codex CLI | https://www.tonyreviewsthings.com/how-to-use-openai-codex · https://aiworkflowpro.com/openai-codex-complete-guide · https://codex.danielvaughan.com/2026/08/01/codex-cli-v0146-multi-folder-projects · https://www.scriptbyai.com/best-cli-ai-coding-agents/ |
| Claude Code | https://lobehub.com/fr/mcp/canarslandev-claude-code-setup · https://www.ainsteinacademy.com/lessons/claude-code-complete-guide-opus-45 · https://dev.to/ji_ai/if-you-installed-claude-code-and-only-chat-with-it-youre-missing-the-point-4elg · https://yboulaamane.github.io/blog/claude-code-practical-workflow · https://www.claudemarket.ai/blog/everything-claude-code |
| Gemini CLI | https://github.com/google-gemini/gemini-cli · https://dev.to/james_miller_8dc58a89cb9e/an-advanced-practical-guide-to-mastering-the-gemini-cli-in-2026-19lk · https://lobehub.com/skills/spillwavesolutions-mastering-gemini-cli · https://howaiworks.ai/ai-tools/gemini-cli · https://aicatchup.com/tools/gemini-google-ai-stack-2026 |
| Copilot | https://code.visualstudio.com/docs/agents/run/agent-harnesses · https://bizarro.dev.to/sourcier/github-copilot-for-engineers-getting-better-results-41l2 · https://blog.csdn.net/Leinwin/article/details/163507399 · https://practicaldev-herokuapp-com.freetls.fastly.net/pwd9000/using-github-copilot-coding-agent-for-devops-automation-3f43 |
| Cursor | https://zairalabs.ai/guide/tools/cursor · https://cowork.ink/blog/cursor-agent-mode · https://toolradar.com/blog/best-mcp-servers-cursor · https://zairalabs.ai/guide/guides/tools-for-cursor |
| VS Code Agent | https://code.visualstudio.com/docs/chat/chat-tools · https://code.visualstudio.com/api/extension-guides/ai/ai-extensibility-overview · https://www.worldprogramming.org/posts/implant-an-extension-to-vs-code-that-exposes-its-apis-to-coding-agents-yygshl · https://code.visualstudio.com/docs/agents/overview · https://www.itechguides.com/vibe-coding-with-github-copilot-agent-mode-and-mcp-in-vs-code-updated-for-2026 |
| JetBrains Junie | https://junie.jetbrains.com/docs/slash-commands.html · https://altaitools.com/what-is-junie · https://junie.jetbrains.com/docs/junie-ide-plugin.html · https://altaitools.com/junie-cli-review · https://altaitools.com/junie-cli/ |
| 生态事件 | https://aisotools.com/blog/gitpod-review-2026 · https://howaiworks.ai/ai-tools/gemini-cli |
| 1 款开源 sub-agent MCP Server | https://github.com/shinpr/sub-agents-mcp/blob/main/README.md |

---

## 4. STAR × GitGit 设计的 5 个直接推论

1. **AGENTS.md 是 Bootstrap 必选** — Codex / Cursor / VS Code / Junie / Windsurf / Aider / Devin / Gemini CLI / Zed / RooCode 等 20+ 工具读它，比 vendor-specific 指令文件覆盖更广
2. **MCP 是增强层、不是唯一入口** — 6/7 款 Agent 支持 MCP，但 0/7 款要求它；Fall-back 到 Git + Shell 必须工作
3. **`star` CLI 必提供 `--json` + 稳定 schema** — 4/7 款 Agent（Codex / Gemini / Copilot / Claude 部分）支持 headless JSON output；machine 模式比 human 文本重要
4. **Git Worktree 是事实标准** — 5/7 款 Agent 原生支持独立 worktree 隔离
5. **Claude Code / Junie 已支持 LSP** — STAR 可选提供 LSP 协议层（不强制），但 Code Intelligence 服务仍是 STAR 责任

---

## 5. 已知缺口（缺标比错标更安全 — per user.md 2026-08-26 强证据）

- ⚠️ JetBrains AI Assistant（非 Junie 主线）的精确能力未单独调研；Junie 是其 agent 形态
- ⚠️ Google Antigravity CLI（`agy`）作为 Gemini CLI 个人账户替代品的详细能力未深查
- ⚠️ Windsurf / Replit / Aider / Goose / Devin / Factory 等更小众 Coding Agent 未在本次范围内
- ⚠️ Cursor MCP "supports only MCP tools, not the full MCP resource model" — 引用 zairalabs 评测（**注**：评测日期 2026-08-19，2026-07-28 新 spec 后可能已变化，**未实测**）
- ⚠️ "VS Code Agent 128 tools/request" 硬上限 — 引用官方文档，**未实测**我们 MCP 工具数到 129 时的实际行为
