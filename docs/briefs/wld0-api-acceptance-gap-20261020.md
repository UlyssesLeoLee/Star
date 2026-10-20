# Brief: wld0-api-acceptance-gap-20261020

**Agent**: explorer
**Phase**: WLD-0
**Created**: 2026-10-20 09:19:51

---

你是 WLD-0 API acceptance gap worker。仅在本 worktree 做有界只读审计，不改产品代码、SQL、测试、数据库，也不运行目标 DB。

范围：crates/star-api-rest/src/group_api/run_worktree_bindings.rs、engineering_runs.rs、cli_sessions.rs，crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs，以及 2026-10-19 Run/Worktree binding migration 和 WLD0 当前审计报告。
产出 attach/reassign/detach acceptance matrix：Project/Run 授权来源、actor scope、Expected relation/focus version、SCD2、幂等键、事务内审计与 Outbox、回滚/冲突语义；逐项列现有测试证据与缺口。提出最小但充分的 PostgreSQL/RLS 集成测试矩阵，覆盖成功写入、精确重放、冲突重放、撤权、跨 Run/Project、CAS 并发竞争、回滚、重复 current row fail-closed；标注哪些必须在 target DB 执行。报告只新增 docs/reports/WLD0-API-ACCEPTANCE-GAP-20261020.md，引用源码路径和行号，不把单元测试或 lazy-pool 路由测试当作数据库验收。不得编辑其他文件。完成后只汇报证据、diff-check 结果和未解决问题，不创建 commit。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
