# 基本設計書 改善計画 v0.2-prep

> **回应**：`docs/basic-design-feedback.md`（审核者：Claude 本会话，8 Finding + 1 笔误，无 Blocker）
> **目标文件**：`docs/basic-design.md`（v0.1，3550 行）
> **执行模式**：开 worktree 修改 → 自审 → merge `--no-ff` → 清理
> **状态**：等待用户确认执行

---

## 1. 反馈要点回顾

| 类别 | Finding | 根因 |
|---|---|---|
| Major | F-01 §2.1 算术错误（19+5=24 ≠ 上游 22 个）| 推导口径不清 |
| Major | F-02 "Development Context" 上游 Domain 下落不明 | 合并/拆分推导未显式 |
| Major | F-03 `domain-local-runtime` 总览章节缺失 | 命名/边界不一致 |
| Minor | F-04 5 处 §N(>47) 引用未对齐文档约定 | 编号空间混用 |
| Major | F-05 WorkItem 默认状态机违反 REQ-WF-001 三态 | 默认值口径漂移 |
| Minor | F-06 tenant_id 计数 12 vs 表格 13 | 局部计数错 |
| Major | F-07 `domain-local-runtime` 命名与 §8.5 冲突 | F-03 同根因 |
| Minor | F-08 AgentSession 状态数 13 vs 实际 14 | 局部计数错 |
| 笔误 | §2.1 表 6 行 `REQ-AUT-001` → `VAL-001` | ID 笔误 |

**根因归类**（来自 feedback 第 109-113 行）：
1. **Module 枚举与详细设计不同步**（F-03/F-07 + F-01/F-02 联动）— 最高优先级
2. **状态机默认值口径漂移**（F-05 + F-08）— 次高
3. **局部计数/引用笔误**（F-04/F-06/笔误）— 不阻断

**作者自评（feedback 第 115 行）**：范围裁剪 / K8s 纪律 / MVP 可追溯性三个维度执行严谨；问题集中在 **Module 边界自洽性** 与 **状态机默认值口径**。

---

## 2. 批次 A：必做（Major + 根因级）

### A.1 F-03 + F-07 合并：`domain-local-runtime` 边界明确化

**问题**：`domain-local-runtime` 在总览章节（§1.2 / §2.1 / §2.3 / 附录 B / 附录 C）系统性遗漏，在详细设计章节（§4.6 / §4.10.4 / §6.1 / §8.5 / 接口稳定承诺）被当作一等 Module。Module 总数 24 与实际 25 自相矛盾；且命名与 §1.1 "Local Daemon" 二进制未做区分。

**修复动作**（7 处）：

| # | 位置 | 动作 |
|---|---|---|
| 1 | §2.1 表格 | 新增第 25 行：`domain-local-runtime` — 职责"Runtime 注册表/Port（管理集群外 Local Daemon 生命周期）"、实体"Runtime, RuntimeCommand, RuntimeObservation"、不变量"Local Daemon 二进制不属此 crate"、依赖"domain-worktree" |
| 2 | §2.1 标题 | 改为"完整 Domain 列表（继承 §6 共 22 个 + 3 个拆分/合并 = 25 个逻辑 Module）" |
| 3 | §1.2 mermaid 逻辑架构图 | 新增 node `domain-local-runtime`，在 `domain-worktree` 附近 |
| 4 | §2.3 依赖方向表 | 新增 local-runtime 行，标依赖"domain-worktree（接收 Runtime Observation）" |
| 5 | 附录 B 模块依赖图 | 补 node + 边 |
| 6 | 附录 C 数据所有权矩阵 | 新增 1 行 |
| 7 | §4.6.1 开篇 | 加段落："本节描述的是服务器侧的 Runtime Registry / Port（`domain-local-runtime` crate，跑在 work-core 进程内），不是 Local Daemon 二进制进程本身（后者是独立 Rust 二进制，运行在 Developer Machine / Self-hosted Runner / Cloud Workspace 上，部署拓扑见 §1.1 LocalRuntime 子图）。" |
| 8 | 接口稳定承诺 #1 | "24 个 Module 划分与依赖方向" → "25 个 Module 划分与依赖方向" |

### A.2 F-01 + F-02 联动：§6 → 25 的推导

**修复动作**（3 处）：

| # | 位置 | 动作 |
|---|---|---|
| 1 | §2.2 脚注 | 完整列出来自 §6 的合并/拆分：<br>- `Collaboration` 拆为 `domain-comment` + `domain-collaboration`（原有）<br>- `Development Context` 合并入 `domain-development`（本次新增）<br>- 新增 `domain-local-runtime`（对应 §23 Local Runtime 的服务器侧管理面） |
| 2 | §2.1 表 `domain-development` 行 | "主要实体"补：`SymbolIndex, RepositoryContext, DevelopmentContext`（对应 §20 实体） |
| 3 | 数字一致性 | §0.2 表格"§6 Domain Boundary"行不动；§2.1 标题改为 25；§2.2 脚注说明 25 的来源 |

