# BD-DEFECT-INTELLIGENCE-001

> 智能缺陷管理 — 基本设计 v1.4 (Draft；实现部分完成，验收未通过)
> 需求：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` v1.1（2026-11-24 现行适用说明）
> 详细设计：`docs/detailed-design/DD-DEFECT-INTELLIGENCE-001.md` v1.3（本配套）
> 验收：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` §7 AC-DI-01～08（当前全部 NOT_RUN）
> 统合边界与状态：`docs/requirements/README.md`（缺陷、任务与看板共享授权任务事实）

---

## §0 设计目标和约束

### 0.1 设计目标

在 Star Lane 工作空间内建立**可信的缺陷闭环**：报告者在工作上下文中记录问题，处理者在已授权 Lane/Worktree 中修复，验证者独立确认，Project 成员在授权范围内看到缺陷风险、趋势与待办。智能辅助减少信息整理与重复检查，所有建议可追溯到证据与版本。

### 0.2 核心约束（不可违背）

| 约束 | 内容 | 出处 |
|------|------|------|
| 导航不变 | 主导航保持 `Project → Lane → Worktree → Branch`；缺陷在 Lane 的 Backlog/Sprint 视图中打开，复用可关闭任务详情窗口 | SRS §1.3 |
| 统合事实源 | 智能缺陷、Backlog、Sprint 与看板共享同一套授权任务事实及写入路径；缺陷不新造第二套权威列表，不新造第二条写入通道 | SRS §1.3、2026-11-24 现行适用说明 |
| 所有权隔离 | 已绑定 Branch 的记录和 CLI 上下文按授权范围隔离；跨 Branch 引用/互传须显式授权与 handoff。无 Worktree 的 Lane 可保存未绑定记录，不推断 Branch。 | SRS §2、FR-DI-002/010 |
| 授权优先 | 401 使会话能力失效，403 只关闭被拒绝范围；未授权内容不得送入模型，不以缓存/预览/示例冒充生产事实 | SRS §5.2、AGENTS.md §0 |
| 幂等与版本 | 修改使用 owner revision/CAS + 幂等命令；冲突返回最新版本与可恢复草稿，禁止自动重放 | SRS FR-DI-020 |
| 复用优先 | 复用 WorkItem、看板、验证、评论、附件、审计、通知、授权组件；不新造第二套权威缺陷列表 | SRS §1、§1.3 |
| 证据诚实 | 未接入的持久化字段、扩展字段与验证结论必须显式标注"未知/未接入"，不得由界面状态反推业务事实 | SRS FR-DI-006、009 |
| 物理事实优先 | 设计的落地状态以**物理事实**（表、列、RLS、迁移文件、调用点）为准，不以 Rust 常量、类型声明或 prop 声明推定；代码存在但未被调用链消费的实现按"未接入"登记 | 守门 #11 缺标比错标、#12 实证引用 |

---

## §1 逻辑上下文

```
Project Owner (Star-native domain-project boundary)
  ├── Project identity + durable membership authorization (M)
  └── Lane context (selectedLaneId / selectedWorktreeId / selectedBranch)

Lane / Worktree / Branch 上下文（navStore）
  └── 选中 Worktree 后，Lane 级 Backlog/Sprint 中打开“缺陷”视图

缺陷情报模块（本模块）
  ├── 缺陷列表（分页、筛选、排序、保存查询）
  ├── 缺陷创建与详情（可关闭子窗口，Bug WorkItem）
  ├── 缺陷生命周期（待分诊→已确认→处理中→待验证→已关闭，±补充/重复/不处理/重开）
  ├── 证据与验证（验证记录、证据引用、附件、评论、历史）
  ├── 跨 branch 显式互传（handoff，target_branch_id + expected_version）
  ├── 智能辅助（摘要、重复候选、原因与修复建议、Agent 工作流启动）
  ├── 通知与订阅（Owner Outbox 推送，重复投递幂等）
  └── 质量分析与治理（现象/原因统计、SLA、归档）

外部 owner（不跨 owner 写表）
  ├── WorkItem owner（缺陷权威事实、历史、审计）
  ├── 工作流/策略 owner（状态、字段、枚举、SLA 策略版本）
  ├── 通知 owner（Outbox/Inbox，事件推送）
  ├── 附件/证据 owner（最小 scope 桥接，可审计）
  ├── Agent 运行 owner（Agent 工作流启动、sandbox、Hook、预算）
  └── AI 模型 provider owner（固定 provider/profile/endpoint digest，重试沿用）
```

### 1.1 模块边界

- **内部**：缺陷列表/详情/创建、生命周期状态机、证据附件、评论、历史、智能建议草稿、验证记录、分诊操作、质量分析投影。
- **外部（只读/按契约调用）**：WorkItem 权威事实、工作流策略、通知 Outbox、附件证据、Agent 执行、AI provider。
- **不做**：外部问题系统导入或双向同步（列为后续可选连接能力，另行定义连接授权、许可、字段映射、冲突策略）。

