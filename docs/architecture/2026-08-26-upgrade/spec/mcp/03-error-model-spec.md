# 16. STAR MCP Error Model Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/agent-api/01-schema.md §3.14 Error](../agent-api/01-schema.md) · [spec/mcp/01-mcp-spec.md §3.2](./01-mcp-spec.md) · [spec/rest/01 §4](../rest/01-rest-strategy.md) · [spec/flows/05 §3](../flows/05-universal-submit.md)

## 0. 目的

定义 STAR MCP server 错误响应的 **6 字段 Error schema**（与 agent-api/v1 §3.14 唯一权威对齐）、**标准错误码**、**retriable 矩阵**、**HTTP 状态码映射**。CLI / MCP / REST / Universal Submit 4 处共用同一 schema（per agent-api/v1 §3.14 + F-06 修复 2026-08-27），本 spec 是 MCP 层的细化。

## 1. 6 字段定义（per agent-api/v1 §3.14 + F-06 重定义 2026-08-27）

> **唯一权威**：本节 6 字段定义**完全引用** [agent-api/01-schema.md §3.14](../agent-api/01-schema.md)。任何 spec / 代码改动**必须**先改 §3.14，再传播。

| # | 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|---|
| 1 | `code` | string (SCREAMING_SNAKE_CASE) | ✅ | 标准化错误码（见 §2）|
| 2 | `message` | string | ✅ | 人类可读消息 |
| 3 | `source_module` | enum string | ✅ | `agent-core` \| `ide-gateway` \| `vcs` \| `policy` \| `mcp` \| `rest` \| `cli`（per agent-api/v1 §3.14） |
| 4 | `source_kind` | enum string | ✅ | `internal` \| `external` \| `policy` \| `validation` \| `user_input` \| `timeout`（per agent-api/v1 §3.14） |
| 5 | `retriable` | boolean | ✅ | 是否可重试（见 §3 矩阵） |
| 6 | `hint` | string? | ❌ | 恢复提示（替换 v0.2 的 `suggested_actions[]`） |

> **v0.2 字段已弃用（per F-06 修复 2026-08-27）**：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details` 6 字段已弃用。**新代码必须用 F-06 重定义后的 6 字段**。

### 1.1 JSON-RPC 2.0 envelope 映射（per mcp/01 §3.2 + P1-G）

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Worktree wt-STAR-1024 has uncommitted changes",
    "data": {
      "code": "WORKTREE_CONFLICT",
      "message": "Worktree wt-STAR-1024 has uncommitted changes conflicting with main",
      "source_module": "vcs",
      "source_kind": "external",
      "retriable": true,
      "hint": "Run `star worktree inspect wt-STAR-1024` then `request_rebase` or commit changes"
    }
  }
}
```

> **JSON-RPC `error.code` = -32000**（per JSON-RPC 2.0 spec 自定义错误码区间 -32000 ~ -32099）；STAR 自定义错误码放 `data.code`（SCREAMING_SNAKE_CASE），**不**用 JSON-RPC `error.code` 表达。

## 2. 标准错误码（30 个，SCREAMING_SNAKE_CASE）

按 `source_module` 分组（30 个 = 5 vcs + 5 policy + 5 agent-core + 4 ide-gateway + 5 mcp + 6 跨域）：

