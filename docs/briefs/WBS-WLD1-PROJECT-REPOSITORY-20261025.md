# Brief: WBS-WLD1-PROJECT-REPOSITORY-20261025

**Agent**: codex-worker
**Phase**: WBS-WLD1
**Created**: 2026-10-25 09:50:19

---

## 目标
在当前 dev 基线的隔离 Worktree 中推进 WBS 子项。不要把部分源码切片标记为生产闭环。

## 当前基线与 CodeGraph 证据
- 基线：dev HEAD 32d9a3ad727ca81c51adc3ef8e1a29df79d00b10。
- 父 Agent 已按 Verify tier 查询当前 dev 图谱；CodeGraph 返回的候选包含 PgWorktreeRepository 与 MultiProviderIdentityPort，但未给出 canonical Project→Repository 关系或具体只读 Project Repository provider。仅有 codegraph_explore 工具可用，coverage 专用工具不可用；请在本分支按 AGENTS 要求对相关源码、迁移和路由做定向核对，不据图谱缺项直接断言不存在。
- dev 的 WBS 已明确当前 WLD-1 缺 Project→Repository 权威关系、Run→Repository owner command、Host 持久可信根目录和独立只读 provider；Worktree 写入需 fail closed。

## 授权范围与文件所有权
- 仅在此 Worktree 操作：C:\Users\leo19\.codex\worktrees\wld1-project-repository\Star
- 你负责 Domain Project/SCM association、只读 repository provider/API、对应迁移/权限/测试及 WLD-1 专属报告或详细设计。
- 不改共享 WBS、根 AGENTS.md 或其他 lane 的代码；把需集成的 WBS/AGENTS 更新点写入交接报告，由协调者统一处理。
- 不要触碰 D:\Star、dirty 的 wld1-repository-contract-20261025、wbs-wld1-boundary-review-20261025 或其他 Worktree；本项目多人并行，请不要回滚/重置他人的改动。

## 实施边界
- 先确定 canonical Project、Repository 与 tenant 授权/所有者及关系基数，再决定能否实现。不得把远端 Branch 当 Run 父节点；Run 可 hostless。
- 仅实现明确安全的只读 Project→Repository 关联/list slice。未有可信 Host-owned local-root registry 时，不探测任意磁盘路径、不创建/导入/删除 Worktree，也不执行 host shell。
- 沿用 owner-local transaction、当前 actor/Project 授权、RLS、审计与 idempotency 约束。禁止 mock、seed、默认 actor 或内存数据冒充 provider。
- 若关系基数、owner 或迁移安全仍无法由代码与已批准设计判定，停止相应实现，提供具体待决问题和证据；不要发明约束。

## 完成标准
1. 变更只覆盖本 brief 所有权范围，代码关系头/图关系按仓库规则更新。
2. 为实际改动运行最窄必要的 fmt/编译/单测/迁移检查；准确记录未能运行的目标 DB/RLS 门。
3. 在本 Worktree 提交完整实现和报告，列出文件、验证结果、开放门；不得声称 WLD-1 整体完成。
4. 工作前后检查本 Worktree 的分支与脏状态；不改其他 checkout。

## Read COMMENTS
请读取并遵守 dispatcher 自动附加的当前任务留言。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
