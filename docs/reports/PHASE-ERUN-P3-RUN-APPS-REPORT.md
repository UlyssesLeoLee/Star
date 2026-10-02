# PHASE-ERUN-P3-RUN-APPS-REPORT

> 状态：🟡 Run Task Cards 条件式 UI/client 与旧 Tauri mock Task 退役已交付；Run Apps/Task CLI 生产闭环仍未完成
> 日期：2026-10-02 JST
> 分支：`codex/erun-p3-run-apps-20261002`，基线 `origin/dev@9db93e9d8e07a11ad591331cae94b826cf5be7ee`
> 对应实施计划：[`WORKTREE-GROUP-IMPL-PLAN-001.md`](../implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md) §6.72-§6.73

## §0 目的

为已有 canonical Run-scoped Work Item API 接入只读 Run Task Cards，并移除独立旧 Tauri shell 中四条演示 Task 和 browser fallback；选中 Run Worktree 后默认展示 Task Cards 同级 App，严格维持 Run owner、宿主认证、数据库/RLS、执行 Runtime 未就绪时 fail-closed。本文只报告本阶段可验证交付，不把生产 Run App 或 CLI 标为完成。

## §1 改动矩阵

| 范围 | 变更 | 状态 |
|---|---|---|
| Run Workspace | 默认选中 `Task Cards` tab；挂接只读 `RunTaskCardsPanel`，保留 Inbox/Canvas/Workflow/BI/Plugin 同级入口为待迁移状态 | 代码已落地；服务端 capability false 时显示明确阻断，不显示假任务 |
| Run Directory API | 新增 typed `listRunWorkItems`，只访问 `/api/v1/engineering-runs/{id}/work-items`；核对 Project/Repository/Branch/Run 全 tuple、Task Card/WorkItem ID alias、字段、游标及重复 ID | 代码已落地；使用当前宿主 Bearer token、no-store、credentials omit、15 秒 deadline 和请求取消 |
| 内存/并发 | 单页最多 12 条，Task response 解码上限 2 MiB，界面只保留当前页、单次浏览最多 100 页；复用 client 2 个活动请求/16 个排队请求 | 已落地；此为产品保护上限，不声称已做设备 RSS 基准 |
| CLI | 卡内按钮明确禁用 | 等待执行 admission、Run/Profile/Hook/预算复验、实际 Runtime/OS sandbox、取消/恢复、独立验证与结果回写验收 |
| 自动化/依赖 | 增加文件日志、有 900 秒上限的 `erun_task_cards.py`；更新锁文件，Fiber 升至 React 19.3 兼容的 9.8.1 | 已落地；依赖原本无法通过普通 `npm ci`，原因与 lock stale 见 §2 |
| Tauri legacy task | MockDb 中四条旧 WorkItem 与 browser-dev fallback 已删除；无 Run provider 时 fail closed | Desktop hook/card 与 BoardView error-state 定向测试通过；未知服务器行未触碰 |
| Canvas E2E | 前置节点由已删除 Task 改为现存 Worktree node，seed 数量更新为 11 | 已编辑测试，不代表 Playwright 已运行 |
| 文档 | requirements v5.51、basic v5.48、Group DD v4.35、Task DD v1.20、Multica SRS v0.7、实施计划 v5.67、automation §4.40 与 registry v0.40 | 已同步；保留所有生产阻断项，并注明桌面旧任务清理与验证范围 |

## §2 验证摘要

