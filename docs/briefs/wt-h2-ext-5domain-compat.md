# Brief wt-h2-ext-5domain-compat: H2-EXT 5 domain 类型兼容 + workspace_ids + tenant_policy_id 字段扩展 (per HANDOFF-ST-001 §5.3 Blocker)

> **Status**: 🟡 Active (per 2026-09-09 22:31 JST 用户发令"开子代理和worktree并行处理" + ask_4b06eee1bba60b2727e8bccb 拍板 4 个 wt 并行 + 逐个 rebase + ff merge, 推荐项)
> **Created**: 2026-09-09
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **Worktree**: `wt-h2-ext-5domain-compat` (新)
> **依赖**: HANDOFF-ST-001 v0.4 §5.3 Blocker #1 H2-EXT 5 domain 类型不兼容 (DeviceId 强类型 + String→Uuid 业务语义) + #2 H2 原 3 domain service.rs 改造 (~150+ call sites)
> **跨 session 续**: per 守门 #20 v20 + 守门 #27 v27

---

## 0. 任务目标 (Objective)

在 `wt-h2-ext-5domain-compat` worktree 解决 HANDOFF-ST-001 §5.3 H2-EXT 5 domain 类型不兼容 + star_context 字段扩展 (workspace_ids / tenant_policy_id + is_platform_operator helper), 跨 5 domain (comment / identity / project / tenant / work-item) + DeviceId 强类型重构 + device_id String→Uuid 业务语义重设.

## 1. 已知事实 (Known Facts)

