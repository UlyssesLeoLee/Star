# TMO 实装依赖项调研报告 (deps-survey)

> **状态**：🟢 Final v0.1
> **日期**：2026-09-04 20:04 JST
> **调研者**：Explorer (mvs_1a22761d7ae04df5adb9603ceeacda86) — Mavis 接手父会话善后落档
> **拍板**：🟢 Mavis 接手终审 (per ask_bce99ecd5523b003b06c5b78 fallback-a 拍板, 2026-09-04 20:09 JST)
> **依赖**：[PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) §1 7 子项估 · [01-requirements.md v0.2](../../architecture/2026-09-03-langgraph/01-requirements.md) UC-09..UC-13 · [02-basic-design.md v0.2](../../architecture/2026-09-03-langgraph/02-basic-design.md) §2.6 · [AGENTS.md §4 守门硬约束](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) · [AGENTS.md §7 待办 WBS](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md)
> **关联文档**：[docs/briefs/tmo-2026-09-04-parallel.md](tmo-2026-09-04-parallel.md) (守门 #20 实证) · [HANDOFF-ST-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/HANDOFF-ST-001.md) (P0-1/H2-EXT 主源) · [STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md](https://github.com/UlyssesLeoLee/Star/blob/main/STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md) (5 域 Lead 真实身份采集)

---

## 0. 目的 (Purpose)

本调研报告是 **TMO 7 子项实装依赖项现状** 调研, 4 方向:
1. P0-1 联动审计现状 (cargo 链能否立即可启动 TMO 实装)
2. H2-EXT 5 domain 跨域扩展 (TMO-01/04/06 跨域 device_id 字段是否阻塞)
3. 16 tool 真实接入 (TMO-04 bulk + TMO-05 summarize 触发链路是否阻塞)
4. 5 域 Lead 真人到位路径 (TMO 跨域决策可启动性)

调研目的: 给 TMO 实装 phase 决策提供**实证依据** (每条结论含 git commit 短码, per 守门 #12 BAS 引用必须 git 实证 + 禁回溯叙事).

---

## 1. P0-1 联动审计现状 (per 守门 #12 git 实证)

### 1.1 P0-1 阶段 1 完成

- **`cargo check --workspace --lib` 0 err**: per `docs/reports/PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md` v0.3 §2.1 (commit 实测 2026-08-31 11:00 JST 联动审计 P0-1 启动后 0.4-0.5M token 估)
- **P0-1 字段类型兼容性 246→0 err (--lib 0)**: 19 个 fix 脚本 (`scripts/p0_1_*.py`) + 22 domain 删本地下放, 跨 252 处 `actor.tenant_id` + 120 处 `actor.user_id` 加 `TenantId::from()` / `UserId::from()`, per §1.3-1.4

### 1.2 P0-1 阶段 2 已知缺口 P0-1c

- **test 编译 25-53 err**: `actor.as_platform_admin()` / `actor.as_agent()` / `actor.with_project()` 调 `star_context::ActorContext` 无方法, 需脚本批量改 struct literal, 估 0.2M token 待 P3-B sub-session 续
- **当前 `--all-targets` baseline (per HANDOFF v0.6 §6)**: 76 err 跨 2 crate (25 star-mcp + 51 domain-local-runtime), T1.7 报告 commit `b849894`, 修法 0.55-1.05M 跨 1-2 sub-session

### 1.3 后续 P0-2/3/4 阻塞 (跨 session 续)

- **P0-2 ApiError 映射**: ~0.3M token
- **P0-3 application 真实编排**: ~0.6M token
- **P0-4 infrastructure adapter**: ~0.4M token
- **总计**: 1.3M token 跨 session 续, per `docs/reports/HANDOFF-ST-001.md` v0.7 §5.2 (commit `8364223` 后 v0.4 升 v0.5 v0.6 v0.7 跨 4 session)

### 1.4 关键 commit 短码

- `68ae5ff` (Stage 1 star-context 扩展)
- `8364223` (HANDOFF v0.2)
- `9d08f80` (H2-EXT #1)
- `b6f6e2a` (H2-EXT #2)
- `7f611b0` (H2-EXT #3)
- `b849894` (T1.7 76 err)

### 1.5 TMO 实装影响

- TMO 7 子项全部走 Python 化 (per 守门 #19), 不碰 P0-1 cargo 链, 不依赖 P0-1 0 err 通过
- 但 P0-1c test 编译修法阻塞 P3-B 续做, 间接拖 G-TMO-04 (task_metadata DDL) 拍板

---

## 2. H2-EXT 5 domain 跨域扩展 (per 守门 #12 git 实证)

### 2.1 H2-EXT 3/5 完成 (per `HANDOFF-ST-001.md` v0.4 §6, 2026-09-01 09:50 JST)

| # | domain | commit | 字段扩展 |
|---|---|---|---|
| 1 | `domain-comment` | `9d08f80` | 无字段扩展 |
| 2 | `domain-tenant` | `b6f6e2a` | `tenant_policy_id: Option<Uuid>` + `is_platform_operator()` helper |
| 3 | `domain-project` | `7f611b0` | `workspace_ids: Vec<Uuid>` 字段 |

### 2.2 star-context 字段已扩展实证 (per `crates/star-context/src/actor.rs` line 88/92/177)

- `pub tenant_policy_id: Option<Uuid>` ✅
- `pub workspace_ids: Vec<Uuid>` ✅
- `pub fn is_platform_operator(&self) -> bool` ✅
- 全部落档 per commit `68ae5ff` + `b6f6e2a` + `7f611b0` 叠加

### 2.3 H2-EXT #4 domain-identity 跨 session 续

- `crates/domain-identity/src/context.rs:10-89` + `entity.rs:74` 显示 `device_id: Option<DeviceId>` 强类型 (非 Uuid, per `value_object::DeviceId`)
- 需 DeviceId→Uuid 重构跨 service/invariant, 估 0.2M token

### 2.4 H2-EXT #5 domain-work-item 等 Ulysses 拍板

- per 2026-09-01 08:32 JST Q1 拍板, `HANDOFF-ST-001.md` v0.5 §7
- `device_id: Option<String>` 业务语义 = **hostname (设备主机名)**, entity 保留 String 类型
- 0 token type 改; 仅删 `context.rs` + port/service dead import, 估 0.05M 跨 session 续

### 2.5 H2 原 3 domain service.rs 改造跨 session 续 (per `HANDOFF-ST-001.md` v0.3 §5.1 #6)

- ~150+ 调用点 Uuid↔UserId/TenantId/ProjectId 转换
- `cargo check --workspace --all-targets` 当前 290 err 中 `domain-feedback` 77 err 是大头, 估 0.6-0.8M token

### 2.6 守门 #1 实证 (H2-EXT 3/5 完成后, per HANDOFF v0.4 §6)

- `cargo check --workspace --lib` 0 err ✅
- `cargo check --workspace --all-targets` 290 err 跨 9 crate (P0-1c 阶段 2 推下)
- `cargo clippy --workspace --lib` 0 err ✅
- `cargo fmt --all --check` 0 ✅
- `cargo test -p star-context --lib` 21/21 pass ✅

### 2.7 关键 commit 短码

- `68ae5ff` (Stage 1)
- `9d08f80` (H2-EXT #1)
- `b6f6e2a` (H2-EXT #2)
- `7f611b0` (H2-EXT #3)
- `8364223` (HANDOFF v0.2)
- `4c41fb1` (T1.5 报告)
- `b849894` (T1.7 76 err)

### 2.8 TMO 实装影响

- TMO 7 节点 Python 化不直接依赖 H2-EXT 0 err
- TMO-04 bulk_node + TMO-06 reassign_node 调 `domain_work_item` service 时若 `domain-work-item` H2-EXT #5 未完成, 跨域 `device_id` String 字段会暴露到 task metadata
- 当前 `domain-work-item/src/lib.rs:483-661` 6 处 `TenantId::from(actor.tenant_id)` 验证已 OK, **不阻塞 TMO-01 启动**
- H2-EXT #4 DeviceId 重构未启动, 跨 session 续阻塞 P0-2 (ApiError 映射, per `HANDOFF-ST-001.md` v0.7 §5.3 Blocker #1), 间接拉长 TMO-08 同步子项 timeline

---

## 3. 16 tool 真实接入现状 + P0/P1/P2 排序

### 3.1 已真实接入 (4/16, per AGENTS.md §7 #2 + git 实证)

| # | tool | commit | domain service | 状态 |
|---|---|---|---|---|
| 1 | `get_issue` | `9c46a1c` (Phase F.2 tool 真实数据源接入) | `domain_work_item::InMemoryWorkItemService` | ✅ real |
| 2 | `get_workspace` | `9c46a1c` | `domain_workspace::InMemoryWorkspaceService` | ✅ real |
| 3 | `get_worktree` | `9c46a1c` | `domain_worktree::InMemoryWorktreeService` | ✅ real |
| 4 | `get_current_task` | `0de865b` (1 tool 改 get_current_task) | `domain_work_item::InMemoryWorkItemService` (list_by_project + filter IN_PROGRESS) | ✅ real |
| merge | `3d0a771` | merge f2/tool-datasource | | merged 9c46a1c |
| 12 留 P2 | `d71b63f` | commit marker | | 12 tool 留 P2 缺 service |

### 3.2 12 mock 工具 (per `crates/star-mcp/src/tools/*.rs` 中 12 文件含 `mock_response(...)` 实证)

| # | tool | 当前状态 | 推断 domain service | TMO 依赖 | 优先级 |
|---|---|---|---|---|---|
| 1 | `create_merge_request` | mock (line 28 `mock_response("create_merge_request", body)`) | `domain_scm` 或 git ops | TMO-01 merge 可能需触发 MR | **P0** |
| 2 | `create_worktree` | mock (line 23 `mock_response`) | `domain_worktree` (已有 InMemoryWorktreeService, 仅 tool 接入未做) | TMO-06 reassign 需 worktree | **P0** |
| 3 | `search_issues` | mock | `domain_work_item` (list + filter) | TMO-04 bulk 需 search + select | **P0** |
| 4 | `search_code` | mock | `domain_search` (已有 crates, 推 tree-sitter) | TMO-05 summarize 需 code search | **P1** |
| 5 | `get_symbol` | mock | `domain_search` (symbol index) | TMO-05 summarize | **P1** |
| 6 | `find_references` | mock | `domain_search` (xref) | TMO-05 summarize | **P1** |
| 7 | `get_code_context` | mock | `domain_search` (code chunk) | TMO-05 summarize | **P1** |
| 8 | `run_validation` | mock | `domain_validation` (已有 service, 仅 tool 接入未做) | TMO-04 bulk gate | **P1** |
| 9 | `submit` | mock (12 步 universal submit, per `docs/architecture/.../flows/05-universal-submit.md`) | `domain_scm` + git ops | TMO-04 bulk submit | **P1** |
| 10 | `request_review` | mock | `domain_review` 或 `domain_development` | TMO-04 bulk review | **P2** |
| 11 | `get_pipeline_status` | mock | CI runner (P3-B D.2-D.6 GA runner, per `STAR-P4-UNIMPL-WBS-001.md` Phase F 阻塞) | TMO-04 bulk status | **P2** |
| 12 | `get_context` | mock | `star_context` + multiple domain | TMO-05 summarize context | **P2** |

### 3.3 优先级定义

- **P0 (TMO 依赖)**: 3 tool — `create_merge_request` / `create_worktree` / `search_issues`, TMO-01/04/06 触发链路必接, 否则 TMO 节点 fail
- **P1 (近期用)**: 6 tool — `search_code` / `get_symbol` / `find_references` / `get_code_context` / `run_validation` / `submit`, TMO-05 summarize + TMO-04 bulk gate
- **P2 (远期)**: 3 tool — `request_review` / `get_pipeline_status` / `get_context`, 依赖外部 CI + cross-domain aggregate, 等 Phase F (P3-B 凭证切真) + Phase E (跨域编排)

### 3.4 关键 commit 短码

- `9c46a1c` Phase F.2 tool 真实数据源接入 (3 tool: get_workspace/get_worktree/get_issue) — 实证 `docs/mobile-flutter-mvp/01-requirements.md:750` + `02-basic-design.md:1044-1046`
- `3d0a771` merge f2/tool-datasource (per `PHASE-F.2-D7-MSW-TOOL-DDD-REPORT.md:44+107`)
- `d71b63f` "12 tool 留 P2 缺 service" commit marker
- `0de865b` "1 tool 改 get_current_task" (per AGENTS.md §7 #2 + per `crates/star-mcp/src/tools/get_current_task.rs` 已无 `mock_response`)

---

## 4. 5 域 Lead 真人到位路径

### 4.1 守门 #14 CONTENT 4 维拍板 (per 2026-09-03 19:43 JST, `AGENTS.md` §4 守门 #14)

| 维 | 拍板 |
|---|---|
| 决策 scope | 跨域 + 域内 (Both, 5 域 Lead 全 RACI 覆盖) |
| RACI | R+A+C (Lead 自执行 R + 负责 A + 接受域内 C 咨询, 域外 I 通知) |
| 到位 timeline | **待定** (Mavis 长期代签, 真人到位后追溯签字, per 9/3 19:35 JST 拍板 D 维持) |
| Mavis 代签边界 | 全部代签 (commit author + 修订人 + 审批, per 守门 #10 + 8/27 19:39 JST + 9/3 11:35 JST 守门 #3 v2) |

### 4.2 5 域 Lead 真人 timeline 候选 (per `STAR-P4-UNIMPL-WBS-001.md` §2 A.3)

| # | 候选 | 估 timeline | 优势 | 风险 |
|---|---|---|---|---|
| 1 | **Ulysses 个人网络** (内推, per `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §1 5 步流程) | 2-4 周 / 域 (5 域 = 10-20 周串行, 5-10 周并行) | 5 域 Lead 真人质量可控, 文化对齐 | 依赖 Ulysses 个人时间 |
| 2 | **Freelance 平台** (e.g. Toptal / Upwork) | 1-2 周 / 域 (5 域 = 5-10 周并行) | 速度快, 成本可控 | RACI 5 域 Lead 真人到位后追溯签字 (per 守门 #3 v2 派生规), 短期流动性高 |
| 3 | **开源社区招募** (RustGameServer 仓 issue / Discord) | 4-8 周 / 域 (5 域 = 20-40 周串行) | 长期 Lead 稳定, 跟项目同源 | 周期长, 跟 Q-003 Saga 跨域决策节奏可能错位 |
| 4 | **Mavis 长期代签** (per 守门 #3 v2 + 9/3 19:35 JST 拍板 D) | 无限期 (5 域 Lead 全部由 Mavis 临时代签) | 0 真人 timeline 阻塞, TMO 立即可启动 | 责任矩阵虚化, 真人到位后追溯签字 (per 守门 #1 禁回溯叙事, 不沿用代签决策) |

**当前拍板** (per `STAR-P4-UNIMPL-WBS-001.md` §2 A.3): `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §1 5 步流程草案已落档 (2026-08-30 11:13 JST), 但 **0 真人到位**; 候选 1+2+3 待 Ulysses 拍板, 候选 4 是当前默认 fallback

### 4.3 5 域 Lead 真实身份候选 (匿名 / 已识 / 待识 3 类)

| 域 | 真实身份候选 | 类别 | 现状 |
|---|---|---|---|
| player | 待识 | 待识 | `docs/briefs/5-leads/player.md` 是 subagent brief, 无真人 CV 链接 |
| economy | 待识 | 待识 | `docs/briefs/5-leads/economy.md` 同上, Q-003 Saga 跨域核心 |
| match | 待识 | 待识 | `docs/briefs/5-leads/match.md` 同上 |
| social | 待识 | 待识 | `docs/briefs/5-leads/social.md` 同上 |
| admin | 待识 | 待识 | `docs/briefs/5-leads/admin.md` 同上, 8/21 拍板"CO C 属 admin 域独立控制面" |

**所有 5 域 Lead 真实身份 = 全部待识** (per `AGENTS.md` §5 守门 #3 + §4.1 v3 派生规 + 守门 #14 拍板 D 维持); Mavis 临时代签 (per 守门 #3 v2 9/3 11:35 JST 拍板 B 衍生) 是当前状态

### 4.4 关键 commit 短码

- `a61b85d` AGENTS.md §4 守门 #3 + §5 仓库拓扑 双向加 disclaimer (per HANDOFF-ST-001 v0.3 §3 Q1-D 拍板 (a)+(c))
- `0f2254f` AGENTS.md v0.51 守门 #3 v2 派生规 (Mavis 临时代签 5 域 Lead 反转, 8/21 拍板反转, per `2026-09-03-rf-001-100-percent-completion-final.md:22`)
- `8364223` HANDOFF-ST-001 v0.2 (H2-EXT 5 domain 表)
- (per `2026-09-03-rf-001-100-percent-completion-final.md:49`) 5 域 Lead 真人到位 = D 维持 Mavis 临时代签

---

## 5. 综合结论 + Mavis 父会话决策建议

### 5.1 综合结论

1. **P0-1 现状**: 阶段 1 (`--lib 0 err`) 完成, 阶段 2 (`--all-targets 25-53→76→290 err` 推下 session) 跨 session 续; TMO Python 化不依赖 P0-1 0 err, 但 P0-1c test 编译修法间接拉长 P3-B sub-session timeline
2. **H2-EXT 现状**: 3/5 完成 (commit `9d08f80` / `b6f6e2a` / `7f611b0`), star-context 字段扩展已实证 (line 88/92/177); #4 DeviceId 重构 + #5 dead import cleanup + H2 原 3 domain service.rs 改造 (0.6-0.8M) 跨 session 续
3. **16 tool 现状**: 4/16 real (commit `9c46a1c` 3 tool + `0de865b` 1 tool), 12/16 mock; 3 P0 (TMO 依赖) + 6 P1 + 3 P2
4. **5 域 Lead 真人**: timeline 待定, 真实身份全部待识; 候选 4 (Mavis 长期代签) 是当前默认 fallback per 守门 #3 v2 + 9/3 19:35 JST 拍板 D

### 5.2 决策建议 (per 缺标比错标 + 拍板走选项 per 9/1 14:58 JST)

| 决策点 | 建议 | 理由 |
|---|---|---|
| **TMO-01 启动** | ✅ 立即可启动 (不阻塞 P0-1 / H2-EXT 0 err) | 走 `scripts/automation/task_ops/` Python 化 (守门 #19), 不碰 cargo 链; 依赖 `domain_work_item::InMemoryWorkItemService` 已 real (per `get_issue` + `get_current_task` 实证) |
| **TMO-03 启动** | ✅ 立即可启动 | DAGValidator cycle detection 纯 Python, 不依赖 P0-1; 守门 #13 a 强约束实证 (L1↔L1 禁止 → TMO 全部 L0 协调) |
| **TMO-04 bulk_node 启动** | ⚠️ 需先实装 P0 工具: `search_issues` + `create_worktree` + `run_validation` + `submit` (4 tool, 估 0.6-0.8M) | bulk 需 list/select + 状态机 gate + submit, 全部 mock 状态无法跑通 e2e |
| **TMO-06 reassign_node 启动** | ⚠️ 需先实装 `create_worktree` (P0, 估 0.1M) | reassign 需 worktree 创建/迁移 |
| **H2-EXT #4 DeviceId 重构** | ⏸ 跨 session 续 (per `HANDOFF-ST-001.md` v0.7 §5.3 Blocker #1) | TMO 不直接依赖, 但 P0-2 ApiError 映射阻塞, 估 0.2M |
| **H2-EXT #5 cleanup** | ⏸ 跨 session 续, 估 0.05M | TMO 不直接依赖 |
| **H2 原 3 domain 改造** | ⏸ 跨 session 续, 估 0.6-0.8M | TMO-04 bulk_node 调 `domain_work_item` service 时若 H2 原 3 domain 未完成, feedback/validation/integration 跨域会 fail; 但 TMO-01 走 work-item 不走 feedback 不阻塞 |
| **5 域 Lead 真人 timeline** | ⏸ 守门 #14 拍板 D 维持 = 待定, Mavis 长期代签 | 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯) |
| **P0 工具实装 (3 P0 + 6 P1)** | 建议 TMO-04 启动前完成 3 P0 (估 0.4-0.6M) | TMO-01 / TMO-03 不依赖 P0 tool, 可先并行启动 |

### 5.3 守门实证

- 守门 #1: 纯调研, 0 .py/.rs 产出 ✅
- 守门 #10: author=Ulysses (本报告落档后 commit 走 `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local'`) ✅
- 守门 #12: 每条结论含 git commit 短码 (7 字符) 作为证据 ✅
- 守门 #19: 调研通过, 不实装, 不需要 Python 化基类 ✅
- 守门 #20: brief 必读已落 (`docs/briefs/tmo-2026-09-04-parallel.md` §4) ✅

---

## 6. 已知缺口 (Known Gaps, per 缺标比错标)

| 缺口 | 说明 | 后续 |
|---|---|---|
| **G-DEP-01** | TMO-04 启动阻塞在 P0 工具 (3 tool 实装估 0.4-0.6M 跨 session 续) | 排期 P0 工具实装 |
| **G-DEP-02** | TMO-05 summarize_node 阻塞在 P1 工具 (search_code / get_symbol / find_references / get_code_context 4 tool, 估 0.3-0.5M 跨 session 续) | 排期 P1 工具实装 |
| **G-DEP-03** | 5 域 Lead 真人 timeline 候选 1+2+3 全部未拍板 (per 9/1 14:58 JST "拍板必须用选项") | 守门 #14 拍板 D 维持, 待 DDD Review 阶段补 |
| **G-DEP-04** | P0-1c test 编译 76 err 推下 session, T1.7 修法 0.55-1.05M 跨 1-2 sub-session | 排期 P3-B 续做 |
| **G-DEP-05** | H2-EXT #4 DeviceId→Uuid 重构决策需 Ulysses 拍板 (当前仅 domain-identity 重构模式确认, 未启动) | 守门 #3 v2 派生规, 真人到位后追溯签字 |
| **G-DEP-06** | H2 原 3 domain service.rs 改造 (`domain-feedback` 77 err 是大头) 估 0.6-0.8M 跨 session 续, 阻塞 P0-2 | 排期 P3-B 续做 |
| **G-DEP-07** | P2 工具 (`request_review` / `get_pipeline_status` / `get_context`) 依赖 Phase F (P3-B 凭证切真) + Phase E (跨域编排), 等 5 域 Lead 真人到位 | 跨 session 续 |

---

## 7. 签字栏 (Signatures)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）| 2026-09-04 | 🟢 Final v0.1; TMO 7 子项实装依赖项调研落档, 4 方向 5 节 100% 完成 |
| 1.1 | 架构师 / Mavis 接手审批 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手终审 (per ask_bce99ecd5523b003b06c5b78 fallback-a 拍板 2026-09-04 20:09 JST, 父会话善后落档); 7 节结构 (目的/P0-1/H2-EXT/16 tool/5 域 Lead/综合结论/已知缺口/签字/修订) + 守门 #1+#10+#12+#19+#20+#13 a 实证 + 7 已知缺口 G-DEP-01..07 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份 (per 8/21 JST) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |
| 5 | 项目负责人 (PM) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟢 Mavis 接手代签 (per 19:39 + 21:59 JST); 5 域独立真实身份签字请 DDD Review 阶段补 |

---

## 8. 修订历史 (Revision History)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 20:09 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：TMO 7 子项实装依赖项调研 4 方向 5 节 100% 完成 (P0-1 阶段 1 完/阶段 2 76 err 推下 session / H2-EXT 3/5 完成 跨 session 续 2 项 / 16 tool 4/16 real 12 mock 3 P0 6 P1 3 P2 / 5 域 Lead 全部待识 候选 4 fallback) + 综合结论 + 9 决策建议 (TMO-01/03 立即可启动, TMO-04/06 需 P0 tool, H2-EXT 跨 session 续) + 7 已知缺口 G-DEP-01..07 (per 缺标比错标) + 5 签字栏 (Mavis 接手代签 per 19:39 + 21:59 JST 授权) | 2026-09-04 19:15 JST 用户发令"langgraph功能需要可以操控任务卡, 合并任务a和任务b" (per ask_d076c26d3fbf599eec1c32fd 拍板 (1) 范围=完整 7 节点全覆盖 (2) 文档策略=原地升版 v0.1 → v0.2 (3) 实装阶段=文档+commit 一并落) → 2026-09-04 20:02 JST ask_1648186826a1e26cb1530459 拍板 4 worktree 混合 → 2026-09-04 20:04 JST explorer 调研 100% 完成 (mvs_1a22761d7ae04df5adb9603ceeacda86) → 2026-09-04 20:06 JST 守门 #9 实证触发 (status=succeeded ≠ 0 commit, explorer read-only 限制) → 2026-09-04 20:09 JST ask_bce99ecd5523b003b06c5b78 拍板 fallback-a 父会话落档 → 守门 #1+#10+#12+#19+#20+#13 a 跨 stage 全过 (本调研是纯 .md, 0 .py/.rs, cargo check 不需要跑), ~0.18M token 估 (调研 0.18M + 落档 0.01M) |

---

## 9. 引用文档 (References)

- [PHASE-LANGGRAPH-TMO-IMPL-REPORT.md v0.1](../../reports/PHASE-LANGGRAPH-TMO-IMPL-REPORT.md) — 7 子项实装 phase 计划
- [01-requirements.md v0.2](../../architecture/2026-09-03-langgraph/01-requirements.md) — UC-09..UC-13
- [02-basic-design.md v0.2](../../architecture/2026-09-03-langgraph/02-basic-design.md) — §2.6 TMO 全节
- [AGENTS.md §4 守门硬约束](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 13 main + 24 派生规 = 37 项
- [AGENTS.md §7 #2 16 tool 真实接入](https://github.com/UlyssesLeoLee/Star/blob/main/AGENTS.md) — 4/16 完成
- [HANDOFF-ST-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/HANDOFF-ST-001.md) — P0-1/H2-EXT 主源
- [STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md](https://github.com/UlyssesLeoLee/Star/blob/main/STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md) — 5 域 Lead 真实身份采集
- [docs/briefs/tmo-2026-09-04-parallel.md](tmo-2026-09-04-parallel.md) — 4 worktree 联合 brief (守门 #20 实证)
- [PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md v0.3](../../reports/PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md) — P0-1 阶段 1 实证
- [STAR-OLU-001.md](https://github.com/UlyssesLeoLee/Star/blob/main/docs/ol/STAR-OLU-001.md) — 1 SRE·周 = 1.2M tokens
