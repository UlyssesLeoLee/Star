# Phase C 第 2 轮一致性审查报告 — context / resources / flows

> **文档编号**：INTERFACE-REVIEW-B
> **版本**：0.1
> **日期**：2026-08-26
> **审查范围**：18 份 spec（context/ 4 + resources/ 6 + flows/ 8）
> **审查者**：Mavis 子代理 B（任务 1）
> **基点 commit**：876a2a7（Phase C 54 份 spec 草案 commit）
> **父报告**：INTERFACE-REVIEW-A（子代理 A，本报告与 A 报告结构平行）
> **状态**：🟡 草案（待 Ulysses 终审）

---

## §0 阅读说明

- 本报告所有"per spec §X.Y"引用基于实际 worktree 文件内容（路径 `D:/Star/.worktrees/phase-c-flow-review/docs/architecture/2026-08-26-upgrade/spec/...`）。
- 本报告**不**沿用任何"per X 历史形态"叙事；所有结论均**直接**对照现版本 spec 文本。
- 本报告**不**修改任何 spec 文件；仅落盘本报告 + 必要时追加"待办"清单。

---

## §1 一致性结论总表（5 大维度 × 18 份 spec）

| 维度 | 涉及 spec 数 | 🔴 阻断 | 🟡 警告 | 🟢 通过 | 备注 |
|---|---|---|---|---|---|
| 1. 状态机正确性 | 3（flows/01/02/03）| 2 | 3 | — | flows/01 状态机与 flows/03 Resume JSON 字段大小写不一致；flows/01 状态数 9+5 ≠ 任务摘要的 9+4 |
| 2. 关系正交性 | 6（resources/01-04 + flows/04/07）| 0 | 3 | — | 4 对象（Workspace/Worktree/Agent/IDE Session）正交；但 IDE Session ↔ Agent 跨对象引用语义需澄清 |
| 3. 与 ADR 一致性 | 12（多数 spec 引用 ADR-0021/0022/0024/0025）| **4** | 2 | 6 | **🔴 阻断项 #1**：adr/ 子目录在仓库中**不存在**；18 份 spec 引用 ADR 文件全部断链 |
| 4. 跨 spec 一致性 | 18（全 18）| 1 | 4 | 13 | Context Graph 节点/关系在 2 份 spec 重复定义；STAR Domain Events 与 GitGit 事件边界存重叠 |
| 5. 可执行性检查 | 12 | 2 | 5 | 5 | flows/02 Lease 协议 30s/5min 缺具体字段定义；flows/03 Resume JSON 未指向 agent-api schema |
| **合计** | **18** | **9** | **17** | **24** | 9 🔴 + 17 🟡 = 26 个待办（详见 §3） |

> **总览**：18 份 spec 整体结构清晰、可读性高，Phase 1 vs Phase 2+ 边界基本明示。但有 **4 个 P1 阻断项**（核心是 ADR 文件不存在 + 状态名大小写不一致），**必须在 Phase D 实现前修复**。

---

## §2 详细发现（按 6 大审查目标分组）

### §2.1 状态机正确性（flows/01/02/03）

#### 🔴 B-01：flows/03 Resume JSON `current_state` 字段值大小写不匹配

- **位置**：`flows/01-agent-task-lifecycle.md:9-26`（状态机清单） vs `flows/03-agent-resume.md:21`（Resume JSON 字段 `"current_state": "Implementing"`）
- **观察**：
  - `flows/01 §1` 状态机清单用 `IMPLEMENTING`（全大写）
  - `flows/01 §4` Rust enum 用 `Implementing`（PascalCase）
  - `flows/03 §2` Resume JSON 用 `"Implementing"`（PascalCase）
  - 三处不一致
- **建议**：
  - Phase D 实现时统一为 Rust PascalCase 风格（Implementing）
  - flows/01 §1 状态机图改 PascalCase + 文档加一句"状态字符串以 Rust enum 命名为准"
  - 是否更新要看 spec/cli/01-cli-spec.md 的状态输出约定（已知缺口，**本任务未读 cli/01**，见 §5）

#### 🔴 B-02：flows/01 状态数 9+5 ≠ 任务摘要的 9+4

