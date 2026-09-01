# SRE Ops 后台 — 详细设计 (Detailed Design)

> **状态**: Draft v0.1 (2026-09-01)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **关联**: `01-ops-requirements-spec.md` v0.1 + `02-ops-basic-design-spec.md` v0.1

---

## 1. API 设计 (per 守门 ADR-0029 6 字段错误模型)

### 1.1 跨域日志查询 (REQ-OPS-001)
- **Endpoint**: `POST /api/admin/ops/logs/query`
- **Request**:
  ```json
  {
    "query": "error",
    "time_range": { "from": "2026-09-01T00:00:00Z", "to": "2026-09-01T23:59:59Z" },
    "tenant_id_filter": ["t-acme", "t-beta"],
    "domain_filter": ["domain-tenant", "domain-permission"],
    "level_filter": ["warn", "error"],
    "trace_id_filter": null,
    "limit": 100,
    "offset": 0
  }
  ```
- **Response 200**:
  ```json
  {
    "events": [
      {
        "timestamp": "2026-09-01T12:34:56.789Z",
        "tenant_id": "t-acme",
        "domain": "domain-permission",
        "level": "error",
        "message": "Permission denied for actor",
        "trace_id": "abc123",
        "actor_id": "usr-001"
      }
    ],
    "total": 1234,
    "took_ms": 234
  }
  ```
- **错误码** (per ADR-0029):
  - `400 INVALID_QUERY` (time_range 反向 / limit > 1000)
  - `403 PERMISSION_DENIED` (SRE Lead 角色缺失)
  - `429 RATE_LIMITED` (跨 SRE 调 OpenTelemetry 频率限制)
  - `500 BACKEND_UNREACHABLE` (OpenTelemetry collector 不可达)

### 1.2 跨租户性能聚合 (REQ-OPS-002)
- **Endpoint**: `POST /api/admin/ops/metrics/aggregate`
- **Request**:
  ```json
  {
    "metric": "http_request_duration_seconds",
    "aggregations": ["p50", "p95", "p99"],
    "time_window": "1h",
    "tenant_filter": ["*"],
    "domain_filter": ["domain-*"]
  }
  ```
- **Response 200**:
  ```json
  {
    "metric": "http_request_duration_seconds",
    "aggregations": {
      "p50": 0.045,
      "p95": 0.234,
      "p99": 0.567
    },
    "samples": 123456,
    "time_range": { "from": "...", "to": "..." },
    "by_tenant": [
      { "tenant_id": "t-acme", "p95": 0.123 },
      { "tenant_id": "t-beta", "p95": 0.345 }
    ]
  }
  ```

### 1.3 限流调控 (REQ-OPS-005, 高风险, 2-step 确认)
- **Step 1**: `POST /api/admin/ops/ratelimit/preview`
  - Request: `{ "tenant_id": "t-acme", "endpoint": "/api/*", "new_limit_rps": 100, "effective_at": "2026-09-02T00:00:00Z" }`
  - Response: `{ "preview_id": "prv-abc123", "impact_estimate": { "affected_tenants": 1, "rps_reduction": "50%" } }`
- **Step 2**: `POST /api/admin/ops/ratelimit/apply`
  - Request: `{ "preview_id": "prv-abc123", "webauthn_assertion": "<signed-challenge>", "reason": "incident-2026-09-01-001" }`
  - Response 200: `{ "ratelimit_id": "rl-xyz789", "audit_id": "au-12345" }`
  - Response 403: `{ "code": "WEBAUTHN_FAILED" }`

### 1.4 缓存清理 (REQ-OPS-006)
- **Endpoint**: `POST /api/admin/ops/cache/purge`
- **Request**:
  ```json
  {
    "cache_type": "redis",  // redis | llm_response | rag_vector
    "domain_filter": ["domain-llm"],
    "key_pattern": "*workspace-123*",
    "webauthn_assertion": "...",
    "reason": "stale-cache-2026-09-01"
  }
  ```
- **Response 200**: `{ "purged_keys": 12345, "audit_id": "au-12346", "verify_at": "2026-09-01T13:00:00Z" }`

### 1.5 Kubernetes 运维 (REQ-OPS-009)
- **Endpoint**: `GET /api/admin/ops/k8s/pods?namespace=domain-tenant`
- **Response**:
  ```json
  {
    "pods": [
      {
        "name": "domain-tenant-abc123",
        "namespace": "domain-tenant",
        "phase": "Running",
        "ready": true,
        "restarts": 0,
        "age": "2d3h",
        "node": "k3s-node-01",
        "resources": { "cpu": "100m", "memory": "256Mi" }
      }
    ]
  }
  ```
- **写操作**: `POST /api/admin/ops/k8s/pods/{name}/restart` (2-step 确认 + audit)

---

## 2. 数据模型 (per `domain-audit` INV-AU-02 强约束)

### 2.1 AuditLog (复用 domain-audit schema, 新增 ops 字段)
```rust
pub struct OpsAuditLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub operator_id: Uuid,  // SRE Lead user_id
    pub action: String,     // ratelimit.apply | cache.purge | k8s.restart | ...
    pub target: serde_json::Value,  // 跨域 target (灵活 schema)
    pub before_value: Option<serde_json::Value>,
    pub after_value: Option<serde_json::Value>,
    pub reason: String,
    pub webauthn_assertion_id: Option<Uuid>,  // 强 MFA 验证
    pub trace_id: Uuid,
}
```

