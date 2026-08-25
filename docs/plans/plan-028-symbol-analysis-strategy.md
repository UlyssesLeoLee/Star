# Implementation Plan: PLAN-028 — Symbol Analysis Strategy

> **RFC**: RFC-028
> **Domain Lead**: domain-context Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-028, RFC-027, RFC-029
> **相关 Module Spec**: domain-development-spec.md
> **相关 PoC**: POC-025

---

## 目标(Goals)

1. MVP:File-level + Basic Symbol Detection(Tree-sitter)
2. V1:Symbol-level Index(基于 Language Server 协议)
3. 多语言支持(MVP 至少 3 种:Rust / TypeScript / Python)
4. Symbol 识别准确率:MVP 80% / V1 >95%
5. Symbol-level Feedback 支撑(§25.1)
6. Symbol-level Conflict Detection 支撑(§22.4)
7. 避免 Graph Database(§30.6)

## 非目标(Non-Goals)

1. ❌ 完整 IDE Compiler Database(MVP 不现实)
2. ❌ Graph Database 表达 Symbol 关系(§30.6)
3. ❌ 跨文件类型推断(MVP,V1 Language Server 完整)
4. ❌ 100+ 语言支持(MVP 3 种主流,V1 扩展)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-context Lead** | Symbol Index 整体策略 / Tree-sitter 集成 | ❌ |
| **Symbol Analysis Tech Lead** | Language Server 协议集成 / 多语言适配(V1) | ❌(独立于 domain-context) |
| **domain-development Lead** | ChangeSet Symbol 提取 | ❌ |
| **domain-worktree Lead** | Symbol-level Conflict Detection | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-4)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SYM-001** | Tree-sitter 集成(Rust / TypeScript / Python 3 种 grammar) | domain-context | RFC-028 | 500K | 3 种语言 grammar 集成 |
| **SYM-002** | File-level Index(`symbol_index_files` 表) | domain-context | SYM-001 | 300K | 文件级索引 |
| **SYM-003** | Basic Symbol Detection(function / class / interface 识别) | domain-context | SYM-001 | 450K | 准确率 80% |
| **SYM-004** | Symbol 索引构建 Pipeline(Git commit 后异步) | domain-context | SYM-002 | 350K | 1k files < 1s |
| **SYM-005** | Symbol 反查 API `GET /symbols?path=...&line=...` | domain-context | SYM-002 | 300K | P95 < 50ms |
| **SYM-006** | ChangeSet Symbol 提取集成(简化版) | domain-development + domain-context | SYM-003 | 350K | ChangeSet 7 类元数据补全 |
| **SYM-007** | Symbol 索引 Lifecycle Policy(>90d 归档,§5.8) | SRE | SYM-002 | 250K | 归档脚本 + 监控 |
| **SYM-008** | Symbol 索引 CI Gate(增量构建测试) | domain-context + SRE | SYM-004 | 250K | 1k files 索引 P95 < 1s |
| **SYM-009** | Tree-sitter Grammar 维护流程(MVP 3 种语言) | domain-context | SYM-001 | 200K | Grammar 升级流程文档化 |

**Phase 1 合计**:约 **2.95M tokens**

### Phase 2 (V1,Week 5-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **SYM-101** | Language Server 协议集成(Rust Analyzer) | Symbol Analysis Tech | SYM-001 | 800K | Rust 准确率 > 95% |
| **SYM-102** | Language Server 协议集成(TS Server) | Symbol Analysis Tech | SYM-101 | 700K | TypeScript 准确率 > 95% |
| **SYM-103** | Language Server 协议集成(Python Language Server) | Symbol Analysis Tech | SYM-101 | 700K | Python 准确率 > 95% |
| **SYM-104** | 跨文件引用 / 类型推断(基于 Language Server) | Symbol Analysis Tech | SYM-101 | 600K | 跨文件 Symbol 查询 |
| **SYM-105** | Symbol-level Feedback 完整版(§25.1) | domain-feedback + domain-context | SYM-101 | 500K | POC-025 完整验证 |
| **SYM-106** | Symbol-level Conflict Detection(§22.4) | domain-worktree | SYM-101 | 500K | 跨 Worktree Symbol 冲突 |
| **SYM-107** | 语义 Symbol Search(基于 Embedding) | domain-context | SYM-101 | 600K | 语义搜索 |
| **SYM-108** | Tree-sitter 升级到 Language Server 渐进式策略 | domain-context | SYM-101 | 300K | 平滑过渡 |

**Phase 2 合计**:约 **4.7M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **SYM-201** | 更多语言(Go / Java / C++ / Kotlin) | 1.5M |
| **SYM-202** | Semantic Search 优化 | 800K |
| **SYM-203** | AI 辅助 Symbol 识别 | 600K |

**Phase 3 合计**:约 **2.9M tokens**

---

## 依赖矩阵

```
RFC-028 依赖:
  - 无(基础设施层)

RFC-028 被依赖:
  - RFC-027 (ChangeSet Symbol 提取)
  - RFC-029 (Symbol-level Conflict)
  - RFC-023 (Symbol-level Feedback)
  - RFC-024 (Context Compiler Symbol 加载)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Symbol 识别准确率不足 | Medium | POC-025 验证;Fallback 为 File-level;UI 提示"可能不准确" |
| Tree-sitter Grammar 缺失 | Low | MVP 选 3 种主流语言;V1 扩展 |
| Language Server 集成成本 | Medium | V1 POC 提前验证;选 1-2 种语言试点 |
| 跨文件引用不准确 | Low | MVP 不支持跨文件;V1 Language Server 完整支持 |
| Symbol Index 存储增长 | Low | Lifecycle Policy >90d 归档;按需重建 |

## 验收标准(MVP)

1. ✅ Tree-sitter 集成(MVP 3 种语言:Rust / TypeScript / Python)
2. ✅ File-level Index 实现
3. ✅ Basic Symbol Detection(function / class / interface)准确率 80%
4. ✅ Symbol 反查 API P95 < 50ms
5. ✅ Symbol 索引 1k files < 1s
6. ✅ ChangeSet Symbol 提取集成
7. ✅ Lifecycle Policy >90d 归档
8. ✅ Tree-sitter Grammar 维护流程
9. ✅ Symbol 索引 CI Gate
10. ✅ 避免 Graph Database(§30.6)

## Token-OLU 总览

- **Phase 1(MVP)**:2.95M tokens ≈ 10-30 人·天
- **Phase 2(V1)**:4.7M tokens(Language Server 集成成本高)
- **Phase 3(V2)**:2.9M tokens
- **MVP + V1**:7.65M tokens(由 domain-context Lead + Symbol Analysis Tech Lead 2 人 16-20 周完成,Language Server 集成是主要工作量)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
