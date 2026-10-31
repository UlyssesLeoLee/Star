# PHASE — Stable Release W2 Agent View Core

## §0 目的

移除 Web `/agent-view` 发布路径中的旧游戏循环和专用模块，同时保留 Star 的 Agent、Worktree、任务关系画布、关系视图与设置入口。本阶段只处理登记的前端源文件和报告，不删除 Rust/game 源码，也不声称 Agent CLI 或模型执行已可用。

## §1 改动矩阵

| 文件范围 | 改动 | 状态 |
|---|---|---|
| `frontend/src/app/(app)/agent-view/page.tsx` | 删除 XP/金币/生命值、奖励、perk、死亡/复活、模拟成本 Step、相关弹窗与游戏控件；保留 Canvas、筛选、Board 链接、关系视图和设置入口 | 完成 |
| `frontend/src/components/agent-view/AgentCanvasView.tsx` | 删除游戏等级/角色/敌人/装饰和主题依赖；节点改为显示 Agent 类型、ID、当前步骤与状态；保留 Worktree、任务、关系线、注释和画布操作；移除持续脉冲动画及游戏化标签 | 完成 |
| `frontend/src/lib/store.ts` | 删除游戏状态、地图和游戏动作；保留并修复 Agent Settings state/actions | 完成 |
| `frontend/src/components/agent-game/`、`frontend/src/lib/agent-game/` | 移除仅供 Web Agent View 使用的游戏组件、逻辑和测试 | 完成 |
| `frontend/src/components/agent/AgentSettingsTab.tsx`、`frontend/src/lib/agent/settings.ts`、`settings.test.ts` | 设置代码迁至中性 Agent 路径；默认凭据为空，UI 明确说明本地草稿不执行 Agent，安全凭据存储尚未接入 | 完成 |
| Agent View 测试与 Cypher headers | 加入运行上下文断言，修复受影响关系头并刷新 digest | 完成 |
| 本 brief、阶段报告及 dispatcher JSONL | 补记验证证据和剩余边界 | 完成 |

## §2 验证摘要

- `npm ci --no-audit --no-fund`：安装 329 个包。Node 为 `v22.19.0`；npm 对 `jsdom`、`@asamuzakjp/css-color` 与 `@asamuzakjp/dom-selector` 报 engine 警告，要求 Node `^22.22.2` 或更新版本。依赖安装成功；此警告保留为环境限制。
- `npm test -- src/components/agent-view/AgentCanvasView.test.tsx src/lib/agent/settings.test.ts`：2 个测试文件、27 项测试通过。
- `npm run typecheck`：`tsc --noEmit` 通过。
- `npm run build`：Next.js 16.3.5/Turbopack 生产构建通过；52/52 静态页面生成，`/agent-view` 列入生产路由。Next 构建本身跳过类型校验，因此类型依据单独的 typecheck 命令。
- 读取 Agent View 的 client reference manifest 并扫描其 12 个发布 JS chunk：没有 `agent-game`、`GameHUD`、`PerkPicker`、`DeathModal`、`useAgentGame`、`agentGameStates`、Roguelike map/claim 等旧游戏实现标识；仍可找到 `Agent workspace`、`OVERVIEW // CANVAS`、`agent-canvas-svg` 和 `agentSettings` 标记。
- `cgg_guard.py --root . --changed --stamp`：PASS，7 个变更源文件。首次扫描时 Next 生成的 `frontend/next-env.d.ts` 被列为缺失 Cypher header；按生成文件规则恢复了本次构建对该文件的改动，再运行后通过。`frontend/public/mockServiceWorker.js` 的安装期状态变化也已恢复；两者均未进入提交范围。
- `git diff --check`：通过。
- 独立 CodeRabbit review 未能运行：CLI 调用返回 `/bin/bash: line 1: /root/.local/bin/coderabbit: Permission denied`。因此没有独立 CodeRabbit 结论；本报告不把该结果描述为 review 通过。

## §3 已知缺口

生产数据证据边界：Agent View 当前从 useStore(s => s.agentSessions) 读取会话，frontend/src/lib/store.ts 将其初始化为 seed.agentSessions，受跟踪的 frontend/src/lib/seed.ts 定义演示会话。移除旧游戏机制及生产 bundle 扫描不代表实时 Agent/工作项数据已接入。具备 Run/Worktree 授权范围的实时 provider，或 provider/数据缺失时的真实空状态，仍是开放工作；seed、UI 和 bundle 证据均不能作为生产数据或 Agent 执行能力的证明。

