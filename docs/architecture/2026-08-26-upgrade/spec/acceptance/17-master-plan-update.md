# 54. STAR Master Plan Update (2026-08-26)

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **依赖**：[STAR master plan v0.1](../../plan/master-implementation-plan.md) · 本次升级全套 spec

## 1. Master Plan 升 v0.2 必要性

现有 master plan v0.1 (2026-08-25) 状态是 Draft。本次升级引入：

- 10+ 个新增 crate（star-cli / star-mcp / star-rest / star-context / star-workspace / star-agent / star-vcs / star-audit / star-policy / star-ide-gateway / star-ai-gateway / star-code-intelligence）
- 5 套新协议（AGENTS.md / MCP / OpenAPI / LSP / Capability Discovery）
- 4 级 Fallback Ladder
- 9+4 Agent Task Lifecycle
- Universal Submit 11 步

## 2. 升 v0.2 必含

- §0 摘要：新增 AI / IDE 兼容性章节
- §2 阶段路线图：MVP / V1 / V2 重定义
  - MVP = AI/IDE 兼容性闭环（per spec/acceptance/04）
  - V1 = Phase 2（per spec/acceptance/05）
  - V2 = Phase 3（per spec/acceptance/06）
- §4 引用本次升级全套 spec
- §5 token-OLU 重新估算（含 9-13M tokens / 4-6 周窗口）
- §7 SaaS Risk：增列本次升级的 R-001 ~ R-015

## 3. 关键变更

| 项 | 旧 (v0.1) | 新 (v0.2) |
|---|---|---|
| CLI | 无 | 17 个核心命令 |
| MCP | 无 | 13 tools |
| REST | 无 | OpenAPI 3.1 |
| AGENTS.md | 无 | 自动生成器 |
| Universal Submit | 无 | 11 步 |
| VCS Provider | GitGit only | 4 Provider 并列 |
| Agent Identity | 无 | 完整 schema |
| IDE Session | 无 | 独立对象 |
| Audit | 业务级 | 统一 trail（5 ActorType） |
| Fallback | n/a | 4 级 Ladder |
| 测试 | n/a | Unknown / Zero-Knowledge / Unknown IDE |

## 4. 25 Module 重新组织

现有 25 Module 是按业务域（domain-*）划分。本次升级需要按**能力**重新组织：

| 能力层 | 现有 Module | 新增 Module |
|---|---|---|
| Domain | domain-tenant / -workspace / -project / -work-item / -worktree / -agent / ... | 不变 |
| Application | n/a | star-application (新增) |
| AI Gateway | n/a | star-ai-gateway (新增) |
| IDE Gateway | n/a | star-ide-gateway (新增) |
| VCS Abstraction | n/a | star-vcs (新增) |
| Code Intelligence | n/a | star-code-intelligence (新增) |
| Context | n/a | star-context (新增) |
| Audit | n/a | star-audit (新增) |
| Policy | n/a | star-policy (新增) |
| CLI | n/a | star-cli (新增) |
| MCP | n/a | star-mcp (新增) |
| REST | n/a | star-rest (新增) |

## 5. Token-OLU 估算（重）

| 阶段 | Tokens | SRE·周 | 窗口 |
|---|---|---|---|
| Phase A 调研 | 300K | 0.3 | 1 周 |
| Phase B 边界 ADR | 400K | 0.4 | 1 周 |
| Phase C spec (54 份) | 3-4M | 3-4 | 4-5 周 |
| Phase D MVP 闭环 | 5-8M | 5-8 | 3-4 周 |
| **合计** | **9-13M** | **9-13** | **4-6 周（子代理并行）/ 9-13 周（Mavis 单干）** |

> 默认走**子代理并行**，目标 4-6 周。

## 6. 实施位置

- `docs/plan/master-implementation-plan.md` 升 v0.2
- 配套子文档全部更新

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
