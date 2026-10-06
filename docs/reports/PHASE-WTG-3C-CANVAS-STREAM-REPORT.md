# PHASE-WTG-3C-CANVAS-STREAM-REPORT

> **Phase**: Worktree Group Phase 3C/3E — Worktree-scoped Canvas Outbox cursor slice
> **Version**: v0.9 · 2026-10-06 · 🟡 Implementation slice; phase closure gates remain open
> **Author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **Source**: Canvas stream candidate `codex/phase3c-dev-integration-20261005`; conflict source `codex/worktree-group-shell-mvp`
> **Design baseline**: requirements v5.80 · basic design v5.77 · Data Design v1.7 · Group DD v4.55 · implementation plan v6.08

## §0 目的

登记 Worktree 级 Canvas Outbox 顺序 cursor 的实现和可执行验证，并区分授权查询、浏览器恢复提示、服务端 durable consumer 与 realtime 的完成状态。Multipass 管理 Linux guest、guest 内运行 K3s、Host/environment 共享控制面的架构已在 dev 设计基线中确认；Star provider 与 Run/workspace bridge 仍未验收，本报告只覆盖 Canvas stream slice。

## §1 改动矩阵 / 任务完成矩阵 / 引用扫矩阵

| 文件/范围 | 改动 | 当前状态 |
|---|---|---|
| `crates/star-api-rest/src/group_api/canvas.rs` | 新增 Worktree 事件 metadata endpoint、非负 offset/有界 limit 校验；Outbox producer 以 tenant/worktree advisory xact lock 串行化 offset 分配；Canvas transaction 设置 5s statement / 1s lock deadline | workspace all-targets 编译通过；当前文件定向 rustfmt 通过；disposable PostgreSQL 双连接验证锁顺序和 1s timeout fail-closed |
| `db/migrations/2026-10-05-worktree-group-canvas-stream-offset.sql` | 给 append-only Outbox 增加 BIGINT identity offset 与 tenant/worktree 唯一索引 | 在 disposable PostgreSQL 18.6 fixture 双次应用通过，既有 Outbox rows 完成 identity backfill；不等于目标数据库已部署 |
| `frontend/src/lib/group/worktreeGroupApi.ts` / `groupProjection.tsx` | 用 Worktree-scoped 十进制 cursor 轮询所有 Canvas event metadata；验证页结构、BIGINT 上限、单调性与空页；只有投影刷新成功才前移 | 前端定向验证通过 |
| `frontend/src/lib/group/__tests__/worktreeGroupApi.test.ts` | 覆盖 64 位 offset 路由、刷新失败重放和畸形空页拒绝 | 25 项定向测试通过 |
| `scripts/automation/wtg_canvas_stream.py` | 可重跑源码契约、Canvas 定向 Rust tests、Group API Vitest、typecheck 与 diff check；必填 JSONL audit log | runner 当前工作树验证；不复现 PostgreSQL fixture，也不做目标 DB 部署 |
| requirements/basic/Data Design/Group DD/implementation plan | 保留旧单 Canvas composite cursor；添加 Worktree ordered cursor、commit ordering、deadline、W/T/M 分类与未完成边界 | 五层版本及引用已同步 |
| `frontend/src/app/api/group/[...path]/route.ts`、`AuthenticatedWorktreeIndex.tsx`、`group-context.ts` | Project Index 接入 same-origin cookie-backed BFF；Bearer 仅从 HttpOnly cookie 在服务端转发，限制 path/method、同源写入、body 和响应头 | host cookie issuer、生产 API base 与目标 DB/RLS 未接通；不宣称生产授权已启用 |
| `group_api.rs` / `group_api/worktrees.rs` | Project Worktree 投影停止序列化主机 checkout path | WTG-022 / AC-WTG-012 已同步需求和设计；内部 checkout path 仍由受信任 Host/Repository 层解析 |
| `codex/worktree-group-shell-mvp` 冲突对账 | 逐项检查并解决 11 个冲突；保留当前签名 Run fence、Canvas stream 与 `group_canvas_registry` Document 版本设计；去掉旧未调用 CLI helper、平行 `canvas_document_state` Master migration、重复 stream offset migration 和孤立 Canvas client；本地 merge commit `f4430beb` 已落在 dev | 旧 Task Card CLI 未接生产 grant/provisioner/PTY/sandbox，继续禁用；Phase 3/Run Apps 未关闭 |

## §2 验证摘要

