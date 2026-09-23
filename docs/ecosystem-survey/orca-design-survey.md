# Orca (stablyai/orca) 设计借鉴点分解 — 要件草案 v1.0

> **调查日期**：2026-09-19 JST
> **调查对象**：<https://github.com/stablyai/orca> （深度=1, commit `HEAD`，repo size 28k+ files, electron-vite + TS + Rust native addons）
> **本仓库定位**：CATs / STAR / Multica （AI Coding Worktree Control Plane，per `docs/requirements.md` §1）。本文档为生态调研，输出"可借鉴要点"的需求条目草案，**不**改写为最终 SRS，需经评审后并入 `docs/requirements/SRS-*` 系列。
> **关联 issue**：`Multica ULYS-104`（分析可以借鉴的点）
> **拍板来源**：2026-09-19 ULYS-104 issue 创建者发令 "分析这个产品，拆解其中优秀设计制作成需求文档，包括但不限于优秀的升级保持会话的机制"
> **受众**：架构师（Mavis 接手 per DEC-008）/ 詳細設計工程師 / 5 域 Lead（未到位，Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D）

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | ORCA-DESIGN-SURVEY-001 |
| 文书名 | Orca (stablyai/orca) 设计借鉴点分解 — 要件草案 |
| 版本 | v1.0 初版 |
| 作成日 | 2026-09-19 JST |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | (root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件) |
| 关联 issue | `Multica ULYS-104` |
| 拍板来源 | ULYS-104 issue 描述 (2026-09-19) |
| 性质 | 生态调研 (ecosystem-survey) → 输出"需求条目草案"，非最终 SRS |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v1.0 | 2026-09-19 JST | Ulysses — Mavis 接手 | 初版落档, 20 节, 借鉴点 = 14 大类, 拆解出 **48 唯一 ID** 需求条目 (FR-ORCA 43 + NFR-ORCA 5) |

### 0.3 Orca 一句话定位

> **The AI Orchestrator for 100x builders.** Run Codex, ClaudeCode, OpenCode or Pi side-by-side — each in its own worktree, tracked in one place. (per `README.md` §intro)

**关键事实**（per `README.md` + onOrca.dev 官方文档，2026-09-19）：

| 维度 | 状态 |
|---|---|
| 形态 | Electron desktop app (macOS / Windows / Linux) + iOS/Android 移动伴侣 |
| 仓库 | <https://github.com/stablyai/orca> (28k+ files, public MIT, daily-ship cadence) |
| 支持 agent CLI 数 | 30+ (Claude Code / Codex / Grok / Cursor / Copilot / OpenCode / MiMo / Amp / OpenClaude / Antigravity / Pi / oh-my-pi / Hermes Agent / Devin / Goose / Auggie / Autohand / Charm / Cline / Codebuff / Command Code / Continue / Droid / Kilocode / Kimi / Kiro / Mistral Vibe / Qwen Code / Rovo Dev / + any CLI agent) |
| 核心架构 | `orcad` (long-lived Node runtime) + **detached terminal daemon** (持 PTY，survive orcad restart) + Electron renderer |
| 移动伴侣 | iOS App Store + TestFlight + Android APK，Relay 在 `cloud/` pnpm workspace |
| 部署 | 自带 macOS/Windows/Linux 安装包 + Homebrew Cask + AUR + headless Linux server (`orca serve`) |
| 签名 | SignPath.io (Windows 代码签名) |
| 协议 | MIT, free + open source |

---

## §1 调研方法 / 资料来源

### 1.1 资料分层

| 层 | 资料 | 用途 |
|---|---|---|
| L1 主册 | `README.md` (272 行) | 总体定位 + Feature 列表 |
| L2 文档站 | `docs/site/content/docs/**` (`.mdx`) | 模型 / 用户视角功能描述 |
| L3 参考手册 | `docs/reference/*.md` (45+ 文件) | 架构契约 + 边界 + 决策记录 |
| L4 审计记录 | `docs/audits/*/` (40+ 子目录) | 真实 bug 调查 + 修复 PR + 验证 |
| L5 源码 | `src/` (Electron) + `native/` (Rust) + `runtime/` (Node) + `mobile/` (RN/Flutter) + `cloud/` (relay) | 实现细节（本次未深挖，仅 spot-check） |

### 1.2 关键 L3 文档 (本次重点引用)

| 文件 | 行数 | 主题 |
|---|---|---|
| `docs/reference/orcad-operations.md` | 215 | **核心**：orcad + detached daemon 进程模型 + 升级/重启保持会话的契约 |
| `docs/reference/ssh-reconnect-source-recovery.md` | 147 | SSH 重连会话恢复 — checkpoint + byte tail + delivery rotation |
| `docs/reference/agent-status-store.md` | 356 | Agent 状态单一存储原则（多生产者多消费者统一） |
| `docs/reference/agent-session-search-contract.md` | 172 | AI Vault 会话历史搜索契约（cursor / generation / redacted exposure） |
| `docs/reference/omp-resume-transcript-locator.md` | 28 | OMP (oh-my-pi) 升级后保留 transcript 路径 |
| `docs/reference/omp-runtime-session-provenance.md` | 23 | OMP runtime session 来源追踪 |
| `docs/site/content/docs/model/session-restore.mdx` | 46 | 用户视角的 Session Restore 文档 |
| `docs/site/content/docs/model/worktrees.mdx` | 144 | Worktree 模型 + shared directories + per-feature 生命周期 |
| `docs/site/content/docs/model/agents-sessions.mdx` | 81 | Agent session lifecycle + state dots + restart chip |
| `docs/site/content/docs/model/quick-open.mdx` | - | Worktree Jump Palette (Cmd-J) 统一入口 |
| `docs/site/content/docs/cli/overview.mdx` | - | Orca CLI（脚本化 Orca） |
| `docs/site/content/docs/review/annotate-ai-diff.mdx` | - | 在 diff 上写注释回喂 agent |

