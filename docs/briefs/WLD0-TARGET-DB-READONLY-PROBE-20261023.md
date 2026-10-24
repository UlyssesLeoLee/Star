# Brief: WLD0-TARGET-DB-READONLY-PROBE-20261023

**Agent**: wld0_database_probe_worker
**Phase**: WBS-WLD0-TARGET-DB-PROBE
**Created**: 2026-10-23 18:22:35

---

# WLD-0 目标数据库只读取证探针

## 任务与基线

- 任务：为 WBS §5 的 WLD-0 目标数据库门禁增加一个显式、安全、只读的 PostgreSQL 目标探针。
- 基线：dev@194f7b89464432472707164f3126cc4bff9af0d1；本地 dev 比记录的 origin/dev ahead 32。不要 fetch、push 或改动 dev checkout。
- 生产目标数据库尚未识别。当前检查中 psql、pg_isready 缺失；DATABASE_URL、STAR_DATABASE_URL、TARGET_DATABASE_URL、PGHOST、PGDATABASE 均未设置。Multipass inventory 仅有一个已停止实例。没有连接或变更任何数据库。

## 范围

仅拥有并修改 manifest 中的 6 个文件。新增的 scripts/automation/wld0_target_database_probe.py 应要求操作者显式传入 libpq service 名称，以受控参数执行只读 PostgreSQL 目录查询。脚本不得接受连接 URL、任意 SQL、迁移文件或写入模式；必须使用 READ ONLY transaction、短语句超时、固定查询、有限结果、无 shell 的 subprocess 参数列表。不得打印连接信息、凭据、PSQL 原始环境或完整 stderr。缺少 psql、service 配置、目标数据库身份或迁移 ledger 时明确返回 BLOCKED/UNKNOWN，不猜测已部署版本，也不应用迁移。

探针应收集最小必要的数据库/服务身份、登录角色 superuser/BYPASSRLS/INHERIT 属性、已知 migration ledger 表是否存在、WLD-0 关键对象和 RLS/FORCE RLS catalog 状态。只读 catalog 结果不等于迁移历史一致或业务授权通过；如遇未知 ledger 格式，保留 UNKNOWN 并停止推断。

将入口登记到 docs/automation-design.md 的下一个合适章节及 scripts/automation/registry.md。不要修改现有工程 Run migration runner、迁移、Run API、目标数据库或 WBS 总表；父协调者会在集成时更新 WBS。

## 验收

- 只用 Python 标准库；无新依赖。
- 单元测试覆盖 service-name 校验、psql 缺失、只读参数与固定 SQL、秘密不出现在输出、错误码/超时、未知 ledger/RLS 的 UNKNOWN/blocked 处理。
- 测试可以 mock subprocess 来验证边界，但报告必须明确这不是实际数据库验收。
- 本机当前没有可识别的 target DB；不得伪称目标库、部署、migration replay、grants 或 FORCE RLS 实测通过。
- 为新增 Python 源文件维护准确的 CypherGraph 头；按 CypherGraph Guardian 规则更新 fingerprint 并运行 --changed --stamp。只跑该脚本的聚焦 unittest、Python AST/图头和 diff 检查。
- 阶段报告按仓库要求记录目的、改动矩阵、实测验证、已知缺口、子代理失败接手、守门、审核角色和修订历史。

## 协作边界

你负责实现、聚焦验证和提交此 lane。不要操作其他 worktree，不要 rebase、merge、push、删分支或清理 worktree。提交前确认只改 manifest 文件，提交 commit message 用 Ulysses <ulysses@mavis.local>。完成后向协调者报告 commit SHA、文件、实际验证命令/结果和未关闭门禁。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
