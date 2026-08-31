# Q&A-ST-001 — ST 测试中的问题汇总 (待上游 AI 回答)

> **阶段**: ST-001 后续 — 问题汇总 + 上游 AI 解答请求
> **日期**: 2026-08-31 (JST)
> **作者**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签
> **触发**: 2026-08-31 20:30 JST 用户发令"测试中的问题汇总到 Q&A 让上游 AI 回答"
> **范围**: ST-001 轮次 (commit 15e029b) + P0-1 IT/ST 过程中遇到的所有问题

---

## §0 目的

per 用户发令"测试中的问题汇总到 Q&A 让上游 AI 回答":
- 把 ST 测试中遇到的所有**设计性 / 架构性 / 实施性**问题结构化记录
- 让上游 AI (Mavis 之外的高层 AI) 回答 / 决策
- 写 Q&A 入仓 + commit, 作为 P0-2/3/4 启动前的决策依据

问题分类:
- **D 类 (Design)**: 架构设计冲突, 顶层决策
- **I 类 (Implementation)**: 实施细节, 编码模式
- **T 类 (Test)**: 测试相关, IT/ST 用例
- **P 类 (Process)**: 流程 / 守门 / 文档

每条 Q&A 格式:
```
Q[N]-[类]: 问题描述
A[N]: 上游 AI 回答 (待回答) | 基于 audit/AGENTS.md 的参考回答
```

---

## §1 架构设计冲突 (D 类)

### Q1-D: 5 域 (AGENTS.md §5) vs 4 域 (DDD bounded context) 命名不一致

**现象** (per audit P0-1):
- AGENTS.md §5 "5 域独立 Lead" 指 5 个业务子域: `player / economy / match / social / admin` (来自 RGS 仓)
- Star 仓实际 22+ `domain-*` crate 是 DDD bounded context: `identity / permission / work-item / workspace / ...`
- `crates/star-mcp/src/handlers/` 22+ handler, `crates/star-saga/src/lib.rs:32` 定义 `enum Domain { Player, Economy, Match, Social, Admin }` (star-saga)

**困惑**:
- 守门 #3 "5 域独立 Lead" 指的是 5 域 (player/economy/...) 还是 22 domain (DDD)?
- ST-2 5 域独立验证只能验 4 域 (identity/permission/workspace/worktree) — domain-context 子模块有但 star-mcp dev-dep 没加
- 5 域 vs 4 域 mapping (player → ? / economy → ? / match → ? / social → ? / admin → ?)

**A1 (待回答)**: 见 §5 决策请求

---

### Q2-D: `domain_xxx::ActorContext` 顶层 re-export vs port.rs 内部 `use crate::context::ActorContext` 冲突

**现象** (per P0-1 联动审计 + ST 实施):
- P0-1 收敛: `crates/domain-X/src/lib.rs` 顶层 `pub use star_context::ActorContext;` (7 字段, Uuid 弱类型)
- 3 个 domain (feedback / validation / integration) 保留子模块 `pub mod context { pub struct ActorContext { ... } }` (强类型 ID, 有 domain-specific 字段如 `is_agent_session`)
- port.rs / service.rs 内部 `use crate::context::ActorContext;` (强类型版) — port trait method 期待强类型
- star-mcp handler `use domain_feedback::context::ActorContext;` 跟 port trait 匹配
- 但跨 crate 通用 interface 是顶层 `pub use star_context::ActorContext;` (Uuid)

**困惑**:
- 跨 crate 调用方应该用哪个? 顶层 (Uuid) 还是子模块 (强类型)?
- port trait 应该用哪个? 顶层 (Uuid) 还是子模块 (强类型)?
- 设计意图: "跨 crate interface 用 Uuid 通用, 内部 service 用强类型" 是好设计吗?
- 还是"全部统一用顶层 Uuid, domain 内部自己转强类型"更好?

**A2 (待回答)**: 见 §5 决策请求

---

