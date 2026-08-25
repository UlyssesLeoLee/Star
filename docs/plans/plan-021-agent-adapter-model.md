# Implementation Plan: PLAN-021 — Agent Adapter Model

> **RFC**: RFC-021
> **Domain Lead**: domain-agent Lead
> **状态**: Planned
> **创建日期**: 2026-08-25
> **最后更新**: 2026-08-25
> **相关 RFC**: RFC-021, RFC-026, RFC-030
> **相关 Module Spec**: domain-agent-spec.md
> **相关 PoC**: POC-028, POC-029

---

## 目标(Goals)

1. Domain 层通过 `AgentPort` trait 抽象 Agent 操作
2. infrastructure 层实现 Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local / Future Adapter
3. AgentPolicy 跨厂商统一(12 个强制点,§4.2.5)
4. Domain 层零厂商依赖(禁止 `CodexTool` / `ClaudeCodeEvent`)
5. Mock Adapter 支持 Domain 层单元测试
6. 缓解 RISK-030 Agent Vendor Lock-in

## 非目标(Non-Goals)

1. ❌ MCP(Model Context Protocol)集成(V2 候选)
2. ❌ Multi-Agent Comparison(V2)
3. ❌ Agent 厂商完整能力(每个 Adapter 仅实现 MVP 必须能力)
4. ❌ 厂商 SDK 完整封装(仅封装 Domain 层需要的方法)

---

## Owner 矩阵

| Owner 角色 | 负责内容 | 不兼任 |
|---|---|---|
| **domain-agent Lead** | AgentPort trait 设计 / Mock Adapter / Contract Testing | ❌ |
| **Agent Adapter Tech Lead** | Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local Adapter 实现 | ❌(独立于 domain-agent) |
| **domain-permission Lead** | AgentPolicy 12 强制点在 Adapter 集成 | ❌ |
| **SRE Lead** | Adapter 集成测试 / 性能监控 | ❌ |

---

## 阶段划分

### Phase 1 (MVP,Week 1-5)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **AGT-001** | `AgentPort` trait 设计(start / submit_feedback / stop / query_status 4 个方法) | domain-agent | RFC-021 | 300K | Port trait 签名冻结 |
| **AGT-002** | `AgentError` 错误类型 + 错误码体系 | domain-agent | AGT-001 | 200K | 错误分类清晰 |
| **AGT-003** | `Agent` 注册表 + `AgentPolicy` 值对象 | domain-agent | AGT-001 | 350K | 12 强制点字段完整 |
| **AGT-004** | `AgentSession` 聚合根实现(14 状态机) | domain-agent | RFC-026 | 450K | 14 状态机单元测试 100% |
| **AGT-005** | Mock Adapter 实现(测试用) | domain-agent | AGT-001 | 250K | Domain 层单元测试可独立运行 |
| **AGT-006** | Contract Testing 框架(Port 行为契约) | domain-agent + SRE | AGT-001 | 300K | 任何 Adapter 必须通过 Contract Test |
| **AGT-007** | Codex Adapter 实现(优先厂商) | Agent Adapter Tech | AGT-001,004 | 800K | POC-028 验证 AgentSession 完整生命周期 |
| **AGT-008** | Claude Code Adapter 实现(备选厂商) | Agent Adapter Tech | AGT-007 | 800K | 同上,作为备选 |
| **AGT-009** | AgentPolicy 12 强制点集成(Repository / Worktree / Path / Tool / Network / Secret / Runtime / Context / Change Scope / Review / Test / Approval) | domain-permission | AGT-003 | 600K | POC-029 验证越权 Path / Tool / Network / Secret 全部拦截 |
| **AGT-010** | Adapter 单元测试覆盖率 > 80% | Agent Adapter Tech + SRE | AGT-007,008 | 300K | CI 强制 |
| **AGT-011** | Agent Vendor Lock-in 监控(metric) | SRE | AGT-007,008 | 200K | Agent Vendor 数量 / Adapter 复用率 |

