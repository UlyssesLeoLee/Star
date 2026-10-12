# FSMS CLI 命令参考

> **Command**: `star session`
> **Version**: v1.0
> **Config**: `<repo>/.star/config.toml` / `~/.config/star/session.toml`

---

## 全局选项

| 选项 | 简写 | 说明 |
|------|------|------|
| `--json` | `-j` | JSON 输出 (机器可解析) |
| `--repo <path>` | `-r` | 指定仓库根目录 (默认: 向上查找 `.git`) |
| `--config <path>` | `-c` | 指定配置文件 |
| `--verbose` | `-v` | 详细日志 |
| `--help` | `-h` | 显示帮助 |

---

## 会话管理命令

### `star session start` - 创建新会话

```bash
star session start <FILE_PATH> [OPTIONS]
```

#### 参数
| 参数 | 说明 |
|------|------|
| `FILE_PATH` | 相对仓库根目录的文件路径 (必填) |

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--agent <TYPE>` | `-a` | string | 配置值 | claude/codex/opencode/hermes/custom |
| `--title <STR>` | `-t` | string | - | 会话标题 |
| `--tag <TAG>` | `-g` | string[] | [] | 标签 (可重复) |
| `--link-task-card <ID>` | | uuid | - | 关联 Task Card |
| `--link-worktree <ID>` | | uuid | - | 关联 Worktree |
| `--adapter-config <JSON>` | | string | {} | Agent 配置覆盖 |
| `--auto-worktree` | | bool | 配置值 | 创建独立 worktree |
| `--no-auto-worktree` | | bool | - | 禁用 worktree |
| `--auto-commit` | | bool | 配置值 | 自动 git commit |
| `--no-auto-commit` | | bool | - | 禁用自动 commit |

#### 示例
```bash
# 基础用法
star session start crates/star-cli/src/commands/task.rs

# 指定 agent + 标题 + 标签
star session start crates/star-cli/src/commands/task.rs \
  --agent claude \
  --title "重构 extract_args 函数" \
  --tag refactor --tag cli

# 关联 Task Card + 独立 worktree
star session start crates/star-cli/src/commands/task.rs \
  --link-task-card 01a0bf6b-486c-71b8-9bf4-92bde909c7f8 \
  --auto-worktree

# JSON 输出 (管道友好)
star session start crates/star-cli/src/commands/task.rs --json | jq .session_id
```

#### 输出 (默认)
```
✓ Session created: a1b2c3d4-e5f6-7890-abcd-ef1234567890
  File: crates/star-cli/src/commands/task.rs
  Agent: claude (v1.0.45)
  Workspace: .star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4
  Git branch: session/crates-star-cli-src-commands-task-rs/a1b2c3d4
  State: created
