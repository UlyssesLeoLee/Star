# DDD-DOMAIN-MODEL-COMPLETENESS-AUDIT

> **STAR 域模型完备度审计报告 v0.1**
>
> - 状态: 🟢 审计完成（41/41 crate 判定 + 清单对齐）；0 代码改动
> - 触发: 2026-10-25 用户提问「各种 domain model 都齐全么?」
> - 范围: `crates/domain-*` 全部 **41** 个 crate + 24 个非 `domain-*` 域职责 crate + 设计清单 ↔ 代码 ↔ 表清单三向对齐
> - 证据来源: 4 份 brief + 4 路 explore 子代理判定（全部带 `file:line`）；父会话独立复核 9 条关键结论
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-10-25

---

## §0 目的

回答「Star 的域模型层是否齐全」，并给出**逐 crate 可复算**的完备度矩阵与按优先级排序的缺口清单。

**本报告不做的事**（per 用户 2026-10-25 拍板 `ask_b66403e46d985090ade079d4`）：

- **只补文档，不动其他** —— 不改代码、不改表结构、不改分类归属
- `docs/ddd/01-05` 的 5 域 → crate 映射表**不动**，仅在 DDD Review 报告登记冲突（per 守门 #3 拍板约束的合规处理方式）

---

## §1 审计方法与判定口径

### 1.1 判定 rubric（4 份 brief 统一，见 `docs/briefs/DDD-AUDIT-01..04.md`）

| 维度 | 含义 |
|---|---|
| R1 实体 / 值对象 | 承载业务语义的 `struct`/`enum`（强类型 ID newtype、值对象），非纯传输 DTO |
| R2 领域不变量 | 存在校验 / 状态机 / 派生规则等**行为**方法 |
| R3 领域契约 | 存在 `pub trait`（repository port / domain service / policy） |
| R4 领域事件 | 存在 domain event 类型或 outbox 事件载荷 |
| R5 不变量测试 | 有覆盖不变量的单元测试 |
| R6 非骨架自述 | crate 内无「阶段 1 骨架 / 0 业务方法 / 占位 / 仅 mock / TODO 实装 / 留 P1」自述 |

分类规则：`齐全` = R1..R5 全命中且 R6=N；`部分` = 命中 3-4 项；`骨架` = 命中 ≤2 项或 R6=Y。

### 1.2 R6 判定口径（子代理自报，父会话已采信）

只在 **crate 级自述**（crate doc / `Cargo.toml` / 模块 doc）声称**自身业务面**为骨架/占位时记 Y；单个 helper 的占位、Noop 默认实现、历史 changelog 行**不计**。

> 该口径由 DDD-AUDIT-01 子代理提出并说明理由：字面全量扫描会把 `domain-agent`（9690 行实代码，仅因 `NoopAuditSink` 一处注释）误判为骨架，与事实相悖。

### 1.3 复算命令（可重放）

```powershell
# crate 规模指标
Get-ChildItem crates/domain-* -Directory | ForEach-Object { ... LOC / #[test] 计数 ... }
# 表宇宙交集（§4 复算）
$rows = Get-Content docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md |
        Where-Object { $_ -match '^\|\s*T\d+\s*\|' }   # §2 区間 L67-L302 限定
```

---

## §2 完备度矩阵（41/41）

✅ = 父会话**独立复核**过该条结论。

