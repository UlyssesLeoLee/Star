# ADR-0026: STAR AI 兼容性架构

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) · [Protocol Survey](../ecosystem-survey/protocol-survey.md)
> **关联**：[arch/03 STAR AI Compat Arch](../architecture/2026-08-26-upgrade/arch/03-star-ai-compat-arch.md) · [AI Compatibility Matrix](../ecosystem-survey/ai-compatibility-matrix.md)

---

## 1. 背景与问题

STAR 在 2026-08-26 之前没有面向 AI Coding Agent 的标准化接入路径：

- 25 Module crate workspace（main=5181288）只有内部领域 API，外部 Agent 无法 discovery
- 没有任何一层暴露"机器可读 + 稳定 schema"的接口
- 7 款主流 Coding Agent（Codex / Claude Code / Gemini CLI / Copilot / Cursor / Junie / Kiro CLI）中 6 款支持 MCP，STAR 完全错失
- Agent 接入 STAR 只能靠"人写 wrapper"或"vendor 改 SDK"，违反 ADR-0021 Zero Vendor Cooperation 原则

需要一套**Vendor-Neutral** 的 AI 兼容性架构，让任意具备 Git + Shell + FS 能力的 Coding Agent 都能自动接入 STAR。

## 2. 决策

**采用 5 接入通道 + Fallback Ladder 4 级 作为 STAR AI 兼容性架构。**

### 2.1 5 接入通道（per arch/03 §2）

| 通道 | 角色 | 关键约束 |
|---|---|---|
| **Git Compatibility** | 兜底层（L4） | 标准 `git clone` / `push` / `worktree` 100% 走 GitGit（当 GitGit 是 Provider 时） |
| **Shell / CLI** (`star` CLI) | 推荐层（L2） | MVP 17 核心命令 + 11 扩展命令；`--json` 稳定 schema = `agent-api/v1` |
| **MCP Server** (2026-07-28) | 增强层（L1） | 16 个领域语义 tools（per P1-F 修复 2026-08-27：含 `submit`）；stdio transport |
| **REST + OpenAPI 3.1** | 远程/集成层（L3） | Web UI / Automation / External Agent；跟 MCP 共享同一 Domain API |
| **AGENTS.md Bootstrap** | vendor-neutral 引导 | 极薄（≤ 50 行）；仅 3 个最小命令（capabilities / task / submit） |

### 2.2 Fallback Ladder 4 级（per arch/03 §3）

```
Level 1: MCP + CLI + Git + AGENTS.md      (推荐入口)
   ↓
Level 2: CLI + Git + AGENTS.md            (MCP 不可用)
   ↓
Level 3: REST + Git + AGENTS.md           (CLI 不可用)
   ↓
Level 4: Git Only                         (所有抽象都不可用)
```

每一级都**必须**能跑通 Unknown Agent Test（per acceptance/01 §3）。

### 2.3 关键架构约束

- `star` CLI 是 `git` 的 superset（per P1-H 修复 2026-08-27）— 提供 `git` 不具备的领域操作 + 包装 `git` 子命令（diff / commit / push）注入 Policy / Audit / Worktree 上下文
- MCP server **不**暴露 `update_issue_table` 等内部表操作；只表达领域操作
- 2026-07-28 规范 + stdio transport（Rust SDK beta 风险规避，per ADR-0032）
- tool list 排序按 name 字典序 + metadata 含 `ttlMs` / `cacheScope`
- Optional Vendor Adapter 独立 crate（per ADR-0025）— Core 100% vendor-neutral

## 3. 备选方案与拒绝理由

### 备选 A：Vendor Cooperation 路径
- 拒绝理由：依赖 OpenAI / Anthropic / GitHub 等厂商主动为 STAR 写适配代码；与 ADR-0021 直接冲突

### 备选 B：自建全栈（不依赖任何外部标准）
- 拒绝理由：会失去 20+ Agent / 6+ IDE 的现成客户端；维护成本爆炸

### 备选 C：只做 MCP（不实现 Git / CLI / REST）
- 拒绝理由：MCP 2026-07-28 仍在演进（per 2026-08-26 调研：12 个月迁移窗口）；需要 L4 Git 兜底

## 4. 后果与影响

### 4.1 正面

- 任意具备 Git + Shell + FS 能力的 Coding Agent 立即接入（per arch/03 §7 验收）
- 7 款主流 Agent 中至少 4 款实测可接入（per AI Compatibility Matrix）
- 架构完全 vendor-neutral，符合 ADR-0021 + ADR-0025
- Fallback Ladder 4 级确保任何单点抽象失效都不阻断 Agent

### 4.2 负面 / 成本

- 必须双轨实现：增强层（MCP / OpenAPI / CLI）+ 兜底层（Git + Shell + FS）
- 5 接入通道的 schema 稳定化需要长期投入（per acceptance/13 Schema Stability）
- 文档 / 测试 / 培训必须把"5 通道 + 4 级 Fallback"作为硬约束

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| MCP 规范再次大改 | 中 | 中 | Fallback Ladder L2/L3/L4 不依赖 MCP |
| 某款 Agent 5 通道都不支持 | 极低 | 中 | 保持 Git + Shell + FS 兜底（L4） |
| CLI 17 核心命令破坏 schema | 中 | 高 | `agent-api/v1` 稳定化 + 旧 schema 至少 12 个月兼容 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 最高原则
- **依赖**：[ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md) — Core 100% vendor-neutral
- **被依赖**：[ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md)（起草中）
- **被依赖**：[ADR-0028 GitGit Compatibility](0028-gitgit-compat.md)（起草中）
- **被依赖**：[ADR-0029 Universal Submit Protocol](0029-universal-submit.md)（起草中）
- **被依赖**：[ADR-0032 MCP Transport 选型（stdio）](0032-mcp-transport-stdio.md)（起草中）

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
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：5 接入通道 + Fallback Ladder 4 级 | Phase B 起草（per 2026-08-26 升级 Plan） |
