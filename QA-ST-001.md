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

**A1 (上游 AI 已答, 2026-08-31)**: 见 §5 决策矩阵 + §6。推荐 (a)+(c): "5 域独立 Lead" 是 RGS 仓历史命名, 跨仓治理概念 (5 位真人 Lead 的问责结构), 不等于 Star 仓 22 DDD bounded context; 不建立业务子域↔DDD映射 (业务域不对应), 只在文档加一句 disclaimer 说明两者非同一分类。此项属于治理文档措辞, 建议请 Ulysses 确认解读后落地。

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

**A2 (上游 AI 已答, 2026-08-31)**: 见 §5 决策矩阵 + §6。已核实当前代码状态: feedback/validation/integration 3 domain 的 `port.rs`/`service.rs` 内部用 `crate::context::ActorContext` (强类型), 且这些 port trait 是 `pub` — 意味着"跨 crate 边界用 Uuid, 内部强类型"的说法**不成立**, 强类型已经从 pub trait 泄漏到跨 crate 边界。推荐 (b): 删除 3 个子模块, 22 domain 全部统一用顶层 `star_context::ActorContext` (Uuid); 若某 domain 确实需要 domain-specific 字段 (如 feedback 的 `is_agent_session`), 应扩展 star-context 共享 struct 本身, 不应每个 domain 各自 fork 一份平行类型。此项是下游 AI 可执行的具体重构, 已列入 handoff。

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

**A3 (上游 AI 已答, 2026-08-31)**: 见 §5 决策矩阵 + §6。已核实当前代码状态: 22 个 domain 中 20 个已是 `pub struct $name(pub Uuid)`, 仅 domain-scm / domain-workspace 2 个还有未 commit 的同向修改 (工作区 dirty, 内容一致, 尚未落 commit) — 也就是说 P0-1c 的 pub 字段收敛**已经在事实上完成**, 不存在"是否要改"的分歧, 只差临门一脚的 commit。推荐 (a): 保留 pub 字段现状, 是可接受的 Rust newtype 实践 (这些 ID 除了"是个 Uuid"外无其他不变量, pub 访问不破坏封装); 下游 AI 应尽快 commit 这 2 个待落地文件以闭环。

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

**A4 (上游 AI 已答, 2026-08-31)**: 见 §5。推荐 (a): `as_uuid()` 统一返回 `Uuid` (Copy, 非 `&Uuid`) — Uuid 是 16 字节 Copy 类型, 没有理由返回引用; 构造统一推荐 `impl From<Uuid> for XxxId` 作为主要 API (可读性最好), tuple 构造 `XxxId(uuid)` 保留作为宏内部/测试的逃生舱 (因字段已 pub, per A3)。此项是下游 AI 可执行的跨 22 domain 签名统一, 已列入 handoff。

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

**A5 (上游 AI 已答, 2026-08-31)**: 见 §5。"cargo `_unused_user` 自动生成函数"是 rust-analyzer IDE quick-fix (未解析引用 → 生成函数建议), 不是 cargo/rustc 编译器行为; 报错" field is private"是 P0-1c 修复前的过渡态 (彼时字段还非 pub)。per A3 已核实 22 domain 字段现已全 pub, 此症状在当前代码库应已消解 (2 个待 commit 文件落地后彻底闭环)。推荐 (b): 无需特殊处理, 直接用 `UserId` 真实值 (避免 unused import), 不需要 `#[allow(unused_imports)]` 也不需要关闭 dead-code 检测。此项是历史症状, 不需要下游 AI 单独动作, 已随 A3/Q3-D commit 一并解决。

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

**A6 (上游 AI 已答, 2026-08-31)**: 见 §5。与 A2 (Q2-D) 是同一个决策的两个症状, 一并解决: 推荐 (b) 22 domain port trait 全部统一用顶层 Uuid (`star_context::ActorContext`), 不混用。当前"3 强类型 + 19 Uuid"不是分层设计, 是收敛未完成的中间态 (per A2 已核实强类型已从 pub trait 泄漏)。此项是下游 AI 可执行的具体重构 (feedback/validation/integration 3 domain 的 port.rs/service.rs 改用顶层 ActorContext, 删除 context 子模块的对外引用), 已列入 handoff。

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

**A7 (上游 AI 已答, 2026-08-31)**: 见 §5。已核实 `domain-identity/src/lib.rs:448-451`: 第一行角色检查 (`PermissionDenied`) 确实先于第二行跨租户检查 (`CrossTenantDenied`)。推荐 (c): 这是**有意**防御设计 — 对完全没有 `tenant_admin` 角色的调用方, 应该在"你有没有资格做这件事"层面直接拒绝, 不应该继续泄露"这个资源是否跨租户"这类信息 (最小信息暴露原则); 先角色后租户是标准的 fail-closed 顺序。不改 service 代码, IT 测试保留 `.with_role("tenant_admin")` 绕开角色检查以命中 CrossTenantDenied 分支的做法是正确的测试策略, 不是掩盖 bug。此项已闭环, 下游 AI 不需要动作。

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