- **HANDOFF-ST-001 v0.4 §5.3 Blocker 5 项实证**:
  - H2-EXT #1 #5 类型不兼容: `domain-identity` device_id DeviceId 强类型 vs `domain-work-item` device_id Option<String>
  - H2 原 3 domain service.rs 改造 ~150+ call sites (feedback / validation / integration)
  - 5 域 Lead 真人到位 (Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
  - P3-B 拍板 (待 5 域 Lead 真人 + B.5 B.6 凭证拍板)
- **H2 stage 1 已落地 commit `68ae5ff`**: star_context 共享 ActorContext 字段扩展 (is_agent_session / roles / 4 helper)
- **H2 stage 2-3 尝试后 revert** (`8364223`): 3 domain port/service 改 use star_context 暴露 117+ err
- **H2-EXT 5 domain 跨域字段扩展** (per commit `9d08f80/b6f6e2a/7f611b0`): star_context 扩展 (tenant_policy_id / workspace_ids + is_platform_operator helper), 净修 507 err (797 → 290, 跨 9 crate)
- **守门 #1 v18 派生规**: H2 阶段估 ~1.1-1.6M 实测 (3-5x 超支, vs 原估 0.3-0.5M)
- **守门 #9 v3**: 调试控制台走 subprocess 替代 RPC
- **守门 #19 v19**: agent 交互 Python 化
- **守门 #13 c/d**: W/T/M 横展開, RLS 13 類必携

## 2. 路径 (Ruled-out Paths)

- ❌ **H2 3 domain 一次性完成** (跨 session 续做, 估 ~1.1-1.6M tokens, 超出单 session 上限)
- ❌ **改 5 domain 用 String 类型** (破坏 DeviceId 强类型重构目标)
- ❌ **跳过 RLS 13 類必携** (违反守门 #13 c/d)
- ❌ **主分支直接实装** (在 worktree 操作)

## 3. 范围 (Exact Scope)

### 3.1 在 worktree `wt-h2-ext-5domain-compat` 修改

```
crates/star_context/                            # 已存在 (per AGENTS.md §4.1 v16)
├── src/actor.rs                                # ActorContext 字段扩展
│   + tenant_policy_id: Uuid                    # NEW
│   + workspace_ids: Vec<Uuid>                  # NEW
│   + is_platform_operator() helper             # NEW (per H2-EXT commit b6f6e2a)
└── src/lib.rs                                  # re-exports

crates/domain-identity/                         # H2-EXT #4 DeviceId 强类型重构
├── src/types/device.rs                          # DeviceId 强类型 (newtype Uuid)
└── src/state/...                               # device_id 字段类型迁移

crates/domain-work-item/                        # H2-EXT #5 device_id String→Uuid
├── src/types/work_item.rs                      # device_id Option<Uuid>
└── src/state/...                               # 业务语义重设

crates/domain-comment/                          # H2-EXT #1 + workspace_ids
crates/domain-project/                          # H2-EXT #2 + workspace_ids
crates/domain-tenant/                           # H2-EXT #3 + tenant_policy_id

scripts/automation/h2_ext_5domain_migration.py  # 守门 #19 Python 化 (新增)
```

### 3.2 DeviceId 强类型 + Uuid 业务语义

```rust
// crates/domain-identity/src/types/device.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceId(pub Uuid);  // 强类型 newtype

impl DeviceId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
    pub fn as_uuid(&self) -> Uuid { self.0 }
}

// 业务语义迁移
// 旧: domain_work_item::device_id: Option<String>
// 新: domain_work_item::device_id: Option<DeviceId>
```

### 3.3 ActorContext 字段扩展 (per H2-EXT commit b6f6e2a)

```rust
// crates/star_context/src/actor.rs
pub struct ActorContext {
    // 已有 17 字段 (per AGENTS.md §4.1 v16)
    // ...
    // 🆕 H2-EXT 增量 2 字段 + 1 helper
    pub tenant_policy_id: Option<Uuid>,
    pub workspace_ids: Vec<Uuid>,
}

impl ActorContext {
    pub fn is_platform_operator(&self) -> bool {
        // 平台操作员 = tenant_policy_id.is_some() && roles 包含 "platform_operator"
        self.tenant_policy_id.is_some()
            && self.roles.iter().any(|r| r == "platform_operator")
    }
}
```

## 4. 验收 (Acceptance Criteria)

### 4.1 守门合规 (per AGENTS.md §4 + §4.1 v18)

| # | 守门 | 验证 |
|---|---|---|
| 1 | `cargo check --workspace --all-targets -j 4` 0 err (per 守门 #1 v2 + v19) | 实装完成后 |
| 2 | `cargo fmt --check` 0 diff | 实装完成后 |
| 3 | `cargo clippy --all-targets -- -D warnings` 0 warning (per 守门 #7) | 实装完成后 |
| 4 | `cargo test -p <affected-crate> --lib -j 4` 0 fail (per 守门 #1 v25 跳 workspace) | 实装完成后 |
| 5 | 净修 507+ err (797 → 290, 跨 9 crate) per H2-EXT 实证 | 实装完成后 |
| 6 | 5 域 Lead 真人未到位, Mavis 临时代签 (per 守门 #14 v3) | commit author=Ulysses |
| 7 | 0 强制 push / 0 跳过守门 | commit message + 历史 |

### 4.2 跨 session 续做

- **本次 session**: worktree + brief + 报告 v0.1 (Phase A 准备)
- **下次 session #1**: DeviceId 强类型重构 + domain-identity 字段迁移 (~0.3M tokens)
- **下次 session #2**: domain-work-item device_id String→Uuid 业务语义重设 (~0.3M tokens)
- **下次 session #3**: domain-comment + domain-project + domain-tenant workspace_ids 字段扩展 (~0.3M tokens)
- **下次 session #4**: cargo check 0 err + merge

### 4.3 commit message 引用 brief 路径 (per 守门 #21 v21)

```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m "feat(h2-ext): DeviceId 强类型重构 + domain-identity 字段迁移

  per docs/briefs/wt-h2-ext-5domain-compat.md (H2-EXT 5 domain 类型兼容 brief).
  ..."
```

## 5. 守门硬约束

- 守门 #1 v18: H2 阶段估 ~1.1-1.6M tokens (实测 3-5x 超支, 跨 session 续做)
- 守门 #9 v3: 调试控制台走 subprocess 替代 RPC
- 守门 #9 v19: agent 交互 Python 化
- 守门 #13 c/d: DB W/T/M 横展開 + RLS 13 類必携
- 守门 #14 v3: Mavis 永久代签
- 守门 #19 v19: agent 交互 Python 化 (走 scripts/automation/h2_ext_5domain_migration.py)
- 守门 #20 v20: 子代理 dispatch 必先 brief
- 守门 #21 v21: [P] docs 同步必更新 §4 + registry
- 守门 #27 v27 候选: 子代理 RPC 失败 fallback
- 守门 #1 v25: cargo test 改单 crate

## 6. 引用 (References)

- [HANDOFF-ST-001.md v0.4 §5.3 Blocker](../reports/HANDOFF-ST-001.md) — 5 项 Blocker 实证
- [STAR-P3-WBS-001.md v0.6 §7 阻塞 7 项](../../STAR-P3-WBS-001.md) — P3-B 启动前置
- [AGENTS.md §4 #1 v18 H2-EXT 派生规](../../AGENTS.md) — H2 阶段估 ~1.1-1.6M 实测
- [AGENTS.md §4 #16 H2-EXT 已实证 stage 1 + stage 2-3 revert](../../AGENTS.md) — 跨 session 续做证据
- [PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md v0.3 §6.2](../reports/PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md) — 19 个 fix 脚本实证
- [scripts/p0_h2_3domain_migration.py](../../scripts/automation/) — H2 真实尝试证据
- commit `68ae5ff` (H2 stage 1 star-context 扩展) + commit `8364223` (HANDOFF v0.2) + commit `9d08f80/b6f6e2a/7f611b0` (H2-EXT 5 domain)

---

**per 守门 #14 v3 Mavis 永久代签**: author = Ulysses <ulysses@mavis.local>, 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008).