1. Agent Settings 仍是本地配置草稿。安全凭据存储、模型调用、Agent runtime、CLI 执行、取消/恢复和独立验证不属于本阶段，页面明确提示不会启动 Agent。
2. 旧版本浏览器 `star-store:v1` 的本地存储可能仍含有旧 Agent Settings 字段。此 lane 让新设置默认凭据为空且禁用编辑，但未迁移或清除既有本地值；清理必须单独评估兼容性与数据迁移策略，不能宣称旧客户端存储已完成安全清理。
3. 生产构建与 route bundle scan 只证明当前 Web Agent View 可构建且不携带被移除的游戏模块；没有运行浏览器 E2E、远端部署、生产 smoke、Canvas 大规模数据性能基准或整个 Web 应用测试套件。
4. 当前机器 Node `22.19.0` 低于部分锁定依赖声明的 `22.22.2` 最低版本；本次测试和构建虽然通过，仍需在 CI 对齐的 Node 版本复验。
5. 本次只移除 manifest 中 Agent View 专属的前端游戏目录；未删除 `public/` 中任何不在本 lane 清单内的资源，也未删除或更改 Rust/game 源码。
6. CodeRabbit 因运行器权限错误没有完成；需在可执行该 CLI 的环境作独立审查。

## §4 子代理失败接手清单

- 本 lane 未派生子代理；无失败接手项。

## §5 守门规则

1. 只修改登记的 Agent View 路由、Canvas、共享 store、Agent Settings、对应测试、Cypher header 与 lane brief/report/dispatcher 文件。
2. 不修改 package manifest、lockfile、CI、Docker/Helm、WBS、其他 worktree 或 main。
3. 删除范围只包括 Web Agent View 的游戏状态、游戏专用 UI、逻辑、主题/装饰组件与对应测试。
4. 保留 Agent、Worktree、任务卡关系节点和画布连线。
5. 保留用户注释、自由连线、选择、缩放、画布编辑与关系视图。
6. Agent 节点显示已加载的 Agent 类型、ID、当前步骤及状态。
7. 不新增执行行为；页面明确说明它不会启动 Agent。
8. 不把种子数据、空状态或本地设置草稿描述为生产 Agent 执行能力。
9. Agent Settings 凭据默认为空且 UI 编辑控件禁用；旧客户端存储清理与安全凭据存储须作为后续门禁处理。
10. 不删除或改写 Rust/game 源码。
11. 不改全局主题、全局公共资源或其他 Agent 页面。
12. 旧 Agent Game 的 store 字段及动作不得继续进入该路由或共享 store。
13. 变更源文件的 Cypher graph header 必须和实际 import/call 关系一致并刷新 digest。
14. focused tests、frontend typecheck、生产 build 与 route bundle scan 分别记录实测结果；其中一项不得替代另一项。
15. Bundle 扫描只覆盖 Agent View manifest 引用的 client chunks，不推断整站资源均已移除。
16. 保留 Node engine 警告和未完成的独立 CodeRabbit review 作为验证限制。
17. 不合并、不 push、不打 tag、不发布、不部署，也不清理任何其他分支或 worktree。

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | Agent View 保留上下文/关系边界；执行能力明确留待后续阶段 |
| SRE Lead | 本地生产构建通过；线上部署、runtime 与大图性能未验证 |
| 平台 | 12 个 Agent View route chunk 的专项扫描完成；整站及镜像范围不在本 lane |
| 评审主持 | 变更按登记路径人工复核；CodeRabbit 运行器不可用，独立审查未完成 |
| PM | 本阶段切片完成；稳定版总体门禁仍未完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 移除 Agent View 旧游戏循环和专用前端模块；保留运行上下文、关系画布与非执行设置草稿；记录测试、构建、bundle 扫描及剩余限制。 |
| v0.2 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 只读规格复核发现 route/store/seed 链路；明确 Agent/工作项 seed 数据不是生产证据，实时授权 provider 或真实空状态仍开放。 |