---

## §2 事实所有权（W/T/M 表）

每个表必须标注主分类 W（Work，短 TTL）/T（Transaction，业务事实/审计/Append-only）/M（Master，参考/设定/慢变 SCD2）。禁止混合分类。

| 表 | 主分类 | 内容摘要 | Owner | 生命周期与边界 |
|----|--------|----------|-------|--------------|
| `defect_work_item` | T | 缺陷核心事实：ID、标题、现象、预期/实际、重现步骤、状态、严重度、优先级、发现阶段、原因类别、发现版本、修复版本、负责人、报告者、Lane/Worktree/Branch 引用、修订号 | WorkItem owner | SCD2，业务事实禁止物理删除，历史保留 |
| `defect_field_value` | T | 可自定义字段值（类型、范围、必填、只读、检索规则） | WorkItem owner | 字段定义变更不破坏已有值，无法转换显示迁移错误 |
| `defect_relation` | T | 父子、关联、重复、阻塞/被阻塞；禁止自引用、父子环、无权关系 | WorkItem owner | 重复标记保留来源与指向，不自动删除/改写原报告 |
| `defect_status_event` | T | 状态、字段、负责人、关系、验证变更事件（actor、服务端时间、版本、原因、审计） | WorkItem owner | Append-only，支持有权限历史读取 |
| `defect_verification_record` | T | 验证者、固定修复版本、方法、结果、证据 ID、摘要 | WorkItem owner | 验证事实所属 owner 保存，关联失效显示不可访问 |
| `defect_evidence_attachment` | T | 附件与证据：文件名、大小、类型、上传时间、访问授权、摘要 | 附件/证据 owner | 上传前受限大小/类型，内容访问经授权，敏感数据按部署策略处理 |
| `defect_comment` | T | 评论：作者、内容、时间、目标缺陷 ID | WorkItem owner | 按时间和服务端顺序展示，失败保留用户草稿 |
| `defect_ai_suggestion` | T | 智能建议：输入 revision、模型/profile、来源、内容摘要、状态、审阅者、采纳/修改/拒绝 | WorkItem owner | 建议固定输入版本与证据，权限撤销后标过期 |
| `defect_workflow_strategy` | M | 工作流、字段、枚举、通知、SLA 策略：Project/Lane、策略版本、有效期、授权规则 | 工作流/策略 owner | 版本化生效及历史追溯，变更不改写既有操作所用策略 |
| `defect_notification_subscription` | M | 订阅：报告者/负责人/验证者/关注者、事件类型、通道 | 通知 owner | 通知基于 owner 已提交事实及 Outbox，重复投递幂等 |
| `defect_draft` | W | 待上传草稿：范围、revision、到期时间、状态 | 当前会话 | 有界大小和 TTL，来源重建或过期清理 |
| `defect_quality_projection` | W | 质量分析投影：新增、关闭、重开、积压、处理时长、验证失败 | 分析 owner | 筛选条件与授权范围贯穿明细钻取，TTL 清理 |
| `defect_sla_policy` | M | SLA/目标时限：自然时间/工作日历/时区/暂停/起止事件/策略版本 | 工作流/策略 owner | 版本化生效，缺省不设未经确认的统一时限 |

---

## §3 模块划分与接口

### 3.1 模块列表

| 模块 | 职责 | 关联 FR |
|------|------|---------|
| DI-List | 缺陷列表：分页、筛选（状态/严重度/优先级/负责人/报告者/Lane/Worktree/Branch/版本/标签/发现阶段/时间/文本）、排序、保存查询 | FR-DI-004/005/006 |
| DI-Create | 缺陷创建表单：标题/现象/预期/实际/重现/影响/阶段/严重度/优先级/环境/附件 | FR-DI-002/003 |
| DI-Drawer | 缺陷详情可关闭子窗口：现象、预期/实际、复现、环境、附件、关系、修复与验证、历史 | FR-DI-001/010/011/012/013 |
| DI-Lifecycle | 生命周期状态机：分诊/确认/处理/提交验证/关闭/重开/补充/重复/不处理 | FR-DI-015/016/019 |
| DI-FixLink | 修复关联：关联已有任务卡或创建绑定缺陷的修复任务，固定修复标识与证据 | FR-DI-017/018 |
| DI-Verify | 验证：独立验证通过/失败、测试方法与证据，独立性策略强制 | FR-DI-018/019 |
| DI-Evidence | 证据与附件：上传、访问授权、敏感数据处理、失败保留草稿 | FR-DI-012 |
| DI-Suggestion | 智能建议：摘要、重复候选、原因与修复建议、Agent 工作流启动 | FR-DI-021～028 |
| DI-Notify | 通知与订阅：分派/补充/验证/重开/关闭事件推送，幂等重投 | FR-DI-029/030 |
| DI-Quality | 质量分析报表：按状态/严重度/发现阶段/版本/Lane/时间/负责人筛选，现象/原因统计 | FR-DI-031/032/033 |
| DI-Archive | 归档与恢复：保留、审计、权限策略 | FR-DI-034 |

