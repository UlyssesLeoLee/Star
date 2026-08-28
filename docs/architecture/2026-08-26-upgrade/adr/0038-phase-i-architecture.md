# ADR-0038: Phase I Production Rollout 架构

> **状态**：Draft v0.1
> **日期**：2026-08-28
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **审批**：架构师（Mavis 接手 agent per DEC-008）
> **触发**：per ADR-0037 §8 / 2026-08-27 21:59 JST 用户授权 / Phase I production rollout

## §1 背景
Phase H 交付（per commit 9723bae base）：2 份新 spec (integration + saga/02) + ADR-0037 + 28 文件 22 domain handlers + 90 测试 + workspace 495 tests pass。
Phase I 是 Star MVP v1 → production 最后一公里：K8s 部署 + 可观测性 + SLA + Helm chart 框架。

## §2 决策（5 项 D21-D25）
- D21: 新增 spec/deploy/01-k8s-deployment-spec.md — 8 服务 + RBAC + Deployment 模板 + Ingress
- D22: 新增 spec/observability/01-monitoring-spec.md — 8 业务指标 + 5 告警 + 日志 + traces + SLO
- D23: 新增 spec/sla/01-sla-spec.md — 99.9% 月度可用性 + 4 性能 + 4 支持 + RTO/RPO + 错误预算
- D24: deploy/helm/star/ Helm chart 框架 — Chart.yaml + values.yaml + 8 服务 templates
- D25: Phase I+ 多 region 部署 (per SLA RTO=1h) — Phase I 单 region first

## §3 跨 spec/crate 关系表
- spec/deploy/01 ↔ spec/observability/01 (Probe + ServiceMonitor 集成)
- spec/deploy/01 ↔ spec/sla/01 (SLA = 部署目标)
- spec/observability/01 ↔ spec/sla/01 (SLO = SLA 量化)
- spec/observability/01 ↔ crates/star-mcp + 22 handlers (metrics 自动 instrumentation)
- spec/sla/01 ↔ spec/observability/01 (错误预算 = SLO 漏出)
- deploy/helm/star/ ↔ spec/deploy/01 (Helm chart 落地 K8s spec)
- 5 域独立 Lead (per 8/21 JST + ADR-0036/37) ↔ SLA 协商

## §4 12 域 Lead 责任矩阵（per 8/21 JST 续 + ADR-0037 §4）
- 架构域 = Mavis 接手
- SRE 域 = SRE Lead (⏳ 待签)
- 平台域 = 平台工程师 (⏳ 待签)
- 评审域 = 评审主持 (⏳ 待签)
- PM 域 = PM (⏳ 待签)
- Player 域 Lead = ⏳ 待签
- Economy 域 Lead = ⏳ 待签
- Match 域 Lead = ⏳ 待签
- Social 域 Lead = ⏳ 待签
- Admin 域 Lead = ⏳ 待签
- Performance Lead (Phase H 新增) = ⏳ 待签
- **Security Lead (Phase I 新增)** = ⏳ 待签

总计 **12 域 Lead**，全部 ⏳ 真实身份待 DDD Review 阶段补（per 8/27 21:59 JST 三次强化规则）。

## §5 token-OLU 估算（per 8/21 JST 框架）
- Phase I spec 写作 ≈ 2-3M tokens（3 份新 spec + 1 ADR + 1 Helm chart）
- K8s 部署调试 ≈ 5-8M tokens
- 可观测性接入 ≈ 3-5M tokens
- SLA 协商与签发 ≈ 2-3M tokens
- 总 ≈ 12-19M tokens ≈ 2-3 人·周

## §6 与上游 ADR 引用
ADR-0021~0037 全链 + ADR-0037 §8 (本 ADR 是 §8 的 Phase I 落地)

## §7 已知缺口（至少 8 项）
1. 真实 image registry (ghcr.io vs 自建 Harbor) 选型
2. 镜像扫描 (Trivy / Snyk) 接入 (Phase I+)
3. 5 业务域 Lead 真实身份签字 (部署前 DDD Review)
4. 多 region 部署 (Phase I+)
5. 灰度发布 (Argo Rollouts)
6. Disaster Recovery (Velero)
7. 22 domain 真实数据源部署 (依赖 Phase H 真实数据)
8. Security Lead 真实身份 + 渗透测试报告 (Phase I+)

## §8 后果
Phase I 完成后 Star production ready:
- 8 服务 K8s 部署
- 99.9% 月度可用性 SLA 签发
- 完整可观测性 (metrics + logs + traces)
- 12 域 Lead 签字完整

MVP v1 production → Phase J = business rollout (客户接入 + 培训)

## §9 签字栏
| 角色 | 姓名 | 签字日 | 结论 |
|------|------|--------|------|
| 架构师 | Mavis 接手 agent per DEC-008 | 2026-08-28 | 🟢 Mavis 接手代签 |
| SRE Lead | (⏳ 待签) | — | — |
| 平台 | (⏳ 待签) | — | — |
| 评审 | (⏳ 待签) | — | — |
| PM | (⏳ 待签) | — | — |
| Security Lead (Phase I 新增) | (⏳ 待签) | — | — |

## §10 修订历史
| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|------|------|--------|----------|------|
| v0.1 | 2026-08-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版：5 决策 D21-D25 + 12 域 Lead + token-OLU 12-19M | ADR-0037 §8 Phase I |

## §11 引用文档
- adr/0037-phase-h-architecture.md
- adr/0036-phase-g-architecture.md
- spec/deploy/01-k8s-deployment-spec.md (Phase I)
- spec/observability/01-monitoring-spec.md (Phase I)
- spec/sla/01-sla-spec.md (Phase I)
- AGENTS.md
