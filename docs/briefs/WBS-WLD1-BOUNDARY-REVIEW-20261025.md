# Brief: WBS-WLD1-BOUNDARY-REVIEW-20261025

**Agent**: independent-reviewer
**Phase**: WBS-WLD1-BOUNDARY-REVIEW-20261025
**Created**: 2026-10-25 01:51:09

---

## 目标
对当前 dev 中 WLD1 尚未闭合的 Project→Repository 授权、Host-owned trusted root registry、只读 Worktree provider 边界做只读差距审查，并给出可实施的最小依赖 DAG 与文件清单。不得把旧的通用 import registry 误认为生产 Project/Run 授权闭环。

## 当前事实与设计门
dev HEAD：2fda1e16632470312ef07b18fb5bf81066822466。WBS §11 要求继续补齐 Project→Repository grant/master relation、Host-owned root registry、read-only provider 和目标 PostgreSQL/RLS 证据；Worktree 写仍 fail closed。优先检查 crates/star-api-rest/src/group_api/worktrees.rs、crates/domain-scm/**、crates/star-pg-adapter/**、相关 migrations、WLD1 reports、requirements/basic/design/DD 与当前 WBS。

## 父级 CodeGraph 证据
dev 的图谱检索命中 PgWorktreeRepository（crates/worktree-service/src/pg_worktree_repository.rs:196）、ExternalWorktreeImportRegistry（crates/worktree-shared-dir/src/external_worktree_import.rs:531）、API 的 ProjectRepositoryEnvelope。ExternalWorktreeImportRegistry 是 RwLock<HashMap<Uuid, PathBuf>> 的旧内存 registry，不能证明 Host-owned durable root registry 或 Project/Run authorization。图谱未能在一次有界查询中证明授权关系已闭合。Coverage 工具本次不可用；不要从未命中推断不存在，必要时按具体字符串/模块做定向 source fallback。

## 审查问题
- 区分 Project grants、Repository master、Run optional provider metadata、Host root registry、read-only scan 和 Worktree mutation 的 owner / database / authorization boundary。
- 核查当前迁移/RLS/应用 role/transaction/idempotency/replay 对应证据；报告运行时缺口。
- 识别重复或冲突的 owner/type/migration，以及哪些已有分支或 lane 已承载 WLD1 文件，避免重复改动。
- 给一个最小、依赖明确、文件清单互斥的实施计划和 acceptance matrix；说明可安全并行的 lane 与必须串行的 gates。

## 输出与边界
仅在本 lane 写 docs/reviews/WBS-WLD1-BOUNDARY-REVIEW-20261025.md。本任务只读审查，不改生产代码或既有设计文件，不跑测试，不提交/rebase/merge/删除任何 worktree。输出按“已证实 / 未证实 / 差距 / 推荐 lane 与验收”分栏；对关键结论给文件行号和证据，不把计划或 mock 当作已完成。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