### 3.2 接口契约（摘要）

| 接口 | 输入 | Owner 事务/读取 | 输出 |
|------|------|----------------|------|
| 列表查询 | Project/Lane/Worktree 选择、筛选条件、分页游标 | 当前授权；服务端分页，权限过滤 | 分页缺陷列表 + 总计数 + 数据来源标识 |
| 创建缺陷 | 标题、现象、预期/实际、重现步骤等 | 验证必填；Project/Lane/报告者/授权上下文；可绑定 Worktree | 稳定缺陷 ID + revision + 幂等回执 |
| 状态流转 | 缺陷 ID、目标状态、expected_version、理由 | 工作流版本 + 角色 + 字段必填性校验；CAS | 新状态版本 + 审计事件 |
| 验证提交 | 缺陷 ID、修复标识、验证结果、证据 ID | 验证者身份校验；固定修复版本 | 验证记录 + 状态变更或阻断原因 |
| 智能建议 | 输入缺陷 revision、证据摘要 | 权限校验；最小必要输入范围 | 建议内容 + 来源 + revision + 不确定性标记 |
| 质量报表 | 筛选条件、时间窗口、统计口径 | 授权范围贯穿明细钻取 | 统计图表 + 明细钻取入口 + 更新时间 |

### 3.3 幂等与版本（与 SRS FR-DI-020 一致）

- 修改命令携带 `expected_version`（task_lifecycle 当前版本）与 `idempotency_key`。
- 冲突时返回最新版本与可恢复草稿；结果未知时按同一操作身份查询回执，或经用户确认重试。
- 授权失败后不自动重放写入。

### 3.4 缺陷创建、查询与验证的统一契约

1. **入口和身份**：Lane Backlog 的缺陷列表、独立视图和任务详情抽屉复用同一创建命令。创建请求显式携带 `kind=bug`、Project/Lane、当前 actor、授权上下文及可选 Worktree/本地 Branch；前端表单默认值和类型锁必须传至命令实际读取的位置。后端生成或确认稳定 ID，以权威回执和 revision 驱动刷新；不能以本地 store 写入或抽屉关闭当作创建成功。
2. **Owner 与映射**：WorkItem owner 是本专题的目标权威事实源；新 `defect.*` 表及候选路由是待审查实现，必须明确它是同一 owner 的持久化适配器，还是独立 owner 经版本化协议关联。后者不得复制并并行维护第二份权威状态；在身份、写入、事务、审计和事件关系验证完成前保持待接入。该裁决不得依靠表名或代码存在推定。
3. **查询和部分失败**：列表按当前授权服务端分页、过滤并返回同范围计数；结果包含已覆盖范围与失败/失权范围。任何部分读取失败须可见，不把局部结果标为完整；401 使当前会话能力失效，403 只拒绝对应范围，不以 seed 或旧缓存冒充当前数据。缺失扩展字段显示未知或未接入。
4. **验证和关闭**：修复提交固定修复者、版本/标识及证据；验证 owner 依据生效策略核验实际修复者与验证者独立性，持久记录验证人、方法、结果、证据和关联 revision。关闭检查当前权限、必填字段、阻塞关系与有效验证。`done` 只表示映射后的任务状态，不能单独产生或显示验证通过。
5. **错误和恢复**：写命令使用 owner revision/CAS 和幂等键；冲突保留草稿，未知结果以同一操作身份查询回执，授权失败不得自动重放。Outbox 推送、通知与 BI 使用当前授权范围；未经验证的数据不能进入模型上下文或统计。

### 3.5 单一写入路径与物理接线判定（per 2026-11-24 物理事实核对）

SRS §1.3 要求缺陷、Backlog、Sprint 与看板**共享同一套授权任务事实及写入路径**。本节把该要求落为可判定的设计约束，供实现与验收逐项核对。

**3.5.1 写入路径的唯一性**

| 写入口 | 应写入的 owner 事实 | 当前物理接线 | 判定 |
|---|---|---|---|
| Lane Backlog 缺陷视图新建 | `Bug` WorkItem（缺陷扩展字段随 owner 事务写入） | 抽屉 `handleCreate` → 本地 `store.addWorkItem`（`localOnly=true`） | **未接入**：只写本地 store，无 owner 回执、无 revision 驱动刷新 |
| 缺陷页 `/defect` 新建 | 同上 | 同上（共用 `WorkItemDetailDrawer`） | **未接入**（同上） |
| 任务详情抽屉新建 | 同上 | 同上（同一 `handleCreate` 路径） | **未接入**（同上） |
| `POST /api/v1/defects` | `defect.work_items`（PG owner 事务） | `routes/defects.rs` 已注册，但 `DEFECT_SERVICE` 恒为 `DefectService::new(None)` → 恒走 `InMemoryDefectService` | **未接入**：PG 适配器从未被实例化 |

