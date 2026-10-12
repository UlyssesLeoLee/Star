# FSMS REST API 规范

> **Version**: v1.0
> **Base Path**: `/api/v1/file-sessions`
> **Auth**: Bearer Token (JWT), RLS 强制 `tenant_id`
> **Content-Type**: `application/json`
> **Cache**: 所有响应 `Cache-Control: no-store`

---

## 通用规范

### 请求头
```
Authorization: Bearer <jwt>
Content-Type: application/json
X-Correlation-ID: <uuid>          # 可选，用于追踪
Idempotency-Key: <string>         # 幂等键，创建/检查点/恢复必填
```

### 响应头
```
Cache-Control: no-store
Pragma: no-cache
X-Request-ID: <uuid>
```

### 错误格式 (RFC 9457)
```json
{
  "type": "https://api.star.dev/errors/validation_error",
  "title": "Validation Failed",
  "status": 400,
  "detail": "request body validation failed",
  "instance": "/api/v1/file-sessions",
  "errors": [
    { "field": "file_path", "code": "required", "message": "file_path is required" }
  ]
}
```

### 通用错误码
| Status | Type | 说明 |
|--------|------|------|
| 400 | `validation_error` | 请求体校验失败 |
| 401 | `unauthorized` | Token 无效/过期 |
| 403 | `forbidden` | 无权限 (RLS/策略拒绝) |
| 404 | `not_found` | 资源不存在 |
| 409 | `conflict` | 版本冲突/并发修改/幂等键重复 |
| 422 | `unprocessable` | 业务逻辑拒绝 (如状态机非法迁移) |
| 429 | `rate_limited` | 限流 |
| 500 | `internal` | 服务器内部错误 |
| 503 | `unavailable` | 依赖服务不可用 (Git/Adapter/Storage) |

---

## 资源模型

### FileSession (完整)
```json
{
  "meta": {
    "session_id": "uuid",
    "file_path": "string",
    "agent_type": "claude|codex|opencode|hermes|custom",
    "agent_version": "string",
    "adapter_config": { "model": "string", "max_tokens": "integer" },
    "state": "created|running|paused|completed|cancelled|archived",
    "created_at": "datetime",
    "updated_at": "datetime",
    "tags": ["string"],
    "links": {
      "task_card_id": "uuid|null",
      "worktree_id": "uuid|null",
      "agent_session_id": "uuid|null",
      "parent_session_id": "uuid|null",
      "child_session_ids": ["uuid"],
      "related_sessions": [{"session_id": "uuid", "type": "depends|relates|blocks"}]
    }
  },
  "context": {
    "initial_ref": "string",
    "checkpoints": [
      {"number": "integer", "ref": "string", "token_delta": "integer", "at": "datetime"}
    ],
    "current_checkpoint": "integer",
    "token_budget": "integer",
    "token_used": "integer"
  },
  "transcript": {
    "index_ref": "string",
    "chunks": [{"file": "string", "range": ["integer", "integer"]}],
    "total_messages": "integer"
  },
  "artifacts": {
    "patches": ["string"],
    "diffs": ["string"],
    "exports": ["string"]
  },
  "git": {
    "branch": "string|null",
    "worktree_path": "string|null",
    "commits": [{"sha": "string", "message": "string", "at": "datetime"}],
    "mr_links": [{"url": "string", "number": "integer", "platform": "github|gitlab"}]
  },
  "state_snapshots": [
    {"checkpoint": "integer", "ref": "string", "at": "datetime"}
  ]
}
```

### SessionRef (列表项)
```json
{
  "session_id": "uuid",
  "file_path": "string",
  "agent_type": "string",
  "state": "string",
  "created_at": "datetime",
  "updated_at": "datetime",
  "tags": ["string"],
  "linked_task_card_id": "uuid|null",
  "linked_worktree_id": "uuid|null",
  "linked_agent_session_id": "uuid|null"
}
```

### SessionSummary (Manifest 项)
```json
{
  "session_id": "uuid",
  "agent_type": "string",
  "state": "string",
  "title": "string",
  "created_at": "datetime",
  "updated_at": "datetime",
  "checkpoint_count": "integer",
  "token_usage": {"input": "integer", "output": "integer", "total": "integer"},
  "git_branch": "string|null"
}
```

---

## 端点定义

### 1. 创建会话
**POST** `/api/v1/file-sessions`

#### Request
```json
{
  "file_path": "crates/star-cli/src/commands/task.rs",
  "agent_type": "claude",
  "title": "重构 task.rs: 提取 CurrentArgs 解析逻辑",
  "tags": ["refactor", "cli"],
  "adapter_config": {
    "model": "claude-3-5-sonnet",
    "max_tokens": 8192
  },
  "linked_task_card_id": "01a0bf6b-486c-71b8-9bf4-92bde909c7f8",
  "linked_worktree_id": "wt-sub-session-001",
  "auto_worktree": true,
  "auto_commit": true
}
```

