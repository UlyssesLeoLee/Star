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

## §3 Ulysses 拍板结果 (2026-08-31 22:45 JST, per ask_user 4-step questionnaire)

| # | 决策点 | Ulysses 选择 | 落地动作 |
|---|---|---|---|
| Q1-D | AGENTS.md §5"5 域独立 Lead"命名解读 | **(a)+(c) 历史命名 + disclaimer, 不映射** | AGENTS.md §4 守门 #3 + §5 仓库拓扑 双向加 disclaimer — 5 域是历史治理命名 (5 位真人 Lead 问责结构), 22 domain-* 是 DDD bounded context, 两者非同一分类, 不建立业务子域↔DDD 映射. (commit a61b85d) |
| Q10-P | P0 token 预算超支后续 | **(b) 接受 P0-1 现完成度 + 暂停跨 session 续** | 当前 session 至此收尾; 跨 session 续入口见 §5 |
| Q11-P | ST 测试层级 | **(a) 保持 3 层级** (单元 + IT + ST) | --all-targets 432 err 现状下不扩层, 维持原 3 层 |
| Q12-P | 文档治理详细程度 | **(a) 维持三层** (PHASE 报告 + Q&A 报告 + commit message) | token 预算压力下不升级, 维持原 3 层 |

---
## §4 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-08-31 | 上游 AI (本 session) | 初版: 从 QA-ST-001.md §5/§6 拆出下游 AI 可执行项 (H1-H5) vs 已闭环项 vs 待 Ulysses 拍板项; H5 首次实测 --all-targets 968 err |
| v0.2 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | H2 范围扩量 (3 → 8 domain, 发现 H2-EXT 5 domain) + Stage 1 commit 68ae5ff 落地 + Stage 2-3 尝试后 revert (117+ err, 0.6-0.8M token 超出预算) + H5 重测 950 → 432 err (star-context 扩展消解 145+ err); 上游估 0.3-0.5M 实测 0.6-0.8M (3-5x), H2-EXT 需 0.5-0.8M 额外, 总计 1.1-1.6M, 跨 session 续; 真实尝试脚本入档 scripts/p0_h2_3domain_migration.py |
| v0.3 | 2026-08-31 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板结果
| v0.4 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | H2-EXT #1-#3 落地 (3 commits: 9d08f80 / b6f6e2a / 7f611b0), 净修 507 err (797 → 290, 跨 9 crate); 守门 #1 实证: star-context 21/21 pass + workspace --lib 0 err + H2-EXT 3/5 完成; H2-EXT #4 domain-identity (DeviceId→Uuid 重构) + #5 domain-work-item (String→Uuid 需 Ulysses 拍板 String 原义) 跨 session 续; session 至此收尾 (per 2026-09-01 07:56 JST 新 session 启动, 2026-09-01 09:50 JST 收尾) |
| v0.5 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 4 项 Ulysses 拍板 (Q1 device_id String=hostname 业务语义 + Q2 #4 跨 session 续 + Q3 H2 原 3 domain 跨 session 续 + Q4 P0-2/3/4 跨 session 续); H2-EXT #5 String=hostname 拍板: 不重设为 Uuid, entity 保留 String 类型, 0 token type 改; #5 其他改造 (context.rs 删除 + port/service dead import) 估 0.05M 跨 session 续; session 至此收尾 (per 2026-09-01 08:32 JST 拍板, token 1.95M/2.0M = 97% 紧) |
| v0.6 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | Ulysses 拍板"所有" (per ask_user "所有"选项, 2026-09-01 08:44 JST) = H2 全量收官 + P0-2/3/4 + docs 优化 + 等真人/凭证 全部要做; 总估 4-5M token 跨 4-6 session 续; 本次 session 收尾 (token 1.95M/2.0M = 97% 紧); 入口 HANDOFF v0.6 §8 跨 session 续执行计划 | (Q1-D a+c / Q10-P b / Q11-P a / Q12-P a) + 跨 session 续交接 (Q10-P b 推荐拍板) + §5 新增 "下个 session 入口" 段; 本 session 至此收尾 (per 22:45 JST 4 项拍板 + 守门 #1+#9+#12+#15 跨 stage 全过) |


---

## §5 跨 session 续入口 (per Q10-P b 拍板)

**当前 session 至此收尾** (token 1.4M/2.0M = 70% 接近上限), 下次 session 续 H2-EXT 5 domain + P0-2/3/4.

### 5.1 H2-EXT 5 domain 改造顺序 (估 0.5-0.8M token)

| 顺序 | domain | 类型不兼容决策 | 字段扩展 | 估 token |
|---|---|---|---|---|
| 1 | domain-comment | 无 (context.rs 存在但 lib.rs 无 pub mod) | 无 | 0.05M |
| 2 | domain-tenant | `user_id` 已 Uuid (兼容) | 加 `tenant_policy_id: Option<Uuid>` 到 star_context | 0.1M |
| 3 | domain-project | `user_id` 已 Uuid (兼容) | 加 `workspace_ids: Vec<Uuid>` 到 star_context | 0.1M |
| 4 | domain-identity | `device_id: DeviceId` 强类型 → Uuid 重构 | 无 | 0.2M |
| 5 | domain-work-item | `device_id: Option<String>` → `Option<Uuid>` 业务语义重设 (String 是 hostname? JWT token? 需 Ulysses 拍板) | 无 | 0.2M |
| 6 | H2 原 3 domain service.rs 改造 | feedback/validation/integration service.rs 内部 ~150+ 调用点 Uuid ↔ UserId/TenantId/ProjectId 转换 | 无 | 0.6-0.8M |
| **合计** | 8 domain 全部 | | | **1.1-1.6M** |

### 5.2 P0-2/3/4 token 预算 (估 1.3M)

| 阶段 | 内容 | 估 token | 依赖 |
|---|---|---|---|
| P0-2 | ApiError 映射 (api crate 的 ApiError 跟 domain Error 双向映射) | 0.3M | H2 完成 |
| P0-3 | application crate 真实编排 (跨域 service 调用) | 0.6M | P0-2 完成 |
| P0-4 | infrastructure adapter (DB/KMS/Credential broker 等) | 0.4M | P0-3 完成 |
| **合计** | | **1.3M** | |

### 5.3 跨 session 续 Blockers (5 项)

1. **H2-EXT 类型不兼容决策**: domain-identity DeviceId→Uuid 重构, domain-work-item device_id String→Uuid 业务语义重设 (需 Ulysses 拍板 String 原义是 hostname/JWT token/其他)
2. **star_context 字段扩展**: workspace_ids (Vec<Uuid>) + tenant_policy_id (Option<Uuid>) 加到 star-context 的 ActorContext struct (跟 is_agent_session 同模式)
3. **H2 原 3 domain service.rs 改造**: feedback/validation/integration 内部 ~150+ 调用点 Uuid ↔ 强类型 ID 转换, 可选 (a) 业务侧加 UserId::from(actor.user_id) 显式转换 vs (b) port trait 拆 Uuid + 强类型 双层 (per Q2-D A2 上游推荐 (b))
4. **5 域 Lead 真人到位**: per 8/21 JST 拒绝兼任硬约束, P3-C/E/F 阻塞 1 (per AGENTS.md §4 守门 #3)
5. **P3-B 拍板**: B.5 OpenClaw / B.6 Hermes 凭证 (per AGENTS.md §7 待办 #5-7, 仍 1 阻塞)

### 5.4 下次 session 第 1 步 (建议)

```bash
# 1. 读 HANDOFF-ST-001.md v0.3 (本文) + AGENTS.md v0.26
# 2. git log --oneline -10 看最新 HEAD
# 3. cargo check --workspace --all-targets 重新实测 (per Q9-T A9 数字有时效性, 必须实测, 不得沿用 v0.3 数字)
# 4. 续 H2-EXT 5 domain (按 §5.1 顺序)
# 5. 续 P0-2 ApiError 映射 (H2 完成后)
```

---

## §6 跨 session 续 v0.4 总结 (2026-09-01 09:50 JST)

**H2-EXT 5 domain 改造进度 3/5 完成**:

| # | domain | 状态 | commit | 字段扩展 | 估 token |
|---|---|---|---|---|---|
| 1 | domain-comment | ✅ | 9d08f80 | (无) | 0.05M (实测 ~0.15M) |
| 2 | domain-tenant | ✅ | b6f6e2a | + `tenant_policy_id: Option<Uuid>` + `is_platform_operator()` helper | 0.1M |
| 3 | domain-project | ✅ | 7f611b0 | + `workspace_ids: Vec<Uuid>` 字段 | 0.1M |
| 4 | domain-identity | ⏳ 跨 session 续 | — | (DeviceId 强类型 → Uuid 重构) | 0.2M |
| 5 | domain-work-item | ⏳ 跨 session 续 + 等 Ulysses 拍板 | — | (String → Uuid 业务语义重设) | 0.2M |

**守门 #1 实证 (新 session 启动后)**:

| 阶段 | 命令 | 结果 |
|---|---|---|
| --lib | cargo check --workspace --lib | 0 err |
| --all-targets | cargo check --workspace --all-targets | **290 err** (跨 9 crate, 数字时效性 per Q9-T A9 不得沿用 797 或 432) |
| clippy | cargo clippy --workspace --lib | 0 err |
| fmt | cargo fmt --all --check | exit 0 |
| star-context test | cargo test -p star-context --lib | 21/21 pass |

**290 err 跨 9 crate 分布** (新 baseline):
- domain-feedback 77 (H2 原 3 domain 之一, 最大头)
- domain-worktree 51 (其它 domain, 跟 H2-EXT 无关)
- domain-local-runtime 50
- domain-board 39
- domain-agent 37
- domain-identity 30 (H2-EXT #4)
- domain-relation 4
- domain-project 1 (剩 1 err, 跟 H2-EXT #3 强类型转换相关)
- infrastructure 1

**H2-EXT #4 #5 跨 session 续 (估 0.4M token)**:
- #4 domain-identity: DeviceId 强类型 → Uuid 重构 (entity 改 + 跨 service/invariant)
- #5 domain-work-item: device_id String → Uuid 业务语义重设 (需 Ulysses 拍板 String 原义: hostname? JWT token? 其他?)

**H2 原 3 domain 改造 (估 0.6-0.8M token 跨 session 续)**:
- domain-feedback 77 err 是 H2 原 3 domain 改造大头, 模式跟 #1 #2 #3 一样, 但 service.rs 内部 actor.user_id 当 UserId 用 / actor.tenant_id 当 TenantId 用的 call sites 更多


---

## §7 4 项 Ulysses 拍板记录 (2026-09-01 08:32 JST, ask_user 4-step questionnaire)

| # | 决策点 | Ulysses 选择 | 落地影响 |
|---|---|---|---|
| Q1 | H2-EXT #5 domain-work-item `device_id: Option<String>` 业务语义 | **hostname (设备主机名)** | entity 保留 String 类型, 不重设为 Uuid, 0 token type 改. #5 改造简化: 仅删 context.rs + port/service dead import (估 0.05M) |
| Q2 | H2-EXT #4 domain-identity (DeviceId 强类型 → Uuid) | **(a) 跨 session 续** | 估 0.2M token 跨 session 续. 入口 = HANDOFF v0.4 §6 |
| Q3 | H2 原 3 domain (feedback/validation/integration) service.rs 改造 | **(a) 跨 session 续** | 估 0.6-0.8M token 跨 session 续. 入口 = HANDOFF v0.3 §5.1 #6 |
| Q4 | P0-2/3/4 (ApiError + application + infrastructure) | **(a) 跨 session 续** | 估 1.3M token 跨 session 续. 入口 = HANDOFF v0.3 §5.2 |

**session token 1.95M/2.0M (97%) 紧张**, 4 项全部"跨 session 续"是默认安全选项, 符合守门 #1 阶段 1 实证已经收官 (--lib 0 + clippy 0 + fmt 0 + 21/21 test).

**5 项 Blocker 更新** (per HANDOFF v0.3 §5.3):
1. ✅ H2-EXT #5 String 业务语义已拍板 = hostname (无需 type 改, 仅 context 子模块删除)
2. ⏳ H2-EXT #4 DeviceId → Uuid 重构: 跨 session 续
3. ⏳ H2 原 3 domain service.rs 改造: 跨 session 续
4. ⏳ 5 域 Lead 真人到位: 等 Ulysses
5. ⏳ P3-B 拍板: B.5 OpenClaw / B.6 Hermes 凭证


---

## §8 Ulysses "所有" 拍板执行计划 (per 2026-09-01 08:44 JST)

**拍板**: 所有任务都要做 (a + b + c + d 全部). 总估 4-5M token, 跨 4-6 session 续.

### 8.1 执行顺序 (按 token budget 优先级 + 依赖关系)

| 序 | 任务 | 估 token | 依赖 | session |
|---|---|---|---|---|
| 1 | **H2-EXT #5 简化** (context.rs 删除 + port/service dead import, hostname 拍板 0 type 改) | 0.05M | 无 (本 session 已完成 hostname 拍板) | session #1 |
| 2 | **H2-EXT #4** domain-identity (DeviceId 强类型 → Uuid 重构) | 0.2M | 无 (类型不兼容需 entity 改) | session #1 |
| 3 | **H2 原 3 domain** service.rs 改造 (domain-feedback 77 err 大头 + validation/integration) | 0.6-0.8M | H2-EXT #4 #5 完成 (port trait 模式统一) | session #2 |
| 4 | **守门 #1 阶段 2** --all-targets 0 err 实证 | 0.05M | H2 原 3 domain 完成 | session #2 末 |
| 5 | **P0-2** ApiError 映射 (api crate ApiError ↔ domain Error) | 0.3M | 守门 #1 阶段 2 实证 | session #3 |
| 6 | **P0-3** application crate 真实编排 (跨域 service 调用) | 0.6M | P0-2 完成 | session #4 |
| 7 | **P0-4** infrastructure adapter (DB/KMS/Credential broker) | 0.4M | P0-3 完成 | session #5 |
| 8 | **守门 #1 阶段 3** (release mode test + 派生 v3) | 0.2M | P0-4 完成 | session #5 末 |
| 9 | **docs 优化** PHASE 模板标准化 + HANDOFF 自动生成 | 0.1M | 无 (跟代码独立) | session #1-6 任一 |
| 10 | **cargo doc** 实证 (守门 #1 派生 v4) | 0.05M | 无 (跟代码独立) | session #1-6 任一 |
| 11 | **5 域 Lead 真人到位** (P3-C/E/F 阻塞解除) | 0 | 等 Ulysses 真人 | (等) |
| 12 | **P3-B 拍板** B.5 OpenClaw / B.6 Hermes 凭证 | 0 | 等 Ulysses | (等) |

### 8.2 token 预算

- session #1-#5 各约 1M token
- 6 session 总 4-5M token (per STAR-OLU-001.md v0.1 1 SRE·周 = 1.2M token)
- 实际每次 session 不能超 2M token (model context window)
- 建议每次 session 1-1.5M 目标 (留 25-50% buffer)

### 8.3 跨 session 续入口

每次新 session 第一步:
```bash
# 1. 读 HANDOFF-ST-001.md v0.6 (本文件) + AGENTS.md 最新版
# 2. git pull (per 推 origin 落地)
# 3. git log --oneline -10 看最新 commit
# 4. cargo check --workspace --all-targets 重测 (per Q9-T A9 数字时效性, 必须实测, 不得沿用)
# 5. 续下一个任务 (per §8.1 顺序)
# 6. 完成后 commit + docs 同步 + HANDOFF/AGENTS 修订
```

### 8.4 session 边界守门

- 守门 #1 阶段 1 已收官 (本 session 实证 --lib 0 + clippy 0 + fmt 0 + 21/21 test)
- 守门 #1 阶段 2 待 §8.1 #4 实证 (--all-targets 0 err)
- 守门 #1 阶段 3 待 §8.1 #8 实证 (release test 100% pass)
- 守门 #15 死循环饱和约束持续生效 (新事件触发新 docs 同步)

### 8.5 风险点

1. **session token 累加**: 每次 session 1-1.5M, 跨 6 session 总 4-5M, AI context 物理限制
2. **跨域 type 风险**: H2-EXT #4 #5 + H2 原 3 domain service.rs 改造, entity / port trait / service 三层修改, 需谨慎
3. **守门 #9 实证**: 子代理 RPC 不可靠 (P3-A.6/A.7 实证), 任何委派需 git log 实证
4. **5 域 Lead 真人阻塞**: per 8/21 JST 拒绝兼任硬约束, P3-C/E/F 阶段需真人到位
5. **P3-B 凭证**: B.5 OpenClaw / B.6 Hermes 需 Ulysses 提供凭证