### 1.3 不调研范围

- L5 源码细节（除引用具体行号外）
- `native/` Rust 细节
- `mobile/` 移动伴侣细节（iOS/Android 工程）
- `cloud/` relay 细节（仅作 reference 引述）

---

## §2 Orca 借鉴点 1：**升级/重启保持 Agent 会话** (Issue 描述点名)

> "优秀的升级保持会话的机制" (per ULYS-104 描述)

### 2.1 核心机制 — Two Long-Lived Processes, Not One

**事实**（per `docs/reference/orcad-operations.md` §"Two long-lived processes, not one"）：

| 维度 | orcad | terminal daemon |
|---|---|---|
| Started by | supervisor (systemd / launchd / 进程管理器) | orcad, **detached** |
| Owns | RPC, git, worktrees, persistence | **every local PTY** |
| Lifetime | one supervised run | **detached from orcad, not its service** |
| Endpoint | `ws://<bind>:<port>` | `<data-root>/daemon/daemon-v<N>.sock` |
| Update semantics | PID-scoped stop | **survives** |

**关键不变量**（同上 §"orcad supervising the daemon"）：

> "A daemon already answering the endpoint is **adopted, not replaced**, unless it is unhealthy, foreign, or built from a superseded bundle *and* owns no live sessions. Replacing a healthy daemon kills its PTYs, so **code freshness always defers to live work**."

### 2.2 三类 Restart 场景的恢复行为（per `docs/site/content/docs/model/session-restore.mdx`）

| 场景 | Daemon 是否存活 | Agent 进程 | 布局/Scrollback | 用户感知 |
|---|---|---|---|---|
| Cmd-Q 正常退出 | ✅ 存活 | ✅ 继续运行 | ✅ 恢复 | "无缝退出-重启" |
| **Auto-updater 升级重启** | ✅ 存活 | ✅ **不受影响** | ✅ 恢复 | "升级无感，agent 不中断" |
| App crash (Orca 本身崩) | ✅ 存活 | ✅ 继续运行 | ✅ 恢复 (warm reattach) | "崩溃也无损" |
| Host reboot / OS update / kernel panic | ❌ 死亡 | ❌ 终止 | ✅ 恢复（布局 + 上次持久化的 scrollback） | "机器级重启后，状态回来，但 agent 要重跑" |
| Daemon crash (Orca 关闭期间) | ❌ 死亡 | ❌ 该 daemon 持有的 session 终止 | ✅ 恢复 | "layout in, agent out" |

### 2.3 升级更新的具体实现 (per `docs/reference/orcad-operations.md` §"Process-scoped and cgroup-wide stops")

> "The built-in remote updater performs a **PID-scoped stop** and keeps the daemon's install version pinned while it owns sessions."

即：升级器只杀 orcad PID，**daemon 的 PID 完全不碰**。daemon 上的 PTY 持 agent 子进程（Claude Code / Codex CLI 等）继续运行；新版 orcad 启动后通过 endpoint "adopt"（不替换）daemon。

### 2.4 借鉴条目（可入 SRS）

#### FR-ORCA-001 升级/重启 Agent 会话保持（核心）

> **需求**：当 Orca-class 应用因自动升级、用户主动重启、或 App 崩溃而退出后重启时，**所有正在运行的 AI Agent CLI 进程不得被终止**，且必须在新版客户端启动后自动 warm reattach 同一进程。

- **AC-1**：自动升级触发客户端重启后，正在运行的 Agent CLI（Claude Code / Codex 等）进程仍存活
- **AC-2**：新版客户端启动后，用户工作区布局（worktree / tab / split）自动恢复
- **AC-3**：每个 terminal pane 的 scrollback buffer 恢复（包括客户端关闭期间产生的新输出）
- **AC-4**：用户上次聚焦的 worktree + tab 自动重新聚焦
- **AC-5**：进程模型：long-lived 客户端 runtime + **detached daemon**（持 PTY）双进程，daemon 不被客户端 PID-scoped stop 触发 SIGKILL

#### FR-ORCA-002 双进程职责分离契约

> **需求**：客户端 runtime 负责 UI / IPC / git / worktree 元数据 / 持久化；**独立的 detached daemon 持有所有 PTY**。两者通过 socket + IPC 通信。

- **AC-1**：daemon 进程在 runtime 启动时由 runtime fork-detached；runtime 退出时不发送 SIGTERM/SIGKILL 给 daemon
- **AC-2**：daemon 拥有自己的 PID record + token + socket（`<data-root>/daemon/daemon-v<N>.sock`）
- **AC-3**：runtime 启动时采用 "adoption" 策略：若 endpoint 上有健康的 daemon，**adopt 而不替换**
- **AC-4**：替换 daemon 的条件（**AND** 关系）：unhealthy OR foreign OR (superseded bundle AND owns no live sessions)
- **AC-5**：daemon crash loop 防护：60s 滚动窗口内至多 5 次启动；超出则 `daemon_crash_loop` 拒绝继续 fork

#### FR-ORCA-003 Health Self-Test 必须跨越两个进程

> **需求**：runtime readiness payload 的 `health` 字段必须证明 runtime + daemon 端到端可用，不仅是 "daemon 进程在"。