**Phase 1 合计**:约 **4.55M tokens**

### Phase 2 (V1,Week 6-10)

| Task ID | 任务 | 负责 Lead | 依赖 | Token 估算 | 验收 |
|---|---|---|---|---:|---|
| **AGT-101** | Gemini CLI Adapter 实现 | Agent Adapter Tech | AGT-007 | 700K | 厂商扩展性验证 |
| **AGT-102** | OpenAI Compatible Adapter | Agent Adapter Tech | AGT-007 | 700K | 任何兼容 OpenAI API 的厂商可集成 |
| **AGT-103** | Local Adapter(Ollama / LM Studio) | Agent Adapter Tech | AGT-007 | 700K | 本地 LLM 场景支持 |
| **AGT-104** | Agent Handoff 跨厂商支持(§24.5) | domain-agent | AGT-004 | 500K | HandoffContextPacket 跨厂商传递 |
| **AGT-105** | §46 决策表 J.5 V1 中期复审(Port 抽象是否过度) | domain-agent + Architect | AGT-001 | 200K | 决策记录 |
| **AGT-106** | Policy 模板库 | domain-permission | AGT-003 | 400K | Project / Tenant Policy 模板 |

**Phase 2 合计**:约 **3.2M tokens**

### Phase 3 (V2,Week 11+)

| Task ID | 任务 | Token 估算 |
|---|---|---:|
| **AGT-201** | MCP(Model Context Protocol)集成 | 1.2M |
| **AGT-202** | Multi-Agent Comparison UI | 800K |
| **AGT-203** | Agent Performance Analytics | 600K |
| **AGT-204** | AI 辅助 Agent 选择(根据 Task 特征选最适厂商) | 1.0M |

**Phase 3 合计**:约 **3.6M tokens**

---

## 依赖矩阵

```
RFC-021 依赖:
  - 无(基础设施层抽象)

RFC-021 被依赖:
  - RFC-026 (AgentSession 持久化)
  - RFC-030 (Agent Policy 强制)
  - RFC-024 (Context Compiler 调用 Agent)
  - RFC-016 (Worktree 启动 Agent)
```

## 风险与缓解

| 风险 | 影响 | 缓解 |
|---|---|---|
| Agent Vendor Lock-in | Medium | Port 抽象(本 RFC);Policy 跨厂商统一;监控 |
| Port 抽象过度设计 | Medium | §46 J.5 V1 复审;YAGNI 原则;Domain Port 最小化 |
| 厂商 SDK 协议变化 | High | Adapter 隔离 SDK 变化;版本锁定 + 升级测试 |
| 协议差异补偿复杂 | Medium | ACL 模式;Adapter 内部处理差异 |
| Mock Adapter 覆盖不足 | Low | Contract Testing;CI 强制覆盖率 > 80% |

## 验收标准(MVP)

1. ✅ `AgentPort` trait 4 个方法(start / submit_feedback / stop / query_status)
2. ✅ Domain 层零厂商类型
3. ✅ 至少 1 个 Adapter(Codex 或 Claude Code)
4. ✅ Mock Adapter 完整(Domain 层单元测试可独立运行)
5. ✅ AgentPolicy 12 强制点全部生效
6. ✅ Contract Testing 覆盖所有 Adapter
7. ✅ Adapter 单元测试覆盖率 > 80%
8. ✅ POC-028 验证 AgentSession 完整生命周期
9. ✅ POC-029 验证 Policy 强制点全部生效
10. ✅ Vendor Lock-in 监控指标设置

## Token-OLU 总览

- **Phase 1(MVP)**:4.55M tokens ≈ 15-45 人·天
- **Phase 2(V1)**:3.2M tokens
- **Phase 3(V2)**:3.6M tokens
- **MVP + V1**:7.75M tokens(由 domain-agent Lead + Agent Adapter Tech Lead 2 人 16-20 周完成)

## 变更历史

| 日期 | 版本 | 变更 |
|---|---|---|
| 2026-08-25 | v0.1 | 初稿 |