| 字段 | 必填 | 类型 | 说明 |
|------|------|------|------|
| file_path | ✓ | string | 相对仓库根目录的文件路径 |
| agent_type | | string | claude/codex/opencode/hermes/custom，默认配置值 |
| title | | string | 会话标题，用于列表显示 |
| tags | | string[] | 用户标签 |
| adapter_config | | object | Agent 特定配置 |
| linked_task_card_id | | uuid | 可选关联 Task Card |
| linked_worktree_id | | uuid | 可选关联 Worktree |
| auto_worktree | | bool | 是否创建独立 worktree，默认配置值 |
| auto_commit | | bool | 是否自动 git commit，默认配置值 |

#### Response (201 Created)
```json
{
  "session": { /* FileSession 完整对象 */ },
  "workspace_path": "/repo/.star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4",
  "git_branch": "session/crates-star-cli-src-commands-task-rs/a1b2c3d4"
}
```

---

### 2. 列出会话
**GET** `/api/v1/file-sessions`

#### Query Parameters
| 参数 | 类型 | 说明 |
|------|------|------|
| file_path | string | 精确匹配文件路径 |
| file_path_prefix | string | 前缀匹配 (如 `crates/star-cli/`) |
| agent_type | string | 过滤 agent 类型 |
| state | string | 过滤状态 (created/running/paused/completed/cancelled/archived) |
| tag | string | 过滤标签 (可重复) |
| linked_task_card_id | uuid | 关联的 Task Card |
| linked_worktree_id | uuid | 关联的 Worktree |
| limit | integer | 默认 50，最大 200 |
| offset | integer | 分页偏移，默认 0 |
| sort | string | `created_at|updated_at` 默认 `-updated_at` |
| include_summary | bool | 是否包含 summary 字段，默认 false |

#### Response (200 OK)
```json
{
  "sessions": [ /* SessionRef[] */ ],
  "total": 123,
  "limit": 50,
  "offset": 0,
  "has_more": true
}
```

---

### 3. 获取会话详情
**GET** `/api/v1/file-sessions/{session_id}`

#### Query Parameters
| 参数 | 类型 | 说明 |
|------|------|------|
| include_transcript | bool | 包含完整 transcript，默认 false |
| include_artifacts | bool | 包含 artifacts 详情，默认 false |
| transcript_limit | integer | transcript 条数限制，默认 100 |
| transcript_before | string | 游标 (message_id)，加载更早消息 |

#### Response (200 OK)
```json
{
  "session": { /* FileSession 完整对象 */ }
}
```

---

### 4. 更新会话 (元数据)
**PATCH** `/api/v1/file-sessions/{session_id}`

#### Request
```json
{
  "title": "新标题",
  "tags": ["refactor", "urgent"],
  "state": "paused",              // 仅允许 running→paused, paused→running
  "adapter_config": {             // 仅 running 状态可更新部分配置
    "max_tokens": 16384
  }
}
```

> **状态机约束**：只能更新 `title`、`tags`、`state` (合法迁移)、`adapter_config` (非破坏性字段)。其他字段不可变。

#### Response (200 OK)
```json
{
  "session": { /* 更新后的 FileSession */ }
}
```

---

### 5. 创建检查点
**POST** `/api/v1/file-sessions/{session_id}/checkpoint`

#### Request
```json
{
  "message": "完成重构 extract_args 函数",
  "auto_commit": true,
  "create_worktree_snapshot": false
}
```

| 字段 | 必填 | 类型 | 说明 |
|------|------|------|------|
| message | | string | 检查点描述 |
| auto_commit | | bool | 是否自动 git commit，默认配置值 |
| create_worktree_snapshot | | bool | 是否在 worktree 创建快照分支 |

#### Response (200 OK)
```json
{
  "checkpoint": {
    "number": 3,
    "ref": "context/checkpoints/003.json",
    "token_delta": 3200,
    "at": "2026-10-12T10:25:00Z",
    "git_commit_sha": "abc123def"
  },
  "session": { /* 更新后的 FileSession */ }
}
```

---

### 6. 恢复会话
**POST** `/api/v1/file-sessions/{session_id}/resume`

#### Request
```json
{
  "checkpoint_number": 2,         // 可选，默认最新
  "agent_type": "codex",          // 可选，切换 agent
  "adapter_config": {             // 可选，覆盖配置
    "model": "gpt-5"
  }
}
```

#### Response (200 OK)
```json
{
  "session_handle": {
    "process_id": 12345,
    "control_channel": "pty://...",
    "metadata": {}
  },
  "session": { /* 状态更新为 running 的 FileSession */ }
}
```

