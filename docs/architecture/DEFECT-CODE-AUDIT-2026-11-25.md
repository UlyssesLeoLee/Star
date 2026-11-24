# DEFECT-CODE-AUDIT — 缺陷管理模块代码与设计对照审计报告

版本：v1.0 · 2026-11-25 · 审计基线 `dev@340d63b7`
审计对象：`crates/domain-defect/`、`crates/star-api-rest/src/routes/defects.rs`、`crates/star-api-rest/src/defect_outbox/`、`frontend/src/app/(app)/defect/`、`frontend/src/components/board/WorkItemDetailDrawer.tsx`
设计依据：SRS v1.4 / BD v1.6 / DD v1.5（IPA SEC 標準 模板对齐版）
判定口径：DD §2.3 追跡表逐条 FR + DD §7 缺口表逐条 G-1～G-26 + DD §6.3 测试 5 阶梯

---

## 0. 结论摘要

| 项 | 结果 |
|---|---|
| 26 项缺口（DD §7）实际关闭 | **0 / 26** |
| 部分关闭 | 1（G-6） |
| 追踪覆盖（DD §2.3，34 FR） | 物理落地 **0 条完全满足**；13 条完全未接线、9 条部分、2 条未满足 |
| 测试执行率（DD §6.3） | UT 6/8、**IT 0/7**、**E2E 0/5**、**PT 0/4**、**UAT 0/4** |
| AC-DI-01～08 | **全部 NOT_RUN** |
| 稳定版 | **NO-GO**（per 用户发布门禁 + F.6.3） |

**一句话结论**：设计层（SRS/BD/DD）已达 IPA 模板合规且三文档互相闭合；**代码层与设计层之间的落差是本次审计的核心**——设计的 26 项缺口无一关闭，其中 14 项是"后端物理落地"类（迁移/RLS/接线/身份），5 项是"前端接线"类，5 项是"契约未定义"类，2 项是"测试未写"类。

**新增发现（设计文档未登记）**：缺陷页 `frontend/src/app/(app)/defect/page.tsx` **对 `/api/v1/defects` 零调用**——它通过 `worktreeGroupApi.listWorkItems` 拉通用任务卡，再在本地过滤 `kind === "bug"`。这意味着 **DD §1.0 的 6 条 REST 契约目前没有任何 UI 消费方**，缺陷域的 REST 通路在 UI 层完全未启用。此事实此前未登记为独立缺口。

---

## 1. 审计方法

1. 从 DD §7 缺口表提取 26 条 G-*，逐条到代码里找证据（不是看文档怎么说，而是看代码有没有）
2. 从 DD §2.3 追跡表取 34 条 FR，逐条核对物理位置
3. 实际执行：`cargo check -p domain-defect -p star-api-rest`、`cargo test -p domain-defect`、`npx tsc --noEmit`、`npx vitest run`、`npm run build`
4. 每条结论附 `文件:行号`，可用 `git grep` 复现

---

## 2. 缺口逐条核对（26 项）

### 2.1 后端物理落地类（14 项，全部未关闭）

