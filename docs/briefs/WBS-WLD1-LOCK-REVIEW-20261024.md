# Brief: WBS-WLD1-LOCK-REVIEW-20261024

**Agent**: codex-subagent lockfile auditor
**Phase**: WBS-WLD1-LOCK-REVIEW-20261024
**Created**: 2026-10-24 02:54:29

---

# 任务：WBS-WLD1-LOCK-REVIEW-20261024

## 目标
只读审查 WLD1 新增 `star-pg-adapter -> domain-scm` 直接依赖所需的 `Cargo.lock` 最小差异，给出可复核的集成建议。不得编辑 Cargo.toml、Cargo.lock、源码或执行 merge/rebase/cherry-pick/fetch/push/commit。

## 基线与输入
- 当前 dev checkout：`C:\Users\leo19\orca\workspaces\Star\dev-converge-20261007`，HEAD `d4f26ee965106c5bf70dbe07c0b05cbb5639c06b`。
- WLD1 实施 lane：`C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star`，HEAD `5059508e5dcea98cd5a7a3aceb99e85ada60da10`，Cargo.toml 已声明 `domain-scm`，Cargo.lock 的 `star-pg-adapter` 依赖列表尚未更新。
- 本地只读候选依赖分支：`origin/dependabot/cargo/rust-dependencies-fe3c6ef10d`，观察到 HEAD `48f26101`；不要 fetch，remote-tracking refs 新鲜度未知。
- Rust/Cargo 当前不可用，勿运行或声称 Cargo 验证通过。

## 审查问题
1. `domain-scm` package 是否已存在于 dev Cargo.lock；当前 `star-pg-adapter` package entry 缺失的最小依赖边是什么？该边是否可以纯手工确定，或必须由与仓库完全一致的 Cargo 1.99.0 重新生成？
2. 候选 Dependabot commit 的 Cargo.lock 变化包含哪些无关版本升级/降级？逐项概括范围和对把整个 lockfile 搬入 WLD1 的风险。
3. 是否有不依赖大范围 Dependabot diff 的受控方案；如果没有，列出精确阻断条件和必须等待的验证。
4. 不要让 review report 将静态推断写成 Cargo 已生成/验证；清楚区分可见锁文件证据和推断。

## 文件所有权
仅写 `docs/reports/PHASE-WLD1-LOCK-REVIEW-20261024.md`；dispatcher 自动写入 `docs/reports/WBS-DISPATCH-WLD1-LOCK-REVIEW-20261024.jsonl`。本 brief 位于 `docs/briefs/WBS-WLD1-LOCK-REVIEW-20261024.md`。不写其他路径，不改 WLD1 实施/审查 worktree。

## 方法和交付
- 所有 Git 命令通过仓库指定 RTK；仅读取 dev、WLD1 和上述本地 candidate refs。
- 对 `Cargo.lock` 目标 package、workspace package dependencies、差异统计和 candidate 版本变更给出文件/行号或 package 名证据。
- 输出中文只读报告，按仓库报告 7 段结构组织（目的、矩阵/结论、验证摘要、已知缺口、子代理接手、守门规则、签字栏、修订历史）。
- report 写入后只做针对报告的 `git diff --check`；提交/集成由父任务决定。
- 不沿用竞争架构或引入新依赖；无充分证据时给出 fail-closed 结论。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