**3.5.2 物理接线判定规则（写入验收前逐条核对）**

一个能力记为"已接入"必须同时满足下列四项，缺一即按"未接入"登记：

1. **调用可达**：存在从真实请求入口到该实现的生产调用链（路由注册 ≠ 调用链；常量声明 ≠ 被消费）。
2. **物理持久化**：目标表已存在于迁移文件中，且带 RLS 策略与必要索引；仅有 Rust 中的 DDL 常量字符串不构成物理表。
3. **授权真实**：actor 身份来自已验证会话，不得由桩函数随机生成；授权校验使用该真实身份的 tenant/Project。
4. **数据同源**：返回给前端的对象来自该 owner 的持久化读取，不来自另一条并行写入通道。

**3.5.3 判定依据（本轮核对，2026-11-24，`dev@b1db1c99`）**

| 项 | 物理证据 | 判定 |
|---|---|---|
| `defect.*` 六表物理存在 | `db/migrations/` 无 `defect` 迁移文件；DDL 仅存在于 `crates/domain-defect/src/pg_schema.rs` 常量 | 未接入 |
| 六表 RLS | `pg_schema.rs` 搜 `ROW LEVEL SECURITY` / `CREATE POLICY` 零命中（对比 `multica.*` 见 `db/migrations/2026-09-29-worktree-group-phase-2c.sql:139` 起逐表 `ENABLE` + `FORCE` + `CREATE POLICY`） | 未接入 |
| PG 服务可达 | `routes/defects.rs:149` `DEFECT_SERVICE.get_or_init(\|\| DefectService::new(None))` → `pg` 恒 `None` | 未接入 |
| actor 身份真实 | `routes/defects.rs:actor_from_headers` 每次 `Uuid::new_v4()` 生成随机 user/tenant，与请求携带的 tenant 永不相等 | 未接入（任何跨 tenant 校验在真实会话下不可达） |
| 抽屉 kind 锁定 | `WorkItemDetailDrawer` 声明 `defaultKind`/`lockKind` prop 但函数签名未解构（`WorkItemDetailDrawer.tsx:112-114`）；draft 读 `open.defaultKind`（:206），而缺陷页 `handleCreate` 只传 `{ kind:"new", defaultStatus:"todo" }`（`defect/page.tsx:240`） | 未接入：实际创建 kind 为 `"task"` 而非 `"bug"` |
| Outbox 物理可投递 | `defect.outbox` 表列无 `event_id`/`sequence`/`correlation_id`/`digest`/`stream_id`，与 `defect_outbox::contract::Envelope` 所需字段不匹配 | 未接入 |
| 幂等回执完整性 | `pg_service.rs:687` 幂等命中返回 `build_defect_stub`（空标题/空现象/nil reporter），非缓存的原始回执 | 部分：去重成立，回执内容不保真（FR-DI-020 要求回执可恢复） |

以上判定不改变 §3.4 的目标契约，仅登记当前实现状态；各能力的关闭条件见 §6 缺口表。

| 需求 / 差异 | 基本设计机制 | 验收 |
|---|---|---|
| FR-DI-001～009：Bug 身份、授权、可选绑定 | 创建回执、统一 owner 和按范围分页 | AC-DI-01：跨 Project/Lane、同名 Branch、无 Worktree、刷新与撤权 |
| FR-DI-007/010～014：部分失败与历史 | 逐范围/逐项结果、授权关系、版本化历史 | AC-DI-02：成功、403、冲突、无权关系及草稿恢复 |
| FR-DI-015～020：修复和独立验证 | 版本化状态、修复者/验证者策略、证据与 CAS | AC-DI-03：通过、失败、关闭阻断、重开与重复提交 |
| FR-DI-021～032：智能、通知、BI | 输入 revision、最小权限、Outbox 与同范围投影 | AC-DI-04～06：权限隔离、执行门、重投和统计一致 |

以上均为目标设计；AC-DI-01～08 仍按现有验收记录为 NOT_RUN，不因单项源码存在而改变。

---

## §4 数据库设计（摘要，详细设计展开）

### 4.1 核心表（DDL 摘要）

> **2026-11-24 实施对齐**：实际实现落在 `defect.*` schema（`crates/domain-defect/src/pg_schema.rs`），
> 表名为 `defect.work_items` / `defect.verifications` / `defect.suggestions` /
> `defect.command_idempotency` / `defect.audit_events` / `defect.outbox`。
> 下表 `defect_work_item` 等名称为逻辑名，物理实现见详细设计 §4。现有物理表与 WorkItem owner 的关系仍待 §3.4 对账，不能仅凭命名视为同一权威事实或验收完成。

```sql
-- 缺陷核心事实 (T，SCD2)
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

-- 状态/事件日志 (T，Append-only)
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
    status TEXT DEFAULT 'pending', -- pending/accepted/modified/rejected/expired
    reviewed_by UUID,
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- 工作流策略 (M，SCD2)
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
```

### 4.2 索引策略

