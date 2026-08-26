# STAR × GitGit AI / IDE Compatibility Matrix（综合矩阵）

> **调查日期**：2026-08-26
> **汇总自**：
> - [`ai-compatibility-matrix.md`](./ai-compatibility-matrix.md) — 7 款 Coding Agent
> - [`ide-compatibility-matrix.md`](./ide-compatibility-matrix.md) — 6 款 IDE / CDE
> - [`protocol-survey.md`](./protocol-survey.md) — 4 套协议 + tree-sitter / rust-analyzer
>
> **表格说明**：✅ = 公开支持；⚠️ = 部分支持或 vendor-specific 行为；❌ = 不支持

---

## 1. Coding Agent × 能力（11 行 × 10 列）

| 能力 | OpenAI Codex | Claude Code | Gemini CLI¹ | Copilot | Cursor | VS Code Agent | Junie | **Unknown Agent²** | **Unknown IDE²** |
|---|---|---|---|---|---|---|---|---|---|
| **Git CLI** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅³ | ✅³ |
| **Shell** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅³ | ✅³ |
| **AGENTS.md** | ✅ | ❌ | ⚠️ | ⚠️ | ✅ | ✅ | ✅ | **应能读**⁴ | **应能读**⁴ |
| **MCP (2026-07-28)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **可能**⁵ | **可能**⁵ |
| **LSP native** | ❌ | ✅⁶ | ❌ | ⚠️ | ⚠️ | ✅ | ✅ | **可能** | ✅（IDE 客户端） |
| **Git Worktree** | ✅ | ✅ | ⚠️ | ✅ | ✅ | ✅ | ✅ | ✅³ | ✅³ |
| **OpenAPI 调用** | ✅ (via shell) | ✅ (via shell/MCP) | ✅ (via shell) | ✅ | ✅ (via MCP) | ✅ (via extensions) | ✅ | ✅³ | ✅³ |
| **FS access** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅³ | ✅³ |
| **Headless `--json`** | ✅ | ⚠️ | ✅ | ✅ | n/a | ✅ | ⚠️ | **应能 parse** | n/a |
| **Plan Mode** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **应能 follow** | **应能 follow** |
| **Approval modes** | 3 档 | 4 档 | 3 档 | 多档 | Auto+Manual | Manual+Auto | Ask/Code/Brave | **fallback 提示** | **fallback 提示** |

### 注：
1. **Gemini CLI 个人账户 2026-06-18 已停**（迁 Antigravity `agy`）—— 任何依赖"免费个人 Agent"的方案都有 vendor 风险
2. **Unknown Agent / IDE** = 假设明天出现的全新工具，无 STAR 训练数据，无 STAR 专用插件
3. **Git + Shell + FS + Terminal** 是 50 年基石，**任何**工具必须支持；这是"Universal Submit"的兜底层（per §38 Fallback Ladder Level 4: Git Only）
4. AGENTS.md 是事实标准 + 纯 Markdown，新 Agent **应**会读；STAR 还要写
5. MCP 是否被新工具支持不可假设——**必须**有 MCP 不工作的降级路径（per §38 Fallback Ladder）
6. Claude Code 2025-12 首次引入 native LSP；Kiro CLI 同期

---

## 2. IDE / CDE × 能力（11 行 × 7 列）

| 能力 | VS Code | Cursor | JetBrains | Codespaces | Gitpod/Ona | Dev Containers | **Unknown IDE** |
|---|---|---|---|---|---|---|---|
| **Git CLI** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **应能跑 git** |
| **Shell / Terminal** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **应能跑 shell** |
| **MCP 客户端** | ✅ | ✅ | ⚠️ (Junie) | ✅ (via VS Code) | ✅ (via VS Code) | ✅ | **可能** |
| **MCP Server（暴露给 agent）** | ✅ | ⚠️ (仅 tools) | ✅ | ✅ | ✅ | ✅ | **可能** |
| **LSP 客户端** | ✅ | ✅ | ✅ (强) | ✅ | ✅ | ✅ | **可能** |
| **OpenAPI 客户端** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | **可能** |
| **Dev Container** | ✅ | ✅ | ✅ | ✅ (原生) | ✅ (.gitpod.yml) | ✅ (标准) | **应能装 devcontainer** |
| **Port Forwarding** | ✅ | ✅ | ⚠️ | ✅ (强) | ✅ | ✅ (端口属性) | **应能转发** |
| **Cloud / Remote SSH** | ✅ | ✅ | ✅ (Gateway) | ✅ (browser) | ✅ | ✅ (任意 backend) | **可能** |
| **AGENTS.md 扫描** | ✅ | ✅ | ✅ (Junie import) | ✅ | ✅ | ✅ | **应能读** |
| **License** | MIT (core) | Proprietary | Proprietary | 闭源 + 标准 | 闭源 / 开源 | 标准开放 | n/a |

---

## 3. 协议 × 能力（4 套协议 + 2 套工具）

