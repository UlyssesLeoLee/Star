# SRS-MULTICA-RUNTIME-001

> **Multica Runtime Registry 域要件定義書 v0.1** (per ADR-0026 v0.2 §2.1 模式 1, 跟 v32 候选对齐)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-RUNTIME-001.md`](../design/BD-MULTICA-RUNTIME-001.md) (下个 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.1 (v32 候选)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4 (2026-09-10 12:45 JST 反转)
> - 日期: 2026-09-11 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-RUNTIME-001 |
| 文书名 | Multica Runtime Registry 域要件定義書 (v32 候选对齐) |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.1 + §2.1 模式 1 |
| 关联 inventory | inventory §2.1 v32 候选 |
| 上位要件 | (无, 本 SRS 是 Multica 模式级参考的首个专题 SRS) |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 (本文档为需求文档) |
| 模板结构 | 12 段严格按 brief §1.3 (跟 SRS-AGENT-RELATIONSHIP-001 同形) |
| 子能力 | 6 子能力 (RT-1 ~ RT-6) × 28 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.1 实证的 Multica 源码 (`github.com/multica-ai/multica` 5 关键文件 line refs) + §2.1 模式 1 (Runtime Scan + Poisoned 语义) 拍板的 v32 候选, 定义 STAR 平台 **Multica Runtime Registry 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: 模式级参考, 组件级不引用 — 借鉴 Multica 的 25 provider probe + MinVersion gate + login shell 兜底 + 三档 status, **不引入** Multica server / UI / 正交 runtime / Squad / WebSocket / multi-tenant。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 4 类具体痛点 (per ADR-0026 §1.2 + 守门 #9 实证):

1. **Mavis 跑在哪个 CLI 不可观测** — Mavis 一次只跑一个 CLI (mcode), 本机还装哪些 agent CLI 不知道
2. **Auth / 版本失败不可恢复** — 工具 auth 失效或版本过低, Mavis 看不到
3. **GUI 启动 Mavis 看不到登录 shell PATH** — nvm / fnm / volta 装的 CLI 探测不到 (per Multica agents_probe.go:30-50 实证)
4. **MinVersion 缺统一表** — 各 agent 最低版本散落各处, 没集中管理

### 1.3 包含范围 (In-Scope)

6 子能力 (per Multica 5 关键文件 line refs):

| 子能力 | Multica 源 | 关键 file:line |
|---|---|---|
| RT-1 25 provider 探测表 | agents_probe.go:155-306 (probeAgentCLIs) | 全 25 named + BuiltinRuntimes 派生 |
| RT-2 Login shell 兜底 | agents_probe.go:30-50 (shellResolveTTL) | 30min TTL 缓存 |
| RT-3 MinVersion gate + sentinel error | version.go:13-22, 161-178 (MinVersions + CheckMinVersion) | 8 provider 最低 semver |
| RT-4 三档 status (active / stale / poisoned) | 综合 poisoned.go + liveness_store.go | poisoned 不删标 |
| RT-5 Runtime 不可恢复检测 (跟 session poison 区分) | poisoned.go:10-217 (FailureReason × 5) | runtime 探测失败 vs session 不可恢复 (per ADR-0026 v0.2 §1.1 修正 2) |
| RT-6 落档 + automation console 扩展 | docs/reports/runtime-scan/ + frontend | 守门 #1 v15 + 守门 #9 v3 |

### 1.4 不含范围 (Out-of-Scope, per ADR-0026 v0.2 §2.2)

7 项明确不做 (per ADR-0026 v0.2 §2.2 修正):

- ❌ Multica server (Go + PostgreSQL 17)
- ❌ Multica UI (Next.js Kanban)
- ❌ Runtime = daemon × AI tool × workspace 正交
- ❌ Squad / Leader agent 路由
- ❌ WebSocket 心跳 + 3 档 liveness
- ❌ Multi-tenant role matrix
- ❌ Multica daemon (Go 编译产物)

### 1.5 受众范围 disclaimer

- 5 域独立 Lead ≠ Star 22 DDD bounded context (per 2026-08-31 22:45 JST Q1-D 拍板)
- 本 SRS 不引用 RGS 仓 + 不建立业务子域↔DDD 映射
- 本 SRS 跟现有 `SRS-STAR-AGENT-RUNTIME-001.md` (Rust 内部 Runtime, 24 守门 + Lightweight Runtime + Agent-Oriented ECS) **平行但不重叠** — 后者讲 STAR 自身 Runtime, 本 SRS 讲外部 agent CLI 注册表

---

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Provider** | 一个 agent CLI (claude / codex / mcode / ... 25 个), 跟 STAR / Mavis 内部 Runtime 无关 |
| **Runtime** | 在本 SRS 上下文 = 一个 provider 的"探测到 + 探测成功 + 可用" 的逻辑记录 (per Multica 抽象, 但不引入正交三维) |
| **Status** | 三档: 🟢 active (探测 + auth 双过) / 🟡 stale (探测过但 auth 失效或版本过低) / 🔴 poisoned (探测到但 unusable) |
| **MinVersion** | 8 个 provider 的最低 semver (per Multica MinVersions map) |
| **Sentinel error** | 区分"unreadable 版本" (`ErrCLIVersionMissing`) vs "确认旧版本" (`ErrCLIVersionTooOld`), 后者才 take runtime offline (per Multica version.go:58-61, 147-155) |
| **Login shell fallback** | 探测 bare command name 失败时, 走用户登录 shell (`~/.zshrc` / `~/.bashrc`) 解析 PATH, 30min TTL 缓存 |
| **Audit log** | `docs/reports/runtime-scan/<date>.log` 持久化每次 scan 结果 (per 守门 #1 v15) |

---

## §3 业务背景

### 3.1 Multica 5 关键文件实证 (per ADR-0026 v0.2 §1.3)

| 文件 | 行数 | 关键发现 |
|---|---|---|
| `server/pkg/agent/version.go` | 178 | 8 provider MinVersions + dev-build git-describe 例外 + `BelowMinimumError` 类型化 |
| `server/internal/daemon/agents_probe.go` | 330 | 25 named provider + BuiltinRuntimes 派生 + login shell 兜底 + 30min TTL |
| `server/internal/daemon/poisoned.go` | 217 | 4 类 session poison 原因 + classify 函数 (本 SRS 不深入, 跟 v33 / SRS-MULTICA-POISON-001 配套) |
| `server/internal/handler/runtime_liveness_store.go` | 130 | Redis TTL 加速 hot path + Available() fallback (本 SRS 不引入, 跟 Q4 不做对齐) |
| `server/internal/daemon/client.go` | 1287 | ClaimTask + SendHeartbeat + Register + Deregister (本 SRS 不引入, session 模型不需要) |

### 3.2 拍板来源 (per ADR-0026 v0.2 + ask_user 拍板)

- 2026-09-11 16:33 JST Ulysses "是否可以进一步参考 multica"
- 16:35 JST ask_user 选项 path_opt1 (只搬模式, 自己实装) + scope_opt4 (只写设计文档 / ADR, 不实装)
- 16:38 JST Ulysses 发 `https://github.com/multica-ai/multica` 链接
- 16:39 JST ask_user 选项 depth_opt1 (Clone + 读 5 关键文件 + 更新 ADR)
- 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 (分专题) + inventory_opt1 (文档内 + 独立 inventory) + code_opt1 (只出文档)

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发): 本 SRS 落档 = 用户拍板 = 明确新事件
- 守门 #5 (env 安全): scan.py 走 exec + LookPath, 不读 secret
- 守门 #6 (PowerShell only): scan.py subprocess.run(shell=False)
- 守门 #9 v27 (RPC fallback): 本 SRS 不派 subagent, root 直接实装
- 守门 #11 (缺标比错标): §1.4 显式列"不做" 7 项
- 守门 #12 v21 ([P] docs 同步): commit 引用 ADR-0026 + automation-design.md + inventory
- 守门 #14 v4 (Mavis 审核): author=Ulysses, 审批 = 架构师 (Mavis 接手)

