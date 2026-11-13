# Brief: W16-IMPLEMENTATION-DECISIONS-20261107

**Agent**: reviewer
**Phase**: W16-IMPLEMENTATION-DECISIONS-20261107
**Created**: 2026-11-07 07:56:24

---

目标：在 dev@89f2f5bd 的隔离 worktree 内冻结 W16 Identity/Auth 撤权 Outbox → Collaboration Inbox 的实现前接口与数据决策。范围严格限于本 lane 登记的 8 份 SRS/BD/DD/测试设计/brief/phase report/review/dispatch 文件；不改 Rust 代码、migration、WBS 或根工作区文件，不访问受保护的 candidate worktree。决策主题：设备 generation 与 owner-local source_sequence 的关系；一致性 snapshot 与有界 replay API；受认证的服务身份和 tenant scope；撤权 freshness lease；Collaboration owner 的 contiguous checkpoint；版本化 writer-readiness gate 与 legacy writer/session drain；绑定行锁下 replay-floor 初始化/回填；Outbox/Inbox/receipt/quarantine/projection/route work 的 W/T/M、索引、RLS、grants 与 TTL。对照当前源码和文档后给出可实施合同；所有新运行验收保持 NOT_RUN；不把局部 PG/HTTP PASS 扩大成完整实现。过程说明：初步只读 reviewer 已在 brief 创建前完成，本文件为事后如实留档，不声称 brief 早于该次派发。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