| 验证 | 结果 | 证据/限制 |
|---|---|---|
| 前序 Canvas candidate 的 Worktree Canvas API focused Vitest | 🟢 25/25 | 仅证明 Canvas cursor API 测试；不替代本次合并树检查 |
| 当前 dev merge tree 的 Group/Worktree focused Vitest | 🟢 38/38 | 5 个既有测试文件通过；修正 Group projection 测试以匹配当前 `/worktrees/{id}/canvas-events` offset-page 契约 |
| 前序 Canvas candidate 的 `pnpm typecheck` | 🟢 通过 | 仅记录 candidate 当时依赖环境的 `tsc --noEmit` 结果 |
| 当前 dev merge tree 的 `pnpm typecheck` | ⚪ 未完成 | 本地 `node_modules` 缺少 `monaco-editor`；manifest 与 lockfile 已声明该依赖。没有通过安装/改锁文件绕过当前环境缺口 |
| `cargo test -p star-api-rest --lib -j 4 canvas_event` | 🟢 3/3 | 覆盖 Worktree cursor 非负/页大小边界、首屏 offset 边界和旧复合游标完整性 |
| `git diff --check` | 🟢 通过 | 只有 Git 的既有换行转换提示 |
| CypherGraph Guardian | 🟢 通过 | `cgg_guard.py --changed --stamp` exit 0；本次 7 个改动源码文件的关系头与 source freshness stamp 通过确定性校验 |
| `cargo check --workspace --all-targets -j 4` | 🟢 通过 | Candidate working tree（含当前 Canvas 改动）约 3m34s；旧 `codex/worktree-group-shell-mvp` commit 约 2m07s；均用 Rust 1.98.1 |
| Post-rebase `cargo check --workspace --all-targets -j 4` | 🟢 通过 | Rust 1.98.1；显式将 toolchain `bin` 加入进程 PATH 后 exit 0；首次未设置 PATH 的尝试仅因 build script 找不到 rustc 失败 |
| cargo test --workspace --release --lib -j 4 | 🟢 通过 | 当前候选工作树；完整 workspace release library suite，exit 0 |
| cargo build --workspace --release -j 4 | 🟢 通过 | 6m03s；有既有 deprecated/dead-code/linker warnings |
| cargo doc --workspace --no-deps --release -j 4 | 🟢 通过 | 11m38s；exit 0，含大量既有 rustdoc broken-link/HTML/Cypher-header warnings |
| `cargo clippy -p star-api-rest --all-targets -j 4 -- -D warnings` | 🔴 未通过 | 编译 `star-context` 时遇到 `actor.rs` 的既有 `unnecessary_map_or` 与 `field_reassign_with_default` lint；未得到本切片 crate 的 strict Clippy 通过证据 |
| cargo bench --workspace --no-run | 🔴 未通过 | -j 4 与 -j 1 均 exit 1，完整输出未包含编译错误诊断；单独 cargo bench -p star-api-rest --no-run -j 1 -vv 通过，7m02s，生成两个 harness；全 workspace bench 门保持未通过 |
| Rust format | 🔴 全 workspace 未通过 / 定向通过 | `cargo fmt --all -- --check` 报告大量未修改文件格式差异；改动的 `canvas.rs` 通过 rustfmt `--check --edition 2021` |
| Canvas 定向 Rust tests | 🟢 3/3 | 覆盖 Worktree cursor 非负/页大小边界、首屏 offset 边界和旧复合游标完整性；workspace release library suite 另已通过 |
| PostgreSQL migration/RLS/并发锁验证 | 🟢 disposable fixture 通过 | PostgreSQL 18.6；base 与 stream migration 双次应用；2 条旧 Outbox row 回填出不同 identity offset；FORCE RLS=true、scope A/B 隔离、未设 scope 返回 0 行；append-only UPDATE 被拒绝；双连接观察到 advisory wait 并验证 offset 顺序；1s lock_timeout 竞争在约 1.29s client wall time 内 fail-closed。runtime role 是手工最小 grant 的 synthetic role，不代表目标 grants；未测 query plan |
| CodeRabbit review | ⚪ 未完成 | 本次 `coderabbit --version` / `coderabbit auth status` 调用 16 秒无输出后中断；没有取得版本、认证或 review 结果。已完成本地冲突与 diff 检查 |
| 当前 dev merge tree 的 `cargo check --workspace --all-targets -j 4 --locked` | 🟢 通过 | Rust 1.98.1；隔离 `CARGO_TARGET_DIR`；5m17s，exit 0；存在仓库既有 deprecation/unused warnings |
| 当前 dev merge tree 的改动 Rust 文件 `rustfmt --check --edition 2021` | 🟢 通过 | `group_api.rs` 与 `group_api/worktrees.rs` 定向检查 exit 0 |
| 当前 dev merge tree 的 `git diff --check` 与 `git diff --cached --check` | 🟢 通过 | 仅有 Git 的 CRLF→LF 提示 |
| 前序 Canvas candidate 的 WTG-3C automation runner | 🟢 通过 | candidate rebase 到当时的本地 dev 后复跑 `python scripts/automation/wtg_canvas_stream.py --audit-log <temp JSONL> --cargo <Rust 1.98.1 cargo.exe>`：源码契约、focused Rust test、Vitest、typecheck、diff 与 registry check 全部通过（0 errors / 197 existing warnings）；runner 不执行 PostgreSQL fixture；该结果不覆盖本次 merge tree 的当前 typecheck 环境 |

