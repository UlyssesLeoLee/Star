# Requirements Thread C — Handoff

> **日期**: 2026-08-29
> **范围**: `docs/requirements.md`，brainstorming 线程 C（瀑布式 SIer 项目支持：设计书 / 测试工程分级 / 生产事件追溯）

## 1. 新增内容

延续线程 A（核心开发闭环）、线程 B（Review/自审交叉审核）已确立的模式，本线程新增三节，均标注"无对应原提示词章节编号 — 本节为线程 C 新增设计"：

| 章节 | 对象 | Requirement ID | 说明 |
|---|---|---|---|
| §8.3 Design Artifact | `DesignArtifact` | DSG-001/002 | 设计书作为可版本化、可审批对象；复用 §27.4 `ReviewRecord` 与 §8.2 既有 `RequireApproval` Guard，不新建审批状态机 |
| §27.6 Test Level | `ValidationResult.Level` | TST-001/002 | 单体/结合/总合/受入 维度，与既有 `Type` 字段正交；扩展 §27.2 Acceptance Coverage 按 Level 过滤，明确拒绝新建 TestPlan/TestCase 平行对象体系 |
| §29.1 Incident Record | `IncidentRecord` | OPS-001/002/003 | 生产事件追溯，挂接既有 WorkItem→Worktree→ChangeSet→ValidationResult 闭环；开篇即声明与 §30.6 Non-Goals 的边界（不监控、不告警、不自动回滚/修复） |

同步更新：§36（4 条新增 UC-DEV 用例）、§39（Traceability Model 叙事，含 Design Artifact 前置分支与 Incident Record 回溯分支）、§41.1/§41.2（新 ID 前缀与 P0 登记表行）、§47（下一阶段输入清单，补齐 RVW-xxx / DSG-xxx / TST-xxx / OPS-xxx）。

## 2. 自审发现并修复的问题

对 `git diff` 做结构性自审（代码块闭合、章节编号、交叉引用）后，又做了一轮跨章节一致性复核，发现并修复：

1. **§27.4 `ReviewRecord` 字段未跟随 §8.3 泛化** — §8.3 声称 `ChangeSetId` 已泛化为 `ChangeSet | DesignArtifact` 二选一，但 §27.4 的字段块仍写死 `ChangeSetId` 且 `Author` 定义仍绑定"ChangeSet 归属者"。已改为 `Target: ChangeSet | DesignArtifact` 字段 + 泛化后的 `Author` 定义。
2. **§30 Roadmap 缺失线程 C 三个 P0 的分级位置** — §41.2 登记为 P0，但 §30 MVP/V1/V2 完全没有对应条目。已加入 §30.3 V1 Should Have（均为可选能力，非强制瀑布，故不进 §30.2 Must Have），每条附一行"为何不是 MVP"的理由。
3. **§47 `ARCH-OBL-DEV-001~006` 陈旧** — 线程 B 已在 §35 新增 `ARCH-OBL-DEV-007`（Review Segregation of Duties），§47 引用范围未同步。已改为 `~007`。
4. **§42 核心模型图 E（Traceability Model）与 §39 叙述脱节** — §39 声称"对应第 96 章 E 图"，但 §42 的简化图从未纳入线程 B 的 `Review Record` 节点，线程 C 的 `Design Artifact`/`Incident Record` 分支也未反映。已在 §42 E 图补入 `Review Record` 主链节点，以及 `Design Artifact`（前置可选）、`Incident Record`（回溯可选）两个分支，并加一行说明"本图为主链简化版，完整链条见 §39"。
5. **§29.1 REQ-OPS-003 的 Webhook 来源未指向既有机制** — 原文只写"受限的、明确声明来源的外部 Webhook"，未说明走哪个通道。已改为明确指向 §18 Integration 既有 Webhook 机制，避免读者以为要新建入站接口。

## 3. 未变更事项

线程 A、线程 B 已有内容语义未做任何修改，仅新增交叉引用（§27.4 字段块的改动是"补齐泛化"而非"改变泛化决策"——泛化决策本身在 §8.3 首次落地时已经声明，此次只是让 §27.4 的对象定义追上该声明）。

## 4. 下一步

`docs/requirements.md` §47 已列出下一阶段（《基本设计书》）应继承的完整输入清单，含本线程新增的 DSG-xxx / TST-xxx / OPS-xxx。本文档到此为止，不进入生产代码编写（§105 既有约束）。
