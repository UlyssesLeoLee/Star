# 53. AI / IDE Ecosystem Research Summary

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26

## 1. 调研范围

- 7 款 Coding Agent（per [ai-compatibility-matrix.md](../../ecosystem-survey/ai-compatibility-matrix.md)）
- 6 款 IDE / CDE（per [ide-compatibility-matrix.md](../../ecosystem-survey/ide-compatibility-matrix.md)）
- 4 套协议 + 2 套工具（per [protocol-survey.md](../../ecosystem-survey/protocol-survey.md)）

## 2. 关键发现

### 2.1 标准化趋势

- **AGENTS.md** 由 Linux Foundation 旗下 Agentic AI Foundation 维护，20+ 工具读
- **MCP 2026-07-28** 标准化 AI tool 调用，6/7 主流 Agent 客户端支持
- **OpenAPI 3.1** 是机器可读 REST 共识
- **LSP** 是 IDE 端代码智能共识

### 2.2 生态事件

- **2025-10-15** Gitpod Classic 关停 → 强化"vendor 可停服"风险
- **2026-06-18** Gemini CLI 个人账户停服 → 同上
- **2026-07-28** MCP 大版本（stateless core）→ 12 个月迁移窗口

### 2.3 能力覆盖

| 能力 | 7/7 Agent | 6/6 IDE |
|---|---|---|
| Git CLI | ✅ | ✅ |
| Shell | ✅ | ✅ |
| FS | ✅ | ✅ |
| AGENTS.md | 5/7 | 6/6 |
| MCP | 6/7 | 4/6 |
| LSP native | 2/7 | 6/6 |
| Worktree | 5/7 | n/a |

## 3. 接入策略推论

### 3.1 必须实现

- AGENTS.md bootstrap（vendor-neutral）
- Git + Shell + FS 兜底层（7/7 Agent 都支持）
- `star` CLI + `--json` 稳定 schema
- MCP server（增强层，6/7 Agent 客户端支持）
- OpenAPI 3.1（机器可读 REST）

### 3.2 可选实现（Phase 2+）

- LSP 端点（增强层）
- Dev Container integration
- Port Forwarding
- Cloud IDE 集成

### 3.3 不依赖

- 任何 vendor-specific 集成进 Core
- 模型训练数据包含 STAR
- 商业合作

## 4. 详细资料

- 7 款 Agent：[ai-compatibility-matrix.md](../../ecosystem-survey/ai-compatibility-matrix.md)
- 6 款 IDE：[ide-compatibility-matrix.md](../../ecosystem-survey/ide-compatibility-matrix.md)
- 4 套协议：[protocol-survey.md](../../ecosystem-survey/protocol-survey.md)
- 综合矩阵：[compatibility-matrix.md](../../ecosystem-survey/compatibility-matrix.md)

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
