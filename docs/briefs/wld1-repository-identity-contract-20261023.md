# Brief: wld1-repository-identity-contract-20261023

**Agent**: worker
**Phase**: WBS-WLD-NEXT-20261023
**Created**: 2026-10-23 23:07:19

---

只编辑 manifest 内的设计文档和新增阶段报告，不改代码、迁移或目标数据库。

目标：把 WLD-1 下一阶段缺失的 Project→Repository / Host root / Run 绑定来源契约落到需求、基本设计、详细设计和追踪文档中，并与当前已合入 dev 的 WLD-1 审计报告一致。不要把缺少的持久化或 provider 写成已实现。

采用的设计决策（若当前源码/文档存在实质矛盾，记录冲突并停在提案状态）：
1. SCM 领域拥有稳定、不可依赖宿主 Branch 的 Repository 主身份；外部 provider/repository key 是可选元数据。
2. Project 领域拥有 Project↔Repository 授权关联和默认选择；关联必须有版本/状态并可审计。不得把现有 domain-project ProjectRepository 持久化端口误称为 Repository master。
3. 可信 Host Infrastructure owner 拥有按 HostEnvironment + RepositoryId + generation 管理的持久本地 root 注册及验证状态。原始路径仅在可信 host capability 内解析，不返回 API/UI，不交给 Agent/Plugin/任务 CLI 任意访问。
4. Engineering Run 的 repository 绑定是显式可空字段；NULL 表示 Run 尚未选择 Repository，不能由 Project 默认、云端 Branch 或历史 Worktree 推断。Project repository 查询只返回有授权的候选目录；Run-scoped 返回需同时校验 Project membership 和 Run read grant，但候选可见性不构成 Run binding 或 Worktree 权限。
5. 只有显式 Run↔Repository binding、当前授权、fresh Host root registration 和生命周期 provider 全部有效后，才允许 Worktree create/import/CLI；缺任一项继续 fail closed。hostless Run 可存在且可列出，但未绑定 Repo 的 Run 没有可执行 Worktree。

要求：
- 先复核 PHASE-WLD1-REPOSITORY-DIRECTORY-20261023.md 及现有 SRS/BD/DD 对应章节；保留 hostless Run 与 Project→Run→Worktree→Branch 层级。
- 同步这些文件：docs/requirements.md、docs/basic-design.md、docs/requirements/SRS-WORKTREE-CANVAS-001.md、docs/design/BD-WORKTREE-CANVAS-001.md、docs/design/DD-WORKTREE-GROUP-001.md、docs/design/TRACEABILITY-WORKTREE-CANVAS-001.md。
- 新建 docs/reports/PHASE-WLD1-REPOSITORY-IDENTITY-CONTRACT-20261023.md，按项目 PHASE 报告 7 段模板：目的、改动矩阵、验证摘要、已知缺口、子代理清单、守门核对、签字栏和修订历史。只证明设计决策，不声称实现/数据库/运行时验收。
- 补 requirement/design/acceptance traceability 与修订历史。对不应由 Star 内部 owner 控制的外部来源，明确列出交付依赖；不虚构已有 owner。
- 新增产品文案不得出现竞争产品品牌名。保留必要的内部兼容名称只在技术层。
- 验证 git diff --check 和文档交叉引用；不跑测试，因为没有代码变化。

交付前检查唯一写入范围为本 brief manifest 和新增报告，不编辑 WBS 文件（协调者集成后更新 WBS）。Commit message 引用 docs/briefs/wld1-repository-identity-contract-20261023.md。提交后回报 base/head、改动文件、验证和未完成实现门.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
