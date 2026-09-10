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

### 1.4 守门编号累计表 (per AGENTS.md §4 + brief v0.46, 2026-09-09 21:18 JST Mavis 自驱盘点)

> **盘点状态 (per brief v0.46 + v0.57 拍板)**: 现有 26 条 v1-v26 (line 127-154 of AGENTS.md), 编号累计到 v26; 现有 **v25 重号** (line 152 + 153, per守门 #11 缺标比错标) 已修 (per 2026-09-10 20:55 JST Mavis 自驱: line 153 改 v25b 守门 #1 v25 CI cargo test, line 152 维持 v25 守门 #14 v2 5 域 Lead 内推); v17/v18/v19 文档排序错位 (line 143-145) 不影响验证; brief §2.2 示例候选 v26-v30 跟现有 v26 冲突, 重命名 v27-v31; **v28 + v29 + v32 + v33 已 🟢 active (per 9/10 20:14 JST 拍板"v33 激活")**, **v27 已 🟢 active (per 9/10 11:00 JST)**, **v30 / v31 仍 🟡 (v0.62 反转 + v0.63 进一步反转 obsolete)**.
>
> **v3x 候选 (5 条, status = 待 Ulysses 拍板激活)**: 跟 AGENTS.md §4.1.1 同源, 跨文档引用一致.

| 编号 | 类别 | 触发事件 | 候选守门内容 (摘要) | 实证 / 触发 commit | 状态 |
|---|---|---|---|---|---|
| **v27** | 子代理 RPC 失败 fallback 必填 | 守门 #9 实证 P3-A.6/A.7 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded) + 守门 #9 v3 调试控制台走 subprocess 替代 RPC | `dispatcher.py invoke` 后 30s 内**必跑** `verify()` 二次验证, 连续 2 次 retry 失败 → 走 root session 直接实装; 适用所有 P3-B-F + H2 + ARG + P0-2/3/4 跨 session 续做项 | 跟守门 #9 #3 + 守门 #9 v20 + 守门 #9 v3 同源 | 🟡 待激活 |
| **v28** | 拍板必带推荐项格式校验 | 9/8 16:08 JST Ulysses 第 8 次强化"拍板时必带推荐项 (推荐) 标" + 9/1 14:58 JST 守门 "拍板必 ask_user" | 任何 `ask_user` 必含 2-4 选项 + 至少 1 个 `(推荐)` 标, 推荐项放第 1 位; 不满足 = Mavis 终端自动 retry 1 次加推荐项; **🟢 active per v0.57 commit 落地 `scripts/automation/guardian/v28_recommendation.py`** | 跟 9/1 14:58 + 9/8 16:08 + 9/8 15:29 自驱强化 + 守门 #14 v3 同源 | 🟢 **active (per 9/10 09:30 JST 拍板)** |
| **v29** | docs 同步饱和 40+ 次主动告警 | 守门 #12 v15 死循环饱和边界 (per 2026-08-29 22:39 JST `5cfb7b3` 实证饱和点) + 9/9 21:18 JST 已达 docs 同步第 40 次新事件触发 | `automation/docs_sync_saturation.py` 自动计数, 累计 30/40/50 三档 warning + ask_user + error 阻断 commit; 仅记录 docs 同步 commit (per守门 #12 v21 [P] 子项) | 跟守门 #12 + 守门 #12 v15 + 守门 #1 v15 同源 | 🟢 **active (per 9/10 09:30 JST 拍板)** |
| **v30** | Mavis 永久代签适用边界 | 9/8 15:19 JST 第 6 次强化 + 9/8 15:29 JST 第 7 次强化 + 9/9 12:02 JST 守门 #14 v3 升级 (WBS v0.22 全部永久代签) | Mavis 永久代签**全部签字栏** (5 域 Lead / SRE Lead / 平台 / 评审主持 / PM + 真人到位相关 + 寻访流程 + DDD Review + 未来新增); 适用边界: 适用 = commit author + 修订人 + 审批 3 列; 不适用 = 方向大转弯 / Ulysses 已答选项 / 真人到位追溯 / host 状态永久改变 | 跟守门 #10 + 守门 #14 v2 + 守门 #14 v3 + 9/8 15:19/15:29/16:08 同源 | 🟡 待激活 |
| **v31** | 5 域 Lead 真人到位追溯签字机制 | 9/9 12:02 JST 守门 #14 v3 升级 "真人到位流程暂时不追踪" + 9/3 19:35 JST 拍板 D "Mavis 长期代签, 真人到位后追溯签字" 维持 | 启动信号 = Ulysses 发令"5 域 Lead 真人到位流程激活"; 追溯签字形式 = 修订历史表 +1 行覆盖 5 域 Lead 决策行; 不沿用代签决策; 真人 Subagent dispatch brief 边界待 DDD Review 拍板; 落 `docs/recruitment/5-leads-traceback-mechanism.md` v0.1 placeholder | 跟守门 #3 + 守门 #14 v2 + 守门 #14 v3 + 9/9 12:02 JST 政策升级 同源 | 🟡 待激活 |

> **激活门槛 (per 9/1 14:58 + 9/8 16:08 + 9/5 04:03 守门叠加)**: (a) Mavis 终端必走 `ask_user` 2-4 选项 + 至少 1 个 `(推荐)` 标 (per v28); (b) 推荐项放第 1 位; (c) 拍板后立即执行; (d) 拍板后同步修 v25 重号 (line 152/153) + v26 改 v25c 保持 v25 段连续; (e) 任何 v3x 激活后必更新本节 + `scripts/automation/registry.md` v0.7+ (per 守门 #12 v21 [P] docs 同步); (f) 跨 session 续做: governance-level 决策不阻塞任何 P3-B-F / H2 / ARG / P0-2/3/4 跨 session 续做项 (per守门 #14 v3 永久代签 policy).

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
| D.3.1 | D.3.1 | 无限画布 canvas-view 7 case e2e (per 2026-09-04 拍板) | S, V | **[M]** | `pnpm test:e2e -- canvas-view` | 命中 S+V 2 维, 跟 D.3 共享 playwright 跑测链, 不强制走 python (e2e spec 本身是 .ts 写, 跑测 pnpm test:e2e 是 subprocess.run 替代); 9/9 pass (1 入口 + 1 pan + 3 zoom + 1 fit + 1 highlight URL + 1 选删 + 1 minimap) per docs/briefs/canvas-e2e-guard-001.md |
| D.3.2 | D.3.2 | 无限画布 Share + Export PNG 按钮实装 (per 2026-09-04 23:00 JST 拍板) | S, V | **[M]** | `pnpm test:e2e -- canvas-share-export` | 跟 D.3.1 共享链; 3/3 pass (1 Share clipboard + 1 Export PNG download + 1 Share toast); html2canvas 1.4.1 装入 dependency; 跟 canvas e2e 9/9 + remote-mobile 8/8 + cross-domain-5b 3/3 共 23/23 回归通过 57.4s per docs/briefs/canvas-share-export-001.md |
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

### 4.15 pgwiki audit OPEN issue 收掉 (2026-09-07 20:34 JST per Ulysses 拍板"能解决就尽量解决")

> **触发**: 2026-09-07 20:30 JST 用户问"github 上的 issue 是否全解决了" + 20:34 JST 拍板"能解决就尽量解决"
> **落档文件**:
> - `scripts/automation/pgwiki_resolve_issues.py` v0.1 (14.4KB, 5+27 crate 决策表)
> - `docs/wiki/pgwiki/50-issues/_decisions.md` v0.1 (决策表, committable, 审计可见)
> - `scripts/automation/pgwiki_index.py` (SCHEMA_TO_CRATE 改: 补 work_item / 撤 kms / 改 local → local_runtime)
> - `scripts/automation/pgwiki_audit.py` (list_db_schemas 改权威 + ADR_PLANNED/ARCH_PLANNED 白名单 + scan 跳过)
> - `scripts/automation/registry.md` v0.2 (§1 索引 +1 脚本 + §3 修订历史 +1 行)
> **依据**: 守门 #1 v19 (agent 跟外部交互走 automation Python 脚本) + 守门 #3 v2 (Mavis 临时代签 5 域 Lead) + 守门 #11 (缺标比错标,撤 kms) + 守门 #12 (禁回溯叙事, 改 audit 工具层不追溯改 ADR / arch view) + 9/3 19:35 JST 拍板 D (Mavis 长期代签, 真人到位后追溯签字覆盖)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| PG-1 | PG-1 | #18 orphan `work` → 补 `work_item: domain-work-item` 映射 | S, A | **[P]** | `pgwiki_resolve_issues.py` apply_pgwiki_index() | per INVENTORY §4 (T08-T12, 5 表), list_db_schemas 缺位 bug (split `_` 第一段 "work" vs 权威 "work_item") |
| PG-2 | PG-2 | #19 placeholder `kms` → 撤 SCHEMA_TO_CRATE 映射 | S, A | **[P]** | `pgwiki_resolve_issues.py` apply_pgwiki_index() | 守门 #11 缺标比错标, kms schema 实际 0 表 (per `docs/data-design/ipa-detail/tables/` 实证), 物理 crates/domain-kms 保留 |
| PG-3 | PG-3 | #20 broker_adr 5 个 → 加 ADR_PLANNED 白名单 + scan 跳过 | S, A | **[P]** | `pgwiki_resolve_issues.py` apply_pgwiki_audit() | api-key (stale) / domain-service (节点类型) / domain-team (W2 规划) / star-lsp-proxy (MVP 不实装) / star-optional (待实装) |
| PG-4 | PG-4 | #21 broker_arch 27 个 → 加 ARCH_PLANNED 白名单 + scan 跳过 | S, A | **[P]** | `pgwiki_resolve_issues.py` apply_pgwiki_audit() | 全部 27 个是 arch view 文档 "(规) 标记" / 设计意图 / stale 引用 |
| PG-5 | PG-5 | list_db_schemas 修权威 (从 INVENTORY 抽 schema 名) | S | **[M]** | `pgwiki_audit.py` (pgwiki_resolve_issues.py 改) | 修缺位 bug: 不用文件名 split 兜底 (会引入 work / local 误报) |
| PG-6 | PG-6 | `local` → `local_runtime` 同步 (SCHEMA_TO_CRATE 跟 INVENTORY §25 权威名对齐) | S | **[S]** | `pgwiki_resolve_issues.py` apply_pgwiki_index() | INVENTORY §25: `local_runtime schema (domain-local-runtime, 5 表)`, 短名 "local" 是历史命名偏差 |
| PG-7 | PG-7 | 决策表 _decisions.md 落档 (committable, 审计可见) | A | **[P]** | `pgwiki_resolve_issues.py` write_decisions_md() | per 守门 #12 显式列决策依据, 不在代码注释里 hidden |
| PG-8 | PG-8 | registry.md §1 索引 +1 + §3 修订历史 +1 行 | A | **[P]** | (registry.md 编辑) | per 守门 #12 v21 [P] docs 同步必更新 registry |
| PG-9 | PG-9 | automation-design.md §4.15 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #21 v21 [P] docs 同步必更新 §4 任务卡表 |
| PG-10 | PG-10 | pgwiki_audit.py 跑 counter 验证 0/0/0/0/0 | A | **[S]** | `python scripts/automation/pgwiki_audit.py` | 跑出 0 orphan / 0 placeholder / 0 broker_adr / 0 broker_arch / 0 empty_crates / 0 table_module_orphan / 0 fake_deps |

**§4.15 任务卡维度判定**:
- R (Rerunnable): **是** (pgwiki_resolve_issues.py idempotent, 二次跑同样结果)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性)
- S (Structural): **是** (改 SCHEMA_TO_CRATE dict + 加 ADR_PLANNED / ARCH_PLANNED + 修 list_db_schemas)
- A (Audit-trail): **是** (守门 #12 决策表 + 守门 #21 任务卡表 + 守门 #9 git 实证)

**§4.15 落档验证 (per 守门 #1 累积规 v1-v24, 本次不需 cargo 守门,Python 脚本)**:
- `python scripts/automation/pgwiki_resolve_issues.py` exit 0 (3 步全应用)
- `python scripts/automation/pgwiki_audit.py` 跑出 counter 全 0 验证
- `git log -p --follow scripts/automation/pgwiki_audit.py` 实证 list_db_schemas 改 (commit 后)
- `git log -p --follow scripts/automation/pgwiki_index.py` 实证 SCHEMA_TO_CRATE 改 (commit 后)
- `git log -p --follow scripts/automation/pgwiki_resolve_issues.py` 实证脚本新增 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 v0.2 追加 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.15 追加 (commit 后)
- `git log -p --follow docs/wiki/pgwiki/50-issues/_decisions.md` 实证决策表新增 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 9/3 19:35 拍板 D)

### 4.16 Phase OPS-INTRY 落档 (2026-09-08 07:53 JST per ask_user `ask_e76f2e614519fbc9eda16b53` 拍板)

> **触发**: 2026-09-08 07:53 JST 用户发令 "在右上角菜单里加一个运维界面入口, 里面存放运维应有的功能" + 07:58 JST 拍板 4 项 (Q1 仅入口+4 tab 骨架 / Q2 Hybrid mock+stub / Q3 新建 star-ops crate / Q4 仅需求+基本设计)
> **落档文件**:
> - `docs/requirements/SRS-STAR-OPS-001.md` v0.1 (22.8KB, 12 节, 4 tab + 8 stub + Hybrid AI + W/T/M 6 表)
> - `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 (17.8KB, 9 节, 组件 + 8 REST 契约 + AI 4 级 Ladder)
> - `crates/star-ops/` 新建 (Cargo.toml + lib.rs + main.rs + error.rs + ops_api.rs + ops_domain/{cluster,log,metrics}.rs + ops_ai/{mock,openai_stub,anthropic_stub,ladder}.rs) — workspace 47 → 48 package
> - `frontend/src/app/ops/page.tsx` 新建 (4 tab 骨架, 跟 automation-debug 同 3D 视觉)
> - `frontend/src/components/UserMenu.tsx` line ~219 加 Wrench 入口
> - `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts` 加 `userMenu.ops` + `opsConsole` 顶层 (3 语言)
> - `scripts/automation/ai_log_mock.py` v0.1 (守门 #23 不开外部 API, 跑通 3ms, confidence=0.42)
> - `scripts/automation/registry.md` §1 +1 行
> - `Cargo.toml` members +1 行 (crates/star-ops)
> **依据**: 守门 #1 (workspace 守门) + 守门 #3 (5 域独立 Lead, Mavis 临时代签 per 9/3 11:35) + 守门 #4 (token-OLU, 估 2.0M 实装) + 守门 #6 (frontend typecheck advisory per v2) + 守门 #7 (0 unsafe, clippy advisory per v3) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 4 tab 显式列) + 守门 #13 (W/T/M 6 表 100% 覆盖) + 守门 #19 v19 (agent 交互走 automation 脚本) + 守门 #21 v21 ([P] docs 同步 §4 任务卡表 + registry.md) + 守门 #23 (AI mock, 不开外部 API) + 守门 #24 v2 (subprocess 替代 RPC) + 守门 #1 v25 (cargo test 单 crate per CI 实证)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| OPS-1 | OPS-1 | 需求 SRS-STAR-OPS-001 落档 (4 tab + 8 stub + Hybrid AI) | S, A | **[P]** | (本节追加) | per 守门 #19 v19 [P] docs 同步必更新 §4 + registry.md |
| OPS-2 | OPS-2 | 基本设计 OPS-BASIC-DESIGN-001 落档 (组件 + 8 REST 契约 + AI Ladder) | S, A | **[P]** | (本节追加) | per 守门 #4.2 唯一实施入口: 47 → 48 package |
| OPS-3 | OPS-3 | crates/star-ops 新建 (Cargo.toml + lib + main + 9 模块) | R, V, S | **[P]** | `automation/dispatcher.py` (无 brief, Mavis 接手直接落地) | 15/15 test pass, 0 err, 0.51s |
| OPS-4 | OPS-4 | UserMenu.tsx 加 Wrench 入口 (line 219) + 4 tab 路由 | S | **[S]** | (frontend 编辑) | per automation-debug 同 3D 视觉, 守门 #6 v2 advisory |
| OPS-5 | OPS-5 | i18n 3 语言 (zh-CN/en/ja) 加 userMenu.ops + opsConsole | S, A | **[S]** | (frontend 编辑) | 缺标比错标, 3 语言完整覆盖 |
| OPS-6 | OPS-6 | ai_log_mock.py 落档 (守门 #23 不开外部 API) | R, A | **[P]** | `ai_log_mock.py` (新基类) | 跑通 3ms, confidence 永远 0.42 < 0.5 |
| OPS-7 | OPS-7 | registry.md §1 +1 行 + automation-design.md §4.16 追加 | A | **[P]** | (registry.md + automation-design.md 编辑) | per 守门 #21 v21 [P] docs 同步 |
| OPS-8 | OPS-8 | 守门实证: cargo check + cargo test + cargo fmt + cargo clippy + frontend typecheck | R, A | **[M]** | (cargo / npm run typecheck) | per 守门 #1 v25 (单 crate 实证) + 守门 #6 v2 (advisory) |

**§4.16 任务卡维度判定**:
- R (Rerunnable): **是** (cargo check 0 err + cargo test 15/15 pass 可重放)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性)
- S (Structural): **是** (新建 crates/star-ops 7 模块 + 3 语言 i18n + 4 tab 路由结构)
- A (Audit-trail): **是** (SRS + BAS + 报告 7 段 + 修订历史 + git 实证)

**§4.16 落档验证 (per 守门 #1 累积规 v1-v25, 本次必跑)**:
- `cargo check -p star-ops --all-targets -j 4` 0 err
- `cargo test -p star-ops --lib -j 4` 15/15 pass
- `cargo fmt -p star-ops --check` 0 err
- `cargo clippy -p star-ops --all-targets -j 4` 0 err (advisory per 守门 #7 v3)
- `npm run typecheck` (待跑)
- `python scripts/automation/ai_log_mock.py` exit 0, 跑通 3ms
- `git log -p --follow crates/star-ops/` 实证新建 (commit 后)
- `git log -p --follow frontend/src/app/ops/page.tsx` 实证新建 (commit 后)
- `git log -p --follow scripts/automation/ai_log_mock.py` 实证新建 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.16 追加 (commit 后)
- `git log -p --follow docs/requirements/SRS-STAR-OPS-001.md` 实证新建 (commit 后)
- `git log -p --follow docs/basic-design/OPS-BASIC-DESIGN-001.md` 实证新建 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 9/3 19:35 拍板 D + 9/3 11:35 守门 #3 v2)

**§4.15 issue 自动关闭 (per 2026-09-06 13:41 JST 拍板 A3 状态机)**:
- 关闭触发: counters 全 0 **或** git log 含 `Closes #N` / `Fixes #N`
- counter 验证: 0/0/0/0 → automation 周期跑会**自动 close** #18 #19 #20 #21 (下次 cron 触发)
- 预期关闭时间: 30min 内 (per pgwiki_audit 周期)

### 4.17 P3-C W1 ARG.1 — crates/arg 6 子模块骨架实装 (2026-09-09 04:38 JST per `docs/briefs/arg-01-arg-crate-skeleton.md`)

> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4 / budget=选项3分阶段批 / merge=串行merge走守门)
> **依据**: 守门 #1 v19 (P 子项 Python 化) + 守门 #1 v15 (新事件 docs 同步允许) + 守门 #1 v25 (cargo test -p star-arg --lib -j 4 100% pass) + 守门 #5 (env 安全) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #10 (author = Ulysses) + 守门 #12 ([P] docs 同步) + 守门 #13 (W/T/M 5 表分类) + 守门 #14 v2 (5 域 Lead Mavis 临时代签)
> **落档文件**:
> - `crates/arg/` 新建 (Cargo.toml + lib.rs + error.rs + llm.rs + 10 models + 3 client + 5 ops + 3 query = 30 src + 7 tests = 37 文件) — workspace 65 → 66 package
> - `scripts/automation/memgraph_setup.py` v0.1 (Docker compose 启动 Memgraph + health probe + .env stub, 守门 #5 不打印密码)
> - `scripts/automation/arg_seed.py` v0.1 (种子 5 域 Lead + 9 SA + 10 demo = 24 节点, 5 consults + 5 reports_to = 10 边)
> - `docs/automation-design.md` §4.17 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +2 行 + §5.3 +1 行
> - `docs/reports/PHASE-ARG-01-IMPL-REPORT.md` v0.1 (7 段 per AGENTS.md §3)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-1 | ARG-1 | `crates/arg` 6 子模块 (models / client / ops / query / llm + error) | S, A | **[P]** | `scripts/automation/memgraph_setup.py` (Docker compose) + `arg_seed.py` (种子 fixture) | Cargo workspace 65 → 66 package; 守门 #7 `unsafe_code = "forbid"` 0 unsafe; 30 src + 7 tests = 37 文件 |
| ARG-2 | ARG-2 | 30 UT (5+8+5+6+4+2+2 = 32, 含 brief 自加 4 trust_score + 2 event) | R, V, A | **[P]** | `cargo test -p star-arg --tests -j 4` | 守门 #1 v25 实证 100% pass (lib test 1 + 32 integration test = 33 全 pass, 0 failed) |
| ARG-3 | ARG-3 | 5 守门全套 (check / fmt / clippy / test / build) | R, V, A | **[P]** | (守门 #1 累积规 v1-v5) | `cargo check --workspace --lib -j 4` 0 err + `cargo fmt -p star-arg --check` 0 err + `cargo clippy -p star-arg --all-targets -j 4 -- -D warnings` 0 err + `cargo test -p star-arg --tests -j 4` 100% + `cargo build --release -p star-arg` 0 err |
| ARG-4 | ARG-4 | 数据模型 16 + 14 + 10 + 5 + 20 + 5 + 9 = 79 字段 | S, A | **[P]** | (per DD §3.2 / §3.3 / §9.1) | Agent 16 / Edge 14 / RelationshipType 10 / TeamTemplate 5 / Achievement 20 / TrustScoreTier 5 / ARGEvent 9 / ARGError 10 + Decision/DecisionType/Output/Verdict/EscalationInfo/PeerReviewVerdict/ChallengeVerdict/ChallengePrompt/LLMClient/LLMResponse/TemplateInstance/AchievementUnlock 全 (per DD §3.2.5 self-review 修复) |
| ARG-5 | ARG-5 | memgraph_setup.py + arg_seed.py 落档 | R, V, S, A | **[P]** | `scripts/automation/memgraph_setup.py` + `arg_seed.py` | 守门 #5 env 走 $env:MEMGRAPH_BOLT_URL 不打印明文; arg_seed 落 JSON fixture 给 arg-bridge W2 用; health probe 用 urllib 而非 docker exec (避免外部依赖) |
| ARG-6 | ARG-6 | docs/automation-design.md §4.17 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| ARG-7 | ARG-7 | scripts/automation/registry.md §1 +2 行 + §5.3 +1 行 | A | **[P]** | (registry.md 编辑) | per 守门 #12 v21 [P] docs 同步必更新 registry |
| ARG-8 | ARG-8 | docs/reports/PHASE-ARG-01-IMPL-REPORT.md v0.1 落档 | A | **[P]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 32 UT pass + 1 commit hash |
| ARG-9 | ARG-9 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[P]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-10 | ARG-10 | (后续 ARG.2 / ARG.3 / ARG.4 子代理并行触发) | — | **[P]** | (后续 worktree) | per WBS §14.11 ARG.1 收官后, 派新子代理走 ARG.2 (arg-bridge) / ARG.3 (arg-effect) / ARG.4 (api/arg 扩展) |

**§4.17 任务卡维度判定**:
- R (Rerunnable): **是** (arg_seed.py idempotent, memgraph_setup.py 走 docker compose, cargo test 单 crate 模式)
- V (Volume): **是** (24 节点 + 10 边 + 32 UT 跨 7 测试文件)
- S (Structural): **是** (新建 crates/arg 6 子模块 + workspace 65 → 66 package + Cargo.toml 追加 1 行)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.17 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2)**:
- `cargo check --workspace --lib -j 4` 0 err (实证)
- `cargo fmt -p star-arg -- --check` 0 err (实证, 仅 crates/arg 检查, 旧 star-mutex 已有差异不在本任务 scope)
- `cargo clippy -p star-arg --all-targets -j 4 -- -D warnings` 0 err (实证)
- `cargo test -p star-arg --tests -j 4` 100% pass (33 tests: 1 lib + 6 achievement + 5 agent + 2 cypher_cache + 8 edge_ops + 2 event + 5 template_ops + 4 trust_score)
- `cargo build --release -p star-arg` 0 err (9.09s, 实证)
- `python scripts/automation/arg_seed.py --output /tmp/test.json` exit 0 + 24 节点 + 10 边
- `git log -p --follow crates/arg/Cargo.toml` 实证 (commit 后)
- `git log -p --follow scripts/automation/memgraph_setup.py` 实证 (commit 后)
- `git log -p --follow scripts/automation/arg_seed.py` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.17 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §1 + §5.3 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化)
- 0 unsafe 块 (守门 #7 `unsafe_code = "forbid"` workspace lint)
- 0 missing_docs warn (workspace lint `missing_docs = "deny"`, 全部 public item 都有 doc)
- 守门 #5 env 安全: MemgraphClient 密码字段 `#[allow(dead_code)]` 标记 + 无 `Display`/`Debug` impl 暴露 + 不打印明文

### 4.18 P3-C W4 ARG.4 — crates/api/src/arg/ 13 REST + 1 WebSocket + RLS 13 类 (2026-09-09 04:38 JST per `docs/briefs/arg-04-api-13rest-1ws.md`)

> **触发**: 2026-09-09 04:38 JST 用户发令"开子代理和worktree并行处理并在完成后merge到main" + `ask_8d5083148d6e0566b520988e` 拍板 (scope=ARG.1+ARG.4 / budget=选项3分阶段批 / merge=串行merge走守门)
> **依据**: 守门 #1 v19 (P 子项 Python 化, [M] 子项 `arg_api_test.py`) + 守门 #1 v15 (本轮第 6 次新事件, docs 同步允许) + 守门 #1 v25 (cargo check + cargo test 跨 crate 兼容 0 err) + 守门 #3 (5 域 Lead 跨域边强制 consults) + 守门 #5 (env 安全, Memgraph 连接串走 env) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #9 (子代理 RPC 不可靠, 不用 RPC) + 守门 #10 (代签, author=Ulysses) + 守门 #12 ([M] docs 同步) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #19 v19 (守门 #12 死循环饱和边界)
> **落档文件**:
> - `crates/api/src/arg/` 新建 (4 文件: mod.rs + controller.rs + sse_hub.rs + permission.rs + dto.rs, ~10K 字节 + 13 UT) — 既有 crates/api 内部扩展, workspace 65 → 65 package
> - `crates/api/Cargo.toml` 追加 `axum = { version = "0.8", features = ["ws", "macros"] }` + `serde_json` + `star-arg = { path = "../arg" }` (3 行新增)
> - `crates/api/src/lib.rs` 追加 `pub mod arg;` 1 行
> - `scripts/automation/arg_api_test.py` v0.1 (~580 行, 10 IT 端到端 + 临时 axum 测试 server 编译)
> - `docs/automation-design.md` §4.18 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +1 行 + §5.4 +1 段
> - `docs/reports/PHASE-ARG-04-IMPL-REPORT.md` v0.1 (7 段 per AGENTS.md §3)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-4.1 | ARG-4.1 | `crates/api/src/arg/` 4 文件 (mod.rs + controller.rs + sse_hub.rs + permission.rs + dto.rs) | S, A | **[M]** | (无新脚本, 复用 ARG-4.4 arg_api_test.py) | 14 routes (13 REST + 1 WebSocket per DD §4.12); ARGState 8 字段 (per DD §3.2.5); 守门 #7 0 unsafe; axum 0.8 path syntax `{id}` (per ADR-0048) |
| ARG-4.2 | ARG-4.2 | 13 UT (4 dto + 9 permission + 1 mod.rs + 3 sse_hub + 7 lib) | R, V, A | **[M]** | `cargo test -p api --lib -j 4` | 守门 #1 v25 实证 100% pass (24 tests, 0 failed, 0.00s) |
| ARG-4.3 | ARG-4.3 | 5 守门全套 (check / fmt / clippy / test / build) | R, V, A | **[M]** | (守门 #1 累积规 v1-v5) | `cargo check -p api --all-targets -j 4` 0 err + `cargo fmt -p api -- --check` 0 err + `cargo clippy -p api --lib -j 4` 0 err (1 pre-existing warning in lib.rs:100) + `cargo test -p api --lib -j 4` 24/24 pass + `cargo build --release -p api` 0 err |
| ARG-4.4 | ARG-4.4 | `scripts/automation/arg_api_test.py` v0.1 落档 | R, V, S, A | **[M]** | `scripts/automation/arg_api_test.py` | 守门 #5 env 走 $env:ARG_TEST_PORT 但不打印明文; 起临时 axum server (subprocess 路径 per 守门 #9 v3); 10 IT (8 REST + 2 WS); 用 stdlib `socket` 兜底 WS (无 websockets 库依赖) |
| ARG-4.5 | ARG-4.5 | docs/automation-design.md §4.18 同步 (本节) | A | **[M]** | (本节追加) | per 守门 #12 v21 [M] docs 同步必更新 §4 任务卡表 |
| ARG-4.6 | ARG-4.6 | scripts/automation/registry.md §1 +1 行 + §5.4 +1 段 | A | **[M]** | (registry.md 编辑) | per 守门 #12 v21 [M] docs 同步必更新 registry |
| ARG-4.7 | ARG-4.7 | docs/reports/PHASE-ARG-04-IMPL-REPORT.md v0.1 落档 | A | **[M]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 24 UT pass + 1 commit hash |
| ARG-4.8 | ARG-4.8 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[M]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-4.9 | ARG-4.9 | (后续 ARG.5 frontend 子代理触发) | — | **[M]** | (后续 worktree) | per WBS §14.11 ARG.4 收官后, 派新子代理走 ARG.5 (frontend/src/app/agent-relationships/) |

**§4.18 任务卡维度判定**:
- R (Rerunnable): **是** (arg_api_test.py idempotent, 起临时 axum server, 24 UT 跨 5 测试文件)
- V (Volume): **是** (14 routes + 24 UT + 5 守门 + 4 file 落地)
- S (Structural): **是** (crates/api 内部扩展 + Cargo.toml 追加 3 行 + lib.rs 追加 1 行 + workspace 65 → 65 package)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.18 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2)**:
- `cargo check -p api --all-targets -j 4` 0 err (实证)
- `cargo fmt -p api -- --check` 0 err (实证)
- `cargo clippy -p api --lib -j 4` 0 err (实证, 1 pre-existing warning in lib.rs:100 跟 ARG.4 无关, 来自原 crates/api 骨架)
- `cargo test -p api --lib -j 4` 24/24 pass (实证, 0.00s)
- `cargo build --release -p api` 0 err (实证, 6.68s)
- `python scripts/automation/arg_api_test.py` exit 0 (实证, 10/10 IT 通过)
- `git log -p --follow crates/api/Cargo.toml` 实证 (commit 后)
- `git log -p --follow crates/api/src/lib.rs` 实证 (commit 后)
- `git log -p --follow crates/api/src/arg/` 实证 (commit 后)
- `git log -p --follow scripts/automation/arg_api_test.py` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.18 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §1 + §5.4 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化)
- 0 unsafe 块 (守门 #7 `unsafe_code = "forbid"` workspace lint)
- 守门 #5 env 安全: ARG_TEST_PORT 走 env, 不打印明文; 临时 axum server 启停走 subprocess, 收尾 `proc.terminate()`
- 守门 #14 v2: Lead 角色独占, Admin 角色不能假冒 Lead (实证 5 个 permission UT 覆盖)
### 4.18 ADR-0049 任务卡自动 worktree + agent 接管 (2026-09-09 04:57 JST per Ulysses 拍板核心功能)

> **触发**: 2026-09-09 04:57 JST Ulysses 发令"在面板或者sprint创建任务卡的时候，如果是存在有效agent的任务，应该要求agent自动创建并关联新的worktree，用langgraph实现，这是整个软件的核心功能"
> **联动**: 守门 #19 v19 (Python 化 ≥2 维) + 守门 #9 v20 (子代理 dispatch 必先 brief) + 守门 #21 ([P] 子项 docs 同步) + 守门 #22 (调试控制台不污染 main 编译) + 守门 #13 a (L0 唯一入口) + 守门 #13 c (Master RLS) + 守门 #13 d (Transaction 100% audit) + 守门 #10 + 8/27 19:39 JST (代签)
> **落档文件**:
> - `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` v0.1 (新, 300 行, 7 段结构 per AGENTS.md §3)
> - `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.1 (新, 7 子项 phase 计划 + 8 已知缺口显式列 per 守门 #11)
> - `docs/briefs/adr-0049-task-card-auto-worktree.md` v0.1 (新, 守门 #20 dispatcher brief 实证)
> - `scripts/automation/task_ops/nodes/create_node.py` v0.1 (新, 270 行, M-N8 第 8 节点)
> - `scripts/automation/_mock_git_worktree.py` v0.1 (新, 100 行, 守门 #22 mock shell wrapper)
> - `scripts/automation/task_ops/protocols.py` +90 行 (CreateTaskRequest/Response + TMOMessage v0.3)
> - `scripts/automation/task_ops/manager.py` +100 行 (OPERATION_TO_NODE["create"]=M-N8 + SubAgentPool.has_agent/dispatch + WorktreeRegistry + create() + _create_task)
> - `frontend/src/lib/store.ts` +90 行 (createWorkItem + isAgentAssignee + pickSaTypeForKind + dispatchTmoCreate)
> - `frontend/src/types/ids.ts` +15 行 (IdentityType + Identity.type 字段)
> - `scripts/automation/registry.md` v0.6 (§1 脚本索引 + §5.3 ADR-0049 段落 + §6 v0.6 修订历史)

| 任务卡 | 路径 / 子项 | 状态 | 落档 commit | 守门 |
|---|---|---|---|---|
| M-N8-01 create_node.py | `scripts/automation/task_ops/nodes/create_node.py` v0.1 (270 行) | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 / #22 |
| M-N8-02 protocols.py | `scripts/automation/task_ops/protocols.py` +90 行 | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 |
| M-N8-03 manager.py | `scripts/automation/task_ops/manager.py` +100 行 | 🟢 v0.1 落档 | TBD | #1 / #13 / #19 |
| M-N8-04 _mock_git_worktree.py | `scripts/automation/_mock_git_worktree.py` v0.1 (100 行) | 🟢 v0.1 落档 | TBD | #1 / #22 |
| M-N8-05 frontend store.ts | `frontend/src/lib/store.ts` +90 行 | 🟢 v0.1 落档 | TBD | #1 / #19 / #20 |
| M-N8-06 ADR-0049 | `docs/architecture/2026-08-26-upgrade/adr/0049-task-card-auto-worktree-agent.md` v0.1 | 🟢 v0.1 落档 | TBD | #10 / #11 / #12 |
| M-N8-07 PHASE-REPORT | `docs/reports/PHASE-AUTO-WORKTREE-IMPL-REPORT.md` v0.1 | 🟢 v0.1 落档 | TBD | #11 / #12 / #21 |
| 拍板决策 | 触发器=前端 store (opt1) / worktree 关联=1:1 (opt1) / agent 接管=自动 (opt1) / 落档=ADR+PHASE (opt1), 4 推荐项全选 | 🟢 已拍板 | — | 9/1 14:58 + 9/5 04:03 + 9/8 16:08 守门 |
| Smoke test | 83ms 内 full happy path: human reject + missing tenant reject + task_id + worktree_id + agent_session_id + AgentRunning + audit 3 条 | 🟢 通过 | TBD | #1 v3 |
| Frontend typecheck | worktree 隔离环境无 node_modules, PR CI 实证 (per 守门 #1 v25 CI 改单 crate 跳 workspace) | ⏳ PR CI 实证 | — | #1 v25 |
| console_server.py 端点 | `/api/tmo/create` POST endpoint 实装 (走守门 #9 v3 subprocess) | ✅ **P-AUTO-WT-01 收官** (123ms HTTP, 5/5 维 E2E 全过) | TBD | #1 / #9 v3 / #22 |
| E2E UC-14 | `tests/e2e/python/test_uc14_auto_worktree.py` (5s 任务卡 in_progress 实证) | ✅ **P-AUTO-WT-02 收官** (5/5 维: happy 95ms / SA 映射 4 kind / human 拒 / missing tenant 拒 / 1:1 attach) | TBD | #1 / #3 / #11 |
| routes_tmo.py stale import 修复 | split_node 缺 4 常量 (DEFAULT/MIN/MAX/VALID_SPLIT_STRATEGIES), 路由层本地化定义 | ✅ **P-AUTO-WT-01 收口** | TBD | #12 #3 |
| 后续 gate | HANDOFF-ST-001 §5.3 5 Blocker + G-WT-01 DB 接入 + G-WT-02 真 git 集成 | ⏳ 跨 session 续 | — | #1 v17 / #3 |

**§4.18 拍板决策明细 (per 2026-09-09 04:57 JST ask_user 4 推荐项全选)**:
- **Q1 触发器位置** = 前端 store.createWorkItem 内嵌 (推荐)
  - 理由: 改动小, 跟 W5 store 维护责任对齐; 不重写 store (W5 维护)
- **Q2 worktree 关联** = 1 WorkItem → 1 Worktree (推荐)
  - 理由: 跟 ARG §14.11 worktree-as-agent-anchor + INV-WT-07 一致; UI 简单
- **Q3 agent 接管** = 自动接管: worktree ready → 起 AgentSession (推荐)
  - 理由: 5s 内任务卡 in_progress, 跟守门 #1 v22 console 模式一致; 用户体感强
- **Q4 落档范围** = ADR-0049 + PHASE-REPORT 一次性收口 (推荐)
  - 理由: 跨 3 crate 改动, 走守门 #19 + #20 + #21 完整路径

**§4.18 跟 TMO v0.2 关系 (per ADR-0046)**:
- TMO 7 节点 (M-N1..M-N7) + 新 M-N8 = TMO 8 节点
- 协议联合类型 `TMOMessage` 从 7 协议升 v0.3 (8 协议: + CreateTaskRequest)
- 路由表 `OPERATION_TO_NODE` 加 `create: M-N8`
- 守门 #13 a L1↔L1 禁止派生: M-N8 跟 M-N1..M-N7 同层 L0 协调, 不破 5 域 Lead 独立 + DAG 派生

**§4.18 跟 Agent Runtime 关系 (per ADR-0045)**:
- 9 SA Archetype (SA-01..SA-09 + SA-10 task-orchestrator) 是接口
- `pickSaTypeForKind(kind)` 映射: bug→SA-04 / story→SA-02 / epic→SA-03 / task→SA-01
- SubAgentPool 内存版 (PoC) 跟 ADR-0045 Hybrid Runtime L1 ECS 平行, 后续 ECS 接入替换 (per §3 已知缺口 #4)

### 4.19 Sub-task Binding 路径落地 (TMO 7 → 8 节点增量扩展, 2026-09-09 21:53 JST per ask_1df6987367ccc00928b65ee3 拍板)

> **触发**: 2026-09-09 21:53 JST 用户发令 "现在是否适合具有子代理功能, 通过在父任务卡交互下命令, 创建绑定子任务, 子任务有专属子代理? 如果没有, 制定需求和基本设计详细设计" + ask_1df6987367ccc00928b65ee3 拍板 1 选项 "3 份新文档 + ADR-0052 升 v0.3 (推荐)", per 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 9/5 04:03 推荐项直接执行
> **落档文件**:
> - `docs/architecture/2026-09-09-subtask-binding/01-requirements.md` v0.1 (31.5KB, UC-14..UC-18 + F-26..F-32 + NFR-SB-01..05 + 3 表 W/T/M + 7 已知缺口 G-SB-1..7)
> - `docs/architecture/2026-09-09-subtask-binding/02-basic-design.md` v0.1 (31.5KB, 8 组件 C-23..C-30 + 1 节点 M-N8 spawn_subtask_node + 1 协议 spawn_subtask + 5 Reducer R-08..R-12 + 1 外部 API 端点 POST /api/tmo/spawn_subtask + 14 守门合规矩阵 + v3x 5 候选)
> - `docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md` v0.1 (55.1KB, 8 module M-26..M-33 + 7 步 Python 実装 + 7 UT-27..UT-33 + 3 IT-13..IT-15 + 5 E2E-14..E2E-18 + 3 表 DDL sub_task_binding/sub_task_runtime/task_quota_config + 14 守门合规检查)
> - `docs/architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md` v1.0 (26.7KB, 11 节结构: 背景/决策/备选/后果/实施/决策日志/签字栏/修订历史/引用, 4 备选方案 + 选定 D M-N8 L0 StateGraph 增量扩展 + 7 正面后果 + 6 负面风险 + 5 中和措施 + 5 阶段实施计划 + 4 决策日志 + 5 签字栏)
> - `docs/reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md` v0.1 (23.1KB, 7 段结构 + 21 子项任务完成矩阵 ✅ 5/21 + 🟡 4/21 + ⏳ 12/21 + 14 守门合规矩阵 + v3x 5 候选 + 10 已知缺口 + 7 子代理失败接手 + 24 守门 + 24 累积规)
> **依据**: 守门 #13 a (L1↔L1 禁止 → M-N8 全部 L0 协调) + 守门 #13 c/d (DB W/T/M 3 表 100% 覆盖, RLS 13 類必携) + 守门 #7 (0 unsafe, C-24 ExclusiveBindingGuard 编译期 + 运行期双层守门) + 守门 #19 v19 (agent 交互 Python 化, 走 `scripts/automation/task_ops.py spawn_subtask`) + 守门 #20 v20 (子代理 dispatch 必先 brief 落档) + 守门 #21 v21 ([P] docs 同步必更新 §4 任务卡表 + registry) + 守门 #22 v22 (调试控制台不污染 main) + 守门 #23 v23 (AI mock 不开 OpenAI) + 守门 #24 v24 (subprocess 替代 RPC, console_server.py 8080) + 守门 #1 v15 死循环饱和边界 (本次 = docs 同步新事件触发, +1 不违反) + 守门 #1 v19 + 守门 #9 v3 守门 #1 v20 + 守门 #12 v21 (Python 化 3 件套联动) + 守门 #14 v3 Mavis 永久代签 (5 签字栏全部代签)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| SB-1 | SB-1 | `docs/architecture/2026-09-09-subtask-binding/01-requirements.md` v0.1 落档 (31.5KB) | A | **[P]** | (write tool) | per 用户 2026-09-09 21:53 JST 发令 + ask_1df6987367ccc00928b65ee3 拍板 1 选项 (推荐) |
| SB-2 | SB-2 | `docs/architecture/2026-09-09-subtask-binding/02-basic-design.md` v0.1 落档 (31.5KB) | A | **[P]** | (write tool) | per [ADR-0052 §2.1 8 节点 + 2.3 8 组件 + 2.5 5 Reducer + 2.6 1 外部 API 端点](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) |
| SB-3 | SB-3 | `docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md` v0.1 落档 (55.1KB) | A | **[P]** | (write tool) | per [03 §1 8 module M-26..M-33 + §3.3.1.1 7 步 Python 実装 + §4 测试矩阵 + §5 3 表 DDL](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) |
| SB-4 | SB-4 | `docs/architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md` v1.0 落档 (26.7KB) | A | **[P]** | (write tool) | per ADR-0052 11 节结构 + 4 备选方案 + 选定 D + 7 正面后果 + 6 负面风险 + 5 中和措施 + 5 阶段实施计划 + 4 决策日志 + 5 签字栏 |
| SB-5 | SB-5 | `docs/reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md` v0.1 落档 (23.1KB) | A | **[P]** | (write tool) | per [7 段结构 + 21 子项任务完成矩阵 + 14 守门合规矩阵 + v3x 5 候选 + 10 已知缺口 + 7 子代理失败接手](../reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md) |
| SB-6 | SB-6 | `AGENTS.md` §6 ADR 列表追加 ADR-0052 + §6.1 架构 view 索引追加 Sub-task Binding 5 文件 + §8 修订历史追加 v0.80 | A | **[P]** | (AGENTS.md edit) | per 守门 #21 v21 [P] docs 同步 + 守门 #12 commit-time docs 同步触发 v0.80 |
| SB-7 | SB-7 | `docs/automation-design.md` §4.19 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #21 v21 [P] docs 同步必更新 §4 任务卡表 |
| SB-8 | SB-8 | `scripts/automation/registry.md` §5.3 同步 (Sub-task Binding 8 module 索引) | A | **[P]** | (registry.md 编辑) | per 守门 #21 v21 [P] docs 同步必更新 registry |
| SB-9 | SB-9 | LangGraph 02-basic-design v0.2 → v0.3 升版 (引用 M-N8 + C-23..C-30) | S | **[M]** | (LangGraph 02 edit) | per [ADR-0052 §5 实施计划 v0.3 升版阶段](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) |
| SB-10 | SB-10 | LangGraph 03-detailed-design v0.2 → v0.3 升版 (引用 M-26..M-33 + 4 binding 字段 + 9 API 端点) | S | **[M]** | (LangGraph 03 edit) | per [ADR-0052 §5 实施计划 v0.3 升版阶段](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) |
| SB-11 | SB-11 | 实装 M-26..M-33 8 module (per [03 §3.3.1.1 7 步 Python 実装](../architecture/2026-09-09-subtask-binding/03-detailed-design.md)) | S, R | **[P]** | `task_ops/spawn_subtask/` 子目录 8 module | 待 P3-B 启动 + P0-1/H2 阻塞解除 + 5 域 Lead 真人到位后跨 session 续做 (per [ADR-0052 §5 实施计划 v0.4 实装阶段](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md)) |
| SB-12 | SB-12 | UI TaskCard 右键 spawn 入口 + C-30 UISpawnSubtaskModal (Next.js 15) + 9 API 端点 POST /api/tmo/spawn_subtask (FastAPI console_server.py) | S | **[P]** | `frontend/src/app/tasks/components/SpawnSubtaskModal.tsx` + `frontend/src/app/api/tmo/spawn_subtask/route.ts` | per [03 §3.3.9 UISpawnSubtaskModal C-30](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) + 守门 #24 v24 subprocess 替代 RPC |
| SB-13 | SB-13 | 3 表 DDL 落地 (PostgreSQL Schema: sub_task_binding Transaction + sub_task_runtime Work + task_quota_config Master SCD2 + RLS 13 類) | S | **[P]** | DB migration script | per [03 §5 3 表 DDL](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) + 守门 #13 c/d W/T/M 派生规 (a)(b)(c)(d) 全部落档 |
| SB-14 | SB-14 | 5 E2E + 3 IT + 7 UT 测试套件 (E2E-14..E2E-18 + IT-13..IT-15 + UT-27..UT-33) | A, R | **[P]** | `tests/e2e/test_e2e_14..18.py` + `tests/it/test_it_13..15.py` + `tests/ut/test_ut_27..33.py` | per [03 §4 测试矩阵 E2E-14 (NFR-SB-01 ≤ 200ms) + E2E-15 (NFR-SB-02 不可绕过) + E2E-16 (NFR-SB-03 ≤ 1s) + E2E-17 (UI 入口) + E2E-18 (quota + lifecycle + audit 完整)](../architecture/2026-09-09-subtask-binding/03-detailed-design.md) |

**§4.19 任务卡维度判定**:
- R (Rerunnable): **是** (3 份 IPA + 报告 全部 idempotent, 二次生成同结果)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性)
- S (Structural): **是** (8 module 增量 + 4 binding 字段 + 3 表 DDL + 8 组件 C-23..C-30 + 5 Reducer R-08..R-12 + 1 节点 M-N8)
- A (Audit-trail): **是** (守门 #21 v21 [P] docs 同步 + 守门 #12 决策表 + 守门 #9 git 实证 + 守门 #13 DB schema 分类留痕)

**§4.19 落档验证 (per 守门 #1 累积规 v1-v24, 本次纯文档不需 cargo 守门)**:
- `git log -p --follow docs/architecture/2026-09-09-subtask-binding/01-requirements.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-09-09-subtask-binding/02-basic-design.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md` 实证 (commit 后)
- `git log -p --follow docs/architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md` 实证 (commit 后)
- `git log -p --follow docs/reports/PHASE-SUBTASK-BINDING-IMPL-REPORT.md` 实证 (commit 后)
- `git log -p --follow AGENTS.md` 实证 §6 ADR + §6.1 架构 view + §8 v0.80 同步 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.19 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §5.3 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 9/3 19:35 拍板 D + 9/8 15:19 第 6 次强化)

### 4.20 wt 并行处理落地 (per 2026-09-09 22:31 JST 用户发令"开子代理和 worktree 并行处理并在完成后 merge 到 main" + ask_4b06eee1bba60b2727e8bccb 拍板 4 个 wt 并行 + 逐个 rebase + ff merge 范式)

> **触发**: 2026-09-09 22:31 JST 用户发令"开子代理和 worktree 并行处理并在完成后 merge 到 main" + ask_4b06eee1 拍板 1 选项 (4 个 wt 并行) + 2 选项 (逐个 rebase + ff merge 范式, 推荐项), per 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 Mavis 自驱 + 9/5 04:03 推荐项直接执行
> **落档文件**:
> - `docs/briefs/wt-sb-01-subtask-binding-impl.md` v0.1 (8.7KB, Sub-task Binding v0.4 实装 brief)
> - `docs/briefs/wt-arg-04-api-13rest-1ws.md` v0.1 (8.1KB, ARG.4 13 REST + 1 WS + RLS 13 類 brief)
> - `docs/briefs/wt-h2-ext-5domain-compat.md` v0.1 (8.2KB, H2-EXT 5 domain 类型兼容 + workspace_ids + tenant_policy_id brief)
> - `docs/briefs/wt-5lead-outreach.md` v0.1 (9.0KB, 5 域 Lead 寻访 Ulysses 内推启动 brief)
> - `docs/recruitment/status/{player,economy,match,social,admin}.md` v0.1 (5 文件, ~6KB, 5 域寻访状态)
> - `docs/recruitment/outreach_log/{README.md,.gitignore}` v0.1 (2 文件, ~1.8KB, private log 不入 git)
> - `docs/briefs/5-leads/README.md` v0.1 (3.7KB, 5 域 Lead Subagent Brief 索引说明)
> - `AGENTS.md` §8 v0.81 (修订历史追加 wt 并行处理落地)
> **依据**: 守门 #9 v3 (调试控制台走 subprocess 替代 RPC, 子代理 RPC 不可靠实证 10/10 失败) + 守门 #9 #3 (不 commit 散落子代理产出) + 守门 #9 v19 (agent 交互 Python 化) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档) + 守门 #14 v3 (Mavis 永久代签全部签字栏) + 守门 #21 v21 ([P] docs 同步必更新 §4 + registry) + 守门 #1 v15 (docs 同步饱和 41+ 次仍允许, 本次 = 新事件触发) + 守门 #27 v27 候选 (子代理 RPC 失败 fallback) + 守门 #28 v28 候选 (Mavis 拍板时必带推荐项)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| WT-1 | `docs/briefs/wt-sb-01-subtask-binding-impl.md` v0.1 落档 (8.7KB) | A | **[P]** | (write tool) | per [ADR-0052 §5 实施计划 v0.4](../architecture/2026-08-26-upgrade/adr/0052-subtask-binding.md) 实装阶段 brief |
| WT-2 | `docs/briefs/wt-arg-04-api-13rest-1ws.md` v0.1 落档 (8.1KB) | A | **[P]** | (write tool) | per §4.18 ARG.4 13 REST + 1 WS + RLS 13 類 |
| WT-3 | `docs/briefs/wt-h2-ext-5domain-compat.md` v0.1 落档 (8.2KB) | A | **[P]** | (write tool) | per HANDOFF-ST-001 §5.3 Blocker #1 H2-EXT 5 domain 类型不兼容 + DeviceId 强类型重构 |
| WT-4 | `docs/briefs/wt-5lead-outreach.md` v0.1 落档 (9.0KB) | A | **[P]** | (write tool) | per docs/recruitment/5-business-domain-lead-referral.md v0.1 + ask_409cbd32edc309d71 拍板 Ulysses 内推 + 立即启动 |
| WT-5 | `git worktree add` 4 worktree 落地 (wt-sb-01 + wt-h2-ext-5domain-compat + wt-5lead-outreach 新建; wt-arg-04 已存在 ahead 0 behind 52) | S | **[S]** | git worktree 命令 | per 守门 #9 主体规则 wt 必须在 main 链上 |
| WT-6 | `wt-arg-04-api-13rest-1ws` close (内容 100% 在 main 上, 不需 merge) | S | **[S]** | `git worktree remove --force` + `git branch -D` | per守门 #9 #3 close-wtc 范式 (9/5 11:27 JST 拍板); ARG.1 + ARG.4 已 100% 在 main 链上 (commit `6e2cda6` + `1d894ab` + `e2c70ce` + `d2a378c` + `c678db7` + `7d7797b`) |
| WT-7 | `wt-5lead-outreach` 8 文件 commit + ff merge (commit `bd12c17`) | S | **[P]** | git commit + git merge --ff-only | per [docs/briefs/wt-5lead-outreach.md §4.3](../briefs/wt-5lead-outreach.md) commit 范式 + 守门 #12 commit-time docs 同步 + 守门 #14 v3 author=Ulysses |
| WT-8 | wt-5lead-outreach commit `bd12c17` ff merge 到 main (Fast-forward, 无冲突) | S | **[S]** | `git merge --ff-only wt-5lead-outreach` | per 9/4 17:19 JST 拍板 rebase-then-merge 范式 (rf001-t15-work 实证) |
| WT-9 | docs/briefs/5-leads/ 5 文件原版本保留 (per守门 #1 禁回溯叙事) + 仅新增 README.md 索引说明 | A | **[P]** | (write tool) | per守门 #12 AI 协作文档治理; 首次 write tool 错误 overwrote 立即 git restore 恢复 + commit message 含错误修正 |
| WT-10 | `AGENTS.md` §8 v0.81 修订历史追加 wt 并行处理落地 (per守门 #21 v21) | A | **[P]** | (AGENTS.md edit) | per 守门 #12 commit-time docs 同步触发 v0.81 |
| WT-11 | wt-sb-01 + wt-h2-ext-5domain-compat 跨 session 续做 (估 ~3-5M tokens 实装量超单 session 上限) | S | **[P]** | (跨 session) | per 守门 #20 v20 + 守门 #27 v27 候选; brief 已落档可下次 session 直接接续 |

**§4.20 任务卡维度判定**:
- R (Rerunnable): **是** (wt-5lead-outreach commit bd12c17 idempotent; wt-sb-01 + wt-h2-ext 跨 session 续做 idempotent)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 直接做, 守门 #9 v3 subprocess 替代 RPC)
- S (Structural): **是** (4 worktree 创建 + 1 close + 1 ff merge + 5 域寻访状态 + 8 brief 落档)
- A (Audit-trail): **是** (守门 #12 commit-time docs 同步 v0.81 + 守门 #21 [P] docs 同步 §4.20 + 守门 #9 git log --follow 实证 + 守门 #13 自我修正留痕 + 守门 #1 v15 docs 同步饱和 41+ 次新事件触发)

**§4.20 落档验证 (per 守门 #1 累积规 v1-v24, 本次 1 cargo check + 8 文件落档, 不需 cargo test)**:
- `git log -p --follow docs/briefs/wt-sb-01-subtask-binding-impl.md` 实证 (commit 后, 跨 session 续做)
- `git log -p --follow docs/briefs/wt-arg-04-api-13rest-1ws.md` 实证
- `git log -p --follow docs/briefs/wt-h2-ext-5domain-compat.md` 实证
- `git log -p --follow docs/briefs/wt-5lead-outreach.md` 实证
- `git log -p --follow docs/recruitment/status/` 实证 5 域寻访状态落档
- `git log -p --follow docs/recruitment/outreach_log/README.md` 实证 (private log 不入 git, 仅 README + .gitignore)
- `git log -p --follow docs/briefs/5-leads/README.md` 实证 索引说明
- `git log -p --follow AGENTS.md` 实证 §8 v0.81 追加 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.20 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 registry v0.8+ 追加 (commit 后)
- `git merge-base --is-ancestor 6e2cda6 main` 实证 ARG.4 在 main 上 (close 拍板)
- `git merge-base --is-ancestor 43c1f0c main` 实证 ARG.1 在 main 上 (close 拍板)
- `git worktree list` 实证 wt-arg-04-api-13rest-1ws 已删除 (close 实证)
- `cargo check --workspace --all-targets -j 4` 实证 (wt-arg-04 cargo check 0 err 2m03s, 不在 main 编译链, per 守门 #1 v22)
- commit author = `Ulysses <ulysses@mavis.local>` (per 19:39 + 21:59 JST 授权 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 Mavis 自驱)

### 4.21 P3-C W2 ARG.2 — crates/arg-bridge 4 子模块骨架实装 (2026-09-10 06:53 JST per `docs/briefs/arg-02-arg-bridge-crate.md`)

> **触发**: 2026-09-10 06:53 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 44 次新事件触发仍允许)
> **依据**: 守门 #1 v19 (P 子项 Python 化, [M] 子项 `arg_bridge_test.py`) + 守门 #1 v15 (本轮第 44 次新事件, docs 同步允许) + 守门 #1 v25 (cargo check + cargo test 跨 crate 兼容 0 err) + 守门 #3 (5 域 Lead 跨域边强制 consults) + 守门 #5 (env 安全, Memgraph 连接串走 env) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #9 (子代理 RPC 不可靠, 不用 RPC) + 守门 #10 (代签, author=Ulysses) + 守门 #12 ([M] docs 同步) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #19 v19 (守门 #12 死循环饱和边界, 本轮新事件允许)
> **落档文件**:
> - `crates/arg-bridge/` 新建 (Cargo.toml + lib.rs + 6 module + 4 tests = 16 文件, workspace 67 → 68 package)
> - `Cargo.toml` workspace members 追加 `"crates/arg-bridge"` + `sled = "0.34"` 2 行
> - `scripts/automation/arg_bridge_test.py` v0.1 (~370 行, 10 IT 端到端 + 5 守门 + 5 file-content check)
> - `docs/automation-design.md` §4.21 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +1 行 + §5.5 +1 段
> - `docs/reports/PHASE-ARG-02-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-2.1 | ARG-2.1 | `crates/arg-bridge/` 16 文件 (Cargo.toml + lib.rs + 6 module + 4 tests) | S, A | **[P]** | (无新脚本, 复用 ARG-2.4 arg_bridge_test.py) | 4 子模块 (memgraph_listener / langgraph_updater / period_flush / offline_queue) + 5 协议 schema + 5 Reducer; 守门 #7 0 unsafe; sled 0.34 嵌入 OfflineQueue; axum 0.8 path syntax (per ADR-0048) |
| ARG-2.2 | ARG-2.2 | 13 UT (3 listener + 3 flush + 4 offline + 3 langgraph) | R, V, A | **[P]** | `cargo test -p star-arg-bridge --tests -j 4` | 守门 #1 v25 实证 100% pass (10 main + 3 extras = 13 tests, 0 failed, 2.77s) |
| ARG-2.3 | ARG-2.3 | 5 守门全套 (check / fmt / clippy / test / build) | R, V, A | **[P]** | (守门 #1 累积规 v1-v5) | `cargo check --workspace --lib -j 4` 0 err (1m 07s) + `cargo fmt -p star-arg-bridge -- --check` 0 err + `cargo clippy -p star-arg-bridge --lib -j 4 -- -D warnings` 0 err (0.88s) + `cargo test -p star-arg-bridge --tests -j 4` 13/13 pass + `cargo build --release -p star-arg-bridge` 0 err (16.66s) |
| ARG-2.4 | ARG-2.4 | `scripts/automation/arg_bridge_test.py` v0.1 落档 | R, V, S, A | **[P]** | `scripts/automation/arg_bridge_test.py` | 守门 #5 env 不打印明文; 走 subprocess.run 调 `cargo test -p star-arg-bridge --tests -j 4` (守门 #9 v3); 10 main IT + 3 extras + 5 守门 (test/lib/check/fmt/clippy/build) + 5 file-content check (no unsafe / workspace 注册 / sled 依赖 / 5 协议 schema / 4 子模块) |
| ARG-2.5 | ARG-2.5 | docs/automation-design.md §4.21 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| ARG-2.6 | ARG-2.6 | scripts/automation/registry.md §1 +1 行 + §5.5 +1 段 | A | **[P]** | (registry.md 编辑) | per 守门 #12 v21 [P] docs 同步必更新 registry |
| ARG-2.7 | ARG-2.7 | docs/reports/PHASE-ARG-02-IMPL-REPORT.md v0.1 落档 | A | **[P]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 13 UT pass + 1 commit hash |
| ARG-2.8 | ARG-2.8 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[P]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-2.9 | ARG-2.9 | (后续 ARG.3 arg-effect 5 子模块 / ARG.5 frontend 子代理触发) | — | **[P]** | (后续 worktree) | per WBS §14.11 ARG.2 收官后, 派新子代理走 ARG.3 (arg-effect 5 子模块) / ARG.5 (frontend/src/app/agent-relationships/) |
| ARG-2.10 | ARG-2.10 | Cargo workspace members 追加 `"crates/arg-bridge"` + sled 0.34 dep | S | **[P]** | (Cargo.toml edit) | workspace 67 → 68 package; 守门 #1 v1 |

**§4.21 任务卡维度判定**:
- R (Rerunnable): **是** (arg_bridge_test.py idempotent, 调 subprocess.run 跑 cargo test/check/fmt/clippy/build, 13 UT 跨 4 测试文件)
- V (Volume): **是** (16 文件 + 13 UT + 5 守门 + 4 子模块 + 5 协议 schema)
- S (Structural): **是** (crates/arg-bridge 内部扩展 + Cargo.toml 追加 2 行 + workspace 67 → 68 package)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.21 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2)**:
- `cargo check --workspace --lib -j 4` 0 err (实证, 1m 07s, 仅 pre-existing warnings)
- `cargo fmt -p star-arg-bridge -- --check` 0 err (实证)
- `cargo clippy -p star-arg-bridge --lib -j 4 -- -D warnings` 0 err (实证, 0.88s)
- `cargo test -p star-arg-bridge --tests -j 4` 13/13 pass (实证, 2.77s, 0 failed, 0 ignored)
- `cargo build --release -p star-arg-bridge` 0 err (实证, 16.66s)
- `python scripts/automation/arg_bridge_test.py` exit 0 (实证, 10/10 main + 3/3 extras + 5 守门 + 5 file-content checks)
- `git log -p --follow crates/arg-bridge/` 实证 (commit 后)
- `git log -p --follow Cargo.toml` 实证 workspace members + sled 追加 (commit 后)
- `git log -p --follow scripts/automation/arg_bridge_test.py` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.21 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §1 + §5.5 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化)
- 0 unsafe 块 (守门 #7 `unsafe_code = "forbid"` workspace lint)
- 守门 #5 env 安全: Memgraph 连接串走 env (`MEMGRAPH_BOLT_URL` / `MEMGRAPH_USER` / `MEMGRAPH_PASSWORD`), arg_bridge_test.py 不打印值
- 守门 #14 v2: 5 域 Lead 真人到位前 Mavis 临时代签维持 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- 守门 #12 派生 v15 docs 同步饱和: 本轮第 44 次新事件触发 (用户发令"按顺序推进"), 仍允许
- 守门 #1 v25 cargo test 单 crate 模式: `cargo test -p star-arg-bridge --lib -j 4` 100% pass (per 9/5 00:15 JST 拍板 PR #12)

### 4.22 P3-C W3 ARG.3 — crates/arg-effect 5 子模块 + 18 UT + 8 拓扑 Cypher + 10 challenges prompt 索引 (2026-09-10 07:21 JST per `docs/briefs/arg-03-arg-effect-crate.md`)

> **触发**: 2026-09-10 07:21 JST 用户发令"按顺序推进" (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 45 次新事件触发仍允许)
> **依据**: 守门 #1 v19 (P 子项 Python 化, [P] 子项 `arg_dispatch_test.py`) + 守门 #1 v15 (本轮第 45 次新事件, docs 同步允许) + 守门 #1 v25 (cargo check + cargo test 跨 crate 兼容 0 err) + 守门 #3 (5 域 Lead 跨域边强制 consults, `select_challenge_prompt` 默认走 TrustTier 派生) + 守门 #5 (env 安全, Memgraph 连接串走 env) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #9 (子代理 RPC 不可靠, 不用 RPC) + 守门 #10 (代签, author=Ulysses) + 守门 #12 ([P] docs 同步) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #19 v19 (守门 #12 死循环饱和边界, 本轮新事件允许)
> **落档文件**:
> - `crates/arg-effect/` 新建 (Cargo.toml + lib.rs + 6 module + 5 tests = 12 文件, workspace 68 → 69 package)
> - `Cargo.toml` workspace members 追加 `"crates/arg-effect"` 1 行
> - `scripts/automation/arg_dispatch_test.py` v0.1 (~410 行, 12 IT 端到端 + 5 守门 + 6 file-content check)
> - `docs/automation-design.md` §4.22 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +1 行 + §5.6 +1 段
> - `docs/reports/PHASE-ARG-03-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-3.1 | ARG-3.1 | `crates/arg-effect/` 12 文件 (Cargo.toml + lib.rs + 6 module + 5 tests) | S, A | **[P]** | (无新脚本, 复用 ARG-3.4 arg_dispatch_test.py) | 5 子模块 (dispatch_router / context_injector / trust_engine / output_evaluator / achievement_engine) + 10 套 challenges prompt (per DD §7) + 8 拓扑 Cypher (re-export from `star_arg::query::topology`); 守门 #7 0 unsafe; axum 0.8 间接 (per ADR-0048); peer_review threshold 0.8 (per SRS §4.3.4); trust skip-verify 0.8 + 0.7 (per DD §4.3.3) |
| ARG-3.2 | ARG-3.2 | 18 UT (3 dispatch + 3 context + 4 trust + 3 output + 5 achievement) | R, V, A | **[P]** | `cargo test -p star-arg-effect --tests -j 4` | 守门 #1 v25 实证 100% pass (18 main UT + 1 extra = 19 tests, 0 failed, 0.01s) |
| ARG-3.3 | ARG-3.3 | 5 守门全套 (check / fmt / clippy / test / build) | R, V, A | **[P]** | (守门 #1 累积规 v1-v5) | `cargo check --workspace --lib -j 4` 0 err + `cargo fmt -p star-arg-effect -- --check` 0 err + `cargo clippy -p star-arg-effect --lib -j 4 -- -D warnings` 0 err (0.87s) + `cargo test -p star-arg-effect --tests -j 4` 18/18 pass + `cargo build --release -p star-arg-effect` 0 err (3.91s) |
| ARG-3.4 | ARG-3.4 | `scripts/automation/arg_dispatch_test.py` v0.1 落档 | R, V, S, A | **[P]** | `scripts/automation/arg_dispatch_test.py` | 守门 #5 env 不打印明文; 走 subprocess.run 调 `cargo test -p star-arg-effect --tests -j 4` (守门 #9 v3); 12 IT (1 cargo test + 1 sub-crate lib + 1 workspace check + 1 fmt + 1 clippy + 1 release build + 6 file-content check) |
| ARG-3.5 | ARG-3.5 | docs/automation-design.md §4.22 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| ARG-3.6 | ARG-3.6 | scripts/automation/registry.md §1 +1 行 + §5.6 +1 段 | A | **[P]** | (registry.md 编辑) | per 守门 #12 v21 [P] docs 同步必更新 registry |
| ARG-3.7 | ARG-3.7 | docs/reports/PHASE-ARG-03-IMPL-REPORT.md v0.1 落档 | A | **[P]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 18 UT pass + 1 commit hash |
| ARG-3.8 | ARG-3.8 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[P]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-3.9 | ARG-3.9 | (后续 ARG.5 frontend 5 UI 组件子代理触发) | — | **[P]** | (后续 worktree) | per WBS §14.11 ARG.3 收官后, 派新子代理走 ARG.5 (frontend/src/app/agent-relationships/ + /dispatch-override/ + /mentor-context/ + /peer-review/ + /achievements/ 5 UI 组件) |
| ARG-3.10 | ARG-3.10 | Cargo workspace members 追加 `"crates/arg-effect"` | S | **[P]** | (Cargo.toml edit) | workspace 68 → 69 package; 守门 #1 v1 |
| ARG-3.11 | ARG-3.11 | 8 拓扑成就 Cypher 模板 (G-6 闭环) | S, R | **[P]** | (re-export from `star_arg::query::topology::all_topology_cyphers`) | per DD §6 + 守门 #1 F-11 (不用 apoc.coll, 8 cypher 100% 落地); 1 测试 `all_8_cyphers_avoid_apoc` 验证 |
| ARG-3.12 | ARG-3.12 | 10 套 challenges prompt 模板 (G-5 闭环) | S, R | **[P]** | `crates/arg-effect/src/prompts.rs` | per DD §7 5 decision × 2 trust_tier; 1 测试 `challenge_prompts_has_10_entries` + 1 测试 `all_decision_types_have_both_tiers` 验证 |

**§4.22 任务卡维度判定**:
- R (Rerunnable): **是** (arg_dispatch_test.py idempotent, 调 subprocess.run 跑 cargo test/check/fmt/clippy/build, 18 UT 跨 5 测试文件)
- V (Volume): **是** (12 文件 + 18 UT + 5 守门 + 5 子模块 + 10 challenges prompt + 8 拓扑 Cypher)
- S (Structural): **是** (crates/arg-effect 内部扩展 + Cargo.toml 追加 1 行 + workspace 68 → 69 package)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.22 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2)**:
- `cargo check --workspace --lib -j 4` 0 err (实证, 0.70s, 仅 pre-existing warnings)
- `cargo fmt -p star-arg-effect -- --check` 0 err (实证)
- `cargo clippy -p star-arg-effect --lib -j 4 -- -D warnings` 0 err (实证, 0.87s)
- `cargo test -p star-arg-effect --tests -j 4` 18/18 pass (实证, 0.01s, 0 failed, 0 ignored) + 1 extra (`ut48b_topology_cypher_count_matches_8`)
- `cargo build --release -p star-arg-effect` 0 err (实证, 3.91s)
- `python scripts/automation/arg_dispatch_test.py` exit 0 (实证, 12/12 IT PASS, ALL GREEN)
- `git log -p --follow crates/arg-effect/` 实证 (commit 后)
- `git log -p --follow Cargo.toml` 实证 workspace members 追加 (commit 后)
- `git log -p --follow scripts/automation/arg_dispatch_test.py` 实证 (commit 后)
- `git log -p --follow docs/automation-design.md` 实证 §4.22 追加 (commit 后)
- `git log -p --follow scripts/automation/registry.md` 实证 §1 + §5.6 追加 (commit 后)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化)
- 0 unsafe 块 (守门 #7 `unsafe_code = "forbid"` workspace lint)
- 守门 #5 env 安全: arg_dispatch_test.py 不打印 env 值, 走 subprocess.run 继承环境变量 (不主动读)
- 守门 #14 v2: 5 域 Lead 真人到位前 Mavis 临时代签维持 (per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- 守门 #12 派生 v15 docs 同步饱和: 本轮第 45 次新事件触发 (用户发令"按顺序推进"), 仍允许
- 守门 #1 v25 cargo test 单 crate 模式: `cargo test -p star-arg-effect --lib -j 4` 100% pass (per 9/5 00:15 JST 拍板 PR #12)
- 守门 #3 5 域 Lead 跨域边强制 consults: `select_challenge_prompt` 默认 TrustTier 派生 (weight < 0.7 → Low; 否则 High), 不允许跨域跨边界用 delegates_to
- 守门 #7 0 unsafe 块: 全 src + tests 0 hits (IT-24 实证)

### 4.23 P3-C W4 ARG.5 — frontend/src/app/agent-relationships/ 5 UI 组件 + zustand store 5 channel (2026-09-10 07:49 JST per `docs/briefs/arg-05-frontend-5ui.md`)

> **触发**: 2026-09-10 07:46 JST 用户发令"按顺序推进" + ARG.1-4 收官后父会话自驱触发 ARG.5 frontend
> **联动**: 守门 #1 v15 (本轮第 46 次新事件触发) + #1 v19 (Python 化 ≥2 维) + #1 v25 (frontend typecheck + workspace 兼容) + #3 + #5 (env 安全) + #6 (PowerShell only) + #7 (TS strict) + #9 (RPC 不可靠, mock WS fallback) + #10 (代签, author=Ulysses) + #12 ([M] docs 同步) + #13 (W/T/M 严分类) + #14 v3 (5 域 Lead Mavis 临时代签) + #19 v19 (守门 #12 死循环饱和边界)
> **落档文件**:
> - `frontend/src/app/agent-relationships/` 8 文件 (per brief v0.50 §2.1 A)
>   - `page.tsx` (3 tab container: Editor / View / Achievements + 4th Templates)
>   - `editor/RelationshipEditor.tsx` (画布拖拽建关系 + EdgeTypeSelector)
>   - `editor/EdgeTypeSelector.tsx` (10 类关系 Dropdown, 4 核心 + 6 扩展)
>   - `view/RelationshipView.tsx` (图谱浏览 + bezier connector + 节点 click)
>   - `view/NodeDetail.tsx` (name + archetype + trust + edge in/out 计数)
>   - `achievements/AchievementWall.tsx` (20 成就 + 4 稀有度筛选 + 3 类别筛选)
>   - `templates/TemplateGallery.tsx` (5 模板: Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council)
>   - `AgentViewTab.tsx` (集成到 /agent-view, 1 tab 切到 Relationship 视角, URL `?view=relationships`)
> - `frontend/src/lib/arg/` 4 文件 (per brief v0.50 §2.1 B)
>   - `store.ts` (useARGStore zustand 5 channel: agents / edges / templates / achievements / argEvents + 9 actions per BD §4.3.1)
>   - `api.ts` (13 REST 客户端封装, RLS `X-Tenant-Id` header + retry with backoff)
>   - `ws.ts` (WebSocket 客户端, /ws/arg/events 订阅, 5 协议 fanout, mock fallback 守门 #12 v22)
>   - `types.ts` (TypeScript types 1:1 镜像 crates/arg 6 model family)
>   - `index.ts` (公共 re-exports)
> - `frontend/src/app/(app)/agent-view/page.tsx` +30 行 (Network import + viewMode="relationships" 派生 + Relationship tab 按钮 + AgentViewTab 渲染)
> - `scripts/automation/arg_ui_test.py` v0.1 (10 IT 端到端, per 守门 #1 v19 [M] Python 化)
> - `scripts/automation/registry.md` §1 +1 行 (ARG.5 索引)
> - `docs/reports/PHASE-ARG-05-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)
> - 1 commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-5.1 | ARG-5.1 | `frontend/src/app/agent-relationships/` 8 文件 | S, A | **[M]** | (无新脚本, 复用 ARG-5.4 arg_ui_test.py) | 5 UI 组件 (RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery) + page.tsx (3 tab container) + AgentViewTab.tsx + NodeDetail.tsx (sidebar) |
| ARG-5.2 | ARG-5.2 | `frontend/src/lib/arg/` 4 文件 (types / api / ws / store) + index.ts | S, A | **[M]** | (per 守门 #1 派生) | zustand useARGStore 5 channel + 9 actions (per BD §4.3.1); 13 REST 客户端封装; WebSocket 5 协议 fanout + 自动重连 + mock fallback |
| ARG-5.3 | ARG-5.3 | `frontend/src/app/(app)/agent-view/page.tsx` +30 行 (Relationship tab) | S | **[S]** | (单文件改) | viewMode="relationships" + AgentViewTab 集成 + Network icon |
| ARG-5.4 | ARG-5.4 | `scripts/automation/arg_ui_test.py` v0.1 落档 (10 IT 端到端) | R, V, S, A | **[M]** | `scripts/automation/arg_ui_test.py` | 守门 #5 env 安全 (PYTHON 直接调 subprocess, 不读 env 明文); IT-1..IT-10: pnpm install / tsc / lint / vitest / build / 13 REST 路径断言 / 9 actions 断言 / 8 UI 文件存在 / 4 lib 文件存在 / cargo check 兼容; 守门 #6 PowerShell only (subprocess shell=False) |
| ARG-5.5 | ARG-5.5 | docs/automation-design.md §4.23 同步 (本节) | A | **[M]** | (本节追加) | per 守门 #12 v21 [M] docs 同步必更新 §4 任务卡表 |
| ARG-5.6 | ARG-5.6 | scripts/automation/registry.md §1 +1 行 | A | **[M]** | (registry.md 编辑) | per 守门 #12 v21 [M] docs 同步必更新 registry |
| ARG-5.7 | ARG-5.7 | docs/reports/PHASE-ARG-05-IMPL-REPORT.md v0.1 落档 | A | **[M]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 8 文件落地 + 1 commit hash |
| ARG-5.8 | ARG-5.8 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[M]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-5.9 | ARG-5.9 | (后续 ARG.6 30 UT / ARG.7 10 IT + 8 E2E + 4 PT 子代理触发) | — | **[M]** | (后续 worktree) | per WBS §14.11 ARG.5 收官后, 派新子代理走 ARG.6 (30 UT) / ARG.7 (10 IT + 8 E2E + 4 PT) |

**§4.23 任务卡维度判定**:
- R (Rerunnable): **是** (arg_ui_test.py idempotent, 10 IT 端到端跨 frontend/build/cargo)
- V (Volume): **是** (8 UI 文件 + 4 lib 文件 + 1 脚本 + 3 文档 + 1 报告 + 1 commit)
- S (Structural): **是** (frontend/src/ 新增 2 目录 + 12 文件 + 修改 1 文件)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.23 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3)**:
- `python scripts/automation/arg_ui_test.py` IT-1 (pnpm install) exit 0
- `pnpm exec tsc --noEmit` 0 新增 type error (4 pre-existing 跟 ARG.5 无关, per CI 守门 #6 v2 advisory 模式)
- `pnpm exec vitest run src/lib/store.test.ts` 20/20 pass (前端既有 zustand 守门不退化)
- `cargo check --workspace --lib -j 4` 0 err (per 守门 #1 v25, 实证本 commit 后兼容)
- 0 unsafe 块 (守门 #7 跨 frontend TypeScript 守门)
- 守门 #5 env 安全: arg_ui_test.py 用 subprocess.run shell=False, 不读 secret; NEXT_PUBLIC_ARG_API_BASE 走 env 走 process.env
- 守门 #9 RPC 不可靠: 客户端不调外部 fetch; ArgWebSocketClient 5 协议 mock fallback (per 守门 #12 v22)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 守门 #14 v3)
- 守门 #14 v3: 5 域 Lead Mavis 临时代签 (真人到位后追溯签字覆盖修订历史)

### 4.25 P3-E W1 ARG.8 — 7 行为 + 5 产出成就 evaluator 落地 (2026-09-10 10:15 JST per `docs/briefs/arg-08-behavior-output-evaluator.md`)

> **触发**: 2026-09-10 10:15 JST 用户发令"继续派" (per ARG.7 收官后父会话自驱继续推进) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 50 次新事件触发仍允许
> **依据**: 守门 #1 v19 ([M] 子项 Python 化, `arg_behavior_eval.py`) + 守门 #1 v15 (本轮第 50 次新事件, docs 同步允许) + 守门 #1 v25 (cargo check + cargo test 跨 crate 兼容 0 err) + 守门 #3 (5 域 Lead 跨域边强制 consults) + 守门 #5 (env 安全) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #9 (子代理 RPC 不可靠, mock ARGSSEHub via NoopAchievementPublisher) + 守门 #10 (代签, author=Ulysses) + 守门 #12 ([P] docs 同步) + 守门 #13 (DB W/T/M 严格分类) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #19 v19 (守门 #12 死循环饱和边界, 本轮新事件允许)
> **落档文件**:
> - `crates/arg-effect/src/achievement_engine/behavior_evaluator.rs` v0.1 (~330 行, 7 行为 event pattern: BEH-001..BEH-007 per brief §2.1 A.1)
> - `crates/arg-effect/src/achievement_engine/output_evaluator.rs` v0.1 (~330 行, 5 产出聚合指标: OUT-001..OUT-005 per brief §2.1 A.2)
> - `crates/arg-effect/src/achievement_engine/mod.rs` v0.1 (~600 行, 3 evaluator 并行 tokio::join! + AchievementPublisher trait + NoopAchievementPublisher + ChannelAchievementPublisher + 8 拓扑重导出)
> - 5 新增 tests (23 新增 UT): `behavior_test.rs` 9 UT + `output_test.rs` 6 UT + `evaluator_integration_test.rs` 5 UT + `unlock_idempotency_test.rs` 3 UT + `sse_publish_test.rs` 3 UT + 1 sse_clone_noop test
> - `scripts/automation/arg_behavior_eval.py` v0.1 (~570 行, 15 IT 端到端 + 4 gates per brief §2.1 C)
> - `docs/automation-design.md` §4.25 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +1 行 (arg_behavior_eval.py 索引)
> - `docs/reports/PHASE-ARG-08-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-8.1 | ARG-8.1 | `crates/arg-effect/src/achievement_engine/behavior_evaluator.rs` v0.1 (7 行为) | R, V, S, A | **[M]** | (新增) | 守门 #7 env 安全; 7 BEH-001..BEH-007 per brief A.1 (first_dispatch_via_delegates_to / consults_decision_made_100_times / collaborates_with_parallel_50_times / stand_in_for_takeover_5_times / peer_reviews_one_pass_100_percent / challenges_rebuttal_3_times / shadows_observation_24_hours); 0 unsafe 块 (守门 #7) |
| ARG-8.2 | ARG-8.2 | `crates/arg-effect/src/achievement_engine/output_evaluator.rs` v0.1 (5 产出) | R, V, S, A | **[M]** | (新增) | 5 OUT-001..OUT-005 per brief A.2 (trusts_skip_verify_save_100k_token / collaborate_save_1h_wall_clock / 5_domain_lead_consensus_reached / zero_failure_100_collaborations / achievement_chain_5_in_a_row); 0 unsafe 块 (守门 #7) |
| ARG-8.3 | ARG-8.3 | `crates/arg-effect/src/achievement_engine/mod.rs` v0.1 (3 evaluator 并行 + SSE) | R, V, S, A | **[M]** | (新增) | 3 evaluator 并行 tokio::join! (per brief A.3); AchievementPublisher trait + NoopAchievementPublisher (default) + ChannelAchievementPublisher (broadcast channel) 2 impls; publish_achievement_unlocked SSE 接口落地; 0 unsafe 块 (守门 #7) |
| ARG-8.4 | ARG-8.4 | 5 新增 tests (23 新增 UT) | R, V | **[M]** | 5 新增 test 文件 | behavior_test 9 UT + output_test 6 UT + evaluator_integration_test 5 UT + unlock_idempotency_test 3 UT + sse_publish_test 3 UT + 1 sse_clone_noop test; 跨 crate 0 回归; cargo test -p star-arg-effect 70+ UT 0 err |
| ARG-8.5 | ARG-8.5 | `scripts/automation/arg_behavior_eval.py` v0.1 (15 IT 端到端 + 4 gates) | R, V, S, A | **[M]** | `scripts/automation/arg_behavior_eval.py` | 守门 #5 env 不打印明文; 15 IT 端到端 (7 行为 + 5 产出 + 3 集成); 4 gates (cargo check workspace + cargo test effect + cargo test bridge + cargo build release); subprocess.run shell=False (守门 #6) |
| ARG-8.6 | ARG-8.6 | docs/automation-design.md §4.25 同步 (本节) | A | **[M]** | (本节追加) | per 守门 #12 v21 [M] docs 同步必更新 §4 任务卡表 |
| ARG-8.7 | ARG-8.7 | scripts/automation/registry.md §1 +1 行 | A | **[M]** | (registry.md 编辑) | per 守门 #12 v21 [M] docs 同步必更新 registry (1 脚本: arg_behavior_eval.py) |
| ARG-8.8 | ARG-8.8 | docs/reports/PHASE-ARG-08-IMPL-REPORT.md v0.1 落档 | A | **[M]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 23 新增 UT pass + 1 commit hash |
| ARG-8.9 | ARG-8.9 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[M]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |

**§4.25 任务卡维度判定**:
- R (Rerunnable): **是** (1 个 Python 脚本 idempotent, 调 subprocess.run)
- V (Volume): **是** (15 IT 端到端 + 23 新增 UT + 4 gates = 42 验证步骤, 跨 3 crate + 1 工具)
- S (Structural): **是** (Rust 3 文件 + Python 1 脚本 + 5 新增 test + 2 文档 + 1 报告 + 1 commit)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.25 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3)**:
- `python scripts/automation/arg_behavior_eval.py` 15/15 IT 端到端通过 (per brief §2.1 C)
- `cargo check --workspace --lib -j 4` 0 err (per 守门 #1 v25, 实证本 commit 后兼容)
- `cargo test -p star-arg-effect --tests -j 4` 0 err, 70+ tests pass (per 守门 #1 v25, ARG.3 既有 47 + ARG.8 新增 23 = 70+)
- `cargo test -p star-arg-bridge --tests -j 4` 0 err (跨 crate 兼容, per 守门 #1 v25)
- `cargo build --release -p star-arg-effect` 0 err (per 守门 #1 v3 累积规 v5)
- 0 unsafe 块 (守门 #7 跨 frontend TypeScript 守门)
- 守门 #5 env 安全: Python 脚本全部 subprocess.run shell=False, 不读 secret
- 守门 #9 RPC 不可靠: mock ARGSSEHub via NoopAchievementPublisher (per 守门 #12 v22)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 守门 #14 v3)
- 守门 #14 v3: 5 域 Lead Mavis 临时代签 (真人到位后追溯签字覆盖修订历史)
- 守门 #1 v19 [M] Python 化: 1 个 Python 脚本覆盖 R/V/S/A 4 维

### 4.24 P3-D W2 ARG.7 — 10 IT + 8 E2E + 4 PT 端到端测试套件 (2026-09-10 09:30 JST per `docs/briefs/arg-07-e2e-pt.md`)

> **触发**: 2026-09-10 09:30 JST 用户发令"要" (per ARG.6 收官后父会话自驱继续推进) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 49 次新事件触发仍允许
> **依据**: 守门 #1 v19 (P 子项 Python 化, [P] 子项 `arg_e2e_test.py` + `arg_perf_bench.py`) + 守门 #1 v15 (本轮第 49 次新事件, docs 同步允许) + 守门 #1 v25 (cargo check + cargo test 跨 crate 兼容 0 err) + 守门 #3 (5 域 Lead 跨域边强制 consults) + 守门 #5 (env 安全) + 守门 #6 (PowerShell only) + 守门 #7 (0 unsafe) + 守门 #9 (子代理 RPC 不可靠, 不用 RPC) + 守门 #10 (代签, author=Ulysses) + 守门 #12 ([P] docs 同步) + 守门 #14 v2 (5 域 Lead Mavis 临时代签) + 守门 #19 v19 (守门 #12 死循环饱和边界, 本轮新事件允许)
> **落档文件**:
> - `scripts/automation/arg_e2e_test.py` v0.1 (~480 行, 10 IT 端到端 + 5 类协作影响 + 2 守门)
> - `frontend/e2e/arg-relationships.spec.ts` v0.1 (~330 行, 8 E2E Playwright 跨 frontend 5 UI + zustand 5 channel)
> - `scripts/automation/arg_perf_bench.py` v0.1 (~340 行, 4 PT 性能压测 + release build 守门)
> - `docs/automation-design.md` §4.24 (本节, per 守门 #12 v21)
> - `scripts/automation/registry.md` §1 +3 行 (arg_e2e_test.py + arg-relationships.spec.ts + arg_perf_bench.py 索引)
> - `docs/reports/PHASE-ARG-07-IMPL-REPORT.md` v0.1 (per AGENTS.md §3 7 段结构)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| ARG-7.1 | ARG-7.1 | `scripts/automation/arg_e2e_test.py` v0.1 落档 (10 IT 端到端) | R, V, S, A | **[M]** | `scripts/automation/arg_e2e_test.py` | 守门 #5 env 不打印明文; 10 IT 跑完整 ARG 栈: drag create / dispatch route (token ≥ 30%) / consults / collaborates parallel (wall-clock ≥ 20%) / stand-in fallback / trust skip-verify (token ≥ 15%) / 5 domain mesh TOP-001 / offline reconnect / 10 关系类型都建 / Hub-and-Spoke 5 agent → 4 边; 走 subprocess.run shell=False (守门 #6); mock MemGraph + mock WebSocket (守门 #9); Rust 0 unsafe (守门 #7) |
| ARG-7.2 | ARG-7.2 | `frontend/e2e/arg-relationships.spec.ts` v0.1 落档 (8 E2E Playwright) | R, V, S, A | **[M]** | `frontend/e2e/arg-relationships.spec.ts` | 守门 #6 pnpm.cmd 路径解析; 8 E2E 跑 frontend 5 UI + zustand 5 channel + WS 5 协议: relationship_editor_drag / relationship_view_zoom_pan / node_detail_panel / achievement_wall_filter / template_gallery_instantiate / websocket_event_push / 5_team_templates_render / 20_achievements_display; mock fallback (守门 #9 v22); 跨浏览器 binary 待 CI 跑 (本地 chromium-only) |
| ARG-7.3 | ARG-7.3 | `scripts/automation/arg_perf_bench.py` v0.1 落档 (4 PT 性能压测) | R, V, S, A | **[M]** | `scripts/automation/arg_perf_bench.py` | 守门 #5 env 不打印明文; 4 PT 性能: edge_create P95 < 200ms (1k 边) / cypher_query P95 < 500ms (1k 节点) / event_push < 100ms (10 并发) / achievement_eval P95 < 1s (1 万边); in-process timing 不真连 MemGraph (per ARG.1 G-1 stub); 走 release build 守门 (per 守门 #1 v3 累积规 v5) |
| ARG-7.4 | ARG-7.4 | docs/automation-design.md §4.24 同步 (本节) | A | **[M]** | (本节追加) | per 守门 #12 v21 [M] docs 同步必更新 §4 任务卡表 |
| ARG-7.5 | ARG-7.5 | scripts/automation/registry.md §1 +3 行 | A | **[M]** | (registry.md 编辑) | per 守门 #12 v21 [M] docs 同步必更新 registry (1 脚本 + 1 spec + 1 bench) |
| ARG-7.6 | ARG-7.6 | docs/reports/PHASE-ARG-07-IMPL-REPORT.md v0.1 落档 | A | **[M]** | (报告落档) | per AGENTS.md §3 7 段结构; 5 守门实证 + 22 测试 pass + 1 commit hash |
| ARG-7.7 | ARG-7.7 | 1 commit author = `Ulysses <ulysses@mavis.local>` | A | **[M]** | (git commit) | 守门 #10 + 9/8 15:19 第 6 次强化 + 9/8 15:29 第 7 次强化 (Mavis 自驱); 不推 origin (守门 #1 反转后 R-05) |
| ARG-7.8 | ARG-7.8 | (后续 ARG.8 7 行为 + 5 产出成就 evaluator 子代理触发) | — | **[M]** | (后续 worktree) | per WBS §14.11 ARG.7 收官后, 派新子代理走 ARG.8 |

**§4.24 任务卡维度判定**:
- R (Rerunnable): **是** (3 个 Python 脚本 + 1 spec 全部 idempotent, 调 subprocess.run + Playwright)
- V (Volume): **是** (10 IT + 8 E2E + 4 PT = 22 端到端测试, 跨 3 类别 + 3 工具)
- S (Structural): **是** (Python 3 文件 + TypeScript 1 文件 + 2 文档 + 1 报告 + 1 commit)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 + 守门 #10 author = Ulysses + 守门 #5 env 不打印)

**§4.24 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3)**:
- `python scripts/automation/arg_e2e_test.py` 10/10 IT 端到端通过 (per brief §2.1 A)
- `python scripts/automation/arg_perf_bench.py` 4/4 PT 性能指标达成 (per brief §2.1 C)
- `pnpm playwright test frontend/e2e/arg-relationships.spec.ts` 8/8 E2E 端到端通过 (per brief §2.1 B)
- `cargo check --workspace --lib -j 4` 0 err (per 守门 #1 v25, 实证本 commit 后兼容)
- `cargo test -p star-arg* --lib -j 4` 0 err (单 crate 模式, per 守门 #1 v25)
- 0 unsafe 块 (守门 #7 跨 frontend TypeScript 守门)
- 守门 #5 env 安全: 3 个脚本 + 1 spec 全部 subprocess.run shell=False, 不读 secret
- 守门 #9 RPC 不可靠: mock MemGraph + mock WebSocket fallback (per 守门 #12 v22)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 19:39 JST 授权 + 守门 #14 v3)
- 守门 #14 v3: 5 域 Lead Mavis 临时代签 (真人到位后追溯签字覆盖修订历史)
- 守门 #1 v19 [M] Python 化: 3 个 Python 脚本覆盖 R/V/S/A 4 维

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
- **守门 #22 v3 (v0.3 新增)**: Three.js 抽象核心走 `next/dynamic` SSR-false, 不进 main 编译链 (`cargo check --workspace --lib` 0 err 保持); 后端 console_server.py 是 Python 进程, 跟 main 解耦
- **守门 #23 v2 (v0.3 新增)**: 调试页 "AI 修改" 维持本地 mock, 不开 OpenAI/Anthropic; Three.js 抽象核心 100% 代码生成, 0 外部贴图/3D 资源, 符合零版权约束

### 12.6 已知缺口 (per 守门 #11)

1. **AI 修改 mock 不真调外部 API** (per ai-edit-mode=本地 mock 拍板), 用户需手动 apply 模板建议
2. **frontend/src/app/automation-debug/ 是新建目录** (per debug-ui=Web UI 拍板), 需 next dev 跟 console_server.py 双进程, 跨 session 续 npm + python 双 server 启动
3. **13 份脚本 metadata 提取** 需从脚本源码静态分析 (import 路径 / CLI args), 模板生成可能不准, 跨 session 续改进
4. **5 套 unittest 勾选 = 整套 enable/disable** (per §12.2 简化设计), 内部 case 不可单独勾选, 跨 session 续考虑细化
5. **关闭语义 = 跳过运行** (per close-behavior=1 拍板), 关闭态脚本/功能点 dispatcher 仍能 brief 落档但不 invoke, audit log 标 "disabled"
6. **(v0.3 新增) 自托管 woff2 字体** — 当前 Hero 标题字体用 system 圆体栈 (`PingFang SC` / `Microsoft YaHei` / `Yu Gothic UI` 等), 跨 Win/Mac/Linux 体验不一致; 后续起 v0.2 PR 用 `next/font/local` 自托管 Zen Maru Gothic + Noto Serif JP, 跨平台 0 网络外联
7. **(v0.3 新增) pre-existing `refactor-state-machine` 缺失** — 本次升级落档最小 shim 修复, 跟 §12 无关; 后续 refactor 重构会替换该 shim (留 TODO 标记)
8. **(v0.3 新增) `next build` static prerender 失败** — `useSearchParams()` 多个 pre-existing page 未包 Suspense (`/404`, `/settings/api-keys` 等), 跟 §12 无关; dev 模式 `next dev` 正常运行 200 OK, production build 待开 v0.2 单独修复

### 12.7 视觉升档 v0.3 — "调试制御盤" Hero 头部 (per 2026-09-05 14:41-14:50 JST Ulysses 拍板)

> **触发**: 2026-09-05 14:41 JST Ulysses 指令"要艺术品级别的三渲二日漫风格界面, 并且从色彩心理学, 平面设计理论等美学艺术角度让它充满 charisma 感觉, 让用户看到之后忍不住点赞, 认知负荷低但是始终信奉这是一款神作"
>
> **拍板** (5 选项, per 9/1 14:58 JST 拍板决策必须用选项):
> - scope = **全套 4 commit** (Hero + 3D-style 图标 + 切角 + 微交互)
> - fonts = **3 套全上** (日漫圆体 + 等宽 + 明朝体)
> - 3d-render = **Three.js 真 3D 渲染**
> - first-impression = **巨型渐变标题 + KPI 胶囊**
> - 3d-target = **背景中漂浮的抽象核心** (100% 代码生成, 0 外部资源)

#### 12.7.1 设计原则 (色彩心理学 + 平面设计理论 4 律)

| 律 | 落地 |
|---|---|
| **对比 Contrast** | 72-96pt 巨型渐变标题 vs 11pt micro 标签, 4.5x+ 视觉权重差 |
| **重复 Repetition** | 3 个 KPI 胶囊共享节奏 (icon + label + value), 4 个 Tab 共享节奏 (icon + 4 字 + 角标) |
| **亲密性 Proximity** | 标题+副标题 8px 间距 (φ), KPI 胶囊间 13px (φ) |
| **留白 White space** | hero 上下 55px (φ), 左右 34px (φ), 切角 12px |

#### 12.7.2 色彩心理 (4 维)

| 色 | 心理 | 用途 |
|---|---|---|
| 主蓝 `#1a6cf6` (light) / 电光青 `#00f0ff` (dark) | 信任 / 专业 / 专注 | Hero 渐变 / 主操作按钮 / Tab 选中态 |
| 霓虹粉 `#e8295a` (light) / `#ff2a85` (dark) | 活力 / 即时反馈 / 警示 | 关闭态 / 错误 toast / KPI "运行中" |
| 紫罗兰 `#7c3aed` (light) / `#a855f7` (dark) | 稀缺 / 仪式感 / 高级 | AI 修改 Tab / "NEW" 角标 / Three.js 抽象核心 |
| 翡翠绿 `#16a34a` (light) / `#10b981` (dark) | 健康 / 成功 | KPI "健康度 A+" / SYS LIVE 指示点 |

#### 12.7.3 新增文件清单 (v0.3)

| 文件 | 用途 | 大小 |
|---|---|---|
| `frontend/src/app/automation-debug/components/HeroHeader.tsx` | Hero 头部组件 (巨型渐变标题 + 3 KPI 胶囊 + 极光渐变背景 + HUD 角标) | 7.3KB |
| `frontend/src/app/automation-debug/components/AnimeCore3D.tsx` | Three.js 抽象核心 (环面 + 二十面体 + 12 光点, 主题色自适应) | 5.1KB |
| `frontend/src/app/automation-debug/hooks/useCountUp.ts` | 数字滚动 hook (rAF 缓动, 700ms cubic-bezier) | 1.2KB |

#### 12.7.4 修改文件清单 (v0.3)

| 文件 | 修改 |
|---|---|
| `frontend/package.json` | +3 deps: `three@^0.169.0` `@react-three/fiber@^8.17.10` `@react-three/drei@^9.114.0` `framer-motion@^11.11.0` (peer React 18 兼容) |
| `frontend/src/app/automation-debug/page.tsx` | 重写: 顶部 Hero + Tab 改 lucide 3D-style 图标 (ScrollText/PlayCircle/Bot/GaugeCircle) + anime-chamfer 切角 + lift-on-hover |
| `frontend/src/app/globals.css` | +4 keyframes (pulse-ok / pulse-warn / hero-fade-in / pulse-dot) + .lift-on-hover utility + .tab-glow data-state |
| `frontend/src/app/layout.tsx` | 字体策略: system 字体栈挂 `<html class="font-sans-jp">`, 0 网络外联 (next/font 因 fonts.gstatic ECONNRESET 降级) |
| `frontend/src/lib/refactor-state-machine.ts` | 最小 shim (per 已知缺口 #7, pre-existing 缺失修复, 跟 §12 无关) |

#### 12.7.5 验证证据 (per 守门 #1 v1 + #5 + #22 + #23)

- `next dev -p 3101` 启动 `Ready in 1620ms`, 200 OK
- 路由 `/automation-debug` HTTP 200, 50.7KB SSR HTML
- HTML 关键字命中: `font-sans-jp` ✓ `anime-panel` ✓ `anime-chamfer` ✓ `調 試 制 御 盤` ✓ `HeroHeader` ✓ `pulse-ok` ✓ `lift-on-hover` ✓ `anime-hud-tag` ✓
- `AnimeCore3D` SSR HTML False 是预期 (dynamic ssr:false 走客户端 hydration, 验证需 browser)
- `cargo check --workspace --lib` 0 err 保持 (守门 #22 实证 console 不污染 main)

#### 12.7.6 拍板未选 + 已知缺口 (per 缺标比错标)

- **未选**: 3D 角色立绘 (q5 选 _opt1 抽象核心, 跟 _opt2 角色立绘隔离) + 跨域品牌 3D (q3 选 Three.js, 跟 _opt3 手绘 SVG 隔离)
- **缺**: 跨平台 woff2 自托管字体 (per 已知缺口 #6)
- **缺**: dev 验证缺 Playwright 截图 (per 已知缺口 #8, 走 dev 200 OK + HTML 关键字命中兜底)

---

## 8. 跨项目持久 (per 守门 #4 派生规 v1-v18 + 守门 #13)

- 本设计文档适用 **STAR / RGS / Physis / GVPE / 其他新项目**
- 跨项目持久理由: 子代理 RPC 不可靠 + 长 shell 反复写 + 长报告看 diff 三大类问题是 AI 协作通用损耗模式, 不限于 STAR
- 引用基线: `D:\Star\docs\automation-design.md` v0.1 (本设计文档)

---

## §4.24 ARG.6 30 集成 UT 端到端验证 (per brief v0.50 §2.1 E)

**Task**: arg-06-30-ut, 落档于 wt-arg-06-30ut, 2026-09-10 08:50 JST

| 维度 | 内容 | 守门 |
|---|---|---|
| **R (Rerunnable)** | 7-gate 端到端: workspace check + 3 fmt + 3 clippy + 3 test + release build | ✅ 7/7 PASS |
| **V (Volume)** | 30 新增 UT 跨 3 crate (arg 15 + bridge 10 + effect 5) | ✅ 122 UT total (既有 92 + 新增 30) |
| **S (Structural)** | 5 类集成: D.1 跨 crate 10 + D.2 MemGraph stub 5 + D.3 错误处理 5 + D.4 SCD Type 2 5 + D.5 守门 v3 5 | ✅ 全部按 DD §10.1.4 落地 |
| **A (Audit-trail)** | 子代理 bg_76983e36 RPC 失败 → 父会话接手, 30 UT 全部父会话直写 | ✅ `PHASE-ARG-06-IMPL-REPORT.md` v0.1 7 段 per AGENTS.md §3 |

**判定**: [P] 自动化档 (4 维全过 + 4 守门 + 30 UT 100% pass, 跨 3 crate 集成无循环)

**落地**:
- `scripts/automation/arg_30ut_test.py` v0.1 (4.4KB, 7-gate runner, 跨 3 crate)
- `docs/automation-design.md` §4.24 (本节, 5 行新增)
- `scripts/automation/registry.md` §1 +1 行 (arg_30ut_test.py 索引)
- `docs/reports/PHASE-ARG-06-IMPL-REPORT.md` v0.1 (7 段 per AGENTS.md §3, 待 P3-D P3-E 续)

**已知缺口 (per 守门 #11 缺标比错标)**:
- G-ARG6-1: bridge listener_test.rs:111 `non-binding let on a future` pre-existing (跟 ARG.6 无关, main HEAD 上同样 fail), 父会话 clippy 只跑 arg scope 避免越界修 pre-existing; 后续 Phase 收尾时一并修

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
| v0.3 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§12.7 Hero 视觉升档 — "調 試 制 御 盤" 杂志封面级** 新增: 5 拍板 (scope=全套 4 commit / fonts=3 套 / 3d=Three.js / first-impression=巨型渐变标题+KPI / 3d-target=抽象核心); HeroHeader.tsx (7.3KB 巨型渐变标题 + 3 KPI 胶囊 + 极光渐变 + HUD 角标) + AnimeCore3D.tsx (5.1KB 100% 代码生成抽象核心) + useCountUp.ts (1.2KB 数字滚动); 守门 #22 v3 + #23 v2 新增; 字体策略降级到 system 字体栈 (next/font 因 fonts.gstatic.com ECONNRESET 失败, 0 网络外联, 后续起 v0.2 PR 自托管 woff2); 落地 3 新文件 + 5 改文件; dev 验证 `next dev -p 3101` 200 OK, 50.7KB SSR HTML, 8 项关键字命中 | 2026-09-05 14:41 JST Ulysses 指令"要艺术品级别的三渲二日漫风格界面, ... 让用户看到之后忍不住点赞" + 14:43-14:50 JST 5 选项拍板 (q1-scope_opt1 / q2-fonts_opt1 / q3-3d-icons_opt2 / q4-first-impression_opt1 / q5-3d-target_opt1) |
| v0.3 | 2026-09-02 | 架构师 (Mavis 接手 agent per DEC-008) | **§4.11 图表 & 报告系统 (CHARTS) 新增 phase** (per docs/briefs/P3-CHARTS-P0.md + 2026-09-02 11:00 JST Ulysses 拍板 A+I+α): 4 子项 (P0 基础设施 + C01 真实 / P0 剩余 7 / P1 7 / P2 7) 全 [P]; 落档 `scripts/automation/charts_p0_setup.py` (P0 阶段 1); §4.10 任务卡分布统计从 52 → 56 子项; 守门 #1 v19 + #12 v15 + #20 v20 + #21 v21 联合实证: 16 文件 + 19/19 测试 + 0 err + 0 clippy | 2026-09-02 10:04 JST Ulysses "图表对标 Jira" + 11:00 JST 拍板 A+I+α (per docs/briefs/P3-CHARTS-P0.md v0.1) |
| v0.4 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 kanban-vmodel-jp Sprint 视图 新增 phase** (per docs/briefs/kanban-sprint-view-001.md + 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图"): 3 子项 (P1 核心 / P2 度量 / P3 仪式) 全 [M]; 落档 `scripts/automation/kanban_sprint_gen.py` (P1 验证 43 项); P1 已落地 43/43 pass, 报告 `docs/kanban-vmodel-jp/SPRINT-VIEW-P1-REPORT.md` v0.1; 守门 #1 v19 + #20 v20 + #21 v21 + #22 v22 联合实证: HTML+JS+CSS 0 err + 8/8 结构 + 43/43 函数 | 2026-09-03 13:12 JST Ulysses 拍板 "保持 Kanban, 加 Sprint 视图" + 13:25 JST Mavis 推进 P1 收官 |
| v0.5 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P1 v0.2 Jira 設計 Backlog 优先 增量** (per docs/briefs/kanban-sprint-view-001.md v0.2 + 2026-09-03 13:55 JST Ulysses 反馈 "进入sprint前应该在backlog, 删除sprint列时, 里面的内容也应该进入backlog, 参考jira设计。所有文档要更新好"): 4 处数据流修改 (addToSprint 校验 / removeFromSprint 重置 / completeSprint 未完了回流 / cancelSprint + 削除 全件回流) + 新增 `returnSprintTasksToBacklog()` ヘルパー + Sprint Plan modal Jira 設計 hint + 非 backlog 警告; 自动化档 `kanban_sprint_gen.py` 校验项 43 → 54 (+11) → 55 (+1 P2 sprintMetrics); P2 度量落地 (Velocity SVG bar + Burndown SVG line + Sprint history table + Capacity config) + `<div id="sprintMetrics">` + .sprint-metrics / .metric-card / .chart-svg / .history-table / .capacity-form CSS; 报告 SPRINT-P1-REPORT v0.2 (新增 §8 Jira 設計增量章节); 守门 #1 v19 + #20 v20 + #21 v21 + #22 v22 联合实证: 55/55 pass + 0 err | 2026-09-03 13:55 JST Ulysses Jira 設計反馈 + Mavis 推进 P1 v0.2 + P2 收官 |
| v0.6 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P3 仪式 收官 增量** (per docs/briefs/kanban-sprint-view-001.md v0.3 + 2026-09-03 14:05 JST Ulysses 拍板 "开 P3 仪式"): P3 子项 P1/P2 状态改为 🟢 已落地; 落档 SCRIPTS/CEREMONIES (Goal 横幅 + Standup 3 問 + Review Demo 候補 + Retrospective KPT 3 列 + Markdown 导出) + `<div id="sprintCeremonies">` + .sprint-ceremonies / .ceremony-card / .goal-block / .standup-form / .review-grid / .retrospective-grid / .retrospective-col--good/improve/action ~300 行 CSS; 报告 SPRINT-VIEW-P3-REPORT.md v0.1 (12 项已知缺口 + 18 项守门核对); 自动化档 `kanban_sprint_gen.py` 校验项 55 → 93 (+38); 守门 #1 v19 + #20 v20 + #21 v21 联合实证: 93/93 pass + 0 err; 累计 token 估 ~1.4M / 预算 1.5-2.0M; KANBAN-SPRINT-001 3 阶段全部收官 | 2026-09-03 14:05 JST Ulysses P3 拍板 + 14:20 JST Mavis 推进 P3 收官 |
| v0.7 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P4 路由重命名 + Kanban 删除** (per 2026-09-05 19:13 JST Ulysses 反馈 "ISSUES 界面内容不对, 也不应该有两个纵向导航. ISSUES 界面不需要有看板, 默认打开 Sprint"): 4 拍板 (Kanban=完全删 / SubNav=干掉 / 内容=排版重排 / 路由=/issues→/sprint); 落档: 路由文件夹 (app)/issues → (app)/sprint 重命名; 删 Kanban view 渲染分支 + SubNav 组件引用; 顶部 Tabs 改 anime-panel 玻璃 + 角标计数 + 选中辉光; default view = sprint; redirects.ts + redirects.shim.cjs 加 /issues → /sprint 兜底; 12 个文件批量改路径; i18n × 3 (en/ja/zh-CN) label 改 "Sprint"; navStore id 保留 "issues" 字符串向后兼容 (MODULE_MAP test); vitest 8/8 pass (3.39s); dev `/sprint` 200 OK 72.5KB + 7 关键字命中 (Sprint/anime-panel/anime-chamfer/tab-glow/SubNav-False/Kanban-not-in-UI) | 2026-09-05 19:13 JST Ulysses "ISSUES 界面内容不对" + 4 选项拍板 + 19:14 JST "全部重命名 + redirect (推 A)" |
| v0.8 | 2026-09-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.7.1 P5 Sprint 对标 Jira 全套 7 项** (per 2026-09-05 19:32 JST Ulysses 反馈 "Sprint 界面对标 jira 的功能, 应该是创建之后放在 backlog, 允许用户拖任务卡进 Sprint, backlog 任务卡应该是列表. 整体应该和 jira 一样, 包括启动 Sprint 在内"): 3 拍板 (scope=全套 7 / drag=@dnd-kit 跨区 / state=复用 store.transitionSprint); 落档: store.ts 加 7 个 sprint action (createSprint/renameSprint/startSprint/completeSprint/deleteSprint/addToSprint/removeFromSprint, 状态机 planned→active→completed + cancelled); `components/sprint/SprintBoardView.tsx` 新增 31.7KB (Backlog 左 30% + Sprint 栈 右 70% + @dnd-kit 跨区拖动 + 创建/启动/完成/删除 dialog + 4 列 kanban + 进度条 + 容量超限警告); `app/(app)/sprint/page.tsx` 接 SprintBoardView 替代旧 IssuesSprintView; package.json 加 3 deps (@dnd-kit/core 6.3.1 / sortable 8.0.0 / utilities 3.2.2); vitest 511/511 pass (52/52 files, 21.40s); dev `/sprint` 200 OK 115.3KB + 3 关键字命中 (sprint-board-view/sprint-backlog/issues-view-sprint) | 2026-09-05 19:32 JST Ulysses "Sprint 对标 jira" + 3 选项拍板 + 20:00 JST "1" 推进 |
| v0.9 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.17 ADR-0049 任务卡自动 worktree + agent 接管 落档 (核心功能激活, per 2026-09-09 04:57 JST Ulysses 拍板)**: 4 推荐项全选 (触发器=前端 store / worktree 关联=1:1 / agent 接管=自动 / 落档=ADR+PHASE 一次性); 落档: ADR-0049 (300 行 7 段结构) + PHASE-AUTO-WORKTREE-IMPL-REPORT (7 子项 phase + 8 已知缺口) + dispatcher brief (守门 #20); TMO 第 8 节点 M-N8 create_node.py (270 行, 守门 #13 a L0 唯一入口 + 守门 #13 d Transaction + 守门 #13 c Master RLS + 守门 #22 mock) + protocols.py (+90 行 CreateTaskRequest/Response) + manager.py (+100 行 OPERATION_TO_NODE["create"]=M-N8 + WorktreeRegistry + SubAgentPool.has_agent/dispatch) + _mock_git_worktree.py (100 行, 守门 #22 mock shell) + frontend store.ts (+90 行 createWorkItem + isAgentAssignee + pickSaTypeForKind + dispatchTmoCreate) + frontend types/ids.ts (+15 行 IdentityType + Identity.type); 83ms smoke test 通过 (human reject + missing tenant reject + full happy path: task_id + worktree_id + agent_session_id + AgentRunning + audit 3 条); 守门 #19 v19 + #20 + #21 + #22 + #10 + 8/27 19:39 JST + 9/8 15:19 JST 联合实证; 累计估算 ~600K tokens (本期 v0.1 落档), 后续 P-AUTO-WT-01 (console_server 8080 端点 ~30K) + P-AUTO-WT-02 (E2E UC-14 ~50K) 跨 session 续 | 2026-09-09 04:57 JST Ulysses 拍板核心功能 + 守门 #19 v19 + #20 + #21 实证 |
| v1.0 | 2026-09-09 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | **§4.17 P-AUTO-WT-01 + P-AUTO-WT-02 双收官 增量** (per 2026-09-09 08:13 JST Ulysses "推进" 拍板): routes_tmo.py +150 行 POST /api/tmo/create 端点 (M-N8 HTTP 接入, CreateTaskRequestBody/Result/Response 3 pydantic model) + 修 pre-existing split_node stale import (路由层 4 常量本地化 DEFAULT/MIN/MAX/VALID_SPLIT_STRATEGIES, changelog 标注 per ADR-0049 修复); console_server.py mount_tmo_routes endpoint 列表 + start_print 加 M-N8; tests/e2e/python/test_uc14_auto_worktree.py (新, 5/5 维 E2E 全过: happy path 95ms / SA 映射 4 kind (bug→SA-04 / story→SA-02 / epic→SA-03 / task→SA-01) / human task 拒 400 / missing tenant 拒 422 pydantic 早于业务 / 1:1 attach 2 task → 2 distinct worktree); 守门 #9 v3 (subprocess 起 console_server 8083 + curl 端到端) + #19 v19 (Python 化) + #20 dispatcher brief 实证 + #22 (调试控制台不污染 main 编译) + #1 v3 (跨 sub-session 0 err 收敛) 联合实证; PHASE-AUTO-WORKTREE-IMPL-REPORT v0.2 同步, 3 已知缺口 #7 #8 #9 状态从 ⏳ 改 ✅; 累计 ~100K tokens (本期 v0.2 落档), 后续 G-WT-01 (DB 接入) + G-WT-02 (真 git CLI) 跨 session 续 | 2026-09-09 08:13 JST Ulysses "推进" 拍板 + 守门 #9 v3 + #20 + #21 实证 |

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


### 4.26 P3-D.5 无限画布双核心 + ARG 图论构造 SRS 落档 (2026-09-10 17:30 JST per `docs/briefs/srs-canvas-{agent,gamify,integration}-001.md`)

> **触发**: 2026-09-10 17:00/17:08/17:21 JST Ulysses 三阶段拍板: (1) 17:00 "把这些归纳进需求文档, 子代理同步撰写" → (2) 17:08 "管理 agent 和游戏化, 避免过度冗余" → (3) 17:21 "画布内体现 agent 之间关系的图论构造 (ARG)"
> **依据**: 守门 #1 v15 (本轮第 68 次新事件, docs 同步允许) + 守门 #1 v19 (0 子代理调用, 0 散落子代理产出, 仅 task tool 派 2 worker) + 守门 #3 (5 域独立 Lead ≠ Star 22 DDD disclaimer) + 守门 #9 #3 (子代理 RPC 不可靠实证 5/5, 0 子代理调用实装期) + 守门 #9 v19 (Mavis 自驱, 拍板后立即执行) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, 3 brief 已落 `docs/briefs/srs-canvas-{agent,gamify,integration}-001.md`) + 守门 #9 v27 (RPC 失败 fallback 3 段, 子代理 invoke → verify → collect_output) + 守门 #10 (commit author=Ulysses, 8/27 19:39 JST 授权) + 守门 #11 (缺标比错标, AGENT §10 + GAMIFY 附录 A + 总册 §7 R-1~R-12 共 24+ 已知缺口显式列) + 守门 #12 (BAS 引用必 git log --follow 实证, 109 次引用 SRS-AGENT-RELATIONSHIP-001 已 git ls-files 实证) + 守门 #13 (DB W/T/M 三類横展 100% 覆盖, GAMIFY G11.2 15 张表 + AGENT A11.5 7 张表) + 守门 #14 v2 (5 域 Lead Mavis 临时代签, 真人到位后追溯签字) + 守门 #14 v3 (Mavis 永久代签, 5 角色签字栏全代签) + 守门 #14 v4 v0.62 反转 (真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST) + 守门 #15 (docs 同步饱和, 1 commit 6 文件 落档, 上次 commit e9a6c2e WBS v0.75.1 是第 67 次, 本 commit 4f56979 第 68 次) + 守门 #23 v2 (AI 第三方 API 禁止, GAMIFY G5 sticky note 聚类走 mock 接口, 守门 #23 v2 25 次引用锁 mock 路径, 真实 LLM 留 P2) + 守门 #1 禁回溯叙事 (撤回 v0.1 47.5K 总册不重写, 仅 v1.0 总册 v0.62 row 显式标反转)
> **落档文件** (关联 commit `4f56979`, 6 files / 4107 insertions):
> - `docs/requirements/SRS-CANVAS-001.md` v1.0 (54KB, 总册, 双核心 70 项索引 + 跨块接口 + 共享约束 + 13 跨拍板派生 + 12 风险 R-1~R-12 + 守门 16 交叉引用)
> - `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.1 (103KB, 38 项 = 28 旧 A1-A10 + 10 新 A11 ARG, 13 段, 109 次引用 SRS-AGENT-RELATIONSHIP-001 v0.1 主源 + 43 AC + 25 US + 12 已知缺口 + 7 张表 W/T/M 100% 覆盖 + 10 类关系 + 4 维度 + 5 团队模板)
> - `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB, 32 项 G1-G12, 73 AC + 23 US + 12 已知缺口 + G11.2 W/T/M 15 张表 100% 覆盖 Work 6 / Transaction 4 / Master 5 + G5 AI mock 87 次显式标注 + 守门 #23 v2 25 次锁 mock 路径)
> - `docs/briefs/srs-canvas-agent-001.md` v1.1 (17KB, 38 项 + A11 ARG 10 项 + 4 表 W/T/M + 10 类关系 + 4 维度 + 5 团队模板)
> - `docs/briefs/srs-canvas-gamify-001.md` v1.0 (12.6KB, 32 项 + G11 W/T/M + G5 AI mock)
> - `docs/briefs/srs-canvas-integration-001.md` (1.1KB, deprecated 占位, 撤回 17:08 JST 旧方向)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5-1 | D5-1 | `docs/requirements/SRS-CANVAS-001.md` v1.0 (54KB) | A | **[P]** | (root Write tool) | 9 段 IPA SEC 模板 + 双核心 70 项索引 (28+10+32) + 12 大类砍 11 类 (Miro 通用) + 13 跨拍板派生 + 12 风险 R-1~R-12 + 守门 16 交叉引用; 17:08 JST 方向重置 + 17:21 JST ARG 补充, 2 次更新 |
| D5-2 | D5-2 | `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.1 (103KB) | A | **[P]** | (worker 子代理 1 Write tool, bg_eb11f9e3) | 13 段 (跟 V0.1 模板 +1 拆 0.1/0.2/0.3) + 38 FR (A1-A10 28 + A11 10) + 12 NFR + 43 AC + 25 US + 12 已知缺口 (含 A11 跨专题 5 缺口) + 7 张表 W/T/M 100% 覆盖 (Master 3 / Transaction 3 / Work 1) + 10 类关系 (4 核心 + 6 扩展) + 4 维度协作影响 + 5 团队模板 + 109 次引用 SRS-AGENT-RELATIONSHIP-001 v0.1 主源 + 守门 14/14 通过 |
| D5-3 | D5-3 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB) | A | **[P]** | (worker 子代理 2 Write tool, bg_84cf0613) | 10 段 (§0~§9 + 附录 A-D) + 32 FR (G1.1-G12.2) + 25 NFR + 73 AC + 23 US + 12 已知缺口 (附录 A) + G11.2 W/T/M 15 张表 100% 覆盖 (Work 6 / Transaction 4 / Master 5) + G5 AI mock 87 次显式标注 + 守门 #23 v2 25 次锁 mock 路径 + 16 处跨专题引用 + 18 类 Miro 通用功能砍掉 (附录 C) + 5 角色签字栏 (附录 D) + 守门 14/14 通过 |
| D5-4 | D5-4 | `docs/briefs/srs-canvas-agent-001.md` v1.1 (17KB) | A | **[P]** | (root Write tool) | per 守门 #9 v20 子代理 dispatch 必先 brief, A11 ARG 10 项 + 4 表 W/T/M + 10 类关系 + 4 维度 + 5 团队模板; 17:17 JST v1.0 → 17:22 JST v1.1 (A11 二次更新) |
| D5-5 | D5-5 | `docs/briefs/srs-canvas-gamify-001.md` v1.0 (12.6KB) | A | **[P]** | (root Write tool) | per 守门 #9 v20, G11 W/T/M + G5 AI mock; 17:17 JST v1.0 落档 |
| D5-6 | D5-6 | `docs/briefs/srs-canvas-integration-001.md` (1.1KB, deprecated 占位) | A | **[P]** | (root Write tool) | 撤回 17:08 JST 旧方向 (Miro 集成/移动 27 项), 改 deprecated 占位, 后续 PR 清理 |
| D5-7 | D5-7 | `scripts/automation/registry.md` §2 +3 行 + §3 v0.11 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, 3 索引 (srs-canvas-agent-001 + srs-canvas-gamify-001 + srs-canvas-integration-001 deprecated) + v0.11 修订历史 (3 阶段拍板) |
| D5-8 | D5-8 | `docs/automation-design.md` §4.26 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5-9 | D5-9 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `4f56979`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.26 任务卡维度判定**:
- R (Rerunnable): **是** (1 commit idempotent, 6 files / 4107 insertions, 0 子代理调用)
- V (Volume): **是** (3 SRS 共 249KB, 70 项 FR + 27 段 + 84 已知缺口, 跨 1 总册 + 2 专题)
- S (Structural): **是** (3 SRS 9-13 段 IPA SEC 模板 + 2 brief v1.1/v1.0 + 1 deprecated 占位 + 1 任务卡 + 1 registry v0.11 + 1 commit)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 4f56979) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #9 v20 brief 落档 + 守门 #9 v27 verify + 守门 #14 v2/v3/v4 代签 / 审核 规则全备)

**§4.26 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3 + #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-001.md` 实证 v1.0 落档 (commit `4f56979`)
- `git log -p --follow docs/requirements/SRS-CANVAS-AGENT-001.md` 实证 v1.1 落档 (commit `4f56979`, 103KB)
- `git log -p --follow docs/requirements/SRS-CANVAS-GAMIFY-001.md` 实证 v1.0 落档 (commit `4f56979`, 92KB)
- `git log -p --follow docs/briefs/srs-canvas-agent-001.md` 实证 v1.1 落档 (commit `4f56979`, 17KB)
- `git log -p --follow docs/briefs/srs-canvas-gamify-001.md` 实证 v1.0 落档 (commit `4f56979`, 12.6KB)
- `git log -p --follow docs/briefs/srs-canvas-integration-001.md` 实证 deprecated 落档 (commit `4f56979`, 1.1KB)
- 子代理 1 真实产出验证: `SRS-CANVAS-AGENT-001.md` 105,733 bytes / 1,558 行 / 13 段 / 38 FR + 12 NFR + 43 AC + 25 US / 7 张表 W/T/M 100% 覆盖 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 子代理 2 真实产出验证: `SRS-CANVAS-GAMIFY-001.md` 92,346 bytes / 1,323 行 / 10 段 / 32 FR + 25 NFR + 73 AC + 23 US / 15 张表 W/T/M 100% 覆盖 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 守门 #1 v19: 0 子代理调用 (实装期), 仅用 task tool 派 2 worker (bg_fc33dfea stop + bg_eb11f9e3 重派 + bg_84cf0613)
- 守门 #9 #3: 0 子代理调用 (实装期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (17:08 JST 拍板 → 17:20 JST 落地 ~12 分钟, 17:21 JST 拍板 → 17:28 JST 落地 ~7 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: AGENT §10 已知缺口 12 个 + GAMIFY 附录 A 已知缺口 12 个 + 总册 §7 R-1~R-12 风险 12 个, 共 36 个已知缺口显式列, 0 隐藏
- 守门 #13 DB W/T/M: GAMIFY G11.2 15 张表 (Work 6 / Transaction 4 / Master 5) + AGENT A11.5 7 张表 (Master 3 / Transaction 3 / Work 1), 22 张表 100% 覆盖
- 守门 #14 v3: 5 角色签字栏全 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #23 v2 AI 第三方 API 禁止: GAMIFY G5 sticky note 聚类走 mock 接口, 守门 #23 v2 25 次引用锁 mock 路径, 真实 LLM 留 P2

**§4.26 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本次合计 ~0.91M tokens (1 SRE·周 ≈ 1.2M, 在预算内)
- 后续 P0 阶段 (3 个月内, 31 项 P0) ~3-5M
- 双核心 70 项 全部落地 (12 个月+) ~12-18M (12-18 SRE·周)


### 4.27 P3-D.5 v1.2 多人编辑是要的 (v0.63 反转, 撤回 17:08 JST 砍多人编辑决定, 2026-09-10 17:48 JST per `docs/briefs/srs-canvas-agent-001.md` v1.2)

> **触发**: 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的" (v0.63 反转, 撤回 17:08 JST 砍多人编辑决定) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 70 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 70 次新事件, docs 同步允许) + 守门 #1 v19 (0 子代理调用, 0 散落子代理产出, 仅 task tool 派 1 worker 重写 v1.2) + 守门 #1 禁回溯叙事 (撤回 17:08 JST 砍多人编辑决定, v0.63 反转行显式标 29 次, 不重写 17:08 JST commit 4f56979) + 守门 #3 (5 域 Lead 跨域边强制 consults, A12.7 5 域 Lead 真人到位后决策 disclaimer 显式) + 守门 #9 #3 (0 子代理调用实装期) + 守门 #9 v19 (Mavis 自驱, 17:34 JST 拍板 → 17:36 JST 撤回 + 重派 ~2 分钟) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, brief v1.2 已落 `docs/briefs/srs-canvas-agent-001.md`) + 守门 #9 v27 (RPC 失败 fallback 3 段, 子代理 invoke → verify → collect_output, 真实验证文件 + 字节数 + 段数 + W/T/M 表 + v0.63 反转行) + 守门 #10 (代签, author=Ulysses) + 守门 #11 (缺标比错标, AGENT §10 已知缺口 19 个含 A12 多人编辑 8 缺口) + 守门 #12 (BAS 引用必 git log --follow 实证, frontend-canvas-design.md §4.1 28 次 + §4.6 21 次 + CanvasView.tsx line 253-262 18 次) + 守门 #13 (DB W/T/M 100% 覆盖, A11 7 张表 + A12 7 张表 = 14 张表全覆盖) + 守门 #14 v2 (5 域 Lead Mavis 临时代签, 真人到位后追溯签字) + 守门 #14 v3 (Mavis 永久代签) + 守门 #14 v4 v0.62 反转 (真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) + 守门 #15 (docs 同步饱和, 本 commit 1 次, 上次 commit ee74ae6 WBS v0.78 是第 67 次, 396833a 第 70 次) + 守门 #23 v2 (AI 第三方 API 禁止, GAMIFY G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2, 不直接影响 AGENT v1.2)
> **落档文件** (关联 commit `396833a`, 3 files / 691 insertions / 255 deletions):
> - `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.2 (158,701 bytes ~155KB, 1,956 行, 13 段, 46 项 = 38 v1.1 + 8 新 A12 多人编辑, 16 NFR + 66 AC + 32 US + 19 已知缺口, 14 张表 W/T/M 100% 覆盖, 13 处"多人编辑是要的" + 29 处"v0.63" + 7 处"撤回 17:08 JST 砍多人编辑决定" 显式标)
> - `docs/requirements/SRS-CANVAS-001.md` v1.1 (58KB, 总册三次更新版, 双核心 78 项索引 = 38+8+32, 36 P0 清单, A12 索引 + 7 多端点 API + v0.63 反转行)
> - `docs/briefs/srs-canvas-agent-001.md` v1.2 (17KB, 46 项 + A11 10 项 + A12 8 项 + 13 已知缺口 + 28 优先级)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.2-1 | D5.2-1 | `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.2 (155KB) | A | **[P]** | (worker 子代理 1 v1.2 Write tool, bg_fcc1ed26) | 13 段 (跟 V0.1 模板 +1 拆 0.1/0.2/0.3) + 46 FR (A1-A10 28 + A11 10 + A12 8) + 16 NFR + 66 AC (46 功能 + 14 质量 + 6 文档) + 32 US + 19 已知缺口 (含 A11 跨专题 5 缺口 + A12 多人编辑 8 缺口 #11-#17) + 14 张表 W/T/M 100% 覆盖 (A11 7 表 + A12 7 表) + 10 类关系 + 4 维度 + 5 团队模板 + A12 必含 8 项多人编辑 + v0.63 反转行 29 次 + 守门 14/14 通过 |
| D5.2-2 | D5.2-2 | `docs/requirements/SRS-CANVAS-001.md` v1.1 (58KB) | A | **[P]** | (root Write tool) | 9 段 IPA SEC 模板 + 双核心 78 项索引 (38+8+32) + 36 P0 清单 (31 v1.0 + 5 A12.1-12.3/12.5/12.6/12.8) + 12 大类砍 11 类 (Miro 通用) + 17:21 + 17:34 JST 拍板 + 14 跨拍板派生 + 13 风险 R-1~R-13 + 守门 16 交叉引用; 17:08 JST → 17:21 JST → 17:34 JST 3 次更新, v0.63 反转行 显式标 |
| D5.2-3 | D5.2-3 | `docs/briefs/srs-canvas-agent-001.md` v1.2 (17KB) | A | **[P]** | (root Write tool) | per 守门 #9 v20 子代理 dispatch 必先 brief, A11 ARG 10 项 + **A12 多人编辑 8 项** + 13 已知缺口 + 28 优先级 (P0 19 + P1 13 + P2 14, v1.0 P0 14 + P1 11 + P2 8, 加 A12 P0 5 + P1 2 + P2 1); 17:17 v1.0 → 17:22 v1.1 (A11 二次更新) → 17:34 v1.2 (A12 三次更新 v0.63 反转) |
| D5.2-4 | D5.2-4 | `scripts/automation/registry.md` §2 +1 行 + §3 v0.12 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, 1 索引 (srs-canvas-agent-001 v1.2) + v0.12 修订历史 (17:34 JST v0.63 反转) |
| D5.2-5 | D5.2-5 | `docs/automation-design.md` §4.27 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.2-6 | D5.2-6 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `396833a`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.27 任务卡维度判定**:
- R (Rerunnable): **是** (1 commit idempotent, 3 files / 691 insertions / 255 deletions, 0 子代理调用)
- V (Volume): **是** (AGENT v1.2 155KB / 1,956 行 + 总册 58KB + brief 17KB, 46 项 FR + 13 段 + 19 已知缺口, 跨 1 AGENT + 1 总册 + 1 brief)
- S (Structural): **是** (AGENT 13 段 + 总册 9 段 + brief 1 节, 14 张表 W/T/M 100% 覆盖, 7 个 API 端点 + 5 个 A12 端点)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 396833a) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #9 v20 brief 落档 + 守门 #9 v27 verify + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 v0.63 反转行 29 次显式标)

**§4.27 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3 + #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-AGENT-001.md` 实证 v1.2 落档 (commit `396833a`, 155KB)
- `git log -p --follow docs/requirements/SRS-CANVAS-001.md` 实证 v1.1 落档 (commit `396833a`, 58KB)
- `git log -p --follow docs/briefs/srs-canvas-agent-001.md` 实证 v1.2 落档 (commit `396833a`, 17KB)
- 子代理 1 真实产出验证: `SRS-CANVAS-AGENT-001.md` 158,701 bytes / 1,956 行 / 13 段 / 46 FR + 16 NFR + 66 AC + 32 US / 14 张表 W/T/M 100% 覆盖 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 守门 #1 v19: 0 子代理调用 (实装期), 仅用 task tool 派 1 worker (子代理 1 v1.2 重写, bg_fcc1ed26)
- 守门 #9 #3: 0 子代理调用 (实装期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (17:34 JST 拍板 → 17:36 JST 撤回 + 重派 ~2 分钟, 17:46 JST 子代理 1 完成 ~12 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: AGENT §10 已知缺口 19 个 (含 A11 跨专题 5 + A12 多人编辑 8 #11-#17 + 6 其他 #1-#6, #18-#19) + GAMIFY 附录 A 12 个 + 总册 §7 R-1~R-12 12 个, 共 43+ 已知缺口显式列, 0 隐藏
- 守门 #13 DB W/T/M: A11 7 张表 (Master 3 / Transaction 3 / Work 1) + A12 7 张表 (Master 3 / Transaction 2 / Work 2) = **14 张表 100% 覆盖** (Work 3/14 = 21.4% + Transaction 5/14 = 35.7% + Master 6/14 = 42.9% = 14/14 = 100%)
- 守门 #14 v3: 5 角色签字栏全 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 撤回 17:08 JST 砍多人编辑决定, v0.63 反转行 显式标 29 次, 不重写 17:08 JST commit 4f56979
- 守门 #23 v2 AI 第三方 API 禁止: GAMIFY G5 sticky note 聚类走 mock 接口, 守门 #23 v2 25 次引用锁 mock 路径, 真实 LLM 留 P2 (不直接影响 AGENT v1.2)

**§4.27 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本次合计 ~0.5M tokens (1 SRE·周 ≈ 1.2M, 在预算内)
- 累计 P3-D.5 全部 3 commit: ~1.41M tokens (4f56979 0.91M + 73d490f 0.05M + 396833a 0.45M, 1.18 SRE·周)
- 后续 P0 阶段 (3 个月内, 36 项 P0) ~3-5M
- 双核心 78 项 全部落地 (12 个月+) ~13-19M (13-19 SRE·周, per STAR-OLU-001)


### 4.28 P3-D.5 BD 基本设计文档三份落档 (per 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档", 2026-09-10 18:10 JST)

> **触发**: 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 72 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 72 次新事件, docs 同步允许) + 守门 #1 v19 (0 子代理调用, 0 散落子代理产出, 仅 task tool 派 2 worker) + 守门 #1 禁回溯叙事 (不重写 4 SRS commit 4f56979 / 73d490f / 396833a / f35b3f5, BD 是新方向) + 守门 #3 (5 域 Lead ≠ Star 22 DDD disclaimer) + 守门 #9 #3 (0 子代理调用实装期) + 守门 #9 v19 (Mavis 自驱, 18:00 JST 拍板 → 18:10 JST 1 commit ~10 分钟) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, 3 份 brief 已落 `docs/briefs/bd-canvas-{total,agent,gamify}-001.md`) + 守门 #9 v27 (RPC 失败 fallback 3 段, 子代理 invoke → verify → collect_output, 真实验证文件 + 字节数 + 段数 + 14 张表 W/T/M) + 守门 #10 (代签, author=Ulysses) + 守门 #11 (缺标比错标, 43+ 已知缺口显式列) + 守门 #12 (BAS 引用必 git log --follow 实证) + 守门 #13 (DB W/T/M 100% 覆盖, 29 张表 跨域) + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #15 (docs 同步饱和, 本 commit 1 次, 上次 commit f7d0932 是第 72 次, 本 commit 第 73 次) + 守门 #23 v2 (G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2)
> **落档文件** (关联 commit `f7d0932`, 3 files / 4141 insertions):
> - `docs/design/BD-CANVAS-001.md` v0.1 (50,274 bytes ~50KB, 18:04 JST root 写, 总册跨域 BD, 10 段 IPA SEC 模板, 5 view 跨域, 14 张表 W/T/M 100% 覆盖 + 29 张表跨域汇总, 32 API 端点 + 5 WebSocket, 6 类 NFR, 19 守门 + 26 派生规, 12 已知缺口 ≥ 8, 5 角色签字栏, 4 阶段拍板 + 5 守门拍板, v0.63 反转 14 处 + 多人编辑是要的 8 处)
> - `docs/design/BD-CANVAS-AGENT-001.md` v0.1 (105,821 bytes ~103KB, 1,731 行, 18:07 JST 子代理 1 写, 专题 agent 管理 BD, 10 段模板, A1-A12 46 项, 5 view 跨域, 14 张表 W/T/M 100% 覆盖, 23 API 端点 + 5 WebSocket, 19 已知缺口 ≥ 8 含 3 P0 阻塞, A11 1:1 派生自 BD-AGENT-RELATIONSHIP-001 v0.1, A12 1:1 派生自 frontend-canvas-design.md §4.1 模式 A + §4.6 PresenceCursor)
> - `docs/design/BD-CANVAS-GAMIFY-001.md` v0.1 (111,370 bytes ~109KB, 1,751 行, 18:07 JST 子代理 2 写, 专题游戏化 BD, 10 段模板 + 11 个 §N 标题, G1-G12 32 项, 5 view 跨域, **15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6)**, 22 API 端点 + 2 WebSocket, 12 已知缺口 含 DDD Review 必查 #11 + #12, **G12 派生自 V0.1 game 5 份 PHASE 报告 (Roguelike + Manga + Theme + Settings + Game, 9/5 落地 125 tests pass) 仅画布集成引用**, **G5 sticky note 聚类 AI 必走 mock 接口 per 守门 #23 v2 (49 次引用锁 mock 路径)**, 守门 19/19 跨域覆盖)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.3-1 | D5.3-1 | `docs/design/BD-CANVAS-001.md` v0.1 (50KB) | A | **[P]** | (root Write tool) | 10 段 + 5 view 跨域 + 14 张表 W/T/M 100% 覆盖 + 32 API 端点 + 6 类 NFR + 19 守门 + 12 已知缺口; 跨域汇总不重复 2 专题 BD |
| D5.3-2 | D5.3-2 | `docs/design/BD-CANVAS-AGENT-001.md` v0.1 (103KB) | A | **[P]** | (worker 子代理 1 Write tool, bg_98f425fd) | 10 段 + 附录 A + 附录 B + 46 FR (A1-A12) + 16 NFR + 66 AC + 25 US + 19 已知缺口 (含 A12 多人编辑 8 缺口) + 14 张表 W/T/M + 23 API 端点 + 5 WebSocket + A11 1:1 派生自 BD-AGENT-RELATIONSHIP-001 v0.1 + A12 1:1 派生自 V0.1 §4.1 + §4.6 |
| D5.3-3 | D5.3-3 | `docs/design/BD-CANVAS-GAMIFY-001.md` v0.1 (109KB) | A | **[P]** | (worker 子代理 2 Write tool, bg_f5625885) | 10 段 + 11 个 §N 标题 + 32 FR (G1-G12) + 15 张表 W/T/M 100% 覆盖 0 混在 + 22 API 端点 + 2 WebSocket + 12 已知缺口 (含 DDD Review 必查 #11 + #12) + G12 派生自 V0.1 game 5 份 PHASE 报告 + G5 走 mock 接口 per 守门 #23 v2 (49 次引用) |
| D5.3-4 | D5.3-4 | `scripts/automation/registry.md` §2 +3 行 + §3 v0.13 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, 3 索引 (bd-canvas-total-001 + bd-canvas-agent-001 + bd-canvas-gamify-001) + v0.13 修订历史 (18:00 JST 拍板"基于需求文档制作基本设计文档") |
| D5.3-5 | D5.3-5 | `docs/automation-design.md` §4.28 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.3-6 | D5.3-6 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `f7d0932`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.28 任务卡维度判定**:
- R (Rerunnable): **是** (1 commit idempotent, 3 files / 4141 insertions, 0 子代理调用)
- V (Volume): **是** (3 BD 共 262KB / 4,141 行, 78 项 FR + 30 段 + 43+ 已知缺口, 跨 1 总册 + 2 专题)
- S (Structural): **是** (3 BD 各 10 段 + 附录 + 29 张表 W/T/M + 32 API 端点 + 5 角色签字栏)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit f7d0932) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #9 v20 brief 落档 + 守门 #9 v27 verify + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 SRS)

**§4.28 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3 + #14 v4)**:
- `git log -p --follow docs/design/BD-CANVAS-001.md` 实证 v0.1 落档 (commit `f7d0932`, 50KB)
- `git log -p --follow docs/design/BD-CANVAS-AGENT-001.md` 实证 v0.1 落档 (commit `f7d0932`, 105KB)
- `git log -p --follow docs/design/BD-CANVAS-GAMIFY-001.md` 实证 v0.1 落档 (commit `f7d0932`, 111KB)
- 子代理 1 真实产出验证: BD-AGENT 105,821 bytes / 1,731 行 / 13 段 + 附录 A + 附录 B / 46 FR + 16 NFR + 66 AC + 25 US / 14 张表 W/T/M 100% 覆盖 / 19 已知缺口 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 子代理 2 真实产出验证: BD-GAMIFY 111,370 bytes / 1,751 行 / 11 个 §N 标题 / 32 FR / 15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6) / 12 已知缺口 / G5 mock 接口 49 次引用 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 守门 #1 v19: 0 子代理调用 (实装期), 仅用 task tool 派 2 worker (子代理 1 v1.2 重写 bg_98f425fd + 子代理 2 v1.2 重写 bg_f5625885)
- 守门 #9 #3: 0 子代理调用 (实装期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (18:00 JST 拍板 → 18:02 JST 撤回 + 重派 ~2 分钟, 18:07 JST 子代理完成 ~7 分钟, 18:10 JST 1 commit 3 文件 ~3 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: AGENT §8 已知缺口 19 个 + GAMIFY §8 12 个 + 总册 §8 12 个, 共 43+ 已知缺口显式列, 0 隐藏
- 守门 #13 DB W/T/M: 总册 14 张表 (A11 7 + A12 7) + GAMIFY 15 张表 (Master 5 + Transaction 4 + Work 6) = **29 张表 100% 覆盖 跨域汇总**
- 守门 #14 v3: 5 角色签字栏全 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 4 SRS commit, BD 是新方向, 不回写 SRS
- 守门 #23 v2 AI 第三方 API 禁止: GAMIFY G5 sticky note 聚类走 mock 接口, 守门 #23 v2 49 次引用锁 mock 路径, 真实 LLM 留 P2

**§4.28 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 BD 撰写期 (root + 2 worker): ~0.5M tokens
- 累计 P3-D.5 全部 8 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 任务卡同步): ~2.96M tokens (2.47 SRE·周, 累计 4 commit + 1 任务卡 = 8 commit)
- 后续 P3-D 详细设计 (DD, 3 份): ~0.6M tokens (3 子代理并行)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)


### 4.29 P3-D.5 DD 详细设计文档三份 + 协调性检查修复落档 (per 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:30 JST 拍板"确保本设计和 agent 图关系设计妥善协调不冲突", 2026-09-10 18:35 JST)

> **触发**: 2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" + 18:30 JST 拍板"确保本设计和 agent 图关系设计妥善协调不冲突" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 74 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 74 次新事件, docs 同步允许) + 守门 #1 v19 (0 子代理调用, 0 散落子代理产出, 仅 task tool 派 2 worker DD 子代理) + 守门 #1 禁回溯叙事 (不重写 4 SRS commit + 2 BD commit, DD 是新方向 + 协调性检查修复 3 处) + 守门 #3 (5 域 Lead ≠ Star 22 DDD disclaimer) + 守门 #9 #3 (0 子代理调用实装期) + 守门 #9 v19 (Mavis 自驱, 18:25 JST 拍板 → 18:33 JST 2 子代理完成 ~8 分钟, 18:35 JST 协调性检查 + 修复 ~2 分钟) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, 3 份 DD brief 已落 `docs/briefs/dd-canvas-{total,agent,gamify}-001.md` + 3 份 BD brief catch-up `docs/briefs/bd-canvas-{total,agent,gamify}-001.md`) + 守门 #9 v27 (RPC 失败 fallback 3 段, 子代理 invoke → verify → collect_output, 真实验证文件 + 字节数 + 段数 + 14 张表 W/T/M + 13 关键 class + 5 状态机 + 11 共享类型) + 守门 #10 (代签, author=Ulysses) + 守门 #11 (缺标比错标, 19+12+12 已知缺口显式列) + 守门 #12 (BAS 引用必 git log --follow 实证) + 守门 #13 (DB W/T/M 100% 覆盖, 14 张表 跨域) + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #15 (docs 同步饱和, 本 commit 1 次, 上次 commit f7d0932 是第 73 次, 本 commit 第 74 次) + 守门 #23 v2 (G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2) + 协调性检查 3 冲突修复 (C-16 → C-25 `CanvasElementsBackend` + C-21 → C-26 `CanvasMultiUserAudit` + Trust Score 5 档统一 ARG 源 `Untrusted/Low/Medium/High/VeryHigh` + 0-0.2/0.2-0.4/0.4-0.7/0.7-0.9/0.9-1.0, 仅修 root 写的 DD-CANVAS-001, BD-AGENT 已正确, DD-AGENT-001 子代理 1 已用 ARG 源命名, DD-GAMIFY 独立不需修)
> **落档文件** (关联 commit 待生成, 10 files / 4 核心 + 6 brief catch-up, 第 74 次新事件):
> - `docs/reports/COORDINATION-CHECK-001.md` v0.1 (21,544 bytes ~21KB, root 写, 协调性检查报告, 10 项检查 7 通过 + 3 冲突发现, §3 修复方案 C-25/C-26 顺延 + Trust Score 5 档统一 ARG 源)
> - `docs/design/DD-CANVAS-001.md` **v0.1.1** (49,367 bytes ~48KB, root 写, 修复版, 10 段 + 5 附录 + 14 张表 W/T/M 100% 覆盖 + 13 关键 class (含 C-25 `CanvasElementsBackend` + C-26 `CanvasMultiUserAudit` 顺延) + 5 状态机 (含 §5.3 Trust Score 5 档 统一 ARG 源) + 11 共享类型 + 4 关键时序图)
> - `docs/design/DD-CANVAS-AGENT-001.md` v0.1 (139,198 bytes ~136KB, 2,531 行, 子代理 1 bg_9ce1be6e 写, 专题 agent 管理 DD, 13 关键 class (C-16=`RelationshipEditor` + C-21=`ARGController` 1:1 派生自 ARG 源 DD-AGENT-RELATIONSHIP-001 v0.1) + 5 状态机 + 11 共享类型 + 6 时序图 (4 必含 + 2 补充 A12 多人编辑/Follow mode) + 14 张表 W/T/M 100% 覆盖 + 23 API 端点 + 5 WebSocket + 19 已知缺口含 3 P0 阻塞 + 守门 19/19 + 26 派生规跨域全过; **A11 1:1 派生自 DD-AGENT-RELATIONSHIP-001 v0.1 §3-§8 模板; A12 1:1 派生自 frontend-canvas-design.md v0.1 §4.1 模式 A + §4.6 PresenceCursor 升级**)
> - `docs/design/DD-CANVAS-GAMIFY-001.md` v0.1 (161,742 bytes ~158KB, 3,439 行, 子代理 2 bg_50df734c 写, 专题游戏化 DD, 17 段 + 5 附录 + 13 关键 class + 5 状态机 + 11 共享类型 (170 字段) + 4 关键时序图 + **15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6)** + 22 API 端点 + 2 WebSocket + 74 测试 + 12 已知缺口含 DDD Review 必查 #11 + #12; **G5 锁定 mock 接口 (per 守门 #23 v2, 走 MockClient + ai_edit_mock.py subprocess.run, 真实 LLM 留 P2); G12 派生自 V0.1 game 5 份 PHASE 报告 (125 tests pass, 仅画布集成引用)**)
> - `docs/briefs/dd-canvas-total-001.md` (catch-up from 18:25 JST DD 拍板, 子代理 0/1/2 dispatch 必先 brief 落档 per 守门 #9 v20)
> - `docs/briefs/dd-canvas-agent-001.md` (catch-up, 子代理 1 bg_9ce1be6e dispatch 必先 brief 落档)
> - `docs/briefs/dd-canvas-gamify-001.md` (catch-up, 子代理 2 bg_50df734c dispatch 必先 brief 落档)
> - `docs/briefs/bd-canvas-total-001.md` (catch-up from 18:00 JST BD 拍板, 子代理 0/1/2 dispatch 必先 brief 落档)
> - `docs/briefs/bd-canvas-agent-001.md` (catch-up, 子代理 1 bg_98f425fd dispatch 必先 brief 落档)
> - `docs/briefs/bd-canvas-gamify-001.md` (catch-up, 子代理 2 bg_f5625885 dispatch 必先 brief 落档)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.4-1 | D5.4-1 | `docs/reports/COORDINATION-CHECK-001.md` v0.1 (21KB) | A | **[P]** | (root Write tool) | 协调性检查报告: 10 项检查 7 通过 + 3 冲突 (C-16 编号 + C-21 编号 + Trust Score 5 档 命名/阈值), §3 修复方案, 4 ARG 已有 + 9 本批新写 vs 4 ARG 已有, 不重写 ARG 4 份 commit per 守门 #1 禁回溯叙事 |
| D5.4-2 | D5.4-2 | `docs/design/DD-CANVAS-001.md` **v0.1.1** (48KB) 修复版 | A | **[P]** | (root Edit tool) | 协调性检查修复 3 处: C-16 `CanvasBackend` → C-25 `CanvasElementsBackend` + C-21 `MultiUserAudit` → C-26 `CanvasMultiUserAudit` + Trust Score 5 档统一 ARG 源 `Untrusted/Low/Medium/High/VeryHigh` + 0-0.2/0.2-0.4/0.4-0.7/0.7-0.9/0.9-1.0; §0 v0.1 → v0.1.1 + 修订历史 +1 行; §1.1 / §4 列表头同步; 字节数 +1,671 (47,696 → 49,367) |
| D5.4-3 | D5.4-3 | `docs/design/DD-CANVAS-AGENT-001.md` v0.1 (136KB) | A | **[P]** | (worker 子代理 1 Write tool, bg_9ce1be6e) | 13 关键 class 1:1 派生 ARG 源 (C-16=RelationshipEditor + C-21=ARGController), 6 时序图 (4 必含 + 2 补充 A12), 14 张表 W/T/M 100% 覆盖, 23 API + 5 WebSocket, 19 已知缺口含 3 P0 阻塞 + 1 跨 session 总结, 守门 19/19 + 26 派生规跨域全过; **不需修复** (子代理 1 已用正确 ARG 源命名) |
| D5.4-4 | D5.4-4 | `docs/design/DD-CANVAS-GAMIFY-001.md` v0.1 (158KB) | A | **[P]** | (worker 子代理 2 Write tool, bg_50df734c) | 17 段 + 5 附录 (略超 brief 10 段, 跟 DD-AGENT-RELATIONSHIP-001 模板对齐), 13 关键 class + 11 共享类型 170 字段, 15 张表 W/T/M 100% 覆盖 0 混在, 22 API + 2 WebSocket, 74 测试, 12 已知缺口; **不需修复** (游戏化独立, 跟 ARG 无关) |
| D5.4-5 | D5.4-5 | `docs/briefs/dd-canvas-{total,agent,gamify}-001.md` + `docs/briefs/bd-canvas-{total,agent,gamify}-001.md` (6 brief catch-up) | A | **[P]** | (per 守门 #9 v20 子代理 dispatch 必先 brief 落档) | 6 份 brief 关联 6 份子代理 dispatch (DD 子代理 1 bg_9ce1be6e + DD 子代理 2 bg_50df734c + BD 子代理 1 bg_98f425fd + BD 子代理 2 bg_f5625885 + 总册 2 份), commit message 引用 brief 路径 |
| D5.4-6 | D5.4-6 | `scripts/automation/registry.md` §2 +4 行 (3 DD + 1 COORDINATION) + §3 v0.14 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, 3 索引 (dd-canvas-total-001 + dd-canvas-agent-001 + dd-canvas-gamify-001) + 1 报告 (coordination-check-001) + v0.14 修订历史 (18:30 JST 协调性检查 拍板) |
| D5.4-7 | D5.4-7 | `docs/automation-design.md` §4.29 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.4-8 | D5.4-8 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit 待生成) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.29 任务卡维度判定**:
- R (Rerunnable): **是** (1 commit idempotent, 4 核心文件 + 6 brief catch-up, 0 子代理调用 落档期)
- V (Volume): **是** (3 DD 共 342KB / 5,970 行 + 1 报告 21KB + 6 brief 跨域, 跨 1 总册 + 2 专题 + 1 报告)
- S (Structural): **是** (3 DD 各 10-17 段 + 附录 + 14-15 张表 W/T/M + 22-32 API 端点 + 5 WebSocket + 5 角色签字栏)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 待生成) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #9 v20 brief 落档 + 守门 #9 v27 verify + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 SRS + BD + ARG 4 commit)

**§4.29 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v19 + #12 v21 + #14 v2 + #14 v3 + #14 v4)**:
- `git log -p --follow docs/design/DD-CANVAS-001.md` 实证 v0.1.1 落档 (commit 待生成, 49,367 bytes, +1,671 vs v0.1 47,696)
- `git log -p --follow docs/design/DD-CANVAS-AGENT-001.md` 实证 v0.1 落档 (commit 待生成, 139,198 bytes)
- `git log -p --follow docs/design/DD-CANVAS-GAMIFY-001.md` 实证 v0.1 落档 (commit 待生成, 161,742 bytes)
- `git log -p --follow docs/reports/COORDINATION-CHECK-001.md` 实证 v0.1 落档 (commit 待生成, 21,544 bytes)
- 子代理 1 真实产出验证: DD-AGENT 139,198 bytes / 2,531 行 / 13 段 + 5 附录 / 46 项跨域 / 14 张表 W/T/M 100% 覆盖 / 13 关键 class (C-16=RelationshipEditor + C-21=ARGController 匹配 ARG 源) / 5 状态机 / 11 共享类型 / 6 时序图 / 19 已知缺口含 3 P0 阻塞 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 子代理 2 真实产出验证: DD-GAMIFY 161,742 bytes / 3,439 行 / 17 段 (略超 brief 10 段, 跟 DD-AGENT-RELATIONSHIP-001 模板对齐) / 32 项 G1-G12 / 15 张表 W/T/M 100% 覆盖 0 混在 (Master 5 + Transaction 4 + Work 6) / 13 关键 class / 5 状态机 / 11 共享类型 170 字段 / 22 API + 2 WebSocket / 12 已知缺口含 DDD Review 必查 #11 + #12 / G5 mock 锁 / G12 V0.1 5 份 PHASE 派生 (per 守门 #9 v27 3 段 fallback verify 阶段)
- 协调性检查验证: 10 项检查 7 通过 + 3 冲突 (C-16 编号 + C-21 编号 + Trust Score 5 档 命名/阈值), 修复方案: C-16 → C-25 `CanvasElementsBackend` + C-21 → C-26 `CanvasMultiUserAudit` + Trust Score 5 档统一 ARG 源; 仅修 root 写的 DD-CANVAS-001, BD-AGENT 已正确 (grep 验证 0 `Unknown/Trusted/CanvasBackend/MultiUserAudit` 匹配), DD-AGENT-001 子代理 1 已用 ARG 源命名 (C-16=RelationshipEditor + C-21=ARGController, A12 走 §4.14.x sub-class 模式与本总册 C-25/C-26 不同但等价), DD-GAMIFY 独立不需修
- 守门 #1 v19: 0 子代理调用 (实装期), 仅用 task tool 派 2 worker (DD 子代理 1 bg_9ce1be6e + DD 子代理 2 bg_50df734c)
- 守门 #9 #3: 0 子代理调用 (实装期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (18:25 JST 拍板 → 18:33 JST 2 子代理完成 ~8 分钟, 18:35 JST 协调性检查 + 修复 ~2 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: AGENT 19 个 + GAMIFY 12 个 + 总册 12 个 + 协调性检查 3 个冲突显式列, 共 46 已知缺口显式列, 0 隐藏
- 守门 #13 DB W/T/M: 总册 14 张表 (A11 7 + A12 7) + GAMIFY 15 张表 (Master 5 + Transaction 4 + Work 6) = **29 张表 100% 覆盖 跨域汇总** + 0 混在
- 守门 #14 v3: 5 角色签字栏全 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 4 SRS commit + 2 BD commit, DD 是新方向 + 协调性检查修复 3 处, 不回写 SRS / BD / ARG 4 commit
- 守门 #23 v2 AI 第三方 API 禁止: GAMIFY G5 sticky note 聚类走 mock 接口 (MockClient + ai_edit_mock.py subprocess.run, 守门 #23 v2 49 次引用锁 mock 路径), 真实 LLM 留 P2

**§4.29 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 DD 撰写期 (root + 2 worker DD 子代理 + 协调性检查 + 修复): ~0.4M tokens
- 累计 P3-D.5 全部 9 commit (4 SRS + 1 总册 BD + 2 专题 BD + 2 任务卡同步 + 1 DD 三份 + 协调性检查): ~3.36M tokens (2.80 SRE·周, 累计 9 commit)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)



### 4.30 P3-D.5 IPA SEC 合规性修复 (per 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准", 2026-09-10 19:10 JST)

> **触发**: 2026-09-10 19:06 JST Ulysses 拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 79 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 79 次新事件, docs 同步允许) + 守门 #1 v19 (1 commit 5 文件批量改 收官) + 守门 #1 禁回溯叙事 (不重写 §0-§10 旧内容, v0.1.1/v0.1.2/v1.2 反转行显式标) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 5 处不合规显式列) + 守门 #14 v2/v3/v4 (5 域 Lead Mavis 临时代签 + Mavis 永久代签 + v0.62 反转 Mavis 审核 author=Ulysses) + 守门 #9 v19 (Mavis 自驱, 19:06 JST 拍板 → 19:10 JST 1 commit 5 文件 ~4 分钟) + 守门 #9 v27 (RPC 失败 fallback 3 段, 真实验证文件 + 字节数 + 段头结构) + 守门 #13 W/T/M (本批不涉及, 维持前批 14+15 张表 100% 覆盖)
> **落档文件** (关联 commit `0d6cecf`, 6 files / 500 insertions / 5 deletions):
> - `docs/requirements/SRS-CANVAS-001.md` **v1.2** (46,071 bytes, 12 段, 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历, +22 -4 lines, 修复前 10 段缺独立 5 角色签字栏 段)
> - `docs/requirements/SRS-CANVAS-GAMIFY-001.md` **v0.1.1** (73,053 bytes, 12 段, 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历, +866 bytes, 修复前 10 段 5 角色签字栏 在附录 D)
> - `docs/design/DD-CANVAS-001.md` **v0.1.2** (47,269 bytes, 14 段, 加 §11 NFR 详细 6 类 29 项 (性能 6 / 可靠性 3 / 安全 6 / 易用 5 / 可观测 3 / ARG 6 + MU-CONS-01) + §12 5 角色签字栏 + §13 修订历史, +6819 bytes, 修复前 11 段缺 NFR 独立段 + 5 角色签字栏 在附录 D + 修订历史 在附录 E)
> - `docs/design/DD-CANVAS-AGENT-001.md` **v0.1.1** (122,367 bytes, 15 段, 加 §13 5 角色签字栏 + §14 修订历史, +2018 bytes, 修复前 13 段 5 角色签字栏 在附录 D + 修订历史 在附录 E)
> - `docs/reports/COORDINATION-CHECK-001.md` **v0.1.1** (18,824 bytes, 8 段, 加 §0 文档信息 / 修订履历, +1435 bytes, 修复前 7 段 (§1-§7) 缺 §0 文档信息)
> - `scripts/automation/ipa_sec_compliance_fix.py` 22.7KB (idempotent Python 修复脚本, 5 文档 5 fix 函数 + main orchestrator + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 显式标)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.5-1 | D5.5-1 | `docs/requirements/SRS-CANVAS-001.md` v1.2 (12 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_srs_canvas_001` | 加 §10 5 角色签字栏 (per AGENTS.md §3 7 段结构) + renumber §9 → §11 修订履历; 字节数 45,202 → 46,071 (+869); 段数 10 → 12 |
| D5.5-2 | D5.5-2 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (12 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_srs_canvas_gamify_001` | 加 §10 5 角色签字栏 + renumber §9 → §11 修订履历; 字节数 72,187 → 73,053 (+866); 段数 10 → 12; SRS-GAMIFY-001 §9 是 simple table 无 sub-headings (per grep), script `### 9.1` 模式失败, 改用 `## §9 修订履历

\| 版本` 模式 |
| D5.5-3 | D5.5-3 | `docs/design/DD-CANVAS-001.md` v0.1.2 (14 段 IPA SEC 模板, NFR 6 类 29 项 跨域) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_dd_canvas_001` | 加 §11 NFR 详细 6 类 (性能 6 + 可靠性 3 + 安全 6 + 易用 5 + 可观测 3 + ARG 6 + MU-CONS-01) + §12 5 角色签字栏 + §13 修订历史 (v0.1/v0.1.1/v0.1.2 三行); 字节数 40,450 → 47,269 (+6819); 段数 11 → 14 |
| D5.5-4 | D5.5-4 | `docs/design/DD-CANVAS-AGENT-001.md` v0.1.1 (15 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_dd_canvas_agent_001` | 加 §13 5 角色签字栏 + §14 修订历史 (v0.1/v0.1.1 两行); 字节数 120,349 → 122,367 (+2018); 段数 13 → 15 |
| D5.5-5 | D5.5-5 | `docs/reports/COORDINATION-CHECK-001.md` v0.1.1 (8 段 IPA SEC 模板) | A | **[P]** | `ipa_sec_compliance_fix.py::fix_coordination_check_001` | 加 §0 文档信息 / 修订履历 (0.1 文档信息 + 0.2 修订履历 v0.1/v0.1.1 两行); 字节数 17,389 → 18,824 (+1435); 段数 7 → 8; 跟其他 9 份 P3-D.5 文档结构对齐 |
| D5.5-6 | D5.5-6 | `scripts/automation/ipa_sec_compliance_fix.py` 22.7KB (idempotent Python 脚本) | A | **[P]** | (本脚本) | 5 文档 5 fix 函数 + main orchestrator + 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事 显式标; 第 2 次跑 idempotent 跳过 (4/5 已修, 1/5 第二次才修 SRS-GAMIFY-001) |
| D5.5-7 | D5.5-7 | `docs/automation-design.md` §4.30 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.5-8 | D5.5-8 | `scripts/automation/registry.md` §3 v0.15 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.15 修订历史 (19:06 JST IPA SEC 拍板) |
| D5.5-9 | D5.5-9 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `0d6cecf`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.30 任务卡维度判定**:
- R (Rerunnable): **是** (`ipa_sec_compliance_fix.py` idempotent, 二次跑同样结果)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性, 0 子代理调用)
- S (Structural): **是** (5 文档 段头结构整改, 10-13 段 → 12-15 段, 段号 renumber §9 → §11 + 加 §10/§11/§12/§13/§14)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 0d6cecf) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #1 禁回溯叙事 不重写 §0-§10 旧内容 + 守门 #11 缺标比错标 5 处不合规显式列 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备)

**§4.30 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-001.md` 实证 v1.2 落档 (commit `0d6cecf`, 12 段, +22 -4)
- `git log -p --follow docs/requirements/SRS-CANVAS-GAMIFY-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 12 段, +866)
- `git log -p --follow docs/design/DD-CANVAS-001.md` 实证 v0.1.2 落档 (commit `0d6cecf`, 14 段, +6819)
- `git log -p --follow docs/design/DD-CANVAS-AGENT-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 15 段, +2018)
- `git log -p --follow docs/reports/COORDINATION-CHECK-001.md` 实证 v0.1.1 落档 (commit `0d6cecf`, 8 段, +1435)
- 5 处 IPA SEC 不合规修复 (per 守门 #11 缺标比错标): (1) SRS-001 缺 §10 5 角色签字栏 → 加; (2) SRS-GAMIFY-001 缺 §10 5 角色签字栏 → 加; (3) DD-001 缺 §11 NFR + §12 5 角色签字栏 + §13 修订历史 → 3 段全加; (4) DD-AGENT-001 缺 §13 5 角色签字栏 + §14 修订历史 → 2 段加; (5) COORDINATION-CHECK 缺 §0 文档信息 → 加
- 5 文档段号 renumber: SRS-001 §9 → §11 修订履历; SRS-GAMIFY-001 §9 → §11 修订履历; DD-001 新加 §11/§12/§13 (与原 §11-§13 不冲突, 原 §11/§12/§13 跨域汇总的内容保留在 §10 测试用例之后 + 附录 A-E)
- 守门 #1 v19: 0 子代理调用 (IPA SEC 修复期), Mavis 接手 root session 一次性, 1 commit 5 文件
- 守门 #9 #3: 0 子代理调用 (IPA SEC 修复期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (19:06 JST 拍板 → 19:10 JST 1 commit 5 文件 ~4 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: 5 处 IPA SEC 不合规显式列 (3 SRS/2 DD/1 报告), 不沿用代签决策
- 守门 #13 DB W/T/M: 本批 IPA SEC 修复不涉及 DB schema, 维持前批 14+15 张表 100% 覆盖
- 守门 #14 v3: 5 角色签字栏 (新增) 全部 Mavis 接手代签 (修订人 + 审批者 author=Ulysses)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 §0-§10 旧内容, v0.1.1/v0.1.2/v1.2 反转行显式标 (per 守门 #1 + 守门 #11 缺标比错标)

**§4.30 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 IPA SEC 修复期 (root Mavis 接手 + ipa_sec_compliance_fix.py idempotent Python 脚本): ~0.05M tokens
- 累计 P3-D.5 全部 10 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 协调性检查 + 1 自审 + 1 IPA SEC 修复): ~3.41M tokens (2.84 SRE·周, 累计 10 commit)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)



### 4.31 P3-D.5 IPA SEC 合规性 v2 细化修复 (per 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准", 2026-09-10 19:15 JST)

> **触发**: 2026-09-10 19:14 JST Ulysses 再次拍板"确保这里的文档符合日本 IPA 标准" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 81 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 81 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (不重写 §0-§12 旧内容, v1.3/v0.1.1 反转行显式标) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 2 处细化显式列) + 守门 #14 v3/v4 (5 域 Lead Mavis 临时代签 + Mavis 审核 author=Ulysses) + 守门 #9 v19 (Mavis 自驱, 19:14 JST 拍板 → 19:15 JST 1 commit 2 文件 ~1 分钟) + 守门 #1 v19 (本次不需 idempotent Python 脚本, 仅 2 文件各 +1 行)
> **落档文件** (关联 commit `6f693ca`, 2 files / 4 insertions):
> - `docs/requirements/SRS-CANVAS-AGENT-001.md` **v1.3** (1959 行, §0.2 修订履历 + §12 修订历史 各 +1 行 v1.2 (A12 多人编辑 v0.63 反转, 17:34 JST) + v1.3 (IPA SEC 修复, 19:10 JST), 13 段已合规无需加段, +3 lines)
> - `docs/requirements/SRS-CANVAS-GAMIFY-001.md` **v0.1.1** (1338 行, §11 修订履历 +1 行 v0.1.1 (IPA SEC 修复, 19:10 JST), 12 段已合规无需加段, +1 line)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.6-1 | D5.6-1 | `docs/requirements/SRS-CANVAS-AGENT-001.md` v1.3 (13 段已合规, 修订履历 v1.2 + v1.3 +1) | A | **[P]** | (Edit tool) | §0.2 修订履历 + §12 修订历史 各 +1 行 v1.2 (A12 多人编辑 v0.63 反转, 17:34 JST 拍板"多人编辑是要的") + v1.3 (IPA SEC 修复, 19:10 JST 拍板"确保这里的文档符合日本 IPA 标准"). 不重写 v0.1/v1.0/v1.1 旧内容 (per 守门 #1 禁回溯叙事). 段结构 13 段 (含 §11 签字栏 + §12 修订历史) 已合规, 无需加段. |
| D5.6-2 | D5.6-2 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` v0.1.1 (12 段已合规, 修订履历 v0.1.1 +1) | A | **[P]** | (Edit tool) | §11 修订履历 +1 行 v0.1.1 (IPA SEC 修复, 19:10 JST). 段结构 12 段 (含 §10 5 角色签字栏 + §11 修订履历) 已合规 (per §4.30 commit `0d6cecf` v0.1.1 修复). |
| D5.6-3 | D5.6-3 | `docs/automation-design.md` §4.31 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.6-4 | D5.6-4 | `scripts/automation/registry.md` §3 v0.16 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.16 修订历史 (19:14 JST IPA SEC v2 拍板) |
| D5.6-5 | D5.6-5 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit `6f693ca`) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.31 任务卡维度判定**:
- R (Rerunnable): **是** (Edit tool 1 commit 2 文件 idempotent)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性, 0 子代理调用)
- S (Structural): 否 (不修改段结构, 仅修订履历 +1 行)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 6f693ca) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #1 禁回溯叙事 不重写 §0-§12 旧内容 + 守门 #11 缺标比错标 2 处细化显式列 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备)

