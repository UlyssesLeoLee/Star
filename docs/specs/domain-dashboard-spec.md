# domain-dashboard 实施 spec

> **状态**: Draft v0.1 (2026-09-01)
> **触发**: per 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01)
> **下游交付**: Implementation team — Rust crate 路径 `crates/{display}/`

> **dual-use 警告** (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板):
> domain-dashboard 是 Star 仓 supporting domain crate (per [basic-design §6 22 logical domain + 7 supporting crate](../../../basic-design.md))
> 不在 22 bounded context 主域列表,与 5 域 (player/economy/match/social/admin) 历史治理命名**不建立映射**。

---

## 1. 职责与边界

`domain-dashboard` 负责 **Star Dashboard Engine (per crates/domain-dashboard/src/lib.rs v0.1 实施实装)**。

**属于本 crate 的**:
- 12-grid 布局 (Tailwind 标准)
- 10 Gadget 类型 (WorkItem 列表 / Sprint 燃尽 / 报表快照 / 自定义查询 / ...)
- Wallboard 全屏模式
- 共享 / 权限
- 订阅 + 邮件

**不属于本 crate 的**:
- WorkItem 数据 (从 `domain-work-item` Projection 读)
- Report 数据 (从 `domain-report` Projection 读)
- User / 权限 (从 `domain-identity` / `domain-permission` 读)

## 2. 关键实体

- `Dashboard` (聚合根): dashboard_id / tenant_id / owner_id / title / layout (12-grid) / gadgets[] / shared_with[]
- `DashboardGadget`: gadget_id / type (10 种) / position (x, y, w, h) / config{}
- `DashboardSubscription`: dashboard_id / subscriber_id / cadence (realtime / hourly / daily)

## 3. 关键不变量

| ID | 不变量 |
|---|---|
| INV-DASH-01 | 12-grid 布局严格 12 列 (Tailwind 标准) |
| INV-DASH-02 | Gadget 不重叠 (静态分析检测) |
| INV-DASH-03 | Wallboard 模式无编辑权限 (read-only) |

## 4. 接口契约

- `DashboardCommandPort`: create / update / add-gadget / remove-gadget / reorder / share / delete
- `DashboardQueryPort`: get / list-by-owner / list-shared-with-me / get-wallboard
- `DashboardSubscriptionPort`: subscribe / unsubscribe / notify-update

## 5. 跨 domain 接触面 (v0.16 协作细化新增)

per [basic-design v0.16 §3.2.9 22 domain contact face 表](../../../basic-design.md) + [ADR-0039 §D26-D32 Worktree Orchestration 跨域协作](../../architecture/2026-08-26-upgrade/adr/0039-worktree-orchestration-cross-domain.md),`domain-dashboard` 作为 supporting crate 跨 22 bounded context + 6 supporting crate 的接触面。

**协作模式** (per [basic-design v0.16 §3.1 解耦机制](../../../basic-design.md) 8 种):

| 接触类型 | 目标 domain | 协作方式 | 引用 |
|---|---|---|---|
| `dashboard` 触发工单创建 | work-item | Customer-Supplier (Port) | per `dashboard` 提交触发 |
| `dashboard` 读取用户身份 | identity | Shared Kernel (UserId) | per User 引用 |
| `dashboard` 审计所有操作 | audit | Separate Ways (Append-only) | per AuditRecorder Port |
| `dashboard` 触发降噪通知 | notification | Separate Ways (异步) | per REQ-NOTIF-002 |

> 详细接触面待 [basic-design v0.16 §3.2.9](../../../basic-design.md) 后续 sweep 补充 (per GAP-01 后续 P3 阶段)。

## 6. 风险与缓解

| Risk | 影响 | 缓解 | 引用 |
|---|---|---|---|
| RISK-DASH-01: 大 Dashboard 性能 | 懒加载 + 虚拟滚动 | — | domain-dashboard §6 |
| RISK-DASH-02: Wallboard 模式被滥用 | 只读强制 + Audit | — | domain-dashboard §6 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版：7 章简化模板 (职责 / 实体 / 不变量 / 接口 / 接触面 / 风险 / 修订历史) | 2026-09-01 15:03 JST GAP-01 7 supporting crate 加 spec (per PHASE-INTER-COLLAB-REFINE-REPORT §3 GAP-01) |

---

> **审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01
> **per AGENTS.md §1.0 用户授权升级 v0.5 + 8/27 19:39/20:56/21:59 JST 三次强化**: Mavis 接手默认代签 Ulysses 无需再问
