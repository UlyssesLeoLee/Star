# Brief: F-01 Ops Cluster Update 端到端实装 (per WBS §14.10.2 + 拍板 9/8 14:31 JST)

> **状态**: 🟡 Brief v0.1
> **拍板**: 2026-09-08 14:31 JST 用户发令"A" (拍板 4 子项 F-01..F-04 派单优先级: F-01 集群更新)
> **wt-branch**: `wt-ops-f01-cluster-update`
> **base**: `main @ 472bab2` (per PR #25 squash + F-02 收官后)
> **触发**: 2026-09-08 14:21 JST PR #25 merge 后, 累计 2/4 子项 50% 收官, 拍 F-01 继续
> **关联**: [WBS §14.10.2 4 子项端到端](../../reports/STAR-P3-WBS-001.md) · [OPS-DETAILED §3 Hybrid AI + cluster](../../detailed-design/OPS-DETAILED-DESIGN-001.md) · [OPS-BASIC §3.1 F-01 Cluster + §3.3 F-04 Docs](../../basic-design/OPS-BASIC-DESIGN-001.md) · [SRS §4 F-01 §8.1 6 表 W/T/M](../../requirements/SRS-STAR-OPS-001.md) · [ADR-0048 framework 锁 axum 0.8](../../architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md) · [PR #25 F-02 brief 同模式](../../briefs/ops-f02-log-ai-impl.md) · [守门 #9 子代理 RPC 不可靠实证](../../../AGENTS.md)

---

## 1. 目标 (Objective)

实装 `star-ops` 端 4 个 Cluster 端到端 (list_releases / canary / rollback / status), 让 K8s/Helm 灰度/回滚通过 kube-rs 客户端真实调, UI `/ops/page.tsx` cluster tab 接真实 API 返 200 + 解析 + 渲染. 走守门 #1 (R-05 不推 origin 不动生产) + 守门 #13 (W/T/M 100% 覆盖) + 守门 #5 v2 (API key 安全) + 守门 #19 v19 (agent 交互走 scripts/automation/).

**范围** (per WBS §14.10.2 F-01 估 600K token):
- 1 后端实装 (kube-rs client 引入 + 4 cluster endpoint 真实化)
- 1 Helm release DDL 雏形 (跟 F-02 3 表 DDL 同模式)
- 1 前端实装 (ClusterTab.tsx 4 卡片 + useQuery)
- 1 mock 升级 (helm_canary_mock.sh subprocess 路径启用)
- 1 IT 雏形 (kube-rs mock 跨 crate IT + DDL 验证)
- 1 PT 雏形 (criterion bench P95 < 200ms 守门)
- 8 commit 链 + 1 PR 描述 + 1 PHASE 报告

## 2. 范围 (Scope)

### 2.1 In-Scope (F-01 端到端)

**后端**:
- `crates/star-ops/Cargo.toml`: 加 `kube = "0.95"` (per SRS-001 §10.2 F-01 显式) + `k8s-openapi` 派生
- `crates/star-ops/src/ops_domain/cluster.rs`: 4 endpoint 真实 (list_releases 调 `kube::Client` + `helm list` / canary 调 `helm upgrade --canary` / rollback 调 `helm rollback` / status 调 `kubectl get pods`)
- `crates/star-ops/src/ops_api.rs`: cluster_* 4 handler 改真实 (含 release_name / canary_weight / target_revision 校验, 1MB body 限制 per 守门 #5 v2)
- `crates/star-ops/src/ops_ai/ladder.rs`: 不变 (F-02 已 retriable 修, F-01 不动)
- 2 表 DDL 雏形: `db/migrations/2026-09-08-ops-cluster.sql` (ops_helm_release_state T 类 / ops_cluster_action_log T 类, 跟 F-02 3 表同 pattern)
- IT 雏形: `crates/star-ops/tests/it_cluster_update.rs` (kube-rs mock 跨 crate IT + helm mock subprocess 跑通)
- PT: `crates/star-ops/benches/cluster_bench.rs` (criterion P95 < 200ms 守门)

**前端**:
- `frontend/src/app/ops/components/ClusterTab.tsx` 新建 (4 卡片: 列出 release / 触发灰度 / 回滚 / 状态)
- `frontend/src/lib/ops-api.ts`: 加 cluster_* 4 endpoint fetch wrapper (已有 8 endpoint, 扩 4 = 12)
- i18n 3 语言 (zh-CN/en/ja) 加 cluster 文案: `releaseList`, `canarySlider`, `rollbackSelector`, `releaseStatus`, `confirm.canary`, `confirm.rollback`, `error.helmNotFound`, `error.timeout`

**mock (per 守门 #1 R-05 不动生产 + 守门 #19 v19 agent 交互)**:
- `scripts/automation/helm_canary_mock.sh` 新建 (bash mock, 模拟 `helm upgrade --canary` / `helm rollback`, 输出 JSON, subprocess 路径调用)
- `crates/star-ops/src/ops_domain/cluster.rs` 启用 subprocess mock 路径 (跟 F-02 mock.rs 同 pattern)

**文档**:
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md` v0.1 (7 段 per AGENTS.md §3)
- `docs/reports/PR-F01-CLUSTER-UPDATE-001.md` (PR 描述)

### 2.2 Out-of-Scope (不修)

- 真实 K8s cluster 连接 (per 守门 #1 R-05, 仅 mock 路径, owner 拍板后切生产)
- 真实 helm exec (`helm upgrade --canary` 真实执行) — 仅 subprocess mock
- log AI 端到端 (F-02 已收官)
- 运维数据 metrics (F-03 后续)
- 文档扫描 (F-04 后续)
- OAuth 2.0 / mTLS — 复用 `star-context::ActorContext` (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯

## 3. 实施步骤 (8 commit 链)

| # | commit | 标题 | 估 token | 关键守门 |
|---|---|---|---|---|
| 1 | `wt1` | feat(ops): helm_canary_mock.sh 新建 (subprocess mock, 守門 #1 R-05 不动生产) | 40K | #1 R-05 + #19 v19 + #24 v2 |
| 2 | `wt2` | feat(ops): Cargo.toml 加 kube = "0.95" + k8s-openapi 派生 (per SRS-001 §10.2) | 50K | #4.2 唯一实施入口 + ADR-0048 |
| 3 | `wt3` | feat(ops-domain): cluster.rs 4 endpoint 真实 (list_releases/canary/rollback/status 调 helm_canary_mock.sh) | 180K | #1 R-05 + #5 v2 + #13 RLS 13 類 |
| 4 | `wt4` | feat(ops-api): cluster_* 4 handler 真实 + 1MB body 限制 (per 守門 #5 v2) | 100K | #5 v2 + #1 v25 |
| 5 | `wt5` | feat(db): 2 表 DDL 雏形 (ops_helm_release_state T + ops_cluster_action_log T, per 守門 #13 W/T/M 100%) | 60K | #13 100% 覆盖 |
| 6 | `wt6` | test(ops): IT 跨 crate 雏形 (kube-rs mock + helm subprocess 真调) + criterion bench P95 < 200ms | 80K | #1 v25 + #1 v3 |
| 7 | `wt7` | feat(frontend): ClusterTab.tsx (4 卡片 + useQuery) + ops-api.ts 扩 4 endpoint + i18n 3 语言 | 60K | #6 v2 + #5 v2 |
| 8 | `wt8` | docs(phase): PHASE-F01-CLUSTER-UPDATE-REPORT v0.1 (7 段) + PR-F01-CLUSTER-UPDATE-001 描述 | 30K | #12 + #21 v21 |
| **累计** | | | **~600K** | |

每个 commit 必先 `git log -p --follow <file>` 实证 worktree commit 在 chain 上 (per 守門 #9 主体), author = Ulysses (per 守門 #10). commit message 含 守門编号引用.

## 4. 守門 (per AGENTS.md §4 累积规 v1-v26 + brief F-02 同模式)

每 commit 必跑 6 项 (Windows PowerShell 7, 跟 F-02 实证):

```powershell
# 1. cargo check -p star-ops (单 crate per 守門 #1 v25)
cargo check -p star-ops --all-targets -j 4  # 0 err

# 2. cargo test -p star-ops (单 crate 跳过 workspace per 守門 #1 v25 + v26)
cargo test -p star-ops --lib -j 4  # 全 pass, 含 F-01 IT

# 3. cargo fmt + clippy
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 0 err (advisory per 守門 #7 v3)

# 4. workspace 必跑 (per 守門 #1 v1 派生, ~1m 19s 实证)
cargo check --workspace --all-targets -j 4  # 0 err

# 5. bash mock 跑通 (守門 #1 R-05 + 守門 #19 v19 subprocess)
bash scripts/automation/helm_canary_mock.sh --release star-mcp --canary-weight 10  # exit 0, JSON 输出

# 6. frontend typecheck 我改的文件 0 错
cd frontend && npx tsc --noEmit  # 0 错 (我改的)
```

**16 维守門** (per WBS §14.10.5, 跟 F-02 实证同):
\#1 4 守門 + #1 v19/v25 + #3 #4 #5 v2 #6 v2 #7 v3 #9 #10 #11 #12 #13 + #14 v2 + #19 v19 #21 v21 #23 #24 v2 + #25 v25 + #26 v26.

**新增 8 表累计** (F-02 3 + F-01 2 = 5 + 既有 6 = 11 表 W/T/M 100% 覆盖, per 守門 #13).

## 5. 子代理 brief 规则 (per 守門 #9 v20 + v3 实证, 跟 F-02 同模式)

### 5.1 派前必做 (owner 必先)

1. 落档本 brief (本文件, 已落档)
2. worktree commit brief + push origin (per 守門 #9 v20, 必先 `git log -p --follow docs/briefs/ops-f01-cluster-update-impl.md` 实证)
3. 派 worker 子代理 (run_in_background=true), 引用本 brief 路径
4. 子代理 status="succeeded" ≠ 实际成功, **owner 必在子代理返回后跑 owner evidence check** (per 守門 #9 主体实证, F-02 实证 5/5 通过经验)

### 5.2 子代理不能擅自做的 (跟 F-02 实证同)

- 推 origin (必 owner 拍板 per 守門 #1 反转 8/30)
- merge main (必 PR 流程 per 守門 #26 v26)
- 改 守門 16 维任何一条 (必 owner 拍板)
- 跳过 cargo check 守门 (必跑 0 err)
- 跳过 IT/PT 验证 (必跑 跨 crate IT + P95 守门)
- 编造历史 (守門 #1 禁回溯)
- 真实 K8s/Helm 调用 (守門 #1 R-05, 仅 mock 路径)

### 5.3 owner evidence check (per 守門 #9 主体)

子代理 status=succeeded 后, owner 必**实际跑** (跟 F-02 实证同 6 项):

```powershell
# 1. 8 commit 在 wt-ops-f01-cluster-update branch 上
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline main..wt-ops-f01-cluster-update | Measure-Object -Line  # 必 8/9 commit

# 2. 跨 crate 实证
cd D:\Star\.worktrees\wt-ops-f01-cluster-update
cargo check -p star-ops --all-targets -j 4  # 必 0 err
cargo test -p star-ops --lib -j 4  # 必 31+ pass
cargo test -p star-ops --tests -j 4  # 必 lib + IT pass
cargo bench -p star-ops --bench cluster_bench -- --quick  # P95 < 200ms 实证 (子代理实测)
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4
cargo check --workspace --all-targets -j 4  # 0 err

# 3. bash mock 真调
bash scripts/automation/helm_canary_mock.sh --release star-mcp --canary-weight 10  # exit 0, JSON

# 4. worktree commit 在 origin 远端
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f01-cluster-update
# 期望: 1 row hash (brief)

# 5. frontend typecheck (F-01 4 新文件 0 错)
cd frontend && npx tsc --noEmit 2>&1 | Select-String -Pattern "src/app/ops/components/ClusterTab|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

## 6. 输出格式 (跟 F-02 实证同, per pre-pr-review skill)

子代理返回时, 必须含:

```
# F-01 cluster update 端到端实装报告 (worker 子代理 output)

## 1. 8 commit 链 (per `git log wt-ops-f01-cluster-update`)
## 2. 守門实证 (cargo check / test / fmt / clippy / workspace / bash mock / typecheck 输出)
## 3. 16 维守門 0 违反
## 4. 跟既有 star-ops 兼容性 (15 文件清单 + 12 REST 端点 + Hybrid AI 4 級 Ladder)
## 5. 11 表 W/T/M 覆盖 (F-01 2 + F-02 3 + 既有 6 = 11 表 100%)
## 6. 已知缺口 (per 守門 #11 缺标比错标, 至少 5 项)
## 7. owner evidence check 准备
## 8. 待 PR 描述 (`docs/reports/PR-F01-CLUSTER-UPDATE-001.md` 摘要)
```

## 7. 失败处理 (per 守門 #9 主体实证 + F-02 实证同)

- cargo check 任何 0 err 不通过 → **修, 不跳过**
- cargo test 任何 fail → **修, 不跳过**
- 16 维守門任何违反 → **修, 不跳过** (per 守門 #11 缺标比错标)
- 推 origin → **不推**, 留给 owner 拍板 (per 守門 #1 反转 8/30 拍板)
- merge main → **不 merge**, 留给 owner 拍板 (per 守門 #26 v26)
- 编造历史 (无 git 实证) → **绝对禁止** (per 守門 #1 禁回溯)
- 8 commit 缺 1 → **重做, 不接受 7 commit "差不多"**
- token 超 600K 估 → **停下来, 报告 owner, 不擅自扩 scope**
- 真实 K8s/Helm 调用 → **不调**, 仅 mock (per 守門 #1 R-05)

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
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 brief 落档 (9 节 + 8 commit 链 + 守门规则 + 子代理规则 + 失败处理) | 2026-09-08 14:31 JST 用户发令"A" (F-01 拍板) |

## 10. 引用文档

- `docs/reports/STAR-P3-WBS-001.md` v0.11 §14.10.2
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-01 + §10.2 kube = "0.95" 引入
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.1 F-01 Cluster + §3.3 F-04 Docs
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + cluster 集成
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `docs/briefs/ops-f02-log-ai-impl.md` v0.1 (F-02 实证同模式参考)
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (F-02 报告, 7 段模式)
- `crates/star-ops/src/ops_domain/cluster.rs` (F-02 改 mock 路径前)
- `crates/star-ops/src/ops_api.rs` 8 REST stub (4 cluster + 2 log + 1 metrics + 1 docs)
- `crates/star-ops/Cargo.toml`
- `frontend/src/app/ops/page.tsx` 4 tab 骨架
- `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts`
- `AGENTS.md` §4 + §4.1 累积规 v1-v26 + §6 ADR 索引