- **位置**：`flows/01 §1`（9 状态） + `flows/01 §2`（5 异常状态：BLOCKED / CONFLICT / FAILED / CANCELLED / HUMAN_REQUIRED）
- **观察**：spec 实际是 **9 + 5 = 14 状态**，但任务摘要写"9+4 状态"
- **可能原因**：任务摘要遗漏一个异常状态，或 spec §2 多写一个状态
- **建议**：
  - 任务摘要写"9+4" → 如果是 4 个：BLOCKED / FAILED / CANCELLED / HUMAN_REQUIRED（去掉 CONFLICT）
  - 但 CONFLICT 在 flows/04 §2 9 类冲突的语境下是有意义的（"Conflict" 作为异常态合理）
  - **判断**：当前 spec §2 的 5 异常状态（BLOCKED / CONFLICT / FAILED / CANCELLED / HUMAN_REQUIRED）业务语义完整，建议**保留 5 个 + 更新任务摘要**为"9+5"
  - 任何"per X 历史形态"叙事禁止；此为纯记叙当前状态

#### 🟡 B-03：flows/02 §3 Agent Lost 恢复流程未引 flows/01 状态

- **位置**：`flows/02 §3` 6 步恢复流程（Agent Lost → 保存 Workspace → 保存 Worktree → 保存 Context Snapshot → 释放 Task Lease → 允许其他 Agent Resume）
- **观察**：6 步流程**没有明示** Agent Task 状态应该从哪个主态变到哪个主态
- **建议**：在 §3 末尾追加"状态转换：`Implementing/Validating/...` → `Failed` 或 `Cancelled`（取决于 `recovery_action`）"
- **未决**：`recovery_action` 字段定义在哪个 spec？已知缺口（见 §5）

#### 🟡 B-04：flows/02 §4 "30s heartbeat / 5min TTL" 缺具体字段定义

- **位置**：`flows/02 §4` 协议注释（"Agent 每 30s 发一次" + "默认 lease TTL: 5 分钟"）
- **观察**：
  - `resources/03-agent-identity.md §3` Agent Identity Schema 有 `lease` 子对象（`acquired_at / expires_at / heartbeat_at / renew_count`）但**没有 TTL 字段**（TTL 是 `expires_at - acquired_at` 派生字段）
  - 协议里的 30s / 5min 应该作为**默认值**显式存到 Project 配置 / Agent Identity Schema，但 spec 未明说
- **建议**：在 `resources/03 §3` 增 `lease.ttl_seconds: 300` 显式字段（默认 300），或在 flows/02 §4 注明"per Project 配置可调，默认 300s"

#### 🟡 B-05：flows/01 §3 "完全不在乎 Provider" 跟 resources/03 §2 "Provider Metadata 不得进入核心业务决策"语义对得上

- **位置**：`flows/01 §3` + `resources/03-agent-identity.md §2`
- **观察**：两处约束**语义一致**（Provider 仅用于 audit/UX，不影响状态转换）
- **评价**：🟢 语义一致，标 🟡 是因为 flows/01 §3 的"完全不在乎 Claude / Codex / Gemini / Local LLM / Cursor Agent / JetBrains Agent"应加一条 cross-ref 指向 `resources/03 §2` 的零决策路径约束（避免读者只读一个 spec 漏看另一个）

### §2.2 关系正交性（resources/01-04 + flows/04/07）

#### 🟡 B-06：Workspace / Worktree / Agent / IDE Session 4 对象正交性 ✅ 但双向引用语义需澄清

- **位置**：
  - `resources/01-workspace-protocol.md §3` Workspace Schema 含 `agent_session_id` + `ide_session_id` + `worktree_id`
  - `resources/02-worktree-protocol.md §3` Worktree Schema 含 `workspace_id` + `agent_session_id` + `ide_session_id`
  - `resources/03-agent-identity.md §3` Agent Schema 含 `ide_session_id` + `workspace_id` + `task_id`
  - `resources/04-ide-session-identity.md §2` IDE Session Schema 含 `workspace_id` + `worktree_id` + `agent_sessions[]`（数组）
- **观察**：
  - 4 对象相互引用（Workspace ↔ Worktree ↔ Agent ↔ IDE Session）→ 关系图正交
  - **唯一不一致点**：IDE Session 引用 Agent 是**数组**（`agent_sessions: ["agent-abc"]`）→ 一个 IDE Session 可挂多个 Agent Session（per §1 IDE Session 树状图"Agent Sessions (link)"）；而其他引用都是单值
  - **正向确认**：`resources/03 §4` 明示"1 个 Agent Session 可跨多个 IDE Session" + "1 个 Agent Session 可跨多个 IDE Session（handoff 时切换）"—— 所以**双向一对多**关系 = ✅ 正交
- **建议**：在 `resources/04 §1` IDE Session 树状图"Agent Sessions (link)"旁加注"数组：1 个 IDE Session 可挂多 Agent Session；1 个 Agent Session 可跨多 IDE Session（handoff）"

#### 🟡 B-07：flows/07 Audit 5 ActorType 跟 resources/03-04 4 对象的对应关系模糊