| # | crate | LOC | 测试 | 分类 | 关键证据 | 缺口要点 |
|---|---|---|---|---|---|---|
| 1 | `domain-agent` | 10335 | 142 | **齐全** | 强类型 ID `lib.rs:68` + 14 状态机 `lib.rs:177` + 11 trait + `AuditEventSpec` `policy_hooks.rs:173` | R4 弱：无 `star.events.*` subject，与 audit/batch/comment 事件契约不一致 |
| 2 | `domain-audit` | 1651 | 12 | **齐全** | 4 不变量 `lib.rs:431-504` + `AuditEventKind` `lib.rs:644` | `InMemoryAuditRecorder.record_ai` 未实装 `lib.rs:1299`（测试替身，主路径已实现 `lib.rs:984`） |
| 3 | `domain-automation` | 4024 | 30 | **齐全** | `AutomationRule` `lib.rs:387` + 6 不变量 `lib.rs:510-537` + `AutomationEvent` `lib.rs:611` | — |
| 4 | `domain-cli` | 4343 | 103 | **齐全** | `ApiKey`/`CliProfile` + AES 加密 `lib.rs:648` + `inv_01~04` `lib.rs:688` + `AuditSink` `api_monitor.rs:181` | 内含 HTTP 适配器（`hermes_client.rs` / `openclaw_client.rs`），边界混合 |
| 5 | `domain-comment` | 3670 | 38 | **齐全** | `Comment` `entity.rs:21` + 5 类事件 `event.rs:79` + 7 端口 `port.rs:79/124/148` | `ALL_INVARIANT_CHECKS` 空切片 `invariants.rs:125`；INV-C-02/04/06 空实现 `invariants.rs:37/64/90` |
| 6 | `domain-integration` | 3514 | 32 | **齐全** | 6 端口 + 事件载荷 + 完整服务层 | `adapter.rs` + `confluence.rs`（747 行）适配代码混在域 crate 内 |
| 7 | `domain-scm` | 3724 | 29 | **齐全** | 8 态 PR 机 + 8 不变量 + `ScmEvent` `lib.rs:860` + 3 port | — |
| 8 | `domain-validation` | 2853 | 23 | **齐全** | 10 不变量 `invariants.rs:18-202` + 6 态机 + `ValidationEvent` `event.rs:113` + 3 port | — |
| 9 | `domain-workspace` | 1364 | 14 | **齐全** | 4 不变量 + `WorkspaceEvent` `lib.rs:407` + 2 port | — |
| 10 | `domain-agent-windows` | 1570 | 31 | 部分 | Window/Tab 聚合 `lib.rs:94/119` + `inv_01~04` `lib.rs:451` | 无域事件；`CliPort` 全仓 0 实现（仅 trait `lib.rs:295`） |
| 11 | `domain-ai` | 488 | 8 | 部分 | 值对象 `lib.rs:24/46/71/143` + 端口 `lib.rs:173` | `AiError` 4 变体 `lib.rs:287-300` **从未被构造**（`Err(` 0 次）；仅 `MockLlmProvider` `lib.rs:310` |
| 12 | `domain-board` | 2512 | 26 | 部分 | `Board` 聚合 `lib.rs:284` + `WipGuard` `wip_swimlane.rs:39` + 3 端口 | 无域事件；同 crate 类型重复（`Swimlane` `lib.rs:235` vs `wip_swimlane.rs:89`；两套 `BoardService`） |
| 13 | `domain-collaboration` | 2140 | 17 | 部分 | Session/Presence/Whiteboard `lib.rs:209/288/471` + `assert_tenant` `lib.rs:257` | 无域事件 |
| 14 | `domain-context` | 1645 | 14 | 部分 | `ContextPacket` `lib.rs:182` + `verify_packet` `lib.rs:682` + 3 端口 | 无域事件 |
| 15 | `domain-dashboard` | 391 | 7 | 部分 | `Dashboard` 聚合 `lib.rs:129` + 上限 20 `lib.rs:173` + Global 需 admin `lib.rs:251` | **0 个 `pub trait`**、无事件 |
| 16 | `domain-development` | 1801 | 22 | 部分 | `ChangeSet` `lib.rs:97` + `can_transition_to` `lib.rs:208` + 3 端口 | 无域事件 |
| 17 | `domain-form` | 592 | 11 | 部分 | 实体/校验 `lib.rs:298,379` | **0 个 `pub trait`**、无事件；`rate_limit_per_hour` 无执行逻辑 `lib.rs:248` |
| 18 | `domain-hook` | 1237 | 17 | 部分 | fail-closed 判定在域内 `lib.rs:548-687` | **0 个 `pub trait`**（策略来源/发布无契约）、无事件 |
| 19 | `domain-identity` | 1227 | 14 | 部分 | User/Device/Credential 实体 + 3 端口 | **无凭证签发/校验路径** → INV-ID-03「绝不存明文」不可执行（`secret_hash: String` `lib.rs:250` 无格式校验） |
| 20 | `domain-infrastructure` | 2610 | 29 | 部分 | 容量 `capacity.rs:395` + 就绪 `readiness.rs:307` + outbox 载荷 `lib.rs:469` | 无 port trait（host/cluster 预留无契约） |
| 21 | `domain-notification` | 2298 | 39 | 部分 | 抑制/重试/DLQ `lib.rs:124,140,630,727` | 仅判别枚举 `lib.rs:55`，无事件载荷 |
| 22 | `domain-permission` | 2232 | 22 | 部分 | `decide()` 授权判定在域内 `lib.rs:645-736` | 无领域事件 / 审计载荷 |
| 23 | `domain-planning` | 1901 | 20 | 部分 | Sprint/Milestone/Capacity 状态机 + 重叠校验 `lib.rs:984` | 无领域事件 |
| 24 | `domain-project` | 970 | 16 | 部分 | 实体 + 策略 + 模板，slug 唯一 / 跨租户有测试 | 无领域事件 |
| 25 | `domain-relation` | 1713 | 16 | 部分 | 强类型 ID + 3 port | 全 crate 0 处 event |
| 26 | `domain-tenant` | 1087 | 13 | 部分 | 4 聚合根 + slug 唯一 / 跨租户不变量 `lib.rs:572,665` | 无事件类型 |
| 27 | `domain-work-item` | 3500 | 47 | 部分 | 三态机 + 3 port + 47 测试 | ✅ `lib.rs:667` 明写「不发送事件（避免污染 event bus）」 |
| 28 | `domain-workflow` | 1724 | 17 | 部分 | 严格转换表 + 5 不变量 + 3 port `lib.rs:401/451/489` | 无事件 |
| 29 | `domain-worktree` | 2859 | 59 | 部分 | 17 态机 + 5 port + 59 测试 | 无事件（`edit.rs:208` 仅为回执非事件） |
| 30 | `domain-batch` | 2506 | 20 | **骨架** | 模型层齐（Task/Dag + 11 事件 + Kahn 环检测） | 唯一端口实现是 `NoopBatchService`（`port.rs:404/513`，22 处 `not_implemented`）；`error.rs:97` 自述「P2 阶段未实装」；`invariant.rs:115` 12 条不变量仅 4 条实装 |
| 31 | `domain-canvas` | 174 | 5 | **骨架** | ✅ `Cargo.toml:13` 明写「阶段 1 骨架 … 占位 endpoint」 | 仅 healthz/version，无实体/端口/事件 |
| 32 | `domain-feedback` | 2420 | 18 | **骨架** | 域模型本完整 | ✅ `invariants.rs:10,16,24,38,81` 5 个 `*_placeholder` + `ALL_INVARIANT_CHECKS = &[]`（`invariants.rs:94`）→ **检查入口永不触发** |
| 33 | `domain-kms` | 371 | 3 | **骨架** | 租户隔离在域内 `lib.rs:324-329` | 唯一实现 `LocalMockKms` `lib.rs:188`；轮换周期不变量（master 90 天 / DEK 30 天）**无代码强制**，`rotate_dek` `lib.rs:350-362` 只生成新 key，actor 仅进 `warn!`，无审计事件 |
| 34 | `domain-llm` | 5625 | 126 | **骨架** | `openai.rs:321` + `anthropic.rs:333` 已实现 `LlmProvider` | **自述过期**：`lib.rs:9,14` + `chat.rs:6` 仍称「v0.0.1 stub / no real provider wiring」→ 文档与代码冲突 |
| 35 | `domain-local-runtime` | 11555 | 197 | **骨架** | 授权边界真在域内（`task_execution.rs:196,214` Ed25519 grant） | `cli_session_lock.rs:18` + `graceful_shutdown.rs:22-23,97,320,390` 留 P1/Windows stub；**约 2200 行属 OS/PTY/SSE/传输适配**，应迁 infrastructure |
| 36 | `domain-mcp` | 178 | 4 | **骨架** | 全文件自述 stub `lib.rs:9,14,84,87,89` | ✅ 存储 `HashMap<Uuid,String>` 裸串 `lib.rs:93`；✅ **全仓 0 依赖方**（仅自身 Cargo.toml 引用）；实现其实在 `crates/star-mcp`（10271 行 / 230 测试） |
| 37 | `domain-report` | 4079 | 68 | **骨架** | `lib.rs:5-8` 自述 6 P1 stub / 8 P2 stub / 13 路由 stub | 走 `NotImplemented` `lib.rs:406` / `generate_stub` `lib.rs:524`；仅 C01 落地 |
| 38 | `domain-search` | 2697 | 33 | **骨架** | `jql.rs:10` 「本任务 stub 形式」+ `jql.rs:11` 「真实数据接入 Phase 2」 | JQL 执行器跑内存行集 `jql.rs:444/447/461`；`lib.rs:238/254` 列号恒为 0 |
| 39 | `domain-task` | 178 | 4 | **骨架** | `lib.rs:14,89,117` 自述「v0.0.1 stub」 | 仅 `TaskQueue` trait + 内存 map，无业务实体、无状态机、无租户边界 |
| 40 | `domain-theme` | 1025 | 14 | **骨架** | — | ✅ `service.rs:32` `ThemeId::Light, // 占位, 真实应按"当前激活"字段查`；`:43` 无条件返回 Light → **三层解析链实际不生效** |
| 41 | `domain-tool` | 569 | 15 | **骨架** | `lib.rs:14,161,252,256`「v0.0.1 stub / Sprint 2 stub」 | `execute` 恒返 Backend 错误；实体层仅 `ToolExecutionMode` `traits.rs:135` |

