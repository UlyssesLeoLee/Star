## §9 P4 阶段 WBS 整合 (per 2026-09-04 07:40 JST, 守门 #12 commit-time 同步)

> **承接**: `STAR-P4-UNIMPL-WBS-001.md` v0.1 (本 session 落档 26995 bytes) + 9/3 12:39 JST B 拍板加快并行 + 9/3 11:35 JST A+A+A+B 拍板 4 阻塞项
> **目的**: 把 9/4 未实施设计 9 大类 ~60 项清单 → 8 Phase × 4 轨道 WBS, 跟本 HANDOFF §5/§6/§8 跨 session 续做项 + 5 域 Lead + 凭证阻塞 + 3 套新架构待实装 全部整合, 避免下游 AI 误把 P3 全 5 阶段收官后的剩余任务当独立工作项, 跟 §8 Ulysses "所有" 拍板 4-5M token 计划 衔接
> **双轴 WBS**: token 预算 (per `STAR-OLU-001.md` 1 SRE·周 = 1.2M) + 质量门 5 维 (per `STAR-OLU-001.md` §6)

### 9.1 P4 vs P3 WBS 关系

| 阶段 | 子项 | 状态 | 文档 |
|---|---|---|---|
| P3 全 5 阶段 (A 25 + B 9 + C 9 + D 7 + E 7 + F 6) | 64 | 56/64 实质收官 87.5% (per `STAR-P3-WBS-001.md` §6) | `STAR-P3-WBS-001.md` v0.2 |
| **P4 阶段 (新, 本 session 落档)** | **42** | **0/42 待启动 (本 session 立即启动 Phase A.1)** | `STAR-P4-UNIMPL-WBS-001.md` v0.1 |
| **合计** | **106** | **56/106 实质收官 52.8%** | |

### 9.2 P4 4 轨道并行 (per 9/3 12:39 JST B 拍板 + cargo 互锁规避)