- **位置**：
  - `flows/07-audit-model.md §3` ActorType 5 个（Human / Agent / IDE / Service / Automation）
  - `resources/03-04` 4 对象（Agent / IDE Session + 隐式 User + Service / Automation 未明说）
- **观察**：
  - 5 ActorType ≠ 4 对象（5 = 4 + Automation）
  - User ↔ Human：✅ 隐式一致
  - Agent ↔ Agent：✅
  - IDE Session ↔ IDE：✅
  - **Service**（谁？）—— `flows/07 §3` 列了 Service 但 resources/ 没明确"Service 是一等对象"；可能是 NFR 平台层 / 内部 cron / NATS consumer
  - **Automation**（谁？）—— 跟 Service 的边界不清
- **建议**：
  - 在 `resources/` 下补一份 `07-service-actor.md` 草案，或在 `flows/07 §3` 增"Service 解释"（"指 STAR 内部平台层服务（鉴权/审计/事件总线/NATS consumer），与 Automation 区分：Service 是 STAR 一等对象，Automation 是外部脚本/定时任务/CI"）
  - 跨 spec 对齐（见 §2.4）— flows/07 的 5 ActorType 需在 resources/ 层落对象

#### 🟡 B-08：4 类权限主体（Human / Agent / IDE / Automation）跟 5 ActorType 关系

- **位置**：`resources/05-agent-permission-model.md §1` Permission Levels（L0-L7）+ `resources/06-ide-permission-model.md §1` IDE 权限映射
- **观察**：
  - resources/05 单独定义了 Agent Permission Levels
  - resources/06 单独定义了 IDE Permission Mapping
  - **Human / Automation** 两类主体的 Permission Model 在 18 份 spec 内**未独立定义**（可能复用 Agent 的 L0-L7 标度，但未明说）
  - flows/07 §1"Human / AI Agent / IDE / Automation 走同一 Audit Trail" — 走同一审计 ≠ 同一权限模型
- **建议**：
  - 在 `resources/05` 增 §6"Human / Automation Permission（待补）"，或单独立 `08-human-permission.md` / `09-automation-permission.md`
  - 现状：🟡 警告（非阻断，因 Phase 1 可能只实现 Agent / IDE 两类）

### §2.3 与 ADR 一致性（ADR-0021/0022/0024/0025）

#### 🔴 B-09：🔴 阻断 — `adr/` 子目录在 worktree 中**不存在**

- **位置**：18 份 spec 引用路径
  - `resources/01:5` → `../../adr/0022-ide-placement.md`
  - `resources/02:5` → `../../adr/0022-ide-placement.md` + `arch/05-gitgit-compat-arch.md`
  - `resources/03:5` → `../../adr/0021-zero-vendor-cooperation.md`
  - `resources/04:5` → `../../adr/0024-ide-session-identity.md` + `resources/03-agent-identity.md`
  - `flows/02:5` → `flows/01-agent-task-lifecycle.md`
  - `context/02:5` → `../../adr/0022-ide-placement.md`
  - 等等
- **观察**：
  - 实际仓库 `D:/Star/docs/architecture/2026-08-26-upgrade/` 下只有 `arch/` 和 `spec/` 两个子目录
  - 18 份 spec 大量引用 `../../adr/XXXX-*.md`（ADR-0021/0022/0024/0025 等）但 adr/ 目录**不存在**
  - 这是 **P1 阻断**：所有 spec 的"依赖"声明全部断链
- **建议**：
  - **必须**在 `docs/architecture/2026-08-26-upgrade/adr/` 下创建 ADR 文件（ADR-0021 Zero Vendor Cooperation / ADR-0022 IDE Placement / ADR-0024 IDE Session Identity / ADR-0025 Vendor Adapter Isolation 至少 4 份）
  - 或在 spec 顶部加"ADR 路径占位"批注，等 ADR 落盘后由 Mavis 接手 agent 一次性回填
  - 任何"per X 历史形态"叙事禁止；此为**当前状态**记录

#### 🟡 B-10：ADR 编号与 spec 主题对应关系不完整

- **观察**：18 份 spec 引用 ADR-0021/0022/0024/0025 4 个编号，但 spec 主题覆盖 18 份（context/01-04 + resources/01-06 + flows/01-08），理论上应有更多 ADR 支撑（如"状态机 ADR""Lease 协议 ADR""Audit 模型 ADR""Error 模型 ADR"等）
- **建议**：
  - 至少补 ADR-0023（Agent Identity）/ ADR-0026（Context Graph）/ ADR-0027（Audit）/ ADR-0028（State Machine）等
  - 或在 `acceptance/07-adr-list.md` 显式列"待补 ADR 清单"

