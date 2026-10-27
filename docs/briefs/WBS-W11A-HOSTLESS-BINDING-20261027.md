# Brief: WBS-W11A-HOSTLESS-BINDING-20261027

**Agent**: worker
**Phase**: WBS-W11A-HOSTLESS-BINDING-20261027
**Created**: 2026-10-27 05:23:11

---

任务：推进 WBS §23.2 的 W11-A hostless Run binding 设计同步，只处理无冲突且已登记的文档。

基线：dev@4f629ce5103fb4a9fb67a775341475ef66bbf2ea。

唯一可编辑文件：docs/requirements/SRS-WLD1-REPOSITORY-BOUNDARY-001.md、docs/basic-design/BD-WLD1-REPOSITORY-BOUNDARY-001.md、docs/detailed-design/DD-WLD1-REPOSITORY-BOUNDARY-001.md、docs/data-design/phase9f4d-run-owner-wtm.md、docs/data-design/phase9f4d-infrastructure-owner-wtm.md、本 brief 与 docs/reports/PHASE-WBS-W11A-HOSTLESS-BINDING-20261027.md。不要写其他文件。

MAGOS 已显示若尝试编辑 docs/design/DD-LOCAL-INFRASTRUCTURE-001.md 会与用户 D:\Star main 的未提交删除/移动冲突；该目录中 DD-LOCAL 文档正由用户移动到 docs/detailed-design，必须原样保留。不要编辑、覆盖、删除、暂存、复制或清理 D:\Star 的任何内容，也不要编辑目标新路径。当前 W14 remediation lane 持有 WBS 文件，所以本 lane 不得修改 WBS；在报告中注明 WBS 同步待后续 lane。

设计目标：Engineering Run 的 identity、创建/授权/list 均不需要宿主 Cloud Branch。远端 Repository/Branch/provider 只作为可选、版本化 metadata，不作为 Run 父级、授权前提或本地 Branch 替代。RunInfrastructureBinding 仅在 Run 明确选择基础设施 placement 时存在；选择 K3s 时须固定 tenant/project/run scope、Infrastructure Profile digest、HostEnvironment 与 K3s cluster 的精确 ID+generation、namespace、runtime profile 与预算。未选择 placement 的 Run 可保持无 binding；涉及指定执行环境的 TaskExecutionRun 仍必须检查其所需 placement 与 readiness，缺少时 fail closed。Worktree local checkout ref 属于 Worktree scope，不写入 Run identity。

同步 API/schema/migration/digest/replay 规则：设计一个明确的新 wire/schema version（建议 V3），不把 nil UUID 或伪造 Branch 用作无宿主占位；新 canonical digest domain 分离版本、Run scope、可选 remote metadata 与安全敏感 placement/profile generation；同一幂等键的历史 V2 replay 必须按旧 V2 canonicalization 验证，V2 和 V3 不跨版本猜测/重哈希，不得把无效旧 Branch 映射为空；新写只用 V3。迁移方案需 additive、兼容、可审计；只有可从权威 Project/Repository owner 证明的旧 metadata 才迁移，模糊/撤权行隔离，不合成身份。只写设计，不改 Rust/SQL migration，不声称实现或 DB 验证通过。

改动要求：在指定 SRS/BD/DD 和两个 W/T/M 说明中统一术语、权限边界、API字段、schema 版本、迁移、摘要与回放兼容及验收标准；文档保留已有 Multipass 两实例上限、owner-local Inbox/Outbox、RLS 与 fail-closed 约束。同步各自版本/修订历史，标明哪些是设计决策，哪些实现仍未验证。对外产品文案不得引用竞品品牌。

报告必须逐项说明修改文件、字段/迁移/digest/replay决策、与旧 V2 的兼容边界、W11-A 完成度和未完成的 DD-LOCAL/WBS 同步门。不得声称 W11-A 全部关闭。验证只做文档 diff、自审、交叉术语/版本和 Markdown 静态检查；不运行 tests/build/数据库/VM/运行时，也不改源代码。不要提交、rebase 或 merge，完成后交还给 root。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
