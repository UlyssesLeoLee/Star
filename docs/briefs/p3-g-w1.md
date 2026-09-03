# Brief: p3-g-w1

**Agent**: Mavis (root, 不派子代理 per 守门 #9 #3 实证 5/5 RPC 不可靠 + 守门 #24 v3 调试控制台走 subprocess 替代 RPC)
**Phase**: P3-G-W1 (基础层, Agent Jira 化 5 段 20 子项第一段)
**Created**: 2026-09-03 12:00 JST (init) / 12:05 JST (rename P3-B → P3-G per 命名空间冲突 - 跟现有 P3-B OpenClaw 9 子项共存)
**基点 commit**: `f537aab` (AGENTS.md v0.53)
**拍板触发**: 2026-09-03 11:50 JST Ulysses "agent 也应该像 jira 管理团队成员那样管理, 权限所属团队等都要可以管理" + 3 步 ask_user 拍板 (direction_opt4 + team_dimension_opt4 + subagent_persist_opt4) + 3 步开工拍板 (5 段独立验收 + 1.5M 现在 + Mavis 代签 5 域 Lead)
**WBS 文档**: `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` v0.1

---

W1 5 子项 1.9M token 落档 (per 守门 #4 token-OLU 软预算 1.5M 偏差 +0.4M, 软参考可接受):
- G.1 `permission.user_account` (M 类 SCD-2 + RLS 13 類, 跟 `permission.role` 平行, 不破坏 25 schema 约束 per data-design §0.4) — 0.4M
- G.2 `permission.group` (M) + `permission.group_member` (T) — 0.5M
- G.3 `permission.team` (M 类) — 0.3M
- G.4 `permission.team_member` (T, 1 agent → N team 多重隶属 per team_dimension_opt4 拍板) + `permission.role_per_team` (T, 跨 team 不同 role) — 0.5M
- G.5 `agent.user_account_link` (双层 L1 桥接 `domain-agent` 的 `agent.agent.display_name` 跟 `permission.user_account.display_name`) — 0.2M

W1 守门 0 违反验证 (per 守门 #1 v1-v14 + 守门 #13 DB 三類横展開 + 守门 #21 [P] docs 同步 + 守门 #6 PowerShell only + 守门 #7 0 unsafe):
- `cargo check --workspace --all-targets` 0 err
- `cargo fmt --all` 0 diff
- `cargo clippy --workspace --all-targets -- -D warnings` 0 err
- `cargo test --workspace --release --lib` 100% pass
- 5 新表 100% RLS + FORCE RLS + 13 類 policy
- 5 新表 W/T/M 分类显式列 + 主分类单计 + §已知缺口 显式列 (per 守门 #13 派生规)
- docs 同步 5 表设计 + data-design.md / basic-design.md / domain-permission-spec.md / automation-design.md §P3-G-W1 / scripts/automation/registry.md / AGENTS.md §4.1 派生 v25 全部 git 实证

W1 不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠, Mavis 直接写文件 + 落 commit + 跑守门 #1 全套验证)。W1 预计 10-15 commits, W1 完工后写 `docs/reports/PHASE-P3-G-W1-REPORT.md` 7 段结构。

W2-W5 后续 4.0M (推 origin 后走, 15 子项 G.6-G.20) 跨 session 续。

**P3-G 命名空间备注**: 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per `docs/automation-design.md §4.1`) 命名空间共存, P3-G 用 G.x 连续编号 (G.1-G.20), P3-B 沿用 B.1-B.9。不沿用 P3-B 字头, 避免命名冲突。