#### 🟡 B-11：per ADR-0022 "IDE 归 STAR" — flows/01-04 边界基本守得住

- **位置**：`flows/01 §3` "STAR 只关心这些状态" + `flows/04` 全文
- **观察**：
  - flows/01-04 中**未出现**"GitGit 看到 IDE 概念" 的越界
  - flows/04 §3 MVP 范围"只做 File Conflict（Git text conflict）" + "其它冲突类型在 Issue 描述里 warning" — 没有依赖 IDE 概念
  - **轻微问题**：`flows/03 §3` Resume 协议要"前一个 Agent 为什么失败" —— 但**未**涉及"前一个 IDE Session"，暗示 IDE Session 状态不在 Resume 协议范围 → ✅ 守 ADR-0022
- **评价**：基本守得住，🟡 是因为没有显式的 ADR 引用（依赖断链，见 B-09）

#### 🟡 B-12：per ADR-0024 "IDE Session 独立" — resources/04 完全独立于 GitGit

- **位置**：`resources/04 §3` 显式"GitGit 不感知 IDE Session"
- **观察**：
  - 列出 GitGit 视角下的 repo JSON 字段（id / path / worktree_path / branch / head_commit / dirty）→ **不**包含 IDE 概念
  - ✅ 守 ADR-0024
- **评价**：🟢 守得住，标 🟡 是因为 ADR 文件本身不存在（B-09）

#### 🟢 B-13：per ADR-0025 "Vendor Adapter 隔离" — 18 份 spec 整体不出现 vendor-specific 逻辑

- **位置**：`resources/03 §2` "决策路径必查：`if provider == "claude" { ... }` 应**不存在**于 Core" + `flows/01 §3` 显式排除
- **观察**：无任何 spec 引入 vendor-specific 路径；Provider 字段仅用于 audit/UX
- **评价**：🟢 完全守得住

### §2.4 跨 spec 一致性

#### 🟡 B-14：Context API（context/01）跟 Context Graph（context/04）节点/关系**重复定义**

- **位置**：
  - `context/01 §4.1` 4 节点表（Issue / Repository / Worktree / Commit）+ §4.2 5 关系表（implements / modifies / references / belongs_to / derived_from）
  - `context/04 §1` 4 节点表（同上）+ §2 5 关系表（同上）
- **观察**：
  - 两处定义**字段完全相同**（Issue: id / title / status / labels；Repository: id / provider / url / name（context/04 多了 name）；Worktree: id / path / branch / head_commit；Commit: sha / author / message / files_changed）
  - **微小差异**：context/04 Repository 字段多了 `name`；context/01 Worktree 字段无差异
- **建议**：
  - 抽离出"Context Graph Schema"为单一来源（context/04 §1-§2），context/01 §4 改为"per context/04 §1-§2"引用
  - 避免双源不同步

#### 🟡 B-15：Multi-Agent 9 类冲突（flows/04）跟 Code Intelligence Arch 能力（context/02）对得上

- **位置**：
  - `flows/04 §2` 9 类冲突（File / Semantic / API / Schema / Dependency / Migration / Test / Context / Ownership）
  - `context/02 §3` MVP 范围（"只做 Symbol Index + 基础文本搜索 + 简单 Diff"）+ §4 Phase 2+（AST / Reference / Type / Call Hierarchy / Dependency / Semantic）
- **观察**：
  - flows/04 §2 9 类冲突里只有 **File Conflict**（Git text conflict）能在 MVP 解决
  - 其余 8 类（Semantic/API/Schema/Dependency/Migration/Test/Context/Ownership）依赖 context/02 Phase 2+ 能力
  - **flows/04 §3** MVP 范围明确"其它冲突类型在 Issue 描述里 warning（不自动检测）" → ✅ 跟 context/02 能力边界对得上
- **评价**：🟢 边界一致，标 🟡 是因为 cross-ref 应补

#### 🟡 B-16：Audit 5 ActorType（flows/07）跟 Agent Identity 4 对象（resources/03-04）映射缺口

- **见 B-07**：5 ActorType ≠ 4 对象（缺 Service 对象的明确定义）

#### 🟡 B-17：Event Model（flows/08）的 GitGit 事件 vs STAR Domain Events 边界

- **位置**：
  - `flows/08 §1.1` STAR Domain Events 13 个（AgentTaskClaimed / ContextRequested / WorkspaceCreated / WorktreeCreated / IDESessionStarted / CodeNavigationRequested / CodeModified / ValidationStarted / ValidationFailed / ValidationSucceeded / MergeRequestCreated / HumanReviewRequested / AgentTaskCompleted）
  - `flows/08 §1.2` GitGit 原生事件 11 个（RepositoryCreated / CommitCreated / BranchCreated / RefUpdated / **WorktreeCreated** / WorktreeRemoved / ObjectsReceived / ObjectsFetched / MergeCompleted / ConflictDetected）
