# Spec-01: 可观测性规范 (Metrics + Logs + Traces)

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0037 §8 / 2026-08-27 21:59 JST 用户授权 / Phase I production rollout

## §1 目的
定义 Star production 环境的可观测性规范：metrics (Prometheus) + logs (Loki/ELK) + traces (Jaeger/Tempo)。Phase I 必须 P50/P95/P99 latency + error rate 实时可见。

## §2 Metrics (Prometheus)

### 2.1 业务指标 (8 类)
| 指标名 | 类型 | Labels | 用途 |
|--------|------|--------|------|
| `star_http_requests_total` | counter | service/method/path/status | HTTP 请求数 |
| `star_http_request_duration_seconds` | histogram | service/method/path/status | 延迟直方图 |
| `star_cache_hits_total` | counter | service/cache_type | 缓存命中 |
| `star_cache_misses_total` | counter | service/cache_type | 缓存未命中 |
| `star_saga_started_total` | counter | service/saga_name | Saga 启动 |
| `star_saga_completed_total` | counter | service/saga_name/result | Saga 完成 (Completed/Compensated/Failed) |
| `star_domain_handler_read_total` | counter | domain/result | 22 domain 读 (per spec/integration/01) |
| `star_provider_request_total` | counter | provider/method/result | 4 Git Provider 请求 (per spec/vcs/05) |

### 2.2 系统指标
- `node_cpu_seconds_total` / `node_memory_MemAvailable_bytes`
- `container_cpu_usage_seconds_total` / `container_memory_working_set_bytes`
- `kube_pod_info` / `kube_deployment_status_replicas`

### 2.3 告警规则 (5 类)
| 告警 | 阈值 | 持续 | 通知 |
|------|------|------|------|
| error_rate | > 1% | 5min | SRE Lead |
| p99_latency | > 500ms (per spec/sla/01 §3) | 5min | SRE Lead |
| cache_hit_rate | < 80% (per ADR-0036 §2 D15) | 15min | Performance Lead |
| saga_failed_rate | > 0.1% | 5min | SRE + Economy Lead (per 8/21 JST 5 域独立) |
| pod_restart | > 3/min | - | SRE Lead |

## §3 Logs (Loki/ELK)
- **结构化 JSON**: `{"timestamp": "...", "level": "INFO|WARN|ERROR|DEBUG", "service": "...", "trace_id": "...", "message": "..."}`
- 保留 30 天热存储 + 1 年冷存储 (S3/GCS)
- **敏感字段 mask** (per 8/27 11:06 JST secret 安全): token / password / secret / authorization / cookie
- log level: ERROR / WARN / INFO / DEBUG (生产默认 INFO)

## §4 Traces (Jaeger/Tempo)
- **OpenTelemetry SDK 集成** (8 服务统一)
- 每个 HTTP 请求 + Saga step 都有 `trace_id`
- 22 domain handler read 自动 instrumentation (per spec/integration/01 §3 验证项 1)
- Saga 跨域步骤全链路追踪 (per spec/saga/01 §4 Q-003 示例)

## §5 SLO 指标
per 性能基线 (per spec/sla/01 + ADR-0037 §2 D19)：
- **可用性**: 99.9% (per spec/sla/01 §2 月度 43.2min 停机预算)
- **P99 latency**: < 500ms
- **错误率**: < 0.1%
- **错误预算**: 30 天 0.3% (43.2 分钟)

## §6 已知缺口
1. Grafana dashboard 设计 (Phase I+ 实测)
2. Alertmanager routing 规则 (Phase I+ 跟 SRE 域 Lead 拍板)
3. 日志采样率 (高流量场景)
4. Tracing sampling (1% vs 100%)
5. 5 域业务域 Lead 告警路由 (per ADR-0036 §4)
6. PII 字段 mask 规则 (与 8/27 11:06 JST secret 安全对齐)
7. 22 domain handler 真实数据延迟基线 (Phase H+ 真实接入后)

## §7 引用文档
- adr/0037-phase-h-architecture.md
- adr/0038-phase-i-architecture.md
- spec/sla/01-sla-spec.md (Phase I)
- spec/deploy/01-k8s-deployment-spec.md (Phase I)
- spec/cache/01-cache-contract-spec.md
- spec/saga/01-saga-coordination-spec.md
- AGENTS.md

## §8 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：8 业务指标 + 5 告警 + 日志 + traces + SLO + 7 已知缺口 | ADR-0037 §8 Phase I |