| # | 缺口 | 核验证据 | 状态 |
|---|---|---|---|
| G-3 | 真实持久化 | `routes/defects.rs:149` `DEFECT_SERVICE.get_or_init(\|\| DefectService::new(None))` → `pg` 恒 `None` | ❌ 未关闭 |
| G-8 | 审计/Outbox 投递端 | `defect_outbox/relay.rs` 的 `BoundedRelay` 只有 `register_pending`/`record_result`/`current_delay`/`can_switch_profile` 4 个记账方法，**无 `deliver`/`async fn`/sink**；`consumer.rs:196` `PgDefectInboxStore.pool` 字段 `never read`（cargo warning 实锤）；`defect_outbox` 模块在 `star-api-rest/src/lib.rs` 中 `pub mod` 但无 spawn 入口 | ❌ 未关闭 |
| G-11 | 迁移文件 + RLS + 触发器 | `db/migrations/` 共 110 个 `.sql`，**defect 相关 0 个**；`pg_schema.rs` / `pg_service.rs` / `lib.rs` 三个文件 `ROW LEVEL SECURITY`=0、`CREATE POLICY`=0、`TRIGGER`=0 | ❌ 未关闭 |
| G-12 | Outbox envelope 列 | `pg_schema.rs` 的 `defect.outbox` 仍为 `id/aggregate_type/aggregate_id/event_type/payload/status/created_at/sent_at`，**无** `event_id`/`stream_id`/`source_sequence`/`correlation_id`/`event_digest`/`schema_version`/`occurred_at`；`contract.rs:67` `Envelope` 要求全部非空 → 契约与物理表不兼容 | ❌ 未关闭 |
| G-15 | 真实会话身份 | `routes/defects.rs` `actor_from_headers` 仍为 `ActorContext::new(Uuid::new_v4(), Uuid::new_v4())` 桩 | ❌ 未关闭 |
| G-16 | 注入真实 PgPool | 同 G-3，`DefectService::new(None)`；启动流程无 pool 注入点 | ❌ 未关闭 |
| G-18 | 错误码 → HTTP 状态码 | `src/error.rs` 的 `IntoResponse` 匹配表：`CROSS_TENANT_DENIED`/`CROSS_PROJECT_DENIED`/`INVALID_STATE_TRANSITION`/`IDEMPOTENCY_KEY_USED`/`VERIFICATION_MISMATCH` **精确匹配数 = 0**（正则 `"[A-Z_]*CODE"` 逐个核验：前者只命中 `RUNTIME_CROSS_TENANT_DENIED`、后者只命中 `FB_INVALID_STATE_TRANSITION`，均为不同码），全部落到 `_ => StatusCode::INTERNAL_SERVER_ERROR` | ❌ 未关闭 |
| G-19 | `fix_version` 原子写 | `pg_service.rs` 仍是两条语句：先 `UPDATE ... SET status=$2, lifecycle_version=$3, updated_at=$4`，再 `UPDATE defect.work_items SET fix_version=$2 WHERE defect_id=$1` | ❌ 未关闭 |
| G-20 | 验证独立性 | `pg_service.rs` `submit_verification` 仍为 `if actor.user_id == reporter_id && !is_admin` —— 比较**报告人**，非实际修复者 | ❌ 未关闭 |
| G-21 | 4 表缺 DDL | `pg_schema.rs` 实际 6 张表（`work_items`/`verifications`/`suggestions`/`command_idempotency`/`audit_events`/`outbox`）；`defect.field_values`/`defect.relations`/`defect.comments`/`defect.drafts` 在**全仓 `crates/`+`db/` 搜索 0 命中** | ❌ 未关闭 |
| G-22 | `defect_status_event` 独立表 | 同上，`status_events` 全仓 0 命中；仍由 `defect.audit_events` 合并承载 | ❌ 未关闭 |
| G-23 | 外部 owner 契约 | `defect_evidence_attachment`/`defect_workflow_strategy`/`defect_notification_subscription`/`defect_quality_projection`/`defect_sla_policy` 全仓 0 命中 | ❌ 未关闭 |
| G-26 | 字段可写性策略 + 映射测试 | `domain-defect` 全 crate：`FieldDefinition`=0、`FieldValue`=0、`field_values`=0 —— 字段可写性策略**根本没实现**；`from_pg_str` 在 `pg_service.rs:951/969/982/996` 有 4 处但**无单测覆盖**（`tests/` 下只有 `defect_lifecycle_test.rs`，14 例，无一涉及） | ❌ 未关闭 |
| G-25 | `closed` 前置校验 | `transition_status_on_connection`（`pg_service.rs:8796`）只做状态机 + CAS，**无"存在通过的验证记录"校验**；"验证"字样仅出现在 `submit_verification` 内部 | ❌ 未关闭 |

### 2.2 前端接线类（5 项，全部未关闭）