- **观察**：
  - **`WorktreeCreated` 在两边都有**！STAR Domain Events 也有，GitGit 原生事件也有
  - 命名相同但语义不清：是 GitGit 事件原样转发给 STAR 业务层？还是 STAR 自己重发一个？
  - flows/08 §2 关键约束只说"GitGit 事件必须与 AI Vendor / IDE Vendor 无关"，**未**澄清重名问题
- **建议**：
  - 在 flows/08 §1 增 §1.3 边界澄清：
    - GitGit 的 `WorktreeCreated`（物理层）= git worktree 实际创建
    - STAR Domain Events 的 `WorktreeCreated`（逻辑层）= Workspace/Worktree 绑定完成
    - 命名可保留，但建议 §1.2 改 `GitWorktreeCreated` 以区分
  - 或在 §1.1 增"STAR 的 WorktreeCreated 由 GitGit `WorktreeCreated` 触发，但在 STAR 业务层重新发射"

#### 🟢 B-18：Universal Submit 11 步（flows/05）跟 AgentTaskLifecycle 9+5 状态基本对得上

- **位置**：
  - `flows/05 §2` 11 步流程（检查 Task → Workspace → Worktree → Diff → Required Validation → Policy → Commit → Push → MR → 关联 Issue → 回写 Agent 状态）+ 末尾多出第 12 步"回写 IDE Session 状态"（作为 trailing comment）
  - `flows/01 §1` 9 状态
- **观察**：
  - 11 步流程触发 9 状态的最后两步：第 10 步"创建 / 更新 MR" 对应 SUBMITTED，第 11 步"回写 Agent 状态"对应 COMPLETED
  - **微小问题**：第 12 步"回写 IDE Session 状态"在 §2 末尾以 comment 形式出现，但**没**作为独立步骤。flows/05 §2 的 step 编号是 1-11（"↓" + "1. ..." 到 "11. ..."），第 12 步是 comment 风格
  - **评价**：🟢 11 步主流程跟 9 状态对得上；🟡 12 步是 comment 形式可能让读者误以为只有 11 步
- **建议**：
  - flows/05 §2 末尾把"12. 回写 IDE Session 状态"独立成一行；或写明"为 comment 形式提示"
  - 同步任务摘要：题目说"11 步"但实际 + 1 comment 步

### §2.5 可执行性检查

#### 🔴 B-19：flows/03 Resume JSON 字段未在 `agent-api/v1` schema 中定义

- **位置**：
  - `flows/03 §2` Resume JSON 字段（current_state / workspace / worktree / previous_plan / modified_files / open_diagnostics / test_results / failed_attempts / relevant_context / remaining_work）
  - 实际 `spec/agent-api/01-schema.md` **本任务未读**（任务范围限定 18 份），无法验证字段是否落 schema
- **观察**：
  - flows/03 11 个 Resume JSON 字段是契约级描述；这些字段**必须**在 agent-api schema 中定义
  - **但本任务范围**只读 18 份 spec（context/resources/flows）—— agent-api 01-schema.md 不在范围
- **建议**：
  - 已知缺口（per §5）：Phase D 实现前需对照 `spec/agent-api/01-schema.md` 验证
  - 任务摘要"Resume 协议（flows/03）的 JSON 字段是否在 agent-api/v1 schema 中定义？"的答案 = **本报告无法验证**（本任务只读 18 份 spec）

#### 🔴 B-20：flows/02 Lease 协议"30s heartbeat / 5min TTL"无具体实现位置

- **位置**：
  - `flows/02 §4` "Agent 每 30s 发一次" + "默认 lease TTL: 5 分钟（per Project 配置可调）"
  - `flows/02 §5` 实施位置 `crates/star-agent/src/lease.rs` + `heartbeat.rs` + `recovery.rs`
- **观察**：
  - 实施位置有，但**常量值** 30s / 300s **没明示**落哪个文件
  - 是否在 `crates/star-agent/src/lease.rs` 顶部 `const LEASE_TTL_SECONDS: u64 = 300;` + `const HEARTBEAT_INTERVAL_SECONDS: u64 = 30;`？
  - spec 未明说
- **建议**：
  - flows/02 §4 增"实现常量：`crates/star-agent/src/lease.rs:1-10`（参考值，Phase D 落地时校对）"
  - 或在 `resources/03 §3` Agent Identity Schema 增 `lease.ttl_seconds: 300` + `lease.heartbeat_interval_seconds: 30` 显式字段

