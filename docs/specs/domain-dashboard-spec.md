# domain-dashboard 实施 spec v1.0

> **状态**: v1.0 (2026-09-02)
> **触发**:
> - v0.1 (2026-09-01): GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> - v1.0 (2026-09-02): 升 v1.0 扩写, 10 Gadget 完整版 + 12-grid 拖拽 + Wallboard 模式 (per 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira")
> **下游交付**:
> - 需求基线: [docs/requirements/charts-and-reports.md v1.0](../../requirements/charts-and-reports.md)
> - 基本设计: [docs/basic-design/charts-and-reports.md v1.0](../../basic-design/charts-and-reports.md)
> - 详细设计: [docs/design/charts/](../../design/charts/) (22 份, chart-* 图表的 Gadget 实现细节在此)
> - 实现路径: `crates/domain-dashboard/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-dashboard 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-dashboard` 负责 **Star Dashboard Engine**,提供 10 Gadget 类型 + 12-grid 拖拽 + Wallboard 模式 + 共享权限。

**属于本 crate 的** (per v1.0 升版, 与 domain-report 解耦):
- **10 Gadget 类型** (per 需求 §6.1):
  - `chart-*` Gadget (22 图表 type 包装, 通过 ReportDefinition 引用) — 实际上 22 种, 但归类为 1 类
  - `text` (markdown 文本)
  - `activity` (活动流)
  - `assigned-to-me` (个人待办)
  - `filter-results` (过滤结果列表)
  - `wallboard-clock` (数字时钟)
  - `wallboard-sla` (大字号 SLA 数字)
  - `heatmap` (独立 Gadget, 22 图表 C21 也走这个)
  - `two-dimensional-filter` (二维过滤, Jira 经典 Gadget)
  - `rich-text` (富文本, V1.1)
- 12-grid 布局 (Tailwind 标准, INV-DASH-01)
- 拖拽 / 调整大小 (Gadget 不重叠, INV-DASH-02)
- Wallboard 全屏模式 (read-only, INV-DASH-03)
- 共享 / 权限 (per dashboard_share M 表)
- 订阅 + 邮件 (per REQ-NOTIF-002)

**不属于本 crate 的**:
- 图表数据 (从 `domain-report` 走 ReportDefinition 引用)
- WorkItem 数据 (从 `domain-work-item`)
- User / 权限 (从 `domain-identity` / `domain-permission`)

---

## 2. 关键实体 (v1.0 完整版)

```rust
// 完整定义见 [basic-design §3.4](../../basic-design/charts-and-reports.md#34-dashboard--gadget-聚合根)

pub struct Dashboard {                       // 聚合根
    pub dashboard_id: DashboardId,
    pub tenant_id: TenantId,
    pub owner_id: UserId,
    pub title: String,
    pub description: Option<String>,
    pub layout: DashboardLayout,             // 12-grid
    pub gadgets: Vec<DashboardGadget>,
    pub shared_with: Vec<ShareGrant>,
    pub is_wallboard: bool,                  // 全屏模式
    pub auto_refresh_seconds: Option<u32>,   // 默认 30
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

pub struct DashboardLayout {
    pub columns: u32,                        // 12 (Tailwind 标准)
    pub row_height: u32,                     // 默认 80px
    pub gap: u32,                            // 默认 16px
}

pub struct DashboardGadget {
    pub gadget_id: GadgetId,
    pub gadget_type: GadgetType,
    pub position: GadgetPosition,            // {x, y}
    pub size: GadgetSize,                    // {w, h}
    pub config: serde_json::Value,           // 各 gadget 不同
    pub chart_report_id: Option<ReportId>,   // chart-* gadget 引用 report
    pub title: String,                       // 覆盖默认标题
}

pub struct GadgetPosition {
    pub x: u32,                              // 0-11
    pub y: u32,
}
pub struct GadgetSize {
    pub w: u32,                              // 1-12
    pub h: u32,
}

pub enum GadgetType {
    Chart(ChartType),                        // 22 图表 type
    Text,                                    // markdown
    Activity,                                // 活动流
    AssignedToMe,                            // 个人待办
    FilterResults,                           // 过滤列表
    WallboardClock,                          // 数字时钟
    WallboardSla,                            // 大字号 SLA
    Heatmap,                                 // 7×24 活跃度
    TwoDimensionalFilter,                    // 二维过滤
    RichText,                                // V1.1 占位
}

pub struct ShareGrant {
    pub grantee_type: GranteeType,           // user / group / link
    pub grantee_id: Option<Uuid>,            // user/group id
    pub link_token: Option<String>,          // 公开链接 (Wallboard)
    pub permission: SharePermission,         // view / edit
    pub expires_at: Option<DateTime<Utc>>,   // 公开链接 30d
}
```

---

## 3. 关键不变量 (v1.0 完整版)

| ID | 不变量 | 适用 |
|---|---|---|
| INV-DASH-01 | 12-grid 布局严格 12 列 (Tailwind 标准) | 全部 |
| INV-DASH-02 | Gadget 不重叠 (静态分析检测) | 全部 |
| INV-DASH-03 | Wallboard 模式无编辑权限 (read-only) | wallboard |
| INV-DASH-04 | Dashboard 加载 Gadget ≤ 12 (12-grid 上限) | 全部 |
| INV-DASH-05 | 单 Gadget 数据点 ≤ 100K (per INV-REPORT-10) | 全部 |
| INV-DASH-06 | 公开链接 (link) 30 天过期 | wallboard 公开 |
| INV-DASH-07 | 拖拽时实时校验 INV-DASH-02, 重叠时禁止放置 | 全部 |
| INV-DASH-08 | chart-* Gadget 引用必须存在的 ReportDefinition | chart-* |
| INV-DASH-09 | 同一 Dashboard 共享数 ≤ 50 | 全部 |
| INV-DASH-10 | Wallboard 模式 30s auto-refresh | wallboard |
| INV-DASH-11 | Gadget 加载失败显示 fallback, 不阻塞其他 Gadget | 全部 |
| INV-DASH-12 | Dashboard title 长度 1-200, description 长度 0-2000 | 全部 |
| INV-DASH-13 | 跨租户访问 Dashboard 403 (RLS 必携) | 全部 |
| INV-DASH-14 | Wallboard 模式不返回 owner_id / 编辑 API | wallboard |
| INV-DASH-15 | Dashboard 删除走软删除 + Audit, 物理删除 90d 后 | 全部 |

---

## 4. 接口契约 (v1.0 完整版)

### 4.1 Command Port

```rust
#[async_trait]
pub trait DashboardCommandPort: Send + Sync {
    async fn create(&self, cmd: CreateDashboardCmd, actor: &ActorContext) -> Result<Dashboard, DashboardError>;
    async fn update(&self, cmd: UpdateDashboardCmd, actor: &ActorContext) -> Result<Dashboard, DashboardError>;
    async fn delete(&self, dashboard_id: DashboardId, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn add_gadget(&self, dashboard_id: DashboardId, gadget: AddGadgetCmd, actor: &ActorContext) -> Result<DashboardGadget, DashboardError>;
    async fn remove_gadget(&self, dashboard_id: DashboardId, gadget_id: GadgetId, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn move_gadget(&self, dashboard_id: DashboardId, gadget_id: GadgetId, new_pos: GadgetPosition, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn resize_gadget(&self, dashboard_id: DashboardId, gadget_id: GadgetId, new_size: GadgetSize, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn reorder_gadgets(&self, dashboard_id: DashboardId, order: Vec<GadgetId>, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn share(&self, dashboard_id: DashboardId, grant: ShareGrant, actor: &ActorContext) -> Result<ShareGrant, DashboardError>;
    async fn revoke_share(&self, dashboard_id: DashboardId, share_id: ShareId, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn enable_wallboard(&self, dashboard_id: DashboardId, actor: &ActorContext) -> Result<String /* link_token */, DashboardError>;
    async fn disable_wallboard(&self, dashboard_id: DashboardId, actor: &ActorContext) -> Result<(), DashboardError>;
}
```

### 4.2 Query Port

```rust
#[async_trait]
pub trait DashboardQueryPort: Send + Sync {
    async fn get(&self, dashboard_id: DashboardId, actor: &ActorContext) -> Result<Dashboard, DashboardError>;
    async fn list_by_owner(&self, owner_id: UserId, actor: &ActorContext) -> Result<Vec<Dashboard>, DashboardError>;
    async fn list_shared_with_me(&self, actor: &ActorContext) -> Result<Vec<Dashboard>, DashboardError>;
    async fn list_by_project(&self, project_id: ProjectId, actor: &ActorContext) -> Result<Vec<Dashboard>, DashboardError>;
    async fn get_wallboard(&self, link_token: String) -> Result<Dashboard, DashboardError>;  // 公开, 无 actor
    async fn load_gadget_data(&self, dashboard_id: DashboardId, gadget_id: GadgetId, actor: &ActorContext) -> Result<GadgetData, DashboardError>;
}
```

### 4.3 Subscription Port

```rust
#[async_trait]
pub trait DashboardSubscriptionPort: Send + Sync {
    async fn subscribe(&self, dashboard_id: DashboardId, user_id: UserId, cadence: Cadence, actor: &ActorContext) -> Result<DashboardSubscription, DashboardError>;
    async fn unsubscribe(&self, dashboard_id: DashboardId, user_id: UserId, actor: &ActorContext) -> Result<(), DashboardError>;
    async fn notify_update(&self, dashboard_id: DashboardId) -> Result<(), DashboardError>;
}
```

### 4.4 错误模型

```rust
pub enum DashboardError {
    NotFound(DashboardId),
    PermissionDenied(actor_id, action),
    ValidationFailed(Vec<FieldError>),
    GadgetOverlap { gadget_id: GadgetId, with: GadgetId },
    GadgetLimitExceeded { count: u32, limit: u32 },
    ChartReportNotFound(ReportId),
    WallboardLinkExpired,
    GadgetLoadFailed { gadget_id: GadgetId, error: String },
    Internal(String),
}
```

---

## 5. 10 Gadget 类型详细 (v1.0 完整版)

### 5.1 chart-* (22 图表 Gadget)

| 字段 | 值 |
|---|---|
| gadget_type | `chart-{C01-C22}` |
| config | `{ chart_type, filter_id, scope, config_overrides }` |
| chart_report_id | 必须引用 ReportDefinition |
| 默认 size | `6×4` (半宽) |
| 数据源 | domain-report (通过 ReportDefinition) |

### 5.2 text (markdown 文本)

| 字段 | 值 |
|---|---|
| gadget_type | `text` |
| config | `{ markdown: String, allow_html: bool }` |
| 默认 size | `6×2` |
| 数据源 | 无 (静态) |
| 渲染 | markdown-it |

### 5.3 activity (活动流)

| 字段 | 值 |
|---|---|
| gadget_type | `activity` |
| config | `{ project_ids: [ProjectId], max_items: u32, days: u32 }` |
| 默认 size | `4×6` |
| 数据源 | domain-audit (audit event) |

### 5.4 assigned-to-me (个人待办)

| 字段 | 值 |
|---|---|
| gadget_type | `assigned-to-me` |
| config | `{ status_filter: [StatusType], sort: 'priority'|'due' }` |
| 默认 size | `4×6` |
| 数据源 | domain-work-item (per user_id) |

### 5.5 filter-results (过滤结果列表)

| 字段 | 值 |
|---|---|
| gadget_type | `filter-results` |
| config | `{ filter_id: FilterId, max_items: u32, columns: [String] }` |
| 默认 size | `6×6` |
| 数据源 | domain-work-item (via Filter) |

### 5.6 wallboard-clock (数字时钟)

| 字段 | 值 |
|---|---|
| gadget_type | `wallboard-clock` |
| config | `{ timezone: IANA, format: '24h'\|'12h' }` |
| 默认 size | `3×2` |
| 数据源 | 客户端时间 (no server) |

### 5.7 wallboard-sla (大字号 SLA)

| 字段 | 值 |
|---|---|
| gadget_type | `wallboard-sla` |
| config | `{ project_id, priority, sla_id }` |
| 默认 size | `3×3` |
| 数据源 | domain-report (C12 SLA Compliance) |

### 5.8 heatmap (7×24 活跃度)

| 字段 | 值 |
|---|---|
| gadget_type | `heatmap` |
| config | `{ project_id, value: 'created'\|'resolved', days: u32, timezone: IANA }` |
| 默认 size | `8×4` |
| 数据源 | domain-report (C21 Heatmap) |

### 5.9 two-dimensional-filter (二维过滤)

| 字段 | 值 |
|---|---|
| gadget_type | `two-dimensional-filter` |
| config | `{ x_axis: 'assignee'\|'component'\|'priority', y_axis: 'status'\|'type', filter_id }` |
| 默认 size | `8×6` |
| 数据源 | domain-work-item (group by x, y) |

### 5.10 rich-text (富文本, V1.1 占位)

| 字段 | 值 |
|---|---|
| gadget_type | `rich-text` |
| config | `{ content: String, attachments: [Url] }` |
| 默认 size | `6×4` |
| 数据源 | 无 (静态) |
| 状态 | V1.1 实现, v1.0 占位 |

---

## 6. 跨 domain 接触面 (v1.0 完整版)

| 接触 | 协作方式 | Port | 实现 crate |
|---|---|---|---|
| dashboard 嵌入 report | Shared Kernel (ReportDefinition 同构) | `ReportDefinition` | domain-report |
| dashboard 读 work-item (assigned-to-me / filter-results) | Customer-Supplier (OHS) | `WorkItemQueryPort` | domain-work-item |
| dashboard 读 audit (activity) | Customer-Supplier (OHS) | `AuditQueryPort` | domain-audit |
| dashboard 读 user | Customer-Supplier (OHS) | `UserQueryPort` | domain-identity |
| dashboard 校验权限 | Customer-Supplier (OHS) | `PermissionPort` | domain-permission |
| dashboard 写 audit | Conformist | `AuditRecorderPort` | domain-audit |
| dashboard 触发通知 (订阅更新) | Separate Ways (异步) | `NotificationPort` | domain-notification |
| dashboard 共享 (link) | Conformist | (内部) | domain-dashboard (本 crate) |

---

## 7. 风险与缓解 (v1.0 完整版)

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-DASH-01: 大 Dashboard 性能 | 懒加载 Gadget + 12 上限 + 共享缓存 | INV-DASH-04/05 | v1.0 §7 |
| RISK-DASH-02: Wallboard 模式被滥用 | 只读强制 + Audit + 30d 过期 | INV-DASH-03/06 | v1.0 §7 |
| RISK-DASH-03: 拖拽重叠 | 实时校验 + 静态分析 | INV-DASH-02/07 | v1.0 §7 |
| RISK-DASH-04: chart-* Gadget 与 Report 同步 | dashboard 持有 chart_report_id 引用, 跟随 ReportDefinition 更新 | INV-DASH-08 | v1.0 §7 |
| RISK-DASH-05: Wallboard 公开链接泄露 | 30d 过期 + IP 白名单 (可选) + 强制 audit | INV-DASH-06/14 | v1.0 §7 |
| RISK-DASH-06: 12 Gadget 同时查询风暴 | 合并查询 + 共享 5min 缓存 + 并行加载 | INV-DASH-05/11 | v1.0 §7 |
| RISK-DASH-07: 删除 Dashboard 数据丢失 | 软删除 + 90d 后物理删除 + Audit | INV-DASH-15 | v1.0 §7 |
| RISK-DASH-08: 跨域权限漏洞 | RLS + 应用层二次校验 + INV-DASH-13 | INV-DASH-13 | v1.0 §7 |
| RISK-DASH-09: Wallboard 30s auto-refresh 阻塞 | 增量更新 + 仅重渲染变更 Gadget | INV-DASH-10 | v1.0 §7 |

---

## 8. 12-Grid 布局算法 (v1.0 核心实现)

### 8.1 拖拽校验 (INV-DASH-02/07)

```rust
pub struct LayoutValidator;

impl LayoutValidator {
    pub fn validate_gadget_placement(
        existing: &[DashboardGadget],
        new: &DashboardGadget,
    ) -> Result<(), DashboardError> {
        for g in existing {
            if g.gadget_id == new.gadget_id {
                continue;
            }
            if Self::overlaps(&g.position, &g.size, &new.position, &new.size) {
                return Err(DashboardError::GadgetOverlap {
                    gadget_id: new.gadget_id,
                    with: g.gadget_id,
                });
            }
        }
        // 边界检查
        if new.position.x + new.size.w > 12 {
            return Err(DashboardError::ValidationFailed(vec![FieldError {
                field: "size.w".into(),
                message: "exceeds 12 columns".into(),
            }]));
        }
        Ok(())
    }

    fn overlaps(p1: &GadgetPosition, s1: &GadgetSize, p2: &GadgetPosition, s2: &GadgetSize) -> bool {
        p1.x < p2.x + s2.w &&
            p2.x < p1.x + s1.w &&
            p1.y < p2.y + s2.h &&
            p2.y < p1.y + s1.h
    }
}
```

### 8.2 自动布局 (拖拽后建议)

```typescript
// 紧凑化算法: 移除空白行, 让所有 gadget 紧密堆叠
function compactLayout(gadgets: DashboardGadget[]): DashboardGadget[] {
  // 1. 按 y 升序, 同 y 按 x 升序
  const sorted = [...gadgets].sort((a, b) => a.position.y - b.position.y || a.position.x - b.position.x);
  // 2. 用 row 指针追踪下一个可放位置
  let nextY = 0;
  const result: DashboardGadget[] = [];
  for (const g of sorted) {
    if (g.position.y > nextY) {
      nextY = g.position.y;
    }
    result.push({ ...g, position: { x: g.position.x, y: nextY } });
    nextY += g.size.h;
  }
  return result;
}
```

### 8.3 Wallboard 模式 (INV-DASH-03/10)

- 隐藏所有编辑 UI
- 30s auto-refresh (per INV-DASH-10)
- 大字号 / 高对比度 (per a11y)
- 公开链接 30d 过期 (per INV-DASH-06)

---

## 9. DB Schema (W/T/M 分类, per AGENTS.md §4 #13)

> per AGENTS.md §4 守门 #13 (per 2026-09-01 18:30 JST 拍板), DB 表必须 W/T/M 三類横展開

#### Master 表 (SCD Type 2, 物理删除禁止, RLS 13 類必携)

```sql
-- Dashboard 定义 (M 类)
CREATE TABLE dashboard (
    dashboard_id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    owner_id UUID NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    layout JSONB NOT NULL,  -- {columns, row_height, gap}
    is_wallboard BOOLEAN NOT NULL DEFAULT FALSE,
    auto_refresh_seconds INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to TIMESTAMPTZ,
    is_current BOOLEAN NOT NULL DEFAULT TRUE
);
-- M 类: 物理删除禁止 / SCD Type 2 / RLS 必携

-- Dashboard Gadget (M 类, 跟随 dashboard)
CREATE TABLE dashboard_gadget (
    gadget_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboard(dashboard_id),
    gadget_type TEXT NOT NULL,
    position JSONB NOT NULL,  -- {"x": 0, "y": 0}
    size JSONB NOT NULL,      -- {"w": 6, "h": 4}
    config JSONB NOT NULL,
    chart_report_id UUID,
    title TEXT,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- M 类

-- Dashboard 共享 (M 类)
CREATE TABLE dashboard_share (
    share_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboard(dashboard_id),
    grantee_type TEXT NOT NULL,  -- 'user' / 'group' / 'link'
    grantee_id UUID,
    link_token TEXT UNIQUE,
    permission TEXT NOT NULL,    -- 'view' / 'edit'
    expires_at TIMESTAMPTZ,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- M 类

-- Dashboard 订阅 (M 类)
CREATE TABLE dashboard_subscription (
    subscription_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboard(dashboard_id),
    user_id UUID NOT NULL,
    cadence TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- M 类
```

#### Transaction 表 (Append-only, 監査必須, RLS 13 類必携)

```sql
-- Dashboard 查看审计 (T 类)
CREATE TABLE dashboard_view_audit (
    audit_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    user_id UUID NOT NULL,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip_address INET,
    user_agent TEXT,
    is_wallboard BOOLEAN NOT NULL DEFAULT FALSE,
    tenant_id UUID NOT NULL
);
-- T 类: 物理删除禁止 / 監査必須 / RLS 必携

-- Dashboard 编辑审计 (T 类)
CREATE TABLE dashboard_edit_audit (
    audit_id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    user_id UUID NOT NULL,
    edit_type TEXT NOT NULL,  -- 'create' / 'update' / 'delete' / 'add_gadget' / 'remove_gadget' / 'move' / 'resize' / 'share'
    before_state JSONB,
    after_state JSONB,
    edited_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    tenant_id UUID NOT NULL
);
-- T 类
```

#### Work 表 (短 TTL, 物理删除, retention 明示)

> Dashboard 域无典型 Work 类任务 (无生成任务)。仅 Wallboard 公开链接生成可作为 Work 类, 但量小不留表。

**RLS Policy 示例** (per T/M 类必携):
```sql
ALTER TABLE dashboard ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON dashboard
    USING (tenant_id = current_setting('app.current_tenant')::UUID);
```

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板（10 Gadget） | 2026-09-01 15:03 JST GAP-01 |
| v1.0 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 升 v1.0: 10 Gadget 完整版 + 15 不变量 + 4 Port + 9 风险 + 12-grid 校验算法 + 自动布局算法 + Wallboard 模式 + DB DDL W/T/M 三類横展開 + 接触面 v1.0 完整版 | 2026-09-02 10:04 JST Ulysses 拍板 "图表对标 Jira" |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-02
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
