# Implementation Plan: PLAN-019 — Local Runtime Security Model

> **RFC**: RFC-019
> **Domain Lead**: Security Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-019, RFC-018, RFC-030
> **相关 Module Spec**: domain-local-runtime-spec.md
> **相关 PoC**: POC-016, POC-030

---

## 目标(Goals)

1. mTLS + Device Identity + Command 白名单 + Filesystem Scope + Process Scope + Credential Broker 6 层防御
2. §23.2 16 项强制项全部实现
3. Filesystem Scope 跨平台一致(Linux Landlock / macOS Seatbelt / Windows Job Object)
4. Device Revocation / Remote Disable 立即生效(RISK-016 缓解)
5. Credential Broker 代理 Git Credential / AI API Key(缓解 RISK-018)

## 非目标(Non-Goals)

1. ❌ Hardware Security Module(HSM)(V2 候选)
2. ❌ SELinux / AppArmor 集成(MVP 阶段 OS 原语即可)
3. ❌ Zero-Knowledge 加密(超出 MVP 范围)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **Security Lead** | 6 层防御架构 / 16 项强制项验证 | ❌ |
| **Local Daemon Tech Lead** | Filesystem / Process Scope 跨平台实现 | ❌(独立于 Security) |
| **SRE Lead** | Certificate Authority / Revocation 列表 | ❌ |
| **domain-local-runtime Lead** | 集群内 Security Policy 协调 | ❌ |
| **Compliance Lead** | §6.8 AI Content Retention 合规审查 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-6,与 RFC-018 协同)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SEC-001** | mTLS 双向认证实施(继承 LRT-002) | Security + Local Daemon Tech | RFC-018 | (含于 LRT-002) | TLS 1.3+ 握手 < 500ms |
| **SEC-002** | Device Identity 唯一 + Revocation API | Security + SRE | SEC-001 | 350K | 设备丢失可立即吊销 |
| **SEC-003** | Command Token TTL 5min + 重新申请(继承 LRT-004) | Security | SEC-001 | (含于 LRT-004) | Token 过期窗口符合预期 |
| **SEC-004** | 8 个白名单 Command 严格校验(继承 LRT-005) | Security | SEC-001 | 250K | 任意 Shell 命令被拒 |
| **SEC-005** | Filesystem Scope: Linux Landlock | Local Daemon Tech | SEC-001 | 500K | POC-030 验证 9 项隔离 |
| **SEC-006** | Filesystem Scope: macOS Seatbelt | Local Daemon Tech | SEC-005 | 450K | 同上,macOS 平台 |
| **SEC-007** | Filesystem Scope: Windows Job Object | Local Daemon Tech | SEC-005 | 500K | 同上,Windows 平台 |
| **SEC-008** | Process Scope: Linux seccomp | Local Daemon Tech | SEC-005 | 400K | Agent 进程不能 fork 到 Worktree 外 |
| **SEC-009** | Process Scope: macOS sandbox-exec | Local Daemon Tech | SEC-006 | 400K | 同上,macOS 平台 |
| **SEC-010** | Process Scope: Windows Job Object | Local Daemon Tech | SEC-007 | 450K | 同上,Windows 平台 |
| **SEC-011** | Credential Broker 代理 Git Credential | Security + domain-scm | SEC-001 | 600K | Agent 不直接持有 Git Credential |
| **SEC-012** | Credential Broker 代理 AI API Key | Security + domain-agent | SEC-001 | 600K | Agent 不直接持有 AI API Key |
| **SEC-013** | Remote Disable 强制停止接受新 Command | Security + Local Daemon Tech | SEC-002 | 300K | 设备丢失 / 离职可立即切断 |
| **SEC-014** | §23.2 16 项强制项 验证矩阵 | Security | SEC-001~013 | 400K | POC-016 全部通过 |
| **SEC-015** | Audit Log 强制(Command / File Access / Process Fork) | Security + domain-audit | SEC-001 | 300K | Audit 完整可查 |
| **SEC-016** | Filesystem Scope 配置 dry-run 模式 | Security | SEC-005,006,007 | 250K | 配置变更不立即生效,可验证 |
| **SEC-017** | §29 平台差异矩阵文档 | Security + Local Daemon Tech | SEC-005~010 | 200K | 三平台差异明确文档化 |
| **SEC-018** | Device Key 存储介质选型(§6 Keychain / §6.5 TPM 评估) | Security | SEC-002 | 300K | 安全团队决策落地 |

**Phase 1 合计**:约 **6.05M tokens**(安全工程量大)

### Phase 2 (V1,Week 7-12)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SEC-101** | Filesystem Scope 性能优化(< 5% 开销) | Local Daemon Tech | SEC-005~007 | 400K | 性能达标 |
| **SEC-102** | Runtime Policy 模板(V1) | domain-local-runtime | SEC-001 | 350K | Policy 模板复用 |
| **SEC-103** | Token Rotation 自动化(mTLS Client Certificate) | Security + SRE | SEC-001 | 300K | 证书自动轮转 |
| **SEC-104** | Cross-platform Penetration Test | Security | SEC-001~018 | 500K | 渗透测试报告 + 修复 |
| **SEC-105** | Security Audit(独立第三方) | Security + Compliance | SEC-104 | 600K | 第三方审计报告 |

**Phase 2 合计**:约 **2.15M tokens**

### Phase 3 (V2,Week 13+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **SEC-201** | HSM 集成 | 1.5M |
| **SEC-202** | Zero-Knowledge 加密 | 1.2M |
| **SEC-203** | AI-driven Anomaly Detection(异常行为检测) | 800K |

**Phase 3 合计**:约 **3.5M tokens**

---

## 依赖矩阵

```
RFC-019 依赖:
  - RFC-018 (Local Runtime 架构基础)

RFC-019 被依赖:
  - RFC-030 (Agent Policy 在 Local Runtime 强制)
  - RFC-016 (Worktree 隔离)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Local Runtime Compromise | Critical | 6 层防御;Device Revocation;Remote Disable;Audit 完整 |
| Filesystem Scope 跨平台不一致 | High | §29 平台差异矩阵;CI 三平台测试;第三方渗透测试 |
| Filesystem Scope 性能开销 | Medium | 静态规则预编译;按需启用;性能监控 |
| Device Key 泄露 | Critical | Keychain / TPM 存储;mTLS 双向;Revocation 即时 |
| Command Token 泄露 | High | Short-lived TTL;Token Rotation;mTLS 双向 |

## 验收标准(MVP)

1. ✅ §23.2 16 项强制项全部实现 + POC-016 验证
2. ✅ Filesystem Scope 三平台集成(POC-030 验证)
3. ✅ Process Scope 三平台集成
4. ✅ Device Identity 唯一 + Revocation 立即生效
5. ✅ Command Token TTL 5min
6. ✅ Credential Broker 代理 Git Credential / AI API Key
7. ✅ Remote Disable 强制停止
8. ✅ Audit Log 完整
9. ✅ 第三方渗透测试通过(Phase 2 末)
10. ✅ §6.8 AI Content Retention 合规

## Token-OLU 总览

- **Phase 1(MVP)**:6.05M tokens ≈ 20-60 人·天(安全工程量大)
- **Phase 2(V1)**:2.15M tokens
- **Phase 3(V2)**:3.5M tokens
- **MVP + V1**:8.2M tokens(由 Security Lead + Local Daemon Tech Lead 2 人 18-22 周完成,Security Lead 不兼任 Local Daemon Tech)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