**A8 (上游 AI 已答, 2026-08-31)**: 见 §5。承接 A1: 既然不建立 5 域(业务)↔4/5域(DDD)映射, ST 报告用"5 域"字眼会让读者误以为验了 AGENTS.md §5 的 5 个业务子域, 实际只验了 4 个 DDD bounded context。推荐 (a): ST 报告改名"4 域独立"(精确数字), 同时下游 AI 可选择性给 domain-context 补齐 star-mcp dev-dep 让它加入可验证集合 (若补齐则回到 5 域, 但此时"5 域"含义是 4+1 DDD context 而非 AGENTS.md §5 业务子域, 仍需保留 disclaimer)。此项是下游 AI 可执行的报告措辞修正 + 可选依赖补齐, 已列入 handoff。

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

**A9 (上游 AI 已答, 2026-08-31)**: 见 §5。**重要实测更新**: 本次回答前重跑 `cargo check --workspace --all-targets` (2026-08-31 当前工作区, 含 domain-scm/domain-workspace 2 个 dirty 文件), 实测 **968 个 error, 跨 23 个 crate**, 不是 Q&A 原文的 170, 也不是 AGENTS.md v0.24 (2026-08-31 13:18 JST) 记录的 0 err — 说明 v0.24 那次"0 err"是针对当时那个 commit 状态的真实记录, 之后的改动(P0-1c ActorContext 收敛跨 crate 影响)让 --all-targets 错误数反弹, 数字有时效性, 不能引用旧数字。错误主要模式是 test 代码里 `TenantId` vs `Uuid` 类型不匹配 (per A2/A6 强类型收敛未完成的直接后果, 不是独立问题)。推荐 (c): 分两阶段 — P0-1 现阶段仍以 `--lib` 0 err 为完成标志 (已达成), `--all-targets` 968 err 立项为独立可测量的收尾任务 (预估随 A2/A4/A6 决策落地后大部分自动消解, 收敛完成后重新测量再定 token 预算); **任何后续 PHASE 报告引用 --all-targets 数字前必须重新实测, 不得沿用旧数字**。此项(重新测量 + 立项跟踪)已列入 handoff。

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

**A10 (上游 AI 已答, 2026-08-31)**: 见 §5。per A9 实测, --all-targets 968 err (doc 原估 170, 5.7x) 印证 Q&A 原文已指出的"P0-1 实际 5x 预算"模式在 --all-targets 维度上更严重。推荐 (b): 接受 P0-1 现阶段完成度 + 暂停, 跨 session 续 P0-2/3/4, 优先把 A2/A4/A6/A9 的收敛做完再重新估算 P0-2/3/4 token, 避免在错误基数不明的情况下继续扩大范围。**此项是 token 预算/继续与否的决策, 需要 Ulysses 拍板, 上游 AI 仅给出推荐, 不代下游 AI 执行。**

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

**A11 (上游 AI 已答, 2026-08-31)**: 见 §5。推荐 (a): 保持当前 3 层级 (单元 + IT + ST), 不新增 acceptance/smoke/e2e 层级 — 在现有 3 层还没有做到 --all-targets 0 err (per A9 968 err) 之前, 加新测试层级只增加流程成本, 不增加实质质量。**此项是测试体系扩展与否的决策, 需要 Ulysses 拍板**, 上游 AI 仅给出推荐。

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

**A12 (上游 AI 已答, 2026-08-31)**: 见 §5。推荐 (a): 维持三层 (PHASE 报告=高层决策依据 + Q&A 报告=问题/决策点 + commit message=变更本身), 不升级到 (c) 详细调试日志 — per A10 已指出的 token 预算压力, 更细的过程记录会进一步放大 token 消耗而非提升可追溯性; commit message + git log 本身已是可靠的过程证据来源。**此项是文档治理形式的决策, 需要 Ulysses 拍板**, 上游 AI 仅给出推荐。

---

## §5 上游 AI 决策请求 (Q&A 已回答, 见"推荐"列 + §6)

### 决策矩阵

