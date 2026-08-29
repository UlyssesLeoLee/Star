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

## 当前状态 (2026-08-29)

| 维度 | 状态 | 入口 |
|---|---|---|
| **守门** | 7 层级全过 | `AGENTS.md` §4 + `STAR-OLU-001.md` §6 质量门 |
| **P3-A 阶段** | 🟢 17/17 收官 (8 原始 + 8 守门补救 + 1 阶段收官) | `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` |
| **P3-B-F 阶段** | 🟡 占位 + 7 阻塞项 | `STAR-P3-WBS-001.md` §0 表格 + §7 阻塞项 |
| **累计 token 实证** | ~27.1M / 30M 软预算 (10% 余量) | `STAR-P3-WBS-001.md` §6 累计统计 |
| **架构入口** | 11 模块 (domain-local-runtime) + 4 新 crate (report/dashboard/form/ai) | `docs/architecture/domain-local-runtime.md` |
| **MSW real-mode** | 10 cli endpoint 切换, 3 handler 留 TODO (per A.7 §3 #1) | `docs/architecture/msw-real-mode.md` |
| **CI 守门** | `.github/workflows/ci.yml` 4 job (rust-ci / e2e-integration / cross-platform / frontend-ci) | 待 P3-A.6 CI runner 跑通 |

**新 agent 入坑路径**: 读 `AGENTS.md` → `STAR-OLU-001.md` §6 质量门 → `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` 阶段收官 → 16 份 `PHASE-P3-A{1-A16}-IMPL-REPORT.md` → 2 架构 doc → 41 域 crate 实装。