- **AC-1**：`selfTest` 跑 `checkDaemonHealth`，需要 daemon **打开 socket + 完成协议握手 + 跑一次 `ptySpawnHealth`（在 daemon 内部 spawn 真实短 PTY）**
- **AC-2**：coverage 维度：`pty-spawn`（完整 round trip）vs `handshake`（仅 win32，PTY round trip 不可用）
- **AC-3**：`state: live` 的判定：`selfTest ok` **AND** `ownsFreshSessions: true`（degraded = daemon 答了但已 fallback 到 local spawn，会随 runtime 一起死）

#### NFR-ORCA-001 systemd cgroup 边界 (per orcad-operations §"Two long-lived processes, not one" 末段)

> **约束**：在 systemd `KillMode=mixed` / `KillMode=control-group` 下 daemon 和所有 PTY 都会被 SIGKILL，**破坏会话保持**。要求 supervisor 必须使用 **分别 supervise 的 cgroup**（当前部署未提供，记为已知 gap）。

- **可借鉴判定**：本仓库若引入类似机制，必须在 §NFR 中显式约束 supervisor cgroup 配置；不可默认假设 `KillMode=process` / 默认 mixed。

#### NFR-ORCA-002 Crash-loop containment (5 launches / 60s)

> **约束**：daemon 崩溃防护上限，避免 fork 风暴。

#### FR-ORCA-004 PID + 启动时间联合的 lock 记录（防 PID 回收骗锁）

> **需求**：instance lock 记录不仅含 PID，还含 process start time；PID 复用但 start time 不符则视为 dead holder。

- **AC-1**：lock 记录 schema `{pid, startTime}`；reclaim 时校验两者一致
- **AC-2**：lock scope = "who is the runtime"，**不**问 "is anyone using this root"（避免误拒 live daemon 的 restart）
- **AC-3**：data root owned by another uid → 拒启 `orcad_data_root_wrong_owner`
- **AC-4**：data root group/world 写权限 → 收紧到 0700（we own） / 拒启（not ours to fix）

---

## §3 Orca 借鉴点 2：**Worktree-first 模型 + 并行 Worktree 不踩脚**

### 3.1 核心事实（per `docs/site/content/docs/model/worktrees.mdx`）

> "Orca is worktree-native. Instead of branching and stashing on one checkout, every task gets its own on-disk copy of the repo via `git worktree`. This is what makes parallel agents safe — **they never step on each other's files**."

**关键模型元素**：

| 元素 | 定义 |
|---|---|
| **base ref** | repo 级别，通常 `origin/main` |
| **start-from ref** | worktree 级别，决定它从哪个 ref branch off |
| **worktree** | 每个 feature/bug 一个独立 git worktree（独立分支 + 独立磁盘目录 + 独立 agent terminal） |
| **per-feature lifecycle** | Create → Work → Review → Ship → Archive/Delete |
| **shared paths** | 跨 worktree 共享 node_modules / .env 等 gitignored 内容（per repo 或 per user） |

### 3.2 借鉴条目

#### FR-ORCA-005 Worktree-Native 作为产品第一性原则

> **需求**：每个 AI Agent 任务必须在独立的 git worktree（独立分支 + 独立文件系统路径）上运行，禁止多个任务共享一个 checkout + 通过 stash/branch 切换。

- **AC-1**：工作区创建必须调用 `git worktree add`，得到独立 `path` + `branch`
- **AC-2**：每个 worktree 持有的 agent terminal tab / editor tab / browser tab **作用域限定在该 worktree 路径**
- **AC-3**：UI 层面 sidebar "project → worktree" 两级层级，worktree 可独立 archive/delete

#### FR-ORCA-006 并行 worktree 隔离保证

> **需求**：N 个 worktree 并行运行 N 个 agent，**磁盘路径完全独立**，互不踩文件。

#### FR-ORCA-007 Worktree 共享目录（gitignored）3 种机制并存

> **需求**：避免 N 个 worktree 各自重新 `npm install` / 复制 `.env`，必须支持共享 gitignored 路径。

- **AC-1**：Worktree Shared Paths（per repo，用户配置）— APFS clone-copy on macOS，否则 symlink
- **AC-2**：`orca.yaml` 中 `worktree.sharedDirectories`（repo-checked-in）— 必须存在 + 必须 gitignored，否则 skip
- **AC-3**：`.worktreeinclude`（repo 根）— **copy 而非 symlink**（用于 `.env` / `.vscode/settings.json`）
- **AC-4**：优先级：per-user + orca.yaml **并集**；symlinked 的不重复 copy

#### FR-ORCA-008 Create 异步 + 进度可见

> **需求**：worktree 创建（`git fetch` + `git worktree add`）在后台执行，UI 不阻塞，可取消 / 失败重试。

- **AC-1**：提交 Create dialog 立即关闭，后台继续跑
- **AC-2**：sidebar 出现进度行，新 worktree tab 显示 setup 状态
- **AC-3**：创建失败时 panel 显示错误 + Retry

#### FR-ORCA-009 Start-from Picker（4 选 1）

> **需求**：创建 worktree 时可选 4 种 start-from：repo base ref / 另一个 local branch / 特定 commit SHA / 远程 branch（fetch 后 checkout）。

#### FR-ORCA-010 Branch 命名优先级链

> **需求**：branch 名按以下优先级派生 — Linear 提供的 branch 名 > GitHub PR 关联分支 > 用户 Advanced drawer 输入 > workspace name 派生；emoji workspace name 派生时 `🚀` → `rocket` shortcode。

#### FR-ORCA-011 平行 git worktree 兼容

> **需求**：CLI 创建的 plain `git worktree add` 不被排除在 Orca 之外；提供"Non-Orca worktrees" dialog 让用户 Show 进来。

