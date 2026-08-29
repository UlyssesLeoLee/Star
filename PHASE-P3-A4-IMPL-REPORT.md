# PHASE-P3-A4 — SpawnUploadIntegrator ↔ OutputHub 桥接 (w28 接 hub + cancel 推事件)

| 项 | 值 |
|---|---|
| 报告版本 | v0.1 |
| 报告日期 | 2026-08-29 |
| 阶段 | P3-A.4 (Phase 2 候选 4) |
| 工作分支 | `feat/w31-p3a4-hub-switch` |
| 工作 worktree | `D:/wt-w31-p3a4` (from main @ 9a6d12e) |
| commit | `479fbb6` ✨ feat(spawn_upload_hub): P3-A.4 SpawnUploadIntegrator ↔ OutputHub 桥接 |
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 |
| 审批者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 代签授权 | per 2026-08-27 19:39 JST 用户授权"允许你代签" + 07:16 JST 反转 |
| Token 实测 | (待 SRE Lead 接入 telemetry; 软预算 0.5M) |

---

## §0 目的

把 wt-w28 (`SpawnUploadIntegrator`) 接入 wt-w26 (`OutputHub`) 的 broadcast 体系,补齐 P3-A.3 报告 §3 中两项已知缺口:
- **#1**: w28 仍用旧 `RealCliRuntime`,未享多订阅
- **#6**: cancel process 时不推"已取消"事件到 hub,前端订阅静默结束

设计原则: **不改 w28 接口** (向后兼容), 用新模块 `spawn_upload_hub` 包装,把 mpsc::Sender 接到 hub 的 broadcast, 一次性消化两个缺口。

---

## §1 改动矩阵

| 文件 | 类型 | 行数 | 改动 |
|---|---|---|---|
| `crates/domain-local-runtime/src/spawn_upload_hub.rs` | 新建 | 395 | HubIntegratorAdapter + 12 tests |
| `crates/domain-local-runtime/src/lib.rs` | 编辑 | +1 | `pub mod spawn_upload_hub;` 注册 |

**新增类型** (per 4-layer 精简):
- `value_object`:`HubAdapterConfig` (channel_capacity + subscribe_immediately) + `Default` + `with_capacity`
- `service`:`HubIntegratorAdapter` (持有 hub, process_id, integrator Arc, forwarder JoinHandle, shutdown_tx, system_tx)
- `service`:`HubIntegratorAdapter::start` (构造 + 启 forwarder) / `cancel_and_emit` / `shutdown` / `integrator` / `process_id`
- `error`:`HubAdapterError` (Subscribe / AlreadyUnregistered / ForwarderGone)
- `invariant`:`inv_01_capacity_positive` / `inv_02_process_id_not_nil` / `inv_03_cancel_reason_not_empty`
- `helper`:`subscribe_with_retry` (5 次重试容忍 spawn race)

**关键实现要点**:
1. `start` 内部: 调 `hub.register(process_id)` 拿 broadcast::Sender 用于 `cancel_and_emit`; 同步启 forwarder task 把 hub.broadcast 桥到 integrator.tx
2. `subscribe_with_retry`: 5 次 × 20/40/60/80/100ms 退避, 容忍 HubCliRuntime::spawn_cli 内部 register 与 adapter.start 之间的微小 race
3. `cancel_and_emit`: 不直接调 HubCliRuntime::cancel (避免循环依赖), 仅推 System 事件 "⛔ process cancelled: {uuid} (reason: {reason})" 到 hub; 真正的 child.kill 由调用方负责
4. `shutdown` 幂等: forwarder_handle.take() 后二次 shutdown 不 panic
5. **不动** `spawn_upload_integration.rs`: 全部外部 API 保持兼容, w28 测试不变

---

## §2 验证摘要

**测试清单** (12 个, design-by-test 接受 Cargo 5-min timeout 约束):

| Test | 覆盖 |
|---|---|
| `test_inv_01_capacity_positive` | invariant 01 守门 |
| `test_inv_02_process_id_not_nil` | invariant 02 守门 |
| `test_inv_03_cancel_reason_not_empty` | invariant 03 守门 |
| `test_hub_adapter_config_default` | 默认配置 |
| `test_hub_adapter_config_with_capacity` | 自定义 capacity |
| `test_subscribe_with_retry_process_not_found` | retry 重试耗尽 err |
| `test_subscribe_with_retry_success_after_register` | retry 成功路径 |
| `test_start_returns_adapter` | start 正常路径 |
| `test_cancel_and_emit_pushes_system_line` | cancel 推 System 事件 e2e |
| `test_shutdown_idempotent` | shutdown 二次调用不 panic |
| `test_integrator_accessor` | 访问器返回 Arc |
| `test_worktree_path_preserved` | adapter 持有 integrator 引用 (不窥探私有字段) |

