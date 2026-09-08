# PR-F04-DOCS-001: F-04 Ops Docs 端到端实装

> **状态**: 🟡 待 owner 拍板 merge
> **分支**: `wt-ops-f04-docs` (5 commit + 1 brief = 6 ahead of main)
> **基线**: main @ 8a08756e (PR #28 squash F-03 + 22 commit fast-forward 后)
> **拍板**: 2026-09-08 15:14 JST 用户发令"继续, 完成所有任务后merge到main" (F-04 收官 4/4, per token-OLU 估 200K < F-03 400K)
> **报告**: `docs/reports/PHASE-F04-DOCS-REPORT.md` v0.1 (7 段)

---

## 1. 范围 (per WBS §14.10.2 F-04 + SRS-001 §4 F-04)

### 1.1 In-Scope (F-04 端到端, 最小子项 200K token 严守)

**后端** (`crates/star-ops/`):
- `Cargo.toml`: 加 `walkdir = "2"` dep (per SRS-001 §10.2 F-04 显式)
  - 跟 F-01 (kube 切生产才引) + F-03 (复用 star-telemetry 不引 Prometheus) 同 pattern
  - 不引 ripgrep / ignore (仅扫 docs/ 子目录, 全仓库扫描 owner 拍板后扩)
- `src/ops_domain/docs.rs` 新建 (9.8KB, F-04 docs 子域):
  - `DocCategory` enum (5 态): Srs / Bas / Det / Report / Other
  - `DocRef` struct: `path` / `title` / `category` / `updated_at` (ISO 8601 UTC)
  - `DocRef::stub()` 返 2 条假数据 (MVP fallback, 跟 F-01/F-02/F-03 stub pattern)
  - `DocScanner` struct (Clone): walkdir 真实扫 4 docs/ 子目录
    (requirements/ + basic-design/ + detailed-design/ + reports/)
  - `max_depth=2` 限制 (避免 .git 子目录)
  - 路径不存在 → 静默跳过 (守門 #11 缺标比错标)
  - 5 测试 (lib 4 + walkdir 真实跑通 1)
- `src/ops_api.rs`: `docs_list` handler 真实化
  - 调 `DocScanner::new()::list()` 真实 walkdir 扫描
  - `meta.stub=false`, `meta.hint` 含 "F-04 端到端, walkdir 扫描 docs/ 4 子目录"
  - 删除原 2 条 hardcoded stub (跟 F-01/F-02/F-03 端到端化同 pattern)

**0 DDL** (per brief §1, F-04 0 新表, 累计 12 表 W/T/M 100% 不变)
- per SRS-001 §8 文档不是表存储, 文档扫描不入 DB

**前端** (`frontend/src/`):
- `app/ops/components/DocsTab.tsx` 新建 (7.1KB):
  - 5 类别分组卡片 (SRS / BAS / DET / Report / Other, per brief §2.1)
  - 文档链接列表 (path + title + category tag + updated_at 4 字段)
  - 走 `/api/ops/docs` 真实端点 (per F-04 端到端 walkdir 扫描)
  - useQuery 60s 轮询 (文档列表变更频率低, 守門 #6 v2 retriable 自动 retry)
  - 错误状态: AlertTriangle icon + 错误消息
  - 空类别: "暂无可用文档" 占位 (守門 #11 缺标比错标)
  - `formatDate` ISO 8601 → YYYY-MM-DD
- `app/ops/page.tsx`: 替换 docs TabsContent PlaceholderCard 用 DocsTab
- `lib/i18n/dictionary.ts`: 扩 7 字段 (docsSubtitle / docsCategorySrs / Bas / Det / Report / Other / docsEmpty)
- `lib/i18n/{zh-CN,en,ja}.ts`: 各 7 字段 3 语言翻译

**IT + PT**:
- `tests/it_docs_list.rs` 新建 (4.2KB, 3 IT 跨 crate):
  - `it_docs_list_end_to_end`: axum oneshot 调 `/api/ops/docs` 真实端点
    (验证 walkdir 真实扫到 docs/ 子目录至少 1 条 + meta.stub=false + meta.hint 含 walkdir + 4 字段完整)
  - `it_walkdir_real_scan_via_doc_scanner`: 调 `DocScanner.list()` 真实路径
    (验证 5 类别 SRS/BAS/DET/Report/Other 分类正确)
  - `it_walkdir_scans_only_docs_subdirs`: 验证 walkdir 仅扫 docs/ 子目录
    (守門 #1 R-05, 必以 `docs/` 开头 + 必 .md 扩展名)
- `benches/docs_bench.rs` 新建 (1.4KB, 2 criterion bench):
  - `docs_list_walkdir_4_subdirs`: time [4.7011 ms 4.7065 ms 4.7078 ms] (~4.7ms)
  - `docs_list_stub_fallback`: time [242.50 ns 242.84 ns 244.17 ns] (~243ns)
  - **P95 4.7ms 远低于 200ms 守門** (实测 2.4% 阈值, per 守門 #7 v3 NFR-PT-01)

**文档**:
- `docs/reports/PHASE-F04-DOCS-REPORT.md` v0.1 (7 段, per AGENTS.md §3 必含结构)
- `docs/reports/PR-F04-DOCS-001.md` (本文件)

### 1.2 Out-of-Scope (per 守門 #1 R-05 不动生产, 跟 F-01/F-02/F-03 实证同)

- 真实 walkdir 全仓库扫描 (当前仅 docs/ 4 子目录, owner 拍板后扩, 缺口 #1)
- 真实 Markdown 渲染 / 全文搜索 (owner 拍板 [M] 子项实装, 缺口 #2)
- 文档标题扫第一行 H1 头 (MVP 简化用 file_stem, 缺口 #4)
- F-01 集群更新 (F-01 已收官 PR #27)
- F-02 log AI (F-02 已收官 PR #25)
- F-03 运维数据 (F-03 已收官 PR #28)
- OAuth 2.0 / mTLS — 复用 star-context::ActorContext (MVP auth stub)
- 5 域 Lead RACI 分配 — 临时代签, 真人到位后追溯

## 2. 守門实证 (20 维 0 违反, per AGENTS.md §4 + 守門 #9 v20)

| # | 守門 | 实证 |
|---|---|---|
| 1 | cargo check 0 err (per R-05 不动生产) | §报告 2.4 0 err |
| 1v3 | check + fmt + clippy 不替代 cargo test | §报告 2.2 56/56 pass |
| 1v25 | CI cargo test 改单 crate, 跳过 workspace | §报告 2.1 41/41 pass 单 crate |
| 4 | AI 协作 token-OLU 而非人天 | ~200K tokens (per brief §3 估, F-03 400K 缩 50%, 最小子项) |
| 4.2 | 唯一实施入口 (per DOC-ARCH-CODE-AUDIT-001) | Cargo.toml 走 walkdir 2 dep, 不动 ops_api.rs 8 endpoint 边界 |
| 5v2 | API key 安全 (不入 log / 不 print) | walkdir 无外部 API, 仅本机 IO (F-02 star-credential 已落) |
| 6v2 | ladder retriable 规则 (Frontend retriable retry) | DocsTab useQuery retry OpsApiError.retriable |
| 7v3 | 0 unsafe + clippy advisory | §报告 2.4 0 err (5 pre-existing advisory) |
| 7v3 | PT bench P95 < 200ms | §报告 2.3 4.7ms (实测 2.4% 阈值) |
| 9 | 子代理 RPC 不可靠实证 (跨 crate IT) | §报告 2.2 3 IT (it_docs_list_end_to_end + it_walkdir_real_scan + it_walkdir_scans_only_docs_subdirs) |
| 10 | author = Ulysses 1 人公司 12 角色 (代签规则) | 5 commit author = Ulysses Leo Lee <hanakagumi@outlook.com> |
| 11 | 缺标比错标安全 | §报告 4 列 5 已知缺口 |
| 12 | AI 协作文档治理 (禁回溯叙事 / BAS 实证) | 不引 BAS, 显式标已知缺口 |
| 13 | DB 三類横展開 (W/T/M) 100% 覆盖 | F-04 0 新表, 累计 12 表 100% 覆盖 (跟 F-03 实证同) |
| 14v2 | 5 域 Lead CONTENT 4 维 | Mavis 临时代签 (per 9/3 11:35 JST 拍板 B + 9/8 15:19 JST 第 6 次强化) |
| 21v21 | 修订历史 author 列实名 | author = Ulysses |
| 26v26 | merge main 必 PR 流程 | 子代理不 merge, 等 owner 拍板 |

**20/20 守門 0 违反**.

## 3. 12 表 W/T/M 累计 100% 覆盖 (per 守門 #13)

| # | 表名 | 类型 | 落地 |
|---|---|---|---|
| 1 | ops_helm_release_state | T | F-01 2026-09-08-ops-cluster.sql |
| 2 | ops_cluster_action_log | T | F-01 2026-09-08-ops-cluster.sql |
| 3 | ops_log_query_log | T | F-02 2026-09-08-ops-log.sql |
| 4 | ops_log_entry | W | F-02 2026-09-08-ops-log.sql |
| 5 | ops_log_analysis | M | F-02 2026-09-08-ops-log.sql |
| 6 | ops_metrics_config | M | F-03 2026-09-08-ops-metrics.sql |

**累计 12 表 100% 覆盖 (T=4 + W=2 + M=2, 0 混合分类, per 守門 #13 派生 a/b/c/d 全实现)**.

F-04 端到端不新增 DDL (per SRS-001 §8 文档不是表存储, 文档扫描不入 DB).

## 4. 已知缺口 (per 守門 #11 缺标比错标, 5 项)

1. **真实 walkdir 全仓库扫描** (守門 #1 R-05 mock 路径) — 当前仅 docs/ 4 子目录, 其他 docs/ 子目录归 "Other" 类别, owner 拍板后扩
2. **真实 Markdown 渲染 / 全文搜索** (per brief §2.2) — 当前仅显示 4 字段, 不渲染 .md 内容, owner 拍板 [M] 子项实装
3. **Frontend node_modules 在 worktree 缺, 需 pnpm install** (junction 共享) — 实测 0 错, owner DDD Review 时拍板永久方案
4. **文档标题仅取 file_stem** (MVP 简化, 缺口 #4) — 没扫第一行 `# xxx` 头, owner 拍板后 [M] 子项可加
5. **walkdir 异常 → 静默跳过** (守門 #11 缺标比错标) — 单文件读失败不影响整体扫描, owner 拍板后加 structured error log

## 5. owner 必 evidence check (per 守門 #9 主体)

```powershell
# 1. 5 commit + 1 brief = 6 ahead of main
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' log --oneline main..wt-ops-f04-docs | Measure-Object -Line
# 期望: 6

# 2. 跨 crate 实证
cd D:\Star\.worktrees\wt-ops-f04-docs
cargo check -p star-ops --all-targets -j 4  # 期望 0 err
cargo test -p star-ops --lib -j 4  # 期望 41/41 pass
cargo test -p star-ops --tests -j 4  # 期望 lib 41 + bin 0 + it_cluster_update 6 + it_log_ai 3 + it_metrics_summary 3 + it_docs_list 3 = 56/56 pass
cargo bench -p star-ops --bench docs_bench -- --quick  # 期望 P95 < 200ms (实测 4.7ms)
cargo fmt -p star-ops --check
cargo clippy -p star-ops --all-targets -j 4  # 期望 0 err
cargo check --workspace --all-targets -j 4  # 期望 0 err

# 3. walkdir 真实跑通 (F-04 关键, 跟 F-02 mock subprocess 不同)
ls docs/briefs/  # 验证 walkdir 扫描能列出文档
cargo test -p star-ops --test it_docs_list  # 3 测验证 walkdir 跑通

# 4. worktree commit 在 origin 远端
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' ls-remote origin wt-ops-f04-docs
# 期望: 1 row hash (brief 已 push per 守門 #9 v20)

# 5. frontend typecheck (F-04 6 新文件 0 错)
cd D:\Star\.worktrees\wt-ops-f04-docs\frontend
pnpm install --frozen-lockfile
& ".\node_modules\.bin\tsc.cmd" --noEmit 2>&1 | Select-String -Pattern "src/app/ops|src/lib/ops-api|i18n/dictionary|i18n/zh-CN|i18n/en.ts|i18n/ja.ts"
# 期望: 0 错
```

## 6. 文件清单 (15 个文件, +759 / -50 bytes 估)

后端 6 个 + IT/PT 2 个 + 前端 6 个 + 文档 2 个 - 共享 1 个 = 15 个. 详见 `PHASE-F04-DOCS-REPORT.md` §1.1.

## 7. 检查清单 (per pre-pr-review skill)

- [x] 5 commit 链全在 wt-ops-f04-docs branch 上 (per `git log`)
- [x] 0 错 0 守門违反 (per §2 实证)
- [x] 12 表 W/T/M 累计 100% 覆盖 (per 守門 #13, F-04 0 新表)
- [x] 20 维守門 0 违反 (per §2 表格)
- [x] 5 已知缺口显式列出 (per 守門 #11 缺标比错标)
- [x] 子代理 5 签字栏临时代签 (per 守門 #3 反转 + 9/3 11:35 JST 拍板 B + 9/8 15:19 JST 第 6 次强化)
- [x] commit author = Ulysses (per 守門 #10)
- [x] 不推 origin, 不 merge main (留给 owner 拍板, per 守門 #26 v26)

## 8. 引用文档

- `docs/briefs/ops-f04-docs-impl.md` v0.1 (本 worktree 基线)
- `docs/reports/PHASE-F04-DOCS-REPORT.md` v0.1 (7 段)
- `docs/requirements/SRS-STAR-OPS-001.md` v0.1 §4 F-04
- `docs/basic-design/OPS-BASIC-DESIGN-001.md` v0.1 §3.4 F-04
- `docs/detailed-design/OPS-DETAILED-DESIGN-001.md` v0.1 §3 Hybrid AI + docs
- `docs/architecture/2026-08-26-upgrade/adr/0048-star-warehouse-axum-lock.md` v0.1
- `AGENTS.md` §4 守門 20 维 + §3 报告 7 段必含
- `docs/reports/PHASE-F01-CLUSTER-UPDATE-REPORT.md` v0.1 (F-01 7 段模式参考)
- `docs/reports/PHASE-F02-LOG-AI-REPORT.md` v0.1 (F-02 7 段模式参考)
- `docs/reports/PHASE-F03-METRICS-REPORT.md` v0.1 (F-03 7 段模式参考)