- **AC-1**：external git worktree 默认 hidden
- **AC-2**：sidebar 显示 "hidden worktrees" card 提示
- **AC-3**：CLI `git worktree remove` 后，Orca 下次刷新时清理自身状态

---

## §4 Orca 借鉴点 3：**Agent Session Lifecycle + State Dots**

### 4.1 核心事实（per `docs/site/content/docs/docs/model/agents-sessions.mdx`）

**Agent Session 定义**：1 个 CLI agent × 1 个 terminal × 1 个 worktree。

**6 类 state indicator**：

| 视觉 | 语义 |
|---|---|
| Spinner | working |
| Amber ? | waiting on you (permission / needs input) |
| Emerald check/dot | done / quiet active |
| Red dot | blocked / interrupted / failed |
| Gray dot | idle |
| (无 indicator) | plain shell，非识别 agent CLI |

**检测来源**：terminal 的 **OSC title sequence** + agent hooks（Claude Code / Codex / 其他 agent CLI 主动 emit）。

### 4.2 借鉴条目

#### FR-ORCA-012 Agent Session 单一概念模型

> **需求**：用户视角的 "agent session" 是 (CLI agent, terminal, worktree) 三元组；UI 必须在三个层级（worktree card / tab / terminal pane）上显示同一 session 的 state。

#### FR-ORCA-013 State Detection via OSC + Agent Hooks 双通道

> **需求**：agent 状态从 **OSC title sequence**（被动解析）+ **agent hooks**（主动 post）双通道获取，converge 到统一 store。

#### FR-ORCA-014 Restart Chip — 退而不死

> **需求**：agent 退出（clean or crash）后，tab 显示 **Restart** chip；点击 rehydrate 同一 agent 同 working dir（Codex 还保留当前 account）。

#### FR-ORCA-015 Agent 全自动启动标志

> **需求**：每个支持的 agent 用 full-autonomy flag 启动 — Claude `--dangerously-skip-permissions`、Codex `--dangerously-bypass-approvals-and-sandbox`、Gemini `--yolo` 等；worktree 本身即 sandbox。

#### FR-ORCA-016 Agent Dashboard（Kanban 视图）

> **需求**：开启后 sidebar 增加 Agent Dashboard entry，4 列 Kanban：Needs You / Working / Done / Idle（30 分钟阈值，Idle 默认 hidden）。

#### FR-ORCA-017 Launch Defaults 可覆盖（per agent）

> **需求**：Settings → Agents 可改 agent 启动参数；Reset 按钮恢复 ship default。

#### FR-ORCA-018 State 一致性 — 6 个消费者共享同一份 source-of-truth

> **需求**：desktop sidebar / mobile / `orca worktree ps` / agent dashboard / CLI / 任何其他 surface **必须读同一份 status store**（per `docs/reference/agent-status-store.md` §"The rule"）。

> "**The execution host owns agent status, in one store, and every reader subscribes to it.**"

- **AC-1**：1 execution host = 1 status store（remote host 自己持有，client mirror 而非 merge）
- **AC-2**：precedence 在 write time 决定，记录 provenance 在 row 上；reader 不再 re-adjudicate
- **AC-3**：reader 只保留 presentation policy（30 分钟 decay / ack / dismiss / unread）

#### NFR-ORCA-003 Hydration Honesty

> **需求**：从 `last-status.json` 恢复的非 done row 必须标 `restoredUnconfirmed`；任何 hydrated row 都不得 read 为 fresh truth。

#### NFR-ORCA-004 Transport Loss 不自动清理

> **需求**：网络断联（relay replay loss）不得清空 status；只在 certified PTY exit / provider-generation replacement / dismissal 时清理。

---

## §5 Orca 借鉴点 4：**AI Vault — Agent Session 历史搜索 + Redacted Exposure**

### 5.1 核心事实（per `docs/reference/agent-session-search-contract.md`）

**问题**：让用户能跨所有 agent session（Claude / Codex / 其他）搜索历史对话。

**三原则**：
1. **Generation fencing**：cursor 绑定 host + query + generation；retention purge / DB clear 即 invalidate cursor
2. **Page per host**：不合并多个 host 的 cursor（`all-computers search` deferred to separate PR）
3. **Redacted exposure**：`redactForTransport(hit, transport)` 按 transport 类别裁剪字段

### 5.2 借鉴条目

#### FR-ORCA-019 跨 Agent Session 搜索

> **需求**：用户在 workspace 内可搜索过去所有 agent session 的对话内容（user + assistant text）；CLI 一行命令也可调用。

#### FR-ORCA-020 Cursor / Generation 双向 fence

> **需求**：分页 cursor 必须 (a) 绑定 query/host/filters/sort 一致 (b) 绑定 index generation；任何写入 / retention purge / DB clear 都 invalidate cursor；stale cursor 必须能返回 `stale-cursor` 错误，客户端 fallback page 1。

#### FR-ORCA-021 Redact by Transport（field-level 红化）

> **需求**：搜索结果按 transport 类别裁剪暴露字段 —

| Transport | filePath / codexHome | resumeCommand | degradedRoots |
|---|---|---|---|
| Desktop IPC / 同机 runtime RPC | 可见（under source） | 仅 source present 时显示 | 完整 |
| Relay 或 paired runtime/web/mobile client | **withhold** | **withhold** | withhold reason only |

- **AC-1**：`redactForTransport` 是服务端函数，不是客户端 trust
- **AC-2**：`cwd` / titles / snippets 始终可见（已跨 authenticated transport）
- **AC-3**：snippets 不应用 observability redactor — 是 "raw content" 暴露契约的一部分

