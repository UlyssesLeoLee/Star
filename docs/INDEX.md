# docs/ 导航索引 (INDEX)

> 2026-11-13 由 dedupe lane-b 生成。仅导航，不改变任何文档内容；状态: active = 现行，archive = 历史/审计证据（不重写）。

## 顶层规范文档（canonical 入口）
| 领域 | 入口 | 状态 | 说明 |
|---|---|---|---|
| 需求 | [requirements.md](requirements.md) + [requirements/](requirements/) | active | 主需求(统合版)；requirements/ 为各专题 SRS |
| 基本设计 | [basic-design.md](basic-design.md) + [basic-design/](basic-design/) | active | 主文件 + 专题增量，非重复 |
| 详细设计 | [detailed-design/](detailed-design/) + 各 *-design.md | active | api/data/security/runtime/integration/ai-agent/external/internal/operation-design |
| 数据设计 | [data-design.md](data-design.md) + [data-design/](data-design/) | active | db-design/ 仅 1 份补充表设计 |
| 前端设计 | [frontend-design.md](frontend-design.md) + [frontend/design/](frontend/design/) | active | frontend-internal-01..04 为其子文档 |
| 测试 / UAT | [test-design.md](test-design.md), [uat-design.md](uat-design.md), [uat-runbook.md](uat-runbook.md) | active | |
| 统一语言 | [ubiquitous-language.md](ubiquitous-language.md) | active | |
| 计划 | [implementation-plans/](implementation-plans/) | active | 现行 WBS/实施计划 |
| 计划(旧) | [plan/](plan/), [plans/](plans/) | archive | 2026-08/09 草案，见 refactor/dedupe-20261113-lane-b.md |
| 评审反馈 | [basic-design-feedback.md](basic-design-feedback.md), [detailed-design-feedback.md](detailed-design-feedback.md), [frontend-design-feedback.md](frontend-design-feedback.md) | archive | 审计证据 |

## 子目录
| 目录 | 用途 | 状态 |
|---|---|---|
| [adr/](adr/), [architecture/](architecture/) | 架构决策与架构视图 | active |
| [specs/](specs/) | 各 domain spec | active |
| [ddd/](ddd/) | 业务域边界 | active |
| [design/](design/) | 专题设计 | active |
| [deployment/](deployment/) | 部署 | active |
| [governance/](governance/), [guardian/](guardian/) | 治理 / 守门 | active |
| [arch-live/](arch-live/) | 架构文档站 | active |
| [refactor/](refactor/) | 重构/去重规划 | active |
| [rfcs/](rfcs/), [rfc/](rfc/) | RFC | active |
| [migrations/](migrations/) | 迁移说明 | active |
| [qa/](qa/), [pt/](pt/), [testing/](testing/), [uat/](uat/), [test-design/](test-design/) | 质量/测试 | active |
| [reports/](reports/) | 阶段/审计报告（证据，只增不改） | archive |
| [briefs/](briefs/) | 子代理 brief（证据） | archive |
| [reviews/](reviews/) | 评审记录 | archive |
| [recruitment/](recruitment/) | 5 域 Lead 寻访（已作废流程） | archive |
| [poc/](poc/), [prototypes/](prototypes/), [ecosystem-survey/](ecosystem-survey/), [mobile-flutter-mvp/](mobile-flutter-mvp/), [kanban-vmodel-jp/](kanban-vmodel-jp/) | PoC/调研 | archive |
| [wiki/](wiki/) | Wiki 内容 | active |
