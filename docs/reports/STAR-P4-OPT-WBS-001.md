# STAR-P4-OPT-WBS-001 P4+ 阶段 优化专用 WBS (per 9/7 11:53 JST 用户发令"开子代理全面排查未实现的需求, 制定优化专用 wbs")

> **Status**: 🟡 Draft v0.1 (优化专用, 4 子代理 4 维度扫描结果汇总, 待 Ulysses 拍板顺序 / scope / token 分配)
> **Created**: 2026-09-07 12:00 JST
> **Authority**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 (per 8/27 19:39 JST + 21:59 JST 三次强化 + 9/5 10:43 JST 5 域 Lead 反转)
> **触发**: 2026-09-07 11:53 JST 用户发令"开子代理全面排查未实现的需求, 制定优化专用 wbs"
> **关联基线**:
> - `docs/reports/STAR-P4-UNIMPL-WBS-001.md` v0.1 (9/4 19:30 JST, P4 未实施 WBS 草稿, 本 WBS 增量不冲突)
> - `docs/reports/STAR-P3-WBS-001.md` v0.6 (P3 96 子项 85.4% 收官)
> - `docs/reports/HANDOFF-ST-001.md` v1.4 (H1-H4 + H2-EXT 8 domain)
> - `AGENTS.md` §7 待办 8 项 (1 项部分完成 + 1 项实质完成 + 6 项落地/阻塞)
> - 4 份 OPT-A1/A2/A3/A4 子代理扫描报告 (9/7 11:55-12:00 JST 落地)
> **排序原则 (per 2026-09-04 13:43 JST 用户发令)**: 不按日期,按**粗略预估消耗量**降序;推进门槛是**质量门禁 ≥4/5**,不是截止日期
> **守门硬约束**: 14 主体 + 26 派生规 + 7 实装前一致性门 = 47 守门点 (per AGENTS.md §4 + §4.1 + §4.2)
> **禁回溯叙事** (per守门 #12): BAS / BAS-001 / DTL-036 等历史文档引用必 `git log -p --follow` 实证

---

## §0 一句话硬约束

> **代码 80 stub (0 P0 / 22 P1 / 45 P2 / 13 P3) + WBS 23 Phase pending + ADR 23 草案 + 质量 18 缺口 = 144 优化项, 跨 5 优化域 (代码 / WBS / ADR / 质量 / 治理) , 估 12-25M tokens;Mavis 接手推进 1 P0 (7 ahead dirty 实测) + 5 P1 (H2-EXT 5 domain / P0 tool / P1 tool / G-TMO-04 DDL / Runtime 4 crate) = 6 项本 session 立即可启动 (估 1.5-2.5M), 其余 138 项等 5 域 Lead 真人到位 (T3 ~ 9/26) + 凭证切真 + DDD Review 拍板**
>
> 排序 per 9/4 13:43 JST 拍板 (预估消耗量降序) + 9/5 10:43 JST 拍板 (5 域 Lead 内推) + 9/3 19:35 JST 拍板 D (Mavis 临时代签) + 守门 #11 缺标比错标

---

## §1 4 维度扫描汇总 (per OPT-A1/A2/A3/A4 9/7 落地)

### 1.1 4 份 brief + 4 份 output (per 守门 #9 v20 + #19 v19 实证)

| 维度 | 子代理 | brief 路径 | output 路径 | 关键数字 |
|---|---|---|---|---|
| **代码 TODO/stub** | explore (mvs_36c75e) | `docs/briefs/OPT-A1-code-todo-scan.md` | `docs/briefs/OPT-A1-code-todo-scan.output.md` (22.3KB) | ~80 条 = 0 P0 + 22 P1 + 45 P2 + 13 P3 |
| **WBS/Phase pending** | explore (mvs_c62f5e) | `docs/briefs/OPT-A2-wbs-pending-scan.md` | `docs/briefs/OPT-A2-wbs-pending-scan.output.md` (43.2KB) | 23 Phase pending + 19 守门缺口 + 14 P3 阻塞 + 42 P4 待启动 |
| **ADR/IPA 未实装** | explore (mvs_e0d7bb) | `docs/briefs/OPT-A3-adr-pending-scan.md` | `docs/briefs/OPT-A3-adr-pending-scan.output.md` (14.7KB) | 28 ADR (23 草案) + 100/100 W/T/M + 4 crate 命名待拍 + 5 域 0 到位 |
| **质量 / 技术债** | explore (mvs_47946e) | `docs/briefs/OPT-A4-quality-debt-scan.md` | `docs/briefs/OPT-A4-quality-debt-scan.output.md` (30.7KB) | 0 err / 86 warning / 234-600 missing_docs + 33 wt 散落 + 18 缺口 |

### 1.2 4 维度交叉 (去重聚合)

| 优化域 | 优化项 | 来源 | token 估 (粗) |
|---|---|---|---|
| **代码** (OPT-A1) | 80 stub 函数 / 47 stub 函数 + 25 MOCK + 6 空 impl + 8 重模式 | OPT-A1 §3-§6 | 5-10M |
| **WBS** (OPT-A2) | 23 Phase pending + 19 守门缺口 + 14 P3 阻塞 | OPT-A2 §1-§3 | 3-6M |
| **ADR** (OPT-A3) | 23 ADR 草案升 v0.2 + 4 crate 命名拍板 + 4 混合表分类 + 3 IPA v0.1 升 v0.2 | OPT-A3 §1-§6 | 1-2M |
| **质量** (OPT-A4) | 18 缺口 + 33 worktree 散落 + 600+ missing_docs + 12 crate 未实装设计意图 | OPT-A4 §10 | 2-5M |
| **治理** (跨 4 维度) | 5 域 Lead 真人到位 + 凭证切真 + AGENTS.md §7 表头 main HEAD 同步 + 4 报告签字栏 | OPT-A2 §6 + OPT-A4 §10 | 1-2M |
| **合计** | **~144 项, 估 12-25M tokens** | | **~144 项, 12-25M** |

---

## §2 5 优化域 × 8 Phase 矩阵 (per 守门 #12 派生 + §13 + §14 4 维)

### 2.1 5 优化域 vs 8 Phase 立项矩阵

| Phase \ 优化域 | 代码 (OPT-A1) | WBS (OPT-A2) | ADR (OPT-A3) | 质量 (OPT-A4) | 治理 (跨) | Phase 估 |
|---|---|---|---|---|---|---|
| **Phase A 阻塞解铃** | 0 | 5 (A.1-A.5) | 0 | 1 (#4 5 域 Lead) | 5 (Ulysses + 凭证 + 真人) | 0.3M |
| **Phase B T1.7 修法** | 0 | 4 (B.1-B.4) | 0 | 0 | 0 | 0.6M |
| **Phase C T3.3/T3.1/T1.5** | 0 | 3 (C.1-C.3) | 0 | 0 | 0 | 0.9M |
| **Phase D T3.2/5.6/G-10** | 5 (H2-EXT 8 domain) | 3 (D.1-D.3) | 0 | 1 (#3 H2-EXT 续) | 1 (5 域 Lead 到位) | 0.5-1.7M |
| **Phase E P3-C/E/F 编排** | 0 | 5 (E.1-E.5) | 21 (CONTENT-REVIEW-PACK) | 0 | 5 (5 域 + 4 角色到位) | 4.5M |
| **Phase F 凭证 + DB + CI** | 0 | 5 (F.1-F.5) | 0 | 0 | 4 (凭证 + GA 权限) | 18M (F.1-F.3 凭证) |
| **Phase G G-1~G-9 缺口** | 4 (P0 tool + P1 tool) | 9 (G.1-G.9) | 0 | 1 (#8 G-DEP-01) | 1 (ECS 选型) | 12M |
| **Phase H 3 套新架构 + 终审** | 4 (3 handler + LangGraph RPC) | 8 (H.1-H.8) | 0 | 0 | 1 (真人到位) | 7.5M |
| **横向 (跨 Phase)** | 67 (P2/P3 stub 优化) | 0 | 7 (ADR 升 v0.2 + 4 crate 命名 + 4 混合表) | 15 (#2-#6, #11-#17, #18) | 0 | 5M |
| **合计 (估)** | **80** | **42** | **28** | **18** | **17** | **~50M** |

### 2.2 3x token 超支风险 (per守门 #1 v18 H2 实证 0.3-0.5M → 1.1-1.6M)

- H2 实证: 0.3-0.5M 估 → 1.1-1.6M 实测 (3-5x 超支)
- 整体估 50M, 3x 超支风险 = 150M, 5x 超支 = 250M
- per守门 #1 v18 派生规: 单 sub-session 估 0.6-0.8M 上限, 跨 sub-session 续
- per守门 #1 v19: `cargo check --workspace --all-targets -j 4` 0 err 32.27s 通过
- per守门 #4 1 SRE·周 ≈ 1.2M (per `STAR-OLU-001.md` v0.1), 50M ≈ 42 SRE·周

---

## §3 优化项立项 (按 R/V/S/A 维度 + 预估消耗量降序, per 守门 #1 派生)

### 3.1 立项维度说明 (per 守门 #19 v19 自动化档 + #12 任务卡表)

- **R (Rerunnable)**: 可重跑 (子代理 + Python 化脚本 + e2e 实证)
- **V (Volume)**: 体量级 (估 token 区间)
- **S (Structural)**: 结构性改动 (跨 crate / 跨 view / 跨域)
- **A (Audit-trail)**: 审计追踪 (commit / 守门 0 违反 / git 证据)

### 3.2 立项矩阵 (10 大类 144 项)

| 类 | 立项编号 | 内容 | token 估 | R/V/S/A | 守门 | 触发 | 状态 |
|---|---|---|---|---|---|---|---|
| **#1 代码 stub** | OPT-CODE-01..47 | `star-api-rest` 22 路由 + 3 中间件 + `domain-batch` 22 stub + `domain-report` 14 chart + 4 port stub | 1.5-3.0M | R+V+S+A | #1+#11+#12 | per OPT-A1 §3 | 🟡 Phase D/H 续 |
| **#2 代码 stub** | OPT-CODE-48..67 | `application/api/infrastructure` 3 supporting crate 占位结构 (12 占位) | 0.3-0.5M | R+V+S | #1+#11+#4.2 | per OPT-A1 §3.5 | 🟡 Phase 2 删 |
| **#3 代码 stub** | OPT-CODE-68 | `star-vcs/src/cache.rs` 完整空壳 (R-007 cache 0 实装, 4 TODO 块) | 0.5-1.5M | V+S+A | #1+#11+#4.2 | per OPT-A1 §3.2 | 🟡 Phase D/H 续 |
| **#4 代码 stub** | OPT-CODE-69..73 | 4 `star-sa` provider stub (GitHub/GitLab/Bitbucket/Gitea) + 1 `domain-local-runtime` http_client + 1 `star-cache` in_memory_backend | 0.3-0.5M | R+V | #1+#11 | per OPT-A1 §3.5 | 🟡 Phase H 续 |
| **#5 WBS 守门** | OPT-WBS-01..19 | 19 守门缺口 (含 §4.2 实装前一致性门 7 条 partial) | 0.5-1.0M | R+A | #1+#4+#12 | per OPT-A2 §2 | 🟢 已落地 9 + 🔴 阻塞 5 + 🟡 5 |
| **#6 WBS Phase** | OPT-WBS-20..42 | 23 Phase pending (A.1-A.5 + B.1-B.4 + C.1-C.3 + D.1-D.3 + E.1-E.5 + F.1-F.5) | 8-15M | R+V+S+A | #1+#3+#9 | per OPT-A2 §1 | 🟡 4/23 本 session 启动 |
| **#7 ADR 升版** | OPT-ADR-01..23 | 23 草案 ADR 升 v0.2 (含 0021-0032 早期 + 0037-0040 + 0042 + 0047) | 0.3-0.5M | R+A | #1+#4+#12 | per OPT-A3 §1 | 🟡 守门 #12 拍板启动 |
| **#8 ADR 拍板** | OPT-ADR-24..27 | 4 crate 命名拍板 (`domain-task` / `domain-llm` / `domain-mcp` / `domain-tool`) | 0.1-0.2M | S+A | #1+#4+#9 v20 | per OPT-A3 §4 | 🔴 等 DDD Review 拍板 |
| **#9 ADR 分类** | OPT-ADR-28..31 | 4 混合表分类拍板 (`workspace.workspace` M/T / `planning.roadmap` M/T / `audit.audit_event_outbox` T/W / `local_runtime.runtime_observation` T/W) | 0.05-0.1M | S+A | #1+#13+#9 v20 | per OPT-A3 §3.4 | 🔴 等 DDD Review 拍板 |
| **#10 质量债** | OPT-Q-01..18 | 18 已知缺口 (P0×2 + P1×6 + P2×9 + P3×1) | 3-5M | R+V+S+A | #1+#7+#26+#6 v2 | per OPT-A4 §10 | 🟡 9/18 P1/P2 推下 session |
| **#11 治理** | OPT-GOV-01..05 | 5 域 Lead 真人寻访 + 凭证切真 (B.5/B.6/E.4) + GA 权限 + 4 报告签字栏 | 0.3-0.5M | V+S+A | #3+#14+#1 1a | per OPT-A2 §6 + §5.3 | 🔴 阻塞 4/5, 等 Ulysses + 真人到位 |
| **#12 wiki 漂移** | OPT-Q-19..25 | 7 docswiki 主题 vs 212 pgwiki 节点 + 12 crate 未实装设计意图 + 4 pgwiki 50-issues 待办 | 1-2M | S+A | #1+#12 | per OPT-A4 §8 | 🟡 推下 session |
| **#13 worktree 清理** | OPT-Q-26..58 | 33 散落 (26 wt + 6 log + 1 archive), 含 P4-UNIMPL A.2 永久删 3 项 | 0.05-0.1M | R+A | #1+#9 v3 | per OPT-A4 §7 | 🟡 Ulysses 手动 (per P4-UNIMPL A.2) |
| **#14 e2e 补** | OPT-Q-59..65 | 7 e2e_integration 实证缺口 + 1 test_tmo_bulk_dag ImportError + star-cache flake | 0.5-1.0M | R+V+A | #1+#26+#4 | per OPT-A4 §3-§4 | 🟡 推下 session |
| **#15 frontend 清理** | OPT-Q-66..75 | 4 tsc err (FeatureToggles onCheckedChange + refactor-state-machine 缺 + tailwind-merge 缺) + 6 vitest 实证 + 1 next build | 0.3-0.5M | R+V | #1+#6 v2 | per OPT-A4 §4 | 🟡 advisory 派生后不阻断 |
| **#16 docs 同步** | OPT-Q-76..80 | AGENTS.md §7 表头 main HEAD 同步 + 4 PHASE 报告签字栏追溯 + 5 域 Lead 5 份 brief commit | 0.05-0.1M | R+A | #1+#12+#10 | per OPT-A2 §7.1 | 🟢 守门 #12 饱和约束保持 |
| **#17 IPA 升 v0.2** | OPT-ADR-32..34 | 3 份 v0.1 IPA 文档升 v0.2 (LangGraph 04 + Agent Runtime 02/03) | 0.1-0.2M | R+S | #1+#12+#4 | per OPT-A3 §2 | 🟡 守门 #12 拍板启动 |
| **#18 守门 #13 派生** | OPT-Q-81..86 | 4/10 派生守门待 P3-B SRE Lead 拍板 (CW-04/CW-06/CW-07~10) | 0.05-0.1M | S+A | #1+#13+#4 | per OPT-A3 §3.2 | 🔴 阻塞 4/10, 等 SRE Lead 到位 |
| **合计** | **144 项** | (跨 18 大类) | **~50M (估)** | | | | |

---

## §4 立即可启动 6 项 (本 session 后续 / 推下 session 1 周内, 估 1.5-2.5M)

### 4.1 立即可启动 6 项明细

| # | 立项编号 | 内容 | token 估 | 守门 | 启动条件 | 落地路径 |
|---|---|---|---|---|---|---|
| 1 | **OPT-Q-01** | 7 ahead main HEAD dirty 实测 `cargo check --workspace --all-targets -j 4` | 0.05M | #1 v19 | 0 (本 session 立即) | parent 跑 cargo check + git log 实证 |
| 2 | **OPT-CODE-69..73** | 4 `star-sa` provider stub + 1 `domain-local-runtime` http_client + 1 `star-cache` in_memory_backend | 0.3-0.5M | #1+#9 v20 | 0 (凭证可 mock) | worker 子代理 dispatch (per `scripts/automation/`) |
| 3 | **OPT-CODE-68** | `star-vcs/src/cache.rs` 完整空壳 → 4 TODO 块实装 (cache-trait + impl-inmem + provider-integration + tests) | 0.5-1.0M | #1+#4.2+#11 | Phase D 启动 (per 9/4 P4-WBS §5) | worker 子代理 dispatch (per `STAR-P4-UNIMPL-WBS-001.md` §5) |
| 4 | **OPT-WBS-06..08** | Phase B T1.7 修法 4.1+4.2+4.3+4.4 (4 子项) | 0.5-1.0M | #1 v19+#19 v19 | 0 (pure cargo) | worker 子代理 dispatch (per `STAR-P4-UNIMPL-WBS-001.md` §3) |
| 5 | **OPT-WBS-09..11** | Phase C T3.3 + T3.1 + T1.5 3 子项 | 0.3-0.5M | #1+#12 | B.1 避免污染 | worker 子代理 dispatch (per `STAR-P4-UNIMPL-WBS-001.md` §4) |
| 6 | **OPT-Q-26..58** | 33 worktree 散落清理 (含 P4-UNIMPL A.2 永久删 3 项) | 0.05-0.1M | #1+#9 v3 | Ulysses 手动 (Mavis 不越权) | ask_user 拍板后 batch script `git worktree remove --force <path>` |

### 4.2 立即可启动 6 项跟 AGENTS §7 待办 对齐

| AGENTS §7 # | 内容 | 跟立即 6 项关系 |
|---|---|---|
| 1 | 25 domain-* crate 真实数据接入 | OPT-CODE-01..47 (Phase D/H 续) |
| 4 | Prompts 实际模板 / Resources 独立资源类型 | (不立即, 拍板后) |
| 5 | 9 个 wt 是否 merge | OPT-Q-26..58 跟 1 wt TBD 评估 |
| 6 | 4 份报告签字栏 DDD Review 终审 | (不立即, 真人到位) |
| 8 | Star LangGraph 統合 + TMO 7 节点 | (不立即, P0-1 + H2-EXT 完) |
| 8.1 | TMO 7 子项实装 phase | (不立即, P0-1 + H2-EXT 完) |

### 4.3 推下 session 8 项 (1-2 周内, 等 5 域 Lead 真人到位启动, 估 8-15M)

| # | 立项编号 | 内容 | 触发 | token 估 |
|---|---|---|---|---|
| 1 | **OPT-WBS-12..14** | Phase D T3.2/5.6/G-10 3 子项 (H2-EXT 5 domain 跨域字段扩展) | 5 域 Lead T3 (9/26) 到位 | 0.3-1.6M (per 守门 #1 v18 实证 3-5x 超支) |
| 2 | **OPT-WBS-15..18** | Phase E 5 子项 (E.6 5 域 Saga + E.7 DDD 验证 + F.1 5 角色 + CONTENT-REVIEW-PACK 21 docs + REGISTRY 5 行追溯) | 5 域 Lead + SRE/平台/评审/PM 到位 | 4.5M |
| 3 | **OPT-WBS-19..23** | Phase F 5 子项 (B.5 OpenClaw + B.6 Hermes + E.4 KMS + DB W/T/M + CI runner) | 凭证到位 + GA 权限 | 18M |
| 4 | **OPT-WBS-24..32** | Phase G G-1~G-9 9 子项 (L0 + ECS + EventBus + Shared Pool + Tenant + Memory + Recovery + Context + Telemetry) | 独立 (可跟 B 并行) + ECS 选型 | 12M |
| 5 | **OPT-WBS-33..40** | Phase H 8 子项 (LangGraph PG checkpointer + 跨仓 RPC + 16 tool sub-agent 経由 + State schema + Tree-sitter + react-flow + symbol resolver + DDD Review 终审) | Phase E.3 真人 + Phase G ECS 选型 + 16 tool 真实接入 | 7.5M |
| 6 | **OPT-CODE-01..47** | star-api-rest 25 + domain-batch 22 + domain-report 18 stub 实施 | Phase D/H 续 | 1.5-3.0M |
| 7 | **OPT-Q-08..10** | G-DEP-01/02/04/05 (P0/P1 tool 实装 + task_metadata DDL + LangGraph SDK 0.2.x 确认) | (P3-F #5 实证) | 0.7-1.1M |
| 8 | **OPT-ADR-01..27** | 23 草案 ADR 升 v0.2 + 4 crate 命名拍板 + 4 混合表分类 | 拍板启动 (per守门 #20) | 0.5-0.9M |

---

## §5 守门 0 违反实证 (per STAR-OLU-001 §6 质量门 5 维)

### 5.1 守门硬约束 14 + 26 + 7 = 47 点 (per AGENTS §4 / §4.1 / §4.2)

| 类别 | 数量 | 守门点 |
|---|---|---|
| **§4 主体** | 14 | #1 推 origin + #3 5 域 Lead + #4 token-OLU + #5 环境变量 + #6 PowerShell + #7 0 unsafe + #8 不沿用 bc23d6c + #9 不 commit 散落 + #10 代签 + #11 缺标 + #12 AI 文档 + #13 W/T/M + #14 5 域 Lead CONTENT 4 维 + (P3-A 阶段 #2/#3 落地) |
| **§4.1 派生** | 26 | v1-v26 (含 v25 PR #12 + 9/5 拍板) |
| **§4.2 实装前一致性门** | 7 | 现行 package 清单 / Runtime 名称映射 / lint 不绕 / 基线不误报 / handoff 复核 / 功能闭环 / 唯一入口 |

### 5.2 5 维质量门 (per STAR-OLU-001 §6)

| 维度 | 本 WBS 状态 | 实证 |
|---|---|---|
| **功能完整** | 🟡 144 项 / 6 立即启动 / 8 推下 session | per §3 + §4 |
| **测试覆盖** | 🟢 41/41 crate 100% 守门覆盖 (per守门 #1 v12 实证) + 1384 tests | per OPT-A4 §5 |
| **守门 0 违反** | 🟢 PR #12 9/9 pass (per 9/5 实证) + 4 enforced 守门 | per OPT-A4 §4 |
| **文档同步** | 🟡 守门 #12 饱和边界保持 (per `5cfb7b3` 9/6 拍板) | per AGENTS §4.1 v15 |
| **git 证据** | 🟢 4 子代理扫描 commit 引用 7 字符 hash + file:line 实证 | per OPT-A1-A4 §0-§10 |

### 5.3 守门基线 (4 维度扫描结果守门 0 违反 实证)

| 守门 | 状态 | 实证来源 |
|---|---|---|
| **#1 v19** | 🟢 | `cargo check --workspace --all-targets -j 4` 0 err 32.27s (per 9/3 RF-001 T1.5 实证, commit `0e6a965`) |
| **#6 v2** | 🟢 | Frontend advisory 派生 (per PR #12 `ca40edb`) |
| **#7 v3** | 🟢 | Clippy `-D warnings` 改 advisory (per PR #12 `9d10565`) |
| **#9 v20** | 🟢 | 子代理 dispatch 必先 brief (per `scripts/automation/dispatcher.py` 实证) |
| **#10** | 🟢 | 代签允许 (per 8/27 19:39 JST + 21:59 JST 三次强化) |
| **#11** | 🟢 | 缺标比错标安全 (4 份子代理报告每份 §7/§10 已知缺口显式列) |
| **#12** | 🟢 | AI 文档治理 (无回溯叙事, BAS 引用 git 实证, 子代理授权"无证据叙事=禁止") |
| **#13** | 🟢 | DB W/T/M 100/100 覆盖 (per `00-CLASSIFICATION-W-T-M.md` v0.1) |
| **#15 (派生)** | 🟢 | 守门 #12 死循环饱和边界 (per `5cfb7b3` 9/6 拍板) |
| **#19 (派生)** | 🟢 | agent 交互 Python 化 (per `docs/automation-design.md` v0.1) |
| **#22 (派生)** | 🟢 | 调试控制台不污染 main (per `console_server.py` 实证, commit `2bdbbdd`) |
| **#23 (派生)** | 🟢 | 调试页 AI mock (per `ai_edit_mock.py` 实证) |
| **#24 (派生)** | 🟢 | 调试控制台 subprocess 替代 RPC (per Next.js + FastAPI 8080 实证) |
| **#25 (派生)** | 🟢 | 5 域 Lead 真人 Ulysses 内推 brief (per `5-business-domain-lead-referral.md` v0.1, commit 待生成) |
| **#26 (派生)** | 🟢 | CI 4 守门修订反转 (per PR #12 `9d10565` + `76baafb` + `0c447c5` + `ca40edb`) |

---

## §6 5 域 Lead 依赖 (per守门 #3 + #14 v2 5 域 Lead CONTENT 4 维)

### 6.1 5 域 Lead CONTENT 4 维 (per 9/3 19:43 JST 拍板)

| 域 | 决策 scope | RACI | 到位 timeline | 当前 (per 9/7 12:00 JST) |
|---|---|---|---|---|
| **player** | 玩家账号/角色/存档/登录 | R+A+C+I 全 | T3 (2026-09-19~26) | ⏳ T0 启动 (9/5), 0 真人到位 |
| **economy** | 经济/库存/订单/Saga (Q-003) | 同上 | T3 | 同上 |
| **match** | 匹配/对战/战斗/战报 | 同上 | T3 | 同上 |
| **social** | 聊天/好友/公会/排行榜/通知 | 同上 | T3 | 同上 |
| **admin** | COC/审计/合规/RBAC | 同上 | T3 | 同上 |
| **架构师 (Mavis 接手)** | ADR/SPEC/审批/拍板 | 全 RACI | 已到位 (代签) | ✅ 全部代签 |
| **SRE / 平台 / 评审 / PM** | (per 守门 #14 v2 派生) | 全 RACI | ⏳ 真人未到位 | ⏳ 全部代签 |

### 6.2 真人到位触发清单 (per 6.1 5 域 + 4 角色)

| 项 | 触发 | 当前阻塞子项 |
|---|---|---|
| **T3 ≥1 域到位** | ADR-0047 装装 E-1 启动 + G-DEP-08 PostgreSQL checkpointer Tier 3 启动 | Phase D + E + F + G + H 全部 |
| **T4 满员 (5 域)** | Phase E.5 REGISTRY 5 行追溯签字 + 4 报告签字栏覆盖 | Phase E.1-E.5 |
| **T5 全部角色到位** | 5 角色 (架构+SRE+平台+评审+PM) 真人到位 | DDD Review 阶段 |
| **T5 覆盖 (5 角色到位)** | 修订历史表 +1 行追溯签字 (per §1.2 T5) | 全部 Mavis 临时代签决策 |

### 6.3 token-OLU 估 (per STAR-OLU-001 v0.1, 1 SRE·周 ≈ 1.2M)

| 项 | token 估 | SRE·周 |
|---|---|---|
| 立即可启动 6 项 | 1.5-2.5M | 1.25-2.08 SRE·周 |
| 推下 session 8 项 | 8-15M | 6.67-12.5 SRE·周 |
| 5 域 Lead 真人到位后续 30 项 | 20-30M | 16.67-25 SRE·周 |
| **P4 累计 (含本 WBS)** | **~50M (估)** | **~42 SRE·周 (估)** |

---

## §7 跟 P4-UNIMPL-WBS-001 (9/4 草稿) 对齐

### 7.1 增量不冲突

| 维度 | P4-UNIMPL-WBS-001 (9/4) | 本 WBS STAR-P4-OPT-WBS-001 (9/7) | 关系 |
|---|---|---|---|
| **立项数** | 42 子项 (8 Phase × 4 轨道) | 144 项 (5 优化域 × 18 立项类) | 增量, 不覆盖 |
| **token 估** | ~47M | ~50M | 增量 ~3M (4 子代理扫描 + WBS 立项) |
| **维度** | 8 Phase 4 轨道 | 5 优化域 18 立项类 | 维度不同, 互补 |
| **触发** | 9/4 07:14 JST 60+ 未实施清单 | 9/7 11:53 JST 用户发令"开子代理全面排查" | 续做, 升级 |
| **5 域依赖** | 5 域 Lead 真人 T0-T5 timeline | 5 域 Lead 真人 T0-T5 timeline (per `5-business-domain-lead-referral.md` v0.1) | 一致 |
| **基线** | 守门 #1+#3+#4+#5+#6+#7+#8+#9+#10+#11+#12+#13+#14+#15 (派生) | 守门 #1 v19+#3+#4+#5+#6+#7+#9+#10+#11+#12+#13+#14+#15+#19+#20+#22+#23+#24+#25+#26 (派生) | 升级 (per PR #12 9/5 拍板) |

### 7.2 关键差异 (本 WBS 增量项)

| 项 | P4-UNIMPL-WBS-001 | 本 WBS 增量 |
|---|---|---|
| **代码 stub 集中区** | 0 (per 9/4 草稿未扫) | 4 (star-api-rest 25 + domain-batch 22 + domain-report 18 + star-vcs 完整空壳) per OPT-A1 |
| **IPA 文档 v0.1 草稿** | 0 (per 9/4 草稿未扫) | 3 份 (LangGraph 04 + Agent Runtime 02/03) per OPT-A3 §2 |
| **混合表分类 4 表** | 0 (per 9/4 草稿未扫) | 4 表 (workspace.workspace M/T / planning.roadmap M/T / audit.audit_event_outbox T/W / local_runtime.runtime_observation T/W) per OPT-A3 §3.4 |
| **Runtime 概念→物理 crate 映射** | 0 (per 9/4 草稿未扫) | 4 crate 待 ADR 拍板 (domain-task / domain-llm / domain-mcp / domain-tool) per OPT-A3 §4 |
| **worktree 33 散落** | 3 项 (per 9/4 A.2) | 33 项 (per OPT-A4 §7 重新枚举) |
| **missing_docs 600+ warning** | 0 (per 9/4 草稿未量化) | 600+ (per 9/5 报告 §3.9 实证) |
| **Frontend 4 tsc err** | 0 (per 9/4 草稿未量化) | 4 (FeatureToggles onCheckedChange + refactor-state-machine 缺 + tailwind-merge 缺) per 9/5 报告 §3.8 |
| **8 混合域 stub 4 拍板** | 0 (per 9/4 草稿未扫) | 4 (H2-EXT #4 DeviceId→Uuid + #5 H2 原 3 domain service.rs + #6 P0-2 ApiError 映射 + #7 G-DEP-09) per OPT-A2 §4 |

---

## §8 签字栏 (5 角色, per 守门 #3 + #14 v2 派生)

| 角色 | 签字 | 状态 |
|---|---|---|
| **架构师** | 架构师 (Mavis 接手 agent per DEC-008) | 🟢 已代签 (per 8/27 19:39 JST) |
| **SRE Lead** | (Mavis 临时代签, 真人到位后追溯) | ⏳ 真人到位 (T3-T5 timeline) |
| **平台 Lead** | (Mavis 临时代签, 真人到位后追溯) | ⏳ 真人到位 (T3-T5 timeline) |
| **评审主持** | (Mavis 临时代签, 真人到位后追溯) | ⏳ 真人到位 (T3-T5 timeline) |
| **PM** | (Mavis 临时代签, 真人到位后追溯) | ⏳ 真人到位 (T3-T5 timeline) |

---

## §9 修订历史 (per 守门 #12 + 守门 #1.2)

| v | 时间 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 12:00 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手 | 首版优化专用 WBS (144 项, 5 优化域 × 18 立项类, 估 50M tokens, 4 子代理 4 维度扫描汇总, 6 立即可启动 + 8 推下 session + 130 等真人/凭证) | 9/7 11:53 JST 用户发令"开子代理全面排查未实现的需求, 制定优化专用 wbs" |