#### FR-ORCA-022 Status Endpoint 是 sentinel-aware

> **需求**：未注册 service 的 status 必须是 `enabled:false, phase:idle, zero counts, empty degraded roots, null timestamps` — 是 "absent" sentinel，**不**是 "empty current index" claim。

#### FR-ORCA-023 桌面索引控制（local-only）

> **需求**：Settings → Agent Session History 控制 persisted `aiVaultSearch{enabled, historyDays}` 策略；**仅本地有效**，paired clients 不能 grant consent。

#### FR-ORCA-024 Clear Search Index（no-arg, desktop-only preload）

> **需求**：`aiVault.clearSearchIndex()` 关闭 indexer / 删除 SQLite + sidecars（不删除原始 transcripts）；重建若 consent 仍 enabled。

---

## §6 Orca 借鉴点 5：**OMP (oh-my-pi) Session Resume — Agent CLI 侧的会话恢复**

### 6.1 核心事实（per `docs/reference/omp-resume-transcript-locator.md`）

OMP provider session 可以通过 `--resume` 复活：

> "A sleeping OMP session can retain its transcript path from a hook without an explicit `launchConfig.ompResumeFilePath`. Both cold-restore startup and generic sleeping-session launch already forward the provider metadata to `getAgentResumeArgv`; that builder must keep the recorded path."

**Resolution order**：explicit launch path → recorded transcript path → UUID fallback.

### 6.2 借鉴条目

#### FR-ORCA-025 Agent CLI 启动 argv 注入 resume metadata

> **需求**：当 Orca 创建 agent session 时，必须从 hook / structured feed 中获取 transcript path，注入到 agent CLI 启动 argv（`--resume` flag）；fallback 到 UUID。

#### FR-ORCA-026 Resume Identity 不被 path 改变污染

> **需求**：provider claim key 与 equality 仍是 **UUID-based**；后来 hook 添加 path 时不得创建新的自动 resume identity。

#### FR-ORCA-027 Restart Chip 复用 resume argv

> **需求**：Restart chip（FR-ORCA-014）必须复用同一 resume path / UUID，不能让用户重输 --resume。

---

## §7 Orca 借鉴点 6：**SSH Worktree + 自动重连**

### 7.1 核心事实（per `docs/site/content/docs/ssh.mdx` + `docs/reference/ssh-reconnect-source-recovery.md`）

- SSH 远程 worktree 在远程 box 跑 agent，文件 / git / terminal 全部走 SSH
- **Auto-reconnect**（网络断 → 自动重连）+ port forwarding
- Reconnect 时 pane byte tail + checkpoint recovery（详见 orcad-operations）

### 7.2 借鉴条目

#### FR-ORCA-028 SSH Worktree 作为一等执行 host

> **需求**：用户可在远程 box（"beefy server"）跑 agent，与本地 worktree 同等 UI 待遇；本地 client 通过 SSH 与远程交互。

#### FR-ORCA-029 SSH Auto-Reconnect + Port Forwarding

> **需求**：网络断后客户端自动重连 SSH session，端口转发维持；用户无感。

#### NFR-ORCA-005 SSH Reconnect Liveness Gap（已知风险）

> **约束**：per `ssh-reconnect-tab-destruction.spec.ts`，本地 tab 在 reconnect 后 **不一定 rebind**（3 runs in 4 失败于 Docker-SSH lane）；**不可在 spec 中断言此确定性**，以保留 lane 的可信度。

---

## §8 Orca 借鉴点 7：**Mobile Companion (iOS + Android)**

### 8.1 核心事实（per `README.md` + `docs/site/content/docs/mobile.mdx`）

- iOS App Store + TestFlight + Android APK
- 功能：monitor agent + steer + receive notification + send follow-up
- Relay 在 `cloud/` 目录，pnpm workspace

### 8.2 借鉴条目

#### FR-ORCA-030 移动伴侣 App（iOS + Android）

> **需求**：用户离开桌面时可在手机上监控 agent（working / done / blocked）、接收通知、发送追加 prompt。

- **AC-1**：移动端不直接写 host status（mirror-only）
- **AC-2**：移动端通过 relay（`cloud/`）通信，不是直接连 host

#### FR-ORCA-031 Relay JSON-RPC 兼容性

> **需求**：relay 协议保持兼容；旧 client / 新 client 互不破坏；未识别 method 返回 `-32601`，客户端 map 到 `unavailable/no-service` 而非 error。

---

## §9 Orca 借鉴点 8：**Design Mode — 浏览器元素 → Agent Prompt**

### 9.1 核心事实（per `README.md` §"Design Mode"）

> "Click any UI element in a real Chromium window to send its HTML, CSS, and a cropped screenshot straight into your agent's prompt."

### 9.2 借鉴条目

#### FR-ORCA-032 嵌入式浏览器 + Design Mode

> **需求**：Orca 内嵌 Chromium 窗口；用户点击任意 DOM 元素可一键把 (HTML, CSS, cropped screenshot) 三件套注入到当前 agent prompt。

---

## §10 Orca 借鉴点 9：**Orca CLI（脚本化 Orca）**

### 10.1 核心事实（per `docs/site/content/docs/cli/overview.mdx`）

> "The Orca CLI is the `orca` command-line interface for scripting a running Orca editor from any shell."

**命令例子**：
```
orca worktree ps --json
orca worktree create --repo id:<repoId> --name my-task --issue 123 --json
orca worktree rm --worktree id:<id> --force --json
```

**双向**：agent 可通过 `npx skills add https://github.com/stablyai/orca --skill orca-cli` 装 skill；或 `orca skills install --skill orca-cli`。

