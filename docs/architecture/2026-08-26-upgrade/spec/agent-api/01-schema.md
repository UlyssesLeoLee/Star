# 12. Agent CLI JSON Schema

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/cli/01-cli-spec.md](../cli/01-cli-spec.md)

## 1. Versioning

- Schema version: `agent-api/v1`
- Breaking change 必须升 v2
- 任何 field 重命名 / 移除 / 类型变更都算 breaking
- New field 是 additive（minor）

## 2. 核心 Schema（顶层）

```yaml
openapi: 3.1.0
info:
  title: STAR Agent API
  version: "1.0.0"
  description: |
    Machine-readable schema for any Coding Agent / AI Agent
    to interact with STAR.
  license:
    identifier: Apache-2.0
```

## 3. 核心 Schemas（节选）

### 3.1 Task

```json
{
  "id": "STAR-1024",
  "title": "Add authentication timeout",
  "status": "IN_PROGRESS",
  "assigned_to": "agent-abc",
  "context_refs": ["REQ-001", "ADR-005", "MR-789"],
  "acceptance_criteria": [...],
  "labels": ["backend", "auth"],
  "updated_at": "2026-08-26T19:00:00+09:00"
}
```

### 3.2 Worktree

```json
{
  "id": "wt-STAR-1024",
  "path": "/repos/owner/repo/wt-STAR-1024",
  "branch": "feature/STAR-1024",
  "head_commit": "abc123...",
  "dirty": true,
  "agent_session_id": "agent-abc",
  "ide_session_id": "ide-xyz",
  "created_at": "..."
}
```

### 3.3 SubmitResult

```json
{
  "status": "OK",
  "commit_sha": "...",
  "mr_id": "MR-789",
  "pipeline_run_id": "...",
  "validation_passed": true,
  "policy_checked": true
}
```

## 4. 全部 schema 落盘位置

`crates/star-cli/src/schemas/agent-api-v1/`：
- `Issue.json`
- `Task.json`
- `Worktree.json`
- `Workspace.json`
- `MR.json`
- `Context.json`
- `CodeSearchResult.json`
- `SymbolResult.json`
- `SubmitResult.json`
- `Error.json`
- `Capabilities.json`
- `Permissions.json`
- ...

## 5. 验证

```bash
# schema 必须合法
npx ajv validate -s crates/star-cli/src/schemas/agent-api-v1/SubmitResult.json \
                   -d test-data/submit-result.json

# CLI 输出必须符合 schema
star submit --json | python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin), indent=2))"  # 不抛错
```

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
