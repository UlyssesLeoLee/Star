# DD-DEFECT-INTELLIGENCE-001

> 智能缺陷管理 — 详细设计 v1.2 (Draft；实现部分完成，验收未通过)
> 上游：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` v1.1（2026-11-24 现行适用说明）、`docs/basic-design/BD-DEFECT-INTELLIGENCE-001.md` v1.2
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

缺陷页额外区块（DI-Drawer 扩展）：
- "缺陷扩展详情"：现象描述、实际结果、修复与验证（修复版本 / 验证状态）。
- 真实来源未接入时显示"（未填写）/（未设定）/（未提交）"，**不得**由 `done` 状态反推"已验证"
  （per SRS FR-DI-006、009、019）。

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
| AC-DI-01 | FR-001～006、008～009 | 两个 Project、两个 Lane、同名 Branch 及无 Worktree Lane 下创建/筛选/刷新 | ID 与归属正确，用户只看到有权结果，未接入或断线不显示生产缓存 |
| AC-DI-02 | FR-007、010～014 | 批量操作含成功、403、版本冲突；逐条回执准确 | 关联跨 Project、失权、循环和重复均受约束；历史、附件及草稿可恢复 |
| AC-DI-03 | FR-015～020 | 分诊→修复→独立验证→关闭→重开 | 验证失败、未关闭阻塞项、策略不满足与无权限时返回阻断原因；冲突、重复提交和回执丢失不产生重复事实 |
| AC-DI-04 | FR-021～025、027～028 | 输入受限日志并生成候选，记录来源及 revision | 无权数据不出现在候选或计数；证据修改或撤权后建议过期；用户拒绝不改变正式字段 |
| AC-DI-05 | FR-026、NFR-03～05 | 真实受控环境证明身份、Worktree、预算、Hook、sandbox、执行、取消、独立验证和回写 | 缺任一依赖时执行禁用并给出原因 |
| AC-DI-06 | FR-029～032、NFR-01/05/07 | 两个权限不同的客户端接收变更 | 断线补缺、重投及慢消费者恢复；通知不重复；汇总数字与明细一致 |
| AC-DI-07 | FR-033～034、NFR-08 | 配置不同时区/工作日历并校验暂停、重开和超时 | 归档恢复和历史迁移遇冲突/未知状态可追溯、可恢复 |
| AC-DI-08 | NFR-02/03/06/09/10 | 按指定数据量和负载测量 p95、错误率与资源上限 | 桌面/窄屏/缩放/键盘检查；每种部署及固定 AI 样本上记录可复现结果 |

---

## §7 已知缺口（显式列，per 缺标比错标）

### 2026-11-24 实现状态与部署边界

本节登记当前候选提交的实现状态，不改变前文目标契约。

**已具备（代码级）**
- 领域骨架（`crates/domain-defect/src/lib.rs`）：受控状态流转表、租户/Project/能力校验、
  `fix_version` 字段、独立验证记录、幂等去重、`lifecycle_version` 乐观锁；生命周期测试 14/14 通过。
- PG 持久化（`pg_schema.rs` + `pg_service.rs`）：`defect.*` 六表 DDL + owner 事务适配器
  （见 §2.1、§4.1）。
- 前端：缺陷页接入统一 `WorkItemDetailDrawer`（`defaultKind="bug"`）；jira 风格卡片字段
  （§3.2.1）；团队页 Tab 切换接上 state、供应商保存接 `store.agentSettings`（§3.3）；
  任务看板为 `/kanban` 第一个标签页。

**未接入（保持 fail-closed）**
- `defects.rs` 路由未接入 `star-api-rest`：PG 适配器已有但生产流量仍未走持久化（G-3）。
- 审计/Outbox 投递端（消费者）未实现，事件停留 `pending`（G-8）。
- 缺陷扩展字段与验证结论的真实来源未接入；列表对单个 Worktree 读取失败仍静默继续（G-10）。
- 验证独立性仅比较"验证人 vs 报告人"，未按策略校验实际修复者/执行者（per SRS FR-DI-018）。
- Lane Backlog / Sprint / 独立看板入口未收敛为单一写入路径（G-9）。
- 任务详情 CLI/Agent/取消/恢复/独立验证、AI 通知事件、BI 口径统一、设置深链修复均未完成。

**部署边界**
Vercel 承载 Web 呈现与同源 BFF，业务 Owner 与持久化仍在 Star 侧；缺陷域不部署为
Vercel Function（per §0.1、ADR-STAR-VERCEL-DEPLOY-001 §2）。

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
| G-3 | 真实持久化（PostgreSQL writer/API） | PG 适配器与 DDL 已落地；**路由未接入**，生产仍无真实持久化 |
| G-4 | 工作流策略版本化 UI 配置 | 基本设计确定后实施，当前为策略数据结构 |
| G-5 | 桌面/窄屏/缩放/键盘/屏幕阅读器验收 | 以实际页面和辅助技术测试，失败提供文本原因 |
| G-6 | Tab 感知跳转消费逻辑（`router.replace('/?tab=...')`） | ShortcutButton.targetTab 字段已定义，消费逻辑待实现 |
| G-7 | AI 数据出站与模型 profile | 部署前确认数据边界与 provider 策略 |
| G-8 | 审计/Outbox 投递端（消费者） | 表与写事务已就绪；投递、重复消费与断线补缺未实现 |
| G-9 | Lane Backlog / Sprint / 独立看板入口收敛 | 仍有各自入口，共享写入路径未统一 |
| G-10 | 缺陷扩展字段与验证结论的真实来源 | 前端仍显示未知；不得由 `done` 状态反推"已验证" |

---

## 修订历史

| 日期 | 版本 | 内容 |
|------|------|------|
| 2026-11-22 | v1.0 | 建立智能缺陷管理详细设计：实体与版本、API 契约、Project owner 事务、前端模块映射、数据库 DDL、前端代码片段、测试钩子与验收映射、已知缺口 |
| 2026-11-23 | v1.1 | 对照 dev 与 main 的实际缺陷实现和上一轮验收，登记未闭环接口与证据限制。 |
| 2026-11-24 | v1.2 | 对齐 SRS 2026-11-24 现行适用说明；新增 §0.1 部署边界（Vercel 仅 Web/BFF）、§2.1 缺陷 Owner 事务、§3.2.1 jira 风格卡片字段、§3.3 团队页、§4.1 `defect.*` 物理 DDL；登记 G-8～G-10 与实现状态。 |
