# Brief: OPT-NEXT-08 — 23 草案 ADR 升 v0.2 + 4 crate 命名 + 4 混合表 (per OPT-ADR-01..31, 推下 session)

**Agent**: worker
**Phase**: OPT-P4-NEXT-SESSION
**Created**: 2026-09-07 12:30 JST
**Status**: 🟢 3/3 sub-task done (23 ADR v0.2 + 4 crate 命名 + 4 混合表消解)
**Last updated**: 2026-09-07 18:43 JST (Mavis 接手代签, per守门 #10 + 8/27 19:39 JST 授权)
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

完成 ADR 升版 + 命名拍板 + 混合表分类 (per OPT-A3 §1-§6):

- **OPT-ADR-01..23**: 23 草案 ADR 升 v0.2 (0021-0032 早期 + 0037-0040 + 0042 + 0047 等)
- **OPT-ADR-24..27**: 4 crate 命名拍板 (DDD Review):
  - `domain-task` (L0 TaskQueue)
  - `domain-llm` (L2 LLM Pool)
  - `domain-mcp` (L2 MCP Pool)
  - `domain-tool` (L2 Tool Registry)
- **OPT-ADR-28..31**: 4 混合表分类拍板 (DDD Review):
  - `workspace.workspace` (M/T)
  - `planning.roadmap` (M/T)
  - `audit.audit_event_outbox` (T/W)
  - `local_runtime.runtime_observation` (T/W)

**总估 token**: 0.5-0.9M

## 2. 依赖 (per OPT-A3)

| # | 依赖 | 状态 | 实证 commit (per守门 #12) |
|---|---|---|---|
| 1 | 22 草案 ADR 升 v0.2 启动拍板 (brief 写 23 略差, 实际 22 份) | 🟢 **done** | `86ed286` (OPT-ADR-01..22 草案 ADR 升 v0.2) |
| 2 | 4 crate 命名 ADR 拍板 (DDD Review per AGENTS §4.2) | 🟢 **done** | `08ea0e0` (新增 4 crate domain-task/llm/mcp/tool + ADR-0048 拍板) |
| 3 | 4 混合表分类 DDD Review 拍板 (per守门 #13 派生规) | 🟢 **done** | `f13f325` (4 混合表消解 W/T/M v0.2 → v0.3) |
| 4 | 守门 #13 派生 4/10 (CW-04/06/07~10) SRE Lead 拍板 | 🔴 阻塞 (等 SRE Lead 真人到位) | — |

**完成 commit 总数**: 3 commit (per守门 #20 1 per file group) + 1 merge `ac8695f`
**ADR-0048 拍板内容**: 4 crate 命名 (domain-task/llm/mcp/tool) per守门 #4.2 Runtime 名称映射
**W/T/M v0.3 状态**: 6 混合 → 0 已知 混合 (100 表 100% 业务分类, 2 推测 混合推 P3-B SRE Lead 拍板)

**OPT-ADR-01..22 v0.2 升版范围** (per commit message `86ed286`):
- 0021-0028 (8 份): Zero Vendor Cooperation / IDE Placement / VCS Provider / IDE session / 厂商适配反污染 / STAR AI Compat / STAR IDE Gateway / GitGit Compat
- 0029-0032 (4 份): Universal Submit / Agent Lease+Heartbeat+Resume / Context Graph / MCP Transport stdio
- 0033-0040 + 0042 + 0047 = 10 份: Agent Co-Signing Policy / Jira-ification / Phase F-I Architecture / Audit Onboarding Failed / STAR Agent Runtime SRS / STAR Agent Runtime Design / LangGraph TMO / PostgreSQL Checkpointer Tier 3

## 3. 实施路径

### OPT-ADR-01..23 — 23 草案 ADR 升 v0.2

- 23 份 ADR (per OPT-A3 §1.2): 0021-0028 (8) + 0029-0032 (4) + 0033 + 0034-phase-e + 0036 + 0037 + 0038 + 0039 + 0040 + 0047 = 22 份 (Brief 写 23 略差, 实际 22 份 v0.1)
- 每份加 §"实施状态" + 落地 commit 引用 + 守门 0 违反 实证
- 升 v0.1 → v0.2 拍板: 架构师 (Mavis 接手) + SRE Lead 真人到位 (per E.3 触发)

### OPT-ADR-24..27 — 4 crate 命名拍板

- `domain-task`: L0 TaskQueue 物理 crate (per AGENTS §4.2 + SRS-001)
- `domain-llm`: L2 LLM Pool (跟 G.4 联动)
- `domain-mcp`: L2 MCP Pool (跟 16 tool 联动)
- `domain-tool`: L2 Tool Registry (跟 H.3 联动)
- 拍板形式: ADR-0048 新建 (per OPT-A3 §4 建议)
- 实证: workspace 增 member + 各 1 unit test + 各 1 IT test

### OPT-ADR-28..31 — 4 混合表分类拍板

- `workspace.workspace`: 主分类 M (Master) / 次 T (Tenant) — 拍板
- `planning.roadmap`: 主分类 M (Master) / 次 T (Tenant) — 拍板
- `audit.audit_event_outbox`: 主分类 T (Transaction) / 次 W (Work) — 拍板
- `local_runtime.runtime_observation`: 主分类 T (Transaction) / 次 W (Work) — 拍板
- 实证: `00-CLASSIFICATION-W-T-M.md` v0.1 → v0.2 (混合表消解)

## 4. Worktree

```bash
git worktree add -b feat/opt-adr-promote D:/Star/.worktrees/wt-opt-adr-promote main
cd D:/Star/.worktrees/wt-opt-adr-promote
```

## 5. 守门硬约束

- 守门 #1 v19: cargo check 0 err (4 new crate 落档)
- 守门 #10: author=Ulysses
- 守门 #12: 禁回溯叙事
- 守门 #13: W/T/M 派生 (混合表消解)
- 守门 #4.2: Runtime 名称必须先映射 (per AGENTS §4.2 实装前一致性门)

## 6. 提交

3 commit (per 3 sub-task) 或 1 合并

## 7. 失败处理

- 拍板阻塞 → 报告 + 5 域 Lead / DDD Review 拍板
- 仍失败: 报告具体错误, 不 commit

## 8. 状态 (更新于 2026-09-07 18:43 JST)

🟢 **全部完成** (3/3 sub-task done per commit `ac8695f` merge):
- OPT-ADR-01..22: `86ed286` (22 草案 ADR 升 v0.2, brief 写 23 略差 1 份)
- OPT-ADR-24..27: `08ea0e0` (4 crate 命名 + ADR-0048 拍板)
- OPT-ADR-28..31: `f13f325` (4 混合表消解, W/T/M v0.2 → v0.3, 6 混合 → 0 已知 混合)

**brief 关闭条件**: 全部 3 sub-task done, 关闭。

**已知缺口** (per守门 #11 缺标比错标):
- 守门 #13 派生 4/10 (CW-04/06/07~10) 仍等 SRE Lead 真人到位拍板 (本 brief 仅完成 CW-01~03 + 推 P3-B)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §3.2 #7-#9
- 基线: `docs/briefs/OPT-A3-adr-pending-scan.output.md` §1-§6
- ADR 索引: `AGENTS.md` §6 (0021-0047)
- W/T/M: `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1
- Runtime 映射: `AGENTS.md` §4.2
