# DD-DEFECT-INTELLIGENCE-001

> 智能缺陷管理 — 详细设计 v1.0 (Draft)
> 上游：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` v0.1、`docs/basic-design/BD-DEFECT-INTELLIGENCE-001.md` v1.0
> 本设计不启动生产部署，实体与版本、API 契约、Project owner 事务、前端模块映射、测试钩子、已知缺口均已显式列出。

---

## §0 实体与版本

| Logical aggregate | Owner | Identity / version | State |
|-------------------|-------|--------------------|-------|
| `defect_work_item` | WorkItem owner (`domain-work-item` boundary) | `defect_id, revision` | triage / waiting_info / confirmed / in_progress / awaiting_verification / closed / reopened / duplicate / not_processed; Master history |
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

Association/binding revisions are monotonically increasing owner versions. An API client may submit expected version and idempotency key, but may not select actor/tenant identity, modify owner version, or assert current authorization.

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

Drawer save:
- New mode: `Create` → `handleCreate` → `addWorkItem` with `branch_id=null`, `branch_name=selectedBranch`, `localOnly=true`.
- View mode: `Save` → `handleSave` → per-field `updateWorkItemField` (title/status/priority/kind/description/assignee_id/worktree_id/labels).

### 3.3 IPA Skill Workflow Board integration

- `createTaskCard` calls `addWorkItem` with `ipa_skill_ids: [skill.id]`, `labels: ["ipa-skill", skill.phase]`, `onOpenTaskCard(id)`.
- Save button is effective (real `addWorkItem` call, no stub).

### 3.4 Tab-aware shortcut navigation

- `ShortcutButton.targetTab` field defined in navStore.
- Consumer logic (`router.replace('/?tab=...')`) **not yet implemented** — known gap (G-6 in BD).

---

## §4 Database DDL (canonical, matches BD §4)

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

| # | 缺口 | 状态 |
|---|------|------|
| G-1 | 外部问题系统导入或双向同步（连接授权、许可、字段映射、冲突策略） | 后续可选，不作为本期闭环前提 |
| G-2 | 真实 AI provider 接入 | 本仓不涉，走 `crates/star-llm/` |
| G-3 | 真实持久化（PostgreSQL writer/API） | 生产必须真实持久化，不能用 in-memory adapter 充当业务真相 |
| G-4 | 工作流策略版本化 UI 配置 | 基本设计确定后实施，当前为策略数据结构 |
| G-5 | 桌面/窄屏/缩放/键盘/屏幕阅读器验收 | 以实际页面和辅助技术测试，失败提供文本原因 |
| G-6 | Tab 感知跳转消费逻辑（`router.replace('/?tab=...')`） | ShortcutButton.targetTab 字段已定义，消费逻辑待实现 |
| G-7 | AI 数据出站与模型 profile | 部署前确认数据边界与 provider 策略 |

---

## 修订历史

| 日期 | 版本 | 内容 |
|------|------|------|
| 2026-11-22 | v1.0 | 建立智能缺陷管理详细设计：实体与版本、API 契约、Project owner 事务、前端模块映射、数据库 DDL、前端代码片段、测试钩子与验收映射、已知缺口 |
