# CANVAS-IMPL-PLAN-001

> **无限画布 P3-D.5 实施计划 v0.1** (per 日本 IPA SEC 標準 + DD-AGENT-RELATIONSHIP-001 v0.1 模板 + DD-CANVAS-{AGENT,GAMIFY}-001 v0.1 派生)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 詳細設計 → 実装 → テスト → リリース
> - 关联设计: `docs/design/DD-CANVAS-001.md` v0.1.1 (49KB 总册, 14 段) + `docs/design/DD-CANVAS-AGENT-001.md` v0.1 (139KB 专题 1, 15 段) + `docs/design/DD-CANVAS-GAMIFY-001.md` v0.1 (162KB 专题 2, 17 段) + `docs/design/BD-CANVAS-{001,AGENT,GAMIFY}-001.md` v0.1 (3 份 BD)
> - 关联需求: `docs/requirements/SRS-CANVAS-001.md` v1.2 (总册 78 项 = 46+32) + `SRS-CANVAS-AGENT-001.md` v1.3 (46 项) + `SRS-CANVAS-GAMIFY-001.md` v0.1.1 (32 项)
> - 触发: 2026-09-10 19:18 JST Ulysses 拍板"根据详细设计制作 spec 实施计划, 并加入 wbs"
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手) (5 角色签字栏 per AGENTS.md §3, per 守门 #14 v2/v3/v4 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
> - 日期: 2026-09-10 JST
> - 受众: 実装エンジニア / テストエンジニア / アーキテクト / SRE / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
> - 跨域范围 disclaimer: **5 域 Lead (player / economy / match / social / admin) ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 跨域通过 25 module 联动接口对接, 不直接调用其他 view)

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | CANVAS-IMPL-PLAN-001 |
| 文书名 | 无限画布 P3-D.5 实施计划 (Implementation Plan) |
| 版本 | v0.1 (初版, per P3-D.5 DD + BD 3 份 派生) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses) |
| 关联 commit | 待生成 (root 统一 commit, per 守门 #1 v15) |
| 关联文档 | `docs/design/DD-CANVAS-001.md` v0.1.1 + `DD-CANVAS-AGENT-001.md` v0.1 + `DD-CANVAS-GAMIFY-001.md` v0.1 + `BD-CANVAS-{001,AGENT,GAMIFY}-001.md` v0.1 + `SRS-CANVAS-{001,AGENT,GAMIFY}-001.md` v{1.2,1.3,0.1.1} + `DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB ARG 模板) + `frontend-canvas-design.md` v0.1 (V0.1 MVP design) |
| 范围 | P3-D.6 启动实装 (P0 36 项, 1-2 周, 5 域 Lead 真人未到位 Mavis 临时代签) + 跨 session 续做 (5 域 Lead 寻访 + CRDT 选型 + A12 WSS 选型 + V0.1 localStorage 冲突 解决 + 25 module 联动) |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 19:18 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 v0.62 反转)** | **初版落档, 10 段 IPA SEC 模板, 阶段 0-4 共 5 阶段 (docs / 基础 / 业务 / 集成 / 实装), 6 新 crate + 1 BFF + 14+15 张表 + 23+22 API + 5+2 WebSocket, 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT), 19 已知缺口 (含 3 P0 阻塞 + 1 跨 session 总结)** | **2026-09-10 19:18 JST Ulysses 拍板"根据详细设计制作 spec 实施计划, 并加入 wbs"** |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 STAR 平台 **无限画布 P3-D.5** 的实施计划 (Implementation Plan), 基于 P3-D.5 阶段产出的 3 份詳細設計 (DD-CANVAS-001 总册 v0.1.1 + DD-CANVAS-AGENT-001 专题 1 v0.1 + DD-CANVAS-GAMIFY-001 专题 2 v0.1) 提炼出:

- **6 新 Rust crate** (agent-domain / arg / arg-bridge / arg-effect / canvas-collab / api 扩展) + 1 BFF (envoy 独立 deployment, per 9/1 13:05 JST 偏好)
- **14 张表 + 15 张表 SQL DDL** (W/T/M 100% 覆盖, per 守门 #13, 0 混在)
- **23 API 端点 (A1-A10 13 + A11 5 + A12 5) + 5 WebSocket (A11 1 + A12 4)** + 22 BFF REST + 2 WebSocket (GAMIFY)
- **13 关键 class (C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15) + C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit`** (per v0.1.1 修复, C-16 + C-21 顺延, 跟 ARG 源不冲突)
- **5 状态机** (Edge / Agent 14 / Trust Score 5 档 `Untrusted/Low/Medium/High/VeryHigh` / Template Instance / Achievement) + 11 共享类型
- **4 关键时序图** (写关系 / 协作影响 / 成就评估 / 离线降级) + 2 补充 (A12 多人编辑 / Follow mode)
- **74 测试** (52 UT + 10 IT + 8 E2E + 4 PT) 跨域 ≥ 30

作为后续 P3-D.6 实施阶段的唯一依据 (per 守门 #1 禁回溯叙事, 后续 v0.x 修订行显式标).

### 1.2 In-Scope (P3-D.6 启动实装范围)

- **A1-A10 agent 管理** (28 项 SRS, 10 子能力: 节点 / 拓扑 / 状态实时同步 / handoff / worktree 关联 / 监控 / 聚类排序过滤 / 跨域引用 / settings 集成)
- **A11 ARG 图论构造** (10 项, 引用 ARG 源 `DD-AGENT-RELATIONSHIP-001.md` v0.1 §3-§8 模板 1:1 派生)
- **A12 多人编辑** (8 项, 引用 V0.1 `frontend-canvas-design.md` §4.1 模式 A + §4.6 PresenceCursor 升级)
- **G1-G12 游戏化** (32 项, 6 类基础 / 互动 / 排行榜 / 升级 / 技能 / 成就 + 6 升级路径)
- **25 module 联动** (worktree + work-item + comment + notification + audit + search + settings + agent-runtime + relation + automation + agent + ...)
- **6 类 NFR 详细落地** (性能 6 / 可靠性 3 / 安全 6 / 易用 5 / 可观测 3 / ARG 6 + MU-CONS-01)
- **5 角色签字栏** (架构师 / SRE Lead / 平台 / 评审主持 / PM, Mavis 接手代签)
- **3 P0 阻塞已知缺口** (#11 A12 WSS 选型 + #13 V0.1 localStorage 冲突 + #15 CRDT 选型, 5 域 Lead 真人到位后决策)

### 1.3 Out-of-Scope

- **5 域 Lead 真人寻访** (per 守门 #14 v2 拍板 D, 跨 session 续做, 5 域各 1 份内推 brief 已落档 `docs/recruitment/5-business-domain-lead-referral.md` v0.1)
- **真实 LLM 接入** (per 守门 #23 v2, G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2)
- **V0.1 game 5 份 PHASE 报告** (per 守门 #1 禁回溯叙事, 9/5 落地 125 tests pass, 仅画布集成引用, 不重写)
- **A12 多人编辑 WSS 真实端到端集成测试** (3 P0 阻塞已知缺口, 等 5 域 Lead 真人 + CRDT 选型 + A12 WSS 选型拍板)

### 1.4 文档结构 (per AGENTS.md §3 7 段结构 + IPA SEC 模板)

10 段 + 5 附录 (跨专题引用 / 跨域组件 / 已知缺口 / 签字栏 / 修订履历).

---

## §2 实施总览 (per P3-D.5 DD 5 view 跨域)

### 2.1 5 view 跨域实施矩阵

| View | A1-A10 (28 项) | A11 (10 项) | A12 (8 项) | G1-G12 (32 项) | 跨域总计 |
|---|---|---|---|---|---|
| **機能 (Functional)** | 28 项 | 10 项 | 8 项 | 32 项 | 78 项 |
| **データ (Data)** | 14 张表 W/T/M 100% | (ARG 4 表, A11.5 必含) | (A12 7 表, A12.8 必含 `canvas_multi_user_audit` 100% RLS 13 类) | 15 张表 W/T/M 100% 0 混在 (M5 + T4 + W6) | 29 张表 跨域汇总 |
| **動作 (Behavior)** | 6 时序图 (A3 14 状态机 + A4 worktree + A5 work-item + A7 监控 + A8 聚类 + A9 跨域引用) | 4 时序图 (写关系 + 协作影响 + 成就评估 + 离线降级) | 2 时序图 (多人编辑 + Follow mode) | 4 时序图 (sticky 聚类 / dot voting / confetti 触发 / daily challenge) | 16 时序图 跨域 ≥ 8 |
| **モジュール (Module)** | 6 新 crate + 18 Rust module + 6 BFF | 4 ARG crate (arg / arg-bridge / arg-effect) | 1 BFF (envoy 独立 deployment, 5 REST + 4 WSS) | 5 Domain + 7 BFF + 14 前端 (per frontend-canvas-design.md) | 24 组件 跨域 |
| **ネットワーク (Network)** | 13 REST 端点 (A1-A10) | 5 REST + 1 WSS 端点 (A11) | 5 REST + 4 WSS 端点 (A12) | 22 BFF REST + 2 WebSocket (GAMIFY) | 32 REST + 5 WSS 跨域 |

### 2.2 跨域实施路径 (per 守门 #19 v19 [P] docs 同步 + 守门 #1 禁回溯叙事)

实施按 5 view 跨域顺序推进, **5 阶段 (阶段 0 → 阶段 4)**, 每阶段独立可发布, Mavis 接手 root session 一次性推进 (0 子代理调用, per 守门 #9 #3 实证 5/5 RPC 不可靠).

### 2.3 5 阶段实施时间表 (per 守门 #4 token-OLU 估算)

| 阶段 | 周 | Token OLU 估算 | 累计 SRE·周 (per STAR-OLU-001 v0.1 1 SRE·周 = 1.2M tokens) | 落地里程碑 |
|---|---|---|---|---|
| 阶段 0 docs 阶段 (已完成) | n/a | ~3.43M | 2.86 SRE·周 (累计 P3-D.5 13 commit, per §4.31) | P3-D.5 文档 100% 落档 (3 SRS + 3 BD + 3 DD + 1 协调性检查 + IPA SEC 修复 v1+v2) |
| 阶段 1 基础 (P3-D.6.1) | 1 周 | ~1.5M | ~1.25 SRE·周 | 6 新 crate 骨架 + DDL 14+15 张 + 多租户路由 + ARG 5 模板 |
| 阶段 2 业务 (P3-D.6.2) | 1-2 周 | ~2.0M | ~1.67 SRE·周 | A1-A10 + A11 + A12 + G1-G12 业务逻辑 + 13 关键 class + 5 状态机 |
| 阶段 3 集成 (P3-D.6.3) | 0.5 周 | ~0.5M | ~0.42 SRE·周 | 25 module 联动 + 23+22 API + 5+2 WSS + 1 BFF (envoy) |
| 阶段 4 实装 (P3-D.6.4) | 0.5-1 周 | ~1.0M | ~0.83 SRE·周 | 74 测试 + 守门 #1 v25 实证 + 6 类 NFR benchmark + 19 已知缺口交叉验证 |
| **合计** | **3-4 周** | **~5.0M** | **~4.17 SRE·周 (含 docs 阶段)** | **P3-D.6 完整实装** |

---

## §3 阶段划分 (P3-D.6 启动实装 5 阶段, per 守门 #9 v19 Mavis 自驱)

### 阶段 0 docs 阶段 (已完成, per §4.31 v0.16 registry + §4.30 v0.15 registry + WBS v0.82 升档)

| 落地 | 大小 | 触发 |
|---|---|---|
| 3 SRS (双核心 78 项索引, 14+15 张表 W/T/M 100% 覆盖) | 305KB (SRS-001 50KB + SRS-AGENT-001 155KB + SRS-GAMIFY-001 92KB) | 17:08 双核心 + 17:21 ARG + 17:34 v0.63 反转多人编辑 |
| 3 BD (5 view 跨域 + 29 张表 + 32 API + 5 WebSocket) | 266KB (BD-001 50KB + BD-AGENT 105KB + BD-GAMIFY 111KB) | 18:00 BD 拍板 |
| 3 DD (10-17 段 + 5 附录 + 13 关键 class + 5 状态机 + 11 共享类型 + 4-6 时序图) | 350KB (DD-001 49KB + DD-AGENT 139KB + DD-GAMIFY 162KB) | 18:25 DD 拍板 |
| 1 协调性检查报告 (10 项 7 通过 + 3 冲突修复) | 21KB | 18:30 协调性 拍板 |
| IPA SEC 合规性修复 v1+v2 (5 文档 + 2 SRS 修订履历 + 1 报告 §0 补) | 6 files + 2 修订 | 19:06 + 19:14 IPA SEC 拍板 |
| **累计** | **~3.43M tokens / 2.86 SRE·周 / 13 commit** | **6 阶段拍板 17:08/17:21/17:34/18:00/18:25/18:30 + 2 修复 19:06/19:14** |

### 阶段 1 基础 (P3-D.6.1, 1 周, 1.25 SRE·周)

**目标**: 6 新 crate 骨架 + 14+15 张表 SQL DDL 落档 + 1 BFF 骨架 + 25 module 联动接口定义.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| 1.1 `crates/agent-domain/` 新 crate 骨架 (per DD-AGENT §3.1 module 布局) | Cargo.toml + lib.rs | ~0.1M | #1 v25 (cargo test 单 crate) |
| 1.2 `crates/arg/` + `crates/arg-bridge/` + `crates/arg-effect/` 3 新 crate 骨架 (per DD-AGENT §3.1, A11 派生 ARG 源) | 1.1 | ~0.3M | #13 W/T/M 100% + #1 v25 |
| 1.3 `crates/canvas-collab/` 新 crate 骨架 (per DD-AGENT §4.14 A12 sub-class) | 1.1 | ~0.2M | #1 v25 + #13 b Transaction 100% audit |
| 1.4 `crates/api/src/{agent,arg,canvas_collab}/` 3 新 module (per DD-AGENT §3.3 跨域) | 1.1-1.3 | ~0.2M | #1 v25 + #14 v4 |
| 1.5 `bff/src/collaboration/` BFF 骨架 (per DD-AGENT §3.1, envoy 独立 deployment per 9/1 13:05 JST 偏好) | 1.3 | ~0.2M | #6 PowerShell only + #14 v2 |
| 1.6 14+15 张表 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表 跨域汇总, per 守门 #13) | 1.1-1.5 | ~0.3M | #13 W/T/M 100% 覆盖 + 0 混在 + #5 env |
| 1.7 25 module 联动接口定义 (worktree + work-item + comment + notification + audit + search + settings + agent-runtime + relation + automation + agent) | 1.1-1.6 | ~0.2M | #1 + #19 v19 累积规 不破坏 V0.1 |

**累计**: 1.5M tokens / 1.25 SRE·周.

### 阶段 2 业务 (P3-D.6.2, 1-2 周, 1.67 SRE·周)

**目标**: A1-A10 + A11 + A12 + G1-G12 全部业务逻辑 + 13 关键 class + 5 状态机 + 11 共享类型.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| 2.1 A1-A10 agent 管理 业务逻辑 (28 项, 10 子能力, per DD-AGENT §4) | 1.1 + 1.4 | ~0.5M | #1 v25 + #11 缺标比错标 + #13 |
| 2.2 A11 ARG 图论构造 (10 项, per DD-AGENT §4 + ARG 源 1:1) | 1.2 + 1.4 | ~0.4M | #1 v25 + #13 d (Transaction 100% audit) + ARG 派生 |
| 2.3 A12 多人编辑 业务逻辑 (8 项, per DD-AGENT §4.14 + §3.1 A12 sub-class) | 1.3 + 1.5 | ~0.4M | #1 v25 + #13 + #11 缺标 + 3 P0 阻塞显式 |
| 2.4 G1-G12 游戏化 业务逻辑 (32 项, per DD-GAMIFY §4) | 1.1 | ~0.5M | #1 v25 + #13 W/T/M 0 混在 + #23 v2 G5 mock 锁 |
| 2.5 13 关键 class 落地 (C-1/C-2/C-3/C-4/C-7/C-8/C-11/C-12/C-13/C-14/C-15 + C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit`, per v0.1.1 修复) | 2.1-2.4 | ~0.1M | #1 v25 + #14 v2 + 跟 ARG 源不冲突 |
| 2.6 5 状态机 + 11 共享类型 Rust enum + struct 落档 (per DD-AGENT §5+§6 + DD-GAMIFY §5+§6) | 2.1-2.4 | ~0.1M | #1 v25 + Trust Score 5 档 `Untrusted/Low/Medium/High/VeryHigh` 统一 |

**累计**: 2.0M tokens / 1.67 SRE·周.

### 阶段 3 集成 (P3-D.6.3, 0.5 周, 0.42 SRE·周)

**目标**: 25 module 联动端到端集成 + 23+22 API 端点 + 5+2 WebSocket + 1 BFF (envoy 独立 deployment).

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| 3.1 23 REST API 端点 落档 (A1-A10 13 + A11 5 + A12 5, per DD-AGENT §7.3) | 2.1-2.3 | ~0.2M | #1 v25 + #5 env + OpenAPI 3.1 spec |
| 3.2 5 WebSocket 端点 落档 (A11 1 `/ws/arg/events` + A12 4 `wss://canvas-*`, per DD-AGENT §7.1) | 2.2 + 2.3 | ~0.15M | #1 v25 + TLS 1.3+ + throttle 50ms |
| 3.3 22 BFF REST 端点 落档 (G1-G12 22 端点, per DD-GAMIFY §7.1) | 2.4 | ~0.1M | #1 v25 + #6 PowerShell + envoy middleware |
| 3.4 25 module 联动端到端 (worktree + work-item + comment + notification + audit + search + settings + agent-runtime + relation + automation + agent) | 1.7 + 3.1-3.3 | ~0.05M | #1 + #19 v19 累积规 |

**累计**: 0.5M tokens / 0.42 SRE·周.

### 阶段 4 实装 (P3-D.6.4, 0.5-1 周, 0.83 SRE·周)

**目标**: 74 测试 + 守门 #1 v25 实证 + 6 类 NFR benchmark + 19 已知缺口交叉验证.

| 任务 | 依赖 | Token 估 | 守门 |
|---|---|---|---|
| 4.1 52 UT (A1-A12 + G1-G12 单元测试, per DD-AGENT §10.1 + DD-GAMIFY §10) | 2.1-2.4 | ~0.3M | #1 v25 + cargo test 单 crate |
| 4.2 10 IT (per DD-AGENT §10.2 + DD-GAMIFY §10) | 3.1-3.3 | ~0.2M | #1 v25 + testcontainers-rs (P3-E 实证) |
| 4.3 8 E2E (per DD-AGENT §10.3 + DD-GAMIFY §10) | 3.1-3.3 | ~0.2M | #1 v25 + MSW handler + Playwright (V0.1 实证) |
| 4.4 4 PT (per DD-AGENT §10.4 + DD-GAMIFY §10) | 2.1-2.4 | ~0.1M | #1 v25 + k6 (V0.1 实证) |
| 4.5 守门 #1 v25 实证 (cargo check --workspace --lib + cargo fmt + cargo clippy + cargo test) | 4.1-4.4 | ~0.1M | #1 v25 |
| 4.6 6 类 NFR benchmark (性能 / 可靠性 / 安全 / 易用 / 可观测 / ARG + MU-CONS-01, per DD-AGENT §11 + DD-GAMIFY §11) | 4.1-4.4 | ~0.1M | #1 + #23 v2 (G5 mock) + #11 缺标 |

**累计**: 1.0M tokens / 0.83 SRE·周.

---

## §4 实装步骤 (按 A1-A12 子能力 + 5 域 module 联动)

### 4.1 A1-A10 agent 管理 (per SRS-CANVAS-AGENT-001 v1.3 + DD-CANVAS-AGENT-001 v0.1)

| A ID | 子能力 | 关键 class / module | 跨域 | Token 估 | 守门 |
|---|---|---|---|---|---|
| A1 | agent 节点卡 | `crates/agent-domain/src/agent.rs` `AgentNode` struct 14 字段 | A2 + A5 | ~0.05M | #1 v25 + #13 |
| A2 | 拓扑 (handoff + 5 域 + 父子 + pipeline) | `agent_relationship_edge.rs` `AgentRelationshipEdge` struct 14 字段 + `edge_ops.rs` | A1 + A11 | ~0.05M | #1 v25 + #13 |
| A3 | 状态实时同步 (14 状态机) | `status_sync.rs` 14 enum + SSE / WSS | A2 + A6 | ~0.05M | #1 v25 + NFR-AGENT-PERF-02 |
| A4 | worktree 关联 | `worktree_relation.rs` `AgentWorktreeRelation` struct 6 字段 | A1 + V0.1 | ~0.03M | #1 v25 + 不破坏 V0.1 (per #19 v19) |
| A5 | work-item 关联 | `work_item_relation.rs` `AgentWorkItemRelation` struct 6 字段 | A1 + V0.1 | ~0.03M | #1 v25 + 不破坏 V0.1 |
| A6 | agent-runtime (3 tier) | `crates/agent-runtime/` 扩展 | A3 + V0.1 | ~0.05M | #1 v25 + #6 PowerShell |
| A7 | 监控 (realFetch + audit) | `agent_monitor.rs` + 3 端点 (A7.1/A7.2/A7.3) | A3 + audit | ~0.05M | #1 v25 + audit 必填 |
| A8 | 聚类 + 排序 + 过滤 | `cluster_sort_filter.rs` + UI | A1 | ~0.05M | #1 v25 + UI 一致性 |
| A9 | 跨域引用 (URL param) | `cross_view_jump.rs` | A1 + 25 module | ~0.03M | #1 v25 + URL param 透传 |
| A10 | settings 集成 (V0.1 已实装) | `AgentSettingsTab.tsx` 复用 | V0.1 | 0 (V0.1 已实装) | 守门 #1 禁回溯叙事 |

**A1-A10 累计**: 0.39M tokens / 28 项 / 6 类 enum 跨域

### 4.2 A11 ARG 图论构造 (per SRS-CANVAS-AGENT-001 v1.3 A11 + DD-CANVAS-AGENT-001 v0.1 §3.1, 引用 ARG 源)

| 任务 | 关键 class / module | Token 估 | 守门 |
|---|---|---|---|
| A11.1 10 类关系 (4 核心 + 6 扩展) | `arg/edges.rs` + `arg-edge.tsx` UI | ~0.1M | #1 v25 + #13 d (SCD Type 2) |
| A11.2 关系编辑 (拖拽 + 3 字段) | `RelationshipEditor.tsx` (per DD-AGENT §4.12, 跟 ARG 源 1:1) | ~0.05M | #1 v25 + NFR-AGENT-PERF-03 |
| A11.3 4 维度 effect (Dispatch/Context/Trust/Output) | `arg-effect/` 4 crate + 4 effect tier | ~0.1M | #1 v25 + #13 d (TrustEngine 100% audit) |
| A11.4 5 团队模板 1-click 部署 | `team_template.rs` + `TemplateGallery.tsx` | ~0.05M | #1 v25 + NFR-AGENT-ARG-05 |
| A11.5 关系 audit log (4 表 W/T/M 100%) | `agent_relationship_edges_audit.rs` (Transaction) | ~0.05M | #13 d (Transaction 100% audit) |
| A11.6 关系权重视觉化 (Trust Score 5 档) | `trust_score_tier.rs` 5 enum + 颜色渐变 | ~0.02M | Trust Score 5 档 `Untrusted/Low/Medium/High/VeryHigh` 统一 |
| A11.7 archive / restore (物理删除禁止) | `edge_ops.rs::archive/restore` (SCD Type 2) | ~0.02M | #13 b (物理删除禁止) |
| A11.8 关系版本控制 (optimistic lock) | `edge_ops.rs::update` (version + 1) | ~0.02M | #13 c (SCD Type 2) |
| A11.9 同步状态 4 状态 (synced/syncing/error/offline) | `SyncStatusBadge.tsx` | ~0.01M | #1 v25 |
| A11.10 /agent-relationships 跨域跳 | `cross_view_jump.rs::relationships` | ~0.03M | #1 v25 + URL param |

**A11 累计**: 0.45M tokens / 10 项 / 4 维度 effect

### 4.3 A12 多人编辑 (per SRS-CANVAS-AGENT-001 v1.3 A12 + DD-CANVAS-AGENT-001 v0.1 §3.1 + §4.14)

| A ID | 子能力 | 关键 class / module | Token 估 | 守门 + 已知缺口 |
|---|---|---|---|---|
| A12.1 多人同时编辑 (10 并发) | `canvas-collab/src/collab.rs` + WSS `/canvas-collab` | ~0.05M | **#11 P0 阻塞 (WSS 选型)** |
| A12.2 实时 cursor 同步 | `PresenceCursor` struct 8 字段 + WSS `/canvas-presence` | ~0.05M | #1 v25 + throttle 50ms |
| A12.3 元素增删改同步 (替代 V0.1 localStorage) | `CanvasElementBackend` struct 11 字段 (per C-25 v0.1.1) + WSS | ~0.1M | **#13 P0 阻塞 (V0.1 localStorage 冲突)** |
| A12.4 Follow mode | `CanvasFollower` struct 5 字段 + WSS `/canvas-follow` | ~0.05M | #1 v25 + NFR-AGENT-PERF-06 |
| A12.5 评论线程 + @ 提醒 | `CanvasComment` struct 9 字段 + WSS `/canvas-comments` | ~0.1M | #1 v25 + notification 域对接 |
| A12.6 CRDT 选型 (Yjs / Automerge / LWW) | (待 5 域 Lead 真人拍板) | ~0.05M | **#15 P0 阻塞 (CRDT 选型)** |
| A12.7 3 级权限 (view / comment / edit) | `CanvasPermission` struct 7 字段 + BFF middleware | ~0.05M | #13 RLS 13 类 |
| A12.8 audit log 多人操作 | `CanvasMultiUserAudit` struct 9 字段 (per C-26 v0.1.1) | ~0.05M | #13 b (Transaction 100% audit) + SCD Type 2 |

**A12 累计**: 0.50M tokens / 8 项 / 3 P0 阻塞 (WSS 选型 / localStorage 冲突 / CRDT 选型) + 1 跨 session 总结 (#17 A12 WSS + CRDT + 5 域 Lead + 25 module 跨 session 续)

### 4.4 G1-G12 游戏化 (per SRS-CANVAS-GAMIFY-001 v0.1.1 + DD-CANVAS-GAMIFY-001 v0.1)

| G ID | 子能力 | 关键 class / module | 跨子能力 | Token 估 | 守门 |
|---|---|---|---|---|---|
| G1-G2 | 6 类基础 (avatar / level / xp / skill-tree / class / badges) | `avatar_node.rs` + `level_node.rs` + `xp_node.rs` + `skill_tree.rs` + `class.rs` + `badge.rs` | G3-G6 | ~0.15M | #1 v25 + V0.1 game 派生 (per 守门 #1 禁回溯叙事) |
| G3 | score 评分 (3 维度) | `score.rs` + `score_rule.rs` | G1 + G9 | ~0.05M | #1 v25 |
| G4 | 升级 + 技能 (4.1 公式 + 4.2 动画 + 4.3 复合) | `level_from_xp.rs` + `apply_claim.rs` | G1 | ~0.05M | #1 v25 |
| G5 | sticky note 聚类 AI | `cluster_mock.rs` Rust pure + `ai_edit_mock.py` subprocess | G1 | ~0.05M | **#23 v2 (mock 锁, 不开 LLM)** |
| G6 | dot voting | `vote.rs` + `vote_state_snapshot.rs` | G9 | ~0.05M | #1 v25 + 5 票/session |
| G7 | reaction 表情 (3s 淡出) | `reaction.rs` + `reaction_picker.tsx` | G1 | ~0.03M | #1 v25 + 3s TTL |
| G8 | confetti (3s 短 TTL) | `confetti.rs` + `confetti_overlay.tsx` | G2.3 | ~0.03M | #1 v25 + 3s TTL |
| G9 | 排行榜 (per workspace + per tenant) | `leaderboard_view.rs` + WSS `/gamify/events` | G3 + G6 | ~0.05M | #1 v25 + per-tenant admin 13 租户 |
| G10 | daily challenge + streak (7 天 Milestone) | `daily_challenge_view.rs` + `streak_state.rs` | G1 | ~0.05M | #1 v25 + TZ + 中断清零 |
| G11 | 道具 (powerup) + 背包 (inventory) | `powerup.rs` + `inventory_item.rs` | G2 | ~0.1M | #1 v25 + 24h 短 TTL |
| G12 | V0.1 game 5 份 PHASE 集成 (Game/Roguelike/Manga/Theme/Settings) | (V0.1 已实装, 仅画布集成) | V0.1 | 0 (V0.1 集成) | **#1 禁回溯叙事 (V0.1 不重写)** |

**G1-G12 累计**: 0.65M tokens / 32 项 / V0.1 5 份 PHASE 派生

### 4.5 25 module 联动 (per DD-AGENT §3.1 + 守门 #19 v19 累积规不破坏 V0.1)

| Module | A1-A10 | A11 | A12 | G1-G12 | Token 估 |
|---|---|---|---|---|---|
| worktree | A4 | - | - | - | ~0.02M |
| work-item | A5 | - | - | - | ~0.02M |
| comment | - | - | A12.5 | - | ~0.02M |
| notification | A7 | A11.5 | A12.5 | G7 | ~0.03M |
| audit | A7 + A11.5 | A11.5 | A12.8 | - | ~0.03M |
| search | A8 | - | - | G3 | ~0.02M |
| settings | A10 | - | - | G4 | (V0.1 复用) |
| agent-runtime | A6 | A11.3 | - | - | ~0.03M |
| relation | A2 | A11.1 | - | - | (跟 A11 合并) |
| automation | A7 + A8 | - | A12.3 | G5 | ~0.05M |
| agent (核心) | A1-A10 | A11 | A12 | G1-G12 | (跟 4.1-4.4 合并) |
| 其他 14 module | (略) | | | | ~0.1M |

**25 module 联动累计**: 0.34M tokens.

---

## §5 关键 class / module 落地清单 (per DD 模板)

### 5.1 13 关键 class (per DD-AGENT §4 + DD-GAMIFY §4 + v0.1.1 修复)

| C ID | Class | 字段数 | 方法数 | 跨域 | 派生 |
|---|---|---|---|---|---|
| C-1 | `MemgraphClient` | 5 | 5 | A11 | DD-AGENT-REL v0.1 §4.1 |
| C-2 | `AgentNodeOps` | 2 | 5 | A11 | DD-AGENT-REL v0.1 §4.2 |
| C-3 | `EdgeOps` | 3 | 8 | A11 | DD-AGENT-REL v0.1 §4.3 |
| C-4 | `TemplateOps` | 3 | 2 | A11 | DD-AGENT-REL v0.1 §4.4 |
| C-7 | `MemgraphEventListener` | 2 | 2 | A11 | DD-AGENT-REL v0.1 §4.5 |
| C-8 | `LangGraphStateUpdater` (PyO3) | 1 | 2 | A11 | DD-AGENT-REL v0.1 §4.6 |
| C-11 | `ARGDispatchRouter` | 1 | 2 | A11.3 | DD-AGENT-REL v0.1 §4.7 |
| C-12 | `ARGContextInjector` | 1 | 5 | A11.3 | DD-AGENT-REL v0.1 §4.8 |
| C-13 | `ARGTrustEngine` | 2 | 3 | A11.3 | DD-AGENT-REL v0.1 §4.9 |
| C-14 | `ARGOutputEvaluator` | 2 | 4 | A11.3 | DD-AGENT-REL v0.1 §4.10 |
| C-15 | `ARGAchievementEngine` | 3 | 1 | A11.4 | DD-AGENT-REL v0.1 §4.11 |
| C-16 | `RelationshipEditor` (TSX) | (TSX props) | 5 | A11.2 | DD-AGENT-REL v0.1 §4.12 (1:1 派生, 跟 ARG 源一致) |
| C-21 | `ARGController` (Axum) | (Axum router) | 13 | A11.3 | DD-AGENT-REL v0.1 §4.13 (1:1 派生, 跟 ARG 源一致) |
| **C-25** | **`CanvasElementsBackend`** | **12** | **10** | **A12.3** | **本批新增, v0.1.1 修复 (per 协调性检查报告 §3.1, 跟表名 `canvas_elements_backend` 一致)** |
| **C-26** | **`CanvasMultiUserAudit`** | **9** | **6** | **A12.8** | **本批新增, v0.1.1 修复 (per 协调性检查报告 §3.2 + 守门 #13 Transaction, 跟表名 `canvas_multi_user_audit` 一致)** |

**13+2 关键 class 总计**: 跨域, 15 / 24 (C-1..C-24) = 62.5% 落地, 9 留 P3-C 实装阶段 (C-5/C-6/C-9/C-10/C-17/C-18/C-19/C-20/C-22/C-23/C-24).

### 5.2 5 状态机 (per DD-AGENT §5 + DD-GAMIFY §5)

| 状态机 | 状态数 | 状态转移函数 | 跨域 |
|---|---|---|---|
| Edge | 3 (Created/Updated/Archived) | `can_transition(from, to) -> bool` | A11.5 + A11.7 |
| Agent | 14 (queued/spawning/initializing/...) | `can_transition_agent(from, to) -> bool` | A3.1 |
| **Trust Score 5 档** | **5 (Untrusted/Low/Medium/High/VeryHigh)** | `TrustScoreTier::from_score(score) -> Self` | A11.3 |
| Template Instance | 3 (Pending/Active/Expired) | `transition_template_instance(state, now, expires_at) -> bool` | A11.4 |
| Achievement | 2 (Locked/Unlocked) | `unlock_achievement(unlocks, code) -> bool` (幂等) | A11.4 + G2.4 |

**5 状态机总计**: 跨域, 27 状态 + 14 转移函数, 5/5 落地 (100%).

### 5.3 11 共享类型 (per DD-AGENT §6 + DD-GAMIFY §6, 1:1 派生自 DD-AGENT-REL v0.1 §3.2.5)

ARGEvent / Decision / Output / Verdict / LLMClient / AchievementUnlock / TemplateInstance / ARGState / EscalationInfo / PeerReviewVerdict / ChallengePrompt.

**11 共享类型 跨域**: A11 + A12 + G5 (LLMClient mock) + 25 module 联动.

### 5.4 4-6 时序图 (per DD-AGENT §8 + DD-GAMIFY §8)

ARG (4 必含 + 2 补充 A12) + GAMIFY (4) = **10 时序图 跨域 ≥ 8**.

---

## §6 测试策略 (UT/IT/E2E/PT, 74 测试 跨域 ≥ 30)

### 6.1 52 UT (per DD-AGENT §10.1 + DD-GAMIFY §10)

- **A1-A10 UT**: 30 UT (每个子能力 3 UT 跨域)
- **A11 UT**: 8 UT (5 模板 + 4 维度 effect + 10 类关系)
- **A12 UT**: 8 UT (8 子能力各 1 UT)
- **G1-G12 UT**: 6 UT (代表性: avatar / level / sticky 聚类 / confetti / vote / streak)

### 6.2 10 IT (per DD-AGENT §10.2 + DD-GAMIFY §10)

- **A11 IT**: 5 (Memgraph stub + EdgeOps::create V2 优先 + 4 维度 effect reload)
- **A12 IT**: 3 (WSS 连接 + 多用户并发 + audit 写入)
- **G IT**: 2 (sticky 聚类 mock + leaderboard 实时)

### 6.3 8 E2E (per DD-AGENT §10.3 + DD-GAMIFY §10)

- **A1-A10 E2E**: 3 (节点创建 / 拓扑编辑 / 跨域引用)
- **A11 E2E**: 2 (关系编辑 + 5 模板 1-click)
- **A12 E2E**: 2 (多人编辑 + Follow mode, **需 #11 P0 阻塞 WSS 选型拍板**)
- **G E2E**: 1 (sticky 聚类 + 投票 + confetti 联动)

### 6.4 4 PT (per DD-AGENT §10.4 + DD-GAMIFY §10)

- A11.2 边创建 latency P95 < 200ms
- A11.3 Cypher P95 < 500ms
- A11.6 事件推送 < 100ms (10 并发)
- A12 多人编辑元素增删改 P95 < 200ms

### 6.5 守门 #1 v25 实证 (per 守门 #1 累积规 v1-v26 + v25 单 crate 模式)

```
cargo check --workspace --lib -j 4   # 0 err
cargo fmt -p <crate> -- --check      # 0 diff
cargo clippy -p <crate> --lib -j 4   # 0 warnings (守门 #7 v3 advisory, per PR #12)
cargo test -p <crate> --lib -j 4     # N/N PASS (单 crate 实证, per 守门 #1 v25)
```

---

## §7 风险 / 依赖 / 跨 session 续做 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标)

### 7.1 3 P0 阻塞已知缺口 (5 域 Lead 真人到位后拍板)

| # | 缺口 | 跨 session 续做 | Token OLU |
|---|---|---|---|
| **#11** | **A12.1 多人同时编辑 WSS 选型** (per 8/9/2026 5 域 Lead 真人未到位, A12.1 实证 WSS 选型待 #17 拍板) | ✅ 等 5 域 Lead 真人到位 | 0.2M |
| **#13** | **A12.3 V0.1 localStorage 冲突** (V0.1 用 localStorage + zustand persist, A12.3 用 backend 持久化 + WSS, 需 V0.1 降级为离线 fallback) | ✅ 等 V0.1 走 fallback 实证 | 0.3M |
| **#15** | **A12.6 CRDT 选型** (Yjs / Automerge / LWW, 3 选 1, 5 域 Lead 真人拍板) | ✅ 等 5 域 Lead 真人到位 | 0.2M |

### 7.2 5 域 Lead 真人寻访 (per 守门 #14 v2 + 内推 brief v0.1)

- 5 域各 1 份内推话术 + token-OLU 11-15 SRE·周估 + 6 周满员 timeline T0-T5
- 已落档 `docs/recruitment/5-business-domain-lead-referral.md` v0.1
- 解除 P3-D.6 + H2 5 项 Blocker (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- 真人到位后追溯签字覆盖修订历史 (per 守门 #3 5 域独立 Lead + 守门 #1 禁回溯叙事)

### 7.3 跨 session 续做项 (per v0.68 §3 + v0.69 §3 + v0.70 §3)

- **v0.70 v32 守门激活** (per 守门 #14 v4 v0.62 反转, Mavis 审核 author=Ulysses) — 已落档 `docs/guardian/v32_audit_boundary.md` 5.6KB
- **v0.71 envoy 业务路由 + TLS 自动签发** (per 9/1 13:05 JST 偏好) — 已落档 `tools/star-flash-mock/k3s/envoy-business-config.yaml` v0.1
- **ARG.10 DDD Review G-9/G-4/G-10** — 5 域 Lead 真人到位后 DDD Review 拍板
- **ARG.11 5 域 Lead 真人到位 (追溯签字覆盖修订历史)** — 等真人到位

### 7.4 关键依赖 (per 守门 #1 累积规 + 守门 #19 v19 累积规)

- **rust toolchain**: 1.75+ (per Cargo.toml)
- **memgraph**: V0.1 + r2d2-memgraph crate 落地 (per ARG.10 G-1 v0.81 stub path)
- **postgres**: star-pg-adapter 11 Repository (per v0.85 P0-4 Stage 3.2 收官)
- **langgraph**: Python 3.11+ + PyO3 binding
- **bff envoy**: 独立 deployment (per 9/1 13:05 JST 偏好)
- **V0.1 不破坏**: 守门 #19 v19 累积规, agent-game / roguelike / manga / theme / settings 5 份 PHASE 报告不重写

---

## §8 守门合规 (per 守门 #1 累积规 v1-v26 + 守门 #14 v2/v3/v4 + 守门 #9 v19 + 守门 #11 缺标比错标 + 守门 #13 W/T/M 100% + 守门 #23 v2 AI mock)

### 8.1 守门 19 项 主守门

| # | 内容 | 跨阶段必跑 | 实证 (per 守门 #1 累积规) |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | 阶段 1-4 | cargo check + fmt + clippy + test 0 错 + 测试全过 |
| #1 v15 | docs 同步饱和边界 | 阶段 0-1 | 6 阶段拍板 6 commit 落档, 本次实施计划 = 第 7 次新事件触发 (per §4.30 v0.15 + §4.31 v0.16) |
| #1 v19 | agent 交互 Python 化 | 阶段 0 | `wbs_v086_insert.py` idempotent Python 脚本 (本批) |
| #1 v25 | cargo test 改单 crate | 阶段 1-4 | per PR #12 实证 |
| #3 | 5 域独立 Lead ≠ Star 22 DDD | 阶段 0-4 | disclaimer 5 处显式 (per 8/31 22:45 JST Q1-D 拍板) |
| #5 | env 安全 hard ban | 阶段 1-4 | $env:MEMGRAPH_BOLT_URL / $env:DATABASE_URL 走 stdin pipe, 不打印 |
| #6 | PowerShell only | 阶段 1-4 | bash 不直用, 走 pwsh 包装 |
| #7 | 0 unsafe | 阶段 1-4 | `cargo clippy -p <crate> --lib -j 4 -- -D warnings` (advisory per #7 v3) |
| #9 | 子代理 commit 实证 | 阶段 1-4 | Mavis 接手 root session 一次性, 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠) |
| #9 v19 | Mavis 自驱 | 阶段 0-4 | 19:18 JST 拍板 → 19:25 JST 落实施计划 + WBS v0.86 + §4.32 + registry v0.17 (per 守门 #9 v19 + 守门 #1 v19) |
| #10 | commit author=Ulysses | 阶段 0-4 | 1 commit 5 files (实施计划 + WBS + §4.32 + registry + insert script) author=Ulysses |
| #11 | 缺标比错标 | 阶段 0-4 | 19 已知缺口 + 3 P0 阻塞 + 1 跨 session 总结 #17, 显式列 §7 |
| #12 | AI 文档治理 | 阶段 0 | BAS 引用必 `git log -p --follow` 实证, 禁回溯叙事 |
| #13 | DB W/T/M 三類横展 100% 覆盖 | 阶段 1 | 14+15 = 29 张表 100% 覆盖 跨域汇总, 0 混在 |
| #14 v2 | 5 域 Lead Mavis 临时代签 | 阶段 0-4 | 真人到位后追溯签字 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) |
| #14 v3 | Mavis 永久代签 | 阶段 0-4 | 5 角色签字栏 author=Ulysses |
| #14 v4 | v0.62 反转 | 阶段 0-4 | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST) |
| #15 | docs 同步饱和 | 阶段 0-4 | 6 阶段拍板 6 commit + 2 IPA SEC 修复 2 commit, 累计 13 commit |
| #19 v19 | 不破坏 V0.1 | 阶段 1-4 | 25 module 联动 + V0.1 game 5 份 PHASE 派生, 0 重写 |
| #23 v2 | AI mock 锁 | 阶段 2 | G5 sticky 聚类走 mock + cluster_mock + ai_edit_mock.py subprocess, 真实 LLM 留 P2 |

### 8.2 守门 v3x 候选 (per AGENTS.md §4.1.1)

| 候选 | 状态 | 落地情况 |
|---|---|---|
| v27 子代理 RPC 失败 fallback | 🟢 active (v0.61 拍板激活) | `scripts/automation/guardian/v27_rpc_fallback.py` |
| v28 拍板必带推荐项 | 🟢 active (v0.56 拍板激活) | `scripts/automation/guardian/v28_recommendation.py` |
| v29 docs 同步饱和 40+ 主动告警 | 🟢 active (v0.56 拍板激活) | `scripts/automation/guardian/v29_docs_saturation.py` |
| v30 Mavis 永久代签 | 🟢 active (v0.62 反转) | v0.62 反转后 v32 替代 |
| v31 5 域 Lead 真人到位追溯签字 | 🟢 active (v0.62 反转) | v0.62 反转后 v32 替代 |
| v32 Mavis 审核 author=Ulysses | 🟢 active (v0.70 拍板激活) | `docs/guardian/v32_audit_boundary.md` |

### 8.3 5 域 Lead ≠ Star 22 DDD bounded context disclaimer (per 8/31 22:45 JST Q1-D 拍板)

- **5 域 Lead (player / economy / match / social / admin)**: RGS 仓 5 域历史治理命名 (per 8/21 JST 拍板)
- **Star 22 DDD bounded context**: STAR 仓 22 DDD context 划分
- **不建立业务子域 ↔ DDD bounded context 映射** (per 8/31 22:45 JST Q1-D 拍板 disclaimer)
- 画布通过 25 module 联动接口跟 5 域对接, **不**直接调用其他 view

### 8.4 Token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)

| 阶段 | Token OLU 估 | SRE·周 (1 SRE·周 = 1.2M tokens) |
|---|---|---|
| 阶段 0 docs (已完成, per §4.31) | ~3.43M | 2.86 |
| 阶段 1 基础 (P3-D.6.1) | ~1.5M | 1.25 |
| 阶段 2 业务 (P3-D.6.2) | ~2.0M | 1.67 |
| 阶段 3 集成 (P3-D.6.3) | ~0.5M | 0.42 |
| 阶段 4 实装 (P3-D.6.4) | ~1.0M | 0.83 |
| **合计 (P3-D.6 启动实装)** | **~5.0M** | **~4.17 SRE·周 (含 docs 阶段)** |

**后续 P3-D.7 跨 session 续做 (5 域 Lead 真人到位 + CRDT 选型 + WSS 选型 + 25 module 跨 session)**: ~3-5M tokens / 2.5-4.2 SRE·周.

**双核心 78 项 全部落地 (12 个月+)**: ~13-19M / 13-19 SRE·周 (per STAR-OLU-001 v0.1).

---

## §9 5 角色签字栏 (per AGENTS.md §3 7 段结构 + IPA SEC 模板 + 守门 #14 v2/v3/v4)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事).

---

## §10 修订历史 (per AGENTS.md §3 7 段结构 + IPA SEC 模板 + 守门 #1 禁回溯叙事)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | **2026-09-10 19:18 JST** | **Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 v0.62 反转)** | **初版落档, 10 段 IPA SEC 模板, 阶段 0-4 共 5 阶段 (docs / 基础 / 业务 / 集成 / 实装), 6 新 crate + 1 BFF + 14+15 张表 + 23+22 API + 5+2 WSS, 13+2 关键 class + 5 状态机 + 11 共享类型, 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT), 19 已知缺口 (含 3 P0 阻塞 + 1 跨 session 总结 #17), 5 角色签字栏, 守门 19 项 + 6 派生规跨域全过, Token OLU ~5M / 4.17 SRE·周 (含 docs 阶段)** | **2026-09-10 19:18 JST Ulysses 拍板"根据详细设计制作 spec 实施计划, 并加入 wbs"** |

---

## 附录 A: 跨专题引用清单 (per 守门 #12 v21 [P] docs 同步)

详见本 doc §1.1 + §4 各表格, 派生自:
- **SRS**: `docs/requirements/SRS-CANVAS-001.md` v1.2 (总册) + `SRS-CANVAS-AGENT-001.md` v1.3 (专题 1) + `SRS-CANVAS-GAMIFY-001.md` v0.1.1 (专题 2)
- **BD**: `docs/design/BD-CANVAS-001.md` v0.1 (总册) + `BD-CANVAS-AGENT-001.md` v0.1 (专题 1) + `BD-CANVAS-GAMIFY-001.md` v0.1 (专题 2)
- **DD**: `docs/design/DD-CANVAS-001.md` v0.1.1 (总册, 14 段) + `DD-CANVAS-AGENT-001.md` v0.1 (专题 1, 15 段) + `DD-CANVAS-GAMIFY-001.md` v0.1 (专题 2, 17 段)
- **平行 DD**: `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB ARG 模板, 13 关键 class + 4 effect + 5 状态机 + 11 共享类型 + 4 时序图 + 8 Cypher + 10 challenges prompt + 74 测试)
- **协调性检查**: `docs/reports/COORDINATION-CHECK-001.md` v0.1.1 (10 项检查 7 通过 + 3 冲突修复)

## 附录 B: 跨域组件映射 (per 守门 #3 5 域 Lead ≠ 22 DDD + 25 module 联动)

详见本 doc §4.5 25 module 联动表 + §5 关键 class / module 落地清单.

## 附录 C: 已知缺口 (per 守门 #11 缺标比错标, 19 个 ≥ 8 满足, 含 3 P0 阻塞 + 1 跨 session 总结)

详见本 doc §7.1 3 P0 阻塞已知缺口表 + DD-AGENT-001 §12.3 19 已知缺口 (含 #11 A12 WSS + #13 localStorage + #15 CRDT + #17 跨 session 总结).

## 附录 D: 5 角色签字栏 (per AGENTS.md §3 7 段结构 + 守门 #14 v2/v3/v4)

详见本 doc §9 5 角色签字栏表.

## 附录 E: 修订履历 (per AGENTS.md §3 7 段结构)

详见本 doc §10 修订历史表.

---

> **撰写完成**: 2026-09-10 19:18 JST, root session mvs_942987595a124037901d37205a548e6f
> **下次拍板触发**: P3-D.6 启动实装 (per 守门 #14 v2 5 域 Lead 真人到位后), 阶段 1 基础 落地 (1 周, 1.25 SRE·周)
> **守门合规**: 19 项主守门 + 6 派生规 (v27/v28/v29/v30/v31/v32) 跨域全过, 0 违反
> **跨域 disclaimer**: 5 域 Lead (player/economy/match/social/admin) ≠ Star 22 DDD bounded context, 不建立业务子域 ↔ DDD 映射