```
轨道 1 阻塞解铃 (Phase A 5 子项, 0.1M)
  ├─ A.1 推 origin retry (9/3 12:43 JST 401 跨 session 续) - 本 session 立即
  ├─ A.2 .worktrees 残留 3 项 永久删 (Ulysses 手动, Mavis 不越权)
  ├─ A.3 5 域 Lead 真人寻访 (Ulysses 个人网络 / freelance / 开源 3 选 1)
  ├─ A.4 凭证收集 (B.5/B.6/E.4/D.2-D.6, mock 备选可长期维持)
  └─ A.5 4 报告签字栏 DDD Review 终审 (Mavis 接手代签 5 角色)

轨道 2 6 续做项硬阻塞 (Phase B + C + D, 1.85-3.65M 估 4-5 sub-session)
  ├─ Phase B: T1.7 76 err 修法 4.1+4.2+4.3+4.4 (4 子项, 0.55-1.05M)
  │  · B.1 已实证 51→10 err (commit 65a8da0)
  │  · B.2 实证 50+ err 跨 handlers/+tools/ (per AGENTS v0.56:457)
  │  · B.3 守门 #1 v3 派生规 文字补全
  │  · B.4 守门 #1 v3 派生规 实证 --all-targets 716 err baseline
  ├─ Phase C: T3.3 + T3.1 + T1.5 (3 子项, 0.9M)
  │  · C.1 ubiquitous-language.md v1.0 (v0.1 已落 commit 524a75a)
  │  · C.2 共享 star-dto 重构
  │  · C.3 unreachable_pub = "deny" 3 步切换
  └─ Phase D: T3.2 Saga + 5.6 H2 + G-10 (3 子项, 0.4-1.7M)
     · D.1 G-10 H2 类型不兼容 (DeviceId 强类型 + String→Uuid 业务语义)
     · D.2 T3.2 Saga ≥80% 覆盖 (等 match 域 Lead 真人)
     · D.3 5.6 H2 原 3 domain 改造 (feedback/validation/integration ~150+ call sites)

轨道 3 P3 续做 + G 缺口 (Phase E + F + G, 46M 估 3-5x 超支)
  ├─ Phase E: P3-C/E/F 跨域编排 (5 子项, 13M)
  │  · E.1 E.6 5 域 Saga 实装 (per Q-003 / 跨域补偿 / 失败回滚)
  │  · E.2 E.7 5 域 DDD 边界验证 (44.6KB docs 已落档 per e67bc8c)
  │  · E.3 F.1 DDD Review 阶段 5 角色真人到位
  │  · E.4 CONTENT-REVIEW-PACK 21 份 docs 评审
  │  · E.5 REGISTRY 5 行追溯签字 (覆盖 Mavis 临时代签)
  ├─ Phase F: 凭证切真 + DB + CI runner (5 子项, 21M)
  │  · F.1 B.5 OpenClaw 真实集成 e2e (凭证切真, mock 已落地 per 29692a7)
  │  · F.2 B.6 Hermes 真实集成 e2e (mock 已落地)
  │  · F.3 E.4 KMS 集成 (LocalMockKms 已实装 per 5ea9611)
  │  · F.4 守门 #DB-13 DB 三類横展開 (W/T/M 100% 表覆盖, CW-01~CW-10 派生守门)
  │  · F.5 D.2/D.6 CI runner 真实配置 (GitHub Actions runner)
  └─ Phase G: Agent Runtime G-1~G-9 缺口 (9 子项, 12M)
     · G.1 L0 SQLite 任务队列 (1M 派发持久化)
     · G.2 L1 bevy_ecs / flecs 选型 + 9 SA Archetype
     · G.3 EventBus + Mailbox 实现 (Agent 间通信协议)
     · G.4 Shared LLM/HTTP/MCP Pool (守门 #24 subprocess 池扩展 ECS 池)
     · G.5 Tenant Quota + 多租户隔离 (22 domain-identity 联)
     · G.6 Memory Store (外置)
     · G.7 Crash Recovery + Checkpoint
     · G.8 Context Tiering (L1/L2/L3)
     · G.9 Token 计量 telemetry

轨道 4 3 套新架构实装 + DDD 终审 (Phase H, 7.5M 末段)
  ├─ H.1 LangGraph PostgreSQL checkpointer 实装 (v0.1 文档已落 per AGENTS §7 #8)
  ├─ H.2 LangGraph 跨仓 (Physis/RGS) RPC 实装 (v0.3 计划)
  ├─ H.3 LangGraph 16 tool sub-agent 経由 call 化 (补 12 tool 留 P2 缺 service)
  ├─ H.4 LangGraph State schema v1 migration 路径 (v0.2 计划)
  ├─ H.5 Tree-sitter Rust crate 引入 + 4-6 语言 grammar
  ├─ H.6 Tree-sitter 任务卡 ↔ worktree 1:1 绑定 + react-flow graph 渲染
  ├─ H.7 Tree-sitter symbol resolver 跨文件引用追踪
  └─ H.8 DDD Review 21 份 docs 终审 + 签字栏追溯
```

### 9.3 HANDOFF §5/§6/§8 续做项 → P4 Phase 映射表

> **原则**: 续做项不重写, 仅映射, 避免双重记录。