- `defect_work_item(tenant_id, project_id, lane_id, status)` —— 列表查询主索引
- `defect_work_item(branch_id, branch_name)` —— branch 级隔离过滤
- `defect_status_event(defect_id, event_time DESC)` —— 历史读取
- `defect_verification_record(defect_id)` —— 验证历史
- `defect_ai_suggestion(defect_id, status)` —— 建议查询

### 4.3 RLS 与审计

- 所有表开启 RLS，按 `tenant_id` / `project_id` / `lane_id` 隔离。
- `app.actor_id` / `app.correlation_id` GUC 用于 Outbox 触发器与审计。
- 敏感字段（附件内容、日志、AI 输入）按部署策略加密存储。

### 4.4 物理 DDL 落地要求（per §3.5.2 判定规则）

上述 RLS/索引要求必须由 `db/migrations/` 下的**迁移文件**落地，并由仓库迁移 runner 应用（每行一条语句，不含 `BEGIN`/`COMMIT`，per `device_outbox` 迁移的既有约定与 `crates/star-api-rest/src/device_outbox/mod.rs:74` 的结构测试）。

缺陷域迁移文件的最小内容（**当前不存在，属缺口 G-11**）：

| 要求 | 内容 | 参照 |
|---|---|---|
| schema 与六表 | `defect.work_items` / `verifications` / `suggestions` / `command_idempotency` / `audit_events` / `outbox` | `crates/domain-defect/src/pg_schema.rs` 的 `CREATE_ALL_TABLES` |
| RLS | 每表 `ENABLE ROW LEVEL SECURITY` + `FORCE ROW LEVEL SECURITY` + 按 `tenant_id = NULLIF(current_setting('app.tenant_id', true), '')::UUID` 的 SELECT/INSERT/UPDATE 策略 | `db/migrations/2026-09-29-worktree-group-phase-2c.sql:139-186` |
| 幂等唯一键 | `(tenant_id, actor_id, idempotency_key)` 主键 + `expires_at` 索引 | `pg_schema.rs` `CREATE_TABLE_DEFECT_COMMAND_IDEMPOTENCY` |
| Outbox 可投递字段 | 增补 `event_id UUID UNIQUE`、`stream_id`、`source_sequence BIGINT`、`correlation_id`、`event_digest BYTEA(32)`、`schema_version`、`occurred_at`，并建 `(stream_id, source_sequence)` 唯一约束 | `device_outbox` 迁移的 outbox 主键与 digest 约束（`device_outbox/mod.rs:112-127` 的结构测试断言） |
| 审计不可变 | `defect.audit_events` 挂 `reject_*_mutation` 触发器，禁止 UPDATE/DELETE | `db/migrations/2026-09-29-worktree-group-phase-2c.sql:137` 的 `multica.reject_task_audit_mutation()` |

**Outbox 物理字段增补的必要性**：`defect_outbox::contract::Envelope` 与 `validate_envelope`（`crates/star-api-rest/src/defect_outbox/contract.rs:167`）要求 `schema_version`、`event_id`、`correlation_id`、`sequence`、`digest` 全部非空且 digest 与 payload 匹配。当前 `defect.outbox` 表仅有 `aggregate_type/aggregate_id/event_type/payload/status/created_at/sent_at`，**无法承载 envelope**，故 Outbox 消费者即使代码存在也无法消费生产事件。这是 G-8 之外必须先关闭的前置条件（登记为 G-12）。

---

## §5 非功能设计

| ID | 类别 | 设计决策 |
|----|------|----------|
| NFR-DI-01 | 可用性 | owner 不可用/授权目录不可用/结果未知时故障关闭写入；读取展示真实错误；重连通过持久游标补缺并去重 |
| NFR-DI-02 | 性能 | 授权列表首屏 p95 ≤ 1 秒（10,000 条缺陷、20 并发用户、每页 50 条）；客户端端到端 p95 ≤ 2 秒 |
| NFR-DI-03 | 容量 | 每 Project/Lane/Agent/用户配置并发、token、执行、上传、缓存、事件队列预算；超限提供可恢复错误 |
| NFR-DI-04 | 安全 | owner 授权 + RLS/服务端能力校验；跨租户/跨 Project/撤权/引用失权正负测试；审计不泄露凭据 |
| NFR-DI-05 | 一致性 | owner revision/CAS + 幂等回执 + Outbox；语义重叠草稿由 owner 在冲突事务中固定双方修订；冲突提交后 p99 ≤ 1 秒收到同一 `conflict_id` |
| NFR-DI-06 | 可访问性 | 核心流程支持键盘、可见焦点、屏幕阅读器、非颜色独占状态、窄窗口、200% 缩放 |
| NFR-DI-07 | 可观测性 | 提交失败、延迟、队列深度、投递/恢复、AI provider 不可用、验证失败等指标 |
| NFR-DI-08 | 迁移 | 历史 Bug WorkItem 保留 ID、权限、关系、历史；旧状态/字段映射有版本、预检、可恢复方案 |
| NFR-DI-09 | 交付 | Web/Vercel 与 Rust 桌面分别记录支持环境及验证结果 |
| NFR-DI-10 | AI 质量 | 固定、获授权、可复现的人工标注样本测试重复候选/摘要准确性/证据引用/分类建议/越权泄露 |

