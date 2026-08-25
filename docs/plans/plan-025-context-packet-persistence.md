# Implementation Plan: PLAN-025 — Context Packet Persistence

> **RFC**: RFC-025
> **Domain Lead**: domain-context Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-025, RFC-024, RFC-026
> **相关 Module Spec**: domain-context-spec.md
> **相关 PoC**: POC-022

---

## 目标(Goals)

1. Context Packet 持久化(元数据 + Provenance 入 PostgreSQL,大文件走 Object Storage)
2. Provenance 反查支持(REQ-AUDIT-002)
3. HandoffContextPacket 由已持久化 Context Packet 派生
4. Lifecycle Policy(>90d 归档,§5.8)
5. 符合 §6.8 AI Content Retention Policy
6. Object Storage 选型 + 多副本 + 备份

## 非目标(Non-Goals)

1. ❌ Context Packet Diff(增量更新,V2 评估)
2. ❌ Context Packet Version Control(Git-like,V2)
3. ❌ 跨 Tenant Context Packet Federation(V2)
4. ❌ 全文 Context Packet 搜索(V2 候选)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-context Lead** | Context Packet 表 Schema / Provenance 索引 | ❌ |
| **SRE Lead** | Object Storage 部署 / 备份 / 故障转移 | ❌ |
| **Compliance Lead** | §6.8 AI Content Retention 合规审查 | ❌ |
| **domain-audit Lead** | Provenance 反查 / AI Audit 集成 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CTX-P-001** | `context_packets` 表 Schema(15 字段 + 三级 tenant 隔离) | domain-context | RFC-025 | 250K | Migration 通过 |
| **CTX-P-002** | `context_packet_artifacts` 表 Provenance 详情(source_type / source_ref / line_range / priority) | domain-context | CTX-P-001 | 350K | 50+ Provenance / packet |
| **CTX-P-003** | Object Storage 部署(MinIO 或阿里云 OSS,S3 兼容) | SRE | CTX-P-001 | 500K | 多副本 + 备份 |
| **CTX-P-004** | Object Storage Adapter 抽象(支持多种 S3 兼容实现) | SRE | CTX-P-003 | 400K | MinIO / 阿里云 OSS / AWS S3 统一 |
| **CTX-P-005** | 大文件(Repository 索引 / Build Log / Large Diff)走 Object Storage | domain-context | CTX-P-003 | 350K | diff_key 引用 |
| **CTX-P-006** | `ContextPacketCommandPort`(save / load / list_provenance) | domain-context | CTX-P-001 | 300K | 3 个方法 + 错误类型 |
| **CTX-P-007** | Provenance 反查 API `GET /context-packets/{id}/provenance` | domain-context | CTX-P-002 | 300K | P95 < 200ms |
| **CTX-P-008** | HandoffContextPacket 复用已持久化 Context Packet(避免重算) | domain-context + domain-agent | CTX-P-006 | 400K | Token 下降 50%+(不复用 = 重算) |
| **CTX-P-009** | Lifecycle Policy(>90d 归档,§5.8) | SRE | CTX-P-001 | 300K | 归档脚本 + 监控 |
| **CTX-P-010** | Object Storage 故障转移 + 备份恢复演练 | SRE | CTX-P-003 | 300K | RTO < 1h,RPO < 15min |
| **CTX-P-011** | §6.8 AI Content Retention Policy 实施(P0 不可裁剪) | domain-context + Compliance | CTX-P-001 | 350K | P0 永久保留;合规审查通过 |
| **CTX-P-012** | AI Audit 集成(Provenance → Audit 维度) | domain-audit | CTX-P-002 | 350K | REQ-AUDIT-002 完全覆盖 |

**Phase 1 合计**:约 **4.15M tokens**

### Phase 2 (V1,Week 5-8)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CTX-P-101** | 冷热分层(Hot 0-30d PostgreSQL / Warm 30-90d PostgreSQL 压缩 / Cold >90d Object Storage) | SRE + domain-context | CTX-P-009 | 500K | 存储成本下降 50% |
| **CTX-P-102** | Context Packet Diff(增量更新)评估 | domain-context | CTX-P-001 | 250K | 评估报告 |
| **CTX-P-103** | P0 不可裁剪长期保留策略(>N 年) | domain-context + Compliance | CTX-P-011 | 300K | 长期存储方案 |
| **CTX-P-104** | Context Packet 写入性能优化(< 10ms) | domain-context | CTX-P-006 | 300K | 性能达标 |

**Phase 2 合计**:约 **1.35M tokens**

### Phase 3 (V2,Week 9+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **CTX-P-201** | Context Packet Version Control(Git-like) | 1.0M |
| **CTX-P-202** | 跨 Agent Context Sharing | 800K |
| **CTX-P-203** | Predictive Context Preloading | 600K |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-025 依赖:
  - RFC-024 (Context Compiler 已生成 Context Packet)
  - RFC-026 (类似持久化模式,参考 AgentSession)

RFC-025 被依赖:
  - RFC-021 (Agent 接收 Context Packet)
  - RFC-024 (HandoffContextPacket 复用)
  - RFC-017 (Execution 持有 ContextPacket)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Context Packet 存储增长 | Medium | Lifecycle Policy >90d 归档;聚合压缩;冷热分层 |
| Object Storage 故障 | High | S3 兼容多副本;定期备份;故障转移(RTO < 1h) |
| AI Content Retention 违规 | High | §6.8 Retention Policy;P0 不可裁剪;Compliance 审查 |
| Provenance 反查性能 | Low | `context_packet_id` + `source_type` 复合索引;分页 |
| Handoff 复用不一致 | Low | Context Packet 快照 + 版本号;Handoff 时明确版本 |

## 验收标准(MVP)

1. ✅ `context_packets` 表 15 字段 + 三级 tenant 隔离
2. ✅ `context_packet_artifacts` 表 Provenance 详情
3. ✅ 大文件(Repository 索引 / Build Log / Large Diff)走 Object Storage
4. ✅ Provenance 反查 API `GET /context-packets/{id}/provenance` P95 < 200ms
5. ✅ HandoffContextPacket 复用(避免重算)
6. ✅ Lifecycle Policy >90d 归档
7. ✅ §6.8 AI Content Retention 实施,P0 不可裁剪
8. ✅ Object Storage 多副本 + 备份
9. ✅ RTO < 1h,RPO < 15min
10. ✅ AI Audit 集成(Provenance → Audit 维度)

## Token-OLU 总览

- **Phase 1(MVP)**:4.15M tokens ≈ 14-42 人·天
- **Phase 2(V1)**:1.35M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:5.5M tokens(可由 domain-context Lead 1 人 12-16 周完成,Object Storage 部署需 SRE 配合)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
