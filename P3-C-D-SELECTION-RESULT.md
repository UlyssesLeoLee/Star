# P3-C + P3-D 拍板结果 (per 2026-08-30 07:50 JST 拍板, 2026-08-30 07:50 JST 落档)

> **Status**: 🟢 Approved
> **拍板时间**: 2026-08-30 07:50 JST (per ask_user questionnaire response)
> **承接**: STAR-P3-C-DECISION-PACK.md (4 选项) + STAR-P3-D-DECISION-PACK.md (4 选项)
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008)

---

## §0 拍板结果

**P3-C 9 子项**: 选项 1 推荐 — 9 子项全按推荐草案拍, 7 wt 并行, 40M / 6.7 周. 凭证型子项 (C.7 Postgres 连接串) 走 mock 备选 (per 29692a7 B.5/B.6 模式).

**P3-D 7 vs 12 范围**: 选项 1 推荐 — 7 子项核心 (D.1-D.7, P3-A 已知缺口) 21M / 3.5 周, 余 14M 留给 P3-C 跨阶段 / P3-E 启动 / Buffer.

---

## §1 P3-C 9 子项 (40M / 6.7 周, 7 wt 并行)

| # | 子项 | 软预算 | 依赖 | 状态 | 拍板 |
|---|---|---|---|---|---|
| C.1 | Workspace 域 (per-tenant workspace CRUD) | 4.4M | 无 | 🟡→🟢 | 推荐 |
| C.2 | Project 域 (per-workspace project CRUD + per_project_role RBAC) | 4.4M | C.1 | 🟡→🟢 | 推荐 |
| C.3 | Identity 域 (Identity + Permission + WorkspaceMember 三实体) | 4.4M | C.1 | 🟡→🟢 | 推荐 |
| C.4 | WorkItem 域 (work_item + status 状态机 + per_project 过滤) | 4.4M | C.2 | 🟡→🟢 | 推荐 |
| C.5 | Workflow 域 (workflow + workflow_state + per_project 自动化) | 4.4M | C.4 | 🟡→🟢 | 推荐 |
| C.6 | Saga 跨域编排 (Q-003 / Per-domain saga + 跨域 compensation) | 4.4M | C.1-C.5 | 🟡→🟢 | 推荐 |
| C.7 | Postgres 持久层 (sqlx + per-tenant connection pool + migration) | 4.4M | C.1 | 🟡→🟢 (mock 备选) | 推荐 |
| C.8 | Tenant 边界 (per-tenant row-level security + tenant context 注入) | 4.4M | C.7 | 🟡→🟢 | 推荐 |
| C.9 | 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束, DDD Review 签字) | 4.4M | 无 | 🟡→🟢 (跨 session 续) | 推荐 |
| **小计** | | **40M** | | **9/9 拍板** | |

---

## §2 P3-D 7 子项核心 (21M / 3.5 周, 7 wt 并行)

| # | 子项 | 软预算 | 依赖 | 状态 | 拍板 |
|---|---|---|---|---|---|
| D.1 | w28 切 HubCliRuntime 入口 | 1M | A.4 | 🟡→🟢 | 推荐 |
| D.2 | 跨平台 e2e 矩阵 (windows/macos) | 5M | A.6 | 🟡→🟢 | 推荐 |
| D.3 | frontend e2e (Playwright) | 6M | 无 | 🟡→🟢 | 推荐 |
| D.4 | realFetch error wrapper | 2M | A.7 | 🟡→🟢 | 推荐 |
| D.5 | agents/analytics/inbox 3 handler real-mode | 2M | A.7 | 🟡→🟢 | 推荐 |
| D.6 | markdownlint + cargo doc CI job | 3M | A.6 | 🟡→🟢 | 推荐 |
| D.7 | UserMenu 状态条 (real-mode 提示) | 2M | D.5 | 🟡→🟢 | 推荐 |
| **小计** | | **21M** | | **7/7 拍板** | |

---

## §3 触发行动

1. **开 14 wt 并行** (C.1-C.9 9 wt + D.1-D.7 7 wt, per 10:58 JST 每子项 1 wt 决策)
2. **C.1 Workspace 单 wt 先启**, C.2-C.5 等 C.1 merge 后开 wt, C.6-C.9 等 C.1-C.5 merge 后开 wt
3. **D.* 7 wt 等 P3-A 已知缺口修复后再启** (D.2 跨平台 e2e 需 GitHub Actions runner 配置, D.6 需守门 #6 CI 实装)
4. **守门基线** (per AGENTS §4.1 v1-v14 累积规): cargo check + tsc + cargo test --workspace --release --lib + cargo build --release + doc + bench --no-run
5. **commit author = Ulysses** (Mavis 接手代签 per 8/27 19:39 JST 用户授权)
6. **子代理 brief 写明"无证据叙事 = 禁止"** (per AGENTS §1.2 派生规 4)
7. **守门 #9 git log --follow 实证** worktree commit 在 main 链上, 子代理 status ≠ 成功

---

## §4 关联决策包 (P3-E / P3-F 仍待拍板)

- `STAR-P3-E-DECISION-PACK.md` (commit 170fed5) — P3-E 7 子项拍板包, 4 选项待 Ulysses 拍板
- `STAR-P3-F-DECISION-PACK.md` (commit 408e591) — P3-F 6 子项拍板包, 4 选项待 Ulysses 拍板 (F.6 推 origin 已落地)

---

## §5 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 P3-C 选项 1 + P3-D 选项 1 同时拍板; 9 + 7 = 16 wt 并行; 40M + 21M = 61M tokens; 6.7 + 3.5 = 10.2 周 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: P3-C 选项 1 + P3-D 选项 1 拍板结果, 9+7=16 子项全按推荐, 触发 INC-SESSION-003 + 16 wt 并行 | 2026-08-30 07:50 JST ask_user 拍板 |