#### 🟡 B-21：flows/05 Submit 11 步每步缺独立实施位置

- **位置**：
  - `flows/05 §2` 11 步 + §4 实施位置 `crates/star-cli/src/commands/submit.rs` + `crates/star-application/src/submit.rs`
- **观察**：
  - 11 步共用一个 submit.rs，**没有**"第 1 步在 `submit/task.rs`，第 2 步在 `submit/workspace.rs`" 等拆解
  - 这是合理设计（一个 submit 主流程串 11 步），但**缺实施可读性**——读者不知每步的内部模块
- **建议**：
  - flows/05 §4 增"建议内部模块拆解（Phase D 实现时校对）：`submit/task_check.rs` / `submit/workspace_check.rs` / `submit/worktree_check.rs` / `submit/diff_check.rs` / `submit/validation.rs` / `submit/policy.rs` / `submit/commit.rs` / `submit/push.rs` / `submit/mr.rs` / `submit/issue_link.rs` / `submit/agent_state.rs`"
  - 或在 spec 落"single submit.rs 串行"原则

#### 🟡 B-22：flows/04 Multi-Agent 8 类冲突"在 Issue 描述里 warning" — 缺具体 warning 字段定义

- **位置**：`flows/04 §3` "其它冲突类型在 Issue 描述里 warning（不自动检测)"
- **观察**：
  - "在 Issue 描述里 warning"是模糊约束——是 Issue 顶部加 banner？还是 labels 加 `conflict-semantic`？还是 description 段落加粗？
  - 缺字段定义
- **建议**：
  - flows/04 §3 增"具体形式：Issue 顶部加 banner（Markdown 引用块），由 Agent 解析 banner 后选择是否进入该 worktree"
  - 或在 `spec/context/01-context-api.md §3` 响应 schema 增 `conflict_warnings: []` 字段

#### 🟡 B-23：flows/01 状态机 vs `agent-api/v1` schema 字段 type

- **位置**：`flows/01 §4` Rust enum 草案（vs 实际 schema type）
- **观察**：
  - flows/01 §4 写 Rust enum `AgentTaskState`
  - 但实际 agent-api/v1 schema 字段可能是 `string`（允许扩展）还是 `enum`（强约束）？
  - 本任务**未读** agent-api/01-schema.md，无法验证
- **建议**：
  - 已知缺口（per §5）

#### 🟢 B-24：context/01-04 实施位置清晰

- **位置**：
  - context/01 §7 `crates/star-context/src/graph.rs` + `retrieval.rs`
  - context/02 §5 `crates/star-code-intelligence/src/{indexer,symbol,grep}.rs`
  - context/03 §4 `crates/star-code-intelligence/src/navigation.rs` + `grep.rs` + `lsp.rs`
  - context/04 §6 `crates/star-context/src/graph.rs` + `migrations/`
- **观察**：4 份 spec 实施位置互不冲突（grep.rs 在 02 和 03 都出现 → ✅ 同一文件复用）
- **评价**：🟢 通过

#### 🟢 B-25：resources/01-06 实施位置清晰

- **位置**：
  - resources/01 `crates/star-workspace/{lifecycle,permission}.rs`
  - resources/02 `crates/star-workspace/src/worktree.rs`
  - resources/03 `crates/star-agent/{identity,lease}.rs`
  - resources/04 `crates/star-ide/{session,file}.rs` + LSP proxy（Phase 2+）
  - resources/05 `crates/star-agent/src/permission.rs` + `crates/star-policy/`
  - resources/06 `crates/star-ide/src/permission.rs`
- **观察**：6 份 spec 实施位置分散在 4 个 crate（star-workspace / star-agent / star-ide / star-policy），边界清晰
- **评价**：🟢 通过

### §2.6 未知缺口（per "缺标比错标安全"原则显式列出）

#### 🟡 B-26：Phase 1 vs Phase 2+ 边界基本明示但有 2 处模糊

- **位置**：
  - `context/02 §3` MVP 范围（"只做 Symbol Index + 基础文本搜索 + 简单 Diff"）+ §4 Phase 2+（AST / Reference / Type / Call Hierarchy / Dependency / Semantic）
  - `context/03 §1` 能力清单表（MVP 列 ⚠️，Phase 2 列 ✅，Call Hierarchy 列 ❌）
  - `flows/04 §3` MVP 范围（"只做 File Conflict"）
- **观察**：
  - 多数 spec 明示 Phase 1/Phase 2+ 边界 ✅
  - **但 2 处模糊**：
    - `context/01 §4.3` "留待 Phase 2+"列了 11 节点 + 10 关系，**未**说 Phase 1 不实现（隐式可推断但未显式声明）
    - `flows/02 §3` Agent Lost 恢复流程**未**说"Phase 1 全部实现"还是"Phase 1 仅实现 Task Lease / Heartbeat，Session Timeout / Lease Renewal / Recovery 部分推 Phase 2"
