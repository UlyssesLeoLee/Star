# 15. STAR MCP Resources + Prompts Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/mcp/01-mcp-spec.md §4-§5](./01-mcp-spec.md) · [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) · [arch/03 §4.3 Fallback Ladder](../../arch/03-star-ai-compat-arch.md) · [ADR-0026 STAR AI Compat](../../adr/0026-star-ai-compat.md)

## 0. 目的

定义 STAR MCP server 的 **Resources**（只读 URI 资源）和 **Prompts**（预置 prompt 模板）规范。本 spec 显式覆盖 [spec/mcp/01-mcp-spec.md §4 Resources](./01-mcp-spec.md) 和 §5 Prompts 中 "MVP 不实现（Level 1-2 不含）" 的占位声明（per F-28 / arch/03 §2.3 v0.3 fix），为 **Phase 2+**（Level 3+ Full MCP 2026-07-28）提供可落地的 URI 命名表 / content type / 权限矩阵和 prompt 模板清单。

> **MVP 范围重申（per F-28 修复 2026-08-27）**：MVP（Level 1-2）不实现 Resources / Prompts，IDE 通过 `tools/call` 即可获得全部 MVP 能力（per arch/03 §2.3 + [acceptance/04 §3](../../acceptance/04-mvp-acceptance.md)）。本 spec 是 Phase 2+ 的**设计预研**，不进 MVP 退出条件。

## 1. Resources 类型

### 1.1 4 类总览

| 类型 | URI scheme | 数量级 | Phase |
|---|---|---|---|
| **workspace** | `workspace://` | 1 per workspace | Phase 2 |
| **worktree** | `worktree://` | N per workspace | Phase 2 |
| **agent** | `agent://` | 1 per active agent session | Phase 2 |
| **decision** | `decision://` | N per workspace（ADR / approval） | Phase 2+ |

### 1.2 workspace 类型

- **URI 模式**：`workspace://<workspace_id>`（per agent-api/v1 §3.15 WorkspaceSummary）
- **content_type**：`application/json`（WorkspaceSummary schema）
- **read 权限**：`agents/{any}` — 所有 agent 可读（聚合视图）
- **write 权限**：❌ 只读，**不**通过 Resources 写（写入走 `update_*` tool）
- **缓存 scope**：`workspace` / `ttlMs=30000`（与 [mcp/01 §2.2 cacheScope](./01-mcp-spec.md) 对齐）
- **典型字段**（per agent-api/v1 §3.15）：`workspace_id` / `name` / `default_branch` / `active_worktrees[]` / `active_agents[]`

### 1.3 worktree 类型

- **URI 模式**：`worktree://<workspace_id>/<worktree_id>`（per agent-api/v1 §3.2 Worktree）
- **content_type**：`application/json`（Worktree schema）
- **read 权限**：
  - `agents/{any}` — 可读 basic（id / branch / status）
  - `agents/{owner}` — 可读全部（含 `worktree_binding[]` per F-24）
- **write 权限**：❌ 只读
- **缓存 scope**：`workspace` / `ttlMs=30000`
- **典型字段**（per agent-api/v1 §3.2）：`worktree_id` / `branch` / `base` / `status` / `worktree_binding[]` / `lease_expires_at`

### 1.4 agent 类型

- **URI 模式**：`agent://<workspace_id>/<agent_id>`（per agent-api/v1 §3.16 Resume）
- **content_type**：`application/json`（Resume schema — 11 字段：id/agent_id/state/last_heartbeat_at/lease_expires_at/current_state/current_step/retry_count/artifacts/checkpoint/recovery_hint，per agent-api/v1 §3.16 v0.2 fix B-19）
- **read 权限**：
  - `agents/{self}` — 读全部
  - `agents/{peer}` — 读脱敏版（去掉 `artifacts` / `checkpoint`，仅留 `id` / `state` / `last_heartbeat_at`）
  - `ide/{owner}` — 读 `current_state` / `current_step`（per ADR-0024 IDE session 边界）
- **write 权限**：❌ 只读（agent 状态变更走 `submit` / lease heartbeat，per ADR-0030）
- **缓存 scope**：`session` / `ttlMs=5000`（per mcp/01 §2.2 cacheScope=session 约定）
- **典型字段**：见 agent-api/v1 §3.16

### 1.5 decision 类型

