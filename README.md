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

## 当前状态 (2026-08-30 08:51 JST)

| 维度 | 状态 | 入口 |
|---|---|---|
| **守门** | 13+ 层级全过 + 派生 v1-v15 累积规 | `AGENTS.md` §4.1 + `STAR-OLU-001.md` §6 质量门 |
| **P3 全 5 阶段** | 🟢 60/65 拍板 + 55/63 子项实质收官 (87.3%) | `PHASE-P3-CROSS-STAGE-INC-SESSION-003.md` 元汇总 |
| **P3-A 阶段** | 🟢 25/25 收官 (8 原始 + 17 守门补救, 42/42 crate 100% 覆盖, 1384 tests 0 fail) | `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` |
| **P3-B 阶段** | 🟢 7/9 收官 + 2 mock 备选 (B.5/B.6 per 29692a7 wiremock) | `PHASE-P3-B1-IMPL-REPORT.md` ~ `PHASE-P3-B9-IMPL-REPORT.md` (7 份) |
| **P3-C 阶段** | 🟢 8/9 收官 + 1 阻塞 (C.9 真人) | `PHASE-P3-C1-IMPL-REPORT.md` + `PHASE-P3-C2-C5-IMPL-REPORT.md` + `PHASE-P3-C6-C8-IMPL-REPORT.md` (3 份 batch) |
| **P3-D 阶段** | 🟢 7/7 收官 + 2 mock 备选 (D.2/D.6 per GitHub Actions runner stub) | `PHASE-P3-D1-D7-IMPL-REPORT.md` |
| **P3-E 阶段** | 🟢 4/7 收官 + 1 mock (E.4 KMS per LocalMockKms) + 3 阻塞 (E.5/E.6/E.7) | `PHASE-P3-E1-E4-IMPL-REPORT.md` |
| **P3-F 阶段** | 🟢 4/6 收官 + 1 阻塞 (F.1 真人) + F.6 已落地 | `PHASE-P3-F1-F5-IMPL-REPORT.md` + 4 deliverable (cross-domain-5b.spec.ts / CHANGELOG.md / cross-domain-5b-mermaid.md / P3-quality-gate-5d.md) |
| **累计 token 实证** | ~179.5M / 200M 软预算 (10% 余量, P3 全 5 阶段拍板 + 实质收官) | `STAR-P3-WBS-001.md` §6 累计统计 |
| **Git ahead of origin/main** | 0 commits (per `git rev-list --count origin/main..HEAD`, 2026-08-30 08:51 JST) | main `93512a9` |
| **架构入口** | 11 模块 (domain-local-runtime) + 4 新 crate (report/dashboard/form/ai) | `docs/architecture/domain-local-runtime.md` |
| **MSW real-mode** | 10 cli endpoint 切换, 3 handler 留 TODO (per A.7 §3 #1) | `docs/architecture/msw-real-mode.md` |
| **MCP Streamable HTTP** | 5 项 spec 能力落地 (session 重连 / server-push / Last-Event-ID / DELETE / Initialize) | `docs/architecture/mcp-streamable-http.md` |
| **CI 守门** | `.github/workflows/ci.yml` 4 job (rust-ci / e2e-integration / cross-platform / frontend-ci) | 待 P3-A.6 CI runner 跑通 |

**新 agent 入坑路径**: 读 `AGENTS.md` → `STAR-OLU-001.md` §6 质量门 → `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` 阶段收官 → 16 份 `PHASE-P3-A{1-A16}-IMPL-REPORT.md` → 2 架构 doc → 41 域 crate 实装。