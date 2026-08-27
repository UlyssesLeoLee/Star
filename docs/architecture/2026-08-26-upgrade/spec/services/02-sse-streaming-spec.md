# 39. SSE Server Push

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/services/01 Service Adapter](01-service-adapter-spec.md) · [spec/services/03 Webhook Adapter](03-webhook-adapter-spec.md) · [spec/flows/08 Event Model](../flows/08-event-model.md) · [arch/03 §8 Event 命名空间](../../arch/03-star-ai-compat-arch.md)

## §0 目的（SSE 在 STAR 推送场景的边界）

STAR 上层（CLI / MCP / REST / IDE Gateway）需要"准实时"感知 VCS / Agent / Worktree 状态变化。三种推送路径并列（per [arch/03 §3 Fallback Ladder](../../arch/03-star-ai-compat-arch.md) Level 1-2）：

| 推送路径 | 触发源 | 实时性 | 适用 |
|---|---|---|---|
| **Webhook Adapter**（inbound POST） | vendor 主动 POST 到 STAR | 秒级 | 已知 vendor 配置 + 公开端点 |
| **SSE Server Push**（本 spec） | STAR 主动推送 | 毫秒-秒级 | 内部组件（Agent / Worktree / Pipeline）状态变化 |
| **Polling fallback** | 客户端定时 GET | 分钟级 | Level 4 Git Only 兜底 |

**SSE 边界**（per [arch/03 §3.1](../../arch/03-star-ai-compat-arch.md)）：
- **是**：内部状态变化的"主动推送"通道（Agent 状态 / Worktree 创建 / Pipeline 进度）
- **否**：vendor 事件入站（由 [03 Webhook Adapter](03-webhook-adapter-spec.md) 负责）
- **否**：MCP tool 调用（由 [mcp/01 §1 stdio transport](../mcp/01-mcp-spec.md) + Phase 2 Streamable HTTP 负责，本 spec 是**STAR 自己的** server 端推送，**不**混入 MCP）

**SSE 与 vendor Webhook 的关系**（关键）：vendor webhook 推入后，STAR 在 Application Service 内**转译**为 [flows/08 §1.1 13 个 STAR Domain Event](../flows/08-event-model.md)（如 `MergeRequestCreated.star`），再**通过 SSE 推**给订阅客户端。两层清晰：
- vendor → STAR = Webhook Adapter（inbound, [03](03-webhook-adapter-spec.md)）
- STAR → 客户端 = SSE（本 spec）

## §1 事件类型

5 类核心事件（per [flows/08 §1.1](../flows/08-event-model.md) 13 个 STAR Domain Events 子集）：

| Event Type | 触发源 | payload 关键字段 | 频次估计 |
|---|---|---|---|
| `MergeRequestCreated.star` | vendor webhook → [03 §4 映射](03-webhook-adapter-spec.md) | `mr_id, title, author, base, head, url` | 低（分钟级）|
| `MergeRequestMerged.star` | vendor webhook | `mr_id, merged_by, merged_at, sha` | 低 |
| `PipelineStatusChanged.star` | vendor webhook（GitHub `check_run` / GitLab `pipeline`）| `pipeline_run_id, status, conclusion, duration_ms` | 中（每个 pipeline 1-10 次）|
| `AgentStateChanged.star` | [flows/02 Lease Heartbeat](../flows/02-agent-lease-heartbeat.md) 触发 | `agent_id, from_state, to_state, lease_expires_at` | 中（每个 agent 数分钟 1 次）|
| `WorktreeChanged.star` | [flows/01 §3 Agent Task Lifecycle](../flows/01-agent-task-lifecycle.md) + GitGit 桥（per [arch/05 §3](../../arch/05-gitgit-compat-arch.md)）| `worktree_id, branch, change_type, trace_id` | 高（每次 commit / push）|

**命名约束**（per [arch/03 §8](../../arch/03-star-ai-compat-arch.md) + [flows/08 §1.1](../flows/08-event-model.md) B-17 修复）：
- 5 类全部用 `.star` 后缀（逻辑层，**不**混 `.gitgit` 物理层）
- `WorktreeChanged.star` **由 STAR 业务层重新发射**（per [flows/08 §1.1](../flows/08-event-model.md) "重名事件边界"），**不**直接转发 GitGit `WorktreeCreated.gitgit`

**Event schema**（per [flows/08 §3 7 字段权威定义](../flows/08-event-model.md) B-27 修复）：
- `event_id` (UUID v7) / `type` (PascalCase + `.star` 后缀) / `source` (`"star"` / `"automation"`)
- `timestamp` (ISO 8601) / `trace_id` / `payload` (JSON object) / `schema_version` (`"star-event/v1"`)