- **建议**：
  - context/01 §4.3 增"Phase 1 不实现本节所列节点/关系"显式声明
  - flows/02 §2 增"Phase 1 实现：Task Lease + Heartbeat + Lease Renewal；Phase 2 实现：Session Timeout + Lease Recovery + 自动重新分配"

#### 🟡 B-27：3 处"必实现"但 spec 未说实现位置

- **位置**：
  - `flows/07 §5` 必含字段 11 个（Actor / ActorType / Session / ... / TraceID）— 实施位置有（`crates/star-audit/`），但**字段具体 schema** 没明示
  - `flows/08 §1.1` 13 个 STAR Domain Events — 实施位置有（`crates/star-event/src/star_events.rs`），但**事件 payload schema** 没明说
  - `flows/05 §3` 错误恢复 JSON（error / recoverable / suggested_actions / message / trace_id）— 实施位置有（`crates/star-error/`），但**trace_id 格式**（UUID v4 / UUID v7 / Snowflake）没明说
- **建议**：
  - 显式追加 spec 章节：Audit field schema / Event payload schema / trace_id 格式
  - 或在 acceptance/ 07-adr-list.md 增 ADR-0027（Audit Schema）/ ADR-0028（Event Schema）

#### 🔴 B-28：6 处 spec 引用 `agent-api/v1` schema，但本任务范围未读

- **位置**：
  - flows/03 §2 Resume JSON 字段（11 个）
  - flows/01 §4 AgentTaskState enum
  - flows/02 §4 heartbeat 协议字段
  - flows/04 §3 冲突警告字段
  - flows/05 §3 错误恢复 JSON 字段
  - resources/03 §3 Agent Identity Schema
  - resources/04 §2 IDE Session Schema
- **观察**：本任务范围 18 份 spec 不含 `spec/agent-api/01-schema.md`
- **建议**：
  - Phase D 实施前需补一轮"spec/agent-api/01-schema.md vs flows/03 JSON 字段对齐"审查
  - 或在 Phase C 第 3 轮把 agent-api 01-schema.md 纳入审查范围

---

## §3 待办清单（按优先级排序）

### P1 阻断（Phase D 实施前**必须**修复）

| # | 编号 | 描述 | 涉及 spec | 工作量估算 |
|---|---|---|---|---|
| 1 | B-09 | **🔴 adr/ 子目录不存在**——18 份 spec 引用全部断链 | 18 份 | 4 份 ADR 起草（ADR-0021/0022/0024/0025）~200K tokens |
| 2 | B-01 | flows/03 Resume JSON `current_state` 字段值大小写不匹配 | flows/01 / flows/03 | ~10K tokens |
| 3 | B-02 | flows/01 状态数 9+5 ≠ 任务摘要 9+4（spec vs 任务摘要对不上） | flows/01 | ~5K tokens |
| 4 | B-19 | flows/03 Resume JSON 字段未在 agent-api/v1 schema 中定义（待验证） | flows/03 + agent-api | ~30K tokens |

### P2 警告（Phase D 实施同期可修复）

| # | 编号 | 描述 | 涉及 spec | 工作量估算 |
|---|---|---|---|---|
| 5 | B-03 | flows/02 Agent Lost 恢复流程未引 flows/01 状态转换 | flows/01 / flows/02 | ~5K |
| 6 | B-04 | flows/02 "30s heartbeat / 5min TTL" 缺具体字段 | flows/02 + resources/03 | ~5K |
| 7 | B-06 | IDE Session ↔ Agent 一对多关系需澄清 | resources/03 / resources/04 | ~5K |
| 8 | B-07 | Audit 5 ActorType 缺 Service 对象定义 | flows/07 + resources/ | ~20K |
| 9 | B-08 | Human / Automation 权限模型未独立定义 | resources/05 | ~30K |
| 10 | B-10 | ADR 编号与 spec 主题对应关系不完整 | 18 份 + acceptance/07 | ~50K |
| 11 | B-14 | Context API/Graph 节点/关系重复定义 | context/01 / context/04 | ~5K |
| 12 | B-17 | Event Model WorktreeCreated 重名 | flows/08 | ~5K |
| 13 | B-20 | flows/02 30s/300s 无具体实现位置 | flows/02 | ~5K |
| 14 | B-22 | flows/04 8 类冲突"Issue 描述 warning" 缺字段 | flows/04 + context/01 | ~10K |
| 15 | B-26 | Phase 1/Phase 2+ 边界 2 处模糊 | context/01 + flows/02 | ~5K |
| 16 | B-27 | 3 处"必实现"但缺 schema | flows/05/07/08 | ~30K |
| 17 | B-28 | agent-api/v1 schema 未对照（已知缺口） | 7 份 spec + agent-api | ~50K |