| # | 类别 | 决策需求 | 选项 | 上游 AI 推荐 |
|---|---|---|---|---|
| Q1-D | 5 域 vs 4 域 | (a) AGENTS.md §5 5 域 (player/economy/...) 是历史命名, 仅作为 RGS 跨仓引用<br>(b) Star 仓应该建映射 player→identity, economy→permission 等<br>(c) 完全独立, 不映射, 仅在文档中说明<br>(d) 重新定义 Star 仓的"5 域" (业务子域 → DDD bounded context) | (a)/(b)/(c)/(d) | (a)+(c) — 历史命名 + 文档 disclaimer, 不映射; 需 Ulysses 确认解读 |
| Q2-D | ActorContext re-export 冲突 | (a) 保留子模块 (feedback/validation/integration), 跨 crate interface 走子模块强类型<br>(b) 删除子模块, 全部统一用顶层 Uuid 通用版, domain 内部自己转强类型<br>(c) 拆 trait: 跨 crate Port 期待 Uuid, 内部 Port 期待强类型 | (a)/(b)/(c) | (b) — 已核实强类型已从 pub port trait 泄漏, "分层"现状不成立 |
| Q3-D | define_uuid_id! 字段 pub | (a) 保持当前 P0-1c 改的字段 pub<br>(b) 改回字段 private, 用 accessor `pub fn uuid() -> Uuid` 替代<br>(c) 保持 private, tuple 构造不允许 (编译错限制调用方) | (a)/(b)/(c) | (a) — 22 domain 中 20 个已 pub, 已是事实上收敛, 仅差 2 个文件 commit |
| Q4-I | 强类型 ID 跨 crate 转换 | (a) 统一 `as_uuid()` 返回 `Uuid` Copy (非 `&Uuid`)<br>(b) 统一 `as_uuid()` 返回 `&Uuid` (Zero-cost)<br>(c) 删 `as_uuid()`, 改用 `Into<Uuid> for XxxId` trait | (a)/(b)/(c) | (a) — Uuid 是 16 字节 Copy, 无理由用引用; `From<Uuid>` 为主构造 API |
| Q5-I | cargo _unused_user 交互 | (a) 加 `#[allow(unused_imports)]` 在 unused import 行<br>(b) 实际使用 `UserId` (避免 unused)<br>(c) 关掉 cargo dead code 检测 | (a)/(b)/(c) | (b) — 是 rust-analyzer 过渡态症状, 随 Q3-D pub 收敛已消解 |
| Q6-I | port trait ActorContext 类型 | (a) 22 domain 全部统一用强类型 ID<br>(b) 22 domain 全部统一用 Uuid 弱类型<br>(c) 混用 (3 强类型 + 19 Uuid) 现状 | (a)/(b)/(c) | (b) — 与 Q2-D 同一决策, 混用是收敛未完成的中间态非分层设计 |
| Q7-T | service 校验逻辑 | (a) 改 service: 第一行跳过 PermissionDenied, 直接 CrossTenantDenied<br>(b) 保持现状, IT 测试加 `with_role("tenant_admin")` 绕开<br>(c) 这是有意防御, 不改 | (a)/(b)/(c) | (b)+(c) — 有意防御 (最小信息暴露), 已核实代码行为, IT 测试维持现状 |
| Q8-T | ST 报告 "5 域" 字眼 | (a) 改名 "4 域独立" (精确数字)<br>(b) 保持 "5 域" 跟 AGENTS.md §5 一致<br>(c) 在 ST 报告里加 disclaimer 说明 5 域 vs 4 域区别 | (a)/(b)/(c) | (a) — 精确数字, 承接 Q1-D 不映射决策, "5 域"字眼会误导 |
| Q9-T | 守门 #1 v1 严格 vs --lib | (a) 接受 --lib 0 err 作为 P0-1 100% 标志<br>(b) 严格 --all-targets 0 err, 修 170 err 估 0.3-0.5M token<br>(c) 分两阶段: P0-1 100% (--lib) → P0-1.5 (--all-targets) | (a)/(b)/(c) | (c) — 实测 968 err (非 170), 分两阶段 + 数字须重新实测不沿用旧值 |
| Q10-P | token 预算 | (a) 继续 P0-2/3/4 (估 1.4-2.0M token, 跨 session)<br>(b) 接受 P0-1 30% + 暂停, 跨 session 续<br>(c) 缩减 P0-2/3/4 范围, 单 session 跑完 | (a)/(b)/(c) | (b) 推荐, **待 Ulysses 拍板** — 968 err 实证印证预算超支模式更严重 |
| Q11-P | 测试层级 | (a) 保持当前 3 层级 (单元 + IT + ST)<br>(b) 加 acceptance test (端到端)<br>(c) 加 smoke test (快速验证)<br>(d) 加 e2e test (全链路) | (a)/(b)/(c)/(d) | (a) 推荐, **待 Ulysses 拍板** — 现有 3 层未达 --all-targets 0 err 前不宜扩层 |
| Q12-P | 文档治理形式 | (a) PHASE 报告 (高层) + Q&A 报告 (问题) + commit message (变更) 三层<br>(b) 简化: 只 PHASE 报告 + commit message<br>(c) 详细: 加调试日志 + 决策依据 + 过程截图 | (a)/(b)/(c) | (a) 推荐, **待 Ulysses 拍板** — 维持现状, token 预算压力下不升级到 (c) |