## §2 连接管理

### 2.1 鉴权（per [arch/06 §3 威胁模型](../../arch/06-threat-model-nfr.md)）

- **Bearer token** 优先：`Authorization: Bearer <agent_session_token>`，token 由 [flows/02 §2 Lease Heartbeat](../flows/02-agent-lease-heartbeat.md) 签发
- **OAuth 2.1 fallback**：per [mcp/01 §1.1 ⑤ RFC 9207 issuer validation](../mcp/01-mcp-spec.md) 同步约束
- **token 解析**：`star-sse` 入口调 [agent-api/v1 §3.15 Error](../agent-api/01-schema.md) 链路的鉴权中间件，**不**重复实现

### 2.2 心跳（heartbeat）

- 间隔：30s（per WHATWG HTML Living Standard SSE spec，proxy 默认 60s idle timeout 留 2x 余量）
- 格式：`:heartbeat\n\n`（SSE comment 行，客户端忽略，**不**算 event）
- 服务端资源开销：1 字节 / 30s / 连接 = 可忽略

### 2.3 重连（reconnect）

- **客户端责任**：断连后客户端**必须**用 `Last-Event-ID` header 重连
- **服务端责任**：保留最近 1 小时事件 ID 索引（in-memory LRU + 持久化 fallback 到 `crates/star-event/` event bus 归档），超 1 小时返回 `410 Gone`，客户端降级 polling
- **重连退避**：客户端实现（建议 exponential backoff base 1s, max 60s, jitter ±20%）

### 2.4 Last-Event-ID

- **服务端行为**：收到 `Last-Event-ID` header，从索引 replay 该 ID 之后的所有事件（按 `event_id` 字典序）
- **格式**：`event_id: <UUID v7 string>\n`（per [flows/08 §3 `event_id`](../flows/08-event-model.md) UUID v7 时间排序）
- **边界**：replay 窗口 ≤ 1 小时；超出 → 返回 `410 Gone`

### 2.5 连接生命周期

```
[Client]                                          [STAR SSE Server]
   | -- GET /events (Authorization: Bearer) -->     |
   |                                                | → auth 校验
   | <-- 200 OK, Content-Type: text/event-stream -- |
   | <-- :heartbeat (every 30s) ----------------    |
   | <-- event: MergeRequestCreated.star            |
   |       id: event-2026-08-27-uuid-v7-...         |
   |       data: {"mr_id": "MR-1024", ...}          |
   |                                                |
   | --- 网络断 ---                                  |
   | -- GET /events                                 |
   |    Last-Event-ID: event-2026-08-27-uuid-v7-... |
   | <-- 200 OK, replay events from that ID         |
```

## §3 与 MCP Streamable HTTP 的边界

| 维度 | SSE Server Push（本 spec）| MCP Streamable HTTP（[mcp/01 §1](../mcp/01-mcp-spec.md) Phase 2+）|
|---|---|---|
| 协议 | HTTP/1.1 长连 text/event-stream | HTTP/1.1 + chunked + Server-Sent Events over POST |
| 用途 | STAR 业务事件推送（准实时）| MCP tool 调用的流式响应（单 tool 单连接）|
| 触发源 | STAR 业务事件总线 | MCP client tool invoke |
| Event schema | [flows/08 §3 7 字段](../flows/08-event-model.md) | MCP JSON-RPC 2.0 envelope |
| 鉴权 | Bearer token | OAuth 2.1 + RFC 9207 |
| Heartbeat | 30s SSE comment | per MCP spec（无硬约束）|
| 端口 | 8081（默认）| per [mcp/01 §6 实施位置](../mcp/01-mcp-spec.md) |

**关键边界**（per [arch/03 §2.3 Level 1-2 MVP 不实现 Streamable HTTP](../../arch/03-star-ai-compat-arch.md) + F-22 修复）：
- MVP 阶段 MCP 用 stdio（per [mcp/01 §1 Transport 选型](../mcp/01-mcp-spec.md)），**不**与本 spec 共享端口 / 进程
- Phase 2+ 评估 MCP Streamable HTTP 时，需**重新审视**与本 spec SSE 的端口 / 鉴权 / schema 是否可统一（per INTERFACE-REVIEW-A F-22）

**不**混用：
- SSE event 推**不**进 MCP tool response（schema 不同）
- MCP `tools/list` 响应**不**经过 SSE 推（一次性响应即可）

## §4 客户端 SDK 示例

### 4.1 浏览器 EventSource

