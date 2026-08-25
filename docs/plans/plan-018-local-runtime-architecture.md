# Implementation Plan: PLAN-018 — Local Runtime Architecture

> **RFC**: RFC-018
> **Domain Lead**: domain-local-runtime Lead(集群内) + Local Daemon Tech Lead(集群外)
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-018, RFC-019, RFC-020, RFC-030
> **相关 Module Spec**: domain-local-runtime-spec.md
> **相关 PoC**: POC-016, POC-030

---

## 目标(Goals)

1. Local Daemon 独立 Rust 二进制,运行于 Developer Machine / Self-hosted Runner / Cloud Workspace
2. mTLS + Device Identity 双向认证
3. 8 个白名单 Command(`StartAgent` / `StopAgent` / `RunCommand` / `ApplyFeedback` / `ReportObservation` / `CreateWorktree` / `CommitChange` / `Cleanup`)
4. 不增加 K8s Workload 计数(§23.1)
5. 不形成 Remote Shell(§23.2)
6. 支持 Linux / macOS / Windows 三平台

## 非目标(Non-Goals)

1. ❌ Cloud Development Runtime(V2 候选,§30.4)
2. ❌ Advanced Runtime Isolation(Kata / Firecracker,V2)
3. ❌ Agent Container 内嵌到 Local Daemon(违反 K8s Tax)
4. ❌ SSH 远程执行(违反 §23.2)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-local-runtime Lead** | 集群内 Runtime Registry / Port / Adapter | ❌ |
| **Local Daemon Tech Lead** | 集群外 Rust 二进制 / 跨平台构建 | ❌(独立于 domain-local-runtime) |
| **SRE Lead** | 设备注册 / Revocation / 监控 | ❌ |
| **Security Lead** | mTLS / Device Identity / Certificate Authority | ❌ |
| **domain-agent Lead** | Agent 在 Local Runtime 启动 / 监控 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-6)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **LRT-001** | Local Daemon 骨架(Rust binary,Linux / macOS / Windows) | Local Daemon Tech | RFC-018 | 500K | 三平台编译通过;Daemon 启动 < 1s |
| **LRT-002** | mTLS 双向认证实现(TLS 1.3+,证书轮转) | Local Daemon Tech + Security | LRT-001 | 600K | POC-016 验证握手 < 500ms |
| **LRT-003** | Device Identity 管理(UUID v4 / v7,Revocation 列表) | Local Daemon Tech + SRE | LRT-002 | 400K | 设备注册 / 撤销 API |
| **LRT-004** | Command Token TTL 5min + 重新申请机制 | Local Daemon Tech | LRT-002 | 300K | Token 过期窗口符合预期 |
| **LRT-005** | 8 个白名单 Command 实现(StartAgent / StopAgent / RunCommand / ApplyFeedback / ReportObservation / CreateWorktree / CommitChange / Cleanup) | Local Daemon Tech | LRT-004 | 800K | POC-016 验证 8 个 Command 全部生效 |
| **LRT-006** | Observed State 上报(1s 批量 Throttle) | Local Daemon Tech | RFC-020 | 300K | 1s 批量,网络流量可控 |
| **LRT-007** | Outbound Only(不开放 Inbound 端口,NAT 友好) | Local Daemon Tech | LRT-001 | 200K | 端口扫描验证无 Inbound |
| **LRT-008** | Daemon 升级策略(强制最低版本,向后兼容) | Local Daemon Tech + SRE | LRT-001 | 400K | 旧版本 Daemon 升级 / Revoke 流程 |
| **LRT-009** | Daemon Supervisor(Systemd / launchd / Windows Service) | Local Daemon Tech | LRT-001 | 300K | Daemon 崩溃后自动重启 |
| **LRT-010** | Crash Report 上报到 Control Plane | Local Daemon Tech | LRT-002 | 200K | 异常行为监控可见 |
| **LRT-011** | `domain-local-runtime` Module:RuntimeRegistry / RuntimeCommand / RuntimeObservation 实体 | domain-local-runtime | RFC-018 | 400K | 集群内 Registry 持久化 |
| **LRT-012** | Runtime Port / Adapter 抽象 | domain-local-runtime | LRT-011 | 350K | Local / Self-hosted / Cloud 统一接口 |
| **LRT-013** | `Remote Disable` 强制停止接受新 Command | Local Daemon Tech | LRT-002 | 250K | 设备丢失 / 离职可立即切断 |
| **LRT-014** | Linux / macOS / Windows Filesystem Scope 集成(Landlock / Seatbelt / Job Object) | Local Daemon Tech | RFC-019 | 500K | POC-030 验证 9 项隔离 |
| **LRT-015** | Process Scope(seccomp / sandbox-exec / Job Object) | Local Daemon Tech | LRT-014 | 450K | POC-030 验证 |

