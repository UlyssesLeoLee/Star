# Brief: DDD-AUDIT-04

**Agent**: explore
**Phase**: DDD-AUDIT
**Created**: 2026-10-25 03:25:56

---

## 目标

做一次**清单对齐审计**：把「设计文档声称的域 / 表」与「代码里真实存在的域 / 表」对齐，找出缺口与漂移。本 brief 是你这一路的范围。

## 背景（父会话已实测，**不要重复跑这些统计**）

- `crates/domain-*` = 41 个 crate；全仓另有 70+ 非 `domain-*` crate 承担域职责
- `db/migrations` = 93 个迁移文件
- `docs/data-design/ipa-detail/tables/` 下有逐表 W/T/M 分类文档；`docs/data-design/ipa-detail/00-INVENTORY.md` 是表清单
- 守门 #13（DB 三类横展开）要求：100% 表覆盖 + 每表 W/T/M 归类 + 派生 10 条 CW 规则
- 守门 #3（2026-09-01 11:45 JST 拍板）：**RGS 的「5 域」是历史治理命名，不等于 Star 仓的 DDD bounded context，且明令「不建立业务子域↔DDD 映射」** —— 你做对齐时**不得**建立这种映射，若发现仓内已有此类映射请点名报告

## 你的范围（只做这 4 件事）

### 1. 权威域清单
从设计文档（重点看 `docs/ddd/`、`docs/basic-design.md` 的域章节、`docs/architecture/` 下相关文档）抽出**仓内自己声明的域 / bounded context 清单**，给出：域名 + 出处 `file:line` + 该域声称的边界（一句）。若文档之间互相矛盾，列出矛盾点。

### 2. 域 ↔ crate 覆盖
把上一步的域清单与 `crates/domain-*`（41 个）以及非 `domain-*` 域职责 crate 对照，产出三类：
- **设计有、代码无**（缺口）
- **代码有、设计未登记**（漂移）
- **两边都有但命名不一致**（映射候选，注意：只列候选，不要建映射表）

### 3. 表覆盖核对（守门 #13 口径）
- `db/migrations` 实际建的表 vs `docs/data-design/ipa-detail/tables/` 下的分类文档 vs `00-INVENTORY.md` 清单，三者对不齐的地方列出来（哪个表有迁移无分类 / 有分类无迁移 / 清单与实际表名不符）
- 抽样核 3-5 张表的分类文档是否真的含 W/T/M 归类 + RLS 13 类 + retention/审计要求（给 file:line）
- 结论：**100% 覆盖是否属实**

### 4. DDD Review 现状
读 `docs/reports/STAR-P3-DDD-REVIEW-PHASE.md` 与 `docs/reports/DDD-LEAD-REVIEW-PROCESS.md`，说明 DDD Review 流程当前处于什么阶段、已覆盖哪些域、未覆盖哪些域（给 file:line）。**注意**：`docs/briefs/AR-ATLAS-*.md` 与 `docs/gemarc-5d-atlas/` 是另一路并发会话的未跟踪产出，只能作参考引用，不得当作权威结论。

## 硬约束

- **只读**：禁止修改、新建、删除任何仓内文件
- 每条结论必须给 `file:line` 证据；无证据的结论无效
- 禁止「大概 / 应该 / 看起来」；禁止编造未实际打开的文件与行号
- **禁止**建立「RGS 5 域 ↔ Star DDD bounded context」的映射（守门 #3 拍板）
- 数量类结论（多少表、多少域、覆盖率%）必须给出你的计数方法与可复算依据，不得凭印象

## 交付物（只返回文本，**不要落盘**）

1. **权威域清单表**：`域名 | 出处 file:line | 边界一句话`
2. **三分类覆盖结果**（设计有代码无 / 代码有设计未登记 / 命名不一致候选）
3. **表覆盖核对结论**：三方差集数量 + 100% 覆盖是否属实 + 抽样证据
4. **DDD Review 现状** + 未覆盖域清单
5. **未能判定项** + 原因

长度上限约 1500 字，表格用 markdown。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
