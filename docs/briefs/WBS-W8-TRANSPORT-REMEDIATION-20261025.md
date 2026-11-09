# Brief: WBS-W8-TRANSPORT-REMEDIATION-20261025

**Agent**: codex-worker
**Phase**: WBS-W8
**Created**: 2026-10-25 09:50:19

---

## 目标
在当前 dev 基线的隔离 Worktree 中审查并选择性修复 W8 native transport 候选的两个 blocker，符合已批准的 owner-local durable event 与 Rust-native replaceable transport 设计。

## 当前基线与 CodeGraph 证据
- 基线：dev HEAD 32d9a3ad727ca81c51adc3ef8e1a29df79d00b10。
- 父 Agent 在 dev 图谱确认 TaskEventReceiveTransport::pull_one 为 at-most-one delivery 的 receive contract，TaskEventReceiveOrigin 保存 provider/log/generation/sequence/message identity；CodeGraph coverage 专用工具不可用，源码结论须定向核对。
- 只读候选：C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star（detached、dirty）。只读 review 报告指出 (P1) migration revoke runtime EXECUTE on platform_transport.pull_one 后未恢复受限 role grant；(P2) legacy source-sequence uniqueness 被弱化，允许相同 origin sequence / different message_id 绕过冲突隔离。PostgreSQL/RLS/mTLS/restart/owner settlement 尚未证明。

## 授权范围与文件所有权
- 仅在此 Worktree 操作：C:\Users\leo19\.codex\worktrees\wbs-w8-transport-remediation\Star
- 负责 W8 transport adapter、关联 migration/grants、quarantine uniqueness、针对性测试和 W8 专属报告。
- dirty candidate Worktree 仅作只读参考；不得修改、stage、commit、archive 或清理。不要触碰 D:\Star 或其他 lane。项目多人并行，请不要回滚/重置他人的改动。
- 不改共享 WBS 和根 AGENTS.md；将应更新的 WBS/规则写入交接报告，由协调者统一处理。

## 实施边界
- 从当前 dev 开始，逐个审查并只移植与已批准 W8 owner/transport 契约相容的必要内容；不要整分支 merge、复制无关大改造、引入不必要依赖或放宽 fail-closed。
- 必须修正并测试：pull_one 所需 runtime 权限仅授予明确受限 service role，不能恢复 PUBLIC/宽泛 grant；同一 pinned log generation/source sequence 上不同 message_id 必须被隔离为 identity conflict，精确重复只返回既有 receipt。
- 若所需 runtime/迁移并不存在于 dev，不得只加“看起来修复了”的孤立 migration；说明依赖链，并将最小可运行闭环实现出来或如实标为未完成。
- 不使用内存队列/socket write 成功冒充 durable acceptance。不能声称 target PostgreSQL grants/RLS、跨实例、mTLS、重启恢复或生产 settlement 已通过，除非实际证据覆盖。

## 完成标准
1. 变更只覆盖本 brief 所有权范围；代码关系头按仓库规则更新。
2. 加入能区分正例/反例的针对性测试：least-privilege pull_one、相同序号不同 message_id 冲突、exact replay 稳定回执；迁移可重复应用/权限错误 fail closed。
3. 运行最窄必要 Rust/SQL gates；记录命令和结果，目标 DB/mTLS/恢复等未跑项明确 NOT_RUN。
4. 在本 Worktree 提交结果和交接报告，指出哪些候选内容被选择性采纳及理由；不声称 W8/整体 BI 已完成。

## Read COMMENTS
请读取并遵守 dispatcher 自动附加的当前任务留言。


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