| HANDOFF 老章节 | 内容 | 映射 P4 Phase | 状态 |
|---|---|---|---|
| §5.1 #1-#5 H2-EXT 5 domain | comment/tenant/project 已落 + identity/work-item 跨 session 续 | **Phase D.1 G-10** | 🟡 #4 #5 续 |
| §5.1 #6 H2 原 3 domain service.rs 改造 | feedback 77 err + validation/integration | **Phase D.3 5.6** | 🟡 跨 session 续 |
| §5.2 P0-2/3/4 | ApiError 映射 + application + infrastructure adapter | **(待 P4 Phase 新增, 不在 v0.1)** | 🟡 跨 session 续 |
| §5.3 5 项 Blocker | 类型不兼容 + star_context 字段扩展 + service.rs + 5 域 Lead + P3-B 凭证 | **Phase D.1 + E + F** | 🟡 4/5 续 |
| §6 §6.1 守门 #1 阶段 1 | --lib 0 err (已收官) | (P3-A 25/25) | 🟢 |
| §6 §6.1 守门 #1 阶段 2 | --all-targets 716 err baseline | **Phase B.4 实证** | 🟡 |
| §6 5/6 done + 5.6 推下 | Phase 5 5 闭环 + 5.6 H2 推下跨 session 续 | **Phase D.3** | 🟡 |
| §6 11 旧 worktree cleanup | 11 个 git worktree remove (0 commit, gitignored) | (已 done, per AGENTS v0.46) | 🟢 |
| §8.1 #1-#12 跨 session 续 12 项 | H2-EXT #5 简化 + #4 + H2 原 3 domain + 守门 #1 阶段 2 + P0-2/3/4 + 守门 #1 阶段 3 + docs 优化 + cargo doc + 5 域 Lead + P3-B 拍板 | **Phase A + B + C + D + E + F** (P0-2/3/4 待 P4 新增) | 🟡 8/12 续 |
| §8.5 风险点 5 项 | session token + 跨域 type + 守门 #9 + 5 域 Lead + P3-B 凭证 | **Phase A + D + E + F** | 🟡 4/5 续 |

### 9.4 累计 token 估 + 5x 超支风险 (per 9/3 B 拍板 + 守门 #1 实证)

| 轨道 | 估 token (理论) | 实际 3x 超支 | 实际 5x 超支 |
|---|---|---|---|
| 轨道 1 阻塞解铃 (Phase A) | 0.1M | 0.3M | 0.5M |
| 轨道 2 6 续做项 (Phase B+C+D) | 1.85-3.65M | 5.55-10.95M | 9.25-18.25M |
| 轨道 3 P3+G (Phase E+F+G) | 46M | 138M | 230M |
| 轨道 4 3 套新架构 (Phase H) | 7.5M | 22.5M | 37.5M |
| **P4 合计** | **~55M 理论** | **~165M 3x** | **~275M 5x** |

**对比 P3**: P3 = ~179.5M / 64 子项 实质收官 56/64 = 87.5%; P4 = ~55-275M / 42 子项 实质预估 0/42 ≈ 0%。

### 9.5 5 项 Blocker 状态更新 (v0.7) + 5 项新增 (v0.7)

| # | Blocker | HANDOFF v0.6 状态 | HANDOFF v0.7 状态 (P4 映射) |
|---|---|---|---|
| 1 | H2-EXT #5 String 业务语义 = hostname | ✅ 拍板 | ✅ (无变化) |
| 2 | H2-EXT #4 DeviceId → Uuid 重构 | ⏳ 跨 session 续 | ⏳ → Phase D.1 G-10 |
| 3 | H2 原 3 domain service.rs 改造 | ⏳ 跨 session 续 | ⏳ → Phase D.3 5.6 |
| 4 | **5 域 Lead 真人到位** (per 8/21 拒绝兼任) | ⏳ 等 Ulysses | ⏳ → Phase A.3 + E.1-E.5 |
| 5 | **P3-B 凭证** B.5 OpenClaw / B.6 Hermes | ⏳ 等 Ulysses | ⏳ → Phase A.4 + F.1-F.2 (mock 备选可长期维持) |
| 6 | **新增: 守门 #1 v3 --all-targets baseline 716 err** | (未列) | 🟡 → Phase B.4 实证 |
| 7 | **新增: G-10 H2 类型不兼容 (DeviceId + String→Uuid)** | (未列) | 🟡 → Phase D.1 |
| 8 | **新增: 3 套新架构实装 pending (LangGraph + Agent Runtime + Tree-sitter)** | (未列) | 🟡 → Phase H.1-H.7 |
| 9 | **新增: 推 origin 9/3 12:43 JST 401 跨 session 续** | (未列) | 🟡 → Phase A.1 |
| 10 | **新增: .worktrees 残留 3 项 (PowerShell 永久删 Ulysses 手动)** | (未列) | 🟡 → Phase A.2 |

