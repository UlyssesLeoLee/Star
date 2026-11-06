# Brief: PHASE-NAMING-CLI-REMEDIATION-20261106

**Agent**: worker
**Phase**: PHASE-NAMING-CLI-REMEDIATION-20261106
**Created**: 2026-11-05 16:29:51

---

完成命名审计 A3/A8 的最小修复：将 star CLI 的 worktree picker/help 文案与 star-workflow 默认工作流公开描述改为中性能力名称，删除用户可见的外部产品/调研引用；保留命令、参数、序列化和兼容 ID 行为。只修改 crates/star-cli/src/main.rs、crates/star-cli/src/commands/worktree.rs、crates/star-workflow/src/lib.rs；不要改 skill_registry.rs 或旧协议/数据库 ID。运行对应 Rust 包测试/帮助构建和静态检查，并在报告中按 AGENTS.md §0–§7 记录每个文件、命令、结果和未验证项。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
