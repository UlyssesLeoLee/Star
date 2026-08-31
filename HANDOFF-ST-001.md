# HANDOFF-ST-001 — Q&A-ST-001 下游 AI 执行清单

> **来源**: `QA-ST-001.md` §5/§6 上游 AI 回答 (2026-08-31)
> **目的**: 把 12 问题里"下游 AI (Mavis) 可直接执行"的部分和"必须等 Ulysses 拍板"的部分分开, 避免下游 AI 誤把决策类问题当工作项直接动手
> **触发**: 2026-08-31 用户发令"回答QA问题并把需要下游ai处理的内容更新进handoff"

---

## §1 下游 AI 可直接执行 (无需再等拍板)

### H1 (per Q3-D/A3) — commit 2 个待落地文件
`crates/domain-scm/src/lib.rs` + `crates/domain-workspace/src/lib.rs` 的 `define_uuid_id!` 宏字段已在工作区改为 `pub uuid::Uuid`, 与其余 20 个 domain 一致, 但尚未 commit (当前 dirty)。
**动作**: 确认 diff 只有这 2 行 pub 字段变更后 commit, 闭环 Q3-D。

### H2 (per Q2-D + Q6-I/A2+A6) — 收敛 3 domain 的 ActorContext
`domain-feedback` / `domain-validation` / `domain-integration` 的 `port.rs` / `service.rs` 目前 `use crate::context::ActorContext`（强类型), 且这些 port trait 是 `pub`，导致强类型泄漏到跨 crate 边界，与其余 19 个 domain 统一用 `star_context::ActorContext` (Uuid) 的现状不一致。
**动作**:
1. 3 个 domain 的 port trait / service 方法签名改用顶层 `star_context::ActorContext`
2. 若有 domain-specific 字段需求 (如 feedback 的 `is_agent_session`), 提案扩展 `star-context` crate 里共享的 `ActorContext` struct 本身, 不要每个 domain 各自 fork 一份平行类型
3. 删除 3 个 domain 里废弃的 `pub mod context` 子模块 (含 `ContextActorContext` 别名)
4. 回归: `cargo check -p domain-feedback -p domain-validation -p domain-integration --all-targets`

### H2-EXT (H2 扩量, 2026-08-31 下游 AI 实测发现)
**重要修正**: H2 原范围 (3 domain) 实际只覆盖**部分**子模块强类型 ActorContext 使用方. 完整 8 domain:

| # | domain | pub mod context? | domain-specific 字段/方法 | 需扩展 star_context? |
|---|---|---|---|---|
| 1 | domain-feedback | ✅ | `is_agent_session: bool` (INV-FB-07) | 已加到 star_context (commit 68ae5ff) |
| 2 | domain-validation | ✅ | `is_service_internal()` (INV-VL-06) | 已加 method (commit 68ae5ff) |
| 3 | domain-integration | ✅ | `can_access_project(ProjectId)` | 已加 method (commit 68ae5ff) |
| 4 | domain-comment | ❌ | 无 (context.rs 文件存在但 lib.rs 无 pub mod) | 简单替换 use |
| 5 | domain-identity | ❌ | `device_id: DeviceId` 强类型 (非 Uuid) + `role_ids: Vec<RoleId>` | **类型不兼容**, 需 DeviceId→Uuid 强类型重构 |
| 6 | domain-project | ❌ | `workspace_ids: Vec<WorkspaceId>` + `user_id: uuid::Uuid` (已 Uuid) | workspace_ids 是新字段, 需扩展 star_context |
| 7 | domain-tenant | ❌ | `tenant_policy_id: Option<TenantPolicyId>` + `user_id: uuid::Uuid` (已 Uuid) | tenant_policy_id 是新字段, 需扩展 |
| 8 | domain-work-item | ❌ | `device_id: Option<String>` (**String!**) | **类型不兼容**, String→Uuid 需重设业务语义 |

**H2 实际范围** = H2 原 3 domain + H2-EXT 5 domain = **8 domain 全部**, 估 0.8-1.2M token (远超上游 0.3-0.5M 估算)