---

### 7. 归档会话
**POST** `/api/v1/file-sessions/{session_id}/archive`

#### Request
```json
{
  "compress_transcript": true
}
```

#### Response (200 OK)
```json
{
  "session": { /* state=archived 的 FileSession */ },
  "git_commit_sha": "fedcba987"
}
```

---

### 8. 删除会话
**DELETE** `/api/v1/file-sessions/{session_id}`

#### Query Parameters
| 参数 | 类型 | 说明 |
|------|------|------|
| force | bool | true=物理删除，false=软删除移入 `.trash/` (默认) |
| purge_git_history | bool | 是否从 git 历史彻底清除 (需管理员权限) |

#### Response (200 OK)
```json
{
  "deleted": true,
  "session_id": "uuid",
  "soft_deleted": true,
  "trash_path": ".star/sessions/.trash/crates/.../a1b2c3d4"
}
```

---

### 9. 分叉会话
**POST** `/api/v1/file-sessions/{session_id}/fork`

#### Request
```json
{
  "title": "探索方案 B: 使用宏重写",
  "tags": ["experiment"],
  "inherit_checkpoints": true,     // 继承历史 checkpoint
  "start_from_checkpoint": 2       // 从指定 checkpoint 分叉
}
```

#### Response (201 Created)
```json
{
  "session": { /* 新 FileSession，parent_session_id 指向源 */ },
  "workspace_path": "...",
  "git_branch": "session/.../fork-e5f6g7h8"
}
```

---

### 10. 关联会话
**POST** `/api/v1/file-sessions/{session_id}/link`

#### Request
```json
{
  "target_session_id": "e5f6g7h8-...",
  "type": "depends",              // depends|relates|blocks
  "description": "依赖 session A 的提取函数"
}
```

#### Response (200 OK)
```json
{
  "session": { /* 更新后的 FileSession，links.related_sessions 新增 */ }
}
```

---

### 11. 获取 Transcript
**GET** `/api/v1/file-sessions/{session_id}/transcript`

#### Query Parameters
| 参数 | 类型 | 说明 |
|------|------|------|
| limit | integer | 默认 100，最大 500 |
| before | string | message_id 游标，加载更早 |
| after | string | message_id 游标，加载更新 |
| type | string | 过滤类型: user/assistant/tool/system |

#### Response (200 OK)
```json
{
  "messages": [
    {
      "id": "msg_001",
      "type": "user",
      "content": "请帮我重构这个函数",
      "timestamp": "2026-10-12T10:00:00Z",
      "token_count": 45
    },
    {
      "id": "msg_002",
      "type": "assistant",
      "content": "好的，我来分析...",
      "timestamp": "2026-10-12T10:00:05Z",
      "token_count": 1200,
      "tool_calls": [
        {"name": "read_file", "args": {"path": "task.rs"}}
      ]
    }
  ],
  "pagination": {
    "limit": 100,
    "before": "msg_001",
    "after": null,
    "has_more": true
  }
}
```

---

### 12. 获取 Artifacts
**GET** `/api/v1/file-sessions/{session_id}/artifacts`

#### Response (200 OK)
```json
{
  "patches": [
    {"path": "artifacts/patches/001.patch", "size": 2048, "created_at": "..."}
  ],
  "diffs": [
    {"path": "artifacts/diffs/001-vs-002.diff", "size": 1024, "created_at": "..."}
  ],
  "exports": [
    {"path": "artifacts/exports/session.md", "format": "markdown", "size": 5120, "created_at": "..."}
  ]
}
```

---

### 13. Git 同步
**POST** `/api/v1/file-sessions/{session_id}/sync`

#### Request
```json
{
  "remote": "origin",
  "push": true,
  "pull": true,
  "dry_run": false
}
```

#### Response (200 OK)
```json
{
  "pushed": 3,
  "pulled": 1,
  "conflicts": [],
  "new_sessions": ["uuid1", "uuid2"],
  "updated_sessions": ["uuid3"]
}
```

---

### 14. 分支管理
**GET** `/api/v1/file-sessions/{session_id}/branch` - 列出分支
**POST** `/api/v1/file-sessions/{session_id}/branch` - 创建/切换分支

#### POST Request
```json
{
  "action": "create|switch|delete|merge",
  "branch_name": "session/.../feature-x",
  "target_branch": "main",        // merge 时必填
  "strategy": "merge|rebase|squash"
}
```

#### Response
```json
{
  "branch": "session/.../feature-x",
  "action": "created",
  "commit_sha": "abc123"
}
```

---

### 15. 导出会话
**POST** `/api/v1/file-sessions/{session_id}/export`

#### Request
```json{
  "format": "markdown|json|patch|mr|bundle",
  "include_transcript": true,
  "include_artifacts": true,
  "redact_secrets": true
}
```

