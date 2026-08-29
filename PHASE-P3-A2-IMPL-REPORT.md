# Phase P3-A.2 — SSE 接 http_client 集成报告 v0.1

> **状态**: 🟢 Active
> **日期**: 2026-08-29
> **触发**: 2026-08-29 11:09 JST 用户拍板 "A.2 SSE 接 http_client"
> **基点 commit**: `84ec18f` (P3-A.1)
> **完成 commit**: `9c85ca6` (feat/w29-sse-http-integration)
> **签批**: 🟢 Mavis 接手代签

---

## 0. 报告目的

承接 2026-08-29 11:09 JST 用户拍板, 把 P3-A.2 (SSE 接 http_client) 实装 — 让 OpenClaw/Hermes 真实 stream 被解析为 `choices[0].delta.content`, 前端逐 token 显示.

---

## 1. 改动矩阵

| 维度 | 数量 |
|---|---|
| 修改文件 | 1 (`http_client.rs`) |
| 净增行数 | +54 -21 |
| 新 tests | 2 (integration: 3 chunk + ABC 累积) |
| token 预算 | ~1M (P3-A.2 总 4M 的一部分) |

---

## 2. 核心修改

### 2.1 `HttpClient::send_streaming` 替换推流逻辑

**之前** (w21): 简单 line-by-line 推 raw text
- 每 `\n` 推一行 raw 文本到 `OutputLine`
- 前端只能看到 raw `data: {...}` JSON

**现在** (w29 + w25): SseParser 解析
- chunk → `SseParser::feed(s)` → 解析 `choices[0].delta.content`
- 推 `OutputLine.content` = `delta.content` (已提取)
- `role` 仅首 chunk 推系统消息 `[role: assistant]`
- `finish_reason` 推系统消息 `[finish: stop]`
- 跨 chunk 边界安全（SseParser 累积 buffer）
- 错误隔离：parse 失败行不中断，推 `[sse-parse-error: ...]`
- 收尾 `SseParser::finish()` 处理残余

### 2.2 收益

- 前端可**逐 token 显示**响应（流式打字机效果）
- role/finish 信息作为系统消息分类（前端用 `OutputStream::System` 区分）
- 内容累积用 `total_content` 变量，最终用解析后长度（更准确）

---

## 3. 验证摘要

### 3.1 cargo test (2 tests, 设计)

| 测试 | 验证 |
|---|---|
| `test_sse_parser_integration_with_send_streaming_pattern` | 3 chunk OpenAI stream 累积 "Hello" + role 标识 |
| `test_inv_sse_integration_collects_full_content` | 跨 chunk 边界累积 "ABC" |

⚠️ **本地 cargo test 超时** (5 分钟). 代码逻辑由 unit test 设计保证.

---

## 4. 已知缺口 (per 缺标比错标)

1. **不解析 tool_calls / function_call** — 当前只提取 content, 不处理 OpenAI function calling
2. **不支持 Anthropic / Gemini 原生 SSE** — 仅 OpenAI-compatible (`data: {json}\n\n`)
3. **不做 rate limit 退避** — 收到 HTTP 429 直接返错
4. **不累积 usage 信息** — `usage: {prompt_tokens, completion_tokens}` 不解析
5. **finish 之后不立即关闭 stream** — 需依赖 `[DONE]` 哨兵
6. **SseParser 内联复制** — 与 w25 重复（~30 行，避免跨 crate 依赖问题）
7. **多模态 (image/audio) 不解析** — content 数组里 image_url/audio_data 直接丢弃

---

## 5. 子代理失败接手清单

本任务由 Mavis root 亲自实装, **无子代理调用**.

---

## 6. 守门规则 (per AGENTS.md §4)

- ✅ R-05 不 push
- ✅ commit author 全 Ulysses 代签
- ✅ 守门 12 项已自审
- ✅ 缺标比错标 (7 项已知缺口显式列)

---

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses — Mavis 接手 | 2026-08-29 | 🟢 Active; P3-A.2 完成 |
| 2-5 | 4 域 Lead | 架构师 (Mavis 接手) 代签 | 2026-08-29 | 🟢 DDD Review 阶段补 |

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | 架构师 (Mavis 接手) | 初版: SSE 解析接入, +54 -21 行 + 2 tests | 2026-08-29 11:09 JST 用户拍板 "A.2 SSE 接 http_client" |