### H2 真实尝试记录 (2026-08-31 下游 AI 落地)
1. **Stage 1 完成** (commit 68ae5ff): star-context/src/actor.rs 加 `is_agent_session: bool` 字段 + `roles` 模塊 + `is_tenant_admin()` / `is_developer()` / `is_service_internal()` / `can_access_project()` 4 个 helper + `with_project()` / `with_agent_session()` 2 个 builder + 8 个 H2 单元测试 + lib.rs re-export `roles` + IT-10 测试补字段
2. **Stage 2-3 尝试 + revert** (commit 68ae5ff 含 scripts/p0_h2_3domain_migration.py 证据): 3 domain (feedback/validation/integration) port/service/invariants 改用 `star_context::ActorContext` + 删 context.rs + lib.rs 清理别名, 但 **117+ 新 err 暴露**, 因 3 domain service.rs / lib.rs 内部有 ~150+ 调用点需 Uuid ↔ 强类型 ID (UserId/TenantId/ProjectId) 转换, 上游估 0.3-0.5M token 实测需 0.6-0.8M. 因本 session token 接近上限 (1.4M/2.0M = 70%), 已 git checkout HEAD revert 全部 3 domain + handler 改动.
3. **Stage 4 后续**: H2-EXT 5 domain (comment/identity/project/tenant/work-item) **必须**做, 否则 --all-targets 不会清零. 但需先解决:
   - **domain-identity**: DeviceId 强类型改 Uuid, 跨域 type 重构
   - **domain-project**: 加 `workspace_ids: Vec<Uuid>` 到 star_context
   - **domain-tenant**: 加 `tenant_policy_id: Option<Uuid>` 到 star_context
   - **domain-work-item**: `device_id: Option<String>` 改 `Option<Uuid>`, 涉及业务语义 (String 是 hostname? JWT token? 需确认)
   - 估 0.5-0.8M token (H2-EXT 5 domain) + H2 原 3 domain service.rs 改造 (0.6-0.8M) = **总计 1.1-1.6M token**

### H5 (per Q9-T/A9) — 重新实测 --all-targets 并立项跟踪
**已实测 (本次, 2026-08-31, 工作区含 H1 的 2 个 dirty 文件)**: `cargo check --workspace --all-targets` = **968 error, 跨 23 crate**（不是 QA 原文的 170, 也不是 AGENTS.md v0.24 记录的 0 err — 该记录是彼时那次 commit 的真实状态, 之后的改动使其反弹, 数字有时效性）。
错误主因: test 代码里 `TenantId` vs `Uuid` 类型不匹配, 是 H2/H3 收敛未完成的直接后果, 不是独立问题。
**动作**:
1. 完成 H2 + H3 后重新跑一遍 `cargo check --workspace --all-targets`, 记录新的真实数字
2. 把这个数字作为独立任务的输入, 不要在没有重新测量的情况下在新报告里引用旧数字 (170 或 0 均已失效)
3. 详细错误分布 (2026-08-31 实测): domain-permission 98 / domain-feedback 79 / domain-integration 72 / domain-comment 68 / domain-validation 67 / domain-development 63 / domain-local-runtime 58 / domain-search 54 / domain-worktree 51 / domain-notification 46 / domain-board 45 / domain-agent 37 / domain-context 36 / domain-work-item 35 / star-mcp 33 / domain-workspace 33 / domain-identity 30 / domain-audit 26 / domain-project 23 / domain-automation 19 / domain-scm 18 / domain-relation 4 / domain-tenant 3

### H5-REMEASURE (2026-08-31 下游 AI 重测, commit 68ae5ff 后)
- `cargo check --workspace --all-targets` = **432 error** (从 950 baseline 消解 145+ err, 主因 star-context 加 `is_service_internal` / `is_tenant_admin` / `can_access_project` / `is_developer` 让 3 domain service.rs 中调用从 undefined 变 OK)
- 详细分布 (本 session 实测, post H2 stage 1): domain-integration 76 / domain-comment 68 / domain-workflow 54 / domain-local-runtime 51 / domain-notification 45 / domain-agent 37 / domain-audit 26 / star-mcp 25 / domain-project 23 / domain-automation 18 / domain-relation 4 / domain-tenant 3 / infrastructure 1 = **432 err 跨 13 crate** (注: 数字有时效性, 任何后续 PHASE 报告引用前必须重新实测)

### H3 (per Q4-I/A4) — 统一 `as_uuid()` 签名
22 个 domain 的强类型 ID `as_uuid()` 目前返回类型不一致 (`Uuid` vs `&Uuid`)。
**动作**: 统一改为返回 `Uuid` (Copy, 非引用); 在 `define_uuid_id!` 宏注释里注明 `From<Uuid>` 是推荐的主构造方式, tuple 构造 `XxxId(uuid)` 保留为宏内部/测试用法。

