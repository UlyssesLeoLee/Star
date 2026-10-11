---
title: "Architecture MOC"
generated: "2026-10-04T20:31:00+09:00"
node_type: "moc"
view_count: 14
---

# Architecture MOC

# Architecture 总览

## 16 个架构 view (per AGENTS.md §6.1)

索引于 2026-10-04 补齐。此前只列 5 个（截至 2026-09-03），而 `docs/architecture/` 下实际已有 14 个 view 目录 —— 缺的是索引，不是文档。

### 基础 view

- [[view-2026-08-26-upgrade]] — `2026-08-26-upgrade` · 114 篇 · ADR 0021-0053 + 20 域 spec

### 编排与运行时

- [[view-2026-09-02-upgrade]] — `2026-09-02-upgrade` · 1 篇 · Flutter MVP
- [[view-2026-09-03-agent-runtime]] — `2026-09-03-agent-runtime` · 2 篇 · ECS 运行时
- [[view-2026-09-03-langgraph]] — `2026-09-03-langgraph` · 4 篇 · L0/L1 两级编排
- [[view-2026-09-03-arg]] — `2026-09-03-arg` · 14 篇 · 唯一完整 SRS→DD→RACI→分阶段实现
- [[view-2026-09-03-treesitter-worktree-graph]] — `2026-09-03-treesitter-worktree-graph` · 2 篇 · 代码图

### 核心机制

- [[view-2026-09-07-exclusion-idempotency]] — `2026-09-07-exclusion-idempotency` · 3 篇
- [[view-2026-09-09-subtask-binding]] — `2026-09-09-subtask-binding` · 3 篇

### 契约与开关

- [[view-2026-09-22-aci-mock-interface]] — `2026-09-22-aci-mock-interface` · 1 篇
- [[view-2026-09-22-mock-switches]] — `2026-09-22-mock-switches` · 1 篇

### 桌面端 Rust→WASM

- [[view-2026-09-28-upgrade]] — `2026-09-28-upgrade` · 1 篇 · 前端内存研究
- [[view-2026-09-29-upgrade]] — `2026-09-29-upgrade` · 2 篇 · 端侧研究 + WASM P0P1
- [[view-2026-09-30-upgrade]] — `2026-09-30-upgrade` · 9 篇 · Tauri 全链路 P0–P10

### 协议与实测基线

- [[view-2026-10-01-upgrade]] — `2026-10-01-upgrade` · 2 篇 · Session Memory 协议 + Tauri PoC 索引
- [[view-2026-10-04-dev-baseline]] — `2026-10-04-dev-baseline` · 实测基线 `dev@c1ce1630` + Automation Schedule 剖析
- [[view-2026-10-02-engineering-run-directory]] — `2026-10-02-engineering-run-directory` · 四层导航代码落点 + 硬编码关闭清单

## 27 份 ADR
- [[adr-0021-zero-vendor-cooperation]] — ADR-0021 zero-vendor-cooperation
- [[adr-0022-ide-placement]] — ADR-0022 ide-placement
- [[adr-0023-version-control-provider]] — ADR-0023 version-control-provider
- [[adr-0024-ide-session-identity]] — ADR-0024 ide-session-identity
- [[adr-0025-vendor-adapter-anti-contamination]] — ADR-0025 vendor-adapter-anti-contamination
- [[adr-0026-star-ai-compat]] — ADR-0026 star-ai-compat
- [[adr-0027-star-ide-gateway]] — ADR-0027 star-ide-gateway
- [[adr-0028-gitgit-compat]] — ADR-0028 gitgit-compat
- [[adr-0029-universal-submit]] — ADR-0029 universal-submit
- [[adr-0030-agent-lease-heartbeat-resume]] — ADR-0030 agent-lease-heartbeat-resume
- [[adr-0031-context-graph]] — ADR-0031 context-graph
- [[adr-0032-mcp-transport-stdio]] — ADR-0032 mcp-transport-stdio
- [[adr-0033-agent-co-signing-policy]] — ADR-0033 agent-co-signing-policy
- [[adr-0034-jira-ification]] — ADR-0034 jira-ification
- [[adr-0034-phase-e-architecture]] — ADR-0034 phase-e-architecture
- [[adr-0035-phase-f-architecture]] — ADR-0035 phase-f-architecture
- [[adr-0036-phase-g-architecture]] — ADR-0036 phase-g-architecture
- [[adr-0037-phase-h-architecture]] — ADR-0037 phase-h-architecture
- [[adr-0038-phase-i-architecture]] — ADR-0038 phase-i-architecture
- [[adr-0039-worktree-orchestration-cross-domain]] — ADR-0039 worktree-orchestration-cross-domain
- [[adr-0040-domain-batch]] — ADR-0040 domain-batch
- [[adr-0041-arch-agent-graph-viewer]] — ADR-0041 arch-agent-graph-viewer
- [[adr-0043-audit-onboarding-failed]] — ADR-0043 audit-onboarding-failed
- [[adr-0044-star-agent-runtime-srs]] — ADR-0044 star-agent-runtime-srs
- [[adr-0045-star-agent-runtime-design]] — ADR-0045 star-agent-runtime-design
- [[adr-0046-langgraph-task-management-operations]] — ADR-0046 langgraph-task-management-operations
- [[adr-0047-postgresql-checkpointer-tier3]] — ADR-0047 postgresql-checkpointer-tier3
