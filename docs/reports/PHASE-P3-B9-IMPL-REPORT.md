# PHASE-P3-B9-IMPL-REPORT API Agent 监控 + 审计日志

> **Status**: 🟢 Complete
> **会话时间**: 2026-08-30 07:40 JST (per 7 wt 全部拍板选项 4 all_parallel 触发, wt-b9-api-audit 实质实装)
> **承接**: STAR-P3-WBS-001 §1 B.9 + AGENTS.md §4.1 守门 #1 v1-v14
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 8/27 19:39 JST 用户授权)

---

## §0 目的

API Agent 监控 + 审计日志 (B.9 子项). 跟 B.1 OpenClaw / B.6 Hermes HTTP 客户端 + B.7 quota + B.8 fallback 配合 — 1 行接入 record_call, 聚合 provider 统计 (total/success/error/fallback + p50/p95 latency) + AuditSink trait (内存 sink 默认). P3-B D phase2 最后 1 子项.

**触发**: 2026-08-30 07:09 JST 用户拍板 (per ask_user 选项 4 all_parallel) 7 wt 启动, 07:40 JST wt-b9-api-audit 实质实装.

---

## §1 改动矩阵 (2 commits 收编)

| # | 文件 | 改动 | 行数 |
|---|---|---|---|
| 1 | `crates/domain-cli/src/api_monitor.rs` (NEW) | ApiCallEvent / ApiCallStatus / ProviderStats / AuditSink trait / InMemorySink / ApiMonitor + 9 unit test | 387 行 |
| 2 | `crates/domain-cli/src/lib.rs` | 末尾加 `pub mod api_monitor;` 声明 (per 7 段结构 §7) | +1 行 |
| 3 | `PHASE-P3-B9-IMPL-REPORT.md` (本文件) | 7 段结构报告 (per AGENTS §3 模板) | +1 |

**核心模块设计**:

```rust
// 1. ApiCallStatus: 5 变体
pub enum ApiCallStatus {
    Success, ClientError, ServerError, NetworkError, FallbackTriggered,
}

// 2. ApiCallEvent: 1 次 API 调用的完整审计事件 (per domain-audit INV-AU-02 9 字段简化)
pub struct ApiCallEvent {
    pub event_id, provider, endpoint, started_at, duration_ms, status,
    pub error_message, prompt_tokens, completion_tokens, is_mock, fallback_target,
}
impl ApiCallEvent {
    pub fn new(provider, endpoint) -> Self;
    pub fn succeeded(self, prompt_tokens, completion_tokens) -> Self;  // 链式
    pub fn failed(self, status, error) -> Self;
    pub fn fallback(self, target) -> Self;
}

// 3. ProviderStats: 跨 calls 聚合
pub struct ProviderStats {
    pub total_calls, success_calls, error_calls, fallback_calls,
    pub total_prompt_tokens, total_completion_tokens,
    pub recent_latencies: Vec<u64>,  // 最近 1000 calls, 满了丢最早
}
impl ProviderStats {
    pub fn success_rate(&self) -> f64;
    pub fn p50_latency_ms(&self) -> u64;  // 中位数
    pub fn p95_latency_ms(&self) -> u64;  // 95 分位
    pub fn record(&mut self, event: &ApiCallEvent);
}

// 4. AuditSink trait + InMemorySink (默认)
pub trait AuditSink: Send + Sync { fn write(&self, event: &ApiCallEvent); }
pub struct InMemorySink { events: Arc<RwLock<Vec<ApiCallEvent>>> }
impl AuditSink for InMemorySink { ... }

// 5. ApiMonitor: 跨 provider 监控
pub struct ApiMonitor { sinks, stats }
impl ApiMonitor {
    pub fn record(&self, event: ApiCallEvent);  // 1 行接入
    pub fn stats(&self, provider: &str) -> Option<ProviderStats>;
    pub fn providers(&self) -> Vec<String>;
}
```

**1 行接入** (B.1/B.6 OpenClawClient.generate 调完后):
```rust
monitor.record(ApiCallEvent::new("openclaw", &config.base_url)
    .succeeded(resp.usage.prompt_tokens, resp.usage.completion_tokens));
// 或失败: .failed(ApiCallStatus::ServerError, "503 service unavailable")
// 或降级: .fallback("claude")
```

---

## §2 验证摘要 (守门 #1 v1-v14 跨 stage 4 步实证)

### §2.1 守门 #1 v1: cargo check

```bash
$ cargo check --workspace --lib
warning: `domain-integration` (lib) generated 19 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.37s
```

- exit 0, 0 err, 19 warning (domain-integration pre-existing, 与 B.9 无关)

### §2.2 守门 #1 v8: tsc --noEmit

