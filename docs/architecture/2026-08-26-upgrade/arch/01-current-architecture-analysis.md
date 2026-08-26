# 01. Current Architecture Analysis（2026-08-26）

> **状态**：🟡 草案 v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签
> **父文档**：[STAR × GitGit 升级 Plan](../../../plan/2026-08-26-upgrade-plan.md)
> **依赖**：[ADR-0021 ~ 0025](../../adr/)

---

## 1. 现状基线

### 1.1 GitGit（单 crate MVP，c89f858 之前 main=1da5f2c）

```
gitgit/
├── Cargo.toml        # name=gitgit, single binary
├── src/
│   ├── main.rs       # CLI entry
│   ├── cli.rs
│   ├── config.rs
│   ├── error.rs
│   ├── repo/         # Repository / store / refs
│   └── server/       # auth / http / smart / subprocess
├── migrations/       # 20260826120000_v0_gui_keychain.sql (未 commit)
├── scripts/          # smoke.ps1
├── docs/
│   ├── adr/          # ADR-0020 (V0 WBS)
│   ├── operations/
│   └── plan/v0-tasks.md  # V0 WBS 10 个工作块
└── docs_archive_rust_impl_2026_08_26/  # 14-crate 设计归档
```

**关键事实**：
- 已合并 main（5007883 + 1da5f2c）：single-crate gitgit MVP
- 14-crate 设计已归档（c822a60）
- V0 WBS（c89f858）开 Tauri GUI + keychain 任务
- 当前 main 在 `feature/v0-gui-and-keychain` 分支上
- **无** IDE 概念、**无** Agent 概念、**无** Issue/Task 概念

### 1.2 STAR（25 Module crate workspace, main=5181288 ahead 58）

```
star/
├── Cargo.toml         # workspace
├── crates/            # 25 domain-* + 其他（待确认）
├── docs/
│   ├── requirements.md
│   ├── basic-design.md
│   ├── data-design.md
│   ├── api-design.md
│   ├── security-design.md
│   ├── runtime-design.md
│   ├── integration-design.md
│   ├── ai-agent-design.md
│   ├── external-design.md
│   ├── internal-design.md
│   ├── test-design.md
│   ├── operation-design.md
│   ├── frontend-design.md + frontend-canvas + frontend-internal-{01..04}
│   ├── plan/          # master-implementation-plan.md (Draft v0.1)
│   ├── plans/         # plan-016..024 (RFC-1:1 plans)
│   ├── poc/           # 15 份 PoC
│   ├── rfcs/          # 15 份 RFC
│   ├── specs/         # 25 Module spec
│   └── tmp_plans_ignore/
├── frontend/          # Next.js 14 control plane UI
├── tools/             # 工具脚本
└── frontend-design-feedback.md (未 commit)
```

**关键事实**：
- 25 Module 全部 v0.2 single-file rewrite
- frontend + canvas Miro 模式已合并
- master plan v0.1 仍 Draft
- **无** `star` CLI 骨架、**无** MCP server、**无** AGENTS.md 生成器
- **无** IDE Gateway / AI Gateway / Code Intelligence Arch

### 1.3 工作区未收尾（ahead 58 + 未 commit）

- 12 个 `_commit_*.txt` / `_sow_*.md` / `frontend-design-feedback.md` 未入仓
- 4 个 `wt_clean*.ps1` / `check_wt.ps1` 临时脚本
- 1 个 `.gitignore` 修改未 commit

## 2. 现状能力 gap（per 任务原文 §48）

| 任务原文需求 | 现状 | Gap |
|---|---|---|
| IDE Capability Boundary Analysis | 无 | 需补（per `gitgit-ide-boundary.md` 草案） |
| STAR vs GitGit Responsibility Matrix | 无 | 已有草案（per `star-vs-gitgit.md` v0.1） |
| AI Ecosystem Research | 无 | 已落（per `ai-compatibility-matrix.md`） |
| IDE Ecosystem Research | 无 | 已落（per `ide-compatibility-matrix.md`） |
| AI and IDE Compatibility Matrix | 无 | 已落（per `compatibility-matrix.md`） |
| Current Architecture Analysis | 无 | 本文档 |