| 命令 | 结果 | 证据与范围 |
|---|---|---|
| `npm ci --no-audit --no-fund` | 通过，exit 0 | 安装 329 个包。Node v22.19.0 对 jsdom/css-color/dom-selector 要求 `^22.22.2` 发出 EBADENGINE 警告；警告未阻断本阶段 typecheck/Vitest。 |
| python scripts/automation/erun_task_cards.py --typecheck --tests | 通过，exit 0 | TypeScript tsc --noEmit 0 错；13 个 Vitest 文件、150/150 tests 通过。 |
| python -m py_compile scripts/automation/erun_task_cards.py | 通过，exit 0 | 自动化脚本语法检查。 |
| git diff --check | 通过，exit 0 | 改动 whitespace 检查。 |
| Tauri focused tests | 通过，exit 0 | hook 与 KanbanCard 共 8/8 tests；BoardView fail-closed 单测 1/1 通过（同文件 7 项未执行）。 |
| Tauri desktop npm run build | 失败，exit 2 | legacy desktop 前端在 App.module.css 声明、DEFAULT_KANBAN_COLUMNS/DEFAULT_SWIMLANES 导出、Tauri 全局类型与未使用参数处有编译错误；这些文件不在本次改动范围。 |
| Tauri broad board test attempt | 失败，exit 1 | KanbanBoard 读取未导出的 DEFAULT_KANBAN_COLUMNS；旧 broad suite 被该问题阻断。修正本次 fixture 键后，hook/card/error focused tests 通过。 |
| Rust cargo/rustfmt | blocked | 当前环境找不到 cargo 与 rustfmt，未运行 star-desktop Rust 编译、格式化或单测。 |
| Canvas E2E | 未运行 | 已把旧 Task 节点前置条件改为现存 Worktree 节点，并按 seed 现状更新数量；未运行 Playwright。 |
| python scripts/automation/registry_check.py | 通过，exit 0 | errors=0，warnings=191；本阶段 erun_task_cards.py 已列入 registry，检查器仍报告全仓 automation scripts 的未登记警告。 |
| CodeRabbit `coderabbit review --agent -t uncommitted` | 未运行成功 | Windows wrapper 转入 WSL 后 `/root/.local/bin/coderabbit` Permission denied；按 skill 的 WSL repo-path 方式重试时，Ubuntu WSL 返回无法解析/访问本工作区挂载。已进行本地逐文件复核；不把 CodeRabbit 标为通过。 |

依赖修复原因：原 `@react-three/fiber@9.7.0` peer range 排除项目 React 19.3，首次普通 `npm ci` 以 `ERESOLVE` 失败。更新到 v9 系列 `^9.8.1` 后普通 `npm ci` 成功。重新生成 lockfile 同时修复其与 `package.json` 不一致的 Vite 条目（manifest 要求 `^6.3.5`，旧锁解析成 Vite 8），并补足已声明的 Monaco/Vite root package 条目；未使用 `--legacy-peer-deps`。

未运行根前端完整 build/E2E、Rust workspace 全门、目标 PostgreSQL migration/RLS、视觉浏览器验收或生产 Runtime 测试。Tauri desktop build 单独运行但失败；Canvas E2E 源文件已更新但未运行。上述结果不可外推至这些门。

## §3 已知缺口

1. `frontend/src/app/layout.tsx` 调用 `<Providers>` 未传入 `RunDirectoryHostSession`；宿主认证 token/generation provider 尚未接入，运行中的 Task list 不会请求。
2. `crates/star-api-rest/src/group_api/engineering_runs.rs` 当前固定返回 `run_owned_apps_available=false`、`execution_admission_available=false`。
3. `db/migrations/2026-10-02-task-run-owner.sql` 存在代码，但未在目标数据库执行；实际应用服务身份、grants 与强制 RLS 尚未验收。
4. 旧 owner-null Task 仍只读/unknown；没有审计 reconciliation 与真实 Run owner 数据验收。
5. Task Card 编辑/创建、Inbox、Canvas、Workflow/LangGraph、BI/Benchmark、Plugin App 尚未在本阶段完成 Run UI/读写闭环。
6. 卡内 CLI 仍禁用；Run writer/Runtime provisioning、OS sandbox、实时 revoke/drain、独立 validation、证据/结果持久回写均未闭环。
7. Node 22.19.0 低于当前若干前端测试依赖声明的最低补丁版本 22.22.2；本次针对性测试在现有 Node 上通过，但部署/CI 应统一 Node patch 基线。
8. 本阶段没有设备矩阵、桌面 RSS/CPU、分页大数据或并发负载基准；2 MiB/12 条/100 页是硬上限，不是经过基准测定的最佳值。

## §4 子代理失败接手清单

