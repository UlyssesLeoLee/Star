# FSMS 集成状态与后续裁决

**状态**：2026-10-12 并入 `dev` 的未裁决设计提案。它保存了分支中的需求、接口、CLI 和实现规划，尚未被接受为产品架构或实现承诺。

## 与当前导航/归属规范的差异

当前主导航和协作边界为 **Project → Cloud Branch → Engineering Run → 本地 Worktree**。Run 是任务、Canvas、Inbox、BI/Benchmark、Workflow/LangGraph 与插件应用协作的 canonical scope；Worktree 是绑定 Branch 的本地 checkout。现有提案将 FileSession 直接放在 Worktree 下，并与 Task Card 并列，未经裁决不能视为当前层级。

## 实施前必须裁决

- FileSession 是 Run 内的能力、Worktree 文件视图中的操作，还是独立可插拔 App；不可新增未批准的主导航层级。
- 每个会话如何固定 Project/Branch/Run/Worktree/Task/Actor 身份、授权快照、上下文版本与资源预算。
- 文件级会话是否复用 Star Agent/Run execution、原生 Hook、取消恢复、独立验证、事件和审计边界；外部 CLI 只能经受控 adapter。
- Git 持久化、数据库索引、API/CLI 与并发冲突的唯一事实源和恢复语义。
- Rust 桌面性能、内存/队列上限、扩展隔离、依赖许可证与分发闭包。

## 版本状态说明

API 规范、CLI 参考和实施计划是设计草案，不代表端点、命令、crate 或运行闭环已经实现。合并此文档包只保留提案供后续审查，不改变主设计、导航或执行授权。裁决完成后，应修订需求、基本设计、详细设计及主实施计划，并明确未实现项与验收证据。