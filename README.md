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

## 项目状态 (2026-09-15 JST)

### 核心进展

| 维度 | 状态 |
|------|------|
| **守门体系** | 18+ 层级全过，派生规则 v1-v24 累积 |
| **P3 全 5 阶段** | 60/65 拍板，56/64 子项收官 (87.5%) |
| **P3-A** | 25/25 收官 (41 crate, 1384 tests) |
| **P3-B** | 7/9 收官 + 2 mock 备选 |
| **P3-C** | 8/9 收官，1 阻塞 (C.9 真人) |
| **P3-D** | 7/7 收官 |
| **P3-E** | 5/7 收官，1 阻塞 (E.5 真人) |
| **P3-F** | 4/6 收官 |
| **P3-G** | W1 落地 (Agent Jira 化) |
| **架构** | LangGraph + Agent Runtime 双轨 |
| **CI** | 4 job 守门 (rust-ci, e2e, cross-platform, frontend) |

### 部署方式

**k3s (本地 Kubernetes)**
- Kustomize 清单已就绪：`deploy/k3s-local/kustomization.yaml`
- 包含：envoy proxy、star-api-rest、bff runtime
- 验证方式：`kubectl kustomize deploy/k3s-local/`
- 启动脚本：`deploy/k3s-local/build-and-deploy.sh`、`maintenance/start-k3s-backend.ps1`
- 注意：需 WSL + k3s 环境，当前环境未验证实际启动

**Helm (可选)**
- 模板位置：`deploy/helm/star/templates/`

### Git 分支

| 分支 | 说明 |
|------|------|
| `main` | 生产分支 |
| `dev` | 开发分支，基线 `45a2f820` |

---

### 技术细节 (供工程师查阅)

- **入坑路径**: `AGENTS.md` → `STAR-OLU-001.md` §6 → `PHASE-P3-A-PHASE-CLOSEOUT-REPORT.md` → 16 份 impl 报告 → 41 域 crate
- **架构入口**: 11 模块 (domain-local-runtime) + 4 新 crate (report/dashboard/form/ai)
- **MSW real-mode**: 10 cli endpoint 已切换
- **MCP Streamable HTTP**: 5 项 spec 能力落地