| Format | 说明 |
|--------|------|
| markdown | 人类可读报告 |
| json | 完整数据导出 (可导入) |
| patch | 合并所有 patch 为单文件 |
| mr | GitHub/GitLab MR 描述格式 |
| bundle | 便携包含所有文件的 zip |

#### Response (200 OK)
```json
{
  "format": "markdown",
  "content": "# Session Report...\n\n...",  // 或 base64 编码的二进制
  "filename": "session-a1b2c3d4.md",
  "size": 15360
}
```

---

### 16. 创建 MR
**POST** `/api/v1/file-sessions/{session_id}/mr`

#### Request
```json
{
  "title": "feat: 重构 task.rs extract_args 函数",
  "target_branch": "main",
  "draft": true,
  "reviewers": ["user1", "user2"],
  "labels": ["refactor", "cli"],
  "auto_merge": false
}
```

#### Response (201 Created)
```json
{
  "mr_url": "https://github.com/org/repo/pull/123",
  "mr_number": 123,
  "platform": "github",
  "branch": "session/crates-star-cli-src-commands-task-rs/a1b2c3d4"
}
```

---

## 适配器管理端点

### 列出适配器
**GET** `/api/v1/file-sessions/adapters`

#### Response
```json
{
  "adapters": [
    {
      "name": "claude",
      "type": "claude",
      "version": "1.0.45",
      "status": "healthy",
      "capabilities": ["spawn", "attach", "checkpoint", "resume", "export"],
      "supported_models": ["claude-3-5-sonnet", "claude-3-opus"],
      "config_schema": { /* JSON Schema */ }
    }
  ]
}
```

### 配置适配器
**PATCH** `/api/v1/file-sessions/adapters/{adapter_name}`

#### Request
```json
{
  "config": {
    "default_model": "claude-3-5-sonnet",
    "default_max_tokens": 8192,
    "timeout_seconds": 300
  }
}
```

---

## 配置端点

### 获取配置
**GET** `/api/v1/file-sessions/config`

### 更新配置
**PATCH** `/api/v1/file-sessions/config`

```json
{
  "git": {
    "auto_commit": true,
    "worktree_isolation": true,
    "remote_sync_interval": 300
  },
  "storage": {
    "backend": "filesystem",
    "sqlite_index": true,
    "artifact_store": "s3://bucket/prefix"
  },
  "privacy": {
    "enabled": true,
    "custom_rules_path": ".star/privacy-rules.yaml"
  },
  "defaults": {
    "agent_type": "claude",
    "token_budget": 50000
  }
}
```

---

## WebSocket 实时更新 (可选)

**GET** `/api/v1/file-sessions/{session_id}/ws`

### 订阅事件
```json
{"type": "subscribe", "events": ["state_change", "checkpoint", "message", "artifact"]}
```

### 服务端推送
```json
{"event": "checkpoint", "session_id": "uuid", "checkpoint_number": 3, "at": "..."}
{"event": "message", "session_id": "uuid", "message": { /* transcript message */ }}
{"event": "state_change", "session_id": "uuid", "from": "running", "to": "paused"}
{"event": "artifact", "session_id": "uuid", "artifact": { /* artifact ref */ }}
```

---

## 权限矩阵

| 权限 | 说明 | 默认角色 |
|------|------|----------|
| `file_session:create` | 创建会话 | Editor, Owner |
| `file_session:read` | 读取会话 | Viewer, Editor, Owner |
| `file_session:write` | 更新/检查点/恢复 | Editor, Owner |
| `file_session:delete` | 删除会话 | Owner |
| `file_session:archive` | 归档会话 | Editor, Owner |
| `file_session:fork` | 分叉会话 | Editor, Owner |
| `file_session:link` | 关联会话 | Editor, Owner |
| `file_session:sync` | Git 同步 | Editor, Owner |
| `file_session:export` | 导出会话 | Viewer, Editor, Owner |
| `file_session:mr` | 创建 MR | Editor, Owner |
| `file_session:admin` | 配置/适配器管理 | Owner |

> RLS Policy: `WHERE tenant_id = current_setting('app.tenant_id')::uuid`

---

## 速率限制

| 端点类别 | 限制 |
|----------|------|
| 创建/分叉/MR | 10 req/min per user |
| 检查点/恢复 | 30 req/min per session |
| 列表/详情/Transcript | 100 req/min per user |
| Git 同步 | 5 req/min per repo |
| 导出 | 20 req/min per user |

---

## 版本控制

- URL 路径固定 `/api/v1/file-sessions`
- 破坏性变更发布 v2，v1 维护 12 个月
- `Accept: application/vnd.star.fsms.v1+json` 显式指定版本