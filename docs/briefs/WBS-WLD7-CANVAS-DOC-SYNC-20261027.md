# Brief: WBS-WLD7-CANVAS-DOC-SYNC-20261027

**Agent**: codex
**Phase**: wld7-canvas-doc-sync-20261027
**Created**: 2026-10-27 07:11:51

---

# WBS WLD-7 Canvas 规范文档最小同步

你负责一个已登记的隔离 lane。唯一可写 manifest：
- docs/requirements/SRS-CANVAS-AGENT-001.md
- docs/design/BD-CANVAS-AGENT-001.md
- docs/design/DD-CANVAS-AGENT-001.md
- docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md
- docs/briefs/WBS-WLD7-CANVAS-DOC-SYNC-20261027.md（dispatcher 生成，不编辑）
- docs/reports/PHASE-WBS-WLD7-CANVAS-DOC-SYNC-20261027.md
- docs/reports/WBS-DISPATCH-WLD7-CANVAS-DOC-SYNC-20261027.jsonl（dispatcher audit，不手改）

开始前先完整读 dispatcher brief，并以已合入当前 dev 的审计报告 docs/reviews/WBS-WLD7-CANVAS-CANDIDATE-AUDIT-20261027.md 为逐项依据。审计结论：仅选择性吸收 Keep/Adapt 项，不整份覆盖候选文档。当前 dev 基线为 345c048774535dfb9d7ccbf35aa2d7a68be5772f；旧候选仅供只读比对：分支 codex/wld7-canvas-a12-alignment-20261025，commit 5e5463bb12cf34a53d779e63f7ddc1b139236b76。不得访问其工作树。

规范必须保持：
1. 产品导航 Project → Engineering Run → Worktree → 本地 Branch；Run 可没有宿主 Branch；provider/repository/remote branch 只是可选元数据，不能成为 Run 父级、创建前提或授权前提。
2. Worktree/Run 是授权与数据边界；Canvas 与 Task Card/工作项是选定 Worktree/Run 下的同级能力。不能用远端 Branch 代替本地 checkout 状态。
3. 安全草稿默认仅作者可见；持久冲突只向当前获授权参与者暴露；读取、订阅、恢复、解决每次都重新授权；撤权即停止推送并拒绝读取。
4. Keep owner-local 事务、revision、audit、Outbox 轻量唤醒与按资源授权 fan-out，但全部标为目标契约；不得声称 durable push、持久草稿/冲突或跨实例行为已实现。
5. 把第二份完整 proposal 抵达 owner 定义为 ≤1000 ms 的测试刺激；它不是 commit-time 差值、检测窗口或 holdback。PT-09 的 p99 ≤1 秒从冲突事实提交后计时；两者分别描述。
6. 明确 Resolve 流程：OT 仅做无副作用预览，用户确认后按当前 revision CAS 提交；取消不得产生副作用；stale CAS 保留双方草稿与冲突事实。更新 AC-COLLAB-09、E2E-COLLAB-10/11 映射，引用当前 WBS §22.2 与审计所核对的 spec v1.2、数据设计 v0.4、测试设计 v1.7。对引用版本先核对当前 dev 文件，若已更新则采用实际版本并同步 WBS。
7. 候选中的容量数字只作为待测设计参数；双 gateway、目标 PostgreSQL/RLS、恢复、资源与 PT-09 均不得写成已验证。旧 endpoint/schema/transport/CRDT/LWW 内容如保留，必须清楚标成历史/非规范，不与当前 owner 契约并列生效。
8. 更新三份文档的版本/修订历史，并将 WBS v0.65 更新到 v0.66，追加 §23.6，说明本次只完成设计同步，WLD-7/PT-09/W16 仍未因文档而关闭，并列明后续实现和验收顺序。不可改写历史条目或编造历史。
9. 阶段报告按仓库格式包含 §0–§7：目的、改动矩阵、验证摘要、已知缺口、交接/失败检查、守门规则、签字栏、修订历史。清楚声明仅静态一致性审阅与 rtk git diff --check；不运行测试、构建、Cargo、数据库、VM、浏览器、桌面或性能验收。
10. 新增或修改文本中不得出现竞争产品品牌、对标/仿制措辞；用中性能力描述。

硬边界：不修改或清理旧 WLD-7 branch/worktree；不读取、复制、覆盖、修改或清理 D:\Star 中用户已有的移动/删除/修改文件；不碰其他 worktree/branch；不修改 main、不 fetch、不 push。只改上述 manifest。完成后做静态自审和 rtk git diff --check，不提交；向协调者报告文件、版本、差异、验证、未完成项和当前 HEAD。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