前端依赖的 frozen install 因仓库已有 `@react-three/fiber` manifest/lockfile specifier 不一致而拒绝；随后 `pnpm install --ignore-scripts --no-frozen-lockfile --lockfile=false` 成功，仅生成 candidate 的忽略目录 `node_modules`，没有更新 lockfile。

## §3 已知缺口

1. Workspace all-targets compile 与 release library suite 已通过；Canvas 定向 tests 3/3、focused `star-api-rest` bench harness 通过。全仓 fmt 有既有差异，全 workspace bench --no-run 两种并行度都无诊断 exit 1，故相应门未关闭。
2. 仅 disposable PostgreSQL 18.6 fixture 验证了 migration/backfill/RLS/append-only 与锁行为；目标数据库 migration、正式 runtime grants、应用角色授权、query plan 及完整 API→DB end-to-end 尚未验收。
3. Worktree offset 是读取 cursor，不是 consumer acknowledgement/checkpoint；没有 NATS JetStream、SSE/WebSocket 或跨 App realtime。
4. 宿主 JWT provider、目标库部署、Canvas/Jira relation 与 Phase 3 整体验收仍开放。
5. `373259dc` 的 11 个冲突已经按当前 dev 的授权、Run fence、Canvas cursor 与 schema 事实逐文件审查并解决，并以本地 merge commit `f4430beb` 集成。目标提交 `05075980` 在此 merge 前已是 dev 祖先。workspace all-targets check、改动 Rust 文件 rustfmt 和 focused frontend tests 38/38 通过；前端 typecheck 因本地缺少已在 manifest/lockfile 声明的 `monaco-editor` 未完成。没有 push；本地 remote-tracking refs 未刷新。
6. 旧 shell lane 的未调用 `spawn_authorized_cli` 缺少当前签名 Run fence；不带入该执行路径。旧 `canvas_document_state` 平行 Master 表与已采纳的 `group_canvas_registry` Document 版本字段冲突，重复 `stream_offset` migration 与 2026-10-05 migration 重复，均不带入。保留 same-origin BFF / Project Index UI 代码切片与脱敏 Index projection；cookie issuer、Group Shell provider、DB/RLS 和 Task CLI 生产执行闭环仍开放。

## §4 子代理失败接手清单

本阶段未派发子代理；没有待接手的子代理产出或 RPC 结果。

## §5 守门规则

1. 每个请求重新解析 Bearer actor、Project membership、Worktree 与 `canvas:read` scope。
2. offset 使用十进制字符串；服务端接受非负 BIGINT cursor，页大小限定 1–100；事件响应不含 payload/EntityRef。
3. tenant/worktree advisory xact lock 必须早于 Outbox insert 且持有至事务提交；任何新 Outbox writer 必须复用该锁。
4. DB statement/lock deadline 固定 5s/1s；锁超时 fail closed。
5. 浏览器 cursor 仅按 Worktree 保存，投影 refresh 成功前不得持久化或前移；损坏/失败执行 at-least-once 重放。
6. 不把本地 cursor 描述成 durable consumer、NATS 或实时推送；单 Canvas legacy API 保持兼容。
7. 前序 Canvas candidate 的 workspace all-targets compile/release build/doc/release library suite 与 `star-api-rest` focused bench 已通过；本次 dev merge tree 的 workspace all-targets compile、改动 Rust 文件 rustfmt、frontend focused tests 38/38 与 CypherGraph Guardian 7 个源码文件检查通过。当前工作树 frontend typecheck 因缺少已声明的 `monaco-editor` 未完成；full workspace fmt/bench 门未通过，strict Clippy 被 `star-context` 两条既有 lint 阻断。目标 PostgreSQL、正式 grants、consumer/realtime 与生产 provider 未验收前不得标记 Phase 3 已完成。

## §6 签字栏