### 2.1 统计

| 分类 | 数量 | 占比 |
|---|---|---|
| 齐全 | **9** | 22% |
| 部分 | **20** | 49% |
| 骨架 | **12** | 29% |

**结论：域模型层不齐 —— 41 个域中仅 9 个（22%）达到「齐全」，12 个（29%）实质是骨架。**

---

## §3 三类系统性缺口

### 3.1 缺口 A（影响面最大）：R4 领域事件缺失 —— 12 个 crate

`domain-work-item` ✅`lib.rs:667`、`domain-workflow`、`domain-tenant`、`domain-relation`、`domain-worktree`、`domain-board`、`domain-collaboration`、`domain-context`、`domain-development`、`domain-report`、`domain-search`、`domain-task`、`domain-tool`、`domain-agent-windows`、`domain-form`、`domain-hook`、`domain-identity`、`domain-infrastructure`、`domain-notification`、`domain-permission`、`domain-planning`、`domain-project` —— 这类 crate 的共同形态是**实体 + 不变量 + port 齐全，但不发事件**。

**后果**：域层没有事件源 → owner-local Outbox 无从写入 → BI / saga / 审计回执链在生产上接不上。这不是「补几个类型」的问题，是**域模型缺一个维度**。

**样板**（照抄即可）：`domain-scm/src/lib.rs:820-869`（事件 struct + 聚合 enum）、`domain-workspace/src/lib.rs:369-407`、`domain-comment/src/event.rs:79`。