---

## §6 已知缺口（显式列，per 缺标比错标）

### 2026-11-24 实现状态与设计对照

按 2026-11-24 现行适用说明（缺陷、Backlog、看板共享授权任务事实及写入路径），
本节保留 2026-11-24 当时的候选实现记录，不改变前文目标契约；该时点以后出现的候选路由或其他工作区改动须重新检查，不能沿用下文的“无路由”作为当前事实。

**已具备（代码级）**
- `crates/domain-defect` 领域骨架：`DefectStatus` 受控流转表（`Triage→Confirmed→InProgress→PendingVerify→Closed`，
  `Duplicate`/`NoAction` 终态，禁止任意状态直接跳 Closed）、租户/Project/能力校验、
  修复版本字段、独立验证记录（验证主体/修复版本/方法/结果/证据）、幂等键去重、
  `lifecycle_version` 乐观锁。生命周期测试 14/14 通过。
- PostgreSQL 持久化层：`crates/domain-defect/src/pg_schema.rs`（`defect.*` 六表 DDL）与
  `crates/domain-defect/src/pg_service.rs`（owner 事务：设 tenant/actor GUC → 授权校验 →
  幂等锁 → 幂等查询 → 写事实 + audit + outbox → 保存幂等回执 → COMMIT）。
- `star-api-rest` 缺陷路由：`crates/star-api-rest/src/routes/defects.rs` 6 个端点
  （create/list/get/transition/verify/list-verifications）已注册于
  `build_legacy_mirror_routes`（`src/lib.rs:215-220`），`DefectService` 实现
  `DefectCommandPort`/`DefectQueryPort`；`RestError` 已加 `From<domain_defect::DefectError>`
  映射（`src/error.rs:863-892`，含 CROSS_TENANT_DENIED / CROSS_PROJECT_DENIED /
  INVALID_STATE_TRANSITION / CONFLICT / VERIFICATION_MISMATCH 等 9 变体）。
- 缺陷 Outbox 契约层：`crates/star-api-rest/src/defect_outbox/` 四文件
  （contract / consumer / relay / mod），含 envelope 校验、`decide_apply` 决策矩阵
  （Accept/Replay/DigestConflict/SequenceGap/StaleCursor/InvalidEnvelope）、
  `InMemoryDefectInbox` 与有界退避 `BoundedRelay`。
- 前端：缺陷页 `/defect` 接入统一 `WorkItemDetailDrawer`，卡片使用 jira 风格字段
  （key/优先级/SP 顶行，kind+status，负责人/聊天/架构，labels，关联文档）；
  看板卡片同步重写；任务看板为 `/kanban` 第一个标签页，甘特图与燃尽图位于看板标签页内。

**未接入（保持 fail-closed，不冒充生产能力）**

按 §3.5.2 的四项判定（调用可达 / 物理持久化 / 授权真实 / 数据同源）逐条核对：

| 能力 | 当前物理状态 | 判定 |
|---|---|---|
| PG 持久化写路径 | `defect.*` 六表仅存在于 `pg_schema.rs` 的 DDL 常量；`db/migrations/` 无 defect 迁移文件 → Neon 中不存在这些表 | 未接入 |
| 六表 RLS | `pg_schema.rs` 无任何 `ENABLE/FORCE ROW LEVEL SECURITY` 与 `CREATE POLICY` | 未接入 |
| PG 服务接线 | `routes/defects.rs:149` `DefectService::new(None)` → `pg` 恒 `None`，生产流量全部走 `InMemoryDefectService` | 未接入 |
| 会话身份 | `actor_from_headers` 随机生成 user/tenant UUID，与请求 tenant 永不相等 → 跨 tenant 拒绝路径在真实会话下不可达 | 未接入 |
| Outbox 投递 | `defect.outbox` 缺 envelope 所需列（`event_id`/`sequence`/`correlation_id`/`digest`/`stream_id`），消费者无法消费生产事件 | 未接入 |
| 审计投递与幂等回执 | 审计/Outbox 写逻辑在 PG 事务内；但幂等命中返回 `build_defect_stub`（空标题/空现象/nil reporter），回执内容不保真 | 部分：去重成立，回执内容待修 |
| 缺陷创建 kind | 抽屉 `defaultKind`/`lockKind` prop 未被解构消费；缺陷页 `handleCreate` 只传 `{ kind:"new", defaultStatus:"todo" }` → 实际 kind 为 `"task"` | 未接入（FR-DI-001 未满足） |
| 缺陷扩展字段来源 | 前端 `severity` 恒为 `"unknown"`、`actual_result`/`expected_result`/`fix_version` 恒 `undefined`；验证状态由 `status === "done"` 反推“已验证” | 未接入（违反证据诚实约束） |
| 多 Worktree 部分失败 | `defect/page.tsx:138` 单个 worktree 读取失败被 `catch {}` 吞掉，无失败范围与原因展示 | 未接入（FR-DI-006） |
| 验证独立性 | 比较“验证人 vs 报告人”，未按策略校验实际修复者/执行者 | 未接入（FR-DI-018） |
| 单一写入路径 | Lane Backlog / 缺陷页 / 抽屉三处新建均走本地 `store.addWorkItem`（`localOnly=true`），与 `/defects` REST 路由并行 | 未接入（§1.3 统合边界） |
| 其余模块整合 | Task 详情 CLI/Agent/取消/恢复/独立验证、AI 通知事件、BI 统计口径统一、设置深链修复均未完成 | 未接入 |