| 项目 | 状态 |
|---|---|
| 当前阶段实现 worker | 未派发；本阶段由 root 单独实现，不存在失联或散落产出 |
| 前序只读认证/owner 审阅 | 已作为设计输入；未修改代码、未创建 commit，不需要接手或 cherry-pick |
| dispatch/brief/output artifacts | 本阶段无 worker dispatch，无未回收子代理 worktree/分支 |
| 自动化/验证失败接手 | 初次 TypeScript 两项诊断由 root 修复并复跑；CodeRabbit 因 WSL 执行权限/工作区挂载不可用，保留为审阅限制 |

## §5 守门规则（本阶段适用 15 项）

| # | 守门 |
|---:|---|
| 1 | 不编造历史、commit、部署或批准证据；任何历史叙述需 git 证据。 |
| 2 | author=Ulysses 代签须按授权格式；不得把 Mavis 审核描述为真人人工批准。 |
| 3 | 本地 code slice 与 production-ready 分开报告；缺认证、数据库或 Runtime 证据保持未完成。 |
| 4 | 不覆盖或清理 D:\Star 主工作区及其他 worktree 的未提交更改。 |
| 5 | 不把服务端 capability、JWT 或 owner tuple 降级为客户端自报值。 |
| 6 | Run Task Cards 只消费 Run owner API；禁止回退 Worktree task list、seed 或 mock 历史。 |
| 7 | 单页、响应字节数、请求并发、等待队列、deadline 和页数均有上限。 |
| 8 | 路由变化/卸载取消在途请求，错误态不得保留或展示旧 Run 数据。 |
| 9 | 未通过 admission、sandbox、取消/恢复、独立验证及回写闭环前禁用任务 CLI。 |
| 10 | Runtime/程序未请求授权时不触碰目标数据库或真实历史任务行。 |
| 11 | 测试 fixture 与产品数据隔离，不复用已退役 demo task IDs。 |
| 12 | 修改前端锁文件需核对 manifest；禁止用 `--force`/`--legacy-peer-deps` 隐藏 peer 错误。 |
| 13 | Python runner 子进程输出写文件，不在内存捕获无界日志；运行设 deadline，不自动装包。 |
| 14 | docs 同步 requirements、basic design、两个 detailed design、SRS、plan、automation 与 registry；未实现能力不写成完成。 |
| 15 | 仅报告已实际执行的 TypeScript/Vitest/Python/diff 检查；不推断 DB/RLS、全量 build、E2E、视觉或 Runtime 成功。 |

## §6 签字栏

| 角色 | 签署 | 日期 | 结论 |
|---|---|---|---|
| 架构负责人 | 架构师（Mavis 接手 agent per DEC-008） | 2026-10-02 | 🟢 只读 UI/client 子阶段审核通过；生产门仍开放 |
| SRE Lead | 架构师（Mavis 接手 agent per DEC-008） | 2026-10-02 | 🟢 bounded runner 与失败条件审核通过 |
| 平台工程师 | 架构师（Mavis 接手 agent per DEC-008） | 2026-10-02 | 🟢 Run owner/auth/RLS/capability 阻断记录完整 |
| 评审主持人 | 架构师（Mavis 接手 agent per DEC-008） | 2026-10-02 | 🟢 未把代码切片扩报为生产闭环 |
| 项目负责人（PM） | 架构师（Mavis 接手 agent per DEC-008） | 2026-10-02 | 🟢 可集成到 dev 展示条件式 Run Task Cards 进度 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 初版；记录 Run Task Cards bounded UI/API client、定向验证、依赖兼容修复及 Auth/DB/RLS/CLI 剩余门 | 推进 Engineering Run Task Cards 生产接入阶段 |
| v0.2 | 2026-10-02 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 legacy Tauri Task mock/fallback 退役、Canvas E2E 更新与 Desktop gates：focused 9 tests 通过，full desktop build 失败于独立 legacy compile errors，Rust 工具链不可用；未知服务器 Task 行不变更 | 全仓扫描发现旧 Tauri shell 与 E2E 仍引用演示 Task |
