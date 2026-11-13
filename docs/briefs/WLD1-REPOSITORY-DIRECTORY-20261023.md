# Brief: WLD1-REPOSITORY-DIRECTORY-20261023

**Agent**: worker
**Phase**: WBS-NEXT-20261023
**Created**: 2026-10-23 22:39:15

---

## 目标
推进 WLD-1 identity/provider slice。复核 Project→Repository canonical relation、Run 绑定、创建者/owner grant 与服务端受控 repository-root registry。只有真实持久事实和权限来源均可验证时，才实现 PostgreSQL 只读 repository listing provider 并接入 startup。缺任一来源时只交付审计报告，列出缺口并保持 fail closed；不造 mock/seed、用户路径或隐式目录扫描，不开放 create/import/archive。

## 基线
预 brief dev HEAD=08f2df723c663a41d6d48faf29fcad916b4abddd；brief commit 后的精确 base 由协调者记录。现有 worktree_lifecycle.rs 有 RunWorktreeLifecycleProvider trait、只读 endpoint 和 provider 缺失时 fail-closed 行为；须在当前 base 复核 startup 是否装 concrete provider。
CodeGraph CLI 曾报告索引 up-to-date；重建时 DB 正被 MCP 占用而 EPERM。可用 MCP 没有 check_index_coverage/index_repository，父任务已做有限 codegraph_explore 和定向源码回退；不得声称 coverage 完整。

## 文件清单
仅可改 crates/star-api-rest/src/group_api/worktree_lifecycle_provider.rs、crates/star-api-rest/src/group_api.rs、crates/star-api-rest/src/main.rs、crates/star-api-rest/tests/worktree_lifecycle_provider_routes.rs、docs/reports/PHASE-WLD1-REPOSITORY-DIRECTORY-20261023.md。禁止改 WBS、需求/设计、Cargo.lock、其他 lane、dev/main。若需超出路径或 migration，停止并报告，不扩 manifest。

## 门禁与验证
Rust 1.99.0；遵守 CypherGraph Guardian / Cypher Header；不跨 owner 写表；不把远端 Branch 当 Run 父层；只返回授权且已绑定的安全展示字段，不泄露 URL/主机路径/凭据；缺表、grant、可信 root 或 ACL 时 fail closed。若实现成立，运行 rustfmt 与 cargo test -p star-api-rest --all-targets -j 4、focused provider/PG route tests，并区分 fixture 与目标库证据。若不能实现，报告具体代码/迁移/授权来源及缺口，不新增代码。
提交前检查 diff、status、manifest 边界；提交后回报 base/head/commit、文件、依赖、实跑验证、缺口与 integration 建议。不 push、不 rebase、不清理 worktree。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