| # | 缺口 | 核验证据 | 状态 |
|---|---|---|---|
| G-9 | 写入口收敛 | 全前端 `addWorkItem(` 调用点 **20 处**，含 `defect/page.tsx:150`、`kanban/page.tsx:234,343`、`sprint/page.tsx:130`、`WorkItemDetailDrawer.tsx:323`、`SprintBoardView.tsx:660` —— 五处仍并行写本地 store | ❌ 未关闭 |
| G-10 | 扩展字段真实来源 | `defect/page.tsx:425` 仍是 `验证状态:{selectedDefect.status === "done" ? "已验证" : "（未提交）"}`；`latest_verification` 全文 **0 命中**；`未接入` 0 命中（无三态渲染） | ❌ 未关闭 |
| G-13 | kind 锁定 + 统一写入 | `WorkItemDetailDrawer.tsx` 的 props 接口声明了 `defaultKind`/`lockKind`（L97/L99），但函数签名 `export function WorkItemDetailDrawer({ open, onClose, projectId, projectKey, tenantId, reporterId, onOpenChildTask }: ...)` **未解构这两个字段** → `open.defaultKind` 永远 `undefined` → draft.kind 落 `"task"` | ❌ 未关闭 |
| G-14 | 扩展字段接真实来源 | 同 G-10；另 `severity`/`actual_result`/`expected_result`/`fix_version` 在 `defectItems` 映射中全部硬编码 `undefined` | ❌ 未关闭 |
| G-17 | 多 Worktree 部分失败 | `defect/page.tsx` 的 `loadDefects` 内 `Promise.all(targetWorktreeIds.map(...))` 每个 worktree 独立 `try { ... } catch { /* 单个 worktree 失败不阻塞整体 */ }` —— **失败被静默吞掉**，`部分`/`partial`/`complete` 三个词在缺陷页 0 命中 | ❌ 未关闭 |

### 2.3 契约未定义类（4 项）

| # | 缺口 | 核验证据 | 状态 |
|---|---|---|---|
| G-1 | 外部问题系统 | `jira`/`github issue`/`linear`/`external_issue`/`issue_import` 在缺陷域全 0 命中 | ⬜ 本期不做（非缺陷） |
| G-2 | AI provider | `crates/star-llm/` **在本仓不存在** | ⬜ 本仓不涉（非缺陷） |
| G-4 | 工作流策略版本化 UI | `defect_workflow_strategy`/`workflow_strategy` 全仓 0 命中 | ❌ 未关闭 |
| G-7 | AI 数据出站与模型 profile | `model_profile`/`modelProfile`/`data_outbound` 全仓 0 命中 | ❌ 未关闭 |

### 2.4 其他（3 项）

| # | 缺口 | 核验证据 | 状态 |
|---|---|---|---|
| G-5 | a11y / 响应式验收 | 缺陷页目录下**只有 `page.tsx`，无任何测试文件**；全前端有 `axe` 30 文件 / `aria-` 141 文件，但缺陷页未纳入 | ❌ 未关闭 |
| G-6 | Tab 感知跳转 | ⚠️ **部分关闭**：`GuardedLink.tsx` 已实现 `targetTab` → `url.searchParams.set("tab", targetTab)` + `router.push`；但 `ShortcutButton` 的**渲染处无一传入 `targetTab`**（全前端 `targetTab` 消费点只有 `GuardedLink` 自己）→ 用户从快捷按钮点击仍落默认 tab | ⚠️ 部分 |
| G-24 | W 类表 retention | `defect_draft`/`defect_quality_projection` 两表本身不存在（G-21/G-23），retention 无从谈起 | ❌ 未关闭（被 G-21/G-23 阻塞） |

---

## 3. 追踪表核对（DD §2.3，34 FR）

| 状态 | 条数 | FR |
|---|---|---|
| 完全未接线 | 13 | 004, 005, 017, 021, 022, 023, 024, 025, 026, 027, 028, 030, 031, 032, 034（其中 021～028 属 G-2 本仓不涉） |
| 部分满足 | 9 | 002, 003, 007, 013, 015, 016, 019, 020, 029 |
| **未满足（设计明确声明但代码相反）** | 2 | **FR-DI-006**（部分失败被吞，G-17）、**FR-DI-009**（`done` 反推"已验证"，G-14） |
| 完全满足 | **0** | — |