```

#### 输出 (--json)
```json
{
  "session_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "file_path": "crates/star-cli/src/commands/task.rs",
  "agent_type": "claude",
  "state": "created",
  "workspace_path": ".star/sessions/crates/star-cli/src/commands/task.rs/a1b2c3d4",
  "git_branch": "session/crates-star-cli-src-commands-task-rs/a1b2c3d4",
  "created_at": "2026-10-12T10:00:00Z"
}
```

---

### `star session list` - 列出会话

```bash
star session list [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--file <PATH>` | `-f` | string | - | 精确匹配文件路径 |
| `--file-prefix <PATH>` | `-p` | string | - | 前缀匹配文件路径 |
| `--agent <TYPE>` | `-a` | string | - | 过滤 agent 类型 |
| `--state <STATE>` | `-s` | string | - | 过滤状态 |
| `--tag <TAG>` | `-g` | string | - | 过滤标签 (可重复) |
| `--task-card <ID>` | | uuid | - | 关联的 Task Card |
| `--worktree <ID>` | | uuid | - | 关联的 Worktree |
| `--limit <N>` | `-l` | int | 50 | 最大结果数 |
| `--offset <N>` | `-o` | int | 0 | 分页偏移 |
| `--sort <FIELD>` | | string | -updated_at | 排序字段 |
| `--format <FMT>` | | string | table | table/json/tree |
| `--summary` | | bool | false | 包含 summary 字段 |

#### 状态值
`created` | `running` | `paused` | `completed` | `cancelled` | `archived`

#### 示例
```bash
# 所有会话 (表格)
star session list

# 指定文件的会话
star session list -f crates/star-cli/src/commands/task.rs

# 运行中的 claude 会话
star session list --agent claude --state running

# 树形视图 (按文件分组)
star session list --format tree

# JSON 输出
star session list --json | jq '.sessions[] | {session_id, file_path, state}'
```

#### 输出 (table)
```
┌──────────────────────────────────────┬────────────────────────────────────┬──────────┬──────────┬────────────────────┐
│ Session ID                           │ File Path                          │ Agent    │ State    │ Updated            │
├──────────────────────────────────────┼────────────────────────────────────┼──────────┼──────────┼────────────────────┤
│ a1b2c3d4-e5f6-7890-abcd-ef1234567890 │ crates/star-cli/src/commands/task.rs │ claude   │ running  │ 2026-10-12 10:25   │
│ e5f6g7h8-i9j0-1234-klmn-op5678901234 │ crates/star-api-rest/src/lib.rs      │ codex    │ paused   │ 2026-10-12 09:15   │
└──────────────────────────────────────┴────────────────────────────────────┴──────────┴──────────┴────────────────────┘
Total: 2 sessions
```

#### 输出 (tree)
```
crates/star-cli/src/commands/task.rs
├── a1b2c3d4 (claude, running) - "重构 extract_args 函数" [refactor, cli]
└── e5f6g7h8 (codex, archived)  - "调试 CLI 参数解析" [debug]

crates/star-api-rest/src/lib.rs
└── i9j0k1l2 (hermes, paused)   - "添加 OpenAPI 注解" [api, doc]
```

---

### `star session show` - 显示会话详情

```bash
star session show <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--include-transcript` | | bool | false | 包含完整对话记录 |
| `--include-artifacts` | | bool | false | 包含产出物详情 |
| `--transcript-limit <N>` | | int | 100 | Transcript 条数限制 |
| `--transcript-before <ID>` | | string | - | 加载更早消息 (游标) |

#### 示例
```bash
star session show a1b2c3d4-e5f6-7890-abcd-ef1234567890

# 包含 transcript
star session show a1b2c3d4 --include-transcript --transcript-limit 50

# 仅显示 meta (默认)
star session show a1b2c3d4 --json | jq .meta
```

---

### `star session resume` - 恢复会话

```bash
star session resume <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--checkpoint <N>` | `-c` | int | latest | 从指定 checkpoint 恢复 |
| `--agent <TYPE>` | `-a` | string | 原 agent | 切换 agent 类型 |
| `--adapter-config <JSON>` | | string | {} | 覆盖 adapter 配置 |
| `--background` | `-b` | bool | false | 后台运行 (输出 handle 信息) |

#### 示例
```bash
# 从最新 checkpoint 恢复
star session resume a1b2c3d4

# 从 checkpoint 2 恢复
star session resume a1b2c3d4 --checkpoint 2

# 切换到 codex 继续
star session resume a1b2c3d4 --agent codex --adapter-config '{"model":"gpt-5"}'

# 后台运行 (用于脚本集成)
star session resume a1b2c3d4 --background --json
```

#### 交互模式 (默认)
```
✓ Resuming session a1b2c3d4 (checkpoint 3)
✓ Context restored: 20,700 tokens (initial: 45,000)
✓ Agent: claude (v1.0.45) --continue --resume a1b2c3d4

────────────────────────────────────────────────────────────
Claude Code Session (type 'exit' to detach, 'checkpoint' to save)

> 继续重构 extract_args 函数...
```

---

### `star session checkpoint` - 创建检查点

```bash
star session checkpoint <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--message <STR>` | `-m` | string | "checkpoint" | 检查点描述 |
| `--auto-commit` | | bool | 配置值 | 自动 git commit |
| `--no-auto-commit` | | bool | - | 禁用自动 commit |
| `--worktree-snapshot` | | bool | false | 创建 worktree 快照分支 |

#### 示例
```bash
# 手动触发 checkpoint
star session checkpoint a1b2c3d4 -m "完成 extract_args 重构"

# 无自动 commit
star session checkpoint a1b2c3d4 --no-auto-commit
```

#### 输出
```
✓ Checkpoint 4 created
  Token delta: +2,800 (total: 23,500)
  Git commit: abc123def "chore(session): task.rs checkpoint 4 [a1b2c3d4]"
  Artifacts: 1 patch, 0 diffs
```

---

### `star session pause` - 暂停会话

```bash
star session pause <SESSION_ID> [--json]
```

> 等同于 `star session update <id> --state paused`

---

### `star session archive` - 归档会话

```bash
star session archive <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--compress` | | bool | true | 压缩 transcript |

#### 示例
```bash
star session archive a1b2c3d4
```

#### 输出
```
✓ Session archived: a1b2c3d4
  Git commit: fedcba987 "chore(session): task.rs archive a1b2c3d4"
  Compressed transcript: 150 messages → 45 KB (67% reduction)
  State: archived (read-only)
```

---

### `star session delete` - 删除会话

```bash
star session delete <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--force` | `-f` | bool | false | 物理删除 (默认软删除) |
| `--purge-git` | | bool | false | 从 git 历史彻底清除 (需管理员) |

#### 示例
```bash
# 软删除 (移入 .trash/)
star session delete a1b2c3d4

# 物理删除
star session delete a1b2c3d4 --force

# 彻底清除 git 历史 (慎用)
star session delete a1b2c3d4 --force --purge-git
```

---

### `star session fork` - 分叉会话

```bash
star session fork <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--title <STR>` | `-t` | string | "Fork of <原标题>" | 新会话标题 |
| `--tag <TAG>` | `-g` | string[] | [] | 标签 |
| `--inherit-checkpoints` | | bool | true | 继承历史 checkpoint |
| `--from-checkpoint <N>` | | int | latest | 从指定 checkpoint 分叉 |

#### 示例
```bash
star session fork a1b2c3d4 --title "方案 B: 宏重写" --tag experiment
```

---

### `star session link` - 关联会话

```bash
star session link <SOURCE_SESSION_ID> <TARGET_SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--type <TYPE>` | | string | relates | depends/relates/blocks |
| `--description <STR>` | | string | - | 关联说明 |

#### 示例
```bash
star session link a1b2c3d4 e5f6g7h8 --type depends --description "依赖提取的函数"
```

---

## Git 集成命令

### `star session sync` - Git 同步

```bash
star session sync [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--remote <NAME>` | `-r` | string | origin | 远程仓库名 |
| `--push` | | bool | true | 推送本地变更 |
| `--pull` | | bool | true | 拉取远程变更 |
| `--dry-run` | | bool | false | 仅显示将执行的操作 |
| `--session <ID>` | | uuid | all | 仅同步指定会话 |

#### 示例
```bash
# 双向同步
star session sync

# 仅推送
star session sync --push --no-pull

# 预演
star session sync --dry-run
```

#### 输出
```
✓ Sync completed (origin)
  Pushed: 3 commits (2 new sessions, 1 checkpoint)
  Pulled: 1 commit (1 new session from teammate)
  Conflicts: 0
```

---

### `star session branch` - 分支管理

```bash
star session branch <SESSION_ID> [SUBCOMMAND] [OPTIONS]
```

#### 子命令
| 子命令 | 说明 |
|--------|------|
| `list` | 列出会话相关分支 |
| `create <NAME>` | 创建新分支 |
| `switch <NAME>` | 切换分支 |
| `delete <NAME>` | 删除分支 |
| `merge <TARGET>` | 合并到目标分支 |

#### 选项
| 选项 | 说明 |
|------|------|
| `--strategy <STRAT>` | merge/rebase/squash (merge 时) |

#### 示例
```bash
star session branch a1b2c3d4 list
star session branch a1b2c3d4 create feature/async-refactor
star session branch a1b2c3d4 merge main --strategy squash
```

---

### `star session export` - 导出会话

```bash
star session export <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--format <FMT>` | `-f` | string | markdown | markdown/json/patch/mr/bundle |
| `--output <PATH>` | `-o` | path | stdout | 输出文件路径 |
| `--include-transcript` | | bool | true | 包含 transcript |
| `--include-artifacts` | | bool | true | 包含 artifacts |
| `--redact` | | bool | true | 脱敏 secrets |

#### 示例
```bash
# 导出为 Markdown 报告
star session export a1b2c3d4 -f markdown -o /tmp/report.md

# 导出为 MR 描述
star session export a1b2c3d4 -f mr

# 导出完整 bundle (zip)
star session export a1b2c3d4 -f bundle -o /tmp/session.zip
```

---

### `star session mr` - 创建 Merge Request

```bash
star session mr <SESSION_ID> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--title <STR>` | `-t` | string | 基于 session 标题 | MR 标题 |
| `--target-branch <BRANCH>` | `-b` | string | main | 目标分支 |
| `--draft` | | bool | true | 创建为 Draft |
| `--reviewers <USER>` | | string[] | [] | 评审人 |
| `--labels <LABEL>` | | string[] | [] | 标签 |
| `--auto-merge` | | bool | false | 通过后自动合并 |

#### 示例
```bash
star session mr a1b2c3d4 \
  --title "feat: 重构 task.rs extract_args" \
  --reviewers alice,bob \
  --labels refactor,cli
```

#### 输出
```
✓ MR created: https://github.com/org/repo/pull/123
  Branch: session/crates-star-cli-src-commands-task-rs/a1b2c3d4
  Draft: true
  Reviewers: alice, bob
```

---

## 适配器管理命令

### `star session adapter list` - 列出适配器

```bash
star session adapter list [--json]
```

#### 输出
```
┌────────────┬────────┬──────────┬──────────┬────────────────────────────┐
│ Name       │ Type   │ Version  │ Status   │ Capabilities               │
├────────────┼────────┼──────────┼──────────┼────────────────────────────┤
│ claude     │ claude │ 1.0.45   │ healthy  │ spawn,attach,checkpoint... │
│ codex      │ codex  │ 0.12.3   │ healthy  │ spawn,attach,export        │
│ opencode   │ opencode│ 0.8.1   │ degraded │ spawn,attach (no ckpt)     │
│ hermes     │ hermes │ 0.5.0    │ healthy  │ spawn,attach,checkpoint... │
└────────────┴────────┴──────────┴──────────┴────────────────────────────┘
```

---

### `star session adapter add` - 安装自定义适配器

```bash
star session adapter add <PATH> [OPTIONS]
```

#### 参数
| 参数 | 说明 |
|------|------|
| `PATH` | WASM 文件 / 动态库 / 目录路径 |

#### 选项
| 选项 | 简写 | 类型 | 默认 | 说明 |
|------|------|------|------|------|
| `--name <STR>` | `-n` | string | 目录名 | 适配器名称 |
| `--type <TYPE>` | | string | custom | 类型标识 |

#### 示例
```bash
# 安装 WASM 适配器
star session adapter add ./my-custom-adapter.wasm --name my-agent

# 安装 Rust 动态库
star session adapter add ./target/release/libmy_adapter.so --name my-agent
```

---

### `star session adapter remove` - 移除适配器

```bash
star session adapter remove <ADAPTER_NAME> [--force]
```

---

### `star session adapter configure` - 配置适配器

```bash
star session adapter configure <ADAPTER_NAME> [OPTIONS]
```

#### 选项
| 选项 | 简写 | 类型 | 说明 |
|------|------|------|------|
| `--key <KEY>` | `-k` | string | 配置键 |
| `--value <VALUE>` | `-v` | string | 配置值 |
| `--reset` | | bool | 重置为默认 |

#### 示例
```bash
star session adapter configure claude --key default_model --value claude-3-5-sonnet
star session adapter configure claude --key default_max_tokens --value 16384
```

---

## 配置命令

### `star session config show` - 显示配置

```bash
star session config show [--json] [--global]
```

---

### `star session config set` - 设置配置

```bash
star session config set <KEY> <VALUE> [--global]
```

#### 常用配置键
| Key | 类型 | 默认 | 说明 |
|-----|------|------|------|
| `git.auto_commit` | bool | true | 自动 commit |
| `git.worktree_isolation` | bool | true | Worktree 隔离 |
| `git.remote_sync_interval` | int | 300 | 同步间隔(秒) |
| `storage.backend` | string | filesystem | filesystem/sqlite/s3 |
| `storage.artifact_store` | string | - | S3 artifact store URL |
| `privacy.enabled` | bool | true | 隐私过滤 |
| `privacy.custom_rules` | string | - | 自定义规则文件 |
| `defaults.agent_type` | string | claude | 默认 agent |
| `defaults.token_budget` | int | 50000 | 默认 token 预算 |

#### 示例
```bash
star session config set git.auto_commit true
star session config set defaults.agent_type codex
star session config set privacy.custom_rules .star/privacy-rules.yaml --global
```

---

### `star session config reset` - 重置配置

```bash
star session config reset [KEY] [--global]
```

---

## 高级命令

### `star session rollback` - 回滚到检查点

```bash
star session rollback <SESSION_ID> --to-checkpoint <N> [--force]
```

> 将会话状态回退到指定 checkpoint，丢弃后续变更

---

### `star session promote` - 升级为 AgentSession

```bash
star session promote <SESSION_ID> [--title <STR>] [--json]
```

> 将 FileSession 升级为平台级 AgentSession (PG 持久化，跨设备同步)

---

### `star session migrate` - 数据迁移

```bash
star session migrate [--from-version <V>] [--to-version <V>] [--dry-run]
```

> Schema 版本升级工具

---

### `star session doctor` - 健康检查

```bash
star session doctor [--fix] [--json]
```

#### 检查项
- Git 仓库状态
- `.star/sessions/` 目录完整性
- Registry/Manifest 一致性
- Adapter 可用性
- 磁盘空间
- 权限

#### 输出
```
✓ Git repository: clean
✓ Sessions directory: 42 sessions, 12 MB
✓ Registry: valid (42 entries)
✓ Manifests: all consistent
✓ Adapters: 4/4 healthy
✓ Disk space: 45 GB free
⚠ Privacy rules: 3 custom rules loaded
```

---

## 环境变量

| 变量 | 说明 |
|------|------|
| `STAR_SESSION_REPO` | 默认仓库路径 |
| `STAR_SESSION_CONFIG` | 默认配置文件路径 |
| `STAR_SESSION_DEFAULT_AGENT` | 默认 agent 类型 |
| `STAR_SESSION_AUTO_COMMIT` | 默认自动 commit (true/false) |
| `STAR_SESSION_TOKEN_BUDGET` | 默认 token 预算 |
| `CLAUDE_API_KEY` / `CODEX_API_KEY` / `OPENCODE_API_KEY` / `HERMES_API_KEY` | Agent 凭证 |

---

## 退出码

| 码 | 含义 |
|----|------|
| 0 | 成功 |
| 1 | 通用错误 |
| 2 | 参数错误 |
| 3 | 会话不存在 |
| 4 | 状态机非法迁移 |
| 5 | Git 操作失败 |
| 6 | Adapter 不可用/启动失败 |
| 7 | 存储后端错误 |
| 8 | 权限不足 |
| 9 | 配置错误 |
| 10 | 版本不兼容 |

---

## 补全脚本

### Bash
```bash
# 生成补全
star session --generate-completion bash > /usr/share/bash-completion/completions/star-session

# 或临时加载
source <(star session --generate-completion bash)
```

### Zsh
```bash
star session --generate-completion zsh > ~/.zsh/completions/_star-session
```

### Fish
```bash
star session --generate-completion fish > ~/.config/fish/completions/star-session.fish
```

### PowerShell
```powershell
star session --generate-completion powershell | Out-String | Invoke-Expression
```

---

## 别名建议 (加入 `.bashrc` / `.zshrc`)

```bash
alias sss='star session start'
alias ssl='star session list'
alias ssshow='star session show'
alias ssr='star session resume'
alias ssc='star session checkpoint'
alias ssa='star session archive'
alias ssd='star session delete'
alias ssf='star session fork'
alias ssync='star session sync'
alias ssx='star session export'
alias ssmr='star session mr'
```