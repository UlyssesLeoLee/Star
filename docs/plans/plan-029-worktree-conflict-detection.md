# Implementation Plan: PLAN-029 — Worktree Conflict Detection

> **RFC**: RFC-029
> **Domain Lead**: domain-worktree Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-029, RFC-016, RFC-027, RFC-028
> **相关 Module Spec**: domain-worktree-spec.md
> **相关 PoC**: POC-024, POC-025

---

## 目标(Goals)

1. MVP:File-level Conflict Detection(基于 Git diff metadata)
2. V1:Symbol-level Conflict Detection(基于 Tree-sitter / Language Server + AI 辅助)
3. Heatmap 投影(跨 Worktree 修改文件矩阵)
4. 风险等级分类(None / Low(1-2 file) / Medium(3-5) / High(>5 或核心文件))
5. 性能:100 Worktree / 10k File 下,Conflict Detection < 1s
6. 缓解 RISK-028 Worktree Conflict Explosion

## 非目标(Non-Goals)

1. ❌ 全文 AI 分析(成本爆炸,RFC-029 拒绝)
2. ❌ Conflict Resolution Recommendations(V2)
3. ❌ 跨 Project Conflict(MVP 限于 Project 内)
4. ❌ Conflict 自动 Merge(MVP 仅检测,人工处理)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-worktree Lead** | File-level Conflict Detection / Heatmap | ❌ |
| **domain-context Lead** | Symbol-level Detection(V1 依赖 Symbol Index) | ❌ |
| **SRE Lead** | 性能监控 / 过度告警率监控 | ❌ |
| **domain-collaboration Lead** | Heatmap UI 渲染 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **WCD-001** | `FileConflictDetector` 实现(基于 `git diff --name-only`) | domain-worktree | RFC-029 | 350K | MVP 阶段核心 |
| **WCD-002** | 风险等级分类(None / Low / Medium / High) | domain-worktree | WCD-001 | 300K | 核心文件定义明确 |
| **WCD-003** | 核心文件识别(`package.json` / `Cargo.toml` / 配置文件) | domain-worktree | WCD-001 | 250K | 核心文件定义表 |
| **WCD-004** | Heatmap 投影 Phase 1(Worktree × File 矩阵) | domain-worktree | WCD-001 | 450K | 100 Worktree / 10k File < 500ms |
| **WCD-005** | 性能优化(Git diff 缓存 / Heatmap 预计算 / 增量更新) | domain-worktree | WCD-001,004 | 350K | 性能达标 |
| **WCD-006** | UI 集成(Worktree Dashboard 显示 Conflict Warning) | domain-collaboration | WCD-002 | 300K | UI 渲染流畅 |
| **WCD-007** | 过度告警 UI 提示("File-level 告警,可能不同 Symbol") | domain-collaboration | WCD-001 | 200K | UI 明确标注 |
| **WCD-008** | 性能监控(Conflict Detection P95 / 过度告警率) | SRE | WCD-001 | 250K | 监控 + 告警 |
| **WCD-009** | tenant_id / workspace_id / project_id 三级隔离 | domain-worktree | WCD-001 | 200K | 隔离生效 |
| **WCD-010** | POC-024 验证(100 Worktree / 10k File,Conflict Detection < 1s) | domain-worktree | WCD-001~005 | 300K | POC-024 通过 |

**Phase 1 合计**:约 **2.95M tokens**

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **WCD-101** | Symbol-level Conflict Detection(基于 Tree-sitter / Language Server) | domain-worktree + domain-context | RFC-028 | 700K | V1 准确率 > 90% |
| **WCD-102** | Heatmap 完整版(Worktree × Symbol 矩阵) | domain-worktree + domain-context | WCD-101 | 500K | 100 Worktree / 10k Symbol < 1s |
| **WCD-103** | AI 辅助决策(AI 二次确认,Token 限制) | domain-context | WCD-101 | 400K | AI 仅在 File-level 告警后调用 |
| **WCD-104** | Saved Worktree Views 个性化(§30.3) | domain-collaboration | WCD-004 | 350K | 用户可保存 Worktree 过滤视图 |
| **WCD-105** | 核心文件定义扩展(更多业务核心文件) | domain-worktree | WCD-003 | 300K | 业务可配置 |

**Phase 2 合计**:约 **2.25M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **WCD-201** | Semantic Conflict Detection(基于 Embedding) | 1.0M |
| **WCD-202** | Cross-Worktree Dependency Graph | 800K |
| **WCD-203** | Conflict Resolution Recommendations(AI 辅助 Merge 建议) | 800K |

**Phase 3 合计**:约 **2.6M tokens**

---

## 依赖矩阵

```
RFC-029 依赖:
  - RFC-016 (Worktree 聚合)
  - RFC-027 (ChangeSet 修改文件列表)
  - RFC-028 (Symbol Analysis,V1 依赖)

RFC-029 被依赖:
  - RFC-024 (Context Compiler 加载 Conflict 信息)
  - RFC-017 (Execution 持有 Conflict 状态)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Worktree Conflict Explosion | Medium | File-level 第一阶段(§4.1.6);Heatmap 投影;Symbol-level 推迟 V1 |
| File-level 过度告警 | Medium | UI 明确提示;V1 Symbol-level 精确化 |
| Conflict Detection 性能 | Low | Git diff 缓存;Heatmap 预计算;增量更新 |
| AI 辅助成本 | Low | AI 仅在 File-level 告警后调用;Token 限制;Fallback |
| Language Server 集成成本(V1) | Medium | POC-025 提前验证;选 1-2 种语言试点 |

## 验收标准(MVP)

1. ✅ File-level Conflict Detection 基于 Git diff metadata
2. ✅ Heatmap 投影基础版
3. ✅ 风险等级分类(None / Low / Medium / High)
4. ✅ 100 Worktree / 10k File P95 < 1s
5. ✅ 过度告警 UI 提示
6. ✅ 性能监控指标
7. ✅ tenant_id / workspace_id / project_id 三级隔离
8. ✅ 核心文件定义明确
9. ✅ Conflict 信息进入 Context Packet
10. ✅ POC-024 验证通过

## Token-OLU 总览

- **Phase 1(MVP)**:2.95M tokens ≈ 10-30 人·天
- **Phase 2(V1)**:2.25M tokens(Symbol-level + Heatmap 完整)
- **Phase 3(V2)**:2.6M tokens
- **MVP + V1**:5.2M tokens(可由 domain-worktree Lead 1 人 12-16 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