### 3.2 缺口 B（最危险）：假安全感 —— 声明了但不执行的不变量

| crate | 现象 | 危害 |
|---|---|---|
| `domain-feedback` ✅ | `ALL_INVARIANT_CHECKS = &[]`（`invariants.rs:94`） | 批量不变量检查入口**永远不会触发**，审查时却显示"已实现" |
| `domain-comment` | `ALL_INVARIANT_CHECKS` 空切片 `invariants.rs:125` + INV-C-02/04/06 空实现 `:37/64/90` | 同上 |
| `domain-kms` ✅ | 轮换周期 INV-KMS-04（master 90 天 / DEK 30 天）无任何代码强制 | 密钥可无限期不轮换，且轮换无审计事件 |
| `domain-identity` ✅ | INV-ID-03「绝不存明文」只靠字段名 `secret_hash: String` 声明 | 域内无哈希、无格式校验 → 该不变量不可执行 |
| `domain-batch` | `invariant.rs:115` 12 条不变量仅 4 条实装 | 覆盖率 33% 却在表里以"12 条不变量"呈现 |

> 判别式：**一条不变量如果不能被"违反"检测到，它就不是不变量，是注释。** 空检查表比没有检查更危险 —— 它让审阅者认为已被守住。

### 3.3 缺口 C：孤儿 crate 与名实不符