### Q3-D: `define_uuid_id!` 宏字段可见性 (pub vs 私有)

**现象** (per P0-1c):
- 22 domain 的 `define_uuid_id!` 宏定义: `pub struct $name(uuid::Uuid);` (字段私有)
- `value_object::UserId(uuid::Uuid::new_v4())` tuple 构造需要字段 pub
- 我 P0-1c 改 `pub struct $name(pub uuid::Uuid);` 字段 pub, 修了 cargo 编译错
- star_context::ActorContext 原本就 `pub struct ActorContext { pub user_id: Uuid, ... }` 字段 pub

**困惑**:
- 强类型 ID 字段 pub 是不是 Rust DDD 实践?
- pub 暴露内部 Uuid 字段会不会破坏封装?
- 22 domain 全改字段 pub 是工程上必须的, 还是过度开放?

**A3 (待回答)**: 见 §5 决策请求

---

## §2 实施细节 (I 类)

### Q4-I: 强类型 ID 跨 crate 转换: `UserId::from(Uuid)` vs `UserId(Uuid)` tuple 构造 vs `as_uuid()`

**现象** (per IT-1/IT-2/ST-2 实施):
- `UserId::from(uuid)` 通过 `impl From<Uuid> for UserId` 转换
- `UserId(uuid)` tuple struct 构造, 需要字段 pub
- `user.as_uuid()` 返回 `&Uuid` (在 workspace 大多数 domain) 或 `Uuid` Copy (domain-identity)
- `&user.as_uuid()` deref 后比较 vs `user.as_uuid()` 直接比较 (类型不一致)
- cargo 报 "Uuid vs &Uuid" / "type Uuid cannot be dereferenced" 各种类型错

**困惑**:
- 22 domain 的 `as_uuid()` 返回类型不一致 (有的 `Uuid`, 有的 `&Uuid`) — 应该统一吗?
- 跨 crate 调用方应该用 `from(Uuid)` 还是 `tuple struct 构造` 还是 `as_uuid()`?
- 哪种是 best practice?

**A4 (待回答)**: 见 §5

---

### Q5-I: cargo `_unused_user` 自动生成函数 + 私有字段检测交互

**现象** (per ST-2 实施):
- 当 `value_object::UserId` 在 service.rs import 但未使用时, cargo 自动生成 `_unused_user(_: UserId) -> UserId { uuid::Uuid::new_v4() }` 函数检测 unused imports
- 函数体期望返回 `UserId` 但实际用 `Uuid::new_v4()` — cargo 报 "field is private" 建议 `value_object::UserId(uuid::Uuid::new_v4())`
- 但 `value_object::UserId` 字段是 `pub` (我 P0-1c 改过), 仍然报错?
- 实际: domain-validation `macros.rs` 字段已 pub, 但 `cargo _unused_user` 仍然报 private field 错

**困惑**:
- `cargo _unused_user` 是 cargo 的 dead-code 检测还是 unused-imports 检测?
- 为什么 `pub` 字段仍然被报 private?
- 怎么正确解决? `#[allow(unused_imports)]` 还是真用 `UserId`?

**A5 (待回答)**: 见 §5

---

### Q6-I: star-mcp handler 调 domain service 时 ActorContext 类型选择

**现象** (per P0-1c 实施):
- workspace / worktree / agent / search 等: 顶层 re-export `pub use star_context::ActorContext;` (Uuid)
- feedback / validation / integration: 子模块 `pub use context::ActorContext;` (强类型)
- port.rs trait method `actor: &ActorContext` (强类型, 引用 crate 自己的)
- star-mcp handler 调 service 时: 
  - 强类型域 (feedback) → `use domain_xxx::context::ActorContext;` + `UserId::new(), TenantId::new()` 构造
  - Uuid 域 (workspace) → `use domain_xxx::ActorContext;` + `Uuid::new_v4()` 构造