| 角色 | 审核形式 | 结论 |
|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 审核通过本切片的范围与边界；阶段保持未关闭 |
| SRE Lead | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 保留 PostgreSQL/RLS/运行时验证门 |
| 平台 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Rust workspace compile 通过；数据库与 Host/provider 未验收，不启用生产能力 |
| 评审主持 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按当前证据登记，不扩大完成范围 |
| PM | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 保持 Phase 3 未完成，续做剩余验收 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 Worktree Canvas ordered offset slice、前端测试结果和未完成 Rust/DB/生产验收门 | 按实现更新需求、基本设计、详细设计和 phase plan |
| v0.2 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 candidate 与旧 3C shell lane workspace all-targets 编译、Canvas cursor Rust tests 3/3、当前 Canvas 文件定向格式和前端 typecheck/test 证据；修正 Rust 不可用的过期表述；明确 Docker/PostgreSQL blocker、全包 fmt 既有差异、dev 集成冲突、Task Card CLI 未接线及重复/冲突 schema migration | 找到 Rust 1.98.1 显式工具链并复核旧 WIP branch 与当前 dev 的可合并性 |
| v0.3 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 加入 disposable PostgreSQL 18.6 migration 双次应用、identity backfill、FORCE RLS scope、append-only 负例、双连接 advisory wait/offset 顺序和 1s lock timeout fail-closed 证据；将 WTG-3C [P] runner 登记到 automation-design v2.8 / registry v0.58，将 CypherGraph Guardian 更新为确定性通过，并准确记录 CodeRabbit wrapper 启动失败；同步 requirements v5.75/basic v5.72/Data Design v1.6/Group DD v4.49/plan v5.98；目标 DB/runtime grants/query plan、durable consumer/realtime 与全阶段验收仍开放 | PostgreSQL 18.6 本地工具可用后完成隔离 migration 与竞争验证 |
| v0.4 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新 workspace release library/build/doc 成功、focused star-api-rest bench 成功与 full workspace bench 无诊断 exit 1；记录 workspace fmt drift、确定性 Guardian 5 文件通过并修复 runner 模板零值 hash；纠正完整 release library suite 已运行的状态；同步 implementation plan v6.00；目标 DB grants/query plan、durable consumer/realtime、Full workspace bench 与 Phase 3 closure 仍开放 | 完成 release gate 复核与本次自审发现的报告矛盾修正 |
| v0.5 | 2026-10-05 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 targeted strict Clippy 被 `star-context` 既有 lint 阻断；确认 Multipass/Linux guest/K3s 共享控制面为 dev 设计基线，Star provider/bridge 仍未验收；同步 implementation plan v6.01 | 将 Clippy 实测与最新架构边界写入验证状态 |
| v0.6 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录候选代码 commit `85d792c1` 已 rebase 到本地 `dev` `31b7589c`，但仍待 fresh GitConverge apply；加入 post-rebase workspace check 与 runner 复跑证据；标记旧 shell lane predicted conflict；Multipass VM/K3s 架构保持为已合入设计基线，Star provider/bridge 未验收 | 纠正 rebase 后报告仍称代码未提交的状态差异 |
| v0.7 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Project Index cookie-backed BFF 与 checkout path 脱敏实现；同步 WTG-022 / AC-WTG-012、需求 v5.78、基本设计 v5.75、详细设计 v4.53 和计划 v6.06；逐项解决 `373259dc` 的 11 个冲突并排除不符合当前 Run fence/Canvas schema 的旧执行与重复 migration；登记当前 merge tree 的 focused tests、typecheck 环境缺口和 workspace 检查状态；本地合并与 fresh preview 待完成 | 用户选择先解决冲突，再重新预览并合并到 dev |
| v0.8 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 复核 merge tree workspace all-targets check exit 0（Rust 1.98.1、5m17s）、定向 Rust format、38/38 Group/Worktree focused tests、CypherGraph Guardian 7 源码检查和 diff checks；准确保留缺失 `monaco-editor` 导致当前 frontend typecheck 未完成；同步 implementation plan v6.07，尚未生成本地 merge commit | 完成合并前验证并按当前工作树证据更新状态 |
| v0.9 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 `codex/worktree-group-shell-mvp` 以本地 merge commit `f4430beb` 集成；确认 `05075980` 已在 dev 祖先链而无需重复合并；同步 requirements v5.80/basic v5.77/Group DD v4.55/plan v6.08；保留无 push、remote-tracking refs 未刷新和前端 typecheck 依赖缺口；Multipass/Linux guest/K3s 架构与 Star provider 未验收边界不变 | 用户授权解决旧 branch 冲突后完成本地 merge 并更新跨层文档 |