| crate | 问题 | 证据 |
|---|---|---|
| `domain-mcp` ✅ | 全仓 0 依赖方；实现实际在 `crates/star-mcp` | 仅自身 Cargo.toml 引用 |
| `domain-task` | 与 `star-task` / `shared-task` / `star-taskqueue` 四者同名族并存 | — |
| `domain-llm` ✅ | 自述「no real provider wiring」但 provider 已实现 → 文档漂移 | `lib.rs:9,14` + `chat.rs:6` vs `provider/openai.rs:321` |
| `domain-batch` ✅ | `lib.rs:3` 指向的 `docs/specs/domain-batch-spec.md` **文件不存在** | 实测 False |
| `domain-ai` | spec 声明 `AiAgent`/`AiInvocation`/`AiFeedback` + 3 不变量，代码里只有 `AiRequest`/`AiResponse`/`ModelConfig`/`PromptTemplate` | `domain-ai-spec.md:30-32` |
| `domain-board` | 同 crate 内 `Swimlane`/`SwimlaneGroupBy`/`BoardService` 各两套并存 | `lib.rs:235` vs `wip_swimlane.rs:89` |

---

## §4 跨层断裂：表宇宙与分类宇宙**精确零交集**（✅ 父会话独立复算）

| 集合 | 数量 | 组织方式 |
|---|---|---|
| **生产表**：`db/migrations` 实建 | **175** | **技术性 schema**：`multica` 60 / `infrastructure` 19 / `automation` 11 / `platform_transport` 9 / `platform_messaging` 7 / `bi` 5 / `canvas` 5 / `plugin` 5 / `permission` 3 / `scm` 2 + 49 未限定 |
| **分类表**：`00-INVENTORY.md` / W-T-M 索引 | **100** | **DDD 域 schema**：`development` 9 / `scm` 8 / `validation` 6 / `agent` 5 / `work_item` 5 / `local_runtime` 5 / `identity` 5 / `worktree` 5 / `feedback` 4 … 共 25 schema |

**两集合的表名精确交集 = 0**（父会话用集合运算复算确认；先前子串匹配得到的 6/31 命中系 `agent_policies` ⊃ `agent_policy` 类假阳性，已排除）。

→ **守门 #13 声称的「100% 表覆盖」对生产表实际是 0%。** 该覆盖仅在 100 张设计表宇宙内成立。

### 4.1 W-T-M 索引自身三处数字互斥（✅ 父会话独立复算）

| 位置 | 声称 | 实测 | 判定 |
|---|---|---|---|
| `00-CLASSIFICATION-W-T-M.md` §2（`:67-302`） | 100 表 | **100 行，逐行标记 M=42 / T=46 / W=12，混合 0，未识别 0** | ✅ 真值 |
| 同文件 §3.1（`:309-314`） | M=35 / T=49 / W=14，合計 **100** | 相加 = **98**，与標 100 矛盾 | ❌ 错 |
| 同文件 §3.2（`:322-351`） | 計 **98** | 其 25 个 schema 行相加 = **101** | ❌ 错 |
| 同文件 §3.2 `scm` 行（`:343`） | 計 9 | §2.18 scm 实为 **8** 行 | ❌ 差 +1（唯一逐 schema 偏差） |
| 同文件 `:353` 整合确认 | 「2 混合（`ほか`）待 SRE Lead 拍板」 | §2 已无未识别行、混合 0 | ⚠️ 该待办描述已过期 |

**注**：本报告按用户拍板**不改**这些数字（只补文档、不改分类归属）。真值已记录于此 + `00-CLASSIFICATION-W-T-M.md` §11 差異記録，供 SRE Lead 拍板时直接取用。

---

## §5 治理冲突登记

### 5.1 5 域 → crate 映射与守门 #3 冲突（**按拍板只登记、不改动**）

| 项 | 事实 | 出处 |
|---|---|---|
| 守门约束 | 「RGS 5 域 … とは非対応、業務子域↔DDD マッピングは構築しない」 | `docs/basic-design.md:3937`、`AGENTS.md:135`（守门 #3，2026-09-01 11:45 JST 拍板） |
| 已有映射 | `docs/ddd/01-player-bc.md` §4 已落库 player → `domain-identity`（`:118`）/ `domain-workspace`（`:119`）等映射表 | `docs/ddd/01-05` 共 5 份 |
| 同类文档已自标 | `docs/architecture/cross-domain-5b-mermaid.md:3` 已自标 OBSOLETE（但 `docs/ddd/` 5 份未标） | — |
| 登记位置 | 已按拍板写入 `docs/reports/STAR-P3-DDD-REVIEW-PHASE.md` §6 | — |

