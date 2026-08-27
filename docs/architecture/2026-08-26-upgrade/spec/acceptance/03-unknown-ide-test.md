# 40. Unknown IDE Test

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/acceptance/01-unknown-agent-test.md](01-unknown-agent-test.md) · [spec/rest/01-rest-strategy.md §4 关键 endpoint 草案](../rest/01-rest-strategy.md) · [arch/05 §5 REST API（OpenAPI 3.1）](../../arch/05-gitgit-compat-arch.md)

## 1. 目标（per §44 任务原文）

测试一个没有 STAR 专用插件的 IDE 是否可以通过标准能力接入 STAR。

## 2. 测试条件

只提供：
- Git
- Shell
- Repository
- AGENTS.md
- star CLI
- **OpenAPI（6 项最低能力 — per P2-2 修复 2026-08-27 显式列出）**：
  1. `GET /api/v1/agent/capabilities` — Capability Discovery
  2. `GET /api/v1/agent/permissions` — Permission Discovery
  3. `GET /api/v1/agent/instructions` — Agent Instructions
  4. `GET /api/v1/tasks/current` — Current Task
  5. `GET /api/v1/workspaces/current` — Current Workspace
  6. `GET /api/v1/code/search?q=...` — Code Search

> 6 项 = MVP 14 endpoint 子集（per [arch/05 §5 MVP 12 子集边界](../../arch/05-gitgit-compat-arch.md) + [spec/rest/01 §4](../rest/01-rest-strategy.md)）+ 3 项 agent-* 元信息 endpoint。完整 endpoint 清单见 [arch/05 §5](../../arch/05-gitgit-compat-arch.md)。

## 3. 测试场景

```text
打开 Repository
   ↓
发现 STAR (读 AGENTS.md)
   ↓
通过 OpenAPI 拉取 capabilities / permissions / instructions（per P2-2 修复 2026-08-27 补 1 步）
   ↓
获取当前 Task (OpenAPI: GET /api/v1/tasks/current)
   ↓
获取 Context (OpenAPI: GET /api/v1/workspaces/current)
   ↓
搜索代码 (OpenAPI: GET /api/v1/code/search)
   ↓
定位符号 (OpenAPI: GET /api/v1/code/symbols/{name})
   ↓
修改文件
   ↓
运行测试
   ↓
创建 Commit
   ↓
创建 MR
```

> 10 步 = 9 步原版 + 1 步新增（per P2-2 修复 2026-08-27）"通过 OpenAPI 拉取 capabilities / permissions / instructions"。原 §3 文字提"OpenAPI"但 10 步里**未消费** OpenAPI（per 子代理 C P2-2 弱信号），修法：插入独立一步显式消费 6 项 OpenAPI 最低能力的代表 3 项（capabilities / permissions / instructions）。

## 4. OpenAPI 6 项最低能力 → 10 步消费表（per P2-2 修复 2026-08-27）

| 步 | 行为 | 消费的 OpenAPI 最低能力 | 对应 §2 列项 |
|---|---|---|---|
| 1 | 打开 Repository | — | — |
| 2 | 发现 STAR (读 AGENTS.md) | — | — |
| 3 | 拉取 capabilities / permissions / instructions | `GET /api/v1/agent/capabilities` + `GET /api/v1/agent/permissions` + `GET /api/v1/agent/instructions` | 1 / 2 / 3 |
| 4 | 获取当前 Task | `GET /api/v1/tasks/current` | 4 |
| 5 | 获取 Context（workspace） | `GET /api/v1/workspaces/current` | 5 |
| 6 | 搜索代码 | `GET /api/v1/code/search?q=...` | 6 |
| 7 | 定位符号 | `GET /api/v1/code/symbols/{name}` | (arch/05 完整 endpoint) |
| 8 | 修改文件 | — | — |
| 9 | 运行测试 | — | — |
| 10 | 创建 Commit | `POST /api/v1/repos/{owner}/{name}/worktrees` (per arch/05 MVP 12 endpoint) | (arch/05 完整 endpoint) |
| 11 | 创建 MR | `POST /api/v1/merge-requests` (per rest/01 §4) | (rest/01 完整 endpoint) |

> 表中步 3 / 4 / 5 / 6 = §2 列的 6 项最低能力（4 项直接消费，2 项在后续步消费）；步 7 / 10 / 11 消费 6 项之外的能力。**全部 10 步独立可跑（不依赖任何 IDE 专用 plugin）** = 验收闭环。

## 5. 通过条件

- 10 步全部完成
- IDE 不需要 STAR 专用 plugin
- 通过 Git + Shell + OpenAPI 标准能力完成（OpenAPI 至少 6 项最低能力被显式消费，per §2 + §4 表）
- 错误响应**全部**走 `agent-api/v1#Error` 6 字段 schema（per P1-G 修复 2026-08-27，per [spec/rest/01 §4](../rest/01-rest-strategy.md)）

## 6. 如果必须等 IDE 厂商开发 STAR Plugin

测试失败 → 架构设计失败。

## 7. 实施位置

- `tests/unknown-ide/` — Test harness
- `tests/unknown-ide/run.sh` — Test runner
- `tests/unknown-ide/openapi-consumer/` — 6 项最低能力 OpenAPI consumer harness（per P2-2 修复 2026-08-27）
- 至少 3 轮测试（每轮 1 个不同 IDE）

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：10 步测试场景 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P2-2：§2 显式列 OpenAPI 6 项最低能力（capabilities / permissions / instructions / task current / workspace current / code search） · §3 10 步新增"通过 OpenAPI 拉取 capabilities/permissions/instructions" 1 步 · §4 新增 OpenAPI 6 项 → 10 步消费表 · §7 实施位置加 `tests/unknown-ide/openapi-consumer/` | 8 子代理 INTERFACE-REVIEW-C P2-2 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-3 (P2-2)
