# 25. IDE Permission Model

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/resources/05-agent-permission-model.md](05-agent-permission-model.md) · [spec/resources/04-ide-session-identity.md](04-ide-session-identity.md)

## 1. IDE 端权限

IDE 自身有自己的权限（如 VS Code workspace trust）。STAR 不重复定义，但**映射**：

| IDE 权限 | STAR 权限 |
|---|---|
| workspace trust granted | read_repository ALLOW |
| extension permission | n/a (extension 不进 STAR) |
| terminal allow shell | run_command ALLOW |
| file write to workspace | modify_worktree ALLOW |

## 2. Permission Discovery

```bash
star ide permissions
```

返回结构与 agent permissions 平行（per spec/resources/05 §3）。

## 3. 实施位置

- `crates/star-ide/src/permission.rs`

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
