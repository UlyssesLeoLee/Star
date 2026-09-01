# SRE Ops 后台 — 基本设计 (Basic Design)

> **状态**: Draft v0.1 (2026-09-01)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **关联需求**: `01-ops-requirements-spec.md` v0.1
> **架构基线**: `docs/architecture/2026-08-26-upgrade/adr/` (24 ADR)

---

## 1. 设计目标

1. **跨域 SRE 工具一站式**: 不跨页面/工具切换, SRE Lead 一屏完成 12 REQ 任务
2. **强 SRE 隔离**: 跟 `domain-dashboard` (end-user) 严格 namespace 分离 (`/api/admin/ops/*` vs `/api/*`)
3. **可审计**: 所有写操作进 `domain-audit` (per NFR-OPS-002 强约束)
4. **真人确认**: 高风险操作 2-step 确认 + WebAuthn (per NFR-OPS-001)
5. **5 域命名 disclaimer**: 跟现有 AGENTS v0.26 §5 + HANDOFF v0.3 §1.1 一致 (per REQ-OPS-012)

---

## 2. 架构图 (logical)

```
+-------------------------------------------------------------+
|                  SRE Ops 后台 (frontend)                      |
|                  app/(app)/admin/ops/*                       |
+-------------------------------------------------------------+
                          |  HTTP / WebSocket
                          v
+-------------------------------------------------------------+
|              api-gateway (/api/admin/ops/*)                   |
|  - auth: SRE Lead role check (per NFR-OPS-001)              |
|  - audit: 所有写操作进 domain-audit (NFR-OPS-002)          |
+-------------------------------------------------------------+
        |              |              |              |
        v              v              v              v
+----------------+ +-----------+ +-------------+ +-----------+
| domain-ops     | | domain-  | | domain-      | | domain-  |
| (新建, 跨域)   | | tenant   | | permission   | | audit     |
| - 日志/性能/   | | (跨租户)  | | (RBAC)       | | (审计)   |
|   错误/依赖   | +-----------+ +-------------+ +-----------+
| - 限流/缓存    |
+----------------+
        |              |              |
        v              v              v
+----------------+ +-----------+ +-------------+
| OpenTelemetry  | | Prometheus| | k3s         |
| Collector       | |           | |             |
+----------------+ +-----------+ +-------------+
                          |
                          v
+-------------------------------------------------------------+
|        22 domain-* + 4 star-* + application + api          |
+-------------------------------------------------------------+
```

---

## 3. 模块设计

### 3.1 domain-ops 新建 (跨域 SRE BC)
- **职责**: 跨域 SRE 工具聚合 (日志/性能/错误/依赖), 不重写各 domain 业务
- **聚合端口** (per `domain-permission` 模式):
  - `OpsLogQueryPort`: 跨域日志查询 (调 OpenTelemetry collector)
  - `OpsMetricQueryPort`: 跨域性能聚合 (调 Prometheus)
  - `OpsDependencyQueryPort`: 依赖健康 (调 k3s API + blackbox_exporter)
  - `OpsControlPort`: 限流调控 / 缓存清理 (写操作, 强 audit)
- **强隔离**: domain-ops **不依赖**任何 `domain-*` business 逻辑, 仅调 observability / k8s API

### 3.2 现有 domain 整合
- `domain-tenant` 提供 tenant_id 全局边界
- `domain-permission` 提供 RBAC, 新增 `Role::SreLead` (独立 Lead, 不兼任 per 8/21 JST 硬约束)
- `domain-audit` 提供 audit_log 持久化 (write-only)
- `domain-local-runtime` 提供 Local Runtime 状态查询 (per §1.4)

### 3.3 跨 session 续模块 (本期跳过, 等真人)
- `domain-ops-rbac`: 5 域 Lead 工具 ACL (等 P3-1 5 域 Lead 真人到位, per HANDOFF v0.6 §5.3 Blocker 4)

---

## 4. 关键设计决策

