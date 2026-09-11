# ADR-0026: Managed Agents Runtime 模式参考 — 借鉴 Multica

> **状态**：🟢 Accepted v0.1（patterns only, no vendor-in）
> **日期**：2026-09-11
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **审批**：架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4 (2026-09-10 12:45 JST 反转) + 9/8 15:19 JST 第 6 次强化 Mavis 全权代理
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md)、[ADR-0025 Vendor Adapter Anti-Contamination](0025-vendor-adapter-anti-contamination.md)
> **关联**：[docs/automation-design.md](../automation-design.md) v0.1+、[STAR-P3-WBS-001.md](../../STAR-P3-WBS-001.md)、`AGENTS.md` §4 守门 26 项 + §4.1 守门派生 v1-v26

---

## 1. 背景与问题

在 `STAR × Mavis` 自建体系下，我们已经搭好了三层基础：Mavis root session + 子代理 dispatch + `scripts/automation/` Python 化（per `docs/automation-design.md` v0.1）。但有两个具体能力点反复被触发讨论却迟迟没系统化落地：

1. **自动识别当前 runtime 里 AI 工具并关联** — Mavis 在哪个 CLI 下启动？本机还有哪些 agent CLI 可用？版本 / auth 状态如何？是否被 vendor 砍？当下 Mavis 只知道自己跑哪个，其他 agent CLI 一无所知 → 不可观测。
2. **自动移动工作流内任务卡** — subagent 完成为什么有时不进 WBS done？`claim` 跟 `start` 是两个动作还是一步？失败 / 重试的状态在哪？当下 WBS 只有 4 态（pending / in_progress / completed / cancelled），跟守门 #9 实证的 "subagent status='succeeded' ≠ 实际成功" 痛点无法对齐。

2026-09-11 16:33 JST 用户提问"是否可以进一步参考 multica"。Mavis 调研 Multica（开源 managed agents 平台，`github.com/multica-ai/multica`，~7.5k stars，Go + Next.js + PostgreSQL 17 + pgvector，本机 daemon + 自托管 server）后，给出 ADR 提案。

### 1.1 Multica 的核心抽象

| 抽象 | 内容 |
|---|---|
| **Runtime = daemon × AI tool × workspace** | 三维正交 — 同台机器 + 同 CLI + 不同 workspace = 多个 runtime |
| **Daemon PATH 扫描** | 启动时跑 13+ adapter 的 `command --version`，成功注册 runtime，失败 / 损坏标 `🔴 poisoned`（不删，标 unusable） |
| **任务状态机** | `enqueue → claim → start → complete / fail`，`claim` 是单独一步（多 subagent 不竞争） |
| **4 类触发** | assign（board 指派）/ @-mention（comment 里 @）/ chat（直接对话）/ autopilot（cron 自动建 issue 并指派） |
| **Skill compounding** | 任务完成方案自动变 reusable skill，跨 subagent 复用 |
| **Self-host + local-first** | daemon 在用户机器，server 只协调状态；代码 / 凭据 / 执行环境全在用户侧 |

### 1.2 我们当前的痛点 ↔ Multica 解法

| 痛点 | Multica 解法 | 我们当前状态 | 缺口 |
|---|---|---|---|
| Runtime 里有哪些 agent CLI | 13+ adapter + version probe | Mavis 知道自己跑哪个，其他一无所知 | 不可观测 |
| 工具 auth / 版本失败 | poisoned 语义（标 unusable 不删） | 看 stderr 才知道，没标 | 不可恢复 |
| Subagent 完成 ≠ 实际成功 | `complete / fail` 显式二态 | `status="succeeded"` 不可靠（守门 #9 实证 10/10 `net::ERR_CONNECTION_CLOSED`） | 不可信 |
| 任务卡在哪个状态 | `enqueue / claim / start / complete/fail` 五态 | `pending / in_progress / completed / cancelled` 四态 | `claim` 缺失 |
| 完成 → 入 main | review gate（人工 / 自动 review 完才进 done） | 直接进 done | 无 review |
| 定期任务触发 | autopilot（cron 自动建 issue + 指派） | `cron_create` 跑完发结果 | 不入工作流 |

---

## 2. 决策

**采用"模式级参考，组件级不引用"路径 — 借鉴 Multica 的模式（状态机 / poisoned 语义 / 4 类触发 / autopilot），自己实装，不 vendor-in Multica 服务端 / UI / DB。**

### 2.1 参考的模式（落地候选 v32 / v33 / v34）

