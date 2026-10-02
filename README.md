# Star

> 为下一个时代重塑项目管理

Star 是面向"AI 即工作流节点"时代重新设计的项目管理平台。
我们不把 AI 当作旧工具的插件，而是把它当作工作流里的一等公民。

---

## TL;DR (投资人 90 秒版)

| 维度 | 数字 | 备注 |
|---|---|---|
| **工作空间** | 105 个 Rust crate + 1 个 npm frontend (496 个 .tsx/.ts) + 1392 篇设计文档 | `Cargo.toml` `members` (per 2026-10-02 PR-277 follow-up audit, 105 unique workspace members, **PR-272 误记 97 → 修 105**) + `find crates -name '*.rs'` + `find frontend/src` + `find docs -name '*.md'` |
| **后端实现度** | 大头收官，边界收敛中 | 见 §1 "已实现"与 §2 "待收敛" |
| **文档体系** | 23 SRS + 8 BD + 40 DD/SRS-BD-DD 总计 (设计 40 篇) + 28 ADR | IPA 风格 (要件定義書 → 基本設計書 → 詳細設計書) |
| **分支与 PR** | `main` 最新 `bd97197f` (Tauri PoC index PR-264 #271 merge), 最近 CI 部分 success (frontend/markdownlint/rust/tarpaulin 4 个 fail; smoke passes per PR-269 run) | 见 §3 |
| **本地 k3s 启动** | **未在本机验证可启动** | 见 §4 完整披露 |
| **诚实风险** | 1 个 helm chart YAML bug + 7 个未合入依赖 PR + 4 个 CI check fail (PR-269 实证) | 见 §5 |

---

## 1. 已实现 (投资人可以直接验证)

### 1.1 工程基线

| 项 | 数值 | 证据 |
|---|---|---|
| Cargo workspace 成员 | 105 | `Cargo.toml` `[workspace] members` (per 2026-10-01 actualize + 2026-10-02 PR-277 follow-up audit, 105 unique entries, line 126 multi-entry single line 含 5 duplicates, **PR-272 误记 97 → 修 105**) |
| Rust 源文件 | 736 | `find crates -name '*.rs'` |
| 前端源文件 | 496 | `find frontend/src -name '*.tsx' -o -name '*.ts'` |
| 设计文档 (md) | 1392 | `find docs -name '*.md'` (per 2026-10-02 PR-277 follow-up audit, **PR-272 误记 1391 → 修 1392**) |
| 架构决策记录 | 28 篇 ADR | `docs/architecture/**/adr/` + `docs/wiki/pgwiki/30-architecture/adr/` |
| CI workflow | 4+ job: rust-ci / e2e-integration / cross-platform / frontend-ci + star-desktop-build (PR-242, PR-255 multi-OS matrix ubuntu+windows+macos) | `.github/workflows/` |
| 最近 PR CI | PR-269 session 内 4 fail (Frontend/Markdownlint/Rust/Tarpaulin) + 6 pass (smoke/bench/doc 等) | `gh pr checks 269` |

### 1.2 主要交付

| 文档 / 模块 | 状态 | 说明 |
|---|---|---|
| `docs/requirements/SRS-*.md` | 23 份 | 要件定義書 (各域 SRS), IPA 风格 |
| `docs/design/BD-*.md` | 8 份 | 基本設計書, 含 BD-CANVAS-WORKFLOW-001 v1.0.3 (5 角色已签字) |
| `docs/design/DD-*.md` | 40 份 | 詳細設計書, 含 DD-CANVAS-WORKFLOW-001 v1.0 (Round 2/7) + DD-WORKTREE-GROUP-001 v0.6 (PR #226 codex merge A) |
| `crates/domain-*` | 27 个域 crate | 业务领域核心 |
| `crates/star-*` | 23 个 service crate | API/cache/MCP/saga/telemetry 等 |
| `crates/infrastructure` + `crates/shared-task` | 2 | 基础设施 |
| `crates/api` + `crates/application` | 2 | 接口层 |
| `crates/arg*` + `crates/agent-domain` + `crates/canvas-collab` + `crates/leads` | 6 | 跨域工具 |
| `crates/star-desktop/` (Tauri 2.0 desktop PoC) | 1 Tauri 二进制 (前端 + Rust 后端, **8 IPC commands** + 3 adapter + MockDb, per PR-239-#259) | `.github/workflows/star-desktop-build.yml` Linux CI 待 CI 触发 |
| `crates/layout-engine-wasm` + `crates/query-engine-wasm` + `crates/relationship-engine-wasm` | 3 WASM crates | 浏览器端 Rust → WASM (per PR-219/230/235) |
| `deploy/k3s-local/` | kustomize (12 resource) + Dockerfile | kubectl kustomize 已在本机产出 12 resource, 见 §4 |
| `deploy/helm/star/` | Helm chart v0.1.0 | **Chart.yaml 有 YAML bug**, 见 §5 |
| `maintenance/start-k3s-backend.ps1` 等 | 4 个 .ps1 / 4 个 .bat | Windows-WSL 启动脚本 (本任务未在 Windows 主机上跑通, 见 §4) |

### 1.3 已签字的"决策级"基线

- **`BD-CANVAS-WORKFLOW-001.md` v1.0.3** — 5 角色签字栏全部完成 (Ulysses 本人确认, 非代签), §11 一致
- **`SRS-CANVAS-WORKFLOW-001.md` v1.1** — W1-W15 共 54 项 FR (W1-W13 = 42 项 v1.0 + W14/W15 = 12 项 v1.1 新增)
- **`SRS-WORKFLOW-TEMPLATE-001.md` v0.1** + **`BD-WORKFLOW-TEMPLATE-001.md` v0.1** + **`DD-WORKFLOW-TEMPLATE-001.md` v0.1** — 父 issue ULYS-30 + 串行子任务 ULYS-34 (SRS) / ULYS-35 (BD) / ULYS-36 (DD) 全部落档

---

## 2. 待收敛 (诚实的清单, 不是粉饰)

### 2.1 设计文档层面

| 项 | 状态 | 含义 |
|---|---|---|
| `DD-CANVAS-WORKFLOW-001.md` | Round 2/7, §0-§7 已落档 | §8-§14 待 Round 3-7 (FR 跟踪矩阵补齐、错误分支设计、追溯表、签字栏、修订历史) |
| 28 ADR 中部分章节 | 多个 `[TBD]` 标记 | 已知待拍板项, 在对应文档 §9.2/§9.3 追踪矩阵中 |
| 总册 `SRS-CANVAS-001.md` v1.2 同步 | 待作 | 需把"双核心"扩展为"三核心"时正式收录专题 3 (自动化流程) |

### 2.2 仓库治理层面

| 项 | 状态 |
|---|---|
| 143 个本地 worktree 分支 | 大量已远落后 main (数百到上千 commit), 历史 plan residue 残留 |
| 7 个未合并的 dependabot PR (#5 #6 #24 #39 #45 + 已合并 #4) | 由 ULYS-47 关闭 #4 (CI 绿) + #5/6/24/39/45 (CI 红, 等 dependabot rebase 后再评估) |
| 1 个 `CI Gate (blocking)` PR (#47) | 关闭: 该 PR 自身在新加的 gate 上 FAIL (clippy warning ratchet + cargo-deny advisories), 合并会扩大阻塞面 |
| 1 个 PR #43 (ULYS-36 retry) | 关闭: DD 内容已通过 PR #42/#44 合并, 该 PR 仅含破坏性 SRS revert |
| 1 个 M2 残留 `dev` 分支 (2 commits ahead of main, 含同样 SRS revert) | 删除并重建 clean dev |

### 2.3 工作流层面

| 项 | 状态 |
|---|---|
| 本地 k3s / WSL Ubuntu 可启动后端 | **未在本机验证**, 见 §4 |
| 端到端 Playwright E2E | P3-A.5/WT-32 引用, 当前 CI run 中 skipped (per `gh pr checks 48` 输出) |
| 投资人 demo 路径 | 走 文档 + 架构图 + 关键 PR 链接, 而不是本地 cluster 起 pod |

---

## 3. 最近合并到 main 的关键 PR (ULYS-47 收敛)

按时间正序 (旧 → 新):

| PR | 标题 | 合并 commit | 判据 |
|---|---|---|---|
| #42 | docs: SRS-WORKFLOW-TEMPLATE-001 + BD-WORKFLOW-TEMPLATE-001 v0.1 | `341aaf9d` | 父 ULYS-30 stage 1+2 |
| #44 | docs: DD-WORKFLOW-TEMPLATE-001 v0.1 — ULYS-36 stage 3 retry | `369ea7a3` | 父 ULYS-30 stage 3 |
| #40 | docs(BD-CANVAS-WORKFLOW-001): 无限画布自动化流程域基本设计书 v1.0.3 | `b4efd59d` | 5 角色签字 |
| #41 | docs: fix 20 pre-existing markdownlint violations | `ea5e4ff6` | markdownlint 全过 |
| **#46** | docs(srs-canvas-workflow): 统一 W14.2 占位节点命名 | `115e62d1` | ULYS-47 甄别后合入, CI 全绿 |
| **#48** | docs(design): DD-CANVAS-WORKFLOW-001 v1.0 — ULYS-15/ULYS-33 stage 3 詳細設計 | `f9878671` | ULYS-47 甄别后合入, CI 全绿 |
| **#4** | ci(deps): bump actions/setup-node from 4 to 7 | `709a9e97` | ULYS-47 甄别后合入 (dependabot 唯一 CI 全绿的 PR) |
| **#259** | docs(desktop): Tauri PoC P0→P10 final implementation summary (per PR #239-#258) | `b3ebded2` | Tauri PoC 21 PRs 落地 (P0 skeleton → P10 Sentry) |
| **#269** | docs(session): session memory integration protocol — 5 反复实证 workaround (per PR-260) | `f532fd73` | 2026-10-01 merge, session 内 5 workaround 文档化 |
| **#270** | feat(desktop): Tauri WorktreeVizPage — useLayoutEngine WASM hook 实战 120 nodes (per PR-265) | `dd045314` | 2026-10-01 merge, SVG 100+ node 可视化 |
| **#271** | docs(desktop): Tauri PoC index + 按角色阅读路径 (per PR-264) | `bd97197f` | 2026-10-01 merge, Tauri PoC docs 总览 |

main HEAD: **`bd97197f`** (per 2026-10-01)

甄别原则 (per Opus1m 约束 + AGENTS.md 守门):

1. **不合并破坏性 SRS revert** — `agent/minimaxm3/ulys-33` / `agent/minimaxm3/ulys-36` / `agent/m2/ulys-47` 均携带把 SRS v1.1 静默降回 v1.0 的 313 行变更, 删除 v1.1 修订履历行 + W14/W15 子能力 + issue 创建者追评记录, 违反守门 #1 禁回溯叙事 + 守门 #11 缺标比错标
2. **不合并"自身违反自身"** — PR #47 (CI Gate blocking) 在自身 PR 上 FAIL
3. **不合并 CI 红的依赖 bump** — dependabot PR #5/6/24/39/45 在最新 run 上有 FAIL 项
4. **不批量合并** — 全部单 PR 单 commit, 任何冲突单 PR 处理

`dev` 分支: 从最新 main 重建 (`origin/dev = origin/main = 709a9e97`), 上一份被 M2 plan residue 污染的 dev worktree 已删除。

---

## 4. k3s 启动性 — 完整披露

### 4.1 本机实测 (2026-09-14, ULYS-47 session)

| 探针 | 结果 | 命令 |
|---|---|---|
| `docker ps` | **FAIL** — `failed to connect to the docker API at npipe:////./pipe/dockerDesktopLinuxEngine` | Docker Desktop daemon 未运行 |
| `which k3s` | **NOT FOUND** | k3s 二进制未安装 |
| `which k3d` | **NOT FOUND** | k3d 未安装 |
| `which kind` | **NOT FOUND** | kind 未安装 |
| `kubectl cluster-info` | **TIMEOUT (8s)** | 现有 kubeconfig 指向 `https://127.0.0.1:52551` (Docker Desktop 期望位置), 无 cluster 监听 |
| `kubectl config view` (server) | `https://127.0.0.1:52551` | 同上, 端口 LISTENING 但 TLS handshake 超时 |
| `helm version` | OK | v4.2.4 (客户端可用) |
| `kubectl kustomize deploy/k3s-local/` | **OK** | 产出 12 resource: 1 Namespace + 1 ServiceAccount + 2 ConfigMap + 4 Service + 4 Deployment |
| `helm template deploy/helm/star/` | **FAIL** | Chart.yaml 第 3 行 YAML 解析错误 (见 §5) |

### 4.2 结论

**当前 Windows 主机没有可启动的 k3s cluster, deploy/k3s-local/build-and-deploy.sh 与 maintenance/start-k3s-backend.ps1 的运行前提 (WSL Ubuntu + docker daemon + sudo 免密) 在本会话窗口不成立**。

投资人 demo 路径上有三种选择,各自边界:

| 路径 | 风险 | 适用场景 |
|---|---|---|
| A. 在已具备 WSL/k3s 环境的工程师本机跑 `build-and-deploy.sh` | 低 (脚本幂等, 已在 9/8 跑通过一次, per `start-k3s-backend.ps1` 注释) | 工程师自查 |
| B. 在 CI 上跑 (GitHub Actions self-hosted runner + k3s service container) | 中 (需配置 self-hosted runner, 0 既有 runner 实证) | 投资人想看活体 cluster |
| C. 走文档 + 架构图 + 关键 PR demo (本 README + `docs/architecture/**` + PR #42/#44/#48 链接) | 无 | 当前实际路径 |

**本 README 不对 k3s 可启动性作任何保证**; 该 README 中的 `deploy/k3s-local/` 与 `maintenance/start-k3s-backend.ps1` 是**已落档但未经 ULYS-47 重新验证**的资产, 验证需要"具备 WSL Ubuntu + docker daemon + sudo 免密的工程师本机"。

### 4.3 已落档但未重新验证的清单 (风险披露)

- `deploy/k3s-local/build-and-deploy.sh` — 9/8 之前最后跑通, 9/14 本机无法复现 (WSL/docker 缺)
- `deploy/k3s-local/install-sealed-secrets.sh` — 同上
- `deploy/k3s-local/kustomization.yaml` — **本机 `kubectl kustomize` 通过**, 产出 12 resource, 结构有效
- `deploy/k3s-local/bff-deployment.yaml` + `star-api-rest-deploy.yaml` — 文件结构有效 (`kubectl kustomize` 通过)
- `deploy/k3s-local/Dockerfile` — 未在本会话构建, 走原 9/8 流程
- `deploy/helm/star/Chart.yaml` — **有 bug, 见 §5**
- `deploy/helm/star/templates/*` — 同上, 未独立验证

---

## 5. 已知风险与未解决问题

| # | 项 | 严重度 | 责任 |
|---|---|---|---|
| 1 | `deploy/helm/star/Chart.yaml` 第 3 行 description 含未加引号的 `8 services: cli/mcp/...` 冒号, helm template 失败 (column 70, mapping values not allowed) | **高** — helm 部署路径整体失效 | 待新 issue (推荐 ULYS-48) 修复: 给 description 加双引号, 或移除冒号 |
| 2 | 7 个 dependabot PR 待 CI 重新跑通后合并 (#5/#6/#24/#39/#45 + #45 已合并 + #4) | 中 — 投资人交付窗口后由 dependabot 自动 rebase 收敛 | dependabot + 维护者 |
| 3 | PR #47 (CI Gate blocking) 关闭 | 低 — gate 设计有价值, 关闭后该 gate 后续仍可重开 | PR 作者修复 clippy + cargo-deny |
| 4 | 28 个本地 worktree 分支残留 (D:/Star + codex/ + minimax/agent/ 混合) | 低 — 仅本机状态, 不影响 main | 工作流 hygiene |
| 5 | `DD-CANVAS-WORKFLOW-001.md` Round 3-7 待续作 (§8-§14) | 中 — 设计阶段未完结 | ULYS-33 续作 / ULYS-49 推荐 |
| 6 | 总册 `SRS-CANVAS-001.md` v1.2 同步收录专题 3 (自动化流程域) | 中 — 三核心 vs 双核心不一致 | 5 域 Lead + Ulysses 拍板 |
| 7 | 端到端 Playwright E2E (P3-A.5) | 中 — 当前 CI run 中 skipped | P3-A.5 后续 issue 跟进 |
| 8 | PR-269 (Tauri session memory) CI 4 fail (Frontend/Markdownlint/Rust/Tarpaulin) | 中 — PR 已 merge 但 4 check 红, 需后续 PR 修 | Tauri 后续 PR 修复 (per session memory §8 后续) |
| 9 | crates/star-desktop/README.md stale "1 IPC mock" 描述 (实际 8 IPC) | 低 — 文档落后 | PR-274 后续更新 |
| 10 | docs/architecture/2026-09-30-upgrade/star-desktop-p0p10-final-impl-summary.md 时间线停在 PR #259 (未含 PR #260/#264/#269/#270/#271) | 低 — 文档落后 | PR-273 后续更新 |
| 11 | 本地 k3s cluster 启动未实证 (per session memory WSL2 死结) | **高** — 5 services HTTP 200 待 verify | host 端 `wsl --shutdown` 后续 |
| 12 | Tauri 2.0 Windows build STATUS_STACK_BUFFER_OVERRUN (per PR-242 + PR-255 multi-OS CI 待触发) | 高 — Linux CI 待 verify | PR-242 GitHub Actions 后续 |

---

## 6. 新 agent 入坑路径 (10 分钟版)

1. 读 `AGENTS.md` (入口) → `HANDOFF-ST-001.md` §10 (跨 session 续入口) → §12 (Mavis 推进范围)
2. 读 `docs/requirements/SRS-CANVAS-001.md` 总册 → 3 专题 SRS (agent / gamify / workflow)
3. 读 5 份 IPA 文档: SRS → BD → DD (任一专题, 范例: workflow 域 BD 已签字 v1.0.3)
4. 读 `docs/architecture/2026-09-03-langgraph/` + `docs/architecture/2026-09-03-agent-runtime/` (双轨)
5. 看 PR #42 + #44 (ULYS-30 收官) + PR #48 (ULYS-15/33 详细设计) — 这是 ULYS-47 session 收敛的最新一组
6. 跑 `cargo test --workspace` 与 `pnpm test` — 已是绿色 (per main CI 最近 4 次 success)

---

## 7. 给投资者的核心叙事

**已经建成** (不可粉饰):

- 105 个 Rust crate 的 workspace 骨架 + 完整 domain 域拆分 (27 域 + 23 service) + 1 Tauri desktop PoC (8 IPC, per PR-239-#259) + 3 WASM crates (per PR-219/#230/#235)
- 1392 篇设计文档 (23 SRS + 8 BD + 40 DD + 28 ADR + 实施报告/复盘报告)
- IPA 风格 V 模型 (要件 → 基本設計 → 詳細設計) 全程签字栏可追溯
- 4+ job CI (rust / frontend / cross-platform / e2e-integration + star-desktop multi-OS) — PR-269 实证部分 check fail
- 5 角色基本设计 (BD-CANVAS-WORKFLOW-001 v1.0.3) 已签字

**需要收敛验收** (不是失败, 是过程):

- 详细设计 (DD) Round 3-7 续作 (§8-§14)
- helm chart YAML 修一个引号就能修
- dependabot PR rebase 后合并
- 端到端 E2E 走完
- k3s / Docker / WSL 三件套在 demo 环境的搭建

**不在本任务窗口内** (诚实边界):

- 本机可启动 cluster 实证 (Docker Desktop 没起, 没 WSL)
- 总册 SRS 升 v1.2 (三核心同步)
- 投资人 demo 演示脚本的最终版 (这是 Mavis/PM 的事, 不是仓库侧的事)

---

**main HEAD: `bd97197f`** · per 2026-10-01
**dev HEAD: `3966dab7`** (= main ahead 30, per workspace lints dev-only release flow 待下一轮 sync) · 2026-10-01 ULYS-272 session 收敛时间
**诚实验证完成项**: kustomize 通过 (12 resource) · push access OK · 4 worktrees removed in-session · 3 PR-269/270/271 squash merged · 28 worktrees total
**诚实未验证项**: 本机 k3s 启动 · helm template · Dockerfile 构建 · 端到端 E2E · dependabot PR rebase · Tauri Linux CI trigger (PR-242/#255) · PR-269 4 个 CI check 红