| 能力 | MCP 2026-07-28 | AGENTS.md | OpenAPI 3.1 | LSP | tree-sitter | rust-analyzer |
|---|---|---|---|---|---|---|
| **Maturity** | GA (2026-07-28) | 60k+ repos, 23.7k stars | GA | 稳定 | 19k stars, MIT | rustup 默认 |
| **Vendor-neutral** | ✅ (Linux Foundation 风格) | ✅ (Linux Foundation 旗下) | ✅ (OpenAPI Initiative) | ✅ (Microsoft 维护) | ✅ | ✅ (Rust 官方) |
| **Machine-readable** | ✅ JSON-RPC | ✅ Markdown | ✅ YAML/JSON | ✅ JSON-RPC | n/a | ✅ LSP |
| **AI/Agent 集成** | ✅ 6/7 Agent 支持 | ✅ 20+ 工具读 | ✅ | ⚠️ 2/7 Agent native | ⚠️ Cursor/Neovim | ⚠️ Claude Code / Junie |
| **Rust 支持** | ⚠️ SDK beta | ✅ 纯文本 | ✅ | ✅ | ✅ binding | ✅ native |
| **必选 / 可选** | **必选** | **必选** | **必选** | 可选 | Phase 2+ | Phase 2+ |
| **License** | MIT (SDK) | MIT | Apache 2.0 | MIT | MIT | MIT/Apache 2.0 |

---

## 4. STAR 接入路径（按"零厂商适配"原则）

```
┌─────────────────────────────────────────────────────────────────┐
│  Any Unknown AI Agent / IDE                                     │
│                                                                 │
│  必带（4 项）：                                                  │
│    1. Git CLI              ← git clone / push / pull / worktree│
│    2. Shell + Terminal     ← sh / bash / zsh / powershell      │
│    3. 文件系统访问         ← read / write / edit                │
│    4. Markdown 读          ← AGENTS.md bootstrap                │
│                                                                 │
│  应带（2 项，STAR 通过标准化确保可发现）：                       │
│    5. OpenAPI              ← REST API machine-readable         │
│    6. 进程间通信           ← MCP server (stdio 优先)           │
│                                                                 │
│  可选（增强层）：                                               │
│    7. LSP 客户端            ← Phase 2 Code Intelligence         │
│    8. Dev Container         ← STAR 一键启动 dev env            │
│    9. Port Forwarding       ← cloud IDE 友好                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
                  ┌───────────────────────┐
                  ↓                       ↓
        GitGit 标准化               STAR AI/IDE Gateway
        (标准 Git)                  (CLI + MCP + REST)
                  ↓                       ↓
                  └───────────┬───────────┘
                              ↓
                          STAR Core
```

**关键事实**（per 2026-08-26 调研）：
- 7/7 主流 Coding Agent 都有 Git + Shell + FS 能力 → **保证 Universal Submit 协议可用**
- 6/7 Agent 支持 MCP → **MCP 是增强层，5/7 仍可工作于 MCP-off 降级模式**
- 20+ 工具读 AGENTS.md → **vendor-neutral bootstrap 是事实标准**
- 5/7 Agent 独立 worktree → **Git Worktree 是隔离原语**

---

## 5. 终极验证问（per §50）

> **Q1: 全新 Coding Agent 从未听说 STAR/GitGit，但会 Git + Shell，能接入吗？**
> **A: YES** — AGENTS.md bootstrap 告诉它"先跑 `star agent capabilities`"，star CLI 通过标准 Unix 进程 + JSON 输出；它 clone、读 AGENTS.md、跑命令、提交、回写 Issue
>
> **Q2: 全新 IDE 从未听说 STAR/GitGit，但支持 Git + Shell + FS + Terminal，能接入吗？**
> **A: YES** — IDE 不需要知道 GitGit；通过 Git clone GitGit repo；通过 `star` CLI 接管研发流；通过 OpenAPI 自动化；Universal Submit 协议封装所有状态机

---

## 6. 已知缺口（缺标比错标安全 — per user.md 2026-08-26 强证据）

- ⚠️ 2026-08-26 后 30 天内可能的新事件未覆盖（如 MCP 增量 release / 新 Agent 发布）
- ⚠️ "Unknown Agent" 的"必带 4 项"是基于历史推断，**新工具是否真支持 Git CLI 不可硬保证**（需要 MVP 实测兜底）
- ⚠️ MCP Rust SDK 仍在 beta — STAR 选用 stdio transport 是为了规避 Streamable HTTP 实现风险
- ⚠️ Windsurf / Replit / Devin / Aider / Goose / Factory 等次主流 Coding Agent 详细能力未在本次范围
- ⚠️ Antigravity CLI（Gemini 个人账户替代）详细能力未深查
- ⚠️ DevPod / Coder / CodeSandbox 等 CDE 详细对比未做
- ⚠️ OpenAPI 3.2 vs 3.1 实际差异未深查