#### 模式 1：Runtime Scan + Poisoned 语义（**v32 候选**）

- `scripts/automation/registry/scan.py` 扫描本机 agent CLI
- 每个 CLI 一个 adapter：探测命令 + 最低版本 + auth 探测（如 `claude auth status`）
- 三档状态：🟢 active（探测 + auth 双过）/ 🟡 stale（探测过但 auth 失效或版本低于最低）/ 🔴 poisoned（探测到但 unusable — auth 永久失效 / 损坏 / 不在 PATH）
- **🔴 poisoned 不删标**，跟守门 #11 "缺标比错标安全" 一致
- automation console 加"环境扫描"页可视化（per 守门 #9 v3 调试控制台扩展）
- 探测失败 → 写 `docs/reports/runtime-scan/<date>.log` 持久化

#### 模式 2：WBS 状态机扩展（**v33 候选**）

- 现有 4 态 → **5 态**：`pending → claimed → in_progress → completed / failed`
- `claimed` 单独存在：subagent 已 claim 但未 start（防多 subagent 竞争，跟守门 #9 v27 RPC fallback 配套）
- `failed` 显式保留，跟 `cancelled` 区分：`failed` = subagent 报 fail（自动）/ `cancelled` = 人工取消（手动）
- **review gate**：subagent `complete` → Mavis `review` → 才进 `done`，跟 Multica 一致
- 落 `scripts/automation/wbs_migrate_v33.py` 自动迁移 WBS 现有 41 子项 status 字段

#### 模式 3：Autopilot 升级（**v34 候选**）

- 现有 `cron_create` 任务升级：先建 WBS row + 指派 subagent + 跑任务 + 收 output
- 等于"自动建任务卡 + 自动指派 + 自动完成 + 自动 review"
- 跟守门 #1 v15 docs 同步饱和机制协调（autopilot 跑完必落档 + 同步 docs）
- 跟 9/1 14:58 JST ask_user 决策流配合（autopilot 启用前必 ask_user 拍板）

### 2.2 不参考的部分（明确不引入）

| 不引入 | 原因 |
|---|---|
| Multica 服务端（Go + PostgreSQL 17） | 跟 Mavis root session 模型冲突；多一层 server / DB 维护成本 |
| Multica UI（Next.js Kanban） | 跟 WBS markdown 模型冲突；负向 |
| Workspace × Tool × Daemon 正交 runtime | 跟 Mavis 单 session 单 CLI 简单模型冲突；过设计 |
| Squad / Leader agent 路由 | 跟 5 域 Lead 真人代签 + Mavis 全权代理冲突；过设计 |
| WebSocket 心跳 + 三档 liveness | session 模型是短连接，不需要 |
| Multi-tenant role matrix | 一人公司 12 角色不需要再细分（per ADR-0021） |
| Multica 自己的 daemon（Go 编译产物） | 跟守门 #6 PowerShell only + 守门 #5 env 安全约束冲突；自装自管 |

### 2.3 跟现有 ADR / 守门的关系

- **继承 ADR-0021 Zero Vendor Cooperation**：Multica 是 vendor，我们只读不集成（不依赖任何 vendor 适配代码）
- **继承 ADR-0025 Vendor Adapter Anti-Contamination**：v32 探测出来的 13+ agent CLI adapter 必须放 `star-optional-*` 子 crate（如未来实装），不进 Core
- **跟 ADR-0024 IDE Session Identity 一致**：每个 session 一个 identity，不多 agent 共用（Multica 正交 runtime 模型不适用）
- **触发守门**：v32 触发守门 #9 v3（调试控制台扩展）+ 守门 #1 v25（cargo 探测模式类比）；v33 触发守门 #9 v27（RPC fallback）+ 守门 #12 v15（docs 同步饱和）；v34 触发守门 #1 v15（docs 同步饱和）

---

## 3. 备选方案与拒绝理由

### 备选 A：vendor-in Multica（自托管 docker compose）
- **拒绝理由**：跟 Mavis root session 模型冲突；多一层 server / DB 维护成本；适配 22 domain 仓架构工作量大；违反 ADR-0021 Zero Vendor Cooperation（虽然 self-host 但仍然依赖 Multica 这个 vendor 的协议）
- **适用场景**：明确需要 Kanban + multi-tenant + 多 tool 跨 workspace 路由的中大型团队

### 备选 B：完全跳过（Multica 模式不引入）
- **拒绝理由**：5 态状态机 / poisoned 语义 / autopilot 三个模式有真实价值，跳过等于放弃这些抽象；守门 #9 实证的 subagent 不可靠痛点没有结构化对策
- **适用场景**：当前 WBS 进度 + 跨子代理竞争不严重的小团队

