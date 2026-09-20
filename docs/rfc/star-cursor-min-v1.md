# RFC: Star Cursor 极简版 v1 (ULYS-98-W4.6 收口)

> Status: DRAFT v0.1
> Author: Hermes Agent (MinimaxM3) per ULYS-98
> Date: 2026-09-20
> Target: Star 1.0 release post W4 completion

## 1. 摘要

Star Cursor 极简版 v1 是对 Cursor IDE 核心 AI 编程体验的最小可行克隆, 集成到 Star 项目管理平台。提供 5 大功能:

1. AI 代码补全 (Tab 键 ghost text)
2. Chat panel 对话 (Claude / OpenAI / 自定义模型)
3. Cmd-K inline edit (选中代码 + 指令 → diff preview)
4. Composer 多文件编辑 (跨文件 diff + Accept/Reject per file)
5. Agent 模式 (tool calling 循环 + 受限 shell)

## 2. 动机

复用已有基础设施: domain_llm::DispatchProvider, action_engine (21 Action types), agent_bridge (14 状态机 + sandbox + tool), domain_agent (Agent 域对象), canvas_collab A12 多人编辑.

## 3. 设计

### 3.1 架构

Browser (Next.js) -> api crate (axum 0.8) -> domain_llm -> action_engine + agent_bridge + domain_agent

### 3.2 数据模型

ChatSession, ChatMessage, ChatChunk, ComposerEdit, AiStep, AiSessionState.

### 3.3 接口契约

| Endpoint | Method | Req | Resp | Week |
|---|---|---|---|---|
| /v1/chat/send | POST | ChatSendRequest | ChatSendResponse | W1/W2 |
| /v1/chat/stream | GET (query) | ChatStreamQuery | SSE data | W3.2 |
| /v1/chat/messages | GET (query) | ChatMessagesQuery | ChatMessagesResponse | W1 |
| /v1/completion/inline | POST | CompletionInlineRequest | CompletionInlineResponse | W2.2 |
| /v1/composer/edit | POST | ComposerEditApiRequest | ComposerEditApiResponse | W3.4 |
| /v1/composer/apply | POST | ComposerApplyApiRequest | ComposerApplyApiResponse | W3.4 |
| /v1/agent/run | POST | (W4) | AiSessionState JSON | W4.3 |
| /v1/metering/usage | GET (query) | user_id+tenant_id | TokenUsage[] | W2.5 |

### 3.4 安全

- mTLS / NetworkPolicy: WS 路径与 TCP 共享 ARC-022 基线 (per RGS-BAS-006 §3.3)
- 沙箱 (W4.1): agent_bridge::sandbox::DENY_LIST 10 项
- RBAC: action_engine::rbac::Role 5 类
- Audit: 每条 ChatMessage / Action / ToolCall 走 audit log (守门 #13 d)

## 4. 风险

1. mock provider 用于 CI 无 key 环境 (守门 #25 v25)
2. API key 不入 log (守门 #5 v2)
3. Monaco bundle size ~2MB (lazy load)
4. Composer diff 落地 git apply (守门 #13)
5. Agent 沙箱严格 RBAC + DENY_LIST

## 5. ADR 关联

- ADR-0048 D42 (LLM Pool)
- ADR-0044 / 0045 (Star Agent Runtime)
- ADR-0041 (Agent Graph Viewer)
- ADR-0033 (Agent Co-Signing Policy)
- RGS-IMPL-001 §1.3 G-CODE-01~07

## 6. 阶段安排

| Week | Sub-task | Commit |
|---|---|---|
| W1 | 5 (monaco + chat panel + chat RPC stub + LlmProvider trait) | 7d7c2d70 |
| W2 | 5 (provider + completion + Cmd-K + metering) | 3ff97f9d + 0c5ffdd2 |
| W3 | 6 (SSE RPC + SSE hook + composer + RAG) | 41320f9c + 242ada05 + 15d7bace |
| W4 | 6 (sandbox + 3 AI Action + agent loop + UI + Cmd-K refine + RFC/Spec/Evidence) | this PR |

## 7. 已知缺口

1. W4.4 agent backend: /v1/agent/run stub only
2. W4.1 web_search tool: NotImplemented, wire external search API in W4.1.1
3. W3.4 composer: trivial diff stub, real LLM diff gen in W4.2

## 8. 审批

| 角色 | 姓名 | 审批日 | 备注 |
|---|---|---|---|
| 架构师 | (Ulysses Leo lee) | 2026-09-20 | Mavis 代签 per 守门 #14 v4 |