- **URI 模式**：`decision://<workspace_id>/<decision_id>`（per ADR-0018 Decision Record）
- **content_type**：`application/json`（Decision schema — `decision_id` / `kind` / `decided_by` / `decided_at` / `rationale` / `evidence_refs[]` / `supersedes?`）
- **read 权限**：
  - `agents/{any}` — 读 `decided_at` / `kind` / `decided_by`
  - `agents/{peer-of-decider}` — 读 `rationale` / `evidence_refs[]`
  - `ide/{any}` — 读脱敏版（per ADR-0022 IDE Placement）
- **write 权限**：❌ 永久只读（decision 不可改写，只能 supersede）
- **缓存 scope**：`workspace` / `ttlMs=300000`（decision 变更频率低，5 分钟缓存）
- **典型字段**：见 ADR-0018

> **F-28 守门**：4 类 Resources 在 MVP（Level 1-2）不实现，本节仅为 Phase 2+ 设计。MVP 退出条件 [acceptance/04 §3](../../acceptance/04-mvp-acceptance.md) 不含 Resources 数量。

### 1.6 Resources 通用属性

跨 4 类 resource 的统一约定（per MCP 2026-07-28 spec + arch/03 §2.3）：

- **`mimeType`** 必填 — 4 类全部为 `application/json`（per mcp/01 §1.1 ① Stateless core），二进制资源（图片 / 视频）Phase 3+ 再评估
- **annotations** — `audience: ["user", "assistant"]` / `priority: 0.5`（默认）；高优先级 resource（agent 启动必需）标 `priority: 0.9`
- **size** — server 端在 `resources/list` 响应里返回估算字节数（per MCP 2026-07-28 §4 Resources），client 据此分页
- **MIME 不一致 → 错误码 `MIME_MISMATCH`**（per [03 §2 标准错误码表](./03-error-model-spec.md)）—— 写 Phase 2+ 评估
- **`resources/subscribe` 协议** — Phase 3+ 才落地（依赖 Streamable HTTP server-push，per arch/03 §5.1 F-22）；MVP 无订阅语义
- **list 结果可缓存**（per mcp/01 §1.1 ④）— `resources/list` 响应 metadata 含 `ttlMs=30000` + `cacheScope=workspace`

## 2. Prompts 模板

### 2.1 5 个模板总览

| Prompt name | 用途 | 触发场景 | Phase |
|---|---|---|---|
| `submit` | Universal Submit 12 步流程模板 | agent 准备提交 MR 时 | Phase 2 |
| `review` | Code review 模板 | `request_review` 触发后 | Phase 2 |
| `context` | Context Graph 拉取模板 | agent 启动 / 切换 worktree 时 | Phase 2+ |
| `workflow` | Fallback Ladder 降级模板 | IDE 能力降级时 | Phase 2+ |
| `debug` | Trace + retry 调试模板 | agent 失败后 | Phase 2 |

### 2.2 `submit`

- **name**：`submit`
- **description**："Universal Submit 12 步流程模板（per [spec/flows/05](../flows/05-universal-submit.md)）—— 让 agent 按 12 步顺序准备 submission payload，再调 `submit` tool（per mcp/01 §2.3 P1-F）"
- **arguments[]**：
  - `{name: "worktree_id", required: true, description: "目标 worktree（per agent-api/v1 §3.2）"}`
  - `{name: "force", required: false, description: "是否跳过 conflict 检查（与 `submit` tool 的 `force` 参数对齐）"}`
  - `{name: "include_validation", required: false, description: "是否在第 7 步强制 `run_validation`"}`

### 2.3 `review`

- **name**：`review`
- **description**："Code review 模板 —— 输出 review checklist + reviewers 列表 + 期望 SLA"
- **arguments[]**：
  - `{name: "mr_id", required: true, description: "待 review 的 MR ID（per agent-api/v1 §3.7）"}`
  - `{name: "reviewers", required: false, description: "指定 reviewers（5 域 Lead 不接受兼任 per 8/21 JST）"}`
  - `{name: "depth", required: false, description: "review 深度：`shallow` | `normal` | `deep`"}`

### 2.4 `context`

- **name**：`context`
- **description**："Context Graph 拉取模板（per ADR-0031 Context Graph — MVP 4 节点 / 5 关系）—— 让 agent 启动时按节点 / 关系剪裁必要上下文"
- **arguments[]**：
  - `{name: "root_node", required: true, description: "根节点（`issue` | `worktree` | `agent` | `decision`）"}`
  - `{name: "depth", required: false, description: "BFS 深度（默认 2，最大 5）"}`
  - `{name: "relations", required: false, description: "关系类型白名单（per ADR-0031 §3 关系集）"}`