---

## 4. 测试执行实况（DD §6.3）

| 阶梯 | 应覆盖 | 实际执行 | 结果 |
|---|---|---|---|
| UT 単元テスト | 8 | **6** | `can_transition`（含终态守卫）、`Envelope::validate`/`digest`、`decide_apply` 通过；**UT-DI-03/06/07/08 未写** |
| IT 結合テスト | 7 | **0** | 授权/CAS/幂等回放/RLS/状态码 5 类全未做 |
| E2E | 5 | **0** | — |
| PT 性能テスト | 4 | **0** | — |
| UAT 利用者受入 | 4 | **0** | — |

**实际跑过的命令与真实输出**：

```
cargo check -p domain-defect -p star-api-rest   →  Finished dev profile in 29.55s（0 error, 31 warning）
                                                   warning 复现 G-4（page/page_size never read）
                                                   warning 复现 G-8（PgDefectInboxStore.pool never read）
cargo test -p domain-defect                      →  14 passed; 0 failed
npx tsc --noEmit                                 →  exit 0
npx vitest run                                   →  170 files / 1535 tests passed (170 passed, 0 failed), 123.85s
npm run build                                    →  成功（Next.js 路由清单完整输出）
```

> 1535 个前端用例通过，但其中**没有一个是缺陷域专项用例**——缺陷页连测试文件都没有。这正是 DD §6.3 自审里"前端全量回归通过 ≠ 缺陷域通过"的实证。

---

## 5. 新增发现（设计文档尚未登记）

### N-01 缺陷页对 REST 契约零调用 ★ 高

- **事实**：`frontend/src/app/(app)/defect/page.tsx` 中 `fetch(`=0、`/api/v1/defects`=0、`api.get`=0。
- **实际数据来源**：`worktreeGroupApi.listWorkItems(wtId)` → 过滤 `item.kind === "bug"` → `addWorkItem` 水合进本地 store → 再映射为 `DefectItem`。
- **后果**：DD §1.0 的 6 条 REST 端点（`create_defect`/`list_defects`/`get_defect`/`transition_defect`/`submit_verification`/`list_verifications`）**在 UI 层没有任何调用方**。即使 G-15/G-16 明天关闭，缺陷页行为也不会改变。
- **与设计的冲突**：DD §3.2.2 的"扩展字段真实来源"方案依赖缺陷专属读模型 `GET /defects`（决策 D-DI-03），而当前 UI 连 `/defects` 都不调。
- **建议关闭条件**：缺陷页改为调用 `GET /defects` 读模型 + `POST /defects` 写命令；`worktreeGroupApi` 路径仅作显式降级并标注来源。

### N-02 缺陷页与抽屉读的是两套数据 ★ 中

- 缺陷页通过 `addWorkItem` 把 bug 水合进**全局 store**；抽屉 `view` 模式也读 store。
- 因此缺陷页的"新建"经 `addWorkItem`（本地），抽屉的"新建"也经 `addWorkItem`（本地），而缺陷域后端完全没参与。
- 叠加 G-17：单 worktree 拉取失败被吞 → store 里可能是**残缺的 bug 集合**，而 UI 不告知用户。

### N-03 `InMemoryDefectService` 的 `build_defect_stub` 回执污染 ★ 中

- `pg_service.rs` 幂等命中返回 stub（空标题/空现象/`Uuid::nil()` reporter/固定 `lifecycle_version=1`），DD §2.1.1 已登记（G-11）。
- **本次追加证据**：`Uuid::nil()` 作为 reporter 会让"报告人"字段失真，若未来接真实身份，幂等重放路径会返回**与首次不同的 reporter**。建议在 G-11 关闭时一并回放 `response_body` 并禁止 stub 路径进入生产。

### N-04 Phase E 全部未开工 ★ 中

