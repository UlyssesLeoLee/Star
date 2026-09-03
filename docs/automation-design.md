# Star 平台 — Agent 交互自动化设计 (Automation Design)

> **文档版本**: v0.1 (2026-09-02)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **触发**: 2026-09-02 00:39 JST Ulysses 指令"所有涉及与 agent 交互的功能点,都应该尽可能使用 python 脚本,避免长上下文的中间内容丢失损耗忽略问题, 这部分的设计文档首先完善出来,筛选出哪些任务卡里的需求可以这么做"
> **范围**: STAR 仓 (`D:\Star`) P3-A 收官后所有剩余任务卡 (P3-B / P3-C / P3-D / P3-E / P3-F / H2 / 5 wt 后续 / kanban-vmodel P1-P9 后续 / DB W/T-M) + 子代理 dispatch / CLI 调用 / 代码改造 3 类功能点
> **依赖**: `AGENTS.md` §4 守门 17 项 + §4.1 守门派生 v1-v18 + `STAR-P3-WBS-001.md` §1-§5 / §14 任务卡 / `STAR-OLU-001.md` 1 SRE·周 = 1.2M token 换算
> **基线脚本库**: `scripts/automation/` (本设计文档 §6 落档 4 个基类 + 1 个判定 CLI)

---

## 0. 文档说明

### 0.1 文档目的与定位

本设计文档是 `AGENTS.md` §4 守门 #1 v17 / v18 实证 (P0-1 + H2 token 估 0.3-0.5M 实测 0.6-1.6M, 3-5x 超支) + 守门 #9 子代理 RPC 实证 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded) 之后的"**根因对策**":

- **P0-1 联动审计** (2026-08-31 11:00 JST) 实证 19 个 fix 脚本 (scripts/p0_1_*.py) 落地, 净修 246→0 err, 但写脚本本身的 token 没被精确量化
- **H2-EXT 5 domain 跨 session 续做** (2026-08-31 22:00 JST) 实证 star-context 扩展 4 helper 落地 commit `68ae5ff` + 净修 145+ err, 但 5 domain 实际改造 0.6-0.8M token 仍卡在 sub-context
- **5 wt 后续 / P3-B-F / H2 强类型重构** 等任务卡, 如果不**先把"agent 交互 → python 脚本"规范化**, 后续 P3-B/E/F + H2 + kanban-vmodel 全 56 子项会反复重蹈"agent 在主上下文写大段 shell / 写大段 Edit" 的覆辙

**结论**: 任何**"agent 跟外部交互"的功能点 (3 类全包)** 强制走 python 脚本, 避免中间内容 (长 shell 命令 / 大量 Edit diff / 子代理 RPC payload) 占用主上下文 → 导致后续 turn 上下文丢失 / 损耗 / 忽略。

### 0.2 与其他设计文档的关系

| 设计文档 | 关系 |
|---|---|
| `docs/ai-agent-design.md` (v0.2) | 上游 — 讲 AI 子系统 (Context Compiler / AgentSession / Provider Data Boundary); 本文档 §3.1 引用其 14 状态 AgentSession / 5 Priority / 3 态 Decision |
| `docs/basic-design.md` | 上游 — 讲 5 域 DDD / 26 子域架构; 本文档 §3.2 引用其 §7.4 AgentSession |
| `docs/test-design.md` v0.3 | 兄弟 — AC 矩阵生成器 (`scripts/generate_ac_matrix.py`) 是本设计文档 §6.4 范式第 1 份实装 (T.1 子项 commit `4fa31d7`) |
| `docs/frontend/design/mock-msw-handlers.md` | 兄弟 — 5 域 MSW handler (commit `3dde2b4` `b424611`) 是本设计文档 §3.3 数据量阈值实证 |
| `STAR-P3-WBS-001.md` | 下游 — §1-§5 / §14 任务卡逐条 [P]/[S]/[M] 标 (本设计文档 §4 落表) |
| `AGENTS.md` §4.1 守门派生 v19 | 下游 — 本设计文档定稿后追加守门派生规 |

### 0.3 适用读者

- Mavis root agent (Mavis 接手 per DEC-008) — 主导 P3-B/E/F + H2 落地
- 子代理 (worker / explorer / verifier) — worktree 内调用 scripts/automation/ 基类
- 5 域 Lead 真人 (DDD Review 阶段到位后) — review 本设计文档 + 拍板 §4 任务卡标注
- SRE Lead (per STAR-OLU-001 §6 质量门 5 维) — 终评本设计文档 + 守门 #1+#9+#12+#19 联合实证

### 0.4 引用约定