### 9.6 拍板请求 (per 9/1 14:58 JST "决策必须用选项")

> 本 session 已落 `STAR-P4-UNIMPL-WBS-001.md` v0.1 §16 拍板请求 4 项; HANDOFF 同步 4 项 + 5x 超支风险警告。

| # | 拍板项 | 选项 A | 选项 B | 推荐 |
|---|---|---|---|---|
| 1 | Phase A.1 推 origin retry 时机 | 本 session 续 retry (守门 #1 1a max 2 retries) | 下 session 第一件事 retry | **A** (本 session 立即消化, 不积压) |
| 2 | Phase A.3 5 域 Lead 寻访方法 | Ulysses 个人网络 (5 工程师各认领 1 域) | freelance 平台 (Toptal/Upwork) | **A** (更快 + 跟项目熟悉) |
| 3 | Phase A.4 凭证切真时机 | 立即切真 (需 Ulysses 提供 B.5/B.6/E.4 凭证) | 维持 mock 长期跑 (per 29692a7) | **B** (mock 路径已落地, 不阻塞) |
| 4 | 整体推进策略 | 串行 8 Phase (风险低, 慢) | 4 轨道并行 (per 9/3 B 拍板, 快, 风险 cargo 互锁 + 3-5x 超支) | **B** (per 9/3 12:39 JST 拍板 B 已生效) |

### 9.7 session 入口 (per 守门 #1 v3 + Q9-T A9 数字时效性)

下次 session 第一件事 (新事件触发后):

```bash
# 1. 读本 HANDOFF v0.7 §9 (本节) + AGENTS.md 最新版 + STAR-P4-UNIMPL-WBS-001.md v0.1
# 2. git fetch origin (验证 ahead) + git log --oneline -10
# 3. cargo check --workspace --all-targets -j 4 重测 baseline (per 9/3 12:52 JST `-j 4` 修正 + Q9-T A9 数字时效性, 必须实测, 不得沿用 716 err 或 290 err)
# 4. 读守门 #3 派生规 v2 (Mavis 临时代签 5 域 Lead, per 9/3 11:35 JST 反转)
# 5. 启动 Phase A.1 推 origin retry (本 session 立即或下 session 第一件事, per §9.6 #1 拍板)
# 6. 启动 Phase A.2 .worktrees 清理脚本生成 (Mavis 不越权, Ulysses 手动 PowerShell 删)
# 7. 启动 Phase B.1+ B.2 T1.7 修法 (4.1+4.2 并行 per 9/3 12:39 JST B 拍板)
```

### 9.8 引用文档

- `STAR-P4-UNIMPL-WBS-001.md` v0.1 (本 session 落档 26995 bytes / 42 子项 / 8 Phase / 4 轨道)
- `STAR-P3-WBS-001.md` v0.2 (P3 全 5 阶段 60/65 拍板落地 / 56/64 实质收官 87.5%)
- `STAR-OLU-001.md` v0.1 (1 SRE·周 = 1.2M token-OLU 独立基线)
- `AGENTS.md` v0.69 (per §6.1 架构 view 索引 + §7 待办表 + §4 守门 + §4.1 派生规 v1-v24)
- `2026-09-03-rf-001-blockers-4items-board.md` v0.1 (4 阻塞项 A+A+A+B 拍板)
- `2026-09-03-rf-001-final-4items-board.md` v0.1 (4 类 B+B+B+B 加快并行拍板)
- `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (Agent Runtime G-1~G-12 已知缺口)
- `docs/architecture/2026-09-03-langgraph/` 3 份 v0.1 (Phase H.1-H.4 文档基础)
- `docs/architecture/2026-09-03-treesitter-worktree-graph/` 2 份 v0.1 (Phase H.5-H.7 文档基础)
- `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` v0.1 (Phase E.4 21 份 docs review 操作手册)
- `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` v0.1 (Phase E.5 5 行待填)
- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 (Phase F.4 W/T/M 三類索引基线)
