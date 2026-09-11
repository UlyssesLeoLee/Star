# Multica 模式级参考 — 缺失功能多维统计 (inventory)

> **文档版本**: v0.1 (2026-09-11)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> **触发**: 2026-09-11 20:10 JST Ulysses 拍板"multica 的核心功能我原则上都要有" + 20:43 JST ask_user 选项 form_opt2 (分专题) + inventory_opt1 (文档内 + 独立 inventory) + code_opt1 (只出文档)
> **依赖**: [ADR-0026 Managed Agents Runtime 模式参考 — 借鉴 Multica](../adr/0026-multica-patterns-borrow.md) v0.2
> **关联**: 5 专题 SRS (`docs/requirements/SRS-MULTICA-*-001.md`) + 5 专题 BD (`docs/design/BD-MULTICA-*-001.md`, 下个 turn 落档)
> **受众**: 5 域 Lead 真人 / DDD Review / SRE Lead / Mavis 自驱排期

---

## 0. 文档目的

把 Multica 6 大核心功能在 STAR / Mavis 当前现状下的 **缺失度 / 落地候选 / 优先级 / 工作量估** 落到一张 4 象限表 + 2 张附表，让 v32 / v33 / v34 / v35+ 候选的拍板有实证基础。

---

## 1. 4 象限总表 (per 守门 #11 缺标比错标)

| 象限 | 含义 | Multica 模式 | STAR 现状 | 落地候选 | 优先级 | 工作量估 |
|---|---|---|---|---|---|---|
| **Q1 必做** | 跟守门 #9 v27 RPC fallback 实证 + 守门 #1 v15 死循环饱和核心痛点直接对齐 | 5 态状态机 + (agent, issue) session poisoning + review gate | WBS 4 态 + subagent "succeeded" 不可信 | v33 (WBS 5 态 + session poison 标记) | **P0** | 2-3 session |
| **Q1 必做** | 25 provider probe + min version gate + login shell 兜底 | daemon PATH 扫描 + sentinel error 区分 | Mavis 不知道自己跑哪个 CLI + 不可观测 | v32 (Runtime scan + poisoned) | **P0** | 1-2 session |
| **Q1 必做** | create_issue vs run_only 双模式 + rule versioning + admission check | autopilot 升级 | cron_create 跑完发结果不入流 | v34 (autopilot 升级) | **P0** | 2-3 session |
| **Q2 后续做** | Skill 跨 subagent 复用 + SKILL.md 模板 + local vs server skill 分离 | skill compounding | `docs/skills/` 半自动 + `scripts/automation/` 复用 | **v35+ 候选** (新增, ADR-0026 v0.2 未列) | **P1** | 3-4 session |
| **Q2 后续做** | 4 类触发中除 assign 外的 3 类 (@-mention / chat / autopilot) | 4 触发统一 | 缺 Mavis 跟外部 comment / chat 集成 | **v36+ 候选** (新增) | **P1** | 3-4 session |
| **Q2 后续做** | Idempotency key (60s window) + 审计 trail | autopilot idempotency | 重试可能双跑 | **v37+ 候选** (跟 v34 配套) | **P1** | 1 session |
| **Q3 评估中** | WebSocket event push (real-time UI) | WSRPC daemon ↔ server | 用 polling + console 替代 | 评估中, per 守门 #9 v24 v2 调试控制台 | **P2** | 1-2 session |
| **Q3 评估中** | Squad (leader agent 路由) | squad | 5 域 Lead 真人 + Mavis 全权代理 | 不做 (per ADR-0026 §2.2) | — | — |
| **Q3 评估中** | WebSocket heartbeat 15s + 3 档 liveness | daemon ↔ server 实时心跳 | session 模型短连接不需要 | 不做 (per ADR-0026 §2.2) | — | — |
| **Q3 评估中** | 13 类触发 + 4 模式 run_only admission check (per MUL-1899) | autopilot run_only offline 即 skipped | cron 离线时堆积任务 | 跟 v34 一并落 | — | — |
| **Q4 不做** | Multica server (Go + PostgreSQL 17) | server 端 | 跟 Mavis root session 模型冲突 | 不做 (per ADR-0026 §2.2) | — | — |
| **Q4 不做** | Multica UI (Next.js Kanban) | UI | 跟 WBS markdown 模型冲突 | 不做 | — | — |
| **Q4 不做** | Workspace × Tool × Daemon 正交 runtime | runtime 抽象 | 跟 Mavis 单 session 单 CLI 简单模型冲突 | 不做 | — | — |
| **Q4 不做** | Multi-tenant role matrix | 权限 | 跟一人公司 12 角色冲突 | 不做 | — | — |
| **Q4 不做** | Multica daemon (Go 编译产物) | 本机代理 | 跟守门 #6 PowerShell only + 守门 #5 env 安全冲突 | 不做 | — | — |

---

## 2. 候选详细表 (Q1 + Q2 必做 + 后续做)

