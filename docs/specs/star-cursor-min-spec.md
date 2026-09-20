# Spec: Star Cursor 极简版 v1 (ULYS-98-W4.6 收口)

> Status: DRAFT v0.1
> Companion to docs/rfc/star-cursor-min-v1.md
> Date: 2026-09-20

## 1. 接口契约 (Acceptance Criteria)

### 1.1 Chat API

AC-CHAT-001: POST /v1/chat/send
- Req: { session_id: UUID, user_id: UUID, content: string (non-empty), model?: string }
- Resp 200: { session_id, message_id, assistant_content, model, created_at, usage?: { input_tokens, output_tokens, total_tokens } }
- Validation: content empty -> 400 VALIDATION_FAILED
- Provider call: DispatchProvider.chat_completion (mock fallback if no API key)

AC-CHAT-002: GET /v1/chat/stream?session_id=...&user_id=...&content=...&model=...
- Content-Type: text/event-stream
- Each event: data: { "delta": "...", "done": false, "usage": null }
- Terminal: data: { "delta": "", "done": true, "usage": {input,output,total} }
- Keep-Alive: 15s interval
- Error: data: { "delta": "", "done": true, "error": "..." }

AC-CHAT-003: GET /v1/chat/messages?session_id=...
- Resp 200: { session_id, count, messages: ChatMessage[] }
- Unknown session -> 404 RESOURCE_NOT_FOUND

### 1.2 Completion API

AC-COMP-001: POST /v1/completion/inline
- Req: { file_path, cursor_line, cursor_col, prefix (8KB max), suffix (8KB max), language }
- Resp 200: { completion: string, token_usage?: {...} }

### 1.3 Composer API

AC-COMP-002: POST /v1/composer/edit
- Req: { files: [{ path, content, selection?: { start, end } }], instruction: string (non-empty), user_id, model?, session_id? }
- Resp 200: { session_id, edits: FileEditOut[], model }
- FileEditOut: { path, diff, new_content }
- Validation: empty files -> 400, empty instruction -> 400

AC-COMP-003: POST /v1/composer/apply
- Req: { session_id, paths: string[], user_id }
- Resp 200: { applied_paths: string[], skipped_paths: string[] }

### 1.4 Agent API

AC-AGT-001: POST /v1/agent/run (W4.3 stub)
- Req: { session_id, user_id, prompt, model?, max_steps? }
- Resp 200: { session_id, steps: AiStep[], final_answer: string? }
- AiStep: { step_id, thought, tool_call?, observation? }

### 1.5 Sandbox (W4.1)

AC-SBX-001: agent_bridge::sandbox::DENY_LIST 必须包含 10 项: rm -rf /, rm -rf /*, mkfs, dd if=, shutdown, reboot, git push --force*, DROP DATABASE/TABLE
- 测试: is_denied() 命中所有 10 项, 不命中 ls/cargo/git status

AC-SBX-002: agent_bridge::sandbox::run() 走 tokio::process::Command
- 5 测试: deny 拒绝 / 空命令拒绝 / echo 成功 / workdir 不存在

### 1.6 Frontend hooks

AC-FE-001: useChatStream 接 EventSource + 累积 delta
- 返回 { text, streaming, error, reset }

AC-FE-002: ComposerPanel 调用 /v1/composer/edit + /v1/composer/apply
- 返回 { edits, error, submit, apply, reject, accepted, toggleAccept }

AC-FE-003: AgentRunner 调用 /v1/agent/run + 实时展示 steps
- 返回 { steps, final_answer, busy, error, run, reset }

## 2. 数据模型约束

- ChatSession.id: UUID v4
- ChatMessage.role: enum {User, Assistant, System}
- ChatChunk.delta: string (UTF-8, may be empty on terminal)
- AiStep.thought: string (provider response content)
- AiStep.tool_call: Option (forward declared; v0.0.1 always None)

## 3. 不变量

INV-CURSOR-001: WebSocket 路径与 TCP 路径共享 ARC-022 mTLS 基线 (per RGS-BAS-006 §3.3)
INV-CURSOR-002: mock provider 用于 CI 无 key 环境 (守门 #25 v25)
INV-CURSOR-003: 0 unsafe blocks (workspace lint)
INV-CURSOR-004: 全 dep 来自 [workspace.dependencies] (守门 #11 缺标比错标)
INV-CURSOR-005: audit log every ChatMessage / Action / ToolCall (守门 #13 d)
