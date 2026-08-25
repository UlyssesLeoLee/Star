# Implementation Plan: PLAN-022 — SCM Adapter Model

> **RFC**: RFC-022
> **Domain Lead**: domain-scm Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-022, RFC-017, RFC-027
> **相关 Module Spec**: domain-scm-spec.md
> **相关 PoC**: POC-026, POC-027

---

## 目标(Goals)

1. Domain 层通过 `SCMPort` trait 抽象 SCM 操作
2. infrastructure 层实现 GitHub / GitLab Adapter(MVP)
3. 业务对象统一:`Repository / Branch / Commit / PullRequest / Review / Pipeline`
4. Domain 层零厂商对象(禁止 `GitHubPullRequestObject`)
5. Rate Limit 兜底 + Webhook 协议转换
6. SCM Sync Loop 防护(RISK-027 缓解)

## 非目标(Non-Goals)

1. ❌ Gitea / Bitbucket / Azure DevOps Adapter(V1 评估)
2. ❌ Self-hosted Git 适配(V2)
3. ❌ 完整 SCM 平台功能(只 Star 平台需要的子集)
4. ❌ SCM 平台 UI 复制(只集成必要数据)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-scm Lead** | SCMPort trait / 业务对象 / 业务逻辑 | ❌ |
| **SCM Adapter Tech Lead** | GitHub / GitLab Adapter 实现 / Webhook 处理 | ❌(独立于 domain-scm) |
| **SRE Lead** | Rate Limit 监控 / Webhook 接收 / 故障转移 | ❌ |
| **domain-feedback Lead** | PR Review Comment → Structured Feedback 解析 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-5)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SCM-001** | `SCMPort` trait 设计(list_repositories / get_repository / list_branches / get_commit / list_pull_requests / create_webhook 6 个方法) | domain-scm | RFC-022 | 350K | Port trait 签名冻结 |
| **SCM-002** | 业务对象:`Repository / Branch / Commit / PullRequest / Review / Pipeline` | domain-scm | SCM-001 | 500K | 6 个业务对象完整 |
| **SCM-003** | `SCMPullRequest` 命名统一(GitLab MR 翻译为 PR) | domain-scm | SCM-002 | 200K | UI 层做翻译,Domain 层统一 PR |
| **SCM-004** | GitHub Adapter 实现 | SCM Adapter Tech | SCM-001,002 | 1.0M | POC-026 验证 Repository / Branch / Commit / PR / Review / Webhook 全功能 |
| **SCM-005** | GitLab Adapter 实现(含 MR / Pipeline) | SCM Adapter Tech | SCM-001,002 | 1.0M | POC-027 验证同上,含 MR / Pipeline |
| **SCM-006** | Mock SCM Adapter(测试用) | domain-scm | SCM-001 | 250K | Domain 层单元测试可独立运行 |
| **SCM-007** | Contract Testing(Port 行为契约) | domain-scm + SRE | SCM-001 | 300K | GitHub / GitLab Adapter 全部通过 |
| **SCM-008** | Webhook 接收服务(GitHub: X-GitHub-Event / GitLab: X-Gitlab-Event) | SCM Adapter Tech + SRE | SCM-004,005 | 500K | 协议转换统一为 Domain Event |
| **SCM-009** | Rate Limit 兜底(Exponential Backoff / Token Bucket) | SCM Adapter Tech | SCM-004,005 | 300K | Rate Limit 触发不阻塞用户 |
| **SCM-010** | SCM Sync Loop 防护(Idempotency Key / Sync Token) | domain-scm | SCM-001 | 400K | RISK-027 缓解,Loop 检测 < 1% |
| **SCM-011** | PR Review Comment → Structured Feedback 解析(V1 准备,MVP 基础结构) | domain-feedback | SCM-008 | 400K | 解析失败的 Comment Fallback 为普通 Comment |
| **SCM-012** | Adapter 单元测试覆盖率 > 80% | SCM Adapter Tech + SRE | SCM-004,005 | 300K | CI 强制 |

**Phase 1 合计**:约 **5.5M tokens**

### Phase 2 (V1,Week 6-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SCM-101** | PR Review Feedback Import 完整版(解析 Review Comment → Structured Feedback) | domain-feedback | SCM-011 | 500K | 解析率 > 80% |
| **SCM-102** | Advanced Rate Limit 策略(预分配 / 优先级队列) | SCM Adapter Tech | SCM-009 | 350K | 高峰时段 QPS 稳定 |
| **SCM-103** | Webhook 重试策略(失败主动轮询) | SRE | SCM-008 | 300K | Webhook 接收失败时主动拉取 |
| **SCM-104** | Gitea / Bitbucket / Azure DevOps Adapter 评估 | domain-scm + SCM Adapter Tech | SCM-004,005 | 500K | 评估报告 + 实施优先级 |
| **SCM-105** | Domain 层业务逻辑(PR 自动关联 WorkItem / Worktree) | domain-scm | SCM-001 | 400K | PR 自动识别 WorkItem |

**Phase 2 合计**:约 **2.05M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **SCM-201** | Self-hosted Git 适配 | 1.0M |
| **SCM-202** | SCM Performance Analytics | 600K |
| **SCM-203** | Cross-SCM 迁移工具 | 800K |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-022 依赖:
  - 无(基础设施层抽象)

RFC-022 被依赖:
  - RFC-017 (Commit / PR 关联到 Execution)
  - RFC-027 (ChangeSet 引用 Git Diff)
  - RFC-016 (Worktree 关联 SCM Repository)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| SCM Sync Loop | High | Idempotency Key + Sync Token 校验;Loop 检测监控 |
| Rate Limit 触发 | Medium | Adapter 内置 Rate Limit 兜底;Exponential Backoff;Token Bucket |
| Webhook 协议变化 | High | Adapter 隔离 Webhook 格式;版本锁定;升级测试 |
| 厂商能力差异 | Medium | ACL 模式;Adapter 内部处理差异 |
| 测试覆盖不足 | Low | Contract Testing;CI 强制覆盖率 > 80% |

## 验收标准(MVP)

1. ✅ `SCMPort` trait 6 个方法完整
2. ✅ 6 个业务对象统一(Repository / Branch / Commit / PullRequest / Review / Pipeline)
3. ✅ Domain 层零厂商对象
4. ✅ GitHub + GitLab Adapter 完整
5. ✅ Mock Adapter + Contract Testing
6. ✅ Rate Limit 兜底
7. ✅ Webhook 协议转换统一
8. ✅ SCM Sync Loop 防护(Idempotency Key / Sync Token)
9. ✅ Adapter 单元测试覆盖率 > 80%
10. ✅ POC-026/027 验证全功能

## Token-OLU 总览

- **Phase 1(MVP)**:5.5M tokens ≈ 18-55 人·天
- **Phase 2(V1)**:2.05M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:7.55M tokens(由 domain-scm Lead + SCM Adapter Tech Lead 2 人 16-20 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