## 3. 现状架构的 5 个关键限制

1. **无 CLI 入口** — STAR 没有 `star` CLI（master plan §1.3 提到的命令集 100% 不存在）
2. **无 MCP server** — 7 款主流 Agent 中 6 款支持 MCP，STAR 完全错失
3. **无 AGENTS.md 生成器** — Codex / Cursor / VS Code / Junie 都会读，STAR 项目根目录没生成
4. **IDE / Agent 概念完全缺失** — 25 Module 是"业务域"维度，不是"研发工作流"维度
5. **Git Provider 抽象缺失** — GitGit 是唯一 VCS 后端，GitHub/GitLab/Gitea 用户零接入路径

## 4. 本次升级的核心变更

| 层 | 现状 | 升级后 |
|---|---|---|
| **CLI 入口** | 无 | `star` CLI（17 个核心命令） |
| **机器可读输出** | 无 | `--json` + 稳定 schema（`agent-api/v1` + `ide-api/v1`） |
| **MCP server** | 无 | 13 个领域语义 tools |
| **AGENTS.md** | 无 | 自动生成器 + 薄 bootstrap 内容 |
| **OpenAPI** | 无 | 3.1 spec |
| **Universal Submit** | 无 | 状态机 11 步 |
| **Agent Task Lifecycle** | 无 | 9 状态 + 4 异常 |
| **Agent Lease / Resume** | 无 | Heartbeat + Handoff 协议 |
| **VCS Provider 抽象** | GitGit only | 4 Provider 并列 |
| **IDE Gateway** | 无 | 抽象 + LSP 端点 |
| **Code Intelligence** | 无 | tree-sitter + rust-analyzer (Phase 2+) |
| **Context Graph** | 无 | Issue↔Repo↔Worktree↔Commit 4 类节点 (MVP) |
| **Audit** | 无 | ActorType=Human/Agent/IDE/Service/Automation 统一 trail |

## 5. 关键决策（per Phase B ADR）

- **[ADR-0021]** Zero Vendor Cooperation Principle 最高原则
- **[ADR-0022]** IDE 归 STAR, GitGit 只做 VCS Core
- **[ADR-0023]** Version Control Provider 抽象（GitGit/GitHub/GitLab/Gitea 并列）
- **[ADR-0024]** IDE Session 独立于 GitGit
- **[ADR-0025]** Vendor Adapter Anti-Contamination（Optional Adapter 子 crate）

## 6. 后续 Phase C 任务映射

| 任务原文 | 本文档位置 |
|---|---|
| Current Architecture Analysis | 本文件 |
| IDE Capability Boundary Analysis | `arch/02-ide-capability-boundary.md` |
| STAR vs GitGit Responsibility Matrix | `../../responsibility-matrix/star-vs-gitgit.md` |
| AI Ecosystem Research | `../../ecosystem-survey/ai-compatibility-matrix.md` |
| IDE Ecosystem Research | `../../ecosystem-survey/ide-compatibility-matrix.md` |
| AI and IDE Compatibility Matrix | `../../ecosystem-survey/compatibility-matrix.md` |
| Zero Vendor Cooperation ADR | `../../adr/0021-zero-vendor-cooperation.md` |
| IDE Placement ADR | `../../adr/0022-ide-placement.md` |
| STAR AI Compatibility Architecture | `arch/02-star-ai-compat-arch.md` |
| STAR IDE Gateway Architecture | `arch/03-star-ide-gateway-arch.md` |
| GitGit Compatibility Architecture | `arch/04-gitgit-compat-arch.md` |
| GitGit IDE Boundary Specification | `../../responsibility-matrix/gitgit-ide-boundary.md` |
| ... | ... (其余见各 spec 目录) |

## 7. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | 平台工程师 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版 | Phase C 第 1 轮 |
