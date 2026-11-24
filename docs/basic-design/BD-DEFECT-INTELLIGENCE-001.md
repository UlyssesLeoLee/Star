# BD-DEFECT-INTELLIGENCE-001

> 智能缺陷管理 — 基本设计 v1.2 (Draft；实现部分完成，验收未通过)
> 需求：`docs/requirements/SRS-DEFECT-INTELLIGENCE-001.md` v1.1（2026-11-24 现行适用说明）
> 详细设计：`docs/detailed-design/DD-DEFECT-INTELLIGENCE-001.md` v1.2（本配套）
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
| 所有权隔离 | 每个 branch 独立一套数据；跨 branch 互传走显式 handoff，不传时各自独立 | SRS §2、用户逐字指令 |
| 授权优先 | 401 使会话能力失效，403 只关闭被拒绝范围；未授权内容不得送入模型，不以缓存/预览/示例冒充生产事实 | SRS §5.2、AGENTS.md §0 |
| 幂等与版本 | 修改使用 owner revision/CAS + 幂等命令；冲突返回最新版本与可恢复草稿，禁止自动重放 | SRS FR-DI-020 |
| 复用优先 | 复用 WorkItem、看板、验证、评论、附件、审计、通知、授权组件；不新造第二套权威缺陷列表 | SRS §1、§1.3 |
| 证据诚实 | 未接入的持久化字段、扩展字段与验证结论必须显式标注"未知/未接入"，不得由界面状态反推业务事实 | SRS FR-DI-006、009 |

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

---

## §4 数据库设计（摘要，详细设计展开）

### 4.1 核心表（DDL 摘要）

> **2026-11-24 实施对齐**：实际实现落在 `defect.*` schema（`crates/domain-defect/src/pg_schema.rs`），
> 表名为 `defect.work_items` / `defect.verifications` / `defect.suggestions` /
> `defect.command_idempotency` / `defect.audit_events` / `defect.outbox`。
> 下表 `defect_work_item` 等名称为逻辑名，物理实现见详细设计 §4。

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
本节登记当前候选提交的实现状态，不改变前文目标契约。

**已具备（代码级）**
- `crates/domain-defect` 领域骨架：`DefectStatus` 受控流转表（`Triage→Confirmed→InProgress→PendingVerify→Closed`，
  `Duplicate`/`NoAction` 终态，禁止任意状态直接跳 Closed）、租户/Project/能力校验、
  修复版本字段、独立验证记录（验证主体/修复版本/方法/结果/证据）、幂等键去重、
  `lifecycle_version` 乐观锁。生命周期测试 14/14 通过。
- PostgreSQL 持久化层：`crates/domain-defect/src/pg_schema.rs`（`defect.*` 六表 DDL）与
  `crates/domain-defect/src/pg_service.rs`（owner 事务：设 tenant/actor GUC → 授权校验 →
  幂等锁 → 幂等查询 → 写事实 + audit + outbox → 保存幂等回执 → COMMIT）。
- 前端：缺陷页 `/defect` 接入统一 `WorkItemDetailDrawer`，卡片使用 jira 风格字段
  （key/优先级/SP 顶行，kind+status，负责人/聊天/架构，labels，关联文档）；
  看板卡片同步重写；任务看板为 `/kanban` 第一个标签页，甘特图与燃尽图位于看板标签页内。

**未接入（保持 fail-closed，不冒充生产能力）**
- PG 适配器尚未接入 `star-api-rest` 路由（无 `defects.rs`），生产流量仍未走持久化。
- 审计与 Outbox 表已建、写逻辑已在 PG 事务内，但投递端（消费者）未实现，事件停留在 `pending`。
- 缺陷扩展字段（现象/实际结果/修复版本/验证状态）在前端仍为未知值；缺陷列表对单个 Worktree
  读取失败仍静默继续，未按 SRS FR-DI-006 报告失败范围。
- 验证独立性当前比较"验证人 vs 报告人"，尚未按策略校验实际修复者/执行者（SRS FR-DI-018）。
- Lane Backlog 与独立看板仍各有入口，尚未收敛为单一写入路径（SRS §1.3 统合边界）。
- Task 详情中的 CLI/Agent/取消/恢复/独立验证、AI 通知事件、BI 统计口径统一、设置深链修复均未完成。

**验收状态**
AC-DI-01～08 全部为 `NOT_RUN`。上一轮 `dev@ddb0b6a6` 的失败结论、`dev@df672de7` 的源码复核
与本节实现状态分列，不得以任何一节的结论替代候选提交的端到端证据。

### 2026-11-23 实现与验收对照

- `dev@ddb0b6a6` 的模块整合验收未通过，问题包括缺陷创建入口、内存服务写入授权和状态/验证边界。四组前端测试 37 项通过；看板 5 项通过时 NEMA WASM 被临时替换为 JS 回退；Rust 缺陷 crate 编译通过但有 0 个测试。真实数据库、授权和浏览器端到端未验收。
- 随后 `dev@df672de7` 已在缺陷页接入统一任务抽屉，并加入租户/Project 检查、状态迁移检查及验证记录结构。创建模式仍未将 `defaultKind` 放入 `open`，抽屉读取 `open.defaultKind`；创建仍经本地 `addWorkItem`。缺陷数据汇总仍吞掉部分 Worktree 读取错误，扩展字段和验证状态缺少真实来源。
- 当前工作区 `main@4f54b2c9` 在 dev 基础上增加内存迁移命令的修复版本字段和测试文件；该提交尚不在上述 dev 基线。本次未重跑其测试，不能据此关闭生产持久化、授权、独立验证、审计和 Outbox 门禁。
- 设计目标仍为 WorkItem owner 的 Bug 身份及授权事实。落地前须补齐创建/流转的身份和范围复验、修复版本及独立验证持久化、事务审计/Outbox 和失败恢复，并以同一候选提交执行 AC-DI-01～03。

| # | 缺口 | 状态 |
|---|------|------|
| G-1 | 外部问题系统导入或双向同步（连接授权、许可、字段映射、冲突策略） | 后续可选，不作为本期闭环前提 |
| G-2 | 真实 AI provider 接入（OpenAI/Anthropic 等） | 本仓不涉，走 `crates/star-llm/` |
| G-3 | 真实持久化（PostgreSQL writer/API） | PG 适配器与 DDL 已落地；**路由未接入**，生产仍无真实持久化 |
| G-4 | 工作流策略版本化 UI 配置 | 基本设计确定后实施，当前为策略数据结构 |
| G-5 | 桌面/窄屏/缩放/键盘/屏幕阅读器验收 | 以实际页面和辅助技术测试，失败提供文本原因 |
| G-6 | AI 数据出站与模型 profile | 部署前确认数据边界与 provider 策略 |
| G-8 | 审计/Outbox 投递端（消费者） | 表与写事务已就绪；投递、重复消费与断线补缺未实现 |
| G-9 | Lane Backlog / Sprint / 独立看板入口收敛 | 仍有各自入口，共享写入路径未统一 |
| G-10 | 缺陷扩展字段与验证结论的真实来源 | 前端仍显示未知；不得由 `done` 状态反推"已验证" |

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