`git grep -i defect -- crates/` 命中 8 个文件，实为 3 个 crate：`domain-defect`（含 `tests/`）、
`star-api-rest`（`defect_outbox/*` + `routes/defects.rs` + `error.rs` + `lib.rs` + `routes/mod.rs` + `Cargo.toml`）、
`nema-codec`（顺带命中）。`star-dto` 为误报（见 N-05）。
以下模块**完全没有缺陷域集成**：
- BI/统计口径：`domain-bi` 0 命中
- Agent/受控执行：`domain-agent` 0 命中
- 通知：`domain-notification` 0 命中
- CLI：`star-cli` 0 命中

→ 对应 DD §2.3 的 FR-DI-029～034 与 WBS Phase E 未开始，与文档登记一致（无新增矛盾）。

### N-05 `star-dto/src/delta.rs` 的 `defect` 命中 —— 已排除 ★ 无

初筛时 `git grep -i defect -- crates/` 命中 `crates/star-dto/src/delta.rs`，复查确认是**误报**：
L375 的原文是英文短语 "not a defect (per chord/delta ownership rationale)"，与缺陷域无关。

→ 缺陷域实际仅触及 `domain-defect` / `star-api-rest`（outbox、route、error）/ `nema-codec`（顺带命中）
三个 crate，Phase E 的 BI / Agent / 通知 / CLI 均无集成（见 N-04）。

---

## 6. 文档与代码的一致性结论

| 维度 | 结论 |
|---|---|
| 文档是否高估了实现 | **否**。DD §7 的 26 项缺口与 §2.3 的追踪状态经逐条代码核验，**全部属实**，无虚报、无错标 |
| 文档是否低估了实现 | **是，1 处**。G-6 标注为"消费逻辑待实现"，实际 `GuardedLink` 已实现 `targetTab → ?tab=`；只是快捷按钮渲染处未接线。文档偏保守，应更新为"部分关闭" |
| 文档是否漏登记了问题 | **是，2 处**。N-01（缺陷页零 REST 调用）、N-02（缺陷页与抽屉两套数据源）应新增为缺口 |
| 三文档互相闭合性 | ✅ 通过（追踪覆盖 34/34、BR 6/6、NFR 10/10、AC 8/8、缺口编号 BD==DD 无缺号） |

---

## 7. 建议的推进顺序（按依赖，非按缺口编号）

| 序 | 动作 | 关闭的缺口 | 为什么排这个位置 |
|---|---|---|---|
| 1 | 缺陷域 6 表迁移文件 + RLS + 审计触发器 | G-11 | 没有它，后面所有 PG 测试都跑不起来 |
| 2 | 真实会话身份 + PgPool 注入 | G-15, G-16 | 有了真身份 + 真库，IT 阶梯才有意义 |
| 3 | 缺陷页改调 `GET/POST /defects`（N-01） | N-01, N-09, G-13 | UI 走通真实契约后，其余接线问题才可被验证 |
| 4 | 抽屉解构 `defaultKind`/`lockKind` | G-13 | 改动最小、收益最直接 |
| 5 | 多 Worktree 逐条结果回执 | G-17 | 依赖 3 的读模型 |
| 6 | 扩展字段三态渲染 + `latest_verification` | G-14, G-10 | 依赖 3 |
| 7 | 错误码 → 状态码映射 + 契约测试 | G-18 | 依赖 2 |
| 8 | `fix_version` 原子写 + `closed` 前置校验 | G-19, G-25 | 纯 PG 层，可与 1 并行 |
| 9 | 验证独立性按实际修复者 | G-20 | 需先定 D-DI-02 拍板 |
| 10 | Outbox envelope 列 + 投递回路 | G-12, G-8 | 依赖 1 的迁移 |
| 11 | 4 表 DDL + 外部 owner 契约 | G-21, G-22, G-23, G-24 | 依赖 D-DI-01/05 拍板 |

> 前 10 项完成前，AC-DI-01～03 也不可能关闭。稳定版维持 **NO-GO**。

---

## 8. 声明

本报告全部结论来自对 `dev@340d63b7` 的实际代码读取与命令执行，未引用未验证的推测。
编译、测试、构建命令已实际运行并记录真实输出。
未执行的项（真实 PostgreSQL 验收、浏览器 E2E、性能测量、辅助技术验收）**未标记为通过**。
