# 04. STAR IDE Gateway Architecture

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **依赖**：[ADR-0022 IDE Placement](../../adr/0022-ide-placement.md) · [ADR-0024 IDE Session Identity](../../adr/0024-ide-session-identity.md)

---

## 1. 目标

任何 IDE（VS Code / Cursor / JetBrains / Vim / Helix / 全新 IDE）都能通过标准能力接入 STAR，**无需**为 STAR 开发专用 plugin。

## 2. 架构

```text
         Any IDE (browser or native)
                │
   ┌────────────┼────────────┐
   ↓            ↓            ↓
Git CLI     LSP client    MCP client
   │            │            │
   │            │            ↓
   │            │       star-mcp
   │            │            │
   │            ↓            │
   │     star-lsp-proxy     │
   │            │            │
   └────────────┼────────────┘
                ↓
         STAR IDE Gateway
                ↓
         STAR AI Gateway (per [arch/03](03-star-ai-compat-arch.md))
                ↓
         STAR Application Layer
                ↓
         STAR Domain Core
```

## 3. IDE Session 对象（per ADR-0024）

```rust
// crates/star-ide/src/session.rs
pub struct IdeSession {
    pub id: IdeSessionId,
    pub user_id: UserId,
    pub workspace_id: WorkspaceId,
    pub repository_id: RepoId,
    pub worktree_id: WorktreeId,
    pub client: IdeClientKind,           // VSCode | Cursor | JetBrains | Vim | Web | Unknown
    pub client_version: String,
    pub open_files: Vec<OpenFile>,
    pub active_symbol: Option<SymbolRef>,
    pub selection: Option<Selection>,
    pub diagnostics: Vec<Diagnostic>,
    pub terminal_id: Option<TerminalId>,
    pub agent_sessions: Vec<AgentSessionId>,
    pub audit_id: AuditId,
}
```

## 4. IDE Gateway 责任

| 责任 | 说明 |
|---|---|
| **IDE Session lifecycle** | start / pause / resume / end |
| **Workspace mapping** | IDE workspace → STAR workspace |
| **File state sync** | OpenFile 状态通过 LSP / MCP 双向同步 |
| **Diagnostic 上报** | LSP `textDocument/publishDiagnostics` → STAR |
| **Symbol 上报** | LSP `textDocument/documentSymbol` → STAR |
| **Selection 跟踪** | LSP `textDocument/selectionRange` → STAR |
| **Permission binding** | IDE 端 permission ↔ STAR Agent permission |

## 5. IDE 接入的 3 个最低要求

1. **Git CLI** — IDE 调 `git` 命令（任何 IDE 都有）
2. **LSP 客户端** — IDE 支持 LSP 协议（VS Code / Cursor / JetBrains / Helix / Neovim 都支持）
3. **MCP 客户端** — IDE 支持 MCP 2026-07-28（per Compatibility Matrix：VS Code ✅ / Cursor ✅ / Junie ✅ / JetBrains（via Junie）✅）

**未达 3 个最低要求**的 IDE 走 Level 2 / 3 / 4 Fallback Ladder（per [arch/03](03-star-ai-compat-arch.md) §3）。

## 6. LSP Proxy（可选增强层）

IDE → `star-lsp-proxy`（独立 binary）→ 标准 LSP server

**为什么需要 proxy**：
- IDE 直接连 rust-analyzer / ty / 等 LSP server 时，这些 server 不感知 STAR
- proxy 把 LSP 请求 + STAR 上下文（如"当前 Issue 是 STAR-1024"）打包
- 增强输出：比如 hover 时多一行"Related to STAR-1024"

**MVP 阶段**：不实现 proxy。IDE 直接连标准 LSP server。Phase 2 再加。

## 7. IDE 接入测试（per §44 Unknown IDE Test）

测试一个**没有 STAR 专用 plugin**的 IDE 是否可以通过标准能力接入 STAR：

只提供：Git + Shell + Repository + AGENTS.md + star CLI + OpenAPI

成功标准：
```text
打开 Repository
   ↓
发现 STAR (读 AGENTS.md)
   ↓
获取当前 Task (star task current)
   ↓
获取 Context (star context current)
   ↓
搜索代码 (star code search)
   ↓
定位符号 (star code symbol)
   ↓
修改文件 (直接编辑 + git commit)
   ↓
运行测试 (star test affected)
   ↓
创建 MR (star mr create)
```

如果必须等 IDE 厂商开发 STAR Plugin，则测试失败。

## 8. IDE Gateway 不应提供

- ❌ IDE-specific 集成到 Core（per ADR-0025，必须放 Optional Adapter 子 crate）
- ❌ IDE 状态管理（由 IDE 自身负责）
- ❌ 文件内容的 Source of Truth（由 Git 仓库负责）

## 9. 签字栏 / 修订历史

per [arch/01](01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