**验收状态**
AC-DI-01～08 全部为 `NOT_RUN`。上一轮 `dev@ddb0b6a6` 的失败结论、`dev@df672de7` 的源码复核
与本节实现状态分列，不得以任何一节的结论替代候选提交的端到端证据。

### 2026-11-23 实现与验收对照

- `dev@ddb0b6a6` 的模块整合验收未通过，问题包括缺陷创建入口、内存服务写入授权和状态/验证边界。四组前端测试 37 项通过；看板 5 项通过时 NEMA WASM 被临时替换为 JS 回退；Rust 缺陷 crate 编译通过但有 0 个测试。真实数据库、授权和浏览器端到端未验收。
- 随后 `dev@df672de7` 已在缺陷页接入统一任务抽屉，并加入租户/Project 检查、状态迁移检查及验证记录结构。创建模式仍未将 `defaultKind` 放入 `open`，抽屉读取 `open.defaultKind`；创建仍经本地 `addWorkItem`。缺陷数据汇总仍吞掉部分 Worktree 读取错误，扩展字段和验证状态缺少真实来源。
- 历史 `main@4f54b2c9` 在当时 dev 基线上增加内存迁移命令的修复版本字段和测试文件；该记录是历史比较，不代表当前工作区或生产部署。未重跑测试不能据此关闭生产门禁。
- 设计目标仍为 WorkItem owner 的 Bug 身份及授权事实。落地前须补齐创建/流转的身份和范围复验、修复版本及独立验证持久化、事务审计/Outbox 和失败恢复，并以同一候选提交执行 AC-DI-01～03。

