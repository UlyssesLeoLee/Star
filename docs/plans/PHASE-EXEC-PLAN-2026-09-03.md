# PHASE-EXEC-PLAN-2026-09-03 — 9/3 早晨三件套 commit 后的综合开发计划

> **报告版本**: v0.1 (2026-09-03 06:56 JST, 当 session 起草)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-03 06:53 JST commit `a04c4c1` (Ulysses 真实签) 落档 3 件套 (AUDIT-001 + RF-001 + domain-app-spec), 用户发令"查阅 handoff, 制定开发计划 phase"
> **前置依赖**:
> - 3 份 Handoff: `HANDOFF-ST-001.md` v0.6 (H2/H2-EXT ActorContext 统一) / `HANDOFF-RF-001.md` v0.3 (代码健康度重构) / `HANDOFF-BATCH-001.md` v0.1 (domain-batch v0 phase 2 session 2)
> - `docs/plans/post-merge-development-plan-2026-09-02.md` v0.1 (9/2 智能合并后未完成开发计划, 8 步推进路径)
> - 9/3 commit `a04c4c1` 落档: `docs/refactor/AUDIT-001-requirements-basicdesign-specs.md` v0.1 + `docs/refactor/RF-001-spec.md` v0.3 + `docs/refactor/WBS-001-refactor.md` v0.3 + `docs/refactor/HANDOFF-001.md` v0.3 + `docs/reports/2026-09-02-audit-001-redteam-findings.md` v0.1 + `docs/specs/domain-app-spec.md` v0.1
> - `STAR-P3-WBS-001.md` v0.2 (P3 全 5 阶段 56/64 实质收官 87.5%) + `STAR-OLU-001.md` v0.1 (1 SRE·周 = 1.2M token) + `AGENTS.md` v0.11 (24 commits 守门派生 v1-v24, 含 #19 v22/v23/v24)

---

## 0. 报告目的

落地 9/3 综合开发计划, 整合 3 份 Handoff + 9/2 智能合并 plan + 9/3 早晨新 commit 三件套, 制定一份**当前 main `a04c4c1`** 可推进的 phase 计划:

- **§1 现状实测 + 三件套梳理**: 9/3 commit 8 文件 / 894 行内容; git state 实证; 3 handoff 状态矩阵
- **§2 Phase 分类与推进顺序**: 4 phase 切片 (per token-OLU 降序 + 依赖关系), 总预算 ~4.0M token
- **§3 守门验证**: 6 项守门实证 (per 守门 #1+#9+#12+#15+#19+#20)
- **§4 已知缺口**: 6 项显式缺口 (per 缺标比错标, per 9/2 守门派规)
- **§5 子代理失败接手清单**: 4 项 RPC 不可靠实证 (per 守门 #9 v20)
- **§6 4 Phase 推进路径 + 4 项拍板项**: 4 step ask_user 拍板 (per 14:58 JST 规则)
- **§7 签字栏**: 5 角色 (per AGENTS.md §3 模板)
- **§8 修订历史**

---

## 1. 现状实测 (per 9/3 6:53 JST commit `a04c4c1`)

### 1.1 git 实证

| 维度 | 数据 | 证据 |
|---|---|---|
| 当前 branch | `main` | `git status` |
| HEAD | `a04c4c11c6f713dbb9d7fadb13450af3e464e68d` | `git rev-parse HEAD` |
| author | `Ulysses Leo Lee <hanakagumi@outlook.com>` (**真实签**, 非 Mavis 代签) | `git log -1` |
| ahead origin/main | **34 commits** | `git rev-list --count origin/main..HEAD` |
| 守门 #1 守门 | --lib 0 err (per AGENTS.md v0.11 + 9/2 plan §7.1 实证) | 跨 P3-A 25 守门 + 9 wt merge + 9/3 commit 后续 |
| 工作区 | 0 modified + 0 untracked | `git status --porcelain \| wc -l` |
| worktree 目录 | 11 个 (per 9/2 plan §1.3) | `git worktree list` |

### 1.2 9/3 commit `a04c4c1` 三件套内容 (8 files / 894 行)

| # | 文件 | 行数 | 类型 | 内容 |
|---|---|---|---|---|
| 1 | `Cargo.lock` | +37 | 依赖 | (Ulysses 真实签后依赖 lock) |
| 2 | `PG` | +0 | marker | (空文件, 推测是 PG 进程占位) |
| 3 | `docs/refactor/AUDIT-001-requirements-basicdesign-specs.md` | +105 | 报告 | 后端 3 层文档链一致性核查, 6 大发现 (2 高优 1 中 1 低 2 备查) |
| 4 | `docs/refactor/HANDOFF-001.md` | +101 | Handoff | RF-001 下游 AI 执行清单 (T1 5 + T2 4 + T3 3, 边界声明含 ST-001) |
| 5 | `docs/refactor/RF-001-spec.md` | +166 | Spec | 代码健康度重构 4 档 (T1 机械 / T2 行为保持 / T3 拍板); §2.4 文档同步映射表 v0.3 修正版 |
| 6 | `docs/refactor/WBS-001-refactor.md` | +130 | WBS | 12 任务 token 估算 ~4.15-4.95M, 执行顺序图 |
| 7 | `docs/reports/2026-09-02-audit-001-redteam-findings.md` | +54 | 报告 | 红方挑刺 26 项 (高 13 / 中 8 / 低 5), 待蓝方复核 |
| 8 | `docs/specs/domain-app-spec.md` | +301 | Spec | domain-app 实施 spec (RF-001 T1 后才能实施 T1 任务分解) |

### 1.3 3 份 Handoff + 9/2 plan 状态矩阵 (per 6.1 6.2 拍板项汇总)

| # | 文档 | 版本 | 子项 | 已完成 | 阻塞 | token 估 | 9/3 增量 |
|---|---|---|---|---|---|---|---|
| 1 | `HANDOFF-ST-001.md` | v0.6 (9/1) | H1-H5 (1) + H2-EXT 5 (3) + 8 step 执行计划 | 1/5 + 3/5 + H1 0/1 (待 commit) | H2-2/4/5 (强类型) | ~3.8M | 无新 (9/2 plan 已含) |
| 2 | `HANDOFF-RF-001.md` | v0.3 (9/2) | T1 5 + T2 4 + T3 3 | 0/12 启动 | T1.3 需先判定 | ~4.15-4.95M | **9/3 新 commit `a04c4c1` 落地** |
| 3 | `HANDOFF-BATCH-001.md` | v0.1 (9/1) | T5-T8 (16 step) | 0/16 启动 | GAP-H1-04 shell 沙箱选型 | ~0.2M (估 → 0.6-1.0M) | 无新 |
| 4 | `post-merge-dev-plan` | v0.1 (9/2) | 8 step + 3 拍板项 | 0/8 启动 | 3 项需 Ulysses 拍板 | ~4.0M | 无新 |
| 5 | **AUDIT-001** | v0.1 (9/3) | 6 大发现 | 报告落档 | 蓝方复核 + 拍板 | 0 (仅报告) | **9/3 新** |
| 6 | **红方挑刺** | v0.1 (9/3) | 26 项 (高 13) | 原始输出落档 | 蓝方复核定性 | 0 (仅报告) | **9/3 新** |
| 7 | **domain-app-spec** | v0.1 (9/3) | T1-T9 (9 任务) | spec 起草 | **T 实施阻塞于 RF-001 T1** | 待 T1 完成后估 | **9/3 新** |

---

## 2. Phase 分类与推进顺序 (4 phase 切片, per token-OLU 降序)

> **排序原则** (per `STAR-OLU-001.md` 1 SRE·周 = 1.2M token 换算 + AGENTS.md §4 守门 #4 token-OLU 而非人天):
> - **优先级 1**: 实质工作 (代码 + docs), 排除签字/push 类收尾
> - **优先级 2**: token 预算降序 (大块先做, 避免阻塞)
> - **依赖关系**: Phase 间串行; Phase 内子项可并行 (per 守门 #9 实证子代理 RPC 不可靠, 并行慎用)
> - **拍板项隔离**: Phase 4 全部是拍板 + 收尾, 等 Ulysses

### 2.1 Phase 1 — 阻塞解除 (估 0.4M token, ~0.07 周)

**目标**: 把所有"卡住等拍板 / 卡住等子代理"的项目解锁, 释放后续 Phase 2/3 的可推进路径。

| 子项 | 来源 | token | 状态 | 守门 |
|---|---|---|---|---|
| **1.1** H1 commit 2 个 dirty 文件 (`crates/domain-scm/src/lib.rs` + `crates/domain-workspace/src/lib.rs` 的 `pub uuid::Uuid` 字段) | HANDOFF-ST-001 §1 H1 | 0.01M | 🟡 待 commit | #1+#9+#12 |
| **1.2** RF-001 T1.3 孤儿 crate 判定 (`star-vcs` 注册或删除) | HANDOFF-RF-001 §1 + RF-001-spec §2 T1.3 | 0.1M | 🟡 未开始 (per 9/2 23:46 JST Ulysses 拍板待启) | #1+#9+#12+#19 |
| **1.3** 蓝方复核红方挑刺 26 项 → 26 项定性 ("文档笔误" vs "真实架构矛盾") | 红方挑刺 + AUDIT-001 7 优先项 | 0.2M | 🟡 未开始 | #1+#9+#12 |
| **1.4** 工作区 2 份 stash 评估 (per 9/2 plan §4.5 9 项, `git stash list` 实证) | 9/2 plan §2 + §4.5 | 0.05M | 🟡 待 stash pop 验证 | #9+#12 |
| **1.5** AGENTS.md v0.12 修订 (把 9/3 commit 实证 + 三件套引用落档, 守门派生 v22/v23/v24 已实装) | 守门 #12 实证 | 0.05M | 🟢 立即可做 (无依赖) | #12+#15 |
| **合计** | | **~0.4M** | | |

### 2.2 Phase 2 — 实质重构 (估 2.3M token, ~0.4 周)

**目标**: RF-001 T1 收官 + T2 启动, 把 9/2 plan 中"H2 续做"和"RF-001 重构"两条线合并推进。

| 子项 | 来源 | token | 依赖 | 守门 |
|---|---|---|---|---|
| **2.1** RF-001 T1 全部 5 项 (根目录归档 + 散件清理 + star-vcs 注册 + 死依赖 + lint deny) | HANDOFF-RF-001 §1 + RF-001-spec §2 T1 | 0.75M | Phase 1.2 (star-vcs 判定) | #1+#9+#12+#19 |
| **2.2** H2-EXT #4 domain-identity (DeviceId 强类型 → Uuid 重构) | HANDOFF-ST-001 §5.1 #4 + 9/2 plan §6.1 #5 | 0.2M | 无 (已拍板 hostname 等价 String, 见 9/1 23:59 JST) | #1+#9+#12+#19 |
| **2.3** H2-EXT #5 domain-work-item (context.rs 删除 + port/service dead import, hostname 拍板 0 type 改) | HANDOFF-ST-001 §5.1 #5 + §7 Q1 | 0.05M | 无 (hostname 拍板已就) | #1+#9+#12+#19 |
| **2.4** RF-001 T2.4 大 crate 拆分评估报告 (3 crate, 只出报告不动代码) | HANDOFF-RF-001 §2 + WBS §2 T2.4 | 0.3M | Phase 2.1 (T1 收官后) | #1+#9+#12+#19+#22 |
| **2.5** RF-001 T3 全部 3 项选项报告 (DTO 去重 / Saga 覆盖 / 统一语言) | HANDOFF-RF-001 §3 + WBS §3 T3.1-3.3 | 0.7M | 无 (可与 2.1 并行, 都是文档) | #1+#9+#12+#19+#22 |
| **2.6** H2 原 3 domain service.rs 改造 (feedback/validation/integration ~150+ call sites) | HANDOFF-ST-001 §5.1 #6 + §7 Q3 | 0.3M | Phase 2.2 (H2-EXT #4 完成后) | #1+#9+#12+#19 |
| **合计** | | **~2.3M** | | |

### 2.3 Phase 3 — 应用层 + 基础设施 (估 1.2M token, ~0.2 周)

**目标**: 9/2 plan 中"domain-app 子项目"和"domain-batch v0 phase 2"两条线 + P0-2/3/4 编排。

| 子项 | 来源 | token | 依赖 | 守门 |
|---|---|---|---|---|
| **3.1** domain-app T1 任务分解 (per spec v0.1 §9, 上传 App + 菜单下发 + tenant 安装) | domain-app-spec v0.1 + RF-001 T1 收官 | 0.3M | Phase 2.1 (RF-001 T1 收官) | #1+#9+#12+#19+#22+#24 |
| **3.2** domain-batch v0 phase 2 (T5 NodeExecutor + T6 DagOrchestrator + T7 Scheduler + T8 状态机) | HANDOFF-BATCH-001 §2.1 16 step | 0.2M (估 → 0.6-1.0M per 9/2 实证) | 无 (跟 RF-001 独立) | #1+#9+#12+#19+#22 |
| **3.3** P0-2 ApiError 映射 (api crate ApiError ↔ domain Error) | HANDOFF-ST-001 §5.2 + §8 #5 | 0.3M | Phase 2.6 (H2 原 3 domain 完成) | #1+#9+#12+#19+#22 |
| **3.4** P0-3 application crate 真实编排 (跨域 service 调用) | HANDOFF-ST-001 §5.2 + §8 #6 | 0.4M | Phase 3.3 (P0-2 完成) | #1+#9+#12+#19+#22 |
| **合计** | | **~1.2M** | | |

### 2.4 Phase 4 — 收尾 (估 0.1M token + 拍板 + 等真人)

**目标**: 推 origin + 4 份签字栏 DDD Review 终审 + 5 域 Lead 寻访 (3 项需 Ulysses 真人到位)。

| 子项 | 来源 | token | 状态 | 守门 |
|---|---|---|---|---|
| **4.1** 推 origin (per 守门 #1 反转已落地, 等网络恢复 + Ulysses PAT) | AGENTS.md §4 守门 #1 (R-05 反转) + 9/2 plan §4.2 B-8 | 0.05M | 🟡 等 Ulysses 凭证 | #1+#12+#15 |
| **4.2** 4 份报告签字栏 DDD Review 终审 (Mavis 代签临时, 真人到位后追溯) | 9/2 plan §4.2 B-9 + AGENTS.md §3 模板 | 0.05M | 🟡 Mavis 代签 OK | #1+#12+#15 |
| **4.3** 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束) | 9/2 plan §4.1 #4 + WBS §14.4 B-2 | 0 | 🔴 阻塞 (等 Ulysses) | #3+#12+#15 |
| **4.4** 推 origin 守门 #1 阶段 3 (release test 100% pass) | 9/2 plan §6.1 #8 + HANDOFF-ST-001 §8.1 #8 | 0 | 🔴 阻塞 (推 origin 后) | #1+#12+#15 |
| **合计** | | **~0.1M** | | |

### 2.5 Phase 总计

| Phase | token 估 | 软参考周 | 状态 | 阻塞 |
|---|---|---|---|---|
| Phase 1 — 阻塞解除 | 0.4M | 0.07 周 | 🟡 部分可立即 | 1.2 需 T1.3 判定 |
| Phase 2 — 实质重构 | 2.3M | 0.4 周 | 🟡 依赖 1.x | 1.2 |
| Phase 3 — 应用 + 基建 | 1.2M | 0.2 周 | 🟡 依赖 2.x | 1.2 |
| Phase 4 — 收尾 | 0.1M + 拍板 | — | 🟡 依赖 1-3 + 真人 | 真人 + 凭证 |
| **总计** | **~4.0M** | **~0.67 周** | | **3 拍板** |

**注**: token 估包含文档同步 (per 守门 #12 commit-time docs 同步, 每项 commit 必含 docs)。**不含**真签字栏追溯 (Phase 4.2 待真人到位)。

### 2.6 Phase 5 — 重构最后 (v0.4 拍板, per 2026-09-03 07:19 JST ask_user 2-step)

> **v0.4 拍板触发**: 2026-09-03 07:16 JST Ulysses 发令"最后做重构, 其他按照你的计划做"。ask_user 2-step questionnaire 拍板 A+A: **重构范围 = Phase 2 全部 6 子项, 排到 Phase 4 之后 = Phase 5**。
> 
> **新 plan 顺序**: Phase 1 (1.1+1.3 余项) → Phase 3 (应用+基建) → Phase 4 (收尾) → **Phase 5 (重构最后, 原 Phase 2 整体)**。总预算不变 ~4.0M, Phase 4.3 真人到位 + 凭证 是 Phase 5 启动的前置 (双重依赖)。

| 子项 | 任务 (原 Phase 2) | token | 依赖 | 守门 |
|---|---|---|---|---|
| **5.1** | RF-001 T1 全部 5 项 (根目录归档 T1.1 + 散件清理 T1.2 + 死依赖 T1.4 + lint deny T1.5, T1.3 ✅ done) | 0.75M | Phase 4.3 真人到位 (DDD Review 签字) | #1+#9+#12+#19 |
| **5.2** | H2-EXT #4 domain-identity (DeviceId 强类型 → Uuid 重构) | 0.2M | Phase 4.3 (per 9/1 23:59 拍板 DeviceId 业务语义, 仍需 Ulysses 真人确认) | #1+#9+#12+#19 |
| **5.3** | H2-EXT #5 domain-work-item (context.rs 删除 + port/service dead import, hostname 拍板 0 type 改) | 0.05M | Phase 5.2 (跟 H2-EXT 模式对齐) | #1+#9+#12+#19 |
| **5.4** | RF-001 T2.4 大 crate 拆分评估报告 (3 crate, 只出报告) | 0.3M | Phase 5.1 (T1 收官后) | #1+#9+#12+#19+#22 |
| **5.5** | RF-001 T3 全部 3 项选项报告 (DTO 去重 / Saga 覆盖 / 统一语言) | 0.7M | 无 (可与 5.1 并行) | #1+#9+#12+#19+#22 |
| **5.6** | H2 原 3 domain service.rs 改造 (feedback/validation/integration ~150+ call sites) | 0.3M | Phase 5.2 (H2-EXT #4 完成后) | #1+#9+#12+#19 |
| **小计** | | **~2.3M** | | |

**v0.4 拍板后总 Phase 表** (替换 §2.5):

| Phase | token 估 | 软参考周 | 状态 | 阻塞 |
|---|---|---|---|---|
| Phase 1 — 阻塞解除 | 0.4M (1.1+1.3 余 0.25M) | 0.07 周 | 🟡 余项立即 | — |
| Phase 3 — 应用 + 基建 | 1.2M | 0.2 周 | 🟡 依赖 Phase 1 1.3 蓝方复核 | — |
| Phase 4 — 收尾 | 0.1M + 拍板 | — | 🟡 依赖 Phase 3 + 真人 | 真人 + 凭证 |
| **Phase 5 — 重构最后** (v0.4 新) | **2.3M** | **0.4 周** | 🟡 依赖 Phase 4 全部 + 真人 | **真人 + Phase 4.3 到位** |
| **总计** | **~4.0M** | **~0.67 周** | | **1 拍板 (Phase 4.3 真人) + Phase 5 6 子项内部拍板 (T3 3 项)** |

**Phase 5 启动条件 (per v0.4)**:
1. Phase 4.3 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束, per 19:39 JST Mavis 代签临时, 真人到位后追溯签字)
2. Phase 4.4 推 origin 完成 (per 守门 #1 反转 R-05, 但 github.com 443 不可达 + 无 PAT 实证 GAP-5)
3. Phase 4.2 4 份报告签字栏 DDD Review 终审完成 (Mavis 代签 OK, 真人到位后追溯)
4. 全部 Phase 4 子项收官

**Phase 5 跨 session 续做入口** (per HANDOFF-ST-001 §5.4):
- 新 session 第一步: 读本 plan v0.4 (本版) + HANDOFF-RF-001.md v0.3 + HANDOFF-ST-001.md v0.6 → git pull → git log --oneline -10 → cargo check --workspace --all-targets 重测 → 续 Phase 5.1 T1.1 根目录归档
- 估 session 1-3 完成 Phase 5 全部 6 子项 (1-1.5M / session per HANDOFF-ST-001 §8.2 buffer)

---

## 3. 守门验证 (per AGENTS.md v0.11 §4 守门 13+ 派生 v1-v24)

### 3.1 6 项关键守门本任务实证

| 守门 | 规则 | 本次任务实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 (4 层级 + workspace + release) | 本计划无代码改动, 仅 git/docs 规划, 不触发 cargo | ✅ (守门保持) |
| #9 | 不 commit 散落子代理产出 + git 实证 | 本计划是 plan 文档, 0 commit (落档需 1 commit 由 Ulysses 签) | ✅ |
| #12 | commit-time docs 同步 | 计划落档 `docs/plans/PHASE-EXEC-PLAN-2026-09-03.md` 同时修订 HANDOFF-001 / HANDOFF-ST-001 / AGENTS.md (守门 #15 饱和约束: 113 ahead 落地 6 commits, 当前 34 ahead 留 buffer) | ✅ |
| #15 | docs 同步饱和约束 | 当前 34 ahead origin/main, 离 113 饱和点 buffer 79 commits, 新事件触发新 docs 同步 (per 守门 #15 实证 v15) | ✅ |
| #19 | agent 交互 Python 化 (3 维 R/V/S/A ≥ 2 必走 `scripts/automation/`) | 本计划无 agent 交互, 仅 plan 文档; 若 Phase 1.2 (T1.3 star-vcs 判定) 需 agent 交互, 必走 `scripts/automation/refactor_template.py` (per WBS §0) | ✅ (备查) |
| #20 | 子代理 dispatch 必先 `automation/dispatcher.py brief(...)` → `docs/briefs/` | 本 plan 无子代理 dispatch; 若 Phase 2.4 / 2.5 需派子代理, 必先 brief | ✅ (备查) |

### 3.2 守门派生 v15-v24 增量提醒 (per AGENTS.md v0.11)

| 派生 | 触发 | 应用 |
|---|---|---|
| v15 | docs 同步饱和 (113 ahead 落地 6 commits) | 当前 34 ahead, 离饱和 buffer 79 commits, **新事件触发才推新 commit**, 不主动 docs 同步 |
| v16 | P0-1 联动审计触发 (22 domain ActorContext 17 份重复) | Phase 1.5 AGENTS.md 修订需补 P0-1 实证字段 |
| v17-v18 | H2 / H2-EXT 范围扩量 (3 → 8 domain, 0.3-0.5M 估 → 1.1-1.6M 实测 3-5x 超支) | Phase 2.2/2.3/2.6 token 估已含实测 (per HANDOFF-ST-001 §5.1) |
| v19 | 守门 #19 Python 化 (主上下文 ≥ 5K token [S] 升 [M], ≥ 10K 升 [P]) | Phase 2.1 RF-001 T1 跨 stage 5 项合计估 ~10K token 触发 [P], 必走 `scripts/automation/refactor_template.py` |
| v20 | 守门 #9 子代理 dispatch 必先 brief | Phase 2.4/2.5 派子代理前必先 `automation/dispatcher.py brief` |
| v21 | 守门 #12 [P] 子项 docs 同步 (automation-design.md §4 + registry.md) | Phase 1.5 修订时同步落档 |
| v22-v24 | 守门 #1 v20 / #5 v2 / #9 v3 (调试控制台 + AI 修改 mock + subprocess 替代 RPC) | Phase 1.5 / Phase 3.1 (domain-app 调试页) 需引此派生规 |

### 3.3 5 维质量门 (per STAR-OLU-001 §6)

- **功能完整**: 本 plan 含 4 Phase / 19 子项 / ~4.0M token, 覆盖 3 份 handoff + 9/2 plan + 9/3 三件套 ✅
- **测试覆盖**: 不适用 (本任务无代码改动, 仅规划) — 推进门槛 4/5 ≥ 4 ✅
- **守门 0 违反**: 6 项守门实证 (§3.1), 0 违反 ✅
- **文档同步**: 本文档 (EXEC PLAN) + 修订历史 v0.1 + §4 已知缺口 + §5 子代理清单 + 6 项守门 ✅
- **git 证据**: 当前 HEAD `a04c4c1` 34 ahead origin/main + 9/3 commit 8 files/894 行 + worktree 11 个 实证可查 ✅

**总分**: **4/5** (测试覆盖 不适用, 0 行代码改动) → 推进门槛 4/5 ≥ 4 ✅

---

## 4. 已知缺口 (per 缺标比错标, 显式列, per 守门 #1 v15 实证 + 9/2 plan §7.3)

| # | 缺口 | 影响 | 触发 | 备注 |
|---|---|---|---|---|
| GAP-1 | 蓝方复核红方挑刺 26 项 (高 13 / 中 8 / 低 5) 未定性 | Phase 1.3 启动阻塞 | 2026-09-03 | 9/3 commit `a04c4c1` 落档 26 项原始输出, 蓝方复核未做 |
| GAP-2 | AUDIT-001 发现 1/2/6-A/6-B (basic-design §2.1 内部矛盾 + 6 边方向 + 25 表逐行核查) 未拍板 | Phase 1.3 启动阻塞 | 2026-09-03 | 9/3 commit `a04c4c1` 落档 7 大发现, 建议处理顺序 §100-104 |
| GAP-3 | domain-work-item `device_id: Option<String>` 业务语义 (9/1 23:59 JST 拍板 = hostname, 0 type 改) 跨 session 续验证 | Phase 2.3 启动 | 2026-09-03 | hostname 拍板已就, 但 entity 改后 type 不变仅 0.05M |
| GAP-4 | 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束) | Phase 4.3 阻塞 | 2026-09-03 | Mavis 代签临时, 真人到位后追溯 |
| GAP-5 | 推 origin (github.com 443 不可达 + 无 PAT/GITHUB_TOKEN) | Phase 4.1 阻塞 | 2026-09-03 | per 9/2 plan §4.2 B-8 实证, 9/1 23:59 推失败 |
| GAP-6 | 11 个 worktree 目录未清理 (per 9/2 plan §1.3) | 跨 session 续 | 2026-09-03 | 建议保留到 DDD Review 阶段, 9/3 plan 不动 |

---

## 5. 子代理失败接手清单 (per 守门 #9 v20 派生规 + 守门 #9 #3 实证)

> **实证 (per AGENTS.md §4 #9)**: 10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded。子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证 worktree commit 在 main 链上。

### 5.1 子代理 RPC 不可靠场景

| 场景 | 实证 | 接手策略 |
|---|---|---|
| worker 子代理 status=succeeded 但实际失败 (5/5 实证 per 守门 #9 #3) | 5/5 subagent RPC 不可靠 | 不派 worker 子代理, 走 subprocess.run 替代 (per 守门 #9 v3 #24) |
| brief 落地后子代理收不到任务 | 守门 #9 #3 实证 #4 | `automation/dispatcher.py brief` 必先落档 `docs/briefs/<task_id>.md`, 子代理仅读 brief 不读对话 |
| 子代理产出于 worktree 但未 merge | 守门 #9 #2 实证 | `git log -p --follow <wt-branch>` 实证 worktree commit 在 main 链上, 否则 revert + commit 重做 |

### 5.2 Phase 1-3 子代理风险点 (per WBS §0 自动化档标注)

| Phase | 子项 | 风险 | 缓解 |
|---|---|---|---|
| 1.2 | T1.3 star-vcs 判定 | 判定"保留/删除"需 git log --follow 实证历史 | 走 `scripts/automation/refactor_template.py` (per WBS §0 [P] 标注) |
| 1.3 | 蓝方复核 | 26 项定性需 read 14 份文档链 | 不派子代理, 本人 read (per 守门 #9 v3) |
| 2.1 | RF-001 T1.1 根目录归档 (98 md) | `git mv` 跨 98 文件需同步更新内部链接 | 走 `scripts/automation/refactor_template.py` (per WBS [P] 标注) |
| 2.2 | H2-EXT #4 (DeviceId→Uuid) | 跨域类型重构需精确改动 | 走 `scripts/automation/refactor_template.py` + 自审后 commit |
| 2.4 | T2.4 大 crate 拆分评估 (3 crate) | 评估报告需 read 各 crate `lib.rs` mod 声明 | 不派子代理, 本人 read |
| 2.5 | T3.1 DTO 去重 + T3.2 Saga 覆盖 + T3.3 统一语言 | 报告类需 read 跨 crate 调用点 | 不派子代理, 本人 read + write |
| 3.2 | domain-batch v0 phase 2 session 2 | 16 step 跨域调用, 估 0.2M → 0.6-1.0M | 拆 2 sub-session (per HANDOFF-BATCH-001 §4 R-H1-1) |

---

## 6. 4 项 Ulysses 拍板项 (per 2026-09-01 14:58 JST 拍板决策必须用选项) — **v0.2 拍板落档**

> **拍板决策必须用 ask_user** 规则: 任何需要 Ulysses 拍板的事必须用 ask_user 给选项, 不能直接做/只问"可以吗"。本 plan 含 4 项需拍板 (per 9/2 plan §6.2 3 项 + 9/3 增量 1 项)。

### 6.1 4 项拍板项 (per token-OLU 优先级)

| 序 | 拍板项 | 关联 Phase | 选项 | 落地 token |
|---|---|---|---|---|
| 1 | **RF-001 T1.3 `star-vcs` 孤儿 crate 判定** | Phase 1.2 | A. 注册 / B. 删除 / C. 暂不处理 | 0.1M |
| 2 | **9/2 智能合并后 stash 暂存处理** | Phase 1.4 | A. 逐项 cherry-pick 评估 / B. 全部丢弃 / C. 留 stash 跨 session 续 | 0.05M |
| 3 | **9/2 归档分支 P1 cherry-pick** | Phase 1.4 (并行) | A. 现在做 P1 (4 子项, arch-graph + onboarding 基础) / B. 推迟到 Phase 2 完成后 / C. 不做 | 0.3M |
| 4 | **domain-app-spec T 任务分解启动** | Phase 3.1 | A. RF-001 T1 收官后启动 / B. 推迟到 Phase 2 全部完成 / C. 不启动 | 0.3M |

### 6.2 拍板结果 (per 2026-09-03 07:04 JST ask_user 4-step questionnaire, 4 项全 A)

| 序 | 拍板项 | Ulysses 拍板 | 落地动作 |
|---|---|---|---|
| 1 | T1.3 star-vcs 判定 | **A. 注册** (推荐) | `Cargo.toml` workspace.members 加 `crates/star-vcs` + 同 commit 新建 `docs/specs/domain-vcs-spec.md` 骨架; 若 crate 本身无独立 `Cargo.toml` 或 src/ 空, 改走 B (删除) |
| 2 | stash 暂存处理 | **A. 逐项 cherry-pick 评估** (推荐) | `git stash show stash@{0}` + `git stash show stash@{1}` 看内容, 逐项评估 (per 9/2 plan §4.5 9 项清单) |
| 3 | 归档分支 P1 cherry-pick | **A. 现在做** (推荐) | `git show archive/auto-20260902-17ef4658:...` 看 P1 4 子项内容, 跟 main HEAD 实际 diff, 走 `git checkout` 单文件复制 (避免 cherry-pick 引发 rebase 冲突) |
| 4 | domain-app 启动 | **A. RF-001 T1 收官后启动** (推荐) | Phase 2.1 (RF-001 T1, 0.75M) 完成后, Phase 3.1 (domain-app T1, 0.3M) 启动 |

### 6.3 拍板后落地优先级 (per 拍板结果 + token-OLU 降序 + 依赖关系) — **v0.4 拍板后修订**

> **v0.6 拍板落档**: 2026-09-03 07:49 JST ask_user 4-step 拍板 A+A+A+A 全部 7 项必拍板项, 详见 §6.4。

| 优先级 | 任务 | token | 依赖 | 守门 | 备注 |
|---|---|---|---|---|---|
| ✅ 1 | T1.3 star-vcs 注册 | 0.1M | 无 | #1+#9+#12+#19 | Phase 1.2 done (commit `b7ec06e+93cf36b`) |
| ✅ 2 | stash drop 2 份 | 0.05M | 无 | #9+#12 | Phase 1.4 done (main HEAD 已 supersede) |
| ✅ 3 | P1 cherry-pick 4 docs | 0.3M | 无 (可与 1/2 并行) | #1+#9+#12+#19 | Phase 1.4 done (commit `982e399`) |
| ✅ 4 | AGENTS.md v0.36 + plan v0.3 docs 同步 | 0.05M | 无 | #12+#15 | Phase 1.5 done (commit `cb8c76f`) |
| ✅ 5 | H1 commit 2 dirty files | 0.01M | 无 | #1+#9+#12 | **Phase 1.1 done (commit `dd27983` 8/31, plan §6.3 优先级 5 已过时, 跳过不重做)** |
| **6 → ✅** | 蓝方复核红方挑刺 26 项 → 26 项定性 (文档笔误 vs 真实架构矛盾) | 0.2M | 无 | #1+#9+#12 | **Phase 1.3 done (本 commit, 26 项定性: 9 笔误 + 7 批量修复 + 7 必拍板 + 3 缺口, 14KB 报告落档)** |
| 7 | Phase 3 — 应用 + 基建 (domain-app T1 + domain-batch v0 phase 2 + P0-2/3/4) | 1.2M | 优先级 5+6 | #1+#9+#12+#19+#22 | Phase 3, 跟 5/6 串行 |
| 8 | Phase 4 — 收尾 (推 origin + 签字栏 DDD Review + 真人到位) | 0.1M | 优先级 7 | #1+#12+#15 | Phase 4, 真人 + 凭证 |
| **9** | **Phase 5 — 重构最后** (原 Phase 2 全部 6 子项) | **2.3M** | **优先级 8 (Phase 4.3 真人到位)** | #1+#9+#12+#19+#22 | **Phase 5, 跨 1-3 session 续 (v0.4 拍板)** |
| **小计** | | **~4.0M** | | | **本 session + 跨 session 总目标** |

**session 预算分配**: 当前 session (v0.4 拍板后) 立即可做优先级 5 (H1 commit 0.01M) + 优先级 6 (蓝方复核 0.2M) + Phase 3 部分 (估 0.5-0.8M), 留 0.5-0.8M buffer per HANDOFF-ST-001 §8.2 (1-1.5M / session 目标).

**v0.5 修订实证** (per 2026-09-03 07:34 JST Ulysses 发令"立刻做" + 蓝方报告落档):
- H1 commit (优先级 5) 实测: 实际 8/31 commit `dd27983` H1: domain-scm + domain-workspace define_uuid_id! 字段 pub 化 (per HANDOFF-ST-001); git status 0 dirty, 不重做 ✅
- 蓝方复核 (优先级 6) ✅: 26 项定性报告 `docs/reports/2026-09-03-blue-team-26-items-review.md` v0.1 (14KB) 落档, 7 项 Ulysses 必拍板项 + 16 项蓝方代修清单 + 5 已知缺口
- 蓝方代修 16 项 (估 0.1M token, 1 commit) 推下 session 续 (per 守门 #15 实证 6 ahead origin/main, 留 buffer)
- 7 项拍板项必拍 Ulysses (per 14:58 JST 拍板规则), 排到 Phase 4 真人到位 + Phase 3 应用层完成之后

### 6.4 7 项 Ulysses 必拍板项结果 (v0.6 拍板落档, per 2026-09-03 07:49 JST ask_user 4-step)

| 拍板 # | Finding | Ulysses 选择 | 落地动作 | token | commit |
|---|---|---|---|---|---|
| **拍 1** (F9 25-Module 表过期) | basic-design.md §2.1 25-Module 表实际 34 crate 缺 9 | **A. 重写 §2.1 表为 34 crate** | 加 §2.1.4 (9 crosscut supporting) + §2.1.5 (10 star-* infra), 主体 §2.1.1-§2.1.3 不动 (25 logical 已在) | 0.05M | 本 commit `0a8b4f4` 起 |
| **拍 2** (§4.9 集群 2 项: Workflow + Board/Planning) | work-item ↔ workflow 循环 + board ↔ planning 循环 | **A. 单向只读投影** — work-item → workflow 只读 + board/planning 各自独立 | 修 basic-design §2.1 row 1/9/10/11 字段 + 3 spec 附录 B 显式 "无核心依赖" | 0.1M | 跨 session 续 #2 |
| **拍 3** (§2.3 硬禁线 2 项: Worktree/WorkItem + SCM) | §2.1 边 vs §2.3 禁线冲突 | **A. 删 §2.1 边 + 保留禁线** | 修 basic-design §2.1 row 2 (worktree 删 work-item) + row 7 (scm 删 worktree), §2.3 禁线原文不动 | 0.1M | 跨 session 续 #3 |
| **拍 4** (spec 权威 2 项: Validation + Agent) | §2.1 表 vs spec 字段不一致 | **A. spec 权威 + 16 项代修同意** | 修 basic-design §2.1 row 3 (agent 改 work-item/permission) + row 6 (validation 改 work-item/worktree); 16 项蓝方代修 14 docs 1 commit 批量修 | 0.3M | 跨 session 续 #4 + #5 |

**v0.6 拍板后总推进顺序** (按 token-OLU 降序 + 依赖关系):

| 序 | 任务 | token | 依赖 | 守门 |
|---|---|---|---|---|
| **A** | 拍 1 加 §2.1.4 + §2.1.5 (basic-design §2.1 修订) | 0.05M | 无 | #1+#9+#12+#19 |
| **B** | 拍 2 §4.9 集群修 (1 commit, 4 row basic-design + 3 spec 附录 B) | 0.1M | A | #1+#9+#12+#19 |
| **C** | 拍 3 §2.1 边删 (1 commit, 2 row basic-design) | 0.1M | A | #1+#9+#12+#19 |
| **D** | 拍 4 spec 权威 (1 commit, 2 row basic-design) | 0.1M | A | #1+#9+#12+#19 |
| **E** | 16 项蓝方代修批量修 (1 commit, 14 docs) | 0.1M | A | #1+#9+#12+#19 |
| **F** | AGENTS.md v0.39 docs 同步 (1 commit) | 0.05M | A-E | #12+#15 |
| **小计** | | **~0.5M** | | |

**session 预算分配**: 本 session (v0.6 拍板后) 估 0.5M token 实装 5 步骤 (A-F), 留 1.0-1.5M buffer per HANDOFF-ST-001 §8.2 (1-1.5M / session 目标). 总 A-E 6 commits 落档 (per 守门 #12 commit-time docs 同步), ahead origin/main 从 7 → 13, 离 113 饱和点 buffer 100 充足 (per 守门 #15 实证).

### 6.4 跨 session 续做建议 (per HANDOFF-ST-001 §5.4)

- **session 启动 step 1-5** (per HANDOFF-ST-001 §5.4): 读本 plan v0.2 (本版) + HANDOFF-ST-001 + AGENTS.md v0.12 → git pull → git log --oneline -10 → cargo check --workspace --all-targets 重测 (per Q9-T A9 数字时效性) → 续 Phase 1.5 AGENTS.md 修订
- **session token 预算**: 1-1.5M / session (留 25-50% buffer per HANDOFF-ST-001 §8.2)
- **拍板后续**: Phase 2.1 (RF-001 T1 收官) 估 0.75M, 1-2 session 内收官; Phase 3.1 (domain-app T1) 估 0.3M, 紧接 Phase 2.1 之后

---

## 7. 签字栏 (per AGENTS.md §3 模板, 5 角色)

| # | 角色 | 审批者 | 日期 | 状态 |
|---|---|---|---|---|
| 1 | 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-03 | 🟢 Mavis 接手代签 |
| 2 | SRE Lead | (跨 session 续 / 等真人到位) | — | ⏳ 待签 (per 19:39 JST 授权, Mavis 接手可代签临时) |
| 3 | 平台 | (跨 session 续 / 等真人到位) | — | ⏳ 待签 (同上) |
| 4 | 评审主持 | (跨 session 续 / 等真人到位) | — | ⏳ 待签 (同上) |
| 5 | PM | (跨 session 续 / 等真人到位) | — | ⏳ 待签 (同上) |

**注**: 5 角色中架构师 Mavis 接手代签临时, SRE Lead/平台/评审/PM 4 角色等真人到位 (per 8/21 JST 拒绝兼任硬约束)。5 域 Lead 真人 (per 8/21 JST) 跨 session 续, 不在本签字栏。

---

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 06:56 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: 整合 3 份 Handoff (ST-001/RF-001/BATCH-001) + 9/2 智能合并 plan + 9/3 commit `a04c4c1` 三件套 (AUDIT-001 + RF-001-spec/WBS/HANDOFF-001 + domain-app-spec + 红方挑刺); 4 Phase 分类 (阻塞解除 0.4M / 实质重构 2.3M / 应用+基建 1.2M / 收尾 0.1M) 总 ~4.0M token ~0.67 周; 6 项守门 + 6 项缺口 + 5 项子代理风险 + 4 项 Ulysses 拍板项 | 2026-09-03 06:53 JST commit `a04c4c1` 落档 8 files / 894 行 + 用户发令"查阅 handoff, 制定开发计划 phase" |
| v0.2 | 2026-09-03 07:04 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 4 项 Ulysses 拍板结果落档 (T1.3=A 注册 / stash=A 逐项评估 / P1=A 现在做 / domain-app=A RF-001 T1 后); §6 改写"4 项拍板结果"表 + §6.3 拍板后落地优先级 (本 session 目标 ~1.15M: T1.3 0.1M + stash 0.05M + P1 0.3M + AGENTS v0.12 0.05M + RF-001 T1 收官 0.65M); 修订历史 +1 行 | 2026-09-03 07:04 JST ask_user 4-step questionnaire 4 项全 A 拍板 |
| v0.3 | 2026-09-03 07:30 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 4 项拍板落地实证 (本 session 完成 3 项 + 1 项 跨 session 续): **T1.3 star-vcs 注册** (commit `b7ec06e` 5 files + `93cf36b` Cargo.lock followup, cargo check 0 err 21.40s + cargo fmt 0 + cargo clippy 0 warning 1.89s, 守门 #1+#12+#19 实证); **P1 cherry-pick 4 docs** (commit `982e399` arch-graph-viewer ADR+spec + audit-onboarding ADR+spec, 4 files 1792 insertions, 守门 #1+#12+#19 实证); **stash drop 2 份** (stash@{0} Cargo.lock + stash@{1} 12 files, 全部 main HEAD 已 supersede via 6d71f39/b811800/6af1482/6bce434/8c893a9/93cf36b, 0 commit, 守门 #9 实证); **domain-app T1 启动**: 跨 session 续 (per §6.2 #4 拍板 A RF-001 T1 后); §8 修订历史 +1 行 | 2026-09-03 07:30 JST 本 session 3 项落地 + 守门 #1+#9+#12+#15+#19 跨 stage 全过 |
| v0.4 | 2026-09-03 07:19 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | **重构最后做, 其他按原 plan** 拍板落档: 2026-09-03 07:16 JST Ulysses 发令 + ask_user 2-step questionnaire 拍板 A+A (范围=Phase 2 全部 / 排到=Phase 4 之后); §2.6 新增 "Phase 5 — 重构最后" (2.3M token 6 子项, 等 Phase 4 全部 + 真人到位); §6.3 拍板后落地优先级 v0.4 修订: 本 session 立即可做 优先级 5 (H1 commit 0.01M) + 6 (蓝方复核 0.2M) + Phase 3 部分 (0.5-0.8M); 新 plan 顺序: Phase 1 余项 (1.1+1.3) → Phase 3 → Phase 4 → Phase 5 (原 Phase 2 整体), 总预算不变 ~4.0M | 2026-09-03 07:19 JST ask_user 2-step questionnaire 拍板 A+A "重构最后做, 其他按原 plan" |
| v0.5 | 2026-09-03 07:35 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | **H1 已落地 (跳过) + 蓝方复核 26 项 ✅ done**: 优先级 5 H1 实测 8/31 commit `dd27983` 已落地 (plan v0.4 §6.3 优先级 5 过时, 跳过不重做); 优先级 6 蓝方复核落档 `docs/reports/2026-09-03-blue-team-26-items-review.md` v0.1 (14KB) — 26 项定性 9 文档笔误 + 7 可批量修复 + 7 真实架构矛盾需拍板 + 3 已知缺口, 7 项 Ulysses 必拍板项按风险升序 (拍 1 = F9 25-Module 表过期 / 拍 2-7 = 6 依赖方向 §2.3 硬禁线冲突), 16 项蓝方代修清单 (估 0.1M 1 commit) 推下 session; §6.3 优先级 5+6 标 ✅ done; §8 修订历史 +1 行 | 2026-09-03 07:34 JST Ulysses 发令"立刻做" + H1 git log --oneline 实证 + 蓝方报告落档 |
| v0.6 | 2026-09-03 07:49 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | **7 项 Ulysses 必拍板项结果 A+A+A+A 拍板落档**: 拍 1 (F9) = A 重写 §2.1 表为 34 crate; 拍 2 (Workflow + Board/Planning) = A 单向只读投影; 拍 3 (Worktree/WorkItem + SCM) = A 删 §2.1 边 + 保留禁线; 拍 4 (Validation + Agent) = A spec 权威 + 16 项代修同意; §6.4 4 项拍板结果表 + 5 步骤实施顺序 A-F (拍 1 0.05M + 拍 2 0.1M + 拍 3 0.1M + 拍 4 spec 权威 0.1M + 16 项代修 0.1M + AGENTS v0.39 0.05M) 总 ~0.5M, 6 commits 落档; §8 修订历史 +1 行 | 2026-09-03 07:49 JST ask_user 4-step 拍板 4 项全 A 落档 |
