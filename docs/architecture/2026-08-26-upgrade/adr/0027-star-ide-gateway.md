# ADR-0027: STAR IDE 网关架构

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md) · [ADR-0024 IDE Session Identity](0024-ide-session-identity.md)
> **关联**：[arch/04 STAR IDE Gateway Arch](../arch/04-star-ide-gateway-arch.md) · [IDE Compatibility Matrix](../../../ecosystem-survey/ide-compatibility-matrix.md)

---

## 1. 背景与问题

STAR 在 2026-08-26 之前没有 IDE 接入层：

- 25 Module workspace 不包含 IDE 概念（per arch/01 §1.2）
- GitGit 作为 VCS Provider 也不包含 IDE 边界（per arch/01 §1.1）
- 6 款主流 IDE（VS Code / Cursor / JetBrains / Junie / Vim / Helix）无法以零 vendor 合作方式接入 STAR
- 业界 6/7 主流 IDE 已支持 MCP 客户端（per IDE Compatibility Matrix），但需要 STAR 端提供 MCP server + IDE 状态管理

需要一套**Vendor-Neutral** 的 IDE 网关架构，让任意具备 Git + LSP + MCP 能力的 IDE 都能以"零 STAR 专用 plugin"方式接入。

## 2. 决策

**采用 "IDE 通过 Git CLI / LSP client / MCP client 三通道接入 STAR IDE Gateway" 的最小架构。**

### 2.1 三通道接入（per arch/04 §5）

| 通道 | IDE 端要求 | 覆盖率 |
|---|---|---|
| **Git CLI** | IDE 调 `git` 命令 | 任何 IDE 都有 |
| **LSP client** | IDE 支持 LSP 协议 | VS Code / Cursor / JetBrains / Helix / Neovim / Vim |
| **MCP client** | IDE 支持 MCP 2026-07-28 | VS Code / Cursor / Junie / JetBrains（via Junie）/ Kiro CLI |

**未达三通道任一**的 IDE 走 Fallback Ladder L2 / L3 / L4（per [ADR-0026 §2.2](0026-star-ai-compat.md)）。

### 2.2 IDE Gateway 责任（per arch/04 §4）

| 责任 | 说明 |
|---|---|
| **IDE Session lifecycle** | start / pause / resume / end（per ADR-0024） |
| **Workspace mapping** | IDE workspace → STAR workspace |
| **File state sync** | OpenFile 状态通过 LSP / MCP 双向同步 |
| **Diagnostic 上报** | LSP `textDocument/publishDiagnostics` → STAR |
| **Symbol 上报** | LSP `textDocument/documentSymbol` → STAR |
| **Selection 跟踪** | LSP `textDocument/selectionRange` → STAR |
| **Permission binding** | IDE 端 permission ↔ STAR Agent permission |

### 2.3 关键架构约束

- IDE Gateway **不**提供 IDE-specific 集成（per ADR-0025 强约束）— 必须在 Optional Adapter 子 crate
- IDE Gateway **不**管理 IDE 状态（由 IDE 自身负责）
- IDE Gateway **不**作为文件内容 Source of Truth（由 Git 仓库负责）
- IDE Session 是独立对象（per ADR-0024），不污染 GitGit
- LSP Proxy（`star-lsp-proxy`）MVP 阶段不实现；Phase 2 再加（per arch/04 §6）

### 2.4 IDE Session 对象（per arch/04 §3 + ADR-0024）

```rust
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

## 3. 备选方案与拒绝理由

### 备选 A：要求每个 IDE 厂商开发 STAR 专用 plugin
- 拒绝理由：违反 ADR-0021 Zero Vendor Cooperation；商业关系不稳定

### 备选 B：自建专用 IDE（不依赖外部 IDE）
- 拒绝理由：失去 6+ 主流 IDE 用户的现成客户端；维护成本爆炸

### 备选 C：LSP Proxy 强制启用（MVP 阶段）
- 拒绝理由：MVP 阶段实现 LSP proxy 风险高，IDE 仍可直连标准 LSP server；Phase 2 再评估

## 4. 后果与影响

### 4.1 正面

- 6 款主流 IDE 立即接入（per IDE Compatibility Matrix）
- Phase D Unknown IDE Test 必须**只**用 Level 3 (REST + Git) 通过（per arch/03 §7 验收）
- 不污染 Core（per ADR-0025 强约束）
- IDE Session 独立对象可被 Agent Resume 协议利用（per ADR-0030）

### 4.2 负面 / 成本

- 三通道的 schema 稳定化需要长期投入
- IDE Session 状态管理需要 Audit / Policy 配合
- LSP Proxy 推迟到 Phase 2，hover / 增强诊断等 IDE 体验受限于标准 LSP server

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 某款 IDE 三通道都不支持 | 极低 | 低 | Fallback Ladder L2/L3/L4 |
| MCP 客户端行为不一致 | 中 | 中 | 通过 conformance 测试覆盖 4+ 款客户端 |
| IDE Session 与 Agent Session 状态冲突 | 中 | 高 | 显式分两个对象，audit_id 关联 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md) — IDE 归 STAR
- **依赖**：[ADR-0024 IDE Session Identity](0024-ide-session-identity.md) — IDE Session 独立于 GitGit
- **依赖**：[ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md) — IDE 集成不污染 Core
- **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — Fallback Ladder 通用
- **被依赖**：[ADR-0029 Universal Submit](0029-universal-submit.md) — Submit 12 步最后一步是"回写 IDE Session 状态"
- **被依赖**：[ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — Resume 协议涉及 IDE Session 状态

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | Platform Engineer | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：三通道 + Gateway 责任 + 验收路径 | Phase B 起草（per 2026-08-26 升级 Plan） |
