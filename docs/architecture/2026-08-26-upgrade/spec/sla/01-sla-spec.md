# Spec-01: SLA 规范 (Service Level Agreement)

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**：per ADR-0037 §8 / 2026-08-27 21:59 JST 用户授权 / Phase I production rollout

## §1 目的
定义 Star production SLA：可用性 / 性能 / 支持 / 错误预算 / RTO/RPO。Phase I 部署前必签。8 服务统一 SLA。

## §2 可用性 SLA
- **99.9% 月度可用性** (per ADR-0037 §2 D19 + 5 域 SRE NFR)
- **月度停机预算**: 43.2 分钟
- **计算**: 30 天 × 24h × 60min × 0.1% = 43.2 min
- **不可计入的停机**:
  - 计划内维护 (提前 72h 通知)
  - 第三方依赖故障 (GitHub / GitLab / Bitbucket / Gitea 官方故障 per spec/vcs/05 §5)
  - 客户侧故障

## §3 性能 SLA
per ADR-0037 §2 D19 + 性能基线 (per spec/integration/01 §6 + bench/perf-baseline.md):

| 指标 | 目标 | 实测 (Phase I 启动) |
|------|------|---------------------|
| P50 latency | < 50ms | TODO (待 bench-runner.sh 实测) |
| P95 latency | < 200ms | TODO |
| P99 latency | < 500ms | TODO |
| HTTP success rate | > 99.9% | TODO |
| 22 domain handler read | < 1ms P99 (per spec/cache/01 §4) | TODO |

## §4 支持 SLA
| 等级 | 响应时间 | 解决时间 | 范围 |
|------|----------|----------|------|
| P0 critical | 15min | 4h | 8 服务全宕 + 数据丢失 |
| P1 high | 1h | 8h | 单服务降级 + 核心功能失效 |
| P2 medium | 4h | 24h | 性能降级 + 非核心功能失效 |
| P3 low | 1d | 1w | 增强请求 + UI bug |

- P0: 7×24 监控
- P1+: 工作日 9-18 + 周末 on-call

## §5 RTO + RPO
- **RTO (Recovery Time Objective) = 1h** — 故障后 1h 内恢复服务
- **RPO (Recovery Point Objective) = 15min** — 数据丢失最多 15min
- 实现: 5min snapshot + 15min WAL + 多 region 备份 (Phase I+)

## §6 错误预算
- **月度预算**: 43.2 分钟
- **烧穿率告警**:
  - 25% 烧穿 → SRE 内部 review
  - 50% 烧穿 → SRE + PM 评估暂停新功能
  - 100% 烧穿 → 暂停所有非 P0 变更，专注可靠性

## §7 已知缺口
1. 客户分级 SLA (per-tenant 不同等级，Phase I+ 评估)
2. 多 region SLA (per RTO/RPO，Phase I+ 部署)
3. 5 业务域 Lead SLA 协商 (per ADR-0036 §7 #7 + 8/21 JST 5 域独立 Lead)
4. 第三方依赖 SLA (GitHub / GitLab / Bitbucket API rate limit / outage, per spec/vcs/05 §4)
5. 客户感知 SLA vs 内部 SLA 区分
6. 季度 SLA 报告模板 (Phase I+ 跟 SRE 域 Lead 拍板)
7. Performance Lead 真实身份签字 (per ADR-0037 §4 新增域)

## §8 引用文档
- adr/0037-phase-h-architecture.md
- adr/0038-phase-i-architecture.md
- spec/deploy/01-k8s-deployment-spec.md (Phase I)
- spec/observability/01-monitoring-spec.md (Phase I)
- spec/integration/01-22-domain-integration-spec.md (Phase H)
- AGENTS.md

## §9 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：99.9% 可用性 + 4 性能 + 4 支持 + RTO/RPO + 6 预算 + 7 已知缺口 | ADR-0037 §8 Phase I |
