# ADR-0025: Vendor Adapter Anti-Contamination

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md)
> **关联**：[Compatibility Matrix 综合](../../../ecosystem-survey/compatibility-matrix.md) §4

---

## 1. 背景

§5 任务原文明确：**禁止在 STAR Core 出现 `ClaudeAdapter` / `CodexAdapter` / `GeminiAdapter` / `CursorAdapter` / `CopilotAdapter` / `VSCodeAdapter` / `JetBrainsAdapter` 等 vendor-specific 命名空间**。

但 vendor-specific 集成有真实需求（如 "让 Claude Code 跑 STAR 内置 IDE 时多展示一个 panel"）。问题是：**怎么放**？

## 2. 决策

**任何 vendor-specific 适配器只允许存在于 `Optional Integration Adapter` 子 crate，删除该 Adapter 后 STAR 核心功能必须仍然完整。**

### 2.1 禁止模式

```rust
// ❌ 禁止 — 污染 Core
fn submit_pr(provider: &str) -> Result<()> {
    if provider == "claude" {
        // Claude specific
    } else if provider == "openai" {
        // OpenAI specific
    }
    // ...
}
```

### 2.2 正确模式

```rust
// ✅ 正确 — Core 完全 vendor-neutral
trait PrSubmitter {
    fn submit(&self, pr: PullRequest) -> Result<()>;
}

struct CorePrSubmitter { /* 通用实现 */ }

// 在独立 crate 中加 vendor-specific
// crates/star-optional-claude-adapter/src/lib.rs
struct ClaudePanelEnhancer {
    core: Arc<dyn PrSubmitter>,  // 注入 Core 能力
    // Claude specific 字段
}
```

### 2.3 Cargo workspace 结构

```
star/                              # Core（无 vendor 命名）
├── crates/star-cli/
├── crates/star-domain/
├── crates/star-application/
├── crates/star-mcp/
├── crates/star-rest/
├── crates/star-audit/
├── crates/star-policy/
├── crates/star-ide-gateway/       # 抽象 IDE 接口
├── crates/star-ai-gateway/        # 抽象 AI 接口
├── crates/star-vcs/               # Version Control Provider
│   ├── src/gitgit.rs
│   ├── src/github.rs              # 公共 GitHub provider（任何 IDE 都能用）
│   ├── src/gitlab.rs
│   └── src/gitea.rs
├── crates/star-code-intelligence/ # Symbol/AST（不区分 IDE）
└── ...

star-optional/                     # Optional Integration Adapter
├── crates/star-optional-claude-panel/  # 给 Claude Code 加 panel（可选）
├── crates/star-optional-cursor-snippet/
├── crates/star-optional-vscode-bridge/
├── crates/star-optional-jetbrains-debug/
└── ...
```

### 2.4 验证规则

| 规则 | 验证方式 |
|---|---|
| Core 不含 vendor 命名空间 | `grep -rE "ClaudeAdapter|CodexAdapter|CursorAdapter|CopilotAdapter|VSCodeAdapter|JetBrainsAdapter" crates/star-*/` 应为空 |
| 删除 Optional Adapter 后 build 仍成功 | `cargo build --workspace --exclude star-optional-*` 必须 pass |
| 删除 Optional Adapter 后功能测试通过 | Phase D 的 Unknown Agent Test 不依赖任何 Optional Adapter |
| Core 不含 `if provider == "claude"` 模式 | `grep -rE 'if.*provider.*==.*"claude"' crates/star-*/` 应为空 |
| 决策路径不依赖 Provider | Code review 必查 + 自动化 linter |

## 3. 备选方案与拒绝理由

### 备选 A：所有 vendor 集成都进 Core
- 拒绝理由：违反 ADR-0021 + 任何 vendor 策略变化都会污染 Core

### 备选 B：完全禁止 vendor 集成
- 拒绝理由：失去用户体验优化机会；现实是很多用户用单一 vendor 工具

### 备选 C：vendor 集成放在一个巨型 `vendor-adapters` crate
- 拒绝理由：违反"删除后 Core 仍完整"原则；巨型 crate 容易产生跨 vendor 依赖

## 4. 后果

### 4.1 正面
- Core 100% vendor-neutral
- 任何 vendor 倒闭 / 改协议都不影响 Core
- Optional Adapter 独立迭代，激进实验不影响主流程

### 4.2 成本
- workspace 多了一层（`star-optional`）
- 必查的 linter 规则需在 CI 配
- "如何让 Core 能力被 Optional Adapter 用"需要稳定的 trait 边界

## 5. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