- 引用 `AGENTS.md` 用 `守门 #N` 形式 (例: 守门 #1 v17)
- 引用 `STAR-P3-WBS-001.md` 用 `WBS §N.M` 形式 (例: WBS §14.2 H2-1)
- 引用 `scripts/automation/<file>.py` 用 `automation/<file>` 形式 (例: automation/dispatcher.py)
- 引用 task 子项 commit 用 7 字符短码 (per 守门 #1 禁回溯叙事)

---

## 1. 根因分析 — 为什么"agent 交互"在主上下文损耗中间内容

### 1.1 三类功能点 + 各自损耗模式

| 类 | 描述 | 典型 token 消耗 | 主上下文损耗模式 |
|---|---|---|---|
| **3.1 子代理 dispatch** | root → worker / explorer / verifier 子代理调用 | RPC payload 5-50K + 返 stdout 10-100K | 子代理 RPC 不可靠 (实证 10/10 ERR_CONNECTION_CLOSED 但 status=succeeded) → 主上下文需要二次验证, 二次验证的内容又进主上下文 |
| **3.2 本地工具 / CLI 调用** | git / cargo / npm / curl / wt / find / xargs 等 shell 命令 | 单条 0.5-2K, 跨 stage 累计 5-20K | 长 shell 管道 (`find -exec` / `cargo test` 重试链) 反复出现, 跨平台差异 (Windows PowerShell vs WSL bash) 重写, token 复用率低 |
| **3.3 代码改造 (refactor / fix / mass-edit)** | 看报告 → 改 100+ 文件, 跨 9 crate 改 507 err | 单次改造 5-30K, 跨 stage 累计 50-200K | Edit tool 逐文件提交, 每个 diff 进主上下文; 看长报告 (CONTENT-REVIEW-PACK 27KB) 进主上下文; 改完 cargo check 报错再回主上下文 |

**根因**: 这 3 类**全部需要"agent 在主上下文里写中间制品"** (长 shell / 长 diff / 长报告), 制品本身**对主上下文的最终决策没有直接价值**, 但**占 token → 推后续 turn 内容出窗口 → 损耗 / 丢失 / 忽略**。

### 1.2 已实证的损耗案例 (per WBS §6 累计统计 + AGENTS.md §4.1 守门派生 v17/v18)

| 实证事件 | 估算 token | 实测 token | 倍数 | 损耗模式 |
|---|---|---|---|---|
| P0-1 ActorContext 单点化 (WBS §14.2) | 0.2M | 0.4-0.5M | 2-2.5x | 19 个 fix 脚本 (scripts/p0_1_*.py) — **有脚本化**但**没规范化**, 脚本与脚本之间不可重入不可观测 |
| H2 范围扩量 (3 → 8 domain, WBS §14.2) | 0.3-0.5M | 0.6-0.8M (原 3) + 0.5-0.8M (H2-EXT 5) = 1.1-1.6M | 3-5x | 长 H2 stage 1-4 报告 (5cfb7b3 / 68ae5ff / 8364223 / 4c8bd5c) 反复在主上下文; revert 决策 (commit `8364223`) 反复回滚 |
| P0-1 联动审计报告 (WBS §14.4 B-10) | 0.1M (估) | 0.4M (实测) | 4x | QA-DRIFT-001 master 报告 103 drifts / 32 P0 / 28 P1 / 27 P2 / 7 unverified 进主上下文, 真人 review 内容确认包 27KB + INC-SESSION-005 10.3KB = 37.3KB 再次进主上下文 |
| 子代理 RPC 实证 (WBS §14.8) | 0 (不估) | 0 (全失败) | N/A | 10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded, 二次验证消耗 0.2-0.3M token |

**结论**: 4 项实证全部指向**"中间内容进主上下文"** 是损耗的根因。规范化 = **"中间内容不进主上下文, 全部走 Python 脚本 → stdout/stderr/file 落档"**。

### 1.3 守门 #1 / #9 / #12 跟本设计文档的关系

| 守门 | 已有规则 | 本设计文档增强 |
|---|---|---|
| 守门 #1 (0 unsafe + 守门实证) | 跨 stage 必跑 4 步 (`cargo check` + fmt + clippy + test) | **守门 #1 派生 v19** (本设计文档定稿后追加): 任何"agent 交互 → python 脚本"必须落 `scripts/automation/<purpose>.py` 标 `[P]`, 实证 commit message 必须含脚本相对路径 |
| 守门 #9 (子代理 commit 实证) | `git log -p --follow <wt-branch>` 实证 worktree commit 在 main 链上 | **守门 #9 派生 v2** (本设计文档定稿后追加): 子代理调用前**必先**调用 `automation/dispatcher.py` 落地 brief + brief 路径写进 commit message |
| 守门 #12 (commit-time docs 同步) | docs 同步跨 4 文件 (WBS / PHASE report / AGENTS.md / 引用文档) | **守门 #12 派生 v2** (本设计文档定稿后追加): 任何 [P] 子项落档后必更新本设计文档 §4 任务卡表 + `scripts/automation/registry.md` 索引 |

---

## 2. 设计原则 — 4 个筛选维度 + 3 档判定

### 2.1 4 个筛选维度 (per 9/2 00:39 JST 拍板, 全部 4 维必含)

| 维度 | 定义 | 判定阈值 | 命中分 |
|---|---|---|---|
| **R — Rerunnable (可复现性)** | 同一脚本可针对不同 input (commit hash / branch / file list / REQ ID) 重跑, 产出 deterministic output | 输入参数化 (CLI args / config yaml) + 输出有迹 (stdout / file / exit code) | +1 |
| **V — Volume (数据量阈值)** | 改动文件数 ≥ 10 或行数 ≥ 200 或 token 输出 ≥ 5K | 任一满足 | +1 |
| **S — Structural (结构性)** | 重复模式 ≥ 3 (例: 22 domain 全部改 pub mod context → use star_context) | 启发式 + AST / regex 操作比逐个 Edit 快 ≥ 2x | +1 |
| **A — Audit-trail (审计可观测)** | Python 脚本 stderr/stdout 可定向到 `docs/reports/<phase>.log` 入档, 便于后续 re-derive commit 来源 | 必填 `audit_log` 参数 + log schema (timestamp / phase / action / input / output / error) | +1 |

### 2.2 3 档判定 (per 9/2 00:39 JST 拍板, WBS 任务卡全过一遍)

| 档 | 得分 | 含义 | 处理 |
|---|---|---|---|
| **[P] Python 化** | ≥ 3 维命中 | 强制走 `scripts/automation/<purpose>.py` 落地, commit message 含脚本路径 | 必须 |
| **[M] Mixed (混合)** | = 2 维命中 | 部分走脚本 + 部分 Shell / Edit, 在 `scripts/automation/<purpose>.py` 落"主调用" + 注释标注 ad-hoc 步骤 | 推荐 |
| **[S] Shell / Edit 直接** | ≤ 1 维命中 | 不需要脚本化, agent 主上下文直接处理 | 允许 |

**例外 (per 守门 #6 派生)**: 任何 [S] 任务卡跨 stage 累计消耗主上下文 ≥ 5K token, 自动升档 [M]; ≥ 10K token 升档 [P] (per §1.2 实证 P0-1 + H2 都是先 [S] 后补 [P])。

### 2.3 判定 CLI (per §6.5 落档 `automation/judge.py`)

`automation/judge.py` 是**辅助工具**, 给任务卡 [P]/[S]/[M] 三档判定提供**打分界面**。**判定结果不自动应用** (per 拍板决策必须用选项, 9/1 14:58 JST 拍板), 而是 Mavis 终端读 CLI 输出后用 `ask_user` 跟 Ulysses 拍板。

```bash
# 用法
python scripts/automation/judge.py --task-id P3-B.5 --hits R,V \
  --note "mock 备选 5 endpoint, 跨 P3-B.1+B.2, 凭证依赖"

# 输出 (JSON)
{
  "task_id": "P3-B.5",
  "hits": ["R", "V"],
  "score": 2,
  "verdict": "[M] Mixed",
  "rationale": "R+V 命中 (5 endpoint 重跑 + ~10K token); 不命中 S (5 endpoint 各自 schema 异); 不命中 A (没有 stderr 持久化需求)",
  "automation_path": "scripts/automation/integration_test.py"
}
```

---

## 3. 三类功能点 + 范式

### 3.1 子代理 dispatch (类 1) 范式

**问题**: root → worker / explorer / verifier 子代理调用, RPC 不可靠, status="succeeded" ≠ 实际成功 (per 守门 #9 实证 10/10 ERR_CONNECTION_CLOSED)。

**范式**: `automation/dispatcher.py` 落地 brief → `automation/dispatcher.py invoke <agent> <brief_path>` 走 exec 替代 RPC → 落地 status + output → 二次验证走 `automation/dispatcher.py verify <task_id>`。

**收益**:
- 子代理调用从 RPC 黑盒 → exec 显式启动进程, 可观测可重试
- brief 入档 `docs/briefs/<task_id>.md` → commit message 引用 brief 路径
- 子代理 output 入档 `docs/briefs/<task_id>.output.md` → 不进主上下文

**基类骨架** (per §6.1 `automation/dispatcher.py`):
- `class SubagentDispatcher`: `def brief(task_id, content) -> Path` / `def invoke(agent, brief_path, timeout) -> TaskHandle` / `def verify(task_id) -> bool` / `def collect_output(task_id) -> Path`

### 3.2 本地工具 / CLI 调用 (类 2) 范式

**问题**: 长 shell 命令 (find -exec / cargo test 重试链 / 跨 wt 文件同步 / 跨平台差异) 在主上下文反复写, 跨 stage 累计 5-20K token 损耗。

**范式**: `automation/cli_helper/<command>.py` 落"主调用脚本", agent 调 `python scripts/automation/cli_helper/<command>.py --args`, agent 主上下文**不写**长 shell。

**收益**:
- 长 shell → Python 1 行调用, 主上下文减少 5-20K token
- 跨平台差异 (PowerShell vs WSL) → Python 内部 `subprocess` 抽象
- 失败可重试 (脚本内 `for attempt in range(3): ...`) → 主上下文不需反复重写

**基类骨架** (per §6.2 `automation/cli_helper/base.py`):
- `class CliHelper`: `def run(cmd, *, retries=1, timeout=60, audit_log=None) -> CmdResult` / `def with_worktree(branch) -> WorktreeContext` / `def cargo(cmd, args)` / `def git(cmd, args)` / `def wt(cmd, args)`

**范式样例** (`automation/cli_helper/cargo_check.py`):
```python
# 替代主上下文里的 6 行 cargo check 实证脚本
from automation.cli_helper.base import CliHelper
h = CliHelper(audit_log=Path("docs/reports/P3-B.5.log"))
result = h.cargo("check", ["--workspace", "--all-targets"], retries=2)
print(f"err_count={result.stderr.count('error[')}")
```

### 3.3 代码改造 (类 3) 范式

**问题**: 看长报告 (PHASE / QA-DRIFT / CONTENT-REVIEW-PACK 27KB) → 改 100+ 文件 (跨 22 crate) → cargo check 报错 → 再改 → 再 check, 中间制品 (报告 / diff / err log) 全进主上下文。

**范式**: `automation/refactor_template.py` 落"看报告 → 解析 → AST/regex 操作 → 改文件 → check → 报告" 全流程, agent 主上下文**只传报告路径** + **收最终报告**。

**收益**:
- 长报告 → 文件路径, 主上下文减 27-37KB token
- AST 操作可重放 (同一脚本对 commit 1 / commit 2 / commit N 复跑)
- 失败可回滚 (脚本自动 `git stash` + rollback)
- 审计可观测 (脚本 log 落 `docs/reports/refactor-<phase>.log`)

**基类骨架** (per §6.3 `automation/refactor_template.py`):
- `class RefactorTemplate`: `def __init__(report_path, *, dry_run=True)` / `def parse_report() -> list[Action]` / `def apply(action) -> ApplyResult` / `def verify() -> VerifyResult` / `def rollback()` / `def run_full() -> FinalReport`

**范式样例** (`scripts/p0_1_actor_context_migration.py` 实证 → 落档本模板):
- 19 个 fix 脚本 (P0-1) 改写为统一模板调用
- 5 domain H2-EXT (WBS §14.2 H2-3) 改写为统一模板调用
- 后续 H2 强类型 ID 重构 (WBS §14.2 H2-4) 直接套模板

### 3.4 横向范式 (跨 3 类) — audit log + brief 索引

| 横向 | 落档 | 内容 |
|---|---|---|
| Audit log | `docs/reports/<phase>.log` | timestamp / phase / action / input / output / error / 脚本路径 |
| Brief 索引 | `docs/briefs/registry.md` | task_id / brief_path / output_path / commit / status |
| 脚本索引 | `scripts/automation/registry.md` | 脚本相对路径 / 用途 / 调用方 / 末次 commit |

---

## 4. WBS 任务卡 [P]/[S]/[M] 判定表 (per §2 范式, 全 56 子项 + 5 行业预设 + H2)

> **判定口径**: 本表是**初判**, per 拍板决策必须用选项 (9/1 14:58 JST 拍板), Mavis 终端用 `ask_user` 跟 Ulysses 逐条拍板后落档到 WBS。
> **打分维度** (per §2.1): R=Rerunnable, V=Volume, S=Structural, A=Audit-trail

### 4.1 P3-B (9 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| B.1 | B.1 | OpenClaw HTTP API 客户端 | R, V | **[M]** | `automation/integration_test.py` | 已收官 (commit `63c34ab`); 重构 5 endpoint × 4 method, R+V |
| B.2 | B.2 | Hermes HTTP API 客户端 | R, V | **[M]** | `automation/integration_test.py` | mock 备选 (per 29692a7); 走 wiremock, R+V |
| B.3 | B.3 | API Key 双模式存储 | S | **[S]** | — | schema 5 字段, 单文件, 改一次 |
| B.4 | B.4 | CliProfile schema 扩展 | S | **[S]** | — | schema 5 字段, 单文件 |
| B.5 | B.5 | OpenClaw 真实集成 e2e | R, V, A | **[P]** | `automation/integration_e2e.py` | 5 endpoint, cross-verify, mock 备选 (per 29692a7) |
| B.6 | B.6 | Hermes 真实集成 e2e | R, V, A | **[P]** | `automation/integration_e2e.py` | 同 B.5, 共享脚本 |
| B.7 | B.7 | API 配额 / 限流 / 重试 | R, S | **[M]** | `automation/quota_test.py` | backoff + 抖动 + retry-after, R+S |
| B.8 | B.8 | API Agent → CLI Agent 降级 | R, S, A | **[P]** | `automation/fallback_chain.py` | fallback 链路跨 5 stage, R+S+A |
| B.9 | B.9 | API Agent 监控 + 审计日志 | R, A | **[P]** | `automation/audit_log.py` | 接入 domain-audit, 必填 audit_log, R+A |

### 4.2 P3-C (9 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| C.1 | C.1 | Workspace 域 | S | **[S]** | — | per-tenant lifecycle, domain-workspace 已有 crate, 单域增强 |
| C.2 | C.2 | Project 域 | S | **[S]** | — | per-workspace CRUD, 单域 |
| C.3 | C.3 | Identity 域 | S | **[S]** | — | per-tenant auth, 单域 |
| C.4 | C.4 | WorkItem 域 | S | **[S]** | — | per-project 状态机, 单域 |
| C.5 | C.5 | Workflow 域 | S | **[S]** | — | per-WorkItem, 单域 |
| C.6 | C.6 | Saga 域 | R, S, A | **[P]** | `automation/saga_e2e.py` | 跨 5 域补偿 + 失败回滚, R+S+A |
| C.7 | C.7 | Postgres 持久层 | R, S, A | **[P]** | `automation/migration_runner.py` | per-tenant schema 隔离 + 跨 9 crate SQL, R+S+A |
| C.8 | C.8 | Tenant 域 | S | **[S]** | — | per-tenant RBAC, 单域 |
| C.9 | C.9 | 5 域 Lead 真人到位 | — | **[S]** | — | 真人寻访, 无 agent 交互 |

### 4.3 P3-D (7 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D.1 | D.1 | w28 切 HubCliRuntime 入口 | S | **[S]** | — | 单文件改入口 |
| D.2 | D.2 | 跨平台 e2e 矩阵 | R, V, A | **[P]** | `automation/cross_platform_e2e.py` | windows/macos 矩阵, R+V+A, mock 备选 CI runner |
| D.3 | D.3 | frontend e2e (Playwright) | R, V, S, A | **[P]** | `automation/playwright_runner.py` | 4 维全命中, 已实装, 落本模板 |
| D.4 | D.4 | realFetch error wrapper | S | **[S]** | — | 单函数包装 |
| D.5 | D.5 | agents/analytics/inbox 3 handler real-mode | R, V, S | **[P]** | `automation/msw_switch.py` | 3 handler × real-mode switch, R+V+S (per 3dde2b4 实证) |
| D.6 | D.6 | markdownlint + cargo doc CI job | R, A | **[M]** | `automation/ci_runner.py` | mock 备选, R+A |
| D.7 | D.7 | UserMenu 状态条 | S | **[S]** | — | 单 UI 组件 |

### 4.4 P3-E (7 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| E.1 | E.1 | Audit 域 | S | **[S]** | — | per-domain-audit 增强, 7 不变量 + 9 字段, 单域 |
| E.2 | E.2 | Notification 域 | S | **[S]** | — | per-workspace 通知, 单域 |
| E.3 | E.3 | Search 域 | S | **[S]** | — | per-tenant tsvector, 单域 |
| E.4 | E.4 | KMS 集成 | R, V, A | **[P]** | `automation/kms_rotate.py` | Vault / AWS KMS, R+V+A, mock 备选 LocalMockKms |
| E.5 | E.5 | 5 域 Lead 真人到位 (DDD Review) | — | **[S]** | — | 真人寻访 |
| E.6 | E.6 | 5 域 Saga 实装 | R, S, A | **[P]** | `automation/saga_e2e.py` | 跨域补偿 + 失败回滚, 共享 C.6 脚本 |
| E.7 | E.7 | 5 域 DDD 边界验证 | R, A | **[M]** | `automation/ddd_review.py` | docs 阶段, R+A |

### 4.5 P3-F (6 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| F.1 | F.1 | 5 域 Lead 真人到位 (DDD Review) | — | **[S]** | — | 真人寻访, 跟 E.5 合并 |
| F.2 | F.2 | 跨域集成测试 (5 域 E2E) | R, V, S, A | **[P]** | `automation/cross_domain_e2e.py` | 4 维全命中, 已实装 (commit `6c1bd6c`), 落本模板 |
| F.3 | F.3 | CHANGELOG 跨域汇总 | R, A | **[M]** | `automation/changelog_gen.py` | 5 域 DDD 边界表, R+A |
| F.4 | F.4 | 架构图 mermaid 化 | R, A | **[M]** | `automation/mermaid_gen.py` | 5 域 DDD 边界图 + Saga 流程图, R+A |
| F.5 | F.5 | 质量门 5 维全 5 实证 | R, V, A | **[P]** | `automation/quality_gate.py` | 5 维全过, 跨 P3 全 5 阶段 56/64 子项, R+V+A |
| F.6 | F.6 | 推 origin (R-05 反转) | R, A | **[P]** | `automation/git_push.py` | 推 3 branch + 守门 + secret 扫描, R+A, 已实装 (per 587b212) |

### 4.6 H2 强类型 ID 重构 (WBS §14.2, 5 子项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| H2-1 | H2-1 | star_context 共享 ActorContext 字段扩展 | R, S, A | **[P]** | `automation/refactor_template.py` | 已落地 (commit `68ae5ff`), 4 helper + 2 builder + 8 unit test |
| H2-2 | H2-2 | 3 domain port/service 改造 | R, V, S, A | **[P]** | `automation/refactor_template.py` | 4 维全命中, 117+ err 实证 (WBS §14.2), revert 实证 |
| H2-3 | H2-3 | 5 domain 跨域改造 | R, V, S, A | **[P]** | `automation/refactor_template.py` | 4 维全命中, 净修 507 err (WBS §14.2), 3/5 完成 |
| H2-4 | H2-4 | 强类型 ID 重构 (DeviceId→Uuid) | R, V, S, A | **[P]** | `automation/refactor_template.py` | 4 维全命中, 业务语义拍板阻塞 (WBS §14.7 已知缺口 #1) |
| H2-5 | H2-5 | H2 原 3 domain service.rs 改造 | R, V, S, A | **[P]** | `automation/refactor_template.py` | 4 维全命中, ~150+ call sites, 阻塞 H2-4 完成 |

### 4.7 kanban-vmodel-jp P1-P9 (WBS §14.1, 13 子项已落地, 后续增量)

| 阶段 | 子项 | 命中维度 | 初判 | 脚本路径 | 备注 |
|---|---|---|---|---|---|
| P1-P5 | 各 12 task × 4 行业 = 60 task | V, S | **[M]** (各 phase 1 脚本) | `automation/kanban_vmodel_gen.py` | 13 commits 落地, 已用 Python 生成, 落本模板 |
| P6 | 6 子阶段 × 4 行业 = 56 task | V, S, A | **[P]** | `automation/test_phase_gen.py` | 已落地, 5 commits 8 测试 |
| P7-P9 | 各 12 task × 4 行业 | V | **[M]** | `automation/release_phase_gen.py` | 已落地, 3 commits |

### 4.7.1 kanban-vmodel-jp Sprint 视图 (per `docs/briefs/kanban-sprint-view-001.md`, 3 子阶段)

| 阶段 | 子项 | 命中维度 | 初判 | 脚本路径 | 备注 |
|---|---|---|---|---|---|
| **P1 Sprint 核心** | Sprint 数据模型 + Tab 切换 + CRUD + Planning UI + Board 过滤 + Jira 設計 Backlog 优先 (v0.2) | V, S | **[M]** | `automation/kanban_sprint_gen.py` (43 + 11 = 54 项) | **🟢 已落地** (2026-09-03), 54/54 pass, 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P1-REPORT.md` v0.2 |
| **P2 Sprint 度量** | Velocity 图 + Burndown 图 + Sprint 历史表 + Capacity | V, S | **[M]** | `automation/kanban_sprint_gen.py` (含 1 项 P2 验证) | **🟢 已落地** (2026-09-03), 55/55 pass, 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P2-REPORT.md` v0.1 |
| **P3 Sprint 仪式** | Standup notes + Sprint Review + Retrospective + Goal 横幅 | V, A | **[M]** | `automation/kanban_sprint_gen.py` (含 38 项 P3 验证) | **🟢 已落地** (2026-09-03), 93/93 pass, 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P3-REPORT.md` v0.1 |

### 4.8 DB W/T/M (WBS §14.3, 6 子项持续验证)

| # | 子项 | 命中维度 | 初判 | 脚本路径 | 备注 |
|---|---|---|---|---|---|
| CW-1~6 | 6 子项 | R, V, S, A | **[P]** | `automation/db_wtm_classifier.py` | 跨 100 表 W/T/M 分类, 4 段检查清单 + 派生守门 10 条 |

### 4.9 后续 5 wt + INC-003 (WBS §12.5)

| # | wt | 命中维度 | 初判 | 脚本路径 | 备注 |
|---|---|---|---|---|---|
| wt-push-origin | 推 origin | R, A | **[P]** | `automation/git_push.py` | 跟 F.6 共享 |
| wt-b5-openclaw-mock | B.5 mock | R, V, S, A | **[P]** | `automation/integration_e2e.py` | 跟 B.5/B.6 共享 |
| wt-b6-hermes-mock | B.6 mock | R, V, S, A | **[P]** | `automation/integration_e2e.py` | 同上 |
| wt-b1-openclaw-http | B.1 真实 | R, V | **[M]** | `automation/integration_test.py` | 跟 B.1 共享 |
| wt-b3-apikey-storage | B.3 真实 | S | **[S]** | — | schema 5 字段, 单文件 |
| wt-b7-api-quota | B.7 真实 | R, S | **[M]** | `automation/quota_test.py` | 跟 B.7 共享 |

### 4.11 图表 & 报告系统 (新增 phase, 2026-09-02 11:00 JST per docs/briefs/P3-CHARTS-P0.md)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| CHARTS-P0 | 基础设施 + C01 全跑通 | 12 Rust 文件 + 4 frontend 文件 + Recharts 依赖 + InMemory cache + Port trait + C01 真实算法 + 19 测试 | R, V, S, A | **[P]** | `automation/charts_p0_setup.py` | 19/19 测试 pass, 0 err / 0 clippy, commit author=Ulysses |
| CHARTS-P0-Bulk | P0 剩余 7 图表 (C02-C05, C06, C07, C13) | 7 图表 × ~200 行 Rust + ~250 行 TSX | R, V, S, A | **[P]** | (待续 charts_p0_bulk.py) | 阶段 2, 复用 C01 模板批量 |
| CHARTS-P1 | P1 批 7 图表 (C08-C12, C14-C15) | 7 图表 + Recharts 系列 | R, V, S, A | **[P]** | (待续) | 阶段 3 |
| CHARTS-P2 | P2 批 7 图表 (C16-C22) | 含 C21 Heatmap 自研 SVG | R, V, S, A | **[P]** | (待续) | 阶段 4 |

### 4.10 任务卡分布统计

| 档 | 数量 | 占比 | 备注 |
|---|---|---|---|
| **[P] Python 化** | 21 (含共享脚本 + 4 图表批) | ~34% | 必落 `scripts/automation/<purpose>.py` |
| **[M] Mixed** | 10 | ~16% | 部分脚本 + 部分 ad-hoc |
| **[S] Shell / Edit** | 25 (含真人寻访) | ~50% | 不需要脚本化 |
| **合计** | 56 (去重) | 100% | P3 全 5 阶段 56 子项 - 4 重复 - 5 真人寻访 + 4 图表批 = 56 |

### 4.12 P3-G Agent Jira 化 (新增 phase, 2026-09-03 12:00 JST per docs/briefs/p3-g-w1.md)

> **命名空间备注**: 跟现有 P3-B (OpenClaw/Hermes/API Key 集成 9 子项 per §4.1) 命名空间共存, P3-G 用 G.1-G.20 连续编号, P3-B 沿用 B.1-B.9。**不沿用 P3-B 字头, 避免命名冲突** (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标, Mavis 主动 rename 12:05 JST)。
> **WBS 文档**: `docs/reports/PHASE-P3-G-JIRA-IFICATION-WBS.md` v0.1
> **Brief**: `docs/briefs/p3-g-w1.md`

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| G.1 | G.1 | user_account 实体 (M 类 SCD-2 + RLS 13 類) | R, S, A | **[P]** | (待 W2 派生 `automation/user_account_mgmt.py`) | `permission.user_account` 表, 0.4M, W1 落地 |
| G.2 | G.2 | group + group_member (M + T) | R, S, A | **[P]** | (待续) | `permission.group` + `permission.group_member`, 0.5M |
| G.3 | G.3 | team 实体 (M 类) | R, S | **[P]** | (待续) | `permission.team`, 0.3M, 跟 user_account 一起 |
| G.4 | G.4 | team_member 多重隶属 + role_per_team (T) | R, S, A | **[P]** | (待续) | `permission.team_member` + `permission.role_per_team`, 0.5M |
| G.5 | G.5 | user_account ↔ agent 关联 (双层 L1) | R, S, A | **[P]** | (待续) | `agent.user_account_link`, 0.2M |
| G.6 | G.6 | subagent 实体 (双层 L2) | R, S, A | **[P]** | `automation/dispatcher.py` 升级 + 落 agent.subagent | 0.6M, W2 落地 |
| G.13 | G.13 | dispatcher.py 自动注册 | R, S, A | **[P]** | `automation/dispatcher.py` 升级 | 0.2M, W2 落地, 跟 G.6 强依赖 |
| G.9-G.20 | G.9-G.20 | 跨域协作 + 集成 + 收尾 (12 子项) | R, V, S, A | **[P]** | (待续 G.9-G.20 各自脚本) | 3.8M, W3-W5 落地, 跨 session 续 |

**W1 5 子项 (G.1-G.5) 总 token**: 1.9M (per 守门 #4 软预算 1.5M 偏差 +0.4M, 软参考可接受)
**W2-W5 15 子项 (G.6-G.20) 总 token**: 4.0M (推 origin 后走, 跨 session 续)
**合计 6.0M ≈ 5 周** (per `STAR-OLU-001.md` 1.2M/SRE·周)

**W1 守门 0 违反验证** (per 守门 #1 v1-v14 + 守门 #13 DB 三類横展開 + 守门 #21 [P] docs 同步 + 守门 #6 PowerShell only + 守门 #7 0 unsafe):
- `cargo check --workspace --all-targets` 0 err
- `cargo fmt --all` 0 diff
- `cargo clippy --workspace --all-targets -- -D warnings` 0 err
- `cargo test --workspace --release --lib` 100% pass
- 5 新表 100% RLS + FORCE RLS + 13 類 policy
- 5 新表 W/T/M 分类显式列 + §已知缺口 显式列 (per 守门 #13 派生规)
- docs 同步 5 表设计 + data-design.md / basic-design.md / domain-permission-spec.md / automation-design.md §4.12 / scripts/automation/registry.md / AGENTS.md §4.1 派生 v25 全部 git 实证
- W1 不派子代理 (per 守门 #9 #3 实证 5/5 RPC 不可靠)

**§4.10 编号错位备注** (per init 阶段 9/3 12:05 JST Mavis 修订): §4.10 任务卡分布统计 实际位于 §4.11 图表&报告系统 之后, 编号顺序在文档流中错位, 但内容独立无依赖, 不影响判定。后续 §4.12+ 按文档流顺序递增。

**§4.12 命名空间合规性 (per 守门 #21 [P] docs 同步)**: 本节是 docs 同步落地, 不需要落 `scripts/automation/<purpose>.py` (G.1-G.5 是数据设计阶段, 实施在 W2-W5 跨 stage 派生)。W2 G.13 dispatcher.py 自动注册 落地后, 同步回填本节 G.6 / G.13 行的"实证 / 备注" 列。

### 4.13 SRS-STAR-AGENT-RUNTIME-001 Baseline 落档 (2026-09-03 18:14 JST per docs/briefs/...)

> **触发**: 2026-09-03 18:14 JST Ulysses 发令"参考这个制作需求文档" + 18:20 JST 拍板 "A. commit + 落档 ADR (推荐)" + "仅文档落档, 不触发 P3-B"
> **落档文件**: `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (53KB / 113 节)
> **ADR**: `docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` v1.0
> **依据**: 守门 #21 v21 派生规 + 守门 #12 缺标比错标 + 守门 #3 5 域单仓 + 守门 #1-#24 + 累积规 v1-v24

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| SRS-1 | SRS-1 | SRS-STAR-AGENT-RUNTIME-001 v1.0 落档 | A | **[P]** | (纯文档, 不需脚本) | 113 节 / 53KB / 7 段结构 + 100 节正式内容, 12 已落地 / 8 部分 / 60 待 P3-B-F / 4 N/A |
| SRS-2 | SRS-2 | ADR-0044 落档 | A | **[P]** | (纯文档, 不需脚本) | 7 段结构 + 5 角色签字栏 + dual-use disclaimer, 编号 0044 续 0043 |
| SRS-3 | SRS-3 | automation-design.md §4.13 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #21 v21 [P] docs 同步必更新 §4 任务卡表 |
| SRS-4 | SRS-4 | registry.md 索引更新 | A | **[S]** | (待办) | per 守门 #21 v21 [P] docs 同步必更新 registry.md, **本节 ⏳ 待 commit 前补** |
| SRS-5 | SRS-5 | P3-B 启动 gate 阻塞 | — | **[S]** | — | per 2026-09-03 18:20 JST Ulysses 拍板 "仅文档落档, 不触发 P3-B", 等 5 域 Lead 真人 + 凭证 (B.5/B.6) + KMS (E.4) + HANDOFF-ST-001 §5.3 5 Blocker + P3-C/D/F 范围 全部到位 |

**§4.13 任务卡维度判定**:
- R (Rerunnable): 否 (纯文档落档, 不涉及重跑)
- V (Volume): 否 (无子代理派发, 一次性写入)
- S (Structural): 否 (不改动 scripts/automation/ 框架, 纯文档)
- A (Audit-trail): **是** (守门 #21 [P] docs 同步必留痕 + 守门 #9 git 实证)

**§4.13 落档验证 (per 守门 #1 累积规 v1-v24, 本次纯文档不需 cargo 守门)**:
- `git log -p --follow docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-08-26-upgrade/adr/0044-star-agent-runtime-srs.md` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.13 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权)

### 4.14 STAR Agent Runtime Basic + Detailed Design 落档 (2026-09-03 19:00 JST per `docs/architecture/2026-09-03-agent-runtime/`)

> **触发**: 2026-09-03 18:48 JST Ulysses 发令"基本设计和详细设计也都到位" + 18:59 JST 拍板 "A. 独立目录 + A. 引用 LangGraph + ADR-0045 + 双落 docs 同步"
> **落档文件**:
> - `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` v0.1 (40KB, 12 章节)
> - `docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` v0.1 (52KB, 15 章节)
> - `docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` v1.0 (14KB)
> **依据**: 守门 #21 v21 派生规 + 守门 #12 缺标比错标 + 守门 #3 5 域单仓 + 守门 #13 DB W/T/M + 守门 #19 自动化 Python 化 + 守门 #1-#24 + 累积规 v1-v24

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| AR-1 | AR-1 | 02-basic-design.md v0.1 落档 | A | **[P]** | (纯文档, 不需脚本) | 40KB / 12 章节 (per LangGraph 9/3 02 范式), 3 层架构 (L0 派发 + L1 ECS + L2 业务) + Runtime 双模式 + 9 SA Type 引用 + 31 domain-* 目标 + 13 Systems + NFR + G-13~G-15 |
| AR-2 | AR-2 | 03-detailed-design.md v0.1 落档 | A | **[P]** | (纯文档, 不需脚本) | 52KB / 15 章节 (per LangGraph 9/3 03 范式), 9 模块 (M-01..M-15) + 13 关键类 (Rust 草案) + 2 状态机 + 4 时序图 (UC-01..UC-04) + 5 表 schema (W/T/M 严格分类, per 守门 #13) + 4 算法 + 7 错误处理 + 4 类测试 (UT/IT/E2E/PT) + G-16~G-17 |
| AR-3 | AR-3 | ADR-0045 落档 | A | **[P]** | (纯文档, 不需脚本) | 14KB / 7 段结构 + 5 角色签字栏 + dual-use disclaimer, 编号 0045 续 0044 |
| AR-4 | AR-4 | automation-design.md §4.14 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #21 v21 [P] docs 同步必更新 §4 任务卡表 |
| AR-5 | AR-5 | registry.md §5.2 同步 | A | **[P]** | (本节追加) | per 守门 #21 v21 必更新 registry |
| AR-6 | AR-6 | AGENTS.md §6 ADR 索引 +0045 | A | **[P]** | (AGENTS.md 编辑) | per 守门 #21 ADR 索引同步 |
| AR-7 | AR-7 | LangGraph 9/3 引用不重写 | A | **[S]** | (拍板 18:59 JST A 路径) | 9 SA Type 引用 LangGraph 9/3 §6.1, 不重写业务逻辑, 节省 0.8M token |
| AR-8 | AR-8 | P3-B 启动 gate 阻塞 (跟 §4.13 SRS-5 一致) | — | **[S]** | — | per 2026-09-03 18:48 JST 用户发令"基本设计 + 详细设计", 跟 §4.13 SRS-5 共用阻塞条件 |

**§4.14 任务卡维度判定**:
- R (Rerunnable): 否 (纯文档落档, 不涉及重跑)
- V (Volume): 否 (无子代理派发, 一次性写入)
- S (Structural): 否 (不改动 scripts/automation/ 框架, 纯文档; AR-7 是拍板确认, AR-8 是依赖)
- A (Audit-trail): **是** (守门 #21 [P] docs 同步必留痕 + 守门 #9 git 实证 + 守门 #13 DB schema 分类留痕)

**§4.14 落档验证 (per 守门 #1 累积规 v1-v24, 本次纯文档不需 cargo 守门)**:
- `git log -p --follow docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-09-03-agent-runtime/03-detailed-design.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-08-26-upgrade/adr/0045-star-agent-runtime-design.md` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.14 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §5.2 追加 (commit 后)
- `git log -p --follow AGENTS.md` 实证 §6 ADR 索引 +0045 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权)

------

## 5. 守门基线 (per 守门 #1 派生 v19 + #9 派生 v2 + #12 派生 v2)

### 5.1 4 步基线 (per WBS §12.6 / §14.5)

1. **必跑** `cargo check --workspace --all-targets` — 0 err
2. **必跑** `cargo fmt + clippy` — 0 err
3. **必跑** `cargo test --workspace --release --lib` — 0 fail
4. **必跑** `cargo build --release + doc + bench --no-run` — 0 err

### 5.2 自动化基线 (本设计文档新增)

5. **必跑** `python scripts/automation/judge.py --all` (per §6.5) — 输出 WBS 任务卡 [P]/[S]/[M] 标**初判表**, Mavis 终端用 `ask_user` 跟 Ulysses 拍板后落档 WBS
6. **必跑** `python scripts/automation/smoke_test.py` (per §6.6) — 跑通 `automation/dispatcher.py` + `cli_helper/base.py` + `refactor_template.py` 4 个基类的最小可运行案例 (无副作用)
7. **必跑** `python scripts/automation/registry_check.py` (per §6.7) — 校验 `scripts/automation/registry.md` 索引跟实际脚本一致

### 5.3 守门 #1 派生 v19 (本设计文档定稿后追加到 AGENTS.md §4.1)

> **v19 — agent 交互 Python 化守门** (per 2026-09-02 00:39 JST 拍板 + `docs/automation-design.md` v0.1):
> 任何"agent 跟外部交互"的功能点 (子代理 dispatch / CLI 调用 / 代码改造 3 类) 强制走 `scripts/automation/<purpose>.py` 落地, 实证 commit message 必须含脚本相对路径; 跨 stage 累计消耗主上下文 ≥ 5K token 的 [S] 子项自动升档 [M], ≥ 10K 升档 [P] (per `docs/automation-design.md` §1.2 实证 P0-1 + H2).

### 5.4 守门 #9 派生 v2 (本设计文档定稿后追加到 AGENTS.md §4.1)

> **v2 — 子代理 dispatch 必先落地 brief** (per 2026-09-02 00:39 JST 拍板 + `docs/automation-design.md` §3.1):
> 子代理调用前**必先**调用 `automation/dispatcher.py` 落地 brief → 路径 `docs/briefs/<task_id>.md` → commit message 引用 brief 路径; 子代理 RPC 不可靠实证 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded, per AGENTS.md §4 #9 主体规则).

### 5.5 守门 #12 派生 v2 (本设计文档定稿后追加到 AGENTS.md §4.1)

> **v2 — Python 化任务卡 docs 同步** (per 2026-09-02 00:39 JST 拍板 + `docs/automation-design.md` §1.3):
> 任何 [P] 子项落档后必更新 `docs/automation-design.md` §4 任务卡表 + `scripts/automation/registry.md` 索引; commit 引用 `automation-design.md §N.M` 章节号.

---

## 6. 基类骨架 + 判定 CLI + 索引 (落档 `scripts/automation/`)

> **落档规则** (per 守门 #12 + 缺标比错标): 4 个基类 + 1 个判定 CLI + 2 个 smoke 脚本 + 1 个索引 md, 共 8 份文件, 落档 `scripts/automation/`。

### 6.1 `automation/dispatcher.py` — 子代理 dispatch 基类

> 替代 root → 子代理 RPC 黑盒调用, 走 exec 显式启动进程, 可观测可重试可重放。

**骨架要点**:
- `class SubagentDispatcher`: brief 落档 → invoke exec → status 落档 → verify 二次验证 → collect_output
- 必填参数: `task_id` / `agent_name` / `brief_content` / `timeout`
- 输出: `docs/briefs/<task_id>.md` (brief) + `docs/briefs/<task_id>.output.md` (output) + `docs/briefs/<task_id>.status.json` (status)
- audit_log 必填, 落 `docs/reports/<phase>.log`

### 6.2 `automation/cli_helper/base.py` — CLI 调用基类

> 替代主上下文长 shell 反复写, 跨平台差异 (PowerShell vs WSL) 抽象, 失败可重试。

**骨架要点**:
- `class CliHelper`: `run(cmd, *, retries, timeout, audit_log)` / `cargo(cmd, args)` / `git(cmd, args)` / `wt(cmd, args)` / `with_worktree(branch)`
- 内部 `subprocess.run` 抽象, 跨平台 (Windows / WSL / macOS / Linux)
- 失败重试默认 1 次, 指数 backoff
- audit_log 必填, 落 `docs/reports/<phase>.log`

### 6.3 `automation/refactor_template.py` — 代码改造基类

> 替代"看长报告 → 改 100+ 文件" 流程, 走"看报告路径 → 解析 → AST/regex → 改 → check → 报告"。

**骨架要点**:
- `class RefactorTemplate`: `__init__(report_path, *, dry_run=True)` / `parse_report() -> list[Action]` / `apply(action) -> ApplyResult` / `verify() -> VerifyResult` / `rollback()` / `run_full() -> FinalReport`
- 子类继承, 重写 `parse_report` + `apply` 即可
- 失败自动 `git stash` + rollback, 实证 commit 写入
- audit_log 必填, 落 `docs/reports/refactor-<phase>.log`

### 6.4 `automation/generate_ac_matrix.py` — 范例 (T.1 已实装)

> 引用 `scripts/generate_ac_matrix.py` 实证 (commit `4fa31d7`, 249 行, 标准库 only, REQ → AC → Test 覆盖矩阵), 作为本设计文档 §3 范式第 1 份实装。

### 6.5 `automation/judge.py` — 任务卡 [P]/[S]/[M] 判定 CLI

> 辅助工具, 给任务卡判定提供打分界面, 输出 JSON, 不自动应用 (per 拍板决策必须用选项 9/1 14:58 JST 拍板)。

**骨架要点**:
- CLI args: `--task-id` / `--hits` (R/V/S/A 任意组合) / `--note`
- 命中维度数 → [P]/[M]/[S] 档
- 输出 JSON: `task_id` / `hits` / `score` / `verdict` / `rationale` / `automation_path` (建议)
- `python scripts/automation/judge.py --all` 跑 WBS 全任务卡, 输出初判表 → Mavis 终端用 `ask_user` 跟 Ulysses 拍板

### 6.6 `automation/smoke_test.py` — 基类 smoke 验证

> 跑通 4 个基类 (dispatcher / cli_helper / refactor_template / generate_ac_matrix) 的最小可运行案例, 无副作用, 验证 import + class 实例化 + method 调用都通过。

**骨架要点**:
- `if __name__ == "__main__":` 入口, 直接 `python scripts/automation/smoke_test.py`
- 每个基类 1 个 smoke case, 5 个全过 = 0 err 退出
- 输出 `docs/reports/automation-smoke.log`

### 6.7 `automation/registry_check.py` — 索引一致性校验

> 校验 `scripts/automation/registry.md` 索引跟实际脚本一致 (脚本路径 / 用途 / 调用方 / 末次 commit)。

**骨架要点**:
- 扫描 `scripts/automation/*.py` 实际文件
- 跟 `registry.md` 表格对照
- 不一致项 → 输出 warning, 不阻塞 CI

### 6.8 `registry.md` — 脚本索引

> 表格: 脚本相对路径 / 用途 / 调用方 / 末次 commit / 状态

---

## 7. 已知缺口 (per 缺标比错标)

1. **WBS 任务卡 [P]/[S]/[M] 终判需 Ulysses 拍板** — 本设计文档 §4 是**初判**, per 9/1 14:58 JST 拍板决策必须用选项, Mavis 终端用 `ask_user` 跟 Ulysses 逐条拍板后落档 WBS
2. **`automation/refactor_template.py` 子类化落地** — 模板落档后, 需把 P0-1 19 个 fix 脚本 + H2-EXT 5 domain 脚本改写为子类 (跨 session 续, 估 0.4-0.6M token)
3. **5 域 Lead 真人到位前, F.5 质量门 5 维全 5 实证** 不能落档, 因为质量门终评需 SRE Lead + 5 域 Lead 真人 (per WBS §14.4 B-9)
4. **`automation/dispatcher.py` 跨平台 exec 抽象** — 当前设计仅覆盖 Windows PowerShell, 跨 WSL / macOS / Linux 需补 `subprocess` 适配层 (per 守门 #6 PowerShell only 派生)
5. **P3-A 25 子项历史脚本未回填** — P0-1 19 fix 脚本 + H2-EXT 5 domain 脚本在 P3-A 阶段已落地, 但 `scripts/automation/registry.md` 是新索引, 历史脚本需逐个回填 (per 守门 #12 缺标比错标)

---

## 12. 调试控制台 (Automation Debug Console, v0.2 新增)

> **触发**: 2026-09-02 09:01 JST Ulysses 指令"这些 py 脚本要运需用户通过填写 api key 的 ai 修改,并且给一个专用脚本调试页面,允许用户在一定范围内勾选脚本生效的功能点,并且允许关闭"
> **拍板** (4 选项, per 9/1 14:58 JST 拍板决策必须用选项):
> - scope = **全部 13 份 Python 脚本 + 5 套 unittest 调试页** (8 份基类 + 5 份 [P] 任务卡脚本 + 5 套 unittest)
> - ai-edit-mode = **本地 mock** (不开外部 AI, 仅调用脚本生成模板建议)
> - debug-ui = **Web UI (Next.js + shadcn + tailwind, 跨客户端浏览器)**
> - close-behavior = **关闭 = 脚本/功能点 跳过运行 (不调用)**
> **依赖**: §3 + §4 + §5 + §6 全部基类; frontend/ (Next.js 14 App Router) 已就绪

### 12.1 架构 (3 层)

```
+--------------------+      +-------------------------+      +-------------------+
| Browser (Chrome)   |      | Next.js Frontend        |      | Python FastAPI     |
| localhost:3000     | <--> | frontend/src/app/       | <--> | scripts/automation/|
| Automation Debug   |      | automation-debug/       |      | console_server.py  |
| Console UI         |      | (shadcn + tailwind)     |      | (port 8080)        |
+--------------------+      +-------------------------+      +-------------------+
                                                                       |
                                                                       v
                                                              +-------------------+
                                                              | 14 份 Python 脚本   |
                                                              | (6 base + 4 [P]    |
                                                              |  + 4 unittest)     |
                                                              +-------------------+
```

### 12.2 14 份 Python 脚本 + 4 套 unittest 清单 (per §4 任务卡表 + SCRIPTS_META 1-1 对应, 含 available_in_debug 标记)

> **重要**: 本清单跟 `console_server.py` 的 `SCRIPTS_META` 字典 1-1 对应 (14 份 = 6 base + 4 [P] + 4 unittest)。`__init__.py` × 2 + `ai_edit_mock.py` + `console_server.py` 本身不在 SCRIPTS_META 里 (是辅助文件, 不在调试页可勾选清单内)。

| 类别 | 脚本路径 | [P]/[M]/[S] | available_in_debug | 功能点 (用户可勾选) |
|---|---|---|---|---|
| base | `scripts/automation/dispatcher.py` | — | ✓ | `brief` / `invoke` (stub) / `verify` (stub) / `collect_output` (stub) |
| base | `scripts/automation/cli_helper/base.py` | — | ✓ | `run` / `cargo` (stub) / `git` (stub) / `wt` (stub) / `with_worktree` (stub) |
| base | `scripts/automation/refactor_template.py` | — | ✓ | `parse_report` / `apply` / `verify` (stub) / `rollback` (stub) / `run_full` |
| base | `scripts/automation/judge.py` | — | ✓ | `judge(task_id, hits, note)` / `judge_all()` |
| base | `scripts/automation/smoke_test.py` | — | ✓ | `dispatcher` / `cli_helper` / `refactor_template` / `judge` 4 case |
| base | `scripts/automation/registry_check.py` | — | ✓ | (单步 check, 不可单独勾选) |
| [P] B.5 | `scripts/automation/integration_e2e.py` | [P] | ✓ | `provider=openclaw` / `provider=hermes` / `dry_run` / `no_dry_run_stub` / `audit_log` |
| [P] C.6 | `scripts/automation/saga_e2e.py` | [P] | ✓ | `fail_domain={none,player,economy,match,social,admin}` / `dry_run` / `audit_log` |
| [P] F.6 | `scripts/automation/git_push.py` | [P] | ✓ | `remote=origin` / `dry_run` / `no_dry_run_stub` / `max_scan_files` / `audit_log` |
| [P] H2-1 | `scripts/automation/h2_refactor.py` | [P] | ✓ | `phase=P3-H2` / `dry_run` / `no_dry_run_stub` / `audit_log` |
| unittest | `scripts/automation/__tests__/integration_e2e_test.py` | [P] | ✓ | 6 OpenClaw + 6 Hermes 12 case (per §4.1) |
| unittest | `scripts/automation/__tests__/saga_e2e_test.py` | [P] | ✓ | 10 case (5 域 × 2 成功/失败回滚, per §4.2) |
| unittest | `scripts/automation/__tests__/git_push_test.py` | [P] | ✓ | 5 case (dry_run + reachable + secret + token + audit, per §4.5) |
| unittest | `scripts/automation/__tests__/h2_refactor_test.py` | [P] | ✓ | 5 case (parse + action1 + action2 + apply + inherits, per §4.6) |

**辅助文件 (不在调试页可勾选清单内)**:
- `scripts/automation/__init__.py` — 包初始化
- `scripts/automation/cli_helper/__init__.py` — 子包初始化
- `scripts/automation/ai_edit_mock.py` — AI 修改 mock (v0.2 新增, 被 console_server.py 调用)
- `scripts/automation/console_server.py` — FastAPI 8080 后端 (v0.2 新增)
- `scripts/automation/_test_console_server.py` — 7 端点 smoke 测试 (v0.2 新增)
- `scripts/automation/_run_baseline.py` — 7 步守门基线 (v0.2 新增)
- `frontend/src/app/automation-debug/` × 11 份 .tsx/.ts — 调试页前端 (v0.2 新增)
- `frontend/src/components/ui/` × 7 份 shadcn fallback — 调试页组件 (v0.2 新增)
- `frontend/src/lib/utils.ts` — cn() helper (v0.2 新增)

**统计**:
- 14 份可调试脚本 (6 base + 4 [P] + 4 unittest) — 跟 console_server.py SCRIPTS_META 1-1 对应
- 11 份辅助文件 (5 Python + 6 前端)
- 总计 25 份新文件 (v0.2 一并落地)

### 12.3 API 端点 (FastAPI 8080)

| 端点 | 方法 | 描述 |
|---|---|---|
| `/api/scripts` | GET | 列 13 份脚本 + 5 套 unittest (含 metadata: name / path / features / status) |
| `/api/scripts/{id}/toggle` | POST | 用户勾选/关闭脚本 (status: enabled/disabled) |
| `/api/scripts/{id}/run` | POST | 跑脚本 (status: enabled 才能跑, 跑完返 output 头 500 字符) |
| `/api/features/{script_id}/{feature_id}/toggle` | POST | 勾选/关闭脚本内功能点 (e.g. `provider=hermes`) |
| `/api/ai_edit` | POST | AI 修改 mock: 读脚本源码 + 模板生成建议 (不开外部 API) |
| `/api/status` | GET | 13 份脚本 + 5 套 unittest 状态总览 (跑 / 关闭 / AI mock 等) |
| `/api/brief` | POST | dispatcher.brief 落档 (per 守门 #20 v2) |
| `/docs` | GET | FastAPI 自动 swagger 文档 |

### 12.4 前端 UI (Next.js 14 + shadcn + tailwind)

```
frontend/src/app/automation-debug/
  page.tsx                          # 主页面 (ScriptSelector + RunPanel + AIEditPanel)
  layout.tsx                        # layout
  components/
    ScriptSelector.tsx              # 13 份脚本 + 5 套 unittest 列表 (checkbox + 关闭开关)
    FeatureToggles.tsx              # 脚本内功能点勾选 (e.g. provider=openclaw)
    RunPanel.tsx                    # 跑脚本 + 显示 output (头 500 字符)
    AIEditPanel.tsx                 # AI 修改 mock (读源码 + 模板建议, 不开外部 API)
    StatusDashboard.tsx             # 13 份脚本 + 5 套 unittest 状态总览
  hooks/
    useDebugConsole.ts              # 调 FastAPI 8080
  api/
    scripts/route.ts                # Next.js API route (proxy → FastAPI 8080)
```

**关键交互**:
- 用户在 `ScriptSelector` 勾选要运行的脚本 (per close-behavior=1 跳过关闭的)
- `FeatureToggles` 显示当前脚本的功能点 (e.g. integration_e2e.py → provider=openclaw/hermes, dry_run)
- `RunPanel` 跑脚本, 输出显示在下方 (头 500 字符避免长 output 占 token)
- `AIEditPanel` 点 "AI 修改" → 后端读脚本源码 → 产生模板建议 (3 条 edit suggestion: add field / remove method / rename class)
- `StatusDashboard` 13 份脚本 + 5 套 unittest 状态 (跑 / 关闭 / AI mock 次数)

### 12.5 守门基线 (per §5 + 本节新增)

- 守门 #1 v20: console_server.py 跑后 `cargo check --workspace --lib` 0 err (新派生, 实证 console_server 不会污染 main 编译)
- 守门 #5 v2: 调试页 AI 修改 mock **不开外部 API**, API key 不走 UI 输入 (per ai-edit-mode=本地 mock 拍板)
- 守门 #9 v3: 调试页 → console_server.py → 13 份脚本, 走 subprocess 替代 RPC, 跟守门 #9 实证 #3 一致 (子代理 RPC 不可靠但 subprocess 可靠)
- 守门 #12 v3: 调试页加新基类 (console_server.py + ai_edit_mock.py) 必更新 §12 清单表 + registry.md
- 守门 #11 实证: 缺标比错标安全, §12.6 列已知缺口

### 12.6 已知缺口 (per 守门 #11)

1. **AI 修改 mock 不真调外部 API** (per ai-edit-mode=本地 mock 拍板), 用户需手动 apply 模板建议
2. **frontend/src/app/automation-debug/ 是新建目录** (per debug-ui=Web UI 拍板), 需 next dev 跟 console_server.py 双进程, 跨 session 续 npm + python 双 server 启动
3. **13 份脚本 metadata 提取** 需从脚本源码静态分析 (import 路径 / CLI args), 模板生成可能不准, 跨 session 续改进
4. **5 套 unittest 勾选 = 整套 enable/disable** (per §12.2 简化设计), 内部 case 不可单独勾选, 跨 session 续考虑细化
5. **关闭语义 = 跳过运行** (per close-behavior=1 拍板), 关闭态脚本/功能点 dispatcher 仍能 brief 落档但不 invoke, audit log 标 "disabled"

---

## 8. 跨项目持久 (per 守门 #4 派生规 v1-v18 + 守门 #13)

- 本设计文档适用 **STAR / RGS / Physis / GVPE / 其他新项目**
- 跨项目持久理由: 子代理 RPC 不可靠 + 长 shell 反复写 + 长报告看 diff 三大类问题是 AI 协作通用损耗模式, 不限于 STAR
- 引用基线: `D:\Star\docs\automation-design.md` v0.1 (本设计文档)

---

## 9. 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 2026-09-02 | 🟢 终审通过 |
| 2 | SRE Lead | (待真人到位) | — | ⏳ 待签 |
| 3 | 平台 | (待真人到位) | — | ⏳ 待签 |
| 4 | 评审主持 | (待真人到位) | — | ⏳ 待签 |
| 5 | PM | (待真人到位) | — | ⏳ 待签 |

**注**: per 8/27 19:39 JST 用户授权升级 + 8/27 21:59 JST 第三次强化, Mavis 接手代签 Ulysses; 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补。

---

## 10. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: 3 类 agent 交互 (子代理 dispatch / CLI 调用 / 代码改造) 全包, 4 个筛选维度 (R/V/S/A) + 3 档判定 ([P]/[M]/[S]), WBS §1-§5 / §14 / kanban-vmodel 任务卡全过初判, 守门 #1 v19 + #9 v2 + #12 v2 派生规; 落档 `scripts/automation/` 4 基类 + 1 CLI + 2 smoke + 1 索引, 共 8 份文件 | 2026-09-02 00:39 JST Ulysses 指令"所有涉及与 agent 交互的功能点,都应该尽可能使用 python 脚本,避免长上下文的中间内容丢失损耗忽略问题, 这部分的设计文档首先完善出来,筛选出哪些任务卡里的需求可以这么做" + 拍板 3 选项 (范围=全 3 类 / 维度=R+V+S+A / 落档=新建 docs/automation-design.md + scripts/automation/) |
| v0.2 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | **§12 调试控制台 (Automation Debug Console)** 新增: 4 拍板 (scope=13 py 脚本+5 unittest / ai-edit=本地 mock / debug-ui=Next.js+shadcn / close-behavior=跳过运行); frontend/src/app/automation-debug/ + scripts/automation/console_server.py + scripts/automation/ai_edit_mock.py 3 份新基类; 守门 #1 v20 + #5 v2 + #9 v3 派生规; docs/automation-design.md §4 任务卡表加 'available_in_debug' 标记 | 2026-09-02 09:01 JST Ulysses 指令"这些 py 脚本要运需用户通过填写 api key 的 ai 修改,并且给一个专用脚本调试页面,允许用户在一定范围内勾选脚本生效的功能点,并且允许关闭" + 拍板 4 选项 |
| v0.3 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | **§4.11 图表 & 报告系统 (CHARTS) 新增 phase** (per docs/briefs/P3-CHARTS-P0.md + 2026-09-02 11:00 JST Ulysses 拍板 A+I+α): 4 子项 (P0 基础设施 + C01 真实 / P0 剩余 7 / P1 7 / P2 7) 全 [P]; 落档 `scripts/automation/charts_p0_setup.py` (P0 阶段 1); §4.10 任务卡分布统计从 52 → 56 子项; 守门 #1 v19 + #12 v15 + #20 v20 + #21 v21 联合实证: 16 文件 + 19/19 测试 + 0 err + 0 clippy | 2026-09-02 10:04 JST Ulysses "图表对标 Jira" + 11:00 JST 拍板 A+I+α (per docs/briefs/P3-CHARTS-P0.md v0.1) |
| v0.4 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 kanban-vmodel-jp Sprint 视图 新增 phase** (per docs/briefs/kanban-sprint-view-001.md + 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图"): 3 子项 (P1 核心 / P2 度量 / P3 仪式) 全 [M]; 落档 `scripts/automation/kanban_sprint_gen.py` (P1 验证 43 项); P1 已落地 43/43 pass, 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P1-REPORT.md` v0.1; 守门 #1 v19 + #20 v20 + #21 v21 + #22 v22 联合实证: HTML+JS+CSS 0 err + 8/8 结构 + 43/43 函数 | 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图" + 13:25 JST Mavis 推进 P1 收官 |
| v0.5 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P1 v0.2 Jira 設計 Backlog 优先 增量** (per docs/briefs/kanban-sprint-view-001.md v0.2 + 2026-09-03 13:55 JST Ulysses 反馈 "进入sprint前应该在backlog, 删除sprint列时, 里面的内容也应该进入backlog, 参考jira设计。所有文档要更新好"): 4 处数据流修改 (addToSprint 校验 / removeFromSprint 重置 / completeSprint 未完了回流 / cancelSprint + 削除 全件回流) + 新增 `returnSprintTasksToBacklog()` ヘルパー + Sprint Plan modal Jira 設計 hint + 非 backlog 警告; 自动化档 `kanban_sprint_gen.py` 校验项 43 → 54 (+11) → 55 (+1 P2 sprintMetrics); P2 度量落地 (Velocity SVG bar + Burndown SVG line + Sprint history table + Capacity config) + `<div id="sprintMetrics">` + .sprint-metrics / .metric-card / .chart-svg / .history-table / .capacity-form CSS; 报告 SPRINT-P1-REPORT v0.2 (新增 §8 Jira 設計增量章节); 守门 #1 v19 + #20 v20 + #21 v21 + #22 v22 联合实证: 55/55 pass + 0 err | 2026-09-03 13:55 JST Ulysses Jira 設計反馈 + Mavis 推进 P1 v0.2 + P2 收官 |
| v0.6 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P3 仪式 收官 增量** (per docs/briefs/kanban-sprint-view-001.md v0.3 + 2026-09-03 14:05 JST Ulysses 拍板 "开 P3 仪式"): P3 子项 P1/P2 状态改为 🟢 已落地; 落档 SCRIPTS/CEREMONIES (Goal 横幅 + Standup 3 問 + Review Demo 候補 + Retrospective KPT 3 列 + Markdown 导出) + `<div id="sprintCeremonies">` + .sprint-ceremonies / .ceremony-card / .goal-block / .standup-form / .review-grid / .retrospective-grid / .retrospective-col--good/improve/action ~300 行 CSS; 报告 SPRINT-VIEW-P3-REPORT.md v0.1 (12 项已知缺口 + 18 项守门核对); 自动化档 `kanban_sprint_gen.py` 校验项 55 → 93 (+38); 守门 #1 v19 + #20 v20 + #21 v21 联合实证: 93/93 pass + 0 err; 累计 token 估 ~1.4M / 预算 1.5-2.0M; KANBAN-SPRINT-001 3 阶段全部收官 | 2026-09-03 14:05 JST Ulysses P3 拍板 + 14:20 JST Mavis 推进 P3 收官 |

---

## 11. 引用文档

- `AGENTS.md` v0.15+ (守门 #1 派生 v1-v18 + 守门 #9 + 守门 #12 + 守门 #13)
- `STAR-P3-WBS-001.md` v0.8+ (§1-§5 / §14 任务卡 + §12.5 INC-SESSION-003 触发条件)
- `STAR-OLU-001.md` v0.1+ (1 SRE·周 = 1.2M token 换算基线)
- `docs/ai-agent-design.md` v0.2 (上游: AI 子系统设计)
- `docs/basic-design.md` (上游: 5 域 DDD / 26 子域架构)
- `docs/test-design.md` v0.3 (兄弟: AC 矩阵生成器范式来源)
- `docs/frontend/design/mock-msw-handlers.md` (兄弟: 5 域 MSW handler 实证)
- `PHASE-P0-1-ACTOR-CONTEXT-IMPL-REPORT.md` v0.3 (实证: 19 个 fix 脚本)
- `..\reports\HANDOFF-ST-001.md` v0.2 (实证: H2 范围扩量 + 强类型重构)
- `scripts/p0_1_actor_context_migration.py` (实证: 第 1 份 P0-1 联动脚本)
- `scripts/generate_ac_matrix.py` (实证: AC 矩阵生成器, T.1 子项)
- `scripts/p0_h2_3domain_migration.py` (实证: H2 真实尝试脚本入档)



### 4.12 ɢ�� WBS ��ȱ�� (per 2026-09-02 18:30 JST, Ulysses �İ忪�Ӵ����� worktree �������)


| ���� | ��Χ | token Ԥ�� | ʵʩ | commit | ��ע |
|---|---|---|---|---|---|
| star-nav-completion-001 ������ A (i18n categoryLabel ͬ��) | 7 module �� 3 ���� (zh-CN/en/ja) = 21 ���滻 + remote entry �¼� | 0.15M | worker �Ӵ��� wt/star-nav-i18n-a (UTF-8 �ֽڼ� + CRLF ����) | `bd918e4` (per git log -p --follow ʵ֤) | brief �� GBK ����, ʵ�� UTF-8 + CRLF, worker ��ʶ���� Python bytes-level |
| star-nav-completion-001 ������ B (HeaderTab 8 ���Ӿ��Ա�ͼ) | light/dark �� 4 active ״̬ (inbox/issues/agents/settings) | 0.20M | worker �Ӵ��� wt/star-nav-shots-b (HEADER_STATES ���û� + dev 200s ��̨) | `8c893a9` (per git log -p --follow ʵ֤) | 8 ��ͼȫ > 16KB, dev 90s timeout û���� |
| star-nav-completion-001 ������ C (���� page SubNav Ⱦɫ) | skip | 0 | ȫ�� <SubNav ʵ��ֻ issues/page.tsx 1 ��, ���� f65744a �� 4 view Ⱦɫ | �� | per ���� #11 ȱ��ȴ���, mark skipped |


**��֪ȱ�� + ʧ��ģʽ**: vitest pass �Ǳ�Ҫ�ǳ������ (2 worktree ���� 41 files / 345 tests pass, ��û�� e2e); 8 �Ž�ͼ�Ӿ��߲����ֹ� byte ���, û����ͼ�� diff; main worktree �� 12 �� untracked/modified ������ WIP ��ͻ, �ϲ��� stash + Move-Item ·���ܿ�. 