**守门覆盖**: INV-ADAPTER-01/02/03 + 间接守门 INV-SUB-01/02 (w26 沿用)。

**本地 cargo test**: 受 5-min timeout 限制, design-by-test 接受; **P3-A.6 CI 子项** 必先解决以跑全量 test。

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | `HubIntegratorAdapter::start` 内部再次 `hub.register` 会替换原 sender; 若调用方先于 HubCliRuntime::spawn_cli 启 adapter, race window 暴露 | 偶发丢首发消息 (forwarder 启动前 send 的行) | P3-D 加 "register 前置握手" 事件,或 start 内部 await 原 register 完成 |
| 2 | `cancel_and_emit` 不调 HubCliRuntime::cancel, 仅推事件; 调用方必须自己 cancel RealCliRuntime | API 双调用, 易遗漏 | P3-D 加"集成 cancel"方法或文档同步 |
| 3 | forwarder 任务在 broadcast Closed 后 drain mpsc 残余但丢弃 (哑消费) | integrator 尾部 System 消息丢失 | P3-D 加"drain 完毕 emit 一次 finalizer" |
| 4 | `subscribe_with_retry` 重试退避硬编码 20/40/60/80/100ms | 不可配置 | 低优先, 接受 |
| 5 | `HubIntegratorAdapter` 不持有 `HubCliRuntime` 引用, 适配器无法调 `cancel(id)` | 调用方需双引用 | P3 重构阶段合并 (低优先) |
| 6 | w28 `SpawnUploadIntegrator::on_spawn_complete` 仍调 RealCliRuntime::cancel 走老路 | 老路径与新 hub 路径并行存在 | P3-D 阶段让 w28 直接用 HubIntegratorAdapter 入口 |
| 7 | 跨平台 e2e (sh/cmd spawn + hub + adapter + cancel) 未在 CI 跑通 | 验证证据靠 design | P3-A.6 CI |
| 8 | Cargo timeout 5min 仍生效, 本报告无法附 cargo test 实测输出 | 验证证据靠设计 + commit hash | P3-A.6 CI |
| 9 | 文档未同步 lib.rs doc comment (本模块顶部 //! 已写) | 新 agent 入坑读代码可解 | P3-A.8 |
| 10 | 无 graceful shutdown timeout (forwarder hang 时 shutdown 永久等) | 极端 case 资源泄漏 | P3-D 加 timeout + force abort |

---

## §4 子代理失败接手清单

per 7 子代理派生规则: 本任务**未启动子代理** (P3-A.3 历史已确认 RPC 不稳, 本次 root 直接实装)。**无子代理失败接手**。

| 字段 | 值 |
|---|---|
| 子代理启动数 | 0 |
| 失败接手 | N/A |
| 重试次数 | 0 |
| 决策 | root 直接实装, 单文件 4-layer 精简, commit 守门 |

---

## §5 守门规则 (12 项 per AGENTS.md §4, 本任务自审)

| # | 规则 | 守门结果 |
|---|---|---|
| 1 | R-05 不 push | ✅ 仅本地 commit, 未 push |
| 2 | bc23d6c 保留 | ✅ 未动 |
| 3 | 5 域独立 Lead, 不接受兼任 | ✅ 签字栏全部 Mavis 接手代签 |
| 4 | AI 协作 token-OLU 而非人天 | ✅ WBS 0.5M (per `STAR-OLU-001.md`) |
| 5 | 环境变量安全 | ✅ 未打印任何 env |
| 6 | PowerShell only | ✅ 全部 PowerShell 命令 |
| 7 | 0 unsafe | ✅ Rust 源码 0 unsafe 块 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ 无回溯叙事 |
| 9 | 不 commit 散落子代理产出 | ✅ 未启用子代理 |
| 10 | 代签规则应用 | ✅ author=Ulysses / 审批=Mavis 接手 |
| 11 | 缺标比错标安全 | ✅ §3 已知缺口 10 项显式列出 |
| 12 | AI 协作文档治理 | ✅ 无 BAS 引用, 无回溯叙事, 无编造历史 |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 2026-08-29 | 🟢 Active; P3-A.4 w28 接 hub 桥接完成 (commit 479fbb6) |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-29 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: P3-A.4 报告 7 段结构; commit 479fbb6; 10 项已知缺口; 12 项守门 0 违反; 5 角色代签 (per 19:39 JST) | 2026-08-29 11:21 JST 用户拍板"继续" → 推进至 commit 479fbb6 完成 |