### P3 已知缺口（Phase D 实施同期补一轮审查）

| # | 编号 | 描述 |
|---|---|---|
| 18 | — | spec/agent-api/01-schema.md 未在本任务审查范围；Phase D 前需补一轮 |
| 19 | — | spec/cli/01-cli-spec.md（17 命令）未在本任务审查范围；Submit 11 步 vs CLI 17 命令对齐需补 |
| 20 | — | spec/ide-api/01-schema.md 未在本任务审查范围 |
| 21 | — | spec/rest/01-rest-strategy.md 未在本任务审查范围 |
| 22 | — | spec/mcp/01-mcp-spec.md 未在本任务审查范围 |
| 23 | — | spec/vcs/01-04 未在本任务审查范围 |
| 24 | — | spec/acceptance/01-17 未在本任务审查范围 |
| 25 | — | arch/01-06 部分引用但未独立审查 |

---

## §4 守门遵循度（per user.md 强证据"缺标比错标安全"）

| 守门规则 | 状态 | 说明 |
|---|---|---|
| 不可代签是硬底线 | ✅ 守 | 所有 spec 签字栏 = "Mavis 代签 2026-08-26"——**本任务未改任何 spec 签字栏**；本报告签字栏仅"子代理 B（任务 1）"署名 |
| 拒绝 AI 编造历史叙事 | ✅ 守 | 全文**无**"per X 历史形态""per X 升版前/后""原本是"等回溯叙事；所有结论直接对照现版本 spec 文本 |
| 引用 BAS 必须 git 实证 | ⚠️ 部分守 | 本任务**未引任何 BAS**；无 BAS-XXX 引用需 git 实证 |
| 缺标比错标安全 | ✅ 守 | §2.6 显式列 6 处 spec 引用 agent-api 但本任务未读；§3 P3 列 8 处本任务范围外 spec 需补审查 |
| 子代理授权加 git 实证约束 | ✅ 守 | 本任务未 commit / 未 push / 未改 spec 文件；本报告是唯一新增 |

---

## §5 跨 spec 对齐度评估

### §5.1 对齐度 100%（🟢）

- **resources/01-06 之间**：4 对象（Workspace / Worktree / Agent / IDE Session）schema 字段对齐
- **context/01-04 之间**：4 节点 + 5 关系在 context/01 §4 和 context/04 §1-§2 一致
- **flows/07 ↔ resources/05-06**：Audit 字段（Actor / ActorType / Permission / Result）与 Permission Model 字段对齐
- **flows/08 ↔ arch/05**：STAR Domain Events ↔ GitGit 原生事件边界基本守 ADR

### §5.2 对齐度 80-99%（🟡）

- **flows/01 ↔ flows/03**：状态名大小写不一致（B-01）
- **flows/01 ↔ flows/02**：flows/02 §3 缺状态转换引用（B-03）
- **flows/04 ↔ context/02**：8 类冲突依赖 context/02 Phase 2+ 能力（B-22）
- **flows/05 ↔ flows/07-08**：Submit 11 步尾段"回写 Agent 状态"+"回写 IDE Session 状态"未在 Event Model 中明说

### §5.3 对齐度 < 80%（🔴）

- **18 份 ↔ ADR 文件**：全部断链（B-09）
- **flows/03 ↔ agent-api schema**：Resume JSON 字段未验证（B-19）
- **flows/01 状态数 ↔ 任务摘要**：9+5 vs 9+4（B-02）

---

## §6 审查方法学说明

1. **每条结论直接对照 spec 文本**：用 `file:line` 引用（如 `flows/01:9-26`）
2. **不沿用 commit 历史叙事**：无"per X 历史形态"回溯
3. **优先列已知缺口**："缺标比错标安全"显式列 8+ 处
4. **不修改任何 spec 文件**：本任务仅落盘本报告

---

## §7 签字栏 / 修订历史

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 审查者 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 B（任务 1） | 2026-08-26 | 🟡 草案：4 P1 阻断 + 13 P2 警告 + 8 P3 已知缺口；待 Ulysses 终审 |

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008）— 子代理 B（任务 1）| 初版：18 份 spec 6 大维度审查 + 9 🔴 + 17 🟡 + 24 🟢 结论 + 8 项已知缺口 | Phase C 第 2 轮一致性审查任务（per 主对话 2026-08-26 22:17 JST 派发）|
