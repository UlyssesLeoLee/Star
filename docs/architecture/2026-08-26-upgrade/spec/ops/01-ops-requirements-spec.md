# SRE Ops 后台 — 需求规格 (Requirements Spec)

> **状态**: Draft v0.1 (2026-09-01)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **触发**: 2026-09-01 13:04 JST Ulysses "推完吧。另外我需要一个后台界面，专门用于运维管理"
> **范围**: SRE Lead 视角的运维管理后台, 跟现有 `domain-dashboard` (end-user 5 域业务) 区分
> **下游依赖**:
> - 现有 `docs/architecture/2026-08-26-upgrade/spec/observability/01-monitoring-spec.md` (Prometheus/OpenTelemetry 数据源)
> - 现有 `domain-tenant` (跨租户权限基础)
> - 现有 `domain-permission` (RBAC)
> - 现有 `domain-audit` (审计日志)
> - 现有 `domain-local-runtime` (Local Runtime 状态)

---

## 1. 角色与边界

`SRE Ops 后台` 是 Star 平台的 **SRE Lead 专属控制面** (per 8/21 JST 拒绝兼任硬约束, SRE 独立于 admin 域 Lead 决策权), 提供:

**属于本后台**:
- 跨域 SRE 工具: 日志查询 / 跨租户性能 / 错误汇总 / 依赖健康
- 限流调控: API rate limit / agent session 限流 / workspace 配额
- 缓存清理: 跨域 Redis cache / LLM 响应缓存 / RAG 向量缓存
- 租户运营: 跨租户查询 (审计/合规目的, SRE 权限)
- 5 域运营报表: player/economy/match/social/admin 域跨租户聚合
- Kubernetes 运维: pod 状态 / deployment rollout / HPA 触发
- Prometheus 运维: rule 配置 / silences / alertmanager 状态
- OpenTelemetry 运维: 采样率调整 / 索引 pipeline 监控

**不属于本后台** (跨边界):
- 终端 user 业务界面 (5 域 dashboard, 归属 `domain-dashboard`)
- 平台运营 (tenant CRUD, RBAC 编辑, 计费, 归属 platform admin 后台, 跨 session 续)
- 5 域 Lead 工具 (player/economy/match/social/admin 域跨租户数据, 等 5 域 Lead 真人到位)

**跨边界 SLA**: SRE Ops 后台与 platform admin 后台 7 段结构共享 (`AGENTS.md §3`), 5 域 Lead 后台 (待 P3-1 真人到位) 跨 session 续.

---

## 2. 需求 (Requirements)

