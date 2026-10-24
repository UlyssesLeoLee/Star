# PHASE-WLD5-SESSION-GENERATION-TEST-20261023 — 阶段报告

> 状态：实现与聚焦验证完成，待协调者集成审查。范围仅是注入式 Group session generation 的前端回归测试；不代表生产登录或宿主身份提供器已接通。

## §0 目的

为 WBS-WLD-NEXT-20261023 的 WLD5 会话代际隔离补充确定性回归覆盖。验证旧 generation 的投影响应和迟到 401 在新 generation 建立后不会覆盖新投影或使新会话失效。实施边界来自 `docs/briefs/wld5-stale-session-401-regression-20261023.md`；production source 不在本 lane 的授权 manifest 中。

## §1 改动矩阵

| 文件 | 改动 | 结果 |
|---|---|---|
| `frontend/src/lib/group/__tests__/groupProjection.test.tsx` | 增加 deferred fetch 时序：旧 session 先有挂起投影，再有挂起 401；替换 `sessionKey` 与注入 token，待新投影 live 后释放旧成功响应与旧 401。更新该测试文件的 Cypher 节点、调用边和 freshness stamp。 | 聚焦测试 6/6 通过。 |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | 增加 UT-FE-SESSION-01、正确的聚焦命令及注入认证的证据边界；升至 v1.2。 | 与实际测试行为及验证范围一致。 |
| `docs/reports/PHASE-WLD5-SESSION-GENERATION-TEST-20261023.md` | 新建本报告，按本阶段七段结构记录结果与缺口。 | 本文件。 |

未修改 Group production source、授权策略、宿主登录、token issuer 或 WBS。

## §2 验证摘要

| 命令 | 结果 |
|---|---|
| `rtk pnpm install --frozen-lockfile --offline`（`frontend/`） | 成功；使用本机 pnpm store 安装锁定依赖。 |
| `rtk pnpm run test src/lib/group/__tests__/groupProjection.test.tsx`（`frontend/`） | **通过**：1 个测试文件、6 个测试通过，Vitest 5.0.1。此处不带分隔符 `--`，以确保 Vitest 将路径作为文件筛选器。 |
| `rtk git diff --check` | 通过，无空白错误。 |
| `rtk python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | 通过：1 个源文件检查；`.agents` 内 canonical validator 不存在，按 Guardian skill 使用安装目录脚本。 |
| `coderabbit review --agent -t uncommitted` | 未完成：RTK 与 Windows CLI wrapper 均转向 WSL `/root/.local/bin/coderabbit`，该路径返回 `Permission denied`。CodeRabbit 不视为通过；本地按实际 diff 与聚焦测试完成人工复查。 |

brief 指定的字面命令 `rtk pnpm run test -- src/lib/group/__tests__/groupProjection.test.tsx` 在当前 pnpm/Vitest 包装下没有聚焦到目标文件，启动了全量 suite；输出出现 KanbanBoard、GanttChart、RunTaskCardsPanel、RunDirectory session、WebSocket protocol 和 global search 的失败项，且该调用未返回最终汇总。该结果不作为本 lane 的聚焦门禁，也不据此判断这些失败的基线或归属。之后使用上一行无 `--` 的文件筛选命令，单独得到 6/6 通过。

## §3 已知缺口

- 测试使用 `WorktreeGroupApiProvider` 的注入 token、sessionKey 和确定性 fetcher；它只证明客户端 generation 的迟到响应隔离，不证明生产登录、cookie/BFF、token issuer、宿主 session owner 或真实身份授权存在。
- host session owner 仍是外部开放门；需要由宿主 owner 提供真实会话回调与 generation 生命周期后另行集成验证。
- 全量 suite 误启动的失败项没有在本 lane 重跑或归因；只保留聚焦测试结果为本阶段验收证据。

## §4 子代理失败接手清单

本 lane 未派发子代理。无子代理失败、产物接手或未纳入 manifest 的修改。聚焦测试结果和全量 suite 误启动情况均已如实记录，交由协调者决定后续全局门禁动作。

## §5 守门核对

- [x] 修改路径严格限制在 brief 列出的三个 manifest 文件。
- [x] 延迟响应使用 deferred Promise；无计时 sleep、真实网络或非确定性并发。
- [x] 断言新 generation 保持 live、新投影不被旧投影覆盖、旧 401 不影响新 generation。
- [x] 不修改生产认证/授权实现，不将注入 token 测试描述为生产登录证据。
- [x] CypherGraph Guardian 对新增测试符号、调用关系和 freshness stamp 检查通过。
- [x] 聚焦测试 6/6；`git diff --check` 通过。
- [ ] rebase/merge 到 dev 及 WBS 状态更新：由协调者完成。

## §6 签字栏与修订历史

| 角色 | 状态 |
|---|---|
| 架构 | Mavis 接手审核；等待协调者集成审查，不表示生产宿主认证已验收。 |
| SRE Lead | 本前端客户端回归切片不执行 SRE/生产服务验收；未签。 |
| 平台 | 宿主 session owner 仍为开放依赖；未签生产平台接通。 |
| 评审主持 | 等待协调者对提交与合并进行审查。 |
| PM | 等待协调者更新 WBS 状态。 |

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v1.0 | 2026-10-23 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增会话 generation 迟到 401 / 旧投影隔离回归测试阶段报告；准确记录聚焦通过和 host session owner 缺口。 |
