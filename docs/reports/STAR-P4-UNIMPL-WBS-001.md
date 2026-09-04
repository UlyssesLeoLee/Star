# STAR-P4-UNIMPL-WBS-001 P4 阶段 未实施设计 WBS 落地表(per 9/4 07:14 JST 未实施清单)

> **Status**: 🟡 Draft v0.1 (P4 阶段草案, 待 Ulysses 拍板 Phase 拆分 + 拍板顺序)
> **Created**: 2026-09-04 07:15 JST
> **Authority**: Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手代签 (per 8/27 19:39 JST + 21:59 JST 用户授权)
> **承接**: 
> - 9/4 07:01 JST "把所有未实施设计列出来" 9 大类 ~60 项清单 (本 session 输出)
> - `STAR-P3-WBS-001.md` v0.2 P3 全 5 阶段 60/65 拍板落地
> - `2026-09-03-rf-001-blockers-4items-board.md` 4 阻塞项 A+A+A+B 拍板
> - `2026-09-03-rf-001-final-4items-board.md` 4 类 B+B+B+B 加快并行拍板
> **双轴 WBS**: token 预算(per `STAR-OLU-001.md` 1 SRE·周 = 1.2M)+ 质量门 5 维
> **守门 #1 派生规**: 子项 ≥2 维 (Rerunnable/Volume/Structural/Audit-trail) 强制走 `scripts/automation/<purpose>.py`(per 9/2 00:39 JST + `docs/automation-design.md` v0.1)

本文件是 P4 阶段(承接 P3 全 5 阶段收官 56/64 实质收官 87.5% 之后)的**未实施设计落地 WBS**。把 9/4 清单 60+ 项重组为 8 个 Phase × 4 个轨道,按阻塞等级 + 依赖 + 可并行性拆分。

---

## §0 一句话硬约束

> **5 域 Lead 真人 + 7 外部凭证 = P4 阶段最大瓶颈;3 套新架构初版文档 v0.1 已落档,实装全部 pending;6 续做项跨 4-5 sub-session;1 项推 origin 401 跨 session 续;.worktrees 残留 3 项待 Ulysses 手动删。**
>
> —— per 9/4 07:01 JST 用户发令"把所有未实施设计列出来" + 9/3 11:35 JST B 拍板"加快并行" + 8/21 JST 5 域独立 Lead 拒绝兼任硬约束

---

## §1 P4 阶段拆分原则(per 9/3 B 拍板 + 守门 #1 实证)

### 1.1 4 轨道并行架构(per 9/3 12:39 JST 拍板 B + cargo 互锁规避)

| 轨道 | 内容 | 跨轨道依赖 | cargo 影响 |
|---|---|---|---|
| **轨道 1:阻塞解铃**(Phase A) | 5 域 Lead 寻访 + 凭证收集 + 推 origin retry + .worktrees 清理 | 无 | 0 |
| **轨道 2:6 续做项硬阻塞**(Phase B) | T1.7 76 err + T3.3 + T3.1 + T3.2 + 5.6 + T1.5 | 跟 G-1/G-3/G-10 同步 | 高(全仓) |
| **轨道 3:P3-B/C/D/E/F 续做 + G-1~G-12 缺口**(Phase C-G) | per P3 阶段排期, 跟 5 域 Lead 到位绑定 | 强依赖 A + B | 中(分 domain) |
| **轨道 4:3 套新架构实装 + DDD Review 终审**(Phase H) | LangGraph + Agent Runtime + Tree-sitter 实装 + 21 份 docs review | 强依赖 A 真人到位 | 低(独立 crate) |