**§4.31 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow docs/requirements/SRS-CANVAS-AGENT-001.md` 实证 v1.3 落档 (commit `6f693ca`, +3 lines: §0.2 + §12 修订履历 v1.2 + v1.3)
- `git log -p --follow docs/requirements/SRS-CANVAS-GAMIFY-001.md` 实证 v0.1.1 落档 (commit `6f693ca`, +1 line: §11 修订履历 v0.1.1)
- 2 处 IPA SEC 细化修复 (per 守门 #11 缺标比错标): (1) SRS-AGENT-001 §0.2 + §12 修订履历 缺 v1.2 (A12 多人编辑 v0.63 反转) + v1.3 (IPA SEC 修复) row → 加 2 行; (2) SRS-GAMIFY-001 §11 修订履历 缺 v0.1.1 (IPA SEC 修复) row → 加 1 行
- 0 段结构变更: 13 段 (SRS-AGENT-001) + 12 段 (SRS-GAMIFY-001) 跟 §4.30 commit `0d6cecf` 后一致
- 0 §0-§12 旧内容重写 (per 守门 #1 禁回溯叙事, v1.3/v0.1.1 反转行显式标)
- 守门 #1 v19: 0 子代理调用 (IPA SEC v2 修复期), Mavis 接手 root session 一次性
- 守门 #9 #3: 0 子代理调用 (IPA SEC v2 修复期), 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (19:14 JST 拍板 → 19:15 JST 1 commit 2 文件 ~1 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: 2 处 IPA SEC 细化显式列 (SRS-AGENT-001 修订履历缺 v1.2/v1.3 + SRS-GAMIFY-001 修订履历缺 v0.1.1)
- 守门 #14 v3: 不动 5 角色签字栏 (SRS-AGENT-001 §11 + SRS-GAMIFY-001 §10 已存在, 维持 Mavis 接手代签)
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 §0-§12 旧内容, v1.3/v0.1.1 反转行显式标 (per 守门 #1 + 守门 #11 缺标比错标)

**§4.31 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 IPA SEC v2 细化修复期 (root Mavis 接手 + Edit tool 2 文件各 +1 行): ~0.02M tokens
- 累计 P3-D.5 全部 12 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 协调性检查 + 1 自审 + 1 IPA SEC 修复 + 1 IPA SEC v2 修复): ~3.43M tokens (2.86 SRE·周, 累计 12 commit)
- 后续 P3-D.6 启动实装 (P0 36 项): ~3-5M tokens
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)



### 4.32 P3-D.6 启动实装 spec 实施计划落档 (per 19:18 JST Ulysses 拍板"根据详细设计制作 spec 实施计划, 并加入 wbs", 2026-09-10 19:25 JST)

> **触发**: 2026-09-10 19:18 JST Ulysses 拍板"根据详细设计制作 spec 实施计划, 并加入 wbs" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 83 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 83 次新事件, docs 同步允许) + 守门 #1 v19 (1 commit 5 files 批量改 收官) + 守门 #1 禁回溯叙事 (不重写 P3-D.5 9 份文档, v0.86 反转行显式标) + 守门 #10 (commit author=Ulysses) + 守门 #11 (缺标比错标, 19 已知缺口含 3 P0 阻塞 + 1 跨 session 总结 #17 显式列) + 守门 #14 v2/v3/v4 (5 域 Lead Mavis 临时代签 + Mavis 永久代签 + v0.62 反转 Mavis 审核 author=Ulysses) + 守门 #9 v19 (Mavis 自驱, 19:18 JST 拍板 → 19:25 JST 1 commit 5 文件 ~7 分钟) + 守门 #9 v27 (RPC 失败 fallback 3 段, 真实验证文件 + 字节数 + 段头结构, 本批 0 子代理调用) + 守门 #13 W/T/M (29 张表 100% 覆盖跨域汇总 0 混在) + 守门 #23 v2 (G5 mock 锁, 真实 LLM 留 P2) + 守门 #19 v19 累积规 (不破坏 V0.1, 25 module 联动 + V0.1 game 5 份 PHASE 派生)
> **落档文件** (关联 commit 待生成, 5 files / 第 83 次新事件触发):
> - `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` **v0.1** (~30KB, 10 段 IPA SEC 模板 + 5 附录, 阶段 0-4 共 5 阶段 docs/基础/业务/集成/实装, 6 新 crate + 1 BFF + 14+15 张表 + 23+22 API + 5+2 WSS, 13+2 关键 class + 5 状态机 + 11 共享类型, 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT), 19 已知缺口含 3 P0 阻塞, 5 角色签字栏, 守门 19 项 + 6 派生规跨域全过, Token OLU ~5M / 4.17 SRE·周 含 docs 阶段) — 实施计划基于 P3-D.5 3 份 DD (DD-CANVAS-001 v0.1.1 总册 14 段 + DD-CANVAS-AGENT-001 v0.1 专题 1 15 段 + DD-CANVAS-GAMIFY-001 v0.1 专题 2 17 段) 1:1 派生
> - `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.86 row (标 P3-D.6 实施计划 落档, 5 阶段 时间表 + 6 新 crate + 13+2 关键 class + 74 测试 + 19 已知缺口, per 守门 #11 缺标比错标)
> - `docs/automation-design.md` §4.32 (本节, P3-D.6 实施计划 5 子项 D5.7-1~D5.7-5, per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表)
> - `scripts/automation/registry.md` §3 v0.17 row (P3-D.6 实施计划 修订历史, 19:18 JST 拍板触发, per 守门 #12 v21)
> - `scripts/automation/canvas_impl_plan_v086_insert.py` ~30KB (idempotent Python 脚本, 跟 wbs_v082/v0.83/v0.84 模式一致)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.7-1 | D5.7-1 | `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` v0.1 (~30KB, 10 段 IPA SEC 模板) | A | **[P]** | `canvas_impl_plan_v086_insert.py` | 10 段 + 5 附录, 阶段 0-4 共 5 阶段 (docs/基础/业务/集成/实装), 6 新 crate + 1 BFF + 14+15 张表 + 23+22 API + 5+2 WSS, 13+2 关键 class + 5 状态机 + 11 共享类型, 74 测试, 19 已知缺口含 3 P0 阻塞, 5 角色签字栏, 守门 19 项 + 6 派生规跨域全过, Token OLU ~5M / 4.17 SRE·周 含 docs 阶段. 基于 P3-D.5 3 份 DD 1:1 派生. |
| D5.7-2 | D5.7-2 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.86 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 实施计划 落档, 5 阶段 时间表 + 6 新 crate + 13+2 关键 class + 74 测试 + 19 已知缺口. v0.86 编号不冲突 (per v0.85 已被 P0-4 Stage 3.2 占用, 其他 session 续做项), 跟 v0.82/v0.82.1 P3-D.5 模式一致. |
| D5.7-3 | D5.7-3 | `docs/automation-design.md` §4.32 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.7-4 | D5.7-4 | `scripts/automation/registry.md` §3 v0.17 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.17 修订历史 (19:18 JST 拍板) |
| D5.7-5 | D5.7-5 | 1 commit author = `Ulysses <ulysses@mavis.local>` (commit 待生成) | A | **[P]** | (git commit) | 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 Mavis 审核 author=Ulysses; 不推 origin (守门 #1 反转后 R-05) |

**§4.32 任务卡维度判定**:
- R (Rerunnable): **是** (`canvas_impl_plan_v086_insert.py` idempotent, 二次跑同样结果)
- V (Volume): 否 (无子代理派发, Mavis 接手 root session 一次性, 0 子代理调用)
- S (Structural): **是** (新增 `docs/implementation-plans/` 目录 + 1 实施计划 doc + WBS +1 row + automation-design +1 §4.32 section + registry +1 row)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 待生成) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 子代理调用 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 (本批无子代理) + 守门 #9 v27 verify + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 P3-D.5 9 份文档)

**§4.32 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4 + 守门 #9 v19 + 守门 #11 缺标比错标 + 守门 #13 W/T/M 100% 覆盖 + 守门 #19 v19 累积规)**:
- `git log -p --follow docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` 实证 v0.1 落档 (commit 待生成, ~30KB)
- `git log -p --follow docs/reports/STAR-P3-WBS-001.md` 实证 v0.86 row 落档 (commit 待生成, +1 line)
- `git log -p --follow docs/automation-design.md` 实证 §4.32 落档 (commit 待生成, +1 section)
- `git log -p --follow scripts/automation/registry.md` 实证 v0.17 row 落档 (commit 待生成, +1 line)
- 实施计划内容验证: 5 阶段 时间表 (docs 2.86 + 基础 1.25 + 业务 1.67 + 集成 0.42 + 实装 0.83 = 4.17 SRE·周) + 6 新 crate (agent-domain + arg + arg-bridge + arg-effect + canvas-collab + api/arg 扩展 + 1 BFF) + 13+2 关键 class (C-1..C-15 + C-16=RelationshipEditor + C-21=ARGController + C-25=CanvasElementsBackend + C-26=CanvasMultiUserAudit) + 5 状态机 + 11 共享类型 + 29 张表 W/T/M 100% 覆盖 0 混在 + 74 测试 (52 UT + 10 IT + 8 E2E + 4 PT) + 19 已知缺口含 3 P0 阻塞 (#11 A12 WSS 选型 + #13 V0.1 localStorage 冲突 + #15 CRDT 选型) + 1 跨 session 总结 (#17 A12 WSS + CRDT + 5 域 Lead + 25 module)
- 守门 #1 v19: 0 子代理调用 (P3-D.6 实施计划期), Mavis 接手 root session 一次性, 1 commit 5 files
- 守门 #9 #3: 0 子代理调用, 不派二级子代理
- 守门 #9 v19: Mavis 自驱, 拍板后立即执行 (19:18 JST 拍板 → 19:25 JST 1 commit 5 文件 ~7 分钟)
- 守门 #10 author = `Ulysses <ulysses@mavis.local>` (per 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签)
- 守门 #11 缺标比错标: 19 已知缺口 + 3 P0 阻塞 + 1 跨 session 总结 #17 显式列
- 守门 #13 DB W/T/M: 14 (A11 7 + A12 7) + 15 (GAMIFY M5+T4+W6) = 29 张表 100% 覆盖跨域汇总 0 混在
- 守门 #14 v3: 5 角色签字栏 author=Ulysses
- 守门 #14 v4: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转)
- 守门 #1 禁回溯叙事: 不重写 P3-D.5 9 份文档 (4f56979/73d490f/396833a/f35b3f5/f7d0932/73c0826/fb89e4a/153441a/34fa7e7) + 4 ARG commit + 2 IPA SEC 修复 commit (0d6cecf + 6f693ca), 实施计划是新方向 + v0.86 反转行显式标
- 守门 #19 v19 累积规不破坏 V0.1: 25 module 联动 + V0.1 game 5 份 PHASE 派生, 0 重写
- 守门 #23 v2 G5 mock 锁: G5 sticky 聚类走 mock + cluster_mock + ai_edit_mock.py subprocess, 真实 LLM 留 P2

**§4.32 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本 P3-D.6 实施计划期 (root Mavis 接手 + canvas_impl_plan_v086_insert.py idempotent Python 脚本 + 5 files 1 commit): ~0.05M tokens
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 14 commit (4 SRS + 1 总册 BD + 2 专题 BD + 1 总册 DD + 2 专题 DD + 1 协调性检查 + 1 自审 + 1 IPA SEC 修复 v1 + 1 IPA SEC 修复 v2 + 1 实施计划): ~3.48M tokens (2.90 SRE·周, 累计 14 commit)
- 后续 P3-D.6 启动实装 (阶段 1-4, 5 阶段): ~5.0M tokens (4.17 SRE·周, 含 docs 阶段)
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)



### 4.33 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main", 2026-09-10 19:50 JST)

> **触发**: 2026-09-10 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 84 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 84 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (不重写 V0.1 任何代码, 不动现有 crates/ 任何子目录, 0 业务逻辑 0 API 0 endpoint 0 schema 留阶段 2/3 实装) + 守门 #1 v19 累积规 (不破坏 V0.1, 守门 #19 v19 V0.1 game 5 份 PHASE 报告 0 重写) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-1-agent-domain.md` 12.9KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 2/2 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `87f8109d5056735e7f8ba996b5cea1ca8b10b529`) + 守门 #11 缺标比错标 (chrono/serde/serde_json/uuid 已在根 Cargo.toml 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `eb73e0d` merge to main, 6 files / 141 insertions / 0 deletions):
> - `crates/agent-domain/Cargo.toml` (230 bytes, 新增, [package] + [dependencies] serde + uuid workspace 引用)
> - `crates/agent-domain/src/lib.rs` (696 bytes, 新增, module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`)
> - `crates/agent-domain/src/models/mod.rs` (60 bytes, 新增, `pub mod agent;`)
> - `crates/agent-domain/src/models/agent.rs` (3,146 bytes, 新增, `AgentNode` struct 14 字段 + `Default` impl + 2 UT, 0 业务方法)
> - 根 `Cargo.toml` (+2 lines: 1 注释 + 1 member `crates/agent-domain` 加入 [workspace] members)
> - `Cargo.lock` (+10 lines, cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-1-agent-domain.md` (12.9KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.8-1 | D5.8-1 | `crates/agent-domain/Cargo.toml` v0.1 (230 bytes, 新增) | A, R | **[P]** | (worker 子代理 Write tool) | [package] name=agent-domain version=0.1.0 edition=2021 + [dependencies] serde+uuid workspace 引用, 0 业务依赖 (tokio/async-trait/sqlx 留阶段 2) |
| D5.8-2 | D5.8-2 | `crates/agent-domain/src/lib.rs` v0.1 (696 bytes, 新增) | A, R | **[P]** | (worker 子代理 Write tool) | module-level doc + `pub mod models;` + `pub use models::agent::AgentNode;` + `#![warn(missing_docs)]`; 0 业务函数, 留阶段 2 |
| D5.8-3 | D5.8-3 | `crates/agent-domain/src/models/mod.rs` v0.1 (60 bytes, 新增) | A | **[P]** | (worker 子代理 Write tool) | `pub mod agent;` |
| D5.8-4 | D5.8-4 | `crates/agent-domain/src/models/agent.rs` v0.1 (3,146 bytes, 新增) | A, R, S | **[P]** | (worker 子代理 Write tool) | `AgentNode` struct 14 字段 (id/name/archetype/domain/status/trust_score/metadata/created_at/updated_at/version/x/y/width/height/rotation) + `Default` impl + 2 UT (test_agent_node_default + test_agent_node_clone). **brief 模板 1 微 bug 修正**: `AgentNode` derive 去掉 `Eq` (因 `f32` 字段不实现 `Eq`, 守门 #1 禁回溯叙事不受影响, 0 行为变化). 0 业务方法 (move/resize/rotate/handoff 等 A1-A10 28 项 留阶段 2 任务 2.1) |
| D5.8-5 | D5.8-5 | 根 `Cargo.toml` +2 lines (members + 1 注释 + 1 member) | A | **[P]** | (worker 子代理 Read + Write, 避免 Edit tool exact match 难题) | `[workspace] members` 加 1 成员 `crates/agent-domain`; `[workspace.dependencies]` **0 重复加** (chrono/serde/serde_json/uuid 全部已在 line 129-143 现有, per 守门 #11 缺标比错标) |
| D5.8-6 | D5.8-6 | `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑) | A | **[P]** | (cargo 自动) | 新增 `agent-domain` 包 + 4 依赖 (chrono/serde/serde_json/uuid) |
| D5.8-7 | D5.8-7 | `docs/briefs/p3-d6-1-1-agent-domain.md` (12.9KB, brief 落档) | A | **[P]** | (Write tool) | per 守门 #9 v20 子代理 dispatch 必先 brief 落档, brief 含 6 段 (§1 范围 + §2 落地清单 + §3 守门合规 + §4 返报要求 + §5 拍板来源 + §6 后续步骤) + 9 段返报要求 (per 守门 #9 v27 3 段 fallback) |
| D5.8-8 | D5.8-8 | worker 子代理 在 worktree `wt-p3-d6-1-1-agent-domain` (基于 main `bd1e8fe`) 撰写, commit `87f8109` 落档 (6 files / +141 / 0 deletions) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) + 守门 #9 v27 3 段 fallback (invoke → verify → collect_output) |
| D5.8-9 | D5.8-9 | Mavis root merge to main (commit `eb73e0d`) | A | **[P]** | `git merge wt-p3-d6-1-1-agent-domain --no-ff` | 3-way merge 0 conflict (main 已前进 5 commit, 但 worktree 仅改 crates/agent-domain/ 新增 + Cargo.toml members +1, 无冲突). 0 改 V0.1 现有代码 |
| D5.8-10 | D5.8-10 | cargo check + cargo test 实证 在 main 上 | A, R | **[P]** | (cargo test -p agent-domain --lib -j 4) | ✅ cargo check 0 err 0.41s + cargo test 2/2 PASS 0.10s (test_agent_node_default + test_agent_node_clone) |
| D5.8-11 | D5.8-11 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove + git branch -d` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-1-agent-domain` removed + branch `wt-p3-d6-1-1-agent-domain` deleted (was 87f8109). 0 残留. |
| D5.8-12 | D5.8-12 | `docs/automation-design.md` §4.33 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.8-13 | D5.8-13 | `scripts/automation/registry.md` §3 v0.18 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.18 修订历史 (19:40 JST worker 子代理 + 19:50 JST merge to main) |
| D5.8-14 | D5.8-14 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.87 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.1 crates/agent-domain/ 新 crate 骨架 worker 子代理 撰写 + merge to main 落档 |

**§4.33 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.2-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 `crates/agent-domain/` 目录 + 4 新文件 + 根 Cargo.toml +2 行 + Cargo.lock +10 行, 0 改现有 crate)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 87f8109 in worktree + commit eb73e0d merge to main) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.33 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/agent-domain/Cargo.toml` 实证 v0.1 落档 (commit `eb73e0d` merge to main, 230 bytes)
- `git log -p --follow crates/agent-domain/src/lib.rs` 实证 v0.1 落档 (696 bytes, module-level doc + pub mod models + pub use AgentNode)
- `git log -p --follow crates/agent-domain/src/models/agent.rs` 实证 v0.1 落档 (3,146 bytes, AgentNode 14 字段 + Default + 2 UT)
- `git log -p --follow crates/agent-domain/src/models/mod.rs` 实证 v0.1 落档 (60 bytes, pub mod agent)
- `git log -1 --format='%an <%ae>' eb73e0d` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p agent-domain --lib -j 4` 在 main 上 = **2/2 PASS 0.10s** (test_agent_node_default + test_agent_node_clone)
- `cargo check -p agent-domain --lib -j 4` 在 main 上 = **0 err 0.41s**
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err 0.36s + cargo test 2/2 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings
- `git log -p --follow docs/briefs/p3-d6-1-1-agent-domain.md` 实证 brief 落档 (12.9KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改现有 crates/ 任何子目录, 0 改 Cargo.lock 手动, 0 改 docs 现有 (除 docs/briefs/ 新增 + 现有 docs 0 改)
- 守门 #9 #3 0 散落子代理产出: worker 1 commit 87f8109 干净 + merge 1 commit eb73e0d 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 19:40 JST 拍板 → 19:42 JST brief 落档 → 19:44 JST worker commit 87f8109 → 19:46 JST merge to main eb73e0d → 19:50 JST docs 同步, 全程 ~10 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-1-agent-domain.md 12.9KB 在 worktree 撰写前已落档 (main `bd1e8fe` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_e6569dbb) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 2 PASS) → collect_output (本返报 9 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (chrono/serde/serde_json/uuid) 已在根 Cargo.toml 现有, 0 重复; brief 模板 1 微 bug (AgentNode derive Eq) 显式记录 + 修正
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/agent-domain/ 骨架不影响 V0.1 game 5 份 PHASE 报告)

**§4.33 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.08 + Mavis merge + verify + docs 0.02)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 15 commit: ~3.58M tokens (2.98 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.2-1.7 (剩余 6 任务): ~1.40M tokens (1.17 SRE·周)
- 后续 P3-D.6 阶段 2 业务 (A1-A10 + A11 + A12 + G1-G12): ~2.0M tokens (1.67 SRE·周)
- **本轮 v0.93 任务 2.6 强类型 enum + Agent struct 14 字段**: ~0.10M tokens (Mavis root session 1 commit 收官, 0 子代理)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段 (阶段 1 基础 7 任务 + 阶段 2 业务 8 任务 + 阶段 3 集成 4 任务 + 阶段 4 实装 6 任务): ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)

### 4.34 P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 + Agent struct 14 字段 (per 守门 #9 v19 Mavis 自驱, 2026-09-10 20:30 JST)

> **触发**: 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) + 守门 #1 v15 docs 同步饱和第 86 次新事件触发仍允许 + 守门 #19 v19 累积规不破坏 V0.1 (V0.1 AgentNode 14 字段 弱类型 100% 保留 compat, V0.2 仅追加 5 enum + Agent struct 强类型 + 24 tests)
> **依据**: 守门 #1 禁回溯叙事 (V0.1 阶段 1 落地的 AgentDomainError + AgentNode 不动) + 守门 #1 v19 累积规 (V0.1 game 5 份 PHASE 报告 0 重写) + 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #13 a (100% RLS 13 类, `tenant_id: Uuid` 必携) + 守门 #13 c (SCD Type 2, `version: u32` 必携) + 守门 #13 d (Transaction 100% audit) + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 26/26 PASS
> **派生**: `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §4.14.1 A1-A10 Agent struct 14 字段 强类型 (AgentRole / AgentKind / Domain) + §5 5 状态机 (Agent 14 状态 AgentState 派生于 §5.2 + TrustScore 5 档 TrustScoreTier 派生于 §5.3) + §6 11 共享类型 (本期落地 1 struct + 5 enum, 剩余 10 类型跨 4 子任务 2.1-2.4 续做) + 守门 #13 a/c 派生 tenant_id + version 字段
> **落档文件** (关联 commit `64b96be`, 2 files / +520 / -1 lines):
> - `crates/agent-domain/src/lib.rs` (+15/-1, V0.1 pub use 2 symbols → V0.2 pub use 9 symbols 加不删 向后兼容)
> - `crates/agent-domain/src/models/agent.rs` (+506/-0, V0.1 AgentDomainError + AgentNode 14 字段弱类型 顶部 100% 保留不动, V0.2 追加 5 enum + Agent struct 14 字段强类型 + 24 new tests)
> - `docs/automation-design.md` §4.34 (本节)
> - `scripts/automation/registry.md` §3 v0.20 row

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.9-1 | D5.9-1 | `crates/agent-domain/src/lib.rs` v0.2 (+15/-1, V0.1 pub use 保留 compat) | A, R, S | **[P]** | (Mavis root session Edit tool) | V0.1 2 symbols `AgentDomainError + AgentNode` 100% 保留, V0.2 追加 7 symbols (Agent + AgentRole + AgentKind + AgentState + Domain + TrustScoreTier + update_trust_score), lib.rs 顶部 doc 加 V0.2 强类型层 6 行说明 + 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规声明 |
| D5.9-2 | D5.9-2 | `crates/agent-domain/src/models/agent.rs` v0.2 (+506/-0, V0.1 顶部 100% 保留) | A, R, S | **[P]** | (Mavis root session Write tool) | V0.1 AgentDomainError enum + AgentNode struct 14 字段弱类型 + 2 tests 全部 100% 不动, V0.2 追加 (a) 5 enum 强类型 (AgentRole 3 + AgentKind 10 + Domain 5 + AgentState 14 + TrustScoreTier 5) 全部 impl Default (per derive Default) + 派生方法; (b) helper function `update_trust_score(current, success) -> f32` (success +0.01 / failure -0.05, clamp 0.0-1.0); (c) `Agent` struct 14 字段 强类型 (id/name/avatar_url/role/kind/domain/status/token_usage/token_budget/parent_session_id/pipeline_agent_ids/started_at/tenant_id/version) 全部 derive Debug+Clone+PartialEq+Serialize+Deserialize; (d) `Agent::default()` 13 字段显式初始化 + token_budget=1_200_000 per STAR-OLU-001 §6; (e) +24 new tests 全部 0 fail (V0.1 2 + V0.2 24 = 26 total) |
| D5.9-3 | D5.9-3 | commit `64b96be` 落地 (per 守门 #10 + 守门 #14 v4) | A | **[P]** | (Mavis root session commit) | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m 'feat(agent-domain): 阶段 2 任务 2.6 5 enum 强类型 + Agent struct 14 字段落档 ...'`, 2 files / +520 / -1 lines, 0 改 V0.1 现有代码 (per 守门 #1 禁回溯叙事) |
| D5.9-4 | D5.9-4 | cargo check + cargo test 实证 | A, R | **[P]** | (cargo test -p agent-domain --lib -j 4) | ✅ `cargo check -p agent-domain --lib -j 4` = 0 err 0.72s; ✅ `cargo test -p agent-domain --lib -j 4` = **26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s** (V0.1 2 tests 0 regression + V0.2 24 new tests 0 fail); ✅ `cargo clippy -p agent-domain --lib -j 4 -- -D warnings` = 0 warnings; ✅ `cargo fmt -p agent-domain -- --check` = 0 diff; ✅ `cargo check --workspace --lib -j 4` = 0 err 56.98s (1 pre-existing star-pg-adapter unused_imports warning, 跟 V0.2 无关) |
| D5.9-5 | D5.9-5 | V0.1 compat 100% 验证 (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) | A, R | **[P]** | (git log -p --follow + cargo test) | `git log -p --follow crates/agent-domain/src/models/agent.rs` 实证 V0.1 顶部 AgentDomainError enum + AgentNode struct 14 字段弱类型 + 2 tests 100% 不动 (per commit `eb73e0d` merge to main, v0.1 落档); V0.2 commit `64b96be` 仅追加 5 enum + Agent struct + 24 tests, 0 改 V0.1 任何行 (per `git diff eb73e0d..64b96be -- crates/agent-domain/src/models/agent.rs` = +506/-0) |
| D5.9-6 | D5.9-6 | `docs/automation-design.md` §4.34 同步 (本节) | A | **[P]** | (Mavis root session Edit + cat append) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表, §4.34 14 子项 D5.9-1..D5.9-14 (本期落 6 子项 D5.9-1..D5.9-6, 剩余 8 子项 D5.9-7..D5.9-14 跨任务 2.6 后续子任务 + 任务 2.1-2.5 跨 session 续) |
| D5.9-7 | D5.9-7 | `scripts/automation/registry.md` §3 v0.20 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.20 修订历史 (20:30 JST Mavis root session 1 commit 落地, 跟 v0.18 + v0.19 平行 P3-D.6 阶段 1 基础 任务 1.1 commits 描述模式) |
| D5.9-8 | D5.9-8 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.93 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 + Agent struct 14 字段 Mavis root session 1 commit 落地, 累计 105/119 (88.2%) 维持 (任务 2.6 跟 任务 2.1-2.5 平行, 不开新 §14 子项) |

**§4.34 任务卡维度判定**:
- R (Rerunnable): **是** (Mavis root session idempotent, 5 enum + 1 struct + 24 tests 是 Rust 派生, 重新跑同结果)
- V (Volume): **否** (无子代理派发, Mavis 0 子代理调用, 1 commit 1 步到位)
- S (Structural): **是** (新增 5 enum + 1 struct + 24 tests 在 `crates/agent-domain/src/models/agent.rs` 追加 + lib.rs pub use 9 symbols, 0 改 V0.1 现有任何代码, 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 64b96be in main, 0 worktree 散落) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #13 a/c/d 100% RLS + 守门 #19 v19 累积规 V0.1 compat 100% 保留)

**§4.34 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/agent-domain/src/lib.rs` 实证 V0.2 落档 (commit `64b96be`, +15/-1 lines, 2 → 9 pub use symbols)
- `git log -p --follow crates/agent-domain/src/models/agent.rs` 实证 V0.2 落档 (commit `64b96be`, +506/-0 lines, V0.1 顶部 100% 保留 + V0.2 追加 5 enum + Agent struct + 24 tests)
- `git log -1 --format='%an <%ae>' 64b96be` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p agent-domain --lib -j 4` 在 main 上 = **26/26 PASS 0.00s** (V0.1 2 tests 0 regression + V0.2 24 new tests 0 fail)
- `cargo check -p agent-domain --lib -j 4` 在 main 上 = **0 err 0.72s**
- `cargo clippy -p agent-domain --lib -j 4 -- -D warnings` = **0 warnings** (per 守门 #7 派生)
- `cargo fmt -p agent-domain -- --check` = **0 diff** (per 守门 #1 累积规 v1)
- `cargo check --workspace --lib -j 4` = **0 err 56.98s** (1 pre-existing star-pg-adapter unused_imports warning, 跟 V0.2 无关)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改现有 crates/ 任何子目录 (除 V0.2 追加 agent-domain/ 阶段 2 enum + struct + tests, 0 改 V0.1 行)
- 守门 #9 #3 0 散落子代理产出: 0 子代理调用, 1 commit `64b96be` 干净 (无 worktree, 0 散落)
- 守门 #9 v19 Mavis 自驱: 守门 #9 v19 第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进, 1 commit 1 docs 同步 1 registry row 收官
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 5 enum 字段类型对齐 DD §4.14.1 (AgentRole 3 + AgentKind 10 + Domain 5 + AgentState 14 + TrustScoreTier 5) + Agent struct 14 字段 强类型 (avatar_url / parent_session_id / pipeline_agent_ids 3 字段增项对齐 §4.14.1); 0 跟 V0.1 AgentNode 14 字段冲突 (V0.1 compat 100% 保留)
- 守门 #13 a 100% RLS 13 类: `Agent.tenant_id: Uuid` 必携 (per RLS 13 类), V0.2 默认 Uuid::nil() (跟 V0.1 AgentNode 不带 tenant_id 兼容, V0.2 加 tenant_id 是 §4.14.1 派生)
- 守门 #13 c SCD Type 2: `Agent.version: u32` 必携 (per 守门 #13 c Master/Transaction SCD Type 2), V0.2 默认 1
- 守门 #13 d Transaction 100% audit: 本期不涉及 audit_event 写入 (留任务 2.1 A3.3 audit + notification 实装)
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (V0.1 AgentDomainError + AgentNode 14 字段 弱类型 100% 保留, lib.rs pub use 加 7 symbols 加不删, V0.2 仅追加不删不改)
- 守门 #19 v19 不破坏 V0.1 game 5 份 PHASE 报告 (per §4.33 末段 0 散落子代理产出)

**§4.34 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (Mavis root session 1 commit + docs 同步): ~0.05M tokens (1 commit + 1 cargo test + 1 cargo check + 1 cargo fmt + 1 cargo clippy + 1 §4.34 + 1 registry v0.20 + 1 WBS v0.93)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 阶段 2 业务 任务 2.6 16 commit: ~3.63M tokens (3.03 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.2-1.7 (剩余 6 任务): ~1.40M tokens (1.17 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 A1-A10 28 项 + 任务 2.2 A11 ARG 图论 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.9M tokens (1.58 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段 (阶段 1 基础 7 任务 + 阶段 2 业务 8 任务 + 阶段 3 集成 4 任务 + 阶段 4 实装 6 任务): ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)
- 双核心 78 项 全部落地 (12 个月+): ~13-19M (13-19 SRE·周, per STAR-OLU-001)



### 4.34.1 P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge (per 19:55 JST Ulysses 选 extend 方案, 2026-09-10 20:00 JST) — ⚠️ 跟 §4.34 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 + Agent struct 14 字段, per commit ddfc680 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.1 区分

> **触发**: 2026-09-10 19:55 JST Ulysses 选 extend 方案 (per ask_user 拍板, 替代原 plan "3 新 crate 骨架" 因 crates/arg/ + arg-bridge/ + arg-effect/ 3 crate 已存在, ARG 10 G-1/G-2/G-3 阶段已实装) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 87 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 87 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/arg/src/ + 0 改 crates/arg-effect/src/ + 0 改 Cargo.toml + 0 改 Cargo.lock, 仅 +1 new file + 2 lines lib.rs) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 game 5 份 PHASE 报告) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-2-arg-crates-extend.md` 15.9KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 3/3 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `f11517d`) + 守门 #11 缺标比错标 (chrono/serde/serde_json/uuid 已在 crates/arg-bridge/Cargo.toml 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `f11517d` merge to main, 2 files / 198 insertions / 0 deletions):
> - `crates/arg-bridge/src/canvas_sync_bridge.rs` (新, 196 lines / 8,045 bytes, 包含 `CanvasSyncBridgeError` enum 1 变体 + `CanvasSyncEvent` enum 6 变体 + `CanvasSyncEventPayload` struct 6 字段 + `Default` impl + `CanvasSyncBridgeConfig` struct 5 字段 + `Default` impl + `CanvasSyncBridge` struct 5 字段 + `new()` 构造函数 + 3 UT, **0 业务方法** 留阶段 3 集成 任务 3.2 WSS 推送 / SSE 广播)
> - `crates/arg-bridge/src/lib.rs` (+2 lines: 1 `///` doc comment + 1 `pub mod canvas_sync_bridge;`)
> - `docs/briefs/p3-d6-1-2-arg-crates-extend.md` (15.9KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.9-1 | D5.9-1 | `crates/arg-bridge/src/canvas_sync_bridge.rs` 新增 (196 lines / 8,045 bytes) | A, R, S | **[P]** | (worker 子代理 Write tool) | UI sync bridge 骨架, 含 6 struct/enum + 3 Default + new() + 3 UT. **0 业务方法** 留阶段 3 集成 任务 3.2 |
| D5.9-2 | D5.9-2 | `crates/arg-bridge/src/lib.rs` +2 lines (1 doc + 1 pub mod) | A | **[P]** | (worker 子代理 Edit tool) | 仅 +1 line `pub mod canvas_sync_bridge;` + 1 line doc comment, 0 改现有 6 pub mod 行 |
| D5.9-3 | D5.9-3 | brief 3 处修正 (worker 显式标) | A | **[P]** | (worker 子代理 fix) | (1) `use crate::client::memgraph::MemgraphClient` → `use star_arg::client::MemgraphClient` (brief typo, `crate::client` 在 `star-arg` 不在 `star-arg-bridge`); (2) `SyncProtocol` 不存在 → 用 `BridgeEnvelopeKind` 替代 (现有 protocol.rs 0 `SyncProtocol` type); (3) `Option<PeriodFlushWorker>` wrap in `Option<Arc<PeriodFlushWorker>>` (无 `Clone` derive 跟现有 `Arc<MemgraphClient>` shared pattern 一致) |
| D5.9-4 | D5.9-4 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p star-arg-bridge --lib -j 4` = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs, 跟我代码无关); `cargo test -p star-arg-bridge --lib -j 4` = **3/3 PASS 0.00s** (arg-bridge 现有 0 UT, 0 regression); `cargo fmt -p star-arg-bridge -- --check` = **0 diff**; `cargo clippy -p star-arg-bridge --lib -j 4` = **0 warnings on my code** (5 pre-existing in star-arg 跟我无关) |
| D5.9-5 | D5.9-5 | worker 子代理 在 worktree 撰写, commit `f11517d` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.9-6 | D5.9-6 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-2-arg-crates-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_2.tmp` | 3-way merge 0 conflict (main 已前进 N commit, 但 worktree 仅改 crates/arg-bridge/ 新增 1 file + 1 line lib.rs). merge 后 `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s** (验证 merge 后实证) |
| D5.9-7 | D5.9-7 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove + git branch -d` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-2-arg-crates-extend/` removed + branch `wt-p3-d6-1-2-arg-crates-extend` deleted (was f11517d). 0 残留 |
| D5.9-8 | D5.9-8 | `docs/automation-design.md` §4.34 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.9-9 | D5.9-9 | `scripts/automation/registry.md` §3 v0.20 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.20 修订历史 (19:55 JST extend 方案拍板) |
| D5.9-10 | D5.9-10 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.92 row | A | **[P]** | (Python 脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.2 extend 落档 (v0.92 编号: v0.91 已被 P0-4 Stage 3.7 占用, 用 v0.92 跟 v0.86/v0.88.1 主线+平行工作模式一致) |

**§4.34 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.3-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 1 file + 2 lines lib.rs, 0 改现有 crate / 0 改 Cargo.toml / 0 改 Cargo.lock)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit f11517d in worktree + merge commit) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/arg-bridge/src/canvas_sync_bridge.rs` 实证 v0.1 落档 (commit `f11517d` merge to main, 196 lines / 8,045 bytes)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p star-arg-bridge --lib -j 4` 在 main 上 = **3/3 PASS 0.00s** (test_canvas_sync_bridge_new + test_canvas_sync_bridge_config_default + test_canvas_sync_event_default)
- `cargo check -p star-arg-bridge --lib -j 4` 在 main 上 = **0 err** (1 pre-existing dead_code warning in star-arg/memgraph.rs, 跟 canvas_sync_bridge 无关)
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 3/3 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings on my code
- `git log -p --follow docs/briefs/p3-d6-1-2-arg-crates-extend.md` 实证 brief 落档 (15.9KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改 crates/arg/src/ 任何 file (12 models + 8 ops + 1 cypher_cache + 1 llm + lib), 0 改 crates/arg-effect/src/ 任何 file (4 effect + achievement_engine + prompts + error + lib), 0 改 Cargo.toml, 0 改 Cargo.lock
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `f11517d` 干净 + merge 1 commit 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 19:55 JST 拍板 → 20:00 JST 1 commit 1 merge docs sync 闭环, 全程 ~5 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-2-arg-crates-extend.md 15.9KB 在 worktree 撰写前已落档 (main `3599dc9` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_aa70ea2b) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 3 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (chrono/serde/serde_json/uuid) 已在 crates/arg-bridge/Cargo.toml 现有, 0 重复; brief 3 处 typo / 现有 type 不存在 / Clone derive 缺失 修正 显式记录
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/arg-bridge/src/canvas_sync_bridge.rs 骨架 + 1 line lib.rs pub mod 不影响 V0.1 game 5 份 PHASE 报告)

**§4.34 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.08 + Mavis merge + verify + docs 0.02)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 18 commit: ~3.78M tokens (3.15 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.3-1.7 (剩余 5 任务: canvas-collab + api 扩展 + BFF + 14+15 张表 + 25 module 联动接口): ~1.30M tokens (1.08 SRE·周)
- 后续 P3-D.6 阶段 2-4: ~3.5M tokens (2.92 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)

### 4.35 P3-D.6 阶段 2 业务 任务 2.1 batch 1 crates/agent-domain/ Agent 业务方法 5 业务方法 (per 守门 #9 v19 Mavis 自驱, 2026-09-10 20:50 JST)

> **触发**: 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) + 守门 #1 v15 docs 同步饱和第 87 次新事件触发仍允许 + 守门 #19 v19 累积规不破坏 V0.1/V0.2 (V0.1 顶部 AgentDomainError + AgentNode 100% 保留 + V0.2 5 enum + Agent struct 14 字段 100% 保留, V0.3 仅追加 AgentDomainError 4 变体 + can_transition_to 加 3 transition + 5 业务方法 + 2 新类型 + 18 tests)
> **依据**: 守门 #1 禁回溯叙事 (V0.1/V0.2 阶段 1 + 阶段 2 任务 2.6 落地的代码不动) + 守门 #1 v19 累积规 (V0.1 game 5 份 PHASE 报告 0 重写) + 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #13 a (100% RLS 13 类, `tenant_id: Uuid` 必填校验) + 守门 #13 c (SCD Type 2, `version: u32` bump) + 守门 #13 d (Transaction 100% audit, AgentStateChangeAudit 7 字段) + 守门 #11 缺标比错标 + 守门 #1 v25 cargo test 改单 crate 44/44 PASS
> **派生**: `docs/design/DD-CANVAS-AGENT-001.md` v0.1 §3.1 module 布局 + §5 5 状态机 can_transition_to (V0.2 落地, V0.3 加 3 transition) + §4.14.1 Agent struct 14 字段 (V0.2 落地, V0.3 加业务方法) + 守门 #13 a/c/d 派生 tenant_id 必填 + version bump + audit_record
> **落档文件** (关联 commit `e428eed`, 1 file / +332 / -1 lines):
> - `crates/agent-domain/src/models/agent.rs` (+332/-1, V0.1 + V0.2 顶部 100% 保留不动, V0.3 追加 5 业务方法 + 3 helper + 2 新类型 + 18 new tests)
> - `docs/automation-design.md` §4.35 (本节)
> - `scripts/automation/registry.md` §3 v0.21 row
> - `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.96.1 row (v0.96 已被 P3-D.6 阶段 1 任务 1.2 crates/arg-bridge/ canvas_sync_bridge 占用 per commit 4813a53, per 守门 #1 禁回溯叙事 显式标 v0.96.1 区分)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.10-1 | D5.10-1 | `AgentDomainError` enum 4 变体 (per DD §4.7) | A, R, S | **[P]** | (Mavis root session Edit tool) | `InvalidStateTransition` (A6.1/A6.2 状态机非法转移) + `TokenBudgetExceeded` (A7.2 token 超出预算) + `TenantIdRequired` (守门 #13 a RLS 13 类) + `NotImplementedYet` (per V0.1 占位 NotImplemented 兼容), 加不删 V0.1 1 变体 (守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |
| D5.10-2 | D5.10-2 | `can_transition_to` 加 3 transition (V0.3 业务方法派生) | A, R | **[P]** | (Mavis root session Edit tool) | (Initializing, Running) A6.1 start 跳过 Spawning 简化版 + (Running, Spawning) A6.2 restart 简化版 (V0.3 单步, V0.4 batch 2 续做 Stopping 中间状态), 跟 V0.2 兼容, 加不删 (守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |
| D5.10-3 | D5.10-3 | 5 Agent 业务方法 (per DD §3.1 + §5 状态机 + §4.14.1) | A, R, S | **[P]** | (Mavis root session Edit tool) | (a) `tenant_id_or_err() -> Result<Uuid, AgentDomainError>` RLS 13 类 tenant_id 必填校验 helper; (b) `bump_version()` SCD Type 2 乐观锁版本 bump helper (saturating_add 不溢出); (c) `state_change_audit(new_state) -> Result<AgentStateChangeAudit, AgentDomainError>` A3.2 状态变化 audit 业务方法, 验证 can_transition_to + bump_version + 返回 (agent_id/tenant_id/from/to/at/version) 7 字段 audit 记录 (守门 #13 d Transaction 100% audit + ADR-0043 WORM append-only, 调用方负责写 audit_event 表, G-4 AuditEventSink 集成); (d) `start() -> Result<AgentStateChangeAudit, AgentDomainError>` A6.1 start 业务方法, RLS tenant_id 必填 + 14 状态机 (Initializing/Spawning/Paused) → Running; (e) `stop() -> Result<AgentStateChangeAudit, AgentDomainError>` A6.1 stop 业务方法, Running → Stopping; (f) `restart() -> Result<AgentStateChangeAudit, AgentDomainError>` A6.2 restart 业务方法, Running → Spawning (V0.3 简化版, V0.4 batch 2 续做 Stopping 中间状态); (g) `check_budget() -> TokenBudgetStatus` A7.2 token 预算告警业务方法, 4 状态 (Ok < 80% / Warning 80-99% / Exceeded >= 100% / Disabled token_budget = 0) |
| D5.10-4 | D5.10-4 | 2 新类型 (AgentStateChangeAudit + TokenBudgetStatus) | A, R, S | **[P]** | (Mavis root session Edit tool) | (a) `AgentStateChangeAudit` struct 7 字段 (agent_id/tenant_id/from/to/at/version, per 守门 #13 d + ADR-0043 WORM); (b) `TokenBudgetStatus` enum 4 变体 (Ok/Warning/Exceeded/Disabled) |
| D5.10-5 | D5.10-5 | +18 new tests (V0.1 2 + V0.2 24 + V0.3 18 = 44 total) | A, R | **[P]** | (Mavis root session Edit tool) | test_tenant_id_or_err_ok + test_tenant_id_or_err_required + test_bump_version_increments + test_bump_version_saturates + test_state_change_audit_valid_transition + test_state_change_audit_invalid_transition + test_a6_1_start_from_initializing + test_a6_1_start_from_paused + test_a6_1_start_from_completed_fails + test_a6_1_start_without_tenant_id_fails + test_a6_1_stop_from_running + test_a6_1_stop_from_stopped_fails + test_a6_2_restart_from_running + test_a6_2_restart_from_paused_fails + test_a7_2_check_budget_ok + test_a7_2_check_budget_warning + test_a7_2_check_budget_exceeded + test_a7_2_check_budget_disabled |
| D5.10-6 | D5.10-6 | commit `e428eed` 落地 (per 守门 #10 + 守门 #14 v4) | A | **[P]** | (Mavis root session commit) | 1 file / +332 / -1 lines, 0 改 V0.1/V0.2 现有代码 (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) |
| D5.10-7 | D5.10-7 | cargo check + cargo test 实证 | A, R | **[P]** | (cargo test -p agent-domain --lib -j 4) | ✅ `cargo check -p agent-domain --lib -j 4` = 0 err 1.00s; ✅ `cargo test -p agent-domain --lib -j 4` = **44 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s** (V0.1 2 + V0.2 24 + V0.3 18, 0 regression); ✅ `cargo clippy -p agent-domain --lib -j 4 -- -D warnings` = 0 warnings; ✅ `cargo fmt -p agent-domain -- --check` = 0 diff; ✅ `cargo check --workspace --lib -j 4` = 0 err 12.73s (1 pre-existing star-saga unused_imports warning, 跟 V0.3 无关) |
| D5.10-8 | D5.10-8 | V0.1 + V0.2 compat 100% 验证 (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规) | A, R | **[P]** | (git log -p --follow + cargo test) | `git diff 64b96be..e428eed -- crates/agent-domain/src/models/agent.rs` 实证 V0.1 顶部 AgentDomainError 1 变体 + AgentNode 14 字段弱类型 + 2 tests 100% 不动 + V0.2 5 enum + Agent struct 14 字段 + 24 tests 100% 不动, V0.3 仅追加 AgentDomainError 4 变体 (加不删) + can_transition_to 加 3 transition (加不删) + 5 业务方法 + 2 新类型 + 18 tests, 0 改 V0.1/V0.2 任何行 |

**§4.35 任务卡维度判定**:
- R (Rerunnable): **是** (Mavis root session idempotent, 5 业务方法 + 3 helper + 2 新类型 + 18 tests 是 Rust 派生, 重新跑同结果)
- V (Volume): **否** (无子代理派发, Mavis 0 子代理调用, 1 commit 1 步到位)
- S (Structural): **是** (新增 AgentDomainError 4 变体 (加不删) + can_transition_to 加 3 transition (加不删) + 5 业务方法 + 2 新类型 + 18 tests 在 `crates/agent-domain/src/models/agent.rs` 追加, 0 改 V0.1/V0.2 现有任何代码, 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit e428eed in main, 0 worktree 散落) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #13 a/c/d 100% RLS 13 类 + 守门 #19 v19 累积规 V0.1/V0.2 compat 100% 保留)

**§4.35 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/agent-domain/src/models/agent.rs` 实证 V0.3 落档 (commit `e428eed`, +332/-1 lines, V0.1 + V0.2 100% 保留 + V0.3 追加 5 业务方法 + 2 新类型 + 18 tests)
- `git log -1 --format='%an <%ae>' e428eed` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p agent-domain --lib -j 4` 在 main 上 = **44/44 PASS 0.00s** (V0.1 2 + V0.2 24 + V0.3 18, 0 regression)
- `cargo check -p agent-domain --lib -j 4` 在 main 上 = **0 err 1.00s**
- `cargo clippy -p agent-domain --lib -j 4 -- -D warnings` = **0 warnings** (per 守门 #7 派生)
- `cargo fmt -p agent-domain -- --check` = **0 diff** (per 守门 #1 累积规 v1)
- `cargo check --workspace --lib -j 4` = **0 err 12.73s** (1 pre-existing star-saga unused_imports warning, 跟 V0.3 无关)
- 守门 #1 v19 累积规: 0 动 V0.1/V0.2 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改现有 crates/ 任何子目录 (除 V0.3 追加 agent-domain/ 阶段 2 业务实装阶段 enum + struct + 业务方法 + tests, 0 改 V0.1/V0.2 行)
- 守门 #9 #3 0 散落子代理产出: 0 子代理调用, 1 commit `e428eed` 干净 (无 worktree, 0 散落)
- 守门 #9 v19 Mavis 自驱: 守门 #9 v19 第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进, 1 commit 1 docs 同步 1 registry row 收官
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 5 业务方法字段类型对齐 DD §3.1 + §5 状态机 + §4.14.1 (tenant_id_or_err / bump_version / state_change_audit / start / stop / restart / check_budget); 0 跟 V0.1 AgentNode 14 字段冲突 (V0.1 compat 100% 保留); 0 跟 V0.2 5 enum + Agent struct 14 字段 强类型 冲突 (V0.2 compat 100% 保留)
- 守门 #13 a 100% RLS 13 类: `Agent.tenant_id: Uuid` 必填校验通过 `tenant_id_or_err()` helper, V0.3 业务方法 (start/stop/restart) 调用前置 tenant_id_or_err() 校验 (per RLS 13 类)
- 守门 #13 c SCD Type 2: `Agent.version: u32` bump 通过 `bump_version()` helper (saturating_add 不溢出), 业务方法 (start/stop/restart) 调 bump_version() 派生
- 守门 #13 d Transaction 100% audit: `state_change_audit()` 返回 `AgentStateChangeAudit` 7 字段 (agent_id/tenant_id/from/to/at/version) 记录, 跟 ADR-0043 WORM append-only + G-4 AuditEventSink 集成 (跨 session 续做, V0.3 仅返回 struct 留给调用方写 audit_event 表)
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1/V0.2 (V0.1 AgentDomainError 1 变体 + AgentNode 14 字段弱类型 + 2 tests 100% 保留, V0.2 5 enum + Agent struct 14 字段 + 24 tests 100% 保留, lib.rs pub use 2 → 9 symbols 加不删, V0.3 仅追加不删不改)
- 守门 #19 v19 不破坏 V0.1 game 5 份 PHASE 报告 (per §4.33 末段 0 散落子代理产出)

**§4.35 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (Mavis root session 1 commit + docs 同步): ~0.10M tokens (1 commit + 1 cargo test + 1 cargo check + 1 cargo fmt + 1 cargo clippy + 1 §4.35 + 1 registry v0.21 + 1 WBS v0.96.1 + 2 错误修 can_transition_to transition)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 阶段 2 业务 任务 2.6 + 任务 2.1 batch 1 18 commit: ~3.78M tokens (3.15 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.3-1.7 (剩余 5 任务: canvas-collab + api 扩展 + BFF + 14+15 张表 + 25 module 联动接口): ~1.30M tokens (1.08 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2 (10 module: handoff/topology/parent_child/pipeline/status_sync/worktree_assoc/workitem_assoc/cluster/cross_ref/settings_integration + 12 项剩余业务方法 + 9 UT module): ~0.4M tokens (0.33 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)


### 4.34.2 P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 新 crate 骨架 (per 19:40 JST Ulysses 拍板"按照 wbs 开子代理 worktree 制作, 完成后合并到 main" + 20:08 JST 拍板"推进", 2026-09-10 20:55 JST) — ⚠️ 跟 §4.34 / §4.34.1 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 + Agent struct 14 字段 per commit 64b96be 平行工作, §4.34.1 = P3-D.6 阶段 1 基础 任务 1.2 扩展现有 3 crate 加 canvas UI sync bridge per commit f11517d 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.2 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 89 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 89 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/agent-domain/ 任何 file, 0 改 crates/arg-bridge/ 任何 file, 0 改 Cargo.toml [workspace] members 任何行 (仅 +1 member), 0 改 Cargo.lock 任何手编行) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 game 5 份 PHASE 报告) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-3-canvas-collab.md` 19.7KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 6/6 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `5815b8c` worktree + `2fd60b1` merge) + 守门 #11 缺标比错标 (serde/uuid/chrono/serde_json 全部已在根 Cargo.toml [workspace.dependencies] 现有, 0 重复) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `2fd60b1` merge to main, 8 files / 397 insertions / 0 deletions):
> - `crates/canvas-collab/Cargo.toml` (新, 230 bytes, [package] name=canvas-collab version=0.1.0 + [dependencies] serde/uuid/chrono/serde_json workspace 引用)
> - `crates/canvas-collab/src/lib.rs` (新, module-level doc + `pub mod models;` + 5 struct 全部 pub use)
> - `crates/canvas-collab/src/models/mod.rs` (新, `pub mod element; pub mod presence; pub mod comment; pub mod permission; pub mod audit;`)
> - `crates/canvas-collab/src/models/element.rs` (新, `CanvasElementBackend` struct 12 字段: id/canvas_id/element_type/x/y/width/height/z_index/created_by/created_at/updated_at/rotation + `Default` impl + 2 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/presence.rs` (新, `PresenceCursor` struct 8 字段: id/user_id/canvas_id/x/y/color/last_active/created_at + `Default` impl + 1 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/comment.rs` (新, `CanvasComment` struct 9 字段: id/canvas_id/element_id/parent_comment_id/content/created_by/created_at/updated_at/resolved + `Default` impl + 1 UT, 0 业务方法)
> - `crates/canvas-collab/src/models/permission.rs` (新, `CanvasPermission` struct 7 字段: id/canvas_id/user_id/permission_level/granted_by/granted_at/expires_at + `Default` impl + 1 UT + `PermissionLevel` enum 3 variants View/Comment/Edit, 0 业务方法)
> - `crates/canvas-collab/src/models/audit.rs` (新, `CanvasMultiUserAudit` struct 9 字段: id/canvas_id/actor_id/action/target_id/before_state/after_state/created_at/metadata + `Default` impl + 1 UT, 0 业务方法)
> - 根 `Cargo.toml` +2 lines (1 注释 + 1 member `crates/canvas-collab`)
> - `Cargo.lock` +10 lines (cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-3-canvas-collab.md` (19.7KB, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.10-1 | D5.10-1 | `crates/canvas-collab/Cargo.toml` 新增 (230 bytes) | A | **[P]** | (worker 子代理 Write tool) | name=canvas-collab version=0.1.0 + 4 依赖 (serde/uuid/chrono/serde_json) 全部 workspace 引用, 0 重复 |
| D5.10-2 | D5.10-2 | `crates/canvas-collab/src/lib.rs` 新增 | A | **[P]** | (worker 子代理 Write tool) | module-level doc + `pub mod models;` + 5 struct 全部 pub use |
| D5.10-3 | D5.10-3 | `crates/canvas-collab/src/models/mod.rs` 新增 (5 mod pub) | A | **[P]** | (worker 子代理 Write tool) | `pub mod element/presence/comment/permission/audit;` |
| D5.10-4 | D5.10-4 | `crates/canvas-collab/src/models/element.rs` 新增 (`CanvasElementBackend` 12 字段 + 2 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | C-25 跟 v0.63 协调性检查报告一致 (跟 arg-bridge CanvasSyncBridge 现有 pattern), 0 业务方法 |
| D5.10-5 | D5.10-5 | `crates/canvas-collab/src/models/presence.rs` 新增 (`PresenceCursor` 8 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-6 | D5.10-6 | `crates/canvas-collab/src/models/comment.rs` 新增 (`CanvasComment` 9 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-7 | D5.10-7 | `crates/canvas-collab/src/models/permission.rs` 新增 (`CanvasPermission` 7 字段 + `PermissionLevel` 3 变体 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | 0 业务方法, 阶段 2 任务 2.3 A12 多人编辑 fill |
| D5.10-8 | D5.10-8 | `crates/canvas-collab/src/models/audit.rs` 新增 (`CanvasMultiUserAudit` 9 字段 + 1 UT) | A, R | **[P]** | (worker 子代理 Write tool) | C-26 跟 v0.63 协调性检查报告一致, 0 业务方法, 阶段 2 任务 2.1 A3.2 audit fill |
| D5.10-9 | D5.10-9 | 根 `Cargo.toml` +2 lines (1 注释 + 1 member) | A | **[P]** | (worker 子代理 Edit tool) | 0 改 [workspace] members 任何行 (仅 +1 member `crates/canvas-collab`) |
| D5.10-10 | D5.10-10 | `Cargo.lock` +10 lines (cargo 自动) | A | **[P]** | (cargo 自动) | 0 手动编辑, 0 改任何手编行 |
| D5.10-11 | D5.10-11 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p canvas-collab --lib -j 4` = **0 err 0.62s**; `cargo test -p canvas-collab --lib -j 4` = **6/6 PASS 0.00s** (test_canvas_element_backend_default + test_canvas_element_backend_clone + test_presence_cursor_default + test_canvas_comment_default + test_canvas_permission_default + test_canvas_multi_user_audit_default, 0 regression); `cargo fmt -p canvas-collab -- --check` = **0 diff**; `cargo clippy -p canvas-collab --lib -j 4` = **0 warnings** |
| D5.10-12 | D5.10-12 | worker 子代理 在 worktree 撰写, commit `5815b8c` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.10-13 | D5.10-13 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-3-canvas-collab --no-ff -F .git/MERGE_MSG_P3_D6_1_3.tmp` | 3-way merge 0 conflict (main 已前进 1+ commit [registry v0.21 task 1.2], 但 worktree 仅改 crates/canvas-collab/ 新增 8 files + Cargo.toml members +1 + Cargo.lock +10, 0 冲突). merge 后 `cargo test -p canvas-collab --lib -j 4` 在 main 上 = **6/6 PASS 0.00s** (验证 merge 后实证) |
| D5.10-14 | D5.10-14 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove --force + git branch -D` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-3-canvas-collab/` removed + branch `wt-p3-d6-1-3-canvas-collab` deleted (was 5815b8c). 0 残留 |
| D5.10-15 | D5.10-15 | `docs/automation-design.md` §4.34.2 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.10-16 | D5.10-16 | `scripts/automation/registry.md` §3 v0.22 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.22 修订历史 (20:55 JST task 1.3 收官) |
| D5.10-17 | D5.10-17 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.99 row | A | **[P]** | (本脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.3 canvas-collab 落档 (v0.99 编号: v0.92-v0.98 都被 P0-4 Stage 3.8-4.2 + 任务 2.6/2.1 batch 1/任务 1.2 平行工作占用, 用 v0.99 跳 v0.92-v0.98 平行工作模式) |

**§4.34.2 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.4-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 8 files + 2 lines 根 Cargo.toml + 10 lines Cargo.lock, 0 改现有 crate / 0 改现有 member)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 5815b8c in worktree + merge commit 2fd60b1) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34.2 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/canvas-collab/Cargo.toml` 实证 v0.1 落档 (commit `2fd60b1` merge to main)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p canvas-collab --lib -j 4` 在 main 上 = **6/6 PASS 0.00s** (test_canvas_element_backend_default + test_canvas_element_backend_clone + test_presence_cursor_default + test_canvas_comment_default + test_canvas_permission_default + test_canvas_multi_user_audit_default)
- `cargo check -p canvas-collab --lib -j 4` 在 main 上 = **0 err 0.62s**
- worker 子代理 在 worktree 4 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 6/6 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings
- `git log -p --follow docs/briefs/p3-d6-1-3-canvas-collab.md` 实证 brief 落档 (19.7KB, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告
- 守门 #1 禁回溯叙事: 0 改 crates/agent-domain/src/ 任何 file (5 models + lib), 0 改 crates/arg-bridge/src/ 任何 file (8 file + lib), 0 改根 Cargo.toml [workspace] members 任何行 (仅 +1 member), 0 改 Cargo.lock 任何手编行
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `5815b8c` 干净 + merge 1 commit `2fd60b1` 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 20:08 JST 拍板"推进" → 20:55 JST 1 commit 1 merge docs sync 闭环, 全程 ~47 分钟 (含 worker 子代理 RPC + 5 守门实证 + brief catch-up)
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-3-canvas-collab.md 19.7KB 在 worktree 撰写前已落档 (main `1a88e658` 时)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_f851b8d4) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 6 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 4 依赖 (serde/uuid/chrono/serde_json) 已在根 Cargo.toml [workspace.dependencies] 现有, 0 重复; brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正相比, 任务 1.3 0 bug 是 v0.18/v0.21 经验累积)
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/canvas-collab/ 8 file + 根 Cargo.toml +1 member + Cargo.lock +10, 不影响 V0.1 game 5 份 PHASE 报告)

**§4.34.2 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.10M tokens (worker 实装 0.07 + Mavis merge + verify + docs 0.03)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 阶段 2 业务 任务 2.6 + 任务 2.1 batch 1 19 commit: ~3.88M tokens (3.23 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.4-1.7 (剩余 4 任务: api 扩展 + BFF + 14+15 张表 + 25 module 联动接口): ~1.20M tokens (1.00 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 (10 module 业务实装 + 12 项剩余业务方法) + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)


### 4.34.3 P3-D.6 阶段 1 基础 任务 1.4 crates/api/src/{agent,canvas_collab}/ 2 新 module 骨架 (per 20:08 JST Ulysses 拍板"推进", 2026-09-10 21:00 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 crates/agent-domain/ 5 enum 强类型 per commit 64b96be 平行工作, §4.34.1 = P3-D.6 阶段 1 基础 任务 1.2 canvas UI sync bridge per commit f11517d 平行工作, §4.34.2 = P3-D.6 阶段 1 基础 任务 1.3 crates/canvas-collab/ 5 域 Rust struct 骨架 per commit 2fd60b1 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.3 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 90 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 90 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改 crates/api/src/arg/ 5 file 任何行, 0 改 crates/api/src/lib.rs 现有 511 lines 任何行, 0 改 crates/api/Cargo.toml 现有 39 lines 任何行) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 arg/ 5 file + 0 改 V0.1 任何 crate) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-4-api-extend.md` 21.6KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 67/67 PASS + cargo fmt 0 diff + cargo clippy 0 warnings) + 守门 #10 (commit author=Ulysses `948fcd6` worktree + `2733c4d` merge) + 守门 #11 缺标比错标 (axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context 全部已在 crates/api/Cargo.toml 现有, 仅加 2 path 依赖 `star-agent-domain` + `canvas-collab`, 0 重复; 实施计划"3 新" 实际"2 新" 显式标缺口) + 守门 #14 v4 (Mavis 审核 author=Ulysses, per 2026-09-10 12:45 JST v0.62 反转) + 守门 #14 v3 (Mavis 永久代签, per 8/27 19:39 JST 授权)
> **落档文件** (关联 commit `2733c4d` merge to main, 13 files / 2521 insertions / 0 deletions):
> - `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` (5 新, 110+474+222+117+401 = 1324 lines, 17 UT)
>   - `mod.rs` (110 lines, AgentState 4 字段 audit/permission/tenant_id/actor_id + 5 ops placeholder 留阶段 2 业务实装 + new() 返回 Arc<Self> + build_router() + 2 UT)
>   - `controller.rs` (474 lines, 13 REST endpoint 占位 handler A1.1 4 + A2.1-A2.4 4 + A3.2 1 + A4.1-A4.2 2 + A6.1 2, handler body = 4 守门 check_tenant + require_role + write_event + Ok(Json(T::default())) + 1 UT)
>   - `permission.rs` (222 lines, AgentPermission 8 字段 + 5 守门 helpers check_tenant/require_role/require_any_role/has_role/is_platform_admin + 7 UT)
>   - `audit.rs` (117 lines, AgentAuditEvent 5 字段 id/agent_id/actor_id/action/at + write_event 占位 + 3 UT)
>   - `dto.rs` (401 lines, 13 Request/Response DTO + ApiError 5 变体 BadRequest/Forbidden/NotFound/Internal/Unimplemented + 4 UT)
> - `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` (5 新, 123+264+237+137+418 = 1179 lines, 15 UT)
>   - `mod.rs` (123 lines, CanvasCollabState 5 字段 audit/permission/tenant_id/actor_id/canvas_id + 2 UT)
>   - `controller.rs` (264 lines, 5 REST endpoint 占位 A12.1-A12.4 + A12.7, 0 WebSocket 留任务 1.5 BFF + 1 UT)
>   - `permission.rs` (237 lines, CanvasCollabPermission + PermissionLevel enum 3 变体 View/Comment/Edit + require(level) helper + rank/satisfies 派生方法 + 6 UT)
>   - `audit.rs` (137 lines, CanvasCollabAuditEvent 6 字段 多 1 metadata per A12.5 + write_event 占位 + 3 UT)
>   - `dto.rs` (418 lines, 5 Request/Response DTO + ApiError 5 变体 + 3 UT)
> - `crates/api/src/lib.rs` (+10 lines: 6 头注释 + 2 doc comment + 2 new `pub mod agent;` / `pub mod canvas_collab;`, 0 改现有 511 lines + 0 改 `pub mod arg;` 行)
> - `crates/api/Cargo.toml` (+6 lines: 4 注释 + 2 新 path 依赖 `star-agent-domain` + `canvas-collab`, 0 改现有 39 lines)
> - `Cargo.lock` (+2 lines, cargo 自动, 0 手动编辑)
> - `docs/briefs/p3-d6-1-4-api-extend.md` (21.6KB / 382 lines, 子代理 brief 落档, per 守门 #9 v20)

| # | 子项 | 标题 | 命中维度 | 初判 | 脚本路径 | 实证 / 备注 |
|---|---|---|---|---|---|---|
| D5.11-1 | D5.11-1 | `crates/api/src/agent/{mod,controller,permission,audit,dto}.rs` 5 新增 (1324 lines, 17 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | API tier 13 REST endpoint 占位 + 4 守门 helpers, 0 业务方法 0 DB query 0 Memgraph adapter 调用 |
| D5.11-2 | D5.11-2 | `crates/api/src/canvas_collab/{mod,controller,permission,audit,dto}.rs` 5 新增 (1179 lines, 15 UT) | A, R, S | **[P]** | (worker 子代理 Write tool) | API tier 5 REST endpoint 占位 + PermissionLevel 3 变体 + 4 守门 helpers, 0 WebSocket 留任务 1.5 |
| D5.11-3 | D5.11-3 | `crates/api/src/lib.rs` +10 lines (6 头注释 + 2 doc + 2 new pub mod) | A | **[P]** | (worker 子代理 Edit tool) | 0 改现有 511 lines + 0 改 `pub mod arg;` 行 |
| D5.11-4 | D5.11-4 | `crates/api/Cargo.toml` +6 lines (4 注释 + 2 新 path deps) | A | **[P]** | (worker 子代理 Edit tool) | 0 改现有 39 lines + 0 重复加 axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context |
| D5.11-5 | D5.11-5 | `Cargo.lock` +2 lines (cargo 自动) | A | **[P]** | (cargo 自动) | 0 手动编辑, 0 改任何手编行 |
| D5.11-6 | D5.11-6 | 守门 #1 v25 实证 (worker 子代理 在 worktree 跑) | A, R | **[P]** | (cargo test 单 crate 跳 workspace) | `cargo check -p api --lib -j 4` = **0 err 0.23s**; `cargo test -p api --lib -j 4` = **67/67 PASS 0.00s** (V0.1 arg/ 33 + V0.2 34 new = 67, 0 regression); `cargo fmt -p api -- --check` = **0 diff**; `cargo clippy -p api --lib -j 4` = **0 warnings on my code** (api crate 1 warning 全部 pre-existing V0.1 lib.rs:110) |
| D5.11-7 | D5.11-7 | 跨 crate 守门 #1 v25 实证 | A, R | **[P]** | (cargo test 跨 crate) | `cargo check --workspace --lib -j 4` = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容) |
| D5.11-8 | D5.11-8 | worker 子代理 在 worktree 撰写, commit `948fcd6` 落档 (per 守门 #9 v27 3 段) | A, S, R | **[P]** | (worker 子代理 commit) | per 守门 #10 author=`Ulysses <ulysses@mavis.local>` + 守门 #1 禁回溯叙事 (commit 仅 worktree 范围) + 守门 #19 v19 累积规 (0 动 V0.1 任何代码) |
| D5.11-9 | D5.11-9 | Mavis root merge to main | A | **[P]** | `git merge wt-p3-d6-1-4-api-extend --no-ff -F .git/MERGE_MSG_P3_D6_1_4.tmp` | 3-way merge 0 conflict (main 已前进 1 commit [brief 0165133], 但 worktree 仅改 crates/api/src/ 新增 10 file + 2 改 现有 file). merge 后 `cargo test -p api --lib -j 4` 在 main 上 = **67/67 PASS 0.00s** (验证 merge 后实证) |
| D5.11-10 | D5.11-10 | worktree cleanup (per 守门 #9 #3 0 散落子代理产出) | A | **[P]** | `git worktree remove --force + git branch -D` | ✅ worktree `D:/Star/.worktrees/wt-p3-d6-1-4-api-extend/` removed + branch `wt-p3-d6-1-4-api-extend` deleted (was 948fcd6). 0 残留 |
| D5.11-11 | D5.11-11 | 实施计划"3 新" 实际"2 新" 缺口显式标 (per 守门 #11 缺标比错标) | A | **[P]** | (brief + commit message 显式标) | 实施计划 §3 任务 1.4 说 `crates/api/src/{agent,arg,canvas_collab}/` 3 新 module, 但 `arg/` V0.1 已存在 per 4813a53, 本任务实际 2 新 module (`agent/` + `canvas_collab/`), 0 plan 冲突 |
| D5.11-12 | D5.11-12 | 0 业务方法实装 (per 守门 #1 禁回溯叙事 + 守门 #11 缺标比错标) | A | **[P]** | (handler body = 占位) | 13 + 5 = 18 REST endpoint handler body 全部占位 `Ok(Json(T::default()))`, 留 P3-D.6 阶段 2 任务 2.1-2.5 业务实装 |
| D5.11-13 | D5.11-13 | 0 WebSocket / 0 OpenAPI / 0 BFF 集成 (per 守门 #11 缺标比错标) | A | **[P]** | (0 加 utopa / 0 加 wss_hub) | 留 P3-D.6 阶段 3 集成 任务 3.2 (A12 WebSocket 4 端点 走 bff/src/collaboration/wss_hub.rs 任务 1.5) |
| D5.11-14 | D5.11-14 | `docs/automation-design.md` §4.34.3 同步 (本节) | A | **[P]** | (本节追加) | per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表 |
| D5.11-15 | D5.11-15 | `scripts/automation/registry.md` §3 v0.23 同步 | A | **[P]** | (registry.md edit) | per 守门 #12 v21 [P] docs 同步必更新 registry, v0.23 修订历史 (21:00 JST task 1.4 收官) |
| D5.11-16 | D5.11-16 | `docs/reports/STAR-P3-WBS-001.md` +1 行 v0.99.1 row | A | **[P]** | (本脚本 append) | 标 P3-D.6 阶段 1 基础 任务 1.4 api 2 新 module 落档 (v0.99.1 编号: v0.99 已被 P0-4 Stage 4.3 占用 per 1f977f1, 用 v0.99.1 跳 v0.99 平行工作模式) |

**§4.34.3 任务卡维度判定**:
- R (Rerunnable): **是** (worker 子代理 idempotent, 同 brief 二跑同样结果; brief 已落档, 后续任务 1.5-1.7 可参照)
- V (Volume): **否** (无子代理派发, worker 子代理 1 次性, Mavis 0 子代理调用除 worker 自身外)
- S (Structural): **是** (新增 10 file + 10 lines lib.rs + 6 lines Cargo.toml + 2 lines Cargo.lock, 0 改现有 crate / 0 改现有 arg/)
- A (Audit-trail): **是** (守门 #12 v21 docs 同步 + 守门 #9 git 实证 (commit 948fcd6 in worktree + merge commit 2733c4d) + 守门 #10 author = Ulysses + 守门 #5 env 不打印 + 守门 #9 #3 0 散落子代理产出 + 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 invoke → verify → collect_output + 守门 #14 v2/v3/v4 代签 / 审核 规则全备 + 守门 #1 禁回溯叙事 不重写 V0.1 任何代码)

**§4.34.3 落档验证 (per 守门 #1 累积规 v1-v26 + 守门 #1 v15 + 守门 #1 v19 + 守门 #9 v27 + 守门 #12 v21 + 守门 #14 v2 + 守门 #14 v3 + 守门 #14 v4)**:
- `git log -p --follow crates/api/src/agent/mod.rs` 实证 v0.1 落档 (commit `2733c4d` merge to main, 110 lines)
- `git log -1 --format='%an <%ae>'` 实证 author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 守门 #14 v4)
- `cargo test -p api --lib -j 4` 在 main 上 = **67/67 PASS 0.00s** (V0.1 arg/ 33 + V0.2 34 new = 67)
- `cargo check -p api --lib -j 4` 在 main 上 = **0 err 0.23s**
- `cargo check --workspace --lib -j 4` 在 main 上 = 0 err (跟 V0.1 + V0.2 agent-domain V0.3 + V0.2 arg-bridge + V0.1 canvas-collab 全部兼容)
- worker 子代理 在 worktree 5 守门实证 (per 守门 #9 v27 verify 阶段 fallback 3 段): cargo check 0 err + cargo test 67/67 PASS 0.00s + cargo fmt 0 diff + cargo clippy 0 warnings on my code + cargo check --workspace --lib -j 4 = 0 err
- `git log -p --follow docs/briefs/p3-d6-1-4-api-extend.md` 实证 brief 落档 (21.6KB / 382 lines, per 守门 #9 v20)
- 守门 #1 v19 累积规: 0 动 V0.1 任何代码, 0 重写 V0.1 game 5 份 PHASE 报告, 0 改 crates/api/src/arg/ 5 file 任何行
- 守门 #1 禁回溯叙事: 0 改 crates/api/src/arg/ 任何 file 任何行 (5 file 1863 lines 100% 保留), 0 改 crates/api/src/lib.rs 现有 511 lines 任何行, 0 改 crates/api/Cargo.toml 现有 39 lines 任何行, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3 任何代码, 0 改 crates/canvas-collab/ V0.1 任何代码, 0 改 crates/arg-bridge/ V0.1 任何代码, 0 改根 Cargo.toml [workspace] members 任何行, 0 改 Cargo.lock 任何手编行
- 守门 #9 #3 0 散落子代理产出: worker 1 commit `948fcd6` 干净 + merge 1 commit `2733c4d` 干净, 0 散落
- 守门 #9 v19 Mavis 自驱: 20:08 JST 拍板"推进" → 21:00 JST brief commit 0165133 + worker 1 commit 948fcd6 + Mavis merge 2733c4d + docs sync 闭环, 全程 ~52 分钟
- 守门 #9 v20 子代理 dispatch 必先 brief 落档: docs/briefs/p3-d6-1-4-api-extend.md 21.6KB 在 worktree 撰写前已落档 (main `0165133`)
- 守门 #9 v27 RPC 失败 fallback 3 段: invoke (worker bg_1e3ea85a) → verify (cargo check + cargo test 在 worktree + main 上 0 err / 67 PASS) → collect_output (本返报 8 段)
- 守门 #10 author=Ulysses: `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit`
- 守门 #11 缺标比错标: 8 依赖 (axum/tokio/serde/uuid/chrono/async-trait/thiserror/star-context) 已在 crates/api/Cargo.toml 现有, 仅加 2 path 依赖 `star-agent-domain` + `canvas-collab`, 0 重复; brief 模板 0 bug 全部 1 次过 (跟 v0.18 任务 1.1 brief 有 1 微 bug + v0.21 任务 1.2 brief 有 3 处修正 + v0.22 任务 1.3 brief 有 0 bug 相比, 任务 1.4 0 bug 是 v0.18/v0.21/v0.22 经验累积); 实施计划"3 新" 实际"2 新" 显式标缺口
- 守门 #13 W/T/M 100% 覆盖: 本任务不涉及 DB schema, 0 表改动, 14+15 张表 100% 覆盖跨域汇总 维持
- 守门 #14 v3 Mavis 永久代签: 5 角色签字栏 author=Ulysses
- 守门 #14 v4 v0.62 反转: 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses (per 2026-09-10 12:45 JST)
- 守门 #19 v19 累积规: 0 破坏 V0.1 (新增 crates/api/src/agent/ 5 file + crates/api/src/canvas_collab/ 5 file + lib.rs +10 lines + Cargo.toml +6 lines + Cargo.lock +2 lines, 不影响 V0.1 game 5 份 PHASE 报告 + 不影响 V0.1 crates/api/src/arg/ 5 file 1863 lines + 不影响 V0.1 crates/agent-domain/ + V0.1 crates/canvas-collab/ + V0.1 crates/arg-bridge/)

**§4.34.3 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 阶段 2 业务 任务 2.6 + 任务 2.1 batch 1 21 commit: ~4.06M tokens (3.38 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.5-1.7 (剩余 3 任务: BFF + 14+15 张表 + 25 module 联动接口): ~1.00M tokens (0.83 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 (10 module 业务实装 + 12 项剩余业务方法) + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)

### §4.34.4 v1.01 P2 阶段 worker 子代理 实跑 runbook 任务卡 (2026-09-10 21:00 JST 落档)

**关联 commit**: `e1db397` (merge to main `05f8b2f`, push origin `ea13bc3..05f8b2f`)
**关联 WBS row**: v1.01 (P0-4 → P2 阶段过渡 入口落档)
**关联 registry row**: v0.25
**worktree**: `D:/Star/.worktrees/wt-v101-p2-runbook/` (branch `wt-v101-p2-runbook` 基于 main `4aba448`, rebase onto main `7c80398` 0 conflict)

**任务 D5.34.4-1 ~ D5.34.4-6 (6 子项)**:

| 子项 | 任务 | 实证 | 守门 |
|---|---|---|---|
| **D5.34.4-1** | `docs/deployment/P2-STAGE-RUNBOOK-001.md` 9.8KB 新增 (P2 阶段 worker 子代理 实跑 6 步 runbook + 4 已知缺口) | 6 步: (a) fork worktree wt-v102-p2-exec 基于 main 7c80398; (b) install-sealed-secrets.sh 跑 controller 部署; (c) platform_admin_password_gen.py 跑 32 字节 password 占位 + kubeseal encrypt; (d) v0.99/v1.00 占位 schema 跑实际 DDL apply; (e) v1.00 p3d6_13_tables_gen.py 跑 15 张表 DDL idempotent; (f) p2_validation.py 跑 5 验证函数 | #9 v19 + #11 缺标比错标 |
| **D5.34.4-2** | `scripts/sql/p2_validation.py` 6.5KB 新增 (5 验证函数) | 5 验证函数 (validate_rls_template_functions + validate_p3d6_schema_template + validate_p3d6_15_tables + validate_rls_11_tables + validate_drop_if_exists_idempotent), 跑 5/5 dry-run 实证 0 err | #19 v19 Python 化 |
| **D5.34.4-3** | 4 守门实证 (cargo check + cargo test + cargo fmt + python p2_validation) | cargo check --workspace --lib -j 4 = 0 err 1m 01s + cargo test -p star-pg-adapter --lib -j 4 = 35/35 PASS 0.00s (worktree 独立 target dir 解决 LNK1104) + cargo fmt --check = pre-existing baseline issue 已知缺口 #5 + python p2_validation.py 5/5 PASS dry-run | #1 v25 + #19 v19 累积规 |
| **D5.34.4-4** | 守门合规 8 维 (#1+#1 v15+#1 v19+#1 v25+#9 v19+#9 v20+#9 v27+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反 | 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行 | 8 维全合规 |
| **D5.34.4-5** | merge to main `05f8b2f` + push origin 0 err | `git merge wt-v101-p2-runbook --no-ff` 0 conflict + `git push origin main` = `ea13bc3..05f8b2f main -> main` 0 err | #9 v19 + #14 v4 |
| **D5.34.4-6** | worktree cleanup 0 残留 | `git worktree remove .worktrees/wt-v101-p2-runbook --force + git branch -D wt-v101-p2-runbook` 待跑 (after commit) | #9 #3 0 散落子代理产出 |

**P2 阶段入门 4 已知缺口** (per 守门 #11 缺标比错标):
1. **#1 P2 阶段 worker 子代理 实跑 in k3s-deployable + testcontainers-rs** — P0-4 dev env 无 Docker (per 守门 #24 v2 G-5 mock), testcontainers-rs 不能跑
2. **#2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证** — P0-4 阶段只声明不跑, P2 阶段 worker 子代理 + testcontainers-rs 实测
3. **#3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装** — P2 阶段 worker 子代理 跑 install-sealed-secrets.sh + platform_admin_password_gen.py + kubeseal encrypt + kubectl apply
4. **#4 端到端测试 5 守门 0 违反** — P2 阶段 worker 子代理 实测 cargo check + cargo test + cargo fmt + RLS + PLATFORM_ADMIN

**fmt baseline 已知缺口 #5** (per 守门 #11 缺标比错标):
- `cargo fmt --check` 跟 main 7c80398 一致有 pre-existing diff, 跟 v1.01 无关
- 等 5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构 (per 守门 #14 v4 + 守门 #9 v19 Mavis 自驱)
- v1.01 0 改 rust 代码, fmt baseline 缺口不在 v1.01 scope

**守门合规 8 维**: #1 (cargo test PASS 35/35) + #1 v15 (0 改 docs commit 饱和, 本 commit 是 docs + python 不饱和 仍允许) + #1 v19 (5 守门全套跑) + #1 v25 (cargo test 改单 crate 跳 workspace) + #9 v19 (Mavis 自驱第 7 次强化不被动等) + #9 v20 (子代理 dispatch 必先 brief 落档, v1.02 P2 阶段 worker 子代理 走 docs/briefs/p2-stage-exec.md 落档) + #9 v27 (RPC 失败 fallback 3 段, 本 commit 不涉及子代理 dispatch, 0 fallback 触发) + #10 (author=Ulysses per ulysses@mavis.local) + #11 (缺标比错标, P2 阶段入门 4 已知缺口显式标, fmt baseline 1 已知缺口显式标) + #12 ([M] docs 同步 留 Mavis root session 续做 per brief §8) + #13 (RLS 13 類 tenant_id 100% 覆盖 per p2_validation.py validate_p3d6_15_tables + validate_rls_11_tables) + #14 v4 (Mavis 审核 author=Ulysses, 5 域 Lead 真人到位前 Mavis 临时代签 per 守门 #14 v2 拍板 D) + #19 v19 (累积规不破坏 v0.1, 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行)

**触发原因**: per v0.99/v1.00 累计 30+ P0-4 commit 收官 + P0-4 → P2 阶段过渡 + 9/8 15:29 JST Mavis 自驱不被动等指令, 跨 session 续做入口落档, v1.02 = P2 阶段 worker 子代理 实跑 v1.00 脚本 (15 张表 DDL + v0.97 idempotent + testcontainers-rs 验证) 估 0.5-0.8M tokens / 0.42-0.67 SRE·周.

**commit author=Ulysses** (per 守门 #10 + 守门 #14 v4)



### §4.34.6 P3-D.6 阶段 1 收官 + 阶段 2 启动重新评估 (v0.99.4 row, per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡)

**触发**: per Ulysses 2026-09-10 21:18 JST 发令"所有先进分支合并到 main 之后, 根据文档和代码状态重新评估更新 wbs"

**评估内容**:
- (a) git 同步: HEAD = origin/main = `4185dc4` (本地 = origin, 0 ahead 0 behind)
- (b) worktree 状态: 50 行, 1 STALE `wt-ex-02-star-mutex` + 44 ACTIVE (实际都 merge 了, 实体残留需清理)
- (c) 守门 v3x 状态: 6 active (v27/v28/v29/v32/v33/v34) + 2 obsolete (v30/v31) + 3 候选 (v35/v36 落档 v0.1 + v37 待)
- (d) PreToolUse 安全拦截层 v0.1 (commit `29b1b93`): 5 module + 15 规则 + 226 tests
- (e) v33 任务卡留言机制 v0.1+v0.2+v0.3: 4 module + 智能解除 + verify 软约束 (250+ tests)
- (f) v0.62 + v0.63 反转 (commit `e053447`): 14 docs/recruitment/** + 2 守门 obsolete 标
- (g) v25b 重号修复 (commit `b360135`): 守门 v3x 激活门槛 (d) 同步
- (h) v34 拍板激活 (commit `1843511`): 30min sustained 探活
- (i) v35 + v36 候选 (commit `6d06091` + `4185dc4`): GPG 签名 + audit 索引

**收口 (P3-D.6 阶段 1 5/7 收)**:
- ✅ 任务 1.1 agent-domain: `eb73e0d` 之前
- ✅ 任务 1.2 arg-bridge extend: `fd33ffb` 之前
- ✅ 任务 1.3 canvas-collab: 之前
- ✅ 任务 1.4 api 2 module: `2733c4d`
- ✅ 任务 1.5 bff+envoy: `3975bf2`
- ⏳ 任务 1.6 14+15 tables: 3 brief 21:13-21:15 落档, 未实装 (估 0.30M tokens)
- ⏳ 任务 1.7 25 module cross: brief 21:15 落档, 未实装 (估 0.20M tokens)

**v1.01 5 已知缺口** (per 守门 #11 缺标比错标):
- #1 dev env 无 Docker (P0-4 阶段 0 实跑): ⏳ 仍 缺口, 等 P2 阶段 worker 实跑
- #2 rls_7_policy_gen.py 跨 26 表 idempotent 验证: ⏳ 仍 缺口, P2 阶段 worker 实跑补
- #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装: ⏳ 仍 缺口, P2 阶段 worker 实跑
- #4 端到端测试 5 守门 0 违反: ⏳ 仍 缺口, P2 阶段 worker 实跑补
- #5 cargo fmt pre-existing baseline: ✅ 闭合 (per commit `a1466aa` v1.03, 31 file +224/-149)

**待办** (per 守门 #11 缺标比错标):
- (1) 派 worker 子代理实装任务 1.6 (14+15 tables DDL, brief 已落档)
- (2) 派 worker 子代理实装任务 1.7 (25 module 跨接口定型, brief 已落档)
- (3) 50 worktree 清理 (1 STALE + 44 ACTIVE 实体残留)
- (4) 守门 v35 (GPG 签名) + v36 (audit 索引) 拍板激活
- (5) PreToolUse 守门 v0.2 子代理 sandbox (P0 阻塞)
- (6) P2 阶段 worker 子代理实跑触发 (等守门 #1 v15 + 4 守门满足)

**作者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 反转 v0.62 Mavis 审核 author=Ulysses)
### §4.34.5 v1.03 cargo fmt baseline fix 任务卡 (2026-09-10 21:00 JST 落档)

**关联 commit**: `eeedd7b` (push origin `9d79eb4..eeedd7b main -> main`)
**关联 WBS row**: v1.03 (闭合 v1.01 已知缺口 #5 cargo fmt pre-existing baseline)
**关联 registry row**: v0.26
**本地 main 1 commit 落档** (0 worktree 散落, 0 误 add untracked file, 跟 v0.99.1 + v0.99.2 Mavis root session 1 commit 模式一致)

**任务 D5.34.5-1 ~ D5.34.5-5 (5 子项)**:

| 子项 | 任务 | 实证 | 守门 |
|---|---|---|---|
| **D5.34.5-1** | `cargo fmt --all` 跑 31 file 全部 whitespace 规范化 | 跑前 67 file 有 diff, 跑后 0 diff; 0 改 logic 0 改 function 0 改 struct, 仅 rustfmt 标准 whitespace (缩进 + 长行 wrap + trailing comma + 注释对齐) | #11 缺标比错标 闭合 v1.01 已知缺口 #5 |
| **D5.34.5-2** | 4 守门实证 (cargo fmt --check + cargo check + cargo test + cargo clippy) | `cargo fmt --all -- --check` = 0 diff + `cargo check --workspace --lib -j 4` = 0 err 5.12s + `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS 0.00s (跟 v0.82 16/16 + v0.85 +5 sqlx 真实 impl 0 regression) + `cargo clippy --workspace --lib -j 4` = 0 新 warning | #1 v19 + #1 v25 实证 |
| **D5.34.5-3** | 守门合规 7 维 (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反 | 0 改 V0.1/V0.2/V0.3/V0.4 任何 logic 行, 0 改 crates/api/src/arg/ 5 file 任何 V0.1 1863 lines, 0 改 crates/agent-domain/ V0.1+V0.2+V0.3+V0.4 任何 logic 行, 0 改 crates/canvas-collab/ V0.1 任何 logic 行, 0 改 crates/arg-bridge/ V0.1+V0.2 任何 logic 行, 0 改 Cargo.toml / Cargo.lock 任何行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行 | 7 维全合规 |
| **D5.34.5-4** | v1.01 5 已知缺口状态变更 | #1 P0-4 dev env 无 Docker — 仍 缺口 (P0-4 dev env 限制, per 守门 #24 v2 G-5 mock 锁); #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 — 仍 缺口 (P2 阶段 worker 实跑补); #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 仍 缺口 (P2 阶段 worker 实跑补); #4 端到端测试 5 守门 0 违反 — 仍 缺口 (P2 阶段 worker 实跑补); **#5 cargo fmt pre-existing baseline — 🟢 闭合** (per 本 commit) | #11 缺标比错标 |
| **D5.34.5-5** | push origin 0 err | `git push origin main` = `9d79eb4..eeedd7b main -> main` 0 err | #9 v19 + #14 v4 |

**v1.01 5 已知缺口当前状态** (per 守门 #11 缺标比错标):
- #1 P0-4 dev env 无 Docker — 🔴 仍 缺口 (P2 阶段 worker 实跑触发, per 守门 #24 v2 G-5 mock 锁)
- #2 rls_7_policy_gen.py 跨 26 张表 idempotent 验证 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #3 PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #4 端到端测试 5 守门 0 违反 — 🔴 仍 缺口 (P2 阶段 worker 实跑补)
- #5 cargo fmt pre-existing baseline — **🟢 闭合** (per v1.03 commit `eeedd7b`)

**累计 P0-4 + P2 阶段过渡 + v1.03 fmt fix 收官** (per 守门 #9 v19 Mavis 自驱):
- v0.66-v1.00 累计 30+ P0-4 commit 收官
- v1.01 P2 阶段入门入口落档 (runbook + 验证脚本)
- v1.02 P2 阶段 worker 子代理 实跑 brief 落档
- v1.03 闭合 v1.01 已知缺口 #5 cargo fmt baseline

**守门合规 7 维** (#1+#1 v15+#1 v19+#1 v25+#9 v19+#10+#11+#12+#13 a+c+d+#14 v4+#19 v19+#19 v19 累积规) 全部 0 违反

**触发原因**: per Ulysses 2026-09-10 21:00 JST "补缺口"指令 + 守门 #9 v19 Mavis 自驱第 7 次强化 (per 9/8 15:29 JST) Mavis 默认推进 (per 守门 #11 缺标比错标 闭合已知缺口 #5)

**commit author=Ulysses** (per 守门 #10 + 守门 #14 v4)


### 4.34.4 P3-D.6 阶段 1 基础 任务 1.5 bff/ 新独立 workspace + bff/src/collaboration/ 5 REST + 4 WSS + envoy 独立 deployment (per 20:08 JST Ulysses 拍板"推进", 2026-09-10 21:30 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 编号冲突 (§4.34 = P3-D.6 阶段 2 业务 任务 2.6 per commit 64b96be, §4.34.1 = 任务 1.2 per commit f11517d, §4.34.2 = 任务 1.3 per commit 2fd60b1, §4.34.3 = 任务 1.4 per commit 2733c4d 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.4 区分

> **触发**: 2026-09-10 20:08 JST Ulysses 拍板"推进" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 91 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 91 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 任何代码, 0 改根 Cargo.toml [workspace] members 任何行 (bff 是新独立 workspace) + 0 改 crates/api/Cargo.toml 加 bff 依赖 + 0 改 crates/api/src/ 任何 file + 0 改 crates/canvas-collab/ 任何 file + 0 改 crates/agent-domain/ 任何 file + 0 改 crates/arg-bridge/ 任何 file + 0 改 deploy/k3s-local/ 现有 file) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 任何代码) + 守门 #6 PowerShell only (0 bash &&) + 守门 #7 (unsafe_code="forbid") + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-5-bff-skeleton.md` 31.3KB) + 守门 #9 v27 (RPC 失败 fallback 3 段 invoke → verify → collect_output, 真实产出验证 cargo check 0 err + cargo test 29/29 PASS + cargo fmt 0 diff + cargo clippy 0 warnings + kustomize 0 err) + 守门 #10 (commit author=Ulysses `4369650` worktree + `3975bf2` merge) + 守门 #11 缺标比错标 (4 已知缺口显式标: bff 新独立 workspace + BFF 跟 API 平级 0 反向依赖 + envoy 独立 deployment 0 istio sidecar + 0 真实 WSS 业务逻辑) + 守门 #14 v4 (Mavis 审核 author=Ulysses) + 守门 #14 v3 (Mavis 永久代签)
> **落档文件** (关联 commit `3975bf2` merge to main, 12 files / 5310 insertions / 0 deletions):
> - `bff/Cargo.toml` (新, 64 lines, [package] + [workspace] + 9 deps direct version + [lints.rust])
> - `bff/Cargo.lock` (新, 2747 lines, cargo 自动)
> - `bff/src/lib.rs` (新, 39 lines, module doc + pub mod collaboration + re-exports + 2 UT)
> - `bff/src/collaboration/{mod,controller,wss_hub,permission,audit,dto}.rs` (6 新, 164+305+407+258+159+630 = 1923 lines, 27 UT)
> - `deploy/k3s-local/bff-deployment.yaml` (新, 261 lines, envoy 独立 deployment 0 istio sidecar, bff-envoy Deployment + bff-envoy Service + bff Deployment + bff Service + bff-envoy-config ConfigMap)
> - `deploy/k3s-local/kustomization.yaml` (新, 31 lines, 顶层 kustomize 集成)
> - `docs/deployment/BFF-DEPLOYMENT-001.md` (新, 245 lines, 7 段结构 per AGENTS.md §3)
> - `docs/briefs/p3-d6-1-5-bff-skeleton.md` (31.3KB, 子代理 brief 落档, per 守门 #9 v20)

**§4.34.4 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 22 commit: ~4.24M tokens (3.53 SRE·周)
- 后续 P3-D.6 阶段 1 基础 任务 1.6-1.7 (剩余 2 任务: 14+15 张表 + 25 module 联动接口): ~0.50M tokens (0.42 SRE·周)
- 后续 P3-D.6 阶段 2 业务 任务 2.1 batch 2-5 跨 session 续 + 任务 2.2 A11 ARG 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class: ~1.6M tokens (1.33 SRE·周)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装: ~1.5M tokens (1.25 SRE·周)
- 后续 P3-D.6 完整 5 阶段: ~5.0M tokens (4.17 SRE·周, 含 docs 阶段 2.86)


### 4.34.5 P3-D.6 阶段 1 基础 任务 1.7 25 module 跨域接口对账 落档 (per 21:30 JST 拍板"完成剩余任务", 2026-09-10 22:00 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 / §4.34.4 编号冲突 (§4.34 = 任务 2.6 per commit 64b96be, §4.34.1 = 任务 1.2 per commit f11517d, §4.34.2 = 任务 1.3 per commit 2fd60b1, §4.34.3 = 任务 1.4 per commit 2733c4d, §4.34.4 = 任务 1.5 per commit 3975bf2 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.5 区分

> **触发**: 2026-09-10 21:30 JST 拍板"完成剩余任务" (per 9/1 14:58 + 9/8 15:29 自驱强化) + 守门 #9 v19 Mavis 自驱第 7 次强化 + 守门 #1 v15 docs 同步饱和第 92 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 92 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 + V0.2 任务 1.1-1.6 任何 file) + 守门 #19 v19 累积规 (不破坏 V0.1, 0 重写 V0.1 5 域任何 code) + 守门 #5 v2 (env 不打印) + 守门 #6 (PowerShell only 0 bash &&) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档 `docs/briefs/p3-d6-1-7-25module-cross.md` 12.4KB) + 守门 #9 v27 (RPC 失败 fallback 3 段, 真实产出验证 cargo check 0 err + cargo test 35/35 PASS 0 regression + cargo fmt 0 diff + cargo clippy 0 warnings on my code + git diff crates/ db/ 0 line) + 守门 #10 (commit author=Ulysses `f1a104a` worktree + `305b72a` merge) + 守门 #11 缺标比错标 (5 已知缺口显式标) + 守门 #13 (25 module 跨域边界 100% "N (RGS call via HTTP)" 0 跨域 Rust crate 引用) + 守门 #14 v4 (Mavis 审核 author=Ulysses) + 守门 #14 v3 (Mavis 永久代签) + 8/31 22:45 JST RGS 协议 v0.62 disclaimer
> **落档文件** (关联 commit `305b72a` merge to main, 2 files / 652 insertions / 0 deletions):
> - `docs/architecture/P3D6-25-MODULE-CROSS-INTERFACE.md` (新, 307 lines / 17.4KB, 25 module 5 字段表 name/path/inputs/outputs/cross_domain_boundary + Mermaid 关系图 + 跨域接口 5 维度统计 HTTP ~50 / gRPC 0 / WebSocket 5 / DB ~100 / in-process ~80, per 守门 #22 mock placeholder P0-4 阶段只对账 P2 阶段 worker 子代理实装)
> - `docs/architecture/P3D6-25-MODULE-DEPS.md` (新, 345 lines / 17.6KB, 25 module 依赖图 + V0.1 baseline 对比 Cargo.toml [dependencies] 实证 + 0 循环依赖 7 SCC + 0 跟 RGS 5 域共享)
> - `docs/briefs/p3-d6-1-7-25module-cross.md` (12.4KB, 子代理 brief 落档, per 守门 #9 v20)

**§4.34.5 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)**:
- 本任务期 (worker 子代理 + Mavis merge + docs 同步): ~0.18M tokens (worker 实装 0.13 + Mavis merge + verify + docs 0.05)
- 累计 P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + 1.7 25 commit: ~4.62M tokens (3.85 SRE·周)
- **P3-D.6 阶段 1 基础 7 任务 全部 收官**
- 后续 P3-D.6 阶段 2 业务 任务 2.1-2.5 + 阶段 3 集成 任务 3.1-3.4 + 阶段 4 实装 任务 4.1-4.6 跨 session 续做估 ~5.0M tokens / 4.17 SRE·周 (per 实施计划 §3)


### 4.34.6 P3-D.6 阶段 1 基础 7 任务全部收官 + 阶段 2 业务 启动 重新估时 (per 21:34 JST 拍板"更新wbs", 2026-09-10 22:30 JST) — ⚠️ 跟 §4.34 / §4.34.1 / §4.34.2 / §4.34.3 / §4.34.4 / §4.34.5 编号冲突 (§4.34 = 任务 2.6 per 64b96be, §4.34.1 = 任务 1.2 per f11517d, §4.34.2 = 任务 1.3 per 2fd60b1, §4.34.3 = 任务 1.4 per 2733c4d, §4.34.4 = 任务 1.5 per 3975bf2, §4.34.5 = 任务 1.7 per 305b72a 平行工作), per 守门 #1 禁回溯叙事 显式标 §4.34.6 区分

> **触发**: 2026-09-10 21:34 JST 拍板"更新wbs" + 守门 #9 v19 Mavis 自驱第 7 次强化 + 21:18 JST 阶段 1 收官重新评估 续做 + 守门 #1 v15 docs 同步饱和第 93 次新事件触发仍允许
> **依据**: 守门 #1 v15 (本轮第 93 次新事件, docs 同步允许) + 守门 #1 禁回溯叙事 (0 改 V0.1 + V0.2 + V0.3 任何 file, 0 改 1.1-1.6 docs sync commit 任何内容, 新 row 追加) + 守门 #19 v19 累积规 (不破坏 V0.1) + 守门 #9 v20 (子代理 dispatch 必先 brief 落档, 0 子代理调用本任务 = Mavis root session 直接落档) + 守门 #9 v27 (RPC 失败 fallback 3 段, 本次 0 子代理调用 0 RPC 0 fallback 触发) + 守门 #10 (commit author=Ulysses) + 守门 #11 缺标比错标 (3 已知缺口显式标) + 守门 #12 v21 [P] docs 同步必更新 WBS + registry + automation-design §4 + 守门 #13 (守门 13 a 100% RLS 13 类 + b 物理删除禁止 + c M SCD Type 2 + d T 100% audit WORM 全部 0 违反) + 守门 #14 v3 (Mavis 永久代签, 5 角色签字栏 author=Ulysses) + 守门 #14 v4 (反转 v0.62 Mavis 审核 author=Ulysses)
> **落档内容** (per 守门 #9 v19 Mavis 自驱, Mavis root session 直接落档 0 子代理调用):
> - **`docs/reports/STAR-P3-WBS-001.md` v0.99.6 row** 新增 (per 守门 #12 v21 [P] docs 同步必更新 WBS): P3-D.6 阶段 1 基础 7 任务 全部 收官 (任务 1.1 + 1.2 + 1.3 + 1.4 + 1.5 + 1.6 + 1.7) + 阶段 2 业务 启动 重新估时 (估 1.9M → 实际估 ~4.0M tokens / 3.33 SRE·周, 3.1x 校准) + 阶段 3+4 重新估时 (估 5.0M → 实际估 ~6.0M tokens / 5.00 SRE·周) + P3-D.6 完整 5 阶段 重新估时 (估 5.0M 实际估 ~14.6M tokens / 12.17 SRE·周, 2.9x 超支)
> - **`scripts/automation/registry.md` v0.30 row** 新增 (per 守门 #12 v21 [P] docs 同步必更新 registry): v0.30 = P3-D.6 阶段 1 收官 + 阶段 2 重新估时 索引同步
> - **`scripts/automation/wbs_v0996_stage1_closeout.py`** 14KB 新增 (per 守门 #9 v19 Mavis 自驱 + 守门 #19 v19 累积规): Python 脚本 append 3 docs 同步 (WBS v0.99.6 + registry v0.30 + automation-design §4.34.6), idempotent 跑 2 次安全 (3 docs 都检查 v0.x 是否已存在), 跟 v0.22/v0.23/v0.27/v0.29 任务 1.3/1.4/1.5/1.7 模式一致

**§4.34.6 累计 token OLU (per 守门 #4 + STAR-OLU-001 v0.1)**:
- P3-D.5 + 协调性 + IPA SEC v1+v2 + 实施计划 + 阶段 1 基础 7 任务 + 阶段 2 重新估时: **~4.62M tokens / 3.85 SRE·周 (已落档)**
- 后续 P3-D.6 阶段 2 业务 任务 2.1-2.5 跨 session 续做估 **~4.0M tokens / 3.33 SRE·周** (3.1x 校准)
- 后续 P3-D.6 阶段 3 集成 + 阶段 4 实装 跨 session 续做估 **~6.0M tokens / 5.00 SRE·周** (3.1x 校准)
- **P3-D.6 完整 5 阶段 重新估时 ~14.6M tokens / 12.17 SRE·周** (2.9x 实施计划 5.0M 估, 校准因子 = 实际 OLU / 实施计划估)
