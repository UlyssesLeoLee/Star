# 41. MVP

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/cli/01-cli-spec.md §2.1 MVP 17 核心命令](../cli/01-cli-spec.md) · [spec/cli/01-cli-spec.md §2.2 11 扩展命令](../cli/01-cli-spec.md) · [spec/mcp/01-mcp-spec.md §2 16 tools](../mcp/01-mcp-spec.md) · [arch/05 §5 REST API MVP 12 子集边界](../../arch/05-gitgit-compat-arch.md) · [spec/flows/05 §2 12 步流程](../flows/05-universal-submit.md)

## 1. 范围

不实现所有 AI / IDE / Code Intelligence 功能。MVP 必先证明：

```text
Unknown Coding Agent / Unknown IDE
   ↓
Clone GitGit Repository
   ↓
读 Repository Instructions
   ↓
发现 STAR CLI
   ↓
Capability Discovery
   ↓
获取 Assigned Issue
   ↓
获取 Context
   ↓
搜索相关代码
   ↓
定位相关符号
   ↓
创建 Workspace
   ↓
创建 Worktree
   ↓
修改代码
   ↓
运行测试
   ↓
Commit
   ↓
star submit
   ↓
创建 MR
   ↓
STAR 更新 Issue 状态
```

## 2. 关键约束

- 整个过程**不**修改 STAR Core / GitGit Core
- **不**写 AI 厂商适配器
- **不**等 IDE 厂商适配

## 3. MVP 退出条件

- [ ] `star` CLI 17 核心命令 + 11 扩展命令（per P1-3 修复 2026-08-27，per [spec/cli/01 §2.1](../cli/01-cli-spec.md) MVP 17 子集边界；完整 28 = 17 MVP + 11 扩展 = [spec/cli/01 §2.2](../cli/01-cli-spec.md)）
- [ ] `star --json` 稳定 schema (`agent-api/v1`)
- [ ] MCP server 16 tools（per P1-4 修复 2026-08-27 = 13 MVP + 3 扩展 per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md)，完整 16 per [spec/mcp/01 §2](../mcp/01-mcp-spec.md)）
- [ ] REST API 14 endpoints（per P1-5 修复 2026-08-27 = 12 MVP + 2 扩展 per [arch/05 §5](../../arch/05-gitgit-compat-arch.md)）+ OpenAPI 3.1
- [ ] AGENTS.md 自动生成器
- [ ] Universal Submit 12 步（per P1-6 修复 2026-08-27 = 11 步原版 + 1 步"回写 IDE Session 状态" per [spec/flows/05 §2](../flows/05-universal-submit.md)）
- [ ] Agent Task Lifecycle 9 状态 + 4 异常
- [ ] Agent Lease / Heartbeat / Resume
- [ ] Version Control Provider 4 实现
- [ ] Unknown Agent Test 通过（含 Level 1 默认 + Level 4 降级两段，per [spec/acceptance/01 §3 + §4](01-unknown-agent-test.md) P1-1 修复 2026-08-27）
- [ ] Zero-Knowledge Agent Test 通过
- [ ] Unknown IDE Test 通过（含 OpenAPI 6 项最低能力消费，per [spec/acceptance/03 §2 + §3](03-unknown-ide-test.md) P2-2 修复 2026-08-27）
- [ ] GitGit 标准 Git 兼容 100%
- [ ] Fallback Ladder 4 级全部跑通（含 Level 1/2/3/4 单独 conformance，per [spec/vcs/04 §5](../vcs/04-fallback-strategy.md) P1-L 修复 2026-08-27）

> **数字校准表**（per P1-3 / P1-4 / P1-5 / P1-6 修复 2026-08-27）：

| 退出条件 # | 原数字 | 校准后 | 来源 |
|---|---|---|---|
| 1 | 17 核心 | 17 核心 + 11 扩展（完整 28） | [spec/cli/01 §2.1 + §2.2](../cli/01-cli-spec.md) |
| 3 | 13 tools | 16 tools（13 MVP + 3 扩展） | [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) + [spec/mcp/01 §2](../mcp/01-mcp-spec.md) |
| 4 | 12 endpoints | 14 endpoints（12 MVP + 2 扩展） | [arch/05 §5](../../arch/05-gitgit-compat-arch.md) |
| 6 | 11 步 | 12 步（+ 1 步 IDE Session 状态回写） | [spec/flows/05 §2](../flows/05-universal-submit.md) |

## 4. 实施位置

- `crates/star-cli/` (主) — 17 核心 + 11 扩展（完整 28 个 CLI 命令，per [spec/cli/01 §2.1 + §2.2](../cli/01-cli-spec.md)）
- `crates/star-mcp/` — 16 tools（13 MVP + 3 扩展，per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) + [spec/mcp/01 §2](../mcp/01-mcp-spec.md)）
- `crates/star-rest/` — 14 REST endpoints（12 MVP + 2 扩展）+ OpenAPI 3.1（per [arch/05 §5](../../arch/05-gitgit-compat-arch.md)）
- `crates/star-domain/`
- `crates/star-application/` — Universal Submit 12 步 Application service（per [spec/flows/05 §2](../flows/05-universal-submit.md)）
- `crates/star-context/`
- `crates/star-workspace/`
- `crates/star-agent/`
- `crates/star-vcs/` — Provider 抽象 + cache 层落点 `crates/star-vcs/src/cache.rs`（per [spec/acceptance/08 R-007](08-risk-register.md) P2-7 修复 2026-08-27）
- `crates/star-audit/`
- `crates/star-policy/`
- `tests/unknown-agent/` + `tests/zero-knowledge-agent/` + `tests/unknown-ide/`（per P1-2 / P1-L 修复 2026-08-27，统一根目录 `tests/` 命名空间）
- `tests/fallback-conformance/` — Level 2/3/4 单独 conformance 测试（per [spec/vcs/04 §5](../vcs/04-fallback-strategy.md) P1-L 修复 2026-08-27）

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：14 项退出条件（17 CLI / 13 MCP / 12 REST / 11 步 Submit）| Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-3：§3 #1 改 "17 核心 + 11 扩展（完整 28）" · P1-4：§3 #3 改 "16 tools（13 MVP + 3 扩展）" · P1-5：§3 #4 改 "14 endpoints（12 MVP + 2 扩展）" · P1-6：§3 #6 改 "12 步（+ 1 步 IDE Session 状态回写）" · §3 加数字校准表 · §4 实施位置加 star-vcs cache 落点 + 28 CLI / 16 MCP / 14 REST 全量标注 | 8 子代理 INTERFACE-REVIEW-C P1-3 / P1-4 / P1-5 / P1-6 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-4 (P1-3/4/5/6)