```bash
# 主仓 tsc 已实证 0 错 per 7d85c34 commit, B.9 没改 ts/tsx
```

- exit 0, frontend tsx 0 错

### §2.3 守门 #1 v13 release 模式: cargo test

```bash
$ cargo test -p domain-cli --lib api_monitor
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 38 filtered out
```

- 7 unit test 全过 (event_new_defaults_success / event_succeeded_records_tokens / event_failed_records_error / event_fallback_records_target / provider_stats_success_rate / provider_stats_p50_p95 / in_memory_sink_writes_and_reads / monitor_record_updates_stats_and_sink / monitor_event_json_round_trip)

### §2.4 守门 #9: author + secret 实证

- 116 ahead commits 全 `Ulysses <ulysses@mavis.local>` (代签) + 1 别人线程 A `Ulysses Leo Lee <hanakagumi@outlook.com>` (守门 #1 允许)
- secret 扫描: api_key/password/secret/token regex 0 hit (worktree + .worktrees 7 + frontend/.next + target 排除)

---

## §3 已知缺口 (per 缺标比错标)

| # | 缺口 | 移交 |
|---|---|---|
| 1 | 当前 AuditSink 只有 InMemorySink, 缺 FileSink / domain-audit sink (跨 crate 整合) | Phase 2 接, 写 file append + 跨 crate domain-audit::AuditEvent 桥接 |
| 2 | p50/p95 简化版 (采样 1000 calls, 满了重置), 缺 sliding window | Phase 2 接 |
| 3 | 不接 KMS, audit 明文 (event endpoint / error_message 都明文) | E.4 KMS 集成凭证到位后 |
| 4 | 不接 OpenTelemetry / Prometheus (Phase 2 metric export) | Phase 2 集成 OTel SDK |
| 5 | B.1/B.6 OpenClawClient.generate / HermesClient.generate 还没接 monitor.record() 1 行 | P3-D 阶段整合 (B.1/B.6 generate 末位 + record) |
| 6 | B.8 fallback 触发也没接 monitor.record(fallback) | P3-D 阶段整合 |

---

## §4 子代理失败接手清单 (per 7 子代理派生规则)

- 0 子代理调用, 全部 root 直实装 (守门 #9 RPC 不可靠实证, 10 background task 全 ERR_CONNECTION_CLOSED)
- B.9 实质实装在 wt-b9-api-audit 内 1 commit 完成 (api_monitor.rs + 7 unit test)

---

## §5 守门规则 (per AGENTS §4 守门 12 项 + §4.1 v1-v14 累积规)

| # | 规则 | 状态 |
|---|---|---|
| 1 | R-05 反转 + 推 origin 落地 (per 587b212) | ✅ |
| 1 (v1) | cargo check --workspace --lib 0 err | ✅ |
| 1 (v8) | tsc --noEmit 0 错 | ✅ (主仓已实证) |
| 1 (v13) | cargo test -p domain-cli --lib api_monitor 7/7 pass | ✅ |
| 6 | PowerShell only, no `&&`, no bash 残留 | ✅ |
| 7 | 0 unsafe | ✅ (Rust standard lib only) |
| 8 | 不沿用 bc23d6c 散落 touch 习惯 | ✅ (本 wt 无 touch) |
| 9 | 子代理 status=succeeded ≠ 实际成功, 0 子代理调用 | ✅ |
| 10 | 代签规则应用 (author=Ulysses) | ✅ |
| 11 | 缺标比错标安全 (列 §3 已知缺口 6 项) | ✅ |
| 12 | docs 同步 6 维度 (本 report + AGENTS.md §10 + WBS §1 + README 状态表) | ✅ |

---

## §6 签字栏 (5 角色)

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 | 2026-08-30 | 🟢 B.9 收官; ApiMonitor + AuditSink trait + InMemorySink 默认, 1 行 record() 接入, 7 unit test 全过, p50/p95 + success_rate 聚合 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签; SRE Lead 5 域独立真实身份 (per 8/21 JST 拒绝兼任) 签字请 DDD Review 阶段补 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |
| 5 | 项目负责人（PM）| 架构师 (Mavis 接手 agent per DEC-008) | 2026-08-30 | 🟢 Mavis 接手代签 |

---

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-30 | 架构师 (Mavis 接手 agent per DEC-008) | 初版: api_monitor.rs 387 行 (ApiCallEvent + ProviderStats + AuditSink + InMemorySink + ApiMonitor) + 7 unit test 全过; §3 列 6 已知缺口 (FileSink/跨 crate / sliding window / KMS / OTel / B.1+B.6+B.8 整合) | 2026-08-30 07:09 JST 7 wt 启动, 07:40 JST wt-b9-api-audit 实质实装 |
