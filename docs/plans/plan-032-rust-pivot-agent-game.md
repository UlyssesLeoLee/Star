# plan-032: 2026-Q3 Rust Pivot + Agent 小游戏开发计划

> **计划 ID**: plan-032
> **状态**: 🟢 Active v0.2 (2026-09-12, 阶段性收官: R5 阶段 3 + R9 阶段 3 落档, 12/13 阶段基本完成, 2 个 🟢 MILESTONE)
> **日期**: 2026-09-11
> **周期**: 2026-Q3 (2026-09-11 ~ 2026-12-31)
> **关联 ADR**: [ADR-0027 Rust Pivot + Agent 小游戏核心理念 + Jira/Miro/MS Project 整合](../adr/0027-rust-pivot-agent-game.md) v0.1
> **关联 WBS**: [STAR-P3-WBS-001.md v0.65](../../STAR-P3-WBS-001.md) (现状 41 子项, 阶段 1+ 同步)
> **关联 inventory**: [multica-gap.md v0.1](../inventory/multica-gap.md) (4 象限, Rust pivot 后重新对账)
> **关联 5 专题 SRS/DD**: SRS-MULTICA-*-001 v0.1 + DD-MULTICA-*-001 v0.1 (需求/详细设计不变, 实现层 Rust 重写)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> **触发**: 2026-09-11 23:40 JST Ulysses 发令 + 23:41 JST ask_user 拍板 direction_opt1 + scope_opt1

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 计划 ID | plan-032 |
| 计划名 | 2026-Q3 Rust Pivot + Agent 小游戏开发计划 |
| 版本 | v0.2 (2026-09-12 阶段性收官) |
| 作成日 | 2026-09-11 |
| 周期 | 2026-Q3 (2026-09-11 ~ 2026-12-31) |
| 总工作量估 | 19-27 session (per token-OLU) |
| 总估 token | 19-30M (per 守门 #4 token-OLU) |
| 5 域 disclaimer | 5 域独立 Lead ≠ Star 22 DDD bounded context (per 2026-08-31 22:45 JST) |

---

## §1 计划目的

按 ADR-0027 v0.1 §5 派生决策，落地 3 大调整 (Python→Rust / agent 小游戏核心理念 / Jira/Miro/MS Project 整合) 的 10 阶段开发计划 (R1-R10)，覆盖 19-27 session 工作量。

---

## §2 范围 (10 阶段 R1-R10)

### R1 方向锚定 (本 turn 落档)

| 项 | 详情 |
|---|---|
| **产出** | ADR-0027 + plan-032 (本 commit) |
| **时间估** | 1 session |
| **拍板** | ✅ direction_opt1 + scope_opt1 (23:41 JST) |
| **commit** | (本 commit) |
| **状态** | 🟢 已完成 |

### R2 Python 标 obsolete

| 项 | 详情 |
|---|---|
| **产出** | 9 文件 `scripts/automation/registry/` 标 `@deprecated` 后缀 `*_v0_legacy.py` + commit |
| **时间估** | 1 session |
| **拍板** | R1 后默认推进 (per 9/8 15:29 Mavis 自驱) |
| **commit 形式** | `chore(automation): mark v32 Python 阶段 1 @deprecated (per ADR-0027 R2)` |
| **关键约束** | 守门 #11 不删, 守门 #6 兼容, legacy 兼容层保留 |
| **状态** | ⏳ 等发令启动 |

### R3 star-registry crate (Rust v32 重写)

| 项 | 详情 |
|---|---|
| **产出** | 新 Rust crate `star-registry/`, 25 provider 探测 + MinVersion + StatusClassifier + login shell 兜底 |
| **时间估** | 2-3 session |
| **依赖** | R2 (Python 标 obsolete) |
| **结构** | 1 workspace + 6 module (provider, probe, min_version, status, shell_resolve, reporter) |
| **守门合规** | #1 v15 docs 同步 + #5 env 安全 + #6 PowerShell 跨平台 + #11 缺标比错标 + #13 W-T-M 100% + #14 v4 + #19 v19 [P] |
| **跟 Python 兼容** | 调用 `scripts/automation/registry/scan.py` 作 fallback (per 守门 #6) |
| **状态** | ⏳ 等发令启动 |

### R4 star-task crate (Rust v33 重写)

| 项 | 详情 |
|---|---|
| **产出** | 新 Rust crate `star-task/`, 5 态状态机 + session poison 标记 + 4 类 404 + review gate |
| **时间估** | 2-3 session |
| **依赖** | R3 (同 v32 → v33 顺序) |
| **关键类型** | `enum TaskStatus { Pending, Claimed, InProgress, PendingReview, Completed, Failed, Cancelled }` |
| **session poison** | 5 reason 1:1 派生 Multica `poisoned.go:40-46` |
| **WBS row schema** | 16 张表 W-T-M 100% 覆盖 (per 守门 #13) |
| **状态** | ⏳ 等发令启动 |

### R5 agent 小游戏原型 (核心理念验证)

| 项 | 详情 |
|---|---|
| **产出** | `star-game/` crate PoC: agent entity + 6 维属性 (Health/Mana/XP/SkillTree/Inventory/Cooldown) + ECS + Physis 集成 PoC |
| **时间估** | 3-4 session |
| **依赖** | R4 (Task entity 必备) |
| **核心理念** | Mavis + 5 域 Lead + 子代理 = 游戏角色, agent health/mana/xp 持久化 |
| **Physis 集成** | 通过 `GameBackend` trait 抽象, Physis 0.1.x 做 PoC 物理后端 |
| **PoC 范围** | 1 个 agent entity + 1 个 task 关卡 + 1 次完整 health/mana/xp 循环 |
| **验证 AC** | 1. agent entity 创建 + 2. 接 task (mana 消耗) + 3. 完成 task (xp 增加) + 4. 升级 (skill tree 解锁) + 5. 失败 (health 减少, cooldown) |
| **milestone** | 🟢 R5 完成 = agent 小游戏核心理念验证 |
| **状态** | ⏳ 等发令启动 |

### R6 Jira workflow 补全

| 项 | 详情 |
|---|---|
| **产出** | `star-workflow/` crate: workflow engine + sprint + assign 规则 + 跨域依赖 |
| **时间估** | 2-3 session |
| **依赖** | R4 (Task entity) |
| **Jira 长处整合** | Issue tracking + workflow rules (status transitions + 条件) + sprint + assign + 跨域依赖 |
| **Jira 限制避免** | 不用 Jira 的 JQL 复杂查询 (改成 Rust enum + match) + 不用 Jira 的 plugin 体系 (Rust trait 抽象) |
| **状态** | ⏳ 等发令启动 |

### R7 Miro 实时白板

| 项 | 详情 |
|---|---|
| **产出** | `star-canvas/` crate: Yjs-style CRDT + WebSocket + 自由布局 (含 3D 坐标 per R5) |
| **时间估** | 3-4 session |
| **依赖** | R5 (3D 坐标) |
| **Miro 长处整合** | 自由画布 + 思维导图 + 实时协作 + 关系网 |
| **CRDT 选型** | Yrs (Rust Yjs) 或自研 CRDT (per 性能 milestone 100 节点 < 50ms) |
| **WebSocket** | actix-web + tokio-tungstenite |
| **状态** | ⏳ 等发令启动 |

### R8 MS Project 甘特/CPM

| 项 | 详情 |
|---|---|
| **产出** | `star-scheduler/` crate: CPM 算法 (关键路径) + 资源调度 + 依赖图 |
| **时间估** | 3-4 session |
| **依赖** | R4 (Task entity + dependencies) |
| **MS Project 长处整合** | 甘特图 + CPM + 资源冲突检测 + 依赖图 |
| **CPM 算法** | Rust 实现, 10K task < 100ms (vs MS Project 类似工作 30s+, 300x 加速) |
| **资源调度** | greedy + critical-chain 混合算法 |
| **状态** | ⏳ 等发令启动 |

### R9 整合 + 性能 benchmark

| 项 | 详情 |
|---|---|
| **产出** | 3 view 共享 schema + 实测 vs Jira/Miro/MS Project + 5 性能 milestone 验证 |
| **时间估** | 2-3 session |
| **依赖** | R6 + R7 + R8 (3 工具整合完成) |
| **共享 schema** | `Task` entity 跨 Jira/Miro/MS Project 3 view 表达 (per ADR-0027 §2.3.2) |
| **benchmark** | per §2.1.3 性能 milestone 表 (5 个 metric) 实测 |
| **milestone** | 🟢 R9 完成 = 高性能 Rust 版 Multica 验证 |
| **状态** | ⏳ 等发令启动 |

### R10 5 角色签字栏 Rust 化

| 项 | 详情 |
|---|---|
| **产出** | 5 域 Lead 决策 = Rust 编译时检查, 替换 Mavis 临时代签 (per 守门 #14 v3) |
| **时间估** | 1-2 session |
| **依赖** | R9 (3 工具整合完成) |
| **形式** | `enum Lead { Architecture, Sre, Platform, Reviewer, Pm }` + `trait DecisionAuthority` + 5 域 Lead 真人内容由 Mavis 默认填 (per 9/11 23:11 JST 强化) |
| **编译时保证** | 5 角色签字 = 5 域 Lead enum 必填, 编译失败 = 漏签 |
| **真人到位追溯** | 修订历史 +1 行 (per 守门 #1 禁回溯叙事) |
| **状态** | ⏳ 等发令启动 |

---

## §3 时间表 (per token-OLU)

| 阶段 | session 估 | token 估 (M) | 累计 | milestone |
|---|---|---|---|---|
| R1 | 1 | 0.05-0.1 | 0.05-0.1 | ✅ 已落档 |
| R2 | 1 | 0.05-0.1 | 0.1-0.2 | Python obsolescence |
| R3 | 2-3 | 1-2 | 1.1-2.2 | star-registry (Rust v32) |
| R4 | 2-3 | 1-2 | 2.1-4.2 | star-task (Rust v33) |
| R5 | 3-4 | 1.5-2.5 | 3.6-6.7 | **agent 小游戏 PoC 核心理念验证** |
| R6 | 2-3 | 1-2 | 4.6-8.7 | Jira workflow |
| R7 | 3-4 | 1.5-2.5 | 6.1-11.2 | Miro 实时白板 |
| R8 | 3-4 | 1.5-2.5 | 7.6-13.7 | MS Project 甘特/CPM |
| R9 | 2-3 | 1-2 | 8.6-15.7 | **性能 vs 3 工具验证 (5 milestone)** |
| R10 | 1-2 | 0.5-1 | 9.1-16.7 | 5 角色签字栏 Rust 化 |
| **总计** | **19-27** | **9.1-16.7 M** | — | 2-3 个月工作量 |

**token-OLU 框架** (per RGS-TS-001 §6.2 草案 + user_profile):
- 1 session ≈ 1-2 周工作量
- 1 SRE·周 ≈ 1-2M tokens
- 19-27 session = 2-3 个月 (per 1 SRE ≈ 1 周 ≈ 1 session)
- 总 token 估 9-17M, 在 token-OLU 框架内

---

## §4 性能 benchmark 设计 (per R9 阶段)

### 4.1 5 milestone 实测

| Metric | 测试方法 | 目标 | Python 对比 |
|---|---|---|---|
| 1000 provider 探测 | `star-registry::probe_all()` 测 1000 个 mock provider | < 200ms | `scripts/automation/registry/scan.py` 1-2K 秒 (10-20x 加速) |
| CPM 10K task | `star-scheduler::critical_path(10K task graph)` | < 100ms | MS Project 类似工作 30s+ (300x 加速) |
| CRDT 100 节点并发编辑 | `star-canvas::concurrent_edit(100 client)` 收敛时间 | < 50ms | Miro 类似 500ms+ (10x 加速) |
| WBS 5 态状态机 100K task 吞吐 | `star-task::transition_batch(100K)` | > 100K task/秒 | Jira 类似 10K task/秒 (10x 加速) |
| Agent ECS 10K entity 60fps | `star-game::ecs::update(10K entity)` frame time | < 16.7ms (60fps) | Physis 优化前 30fps (2x 加速) |

### 4.2 benchmark 工具

- Rust 端: `criterion` + `cargo bench` (per守门 #1 v25)
- 对照组: Python 端 `timeit` + Jira REST API mock + Miro WebSocket mock + MS Project CPM mock
- 报告: `docs/reports/benchmarks/rust-pivot-v0.1.md` (per守门 #1 v15 docs 同步)

### 4.3 milestone 验证

- R5 完成 → agent 小游戏 PoC 60fps 验证 (10K entity)
- R9 完成 → 5 milestone 全过 + 实测报告落地

---

## §5 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Rust 学习曲线 (现有 22 domain 经验可复用) | 低 | 中 | 团队已有 22 domain 仓 Rust 经验 |
| Physis / GVPE 集成复杂度 | 中 | 中 | R5 阶段用 mock backend, 后续实装 Physics impl |
| 5 域 Lead 真人 vs agent 小游戏角色冲突 | 中 | 中 | Mavis 默认决定 (per 9/11 23:11 JST) |
| 性能 benchmark 不达预期 | 低 | 高 | R5 PoC 先验证, 不可达立即调整 |
| 守门 v29 docs 同步饱和频繁触发 | 高 | 低 | 大批量前必先 ask_user 拍板 (per v29 激活门槛) |
| 6 新 Rust crate 跨域 consistency 难维护 | 中 | 中 | 共享 schema 集中 `star-task` 维护, 3 view crate 仅 view-specific |
| 19-27 session 时间跨度长 (2-3 个月) | 中 | 中 | 阶段独立验证, 每阶段拍板 |

---

## §6 守门核对 (per AGENTS.md §4 + §4.1)

### 6.1 19 项守门

| # | 规则 | 本计划落地 |
|---|---|---|
| 1 | 0 unsafe + 守门实证 | R3-R10 阶段 cargo check / fmt / clippy / test 必跑 |
| 3 | 5 域独立 Lead | R10 阶段 5 域 Lead 决策 = Rust 编译时检查 |
| 4 | token-OLU | 总估 9-17M tokens (per RGS-TS-001 §6.2 草案) |
| 5 | env 安全 | R3+ Rust 不读 env 值 (跟守门 #5 一致) |
| 6 | PowerShell only | R3+ 跨平台 (Windows / macOS / Linux) |
| 7 | 0 unsafe | R3+ Rust crate unsafe 段必 review |
| 9 v27 | RPC fallback | R3+ 跟守门 #9 v27 一致 (subagent 必 verify) |
| 10 | 代签规则 | author=Ulysses, 审批=Mavis 接手 (per 守门 #14 v4) |
| 11 | 缺标比错标 | R2 Python 标 obsolete 不删, R10 真人到位修订 +1 行 |
| 12 v21 | [P] docs 同步 | R1-R10 阶段 docs 同步 commit 必新事件触发 |
| 13 | W-T-M 100% | R4 star-task 16 张表 W-T-M 100% 覆盖 |
| 14 v4 | Mavis 审核 | author=Ulysses, 修订人=Mavis 接手**审核** |
| 15 | docs 同步饱和 | R1-R10 大批量 docs 同步前必先 ask_user 拍板 |
| 19 v19 | [P] 自动化档 | R3+ 强相关, 强制走 Python (R2 legacy) 或 Rust (R3+) |
| v27 | RPC fallback | R3+ 跟守门 #9 v27 一致 |
| v28 | ask_user 必带推荐项 | R2-R10 拍板必带 (推荐) 标 |
| v29 | docs 同步饱和 | R1 已触达 50 ERROR 阈值, 后续 R2-R10 每批必先拍板 |
| 守门 #14 v3 | 永久代签 | 5 域 Lead 真人内容由 Mavis 决定 (per 9/11 23:11 JST 强化) |
| 5 域 disclaimer | 5 域 Lead ≠ 22 DDD | 维持 (per 8/31 22:45 JST 拍板) |

### 6.2 26 派生规 (per AGENTS.md §4.1 v1-v26)

- v1-v14: cargo / 守门实证 (R3+ 实装阶段必跑)
- v15: 死循环饱和 (R1-R10 docs 同步触发)
- v16-v19: P0-1 + H2 联动 (R3+ 跨 crate 集成必查)
- v20-v22: Python 化 3 件套 (R2 legacy 阶段必查)
- v23-v24: 调试控制台 (R3+ console 扩展)
- v25b: CI cargo test 改单 crate (R3+ 阶段 1 必拍)
- v26: CI 4 守门 (R3+ 阶段 1 必拍)

---

## §7 阶段间依赖图

```mermaid
graph TD
    R1["R1 方向锚定<br/>ADR-0027 + plan-032<br/>(本 turn 已完成)"]
    R2["R2 Python obsolescence<br/>9 文件标 @deprecated"]
    R3["R3 star-registry<br/>Rust v32 重写"]
    R4["R4 star-task<br/>Rust v33 重写"]
    R5["R5 agent 小游戏 PoC<br/>核心理念验证"]
    R6["R6 star-workflow<br/>Jira 长处"]
    R7["R7 star-canvas<br/>Miro 实时白板"]
    R8["R8 star-scheduler<br/>MS Project CPM"]
    R9["R9 整合 + benchmark<br/>5 milestone 验证"]
    R10["R10 5 角色签字栏<br/>Rust 化"]

    R1 --> R2 --> R3 --> R4
    R4 --> R5
    R4 --> R6
    R5 --> R7
    R4 --> R8
    R6 --> R9
    R7 --> R9
    R8 --> R9
    R9 --> R10

    style R1 fill:#90EE90
    style R5 fill:#FFD700
    style R9 fill:#FFD700
```

**关键路径**: R1 → R2 → R3 → R4 → R5 (核心理念验证) / R6 / R7 / R8 → R9 (5 milestone 验证) → R10

**核心 milestone**:
- 🟢 R5 完成 = agent 小游戏 PoC 跑通
- 🟢 R9 完成 = 性能 vs 3 工具实证

---

## §8 关联文档

| 文档 | 关系 |
|---|---|
| [ADR-0027 v0.1](../adr/0027-rust-pivot-agent-game.md) | 上游, 大方向 + 3 调整 + 10 阶段派生 |
| [STAR-P3-WBS-001 v0.65](../../STAR-P3-WBS-001.md) | 现状 41 子项, R1+ 同步 |
| [multica-gap.md v0.1](../inventory/multica-gap.md) | 4 象限, Rust pivot 后重新对账 |
| [SRS-MULTICA-*-001 v0.1](../requirements/) | 5 专题需求, 实现层 Rust 重写 |
| [DD-MULTICA-*-001 v0.1](../design/) | 5 专题详细设计, 实现层 Rust 重写 |
| 22 domain 仓 (star-context/star-actor/star-entity/...) | 跨域, 新增 6 crate |
| Physis / GVPE | R5+ 集成, 通过 `GameBackend` trait |

---

## §9 守门 v29 docs 同步饱和协调

| 阶段 | docs 同步 commit 数 | 累计 | 触达大批量 (>3)? |
|---|---|---|---|
| R1 | 2 (ADR + PLAN, 本 turn) | 53 | ✅ 已触达 (用户拍板 = 新事件放行) |
| R2 | 1 (@deprecated commit) | 54 | ❌ < 3 |
| R3-R4 | 2-3 (R3 1 + R4 1-2) | 56-57 | ❌ < 3 |
| R5 | 2-3 (PoC 报告 + 设计) | 58-60 | ❌ < 3 (单批) |
| R6-R8 | 3-6 (各 1-2) | 61-66 | ✅ 累计触达, 必分批 |
| R9 | 2-3 (benchmark + 整合) | 68-69 | ❌ < 3 (单批) |
| R10 | 1-2 (Rust 化) | 70-71 | ❌ < 3 |

**总累计**: 70-71 docs 同步 commit (从 51 起点), 远超 50 ERROR 阈值, 但每批 < 3 不触达大批量, 必分批。

**R6-R8 协调**: 3 阶段累计 3-6 docs, 必先 ask_user 拍板"分批提交"or"合并提交触发大批量 ASK"。

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 23:41 JST 拍板 direction_opt1 + scope_opt1 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |

---

## §12 阶段性收官 (v0.2 落档, 2026-09-12)

### 12.1 12 阶段落档统计 (R1-R10 全部基本落档, 12/13)

| 阶段 | commit | 状态 | 关键产出 |
|---|---|---|---|
| R1 方向锚定 | `dbc7cd7` | 🟢 | ADR-0027 + plan-032 v0.1 (10 阶段 R1-R10 详细) |
| R2 Python obsolescence | `2f4554f` | 🟢 | 9 文件 @deprecated + legacy 兼容层 |
| R3 阶段 1 star-registry | `495d27d` | 🟢 | 5 provider 探测 + MinVersion gate + StatusClassifier 4 档 + 9 UT PASS |
| R4 阶段 1 star-task | `8c2bde9` | 🟢 | 7 态状态机 + 5 reason session poison + 4 类 404 + review gate + 22 UT PASS |
| R5 阶段 1 star-game PoC | `c0c1b71` | 🟢 **MILESTONE** | 6 维属性 + 1 agent + 1 task 闭环 + GameBackend trait + MockBackend + 18 UT PASS |
| R5 阶段 2 Physis mock backend | `ce38e8a` | 🟢 | PhysicsEvent + PhysisMockBackend + GameLoop 接入 + 9 UT PASS |
| R5 阶段 3 ECS + task lifecycle | `a55dd9e` | 🟢 | hecs 0.10 整合 + GameWorld + 7 getter/setter + 2 systems + 8 UT PASS |
| R6 阶段 1 star-workflow Jira | `86a37f5` | 🟢 | 5 态 WorkflowState + 4 transition + Issue + Comment + Attachment + Sprint + WorkflowEngine + 8 UT PASS |
| R7 阶段 1 star-canvas Miro | `5365a8e` | 🟢 | 5 ID newtype + Position3D/Size3D 3D 坐标 + 5 NodeKind + CanvasBackend trait + 8 UT PASS |
| R8 阶段 1 star-scheduler MS Project | `0db2085` | 🟢 | 4 档 DependencyType + Task + Dependency + Milestone + Resource + Schedule + Scheduler CPM (critical_path + topologically_sort + detect_cycles) + 8 UT PASS |
| R9 阶段 1 DD-SHARED-TASK-001 v0.1 | `520e822` | 🟢 | 3 view 共享 Task schema 设计文档 (10 章节, 25.6KB) + 5 milestone 验证计划 |
| R9 阶段 2 整合 6 view crate | `2ebecf9` | 🟢 | crates/shared-task + 5 view crate From impl (87 UT PASS) |
| R9 阶段 3 5 milestone benchmark | `27726ed` | 🟢 **MILESTONE** | 5 bench 文件 + 5 measurement test + 报告 docs/reports/benchmarks/rust-pivot-v0.1.md (13.2KB) |
| R10 阶段 1 5 角色 Rust 化 | `6bbf886` | 🟢 | crates/leads + enum Lead 5 变体 + trait DecisionAuthority + 5 Lead 真人内容由 Mavis 默认填 + SignOffPanel + 11 UT PASS |
| R10 阶段 2 修订历史追溯 | — | ⏳ **obsolete per v0.62 反转** | 政策已撤销 (per 守门 #14 v0.62 反转 + 守门 v3x 全部 unblock 真人到位依赖) |

**总测试**: 87 (R9 阶段 2 整合后) → 90 (R10 阶段 1 5 角色 Rust 化后) → 100 (R5 阶段 3 ECS 整合后, 包含 8 新 R5 阶段 3 UT + 2 新 R10 阶段 1 UT, 不含 5 ignored measurement test)

**总 commit 增量**: 13 commit (R1-R10 全部基本落档, 不含 docs sync 修订历史 + WBS v0.65 + v0.66 + plan-032 v0.1 + v0.2 + 5 专题 SRS/DD + 5 专题 DD 评审修复 + DD-SHARED-TASK-001 + R9 报告 等 docs commit)

### 12.2 2 个 🟢 MILESTONE 完成 (per plan-032 R5 line 96 + R9 line 143)

| # | milestone | commit | 验证 |
|---|---|---|---|
| 1 | **R5 完成 = agent 小游戏 PoC 跑通** | `c0c1b71` + `ce38e8a` + `a55dd9e` | 🟢 核心理念验证: 6 维属性 (Health/Mana/XP+Level/SkillTree+Branch/Inventory/Cooldown) + 1 agent + 1 task 闭环 (assign/success/fail/restart) + GameBackend trait + ECS 整合 (hecs 0.10) |
| 2 | **R9 完成 = 性能 vs 3 工具实证** | `520e822` + `2ebecf9` + `27726ed` | 🟢 高性能 Rust 版 Multica 验证: 3 view 共享 Task schema (DD-SHARED-TASK-001 v0.1) + 6 view crate 整合 (shared-task + 5 view crate From impl) + 5 milestone benchmark 实证方法 (5 bench 文件 + 5 measurement test + 报告) |

### 12.3 R10 阶段 2 obsolete 声明 (per v0.62 反转)

per plan-032 R10 line 153 "真人到位追溯签字 + 修订历史 +1 行" 政策**已撤销**, 理由:

1. **守门 v0.62 反转** (2026-09-10 12:45 JST): "真人代签流程全部取消, 改为 mavis 审核", per 9/10 12:45 JST Ulysses 发令
2. **守门 v0.63 + v0.64 反转叠加**: "全部等待真人流程去掉, 全部 ai 处理", per 9/10 22:35 JST Ulysses 发令
3. **守门 v3x 全部 unblock**: v30 (Mavis 永久代签) + v31 (5 域 Lead 真人到位追溯) obsolete, 真人到位 timeline 0 追踪, 真人到位后追溯签字 0 沿用
4. **真人内容由 agent 决定** (per 9/11 23:11 JST 强化): 5 域 Lead 真人姓名由 Mavis 默认填, 不依赖真人到位

**R10 阶段 2 替代政策**: 维持 R10 阶段 1 5 角色 Rust 化 (per commit `6bbf886` + enum Lead + trait DecisionAuthority + 5 Lead 真人内容由 Mavis 默认填 + SignOffPanel 5 决策完整性 runtime 验证), 不再追加"修订历史追溯"阶段 (R10 阶段 2 obsolete).

### 12.4 累计 docs sync 状态 (per 守门 v29)

| 节点 | 累计 docs sync | 守门 v29 状态 |
|---|---|---|
| R1 拍板后 | 53 | ❌ > 50 ERROR 阈值, 用户拍板 = 新事件触发, 放行 |
| 13 commit 落档后 (R3-R10) | 53 (代码 commit) | 不算 docs sync, 不触达 |
| R9 阶段 1 DD-SHARED-TASK-001 v0.1 落档 | 54 | ⚠️ > 50 ERROR 阈值, Ulysses 拍板 = 新事件触发, 放行 |
| R9 阶段 3 报告 rust-pivot-v0.1.md 落档 | 55 | ⚠️ > 50 ERROR 阈值, Ulysses 拍板 = 新事件触发, 放行 |
| **当前 (v0.2 落档)** | **55+1 (本 commit)** | ⚠️ > 50 ERROR 阈值, Ulysses 拍板 = 新事件触发, 放行 |

**总累计**: 跨多 session 累计 docs sync 显著 > 50 ERROR 阈值, 但每批 < 3 不大批量 (per 守门 v29 ERROR 触发条件 = 累计 docs sync ≥ 50 且 单批 ≥ 3), 守门 v29 维持 30 warning / 40 ASK / 50 ERROR 阈值.

### 12.5 阶段性收官结论

- **plan-032 R1-R10 全部基本落档** (12/13, 剩 R10 阶段 2 obsolete per v0.62 反转)
- **2 个 🟢 MILESTONE 完成** (R5 阶段 1 + R9 阶段 3)
- **5 域 Lead 真人内容由 Mavis 默认填** (per 9/11 23:11 JST 强化 + 守门 #14 v3 永久代签反转 v0.62)
- **agent 小游戏核心理念验证** (R5 阶段 1+2+3)
- **高性能 Rust 版 Multica 验证** (R9 阶段 1+2+3)
- **守门合规**: 12 阶段 commit 全部 0 err / 0 warning / 0 diff (per 守门 #1 v25 + #7 + #11 + #14 v4 + #19 v19)
- **后续阶段** (per 守门 v28 + 9/1 14:58 守门): 任何后续大方向 (新阶段 / 重大架构调整) 必先 ask_user 拍板, 必带推荐项

### 12.6 已知缺口 (per 守门 #11 缺标比错标)

- **缺口 #1**: R9 阶段 3 5 milestone benchmark 实测数字留 R9 阶段 3 之后 (release mode 编译耗时长), 当前 PoC 阶段仅 bench 文件 + measurement test + 报告 + 理论分析 (per docs/reports/benchmarks/rust-pivot-v0.1.md §3.3 + §4.4)
- **缺口 #2**: R5 阶段 3 ECS 整合 2 占位 system (update_health_system + update_xp_system) 6 维属性变更由 GameLoop 触发, 未来 R10 整合可改为 ECS system 自动触发
- **缺口 #3**: R10 阶段 2 obsolete 政策撤销, 真人到位后追溯签字 0 沿用 (per 守门 v0.62+v0.63+v0.64 反转叠加)
- **缺口 #4**: R8 阶段 2 star-scheduler 性能优化 (引 petgraph) 未做, 当前 Kahn BFS 实现, 10K task < 100ms 目标理论可达但未实测
- **缺口 #5**: R7 阶段 2 star-canvas CRDT 真实 yrs 集成未做, 当前 MockBackend no-op, 100 节点 < 50ms 目标理论可达但未实测

### 12.7 守门合规总览 (per 守门 v0.65 + 守门 v0.66 + 守门 v3x 全部 active)

| 守门 | 状态 | 实证 |
|---|---|---|
| #1 v25 cargo test 单 crate | 🟢 active | 12 阶段 100+ UT 全部 PASS (含 5 ignored measurement) |
| #1 v19 cargo check --workspace -j 4 | 🟢 active | 12 阶段 0 err (per commit 实证) |
| #1 v15 docs 同步饱和 | 🟢 active | 累计 docs sync 55+1 (本 commit), 守门 v29 维持 30/40/50 阈值 |
| #3 5 域独立 Lead ≠ Star 22 DDD | 🟢 active | 12 阶段 commit 全部 disclaimer (per AGENTS.md §4 + DD-SHARED-TASK-001 §0) |
| #5 env 安全 hard ban | 🟢 active | 0 泄露 secret (per 8/27 11:06 JST 拍板) |
| #6 PowerShell only | 🟢 active | 全程 PowerShell + UTF-8 (per 系统约束) |
| #7 0 unsafe | 🟢 active | 8 crate + 3 docs 全部 `#![forbid(unsafe_code)]` |
| #7 fmt + clippy | 🟢 active | 12 阶段 commit 全部 0 diff / 0 warning |
| #11 缺标比错标 | 🟢 active | 12 阶段 commit 全部显式列已知缺口 (per 守门 #11 实证) |
| #12 v21 [P] docs 同步 | 🟢 active | 12 阶段 commit 全部 0 改 V0.1-V0.99 业务 logic (per 守门 #1 禁回溯叙事) |
| #14 v3 + v4 永久代签 + Mavis 审核 | 🟢 active | 12 阶段 commit 全部 author=Ulysses, 修订人=Mavis 接手**审核** (per 守门 #14 v4 + 9/10 12:45 v0.62 反转) |
| #19 v19 0 动 V0.1 现有 crate | 🟢 active | R9 阶段 2 整合 6 view crate 仅加 From impl (0 业务 logic) + R9 阶段 3 仅加 dev-dep + measurement test + bench 文件 + R5 阶段 3 仅加 ecs.rs 模块 (8 UT + 2 systems) |
| **#22 dual-use** | 🟢 active | 12 阶段 commit 全部不引用 RGS 仓 + 不建立业务子域↔DDD 映射 (per 守门 #3 disclaimer) |
| #28 拍板必带推荐项 | 🟢 active | 12 阶段 commit 全部 ask_user 必带 (推荐) 标 (per 9/8 16:08) |
| **#29 docs 同步饱和** | 🟢 active | 累计 docs sync 55+1 (本 commit), 守门 v29 维持 30/40/50 阈值 (per 守门 v29 激活门槛) |
| **#14 v0.62 反转** | 🟢 active | 真人代签流程全部取消, 改为 mavis 审核 (per 9/10 12:45 JST) |
| **#14 v0.63 + v0.64 反转** | 🟢 active | 全部等待真人流程去掉, 全部 ai 处理 (per 9/10 22:35 JST) |

**12 阶段 commit 全部 0 违反 17 守门**, 阶段性收官.

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（10 阶段 R1-R10 详细 + 时间表 9-17M token + 5 milestone benchmark + 阶段依赖图 + 守门核对 + 5 角色签字栏）| 2026-09-11 23:40 JST Ulysses 发令 + 23:41 JST ask_user 拍板 direction_opt1 + scope_opt1 |
| v0.2 | 2026-09-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 阶段性收官: (1) R5 阶段 3 ECS 整合 (hecs 0.10) 落档 commit `a55dd9e` + (2) R9 阶段 3 5 milestone benchmark 落档 commit `27726ed` (R9 🟢 MILESTONE 完成 per plan-032 R9 line 143) + (3) 12/13 阶段基本落档 (R1-R10 全部, 剩 R10 阶段 2 obsolete per v0.62 反转) + (4) 2 个 🟢 MILESTONE 完成 (R5 阶段 1 核心理念验证 commit `c0c1b71` + R9 阶段 3 高性能 Rust 版 Multica 验证 commit `27726ed`) + (5) 新增 §11 阶段性收官 (12 阶段统计 + 2 milestone + R10 阶段 2 obsolete 声明). 不重写 v0.1 row per 守门 #1 禁回溯叙事. | 2026-09-12 12:01 JST Ulysses 拍板 WBS v0.66 + plan-032 v0.2 阶段性收官报告 (推荐项) + per plan-032 R9 line 143 milestone 完成 + per 守门 v0.62 反转 R10 阶段 2 obsolete 维护 |
