# Brief: F-02 Ops Log AI 端到端实装 (per WBS §14.10.2 + 拍板 9/8 13:18 JST)

> **状态**: 🟡 Brief v0.1
> **拍板**: 2026-09-08 13:18 JST ask_user `ask_f5016f1d0df89012dfa06965` 选 scope-2M_opt1 = F-02 log AI 端到端
> **wt-branch**: `wt-ops-f02-log-ai`
> **base**: `main @ f094fa7` (per PR #23 squash + 22 commit 推 origin 后)
> **触发**: 2026-09-08 13:15 JST 用户发令"实际编写代码的工作开子代理和worktree完成并merge到main"
> **关联**: [WBS §14.10.2 4 子项端到端](../../reports/STAR-P3-WBS-001.md) · [OPS-DETAILED §3 Hybrid AI 详细](../../detailed-design/OPS-DETAILED-DESIGN-001.md) · [OPS-BASIC §3.2 F-02 LogAI](../../basic-design/OPS-BASIC-DESIGN-001.md) · [SRS §4 F-02 §5.3 mock subprocess](../../requirements/SRS-STAR-OPS-001.md) · [ADR-0048 framework 锁 axum 0.8](../../architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md) · [ai_log_mock.py v0.1](../../../scripts/automation/ai_log_mock.py) · [守门 #9 子代理 RPC 不可靠实证](../../../AGENTS.md)

---

## 1. 目标 (Objective)

实装 `star-ops` 端 `/api/ops/log/upload` + `/api/ops/log/analysis/{id}` 端到端, 让 LLM 通道 (mock / OpenAI / Anthropic) 跑通真实 subprocess 调用, UI `/ops/page.tsx` logai tab 接真实 API 返 200 + 解析 + 渲染. 走守门 #23 (mock 不开外部 API) + 守门 #19 v19 (agent 交互走 scripts/automation/) + 守门 #5 v2 (API key 安全).

**范围** (per WBS §14.10.2 F-02 估 800K token):
- 1 后端实装 (mock subprocess 路径启用, reqwest + serde_json 接 LLM 通道)
- 1 前端实装 (LogAITab.tsx 上传区 + AI 分析结果区 + useQuery)
- 1 mock 升级 (ai_log_mock.py 增强, 加 trace_id + 实时统计)
- 1 integration test (4 表 ops_log_entry / ops_log_query_log / ops_log_analysis 跨 crate IT 雏形)
- 1 PT 雏形 (criterion bench P95 < 200ms 守门)
- 8 commit 链 + 1 PR 描述 + 1 PHASE 报告

## 2. 范围 (Scope)

### 2.1 In-Scope (F-02 端到端)

**后端**:
- `crates/star-ops/src/ops_ai/mock.rs`: 启用 `call_subprocess_stub` 路径, 不再 `#[allow(dead_code)]`, 改 `#[cfg(test)]` 保留测试
- `crates/star-ops/src/ops_ai/openai_stub.rs`: 真实 OpenAI API 接入 (reqwest), API key 走 `star-credential` 加密存储
- `crates/star-ops/src/ops_ai/anthropic_stub.rs`: 真实 Anthropic API 接入 (reqwest), API key 走 `star-credential` 加密存储
- `crates/star-ops/src/ops_ai/ladder.rs`: 修 retriable 规则 (RateLimited 改 retriable per 守门 #6 派生)
- `crates/star-ops/src/ops_api.rs`: log_upload 改真实 (含 trace_id 传递, level_filter 应用, 1MB body 限制 per 守门 #5 v2)
- `crates/star-ops/Cargo.toml`: 加 `reqwest` + `star-credential` 依赖
- 4 表 DDL 雏形: `db/migrations/2026-09-08-ops-log.sql` (ops_log_entry / ops_log_query_log / ops_log_analysis 3 表 W/T/M 100% 覆盖, retention 7d/30d 显式)
- IT 雏形: `crates/star-ops/tests/it_log_ai.rs` (axum 跨 crate IT + subprocess mock 跑通)

**前端**:
- `frontend/src/app/ops/components/LogAITab.tsx` 新建 (上传区 + AI 分析结果区, useQuery 实时轮询)
- `frontend/src/lib/ops-api.ts` 改: 8 REST endpoint 真实 fetch wrapper, error handling, retry
- i18n 3 语言 (zh-CN/en/ja) 加 logai 文案: `uploading`, `analyzing`, `mock_channel`, `openai_channel`, `anthropic_channel`, `error.network`, `error.timeout`, `error.unauthorized`

**Python**:
- `scripts/automation/ai_log_mock.py` 升级 v0.2: 加 `--log-file` / `--trace-id` 参数, 加 `--stats` 输出 anomaly 统计 + elapsed_ms + exit code, 加 `--version` 输出版本

**文档**:
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (7 段 per AGENTS.md §3)
- PR 描述 (`docs/reports/PR-F02-LOG-AI-001.md`)

### 2.2 Out-of-Scope (不修)

- 真实 LLM API 接入 (OpenAI/Anthropic) — 仅 stub 真实调用, 真实 API key 由 Ulysses 配置
- K8s/Helm cluster (F-01 后续)
- 运维数据 metrics (F-03 后续)
- 文档扫描 (F-04 后续)
- OAuth 2.0 / mTLS — 复用 `star-context::ActorContext` (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯

## 3. 实施步骤 (8 commit 链)

| # | commit | 标题 | 估 token |
|---|---|---|---|
| 1 | `wt1` | feat(ops-ai): 启用 mock subprocess 路径 (call_subprocess_stub #[cfg(test)] → production) | 50K |
| 2 | `wt2` | feat(ops-ai): OpenAI stub 真实 reqwest 接入 (api_key 走 star-credential) | 150K |
| 3 | `wt3` | feat(ops-ai): Anthropic stub 真实 reqwest 接入 (api_key 走 star-credential) | 150K |
| 4 | `wt4` | fix(ops-ai): ladder.rs retriable 规则 (RateLimited 改 retriable per 守門 #6) | 30K |
| 5 | `wt5` | feat(ops-api): log_upload 真实 (trace_id 传递 + level_filter + 1MB body 限制) | 100K |
| 6 | `wt6` | feat(db): 3 表 DDL 雏形 (ops_log_entry 7d / ops_log_query_log / ops_log_analysis 30d) | 80K |
| 7 | `wt7` | test(ops): IT 雏形 (axum 跨 crate IT + subprocess mock 跑通) + criterion bench P95 < 200ms | 100K |
| 8 | `wt8` | feat(frontend): LogAITab.tsx + ops-api.ts + i18n 3 语言完整 | 80K |
| 9 | `wt9` | docs(phase): PHASE-F02-LOG-AI-REPORT v0.1 + PR 描述 (7 段) | 60K |
| **累计** | | | **~800K** |

每个 commit 必先 `git log -p --follow` 实证 worktree commit 在 chain 上 (per 守门 #9 主体), author = Ulysses (per 守门 #10).

## 4. 守门 (per AGENTS.md §4 累积规 v1-v26)

每 commit 必跑:

```powershell
# 1. cargo check -p star-ops (单 crate per 守门 #1 v25)
cargo check -p star-ops --all-targets -j 4  # 0 err

# 2. cargo test -p star-ops (单 crate 跳过 workspace per 守门 #1 v25 + v26)
cargo test -p star-ops --lib -j 4  # 全 pass, 含 IT 雏形

# 3. cargo fmt + clippy
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 0 err (advisory per 守门 #7 v3)

# 4. workspace 必跑 (per 守门 #1 v1)
cargo check --workspace --all-targets -j 4  # 0 err, 1m 19s 实证

# 5. python ai_log_mock.py v0.2 跑通 (守门 #23 派生, confidence < 0.5)
python scripts/automation/ai_log_mock.py --log "ERROR test"  # exit 0

# 6. frontend typecheck 我改的文件 0 错
cd frontend && npx tsc --noEmit  # 0 错 (我改的)

# 7. subprocess IT 实证 (守门 #9 + 守门 #24 v2)
# call_subprocess_stub 真实调 ai_log_mock.py, 验证 stdout 解析 0 错
cargo test -p star-ops --lib it_subprocess_real_call
```

**16 守門** (per WBS §14.10.5): #1 4 守門 + #1 v19/v25 + #3 #4 #5 v2 #6 v2 #7 v3 #9 #10 #11 #12 #13 #14 v2 + #19 v19 #21 v21 #23 #24 v2 + #25 v25 + #26 v26.

## 5. 子代理 brief 规则 (per 守门 #9 v20 + v3 实证)

### 5.1 派前必做

1. 落档本 brief (本文件, 已落档 + commit)
2. worktree commit brief + push origin (per 守门 #9 v20, 必先 `git log -p --follow docs/briefs/ops-f02-log-ai-impl.md` 实证)
3. 派 worker 子代理 (run_in_background=true), 引用本 brief 路径
4. 子代理 status="succeeded" ≠ 实际成功, **owner 必在子代理返回后跑 owner evidence check** (per 守門 #9 主体实证, 10 background task `net::ERR_CONNECTION_CLOSED` 但 status=succeeded 教训)

### 5.2 子代理不能擅自做的

- 推 origin (必 owner 拍板 per 守門 #1 反转)
- merge main (必 PR 流程 per 守門 #26 v26)
- 改 守門 16 维任何一条 (必 owner 拍板)
- 跳过 cargo check 守门 (必跑 0 err)
- 跳过 IT/PT 验证 (必跑 跨 crate IT + P95 守门)
- 编造历史 (守門 #1 禁回溯)

### 5.3 owner evidence check (per 守門 #9 主体)

子代理 status=succeeded 后, owner 必**实际跑**:

```powershell
# 1. 8 commit 在 wt-ops-f02-log-ai branch 上
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline wt-ops-f02-log-ai | Measure-Object -Line  # 8/9 commit

# 2. 跨 crate 实证 (不能只信子代理报告)
cargo check -p star-ops --all-targets -j 4  # 必 0 err
cargo test -p star-ops --lib -j 4  # 必 全 pass

# 3. subprocess 真实调
python scripts/automation/ai_log_mock.py --log "ERROR test"  # exit 0

# 4. worktree commit 在 origin 远端 (推 origin 是 owner 决定)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f02-log-ai
```

## 6. 输出格式 (per pre-pr-review skill 期望)

子代理返回时, 必须含:

```
# F-02 log AI 端到端实装报告

## 1. 8 commit 链 (per `git log wt-ops-f02-log-ai`)
## 2. 守门实证 (cargo check / test / fmt / clippy / workspace / python / typecheck 输出)
## 3. 16 维守门 0 违反
## 4. 跟既有 star-ops 兼容性 (14 文件清单 + 8 REST 端点 + Hybrid AI 4 級 Ladder)
## 5. 4 表 W/T/M 覆盖 (3 ops_log 表 + 既有 6 表 = 9 表 100%)
## 6. 已知缺口 (per 守门 #11 缺标比错标)
## 7. owner evidence check 准备 (子代理不能自查, owner 必跑)
```

## 7. 失败处理

- 子代理 status=succeeded 但 cargo check 0 err 没跑 → **拒绝接收, 重派**
- 子代理跳过 IT/PT → **拒绝接收, 重派**
- 子代理 commit 不在 wt-ops-f02-log-ai branch → **重置 worktree, 重派**
- 子代理编造历史 (无 git 实证) → **拒绝接收, hotfix 撤回, per 守门 #1 禁回溯**

## 8. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

## 9. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 brief 落档 (9 节 + 8 commit 链 + 守门规则 + 子代理规则 + 失败处理) | 2026-09-08 13:18 JST ask_user `ask_f5016f1d0df89012dfa06965` 拍板 F-02 端到端 |

## 10. 引用文档

- `docs/reports/STAR-P3-WBS-001.md` v0.11 §14.10.2
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-02 + §5.3 mock subprocess
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.2 F-02 LogAI + §5.3 mock
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + §6.2 ladder retriable
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `scripts/automation/ai_log_mock.py` v0.1
- `crates/star-ops/src/ops_ai/{mod,mock,openai_stub,anthropic_stub,ladder}.rs`
- `crates/star-ops/src/ops_api.rs` 8 REST stub
- `crates/star-ops/Cargo.toml`
- `frontend/src/app/ops/page.tsx` 4 tab 骨架
- `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts` opsConsole i18n
- `AGENTS.md` §4 + §4.1 累积规 v1-v26 + §6 ADR 索引
