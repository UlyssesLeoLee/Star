# 38. Unknown Agent Test

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/vcs/04-fallback-strategy.md](../vcs/04-fallback-strategy.md) · [arch/03 §3 Fallback Ladder](../../arch/03-star-ai-compat-arch.md) · [arch/03 §7 验收（per P1-K 修复）](../../arch/03-star-ai-compat-arch.md)

## 1. 目标

测试 AI 不允许拥有 STAR 训练数据、专用 Plugin、SDK、Adapter；只提供 Git + Shell + Repository + AGENTS.md；测试它能否自己发现 STAR 并完成软件开发任务。

## 2. 测试条件

- 禁止：STAR 训练数据、STAR 专用 Plugin、STAR SDK、STAR Adapter
- 必须有：Git、Shell、Repository、AGENTS.md
- 默认有：MCP server、star CLI（per §3 Level 1 默认环境）
- 显式降级：撤掉 star CLI 走 §4 Level 4 路径

## 3. 测试场景 — Level 1 默认（per P1-K 修复 2026-08-27 / P1-1 修复 2026-08-27）

> **冲突来源**（per 子代理 C P1-1）：原 arch/03 §7 写"必须只用 Level 4 通过"但本节 16 步用了大量 `star` CLI（步骤 4-15），`star` CLI 属 Level 2+ 能力，**不可同时成立**。修法（per P1-K）：Unknown Agent Test 跑 Level 1（用 star CLI），Level 2/3/4 单独跑 conformance（per [spec/vcs/04 §5](../vcs/04-fallback-strategy.md)）。

```text
1. Agent clone GitGit repository
2. 读 AGENTS.md
3. 发现 `star` CLI
4. star agent capabilities
5. 读任务: star task current --json
6. 获取 context: star context current --json
7. 搜索代码: star code search "..." --json
8. 定位符号: star code symbol "..." --json
9. 创建 workspace: star workspace create STAR-N
10. 创建 worktree: star worktree create STAR-N
11. 修改代码
12. 测试: star test affected
13. Commit (标准 git commit)
14. star submit
15. MR 自动创建
16. STAR 更新 Issue 状态
```

> 本 16 步 = Level 1 强约束（必须 MCP + CLI + Git + AGENTS.md 全可用）。任何 Level 2/3/4 不在本测试范围，**单独**由 [spec/vcs/04 §5 conformance 测试](../vcs/04-fallback-strategy.md) 覆盖。

## 4. Level 4 降级路径 — Git Only（per P1-1 修复 2026-08-27 显式分两段）

> **触发条件**：star CLI / MCP server / REST API 全部不可用，但 Git 协议仍工作。Agent 通过 `git push` 提交到 worktree 分支，由 GitGit 的 receive-pack 钩子触发 Universal Submit（per R-004 缓解措施）。

```text
1. Agent clone GitGit repository
2. 读 AGENTS.md
3. 发现 GitGit（即使无 star CLI，AGENTS.md 里 git-only 提示可用）
4. git worktree add ../worktree-STAR-N
5. 修改代码
6. 标准 git commit
7. git push origin worktree-STAR-N
8. GitGit receive-pack 钩子触发 STAR 自动化 Pipeline（test + submit + MR）
9. STAR 更新 Issue 状态
```

> 本 9 步 = Level 4 强约束（仅 Git 协议）。与 §3 Level 1 16 步**不重叠**：§3 测"Agent 通过 star CLI + AGENTS.md 自主发现"；§4 测"Agent 退化到纯 Git 也能完成关键提交"。两段独立验证，**全部通过**才算 Unknown Agent Test 验收闭环。

## 5. Level 1 ↔ Level 4 边界表（per P1-1 修复 2026-08-27）

| 维度 | Level 1（§3 默认） | Level 4（§4 降级） |
|---|---|---|
| 必含能力 | MCP + CLI + Git + AGENTS.md | Git Only |
| Agent 必跑步骤 | 16 步 | 9 步 |
| 提交方式 | `star submit` (Universal Submit 12 步) | `git push` → receive-pack 钩子 |
| AGENTS.md 期望内容 | 3 个最小命令（capabilities / task / submit） | git-only 提示 + `star` 不可用时回退说明 |
| 验收证据 | Phase D Unknown Agent Test Level 1 pass | Phase D Unknown Agent Test Level 4 pass |
| 适用场景 | 默认环境（MCP + CLI 都在） | 全部抽象层 down 时的兜底 |
| 跟 arch/03 §3 映射 | L1 (MCP+CLI+Git+AGENTS.md) | L4 (Git Only) |

> **注意**：Level 2 / Level 3 **不属**本测试，由 [spec/vcs/04 §5](../vcs/04-fallback-strategy.md) 单独跑 conformance。

## 6. 通过标准

- 步骤 1-16 全部完成（Level 1，§3）
- 步骤 1-9 全部完成（Level 4，§4）
- 两段独立验证，**全部通过**才算验收闭环
- 不修改 STAR Core / GitGit Core
- 不写 AI 厂商适配器
- 测试环境**不**联网（无外部 AI 服务）— 测的是**本地 agent** 或 **mock agent**，非真实 7 款主流 Agent 之一（真实 Agent 实测由 [spec/acceptance/16 §2.3](../16-ecosystem-research-summary.md) per P2-12 修复选定 4 款 + Phase D 实测）

## 7. 实施位置

- `tests/unknown-agent/` — Test harness
- `tests/unknown-agent/run.sh` — Test runner
- `tests/unknown-agent/level1.sh` — Level 1 默认测试（per §3 16 步）
- `tests/unknown-agent/level4.sh` — Level 4 降级测试（per §4 9 步）
- 至少 3 轮测试（每轮 1 个不同 agent 实现）

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：16 步单一路径 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-1：§3 标 Level 1 默认 + §4 新增 Level 4 降级 9 步 + §5 加 Level 1↔4 边界表 + §6 通过标准改"两段全通过" + §7 实施位置分 level1.sh / level4.sh · 跟 arch/03 §7 P1-K 修复对齐 | 8 子代理 INTERFACE-REVIEW-C P1-1 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-1 (P1-1)