| code | source_module | retriable | source_kind | 触发场景 |
|---|---|---|---|---|
| `WORKTREE_CONFLICT` | vcs | true | external | worktree 与 main 冲突（per mcp/01 §3.2 示例） |
| `BRANCH_NOT_FOUND` | vcs | false | external | 指定 branch 不存在 |
| `COMMIT_FAILED` | vcs | true | external | git commit 失败（如 hook 拒绝） |
| `PUSH_REJECTED` | vcs | true | external | `git push` 被远端拒绝 |
| `MERGE_CONFLICT` | vcs | false | user_input | MR 合并冲突（需用户解冲突） |
| `POLICY_DENIED` | policy | false | policy | policy engine 拒绝（如 5 域 Lead 越权 per 8/21 JST） |
| `PERMISSION_INSUFFICIENT` | policy | false | policy | agent 权限不足（如 peer 读 agent://{self}） |
| `LEASE_EXPIRED` | policy | true | policy | worktree lease 过期（per ADR-0030） |
| `RATE_LIMITED` | policy | true | policy | 限流触发 |
| `AUDIT_REQUIRED` | policy | false | policy | 操作需审计但 audit log 不可写 |
| `AGENT_TIMEOUT` | agent-core | true | timeout | agent 执行超时 |
| `AGENT_CRASHED` | agent-core | true | internal | agent 进程崩溃（per Resume §3.16 重连） |
| `CONTEXT_OVERFLOW` | agent-core | false | internal | 上下文窗口超限 |
| `STATE_INVALID` | agent-core | false | validation | agent 状态非法转换 |
| `HEARTBEAT_LOST` | agent-core | true | timeout | agent lease heartbeat 丢失（per ADR-0030） |
| `IDE_SESSION_INVALID` | ide-gateway | false | external | IDE session 已过期（per ADR-0024） |
| `IDE_CAPABILITY_MISSING` | ide-gateway | false | external | IDE 能力缺失（per arch/03 §4.3 Fallback Ladder） |
| `IDE_VERSION_INCOMPATIBLE` | ide-gateway | false | external | IDE 版本不兼容（per ADR-0025 反污染） |
| `IDE_BRIDGE_DOWN` | ide-gateway | true | external | IDE bridge 进程挂掉 |
| `TOOL_NOT_FOUND` | mcp | false | validation | 调用未知 tool name |
| `VALIDATION_FAILED` | mcp | false | validation | tool input schema 校验失败 |
| `RESOURCE_NOT_FOUND` | mcp | false | validation | resources/read 找不到（per 02 §5.3） |
| `PROMPT_RENDER_FAILED` | mcp | true | internal | prompts/get 渲染失败（per 02 §5.3） |
| `NOT_IMPLEMENTED` | mcp | false | internal | 调 MVP 不实现能力（per F-28 / 02 §5.3） |
| `INTERNAL_ERROR` | mcp | true | internal | 未分类 server 错误 |
| `INVALID_JSON` | mcp | false | validation | JSON-RPC 请求体 JSON 解析失败 |
| `INVALID_REQUEST` | mcp | false | validation | JSON-RPC 协议字段缺失 / 类型错 |
| `MIME_MISMATCH` | mcp | false | validation | resources content_type 不匹配（per 02 §1.6） |
| `CONFIG_INVALID` | mcp | false | validation | server 配置错误 |
| `DEPENDENCY_DOWN` | mcp | true | external | 依赖服务不可用（DB / VCS / IDE bridge） |

> **未列出的错误码禁止使用**（per F-06 唯一权威 + 防止 sprawl）；新错误码需先在 agent-api/v1 §3.14 落盘再传播。

## 3. retriable 矩阵

按 `source_kind` 维度决定重试策略：

| source_kind | retriable 默认 | 例外 | 用户介入 |
|---|---|---|---|
| `internal` | true | `STATE_INVALID` / `CONTEXT_OVERFLOW` → false | 看 `hint` |
| `external` | true | `MERGE_CONFLICT` / `BRANCH_NOT_FOUND` → false | 看 `hint` |
| `policy` | **false**（policy 拒绝不重试，重试结果一致） | `LEASE_EXPIRED` / `RATE_LIMITED` → true | 必须用户/Lead 介入 |
| `validation` | **false**（入参错不重试，重试结果一致） | 无 | 必须改 input |
| `user_input` | false | 无 | 必须改 input |
| `timeout` | true | 无 | 看 hint（多半自动重试） |

**用户介入信号**：`retriable=false` AND `source_kind IN (policy, validation, user_input)` —— agent 看到这 3 类组合必须**停止重试**并向用户报告。

> **重试上限**：单一错误最多重试 3 次（per ADR-0030 retry policy）；超过 3 次升级到 `AGENT_CRASHED` 走 Resume 流程（per agent-api/v1 §3.16）。

## 4. 与 HTTP 状态码映射（MCP Streamable HTTP Phase 2+）

| HTTP status | 适用错误码 | 语义 |
|---|---|---|
| `400 Bad Request` | `VALIDATION_FAILED` / `INVALID_JSON` / `INVALID_REQUEST` | 入参错 |
| `401 Unauthorized` | `PERMISSION_INSUFFICIENT` | 缺认证 |
| `403 Forbidden` | `POLICY_DENIED` / `AUDIT_REQUIRED` | 认证通过但 policy 拒 |
| `404 Not Found` | `TOOL_NOT_FOUND` / `RESOURCE_NOT_FOUND` / `BRANCH_NOT_FOUND` | 资源不存在 |
| `409 Conflict` | `WORKTREE_CONFLICT` / `MERGE_CONFLICT` | 状态冲突 |
| `422 Unprocessable Entity` | `STATE_INVALID` / `CONTEXT_OVERFLOW` | 语义错（schema 对但语义不行） |
| `429 Too Many Requests` | `RATE_LIMITED` | 限流 |
| `500 Internal Server Error` | `INTERNAL_ERROR` / `AGENT_CRASHED` | server bug |
| `501 Not Implemented` | `NOT_IMPLEMENTED` | 能力未实装 |
| `502 Bad Gateway` | `DEPENDENCY_DOWN` / `IDE_BRIDGE_DOWN` | 下游挂 |
| `503 Service Unavailable` | `LEASE_EXPIRED` / `HEARTBEAT_LOST` / `AGENT_TIMEOUT` | 暂时不可用（可重试） |
| `504 Gateway Timeout` | `AGENT_TIMEOUT` / `IDE_SESSION_INVALID` | 超时 |