### REQ-OPS-001: 跨域日志查询 (L1 Must-Have)
- 给 SRE Lead 提供统一日志查询界面, 跨 domain-* 22 crate + supporting 4 crate (star-*) + frontend + workspace + k3s pod
- 字段: timestamp / tenant_id / domain / level (trace/debug/info/warn/error) / message / trace_id / actor_id
- 查询: 关键字 / 时间窗口 / tenant_id / domain / level / trace_id 6 维度
- 性能: 1M events 范围查询 < 5s (per 守门 #1 派生 v5 P95 预算)
- 数据源: 现有 OpenTelemetry collector (per `01-monitoring-spec.md`)

### REQ-OPS-002: 跨租户性能聚合 (L1 Must-Have)
- 跨 tenant_id 聚合 P50/P95/P99 延迟 / QPS / 错误率 4 维度
- 时间窗口: 5min / 1h / 24h / 7d 4 档
- Domain 维度: 22 domain-* + 4 star-* 切分
- 数据源: Prometheus + OpenTelemetry traces

### REQ-OPS-003: 错误汇总 (L1 Must-Have)
- 跨 domain 错误率排名 / top N error type / 最近 24h 趋势
- 按 domain / error code 维度聚合
- 关联: 跳到对应日志查询 (REQ-OPS-001)

### REQ-OPS-004: 依赖健康 (L2 Should-Have)
- 外部依赖健康: PostgreSQL / Redis / Kafka / Object Storage / KMS / LLM Provider
- 内部微服务健康: 22 domain-* + application / api / star-cli
- 数据源: Prometheus blackbox_exporter + k8s probe

### REQ-OPS-005: 限流调控 (L1 Must-Have, 高风险)
- API rate limit 调整: per tenant / per endpoint / per global
- Agent session 限流: per workspace / per user
- 调整需要 SRE Lead 二次确认 + audit log (per `domain-audit` 强约束)
- 不可逆操作: "lock account" / "global rate limit 设为 0" 等, 需 2-step 确认

### REQ-OPS-006: 缓存清理 (L2 Should-Have)
- 跨域 Redis cache 清理: domain 选择 + cache key pattern
- LLM 响应缓存清理: per model + per workspace
- RAG 向量缓存清理: per index
- 操作前 audit log + 操作后 5min 内验证 hit rate

### REQ-OPS-007: 租户运营 (L2 Should-Have, 跨边界)
- 跨租户查询 (审计/合规目的): tenant_id / user_id / workspace_id / project_id 4 维度
- 数据脱敏: 业务字段 hash 后展示 (PII 数据隐藏)
- 操作: read-only (写操作在 platform admin 后台)

### REQ-OPS-008: 5 域运营报表 (L2 Should-Have, 等真人)
- 5 域 (player/economy/match/social/admin) 跨租户聚合
- 实际业务报表: DAU / MAU / 留存 / 收入 / 跨域调用拓扑
- 依赖 5 域 Lead 真人到位 (per HANDOFF v0.6 §5.3 Blocker 4), 跨 session 续

### REQ-OPS-009: Kubernetes 运维 (L1 Must-Have)
- Pod 列表 / deployment rollout / HPA 状态 / service mesh 配置
- 操作: scale / restart / cordon (高风险, 2-step 确认)
- 数据源: k8s API (k3s) + Prometheus

### REQ-OPS-010: Prometheus 运维 (L2 Should-Have)
- Rule 配置 / silences / alertmanager 状态
- Recording rules / federation 状态
- 操作: 静默规则 (需过期时间) / 调整阈值 (需 2-step 确认)

### REQ-OPS-011: OpenTelemetry 运维 (L2 Should-Have)
- 采样率调整: per service / per span
- Index pipeline 监控 (Loki / Tempo / Pyroscope 状态)
- 字段过滤 (避免 PII 泄漏)

### REQ-OPS-012: 5 域命名 disclaimer (per Q1-D 拍板, AGENTS v0.26)
- 5 域 (player/economy/match/social/admin) 是历史治理命名, **不建立业务子域↔DDD 映射**
- 报表里 5 域指标 label 必须 disclaimer "5 域是历史治理命名, 22 domain-* 是 DDD bounded context"
- ST 测试报告 / 后台 UI 都需带 disclaimer

---

## 3. 非功能需求 (NFR)

### NFR-OPS-001: 权限隔离
- 仅 SRE Lead 角色可访问 (per 8/21 JST 拒绝兼任硬约束, SRE Lead 真人到位后激活)
- 二次验证: 强 MFA + WebAuthn (per 守门 #5 域 Lead 真人到位)
- 不可被 platform admin / 5 域 Lead 越权访问

### NFR-OPS-002: 审计日志 (per 守门 #12)
- 所有写操作 (限流调控 / 缓存清理 / Kubernetes 操作) 必记 audit log
- audit log 字段: timestamp / operator_id (SRE Lead) / action / target / before_value / after_value / reason
- audit log 不可篡改 (append-only, per `domain-audit` INV-AU-01)

### NFR-OPS-003: 性能 (per 守门 #1 派生 v5 P95 预算)
- 列表页 P95 < 1s
- 详情查询 P95 < 5s
- 写操作 P95 < 2s (含 2-step 确认)

### NFR-OPS-004: 跨域数据隔离
- 后台操作不污染业务数据 (read-only 或独立 audit log)
- 跨租户查询严格 RBAC (SRE Lead 全域权限, 但写操作只允许在 SRE 范围内)

### NFR-OPS-005: 可用性
- SLO: 99.9% (SRE 自运维工具不可成为单点)
- 跨 session / 跨 k8s cluster 高可用
- 紧急模式: 当 Prometheus 自身 down, 后台降级到本地缓存展示

---

## 4. 已知缺口 (per 缺标比错标安全, AGENTS §4 守门 #11)

1. **5 域 Lead 真人到位** (P3-1 阻塞): 后台跨 5 域报表功能 (REQ-OPS-008) 需要 5 域 Lead 真人到位拍板, 跨 session 续
2. **3-域安全护栏**: 高风险操作 (REQ-OPS-005/006/009/010) 需要 SRE Lead 真人确认 + 二次验证机制 (per NFR-OPS-001 强 MFA), 当前缺 WebAuthn 集成
3. **依赖版本**: OpenTelemetry Collector / Prometheus / k3s 版本锁尚未确定 (per 现有 observability spec §1)
4. **跨 session 续细化**: REQ-OPS-008 5 域运营报表需要等 5 域 Lead 真人到位拍板才能细化, 跨 session 续 (per HANDOFF v0.6 §5.3 Blocker 4)
5. **限流调控风暴测试**: REQ-OPS-005 跨租户限流调控的"限流风暴"场景未覆盖 (一个 SRE 操作触发 22 domain 联锁限流)

---

## 5. 接口稳定承诺 (给 Implementation / Operation)

- 接口命名: `/api/admin/ops/*` (per `domain-dashboard` 模式, 加 `/admin/` namespace 区分)
- API 稳定承诺: 跨 phase 不能改 path, 只能改 query param
- Audit log schema 跨域一致 (per `domain-audit` INV-AU-02)
- 错误码 6 字段 (per ADR-0029): code / message / hint / http_status / retry_after / trace_id

---

## 6. 文档元信息

- **状态**: Draft v0.1 (2026-09-01)
- **触发**: 2026-09-01 13:04 JST Ulysses 推完 + 启动运维管理后台 4 份设计
- **关联文档**:
  - 基本设计 v0.1: `02-ops-basic-design-spec.md` (本文档 §1 后)
  - 详细设计 v0.1: `03-ops-detailed-design-spec.md`
  - 测试设计 v0.1: `04-ops-test-design-spec.md`
- **下游契约**:
  - 给 Implementation: 估 0.5-1M token (12 REQ + 5 NFR + 4 个 SRE 子系统: k8s / Prometheus / OTel / Cache)
  - 给 Operation: SRE Lead 真人到位 + WebAuthn 集成 + 限流二次确认 UI
- **修订历史**: 见 §7

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 12 REQ + 5 NFR + SRE Ops 后台范围 (SRE Lead 拍板) | 2026-09-01 13:04 JST Ulysses "推完吧 + 后台界面 4 份设计" |
