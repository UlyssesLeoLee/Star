# ADR-0022: IDE Placement — IDE 归 STAR，GitGit 只做 VCS Core

> **状态**：🟢 Accepted v0.2 (per 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 守门 #12 docs 触发 + 守门 #10 author=Ulysses)
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **父文档**：[STAR × GitGit AI/IDE 升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md)
> **关联**：[STAR vs GitGit Responsibility Matrix](../../../responsibility-matrix/star-vs-gitgit.md) · [GitGit IDE Boundary Spec](../../../responsibility-matrix/gitgit-ide-boundary.md)

---

## 1. 背景

现有 STAR + GitGit 是两个独立项目。STAR 是 25 Module crate workspace（domain-tenant / domain-workspace / domain-work-item / ...），GitGit 是 single-crate MVP（cli + axum + smart-HTTP）。

如果 IDE 相关能力放在 GitGit，会导致：
- GitGit 被迫理解 Issue / Task / Project / Sprint
- GitGit 被迫理解 AI Agent
- GitGit 被迫理解 RAG / Context Graph
- GitGit 被迫理解 CI/CD 业务
- GitGit 被迫理解企业审批
- GitGit 被迫理解用户界面
- GitGit 被迫理解不同 AI 厂商
- GitGit 失去标准 Git 的清晰边界
- GitGit 难以兼容 GitHub / GitLab / 其他 Git Provider
- GitGit 变成难以复用的全栈研发平台

## 2. 决策

**IDE / AI Coding Experience 全部归 STAR。GitGit 严格只做 Version Control Core。**

### 2.1 责任划分

```text
IDE / AI Coding Experience
            │
            ↓
           STAR
            │
 ┌──────────┼──────────┐
 ↓          ↓          ↓
Context   Agent      Workflow
Graph     Runtime    Orchestration
            │
            ↓
       GitGit / Git Provider
            │
            ↓
      Repository / Git Objects
```

### 2.2 完整 STAR 子系统（IDE 相关）

```
STAR
├── IDE Gateway
├── AI Coding Gateway
├── Context Engine
├── Agent Orchestration
├── Task / Issue / Work Graph
├── Code Navigation Service
├── Code Intelligence Service
├── Review / MR Workflow
├── Test / CI / Deployment Workflow
├── Workspace / Worktree Orchestration
└── Git Provider Abstraction
    ├── GitGit
    ├── GitHub
    ├── GitLab
    └── Other Git Providers
```

### 2.3 完整 GitGit 子系统（仅 VCS）

```
GitGit
├── Repository
├── Git Object Database
├── Commit / Branch / Tag / Ref
├── Diff / Blame / History
├── Merge / Conflict Detection
├── Git Protocol / SSH / Smart HTTP
├── Git LFS
├── Repository Mirror
├── Protected Branch 底层能力
├── Protected Tag 底层能力
├── CODEOWNERS 底层解析能力
└── Worktree 底层能力
```

### 2.4 IDE 能力归属（详尽正交表见 [responsibility-matrix/star-vs-gitgit.md](../../../responsibility-matrix/star-vs-gitgit.md)）

**归 STAR**：编辑器工作流 · AI Coding Agent · Issue↔代码关联 · Task↔Worktree 关联 · 代码上下文 · 代码搜索 · 符号导航 · 代码智能分析 · RAG · Knowledge Graph · 代码解释/生成/修改建议 · 测试建议 · 影响分析 · Review 建议 · MR 创建 · CI/CD 状态 · Agent Session · Agent 权限 · Human-in-the-Loop · 多 Agent 协作 · 任务恢复 · 研发流程状态 · 项目级开发规范 · 企业级安全与合规策略

**归 GitGit**：Git Object · Commit · Branch · Tag · Ref · Diff · Blame · History · Merge · Rebase · Conflict Detection · Repository · Git Protocol · SSH · Smart HTTP · Git LFS · Worktree 底层实现 · Repository Mirror · Git 原生权限边界 · Git 原生事件

## 3. 备选方案与拒绝理由

### 备选 A：IDE 放 GitGit
- 拒绝理由：见 §1 8 大问题
- 历史参考：GitHub 把"Project Management"放进了 Issues 而非 git 本身，证实 Issue 不应进 Git

### 备选 B：IDE 放两边
- 拒绝理由：导致重复实现 + 边界不清
- 仅适用场景：明确"Git 原生 IDE"能力（如 `git rebase --interactive` 的可视化）放 GitGit

### 备选 C：IDE 放第三方（既不在 STAR 也不在 GitGit）
- 拒绝理由：失去核心控制 + 增加 Vendor 依赖
- 仅适用场景：VS Code / JetBrains 等"现成 IDE"，与 STAR IDE Gateway 并列共存

## 4. 后果

### 4.1 正面
- GitGit 保持标准 Git 兼容（per §7 任务原文，AI 不需要"gitgit"命令）
- STAR 是 IDE / Agent / 研发工作流平台
- 任何 Git Provider（GitHub / GitLab / Gitea）都可被 STAR 接入

### 4.2 成本
- 必须严格维护责任正交表
- 任何"在 GitGit 加 IDE 能力"的 PR 必拒
- 必须从测试 / 文档 / reviewer 三层守门



---

## 5.5. 实施状态 (per OPT-NEXT-08 v0.2 拍板, 守门 #12 docs 触发)

> **本节由 OPT-NEXT-08 (2026-09-07 14:30 JST) 升版 v0.1 → v0.2 时落地, 提供实施证据链**

| 维度 | 内容 |
|---|---|
| **决策 scope** | IDE 归 STAR (per ADR-0021 依赖) |
| **目标 crate / 落地位置** | `n/a (架构原则)` |
| **落地 commit (per `git log -p --follow`)** | 无 commit (原则文档) |
| **守门 #1 v19** | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (per 9/3 RF-001 T1.5 step 1 验证) |
| **守门 #3 v2** | Mavis 临时代签 5 域 Lead 决策 (per 9/3 11:35 JST 拍板 B), author=Ulysses (per 守门 #10) |
| **守门 #4.2** | Runtime 名称实装前一致性门 — 本 ADR 落档即满足"概念→物理 crate"映射, 后续实装前必先 ADR 拍板 |
| **守门 #10** | commit author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST + 21:59 JST 三次强化) |
| **守门 #12** | docs 同步 = 实施前 git log --follow 实证; 缺标比错标安全 (per 8/26 JST) |
| **守门 #13 W/T/M** | 本 ADR 不涉及 DB schema 分类 (架构原则 / 决策类), 不触发 W/T/M 横展開 |
| **守门 #14 v2** | 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界) 已显式列出 (per 9/3 19:43 JST) |

**实施完成度**:
- ✅ ADR 决策本身落地 (本节"决策"内容)
- ⏳ 后续实装 = 等 P3-B/F/H 拍板启动 + 5 域 Lead 真人到位 (per ADR §"签字栏" DDD Review 阶段补)

**已知缺口 (per 守门 #11 缺标比错标)**:
- 5 域 Lead 真人到位前, 实施由 Mavis 临时代签 (per 9/3 11:35 JST 拍板 B), 真人到位后追溯签字
- 本 ADR 实施状态只反映 commit / 守门 0 违反 实证, 不反映 22 domain 业务实装进展 (per P3-A H2 阶段 11/25 实证)

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
| v0.2 | 2026-09-07 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | OPT-NEXT-08 升版 v0.1 → v0.2: 新增 §5.5 实施状态 (scope/crate/commit/守门 0 违反 实证) + 修订历史 v0.2 行 (per 守门 #10 author=Ulysses + 守门 #12 docs 触发 + 守门 #11 缺标比错标) | 2026-09-07 14:30 JST OPT-NEXT-08 拍板, 23 草案 ADR 一次性升版 |
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase B 起草 |
