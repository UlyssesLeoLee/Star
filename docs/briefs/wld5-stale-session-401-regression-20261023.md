# Brief: wld5-stale-session-401-regression-20261023

**Agent**: worker
**Phase**: WBS-WLD-NEXT-20261023
**Created**: 2026-10-23 23:07:19

---

只编辑 manifest 内的前端测试、测试设计文档和新增阶段报告；除非协调者另给 manifest，不修改 production source。

目标：用确定性回归测试证明：旧 Group session 发起的请求在 sessionKey/generation 替换后才返回 401 时，旧 invalidation callback 不会使新 session 失效或让旧 projection 覆盖新数据。该工作只证明现有注入边界的迟到响应隔离，不代表 Star 已有生产登录/token issuer/host session owner。

要求：
1. 阅读现有 WorktreeGroupApiProvider、useWorktreeGroupProjection 及测试；使用受控 deferred fetch/promise 构造旧 session 请求延迟、切到新 session、再返回旧 401 的时序。
2. 断言新 session API 可继续加载/保持 live，旧请求结果不能回写，也不影响新 generation；避免 sleep、并发不确定性和真实网络。
3. 若现有代码本来就通过，只添加最小回归测试；不要重构相邻逻辑。若测试证明现有行为失败，保留失败测试和完整证据后停止，不修改 production behavior，也不扩 manifest，交协调者评估。
4. 更新 docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md 中与认证会话 generation/stale response 对应的用例，明确这是注入式测试且不证明 production provider 存在。
5. 新建 docs/reports/PHASE-WLD5-SESSION-GENERATION-TEST-20261023.md，按 PHASE 报告 7 段模板记录目标、改动矩阵、实际测试命令/结果、已知缺口、子代理清单、守门核对、签字栏/修订历史；明确 host session owner 仍是外部开放门。
6. 源文件已有 @cypher header。若改动源文件，依据实际 imports/calls/TESTS 关系更新 header，刷新 source_sha256，并运行 cgg_guard.py --changed --stamp。若 canonical .agents validator 不存在，使用安装 skill 路径的 validator 并在报告说明。
7. 从 frontend 运行聚焦测试：pnpm run test -- src/lib/group/__tests__/groupProjection.test.tsx。不跑全量 suite。另跑 git diff --check。
8. 禁止改动认证/授权策略、发明 session issuer、读取浏览器凭据、改 Hook 生产开关，或把测试 fixture 描述成 production auth。

提交只限 manifest 文件；不改 WBS 文件（协调者集成后更新 WBS）。Commit message 引用 docs/briefs/wld5-stale-session-401-regression-20261023.md。提交后回报 base/head、改动文件、准确测试结果和剩余缺口.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