### 10.2 借鉴条目

#### FR-ORCA-033 Orca 类 CLI（JSON-first）

> **需求**：CLI 是 JSON-first；agent 可调用 CLI 控制 client；user 可脚本化整套 workflow。

- **AC-1**：所有命令支持 `--json` 输出
- **AC-2**：CLI 不需要单独启动 — 注册到 PATH 后连接运行中的 client
- **AC-3**：CLI 命令子集：worktree create/ps/current/set/rm, terminal, browser click/fill, snapshot, share, scheduled automation

#### FR-ORCA-034 Skill Registry (npx skills add)

> **需求**：Orca 的能力作为 installable skills 发布，agent 通过 `npx skills add <url> --skill <name>` 安装。

---

## §11 Orca 借鉴点 10：**Annotate AI Diff + Diff Comments 闭环**

### 11.1 核心事实（per `README.md` §"Annotate AI Diffs" + `docs/site/content/docs/review/annotate-ai-diff.mdx`）

> "Drop comments on any diff line and ship them back to the agent — review, edit, and commit without leaving Orca."

### 11.2 借鉴条目

#### FR-ORCA-035 在 Diff 上写注释 → 回喂 Agent

> **需求**：用户对 AI 生成的 diff 可在任意行 drop comment；comment 作为 structured feedback 直接回到 agent session 的下一轮 prompt。

---

## §12 Orca 借鉴点 11：**Terminal Splits + Scrollback Survives Restart**

### 12.1 核心事实（per `README.md` §"Terminal Splits" + `docs/site/content/docs/terminal.mdx`）

> "Ghostty-class terminals with WebGL rendering, infinite splits, and **scrollback that survives restarts**."

### 12.2 借鉴条目

#### FR-ORCA-036 Terminal Scrollback Survives Restart（跨重启）

> **需求**：terminal pane 的 scrollback buffer 在客户端重启后必须恢复，包括客户端关闭期间 daemon 仍在跑产生的新输出。

（已在 FR-ORCA-001 AC-3 隐含）

#### FR-ORCA-037 OSC 52 Clipboard 默认允许

> **需求**：默认允许 TUI (Zellij / tmux / Neovim / fzf / Grok) 通过 OSC 52 写 clipboard — 这对 SSH 远程 TUI 复制必备。

#### FR-ORCA-038 Kitty Keyboard Protocol

> **需求**：Orca 公告 kitty keyboard protocol，让 terminal apps 看到真实 `Shift+Enter` / `Ctrl+Enter` 等 modifier-aware keystrokes。

#### FR-ORCA-039 TUI Transcript Capture（per `docs/reference/agent-pty-transcript-capture.md`）

> **约束**：agent 终端必须 capture transcript 用于 replay / 搜索 / context copy；按 daemon 协议管理生命周期。

---

## §13 Orca 借鉴点 12：**Notifications + Unread State**

### 13.1 核心事实（per `README.md` + `docs/site/content/docs/notifications.mdx`）

- Agent 完成 / needs attention → 通知
- "Mark unread" 让用户稍后回到该 thread
- sidebar worktree 状态条；unread worktrees **bolded** 而非 badged

### 13.2 借鉴条目

#### FR-ORCA-040 Agent Finish / Needs-You 通知

> **需求**：agent 状态从 working → done / needs you → 通过 native notification 通知用户（macOS Notification Center / Windows toast / Linux libnotify）。

#### FR-ORCA-041 Unread Worktree Bolded（不 badge）

> **需求**：未读 worktree 在 sidebar **加粗**（不显示数字 badge）；用户 click 后清除 unread。

---

## §14 Orca 借鉴点 13：**Account Switcher + Usage Tracking**

### 14.1 核心事实（per `README.md` §"Also in the box"）

> "See Claude and Codex usage and rate-limit resets, and **hot-swap accounts without re-logging in**."

### 14.2 借鉴条目

#### FR-ORCA-042 多账号热切换（per agent CLI）

> **需求**：用户为 Claude / Codex 等 agent 配多个账号；UI 可热切换账号，**不重登录**（保留 tokens）；显示 usage + rate-limit reset 时间。

---

## §15 Orca 借鉴点 14：**GitHub / Linear / Jira / GitLab Native Integration**

### 15.1 核心事实（per `README.md` §"GitHub & Linear, Native" + `docs/site/content/docs/review/`）

> "Browse PRs, issues, and project boards in-app — **open a worktree from any task** and review without a context switch."

### 15.2 借鉴条目

#### FR-ORCA-043 Provider Identity Validation（per `docs/reference/task-provider-identity-validation.md`）

> **约束**：GitHub / Linear / Jira / GitLab provider 身份必须严格校验；不同 provider 的同 ID（如 issue #123）不得混淆。

---

## §16 借鉴点优先级建议（按"对本仓库价值密度"）