### 2.5 `workflow`

- **name**：`workflow`
- **description**："Fallback Ladder 降级模板（per arch/03 §4.3）—— 当 IDE 能力不足时，按 Level 1 → Level 2 → Level 3 → Level 4 顺序降级 + 提示用户"
- **arguments[]**：
  - `{name: "current_level", required: true, description: "当前 Level（1-4，per arch/03 §4.3 表）"}`
  - `{name: "capability_missing", required: true, description: "缺失的能力名（`mcp_client` | `cli_binary` | `http_client` | `git`）"}`
  - `{name: "ide", required: false, description: "目标 IDE 名（Cursor / Junie / Claude Code / VS Code / Web / Git-only）"}`

### 2.6 `debug`

- **name**：`debug`
- **description**："Trace + retry 调试模板 —— 输出 6 字段 Error 的 `code` / `source_module` / `retriable` / `hint`，按 [spec/mcp/03-error-model-spec.md §3 retriable 矩阵](./03-error-model-spec.md) 决定是否重试"
- **arguments[]**：
  - `{name: "error_code", required: true, description: "SCREAMING_SNAKE_CASE 错误码（per 03 §2 标准错误码表）"}`
  - `{name: "retry_count", required: false, description: "已重试次数（per ADR-0030 lease + retry policy）"}`
  - `{name: "include_trace", required: false, description: "是否包含 trace_id（默认 false，敏感字段）"}`

> **F-28 守门**：5 个 Prompts 在 MVP 不实现（per mcp/01 §5 + arch/03 §2.3）。MVP agent 调 `tools/call` 完成同样目标，prompts 是 **Phase 2+** 优化（让 IDE 端可发现 preset 模板）。

## 3. Resources URI 命名表

| URI scheme | 模板 | 引用 schema | 权限矩阵 | Phase |
|---|---|---|---|---|
| `workspace://` | `workspace://<workspace_id>` | agent-api/v1 §3.15 WorkspaceSummary | `agents/{any}: R` | Phase 2 |
| `worktree://` | `worktree://<workspace_id>/<worktree_id>` | agent-api/v1 §3.2 Worktree | `agents/{any}: R(basic)` / `agents/{owner}: R(all)` | Phase 2 |
| `agent://` | `agent://<workspace_id>/<agent_id>` | agent-api/v1 §3.16 Resume (11 字段) | `agents/{self}: R(all)` / `agents/{peer}: R(redact)` / `ide/{owner}: R(public)` | Phase 2 |
| `decision://` | `decision://<workspace_id>/<decision_id>` | ADR-0018 Decision Record | `agents/{any}: R(meta)` / `agents/{peer-of-decider}: R(all)` / `ide/{any}: R(redact)` | Phase 2+ |

> **权限矩阵符号**：R = read / R(basic) = 只读基础字段 / R(redact) = 脱敏 / R(all) = 全部 / R(meta) = 仅元数据。**不**存在 W（write）—— Resources 全部只读，写入统一走 `create_*` / `update_*` / `request_*` tools（per mcp/01 §3.1 禁止直接暴露）。

## 4. Prompts 列表

| name | arguments | 引用 | Phase |
|---|---|---|---|
| `submit` | worktree_id / force / include_validation | flows/05 §2 | Phase 2 |
| `review` | mr_id / reviewers / depth | mcp/01 §2.3 `request_review` | Phase 2 |
| `context` | root_node / depth / relations | ADR-0031 §3 | Phase 2+ |
| `workflow` | current_level / capability_missing / ide | arch/03 §4.3 | Phase 2+ |
| `debug` | error_code / retry_count / include_trace | 03 §3 retriable 矩阵 | Phase 2 |

> **命名约定（per mcp/01 §2.1）**：prompts name 用 snake_case（与 tools name 一致）；`workflow` / `debug` 是 `noun_verb` 风格；`submit` / `review` / `context` 是单动词（MCP 2026-07-28 spec 允许）。

## 5. 客户端集成

### 5.1 stdio transport（per mcp/01 §1）

