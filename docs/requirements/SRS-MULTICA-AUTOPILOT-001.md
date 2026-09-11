# SRS-MULTICA-AUTOPILOT-001

> **Multica Autopilot 域要件定義書 v0.1** (per ADR-0026 v0.2 §2.1 模式 3, 跟 v34 候选对齐)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-AUTOPILOT-001.md`](../design/BD-MULTICA-AUTOPILOT-001.md) (下个 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.3 (v34 候选)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> - 日期: 2026-09-11 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-AUTOPILOT-001 |
| 文书名 | Multica Autopilot 域要件定義書 (v34 候选对齐) |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.2 + §2.1 模式 3 |
| 关联 inventory | inventory §2.3 v34 候选 |
| 上位要件 | SRS-MULTICA-RUNTIME-001 + SRS-MULTICA-TASK-001 |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 5 子能力 (AP-1 ~ AP-5) × 24 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.2 痛点 + §2.1 模式 3 (autopilot 升级), 定义 STAR 平台 **Multica Autopilot 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: 现有 `cron_create` 任务升级为 autopilot —— 触发时建 WBS row + 指派 subagent + 跑任务 + 收 output; **create_issue vs run_only 双模式** 区分 (per Multica autopilot.go:101-150)。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 4 类具体痛点:

1. **定期任务不入工作流** — `cron_create` 跑完发结果, 跟 WBS 任务卡断开
2. **Cron 离线时堆积** — runtime 离线时, 任务仍派, 堆积在 `task_queue` (per Multica MUL-1899)
3. **重复触发没去重** — 同一 cron 在 60s 内多次触发, 多次派 task (per Multica `autopilotRecentDuplicateWindow = 60s`)
4. **Autopilot 改配置不可追溯** — 谁改的, 改了什么, 没 accountability (per Multica MUL-4302 §3.4)

### 1.3 包含范围 (In-Scope)

5 子能力 (per Multica autopilot.go line refs):

| 子能力 | Multica 源 | 关键 file:line |
|---|---|---|
| AP-1 3 触发类型 | autopilot.go (cron / webhook / manual) | 3 dispatcher 入口 |
| AP-2 2 模式 (create_issue / run_only) | autopilot.go:101-150 (DispatchAutopilot) | 双模式分流 |
| AP-3 Admission check (run_only 离线即 skipped) | autopilot.go:106-114 (MUL-1899) | runtime 不在线就 skipped |
| AP-4 Idempotency key (60s 去重) | autopilot.go:51 (`autopilotRecentDuplicateWindow`) | 60s 窗口去重 |
| AP-5 Rule versioning (accountability) | autopilot.go:79-99 (RecordAutopilotRuleVersion) | 每次 publish +1 version |

### 1.4 不含范围 (Out-of-Scope, per ADR-0026 v0.2 §2.2)

- ❌ Multica assignee_type 'squad' 路径 (per autopilot.go:115-118 注释)
- ❌ Webhook durable delivery (per autopilot.go:132 注释, AdmitAutopilotWebhookDelivery 不在本 SRS)
- ❌ MUL-6951 trigger_owner 复杂归属 (Mavis 单一 root session 不需要)

### 1.5 受众范围 disclaimer

- 本 SRS 跟 `automation-design.md` v0.1 §3.2 (CLI 调用范式) 平行
- autopilot_dispatch.py 落档后跟 `cron_create` 现有 task 兼容 (cron_create 内部调 autopilot_dispatch)

---

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Trigger** | autopilot 触发源, 3 类: cron / webhook / manual |
| **Execution mode** | 2 类: `create_issue` (持久审计, offline 也建) / `run_only` (offline 即 skipped) |
| **Admission check** | run_only 触发时校验 agent runtime 在线, 离线即 `skipped` |
| **Idempotency key** | 60s 窗口内同 (trigger, source, payload_hash) 去重, key = `source + ":" + newAutopilotIdempotencyKey()` |
| **Rule version** | autopilot 配置每次 publish / trigger edit / archive +1, accountability trail |
| **Skipped** | run_only 模式 + agent runtime 离线 → 不派 task, 标 skipped + failure_reason |
| **Triggered actor** | 触发者: scheduled / webhook / api / manual, manual 时是 direct_human (MUL-4302 §4) |
| **Per-run reason code** | 跳过的具体原因 (Mavis 不显示 per-run reason, 仅 aggregate report) |

---

## §3 业务背景

### 3.1 Multica 实证 (per ADR-0026 v0.2 §1.3 + §2.1 模式 3)

| 文件 | 关键 | 引用 |
|---|---|---|
| `autopilot.go:101-150` | `DispatchAutopilot` 入口 + create_issue vs run_only 分流 | AP-1, AP-2, AP-3 |
| `autopilot.go:106-114` | Admission check + 离线 skipped (MUL-1899) | AP-3 |
| `autopilot.go:51` | `autopilotRecentDuplicateWindow = 60s` | AP-4 |
| `autopilot.go:79-99` | `RecordAutopilotRuleVersion` accountability | AP-5 |
| `autopilot.go:53` | `NewAutopilotService` (Queries / TxStarter / Bus / TaskSvc / Entitlements) | 配套 (不引入 Multica 整套 service) |

### 3.2 拍板来源

- 2026-09-11 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 + inventory_opt1 + code_opt1

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发)
- 守门 #5 (env 安全)
- 守门 #6 (PowerShell only)
- 守门 #9 v27 (RPC fallback) — autopilot 跑挂可强制 skip
- 守门 #11 (缺标比错标)
- 守门 #12 v21 ([P] docs 同步)
- 守门 #14 v4 (Mavis 审核)
- 守门 v29 (docs 同步饱和协调) — autopilot 跑完必落档

---

## §4 功能需求 (FR, 24 项)

### AP-1 3 触发类型 (FR-1 ~ FR-6)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-1 | `cron` 触发: crontab expression + 时区, 默认 UTC | P0 |
| FR-2 | `webhook` 触发: HTTP POST endpoint + payload 解析 | P0 |
| FR-3 | `manual` 触发: Mavis 主动按按钮 "Run now" | P0 |
| FR-4 | 3 触发共用同一 dispatcher 入口 `autopilot_dispatch.py` | P0 |
| FR-5 | 触发配置 (cron / webhook payload schema) 落 WBS row + commit message (per 守门 #1 v15) | P0 |
| FR-6 | 触发失败重试: 默认 3 次, 指数退避 | P0 |

### AP-2 2 模式 (FR-7 ~ FR-12)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-7 | `create_issue` 模式: autopilot 触发时建 WBS row + 指派 subagent + 跑任务 + 收 output (跟 Mavis 当前 root session 行为一致) | P0 |
| FR-8 | `create_issue` 模式: 即使 agent runtime offline, 仍建 WBS row, 等 runtime 上线再 claim | P0 |
| FR-9 | `run_only` 模式: autopilot 触发时直接派给 subagent, 不建 WBS row | P0 |
| FR-10 | `run_only` 模式: agent runtime offline 即 `skipped` + failure_reason, 不堆积任务 (per MUL-1899) | P0 |
| FR-11 | 模式选择: 创建 autopilot 时配置, 不可运行时改 (避免误操作) | P0 |
| FR-12 | 模式行为跟 Multica autopilot.go:101-150 一致 (line refs 在 BD-MULTICA-AUTOPILOT-001 §3 详) | P0 |

### AP-3 Admission check (FR-13 ~ FR-16)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-13 | `run_only` 触发时校验 agent runtime 在线 (per Multica `dispatchAutopilot` admission) | P0 |
| FR-14 | 离线即 `skipped` + failure_reason=`agent_runtime_offline` | P0 |
| FR-15 | `create_issue` 模式**不**做 admission check, 持久审计 trail 优先 | P0 |
| FR-16 | admission check 状态写 audit log (per 守门 #1 v15) | P0 |

### AP-4 Idempotency key (FR-17 ~ FR-20)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-17 | 60s 窗口内同 (trigger, source, payload_hash) 去重, 复用 Multica `autopilotRecentDuplicateWindow = 60s` | P0 |
| FR-18 | 重复触发: 后到的触发返 `idempotent_replay: true` 标记, 不创建新 task | P0 |
| FR-19 | Idempotency key 写 commit message (per 守门 #1 v15 docs 同步) | P0 |
| FR-20 | Idempotency 窗口可配 (per WBS row 配置, 默认 60s) | P0 |

### AP-5 Rule versioning (FR-21 ~ FR-24)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-21 | autopilot 配置每次 publish / trigger edit / archive +1 version (per Multica MUL-4302 §3.4) | P0 |
| FR-22 | version snapshot 落 WBS row `autopilot_rule_version: int` 字段 | P0 |
| FR-23 | config 摘要 (assignee_type / assignee_id / status / execution_mode) 写 version snapshot (per Multica `autopilotRuleConfigSummary`) | P0 |
| FR-24 | version 不可删除, 跟守门 #11 缺标比错标一致 | P0 |

---

## §5 非功能需求 (NFR, 5 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | 单次 autopilot 触发派 task < 5s (含 idempotency 校验) |
| NFR-2 可靠性 | run_only 离线即 skipped, 不堆积 task |
| NFR-3 可观测 | 每次 autopilot 触发写 audit log, 含 trigger / mode / idempotency_key / version |
| NFR-4 易用 | 跟现有 `cron_create` 兼容, 内部调 autopilot_dispatch |
| NFR-5 安全 | webhook endpoint 必须有 HMAC 签名验证 (per 守门 #5 env 安全: secret 不打印) |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #5 env 安全 | webhook secret 不打印, HMAC 验证 |
| 守门 #6 PowerShell only | subprocess.run(shell=False) |
| 守门 #11 缺标比错标 | rule version 不删, admission 失败标 skipped |
| 守门 #12 v21 [P] docs 同步 | 每次 autopilot 跑完落档 |
| 守门 v29 docs 同步饱和 | autopilot 跑挂必落档, 防止 commit-time 死循环 |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Webhook 重复触发 → 多 task | 中 | 中 | Idempotency key 60s 窗口 |
| Cron 离线时任务堆积 (run_only) | 中 | 高 | Admission check skipped |
| 9/1 14:58 决策流冲突: autopilot 改 execution_mode 是微决策还是方向选择 | 中 | 中 | 微决策 (per 9/8 15:29 自驱) 不需要 ask_user, Mavis 自决 |
| Rule version 增长失控 | 低 | 低 | 旧 version 30 天后归档 (per Multica 默认) |

---

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | 3 触发类型 (cron / webhook / manual) 全部跑通, 各 1 case |
| AC-2 | create_issue / run_only 双模式分流正确, run_only 离线即 skipped |
| AC-3 | Idempotency 60s 窗口去重, 同 key 第二次返 idempotent_replay=true |
| AC-4 | Rule version 每次配置改 +1, snapshot 落 WBS row |
| AC-5 | Webhook HMAC 签名验证, secret 不打印 (per 守门 #5) |
| AC-6 | 跟现有 cron_create 兼容, 旧任务不挂 |
| AC-7 | audit log 每次 autopilot 跑完 +1 entry |
| AC-8 | 守门 #13 W-T-M 100% 覆盖 (autopilot rule = Master / run = Transaction / idempotency window = Work) |

---

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 Webhook durable delivery 暂未实装 (per Multica `AdmitAutopilotWebhookDelivery` 不在本 SRS) | P1 | 不阻塞 (manual trigger 优先) | 后续 v38+ 候选 |
| #2 `squad` assignee_type 不支持 (per ADR-0026 §2.2 不参考) | P0 | 不阻塞 (Mavis 单一 root session) | 文档加 disclaimer |
| #3 Cron 时区支持仅 UTC, 不支持用户时区 | P1 | 不阻塞 (per Multica `DefaultAutopilotTriggerTimezone = "UTC"`) | 后续 v38+ 候选 |
| #4 Idempotency 窗口 60s 太短 (用户配长一些更安全) | P2 | 不阻塞 | 配可调 (per FR-20) |
| #5 Rule version 缺 rollback (per Multica 不支持 rollback) | P2 | 不阻塞 | 改 = 新 version, 旧 version 留 |
| #6 Webhook secret 存储用什么 backend? (env / vault / KMS) | P0 | 阻塞 AC-5 | 跟 v34 一起落: 暂用 env (per Multica `agent_env.go`), 后续 v38+ 接 KMS |
| #7 Idempotency key 跟守门 v27 RPC fallback 协调 (v27 已 retry 2 次, idempotency 防双跑) | P0 | 阻塞 AC-3 | dispatcher.py 集成时统一处理 |

---

## §9 关联文档

| 类型 | 文档 |
|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.2 + §2.1 模式 3 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.3 v34 候选 |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-RUNTIME-001.md`](../requirements/SRS-MULTICA-RUNTIME-001.md) (Runtime Registry) |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-TASK-001.md`](../requirements/SRS-MULTICA-TASK-001.md) (Task Lifecycle) |
| BD | [`docs/design/BD-MULTICA-AUTOPILOT-001.md`](../design/BD-MULTICA-AUTOPILOT-001.md) (下个 turn 落档) |
| automation-design | [`docs/automation-design.md`](../automation-design.md) v0.1+ §3.1 dispatcher 范式 |
| WBS | [`STAR-P3-WBS-001.md`](../../STAR-P3-WBS-001.md) 状态定义 + rule_version 字段 |
| 守门 | AGENTS.md §4 + §4.1 #1 v15 / #5 / #6 / #9 v27 / #11 / #13 |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（24 FR / 5 NFR / 7 已知缺口 + 5 角色签字栏） | 2026-09-11 20:43 JST ask_user 选项 form_opt2 |