**关键约束(per 9/3 12:39 JST B 拍板 + 守门 #1 v19 派生规)**:
- T1.7 + T3.3 可并行(T3.3 是 docs, T1.7 是 cargo 改)
- 4.1 + 4.2 可并行(节省 sub-session, risk cargo 互锁 per `-j 4` 修正)
- 整体 2-3 sub-session 并行, **实际 token 可能 3-5x 超支**(1.85-3.65M → 5.55-18.25M, per AGENTS v0.54:427)

### 1.2 8 Phase 拆分 + 双轴(per `STAR-P3-WBS-001.md` 命名规范)

| Phase | 主题 | token 估 | 阻塞 | 启动条件 |
|---|---|---|---|---|
| **Phase A** | 阻塞解铃(5 域 Lead 寻访 + 凭证收集 + 推 origin + .worktrees 清理) | ~0.1M | 🔴 Ulysses 手动 | 本 session 立即启动 |
| **Phase B** | T1.7 76 err 修法(4.1+4.2+4.3 并行) | 0.55-1.05M | 🟡 cargo | Phase A 启动后 sub-session #1 |
| **Phase C** | T3.3 ubiquitous-language.md + T3.1 star-dto + T1.5 deny 切换 | 0.9M | 🟡 文档 + cargo | Phase A + B 并行 |
| **Phase D** | T3.2 Saga ≥80% 覆盖 + 5.6 H2 原 3 domain + G-10 类型不兼容 | 0.4-1.7M | 🔴 等 5 域 Lead + 2-3 H2 sub-session | Phase A 真人到位 |
| **Phase E** | P3-C/E/F 跨域编排(E.6 Saga + E.7 DDD 验证 + F.1 DDD Review 阶段 5 角色) | 13M | 🔴 等 5 域 Lead 真人 | Phase A + D |
| **Phase F** | P3-B/D 凭证切真 + DB #DB-13 W/T/M 三類横展開 + CI runner 真实配置 | 13M | 🟡 Ulysses 凭证 / GA runner | 凭证到位后启动 |
| **Phase G** | Agent Runtime SRS-001 G-1~G-9 缺口(L0 队列 + L1 bevy_ecs + EventBus + Memory + Checkpoint) | 12M | 🟡 ECS 选型 + L0 PoC | 独立 sub-session, 跟 B 并行 |
| **Phase H** | 3 套新架构实装(LangGraph + Agent Runtime + Tree-sitter)+ DDD Review 21 份 docs 终审 + 签字栏追溯 | 7.5M | 🔴 真人到位 + 6 续做项完成 | 末段, per P3-F #5 + AGENTS §7 #8 |
| **合计** | | **~47M**(理论)/ **~141M**(3x 超支) / **~235M**(5x 超支) | | |

**对比 P3 全 5 阶段**: P3 = ~179.5M / 64 子项 实质收官 56/64 = 87.5%; P4 = ~47-235M / 估 50+ 子项 实质预估 30+/50+ ≈ 60%。

---

## §2 Phase A 阻塞解铃(0.1M / 立即启动)

> **目标**: 把 P4 阶段最大瓶颈(5 域 Lead 真人 + 外部凭证 + 推 origin + .worktrees 清理)集中消解
> **依赖**: 无
> **自动化档**: 1 [P] / 2 [M] / 1 [S] / 5 真人寻访

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **A.1** | 推 origin 1 commit retry (9/3 12:43 JST 401 跨 session 续) | 0.05M | 🟡 | **[P]** `git_push.py` | 守门 #1 1a: 网络 max 2 retries, **401 跨 session 续 + Ulysses 验证 $env:GHCR_PAT** |
| **A.2** | .worktrees 残留 3 项永久删 (PowerShell 限制, Mavis 不越权) | 0 | 🟡 | **[M]** `cleanup_worktrees.py` | `integration-e2e-openclaw.log` + `wt-nav-i18n-a/` + `wt-nav-shots-b/`(per `rf-001-blockers-4items-board.md:58-62`) |
| **A.3** | 5 域 Lead 真人寻访流程(Ulysses 个人网络 / freelance / 开源 3 选 1) | 0 | 🔴 | **[S]** — | `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §1 5 步流程 + REGISTRY 5 行待填 |
| **A.4** | 外部凭证收集(B.5 OpenClaw / B.6 Hermes / E.4 KMS / D.2-D.6 GA runner) | 0 | 🟡 mock 备选可长期维持 | **[M]** `credential_collect.py` | mock 已落地 per 29692a7 + `5ea9611`;Ulysses 决定切真时机 |
| **A.5** | 4 报告签字栏"审批"列 DDD Review 终审(per AGENTS §7 #6) | 0.05M | 🟡 等真人 | **[S]** — | PHASE-D2-CLI-IMPL-REPORT.md / PHASE-D3-MCP-TRANSPORT-REPORT.md / PHASE-D4-P1-FIX-REPORT.md / PHASE-D5-MCP-STREAMABLE-HTTP-REPORT.md 4 份 |

**小计**: 5 子项,0.1M token,**本 session 立即启动**。

**已知缺口 (per 缺标比错标)**:
1. A.3 真人寻访依赖 Ulysses 个人网络(2026-08-30 11:13 JST CONTENT-REVIEW-PACK §0 已有 5 步流程草案,但 0 真人到位)
2. A.4 凭证 4 项可无限期维持 mock,但 G-5 22 domain-identity 待联(per SRS-001:91)受其阻塞
3. 推 origin 401 错误根因: `GHCR_PAT` token scope 限制或刚失效(per AGENTS v0.54:430 实证)

---

## §3 Phase B T1.7 76 err 修法(0.55-1.05M / 硬阻塞优先)

> **目标**: 消解 star-mcp 25+ err + domain-local-runtime 51 err + 守门 #1 v3 派生规 --all-targets 716 err baseline
> **依赖**: Phase A.1(0 跨,纯 cargo)
> **自动化档**: 4 [P] / 1 [M] / 0 [S]
> **执行顺序**(per 9/3 12:39 JST B 拍板 4.1+4.2 并行):

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **B.1** | T1.7 4.1 `ActorContext::as_local_runtime(mut self) -> Self` helper 落地 | 0.15M | 🟡 已实证 51→10 err (`65a8da0`) | **[P]** `actor_helper.py` | per `AGENTS.md v0.55:438-446` 实证 |
| **B.2** | T1.7 4.2 改写 star-mcp 2 份 tests (消解 25+ err) | 0.2-0.5M | 🟡 实证 50+ err 跨 handlers/+tools/(per AGENTS v0.56:457) | **[P]** `star_mcp_test_refactor.py` | handlers/ + tools/ 修法 跨 sub-session 续做 |
| **B.3** | T1.7 4.3 守门 #1 v3 派生规 文字补全 (实证缺口) | 0.05M | 🟡 | **[S]** — | per `AGENTS.md v0.56:458` 实证: --all-targets 716 err 5.1-5.5 报告"0 行代码改动"未保持 |
| **B.4** | 4.4 守门 #1 v3 派生规 实证 (跨 sub-session 收敛) | 0.15-0.35M | 🟡 | **[P]** `all_targets_baseline.py` | 守门 --workspace --all-targets 0 err 实证(per `--j 4` 修正) |

**小计**: 4 子项,0.55-1.05M token,**sub-session #1-#2**(B.1+B.2 并行 per 9/3 12:39 JST B 拍板,B.3 推下,B.4 收尾)。

**守门 #1 实证 baseline**: per AGENTS v0.55:443 "实际全 19+ crate 错总数 716 err",B.4 需消解到 0 err。

**已知缺口 (per 缺标比错标)**:
1. B.1 已落地但 baseline 716 err 未保持,实际进度需重新 cargo check
2. B.2 实证 50+ err(超越原 25 err 估),handlers/ + tools/ 跨 sub-session
3. cargo 互锁风险(per 9/2 E 阶段 5min timeout + 9/3 12:52 JST B 拍板警告),用 `-j 4` 修正

---

## §4 Phase C T3.3 + T3.1 + T1.5(0.9M / 文档 + cargo 并行)

> **目标**: 共享 star-dto 重构 + ubiquitous-language.md v1.0 + unreachable_pub deny 切换
> **依赖**: B.1(避免 B.2 handlers/ 改动污染)
> **自动化档**: 2 [P] / 0 [M] / 1 [S]

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **C.1** | T3.3 ubiquitous-language.md v1.0(22 domain 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design) | 0.1M | 🟡 v0.1 已落(`524a75a`),扩 v1.0 | **[P]** `ubiquitous_lang_gen.py` | per `AGENTS.md:449` v0.1 已落 |
| **C.2** | T3.1 共享 star-dto 重构(消除 22 domain 字段重复定义) | 0.5M | 🟡 | **[P]** `star_dto_extract.py` | per `rf-001-blockers-4items-board.md:67` 依赖 T1.7 |
| **C.3** | T1.5 `unreachable_pub = "deny"` 3 步切换 | 0.3M | 🟡 | **[S]** — | per `rf-001-blockers-4items-board.md:67` 独立,3 步: 加 allow 属性 → 改 deny → 删 allow |

**小计**: 3 子项,0.9M token,**sub-session #2**(跟 B 并行)。

**已知缺口 (per 缺标比错标)**:
1. C.2 star-dto 抽取会触发 22 domain cargo check 错误,需 sub-session 串行
2. C.3 deny 切换一次性,失败回退成本高,先在 1-2 域试运行

---

## §5 Phase D T3.2 Saga + 5.6 H2 + G-10(0.4-1.7M / 强依赖真人)

> **目标**: 跨域 Saga 编排 + H2 原 3 domain 改造 + 类型不兼容消解
> **依赖**: Phase A.3 真人到位 + Phase C 共享 dto
> **自动化档**: 2 [P] / 1 [M] / 0 [S]

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **D.1** | G-10 H2 类型不兼容修法 (DeviceId 强类型 + String→Uuid 业务语义) | 0.3-1.6M | 🔴 实证 0.3-0.5M 估→1.1-1.6M 实测(per AGENTS v0.54:417 + HANDOFF-ST-001 v0.2 §1) | **[P]** `h2_type_unify.py` | 5 domain 跨域字段扩展: tenant_policy_id + workspace_ids + is_platform_operator helper |
| **D.2** | T3.2 Saga ≥80% 覆盖(5 域 Lead 反转可启动 per 守门 #3 v2) | 0.1M | 🔴 等 match 域 Lead | **[P]** `saga_coverage.py` | per `rf-001-blockers-4items-board.md:67` 依赖 T3.1 + 5 域 Lead |
| **D.3** | 5.6 H2 原 3 domain 改造(feedback/validation/integration ~150+ call sites) | 0.3-1.6M | 🔴 依赖 D.1 helper | **[M]** `h2_3domain_migrate.py` | per `scripts/p0_h2_3domain_migration.py` 已落地,3 阶段串行 |

**小计**: 3 子项,0.4-1.7M token(估区间 4x,**实际 3-5x 超支** per AGENTS v0.54:427),**sub-session #3-#5**。

**已知缺口 (per 缺标比错标)**:
1. D.1 5 domain 跨 session 续做跨 1-2 sub-session(per AGENTS v0.55:444 实证)
2. D.2 依赖 5 域 Lead 真人到位(Mavis 临时代签 per 守门 #3 v2 反转已生效)
3. D.3 H2 原 3 domain 改造 实证 0.3-0.5M 估 → 1.1-1.6M 实测 3-5x 超支(per `AGENTS v0.54:417` + `HANDOFF-ST-001 v0.2 §1`)

---

## §6 Phase E P3-C/E/F 跨域编排(13M / 强依赖真人)

> **目标**: 5 域跨域业务落地 + 真人 review 21 份 docs
> **依赖**: Phase A.3 5 真人到位 + Phase D 类型不兼容消解
> **自动化档**: 3 [P] / 2 [M] / 2 [S]

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **E.1** | E.6 5 域 Saga 实装(跨域补偿/失败回滚 per Q-003) | 4.5M | 🔴 | **[P]** `saga_e2e.py` | per `STAR-P3-WBS-001.md:153` 等 E.5 真人到位启动 |
| **E.2** | E.7 5 域 DDD 边界验证(BoundedContext/Aggregate/Entity 文档 + code review) | 4.5M | 🟡 docs 阶段(per `e67bc8c`,44.6KB 已落) | **[M]** `ddd_review.py` | 真人到位后 review 签字 |
| **E.3** | F.1 DDD Review 阶段 5 角色真人到位(架构+SRE+平台+评审+PM) | 4M | 🔴 | **[S]** — | per `STAR-P3-WBS-001.md:167` per STAR-OLU-001 §6 质量门 5 维终评 |
| **E.4** | CONTENT-REVIEW-PACK 21 份 docs 评审(13 docs + 6 P3 报告 + 2 INC-SESSION) | 0 | 🔴 真人到位 | **[M]** `content_review_runbook.py` | per `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §0 21 份 review 目标 |
| **E.5** | REGISTRY 5 行追溯签字(覆盖 Mavis 临时代签) | 0 | 🔴 | **[S]** — | per `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` §1 5 行待填 |

**小计**: 5 子项,13M token,**真人到位后 sub-session 启动**,每域真人 30-60 分钟 × 5 域 = 2.5-5 小时 + 21 份 docs 评审 5-10 小时。

**已知缺口 (per 缺标比错标)**:
1. E.1 5 域 Lead 真人到位是硬阻塞(Mavis 临时代签 per 守门 #3 v2 反转已生效,真人到位后追溯)
2. E.4 21 份 docs 评审工作量大,需要 5 域 Lead 协同
3. 质量门 5/5 实证(per STAR-OLU-001 §6)需 DDD Review 阶段 Lead 真实身份到位后校准

---

## §7 Phase F 凭证切真 + DB W/T/M + CI runner(13M / 凭证依赖)

> **目标**: B.5/B.6/E.4 切真凭证 + 守门 #DB-13 跨项目落地 + D.2/D.6 CI runner 配置
> **依赖**: Phase A.4 凭证收集
> **自动化档**: 3 [P] / 2 [M] / 0 [S]

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **F.1** | B.5 OpenClaw 真实集成 e2e(凭证切真) | 5M | 🟡 mock 备选已落地 per 29692a7 | **[P]** `integration_e2e.py` | per `STAR-P3-WBS-001.md:73` endpoint + API key 切真 |
| **F.2** | B.6 Hermes 真实集成 e2e(凭证切真) | 5M | 🟡 mock 备选已落地 per 29692a7 | **[P]** `integration_e2e.py` | 同 B.5 |
| **F.3** | E.4 KMS 集成(Vault / AWS KMS 凭证) | 5M | 🟡 LocalMockKms 已实装 per `5ea9611` | **[P]** `kms_rotate.py` | per `STAR-P3-WBS-001.md:151` 凭证切真 |
| **F.4** | 守门 #DB-13 DB 三類横展開(W/T/M) 跨项目 P3-D 阶段落地 | 3M | 🟡 | **[M]** `wtm_classifier.py` | per SRS-001:136 + `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 + CW-01~CW-10 派生守门 |
| **F.5** | D.2/D.6 CI runner 真实配置(跨平台 e2e + markdownlint + cargo doc CI job) | 3M | 🟡 stub 已实装 per `8ace1d5` | **[M]** `ci_runner.py` | per `STAR-P3-WBS-001.md:128,132` 真实 GitHub Actions runner |

**小计**: 5 子项,21M token(原估 13M,CI runner 3M + DB 3M 增量),**凭证到位后启动**。

**已知缺口 (per 缺标比错标)**:
1. F.1-F.3 凭证可无限期维持 mock,Ulysses 决定切真时机
2. F.4 DB W/T/M 跨项目 P3-D 落地需要 domain 数据表清单(100 表 per `00-CLASSIFICATION-W-T-M.md`)
3. F.5 CI runner 需要 Ulysses GitHub repo 管理员权限

---

## §8 Phase G Agent Runtime G-1~G-9 缺口(12M / ECS 选型 + 独立 sub-session)

> **目标**: Agent Runtime SRS-001 12 节部分 / 60 节待 P3-B-F 落地
> **依赖**: 无(可跟 B 并行)
> **自动化档**: 5 [P] / 2 [M] / 2 [S]
> **基线**: `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 G-1~G-12 已知缺口

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **G.1** | G-1 L0 SQLite 任务队列(1M 派发持久化) | 1.5M | 🟡 P3-B L0 PoC | **[P]** `l0_queue_poc.py` | per `SRS-001:87` |
| **G.2** | G-2 L1 bevy_ecs / flecs 选型 + 9 SA Archetype 落地 | 2M | 🟡 P3-B 启动 | **[P]** `ecs_bench.py` | per `SRS-001:88` 选型缺口 G-2 |
| **G.3** | G-3 EventBus + Mailbox 实现(Agent 间通信协议) | 1M | 🟡 P3-B | **[P]** `eventbus_proto.py` | per `SRS-001:89` |
| **G.4** | G-4 Shared LLM/HTTP/MCP Pool(守门 #24 subprocess 池扩展 ECS 池) | 2M | 🟡 P3-C | **[P]** `shared_pool.py` | per `SRS-001:90` |
| **G.5** | G-5 Tenant Quota + 多租户隔离(22 domain-identity 联) | 1.5M | 🟡 P3-D | **[M]** `tenant_quota.py` | per `SRS-001:91` |
| **G.6** | G-6 Memory Store(外置) | 1M | 🟡 P3-D | **[P]** `memory_store.py` | per `SRS-001:92` |
| **G.7** | G-7 Crash Recovery + Checkpoint | 1M | 🟡 P3-D | **[M]** `recovery_proto.py` | per `SRS-001:93` |
| **G.8** | G-8 Context Tiering (L1/L2/L3) | 1M | 🟡 P3-D | **[S]** — | per `SRS-001:94` |
| **G.9** | G-9 Token 计量 telemetry(per §7 v0.8) | 1M | 🟡 P3-B telemetry 落地 | **[S]** — | per `SRS-001:95` |

**小计**: 9 子项,12M token,**独立 sub-session,跟 B 并行**(G 主要是 ECS 选型 + 设计, B 是 cargo 改)。

**已知缺口 (per 缺标比错标)**:
1. G.2 bevy_ecs vs flecs 选型需 Ulysses 拍板(bevy_ecs 社区成熟,flecs 性能更佳)
2. G.4 Shared Pool 跟守门 #24 subprocess 池有差异,需设计 Adapter 模式
3. G.9 telemetry 接入需要 SRE Lead 真实身份到位

---

## §9 Phase H 3 套新架构实装 + DDD Review 终审(7.5M / 末段)

> **目标**: LangGraph + Agent Runtime + Tree-sitter 实装 + 21 份 docs 终审 + 签字栏追溯
> **依赖**: Phase E.3 真人到位 + Phase G ECS 选型
> **自动化档**: 3 [P] / 2 [M] / 1 [S]

| # | 子项 | 标题 | token 估 | 状态 | 自动化档 | 备注 |
|---|---|---|---|---|---|---|
| **H.1** | LangGraph PostgreSQL checkpointer 实装 | 1M | 🟡 v0.1 文档完成, 实装 pending per AGENTS §7 #8 | **[P]** `lg_checkpoint.py` | per AGENTS v0.69:739 缺口 #121 |
| **H.2** | LangGraph 跨仓(Physis/RGS)RPC 实装 | 0.5M | 🟡 v0.3 计划 per AGENTS v0.69:739 缺口 #122 | **[M]** `lg_cross_repo.py` | per AGENTS v0.69 缺口 #122 |
| **H.3** | LangGraph 16 tool sub-agent 経由 call 化(补 12 tool 留 P2 缺 service) | 1.5M | 🟡 跟 AGENTS §7 #2 强绑定 | **[P]** `tool_subagent_bridge.py` | per AGENTS v0.69:739 缺口 #123 |
| **H.4** | LangGraph State schema v1 migration 路径 | 0.5M | 🟡 v0.2 计划 per AGENTS v0.69:739 缺口 #124 | **[S]** — | per AGENTS v0.69 缺口 #124 |
| **H.5** | Tree-sitter Rust crate 引入 + 4-6 语言 grammar | 1.5M | 🟡 v0.1 文档完成(per 2026-09-03 19:5X JST 用户发令) | **[M]** `treesitter_init.py` | 全仓 0 `tree-sitter` 引用(per grep 实证) |
| **H.6** | Tree-sitter 任务卡 ↔ worktree 1:1 绑定 + react-flow graph 渲染 | 1M | 🟡 | **[P]** `task_graph_view.py` | per `docs/architecture/2026-09-03-treesitter-worktree-graph/01-requirements.md` §1.4 |
| **H.7** | Tree-sitter symbol resolver 跨文件引用追踪 | 0.5M | 🟡 | **[P]** `symbol_resolver.py` | per 同上 |
| **H.8** | DDD Review 21 份 docs 终审 + 签字栏追溯(Mavis 代签 → 真人覆盖) | 1M | 🔴 真人到位 | **[S]** — | per `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §0 21 份 |

**小计**: 8 子项,7.5M token,**末段,per P3-F #5 + AGENTS §7 #8**。

**已知缺口 (per 缺标比错标)**:
1. H.1-H.4 依赖 AGENTS §7 #2 16 tool 真实接入完成(11/25 + 3/16 + 1/16 = 12/16 完成,4/16 pending)
2. H.5 Tree-sitter 引入会触发 `Cargo.lock` 大量变更,需全仓 cargo check
3. H.8 真人到位是 Mavis 代签 → 真人覆盖的转换点,涉及 commit author 变更

---

## §10 Phase 推进时序图(per 9/3 B 拍板加快并行)

```
Sub-session #1 (本 session 立即):
  ├─ Phase A.1 推 origin retry (0.05M) ───────────┐
  ├─ Phase A.2 .worktrees 清理脚本生成 (0M) ──────┤ 轨道 1
  ├─ Phase A.3 5 域 Lead 寻访流程 (0M) ────────────┤
  ├─ Phase A.4 凭证收集脚本 (0M) ──────────────────┤
  └─ Phase A.5 4 报告签字栏 (0.05M) ───────────────┘

Sub-session #2 (4-5 sub-session):
  ├─ Phase B.1+ B.2 T1.7 修法 (0.35-0.65M) ────────┐ 轨道 2
  ├─ Phase C.1 T3.3 文档 (0.1M) ──────────────────┤ 跟 B 并行
  ├─ Phase C.2 T3.1 star-dto (0.5M) ──────────────┤
  └─ Phase G.1-G.9 G-1~G-9 缺口 (12M) ───────────┘ 轨道 3 独立

Sub-session #3-#5 (等真人):
  ├─ Phase C.3 T1.5 deny (0.3M) ──────────────────┐ 轨道 2
  ├─ Phase D.1 G-10 H2 类型 (0.3-1.6M) ───────────┤ 轨道 2/3 合并
  ├─ Phase D.2 T3.2 Saga (0.1M) ──────────────────┤
  └─ Phase D.3 5.6 H2 原 3 domain (0.3-1.6M) ──────┘

Sub-session #6+ (凭证到位):
  ├─ Phase F.1-F.3 凭证切真 (15M) ─────────────────┐ 轨道 4
  ├─ Phase F.4 DB W/T/M (3M) ─────────────────────┤
  └─ Phase F.5 CI runner (3M) ─────────────────────┘

末段 (真人到位):
  ├─ Phase E.1-E.5 5 域 Lead 编排 (13M) ───────────┐
  ├─ Phase H.1-H.7 3 套架构实装 (6M) ──────────────┤
  └─ Phase H.8 DDD Review 终审 (1.5M) ────────────┘
```

**总估**: 47M 理论 / 141M 3x 超支 / 235M 5x 超支(per AGENTS v0.54:427 B 拍板风险警告)。

---

## §11 累计统计 + 自动化档汇总

| Phase | 子项 | token 估(理论) | 状态预估 | 自动化档 |
|---|---|---|---|---|
| A 阻塞解铃 | 5 | 0.1M | 4/5 本 session + 1 等真人 | 1P/2M/1S + 5 真人 |
| B T1.7 修法 | 4 | 0.55-1.05M | 4/4 sub-session #1-#2 | 3P/0M/1S |
| C T3.3/T3.1/T1.5 | 3 | 0.9M | 3/3 跟 B 并行 | 2P/0M/1S |
| D T3.2/5.6/G-10 | 3 | 0.4-1.7M | 3/3 等真人 | 2P/1M/0S |
| E P3-C/E/F 编排 | 5 | 13M | 5/5 等真人 | 3P/2M/2S |
| F 凭证 + DB + CI | 5 | 21M | 5/5 等凭证 | 3P/2M/0S |
| G G-1~G-9 缺口 | 9 | 12M | 9/9 跟 B 并行 | 5P/2M/2S |
| H 3 套新架构 + 终审 | 8 | 7.5M | 8/8 末段 | 3P/2M/1S |
| **合计** | **42 子项** | **~55M 理论** / **~165M 3x** / **~275M 5x** | **~38/42 估 90%** | **22P / 11M / 7S = 40/42(去重) + 5 真人** |

---

## §12 守门规则(本文件专属)

| # | 规则 | 出处 |
|---|---|---|
| 1 | 本文件仅作 Phase 拆分草案, **不实施任何子项**, 实施需 Ulysses 拍板 | 9/1 14:58 JST 拍板"决策必须用选项" |
| 2 | Phase 排序按阻塞等级降序, 实装门槛是质量门禁 ≥4/5, 不是截止日期 | `STAR-OLU-001 §0` |
| 3 | 🔴 项需 Ulysses 拍板, 🟡 项可 Mavis 推进但需 brief 落地, 🟢 项可自动 | AGENTS §4 #10 + #12 |
| 4 | token 软预算 ÷ 1.2M SRE·周上限 → 软参考周, **不参与 gating** | STAR-OLU-001 §1 |
| 5 | 任务卡自动化档([P]/[M]/[S])强制落档, 4 维打分, 共享脚本落 `scripts/automation/<purpose>.py` | 9/2 00:39 JST 拍板 + `docs/automation-design.md` v0.1 |
| 6 | Phase 命名沿用 `STAR-P3-WBS-001.md` 规范(STAR-P{N}-{PHASE}-*)| 同上 |
| 7 | 子项 ≥2 维 (Rerunnable/Volume/Structural/Audit-trail) 强制 Python 化 | 守门 #1 v19 + #9 v2 + #12 v2 派生规 |
| 8 | 真人到位追溯签字(Mavis 临时代签 → 真人覆盖)必须 email + 到岗日 + 评审结论 3 字段 | `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` §0 |
| 9 | 推 origin 401 错误不算 timeout, 跨 session 续, Ulysses 验证 $env:GHCR_PAT | AGENTS §4 守门 #1 1a 重试细则 |
| 10 | cargo check --workspace --all-targets 0 err 必跑, 不能只看 --lib | 守门 #1 v3 派生规 |

---

## §13 签字栏(5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses(一人公司 12 角色 per DEC-008)— Mavis 接手 | 2026-09-04 | 🟡 P4 WBS 草案 42 子项 / 8 Phase / 4 轨道; 1 子项本 session 立即启动(A.1 推 origin retry); 5 子项等真人; 5 子项等凭证; 25 子项可推进 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟡 Mavis 接手代签(per 8/27 19:39 JST + 21:59 JST 用户授权) |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟡 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟡 Mavis 接手代签 |
| 5 | 项目负责人(PM)| 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-04 | 🟡 Mavis 接手代签 |

---

## §14 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-04 07:15 JST | 架构师 (Mavis 接手 agent per DEC-008)— Mavis 接手代签 Ulysses | 初版: 8 Phase × 4 轨道拆分, 42 子项 / ~55M token 理论估 / 5x 超支 ~275M; 1 项本 session 立即启动; 5 项等真人; 5 项等凭证; 25 项可推进; 自动化档 22P/11M/7S + 5 真人; 累计统计 + 推进时序图 + 10 守门 + 5 签字栏 + 4 引用 | 2026-09-04 07:01 JST 用户发令"把所有未实施设计列出来" → 9 大类 ~60 项清单 → 拆分 WBS Phase 草案 |

---

## §15 引用文档

- `AGENTS.md` §4 / §4.1 / §6 / §7 — 守门 + ADR 索引 + 待办
- `STAR-OLU-001.md` v0.1 — token-OLU 独立基线(1 SRE·周 = 1.2M)
- `STAR-P3-WBS-001.md` v0.2 — P3 全 5 阶段 60/65 拍板落地(命名规范沿用)
- `2026-09-03-rf-001-blockers-4items-board.md` v0.1 — 4 阻塞项 A+A+A+B 拍板
- `2026-09-03-rf-001-final-4items-board.md` v0.1 — 4 类 B+B+B+B 加快并行
- `2026-09-03-rf-001-h2-3domain-defer.md` v0.1 — H2 3 domain 暂缓
- `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 — Agent Runtime SRS G-1~G-12 缺口
- `docs/architecture/2026-09-03-langgraph/{01-requirements,02-basic-design,03-detailed-design}.md` v0.1 — LangGraph 3 份 IPA
- `docs/architecture/2026-09-03-agent-runtime/{02-basic-design,03-detailed-design}.md` v0.1 — Agent Runtime 2 份 IPA
- `docs/architecture/2026-09-03-treesitter-worktree-graph/{01-requirements,02-basic-design}.md` v0.1 — Tree-sitter 2 份 IPA
- `docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` — Agent Runtime SRS Baseline
- `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` — Agent Runtime Basic+Detailed Design Baseline
- `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` v0.1 — 5 域 Lead 真人 review 操作手册
- `STAR-P3-5-DOMAIN-LEAD-REGISTRY.md` v0.1 — 5 域 Lead 真人注册表
- `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 — 100 表 W/T/M 三類索引
- `docs/data-design/ipa-detail/00-CLASSIFICATION-RULES.md` v0.1 — 跨项目 ルール + CW-01~CW-10
- `docs/automation-design.md` v0.1 — 任务卡自动化档 + registry

---

## §16 拍板请求(per 9/1 14:58 JST "决策必须用选项")

> **本文件是草案, 等 Ulysses 拍板 4 项后才进入实施阶段。**

| # | 拍板项 | 选项 A | 选项 B | 推荐 |
|---|---|---|---|---|
| 1 | **Phase A.1 推 origin retry 时机** | A. 本 session 续 retry (守门 #1 1a max 2 retries) | B. 下 session 第一件事 retry | A (本 session 立即消化, 不积压) |
| 2 | **Phase A.3 5 域 Lead 寻访方法** | A. Ulysses 个人网络 (5 工程师各认领 1 域) | B. freelance 平台 (Toptal/Upwork) | A (更快 + 跟项目熟悉) |
| 3 | **Phase A.4 凭证切真时机** | A. 立即切真 (需 Ulysses 提供 B.5/B.6/E.4 凭证) | B. 维持 mock 长期跑 (per 29692a7) | B (mock 路径已落地, 不阻塞) |
| 4 | **整体推进策略** | A. 串行 8 Phase (风险低, 慢) | B. 4 轨道并行 (per 9/3 B 拍板, 快, 风险 cargo 互锁 + 3-5x 超支) | B (per 9/3 12:39 JST 拍板 B 已生效) |