### 5.2 域清单缺登记

| 类别 | 数量 | 明细 |
|---|---|---|
| 设计有、代码无 | **0** | `basic-design.md` §2.1（25 logical module）+ §2.1.4（9）= 34 个 `domain-*` 全部存在于 `crates/` |
| 代码有、设计未登记 | **7 `domain-*`** | `domain-canvas` `domain-hook` `domain-infrastructure` `domain-llm` `domain-mcp` `domain-task` `domain-tool`（仅散见于单特性文档） |
| 同上（`star-*`） | **21** | §2.1.5 只登记 10 个 `star-*`，实存 31 个 |

### 5.3 DDD Review 流程前提已失效

- `STAR-P3-DDD-REVIEW-PHASE.md` 状态仍为「等 5 域 Lead 真人 + 5 角色真人到位（10 真人）」，§4 已知缺口 #1/#2 亦然
- 但守门 #14 v4 / v0.62–0.64 已连续反转：**到位 timeline 不适用、真人内容由 agent 决定**
- 实证后果：**41/41 `domain-*` crate 至今 0 个经过 DDD Review 流程**

### 5.4 无表级设计登记的 crate

`domain-batch` / `domain-cli` / `domain-ai` / `domain-dashboard` / `domain-agent-windows` / `domain-canvas` —— 6 个 crate 在 `00-INVENTORY.md` 的 25 个 domain schema 段中**命中数均为 0**。

---

## §6 补齐优先级（建议，本次不动手）

| 优先级 | 动作 | 理由 | 影响面 |
|---|---|---|---|
| **P0** | 消灭假安全感：空检查表（`domain-feedback` / `domain-comment`）、不可执行不变量（`domain-kms` 轮换周期 / `domain-identity` 凭证） | 比缺失更危险 —— 审阅时显示"已实现" | 4 crate + 2 条安全不变量 |
| **P0** | 补 R4 领域事件，顺序 `domain-work-item` → `domain-workflow` → `domain-tenant` → `domain-relation` → `domain-worktree` | 前四者实体与 port 已齐，只差事件类型 + outbox 载荷 | 解锁 outbox / BI / saga 链 |
| **P1** | 清孤儿与重名：`domain-mcp`（删或改契约 crate）、`domain-task` 四同名族归一 | 避免"看起来有、实际没人用" | 5 crate |
| **P1** | 修功能缺口而非措辞：`domain-theme`（`service.rs:32` 硬编码 Light）、`domain-search`（JQL 跑内存行集）、`domain-report`（14/22 图表走 stub） | 这三处是**运行时真的不工作** | 3 crate |
| **P1** | 补实体与端口：`domain-canvas`（174 行空壳）、`domain-tool`、`domain-batch`（Noop 全量 `not_implemented`） | 骨架 crate 无端口 = 上层无法验证 | 3 crate |
| **P2** | 架构归位：`domain-local-runtime` ~2200 行 OS/PTY/SSE 适配迁 infrastructure；`domain-integration` `confluence.rs` 747 行外移 | 域 crate 与 adapter 混装 | 2 crate |
| **P2** | 文档同步：过期自述（`domain-llm`）、不存在的 spec 引用（`domain-batch`）、过期 crate 计数（`lib.rs:1` 称"第 23 个"实为 41 个）、`basic-design.md:3891` 称"25 个 Module" | 防止下一个人按错基线干活 | 4 处 |

---

## §7 已知缺口与未判定项

