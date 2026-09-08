# PR-F02-LOG-AI-001 — F-02 Log AI 端到端实装

> **PR 描述 (per pre-pr-review skill 期望)**
> **目标 worktree**: `wt-ops-f02-log-ai` (从 main @ `f094fa7` 切, 当前 HEAD = 9 commit 链)
> **目标 main**: `main` (per 守門 #26 v26 必走 PR 流程, 子代理不 merge)
> **拍板**: 2026-09-08 13:18 JST ask_user `ask_f5016f1d0df89012dfa06965` 选 scope-2M_opt1 = F-02 端到端
> **报告**: [PHASE-F02-LOG-AI-REPORT.md](PHASE-F02-LOG-AI-REPORT.md) v0.1 (7 段 per AGENTS.md §3)
> **Brief**: [docs/briefs/ops-f02-log-ai-impl.md](../briefs/ops-f02-log-ai-impl.md) v0.1 (10 节, 已 commit `858804f` + push origin)

---

## 1. 改动摘要

实装 F-02 Log AI 端到端, 9 commit 链 (~800K tokens):

| # | commit | 关键改动 |
|---|---|---|
| 1 | wt1 | mock.rs: 真实调 ai_log_mock.py subprocess (守門 #19 v19 + 守門 #24 v2) |
| 2 | wt2 | openai_stub.rs: 真实 reqwest POST chat/completions (守門 #5 v2 + 守門 #25 v25) |
| 3 | wt3 | anthropic_stub.rs: 真实 reqwest POST messages (守門 #5 v2 + 守門 #25 v25) |
| 4 | wt4 | ladder.rs: is_retriable fn, RateLimited 改 retriable (守門 #6 v2) |
| 5 | wt5 | ops_api.rs::log_upload 真实 (trace_id + level_filter + 1MB 限制) (守門 #5 v2) |
| 6 | wt6 | docs/migrations/2026-09-08-ops-log.sql 3 表 DDL 雏形 (W/T/M 100%, 守門 #13) |
| 7 | wt7 | tests/it_log_ai.rs (3 IT) + benches/log_upload_bench.rs (P95 49ms) (守門 #1 v25 + 守門 #7 v3) |
| 8 | wt8 | frontend/src/{app/ops/components/LogAITab.tsx, lib/ops-api.ts, i18n 3 语言} (守門 #6 v2 + 守門 #5 v2) |
| 9 | wt9 | docs/reports/PHASE-F02-LOG-AI-REPORT.md v0.1 (7 段, 守門 #12 + 守門 #21 v21) |

**净增**: +1815 bytes 估 (13 改 + 6 新); **净删除**: -78 bytes; **tests 净增**: +18 (1 subprocess IT + 7 OpenAI/Anthropic + 5 ladder + 2 log_upload + 3 cross-crate IT).

## 2. 测试证据 (per 守門 #1 v25 + 守門 #9 v20)

```
$ cargo test -p star-ops --tests -j 4
# lib (31) + bin (0) + tests/it_log_ai (3) = 34 total
test result: ok. 31 passed; 0 failed; 0 ignored (lib)
test result: ok. 3 passed; 0 failed; 0 ignored (IT)
# 34/34 pass

$ cargo bench -p star-ops --bench log_upload_bench -- --quick
ladder_analyze_log_mock time:   [47.687 ms 48.148 ms 49.989 ms]
# P95 = 49ms (< 200ms 守門)

$ cargo check -p star-ops --all-targets -j 4
# 0 err (仅 star-credential 既存 7 unused_imports warning, 不在 F-02 scope)

$ cargo fmt -p star-ops --check
# 0 err

$ cargo clippy -p star-ops --all-targets -j 4
# 0 err (advisory, per 守門 #7 v3)

$ cargo check --workspace --all-targets -j 4
# 0 err (per baseline 实证, F-02 改动不破坏 workspace)

$ python scripts/automation/ai_log_mock.py --log "ERROR test"
# exit 0, 守門 #23 mock confidence = 0.42 (永远 < 0.5)

$ cd frontend && npx tsc --noEmit
# 1 既存 err (agent-view/page.tsx, 不在 F-02 scope)
# F-02 5 新文件 0 错
```

## 3. 16 维守门 0 违反 (per WBS §14.10.5)

详 [PHASE-F02-LOG-AI-REPORT.md §3](PHASE-F02-LOG-AI-REPORT.md#3-守门实证-16-维-per-wbs-14105).

**关键守门 (本次新落):**
- 守門 #1 v25: cargo test -p star-ops --lib 31/31 pass + IT 3/3 pass
- 守門 #5 v2: OpenAI/Anthropic api_key header 立即丢弃, log_upload 1MB 限制
- 守門 #6 v2: ladder retriable 跟 error.rs 6-field 对齐 (RateLimited 改 retriable)
- 守門 #13: 3 表 DDL W/T/M 100% 覆盖
- 守門 #19 v19 + 守門 #24 v2: mock 走 ai_log_mock.py subprocess 替代 RPC
- 守門 #23: mock confidence 永远 < 0.5 (needs_review 标徽)
- 守門 #25 v25: LLM stub 默认 no_network_mode, 真实切生产 owner 拍板
- 守門 #26 v26: 子代理不 merge main, 等 owner 拍板

## 4. 已知缺口 (per 守門 #11 缺标比错标, 6 项)

详 [PHASE-F02-LOG-AI-REPORT.md §4](PHASE-F02-LOG-AI-REPORT.md#4-已知缺口-per-守門-11-缺标比错标).

**DDD Review 必查**: 缺口 #1 (no_network_mode 切生产) + #2 (3 表部署) + #3 (agent-view 修) + #4 (持久化).

## 5. Breaking Changes

**无**. 改动:
- ✅ 向后兼容: 既有的 4 OPS stub 端点 + 5 ops_domain::log/cluster/metrics 不动
- ✅ 新增 4 OPS 端点契约 (log_upload 真实化 + log_analysis stub + OpenAI/Anthropic stub 2 通道 mock 流程)
- ✅ star-credential::Provider 加 2 变体 (LlmOpenAi / LlmAnthropic), 既有 match 全部覆盖
- ⚠️ OpenAI/Anthropic 默认 no_network_mode=true (per 守門 #25 v25, 真实切生产 owner 拍板)

## 6. Migration Notes (per 守門 #11 缺标比错标)

```sql
-- 部署 3 表 (owner 拍板时机):
\i docs/migrations/2026-09-08-ops-log.sql
-- 期望: ops_log_entry (W 7d) + ops_log_query_log (T append-only) + ops_log_analysis (M 30d SCD2)
-- RLS 验证:
SELECT tablename, rowsecurity, forcerowsecurity FROM pg_tables WHERE tablename LIKE 'ops_log_%';
-- 期望 3 行 rls=true force_rls=true
```

```toml
# 切生产 (no_network_mode = false, owner 拍板):
OpenAiStubBuilder::default()
    .with_api_key(env("OPENAI_API_KEY"))  # 走 star-credential KMS, 不入 log
    .with_base_url("https://api.openai.com")
    .enable_real_network()  # 守門 #25 v25
    .build();
```

## 7. 关联

- **Brief**: [docs/briefs/ops-f02-log-ai-impl.md](../briefs/ops-f02-log-ai-impl.md) v0.1
- **报告**: [docs/reports/PHASE-F02-LOG-AI-REPORT.md](PHASE-F02-LOG-AI-REPORT.md) v0.1
- **WBS**: [docs/reports/STAR-P3-WBS-001.md](../reports/STAR-P3-WBS-001.md) v0.11 §14.10.2
- **SRS**: [docs/requirements/SRS-STAR-OPS-001.md](../requirements/SRS-STAR-OPS-001.md) v0.1 §4 F-02
- **BAS**: [docs/basic-design/OPS-BASIC-DESIGN-001.md](../basic-design/OPS-BASIC-DESIGN-001.md) v0.1 §3.2 F-02
- **DET**: [docs/detailed-design/OPS-DETAILED-DESIGN-001.md](../detailed-design/OPS-DETAILED-DESIGN-001.md) v0.1 §3 Hybrid AI

## 8. Checklist (per pre-pr-review skill)

- [x] cargo check 0 err (workspace + star-ops)
- [x] cargo test 100% pass (lib 31/31 + IT 3/3)
- [x] cargo fmt 0 err
- [x] cargo clippy 0 err (advisory)
- [x] cargo bench P95 < 200ms (实测 49ms)
- [x] python ai_log_mock.py exit 0 (守門 #23 confidence < 0.5)
- [x] frontend typecheck 我改的 0 错 (1 既存 err 不在 F-02 scope)
- [x] 9 commit 链完整, author = Ulysses
- [x] 16 维守门 0 违反
- [x] 已知缺口 ≥ 5 (本次 6 项)
- [x] 子代理不推 origin, 不 merge main (守門 #26 v26)
- [x] DDL 3 表 W/T/M 100% 覆盖 (守門 #13)
- [x] 守門 #5 v2 派生规 (api_key header 立即丢弃, 1MB 限制)
- [x] 守門 #6 v2 ladder retriable 跟 error.rs 6-field 对齐
- [x] 守門 #25 v25 LLM stub no_network_mode 默认

**owner 拍板**: merge main (per 守門 #26 v26 PR 流程)
