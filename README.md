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

## 当前状态 (2026-08-29 19:58 JST)

| 维度 | 状态 | 入口 |
|---|---|---|
| **守门** | 13+ 层级全过 + 派生 v1-v14 累积规 | `AGENTS.md` §4.1 + `STAR-OLU-001.md` §6 质量门 |
| **P3-A 阶段** | 🟢 25/25 收官 (8 原始 + 17 守门补救, 41/41 crate 100% 覆盖, 1384 tests 0 fail) | `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` |
| **P3-A 收官后 27 commits** | 🟢 cda49f3 / fcccdc2 / 66d6f8e / 42446aa / 90a9607 / f6c6533 (6 scope-ui-only) + 5b7475f / 7c54a39 / b483f33 / 1123c23 (4 docs 治理) + caae89e / d910164 / a59cfdd / 68ba2b1 / 5864286 / 2ba8966 / 24938f7 / 24e349b / 065c9e0 / 76cb4c9 / 4cba35c / 3ea4d91 / 24e349b / b0158e1 / e2e890e / 579f7e4 / 9145bcc / 85819f3 / 0f4386c / 29692a7 / ee09bf5 / 3ccfc1e / 7d85c34 (22 docs / 守门清理 / WBS 推进 / 5 tab 命名实装) | `PHASE-P3-A-INC-SESSION-001.md` + `PHASE-P3-A-INC-SESSION-002.md` 元汇总 |
| **P3-B-F 阶段** | 🟡 占位 + 6 阻塞项 + 1 备选 (B.5/B.6 mock 起步) | `STAR-P3-WBS-001.md` §0 表格 + §7 阻塞项 + §12.5 触发条件 |
| **累计 token 实证** | ~28.5M / 30M P3-A 软预算 (5% 余量, P3-A 实证) | `STAR-P3-WBS-001.md` §6 累计统计 |
| **Git ahead of origin/main** | 116 commits (per `git rev-list --count origin/main..HEAD`, 2026-08-29 23:03 JST) | main `7d85c34` |
| **架构入口** | 11 模块 (domain-local-runtime) + 4 新 crate (report/dashboard/form/ai) | `docs/architecture/domain-local-runtime.md` |
| **MSW real-mode** | 10 cli endpoint 切换, 3 handler 留 TODO (per A.7 §3 #1) | `docs/architecture/msw-real-mode.md` |
| **MCP Streamable HTTP** | 5 项 spec 能力落地 (session 重连 / server-push / Last-Event-ID / DELETE / Initialize) | `docs/architecture/mcp-streamable-http.md` |
| **CI 守门** | `.github/workflows/ci.yml` 4 job (rust-ci / e2e-integration / cross-platform / frontend-ci) | 待 P3-A.6 CI runner 跑通 |

**新 agent 入坑路径**: 读 `AGENTS.md` → `STAR-OLU-001.md` §6 质量门 → `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` 阶段收官 → 16 份 `PHASE-P3-A{1-A16}-IMPL-REPORT.md` → 2 架构 doc → 41 域 crate 实装。