### 4.1 SRE 后台 namespace 隔离
- 决策: `/api/admin/ops/*` (per `01-monitoring-spec.md` §2 monitoring 模式, 加 admin namespace 区分 end-user)
- 理由: 跟现有 `domain-dashboard` `/api/*` 严格分离, SRE 不能用 end-user 入口, 反之亦然
- 替代方案: 跟 `domain-dashboard` 共享 `/api/*`, 加 `actor.role` 鉴权 — 拒绝 (安全风险, 跨权限污染)

### 4.2 OpenTelemetry vs Prometheus 优先级
- 决策: OpenTelemetry 为主 (traces + logs), Prometheus 为辅 (metrics)
- 理由: 现有 `01-monitoring-spec.md` v0.1 选 OTel, 跨域统一采集; Prometheus 用于 SRE 自身 K8s/Pod 监控
- 替代方案: 全 Prometheus — 拒绝 (业务跨域 trace 关联缺失)

### 4.3 限流调控二次确认机制
- 决策: 2-step 确认 + WebAuthn 强 MFA (per NFR-OPS-001)
- 理由: 限流是高风险操作 (一行命令可让 22 domain 联锁限流), 需要真人确认
- 替代方案: 1-step 确认 — 拒绝 (守门 #5 域 Lead 真人到位精神)

### 4.4 缓存清理操作 audit
- 决策: 操作前 audit log + 操作后 5min 验证 hit rate (per REQ-OPS-006)
- 理由: 误清理 LLM 缓存会导致 $5K-50K/day 浪费, audit 是事后追责
- 替代方案: 仅记录 — 拒绝 (缺验证无法判断是否成功)

### 4.5 跨域数据隔离
- 决策: 后台操作不污染业务数据, read-only 或独立 audit log
- 理由: 防止 SRE 操作意外改写业务状态
- 替代方案: 后台直接写业务 — 拒绝 (安全风险)

---

## 5. 5 域命名 disclaimer 落地 (per Q1-D 拍板)

- 5 域 (player/economy/match/social/admin) 是历史治理命名, 22 domain-* 是 DDD bounded context
- 后台 UI 顶部 disclaimer banner: "5 域 (player/economy/match/social/admin) 是历史治理命名, 22 domain-* 是 DDD bounded context, 两者非同一分类, 不建立业务子域↔DDD 映射 (per AGENTS §5 仓库拓扑, Q1-D 拍板 2026-08-31 22:45 JST)"
- 报表里 5 域指标 label 必须 disclaimer

---

## 6. 跨域影响分析

### 6.1 跨域依赖
| 依赖域 | 用途 | 风险 |
|---|---|---|
| domain-tenant | tenant_id 全局边界 | 低 (只读) |
| domain-permission | RBAC, SreLead 角色 | 中 (跨域 ACL 一致性) |
| domain-audit | audit log 持久化 | 低 (write-only) |
| domain-local-runtime | Local Runtime 状态 | 低 (只读) |
| 22 domain-* | 跨域业务数据展示 (SRE 视角) | 中 (数据脱敏) |

### 6.2 跟现有 spec 关系
- `01-monitoring-spec.md` (Observability): domain-ops 依赖其数据源
- `domain-tenant-spec.md`: tenant_id 边界参考
- `domain-permission-spec.md`: RBAC 扩展 (新增 SreLead 角色)
- `domain-audit-spec.md`: audit log schema 复用

---

## 7. 已知缺口 (per 守门 #11 缺标比错标安全)

1. **5 域 Lead 真人到位**: REQ-OPS-008 5 域运营报表 (per HANDOFF v0.6 §5.3 Blocker 4)
2. **WebAuthn 集成**: 当前缺 (NFR-OPS-001 强 MFA 依赖), 跨 session 续
3. **限流风暴测试场景**: REQ-OPS-005 跨租户限流"联锁限流"风险未覆盖, 跨 session 续
4. **数据脱敏规范**: REQ-OPS-007 PII 脱敏规则未细化, 等真人 SRE Lead 拍板
5. **跨 session 续细化**: domain-ops entity / port / service 详细设计跨 session 续

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 架构图 + 4 关键设计决策 + 5 域命名 disclaimer + 跨域影响分析 | 2026-09-01 13:04 JST |