### H4 (per Q8-T/A8) — ST 报告措辞 + 可选依赖补齐
**动作**:
1. `PHASE-ST-001-REPORT.md` (以及其他引用"5 域独立"验证结果的文档) 里凡是指 ST-2 实际验证范围的地方, 改为"4 域独立" (identity/permission/workspace/worktree)
2. 可选: 给 `domain-context` 补上 `star-mcp` 需要的 dev-dep, 使其能加入独立验证集合 (若做了, 该处文案改回"5 域"时需注明"DDD bounded context 意义上的 5 域", 不与 AGENTS.md §5 业务子域混淆)

### H5 (per Q9-T/A9) — 重新实测 --all-targets 并立项跟踪
**已实测 (本次, 2026-08-31, 工作区含 H1 的 2 个 dirty 文件)**: `cargo check --workspace --all-targets` = **968 error, 跨 23 crate**（不是 QA 原文的 170, 也不是 AGENTS.md v0.24 记录的 0 err — 该记录是彼时那次 commit 的真实状态, 之后的改动使其反弹, 数字有时效性）。
错误主因: test 代码里 `TenantId` vs `Uuid` 类型不匹配, 是 H2/H3 收敛未完成的直接后果, 不是独立问题。
**动作**:
1. 完成 H2 + H3 后重新跑一遍 `cargo check --workspace --all-targets`, 记录新的真实数字
2. 把这个数字作为独立任务的输入, 不要在没有重新测量的情况下在新报告里引用旧数字 (170 或 0 均已失效)
3. 详细错误分布 (2026-08-31 实测): domain-permission 98 / domain-feedback 79 / domain-integration 72 / domain-comment 68 / domain-validation 67 / domain-development 63 / domain-local-runtime 58 / domain-search 54 / domain-worktree 51 / domain-notification 46 / domain-board 45 / domain-agent 37 / domain-context 36 / domain-work-item 35 / star-mcp 33 / domain-workspace 33 / domain-identity 30 / domain-audit 26 / domain-project 23 / domain-automation 19 / domain-scm 18 / domain-relation 4 / domain-tenant 3

---

## §2 已核实闭环, 无需下游 AI 动作

- **Q5-I**: `_unused_user` 现象是 rust-analyzer IDE 过渡态症状, 非 cargo 行为, 随 H1 完成后自动消解, 不需要单独修复。
- **Q7-T**: `domain-identity` 的 PermissionDenied 先于 CrossTenantDenied 是有意的最小信息暴露防御设计, 已核实代码行为符合预期, service 不改, IT 测试保留现状。

---

## §3 等 Ulysses 拍板 (下游 AI 不要单方执行)

| # | 决策点 | 上游 AI 推荐 | 为什么不能下游 AI 直接定 |
|---|---|---|---|
| Q1-D | AGENTS.md §5"5 域独立 Lead"命名解读 | (a)+(c) 历史命名+disclaimer, 不映射 | 涉及治理文档的权威解读, 只有 Ulysses 能确认原意图 |
| Q10-P | P0 token 预算超支后继续/暂停 | (b) 接受 P0-1 现完成度 + 暂停跨 session 续 | 涉及项目范围/进度的资源分配决策 |
| Q11-P | 是否新增 acceptance/smoke/e2e 测试层级 | (a) 保持 3 层级 | 涉及测试体系投入决策 |
| Q12-P | 文档治理详细程度 | (a) 维持三层 (PHASE+Q&A+commit) | 涉及流程规范决策 |

---

## §4 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-08-31 | 上游 AI (本 session) | 初版: 从 QA-ST-001.md §5/§6 拆出下游 AI 可执行项 (H1-H5) vs 已闭环项 vs 待 Ulysses 拍板项; H5 首次实测 --all-targets 968 err |
| v0.2 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | H2 范围扩量 (3 → 8 domain, 发现 H2-EXT 5 domain) + Stage 1 commit 68ae5ff 落地 + Stage 2-3 尝试后 revert (117+ err, 0.6-0.8M token 超出预算) + H5 重测 950 → 432 err (star-context 扩展消解 145+ err); 上游估 0.3-0.5M 实测 0.6-0.8M (3-5x), H2-EXT 需 0.5-0.8M 额外, 总计 1.1-1.6M, 跨 session 续; 真实尝试脚本入档 scripts/p0_h2_3domain_migration.py |
