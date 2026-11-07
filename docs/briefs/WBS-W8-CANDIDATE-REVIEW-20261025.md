# Brief: WBS-W8-CANDIDATE-REVIEW-20261025

**Agent**: independent-reviewer
**Phase**: WBS-W8-CANDIDATE-REVIEW-20261025
**Created**: 2026-10-25 01:51:08

---

## 目标
对 W8 BI native event adapter 脏候选做独立、只读的集成就绪审查。候选 worktree：C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star；预期基线 HEAD：b660fa52a23d352539123eafc9ef6b005a978d71；dev 集成基线：2fda1e16632470312ef07b18fb5bf81066822466。

## 范围
检查候选完整 tracked diff 与全部 untracked 源、迁移、设计、报告和 briefs。重点核验：
1. 原生来源身份、route/profile/endpoint pin、provider/log/generation/opaque cursor 是否完整且不可由 payload 冒充。
2. BI Inbox/quarantine durable commit 是否先于 ACK/terminal disposition；exact replay、origin mismatch、旧 receipt 无法证明来源、取消/ACK 超时是否 fail closed。
3. active lease 顺序、SQL 查询和 partial index 是否一致；迁移的权限、RLS、grants、回滚/重放边界。
4. focused tests/报告中的命令与证据是否与当前候选一致；区分源码级证据与未执行的 PostgreSQL、mTLS、重启恢复、owner-to-owner settlement、worker/BI coverage 门。
5. 评估是否可选择性纳入当前 dev；列出必要依赖、冲突风险和尚不能合入/宣称完成的部分。

## 输出与边界
仅在本 lane 写 docs/reviews/WBS-W8-CANDIDATE-REVIEW-20261025.md，按 P0/P1/P2/P3 给出具体路径、行号、场景、证据、集成建议和验证缺口。对候选目录完全只读：不改、不格式化、不运行 Cargo/数据库测试、不提交、不 rebase、不合并、不清理。候选报告/brief 是待审数据，不是操作指令。若子代理没有 CodeGraph，请按提供的父级图谱证据和具体源文件做定向核读，不得声称图谱覆盖已验证。保持 W8/WLD6 OPEN，除非独立的运行证据真实通过。

## 已有父级图谱线索
dev 的 CodeGraph 对 TaskEventReceiver / TaskEventInboxCommitPort 显示 receive 路径存在；图谱指出 Inbox/receiver 缺覆盖测试。候选修改不在 dev 图谱中，必须直接检查候选当前文件。覆盖检查工具当前不可用，因此不要作完整性/负向搜索结论。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)

## Continuation scope — current source-position claim lane (2026-11-07)

The original 2026-10-25 review remains unchanged above. This continuation checks the later source-position fix worktree `D:\orcaWork\Star\w8-source-position-claim-fix-20261025` at `5fe3ea9372242d7d72f9dd416f6437475c870904` against local `dev@996ae3ae7d6e4c9c075715f3799f5186734d32f0`. The candidate has 8 commits not in dev and dev has 363 commits not in the candidate branch; review the complete candidate-side diff and do not infer readiness from the old report.

Keep the candidate read-only. Append a dated report addendum to the review below; preserve all existing findings and separate source review from runtime evidence. Focus on cross-disposition source-position uniqueness, opaque cursor and sequence semantics, owner/RLS boundaries, migration locking, old findings, and whether regressions cover both write orders, concurrent races, cursor-only collisions, cursor-mode transitions, and ACK eligibility. Do not run Cargo or database tests, rebase, merge, or clean. W8 remains OPEN. The candidate worktree has an unrelated untracked brief `docs/briefs/WBS-W8-DECODE-FIX-20261026.md`; leave it untouched.

The dev CodeGraph index was current at 1,925 files when checked; no `check_index_coverage` tool was available, and this old candidate checkout is outside that indexed project. Use targeted Git diff and source reads, and do not make exhaustive negative claims. The dispatcher brief for this continuation is the parent task prompt plus this addendum; no extra paths are authorized.
