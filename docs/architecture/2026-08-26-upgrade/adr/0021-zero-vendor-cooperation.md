# ADR-0021: Zero Vendor Cooperation Principle

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **关联**：[AI Compatibility Matrix](../ecosystem-survey/ai-compatibility-matrix.md) · [IDE Compatibility Matrix](../ecosystem-survey/ide-compatibility-matrix.md) · [Compatibility Matrix 综合](../ecosystem-survey/compatibility-matrix.md) · [Protocol Survey](../ecosystem-survey/protocol-survey.md)

---

## 1. 背景与问题

设计 STAR × GitGit 的 AI/IDE 兼容性架构时，存在两条根本不同的路径：

1. **Vendor Cooperation 路径** — 假设 OpenAI / Anthropic / Google / Microsoft / GitHub / Cursor / JetBrains 之一会为 STAR / GitGit 写适配代码（官方 plugin、SDK、training data 包含 STAR 知识等）
2. **Zero Vendor Cooperation 路径** — 假设上述所有厂商**永远**不会为 STAR / GitGit 写适配代码

业界生态事件（per 2026-08-26 调研）：
- 2025-10-15 Gitpod Classic 突然关停，用户被迫迁移
- 2026-06-18 Gemini CLI 个人账户突然停止服务
- 2026-07-28 MCP 规范大改（stateless core / 多协议弃用），12 个月迁移窗口
- 2025-12 Claude Code / Kiro CLI 引入 native LSP，但 Codex / Gemini / Copilot 至今未跟进

这证明：**任何 Vendor 可能在任意时刻改协议、停服、改商业策略**。依赖 Vendor 合作的设计会随时崩塌。

## 2. 决策

**采用 Zero Vendor Cooperation Principle 作为 STAR × GitGit 架构的最高原则。**

> **正式声明**：永远假设 OpenAI、Anthropic、Google、Microsoft、GitHub、Cursor、JetBrains 以及未来任何 AI / IDE / Agent 厂商都不会主动为 STAR 或 GitGit 编写任何适配代码。

### 2.1 不得依赖

- ❌ 官方 STAR Plugin
- ❌ 官方 GitGit Plugin
- ❌ 厂商专用 STAR SDK
- ❌ 厂商修改 Agent 来支持 STAR
- ❌ 厂商增加 STAR Tool
- ❌ 模型训练数据提前包含 STAR
- ❌ AI 厂商与 STAR 建立商业合作
- ❌ IDE 厂商为 STAR 增加内置支持

### 2.2 必须支持

- ✅ 任何具备 Git + Shell + FS 能力的 Coding Agent
- ✅ 任何具备 Git + Shell + FS + Terminal 能力的 IDE
- ✅ Markdown（AGENTS.md bootstrap）
- ✅ MCP 2026-07-28（增强层）
- ✅ OpenAPI 3.1（机器可读 REST）
- ✅ 标准 Git 协议（GitGit 暴露层）

### 2.3 通过标准化确保可发现

- AGENTS.md 是事实标准（20+ 工具读）
- MCP 2026-07-28 是事实标准（6/7 主流 Agent 客户端支持）
- OpenAPI 3.1 是事实标准
- Git + Shell + FS + Terminal 是 50 年基石

## 3. 备选方案与拒绝理由

### 备选 A：Vendor Cooperation 路径
- 拒绝理由：商业关系不稳定、生态事件证明 vendor 可随时断供、不可作为架构基线
- 仅适用场景：商业互操作（如"STAR 接入 GitHub Copilot for Business"），不进入 Core

### 备选 B：自建全栈（不依赖任何外部标准）
- 拒绝理由：会失去 20+ Agent / 6+ IDE 的现成客户端
- 仅适用场景：明确知道生态不会变化的小范围内部工具

## 4. 后果与影响

### 4.1 正面
- STAR Core 完全 vendor-neutral（per ADR-0025）
- 任意 Coding Agent / IDE 可立即接入（per §50 验收问）
- 不被 vendor 商业策略绑架
- 与 20+ 工具的现成客户端兼容

### 4.2 负面 / 成本
- 不得"等"厂商支持；任何"专属"集成需求必须降级为通用协议
- 必须双轨实现：增强层（MCP / OpenAPI）+ 兜底层（Git + Shell + FS）
- 文档 / 测试 / 培训必须把"零假设"作为硬约束

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 未来某工具连 Git 都不支持（极端） | 极低 | 高 | Universal Submit 必须保留"纯 Web"降级（per §39 最后一档，但禁止） |
| 现有工具集突然改协议 | 中 | 中 | Fallback Ladder 4 级（per §38）；AGENTS.md 自我描述 |
| Zero Knowledge Agent Test 无法跑通 | 中 | 高 | Phase D 必须有完整 demo 验证（per §42-§44） |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0022 IDE Placement](0022-ide-placement.md) — 决定 IDE 能力归属
- **依赖**：[ADR-0023 Version Control Provider](0023-version-control-provider.md) — 决定 GitGit 与 GitHub/GitLab 并列
- **依赖**：[ADR-0024 IDE Session Identity](0024-ide-session-identity.md) — IDE Session 不污染 GitGit
- **依赖**：[ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md) — Core 不允许 vendor-specific 逻辑
- **被依赖**：[ADR-0026 STAR AI Compatibility Architecture](0026-star-ai-compat-arch.md)（起草中）

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
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草（per 2026-08-26 升级 Plan） |
