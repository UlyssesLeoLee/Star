# Brief: DDD-AUDIT-01

**Agent**: explore
**Phase**: DDD-AUDIT
**Created**: 2026-10-25 03:25:56

---

## 目标

判定 STAR 仓（`D:/Star`）**域模型层是否齐全**，落到 crate 粒度并给证据。本 brief 是你这一路的判定范围与 rubric。

## 背景（父会话已实测，**不要重复跑这些统计**，直接进判定）

- `crates/domain-*` 共 **41** 个 crate；全仓另有 70+ 非 `domain-*` crate 承担域职责（`star-*` / `canvas-*` / `arg*` / `worktree-*` / `agent-*` / `*-engine`）
- 规模差异极大：`domain-agent` 10335 行 / 142 测试 / 11 trait，`domain-canvas` 仅 174 行 / 5 测试 / 0 trait
- 权威对照源（**你不需要去读**，只做代码判定）：`docs/ddd/`、`docs/data-design/ipa-detail/00-INVENTORY.md`、`docs/reports/STAR-P3-DDD-REVIEW-PHASE.md`

## 你的范围（只判这 14 个，其他 crate 归别路）

`crates/domain-agent`、`domain-agent-windows`、`domain-ai`、`domain-audit`、`domain-automation`、`domain-batch`、`domain-board`、`domain-canvas`、`domain-cli`、`domain-collaboration`、`domain-comment`、`domain-context`、`domain-dashboard`、`domain-development`

## 判定 rubric（逐 crate 逐条检查，命中记 Y/N）

- **R1 实体/值对象**：存在承载业务语义的 `struct`/`enum`（强类型 ID newtype、值对象），不是纯传输 DTO
- **R2 领域不变量**：存在校验 / 状态机 / 派生规则等**行为**方法，不是只有 getter/setter
- **R3 领域契约**：存在 `pub trait`（repository port / domain service / policy / 端口接口）
- **R4 领域事件**：存在 domain event 类型或 outbox 事件载荷
- **R5 测试**：有覆盖不变量的单元测试（`#[test]` / `#[tokio::test]`）
- **R6 非骨架自述**：crate 内**无**「阶段 1 骨架 / 0 业务方法 / 占位 / 仅 mock / TODO 实装 / 留 P1」一类自述

## 分类规则

- **齐全** = R1..R5 全命中 且 R6 = N
- **部分** = 命中 3-4 项
- **骨架** = 命中 ≤2 项，或 R6 = Y
- **不承载域模型** = 该 crate 实际是 adapter / 工具 / 传输层 → 明确写出判定理由与证据

## 硬约束

- **只读**：禁止修改、新建、删除任何仓内文件
- 每条判定必须给 `file:line` 证据；无证据的判定无效
- 禁止「大概 / 应该 / 看起来」；禁止编造未实际打开的文件与行号
- crate 自述与实际代码冲突时，**两者都记录**（自述说骨架但代码有实体，也照实写）
- 顺带记录：`crates/domain-canvas` 与 `crates/domain-task` 等极小 crate 的 `Cargo.toml` 注释里是否写明「阶段 1 骨架」（引用具体行）

## 交付物（只返回文本，**不要落盘**）

1. **汇总表**：`crate | LOC | 测试数 | R1 | R2 | R3 | R4 | R5 | R6 | 分类 | 一句话理由`
2. **缺口清单**：所有「部分 / 骨架」crate，每条写：缺什么 + 证据 `file:line` + 建议补齐顺序（先补哪个、为什么）
3. **反例清单**：设计文档声称有但代码没有 / 代码有但设计未登记的项（如果你顺带看到了）
4. **未能判定项**：写明原因

长度上限约 1200 字，表格用 markdown。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
