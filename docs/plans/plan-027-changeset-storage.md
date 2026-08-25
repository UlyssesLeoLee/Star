# Implementation Plan: PLAN-027 — ChangeSet Storage

> **RFC**: RFC-027
> **Domain Lead**: domain-development Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-027, RFC-016, RFC-022, RFC-028
> **相关 Module Spec**: domain-development-spec.md
> **相关 PoC**: POC-021

---

## 目标(Goals)

1. ChangeSet 作为结构化聚合根(非仅 Git Diff)
2. 7 类元数据(Files / Symbols / Risk Signals / Dependency / Schema / Config / Test Changes)
3. Git Diff Reference 走 Object Storage
4. 风险门控(ChangeSet 提交时自动识别风险等级)
5. Symbol-level Feedback 关联(§25.1,V1)
6. Acceptance Coverage 验证(§4.5.5)
7. Storage Lifecycle Policy(§5.1)

## 非目标(Non-Goals)

1. ❌ 完整 IDE Compiler Database 集成(MVP Tree-sitter,V1 Language Server)
2. ❌ Cross-ChangeSet Dependency Graph(§30.4 V2)
3. ❌ ChangeSet 全文搜索(Projection,V2 候选)
4. ❌ Schema 自动应用(MVP 仅识别,不应用)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-development Lead** | ChangeSet 聚合根 / 7 类元数据 / 风险门控 | ❌ |
| **domain-context Lead** | Symbol 提取(MVP 简化,V1 完整) | ❌ |
| **SRE Lead** | Object Storage / Lifecycle Policy | ❌ |
| **domain-worktree Lead** | ChangeSet 与 Worktree 关联 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CS-001** | `change_sets` 表 Schema(7 类元数据字段 + 三级 tenant 隔离) | domain-development | RFC-027 | 400K | 7 类字段完整 |
| **CS-002** | `change_set_diff_objects` 表(Object Storage 引用) | domain-development + SRE | CS-001 | 250K | diff_key 引用 |
| **CS-003** | Git Diff Reference 走 Object Storage | domain-development | CS-002 | 300K | 大文件不存 PostgreSQL |
| **CS-004** | Files 元数据(修改文件列表 / lines added / lines removed) | domain-development | CS-001 | 300K | 字段完整 |
| **CS-005** | Symbols 简化版(Tree-sitter,function / class / interface 识别) | domain-context | CS-001 | 450K | MVP 准确率 80% |
| **CS-006** | Risk Signals(breaking change / 数据库迁移 / public API change) | domain-development | CS-001 | 400K | 风险等级 Low / Medium / High |
| **CS-007** | Dependency Changes 简化(识别主要依赖 package.json / Cargo.toml) | domain-development | CS-001 | 350K | 依赖变更可追溯 |
| **CS-008** | Schema Changes 简化(识别 Prisma / Liquibase / Alembic) | domain-development | CS-001 | 400K | Schema 变更可识别 |
| **CS-009** | Config Changes(env / yaml / toml) | domain-development | CS-001 | 300K | Config 变更可追溯 |
| **CS-010** | Test Changes(新增 / 修改 / 删除) | domain-development | CS-001 | 250K | 字段完整 |
| **CS-011** | 风险门控(ChangeSet 提交时自动识别) | domain-development | CS-006 | 400K | RISK-A27-1 缓解 |
| **CS-012** | Validation 步骤(强制 Git Diff 与元数据一致) | domain-development | CS-001 | 350K | 不一致报错 |
| **CS-013** | Acceptance Coverage 字段(`change_set.acceptance_coverage`) | domain-development | CS-001 | 300K | §4.5.5 验证 |
| **CS-014** | Symbol-level Feedback 关联(基础版,V1 完整) | domain-context + domain-feedback | CS-005 | 350K | V1 渐进(POC-025) |
| **CS-015** | Lifecycle Policy(>90d 归档,§5.1) | SRE | CS-001 | 250K | 归档脚本 + 监控 |

**Phase 1 合计**:约 **5.05M tokens**

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **CS-101** | Symbol-level ChangeSet 完整版(基于 Language Server,POC-025) | domain-context | CS-005 | 600K | 准确率 > 95% |
| **CS-102** | Acceptance Coverage 验证完整版 | domain-validation | CS-013 | 500K | AC 覆盖率自动计算 |
| **CS-103** | 依赖审计扩展(更多依赖格式) | domain-development | CS-007 | 400K | 覆盖主流格式 |
| **CS-104** | Risk Signal 细化(更多规则) | domain-development | CS-006 | 350K | 风险识别率提升 |
| **CS-105** | Cross-ChangeSet 关联(同一 Worktree 多 ChangeSet) | domain-development | CS-001 | 300K | 关联可查询 |

**Phase 2 合计**:约 **2.15M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **CS-201** | ChangeSet Diff 增量(只存差异) | 800K |
| **CS-202** | AI 风险预测(基于历史 ChangeSet) | 1.0M |
| **CS-203** | ChangeSet 性能分析 | 600K |

**Phase 3 合计**:约 **2.4M tokens**

---

## 依赖矩阵

```
RFC-027 依赖:
  - RFC-016 (Worktree 聚合)
  - RFC-022 (SCM Adapter Git Diff 来源)
  - RFC-028 (Symbol Analysis)

RFC-027 被依赖:
  - RFC-024 (Context Compiler 加载 ChangeSet)
  - RFC-029 (Worktree Conflict 基于 ChangeSet)
  - RFC-026 (AgentSession 关联 ChangeSet)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| 元数据提取失败 | Medium | Fallback 为仅 Git Diff;Validation 步骤强制元数据 |
| Storage 增长 | Medium | Lifecycle Policy >90d 归档;Diff 压缩 |
| 元数据不一致 | High | Validation 步骤强制校验(ChangeSet 提交时 Validation) |
| Symbol 提取不准确 | Low | MVP 仅 File-level;V1 渐进(POC-025) |
| 依赖审计覆盖不足 | Low | MVP 仅识别主要依赖;V1 扩展 |

## 验收标准(MVP)

1. ✅ `change_sets` 表 7 类元数据 + 三级 tenant 隔离
2. ✅ Git Diff Reference 走 Object Storage
3. ✅ 风险门控自动识别(Low / Medium / High)
4. ✅ Symbol-level Feedback 基础版(V1 完整)
5. ✅ Acceptance Coverage 字段
6. ✅ Validation 步骤强制校验
7. ✅ Lifecycle Policy >90d 归档
8. ✅ Storage 配额监控
9. ✅ 依赖审计覆盖 package.json / Cargo.toml
10. ✅ 核心文件定义明确(`package.json` / `Cargo.toml` / 配置文件)

## Token-OLU 总览

- **Phase 1(MVP)**:5.05M tokens ≈ 17-50 人·天(7 类元数据 + 风险门控)
- **Phase 2(V1)**:2.15M tokens
- **Phase 3(V2)**:2.4M tokens
- **MVP + V1**:7.2M tokens(由 domain-development Lead + domain-context Lead 2 人 16-20 周完成,Symbol 提取跨域协作)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