**困惑**:
- 22 domain 内部 trait method 应该统一用强类型还是 Uuid?
- 如果用强类型: port trait method 字段强类型, star-mcp handler 也要用强类型 (跨 crate interface 强类型)
- 如果用 Uuid: port trait method 字段 Uuid, 跨 crate interface Uuid, domain 内部 service 转强类型
- 当前混用 (3 domain 强类型 + 19 domain Uuid) 是好还是坏?

**A6 (待回答)**: 见 §5

---

## §3 测试相关 (T 类)

### Q7-T: IT-2 期望 `CrossTenantDenied` 但 service 第一行返回 `PermissionDenied` (role 必要)

**现象** (per IT-2 实施):
- domain_identity::InMemoryIdentityService::create_user 第一行:
  ```rust
  if !actor.is_platform_admin && !actor.has_role("tenant_admin") {
      return Err(IdentityError::PermissionDenied);
  }
  ```
- 我的 IT-2 actor 是 `StarActorContext::new(uuid, uuid)` 默认 role = "developer" — 触发第一行 PermissionDenied
- 期望 CrossTenantDenied 实际 PermissionDenied — 测试 fail
- 修: actor 加 `.with_role("tenant_admin")` 越过第一行

**困惑**:
- service 校验逻辑是否合理? 第一行 PermissionDenied, 第二行 CrossTenantDenied — 用户无 tenant_admin role 怎么会触发跨 tenant 检查?
- 这是**有意**防御性 (避免暴露跨 tenant 信息) 还是**无意识**代码顺序问题?
- IT 测试应该 (a) 严格按 service 行为期望, 还是 (b) 跳过 PermissionDenied 直接测跨 tenant?

**A7 (待回答)**: 见 §5

---

### Q8-T: ST 报告的"5 域"实际只能验 4 域 (domain-context 缺 dev-dep)

**现象** (per ST-2 实施):
- AGENTS.md §5 5 域: `player / economy / match / social / admin` (业务子域, 来自 RGS 仓, 跟 Star 仓**不引用**)
- Star 仓实际 DDD bounded context: identity / permission / work-item / workspace / worktree / context (5 域候选)
- ST-2 5 域独立验证只能验 4 域 (identity/permission/workspace/worktree) — domain-context 有子模块但 star-mcp dev-dep 没加

**困惑**:
- ST-2 应该按 AGENTS.md §5 "5 域" 验还是按 DDD 5 域验?
- 如果按 DDD 5 域, "5 域独立" 是不是 SAAS 多租户 (per 5 域独立 Lead)?
- ST-2 报告 "5 域" 字眼会不会误导?

**A8 (待回答)**: 见 §5

---

### Q9-T: cargo check `--lib` vs `--all-targets` 区别

**现象** (per ST-5 实证):
- `cargo check --workspace --lib`: 0/1 err (P0-1c 残余) — 只检查 lib (不含 test)
- `cargo check --workspace --all-targets`: 170 err — 含 test / bin / examples / benches
- `cargo test --workspace --lib`: test 代码用 `#[cfg(test)]` 块, **不**包含在 `--lib` 检查中
- IT 32/32 pass 是因为 `cargo test -p star-context --test it_actor_context` 单独跑那个 test 文件, 跟 workspace `--lib` 无关

**困惑**:
- 守门 #1 v1 严格定义"cargo check --workspace --all-targets 0 err", 但实际 170 err 阻塞
- 是否应该 (a) 接受 `--lib` 0 err 作为 P0-1 完成标志, 还是 (b) 必须 `--all-targets` 0 err 才算 P0-1 完成?
- 170 err 主要是 test 代码 (P0-1b 撤销残留), 跟 P0-1 实质联动无关, 是否应该分开处理?

**A9 (待回答)**: 见 §5

---

## §4 流程 / 守门 / 文档 (P 类)

### Q10-P: P0 全套 token 预算超出 vs 子阶段继续

