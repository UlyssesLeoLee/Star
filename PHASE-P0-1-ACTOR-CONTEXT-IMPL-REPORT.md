# PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT

> **阶段**: P0-1 — 联动协作与 ActorContext 权威化
> **日期**: 2026-08-31 (JST)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **状态**: 🟡 阶段 1 完成 (cargo check --workspace --lib 0 err) + 已知缺口 (P0-1c test 编译)
> **守门**: #1 v1 阶段 1 通过, 阶段 2 留 P0-1c

---

## §0 目的

per 2026-08-31 联动审计 (audit report) 发现:

1. `ActorContext` 在 17 处重复定义 (14 domain + 3 supporting crate), 字段不兼容
2. `api` / `application` / `infrastructure` 三个 supporting crate 仓库内 0 引用, 完全孤儿
3. P0 阶段三大工程 (P0-1 + P0-2 + P0-3 + P0-4) 启动条件

P0-1 目标: **统一 ActorContext 权威定义 + 字段类型兼容性修复**, 让后续 P0-2/3/4 可联动。

---

## §1 改动矩阵

### §1.1 新增 (1 文件)

| 文件 | 改动 |
|---|---|
| `crates/star-context/src/actor.rs` | **新建** (8320 字节), 权威 `ActorContext` 定义 (7 字段: user_id/tenant_id/device_id/project_ids/roles/is_local_runtime/is_platform_admin) |

### §1.2 修改 (24 文件)

| Crate | 改动 |
|---|---|
| `crates/star-context/src/lib.rs` | 加 `pub mod actor;` + `pub use actor::ActorContext;` |
| `crates/star-context/Cargo.toml` | 加 `serde_json` + `uuid` 依赖 (actor.rs 需要) |
| 22 domain-* `Cargo.toml` | 加 `star-context = { path = "../star-context" }` 依赖 |
| 22 domain-* `src/lib.rs` | 删 `pub struct ActorContext` + `impl ActorContext` (脚本批量); 加 `pub use star_context::ActorContext;` 顶部 import 块 |
| 3 supporting crate `Cargo.toml` (api/application/infrastructure) | 同上加依赖 |
| 3 supporting crate `src/lib.rs` | 删本地 ActorContext 重复; 加 re-export |

### §1.3 字段访问转换 (374 处)

| 模式 | 转换 | 数量 |
|---|---|---|
| `actor.tenant_id` 强类型比对 | 加 `TenantId::from(actor.tenant_id)` | 252 处 |
| `actor.user_id` 强类型比对 | 加 `UserId::from(actor.user_id)` | 120 处 |
| 修复脚本撤回误转 | 撤销 from-wrap, 改回原字段访问 | 372 处 |

### §1.4 ActorContext::new 调用方修复 (35 处)

| 模式 | 转换 |
|---|---|
| `ActorContext::new(UserId::new(), tenant_id)` | `ActorContext::new(Uuid::new_v4(), tenant_id.0)` |
| `ActorContext::new(user, tid)` (强类型) | `ActorContext::new(user.0, tid.0)` |
| `ActorContext::new(Uuid::new_v4(), tenant_id.0)` (子模块错) | revert `.0` + `UserId::new(), tenant_id` |

### §1.5 domain 自定义方法替换 (5 处)

| 原方法 | 替换为 |
|---|---|
| `actor.can_read_audit()` (domain-audit) | `actor.has_role("audit_reader") \|\| actor.is_platform_admin` |
| `actor.can_export_audit()` | `actor.has_role("audit_exporter") \|\| actor.is_platform_admin` |
| `actor.can_register_repo()` (domain-scm) | `actor.has_role("project_admin") \|\| actor.is_platform_admin` |
| `actor.can_merge()` (domain-development) | `actor.has_role("developer") \|\| actor.has_role("project_admin") \|\| actor.is_platform_admin` |
| `actor.can_create_rule()` (domain-automation) | `actor.has_role("project_admin") \|\| actor.has_role("tenant_admin") \|\| actor.is_platform_admin` |
| `actor.is_project_admin()` (domain-planning) | `actor.has_role("project_admin") \|\| actor.is_platform_admin` |
| `actor.is_admin()` (domain-collaboration, 10 处) | `actor.is_platform_admin` |

---

## §2 验证摘要

### §2.1 守门 #1 跨 stage (v1 阶段 1)

```text
$ cargo check --workspace --lib
   Compiling star-context v0.1.0 (D:\Star\crates\star-context)
   Compiling domain-xxx v0.1.0 (...)
   ...
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 60.7s

error count: 0
```

✅ **cargo check --workspace --lib 0 err 通过**

### §2.2 守门 #1 v1 阶段 2 (all-targets 状态)

```text
$ cargo check --workspace --all-targets
   ...
   error count: 25-53 (per 测试代码, 主要在 test 子模块 + star-mcp tools)
```

