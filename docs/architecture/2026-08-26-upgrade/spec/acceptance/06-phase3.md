# 43. Phase 3

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/acceptance/05-phase2.md](05-phase2.md) · [spec/acceptance/04 §3 v0.2 数字校准表](04-mvp.md) · [arch/03 §2.2 + §2.3 + arch/05 §5](../../arch/)

## 1. 范围（Phase 2 之上）

### 1.1 数字继承（per 2026-08-27 校准）

> Phase 3 继承 [spec/acceptance/04 §3 v0.2](04-mvp.md) 校准后数字：
> - `star` CLI: **28** 命令 = 17 核心 + 11 扩展（per [spec/cli/01 §2.1 + §2.2](../cli/01-cli-spec.md)）
> - MCP server: **16** tools = 13 MVP + 3 扩展（per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md)）
> - REST API: **14** endpoints = 12 MVP + 2 扩展（per [arch/05 §5](../../arch/05-gitgit-compat-arch.md)）
> - Universal Submit: **12** 步（per [spec/flows/05 §2](../flows/05-universal-submit.md)）
>
> Phase 3 不引入新的 CLI / MCP / REST 数字（per §2 边界 + [arch/03 §2.4 5 接入通道固化](../../arch/03-star-ai-compat-arch.md)）；Phase 3 增量为 Code Intelligence / Context Graph / Multi-Agent 编排深度。

### 1.2 范围列表

- 完整 RAG（Context Graph 全量 + Code Embedding）
- 完整多 Agent 编排（Multi-Agent 全部 9 类冲突检测）
- Advanced Context Selection
- Decision Memory 完整
- Symbol-level Conflict 完整
- Symbol-level Feedback 准确率 > 95%
- Remote Runner 完整
- Development Heatmap Phase 2

## 2. 不在 Phase 3

- V2 Candidates（per master plan §30.4）
- Future 探索
- 新增 CLI / MCP / REST 通道（5 接入通道 Phase 1 固化，per [arch/03 §2.4 + §5](../../arch/03-star-ai-compat-arch.md) + [spec/acceptance/05 §2 v0.2](05-phase2.md) 跨期约束）
- Web UI Agent API 化（Web UI 是 Human Interface，per [spec/acceptance/05 §1.1 v0.2](05-phase2.md) 跨期约束）

## 3. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：8 项 Phase 3 新增 + 2 项禁项 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| §1.1 加数字继承段（28 CLI / 16 MCP / 14 REST / 12 步 Submit, per acceptance/04 §3 v0.2 校准后）· §2 禁项加"新增 CLI / MCP / REST 通道" + "Web UI Agent API 化" | 8 子代理 INTERFACE-REVIEW-C "校准数字到 28 CLI / 16 MCP / 14 REST" + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-6 (校准 28 CLI / 16 MCP / 14 REST)
