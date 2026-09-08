# Brief: F-04 Ops Docs 端到端实装 (per WBS §14.10.2 + 拍板 9/8 15:14 JST 续)

> **状态**: 🟡 Brief v0.1
> **拍板**: 2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-04 收官, 估 200K token [S] 优先)
> **wt-branch**: `wt-ops-f04-docs`
> **base**: `main @ 8a08756e` (per PR #28 squash F-03 + 22 commit fast-forward 后)
> **触发**: 2026-09-08 15:35 JST PR #28 squash merge 后, F-03 收官 3.75/4 子项 93.75%, 拍 F-04 收官 4/4
> **关联**: [WBS §14.10.2 4 子项端到端](../../reports/STAR-P3-WBS-001.md) · [OPS-DETAILED §3 Hybrid AI + docs](../../detailed-design/OPS-DETAILED-DESIGN-001.md) · [OPS-BASIC §3.4 F-04 Docs](../../basic-design/OPS-BASIC-DESIGN-001.md) · [SRS §4 F-04 §8.1 12 表 W/T/M](../../requirements/SRS-STAR-OPS-001.md) · [ADR-0048 framework 锁 axum 0.8](../../architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md) · [PR #28 F-03 squash 8a08756e](https://github.com/UlyssesLeoLee/Star/pull/28) · [PR #27 F-01 squash d8e916e](https://github.com/UlyssesLeoLee/Star/pull/27) · [PR #25 F-02 squash 472bab2](https://github.com/UlyssesLeoLee/Star/pull/25) · [PR #23 MVP squash 97810c0d](https://github.com/UlyssesLeoLee/Star/pull/23) · [守門 #9 子代理 RPC 不可靠实证](../../../AGENTS.md) · [brief F-03](../../briefs/ops-f03-metrics-impl.md) · [brief F-01](../../briefs/ops-f01-cluster-update-impl.md) · [brief F-02](../../briefs/ops-f02-log-ai-impl.md)

---

## 1. 目标 (Objective)

实装 `star-ops` 端 `/api/ops/docs` 端到端, 让运维文档 (走 walkdir 扫描 + 列表) 真实化, UI `/ops/page.tsx` docs tab 接真实 API 返 200 + 解析 + 渲染. 走守门 #13 (W/T/M 100% 覆盖) + 守门 #5 v2 (API key 安全) + 守门 #1 v25 (CI cargo test 单 crate).

**范围** (per WBS §14.10.2 F-04 估 200K token [S] 优先, 估最小子项):
- 1 后端实装 (walkdir 引入 + docs.rs 真实化 + 1 endpoint 真实化)
- 1 前端实装 (DocsTab.tsx 文档链接列表)
- 0 DDL (F-04 不新增表, 跟既有 12 ops 表 W/T/M 累计, per 守门 #13 100% 覆盖已落)
- 1 IT 跨 crate 雏形 (axum oneshot + walkdir mock 跑通)
- 1 PT 雏形 (criterion bench P95 < 200ms 守门)
- 1 i18n 3 语言 (zh-CN/en/ja) docs 文案 (F-02 已落 opsConsole.docsTitle + docsCategory, 复用 + 扩)
- 5 commit 链 + 1 PR 描述 + 1 PHASE 报告

**累计 12 表 W/T/M** (F-04 0 新 + F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表, 守门 #13 100% 覆盖)

## 2. 范围 (Scope)

### 2.1 In-Scope (F-04 端到端, 最小子项)

**后端**:
- `crates/star-ops/Cargo.toml`: 加 `walkdir = "2"` (per SRS-001 §10.2 F-04 显式)
- `crates/star-ops/src/ops_domain/docs.rs` 新建 (F-04 docs 子域, 1 数据结构 + 1 stub method, 跟 F-02 既有 metrics/log pattern 同)
  - `DocRef` struct: path/title/category/updated_at
  - `list()` 调 walkdir 扫描 `docs/reports/` + `docs/requirements/` + `docs/basic-design/` + `docs/detailed-design/`, 返 DocRef 列表
- `crates/star-ops/src/ops_api.rs`: docs_list handler 真实 (调 docs.rs list)
- `crates/star-ops/src/ops_domain/mod.rs`: pub use DocRef
- 0 DDL (F-04 不新增表, per brief §2.2 守门 #13 100% 覆盖已落)
- IT 雏形: `crates/star-ops/tests/it_docs_list.rs` (axum oneshot + walkdir 扫描 docs/ 真实跑通)
- PT 雏形: `crates/star-ops/benches/docs_bench.rs` (criterion P95 < 200ms)

**前端**:
- `frontend/src/app/ops/components/DocsTab.tsx` 新建 (文档链接列表, 5 类分组: SRS / BAS / DET / 报告 / 其他, 跟 SRS-001 §4 F-04 卡片一致)
- `frontend/src/app/ops/page.tsx` 改 docs TabsContent 用 DocsTab (跟 F-02 logai + F-01 cluster 同 pattern)
- i18n 3 语言 (zh-CN/en/ja) 加 docs 文案: `docsCard.srs`, `docsCard.bas`, `docsCard.det`, `docsCard.report`, `docsCard.other`, `docs.path`, `docs.updated` (F-02 已落 `opsConsole.docsTitle + docsCategory` 复用)

**文档**:
- `docs/reports/PHASE-F04-DOCS-REPORT.md` v0.1 (7 段 per AGENTS.md §3)
- `docs/reports/PR-F04-DOCS-001.md` (PR 描述)

### 2.2 Out-of-Scope (不修, 跟 F-01/F-02/F-03 实证同)

- 真实 walkdir 全仓库扫描 (MVP 仅扫 docs/ 子目录, owner 拍板后扩)
- 真实 Markdown 渲染 / 全文搜索 (owner 拍板后 [M] 子项实装)
- log AI 端到端 (F-02 已收官 PR #25)
- cluster update 端到端 (F-01 已收官 PR #27)
- 运维数据 metrics 端到端 (F-03 已收官 PR #28)
- OAuth 2.0 / mTLS — 复用 star-context::ActorContext (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯
- DDL 落档 (F-04 0 新表, per SRS §8 文档不是表存储)

## 3. 实施步骤 (5 commit 链)

| # | commit | 标题 | 估 token | 关键守門 |
|---|---|---|---|---|
| 1 | `wt1` | feat(ops): Cargo.toml 加 walkdir = "2" + docs.rs 新建 (F-04 端到端, per SRS-001 §4 F-04) | 30K | #4.2 唯一实施入口 + ADR-0048 + F-01/F-03 同 pattern (不加 kube) |
| 2 | `wt2` | feat(ops-api): docs_list handler 真实 (调 docs.rs walkdir 扫描 4 docs 子目录) | 60K | #1 R-05 + #5 v2 + #1 v25 |
| 3 | `wt3` | feat(frontend): DocsTab.tsx + page.tsx 改 docs TabsContent + i18n 3 语言 | 40K | #6 v2 + #5 v2 |
| 4 | `wt4` | test(ops): IT 跨 crate (axum oneshot + walkdir 真实跑) + criterion bench P95 < 200ms | 50K | #1 v25 + #1 v3 + #7 v3 |
| 5 | `wt5` | docs(phase): PHASE-F04-DOCS-REPORT v0.1 (7 段) + PR-F04-DOCS-001 描述 | 20K | #12 + #21 v21 |
| **累计** | | | **~200K (严守, F-04 是最小子项)** | |

每个 commit 必先 `git log -p --follow <file>` 实证 worktree commit 在 chain 上 (per 守門 #9 主体), author = Ulysses (per 守門 #10). commit message 含 守門编号引用 (跟 F-01/F-02/F-03 实证同).

## 4. 守門 (per AGENTS.md §4 累积规 v1-v26 + F-01/F-03 实证同)

每 commit 必跑 6 项 (Windows PowerShell 7, 跟 F-03 owner 接手 5/5 同):

```powershell
# 1. cargo check -p star-ops (单 crate, 守門 #1 v25 实证, ~0.5s)
cargo check -p star-ops --all-targets -j 4  # 0 err

# 2. cargo test -p star-ops (单 crate, 含 IT, 守門 #1 v25)
cargo test -p star-ops --lib -j 4  # 全 pass, 含 IT 雏形

# 3. cargo fmt + clippy
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 0 err (advisory per 守門 #7 v3)

# 4. workspace 必跑 (per 守門 #1 v1 派生, ~1m 19s 实证)
cargo check --workspace --all-targets -j 4  # 0 err

# 5. walkdir 跑通 (F-04 跟 F-03 不同, 走 walkdir 真实扫描, 守門 #1 R-05 不动生产路径)
# 跑 cargo test -p star-ops --test it_docs_list 验证 walkdir 扫描

# 6. frontend typecheck 我改的文件 0 错
cd frontend && npx tsc --noEmit 2>&1 | Select-String -Pattern "src/app/ops|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

**20 维守門** 跟 F-01 实证同, 全 0 违反. **12 表 W/T/M 累计 100% 覆盖** (F-04 0 新 + F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表, per 守門 #13).

## 5. 子代理 brief 规则 (per 守門 #9 v20 + v3 实证, F-01/F-03 owner 接手 5/5 实证经验)

### 5.1 派前必做 (owner 必先)

1. 落档本 brief (本文件, 已落档)
2. worktree commit brief + push origin (per 守門 #9 v20, 必先 `git log -p --follow docs/briefs/ops-f04-docs-impl.md` 实证)
3. 派 worker 子代理 (run_in_background=true), 引用本 brief 路径
4. 子代理 status="succeeded" ≠ 实际成功, **owner 必在子代理返回后跑 owner evidence check 5/5** (per 守門 #9 主体实证 + F-01/F-03 owner 接手 5/5 实证经验, 拒绝子代理"5 min 快速成功" report)

### 5.2 子代理不能擅自做的 (F-01/F-03 实证同)

- 推 origin (必 owner 拍板 per 守門 #1 反转 8/30 拍板)
- merge main (必 PR 流程 per 守門 #26 v26)
- 改 守門 20 维任何一条 (必 owner 拍板)
- 跳过 cargo check 守门 (必跑 0 err)
- 跳过 IT/PT 验证 (必跑 跨 crate IT + P95 守门)
- 编造历史 (守門 #1 禁回溯)
- 5 commit 缺 1 → **重做, 不接受 4 commit "差不多"** (F-04 严控 200K, 最小子项)
- token 超 200K 估 → **停下来, 报告 owner, 不擅自扩 scope** (F-01 600K 严控实证, F-02 850K 略超 6% 可接受, F-03 严守 400K, F-04 严守 200K)
- 引入 kube → **不引** (F-01 教训: 切生产才要, MVP 不需要)
- 真实 walkdir 全仓库扫描 (仅 docs/ 子目录, owner 拍板后扩)
- 新增 DDL 表 (F-04 0 新表, per SRS §8)

### 5.3 owner evidence check (per 守門 #9 主体)

子代理 status=succeeded 后, owner 必**实际跑** (F-03 实证 5 项):

```powershell
# 1. 5 commit + 1 brief = 6 ahead of main (brief 在 main 仓, F-04 5 commit 在 wt branch)
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline main..wt-ops-f04-docs | Measure-Object -Line  # 必 5/6

# 2. 跨 crate 实证
cd D:\Star\.worktrees\wt-ops-f04-docs
cargo check -p star-ops --all-targets -j 4  # 必 0 err
cargo test -p star-ops --lib -j 4  # 必 36+ pass
cargo test -p star-ops --tests -j 4  # 必 lib + IT pass (it_docs_list 3+ 测)
cargo bench -p star-ops --bench docs_bench -- --quick  # P95 < 200ms 实证
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 0 err
cargo check --workspace --all-targets -j 4  # 0 err

# 3. walkdir 真实跑通 (F-04 关键, 跟 F-02 mock subprocess 不同)
ls docs/briefs/  # 验证 walkdir 扫描能列出文档
cargo test -p star-ops --test it_docs_list  # 3+ 测验证 walkdir 跑通

# 4. worktree commit 在 origin 远端
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f04-docs
# 期望: 1 row hash (brief 已 push per 守門 #9 v20)

# 5. frontend typecheck (F-04 2 新文件 0 错)
cd frontend && npx tsc --noEmit 2>&1 | Select-String -Pattern "src/app/ops/components/DocsTab|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

## 6. 输出格式 (跟 F-01/F-02/F-03 实证同, per pre-pr-review skill)

```
# F-04 docs 端到端实装报告 (worker 子代理 output)

## 1. 5 commit 链 (per `git log wt-ops-f04-docs`)
## 2. 守門实证 (cargo check / test / fmt / clippy / workspace / typecheck 输出)
## 3. 20 维守門 0 违反
## 4. 跟既有 star-ops 兼容性 (15 文件清单 + 10 REST 端点 + Hybrid AI 4 級 Ladder)
## 5. 12 表 W/T/M 覆盖 (F-04 0 新 + F-03 1 + F-02 3 + F-01 2 + 既有 6 = 12 表 100%, per 守門 #13)
## 6. 已知缺口 (per 守門 #11 缺标比错标, 至少 5 项)
## 7. owner evidence check 准备
## 8. 待 PR 描述 (`docs/reports/PR-F04-DOCS-001.md` 摘要)
```

## 7. 失败处理 (per 守門 #9 主体实证 + F-01/F-03 owner 接手 5/5 经验)

- cargo check 任何 0 err 不通过 → **修, 不跳过**
- cargo test 任何 fail → **修, 不跳过**
- 20 维守門任何违反 → **修, 不跳过** (per 守門 #11 缺标比错标)
- 推 origin → **不推**, 留给 owner 拍板 (per 守門 #1 反转 8/30 拍板)
- merge main → **不 merge**, 留给 owner 拍板 (per 守門 #26 v26)
- 编造历史 (无 git 实证) → **绝对禁止** (per 守門 #1 禁回溯)
- 5 commit 缺 1 → **重做, 不接受 4 commit "差不多"** (F-04 严控 200K)
- token 超 200K 估 → **停下来, 报告 owner, 不擅自扩 scope** (F-04 是最小子项, 严守 200K)
- 真实 walkdir 全仓库扫描 → **不调**, 仅 docs/ 子目录 (per 守門 #1 R-05)
- 引入 kube → **不引** (F-01 教训)
- 新增 DDL 表 → **不新** (F-04 0 新表)

## 8. 时间预算

F-03 实证 owner 接手 5/5 + 6 commit 链 ~25-40 min. 子代理 ~30-50 min. **总估 30-60 min** (F-04 估 200K 比 F-03 400K 小 50%, 时间预算 50% 缩, 最小子项).

子代理 status=succeeded ≠ 实际成功, owner 必 evidence check (per 守門 #9 主体 10 background task ERR_CONNECTION_CLOSED 教训 + F-01/F-03 owner 接手 5/5 实证).

## 9. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-08 JST |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 临时代签 | 2026-09-08 JST |

## 10. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版 brief 落档 (10 节 + 5 commit 链 + 守门规则 + 子代理规则 + 失败处理) | 2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-04 收官, 4/4 子项 100% 收官) |

## 11. 引用文档

- `docs/reports/STAR-P3-WBS-001.md` v0.11 §14.10.2 + §15 累计
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-04
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.4 F-04 Docs
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + docs 集成
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (F-03 7 段模式参考)
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md` v0.1 (F-01 7 段模式参考)
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (F-02 7 段模式参考)
- `docs/briefs/ops-f03-metrics-impl.md` v0.1 (F-03 brief 模式参考)
- `docs/briefs/ops-f01-cluster-update-impl.md` v0.1 (F-01 brief 模式参考)
- `crates/star-ops/src/ops_domain/` (F-04 新建 docs.rs, 跟 metrics/log/cluster pattern)
- `crates/star-ops/src/ops_api.rs` 9 REST stub (F-04 改 docs_list 真实)
- `crates/star-ops/Cargo.toml` (F-04 加 walkdir)
- `frontend/src/app/ops/page.tsx` 4 tab 骨架 (F-04 改 docs TabsContent)
- `frontend/src/lib/i18n/{dictionary,zh-CN,en,ja}.ts`
- `AGENTS.md` §4 + §4.1 累积规 v1-v26 + §6 ADR 索引