---

## §4 功能需求 (FR, 28 项)

### RT-1 25 provider 探测表 (FR-1 ~ FR-4)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-1 | 探测本机 25 named provider (claude / codex / opencode / codearts / deveco / openclaw / hermes / pi / cursor / copilot / kimi / reasonix / dsh / kiro / codebuddy / antigravity / qoder / qoderclicn / traecli / grok / qwen / qwenpaw / dim / mcode / zeroclaw) | agents_probe.go:155-296 | P0 |
| FR-2 | BuiltinRuntimes 派生循环 (omp 等) 同样探测 | agents_probe.go:206-212 | P0 |
| FR-3 | 每个 provider 对应 `MULTICA_<NAME>_PATH` env (env 优先) + `MULTICA_<NAME>_MODEL` env | agents_probe.go:116-124 | P0 |
| FR-4 | mcode / qwenpaw / zeroclaw 这 3 个 model 由他们自己管, **不读 model env** | agents_probe.go:292-305 | P0 |

### RT-2 Login shell 兜底 (FR-5 ~ FR-7)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-5 | bare command name LookPath 失败 → 走用户登录 shell 解析 (`~/.zshrc` / `~/.bashrc`) | agents_probe.go:97-138 | P0 |
| FR-6 | 解析结果缓存 30min (避免每次 probe fork shell) | agents_probe.go:30 (`shellResolveTTL`) | P0 |
| FR-7 | 解析 env 变更 (PATH / SHELL / HOME) → 缓存立即失效 | agents_probe.go:42-48 (`shellResolveEnvKey`) | P0 |

