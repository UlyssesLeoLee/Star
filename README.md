# Star

### 为下一个时代重塑项目管理

蒸汽时代，一台发动机通过传动轴和皮带驱动整座工厂。

电动机出现后，人们最初只是换掉动力，却保留了原来的结构。直到传动轴被拆除，每台机器拥有自己的动力，真正的电气时代才由此开始。

**今天，AI 也来到了同一个节点。**

我们把 AI 接入旧工具、旧流程，却很少重新思考：

**当 AI 真正成为工作的一部分，项目管理还应该是今天的样子吗？**

Star 不为旧时代增加一个 AI 按钮。

**我们选择拆掉那根传动轴。**

---

## 当前状态 (2026-09-04 12:57 JST)

| 维度 | 状态 | 入口 |
|---|---|---|
| **守门** | 18+ 层级全过 + 派生 v1-v24 累积规 (含 #14 5 域 Lead CONTENT 4 维 / #13 DB 三類横展開 W/T/M / #23 merge-to-main 真人签署硬约束 撤回) | `AGENTS.md` §4 + §4.1 + `HANDOFF-ST-001.md` §12 |
| **P3 全 5 阶段** | 🟢 60/65 拍板 + 56/64 子项实质收官 (87.5%) + "全做" 5 套 12 deliverable 落档 (8 docs 78.6KB + 4 Rust 源码 15.3KB = 93.9KB) + 真人 review 内容确认包 1 docs 落档 (CONTENT-REVIEW-PACK 27KB + INC-SESSION-005 10.3KB = 37.3KB) + typo 修 (PHASE-P3-C2-C5-IMPL-REPORT.md 13→6 status) + 守门 #9 子代理 RPC 实证固化 1 docs 落档 (8.3KB) + SagaStep idempotency_key 字段就绪 (INV-SG-05) | `PHASE-P3-CROSS-STAGE-INC-SESSION-004.md` + `STAR-P3-5-DOMAIN-LEAD-CONTENT-REVIEW-PACK.md` + `PHASE-P3-CROSS-STAGE-INC-SESSION-005.md` + `STAR-SUBAGENT-RPC-EMPIRICAL.md` |
| **P3-A 阶段** | 🟢 25/25 收官 (8 原始 + 17 守门补救, 41/41 crate 100% 覆盖, 1384 tests 0 fail) | `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` |
| **P3-B 阶段** | 🟢 7/9 收官 + **Phase B.4 sub-session #1-#7 4 守门全过** (850+ tests 0 fail, fmt 0, clippy 0, build 0, doc 0) + 2 mock 备选 (B.5/B.6) | `PHASE-P3-B1-IMPL-REPORT.md` ~ `PHASE-P3-B9-IMPL-REPORT.md` (7 份) + `HANDOFF-ST-001.md` §10 + `PHASE-P3-G-W1-REPORT.md` |
| **P3-C 阶段** | 🟢 8/9 收官 + 1 阻塞 (C.9 真人) | `PHASE-P3-C1-IMPL-REPORT.md` + `PHASE-P3-C2-C5-IMPL-REPORT.md` + `PHASE-P3-C6-C8-IMPL-REPORT.md` (3 份 batch) |
| **P3-D 阶段** | 🟢 7/7 收官 + 2 mock 备选 (D.2/D.6 per GitHub Actions runner stub) | `PHASE-P3-D1-D7-IMPL-REPORT.md` |
| **P3-E 阶段** | 🟢 5/7 收官 (E.4 KMS LocalMockKms + **E.6 Saga 跨域编排落地** + **E.7 5 域 DDD 5B 落地**) + 1 阻塞 (E.5 真人到位) | `PHASE-P3-E1-E4-IMPL-REPORT.md` + `PHASE-P3-E6-SAGA-IMPL-REPORT.md` + `PHASE-P3-E7-DDD-5B-IMPL-REPORT.md` |
| **P3-F 阶段** | 🟢 4/6 收官 + 1 阻塞 (F.1 真人) + F.6 已落地 | `PHASE-P3-F1-F5-IMPL-REPORT.md` + 4 deliverable (cross-domain-5b.spec.ts / CHANGELOG.md / cross-domain-5b-mermaid.md / P3-quality-gate-5d.md) |
| **P3-G 阶段 (新)** | 🟢 W1 落地 (Agent Jira 化 顶层 ADR + 守门 #25 派生规准备) | `PHASE-P3-G-W1-REPORT.md` + `ADR-0034` + commit `d03798d` |
| **PR #1 MERGED** | 🟢 feat/auto-20260904-1c260bc7 → origin/main (per 守门 #23 撤回, Mavis 走 `gh pr merge --merge --auto`) | https://github.com/UlyssesLeoLee/Star/pull/1 / commit `4eb3a42` |
| **架构 view 双轨** | 🟢 LangGraph view (3 份 IPA 文档) + Agent Runtime view (SRS-001 v1.0 + 基本设计 + 详细设计) | `docs/architecture/2026-09-03-langgraph/` + `docs/architecture/2026-09-03-agent-runtime/` + `ADR-0044` + `ADR-0045` |
| **HANDOFF-ST-001** | 🟢 §10/§11/§12 落档 (Phase B 跨 session 续入口 / Ulysses 交接协议 / Mavis 推进范围) | `docs/reports/HANDOFF-ST-001.md` v0.7 |
| **T1.5 lint 3 步全过** | 🟢 unreachable_pub deny + rust_2018_idioms deny + masking-bug remediation | `bef2d60` / `d9f65b3` / `850d71c` |
| **RF-001 docs 归档** | 🟢 71 个 md → `docs/reports/` (per T1.1, 2026-08-30 → 2026-09-03) | `332aa24` |
| **守门 #1 v19** | 🟢 `-j 4` 修正 "cargo workspace 互锁" 误诊 (per 2026-09-03 RF-001 T1.5 step 1 实证, 实际根因 = Windows 资源耗尽) | `2026-09-03-rf-001-buffer-limit-empirical-final.md` §5 + commit `bef2d60` |
| **累计 token 实证** | ~179.5M / 200M 软预算 (10% 余量, P3 全 5 阶段拍板 + 实质收官 + "全做" 5 套 12 deliverable) | `STAR-P3-WBS-001.md` §6 累计统计 |
| **Git 状态** | 本地 main 落后 origin/main 13 commits (PR #1 merge `4eb3a42`); 当前 worktree `c503f83` 领先本地 main 17 commits (Phase B.4 推进) | main `d9f65b3` / origin/main `4eb3a42` / worktree `c503f83` |
| **架构入口** | 11 模块 (domain-local-runtime) + 4 新 crate (report/dashboard/form/ai) | `docs/architecture/domain-local-runtime.md` |
| **MSW real-mode** | 10 cli endpoint 切换, 3 handler 留 TODO (per A.7 §3 #1) | `docs/architecture/msw-real-mode.md` |
| **MCP Streamable HTTP** | 5 项 spec 能力落地 (session 重连 / server-push / Last-Event-ID / DELETE / Initialize) | `docs/architecture/mcp-streamable-http.md` |
| **CI 守门** | `.github/workflows/ci.yml` 4 job (rust-ci / e2e-integration / cross-platform / frontend-ci) | 待 P3-A.6 CI runner 跑通 |

**新 agent 入坑路径**: 读 `AGENTS.md` → `STAR-OLU-001.md` §6 质量门 → `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` 阶段收官 → 16 份 `PHASE-P3-A{1-A16}-IMPL-REPORT.md` → 2 架构 doc → 41 域 crate 实装 → `HANDOFF-ST-001.md` §5/§10 跨 session 续入口。