### A.3 F-05：WorkItem 默认状态机改回三态

**修复动作**（5 处）：

| # | 位置 | 动作 |
|---|---|---|
| 1 | §4.9.3 状态机（WorkItem 默认） | 改为三态：`TODO → IN_PROGRESS → DONE`（外加 `ARCHIVED` 终态） |
| 2 | §7.2 mermaid 图 | 同步改为三态图（去掉 `IN_REVIEW` / `BLOCKED` / `CANCELLED` 等） |
| 3 | 新增 §4.9.4 | "Project Policy 自定义扩展示例"，列出 `IN_REVIEW` / `BLOCKED` / `CANCELLED` / `IN_TESTING` / `READY_FOR_DEPLOY` 为可选状态 |
| 4 | §7.6 状态机总览表 | WorkItem 状态数改为"3 + 扩展" |
| 5 | 接口稳定承诺 #6 | "WorkItem 状态机（§7.2）：5 状态" → "3 状态（默认）+ 扩展" |

---

## 3. 批次 B：可缓（Minor + 笔误）

### B.1 F-08 AgentSession 状态数 13 → 14

3 处替换：
- §7.4 文本注释
- §7.6 表格
- 接口稳定承诺 #8

### B.2 F-06 tenant_id 计数 12 → 13

- §4.10.4 标题："强制 tenant_id 携带的对象（12 项）" → "（13 项）"
- §6.1 不动（标题已写"13 类对象"）

### B.3 F-04 5 处 §N(>47) 引用

按 feedback L49 列表逐条替换：

| 位置 | 当前 | 改为 |
|---|---|---|
| L15 | §105 | §47（不输出生产代码，引 requirements.md §0） |
| L69 | §63 | §41.2（P0 登记表） |
| L284 | §85 | §44.3（Worktree 解耦原则） |
| L828 | §52 | §24.5（Multi-Agent Control / Agent Handoff） |
| L1982 | §60 | §14.1（Event 列表） |

### B.4 笔误

§2.1 表第 6 行：`REQ-AUT-001` → `VAL-001`。

---

## 4. 执行路径

```
1. cd D:\Star
2. git worktree add -b feature/basic-design-fixes D:\Star-worktrees\basic-design-fixes main
3. cd D:\Star-worktrees\basic-design-fixes
4. 应用批次 A + 批次 B 全部修改（docs/basic-design.md）
5. 自审：
   - grep CREATE TABLE / ALTER TABLE / PRIMARY KEY  →  应为 0
   - mermaid 块数  →  保持 11 或略增
   - §2.1 表格行数 = 25
   - §4.9.3 状态数 = 3 + 终态
   - §7.4 状态数 = 14
   - 9 处修改点逐一回查
6. git add docs/basic-design.md
7. git commit -m "docs: address review feedback (F-01~F-08 + REQ-AUT-001 typo)"
8. cd D:\Star
9. git merge --no-ff feature/basic-design-fixes -m "merge feature/basic-design-fixes: address review feedback"
10. git worktree remove D:\Star-worktrees\basic-design-fixes
11. git branch -d feature/basic-design-fixes
12. 报告
```

---

## 5. 风险与回滚

| 风险 | 缓解 |
|---|---|
| Module 总数 24 → 25 影响"接口稳定承诺" | 已纳入修复；下游需重新核对 |
| 状态机默认值变化影响 Workflow 实现 | REQ-WF-001 强约束要求，回归原口径正确 |
| 改动面广（6+ 章节）易遗漏 | 自审脚本 + 9 处 check list |
| 回滚 | 在 worktree 内 `git reset --hard HEAD~1` + 删 worktree，main 不动 |
| 冲突（如 main 有新 commit） | 当前 main tip 62d2a0c 稳定，无并发风险 |

---

## 6. 预计消耗（token-OLU 框架）

| 环节 | tokens |
|---|---|
| 子代理改文档（8 Finding + 1 笔误） | 60K-120K |
| 自审（grep + 9 项核对） | 5K-15K |
| 合计 | 65K-135K |
| 折合 0.5-1 人·周 | — |

按你 RGS-TS-001 v0.4 §6.2 草案口径：1 人·周 ≈ 1M tokens，本次属小规模设计修订任务。

---

## 7. 你的决策点（待确认）

- **执行范围**：批次 A + B 一起做，还是只做 A？
- **执行方式**：开 worktree（隔离、可回滚），还是直接 amend main 上 52414a9（轻量但改 history）？
- **plan 文档归宿**：保留 `remediation-plan.md` untracked 留档，还是 commit 到 main 作为审计痕迹？
