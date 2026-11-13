# Brief: STAR-STABLE-W4-AUDIT-SHA-20261103

**Agent**: codex-worker-w4
**Phase**: W4-AUDIT-SHA-20261103
**Created**: 2026-11-03 03:35:39

---

任务：实施 W4 cargo-audit release asset SHA/filename 最小修复。

基线：本 lane 是 MAGOS 登记的独立 worktree，目标 dev@234ea49eafa9f8a6acf466b32f134f9518aeebec。当前 license-gate workflow 中 cargo-audit asset 名与 URL 不匹配，SHA 仍为占位符。官方 release API 与直接下载复核记录位于 docs/reviews/WBS-STAR-STABLE-W4-UPSTREAM-ASSET-REVIEW-20261102.md：版本 0.22.2，asset 名 cargo-audit-x86_64-unknown-linux-musl-v0.22.2.tgz，SHA-256 7fb9497f8594b389e5fce5ef9b92db08432996895b2e0c5a0167a69ed445c428。

范围仅限登记文件：.github/workflows/license-gate.yml、docs/briefs/STAR-STABLE-W4-AUDIT-SHA-20261103.md、docs/reports/PHASE-STAR-STABLE-W4-AUDIT-SHA-20261103.md、docs/reviews/STAR-STABLE-W4-AUDIT-SHA-REVIEW-20261103.md、docs/reports/WBS-DISPATCH-STAR-STABLE-W4-AUDIT-SHA-20261103.jsonl。WBS 主文件由协调者/独立 WBS lane 持有，本 lane 不编辑它。

实现要求：仅修复 asset filename、release download URL 和 SHA-256 对应关系；保持 fail-closed、版本固定、与根 Rust 1.99.0 baseline 一致。不得从 PR #292 带入其余 workflow 或 Rust toolchain 变更，不触碰该 PR、main、远端或 D:\Star 用户文件。不要下载或执行 cargo-audit 二进制，也不要声称 hosted workflow 已通过。

验证：在本机运行可用的最小静态验证（YAML 解析、shell block 语法或已有 workflow 验证器；记录缺少的工具），确认 SHA/文件名/URL 三者一致、无占位符且 diff 仅落在登记文件。检查 git diff --check。阶段报告按仓库报告结构记录目的、改动矩阵、实测验证、缺口、失败/接手清单、守门规则、签批栏和修订历史；review 由你做候选自审，清楚标注协调者仍需独立复核。提交 lane commit 并回报 hash、实际改动路径、命令及结果。未完成代码提交前不要宣称 W4 或 stable release 关闭。

CodeGraph 当前仓库未索引（无 .codegraph），本 lane 只改 workflow/doc，无需结构图查询。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