---

## §6 上游 AI 回答 (已填, 2026-08-31)

| # | 问题 | 答案 | 日期 |
|---|---|---|---|
| Q1-D | 5 域 vs 4 域 | (a)+(c): "5 域独立 Lead" 是 RGS 仓历史治理命名 (5 位真人 Lead 问责结构), 不等于 Star 仓 DDD bounded context, 不建映射, 文档加 disclaimer。需 Ulysses 确认解读。 | 2026-08-31 |
| Q2-D | ActorContext 冲突 | (b): 已核实强类型已从 feedback/validation/integration 的 pub port trait 泄漏到跨 crate 边界, "分层"现状不成立; 删除 3 个子模块, 22 domain 统一顶层 `star_context::ActorContext` (Uuid)。下游 AI 可执行。 | 2026-08-31 |
| Q3-D | define_uuid_id! pub | (a): 已核实 22 domain 中 20 个已 pub, 仅 domain-scm/domain-workspace 2 文件待 commit — 事实上已收敛, 保持 pub, 尽快 commit 闭环。下游 AI 可执行。 | 2026-08-31 |
| Q4-I | 强类型 ID 转换 | (a): `as_uuid()` 统一返回 `Uuid` Copy (非 `&Uuid`), `From<Uuid>` 为主构造 API, tuple 构造保留作逃生舱。下游 AI 可执行。 | 2026-08-31 |
| Q5-I | cargo _unused_user | (b): 是 rust-analyzer IDE 过渡态症状 (未解析引用建议), 非 cargo 编译器行为, 随 Q3-D pub 收敛已消解, 无需单独动作。 | 2026-08-31 |
| Q6-I | port trait 类型选择 | (b): 与 Q2-D 同一决策 — 22 domain 全统一 Uuid 弱类型, 当前 3 强类型+19 Uuid 混用是收敛未完成的中间态。下游 AI 可执行。 | 2026-08-31 |
| Q7-T | service 校验顺序 | (b)+(c): 已核实 `domain-identity/src/lib.rs:448-451` 角色检查先于跨租户检查, 是有意的最小信息暴露防御设计, 不改 service, IT 测试保留 `.with_role("tenant_admin")` 现状。已闭环, 无需动作。 | 2026-08-31 |
| Q8-T | ST 报告"5 域"字眼 | (a): 改名"4 域独立" (精确数字), 承接 Q1-D 不映射决策; domain-context dev-dep 补齐为可选后续项。下游 AI 可执行。 | 2026-08-31 |
| Q9-T | --lib vs --all-targets | (c): 分两阶段, P0-1 仍以 --lib 0 err 为完成标志 (已达成); **实测 --all-targets 当前 968 err (非文档原 170, 非 AGENTS.md v0.24 记录的 0 err — 数字有时效性)**, 立项独立跟踪, 随 Q2/Q4/Q6 收敛落地后重新测量。下游 AI 可执行 (重新实测 + 立项)。 | 2026-08-31 |
| Q10-P | token 预算 | 推荐 (b) 接受 P0-1 现完成度 + 暂停跨 session 续, 理由: 968 err 实证印证预算超支模式比预估更严重。**待 Ulysses 拍板**, 非下游 AI 可单方执行。 | 2026-08-31 |
| Q11-P | 测试层级边界 | 推荐 (a) 保持 3 层级, 理由: 现有层级未达 --all-targets 0 err 前扩层无实质收益。**待 Ulysses 拍板**。 | 2026-08-31 |
| Q12-P | 文档治理形式 | 推荐 (a) 维持三层 (PHASE+Q&A+commit message), 理由: token 预算压力下不宜升级到详细调试日志。**待 Ulysses 拍板**。 | 2026-08-31 |

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
| v0.2 | 2026-08-31 | 上游 AI (本 session) | 12 问题全部回答: §1-§4 每条 A[N] 填答 + §5 决策矩阵"推荐"列填答 + §6 答案表 12 行填答; 关键实测更新 Q9-T: 重跑 `cargo check --workspace --all-targets` 实测 968 err (非原文 170, 非 AGENTS.md v0.24 记录的 0 err, 数字有时效性不可沿用); Q2-D/Q6-I 核实 feedback/validation/integration 3 domain 强类型已从 pub port trait 泄漏, "分层设计"现状不成立, 推荐统一收敛为顶层 Uuid; 下游 AI 可执行项另立 `HANDOFF-ST-001.md` | 2026-08-31 用户发令"回答QA问题并把需要下游ai处理的内容更新进handoff" |
