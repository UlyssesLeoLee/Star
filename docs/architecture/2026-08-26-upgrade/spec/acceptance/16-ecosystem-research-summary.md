# 53. AI / IDE Ecosystem Research Summary

> **状态**：🟡 草案 v0.2
> **日期**：2026-08-26

## 1. 调研范围

- 7 款 Coding Agent（per [ai-compatibility-matrix.md](../../../../ecosystem-survey/ai-compatibility-matrix.md)）
  - **7 款** (per [arch/03 §1 总体架构](../../arch/03-star-ai-compat-arch.md)): Codex · Claude Code · Gemini CLI · Copilot · Cursor · VS Code · Junie
- 6 款 IDE / CDE（per [ide-compatibility-matrix.md](../../../../ecosystem-survey/ide-compatibility-matrix.md)）
- 4 套协议 + 2 套工具（per [protocol-survey.md](../../../../ecosystem-survey/protocol-survey.md)）

### 1.1 Phase D 实测 4 款明确指定（per P2-12 修复 2026-08-27）

> 原 §1 / §2.3 表 7 款只列了能力覆盖数字，但**未指定** Phase D 实测哪 4 款。修法：显式列 4 款（per 子代理 C P2-12 修复）。

**Phase D 实测 4 款**（per NFR-AI-001 "7 款主流 Agent 中 4 款实测通过"）：

| 序 | Agent | 厂商 | 类型 | 选择理由 |
|---|---|---|---|---|
| 1 | **Claude Code** | Anthropic | 终端级 coding agent | 终端级代表；7 款里独立终端型最强的 |
| 2 | **Cursor** | Anysphere | IDE 集成型 coding agent | IDE 集成型代表；用户基数最大 |
| 3 | **Junie** | JetBrains | IDE 集成型 coding agent | JetBrains 生态代表；IDE 集成型 2 选 1 配合 Cursor 验证跨 IDE 集成型适配 |
| 4 | **Codex** | OpenAI | 终端级 coding agent | OpenAI 代表；跟 Claude Code 形成对照（终端级 2 选） |

> **第 5 款备选** (Phase D 兜底): Gemini CLI（per 2026-06-18 个人账户停服事件验证 Zero Vendor Cooperation 价值）。如 4 款里有 1 款在 Phase D 期间停服，第 5 款替补上 4 款实测。

> **未实测 3 款**（per NFR-AI-001 "7 款中至少 4 款"）：
> - Gemini CLI（2026-06-18 个人账户停服已作 Phase D 反面教材，per §2.2）
> - Copilot（终端级，已被 Claude Code + Codex 覆盖）
> - VS Code（IDE 集成型已被 Cursor + Junie 覆盖；VS Code Agent Mode 较新留 Phase 2+ 再评估）

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

### 2.3 能力覆盖（per 7 款名称 = §1 列出）

| 能力 | 7/7 Agent | 6/6 IDE |
|---|---|---|
| Git CLI | ✅ | ✅ |
| Shell | ✅ | ✅ |
| FS | ✅ | ✅ |
| AGENTS.md | 5/7 | 6/6 |
| MCP | 6/7 | 4/6 |
| LSP native | 2/7 | 6/6 |
| Worktree | 5/7 | n/a |

> 7 款 = Codex · Claude Code · Gemini CLI · Copilot · Cursor · VS Code · Junie（per [arch/03 §1](../../arch/03-star-ai-compat-arch.md)）。

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

- 7 款 Agent：[ai-compatibility-matrix.md](../../../../ecosystem-survey/ai-compatibility-matrix.md)
- 6 款 IDE：[ide-compatibility-matrix.md](../../../../ecosystem-survey/ide-compatibility-matrix.md)
- 4 套协议：[protocol-survey.md](../../../../ecosystem-survey/protocol-survey.md)
- 综合矩阵：[compatibility-matrix.md](../../../../ecosystem-survey/compatibility-matrix.md)

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：7/6/4 调研范围 + 3 节关键发现 + 4 详细资料 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P2-12：§1 列出 7 款 Agent 名称（Codex / Claude Code / Gemini CLI / Copilot / Cursor / VS Code / Junie, per arch/03 §1）· §1.1 新增 Phase D 实测 4 款明确指定（Claude Code / Cursor / Junie / Codex, 第 5 款备选 Gemini CLI）· §2.3 表标 7 款名称 | 8 子代理 INTERFACE-REVIEW-C P2-12 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-16 (P2-12)