> **Streamable HTTP 状态守门（per F-22）**：MVP 仅 stdio transport，HTTP 状态码映射是 **Phase 2+**；本表用于 spec 阶段对齐，**不**进 MVP 退出条件。

## 5. 与 agent-api/v1 §3.14 对齐

**本 spec 6 字段定义 = 100% 引用** [agent-api/01-schema.md §3.14](../agent-api/01-schema.md)（per F-06 修复 2026-08-27）。任何字段语义冲突以 §3.14 为准。

- **CLI 端**：引用 [spec/cli/01 §5](../cli/01-cli-spec.md) Error 章节（待 F-06 传播）
- **REST 端**：引用 [spec/rest/01 §4](../rest/01-rest-strategy.md) Error 章节（待 F-06 传播）
- **Universal Submit**：引用 [spec/flows/05 §3](../flows/05-universal-submit.md) Error 章节（待 F-06 传播）
- **mcp/01 §3.2 旧字段**：v0.2 fix 已加 F-06 引用注释，但**未**重写 JSON 示例（per mcp/01 §3.2 fix 说明"引用约定"，**不**重新定义）

> **传播风险（per F-21 强化交叉引用）**：本 spec 落地后，cli/01 / rest/01 / flows/05 4 处 spec 需在 v0.X 修订中**逐个**显式引用 §3.14 6 字段，**不**保留 v0.2 的 6 字段（`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`）。该传播由后续子代理在 cli/rest/flows 域 fix 时执行。

## 6. 已知缺口

1. **HTTP 状态码映射未实装验证**：§4 表是 spec 推导，**未**经 Streamable HTTP 实测（Phase 2+ 才有 streamable-http，per mcp/01 §6 + F-22）；Phase 2 启动时需逐码用 curl 验证 30 个错误码对应 HTTP status。
2. **30 个错误码粒度可能不够**：例如 `POLICY_DENIED` 可能需细化为 `POLICY_DENIED_LEASE` / `POLICY_DENIED_CROSS_WORKTREE` 等子码；当前 30 个是 MVP 基线，Phase 2+ 评估是否扩到 50+。
3. **retry 退避策略未定义**：§3 矩阵只回答 "可不可重试"，**不**回答 "间隔多久 / 几次后放弃"；具体 backoff（指数 / 固定 / jitter）依赖 `spec/policy/01-policy-engine.md`（尚未创建，Phase 2+ 落地）。
4. **`trace_id` 字段未在 6 字段内**：v0.2 有 `trace_id`，F-06 重定义时**移除**（per agent-api/v1 §3.14 注释 "v0.2 的 trace_id 已弃用"）；Phase 2+ 评估是否通过 `details` 扩展或新增 `trace_id` 字段。
5. **错误码跨语言映射**：本 spec 错误码为 SCREAMING_SNAKE_CASE 字符串，**不**含数字前缀；客户端用 string match 不用 integer match。Phase 2+ 评估是否加 `http_status` 数字字段便于 HTTP 客户端路由。
6. **与 mcp/01 §3.2 fix 后内容未对齐**：本 spec 写于 2026-08-27 19:42 JST，mcp/01 v0.2 fix 注释说"§3.2 错误模型明确引用 agent-api/01 §3.15 Error"但实际 §3.14；本 spec 用**真实位置 §3.14**，mcp/01 fix 注释的 §3.15 编号是 W4 初稿残留（per mcp/01 §3.2 fix 注释段 "W4 子代理修复时的初稿编号"）。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Mavis（接手 agent per DEC-008）| 初版：6 字段定义（引用 agent-api/v1 §3.14）+ 30 个 SCREAMING_SNAKE_CASE 标准错误码（5 vcs + 5 policy + 5 agent-core + 4 ide-gateway + 5 mcp + 6 跨域）+ retriable 矩阵 + 12 个 HTTP status 映射 + 4 处传播引用 + 6 项已知缺口 | Phase E spec 子代理任务（per parent 19:39 JST 代签授权）|