**现象** (per ST-3 实证):
- P0-1 单独 ~1.0M token (per audit 估 0.2M, 实际 5x)
- P0 全套预算 2.0M token, P0-1 已占 50%
- P0-2/3/4 估 1.4M token, 实际可能 2.0M+ (P0-1 经验)
- 单 session token 限制 (model context window)

**困惑**:
- 是否继续 P0-2/3/4 (估 1.4-2.0M token)?
- 还是接受 P0-1 30% 完成 + 跨 session 续 P0-2/3/4?
- token 预算超出时, 应该 (a) 暂停 + 用户决策, (b) 自动继续, (c) 缩减 P0-2/3/4 范围?

**A10 (待回答)**: 见 §5

---

### Q11-P: ST 测试 vs IT 测试 vs 单元测试 边界

**现象** (per ST-001 实施):
- 单元测试 (P3-A 守门 v18): 1384+ tests pass (cargo test --workspace --lib)
- 集成测试 IT (P0-1c 轮次): 24/24 pass (star-context + star-mcp 跨 crate)
- 系统测试 ST (ST-001 轮次): 32/32 pass (IT-1 + IT-2 + ST-2)
- 但 `--all-targets` 170 err, 实际跑不到"系统"级别

**困惑**:
- ST 测试范围是什么? 应该测"系统"还是"跨 crate"?
- ST 测试跟 IT 测试区别在哪? (IT 已测跨 crate)
- 是否应该有"acceptance test" / "e2e test" / "smoke test" 多个层级?
- 当前 P0-1 30% 完成, 应该补哪类测试才到 P0-1 100%?

**A11 (待回答)**: 见 §5

---

### Q12-P: ST 报告 "保留各类过程和结果" 的具体形式

**现象** (per ST-001 实施):
- 用户发令"基于需求文档进行 ST 测试, 保留各类过程和结果"
- 我写 PHASE-ST-001-REPORT.md (12.9KB, 7 段结构) + Q&A-ST-001.md
- 过程记录: cargo check 失败 → 调试 → 修脚本 → 跑测试 → 失败 → 修代码 → 跑测试 → pass → commit
- 这种"过程详细"在 P0-2/3/4 继续推进时是否每次都写?

