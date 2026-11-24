# DD-DEFECT-INTELLIGENCE-001

> **2026-11-24 对账说明**：现行创建、部分失败、独立验证与 WorkItem/defect owner 归属以 [基本设计 §3.4](../basic-design/BD-DEFECT-INTELLIGENCE-001.md#34-缺陷创建查询与验证的统一契约) 为准；**物理接线判定规则**见 [基本设计 §3.5](../basic-design/BD-DEFECT-INTELLIGENCE-001.md#35-单一写入路径与物理接线判定per-2026-11-24-物理事实核对)。以下 PG 适配器流程记录候选实现；以报告者代替实际修复者校验验证独立性不能满足 FR-DI-018，需修复并验收。候选表和路由是否属于同一权威 owner 尚待核实，不证明生产接入。

> 智能缺陷管理 — 详细设计 v1.3 (Draft；实现部分完成，验收未通过)
> 上游：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` v1.1（2026-11-24 现行适用说明）、`docs/basic-design/BD-DEFECT-INTELLIGENCE-001.md` v1.4
> 部署边界：Vercel 仅承载 Web 呈现与同源 BFF（`docs/adr/vercel/ADR-STAR-VERCEL-DEPLOY-001.md` v0.2 草案，未批准）；业务 Owner、持久化与执行权威仍在 Star 侧
> 本设计不启动生产部署，实体与版本、API 契约、Project owner 事务、前端模块映射、测试钩子、已知缺口均已显式列出。

---

## §0 实体与版本

| Logical aggregate | Owner | Identity / version | State |
|-------------------|-------|--------------------|-------|
| `defect_work_item` | WorkItem owner (`domain-work-item` boundary) | `defect_id, revision` | triage / waiting_info / confirmed / in_progress / awaiting_verification / closed / reopened / duplicate / not_processed; 事务事实与事件历史（T） |
| `defect_field_value` | WorkItem owner | `field_id, defect_id, value_revision` | active / superseded |
| `defect_relation` | WorkItem owner | `relation_id, source_defect_id, target_defect_id, relation_type, relation_revision` | active / revoked |
| `defect_status_event` | WorkItem owner | `event_id, event_revision` | Append-only; immutable after commit |
| `defect_verification_record` | WorkItem owner | `verification_id, verification_revision` | pending / passed / failed |
| `defect_evidence_attachment` | Attachment owner | `evidence_id, evidence_revision` | uploaded / archived / revoked |
| `defect_comment` | WorkItem owner | `comment_id, comment_revision` | active / deleted (logical) |
| `defect_ai_suggestion` | WorkItem owner | `suggestion_id, suggestion_revision` | pending / accepted / modified / rejected / expired |
| `defect_workflow_strategy` | Strategy owner | `strategy_id, strategy_version` | active / retired |
| `defect_notification_subscription` | Notification owner | `subscription_id, subscription_revision` | active / unsubscribed |
| `defect_draft` | Session owner | `draft_id, draft_revision` | open / submitted / expired |
| `defect_quality_projection` | Analysis owner | `projection_id, generation` | fresh / stale |
| `defect_sla_policy` | Strategy owner | `sla_id, sla_version` | active / expired |
| `defect_command_idempotency` | Defect owner | `tenant_id, actor_id, idempotency_key` | 24h TTL；命中返回缓存回执（T） |
| `defect_audit_event` | Defect owner | `event_id` | Append-only；与事实同事务提交（T） |
| `defect_outbox` | Defect owner | `event_id` | pending / sent / failed；投递端未实现（T） |

> **2026-11-24 实施对齐**：最后三行为 `crates/domain-defect/src/pg_schema.rs` 中实际建表的
> `defect.command_idempotency` / `defect.audit_events` / `defect.outbox`，
> 用于承载 SRS FR-DI-013（审计）、FR-DI-020（幂等回执）与 §5.2（owner 事务 Outbox）。
>
> **2026-11-24 物理事实核对**：上表三行的**目标**身份与物理实现存在缺口，登记为缺口 G-11/G-12：
> - `defect.command_idempotency` 物理列齐备，但 PG 幂等命中时 `pg_service.rs:687` 返回
>   `build_defect_stub`（空标题/空现象/nil reporter/固定 `lifecycle_version=1`），
>   而非缓存的原始响应体 `response_body` → 去重成立但**回执内容不保真**（FR-DI-020）。
> - `defect.audit_events` 物理列齐备，但缺 `reject_*_mutation` 不可变触发器（对比
>   `multica.task_lifecycle_audit`，见 `db/migrations/2026-09-29-worktree-group-phase-2c.sql:137`）。
> - `defect.outbox` 物理列**缺** envelope 必需字段：`event_id`/`stream_id`/`source_sequence`/
>   `correlation_id`/`event_digest`/`schema_version`/`occurred_at`；现有列仅
>   `aggregate_type`/`aggregate_id`/`event_type`/`payload`/`status`/`created_at`/`sent_at`。
>   `defect_outbox::contract::Envelope`（`crates/star-api-rest/src/defect_outbox/contract.rs:67`）
>   与 `validate_envelope`（同文件 :168）要求上述字段全部非空且 digest 与 payload 匹配，
>   故**当前物理表无法承载 envelope**，消费者即便存在也无法消费生产事件。
>
> 补齐后的目标身份（per §4.2）：

| 表 | 目标主键 | 目标关键列 |
|---|---|---|
| `defect.command_idempotency` | `(tenant_id, actor_id, idempotency_key)` | `request_hash BYTEA`（请求摘要，用于同键不同请求判定）、`response_body JSONB`（**原样回放**，不得重建 stub） |
| `defect.audit_events` | `id UUID` | + `reject_defect_audit_mutation()` 触发器，禁 UPDATE/DELETE |
| `defect.outbox` | `(stream_id, source_sequence)` + `event_id UNIQUE` | `schema_version`、`event_id`、`correlation_id`、`event_digest BYTEA(32)`、`occurred_at`、`tenant_id`、`actor_id`、`payload JSONB` |

> 投递语义为 **at-least-once**，不宣称 exactly-once（per `device_outbox` 既有合约）。

Association/binding revisions are monotonically increasing owner versions. An API client may submit expected version and idempotency key, but may not select actor/tenant identity, modify owner version, or assert current authorization.

### 0.1 部署边界对缺陷域的约束

- 缺陷事实、验证记录、审计与 Outbox 由 Star 侧 Owner 持久化；Vercel 上的 Web/BFF 只做呈现与同源转发，
  不形成第二套业务 Owner、数据库权威或迁移权威（per ADR-STAR-VERCEL-DEPLOY-001 §2、§4）。
- Neon PostgreSQL 是缺陷 PG 适配器的目标数据库之一（用户已绑定），但"测试环境或一次性数据库通过"
  不代表生产部署已验收（per SRS NFR-DI-09）。
- `defects.rs` 路由部署在 Star 的 `star-api-rest`，不部署为 Vercel Function（Rust HTTP Functions
  不在当前 ADR 批准的范围内，per ADR §2）。

---

## §1 API contracts

Logical routes are mounted under existing Project/Lane/Worktree API groups. Exact route names must be reconciled with current route conventions during implementation; they must preserve the command semantics below.

### 1.0 已接线的 REST 契约（物理现状，2026-11-24 `dev@b1db1c99`）

以下 6 条端点已注册于 `build_legacy_mirror_routes`（`crates/star-api-rest/src/lib.rs:215-220`），
实现位于 `crates/star-api-rest/src/routes/defects.rs`，路径前缀 `/api/v1`（per `src/lib.rs:91`）。

| Method | Path | Handler | 请求体 / 查询 | 响应 | 物理状态 |
|---|---|---|---|---|---|
| POST | `/defects` | `create_defect` | `CreateDefectCommand`（含 `idempotency_key`/`tenant_id`/`project_id`/`title`/`phenomenon`/…） | `RestResponse<Defect>` | 恒走 InMemory（G-16） |
| GET | `/defects` | `list_defects` | query `project_id`（必填）、`lane_id`、`page`、`page_size` | `RestResponse<Vec<DefectListItem>>` | 恒走 InMemory；`page`/`page_size` **当前未被使用**（无分页实现） |
| GET | `/defects/{id}` | `get_defect` | path UUID | `RestResponse<Defect>` | 恒走 InMemory |
| PATCH | `/defects/{id}/transition` | `transition_defect` | `TransitionDefectCommand`（`target_status`/`expected_version`/`fix_version`/`idempotency_key`） | `RestResponse<Defect>` | 恒走 InMemory |
| POST | `/defects/{id}/verify` | `submit_verification` | `SubmitVerificationCommand`（`fix_version`/`result`/`verification_method`/`evidence_id`） | `RestResponse<(Defect, DefectVerification)>` | 恒走 InMemory |
| GET | `/defects/{id}/verifications` | `list_verifications` | path UUID | `RestResponse<Vec<DefectVerification>>` | 恒走 InMemory |

**当前端点的诚实边界**（per 基本设计 §3.5.2 判定）：

- `actor_from_headers`（`routes/defects.rs:153`）为桩实现，每次请求随机生成 `user_id`/`tenant_id`；
  客户端**无法**选择 actor/tenant 身份，但服务端也**无法**从会话取得真实身份。
  因此所有跨租户/跨 Project 拒绝路径在真实会话下不可达（G-15）。
- `DefectService::new(None)`（`routes/defects.rs:149`）→ `pg` 恒 `None`，六条端点全部走
  `InMemoryDefectService`，**没有任何生产流量经过 PostgreSQL**（G-16）。
- `routes/defects.rs:251` 挂载 `auth_layer_stub`（`src/middleware/auth.rs:23`），非真实鉴权。
- 错误映射：`From<domain_defect::DefectError> for RestError`（`src/error.rs:863-892`）
  已覆盖 9 个变体，但 `IntoResponse` 的 code→HTTP status 表（`src/error.rs:866+`）**尚未登记**
  `CROSS_TENANT_DENIED` / `CROSS_PROJECT_DENIED` / `INVALID_STATE_TRANSITION` /
  `IDEMPOTENCY_KEY_USED` / `VERIFICATION_MISMATCH` → 这 5 类当前落到默认分支，
  须补齐映射后才能区分 401/403/409/422（G-18）。

### 1.0.1 目标 API 契约（尚未接线）

| Operation | Input | Owner transaction / read | Output |
|-----------|-------|-------------------------|--------|
| List defects | Project/Lane/Worktree selection, filter conditions, page cursor, page size | Current authorization; server-side pagination, permission filtering | Paginated defect list + total count + data source identifier |
| Get defect detail | Defect ID | Fresh authorization; read defect + events + relations + evidence + comments | Defect detail + history + linked items |
| Create defect | Title, phenomenon, expected/actual, reproduction, impact, stage, severity, priority, environment, attachments | Validate required fields; Project/Lane/reporter/authorization context; optional Worktree binding | Stable defect ID + revision + idempotency receipt |
| Update draft | Draft ID, field patch, expected draft revision | Session owner CAS + audit | Draft revision |
| State transition | Defect ID, target state, expected_version, idempotency key, reason | Workflow version + role + field requiredness check; CAS | New state version + audit event |
| Link fix task | Defect ID, task ID or new fix task, expected defect revision | Validate same Project; fix task and defect are different facts | Fix task ID + defect revision |
| Submit verification | Defect ID, fix version, verification method, result, evidence ID | Verifier identity check; fixed defect version pinned | Verification record + state change or block reason |
| Verify result (pass/fail) | Verification ID, result, evidence | Verifier authorization; record through/failed | Verification record revision |
| Reopen defect | Defect ID, failure/reoccurrence evidence, expected revision | Verify authorization + current state allows reopen | Reopened defect + history preserved |
| Mark duplicate/not_processed | Defect ID, target defect ID (for duplicate), reason, expected revision | Authorization; duplicate target must be readable; preserve original report | Terminal result + link |
| Get AI suggestion | Defect ID, input revision | Permission check; minimal necessary input scope | Suggestion content + source + revision + uncertainty flag |
| Accept/modify/reject suggestion | Suggestion ID, decision, modified content, expected suggestion revision | User authorization; record decision | Suggestion decision record |
| List quality projection | Filter conditions, time window, statistics scope | Authorization scope carried through detail drill-down | Statistics chart + detail drill-down entry + update time |
| List notifications | Subscription, since cursor | Authorization recheck on read | Notification events + cursor |

> §1.0 与 §1.0.1 的差别是**接线状态**，不是契约冲突：§1.0.1 是目标能力，§1.0 是当前可达且
> 尚未满足目标的部分实现。两者都不得被当作“已完成”的证据。

### 1.1 Idempotency and version (consistent with SRS FR-DI-020)

- Modification commands carry `expected_version` (task_lifecycle current version) and `idempotency_key`.
- Conflict returns latest version and recoverable draft; unknown result queried by same operation identity receipt, or retried after user confirmation.
- Authorization failure does not auto-replay writes.

---

## §2 Project owner transaction (for workflow/strategy)

```text
BEGIN (Strategy owner)
  resolve ActorContext from authenticated middleware
  lock Project/Lane scope + expected strategy revision
  authorize strategy manager capability
  compute request digest; reserve exact idempotency identity
  CAS strategy current projection
  append Audit fact + Strategy Outbox event
  persist idempotent result receipt
COMMIT
```

- Scope key: `(tenant_id, project_id, lane_id?)`; history has unique revision per strategy.
- Active uniqueness prevents duplicate active strategy per Project/Lane. Retiring is an admission fence: new operations referencing it are denied; existing operations remain resolvable until migration/retirement verified.
- Strategy owner never writes `scm.repository_master` or WorkItem core; it references stable IDs and current master revision for validation.
- The actor cannot write/read relations by tenant membership alone. Events include identity/revision/status/correlation only, no path or content.

### 2.1 缺陷 Owner 事务（PG 适配器，per FR-DI-013/019/020、NFR-DI-05）

实现：`crates/domain-defect/src/pg_service.rs`（`PgDefectService`）。
参考实现：`crates/infrastructure/src/run_task_card_pg.rs`（同一 owner 事务模式）。

```text
BEGIN
  set_config('app.tenant_id', actor.tenant_id, true)
  set_config('app.actor_id',  actor.user_id,  true)
  authorize:
    - cmd.tenant_id 必须 == actor.tenant_id        否则 CrossTenantDenied
    - actor.can_access_project(project_id)        否则 CrossProjectDenied
    - 写能力: platform_admin / tenant_admin / project_admin / developer
  lock_idempotency_key:
    pg_advisory_xact_lock(hashtextextended(tenant:actor:key))
  lookup_idempotency:
    命中且未过期 → 返回缓存回执（replayed），不产生重复事实
  [create]  INSERT defect.work_items (status='triage', lifecycle_version=1)
  [transition]
    SELECT ... FOR UPDATE (读取 status/tenant/project/lifecycle_version)
    expected_version 不匹配 → Conflict（返回期望/当前版本）
    DefectStatus::can_transition(from, to) 不通过 → InvalidTransition（携带具体原因）
    UPDATE defect.work_items SET status, lifecycle_version+1
    进入 in_progress/pending_verify 时写入 fix_version
  [submit_verification]
    SELECT ... FOR UPDATE（含 fix_version、reporter_id）
    验证独立性: 验证人 != 报告人（非管理员）
    fix_version 必须已设定且与提交值一致，否则 VerificationMismatch
    INSERT defect.verifications (verifier_id, fix_version, method, result, evidence_id)
    UPDATE defect.work_items SET status(closed|reopened), fix_version, lifecycle_version+1
  INSERT defect.audit_events (event_type, actor_user_id, tenant_id, project_id, target_id, payload)
  INSERT defect.outbox (aggregate_type, aggregate_id, event_type, payload, status='pending')
  save_idempotency:
    INSERT ... ON CONFLICT DO UPDATE INTO defect.command_idempotency (expires_at = now()+24h)
COMMIT
```

不变量：
- **禁止跨 owner 直接写表**：缺陷域只写 `defect.*`；附件/证据、通知、策略、Agent 运行各归其 owner，
  经版本化 API/事件协作（per SRS §5.1、§5.2）。
- **版本单调**：`lifecycle_version` 每命令 +1，`expected_version` 为乐观锁基线。
- **失败不留半事实**：审计与 Outbox 与业务事实同事务，任一失败整体回滚。
- **授权失败不重放**：403/401 直接返回，不自动重试写入（per SRS §5.2、FR-DI-020）。
- **Outbox 投递端未实现**：事件停留在 `pending`；投递、重复消费、断线补缺属未接入（G-8）。

#### 2.1.1 与物理实现的差异（2026-11-24 核对）

| 设计要求 | 实现位置 | 物理差异 | 缺口 |
|---|---|---|---|
| 设 `app.tenant_id` / `app.actor_id` GUC | `pg_service.rs:148` `set_transaction_scope` | 已实现 | — |
| 幂等锁 `pg_advisory_xact_lock(hashtextextended(...))` | `pg_service.rs:657` | 已实现 | — |
| 幂等查询命中返回**缓存回执** | `pg_service.rs:665-691` | 命中后返回 `build_defect_stub`（空标题/空现象/nil reporter/固定 `lifecycle_version=1`），**未回放 `response_body`** | G-11 |
| `request_hash` 用于同键不同请求判定 | `pg_schema.rs:83` `request_hash BYTEA NOT NULL` | 写入时绑的是 `serde_json::to_vec(&key.0)`（即 key 本身，非请求摘要），**从不比对** | G-11 |
| `FOR UPDATE` 行锁 + `expected_version` CAS | `pg_service.rs:268-294` | 已实现 | — |
| 状态机 `can_transition` 受控流转 | `pg_service.rs:298` | 已实现；`from_pg_str` 对未知状态报 `Internal` 不静默降级 | — |
| 进入 `in_progress`/`pending_verify` 写 `fix_version` | `pg_service.rs:324-338` | 已实现，但为**独立的第二条 UPDATE**，非同一语句 → 与状态 UPDATE 之间存在中间态窗口 | G-19 |
| 验证独立性按**实际修复者/执行者** | `pg_service.rs:394`（读 `reporter_id`） | 比较的是**报告人**，非实际修复者 | G-20（FR-DI-018 未满足） |
| 验证要求 `fix_version` 已设定且匹配 | `pg_service.rs` `submit_verification_on_connection` | 已实现 | — |
| 审计与 Outbox 同事务写入 | `pg_service.rs:720-767` | 已实现 | — |
| 六表 RLS | — | **完全缺失**（`pg_schema.rs` 无 `ROW LEVEL SECURITY` / `CREATE POLICY`） | G-11 |
| 物理迁移文件 | — | **缺失**（`db/migrations/` 无 defect 迁移） | G-11 |
| `defect.outbox` 可承载 envelope | — | **缺列** | G-12 |
| PG 服务被实例化 | `routes/defects.rs:149` `DefectService::new(None)` | `pg` 恒 `None` | G-16 |
| actor 来自已验证会话 | `routes/defects.rs:153` `actor_from_headers` | 随机 UUID | G-15 |

> 上表只登记**差异**，不改变 §2.1 的目标事务序列。每条差异的关闭条件见 §7 缺口表；
> 在缺口关闭前，本节描述的事务**不得**被当作生产已接通。

---

## §3 Frontend module mapping

### 3.1 Kanban page integration (`frontend/src/app/(app)/kanban/page.tsx`)

| View tab | Module | Data source | Create entry |
|----------|--------|-------------|--------------|
| 任务看板 (first tab) | DI-List | `visibleWorkItems` (server + localOnly separated) | Top create bar |
| Sprint | DI-List (filtered by sprint) | Sprint board items | SprintBoardView onQuickAdd |
| 时间线 | DI-List (GanttView) | serverItems / visibleWorkItems | Calendar onDayClick → due_date override |
| 日历 | DI-List (CalendarView) | serverItems / visibleWorkItems | onDayClick → draft due_date |
| 燃尽图 | DI-Quality (Chart01Burndown) | visibleWorkItems | Read-only |

Key integration points already in place:
- `VIEW_TABS = ["任务看板", "Sprint", "IPA 流程图", "按文件开卡", "时间线", "日历", "燃尽图"]` — 任务看板 first.
- `visibleWorkItems` single authoritative set (server items filtered by branch/worktree + local items with `localOnly=true`).
- `idempotencyKey` stable per user action for create/move.
- `unavailableState` honest empty/unauthorized/disconnected states.
- `toStoreWorkItem` throws on missing `tenant_id`/`project_id`/`reporter_id` — no `tn-001`/`usr-002`/`prj-physis` fakes.

### 3.2 Drawer integration (`frontend/src/components/board/WorkItemDetailDrawer.tsx`)

| Tab | Content | Module |
|-----|---------|--------|
| 详情 | Title, Status, Priority, Kind, Assignee, Worktree, Description, Labels, Branch truth | DI-Drawer / DI-Lifecycle |
| CLI | TaskCardCommand + writeLine; card/worktree/global send target | DI-Suggestion (FR-DI-026) |
| 评论 | Mock comment list (Phase 2+ connect store.comments) | DI-Evidence |
| 历史 | Created/updated time, reporter, kind, status transition badges | DI-Lifecycle (defect_status_event) |

> **2026-11-24 实施对齐**：`WorkItemDrawerMode` 的 `"new"` 分支新增 `defaultKind` / `lockKind`
> 字段；抽屉初始化消费 `open.defaultKind ?? "task"`。缺陷页以 `defaultKind="bug"` + `lockKind=true`
> 打开，确保新建走 Bug 类型（per SRS FR-DI-001）。

Drawer save:
- New mode: `Create` → `handleCreate` → `addWorkItem` with `branch_id=null`, `branch_name=selectedBranch`, `localOnly=true`.
- View mode: `Save` → `handleSave` → per-field `updateWorkItemField` (title/status/priority/kind/description/assignee_id/worktree_id/labels).

### 3.2.1 缺陷页与看板卡片（jira 风格字段布局）

`frontend/src/app/(app)/defect/page.tsx` 与 `frontend/src/components/board/KanbanCard.tsx`
共用同一套卡片字段布局（per 用户要求"jira 那种几个字段的设计"）：

| 行 | 内容 | 说明 |
|----|------|------|
| Row 1 | key（`//KEY`）+ 故事点 + 优先级徽标 | jira 风格顶行；优先级带 p0/p1/p2/p3 配色 |
| Row 2 | 标题（line-clamp-2） | 主要信息 |
| Row 3 | kind + status 徽标 | `StatusPill` |
| Row 4 | 负责人 + 聊天按钮 + 架构图按钮 | 交互入口 |
| Row 5 | 标签（前 2 个） | `#label` |
| Row 6 | 关联文档（最多 3 个 + 溢出计数） | 点击直接打开文档 |

### 3.2.2 缺陷页扩展字段的真实来源（目标契约 + 当前物理状态）

per SRS FR-DI-006、FR-DI-009、FR-DI-019 与 BD §0.2 证据诚实约束，缺陷扩展字段必须
来自 owner 持久化读取；未接入时显式显示“未接入/未知”，**不得**由界面状态反推业务事实。

| 字段 | 目标来源 | 当前物理来源（`defect/page.tsx`） | 判定 |
|---|---|---|---|
| 现象 `phenomenon` | `defect.work_items.phenomenon` | `item.description`（WorkItem 描述，非缺陷专有列） | 部分：来源非缺陷扩展字段 |
| 预期结果 `expected_result` | `defect.work_items.expected_result` | `undefined`（`:176`） | 未接入 |
| 实际结果 `actual_result` | `defect.work_items.actual_result` | `undefined`（`:175`） | 未接入 |
| 重现步骤 `reproduction_steps` | `defect.work_items.reproduction_steps` | `undefined`（`:177`） | 未接入 |
| 严重度 `severity` | `defect.work_items.severity` | 常量 `"unknown"`（`:179`） | 未接入 |
| 修复版本 `fix_version` | `defect.work_items.fix_version` | `undefined`（`:186`） | 未接入 |
| 验证状态 | `defect.verifications` 最近一条 `result` | `status === "done" ? "已验证" : "（未提交）"`（`:425`） | **未接入且违反证据诚实约束**：`done` 是看板列状态，非验证事实 |
| 负责人 / 报告者 / Lane | `defect.work_items.assignee_id`/`reporter_id`/`lane_id` | `assignee_id: undefined`（`:180`）、`lane_id: undefined`（`:182`） | 未接入 |
| 创建/更新时间 | `defect.work_items.created_at`/`updated_at` | `created_at: ""`/`updated_at: ""`（`:189-190`） | 未接入 |

**当前列表的数据来源**：`worktreeGroupApi.listWorkItems<GroupWorkItemView[]>` 按 worktree
拉取 WorkItem 投影，过滤 `item.kind === "bug"`（`:145`）。`GroupWorkItemView`
（`frontend/src/lib/group/groupProjection.tsx:120-140`）**不含任何缺陷扩展字段**
（无 `severity`/`fix_version`/`expected_result`/`actual_result`/`reproduction_steps`），
故扩展字段在当前数据契约下**无法**从该投影取得 —— 这是 G-14 的结构性成因，
需要缺陷专属读模型（`GET /defects`）而非扩充 WorkItem 投影。

**目标读模型**（G-14 的关闭条件）：

```ts
// frontend/src/lib/defect/defectApi.ts（目标新增）
export interface DefectDetailView {
  id: string;
  tenant_id: string;
  project_id: string;
  lane_id: string | null;
  worktree_id: string | null;
  branch_id: string | null;
  branch_name: string | null;
  title: string;
  phenomenon: string;
  expected_result: string | null;
  actual_result: string | null;
  reproduction_steps: string | null;
  impact_scope: string | null;
  discovery_stage: string | null;
  severity: "S1" | "S2" | "S3" | "S4" | "unknown";
  priority: "P0" | "P1" | "P2" | "P3" | "unscheduled";
  status: string;                       // 缺陷领域状态，非看板列
  reporter_id: string;
  assignee_id: string | null;
  environment: string | null;
  labels: string[];
  discovery_version: string | null;
  fix_version: string | null;
  lifecycle_version: number;
  created_at: string;
  updated_at: string;
  /** 最近验证结论；null = 未提交验证（不得由 status 推断） */
  latest_verification: {
    verifier_id: string;
    fix_version: string;
    method: string | null;
    result: "passed" | "failed";
    evidence_id: string | null;
    created_at: string;
  } | null;
}
```

**诚实三态渲染规则**（per BD §0.2 / WBS D.3.2）：

| 状态 | 渲染 | 依据 |
|---|---|---|
| 字段已接入且有值 | 显示值 | owner 持久化读取 |
| 字段已接入但为 `null` | 显示“（未填写）” | owner 明确返回空 |
| 字段未接入（读模型缺失该列或请求失败） | 显示“（未接入）”并在该区块顶部标注来源不可用 | 不得回退到占位常量 |

> **验证状态专条**：`latest_verification === null` → “（未提交）”；存在且 `result === "passed"`
> → “已验证（{fix_version}）”；存在且 `result === "failed"` → “验证失败（{fix_version}）”。
> 任何情况下都**不得**由看板 `status === "done"` 推断验证结论。

### 3.2.3 缺陷创建的 kind 锁定（FR-DI-001 未满足项）

`WorkItemDetailDrawer` 已声明 `defaultKind`/`lockKind` prop
（`frontend/src/components/board/WorkItemDetailDrawer.tsx:93-96`），且 `WorkItemDrawerMode`
的 `"new"` 变体带 `defaultKind?`/`lockKind?`（`:82`），draft 初始化读 `open.defaultKind ?? "task"`
（`:206`）。但：

- 函数签名（`:112-114`）**未解构** `defaultKind`/`lockKind` → 作为 prop 传入无效；
- 缺陷页 `handleCreate` 只传 `{ kind: "new", defaultStatus: "todo" }`
  （`frontend/src/app/(app)/defect/page.tsx:240`）→ `open.defaultKind` 为 `undefined`
  → draft kind 落到 `"task"`；
- Kind 下拉（`:584-591`）未按 `lockKind` 禁用 → 即便 kind 正确也可被改回其他类型。

**目标行为**：缺陷页新建必须得到 `kind === "bug"` 且不可切换。两种落地方式（择一实现）：

| 方案 | 改动 | 优点 |
|---|---|---|
| A（推荐） | `WorkItemDetailDrawer` 解构 `defaultKind`/`lockKind`，draft 初始化改为 `kind: defaultKind ?? open.defaultKind ?? "task"`，Kind 下拉在 `lockKind` 时 `disabled` + 只读文本 | 与既有 prop 声明一致，无需改调用方签名 |
| B | 缺陷页 `handleCreate` 传 `{ kind:"new", defaultStatus:"todo", defaultKind:"bug", lockKind:true }`，抽屉同时消费 mode 级与 prop 级 | 保留 mode 级语义，但两处来源需统一优先级 |

无论采用哪种，创建提交路径（`handleCreate` → `addWorkItem`）都必须改为调用 owner 创建命令
（`POST /defects`）并以回执 + revision 刷新列表；本地 `store.addWorkItem` 仅可作为
“未接入”降级，且降级时必须在 UI 标注（per G-13、WBS D.1.2）。

### 3.2.4 多 Worktree 部分失败（FR-DI-006）

当前 `defect/page.tsx:131-142` 对每个 worktree 的 `listWorkItems` 调用用 `catch {}` 静默吞掉
失败，失败范围与原因完全不可见，且局部结果被当作完整结果渲染。

**目标契约**：加载结果须携带**逐 worktree 的范围状态**：

```ts
interface WorktreeLoadOutcome {
  worktree_id: string;
  ok: true;
  items: GroupWorkItemView[];
} | {
  worktree_id: string;
  ok: false;
  /** 401 = 会话能力失效；403 = 该范围被拒；其他 = 该范围不可用 */
  reason: "unauthorized" | "forbidden" | "unavailable";
  message: string;
}

interface DefectListLoad {
  outcomes: WorktreeLoadOutcome[];
  /** 全部成功才为 true；任一失败即 false */
  complete: boolean;
}
```

渲染规则：

- `complete === true` → 正常列表，不额外提示。
- `complete === false` → 列表上方显示失败范围摘要（“2 个工作区读取失败：wt-003（403 权限不足）、wt-007（服务不可用）”），
  **不得**只展示成功部分而不加说明。
- 出现 `reason === "unauthorized"` → 会话能力失效，按 SRS §5.2 提示重新登录。
- 出现 `reason === "forbidden"` → 该范围被拒，其余范围仍可用（403 只关闭被拒绝范围）。

### 3.3 团队页面（`frontend/src/app/team/page.tsx`）

| Tab | 内容 | 状态 |
|-----|------|------|
| 成员与身份 | 人类 / Agent 分区身份列表、状态徽标 | 移除/邀请未接后端 → 禁用并标注原因 |
| Agent 供应商 | 每个 Agent 的 model / baseUrl / 启用开关 | 保存写入 `store.agentSettings`（`initAgentSettings` + `updateAgentSetting`） |
| 小队 | 人机混合小队卡片 | 新建为本地草稿；后端调度未接入 → 标注原因 |
| Agent 会话 | 14 状态机、worktree 绑定、token/cost | 只读表格 |

Tab 切换由 `activeTab` state 驱动，各 section 按 `activeTab !== <id>` 隐藏（原先全部 `hidden` 常驻，
切换无效）。

### 3.4 IPA Skill Workflow Board integration

- `createTaskCard` calls `addWorkItem` with `ipa_skill_ids: [skill.id]`, `labels: ["ipa-skill", skill.phase]`, `onOpenTaskCard(id)`.
- Save button is effective (real `addWorkItem` call, no stub).

### 3.5 Tab-aware shortcut navigation

- `ShortcutButton.targetTab` field defined in navStore.
- Consumer logic (`router.replace('/?tab=...')`) **not yet implemented** — known gap (G-6 in BD).

---

## §4 Database DDL (canonical, matches BD §4)

> **2026-11-24 实现对齐**：下列 `defect_work_item` 等为逻辑名；实际物理实现位于 `defect.*` schema，
> 由 `crates/domain-defect/src/pg_schema.rs` 的 `CREATE_ALL_TABLES` 提供（Neon 兼容）。
> 物理表：`defect.work_items`、`defect.verifications`、`defect.suggestions`、
> `defect.command_idempotency`、`defect.audit_events`、`defect.outbox`。
> 逻辑名 → 物理名映射：缺陷核心事实 → `defect.work_items`；验证记录 → `defect.verifications`；
> 智能建议 → `defect.suggestions`；幂等回执 → `defect.command_idempotency`；
> 审计事件 → `defect.audit_events`；Outbox 事件 → `defect.outbox`。

```sql
-- 缺陷核心事实 (T, SCD2)
CREATE TABLE defect_work_item (
    defect_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    project_id UUID NOT NULL,
    lane_id UUID,
    worktree_id UUID,
    branch_id UUID,
    branch_name TEXT,
    title TEXT NOT NULL,
    phenomenon TEXT NOT NULL,
    expected_result TEXT,
    actual_result TEXT,
    reproduction_steps TEXT,
    impact_scope TEXT,
    discovery_stage TEXT,
    severity TEXT CHECK (severity IN ('S1','S2','S3','S4','unknown')),
    priority TEXT CHECK (priority IN ('P0','P1','P2','P3','unscheduled')),
    status TEXT NOT NULL DEFAULT 'triage',
    reporter_id UUID NOT NULL,
    assignee_id UUID,
    environment TEXT,
    labels TEXT[],
    ipa_skill_ids UUID[],
    discovery_version TEXT,
    fix_version TEXT,
    lifecycle_version INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,
    CONSTRAINT fk_project FOREIGN KEY (project_id) REFERENCES project(project_id)
);

-- 状态/事件日志 (T, Append-only)
CREATE TABLE defect_status_event (
    event_id UUID PRIMARY KEY,
    defect_id UUID NOT NULL REFERENCES defect_work_item(defect_id),
    actor_id UUID NOT NULL,
    event_time TIMESTAMPTZ NOT NULL DEFAULT now(),
    from_status TEXT,
    to_status TEXT,
    changed_fields JSONB,
    reason TEXT,
    lifecycle_version INTEGER NOT NULL,
    idempotency_key UUID NOT NULL,
    request_hash TEXT NOT NULL
);

-- 验证记录 (T)
CREATE TABLE defect_verification_record (
    verification_id UUID PRIMARY KEY,
    defect_id UUID NOT NULL REFERENCES defect_work_item(defect_id),
    verifier_id UUID NOT NULL,
    fix_version TEXT NOT NULL,
    verification_method TEXT,
    result TEXT CHECK (result IN ('pass','fail')),
    evidence_id UUID,
    summary TEXT,
    verified_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 智能建议 (T)
CREATE TABLE defect_ai_suggestion (
    suggestion_id UUID PRIMARY KEY,
    defect_id UUID NOT NULL REFERENCES defect_work_item(defect_id),
    input_revision TEXT NOT NULL,
    model_profile TEXT NOT NULL,
    suggestion_type TEXT,
    content_summary TEXT,
    status TEXT DEFAULT 'pending',
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 工作流策略 (M, SCD2)
CREATE TABLE defect_workflow_strategy (
    strategy_id UUID PRIMARY KEY,
    project_id UUID NOT NULL,
    lane_id UUID,
    strategy_version INTEGER NOT NULL,
    valid_from TIMESTAMPTZ NOT NULL,
    valid_until TIMESTAMPTZ,
    status_rules JSONB,
    field_rules JSONB,
    sla_rules JSONB,
    authorization_rules JSONB,
    created_by UUID NOT NULL
);

-- 索引
CREATE INDEX idx_defect_list ON defect_work_item(tenant_id, project_id, lane_id, status);
CREATE INDEX idx_defect_branch ON defect_work_item(branch_id, branch_name);
CREATE INDEX idx_defect_event ON defect_status_event(defect_id, event_time DESC);
CREATE INDEX idx_defect_verify ON defect_verification_record(defect_id);
CREATE INDEX idx_defect_suggestion ON defect_ai_suggestion(defect_id, status);
```

### 4.1 实际物理 DDL（`defect.*` schema，Neon 兼容）

来源：`crates/domain-defect/src/pg_schema.rs`（`CREATE_ALL_TABLES`）。

```sql
CREATE SCHEMA IF NOT EXISTS defect;

CREATE TABLE IF NOT EXISTS defect.work_items (
    defect_id          UUID PRIMARY KEY,
    tenant_id          UUID NOT NULL,
    project_id         UUID NOT NULL,
    lane_id            UUID,
    worktree_id        UUID,
    branch_id          UUID,
    branch_name        TEXT,
    title              TEXT NOT NULL,
    phenomenon         TEXT NOT NULL,
    expected_result    TEXT,
    actual_result      TEXT,
    reproduction_steps TEXT,
    impact_scope       TEXT,
    discovery_stage    TEXT,
    severity           TEXT NOT NULL DEFAULT 'unknown',
    priority           TEXT NOT NULL DEFAULT 'unscheduled',
    status             TEXT NOT NULL DEFAULT 'triage',
    reporter_id        UUID NOT NULL,
    assignee_id        UUID,
    environment        TEXT,
    labels             TEXT[] NOT NULL DEFAULT '{}',
    discovery_version  TEXT,
    fix_version        TEXT,
    lifecycle_version  INTEGER NOT NULL DEFAULT 1,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at         TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_tenant_project
    ON defect.work_items(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_lane
    ON defect.work_items(lane_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_status
    ON defect.work_items(status);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_fix_version
    ON defect.work_items(fix_version);

CREATE TABLE IF NOT EXISTS defect.verifications (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    verifier_id    UUID NOT NULL,
    fix_version    TEXT NOT NULL,
    method         TEXT,
    result         TEXT NOT NULL CHECK (result IN ('passed','failed')),
    evidence_id    UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at     TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_defect
    ON defect.verifications(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_verifier
    ON defect.verifications(verifier_id);

CREATE TABLE IF NOT EXISTS defect.suggestions (
    id                     UUID PRIMARY KEY,
    defect_id              UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    suggested_severity     TEXT,
    suggested_priority     TEXT,
    suggested_discovery_stage TEXT,
    reason                 TEXT,
    status                 TEXT NOT NULL DEFAULT 'draft',
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_suggestions_defect
    ON defect.suggestions(defect_id);

CREATE TABLE IF NOT EXISTS defect.command_idempotency (
    tenant_id      UUID NOT NULL,
    actor_id       UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash   BYTEA NOT NULL,
    response_body  JSONB NOT NULL,
    expires_at     TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);
CREATE INDEX IF NOT EXISTS idx_defect_idempotency_expires
    ON defect.command_idempotency(expires_at);

CREATE TABLE IF NOT EXISTS defect.audit_events (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type     TEXT NOT NULL,
    actor_user_id  UUID NOT NULL,
    tenant_id      UUID NOT NULL,
    project_id     UUID,
    target_id      TEXT,
    payload        JSONB NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_tenant
    ON defect.audit_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_project
    ON defect.audit_events(project_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_created
    ON defect.audit_events(created_at);

CREATE TABLE IF NOT EXISTS defect.outbox (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type TEXT NOT NULL,
    aggregate_id   TEXT NOT NULL,
    event_type     TEXT NOT NULL,
    payload        JSONB NOT NULL,
    status         TEXT NOT NULL DEFAULT 'pending'
                   CHECK (status IN ('pending','sent','failed')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at        TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status
    ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created
    ON defect.outbox(created_at);
```

> `defect.work_items.deleted_at` 为软删除位；业务事实禁止物理删除（per SRS §5.1）。
> `defect.command_idempotency` 以 `(tenant_id, actor_id, idempotency_key)` 为主键，
> 保证重复命令在 owner 事务内唯一（per SRS FR-DI-020、NFR-DI-05）。

### 4.2 目标物理 DDL（迁移文件内容，**当前不存在**，per G-11 / G-12）

以下为缺陷域迁移文件的**目标内容**，尚未在 `db/migrations/` 下落地。
落地要求（per BD §4.4）：每行一条语句，不含 `BEGIN`/`COMMIT`（迁移 runner 自带事务）；
表名与 `crates/domain-defect/src/pg_schema.rs` 的 `CREATE_ALL_TABLES` 一致；
RLS 与触发器参照 `db/migrations/2026-09-29-worktree-group-phase-2c.sql` 的既有写法。

```sql
CREATE SCHEMA IF NOT EXISTS defect;

-- 缺陷核心事实 (T, SCD2；业务事实禁止物理删除)
CREATE TABLE IF NOT EXISTS defect.work_items (
    defect_id          UUID PRIMARY KEY,
    tenant_id          UUID NOT NULL,
    project_id         UUID NOT NULL,
    lane_id            UUID,
    worktree_id        UUID,
    branch_id          UUID,
    branch_name        TEXT,
    title              TEXT NOT NULL,
    phenomenon         TEXT NOT NULL,
    expected_result    TEXT,
    actual_result      TEXT,
    reproduction_steps TEXT,
    impact_scope       TEXT,
    discovery_stage    TEXT,
    severity           TEXT NOT NULL DEFAULT 'unknown' CHECK (severity IN ('S1','S2','S3','S4','unknown')),
    priority           TEXT NOT NULL DEFAULT 'unscheduled' CHECK (priority IN ('P0','P1','P2','P3','unscheduled')),
    status             TEXT NOT NULL DEFAULT 'triage' CHECK (status IN ('triage','confirmed','in_progress','pending_verify','closed','pending_info','duplicate','no_action','reopened')),
    reporter_id        UUID NOT NULL,
    assignee_id        UUID,
    environment        TEXT,
    labels             TEXT[] NOT NULL DEFAULT '{}',
    discovery_version  TEXT,
    fix_version        TEXT,
    lifecycle_version  INTEGER NOT NULL DEFAULT 1,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at         TIMESTAMPTZ
);

-- 验证记录 (T)
CREATE TABLE IF NOT EXISTS defect.verifications (
    id             UUID PRIMARY KEY,
    defect_id      UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    verifier_id    UUID NOT NULL,
    fix_version    TEXT NOT NULL,
    method         TEXT,
    result         TEXT NOT NULL CHECK (result IN ('passed','failed')),
    evidence_id    UUID,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at     TIMESTAMPTZ
);

-- 智能建议草稿 (T)
CREATE TABLE IF NOT EXISTS defect.suggestions (
    id                        UUID PRIMARY KEY,
    defect_id                 UUID NOT NULL REFERENCES defect.work_items(defect_id) ON DELETE CASCADE,
    suggested_severity        TEXT,
    suggested_priority        TEXT,
    suggested_discovery_stage TEXT,
    reason                    TEXT,
    status                    TEXT NOT NULL DEFAULT 'draft',
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 幂等回执 (T；同键不同请求按 request_hash 判定)
CREATE TABLE IF NOT EXISTS defect.command_idempotency (
    tenant_id       UUID NOT NULL,
    actor_id        UUID NOT NULL,
    idempotency_key TEXT NOT NULL,
    request_hash    BYTEA NOT NULL,
    response_body   JSONB NOT NULL,
    expires_at      TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (tenant_id, actor_id, idempotency_key)
);

-- 审计事件 (T；Append-only，禁 UPDATE/DELETE)
CREATE TABLE IF NOT EXISTS defect.audit_events (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type    TEXT NOT NULL,
    actor_user_id UUID NOT NULL,
    tenant_id     UUID NOT NULL,
    project_id    UUID,
    target_id     TEXT,
    payload       JSONB NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outbox 事件 (T；增补 envelope 列后方可被 defect_outbox 消费者消费)
CREATE TABLE IF NOT EXISTS defect.outbox (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_id        UUID NOT NULL,
    source_sequence  BIGINT NOT NULL,
    event_id         UUID NOT NULL UNIQUE,
    event_type       TEXT NOT NULL,
    schema_version   INTEGER NOT NULL DEFAULT 1,
    producer_id      TEXT NOT NULL,
    consumer_id      TEXT NOT NULL,
    tenant_id        UUID NOT NULL,
    actor_id         UUID,
    correlation_id   UUID NOT NULL,
    event_digest     BYTEA NOT NULL CHECK (octet_length(event_digest) = 32),
    payload          JSONB NOT NULL,
    occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    status           TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','sent','failed')),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    sent_at          TIMESTAMPTZ,
    UNIQUE (stream_id, source_sequence)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_defect_work_items_tenant_project ON defect.work_items(tenant_id, project_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_lane ON defect.work_items(lane_id);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_status ON defect.work_items(status);
CREATE INDEX IF NOT EXISTS idx_defect_work_items_fix_version ON defect.work_items(fix_version);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_defect ON defect.verifications(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_verifications_verifier ON defect.verifications(verifier_id);
CREATE INDEX IF NOT EXISTS idx_defect_suggestions_defect ON defect.suggestions(defect_id);
CREATE INDEX IF NOT EXISTS idx_defect_idempotency_expires ON defect.command_idempotency(expires_at);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_tenant ON defect.audit_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_project ON defect.audit_events(project_id);
CREATE INDEX IF NOT EXISTS idx_defect_audit_events_created ON defect.audit_events(created_at);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_status ON defect.outbox(status);
CREATE INDEX IF NOT EXISTS idx_defect_outbox_created ON defect.outbox(created_at);

-- RLS：逐表 ENABLE + FORCE + tenant 策略
ALTER TABLE defect.work_items ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.work_items FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_work_items_tenant_scope ON defect.work_items USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.verifications ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.verifications FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_verifications_tenant_scope ON defect.verifications USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.verifications.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.verifications.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.suggestions ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.suggestions FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_suggestions_tenant_scope ON defect.suggestions USING (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.suggestions.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID)) WITH CHECK (EXISTS (SELECT 1 FROM defect.work_items w WHERE w.defect_id = defect.suggestions.defect_id AND w.tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID));
ALTER TABLE defect.command_idempotency ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.command_idempotency FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_idempotency_tenant_scope ON defect.command_idempotency USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.audit_events ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.audit_events FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_audit_events_tenant_scope ON defect.audit_events USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
ALTER TABLE defect.outbox ENABLE ROW LEVEL SECURITY;
ALTER TABLE defect.outbox FORCE ROW LEVEL SECURITY;
CREATE POLICY defect_outbox_tenant_scope ON defect.outbox USING (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID) WITH CHECK (tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID);
```

> **枚举 CHECK 约束的来源**：缺陷状态取值来自 `domain_defect::DefectStatus::as_str()`
> （`crates/domain-defect/src/lib.rs`，共 9 个：`triage`/`confirmed`/`in_progress`/`pending_verify`/
> `closed`/`pending_info`/`duplicate`/`no_action`/`reopened`）；严重度 5 个、优先级 5 个。
> 迁移中的 CHECK 列表必须与代码枚举**逐值一致**，新增枚举值时同步迁移与 `from_pg_str`
> （`crates/domain-defect/src/pg_service.rs:951`），否则 PG 写入被 CHECK 拒绝而读侧报
> `Internal`（未知状态不静默降级，per §2.1.1）。
>
> **审计不可变触发器**：`defect.audit_events` 须挂禁 UPDATE/DELETE 的触发器，
> 参照 `multica.reject_task_audit_mutation()`（`db/migrations/2026-09-29-worktree-group-phase-2c.sql:137`）。
>
> **Outbox 序列分配**：`source_sequence` 由 `COALESCE(MAX(source_sequence),0)+1` 在同 stream 内分配，
> 参照 `identity_device_outbox` 的写法（`crates/star-api-rest/src/device_outbox/identity_owner.rs:179-185`）；
> `(stream_id, source_sequence)` 唯一约束保证 owner 单调重放顺序。

---

## §5 前端关键代码片段（摘要，与实现保持一致）

### 5.1 Local create — branch_id null, branch_name records branch

```ts
addWorkItem({
  tenant_id: tenantId,
  project_id: projectId,
  title: draft.title.trim(),
  status: draft.status,
  kind: draft.kind,
  priority: draft.priority,
  reporter_id: reporterId,
  description: draft.description,
  assignee_id: draft.assignee_id,
  labels: draft.labels,
  worktree_id: draft.worktree_id,
  // branch_id 保留为服务端 UUID（本地任务无，设 null）；branch_name 记录本地分支名
  branch_id: null,
  branch_name: useNavStore.getState().selectedBranch || null,
  localOnly: true as any,
});
```

### 5.2 Local task filter — uses branch_name, not branch_id

```ts
const localItemsFiltered = boardWorkItems.filter(
  (w) =>
    w.branch_name === selectedBranch &&
    (!selectedWorktreeId || w.worktree_id === selectedWorktreeId) &&
    (w as WorkItem & { localOnly?: boolean }).localOnly === true,
);
```

### 5.3 Charts — use unified visible set

```ts
const charts = useMemo(
  () => buildAnalyticsData(sprints, visibleWorkItems),
  [serverMode, sprints, visibleWorkItems],
);
```

---

## §6 测试钩子与验收映射

| AC | 覆盖要件 | 测试钩子 | 通过条件 |
|----|---------|---------|---------|
| AC-DI-01 | FR-DI-001～006、008～009 | 两个 Project、两个 Lane、同名 Branch 及无 Worktree Lane 下创建/筛选/刷新 | ID 与归属正确，用户只看到有权结果，未接入或断线不显示生产缓存 |
| AC-DI-02 | FR-DI-007、010～014 | 批量操作含成功、403、版本冲突；逐条回执准确 | 关联跨 Project、失权、循环和重复均受约束；历史、附件及草稿可恢复 |
| AC-DI-03 | FR-DI-015～020 | 分诊→修复→独立验证→关闭→重开 | 验证失败、未关闭阻塞项、策略不满足与无权限时返回阻断原因；冲突、重复提交和回执丢失不产生重复事实 |
| AC-DI-04 | FR-DI-021～025、027～028 | 输入受限日志并生成候选，记录来源及 revision | 无权数据不出现在候选或计数；证据修改或撤权后建议过期；用户拒绝不改变正式字段 |
| AC-DI-05 | FR-DI-026、NFR-DI-03～05 | 真实受控环境证明身份、Worktree、预算、Hook、sandbox、执行、取消、独立验证和回写 | 缺任一依赖时执行禁用并给出原因 |
| AC-DI-06 | FR-DI-029～032、NFR-DI-01/05/07 | 两个权限不同的客户端接收变更 | 断线补缺、重投及慢消费者恢复；通知不重复；汇总数字与明细一致 |
| AC-DI-07 | FR-DI-033～034、NFR-DI-08 | 配置不同时区/工作日历并校验暂停、重开和超时 | 归档恢复和历史迁移遇冲突/未知状态可追溯、可恢复 |
| AC-DI-08 | NFR-DI-02/03/06/09/10 | 按指定数据量和负载测量 p95、错误率与资源上限 | 桌面/窄屏/缩放/键盘检查；每种部署及固定 AI 样本上记录可复现结果 |

### 6.1 已有自动化测试（代码级证据，2026-11-24 `dev@b1db1c99`）

| 测试 | 位置 | 覆盖 | 结果 |
|---|---|---|---|
| 缺陷生命周期 14 项 | `crates/domain-defect/tests/defect_lifecycle_test.rs` | 跨 tenant/Project 拒绝、任意状态跳转拒绝、首次提交修复版本、独立验证（验证人 ≠ 报告人）、版本冲突、幂等重试（create/transition） | **14 passed / 0 failed** |
| Outbox 契约单元测试 | `crates/star-api-rest/src/defect_outbox/contract.rs` `mod tests` | digest 长度与稳定性、envelope 校验（正常 / 摘要不符 / 未知类型）、`decide_apply`（Accept / SequenceGap） | 随 `cargo check` 编译通过（用例在 lib 内） |
| 前端全量回归 | `frontend` `vitest run` | 170 文件 / 1535 用例 | **1535 passed / 0 failed** |

### 6.2 尚缺的自动化测试（AC 关闭前必须补）

| 目标 | 缺失内容 | 阻塞项 |
|---|---|---|
| PG 下的 owner 事务 | 跨 tenant/Project 拒绝、`FOR UPDATE` 并发、幂等回放（真 `response_body`）、RLS 策略生效 | G-11、G-15、G-16 |
| REST 契约 | 6 端点的授权/边界场景（含 401/403/409/422 状态码区分） | G-15、G-16、G-18 |
| Outbox 消费回路 | envelope 校验 → `decide_apply` → 幂等 inbox 写入 → 游标推进 → 重复投递/摘要冲突/序列缺口/断线补缺 | G-12 |
| 前端缺陷页 | kind 锁定、多 Worktree 部分失败范围展示、扩展字段三态渲染、验证状态不由 `done` 反推 | G-13、G-14、G-17 |
| 浏览器端到端 | 真实授权与持久化环境下“创建 → 修复 → 独立验证 → 关闭”一条完整链路 | 上述全部 + Neon 环境 |

> 以上测试缺口与 §7 缺口表一一对应；在对应缺口关闭前，AC-DI-01～08 保持 `NOT_RUN`。

---

## §7 已知缺口（显式列，per 缺标比错标）

### 2026-11-24 实现状态与部署边界

本节登记当前候选提交（`dev@b1db1c99`）的实现状态，不改变前文目标契约。
判定规则见基本设计 §3.5.2（调用可达 / 物理持久化 / 授权真实 / 数据同源）。

**已具备（代码级）**
- 领域骨架（`crates/domain-defect/src/lib.rs`）：受控状态流转表（9 状态，`Duplicate`/`NoAction` 终态，
  禁止任意状态直接跳 Closed）、租户/Project/能力校验、`fix_version` 字段、独立验证记录、
  幂等去重、`lifecycle_version` 乐观锁；`ActorContext` 改为复用 `star_context` 权威定义
  （`pub use star_context::ActorContext`），不再自造副本。生命周期测试 14/14 通过。
- PG 适配器（`pg_schema.rs` + `pg_service.rs`）：`defect.*` 六表 DDL 常量 + owner 事务适配器
  （设 GUC → 授权校验 → 幂等锁 → 幂等查询 → 写事实 + audit + outbox → 保存幂等回执）。
- REST 路由（`crates/star-api-rest/src/routes/defects.rs`）：6 端点已注册于
  `build_legacy_mirror_routes`（`src/lib.rs:215-220`）；`DefectService` 实现
  `DefectCommandPort`/`DefectQueryPort` 并在 PG/InMemory 间分派。
- 错误映射（`src/error.rs:863-892`）：`From<domain_defect::DefectError> for RestError`
  覆盖 9 个变体。
- Outbox 契约层（`crates/star-api-rest/src/defect_outbox/`）：`contract.rs`（envelope +
  `validate_envelope` + `decide_apply`）、`consumer.rs`（`DefectInboxStore` +
  `InMemoryDefectInbox` + `PgDefectInboxStore` 占位）、`relay.rs`（`BoundedRelay` +
  `RelayBackoffPolicy` + `ProfileSwitchOutcome`）。
- 前端：缺陷页接入统一 `WorkItemDetailDrawer`；jira 风格卡片字段（§3.2.1）；
  团队页 Tab 切换接上 state、供应商保存接 `store.agentSettings`；任务看板为 `/kanban` 第一个标签页。

**未接入（保持 fail-closed）**

| 能力 | 物理证据 | 缺口 |
|---|---|---|
| 六表物理存在 + RLS | `db/migrations/` 无 defect 迁移；`pg_schema.rs` 无 `ROW LEVEL SECURITY`/`CREATE POLICY` | G-11 |
| 幂等回执保真 | `pg_service.rs:687` 幂等命中返回 `build_defect_stub`（空字段）；`request_hash` 绑的是 key 本身而非请求摘要，从不比对 | G-11 |
| 审计不可变 | 无禁 UPDATE/DELETE 的触发器 | G-11 |
| Outbox 可承载 envelope | `defect.outbox` 缺 `event_id`/`stream_id`/`source_sequence`/`correlation_id`/`event_digest`/`schema_version`/`occurred_at` | G-12 |
| PG 服务被实例化 | `routes/defects.rs:149` `DefectService::new(None)` → `pg` 恒 `None` | G-16 |
| 真实会话身份 | `routes/defects.rs:153` `actor_from_headers` 随机 UUID；`auth_layer_stub` 为桩 | G-15 |
| HTTP 状态码区分 | `IntoResponse` 的 code→status 表未登记 `CROSS_TENANT_DENIED`/`CROSS_PROJECT_DENIED`/`INVALID_STATE_TRANSITION`/`IDEMPOTENCY_KEY_USED`/`VERIFICATION_MISMATCH` | G-18 |
| 缺陷创建 kind | 抽屉 `defaultKind`/`lockKind` 未被解构消费；缺陷页只传 `{kind:"new",defaultStatus:"todo"}` → 实际 kind 为 `"task"` | G-13 |
| 缺陷扩展字段来源 | `severity` 恒 `"unknown"`、`actual_result`/`expected_result`/`fix_version` 恒 `undefined`；`GroupWorkItemView`（`groupProjection.tsx:120-140`）不含缺陷扩展字段 | G-14 |
| 验证状态诚实性 | `defect/page.tsx:425` 由 `status === "done"` 反推“已验证” | G-14 |
| 多 Worktree 部分失败 | `defect/page.tsx:138` 单 worktree 失败被 `catch {}` 吞掉 | G-17 |
| 验证独立性 | 比较“验证人 vs 报告人”，未按策略校验实际修复者/执行者（per FR-DI-018） | G-20 |
| `fix_version` 原子性 | `pg_service.rs:324-338` 为独立第二条 UPDATE，与状态 UPDATE 之间存在中间态窗口 | G-19 |
| 单一写入路径 | Lane Backlog / 缺陷页 / 抽屉三处新建均走本地 `store.addWorkItem`，与 `/defects` REST 并行 | G-13 |
| 其余模块整合 | Task 详情 CLI/Agent/取消/恢复/独立验证、AI 通知事件、BI 统计口径统一、设置深链修复 | G-9（Phase E） |

**部署边界**
Vercel 承载 Web 呈现与同源 BFF，业务 Owner 与持久化仍在 Star 侧；缺陷域不部署为
Vercel Function（per §0.1、ADR-STAR-VERCEL-DEPLOY-001 §2）。`defects.rs` 路由部署在
`star-api-rest`（Rust 侧）。

**验收状态**
AC-DI-01～08 全部 `NOT_RUN`。本节不得替代候选提交在真实环境取得的端到端证据。

### 2026-11-23 验收与实现差异

本节记录实现差异，不改变前文目标契约。上一轮 `dev@ddb0b6a6` 验收未通过；后续 `dev@df672de7` 的缺陷页已打开 `WorkItemDetailDrawer`，但创建模式只设置 `defaultStatus`，独立 `defaultKind="bug"` 未由抽屉初始化消费，且创建走本地 `addWorkItem`。缺陷列表对单个 Worktree 读取错误静默继续，扩展字段仍为未知值，不能依据 `done` 显示验证已通过。

`dev@df672de7` 的 `InMemoryDefectService` 新增租户/Project 检查、状态迁移、验证记录和幂等结构，仍为内存实现，默认 audit/outbox 为 Noop；其验证要求当前 `fix_version` 已存在，而该 dev 版本未提供设置修复版本的状态命令。`main@4f54b2c9` 新增状态迁移命令的 `fix_version` 字段与测试文件，尚未在本次文档同步中执行测试，亦不证明该改动已进入 dev 或生产 owner。

该 dev 版本的状态表允许 `PendingVerify → Closed`，需在命令入口确认通过持久化独立验证才能关闭；现有验证独立性检查比较验证者和报告者，尚须按策略校验实际修复者/执行者。以上为代码审阅中发现的后续验收条件，不视为已在生产入口触发。

实现前必须核对并补齐：Bug WorkItem 权威身份和缺陷扩展的事务边界；租户、Project/Lane、Worktree 及身份的实时授权；完整状态/版本校验；独立验证身份、固定版本和证据；持久化审计、幂等与 Outbox；部分失败和草稿恢复。所有 AC-DI-01～08 保持 `NOT_RUN`，直至在固定候选提交与实际部署环境取得对应验收证据。

| # | 缺口 | 状态 |
|---|------|------|
| G-1 | 外部问题系统导入或双向同步（连接授权、许可、字段映射、冲突策略） | 后续可选，不作为本期闭环前提 |
| G-2 | 真实 AI provider 接入 | 本仓不涉，走 `crates/star-llm/` |
| G-3 | 真实持久化（PostgreSQL writer/API） | 适配器与 DDL 常量已落地；**物理迁移文件、RLS、PG 接线、会话身份均缺**，生产仍无真实持久化（拆分到 G-11/G-15/G-16） |
| G-4 | 工作流策略版本化 UI 配置 | 基本设计确定后实施，当前为策略数据结构 |
| G-5 | 桌面/窄屏/缩放/键盘/屏幕阅读器验收 | 以实际页面和辅助技术测试，失败提供文本原因 |
| G-6 | Tab 感知跳转消费逻辑（`router.replace('/?tab=...')`） | ShortcutButton.targetTab 字段已定义，消费逻辑待实现 |
| G-7 | AI 数据出站与模型 profile | 部署前确认数据边界与 provider 策略 |
| G-8 | 审计/Outbox 投递端（消费者） | 契约层已就绪；物理表（依赖 G-12）与生产投递回路未实现 |
| G-9 | Lane Backlog / Sprint / 独立看板入口收敛 | 仍有各自入口，共享写入路径未统一（依赖 G-13） |
| G-10 | 缺陷扩展字段与验证结论的真实来源 | 前端仍显示未知；不得由 `done` 状态反推“已验证”（依赖 G-14） |
| G-11 | `defect.*` 六表的物理迁移文件、RLS 与不可变触发器 | `db/migrations/` 无 defect 迁移；`pg_schema.rs` 无 RLS；审计无禁改触发器。关闭条件：新增迁移文件含六表 DDL + 逐表 `ENABLE`/`FORCE ROW LEVEL SECURITY` + `tenant_id` 策略 + 索引 + 审计不可变触发器，并经迁移 runner 应用 |
| G-12 | `defect.outbox` 增补 envelope 物理列 | 缺 `event_id`/`stream_id`/`source_sequence`/`correlation_id`/`event_digest`/`schema_version`/`occurred_at`。关闭条件：DDL 增补 + `(stream_id, source_sequence)` 唯一约束 + `event_digest` 长度约束 + 消费者回路测试（校验→apply→幂等→游标）通过 |
| G-13 | 缺陷新建统一走 owner 写入路径 | 三处新建均走本地 `store.addWorkItem`；抽屉 `defaultKind`/`lockKind` 未消费 → 实际 kind 为 `"task"`。关闭条件：三处统一调 owner 创建命令 + 以回执/revision 刷新 + 抽屉真正解构 `defaultKind`/`lockKind` 且 Kind 下拉在锁定时禁用 |
| G-14 | 缺陷扩展字段与验证状态接真实来源 | `severity`/`actual_result`/`expected_result`/`fix_version` 为占位值；验证状态由 `status==="done"` 反推。关闭条件：新增缺陷专属读模型（含 `latest_verification`）+ 三态渲染（值/未填写/未接入）+ 删除 `done` 反推 |
| G-15 | REST 路由使用真实会话身份 | `actor_from_headers` 随机 UUID，`auth_layer_stub` 为桩。关闭条件：从已验证会话取 `ActorContext`，租户/Project/能力校验基于真实身份，401/403 可区分 |
| G-16 | `DefectService` 注入真实 `PgPool` | `DEFECT_SERVICE` 恒 `DefectService::new(None)`。关闭条件：启动流程注入 Neon `DATABASE_URL` 的 pool，保留显式降级开关与降级原因标注 |
| G-17 | 多 Worktree 部分失败显式化 | 单 worktree 失败被吞。关闭条件：逐 worktree 结果（成功/失败范围/原因）进入 UI，`complete` 标志驱动“部分结果”提示，局部结果不得标为完整 |
| G-18 | 缺陷错误码 → HTTP 状态码映射 | `IntoResponse` 表未登记 5 个缺陷错误码。关闭条件：补 `CROSS_TENANT_DENIED`/`CROSS_PROJECT_DENIED` → 403，`INVALID_STATE_TRANSITION` → 409，`IDEMPOTENCY_KEY_USED` → 409，`VERIFICATION_MISMATCH` → 422，并补契约测试 |
| G-19 | `fix_version` 与状态流转同语句原子写 | `pg_service.rs:324-338` 为独立第二条 UPDATE。关闭条件：合并为单条 `UPDATE ... SET status, lifecycle_version, fix_version` 或用 `SELECT ... FOR UPDATE` 包裹的同一事务步 |
| G-20 | 验证独立性按实际修复者/执行者校验 | 当前比较“验证人 vs 报告人”。关闭条件：引入修复者/执行者身份事实（修复任务 owner 事件或缺陷字段），按生效策略校验，例外须显式记录授权理由（per FR-DI-018） |

---

## 修订历史

| 日期 | 版本 | 内容 |
|------|------|------|
| 2026-11-22 | v1.0 | 建立智能缺陷管理详细设计：实体与版本、API 契约、Project owner 事务、前端模块映射、数据库 DDL、前端代码片段、测试钩子与验收映射、已知缺口 |
| 2026-11-23 | v1.1 | 对照 dev 与 main 的实际缺陷实现和上一轮验收，登记未闭环接口与证据限制。 |
| 2026-11-24 | v1.2 | 对齐 SRS 2026-11-24 现行适用说明；新增 §0.1 部署边界（Vercel 仅 Web/BFF）、§2.1 缺陷 Owner 事务、§3.2.1 jira 风格卡片字段、§3.3 团队页、§4.1 `defect.*` 物理 DDL；登记 G-8～G-10 与实现状态。 |
| 2026-11-24 | v1.3 | 按物理事实核对更新：§0 实体表补 envelope/幂等回执/审计触发器的目标身份差异；§1.0 登记已接线 REST 契约与诚实边界、§1.0.1 保留目标契约；§2.1.1 增补 owner 事务与物理实现的逐条差异表；§3.2.2～§3.2.4 新增扩展字段真实来源与三态渲染、kind 锁定、多 Worktree 部分失败契约；§4.2 增补目标物理 DDL（迁移文件内容，含 RLS 与 Outbox envelope 列）；§6.1/§6.2 登记已有测试证据与缺口；§7 新增缺口 G-11～G-20。 |