| # | 项 | 说明 |
|---|---|---|
| 1 | R4 是否为设计要求 | 20 个 crate 判 R4=N 仅基于代码事实，未核对其 spec 是否要求发布事件 → 「缺事件」是完备性观察，**不等于设计违规** |
| 2 | `domain-batch` 归类分歧 | R6=Y（crate 级自述未实装 + 唯一端口为 stub）⇒ 判骨架；若 R6 收窄为「自称骨架」才算 Y，则应为齐全（5/5）。**这是本次最具后果的判断分歧点** |
| 3 | `domain-ai` R2 边界 | `PromptTemplate::render` `lib.rs:158` 算不算「派生规则」；计 Y 仍归部分，结论不变 |
| 4 | `domain-cli` / `domain-agent-windows` 是否算域层 | 二者含 transport/adapter 代码但同时具备实体/值对象/不变量/端口/测试，按 rubric 判承载域模型；若以「含适配器即非域层」为准则需改判 |
| 5 | 18 张 INVENTORY 表无 DDL 支撑 | 无法区分「Lookup 以 seed 定义」与「清单虚登」，需产品侧裁决 |
| 6 | 175 生产表与 100 设计表孰为 SoR | 两集合零交集，仓内无「迁移执行顺序 / owner」权威文档可判 |
| 7 | 7 个未登记 `domain-*` 是否独立 bounded context | 各特性文档按 feature 而非 DDD 边界描述，无 `BoundedContext` 声明 |
| 8 | W-T-M 聚合 98 / 100 / 101 的正解 | §2 明细已无未分类行；「2 混合待拍板」描述过期（`:353`），但 §3.1/§3.2 数字未同步 |
| 9 | 24 个非 `domain-*` crate 的边界 | 已按 4 分类归类（域模型 14 / 应用编排 2 / 基础设施 3 / 工具传输 5），但仓内无逐 crate 边界声明 |

### 7.1 非 `domain-*` 域职责 crate 归属（24/24 已归类，无不确定项）

| 归属 | crate |
|---|---|
| 域模型（14） | `shared-task` `leads` `arg`（混合）`worktree-service`（混合）`relationship-engine` `query-engine` `layout-engine` `action-engine` `risk-engine` `health-engine` `star-workflow` `star-canvas` `star-scheduler` `star-task` `star-game` `agent-domain`（骨架）`canvas-collab`（骨架） |
| 应用编排（2） | `arg-effect` `arg-bridge` |
| 基础设施 / 设置（3） | `event-bus` `ai-vault` `star-cache` |
| 工具 / 传输层（5） | `canvas-engine` `canvas-game` `worktree-canvas` `terminal-stack` `star-desktop` |

---

## §8 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 2026-10-25 | 🟢 审计完成：41/41 判定，齐全 9 / 部分 20 / 骨架 12；本次 0 代码改动（per 用户拍板） |
| 2 | SRE Lead | SRE Lead（Mavis 接手审核） | — | 待真人到位追溯签字 |
| 3 | 平台工程师 | 平台（Mavis 接手审核） | — | 待真人到位追溯签字 |
| 4 | 评审主持人 | 评审主持（Mavis 接手审核） | — | 待真人到位追溯签字 |
| 5 | 项目负责人（PM） | PM（Mavis 接手审核） | — | 待真人到位追溯签字 |

---

## §9 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-25 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版：41/41 crate 完备度矩阵 + 3 类系统性缺口 + 表宇宙零交集实证 + 治理冲突登记 + 补齐优先级 + 9 项未判定 | 用户 2026-10-25 提问「各种 domain model 都齐全么?」+ 拍板 `ask_b66403e46d985090ade079d4`（只补文档不动其他 / 5 域映射只记录不改动） |

---

## 附录 A：审计 brief 与证据链

| brief | 覆盖 | 结论落点 |
|---|---|---|
| `docs/briefs/DDD-AUDIT-01.md` | domain crate 切片 01（14 个） | §2 矩阵 #1-#14 |
| `docs/briefs/DDD-AUDIT-02.md` | domain crate 切片 02（14 个） | §2 矩阵 #17-#30（含 hook/kms/identity 深挖） |
| `docs/briefs/DDD-AUDIT-03.md` | domain crate 切片 03（13 个）+ 24 crate 归属 | §2 矩阵 #25-#41 + §7.1 |
| `docs/briefs/DDD-AUDIT-04.md` | 设计清单 ↔ 代码 ↔ 表清单对齐 | §4 / §5 |

审计日志：`docs/reports/DDD-AUDIT.log`（由 `scripts/automation/dispatcher.py` 生成，per `.gitignore:35` `*.log` 规则不入库）。