- `resources/list` — 列出当前 scope 可见的全部 resources（按 URI scheme 分组返回）
- `resources/read` — 单 resource 读取（uri + 可选 `range` 参数，Phase 2+ 评估）
- `prompts/list` — 列出全部 prompts（按 name 字典序升序，与 mcp/01 §2.2 工具表排序约定对齐）
- `prompts/get` — 单 prompt 渲染（name + arguments[] → 渲染后 messages[]）

### 5.2 Streamable HTTP transport（per arch/03 §5.1 F-22 — Phase 2+）

- **POST `/mcp/resources/list`** — 列出 resources（per [spec/flows/03 §2](../flows/03-mcp-streamable-http.md) Phase 2+ 设计）
- **POST `/mcp/resources/read`** — 读取 resource
- **POST `/mcp/prompts/list`** — 列出 prompts
- **POST `/mcp/prompts/get`** — 渲染 prompt
- **GET `/mcp/resources/subscribe?uri=...`** — resource 变更订阅（Phase 3+ 评估，依赖 Streamable HTTP server-push）

> **MVP 守门（per F-22）**：Streamable HTTP 是 **Phase 2+**，MVP 仅 stdio。resources/prompts 4 个 endpoint 在 Streamable HTTP 阶段才落地。

### 5.3 错误响应（与 [03-error-model-spec.md](./03-error-model-spec.md) 对齐）

resources / prompts 操作的 6 字段 Error 行为（per agent-api/v1 §3.14 + 03 §1）：

- **`resources/read` 失败** — `code: RESOURCE_NOT_FOUND`（`retriable=false` / `source_module=mcp` / `hint: 检查 uri 命名是否匹配 03 §3 命名表`）
- **`resources/read` 权限不足** — `code: POLICY_DENIED`（`retriable=false` / `source_module=policy` / `hint: 申请更高权限`）
- **`prompts/get` 缺 required argument** — `code: VALIDATION_FAILED`（`retriable=false` / `source_module=mcp` / `hint: 补 required arguments 后重试`）
- **`resources/subscribe` 未实现** — `code: NOT_IMPLEMENTED`（`retriable=false` / `source_module=mcp` / `hint: Streamable HTTP Phase 3+ 才落地`）
- **`prompts/get` 渲染失败** — `code: PROMPT_RENDER_FAILED`（`retriable=true` / `source_module=mcp` / `hint: 检查 arguments 类型，1s 后重试`）

## 6. 已知缺口

1. **Resources **Phase 2** 评估延迟**：MVP 不实现（per F-28），4 类资源设计完整但无 Rust 代码实装（per 任务授权："不实装 Rust 代码"）；Phase 2 启动时需重新评估 URI scheme 是否与 2026-07-28 spec 后续版本冲突。
2. **Prompts 模板内容待填**：5 个 prompt 仅 `name` / `description` / `arguments[]`，**不含** `messages[]` 模板正文（mcp/01 §5 也未给正文）。Phase 2 启动时需补每个 prompt 的 实际消息模板（当前是 placeholder）。
3. **subscribe 协议未定**：resources 变更订阅（5.2 GET endpoint）依赖 Streamable HTTP server-push，协议细节 Phase 3+ 再定。
4. **权限矩阵与 ADR-0021 Zero Vendor Cooperation 关系**：本 spec 假设 5 域 Lead 独立（per 8/21 JST），但具体 enforcement（policy engine 拒绝越权 read）依赖 [spec/policy/01](../policy/01-policy-engine.md) Phase 2+ 落地。
5. **decision 类型的 supersede 链**：decision 不可改写只能 supersede，但 URI 表未定义 `decision://.../superseded-by:...` 跳转关系，Phase 2+ 需补 supersede 关系 URI。
6. **与 agent-api/v1 §3.21 Decision schema 对齐**：上面 1.5 提到的 Decision 字段（`decision_id` / `kind` / ...）需以 [agent-api/01 §3.21](../agent-api/01-schema.md) 落盘版本为准，**目前仅引用 ADR-0018 历史记录**（per F-21 强化交叉引用原则，Phase 2+ 需重新对齐）。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Mavis（接手 agent per DEC-008）| 初版：4 类 Resources（workspace/worktree/agent/decision）+ 5 个 Prompts（submit/review/context/workflow/debug）+ URI 命名表 + prompts 列表 + stdio/Streamable HTTP 集成 + 6 项已知缺口；显式标 "MVP 不实现（Level 1-2 不含），Phase 2+ 评估" 守 mcp/01 §4-§5 + F-28 修复 | Phase E spec 子代理任务（per parent 19:39 JST 代签授权）|