### 备选 C：只读不写 ADR（口头记录不落档）
- **拒绝理由**：未来重启 session 容易忘掉这些模式可借鉴；不落档等于零成本但零复用；违反守门 #12 v21 [P] docs 同步必更新
- **本 ADR 即反驳此选项** — 落档是必要的

---

## 4. 后果与影响

### 4.1 正面

- 5 态状态机显式化，subagent 竞争 / 失败 / review 三类行为可观测，跟守门 #9 v27 RPC fallback 实证对齐
- Poisoned 语义对齐守门 #11 "缺标比错标安全"（不删标 / 标 🔴 等人工修）
- Autopilot 升级让"定期任务"无缝接入"工作流"，减少 cron 跑完发结果的碎片
- 模式级参考不引依赖，跟 ADR-0021 / 0025 一致

### 4.2 成本

- v32 / v33 / v34 三个候选实装预估 2-3 session（Mavis 自驱 per 9/8 15:29 JST 第 7 次强化）
- automation console 加"环境扫描"页 = 1 前端页面 + 1 Python 后端（per 守门 #9 v3 调试控制台扩展）
- 状态机迁移需要 WBS 现有 41 子项 status 字段升级（`pending` → `pending / claimed / in_progress`），文档同步
- v34 autopilot 启用前必 ask_user 拍板（per 9/1 14:58 守门）

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 5 态状态机迁移 WBS 现有数据漏改 | 中 | 中 | 落 `scripts/automation/wbs_migrate_v33.py` 自动改 + `registry_check.py` 校验 |
| Poisoned 标了之后没人修 | 低 | 低 | automation console 红 banner + `docs/reports/poisoned-weekly.md` 周报 |
| v34 autopilot 跑挂影响主流程 | 中 | 中 | 跑前 ask_user 拍板"启用"+ 守门 #1 v15 docs 同步饱和机制 |
| Multica 模式被未来 reader 误读为"vendor-in 推荐" | 低 | 中 | 本 ADR §2.2 显式列"不参考" + 标题明示 patterns only |

---

## 5. 后续落地候选（v32 / v33 / v34，per 守门 v28 + 9/1 14:58 + 9/5 04:03）

| 候选 | 内容 | 触发 / 关联 | 落点 |
|---|---|---|---|
| **v32** Runtime scan + poisoned 语义 | 探测本机 agent CLI + 标 poisoned；automation console 加"环境扫描"页 | 守门 #9 v3 调试控制台 + 守门 #1 v25 cargo 探测模式 | `scripts/automation/registry/scan.py` + console 新页 |
| **v33** WBS 状态机扩 `claimed` / `failed` + review gate | 显式 5 态流转；subagent complete → Mavis review → done | 守门 #9 v27 RPC fallback + 守门 #12 v15 docs 同步饱和 | `docs/STAR-P3-WBS-001.md` 状态定义 + `scripts/automation/wbs_migrate_v33.py` |
| **v34** Autopilot 升级（cron 自动建卡 + 指派） | 现有 cron 任务加"先建 WBS row 再指派 subagent" | 守门 #1 v15 docs 同步 + 9/1 14:58 ask_user 决策流 | `scripts/automation/cron_with_card.py` |

**激活门槛**：
- (a) Mavis 终端必走 `ask_user` 2-4 选项 + 至少 1 个 `(推荐)` 标（per 守门 v28）
- (b) 推荐项放第 1 位
- (c) 拍板后立即执行（per 9/5 04:03）
- (d) commit author = `Ulysses <ulysses@mavis.local>`（per 守门 #1 + #10）
- (e) 落档后同步修 `docs/automation-design.md` §4 任务卡表 + `scripts/automation/registry.md`（per 守门 #12 v21）
- (f) 跨 session 续做：governance-level 决策不阻塞任何 P3-B-F / H2 / ARG / P0-2/3/4 跨 session 续做项（per 守门 #14 v3 永久代签 policy）

---

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受（patterns only, no vendor-in）per 2026-09-11 16:35 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（只搬模式 + 列出不搬部分 + v32-v34 落地候选 + 5 角色签字栏） | 2026-09-11 16:33 JST Ulysses 提问"是否可以进一步参考 multica" + ask_user 选项 path_opt1（只搬模式，自己实装）+ scope_opt4（只写设计文档 / ADR，不实装） |