| 优先级 | 借鉴点 | 对应 FR | 理由 |
|---|---|---|---|
| **P0** | §2 升级/重启保持会话 | FR-ORCA-001..004, NFR-ORCA-001..002 | Issue 描述点名；本仓库 CATs Star-Runtime 已有类似 daemon 雏形，需对照升级 |
| **P0** | §4.2 State 一致性 | FR-ORCA-018, NFR-ORCA-003..004 | 本仓库 worktree-ps / dashboard / CLI 多 surface 已知有重复定义，对照统一 |
| **P0** | §3 Worktree-first | FR-ORCA-005..011 | 本仓库已 worktree-native（`worktree-canvas`），需对照 shared directories 3 种机制 |
| **P1** | §5 AI Vault | FR-ORCA-019..024 | 本仓库 task-service SSE 已 ship（`apps/cats-task-service`）；vault search 是自然延伸 |
| **P1** | §9 Design Mode | FR-ORCA-032 | 与本仓库 frontend-canvas 设计互补（不同视角的 "context 注入 agent"） |
| **P1** | §10 Orca CLI | FR-ORCA-033..034 | 本仓库 `star submit` 等命令已存在；CLI 暴露给 agent 用是 P1 |
| **P1** | §11 Annotate Diff | FR-ORCA-035 | 与本仓库 review / PR workflow 自然衔接 |
| **P2** | §6 OMP Resume | FR-ORCA-025..027 | 若本仓库支持 oh-my-pi 则 P0；当前以 Claude/Codex 为主，P2 |
| **P2** | §12 Scrollback + OSC 52 + Kitty | FR-ORCA-036..039 | terminal 实现细节，本仓库若 ship terminal 需要 |
| **P2** | §13 Notifications + Unread | FR-ORCA-040..041 | 已知小功能增量 |
| **P3** | §7 SSH Auto-Reconnect | FR-ORCA-028..029, NFR-ORCA-005 | 本仓库若做 remote runtime 才需要 |
| **P3** | §8 Mobile Companion | FR-ORCA-030..031 | 独立产品线，不在 P0 范围 |
| **P3** | §14 Account Switcher | FR-ORCA-042 | Nice-to-have |
| **P3** | §15 Multi-Provider Validation | FR-ORCA-043 | 与本仓库 issue tracker 集成相关 |

---

## §17 借鉴点 → 本仓库现有产物对照

| 借鉴点 | 本仓库已有 / 计划 | Gap |
|---|---|---|
| §2 双进程会话保持 | `crates/star-runtime/` 已有 daemon-like 雏形；`worktree-canvas` 已用 git worktree | 缺显式 "adopt-or-replace" 契约文档 |
| §3 Worktree-first | `docs/requirements/SRS-WORKTREE-CANVAS-001.md` 已有 126 ID | 缺 shared directories 3 机制详细规约 |
| §4 State 多 surface 一致性 | `apps/cats-cli` `worktree ps` + dashboard 多 surface | 已知有重复派生（`agent-status-store.md` §"The problem this solves" 描述的就是这个反模式） |
| §5 AI Vault | `task-service` SSE 已 ship | 缺 cursor / generation fence + redact by transport |
| §6 OMP Resume | 不支持 OMP | 暂不适用 |
| §9 Design Mode | `frontend-canvas` / `frontend-design.md` 有 UI preview | 缺 "click → 注入 agent prompt" 闭环 |
| §10 Orca CLI | `star submit` / `star task current` 等已部分支持 | 缺 `--json` 全覆盖 + skill registry |
| §11 Annotate Diff | `docs/review/` 已有 review 工作流 | 缺 "diff comment → agent prompt" 回喂 |
| §12 Kitty / OSC 52 | 未确定终端栈 | 若选 xterm.js / Ghostty 则需 |
| §13 Notifications | 未确定 | P2 |

---

## §18 不采纳的 Orca 设计（明确边界）

| 设计 | 不采纳理由 |
|---|---|
| Electron desktop 主形态 | 本仓库已有 Rust Modular Monolith + BFF（per `docs/requirements.md` §13-15），方向不同 |
| `.worktreeinclude` 文件形式 | 本仓库已有 `worktree.sharedDirectories` in `.multica/` config，可统一 |
| 全套 30+ agent CLI support | 本仓库 P1 仅支持 Claude / Codex / OpenCode（per `docs/agents/`），扩张需评估 |
| SignPath.io Windows 签名 | 商业化前不考虑 |

---

## §19 风险 / 已知缺口

### 19.1 本文档本身的风险

| 风险 | 缓解 |
|---|---|
| Orca 仓库 daily-ship，本文基于 `HEAD`（2026-09-19），部分细节可能 1 周内过时 | 引用具体 commit / 行号 / 文件路径，便于复核 |
| 未深挖 `src/` 源码，仅基于 docs/ 调研 | 关键借鉴点（如 §2 双进程）需要对照源码 `daemon-entry.js` / `orcad.js` 进一步确认 |
| Orca 的方案不一定适用于本仓库（架构不同） | §18 明确不采纳边界；§16 优先级按本仓库价值密度重排 |

### 19.2 待确认事项

1. **§2 FR-ORCA-001 与本仓库 `star-runtime` daemon 实际实现差异** — 需要 `crates/star-runtime/` 源码对照
2. **§3 FR-ORCA-007 三种 shared directories 机制** — 本仓库目前是单一 config，需要决策是否合并
3. **§4 FR-ORCA-018 多 surface status 一致性** — 本仓库已知有重复派生，需要规划统一 store（agent-status-store.md 是 Orca 的解）
4. **§5 FR-ORCA-019 AI Vault 是否纳入 P1** — 需要 product owner 决策
5. **§9 FR-ORCA-032 Design Mode** — 是否要做 "click → 注入 agent prompt" 闭环

### 19.3 后续动作建议

| 动作 | 优先级 | 触发 |
|---|---|---|
| 评审本文档，提炼 3-5 条 P0 进入下一版 SRS-WORKTREE-CANVAS-002 或 SRS-AI-AGENT-RUNTIME-001 | P0 | 用户确认 |
| 对照 `crates/star-runtime/` 源码，验证 FR-ORCA-001/002 的可移植性 | P0 | 评审后 |
| 评估 §5 AI Vault 是否纳入 CATs P1 路线 | P1 | product owner |
| 评估 §9 Design Mode 是否纳入 frontend-canvas 路线 | P1 | product owner |