### 2.1 v32 — Runtime Scan + Poisoned 语义 (P0, 1-2 session)

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| 25 provider 探测表 | claude/codex/mcode/... 全 25 个 + BuiltinRuntimes 派生 | ADR-0026 §1.1 + agents_probe.go:155-306 | `scripts/automation/registry/scan.py` |
| Login shell 兜底 | macOS GUI daemon PATH 兜底, 30min TTL 缓存 | agents_probe.go:30-50 | scan.py `_resolve_via_shell()` |
| MinVersion gate | 8 provider 最低 semver + sentinel error 区分 | version.go:13-22, 161-178 | scan.py `_check_min_version()` |
| 三档 status | 🟢 active / 🟡 stale (auth 失效) / 🔴 poisoned (探测到但 unusable) | 综合多源 | scan.py 输出 `RuntimeStatus` |
| Automation console 扩展页 | "环境扫描"页可视化 | 守门 #9 v3 调试控制台 | `frontend/src/app/automation-debug/runtime-scan/page.tsx` |
| 落档 | `docs/reports/runtime-scan/<date>.log` | 守门 #1 v15 docs 同步 | scan.py `--audit-log` |

### 2.2 v33 — WBS 5 态状态机 + Session Poisoning (P0, 2-3 session)

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| 4 态 → 5 态扩展 | pending → claimed → in_progress → completed / failed (+ cancelled 保留) | ADR-0026 §1.2 + 守门 #9 v27 RPC fallback | `docs/STAR-P3-WBS-001.md` §状态定义 |
| Session poison 标记 | 4 原因分类: IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized | poisoned.go:10-217 | `scripts/automation/wbs_migrate_v33.py` + WBS row 加 `session_poisoned: bool` |
| WBS 迁移脚本 | 现有 41 子项 status 字段升级 (pending → pending / claimed) | 守门 #1 v15 | `scripts/automation/wbs_migrate_v33.py` |
| Review gate | subagent complete → Mavis review → 才进 done | Multica 跟守门 #9 v27 一致 | `scripts/automation/subagent_review.py` |
| 4 类 404 区分 | workspace / task / runtime / unauthorized | client.go:35-91 | automation console 显示 4 类删除事件 |
| WBS 字段表 | W-T-M 100% 覆盖 (per 守门 #13) | 守门 #13 | WBS row schema 升级 |

### 2.3 v34 — Autopilot 升级 (P0, 2-3 session)

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| 3 触发类型 | cron / webhook / manual | autopilot.go:101-150 | `scripts/automation/cron_with_card.py` |
| 2 模式 | create_issue (持久审计, offline 也建) vs run_only (offline 即 skipped) | autopilot.go:101-150 | `scripts/automation/autopilot_dispatch.py` |
| Admission check | run_only 触发时, 校验 agent runtime 在线, 离线即 `skipped` | autopilot.go:106-114 | autopilot_dispatch.py `_admit_check()` |
| Idempotency key | 60s 窗口去重 | autopilot.go:51 (`autopilotRecentDuplicateWindow`) | autopilot_dispatch.py |
| Rule versioning | 每次 publish / trigger 改都 +1 版本 (accountability) | autopilot.go:79-99 | WBS row 加 `rule_version: int` |
| 跟守门 #1 v15 协调 | autopilot 跑完必落档 + docs 同步, 触达饱和前必新事件触发 | 守门 v29 | autopilot_dispatch.py `--audit-log` |

### 2.4 v35+ 候选 (P1, 3-4 session) — Skill Compounding

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| Skill 生命周期 | create → store (server) → discover (search) → reuse (load SKILL.md) | Multica skill_create.go + skill_refresh.go | `docs/skills/<skill-name>/SKILL.md` + registry |
| SKILL.md 模板 | name / description / when-to-use / steps / tools | Multica `server/pkg/skill/*.go` | 跟现有 `docs/skills/` 对齐 |
| Local vs server skill 分离 | local = 本机用户, server = 共享, workspace = 项目级 | Multica runtime_local_skills_redis_store.go | automation-design.md §3.4 |
| Skill search | embedding-based 检索 (per Multica pgvector) | Multica PostgreSQL 17 + pgvector | 跟 STAR 知识图谱集成 |
| Skill version | 每次更新 +1 version, 旧 version 留 30 天 | Multica skill_create.go | registry 加 `version: int` |

### 2.5 v36+ 候选 (P1, 3-4 session) — 4 类触发补全

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| @-mention 触发 | comment 里 @Mavis → 自动 dispatch subagent | Multica issue.go + comment.go (MUL-1899) | `scripts/automation/mention_trigger.py` |
| Chat 触发 | 直接跟 Mavis 对话 → 自动派任务 | Multica chat.go | automation console chat 面板 |
| Assign 触发 (已有) | Mavis 接到 task card → 自动 claim → start | 现有 dispatcher.py | 增强 dispatcher |
| Autopilot 触发 | v34 落地后整合到 dispatcher | v34 | dispatcher.py 整合 |

### 2.6 v37+ 候选 (P1, 1 session) — Idempotency

| 项 | 内容 | 关联 | 落地位置 |
|---|---|---|---|
| Idempotency key 生成 | 60s 窗口内同 (trigger, source, payload_hash) 去重 | autopilot.go:51 | dispatcher.py |
| 双跑保护 | retry 场景下, 同一 idempotency key 的 task 不会重复执行 | 守门 #1 + #9 | dispatcher.py |
| 审计 trail | idempotency key 写进 WBS row + commit message | 守门 #1 v15 | dispatcher.py --audit-log |

---

## 3. 不参考附表 (Q4 不做, per ADR-0026 v0.2 §2.2)

| Multica 功能 | 不做原因 | 替代方案 |
|---|---|---|
| Multica server (Go + PostgreSQL 17 + pgvector) | 跟 Mavis root session 模型冲突；多一层 server/DB 维护成本 | 直接用 Mavis session + automation console |
| Multica UI (Next.js Kanban) | 跟 WBS markdown 模型冲突 | 维持 markdown WBS |
| Workspace × Tool × Daemon 正交 runtime | 跟 Mavis 单 session 单 CLI 简单模型冲突；过设计 | 一台机器 + 一个 session + 一个 CLI |
| Squad / Leader agent 路由 | 跟 5 域 Lead 真人代签 + Mavis 全权代理冲突 | Mavis 直接处理 |
| WebSocket 心跳 + 3 档 liveness | session 模型短连接不需要 | 不需要 liveness；session start/end 即心跳 |
| Multi-tenant role matrix | 一人公司 12 角色不需要再细分 (per ADR-0021) | 维持 12 角色 |
| Multica daemon (Go 编译产物) | 跟守门 #6 PowerShell only + 守门 #5 env 安全冲突 | 用 Python automation console + 守门 #6 兼容 |

---

## 4. 拍板建议 (per 守门 v28 + 9/1 14:58 + 9/5 04:03)

| 候选 | 推荐顺序 | 触发条件 | 备注 |
|---|---|---|---|
| **v32** | **第 1 批** (P0, 1-2 session) | 用户下次发令启 v32 | 落 `scripts/automation/registry/scan.py` + console 扩展 |
| **v33** | **第 2 批** (P0, 2-3 session) | v32 落档 + 用户启 v33 | 落 WBS 5 态 + session poison 标记 + 迁移脚本 |
| **v34** | **第 3 批** (P0, 2-3 session) | v33 落档 + 用户启 v34 | 落 autopilot_dispatch.py + 跟 cron 整合 |
| **v35+** | **第 4 批** (P1, 3-4 session) | v34 落档 + 用户启 v35 | skill lifecycle + SKILL.md 模板 |
| **v36+** | **第 5 批** (P1, 3-4 session) | 跟 v34 配套 | @-mention + chat 触发 |
| **v37+** | **第 6 批** (P1, 1 session) | 跟 v34 配套 | idempotency key |

**总工作量估**: 12-17 session (P0 必做 = 5-8 session + P1 后续 = 7-9 session)

**P0 必做 (5-8 session) 完成 = Multica 核心模式级 100% 落地**

---

## 5. 跟守门核对 (per AGENTS.md §4)

| 守门 | 关系 | 备注 |
|---|---|---|
| #1 v15 死循环饱和 | docs 同步 commit 必新事件触发 | 本 inventory 落档 = 用户拍板"统计其他缺失"= 新事件 |
| #5 env 安全 | 不读 env 值 | scan.py 探针 = exec + LookPath, 不读 secret |
| #6 PowerShell only | subprocess shell=False | scan.py 走 subprocess.run(shell=False) |
| #9 v27 RPC fallback | subagent 必 verify | 本工作不派 subagent, root 直接实装 |
| #11 缺标比错标 | 已知缺口显式列 | §3 显式列"不做" |
| #12 v21 [P] docs 同步 | 本 inventory 是 [P] | commit 引用 ADR-0026 + automation-design.md |
| #14 v4 Mavis 审核 | author=Ulysses | 本 inventory 修订人 = Ulysses, 审批 = 架构师 (Mavis 接手) |
| v28 ask_user 必带推荐项 | 选项 path_opt2 + inventory_opt1 + code_opt1 都带 (推荐) 标 | 20:43 JST 拍板 |
| v29 docs 同步饱和 | 本次是 48 次 docs 同步 commit (47 prior + 1 this) | 仍 < 50 阈值, 阻断不会触发; 但下次大批量前必 ask |

---

## 6. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（4 象限总表 + 6 候选详细表 + 7 不参考附表 + 拍板建议 + 守门核对）| 2026-09-11 20:10 JST Ulysses 拍板"multica 的核心功能我原则上都要有" + 20:43 JST ask_user 选项 form_opt2 + inventory_opt1 + code_opt1 |