### RT-3 MinVersion gate (FR-8 ~ FR-11)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-8 | 8 provider 锁最低 semver (claude 2.0.0 / codex 0.100.0 / copilot 1.0.0 / grok 0.2.89 / qwen 0.20.0 / dim 0.3.10 / mcode 0.1.2 / zeroclaw 0.8.0) | version.go:13-22 (`MinVersions`) | P0 |
| FR-9 | 区分"unreadable 版本" (`ErrCLIVersionMissing`) vs "确认旧版本" (`ErrCLIVersionTooOld`), 后者才标 poisoned | version.go:58-61, 147-155 | P0 |
| FR-10 | Dev-build (git-describe 形态 `v0.2.15-235-gdaf0e935`) 一律放过, 让 `make daemon` 不被自己卡死 | version.go:69-93 | P0 |
| FR-11 | 没锁最低版本的 provider (剩余 17 个) `CheckMinVersion` 返 nil 跳过 | version.go:162-165 | P0 |

### RT-4 三档 status (FR-12 ~ FR-15)

| FR | 描述 | Multica 源 | 优先级 |
|---|---|---|---|
| FR-12 | 🟢 active = 探测成功 + auth 探测通过 + MinVersion 满足 | 综合多源 | P0 |
| FR-13 | 🟡 stale = 探测成功但 auth 探测失败 (key 过期) 或 MinVersion 低于最低 | version.go + 自定 auth probe | P0 |
| FR-14 | 🔴 poisoned = 探测到但 unusable, **不删标**, 跟守门 #11 缺标比错标一致 | 综合多源 | P0 |
| FR-15 | 三档 status 写进 `docs/reports/runtime-scan/<date>.log` 持久化 | 自定 | P0 |

### RT-5 Runtime 不可恢复检测 (FR-16 ~ FR-20)

> **跟 session poison 区分 (per ADR-0026 v0.2 §1.1 修正 2)**: runtime 不可恢复 = 本机这个 CLI 本身坏了 (binary 损坏 / auth 永久失效); session 不可恢复 = `(agent, issue) session` resume 必然坏 (v33 / SRS-MULTICA-POISON-001 配套)。