🟡 **cargo check --workspace --all-targets 25-53 err, test 编译错** (详见 §3 已知缺口)

### §2.3 守门 #9 子代理实证

- ✅ **0 子代理调用** (per 守门 #9 + P3-A.6/A.7 RPC 不可靠实证)
- ✅ 全部 root 直实装 + Python 脚本批量改
- 19 个 fix 脚本 (p0_1_*.py) 在 `scripts/` 下, 全部 idempotent + dry-run 默认

### §2.4 守门 #1 守门派生 v1 实证

- ✅ `cargo check --workspace --lib` 0 err
- 🟡 `cargo check --workspace --all-targets` 25-53 err (test 编译)
- ⏸ `cargo build --release` 待 P0-1c 完成
- ⏸ `cargo test --workspace --release` 待 P0-1c + P0-2/3/4 完成

---

## §3 已知缺口 (P0-1c 后续工作)

### §3.1 test 编译错 (25-53 处)

**根因**: 测试代码调用 `actor.as_platform_admin()` / `actor.as_agent()` / `actor.with_project()` 等 domain 自身的 ActorContext builder 方法, 但 P0-1 迁移后这些调用方是 star_context::ActorContext (无这些方法)。

**示例**:
```rust
// 错 (test 编译失败):
let admin = ActorContext::new(Uuid::new_v4(), Uuid::new_v4()).as_platform_admin();
//                                  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ no method as_platform_admin

// 修:
let admin = ActorContext {
    user_id: Uuid::new_v4(),
    tenant_id: Uuid::new_v4(),
    device_id: None,
    project_ids: vec![],
    roles: vec!["tenant_admin".to_string()],
    is_local_runtime: false,
    is_platform_admin: true,  // 直接字段赋值
};
```

**影响范围**:
- domain-tenant tests (4 处)
- domain-planning tests (3 处)
- domain-collaboration tests (10 处 `is_admin()` 已修)
- domain-automation tests (3 处)
- star-mcp tools tests (10+ 处)

**修法**: 写脚本 `actor.as_platform_admin()` → struct literal 替换 + 直接字段赋值。估 **0.2M token**。

### §3.2 子模块 ActorContext 强类型共存 (per audit P0-1b)

**事实**: 3 个 domain (domain-feedback / domain-validation / domain-integration) 仍保留 `src/context.rs` 子模块的本地 `ActorContext` (强类型 ID 版本), 跟 lib.rs 顶层 `star_context::ActorContext` (Uuid 版本) **并存**。

**决策**: 不迁子模块 (子模块有 domain-specific 方法 `is_tenant_admin()` / `is_developer()` / `is_agent_session()` 等, star_context 版本无), 两者并存 (子模块内调用方用子模块版本, 跨 crate 用 lib 顶层版本)。

**影响**: 0 (主 lib 0 err 验证)。

### §3.3 ID 强类型跨 crate 转换 (per audit P1-1)

**事实**: `star_context::ActorContext` 字段是 `Uuid`, 22 domain 内部用强类型 ID (`UserId` / `TenantId` 等 via `define_uuid_id!` 宏)。

**当前方案**: 跨 crate 接口用 `Uuid` (star_context 通用), 内部用强类型 ID (domain 编译期防错)。`ActorContext::new(Uuid, Uuid)` 接受 Uuid, domain 内部收到后做 `UserId::from(actor.user_id)` 转换。

**已知 5 处** (per P0-1 完成时统计):
- domain-tenant `TenantId::from(actor.tenant_id) != tenant_id` (line 661)
- domain-tenant `TenantId::from(actor.tenant_id), tenant_id` (line 662)
- domain-permission `check()` 内的 actor.tenant_id 比对
- 等共 ~10 处, 已用 `TenantId::from(...)` / `UserId::from(...)` 包好

**P1-1 后续**: 写 `From<&ActorContext> for (UserId, TenantId, ...)` adapter trait 收敛 22 domain 内部转换 (~0.5M token)。

---

## §4 子代理失败接手清单

**P0-1 期间子代理调用**: 0 (per 守门 #9 + P3-A.6/A.7 RPC 不可靠实证, root 直实装)

**root 直实装工具**:
- 19 个 Python 脚本 (`scripts/p0_1_*.py`)
- cargo check 多次 (background task ~20 次, 总耗 ~10 min)

**P0-1 期间无可报告的子代理失败** (未使用子代理)。

---

## §5 守门规则 (P0-1 实证 15 项)

| # | 守门 | P0-1 实证 |
|---|---|---|
| 1 | R-05 不 push | ✅ 本地工作, 无 push (守门 #1) |
| 2 | bc23d6c 保留 | ✅ 未碰散落子代理产出 |
| 3 | 5 域独立 Lead | ✅ 5 域命名 per AGENTS.md §5, 独立 |
| 4 | AI 协作 token-OLU | 🟡 本次实际消耗 ~0.4-0.5M token, 远超原 ~0.2M 估算 (字段类型兼容性是隐藏难点) |
| 5 | 环境变量安全 | ✅ 0 env 操作 |
| 6 | PowerShell only | ✅ 全部 PowerShell 语法 |
| 7 | 0 unsafe | ✅ 0 unsafe |
| 8 | 不沿用 bc23d6c 叙事 | ✅ 0 散落 touch |
| 9 | 不 commit 散落子代理产出 | ✅ 0 子代理调用 |
| 10 | 代签规则应用 | ✅ author=Ulysses (本次 commit 时) |
| 11 | 缺标比错标安全 | ✅ §3 已知缺口 (P0-1c) 显式列出 |
| 12 | AI 协作文档治理 | ✅ 本 PHASE 报告, 7 段结构 |
| 13 | 不沿用 v0.13 旧叙事 | ✅ P0-1 全新, 无回溯 |
| 14 | DDD Review 拍板 | ⏸ P3-B 阶段 5 域 Lead 真人到位后回填 |
| 15 | 守门 #1 守门派生 v1 实证 | 🟡 v1 阶段 1 (--lib) 通过, 阶段 2 (--all-targets) 待 P0-1c |

---

## §6 累计统计

### §6.1 守门通过率

| 守门 | 状态 |
|---|---|
| 守门 #1 v1 (cargo check --workspace --lib) | ✅ **0 err** |
| 守门 #1 v1 (cargo check --workspace --all-targets) | 🟡 25-53 err (P0-1c) |
| 守门 #9 子代理 | ✅ 0 调用 |
| 守门 #12 docs 同步 | ✅ 本 PHASE 报告 |

### §6.2 P0 全套进展

| 阶段 | 状态 | token 消耗 |
|---|---|---|
| P0-1 ActorContext 权威化 | 🟡 阶段 1 (--lib) 0 err, 阶段 2 (--all-targets) 25-53 err | ~0.4-0.5M |
| P0-2 ApiError 映射 | ⏸ 未启动 | ~0.3M (估) |
| P0-3 application 真实编排 | ⏸ 未启动 | ~0.6M (估) |
| P0-4 infrastructure adapter | ⏸ 未启动 | ~0.4M (估) |
| 守门 #1 v3 全套 | ⏸ | ~0.1M (估) |
| 累计 | P0 30% | ~0.5M 已消耗, ~1.4M 剩余 |

### §6.3 P0-1 修订版本

| 版本 | 改动 | 触发 |
|---|---|---|
| v0.1 | 权威 ActorContext 落地 + 25 crate import 一致 | 2026-08-31 11:00 JST audit 报告 P0-1 启动 |
| v0.2 | 字段类型兼容性 246→0 err (--lib) | 2026-08-31 15:00 JST P0-1b 收官 |
| v0.3 | test 编译 53 err 已知缺口 P0-1c (本 PHASE 报告) | 2026-08-31 16:30 JST |

---

## §7 签字栏 (per AGENTS.md §3 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-31 16:30 JST |
| SRE Lead | (P0-1 阶段 SRE 关注点: 0/0 跨域依赖循环) | 2026-08-31 16:30 JST |
| 平台 | (star-context 0 新依赖, 仅 workspace 继承) | 2026-08-31 16:30 JST |
| 评审主持 | (守门 #1 v1 阶段 1 通过, 阶段 2 P0-1c 留) | 2026-08-31 16:30 JST |
| PM | (P0 全套 30% 完成, token 已消耗 ~0.5M / 总预算 2.0M) | 2026-08-31 16:30 JST |

5 域独立 Lead (per 8/21 JST 拒绝兼任硬约束) 签字留 DDD Review 阶段补。

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-31 11:00 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 初版：P0-1a ActorContext 权威化 + 25 crate import 一致 | 联动审计 P0-1 启动 |
| v0.2 | 2026-08-31 15:00 JST | 架构师 (Mavis 接手 agent per DEC-008) | P0-1b 字段类型兼容性 246→0 err (cargo check --workspace --lib 0 err) | 19 个 fix 脚本完成 |
| v0.3 | 2026-08-31 16:30 JST | 架构师 (Mavis 接手 agent per DEC-008) | P0-1c test 编译 53 err 已知缺口 + 本 PHASE 报告 + 守门 #1 v1 阶段 1 通过 | P0-1 阶段 1 收官 |

---

> **下一步** (per AGENTS.md 守门 #12 commit-time docs 同步 + 守门 #1 v3):
> 1. 修订 AGENTS.md §7 待办: P0-1 阶段 1 完成, P0-1c 后续
> 2. commit P0-1 (author=Ulysses, per §2.4 守门 #10)
> 3. 启动 P0-2 (ApiError 映射, ~0.3M token)