**困惑**:
- "保留过程" 是写详细调试日志, 还是写高层决策依据?
- PHASE 报告 vs Q&A 报告 vs git log vs commit message — 各自范围?
- 文档治理 (per AGENTS.md §1.2 #12) 具体落地形式?

**A12 (待回答)**: 见 §5

---

## §5 上游 AI 决策请求 (Q&A 待回答)

### 决策矩阵

| # | 类别 | 决策需求 | 选项 | 上游 AI 推荐 |
|---|---|---|---|---|
| Q1-D | 5 域 vs 4 域 | (a) AGENTS.md §5 5 域 (player/economy/...) 是历史命名, 仅作为 RGS 跨仓引用<br>(b) Star 仓应该建映射 player→identity, economy→permission 等<br>(c) 完全独立, 不映射, 仅在文档中说明<br>(d) 重新定义 Star 仓的"5 域" (业务子域 → DDD bounded context) | (a)/(b)/(c)/(d) | ___ |
| Q2-D | ActorContext re-export 冲突 | (a) 保留子模块 (feedback/validation/integration), 跨 crate interface 走子模块强类型<br>(b) 删除子模块, 全部统一用顶层 Uuid 通用版, domain 内部自己转强类型<br>(c) 拆 trait: 跨 crate Port 期待 Uuid, 内部 Port 期待强类型 | (a)/(b)/(c) | ___ |
| Q3-D | define_uuid_id! 字段 pub | (a) 保持当前 P0-1c 改的字段 pub<br>(b) 改回字段 private, 用 accessor `pub fn uuid() -> Uuid` 替代<br>(c) 保持 private, tuple 构造不允许 (编译错限制调用方) | (a)/(b)/(c) | ___ |
| Q4-I | 强类型 ID 跨 crate 转换 | (a) 统一 `as_uuid()` 返回 `Uuid` Copy (非 `&Uuid`)<br>(b) 统一 `as_uuid()` 返回 `&Uuid` (Zero-cost)<br>(c) 删 `as_uuid()`, 改用 `Into<Uuid> for XxxId` trait | (a)/(b)/(c) | ___ |
| Q5-I | cargo _unused_user 交互 | (a) 加 `#[allow(unused_imports)]` 在 unused import 行<br>(b) 实际使用 `UserId` (避免 unused)<br>(c) 关掉 cargo dead code 检测 | (a)/(b)/(c) | ___ |
| Q6-I | port trait ActorContext 类型 | (a) 22 domain 全部统一用强类型 ID<br>(b) 22 domain 全部统一用 Uuid 弱类型<br>(c) 混用 (3 强类型 + 19 Uuid) 现状 | (a)/(b)/(c) | ___ |
| Q7-T | service 校验逻辑 | (a) 改 service: 第一行跳过 PermissionDenied, 直接 CrossTenantDenied<br>(b) 保持现状, IT 测试加 `with_role("tenant_admin")` 绕开<br>(c) 这是有意防御, 不改 | (a)/(b)/(c) | ___ |
| Q8-T | ST 报告 "5 域" 字眼 | (a) 改名 "4 域独立" (精确数字)<br>(b) 保持 "5 域" 跟 AGENTS.md §5 一致<br>(c) 在 ST 报告里加 disclaimer 说明 5 域 vs 4 域区别 | (a)/(b)/(c) | ___ |
| Q9-T | 守门 #1 v1 严格 vs --lib | (a) 接受 --lib 0 err 作为 P0-1 100% 标志<br>(b) 严格 --all-targets 0 err, 修 170 err 估 0.3-0.5M token<br>(c) 分两阶段: P0-1 100% (--lib) → P0-1.5 (--all-targets) | (a)/(b)/(c) | ___ |
| Q10-P | token 预算 | (a) 继续 P0-2/3/4 (估 1.4-2.0M token, 跨 session)<br>(b) 接受 P0-1 30% + 暂停, 跨 session 续<br>(c) 缩减 P0-2/3/4 范围, 单 session 跑完 | (a)/(b)/(c) | ___ |
| Q11-P | 测试层级 | (a) 保持当前 3 层级 (单元 + IT + ST)<br>(b) 加 acceptance test (端到端)<br>(c) 加 smoke test (快速验证)<br>(d) 加 e2e test (全链路) | (a)/(b)/(c)/(d) | ___ |
| Q12-P | 文档治理形式 | (a) PHASE 报告 (高层) + Q&A 报告 (问题) + commit message (变更) 三层<br>(b) 简化: 只 PHASE 报告 + commit message<br>(c) 详细: 加调试日志 + 决策依据 + 过程截图 | (a)/(b)/(c) | ___ |

---

## §6 上游 AI 回答 (待填)

| # | 问题 | 答案 (待上游 AI 填) | 日期 |
|---|---|---|---|

---

## §7 签字栏 (per AGENTS.md §3 7 段结构)

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-31 20:30 JST |
| SRE Lead | (Q&A 入仓, 待上游 AI 回答) | 2026-08-31 20:30 JST |
| 平台 | (P0 全套 token 预算 50% 已用) | 2026-08-31 20:30 JST |
| 评审主持 | (12 问题分类 D/I/T/P 4 类, 决策矩阵 12 项) | 2026-08-31 20:30 JST |
| PM | (P0 30% / ST 32/32 pass / 5 需求 5/5 / 12 问题待上游) | 2026-08-31 20:30 JST |

5 域独立 Lead 签字留 DDD Review 阶段补。

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-31 20:30 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | Q&A-ST-001 初版: 12 问题分 4 类 (D/I/T/P) + 决策矩阵 + 待上游 AI 回答 | 2026-08-31 20:30 JST 用户发令 "测试中的问题汇总到 Q&A 让上游 AI 回答" |