| FR | 描述 | 优先级 |
|---|---|---|
| FR-16 | Runtime 不可恢复检测 = binary 损坏 + auth 永久失效 + MinVersion 永久低于 | P0 |
| FR-17 | 检测到不可恢复 → 标 🔴 poisoned + 写 `docs/reports/runtime-scan/poisoned-<date>.md` 红 banner 警告 | P0 |
| FR-18 | automation console 显示 🔴 poisoned 标 + 修复建议 ("请运行 `claude auth login` 重置" / "请升级到 v0.1.2+") | P0 |
| FR-19 | 🔴 poisoned 不删, 等人工修 (per 守门 #11) | P0 |
| FR-20 | session 不可恢复 (IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized) 不归本 SRS, 归 SRS-MULTICA-POISON-001 | P0 |

### RT-6 落档 + automation console 扩展 (FR-21 ~ FR-28)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-21 | `scripts/automation/registry/scan.py` 主调用, agent 调 `python scan.py` 即可 | P0 |
| FR-22 | `scan.py` 走 subprocess.run(shell=False) (per 守门 #6) | P0 |
| FR-23 | 每次 scan 落 `docs/reports/runtime-scan/<date>.log` (per 守门 #1 v15) | P0 |
| FR-24 | `scan.py` --audit-log 参数 (per 守门 v21 横向 audit log 范式) | P0 |
| FR-25 | automation console 加 "环境扫描" 页, 跟现有 5 域 MSW handler (per `frontend/src/app/automation-debug/`) 集成 | P0 |
| FR-26 | console 页显示三档 status 表 + 修复建议链接 | P0 |
| FR-27 | console 页支持手动 re-scan (per Multica `multica daemon restart` 模式) | P0 |
| FR-28 | console 页显示周报 "🔴 poisoned 累计" 提醒 (per ADR-0026 v0.2 §4.3 风险缓解) | P0 |

---

## §5 非功能需求 (NFR, 6 项)

| NFR | 指标 | 备注 |
|---|---|---|
| NFR-1 性能 | scan 全 25 provider < 5s (warm cache) / < 30s (cold cache 含 login shell fork) | per Multica 5min timeout 实证 |
| NFR-2 可靠性 | scan 失败不挂主流程, 标 🔴 poisoned + 落 log | per Multica Available() fallback |
| NFR-3 跨平台 | Windows / macOS / Linux 三平台一致行为 | per Multica canonical_path_windows.go |
| NFR-4 安全 | 不读 env 值 (per 守门 #5), env 只 invoke 不打印 | per 守门 #5 hard ban |
| NFR-5 可观测 | 每次 scan 落 log, console 页显示历史 | per 守门 #1 v15 |
| NFR-6 易用 | `python scripts/automation/registry/scan.py` 一行调用, agent 主上下文不写长 shell | per 守门 v19 |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #5 env 安全 | scan.py 走 exec + LookPath, 不读 env 值 |
| 守门 #6 PowerShell only | subprocess.run(shell=False) |
| 守门 #11 缺标比错标 | 🔴 poisoned 不删, 等人工修 |
| 守门 #12 v21 [P] docs 同步 | 每次 scan 落 log |
| 守门 #19 v19 [P] 自动化档 | R + V + A 三维命中, 强制走 Python |
| 守门 #1 v15 死循环饱和 | docs 同步 commit 必新事件触发 |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 25 provider 列表变更 (Multica 加新 provider) | 中 | 低 | BuiltinRuntimes 派生循环自动捕获; named provider 加 1 行 env var 即可 |
| 用户设了 `MULTICA_CLAUDE_PATH=/no/such/path` | 中 | 低 | 绝对路径不 fallback (per agents_probe.go:129-131) |
| GUI 启动 Mavis 看不到 nvm shim | 中 | 中 | Login shell fallback (per FR-5) |
| 5 min timeout 触达 (per Multica) | 低 | 中 | scan 拆 warm/cold cache, cold 加 timeout=30s |

---

## §7 验收条件 (AC)

| AC | 描述 | 验证方式 |
|---|---|---|
| AC-1 | scan 25 provider 全部探测, output 含全部 name | `python scan.py` 输出 25 行, 每行 1 provider |
| AC-2 | MinVersion gate 8 provider 全部满足, 低于最低 → 标 🔴 poisoned | unit test + 故意装旧版本验证 |
| AC-3 | Login shell fallback 跑通, GUI Mavis 探测到 nvm shim | macOS GUI 启动 Mavis 验证 |
| AC-4 | 三档 status 全部覆盖, 守门 #11 缺标比错标 满足 | unit test 3 状态各 1 case |
| AC-5 | 不读 env 值 (per 守门 #5) | grep scan.py 无 `os.environ.get(... print` 模式 |
| AC-6 | subprocess.run(shell=False) 全部 (per 守门 #6) | grep scan.py 无 `shell=True` |
| AC-7 | 每次 scan 落 log (per 守门 #1 v15) | `ls docs/reports/runtime-scan/` 含当日 .log |
| AC-8 | Automation console "环境扫描" 页可访问 + 显示三档 | 浏览器 + Playwright 验证 |

---

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 mcode (MiniMax Code CLI) 实际签名 / 探测命令未在本机验证 (没装) | P0 | 阻塞 AC-1 mcode 行 | 用户装 mcode 后验证; 不装 mcode 也能标 🔴 poisoned |
| #2 antigravity (agy) 1.0.6 的 `--model` flag 行为未在本机验证 | P1 | 阻塞 AC-1 antigravity 行 | 装 agy 1.0.6+ 验证 |
| #3 codex Desktop app bundle 路径探测 (macOS only) 未在本机验证 | P1 | 阻塞 AC-1 codex 行 (macOS) | macOS 装 codex Desktop 验证 |
| #4 automation console "环境扫描" 页 frontend 未实装 (per 守门 #9 v3 调试控制台扩展待) | P0 | 阻塞 AC-8 | 跟 v32 一起落 |
| #5 周报 "🔴 poisoned 累计" cron 任务未配 | P2 | 阻塞 AC-8 周报 | 配 cron (per 守门 v29 docs 同步饱和协调) |
| #6 agent-side probe (e.g. Mavis 自己跑着时探测本机其他 CLI) 跟 session 内 probe 区分 | P1 | 不阻塞, 但用户体验 | 当前 Mavis 跑 mcode, 探测结果 = mcode + 其他; 不冲突 |
| #7 nvm / fnm shim 探测跟 mavis 自身 mcode 冲突 (mcode 也是 npm global) | P1 | 不阻塞, 但探测逻辑要细心 | probe 顺序: 直接 PATH → shell → app bundle, mcode 走 PATH 优先 |

---

## §9 关联文档

| 类型 | 文档 | 关系 |
|---|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 | 上位, 拍板路径 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.1 | v32 候选详细表 |
| BD | [`docs/design/BD-MULTICA-RUNTIME-001.md`](../design/BD-MULTICA-RUNTIME-001.md) | 下游 (下个 turn 落档) |
| AGENTS.md | §4 守门 26 项 + §4.1 派生 v1-v26 | 守门合规 |
| automation-design | [`docs/automation-design.md`](../automation-design.md) v0.1+ §0.2 ADR-0026 引用 | 自动化档范式 |
| WBS | [`STAR-P3-WBS-001.md`](../../STAR-P3-WBS-001.md) §1-§5 / §14 | 任务卡标 [P] |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 Mavis 临时代签 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（28 FR / 6 NFR / 7 已知缺口 + 5 角色签字栏） | 2026-09-11 20:10 JST Ulysses 拍板"multica 的核心功能我原则上都要有" + 20:43 JST ask_user 选项 form_opt2 + inventory_opt1 + code_opt1 |