```javascript
// 前端订阅 MergeRequest 事件
const es = new EventSource('/events', {
  headers: { 'Authorization': `Bearer ${agentToken}` }  // 注：浏览器 EventSource 不支持自定义 header
});
// → 浏览器场景必须用 token in query string (?token=...) 模式 + 短期 token
es.addEventListener('MergeRequestCreated.star', (e) => {
  const payload = JSON.parse(e.data);
  console.log('New MR:', payload.mr_id, payload.title);
});
es.addEventListener('PipelineStatusChanged.star', (e) => {
  const payload = JSON.parse(e.data);
  updatePipelineUI(payload.pipeline_run_id, payload.status);
});
```

### 4.2 Node.js 客户端（重连 + Last-Event-ID）

```javascript
// Node.js 订阅（含断线重连 + Last-Event-ID）
const { request } = require('http');

let lastEventId = null;

function connect() {
  const req = request({
    host: 'star.acme.com',
    port: 8081,
    path: '/events',
    headers: {
      'Authorization': `Bearer ${process.env.STAR_AGENT_TOKEN}`,
      'Accept': 'text/event-stream',
      ...(lastEventId && { 'Last-Event-ID': lastEventId })
    }
  }, (res) => {
    if (res.statusCode === 410) {
      console.error('Replay window expired, fallback to polling');
      return pollFallback();
    }
    let buffer = '';
    res.setEncoding('utf8');
    res.on('data', (chunk) => {
      buffer += chunk;
      const lines = buffer.split('\n');
      buffer = lines.pop();
      for (const line of lines) {
        if (line.startsWith('id: ')) lastEventId = line.slice(4);
        else if (line.startsWith('data: ')) handleEvent(JSON.parse(line.slice(6)));
      }
    });
    res.on('end', () => setTimeout(connect, 1000));  // 断线 1s 后重连
  });
  req.on('error', () => setTimeout(connect, 5000));  // 网络错 5s 后重连
  req.end();
}
connect();
```

### 4.3 CLI 订阅（长进程）

```bash
# Phase E+ 计划，MVP 不实现（per G-08 §5 已知缺口）
star events subscribe --type MergeRequestCreated.star --format json
# → 长连 /events，stdout 每行一个 JSON
```

## §5 已知缺口

| # | 缺口 | 状态 | 触发 |
|---|---|---|---|
| G-01 | 浏览器 EventSource **不支持**自定义 header，token 只能放 query string —— 需 server 端 `?token=` 鉴权模式 + 短期 token 配合 | 🟡 待 v0.2 设计 | WHATWG EventSource 限制 |
| G-02 | 1 小时 replay 窗口在 `crates/star-event/` event bus 归档具体落盘策略未明（LSM / WAL / 简单文件）| 🟡 待 Phase E+ 实施 | 本 spec 仅说"1 小时"未说"如何存"|
| G-03 | 多 node 部署时（SSE server ≥ 2 实例）Last-Event-ID replay 跨节点需 sticky session 或 event bus 共享，两者取舍未做 | 🟡 待 Phase 2+ 评估 | 部署架构未定时 |
| G-04 | 客户端 SDK 仅 §4 给了 JS / Node 示例，Python / Rust SDK 未列 | 🟡 Phase 2+ 评估 | 多语言需求待确认 |
| G-05 | 与 [mcp/01 Streamable HTTP Phase 2+](../mcp/01-mcp-spec.md) 端口 / 鉴权 / schema 统一可能性未评估 | 🟡 待 Phase 2+ | per F-22 修复保留 |
| G-06 | `star events subscribe` CLI 命令未实现（[spec/cli/01 §2.1 MVP 17 CLI](../cli/01-cli-spec.md) 不含）| 🟢 MVP 不实现 | per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) Level 1-2 范围 |
| G-07 | MCP `tools/list` 是否要暴露"订阅事件" 类 tool（`subscribe_event` / `unsubscribe_event`）未明 | 🟡 待 v0.2 评估 | MCP 是 request-response，无内置订阅语义 |
| G-08 | `crates/star-sse/` crate 路径为计划位置，本 spec v0.1 **不**实装 Rust 代码 | 🟢 显式不实装 | per 任务规则 5 |

## §6 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 19:39 JST 授权升级）| 初版：SSE 边界（与 Webhook Adapter / MCP Streamable HTTP）+ 5 类事件 + 连接管理 4 节（auth/heartbeat/reconnect/Last-Event-ID）+ 客户端 SDK 3 例 + 8 项已知缺口 | Phase E spec 起草（3 份：01-sa / 02-sse / 03-webhook）|
