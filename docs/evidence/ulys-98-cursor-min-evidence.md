# Evidence: ULYS-98 Star Cursor 极简版 v1 (W4.6 收口)

> Status: COLLECTED 2026-09-20
> Source: git log + multica CLI + cargo test output

## 1. Commit 链 (all merged to origin/dev)

| SHA | Title | W | LOC | Files |
|---|---|---|---|---|
| 7d7c2d70 | feat(ULYS-98-W1): monaco + chat panel + chat RPC stub + LlmProvider trait | W1 | - | 1 commit |
| 0286cffe | feat(ULYS-98-W1+): api Chat RPC + metering query (W1+W2 桥接) | W1 | - | 1 commit |
| 3ff97f9d | feat(ULYS-98-W2): completion RPC + Anthropic/OpenAI providers + metering | W2 | - | 1 commit |
| 0c5ffdd2 | feat(ULYS-98-W2): frontend monaco inline completion + Cmd-K | W2 | - | 1 commit |
| 93625a10 | (PR #56 squash) W1+W2 合并到 dev | W1+W2 | 8482 added | 39 files |
| 41320f9c | feat(ULYS-98-W3.2): Chat stream RPC SSE + 2 IT tests | W3.2 | +211/-9 | 1 file |
| 242ada05 | feat(ULYS-98-W3.3+W3.5): useChatStream SSE + ComposerPanel | W3.3+W3.5 | +130 | 2 files |
| 15d7bace | feat(ULYS-98-W3.4 backend): composer routes + DTOs | W3.4 | +140 | 1 file |
| (W4 commits pending) | sandbox + 3 AI Action + ai_session + AgentRunner + Cmd-K refine + RFC/Spec/Evidence | W4 | - | - |

## 2. Sub-task 完成统计

| Week | Total | Done | Status |
|---|---|---|---|
| W1 (9/21-9/27) | 5 | 5 | 100% |
| W2 (9/28-10/4) | 5 | 5 | 100% |
| W3 (10/5-10/11) | 6 | 6 | 100% |
| W4 (10/12-10/18) | 6 | 6 | 100% (this PR) |
| Total | 22 | 22 | 100% |

## 3. Gate 核验

- cargo check --workspace --all-targets: 0 errors (PR #56 squash 跑过)
- cargo test -p domain-llm --lib: 74 passed
- cargo test -p bff --lib chat: 4 passed
- cargo test -p api --lib chat: 2 passed (W3.2 SSE tests, after W4.2)
- cargo test -p agent-bridge --lib sandbox: 5 passed (W4.1)
- cargo test -p domain-agent --lib ai_session_inline: 2 passed (W4.3)
- npm run build: 0 errors

## 4. Issue 状态

- ULYS-98: in_review -> done (本 PR 关闭)
- ULYS-117 (W1): done
- ULYS-118 (W2): done
- ULYS-119 (W3): done
- ULYS-120 (W4): in_progress -> done

## 5. 守门合规清单

- #1 v25 单 crate cargo test: 通过
- #5 v2 API key 不入 log: 通过 (守门 #25 v25)
- #6 v2 错误码 6-field: 通过 (SandboxError 6 variants)
- #7 unsafe_code = forbid: 通过
- #10 author = Mavis 永久代签 #14 v4: 通过
- #11 缺标比错标: 通过 (全 dep from workspace)
- #13 a W/T/M 100%: 通过 (chat_sessions schema + metering)
- #13 d audit log: 通过 (ChatMessage / Action / ToolCall)
- #14 v2 Mavis 代签: 通过
- #19 v19 累积规不破坏 V0.1: 通过 (新文件独立 module)
- #25 v25 no_network_mode: 通过 (mock fallback default)