| # | 缺口 | 状态 |
|---|------|------|
| G-1 | 外部问题系统导入或双向同步（连接授权、许可、字段映射、冲突策略） | 后续可选，不作为本期闭环前提 |
| G-2 | 真实 AI provider 接入（OpenAI/Anthropic 等） | 本仓不涉，走 `crates/star-llm/` |
| G-3 | 真实持久化（PostgreSQL writer/API） | 2026-11-24 曾记录 PG 适配器与 DDL 及路由缺口；之后候选路由、owner 集成与生产持久化须按实际提交重新核验，未验收。 |
| G-4 | 工作流策略版本化 UI 配置 | 基本设计确定后实施，当前为策略数据结构 |
| G-5 | 桌面/窄屏/缩放/键盘/屏幕阅读器验收 | 以实际页面和辅助技术测试，失败提供文本原因 |
| G-6 | Tab 感知跳转消费逻辑（`router.replace('/?tab=...')`） | ShortcutButton.targetTab 字段已定义，消费逻辑待实现 |
| G-7 | AI 数据出站与模型 profile | 部署前确认数据边界与 provider 策略 |
| G-8 | 审计/Outbox 投递端（消费者） | 契约层（contract/consumer/relay）已就绪；物理表与生产投递未实现（依赖 G-12） |
| G-9 | Lane Backlog / Sprint / 独立看板入口收敛 | 仍有各自入口，共享写入路径未统一（依赖 G-13） |
| G-10 | 缺陷扩展字段与验证结论的真实来源 | 前端仍显示未知；不得由 `done` 状态反推“已验证”（依赖 G-14） |
| G-11 | `defect.*` 六表的物理迁移文件与 RLS 策略 | `db/migrations/` 无 defect 迁移；`pg_schema.rs` 无 RLS。关闭条件：新增迁移文件，含六表 DDL + 逐表 `ENABLE`/`FORCE ROW LEVEL SECURITY` + `tenant_id` 策略 + 必要索引，并经仓库迁移 runner 应用 |
| G-12 | `defect.outbox` 增补 envelope 物理列 | 缺 `event_id`/`stream_id`/`source_sequence`/`correlation_id`/`event_digest`/`schema_version`/`occurred_at`，`defect_outbox::contract::Envelope` 无法承载。关闭条件：DDL 增补 + `(stream_id, source_sequence)` 唯一约束 + 消费者端 `decide_apply` 消费回路测试通过 |
| G-13 | 缺陷新建统一走 owner 写入路径 | Lane Backlog / 缺陷页 / 抽屉三处均走本地 `store.addWorkItem`。关闭条件：三处统一调用 owner 创建命令，以 owner 回执 + revision 驱动刷新；抽屉 `defaultKind`/`lockKind` 须被真正解构消费 |
| G-14 | 缺陷扩展字段与验证状态接真实来源 | `severity`/`actual_result`/`expected_result`/`fix_version` 恒为占位值；验证状态由 `status==="done"` 反推。关闭条件：读取 owner 持久化字段，缺失显式显示“未接入/未知”，删除 `done` → “已验证”的反推 |
| G-15 | REST 路由使用真实会话身份 | `actor_from_headers` 随机生成 user/tenant。关闭条件：从已验证会话（AuthLayer 注入的 extension 或 JWT claim）取得 `ActorContext`，租户/Project/能力校验基于该真实身份；401/403 可区分 |
| G-16 | `DefectService` 注入真实 `PgPool` | `DEFECT_SERVICE` 恒 `DefectService::new(None)`。关闭条件：由应用启动流程注入 `PgPool`（Neon `DATABASE_URL`），并保留显式降级开关与降级原因标注 |
| G-17 | 多 Worktree 部分失败显式化 | `defect/page.tsx` 单 worktree 读取失败被吞。关闭条件：逐 worktree 结果（成功/失败范围/原因）进入 UI，局部结果不得标为完整结果 |
| G-18 | 缺陷错误码 → HTTP 状态码映射 | `RestError::into_response` 的 code→status 表未登记 5 个缺陷错误码（`CROSS_TENANT_DENIED`/`CROSS_PROJECT_DENIED`/`INVALID_STATE_TRANSITION`/`IDEMPOTENCY_KEY_USED`/`VERIFICATION_MISMATCH`）→ 落入默认分支，401/403/409/422 无法区分。关闭条件：补映射 + 契约测试 |
| G-19 | `fix_version` 与状态流转同语句原子写 | `pg_service.rs:324-338` 用独立第二条 UPDATE 写 `fix_version`，与状态 UPDATE 之间存在中间态窗口。关闭条件：合并为单条 `UPDATE ... SET status, lifecycle_version, fix_version` |
| G-20 | 验证独立性按实际修复者/执行者校验 | 当前比较“验证人 vs 报告人”，未按生效策略校验实际修复者/执行者（per SRS FR-DI-018）。关闭条件：引入修复者/执行者身份事实，按策略校验，例外须显式记录授权理由 |

---

## §7 阶段交付物

| 阶段 | 交付目标 | 出口 |
|------|----------|------|
| P0 — 缺陷闭环 | Bug WorkItem、授权列表/详情、分诊、修复关联、验证、历史、并发与幂等 | 明确 WorkItem/RunTaskCard owner 与状态映射，打通真实持久化和授权；AC-01～03 通过 |
| P1 — 智能与协作 | 可追溯建议、重复候选、受控 Agent、推送与 BI | P0 的权威版本、证据和权限契约可用；AC-04～06 通过 |
| P2 — 治理增强 | SLA、批量治理和外部连接 | 工作日历、迁移与连接契约；AC-07/08 及对应接口专项通过 |

> WBS 阶段映射（实施排期见 `docs/architecture/DEFECT-WBS-2026-11-23.md`）：
> WBS Phase A（PG 持久化）+ Phase B（审计/Outbox）+ Phase C（`defects.rs` 路由）对应 P0；
> Phase D（缺陷页面完善）+ Phase E（其余模块整合）对应 P0 尾段与 P1；
> Phase F（测试与交付）覆盖 AC-DI-01～08 的证据登记与发布门禁。

---

## 修订历史

| 日期 | 版本 | 内容 |
|------|------|------|
| 2026-11-22 | v1.0 | 建立智能缺陷管理基本设计：逻辑上下文、W/T/M 事实所有权、模块与接口、数据库设计、非功能设计、已知缺口、阶段交付物 |
| 2026-11-23 | v1.1 | 补充历史验收、dev 与 main 后续实现差异及未关闭的生产门禁。 |
| 2026-11-24 | v1.2 | 对齐 SRS 2026-11-24 现行适用说明（统合事实源、证据诚实约束）；登记 `defect.*` schema 实现映射、PG 适配器 owner 事务设计、jira 风格卡片与看板标签页布局；新增缺口 G-8～G-10 与 WBS 阶段映射。 |
| 2026-11-24 | v1.3 | 补充创建身份、部分失败、独立验证和 FR/AC 追踪；明确新物理表与 WorkItem owner 关系待裁决，限定历史实现记录时点。 |
| 2026-11-24 | v1.4 | 新增 §0.2 “物理事实优先”约束与 §3.5 单一写入路径及四项物理接线判定规则；§4.4 增补物理 DDL 落地要求（RLS、Outbox envelope 列）；§6 按四项判定重写实现状态表并新增缺口 G-11～G-20。 |
