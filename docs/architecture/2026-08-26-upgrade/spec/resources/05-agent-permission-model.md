# 24. Agent Permission Model

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/resources/03-agent-identity.md](03-agent-identity.md)

## 1. Permission Levels（per §28 任务原文）

```
L0 Suggest
L1 Modify
L2 Test
L3 Commit
L4 Create MR
L5 Merge
L6 Deploy
L7 Production Operation
```

## 2. 默认边界

```
Development  L5
Staging      L6
Production   L4
```

## 3. Permission Discovery（per §27）

```bash
star agent permissions
star ide permissions
```

返回：

```json
{
  "agent": {
    "read_repository": "ALLOW",
    "read_issue": "ALLOW",
    "search_code": "ALLOW",
    "navigate_symbols": "ALLOW",
    "create_workspace": "ALLOW",
    "create_worktree": "ALLOW",
    "modify_worktree": "ALLOW",
    "run_test": "ALLOW",
    "commit": "ALLOW",
    "create_mr": "ALLOW",
    "merge_protected": "DENY",
    "deploy_production": "DENY",
    "delete_repository": "DENY",
    "change_permissions": "DENY"
  }
}
```

**关键约束**：Agent 不得"通过尝试危险操作"发现权限。必须主动查询。

## 4. 实施位置

- `crates/star-agent/src/permission.rs` — Permission 模型 + 查询
- `crates/star-policy/` — Policy enforcement (RBAC + ABAC)

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