**Phase 1 合计**:约 **5.95M tokens**(本地基础设施工程量大)

### Phase 2 (V1,Week 7-12)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **LRT-101** | Self-hosted Runner 类型支持(企业内网) | Local Daemon Tech | LRT-001 | 600K | 企业自托管 Runner 可注册 |
| **LRT-102** | Runtime Policy 模板(Project / Tenant) | domain-local-runtime | LRT-011 | 400K | Policy 模板复用 |
| **LRT-103** | Filesystem Scope 性能优化 | Local Daemon Tech | LRT-014 | 350K | 静态规则预编译;延迟 < 5% |
| **LRT-104** | Command Token 申请 P95 < 200ms | Local Daemon Tech | LRT-004 | 250K | 性能达标 |

**Phase 2 合计**:约 **1.6M tokens**

### Phase 3 (V2,Week 13+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **LRT-201** | Advanced Runtime Isolation(Kata / Firecracker) | 1.2M |
| **LRT-202** | Cloud Development Runtime 集成 | 1.5M |
| **LRT-203** | Hardware Security Module(HSM)集成 | 800K |

**Phase 3 合计**:约 **3.5M tokens**

---

## 依赖矩阵

```
RFC-018 依赖:
  - 无(基础设施层)

RFC-018 被依赖:
  - RFC-019 (Security Model)
  - RFC-020 (Observed State)
  - RFC-030 (Agent Policy 在 Local Runtime 强制)
  - RFC-021 (Agent 启动 / 监控)
  - RFC-016 (Worktree 隔离依赖 Local Runtime)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Local Runtime Compromise | Critical | mTLS + Device Identity + Command 白名单 + Filesystem Scope + Revocation + Remote Disable |
| 跨平台 Filesystem Scope 不一致 | High | §29 平台差异矩阵;Platform Abstraction Layer;CI 三平台测试 |
| Local Daemon Version Fragmentation | Medium | 强制最低版本;向后兼容 API;Version 分布监控 |
| Daemon 升级困难 | Medium | 客户端自升级机制;SRE 主动通知 |

## 验收标准(MVP)

1. ✅ Local Daemon 三平台编译通过(Linux / macOS / Windows)
2. ✅ mTLS 双向认证握手 P95 < 500ms
3. ✅ 8 个白名单 Command 全部生效
4. ✅ Command Token TTL 5min,过期重新申请
5. ✅ Observed State 1s 批量上报
6. ✅ 不开放 Inbound 端口(Outbound Only)
7. ✅ Remote Disable 立即生效
8. ✅ Daemon Supervisor 自动重启
9. ✅ Filesystem / Process Scope 9 项隔离(POC-030 验证)
10. ✅ §23.1 不计入 K8s Workload

## Token-OLU 总览

- **Phase 1(MVP)**:5.95M tokens ≈ 20-60 人·天(基础设施工程量大)
- **Phase 2(V1)**:1.6M tokens
- **Phase 3(V2)**:3.5M tokens
- **MVP + V1**:7.55M tokens(由 Local Daemon Tech Lead + domain-local-runtime Lead 2 人 16-20 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
