# Worktree Group 守门 — 实装规范

> **Status**: 🟢 Active (per 2026-09-27 Mavis 接手激活)
> **Refs**: docs/basic-design.md v0.2 §0.3 + §1.3 + §13.4
> **触发 commit**: `8133f628` (codex `docs: define Worktree Group experience`)
> **激活 branches**: `codex/worktree-group-canvas-dev` + `codex/worktree-group-domain-dev` + `codex/worktree-group-frontend-dev`

本文件是 **AGENTS.md §3.4 Worktree Group 守门** 的落地规范 (Mavis 不直接改 AGENTS.md 守门硬约束入口, 改在 docs/ 立子规范, per 守门 #1 禁回溯叙事 + AGENTS.md §0 "违反硬约束的 commit 必须 hotfix 撤回")。

## 1. Group 划分

| Group | Branch 模板 | 守门边界 |
|---|---|---|
| **canvas-group** | `codex/worktree-group-canvas-dev` | 0 改 frontend `src/`、`crates/domain-*`、`crates/api`、`crates/bff`；只动 `crates/canvas-*` + `deploy/canvas-game-k3s.yaml` + GHCR image publish workflow |
| **domain-group** | `codex/worktree-group-domain-dev` | 0 改 frontend `src/`、`crates/canvas-*`、`crates/api`、`crates/bff`；只动 `crates/domain-*` + `crates/star-*` infrastructure |
| **frontend-group** | `codex/worktree-group-frontend-dev` | 0 改 `crates/`、`bff/`、`deploy/`；只动 `frontend/src/` + frontend e2e + frontend tests |
| **core-group** (root) | `dev` / `main` | 无 Group 约束，仅顶层守门 #1+#11+#19 v19 |

## 2. Group ID 二维定位

### 2.1 env 注入

- `NEXT_PUBLIC_GROUP_ID` ∈ {`canvas`, `domain`, `frontend`, `core`} — 4 选 1
- `NEXT_PUBLIC_WORKTREE_ID` — 单层 worktree 标识 (per AppHeader wt-id-badge, PR #205 后已读)
- `NEXT_PUBLIC_MULTICA_ISSUE_ID` — Multica 备选
- **二维定位**: `group_id + worktree_id` 唯一确定 Multica spawn 的 worktree 应用上下文 (per `basic-design.md §1.3 Worker 拓扑` + `§13.4 Worker 扩展`)

### 2.2 AppHeader 显示

AppHeader wt-id-badge 已显示 `WT-<id>·<branch>` (per PR #204 / PR #205)。
Group 二维显示 改 wt-id-badge 为 `group:wt-id·branch`:
- canvas-group: `canvas:wt-1234·dev`
- domain-group: `domain:wt-1235·dev`
- frontend-group: `frontend:wt-1236·dev`
- core-group (root): `core:wt-1237·dev` 或省略 (per UI 复杂度决策)

## 3. Group-level sub-守门 (新增守门 #32/#33/#34)

### 3.1 守门 #32 v0 — Group 边界检查

```bash
# 在 group worktree 中跑:
git diff --stat $(git merge-base HEAD origin/dev)..HEAD   -- crates/ bff/ frontend/src/ deploy/

# Group 边界违规示例:
# - canvas-group PR 含 crates/domain-worktree/ 修改 → 守门 #32 FAIL
# - domain-group PR 含 crates/canvas-engine/ 修改 → 守门 #32 FAIL
# - frontend-group PR 含 crates/ 任何修改 → 守门 #32 FAIL
```

### 3.2 守门 #33 v0 — Group cross-ref 守门

- Canvas source ownership = `crates/canvas-*` + `crates/domain-canvas`；Domain source ownership = `crates/domain-*`（排除 `domain-canvas`）+ `crates/star-*`。两组 workspace package 间 Cargo path dependency 必须双向为 0。
- 此检查只覆盖上面的 Canvas/Domain 源 crate ownership；`crates/api`、`crates/application`、`crates/infrastructure`、Worktree adapters 与共享基础 crate 不通过名称推断为任一方，跨组共享依赖需由对应 API/接口契约管理。
- frontend-group 0 改 `crates/` 任何源文件，由守门 #32 的文件边界检查执行。
- **检查**: `python scripts/automation/group_guard.py check --group canvas` 或 `--group domain`。脚本读取完整 `cargo metadata --format-version 1` 的已解析 workspace 图，逐条检查两组 package 的直接依赖；metadata 缺失、执行失败或解析错误时 fail closed。`frontend` 与 `core` 显示 N/A。

### 3.3 守门 #34 v0 — Group 二维 ID 注入

- Group worktree spawn 时必设 `NEXT_PUBLIC_GROUP_ID` env
- 缺省时按现有开发约定提示 WARN 并 fallback `core`；显式值与当前 Group 不匹配时守门命令 exit 1
- AppHeader 必显示 `group:wt-id` 二维 badge (per §2.2)

## 4. 启动 Group worktree (per basic-design.md v0.2 §13.2)

```bash
# 1. 从 dev 创建 Group worktree branch
git worktree add -b codex/worktree-group-<name>-dev \
  .worktrees/wt-group-<name> origin/dev

# 2. 进 Group worktree 后设 Group ID env
cd .worktrees/wt-group-<name>
export NEXT_PUBLIC_GROUP_ID=<name>   # canvas / domain / frontend
export NEXT_PUBLIC_WORKTREE_ID=$(basename $(git rev-parse --show-toplevel))

# 3. (可选) Multica spawn 自动注入 (per ULYS-158.4)
# Multica spawn dev server 时自动设 NEXT_PUBLIC_GROUP_ID + WORKTREE_ID
```

## 5. Group worktree 实装场景示例

### 5.1 canvas-group 启动新 canvas-engine 业务

```bash
git worktree add -b codex/worktree-group-canvas-engine-gameplay \
  .worktrees/wt-canvas-engine origin/codex/worktree-group-canvas-dev
cd .worktrees/wt-canvas-engine
export NEXT_PUBLIC_GROUP_ID=canvas
export NEXT_PUBLIC_WORKTREE_ID=wt-canvas-engine

# 改 crates/canvas-engine/ + crates/canvas-game/
# 不动 frontend / crates / bff
# 实装完 push → PR → merge to codex/worktree-group-canvas-dev
# Group 分支再 merge to dev
```

### 5.2 domain-group 启动 worktree 业务

```bash
git worktree add -b codex/worktree-group-domain-worktree-v2 \
  .worktrees/wt-domain-worktree origin/codex/worktree-group-domain-dev
cd .worktrees/wt-domain-worktree
export NEXT_PUBLIC_GROUP_ID=domain
export NEXT_PUBLIC_WORKTREE_ID=wt-domain-worktree

# 改 crates/domain-worktree/ + crates/worktree-shared-dir/ (worktree service)
# 不动 frontend / canvas-* / api / bff
# 实装完 push → PR → merge to codex/worktree-group-domain-dev
# Group 分支再 merge to dev
```

## 6. 已知缺口 + 后续 PR

- (a) Group 守门 #32/#33/#34 已有 `scripts/automation/group_guard.py`；#33 实现 Cargo resolved graph 双向 cross-ref 检查，#34 的显式错配会以非零退出，未设置变量仍是开发 fallback 警告。它们不实现产品 GroupContext 授权
- (b) Multica 二维 ID 注入位未实装 (per ULYS-158.4 后续 PR — 改 frontend Multica spawn script)
- (c) Group-level Cargo workspace 拆分 (frontend / bff 已独立 + 新增 crates-canvas / crates-domain) 未实装 (估 ~30 min)
- (d) AppHeader wt-id-badge 改二维 `group:wt-id·branch` UI (估 ~20 min, frontend-group 任务)

## 7. 跨 session 续

- 任何新 Group worktree 创建必走 §4 启动流程
- 任何 Group 内改动必走 §3.1+#3.2 守门边界
- 任何跨 Group 改动必新开 issue 跨 Group coordination