### 2.2 RateLimit (新 schema, 限流调控配置)
```rust
pub struct RateLimitConfig {
    pub id: Uuid,
    pub tenant_id: TenantId,  // 或 "*" 全局
    pub endpoint_pattern: String,  // e.g. "/api/*"
    pub limit_rps: u32,
    pub effective_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,  // 临时限流
    pub created_by: Uuid,  // SRE Lead
    pub created_at: DateTime<Utc>,
    pub audit_id: Uuid,  // 关联 audit log
}
```

---

## 3. 状态机 (WebAuthn 二次确认流程)

```
[Idle] --(operator click 写操作)--> [Step1: Preview]
[Step1: Preview] --(operator click confirm)--> [Step2: WebAuthn Challenge]
[Step2: WebAuthn Challenge] --(WebAuthn success)--> [Step3: Apply]
[Step3: Apply] --(success)--> [Done + audit log]
[Step3: Apply] --(fail)--> [Failed + alert + rollback]
[Step1: Preview] --(operator cancel)--> [Idle]
```

跨域一致: 所有高风险操作 (限流/缓存/K8s) 走同 4 步状态机

---

## 4. 错误码 (per ADR-0029 6 字段)

| 错误码 | HTTP | 含义 |
|---|---|---|
| `INVALID_QUERY` | 400 | 请求 schema 错 |
| `PERMISSION_DENIED` | 403 | SRE Lead 角色缺失 |
| `WEBAUTHN_FAILED` | 403 | 二次验证失败 |
| `WEBAUTHN_REQUIRED` | 401 | 高风险操作需要 WebAuthn 但缺失 |
| `RATE_LIMITED` | 429 | SRE 调后端频率限制 |
| `BACKEND_UNREACHABLE` | 502 | OpenTelemetry/Prometheus/k3s 不可达 |
| `AUDIT_FAILED` | 500 | audit log 写入失败 (per NFR-OPS-002 强约束) |
| `CROSS_TENANT_DENIED` | 403 | 跨租户操作权限 (SRE 仅 read, 写操作在 platform admin 后台) |

---

## 5. 性能预算 (per 守门 #1 派生 v5 P95)

| 端点 | P95 预算 | 数据源 |
|---|---|---|
| `GET /api/admin/ops/logs/query` | < 5s (1M events) | OpenTelemetry |
| `POST /api/admin/ops/metrics/aggregate` | < 3s (1h window) | Prometheus |
| `GET /api/admin/ops/k8s/pods` | < 1s | k3s API |
| `POST /api/admin/ops/ratelimit/*` | < 2s (含 2-step 确认) | DB + audit |
| `POST /api/admin/ops/cache/purge` | < 5s (含 5min verify) | Redis + audit |
| `GET /api/admin/ops/dependencies/health` | < 2s | Prometheus + k8s probe |

---

## 6. 部署架构 (per 守门 #12 commit-time 同步 + §4 守门 #1 反转 R-05)

- **k3s deployment**: `deploy/ops/dashboard.yaml` (envoy proxy per 9/1 13:03 JST 偏好)
- **Reverse proxy**: envoy (per 9/1 13:03 JST Ulysses 偏好: "所有 nginx 都应该替换为 envoy")
- **DB**: 复用 `star_postgres` + `domain-audit` (write-only, append-only)
- **Cache**: 复用 `star_redis` (跨域)
- **Observability**: OpenTelemetry collector (per `01-monitoring-spec.md`)

---

## 7. 安全设计 (per NFR-OPS-001/002)

- **认证**: SRE Lead 角色 + WebAuthn 强 MFA
- **授权**: `domain-permission` `Role::SreLead` 检查 (独立 Lead, 不兼任 per 8/21 JST 硬约束)
- **审计**: 所有写操作进 `domain-audit` (per NFR-OPS-002 强约束, append-only)
- **限流**: API 自身限流 (SRE 调用频率), 避免 SRE 工具被滥用
- **跨租户**: 写操作不允许跨租户 (REQ-OPS-007 read-only 跨租户, 写在 platform admin 后台)

---

## 8. 跨 session 续细化项 (per 守门 #12 docs 同步)

1. **domain-ops entity 详细 schema**: 本期跳过, 跨 session 续
2. **WebAuthn 集成规范**: 跨 session 续 (依赖 NFR-OPS-001 强 MFA, 当前缺)
3. **限流风暴测试场景**: 跨 session 续 (per 已知缺口 #3)
4. **数据脱敏规则**: 跨 session 续 (per 已知缺口 #4, 等 SRE Lead 真人)
5. **5 域报表**: 跨 session 续 (per HANDOFF v0.6 §5.3 Blocker 4, 等 5 域 Lead 真人)

---

## 9. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 5 API + 2 数据模型 + 4 步状态机 + 8 错误码 + 6 性能预算 + 5 安全约束 | 2026-09-01 13:04 JST |
