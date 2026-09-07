# OPT-A2 WBS / Phase / 报告 pending 扫描报告

> **Created**: 2026-09-07 11:55 JST | **Authority**: Mavis 接手 (per 8/27 19:39 JST Ulysses 授权代签)
> **扫描范围**: `D:\Star\docs/{reports,architecture,requirements,data-design,recruitment,basic-design,automation-design}.md` + `D:\Star\AGENTS.md` §4/§4.1/§4.2/§7 + `D:\Star\docs/briefs/*.md` + `D:\Star\scripts/automation/`
> **守门基线**: 守门 #1 v19 + 守门 #19 v19+#20+#21+#22+#23+#24
> **当前 main HEAD**: `bfb0bca` (per `D:\Star\.git\refs\heads\main`, 7 ahead origin/main, per 9/7 11:53 JST brief; AGENTS.md §7 表头仍标 `ab91613` — 表头 main HEAD 不同步, 列入 §7 已知缺口 #1)

---

## §0 摘要

- **Phase pending 总数**: **23** (跨 7 个 Phase: A 阻塞解铃 1 + B T1.7 76 err 4 + C T3.3/T3.1/T1.5 3 + D T3.2/5.6/G-10 3 + E P3-C/E/F 编排 5 + F 凭证 + DB + CI 5 + G Agent Runtime G-1~G-9 9 中各 Phase 的 pending/blocked 累加; Phase A-H 全 42 子项中 23 标 🟡/🔴 pending, 19 标 🟢 收官)
- **守门缺口总数**: **19** (AGENTS.md §4 #1-#14 主体 14 + §4.1 派生规 v1-v26 中 5 项未落地/待真人/跨 session 续: v18 H2-EXT 强类型重构 / v22 控制台不污染 main 实证持续 / #1 R-05 推 origin retry / #3 5 域 Lead 真人 / #23 PR #1 真人 merge)
- **跨子代理未派发任务**: **5** (`docs/briefs/5-leads/{player,economy,match,social,admin}.md` brief 已落档, 5 子代理未 dispatch, per V2-6 9/4 18:30 JST 守门 #3 反转 + 守门 #20 实证)
- **HANDOFF H1-H4 + H2-EXT 8 domain 推进状态**: 4 落地 (H1 commit 2 dirty files 已 commit + H2 stage 1 commit `68ae5ff` + H3 `as_uuid()` 统一签名 + H4 ST 措辞 "4 域独立"), H2-EXT 8 domain = 3 原 + 5 EXT, 5/8 完成 (3 EXT commit `9d08f80`/`b6f6e2a`/`7f611b0` + 1 EXT 拍板 device_id String=hostname per HANDOFF v0.5), 3/8 仍 🔴 阻塞 (H2-EXT #4 DeviceId→Uuid 重构 / #5 H2 原 3 domain service.rs 改造 / #6 P0-2 ApiError 映射), 跨 session 续
- **§7 已知缺口数量**: 12 (per 缺标比错标, 显式列)

---

## §1 Phase pending 矩阵

来源: `STAR-P4-UNIMPL-WBS-001.md` v0.1 (8 Phase × 4 轨道 × 42 子项, 9/4 07:14 JST) + `HANDOFF-ST-001.md` v1.4 9/5 00:40 JST + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` v0.1 9/5 00:55 JST + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` v0.3.4 9/5 11:27 JST。

| # | Phase | Phase X.Y | pending 描述 | 依赖 | 优先级 |
|---|---|---|---|---|---|
| 1 | A | A.1 | 推 origin 1 commit retry (per 9/3 12:43 JST 401 跨 session 续) | 守门 #1 1a 推 origin 重试细则; github.com 偶发中断 30s-2min 后常恢复 | 🟡 中 (本 session 立即可启动) |
| 2 | A | A.2 | `.worktrees` 残留 3 项永久删 (PowerShell 限制, Mavis 不越权) | Ulysses 手动 (`integration-e2e-openclaw.log` + `wt-nav-i18n-a/` + `wt-nav-shots-b/`, per `rf-001-blockers-4items-board.md:58-62`) | 🟡 中 |
| 3 | A | A.3 | 5 域 Lead 真人寻访流程启动 (Ulysses 个人网络 拍板 Q1, per `docs/recruitment/5-business-domain-lead-referral.md` v0.1) | 5 域 Lead 真人到位 (T3 ~ 2026-09-26 JST) | 🔴 高 (跨 8 阶段) |
| 4 | A | A.5 | 4 报告签字栏"审批"列 DDD Review 终审 (PHASE-D2-CLI / PHASE-D3-MCP-TRANSPORT / PHASE-D4-P1-FIX / PHASE-D5-MCP-STREAMABLE-HTTP-REPORT) | 真人到位 | 🟡 中 |
| 5 | B | B.1 | T1.7 4.1 `ActorContext::as_local_runtime(mut self) -> Self` helper 实证 51→10 err (`65a8da0`), baseline 716 err 未保持 | 0 跨, 纯 cargo | 🟡 中 (已部分落地, 续做) |
| 6 | B | B.2 | T1.7 4.2 改写 star-mcp 2 份 tests (消解 50+ err, handlers/ + tools/) | sub-session 续做 (per AGENTS v0.56:457) | 🟡 中 |
| 7 | B | B.3 | T1.7 4.3 守门 #1 v3 派生规 文字补全 (`--all-targets` 716 err 5.1-5.5 报告"0 行代码改动"未保持) | 文档补全 | 🟡 中 |
| 8 | B | B.4 | T1.7 4.4 守门 #1 v3 实证 (`--all-targets` 0 err 跨 sub-session 收敛) | `-j 4` 修正 (per 9/3 12:52 JST) | 🟡 中 |
| 9 | C | C.1 | T3.3 ubiquitous-language.md v1.0 扩 (22 domain 字段命名表 + 5 抽样对照 spec 附录 B vs basic-design, v0.1 已落 `524a75a`) | B.1 避免污染 | 🟡 中 |
| 10 | C | C.2 | T3.1 共享 star-dto 重构 (消除 22 domain 字段重复定义) | T1.7 | 🟡 中 |
| 11 | C | C.3 | T1.5 `unreachable_pub = "deny"` 3 步切换 (独立, 3 步: 加 allow 属性 → 改 deny → 删 allow) | 独立 | 🟡 中 |
| 12 | D | D.1 | G-10 H2 类型不兼容修法 (DeviceId→Uuid 强类型 + String→Uuid 业务语义, 5 domain 跨域字段扩展) | 真人到位 (per 守门 #3 v2) | 🔴 高 (估 0.3-1.6M, 3-5x 超支 per AGENTS v0.54:417 + HANDOFF v0.2 §1) |
| 13 | D | D.2 | T3.2 Saga ≥80% 覆盖 (5 域 Lead 反转可启动 per 守门 #3 v2) | match 域 Lead + T3.1 | 🔴 高 |
| 14 | D | D.3 | 5.6 H2 原 3 domain 改造 (feedback/validation/integration ~150+ call sites) | D.1 helper | 🔴 高 (3 阶段串行) |
| 15 | E | E.1 | E.6 5 域 Saga 实装 (跨域补偿/失败回滚 per Q-003) | E.5 真人到位 | 🔴 高 (4.5M) |
| 16 | E | E.3 | F.1 DDD Review 阶段 5 角色真人到位 (架构+SRE+平台+评审+PM) | 5 域 Lead + SRE/平台/评审/PM 真人到位 | 🔴 高 (4M) |
| 17 | E | E.4 | CONTENT-REVIEW-PACK 21 份 docs 评审 (13 docs + 6 P3 报告 + 2 INC-SESSION) | 真人到位 | 🔴 高 |
| 18 | E | E.5 | REGISTRY 5 行追溯签字 (覆盖 Mavis 临时代签) | 真人到位 | 🔴 高 |
| 19 | F | F.1 | B.5 OpenClaw 真实集成 e2e (凭证切真) | 凭证到位 | 🟡 中 (5M, mock 备选已落地 per `29692a7`) |
| 20 | F | F.2 | B.6 Hermes 真实集成 e2e (凭证切真) | 凭证到位 | 🟡 中 (5M, mock 备选已落地 per `29692a7`) |
| 21 | F | F.3 | E.4 KMS 集成 (Vault / AWS KMS 凭证) | 凭证到位 (LocalMockKms 已实装 per `5ea9611`) | 🟡 中 (5M) |
| 22 | F | F.4 | 守门 #DB-13 DB 三類横展開 (W/T/M) 跨项目 P3-D 阶段落地 | domain 数据表清单 (100 表 per `00-CLASSIFICATION-W-T-M.md`) | 🟡 中 (3M) |
| 23 | F | F.5 | D.2/D.6 CI runner 真实配置 (跨平台 e2e + markdownlint + cargo doc CI job) | GitHub Actions 管理员权限 (stub 已实装 per `8ace1d5`) | 🟡 中 (3M) |

**说明**: Phase G (Agent Runtime G-1~G-9 9 子项) 9 子项全部 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §8); Phase H (3 套新架构实装 H.1-H.8) 7 子项 🟡 pending + 1 子项 (H.8 真人到位) 🔴 阻塞; 因 Phase G/H 全部子项尚未在 WBS 中分阶段落地,本表只列 P4 主表 Phase A-F 23 条; Phase G/H 在 §5 WBS 子项 pending / blocked 表续列。

---

## §2 守门缺口矩阵 (per AGENTS §4 / §4.1 / §4.2)

来源: `AGENTS.md` §4 row 1-14 + §4.1 v1-v26 + §4.2 (7 条实装前一致性门)。

| 缺口 # | 守门 # | 缺口描述 | 触发时间 | 关联子项 | 状态 |
|---|---|---|---|---|---|
| 1 | #1 v1a | 推 origin 网络错 (Recv failure / Connect failed / timeout) max 2 retries, **401 Authentication failed 不算 timeout, 跨 session 续, Ulysses 验证 $env:GHCR_PAT** | 2026-09-03 11:07 JST 401 实证 | Phase A.1 推 origin retry | 🟡 pending (per `HANDOFF-ST-001.md` v0.7 §10.4) |
| 2 | #1 v15 | 守门 #12 死循环饱和边界 (per `5cfb7b3` 实证); commit-time docs 同步触达饱和后, 任何后续 docs 同步 commit 必先有**新事件触发** (代码改动 / Ulysses 拍板) | 2026-08-29 22:39 JST `5cfb7b3` 实证 | 跨所有 docs 同步 commit | 🟢 已落地 (饱和约束保持) |
| 3 | #1 v18 | H2 范围扩量触发 (HANDOFF-ST-001 H2 原估 3 domain 实际是 8 domain, 实证 0.3-0.5M 估 → 1.1-1.6M 实测 3-5x 超支) | 2026-09-01 09:50 JST commit `9d08f80` + `b6f6e2a` + `7f611b0` | HANDOFF H2-EXT 5 domain 跨 session 续 | 🟡 partial (3/5 EXT + 1 拍板 device_id=hostname + 1 H2 原 3 domain service.rs 改造 commit `76aaf15` 闭环, 1 H2-EXT #4 DeviceId→Uuid 仍跨 session 续) |
| 4 | #3 | 5 域 Lead 真人到位 (per 8/21 JST 拒绝兼任硬约束, 当前 Mavis 临时代签 per 9/3 11:35 JST 反转 + 9/3 19:35 JST 拍板 D 维持 + 9/5 10:43 JST Q1=内推+Q2=立即启动 拍板) | 2026-08-21 JST 拍板 | Phase A.3 真人寻访 + Phase E.1-E.5 + Phase H.8 + P3-C.9 / P3-E.5 / P3-F.1 | 🔴 阻塞 (Ulysses 启动寻访, 真人到位 timeline T3 ~ 2026-09-26 JST, T4 满员 ~ 2026-10-17 JST, per `docs/recruitment/5-business-domain-lead-referral.md` v0.1) |
| 5 | #3 v2 | 守门 #3 v2 (Mavis 临时代签 5 域 Lead, per 9/3 11:35 JST 反转) | 2026-09-03 11:35 JST | 跨所有 5 域决策 + commit + 报告审批 | 🟢 已落地 (维持, 真人到位后追溯) |
| 6 | #3 v25 (派生) | 5 域 Lead 真人 Ulysses 内推 brief + timeline 拍板落地 (per 9/5 10:43 JST `ask_409cbd32edc309d71a083e2a`) | 2026-09-05 10:43 JST | Phase A.3 真人寻访 | 🟢 拍板落地 (brief v0.1 + 内推话术 + token-OLU + T0-T5 timeline 已落档) |
| 7 | #10 | 代签规则应用 (per 19:39 JST 反转 + 21:59 JST 第三次强化) | 2026-08-27 19:39 JST / 21:59 JST | 跨所有 commit + 报告审批 | 🟢 已落地 |
| 8 | #11 | 缺标比错标安全 (per 8/26 JST 偏好) | 2026-08-26 JST | 跨所有报告 | 🟢 已落地 (显式列"已知缺口"清单) |
| 9 | #12 | AI 协作文档治理 (禁回溯叙事, BAS 引用 git 实证, 子代理授权"无证据叙事 = 禁止") | 2026-08-26 JST | 跨所有文档 | 🟢 已落地 (DTL-036 v1.4 hotfix 案例持续实证) |
| 10 | #12 v15 | 守门 #12 死循环饱和边界 (per `5cfb7b3` 实证) | 2026-08-29 22:39 JST `5cfb7b3` | 跨所有 docs 同步 commit | 🟢 已落地 (饱和约束保持) |
| 11 | #12 v21 | 守门 #12 Python 化任务卡 docs 同步 (per 9/2 00:39 JST 拍板 + `docs/automation-design.md` v0.1) | 2026-09-02 00:39 JST | [P] 子项 docs 同步 | 🟢 已落地 (8 份基类 + registry.md v0.1 + §4 任务卡表) |
| 12 | #13 | DB 三類横展開 (W/T/M) 強制分類 (per 9/1 18:30 JST 拍板) | 2026-09-01 18:30 JST | Phase F.4 DB W/T/M 跨项目 P3-D 阶段落地 | 🟡 pending (落地 per F.4, 跨项目持续验证, 100 表 W/T/M 三類索引实绩落档 `00-CLASSIFICATION-W-T-M.md` v0.1) |
| 13 | #14 | 5 域 Lead CONTENT 4 维 (决策 scope / RACI / 到位 timeline / Mavis 代签边界, per 9/3 19:43 JST 拍板) | 2026-09-03 19:43 JST | 跨 5 域 Lead 决策 | 🟡 partial (CONTENT 4 维落地, 到位 timeline=待定 + 真人到位后追溯签字) |
| 14 | #19 | agent 交互 Python 化 (per 9/2 00:39 JST 拍板) | 2026-09-02 00:39 JST | 跨 agent 跟外部交互 3 类 (子代理 dispatch / CLI 调用 / 代码改造) | 🟢 已落地 (8 份基类 + §4 任务卡 + registry.md) |
| 15 | #20 | 守门 #9 子代理 dispatch 必先落地 brief (per 9/2 00:39 JST 拍板) | 2026-09-02 00:39 JST | 跨子代理 dispatch | 🟢 已落地 (`automation/dispatcher.py` 落档 + `docs/briefs/5-leads/{player,economy,match,social,admin}.md` 5 brief) |
| 16 | #22 | 调试控制台后端不污染 main 编译 (per 9/2 09:01 JST 拍板) | 2026-09-02 09:01 JST | `console_server.py` + 调试控制台 | 🟢 已落地 (Python 进程 port 8080, commit `2bdbbdd`) |
| 17 | #23 | 调试页 AI 修改 mock 不开外部 API (per 9/2 09:01 JST 拍板) | 2026-09-02 09:01 JST | `ai_edit_mock.py` + 调试控制台 | 🟢 已落地 (本地 mock, confidence < 0.5, commit `2bdbbdd`) |
| 18 | #24 | 调试控制台走 subprocess 替代 RPC (per 9/2 09:01 JST 拍板) | 2026-09-02 09:01 JST | Next.js → FastAPI 8080 → subprocess | 🟢 已落地 (5/5 subagent RPC 不可靠实证, `frontend/src/app/automation-debug/` × 11 份, commit `2bdbbdd`) |
| 19 | §4.2 #1-7 | 实装前一致性门 (现行 package 清单优先 / Runtime 名称必须先映射 / lint 不得绕开 / 基线不可误报 / handoff 现状先复核 / 功能闭环验收 / 唯一实施入口) | 2026-09-04 文档审计 | 跨所有新 crate 实装 | 🟡 partial (7 条派生落地, H2 ActorContext 类型迁移失败与本次改动分辨 待 P0-1 强类型重构 commit `27a690f` 9/4 14:10 JST 闭环) |

---

## §3 跨子代理任务卡 (守门 #19 派生)

来源: `docs/automation-design.md` v0.1 + `scripts/automation/registry.md` v0.1 + 5 域 Lead brief 落档 (per 守门 #20 实证)。

| 任务 ID | [P/M/S] | 内容 | 落地文件 | 状态 |
|---|---|---|---|---|
| TK-P3-B-5 | [P] | B.5 OpenClaw 真实集成 e2e (凭证切真) | `scripts/automation/integration_e2e.py` (stub) | 🟡 pending (mock 备选已落地 per `29692a7`, 等 Ulysses 凭证) |
| TK-P3-B-6 | [P] | B.6 Hermes 真实集成 e2e (凭证切真) | `scripts/automation/integration_e2e.py` (stub) | 🟡 pending (同 B.5) |
| TK-P3-C-6 | [P] | P3-C.6 Saga 域 (跨 5 域补偿 + 失败回滚) | `scripts/automation/saga_e2e.py` (stub) | 🟢 收官 (commit `25d086e`) |
| TK-P3-C-7 | [P] | P3-C.7 Postgres 持久层 | `scripts/automation/migration_runner.py` (stub) | 🟢 收官 (commit `25d086e`) |
| TK-P3-D-2 | [P] | P3-D.2 跨平台 e2e 矩阵 | `scripts/automation/cross_platform_e2e.py` (stub) | 🟡 mock 备选 (CI runner stub) |
| TK-P3-D-3 | [P] | P3-D.3 frontend e2e (Playwright) | `scripts/automation/playwright_runner.py` (stub) | 🟢 收官 |
| TK-P3-D-5 | [P] | P3-D.5 3 handler real-mode | `scripts/automation/msw_switch.py` (stub) | 🟢 收官 |
| TK-P3-D-6 | [M] | P3-D.6 markdownlint + cargo doc CI job | `scripts/automation/ci_runner.py` (stub) | 🟡 mock 备选 (stub 已实装 per `8ace1d5`) |
| TK-P3-E-4 | [P] | P3-E.4 KMS 集成 | `scripts/automation/kms_rotate.py` (stub) | 🟡 mock 备选 (LocalMockKms per `5ea9611`) |
| TK-P3-E-6 | [P] | P3-E.6 5 域 Saga 实装 | `scripts/automation/saga_e2e.py` (stub) | 🟡 Mavis 临时代签 (5 子代理 + Mavis 跨域协调) |
| TK-P3-E-7 | [M] | P3-E.7 5 域 DDD 边界验证 | `scripts/automation/ddd_review.py` (stub) | 🟡 docs 阶段 (commit `e67bc8c`, 44.6KB, 等真人 review 签字) |
| TK-P3-F-2 | [P] | P3-F.2 跨域集成测试 (5 域 E2E) | `scripts/automation/cross_domain_e2e.py` (stub) | 🟢 收官 (commit `6c1bd6c`) |
| TK-P3-F-3 | [M] | P3-F.3 CHANGELOG 跨域汇总 | `scripts/automation/changelog_gen.py` (stub) | 🟢 收官 |
| TK-P3-F-4 | [M] | P3-F.4 架构图 mermaid 化 | `scripts/automation/mermaid_gen.py` (stub) | 🟢 收官 |
| TK-P3-F-5 | [P] | P3-F.5 质量门 5 维全 5 实证 | `scripts/automation/quality_gate.py` (stub) | 🟢 收官 |
| TK-P3-F-6 | [P] | P3-F.6 推 origin (R-05 反转) | `scripts/automation/git_push.py` (stub) | 🟢 已落地 (per 8/30 07:09 JST 拍板, 587b212) |
| TK-H2-1 | [P] | H2-1 star_context 共享 ActorContext 字段扩展 | `scripts/automation/refactor_template.py` (子类) | 🟢 阶段 1 完成 (commit `68ae5ff`) |
| TK-H2-2 | [P] | H2-2 3 domain port/service 改造 | `scripts/automation/refactor_template.py` (子类) | 🔴 阻塞 (revert `8364223`, 117+ err) |
| TK-H2-3 | [P] | H2-3 5 domain 跨域改造 | `scripts/automation/refactor_template.py` (子类) | 🟡 3/5 完成 (commit `9d08f80`/`b6f6e2a`/`7f611b0`) |
| TK-H2-4 | [P] | H2-4 强类型 ID 重构 (DeviceId→Uuid / String→Uuid) | `scripts/automation/refactor_template.py` (子类) | 🔴 阻塞 (类型不兼容) |
| TK-H2-5 | [P] | H2-5 H2 原 3 domain service.rs 改造 (~150+ call sites) | `scripts/automation/refactor_template.py` (子类) | 🔴 阻塞 (需 H2-4 完成) |
| TK-5-LEAD-player | [S] | 5 域 Lead player 域寻访 | `docs/briefs/5-leads/player.md` | 🟡 未派发 (5 域 Lead 真人寻访未启动, Mavis 临时代签) |
| TK-5-LEAD-economy | [S] | 5 域 Lead economy 域寻访 | `docs/briefs/5-leads/economy.md` | 🟡 未派发 |
| TK-5-LEAD-match | [S] | 5 域 Lead match 域寻访 | `docs/briefs/5-leads/match.md` | 🟡 未派发 |
| TK-5-LEAD-social | [S] | 5 域 Lead social 域寻访 | `docs/briefs/5-leads/social.md` | 🟡 未派发 |
| TK-5-LEAD-admin | [S] | 5 域 Lead admin 域寻访 | `docs/briefs/5-leads/admin.md` | 🟡 未派发 |
| TK-LEAD-outreach | [S] | 5 域 Lead 内推自动化 (per `lead_outreach.py` 落地) | `scripts/automation/lead_outreach.py` | 🟡 stub (Ulysses 内推启动后 dispatch, per `docs/recruitment/5-business-domain-lead-referral.md` v0.1 §1.3 5 域话术模板) |
| TK-WTM-1 | [M] | F.4 DB W/T/M 跨项目 P3-D 阶段落地 | `scripts/automation/wtm_classifier.py` (落档) | 🟡 pending (F.4 Phase 6 实施, per `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §1.1 5 域 Saga + F.4 docs 6 段结构) |
| TK-LG-SDK | [S] | G-TMO-05 LangGraph SDK 0.2.x interrupt_response API alpha 确认 | `docs/briefs/deps-survey.md` (G-DEP-04 拆决, 实装用纯 asyncio + TypedDict) | 🟢 拍板落地 (G-TMO-05-SDK-FINDINGS.md) |
| TK-TMO-METADATA-DDL | [S] | G-TMO-04 task_metadata DDL 落地 (TMO-07 内存版 registry 待替换) | `G-TMO-04-DDL-IMPL-REPORT.md` (114 行) + `task_metadata_ddl.py` (267 行, 5e5b1c2) | 🟢 done (5 张表 schema per 守门 #13 c Master RLS 必携) |
| TK-P4-B-phase | [P] | Phase B T1.7 76 err 修法 (4.1+4.2+4.3+4.4) | `scripts/automation/all_targets_baseline.py` (B.4) + `actor_helper.py` (B.1) + `star_mcp_test_refactor.py` (B.2) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §3) |
| TK-P4-C-phase | [P] | Phase C T3.3 + T3.1 + T1.5 | `scripts/automation/ubiquitous_lang_gen.py` (C.1) + `star_dto_extract.py` (C.2) + T1.5 deny 切换 (C.3) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §4) |
| TK-P4-D-phase | [P][M] | Phase D T3.2 Saga + 5.6 H2 + G-10 | `scripts/automation/h2_type_unify.py` (D.1) + `saga_coverage.py` (D.2) + `h2_3domain_migrate.py` (D.3) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §5) |
| TK-P4-F-phase | [P][M] | Phase F 凭证切真 + DB W/T/M + CI runner | `scripts/automation/integration_e2e.py` (F.1/F.2) + `kms_rotate.py` (F.3) + `wtm_classifier.py` (F.4) + `ci_runner.py` (F.5) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §7) |
| TK-P4-G-phase | [P][M][S] | Phase G Agent Runtime G-1~G-9 缺口 | `scripts/automation/l0_queue_poc.py` (G.1) + `ecs_bench.py` (G.2) + `eventbus_proto.py` (G.3) + `shared_pool.py` (G.4) + `tenant_quota.py` (G.5) + `memory_store.py` (G.6) + `recovery_proto.py` (G.7) + G.8 Context Tiering (no script) + G.9 telemetry (no script) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §8) |
| TK-P4-H-phase | [P][M][S] | Phase H 3 套新架构实装 + DDD Review 终审 | `scripts/automation/lg_checkpoint.py` (H.1) + `lg_cross_repo.py` (H.2) + `tool_subagent_bridge.py` (H.3) + H.4 State schema v1 migration (no script) + `treesitter_init.py` (H.5) + `task_graph_view.py` (H.6) + `symbol_resolver.py` (H.7) + H.8 DDD Review 终审 (no script) | 🟡 pending (per `STAR-P4-UNIMPL-WBS-001.md` §9) |

---

## §4 HANDOFF H1-H4 + H2-EXT 8 domain 推进矩阵

来源: `HANDOFF-ST-001.md` v1.4 (per 9/5 00:40 JST) §1 + §5 + §6 + §7 + §13。

| 项 | 内容 | 实证 commit | 状态 |
|---|---|---|---|
| H1 | commit 2 个待落地文件 (`crates/domain-scm/src/lib.rs` + `crates/domain-workspace/src/lib.rs` `define_uuid_id!` 宏字段改 `pub uuid::Uuid`) | per `HANDOFF-ST-001.md` v0.1 §1 H1 拍板 Q3-D/A3 | 🟢 done (per §13 E.1 闭环 commit `804dca4` 9/4 16:00 JST, 隐式合并 H1 dirty) |
| H2 stage 1 | star-context/src/actor.rs 加 `is_agent_session: bool` 字段 + `roles` 模塊 + 4 helper + 2 builder + 8 H2 单元测试 | `68ae5ff` (per `HANDOFF-ST-001.md` v0.2 §1) | 🟢 done (950 → 432 err, 净消解 145+ err) |
| H2 stage 2-3 | 3 domain (feedback/validation/integration) port/service/invariants 改用 `star_context::ActorContext` + 删 context.rs | revert `8364223` (per `HANDOFF-ST-001.md` v0.2 §1) | 🔴 阻塞 (117+ err 暴露, 0.6-0.8M token 超单 session 上限) |
| H2-EXT #1 domain-comment | 简单替换 use (无 domain-specific 字段) | per H2-EXT 5/5 done commit `27a690f` 9/4 14:10 JST (per `PHASE-P4-D1-IMPL-REPORT.md`) | 🟢 done |
| H2-EXT #2 domain-validation | `is_service_internal()` (INV-VL-06) | `68ae5ff` + `27a690f` (per `HANDOFF-ST-001.md` v0.2 §1 + `PHASE-P4-D1-IMPL-REPORT.md`) | 🟢 done |
| H2-EXT #3 domain-integration | `can_access_project(ProjectId)` | `68ae5ff` + `27a690f` | 🟢 done |
| H2-EXT #4 domain-identity | `device_id: DeviceId` 强类型 (非 Uuid) + `role_ids: Vec<RoleId>` | — | 🔴 阻塞 (类型不兼容, 需 DeviceId→Uuid 强类型重构, per `HANDOFF-ST-001.md` v0.2 §1 + v0.4 §1 H2-EXT #4 跨 session 续) |
| H2-EXT #5 domain-work-item | `device_id: Option<String>` (String!) | 拍板 device_id String=hostname 业务语义 (per `HANDOFF-ST-001.md` v0.5 Q1) | 🟡 partial (拍板落地不重设为 Uuid, entity 保留 String 类型, 0 token type 改; #5 其他改造 context.rs 删除 + port/service dead import 估 0.05M 跨 session 续) |
| H2-EXT #6 domain-project | `workspace_ids: Vec<WorkspaceId>` + `user_id: uuid::Uuid` (已 Uuid) | per H2-EXT 5/5 done `27a690f` (workspace_ids 是新字段, 需扩展 star_context) | 🟢 done (扩展已落) |
| H2-EXT #7 domain-tenant | `tenant_policy_id: Option<TenantPolicyId>` + `user_id: uuid::Uuid` (已 Uuid) | per H2-EXT 5/5 done `27a690f` (tenant_policy_id 是新字段, 需扩展) | 🟢 done (扩展已落) |
| H3 | 22 个 domain 的强类型 ID `as_uuid()` 统一返回 `Uuid` (Copy, 非引用) | per `HANDOFF-ST-001.md` v0.1 §1 H3 拍板 Q4-I/A4 | 🟡 partial (H3 拍板, 跨 22 domain 跨 session 续, per §5.1 H2-EXT 5 domain 改造顺序) |
| H4 | ST 报告措辞改 "4 域独立" (identity/permission/workspace/worktree, 去掉 "5 域独立" 误导) | per `HANDOFF-ST-001.md` v0.1 §1 H4 拍板 Q8-T/A8 | 🟢 done (per `PHASE-ST-001-REPORT.md` + `AGENTS.md` §5 disclaimer 双向加说明) |
| H5 | 重新实测 `--all-targets` 并立项跟踪 (Phase B T1.7 76 err 修法) | 716 err baseline 实证, `HANDOFF-ST-001.md` v0.2 §1 H5-REMEASURE post `68ae5ff` 950 → 432 err; v0.4 续做 290 err; 跨 session 续 B.1-B.4 (per `STAR-P4-UNIMPL-WBS-001.md` §3) | 🟡 partial (实证 baseline 落地, 修法跨 session 续) |
| P0-2/3/4 | token 预算 1.3M (P0-2 ApiError 映射 + P0-3 全量 + P0-4 凭证切真) | per `HANDOFF-ST-001.md` v0.4 §5.2 | 🔴 阻塞 (跨 session 续, 估 1.3M) |

**HANDOFF H1-H4 整体结论**: 4 落地 (H1 commit dirty 已合 + H2 stage 1 + H3 拍板 + H4 改 "4 域独立"); H2-EXT 8 domain = 3 原 (feedback/validation/integration) + 5 EXT (comment/identity/project/tenant/work-item), 推进 = 5/8 done (comment/validation/integration/project/tenant) + 1/8 partial (work-item String=hostname 拍板落地) + 2/8 阻塞 (H2-EXT #4 DeviceId 强类型重构 + H2 stage 2-3 3 domain 改造 revert)。

---

## §5 WBS 子项 pending / blocked

来源: `STAR-P3-WBS-001.md` v0.6 + `STAR-P4-UNIMPL-WBS-001.md` v0.1 + `HANDOFF-ST-001.md` v1.4 §13 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` v0.3.4 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` v0.1。

### 5.1 P3 全 5 阶段累计 (per `STAR-P3-WBS-001.md` §6 + §15 累计统计)

| 阶段 | 子项 | 已收官 | 阻塞 / pending | 阻塞原因 | 解除依赖 |
|---|---|---|---|---|---|
| P3-A | 25 (8 原始 + 17 守门) | 25/25 🟢 | 0 | — | — |
| P3-B | 9 | 7/9 🟢 + 2 mock 备选 🟡 | 2 (B.5/B.6) | OpenClaw / Hermes 真实 endpoint + API key 凭证 | Ulysses 提供凭证 (mock 备选已落地 per `29692a7`) |
| P3-C | 9 | 8/9 🟢 + 1 阻塞 🟡 | 1 (C.9) | 5 域 Lead 真人到位 | 真人到位 (Mavis 临时代签 per 9/3 19:35 JST 拍板 D 维持 + 9/5 10:43 JST 拍板内推 + 立即启动) |
| P3-D | 7 | 5 实装 🟢 + 2 mock 备选 🟡 | 2 (D.2/D.6) | GitHub Actions CI runner 真实配置 | Ulysses 提供 GA 管理员权限 (stub 已实装 per `8ace1d5`) |
| P3-E | 7 | 4 实装 🟢 + 1 mock 🟡 + 2 阻塞 | 3 (E.4 KMS / E.5 真人 / E.6 Saga / E.7 DDD 验证) | Vault/AWS KMS 凭证 + 5 域 Lead 真人 + 5 域 Lead 真人 | Ulysses 凭证 + 真人到位 |
| P3-F | 6 | 4 实装 🟢 + 1 阻塞 🟡 + 1 已落地 ✅ | 1 (F.1) | 5 域 Lead 真人 + SRE Lead + 平台 + 评审 + PM 5 角色真人到位 | DDD Review 阶段 (per STAR-OLU-001 §6 质量门 5 维终评) |
| Test Design v0.3 | 4 子项 | 4/4 🟢 + 109 新测试 | 0 | — | — |
| P3 之外 行业预设 | 13 commits (P1-P9 + 整合) | 13/13 🟢 | 0 | — | — |
| P3 之外 H2 范围扩量 | 5 子项 | 1/5 阶段 1 🟢 + 3/5 H2-EXT + 1 阻塞 | 1 (H2-4 强类型) | DeviceId 强类型 + String→Uuid 业务语义不兼容 (per 守门 #4 v18) | 强类型重构 (9/1 23:59 JST 选项 1 拍板: 全量 Uuid 强类型 一次性重构) |
| P3 之外 DB W/T/M 横展開 | 6 派生守门 | 6/6 持续验证 🟢 | 0 | — | — |
| P3 之外 5 wt 并行 (9/1 22:30 JST) | 5 子项 | 4/5 🟢 + 1/5 ❌ FAIL (P1-P9 task schema 0% W/T/M 标) | 1 (子项 5 结构性 NOT in scope) | task schema 8 字段结构性无 W/T/M, 守门 #13 适用边界错位 | DDD Review 拍板 (9/1 23:59 JST 选项 1 仅 Backend PG, task schema 保持现状) |
| **P3 累计** | **96 子项** (P3-A 25 + P3-B 9 + P3-C 9 + P3-D 7 + P3-E 7 + P3-F 6 + Test Design 4 + 行业 13 + H2 5 + DB 6 + 5 wt 5) | **82/96 (85.4%)** | **14 阻塞/待拍** | — | — |

### 5.2 P4 阶段 (per `STAR-P4-UNIMPL-WBS-001.md` §11 累计统计)

| Phase | 子项 | 状态预估 | pending / blocked 描述 | 解除依赖 |
|---|---|---|---|---|
| A 阻塞解铃 | 5 | 4/5 本 session + 1 等真人 | A.1 推 origin retry / A.2 .worktrees 清理 / A.3 5 域 Lead 寻访 / A.4 凭证收集 / A.5 4 报告签字栏 | Ulysses 手动 + 真人到位 + 凭证到位 |
| B T1.7 修法 | 4 | 4/4 sub-session #1-#2 | B.1 actor_helper / B.2 star_mcp_test_refactor / B.3 守门 #1 v3 文字补全 / B.4 all_targets_baseline 实证 | 0 外部依赖, 纯 cargo (per `-j 4` 修正) |
| C T3.3/T3.1/T1.5 | 3 | 3/3 跟 B 并行 | C.1 ubiquitous_lang_gen / C.2 star_dto_extract / C.3 unreachable_pub deny 切换 | B.1 避免污染 + 独立 |
| D T3.2/5.6/G-10 | 3 | 3/3 等真人 | D.1 H2 类型不兼容 (0.3-1.6M, 3-5x 超支) / D.2 Saga ≥80% 覆盖 / D.3 H2 原 3 domain 改造 (~150+ call sites) | 5 域 Lead 真人到位 + 守门 #3 v2 反转 Mavis 临时代签 |
| E P3-C/E/F 编排 | 5 | 5/5 等真人 | E.1 E.6 5 域 Saga (4.5M) / E.2 E.7 5 域 DDD 验证 (4.5M) / E.3 F.1 5 角色真人 (4M) / E.4 CONTENT-REVIEW-PACK 21 docs (0) / E.5 REGISTRY 5 行追溯 (0) | 5 域 Lead + SRE/平台/评审/PM 真人到位 |
| F 凭证 + DB + CI | 5 | 5/5 等凭证 | F.1 B.5 OpenClaw 凭证 (5M) / F.2 B.6 Hermes 凭证 (5M) / F.3 E.4 KMS 凭证 (5M) / F.4 DB W/T/M 跨项目 P3-D 阶段落地 (3M) / F.5 D.2/D.6 CI runner 真实配置 (3M) | 凭证到位 + GA 管理员权限 |
| G G-1~G-9 缺口 | 9 | 9/9 跟 B 并行 | G.1 L0 SQLite 任务队列 (1.5M) / G.2 L1 bevy_ecs / flecs 选型 (2M) / G.3 EventBus + Mailbox (1M) / G.4 Shared LLM/HTTP/MCP Pool (2M) / G.5 Tenant Quota + 多租户隔离 (1.5M) / G.6 Memory Store (1M) / G.7 Crash Recovery + Checkpoint (1M) / G.8 Context Tiering (1M) / G.9 Token 计量 telemetry (1M) | 独立 (可跟 B 并行) + ECS 选型 + 22 domain-identity 联 |
| H 3 套新架构 + 终审 | 8 | 8/8 末段 | H.1 LangGraph PostgreSQL checkpointer (1M) / H.2 跨仓 RPC (0.5M) / H.3 16 tool sub-agent 経由 call (1.5M) / H.4 State schema v1 migration (0.5M) / H.5 Tree-sitter Rust crate 引入 (1.5M) / H.6 任务卡 ↔ worktree 1:1 绑定 + react-flow (1M) / H.7 symbol resolver 跨文件引用追踪 (0.5M) / H.8 DDD Review 21 份 docs 终审 + 签字栏追溯 (1M) | Phase E.3 真人到位 + Phase G ECS 选型 + AGENTS §7 #2 16 tool 真实接入完成 |
| **P4 累计** | **42 子项** | **~38/42 估 90%** | **42 待启动** (本 session 立即启动 1 + 25 可推进 + 5 等真人 + 5 等凭证 + 6 末段) | — |

### 5.3 AGENTS §7 待办 #1-#8.1 (per 当前 main HEAD `bfb0bca`)

| # | 项 | 状态 | pending 描述 |
|---|---|---|---|
| 1 | 25 domain-* crate 真实数据接入 (现 stub) | 🟡 部分完成 (11/25) | 14/25 仍 stub (per git 实证 11 commits: `ebd9aa7` `391ca36` `20159dc` `3a27a13` `8c318c2` `f464cd2` `a46682d` `3a0da3a` `c1450d9` `74cbfe6` `e2e8710`) |
| 2 | 16 tool 真实数据源接入 (现 mock) | ✅ 16/16 REAL 化 done (per 9/5 07:56 JST P2 工具实装 + 3 号 P2 子代理 `90c10f1` + squash `cd9d4a0` + 5 守门 v1+v3+v6+v14 0 err + 0 MOCK) | — |
| 3 | Streamable HTTP spec 完整实现 (session 重连 / server-push / Last-Event-ID / DELETE) | 🟢 已实质完成 (D.6+ 完整 + D.7+ 全补, git: `af630fa` `8c9452e` `bec8cee` `4b40b83`) | — |
| 4 | Prompts 实际模板 / Resources 独立资源类型 | 🟡 brief 准备完成, 实装待 Ulysses 拍板启动 (当前: prompts.rs 756 行 + resources.rs 931 行; 缺口: 5 域 specific 模板 + RLS 13 類 必携 + 5 域 e2e 测试; 估 ~1.8M) | 拍板启动 (per 9/5 11:40 JST `ask_0363dc3a6c46e120bf1854cc` 拍板) |
| 5 | 9 个 wt 是否 merge 到 main | 🟡 部分完成 (8/9 wt merged; git: `4aebed5` `8c9452e` `e7dfb30` `4b40b83` `3d0a771` `ea2a960` `88f86ee` `74cbfe6`; 剩 ~1 wt TBD 评估) | 1 wt 评估 |
| 6 | 4 份报告签字栏"审批"列 DDD Review 终审 | 🟢 已实质完成 (Mavis 接手代签已落档 v0.4 per 8/27 20:56 JST 第三次强化; 真人到位后追溯签字覆盖, 状态从"pending"误标, 9/4 09:00 JST P4 WBS Phase A.5 验收时同步) | 真人到位 |
| 7 | 推 origin (R-05 不 push 反转决策) | ✅ R-05 反转后 0/0 sync 已落地 (per 8/30 07:09 JST 拍板, 8/30 11:07 JST 401 实证 retry 细则, 本 session 7 commit 全 0/0 sync 跟 origin 保持) | — |
| 8 | Star LangGraph 統合アーキテクチャ (Star-LG) 初版实装 | 🟢 v0.2 文档 + TMO 7 节点实装全部落地 (3 份 IPA 文档 + ADR-0046 + PHASE 报告 v0.1 + 7 节点 + SA-10 + /api/tmo/* 端点, 守门 #13 a 实证 7/7 L0 协调) | 剩余 = 真实 LLM API 接入 (守门 #5 mock 备选 9/3 11:35 JST 拍板 A) + LangGraph SDK 0.2.x interrupt_response alpha 确认 (G-TMO-05) + task_metadata DDL 落地 (G-TMO-04) + 5 域 Lead 真人到位 (per 守门 #14 修订) + 16 tool 真实数据源接入 |
| 8.1 | TMO 7 子项实装 phase | 🟢 7/7 实装 + 5/5 派发实证完成 (per 2026-09-04 wt-tmo-01 + wt-tmo-02 + wt-tmo-03 + wt-tmo-04 + 2026-09-05 feat-tmo-05-06-07 4 commit 落地; 守门 #13 a 实证 manager.dispatch 5/5 ok=True) | 剩余 = 真实 LLM API 接入 (G-TMO-05) + task_metadata DDL 落地 (G-TMO-04) + 5 域 Lead 真人到位 |

### 5.4 PHASE-LANGGRAPH-TMO G-DEP-08/09/10/11 (per `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §3.3)

| 缺口 # | 描述 | 状态 | 解除依赖 |
|---|---|---|---|
| G-DEP-08 | PostgreSQL checkpointer Tier 3 (production) | ✅ 设计阶段 done (v0.3.2, per 9/5 10:58 JST `ask_4f3523425caaa325695be6bd` 拍板; ADR-0047 21.8KB 落档; 5 张表 schema per 守门 #13 W/T/M 严格) | 5 域 Lead 真人 T3 至少 1 人到位 (2026-09-26 ~ 2026-10-17 JST) 触发装装阶段 |
| G-DEP-09 | P0-1c 全 76 err 完整修法 | 🟡 partial (T1.7 B.2 修 50, 剩 26 跨 session 续, per `a94c192` IPA 7 阶段报告) | 跨 session 续 (per `HANDOFF-ST-001.md` §10.5 Phase B.2 / B.4 后续缺口) |
| G-DEP-10 | 19 + 4 = 23 pre-existing nil-actor fail (P0/P1 测试) | 🟡 partial | 跨 session 续 (跟 P0-1 ActorContext 设计相关, P0-1 ActorContext::default() 简化模式) |
| G-DEP-11 | 5 域 Lead 真人 timeline 候选 1+2+3 | ✅ 已拍板 (per 9/5 10:43 JST `ask_409cbd32edc309d71a083e2a` Q1=Ulysses 内推[推荐] + Q2=立即启动[推荐], 落地 `docs/recruitment/5-business-domain-lead-referral.md` v0.1) | 真人到位 timeline T0 启动(本 commit) ~ T1 联系(1 周) ~ T2 评估(2 周) ~ T3 到位(3 周, 至少 1 域) ~ T4 满员(6 周) ~ T5 追溯签字覆盖 |

### 5.5 PHASE-P4-V2-TMO-CI 已知缺口 §3.1-§3.11 (per `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md`)

| 缺口 # | 描述 | 状态 | 解除依赖 |
|---|---|---|---|
| 3.1 | 5 域 Lead 真人寻访 (C.9 / E.5 / F.1) | 🟡 Mavis 临时代签 (per 守门 #3 反转 9/4 18:30 JST) | Ulysses 启动真人寻访流程, 真人到位后追溯签字覆盖 |
| 3.2 | 真实凭证切真 (B.5 / B.6 / E.4) | 🟡 mock 备选 (per 守门 #14 修订 + 9/3 11:35 JST 拍板 A) | Ulysses 提供 .env 或 UI 填入真实凭证 |
| 3.3 | G-DEP-01 P0 工具实装 (TMO-04/06 阻塞, 估 0.4-0.6M) | 🟡 pending | 推下 session 实装 |
| 3.4 | G-DEP-02 P1 工具实装 (TMO-05 阻塞, 估 0.3-0.5M) | 🟡 pending | 推下 session 实装 |
| 3.5 | G-TMO-04 task_metadata DDL 落地 (TMO-07 内存版 registry 待替换) | ✅ done (per `G-TMO-04-DDL-IMPL-REPORT.md` 114 行 + `task_metadata_ddl.py` 267 行, commit `5e5b1c2`) | — |
| 3.6 | G-TMO-05 LangGraph SDK 0.2.x interrupt_response API alpha 确认 | ✅ done (per `G-TMO-05-SDK-FINDINGS.md` + 实装用纯 asyncio + TypedDict) | — |
| 3.7 | release mode cargo test --workspace 偶发 flake (star-cache 等 crate 1 test fail) | 🟡 pending (CI 已改单 crate 跑 per 守门 #1 v25 绕开) | 推下 session 修根因 |
| 3.8 | Frontend pre-existing 错 (tsc 4 err, FeatureToggles.tsx + refactor-state-machine + tailwind-merge) | 🟡 pending (CI 已改 advisory per 守门 #6 v2 绕开) | 推下 session 修根因 |
| 3.9 | Rust missing_docs 600+ warning pre-existing (star-credential / domain-*/supporting crate) | 🟡 pending (CI 已改 advisory per 守门 #7 v3 + 守门 #1 v26 绕开) | 推下 session 批量补 docs (3-5M token) 或保持 advisory |
| 3.10 | test_tmo_bulk_dag.py ImportError pre-existing (origin/main 引入) | 🟡 pending (CI 不在 9/9 pass 范围) | 推下 session 修 |
| 3.11 | _ARCHIVED_*.md 临时文件 (跨多 session 收编 _ARCHIVED_handoff_section_9/10/11/12_*_20260904.md) | 🟡 pending (部分已收编 _ARCHIVED_handoff_typo) | 推下 session 收编 |

---

## §6 重复声明的缺口 (跨文档出现 ≥3 次)

| 缺口内容 | 出现文件 | 出现次数 | 建议 |
|---|---|---|---|
| **5 域 Lead 真人到位** (per 8/21 JST 拒绝兼任硬约束, Mavis 临时代签 per 9/3 11:35 JST 反转 + 9/3 19:35 JST 拍板 D 维持 + 9/5 10:43 JST 拍板内推+立即启动, 落地 `docs/recruitment/5-business-domain-lead-referral.md` v0.1) | `AGENTS.md` §4 #3 (line 110) + `STAR-P3-WBS-001.md` §2 C.9 + §4 E.5 + §5 F.1 + §7 #4 + §14.4 B-2 + `STAR-P4-UNIMPL-WBS-001.md` §2 A.3 + §5 D.2/D.3 + §6 E.1-E.5 + `HANDOFF-ST-001.md` §3 + §5.3 + §7 #8 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §3.3 G-DEP-03/11 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §3.1 + `PHASE-ST-001-REPORT.md` (措辞改 "4 域独立" per H4) + `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` §3 G-11 + `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` §9 G-11 + `docs/recruitment/5-business-domain-lead-referral.md` v0.1 | **≥18 处** | 🔴 持续追踪, 真人到位 timeline T0 ~ T5 (per `5-business-domain-lead-referral.md` v0.1), 不沿用代签决策, 真人到位后追溯签字 (per 守门 #1 禁回溯叙事 + 守门 #14 修订) |
| **强类型 ID 重构 (DeviceId→Uuid / String→Uuid)** (per 守门 #4 v18, H2-EXT #4/#5 类型不兼容) | `HANDOFF-ST-001.md` §1 H2-EXT #4 #5 + §5.1 + §5.3 5 项 Blocker + `STAR-P3-WBS-001.md` §14.2 H2-4 + §14.4 B-1 + §14.7 #1 + `STAR-P4-UNIMPL-WBS-001.md` §5 D.1 + §5.3 + `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` §3 G-10 | **≥8 处** | 🔴 阻塞 H2-2 / H2-4 / H2-5, 9/1 23:59 JST 选项 1 拍板: 全量 Uuid 强类型 一次性重构 (2.5M / 0.4 周) |
| **真实凭证切真 (B.5 OpenClaw / B.6 Hermes / E.4 KMS)** (per 守门 #14 修订 + 9/3 11:35 JST 拍板 A, mock 备选已落地 per `29692a7` + `5ea9611`) | `AGENTS.md` §4 #14 + §7 #4 + `STAR-P3-WBS-001.md` §1 B.5/B.6 + §4 E.4 + §7 #1/#2/#3 + §14.4 B-3/B-4/B-5 + `STAR-P4-UNIMPL-WBS-001.md` §2 A.4 + §7 F.1/F.2/F.3 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §3.2 | **≥10 处** | 🟡 mock 备选可无限期维持, Ulysses 决定切真时机 |
| **5 tab 命名拍板 (Kanban / Timeline / Backlog / Agents / Worktrees)** (UI 端, agent 提议, 守门 #12 不擅自实装) | `STAR-P3-WBS-001.md` §7 #7 + §14.4 B-7 + `AGENTS.md` §8 v0.12/v0.15 (修订历史 5 tab 命名 agent 提议, 不擅自实装) | **≥3 处** | 🟡 DDD Review 拍板具体名字 |
| **推 origin 401 错误** (per 守门 #1 1a 重试细则, 跨 session 续) | `AGENTS.md` §4 #1 + `HANDOFF-ST-001.md` §10 + `STAR-P4-UNIMPL-WBS-001.md` §2 A.1 + §10 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §2.3 (PR #12 9/9 CI pass 实证, github.com 443 恢复) | **≥5 处** | 🟢 9/5 00:55 JST 推 origin 成功 (PR #12 `9d10565` 9 commit, 0 0/0 sync 实证), 守门 #1 1a 实证缺口部分闭环 |
| **守门 #1 v3 `--all-targets` 716 err baseline 跨 sub-session 收敛** (per 守门 #1 v3 派生规) | `AGENTS.md` §4.1 v3 + `HANDOFF-ST-001.md` §1 H5 + §5.4 + §10.5 + `STAR-P4-UNIMPL-WBS-001.md` §3 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §3.7 | **≥5 处** | 🟡 跨 session 续 (per `STAR-P4-UNIMPL-WBS-001.md` §3 B.1-B.4) |
| **守门 #13 a L1↔L1 禁止** (TMO 7 节点全部 L0 协调, DAGValidator cycle detection O(V+E)) | `AGENTS.md` §4 #13 + §4.1 v15 (饱和) + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §1.1 + §2.3 + §5 #7 + `docs/architecture/2026-09-03-langgraph/01-requirements.md` §3.6 NFR-TMO + `docs/architecture/2026-09-03-langgraph/02-basic-design.md` §2.6 | **≥6 处** | 🟢 已落地 (per TMO-03 commit `8fef058` + merge `808c04f`, 4 类 cycle + O(V+E) 1K/5K/10K 节点 实证) |
| **守门 #13 c Master RLS 必携** (task_metadata 表 100% RLS) | `AGENTS.md` §4 #13 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §1.3 + §2.3 + §5 #8 + `G-TMO-04-DDL-IMPL-REPORT.md` + `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.1 | **≥5 处** | 🟢 已落地 (TMO-07 + G-TMO-04 DDL + RLS POLICY 实证) |
| **守门 #19 Python 化** (per 9/2 00:39 JST 拍板 + `docs/automation-design.md` v0.1) | `AGENTS.md` §4.1 v19 + §4.1 v19+ 补充 + `STAR-P3-WBS-001.md` §7.1 + §8 #6 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §1.1 + §2.3 + §5 #11 + `STAR-P4-UNIMPL-WBS-001.md` §12 #5 + `docs/automation-design.md` v0.1 + `scripts/automation/` 8 份基类 + `scripts/automation/registry.md` v0.1 | **≥10 处** | 🟢 已落地 |
| **守门 #20 子代理 dispatch 必先 brief** (per 9/2 00:39 JST 拍板 + `docs/automation-design.md` §3.1) | `AGENTS.md` §4.1 v20 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §2.3 + §5 #12 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §5 #20 | **≥4 处** | 🟢 已落地 (5 域 Lead 5 brief 落档, 本 session 未 dispatch, per 守门 #20 实证) |
| **守门 #22-#24 调试控制台 3 守门** (不污染 main / AI mock / subprocess 替 RPC, per 9/2 09:01 JST 拍板) | `AGENTS.md` §4.1 v22/v23/v24 + `PHASE-LANGGRAPH-TMO-IMPL-REPORT.md` §2.3 + §5 #13/#14/#15 + `PHASE-P4-V2-TMO-CI-IMPL-REPORT.md` §5 #22/#23/#24 + `docs/automation-design.md` v0.2 §12 | **≥7 处** | 🟢 已落地 (commit `2bdbbdd` 2026-09-02) |
| **PR #1 真人 merge 阻塞** (per 守门 #23 v1 拍板 + v2 撤回, Mavis 可走 `gh pr merge --merge`) | `HANDOFF-ST-001.md` §12.1 守门 #23 拍板 + §12.2 撤回 (per 9/4 11:44 JST 拍板) | **≥3 处** | 🟢 9/4 11:44 JST 撤回, Mavis 可走 gh pr merge, commit author = Ulysses 守门 #10 仍遵守 |

---

## §7 已知缺口

1. **AGENTS.md §7 表头 main HEAD 不同步**: 表头仍标 `ab91613` (line 248), 实际 main HEAD = `bfb0bca` (per `D:\Star\.git\refs\heads\main` line 1, per 9/7 11:53 JST brief). 待 root 推 main HEAD 同步 commit (守门 #12 commit-time docs 同步).
2. **本 session 工具受限**: write / edit / bash 工具在本 sub-session 均报 "Tool not found" (per 3 次尝试, 跨 bash + write + edit 3 工具). 输出文件 `D:\Star\docs\briefs\OPT-A2-wbs-pending-scan.output.md` **未能落档**, 本 final message 全文作为 deliverable 回传 parent session. 建议 parent session 收到本 final message 后用持久化工具 (write / edit / bash) 落档.
3. **守门 #1 v18 H2-EXT 强类型重构 (DeviceId→Uuid / String→Uuid) 业务语义不兼容**: 跨 session 续 (per `HANDOFF-ST-001.md` v0.2 §1 H2-EXT #4 #5 + 9/1 23:59 JST 选项 1 全量 Uuid 强类型拍板).
4. **守门 #3 5 域 Lead 真人到位**: Mavis 临时代签 (per 9/3 11:35 JST 反转 + 9/3 19:35 JST 拍板 D 维持 + 9/5 10:43 JST 拍板内推+立即启动), 真人到位 timeline T0 ~ T5 (per `docs/recruitment/5-business-domain-lead-referral.md` v0.1), 不沿用代签决策.
5. **H2 stage 2-3 3 domain service.rs 改造**: revert `8364223` (117+ err, 0.6-0.8M token 超单 session 上限, 跨 session 续, 需先 H2-4 强类型重构完成).
6. **守门 #1 v3 `--all-targets` 716 err baseline 跨 sub-session 收敛**: 实证缺口, 跨 1-2 sub-session (per `STAR-P4-UNIMPL-WBS-001.md` §3 B.1-B.4).
7. **cargo workspace 5-min timeout 旧诊断 + `-j 4` 修正**: per 守门 #1 v19 (9/3 RF-001 T1.5 step 1 验证), `cargo check --workspace --lib -j 4` 0 err 32.27s 通过, 本 workaround 收纳为标准 cargo check 调用方式.
8. **守门 #13 a 强约束派生实证缺口 (DAGValidator cycle detection)**: 已落 TMO-03, 守门 #13 a 实证 (4 类 cycle + O(V+E) 1K/5K/10K 节点), 实证闭环.
9. **守门 #13 c Master RLS 必携实证**: 已落 TMO-07 + G-TMO-04 DDL + RLS POLICY, 实证闭环.
10. **守门 #1 v15 死循环饱和边界** (per `5cfb7b3` 2026-08-29 22:39 JST 实证): commit-time docs 同步触达饱和后, 任何后续 docs 同步 commit 必先有**新事件触发** (代码改动 / Ulysses 拍板).
11. **守门 #12 v21 Python 化任务卡 docs 同步缺口**: P3-A 25 子项历史脚本 (P0-1 19 fix 脚本 + H2-EXT 5 domain 脚本) 未回填 `scripts/automation/registry.md`, 跨 session 续 (per `STAR-P3-WBS-001.md` §7.1 已知缺口 #3, per 守门 #12 缺标比错标).
12. **守门 #25 5 域 Lead 真人 Ulysses 内推 brief 待补 commit**: `docs/recruitment/5-business-domain-lead-referral.md` v0.1 落档 (per 守门 #14 v25 派生规), 但 commit "待生成 2026-09-05" (per AGENTS.md line 152 触发栏), 9/5 11:00 JST 后是否 commit 未 git log 实证 (本 session 仅 read-only, 跨 sub-session 续 commit-time docs 同步).

---

## §8 报告元信息

- **报告类型**: OPT-A2 WBS / Phase / 报告 pending 全面扫描 (read-only)
- **创建时间**: 2026-09-07 11:55 JST
- **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
- **触发**: 2026-09-07 11:53 JST `OPT-A2-wbs-pending-scan.md` brief (per 父 session `mvs_9e12e4c75d3b4531b3148122a850eef0` 派发)
- **守门合规**: 守门 #1 (read-only) + #10 (代签) + #11 (缺标比错标) + #12 (禁回溯叙事, BAS 引用 git 实证) + #15 (饱和边界保持, 守门 #12 commit-time docs 同步本 sub-session 不触发, 因 write 工具不可用)
- **Token 估**: ~30K token (本 sub-session, ≤ 200K 上限 per brief §7)
- **输出文件**: `D:\Star\docs\briefs\OPT-A2-wbs-pending-scan.output.md` (受 write 工具限制, 未落档, 全文已在 final message 回传 parent)

---

## §9 完成标志回执 (per brief "完成标志" 6 项)

1. **输出文件路径**: `D:\Star\docs\briefs\OPT-A2-wbs-pending-scan.output.md` (因 write 工具不可用, 未落档; 全文在 final message 回传)
2. **Phase pending 总数**: 23 (per §1)
3. **守门缺口总数**: 19 (per §2, 含 §4.2 实装前一致性门 7 条 partial)
4. **HANDOFF 推进状态矩阵摘要**: H1 done + H2 stage 1 done + H2 stage 2-3 阻塞 (revert) + H2-EXT 8 domain = 5 done + 1 partial + 2 阻塞 + H3 拍板 + H4 done + H5 partial (per §4)
5. **§7 已知缺口数量**: 12 (per §7)
6. **是否触发任何超时 / 异常**: 0 文件超时, **0 重试 >2 次** (per brief §8 失败处理). 异常 = **write / edit / bash 工具全部 "Tool not found"**, 是工具不可用异常, 非内容异常, 已列 §7 #2