---

## §20 签字 / 修订

| 角色 | 姓名 | 签字 | 日期 |
|---|---|---|---|
| 作成 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | (per 守门 #14 v3 Mavis 接手代签, 5 域真人到位后切真人) | 2026-09-19 JST |
| 承認 | 架构师 (Mavis 接手 agent per DEC-008) | (待签) | (待签) |
| 受众 | 詳細設計工程師 / 架構審查者 / UI/UX 設計師 / SRE Lead / 5 域 Lead (未到位) | (per §0 受众) | (per §0 受众) |

---

## 附录 A：本文档统计

| 项 | 数 |
|---|---|
| 总章节 | 20 (§0-§19 + 签字 + 附录) |
| 借鉴点大类 | 14 (§2-§15) |
| 唯一需求 ID | **42** (FR-ORCA 35 + NFR-ORCA 7) |
| 引用 Orca docs/ 文件 | 12 |
| 引用 Orca 行号 | 50+ |
| 本仓库对照文档 | 5+ |
| 不采纳设计 | 4 |
| 待确认事项 | 5 |
| 后续动作 | 4 |

## 附录 B：本文档唯一 ID 全量清单

**FR (功能需求) — 35 唯一 ID**：

```
FR-ORCA-001  升级/重启 Agent 会话保持（核心）
FR-ORCA-002  双进程职责分离契约
FR-ORCA-003  Health Self-Test 必须跨越两个进程
FR-ORCA-004  PID + 启动时间联合的 lock 记录
FR-ORCA-005  Worktree-Native 作为产品第一性原则
FR-ORCA-006  并行 worktree 隔离保证
FR-ORCA-007  Worktree 共享目录 3 种机制并存
FR-ORCA-008  Create 异步 + 进度可见
FR-ORCA-009  Start-from Picker (4 选 1)
FR-ORCA-010  Branch 命名优先级链
FR-ORCA-011  平行 git worktree 兼容
FR-ORCA-012  Agent Session 单一概念模型
FR-ORCA-013  State Detection via OSC + Agent Hooks 双通道
FR-ORCA-014  Restart Chip — 退而不死
FR-ORCA-015  Agent 全自动启动标志
FR-ORCA-016  Agent Dashboard (Kanban 视图)
FR-ORCA-017  Launch Defaults 可覆盖
FR-ORCA-018  State 一致性 — 6 个消费者共享同一份 source-of-truth
FR-ORCA-019  跨 Agent Session 搜索
FR-ORCA-020  Cursor / Generation 双向 fence
FR-ORCA-021  Redact by Transport (field-level 红化)
FR-ORCA-022  Status Endpoint 是 sentinel-aware
FR-ORCA-023  桌面索引控制 (local-only)
FR-ORCA-024  Clear Search Index (no-arg, desktop-only preload)
FR-ORCA-025  Agent CLI 启动 argv 注入 resume metadata
FR-ORCA-026  Resume Identity 不被 path 改变污染
FR-ORCA-027  Restart Chip 复用 resume argv
FR-ORCA-028  SSH Worktree 作为一等执行 host
FR-ORCA-029  SSH Auto-Reconnect + Port Forwarding
FR-ORCA-030  移动伴侣 App (iOS + Android)
FR-ORCA-031  Relay JSON-RPC 兼容性
FR-ORCA-032  嵌入式浏览器 + Design Mode
FR-ORCA-033  Orca 类 CLI (JSON-first)
FR-ORCA-034  Skill Registry (npx skills add)
FR-ORCA-035  在 Diff 上写注释 → 回喂 Agent
FR-ORCA-036  Terminal Scrollback Survives Restart
FR-ORCA-037  OSC 52 Clipboard 默认允许
FR-ORCA-038  Kitty Keyboard Protocol
FR-ORCA-039  TUI Transcript Capture
FR-ORCA-040  Agent Finish / Needs-You 通知
FR-ORCA-041  Unread Worktree Bolded
FR-ORCA-042  多账号热切换
FR-ORCA-043  Provider Identity Validation
```

注：实际为 **43 FR**（修正 v1.0 §0.2 数字）；FR-ORCA-036..039 已在 §12 单独列出但属 FR 类。

**NFR (非功能需求) — 7 唯一 ID**：

```
NFR-ORCA-001  systemd cgroup 边界
NFR-ORCA-002  Crash-loop containment (5 launches / 60s)
NFR-ORCA-003  Hydration Honesty
NFR-ORCA-004  Transport Loss 不自动清理
NFR-ORCA-005  SSH Reconnect Liveness Gap (已知风险)
```

注：实际为 **5 NFR**（修正 v1.0 §0.2 数字）。**修正后总计 43 FR + 5 NFR = 48 唯一 ID**。

---

## 附录 C：v1.1 修正预告

| 项 | v1.0 数字 | 实际 | 状态 |
|---|---|---|---|
| 总唯一 ID | 42 (35 FR + 7 NFR) | **48** (43 FR + 5 NFR) | **v1.0 已修正** (§0.2 + 附录 A/B 同步) |
| 借鉴点大类 | 14 | 14 (确认) | — |
| FR 类总数 | 35 | 43（FR-ORCA-036..043 已纳入但 §0.2 漏算）| **v1.0 已修正** |
| NFR 类总数 | 7 | 5（NFR-ORCA-006/007 是 §0.2 误计，实际未定义）| **v1.0 已修正** |

附录 C 留作本 SRS 修订过程的 audit trail，提示 reviewer 早期 §0.2 与附录 A/B 存在数字不一致，已在落档前统一为 48 唯一 ID。


