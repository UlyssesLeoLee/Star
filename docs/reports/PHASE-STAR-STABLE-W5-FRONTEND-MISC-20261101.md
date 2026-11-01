# PHASE-STAR-STABLE-W5-FRONTEND-MISC-20261101

**版本**：v0.2
**目的**：修复稳定版 W5 中全局搜索、终端 WebSocket、查询引擎 JS fallback 与 Gantt UTC 日期处理的过时断言或行为偏差。
**基线**：`dev@41f84d407e0c245af88ab29938e8dda57c67af94`
**范围**：本次增量只修改 MAGOS revision 10 登记的共享 Gantt 日期 helper、GanttChart、Analytics/Projects 两个调用方、GanttChart 测试及 lane brief/report/dispatch/audit；没有修改 shared WBS、lockfile、`main`、`dev` 或其他 lane。

## §0 目的

将 W5 报告的前端失败与当前协议、Rust 查询实现及日历日期语义逐一核对。移除测试对已删除示例任务的依赖；不恢复任何 demo/seed 任务。后续审查确认 Analytics 与 Projects 调用方仍会先将 UTC 时间戳按本地时区格式化，故把 UTC calendar-day helper 提取为共享模块并统一三个 Gantt 入口。报告只声明已实际执行并记录结果的验证。

本次增量的实现文件：`frontend/src/lib/gantt/dateUtils.ts`、`frontend/src/components/gantt/GanttChart.tsx`、`frontend/src/app/(app)/analytics/page.tsx`、`frontend/src/app/(app)/projects/ProjectsClient.tsx`；回归用例位于 `frontend/src/components/gantt/GanttChart.test.tsx`。其他提交内的 W5 内容未改动。

## §1 改动矩阵

| 区域 | 发现 | 决定与改动 | 结果 |
|---|---|---|---|
| Global Search | 测试依赖已删除的 `Implement Worktree 17-state machine` seed 任务 | 抽出 `searchIndexEntries` 内部可测入口；以测试局部 task fixture 验证搜索，不向产品 seed 添加任务 | 搜索算法行为可测；产品 task 数据仍由当前 index 数据源决定 |
| WebSocket | TS decoder 仅做类型断言；Rust serde tagged enum 会拒绝未知 tag | 按 Rust Client/Server enum 的允许 `type` 集合做运行时 tag 检查；补 client 与 server 未知类型测试 | 未知 tag 按协议失败 |
| Query Engine | 断言 `Show ready` 返回 `ready`，但 `ready` 不是 keyword | 对照 `query-engine-wasm/src/lib.rs` 的 `str::contains` 和 JS `includes`，保持大小写敏感的 keyword substring 语义；断言改为 `[]` | JS/Rust 预期一致 |
| Gantt 与日期范围调用方 | GanttChart 最初已归一 UTC 日期，但 Analytics 与 Projects 仍直接对 UTC timestamp 执行本地 `parseISO`/`format` | 新增 `frontend/src/lib/gantt/dateUtils.ts`，提供 UTC 日期归一、加日和日差；GanttChart、Analytics 与 Projects 统一使用；Gantt 拖动继续按 Sprint 含端点语义 | 日期范围不随浏览器时区向前偏移；加日/日差不受 DST 影响 |
| 关系头 | 修改文件缺少或未覆盖当前代码关系的 Cypher header | 为变更的源码与测试补齐关系，刷新 freshness stamp | CypherGraph Guardian 检查通过 |

UTC 日期范围回归刺激：`2026-02-10T00:00:00Z`；验证环境时区为 `America/Tijuana`（UTC−08:00），原生本地解析结果为 `2026-02-09 16:00`，直接格式化会得到错误的 `2026-02-09`；预期 UTC 日历日是 `2026-02-10`，helper 实际得到 `2026-02-10`。边界偏移刺激 `2026-02-10T23:30:00-02:00` 预期并实际归一为 `2026-02-11`。DST 日历算法刺激 `2026-03-08 + 1` UTC 日预期并实际为 `2026-03-09`，UTC 日差为 1。

Gantt 拖动刺激与结果：Milestone `clientX 100 → 160`，`PX_PER_DAY=60`，预期 `2026-02-10 → 2026-02-11`，实际回调 `2026-02-11`；Sprint `clientX 200 → 320`，`PX_PER_DAY=60`，预期 `2026-02-01..2026-02-14 → 2026-02-03..2026-02-16`，实际回调一致。

## §2 验证摘要

| 验证 | 命令 | 结果 |
|---|---|---|
| Gantt focused test | `rtk proxy npm test -- --run src/components/gantt/GanttChart.test.tsx --reporter=dot`（工作目录 `frontend`） | PASS，1 file / 7 tests |
| 四个受影响的 Vitest 文件 | `rtk proxy npm test -- --run src/lib/search/globalSearch.test.ts src/lib/terminal/wsProtocol.test.ts src/lib/wasm/query-engine.test.ts src/components/gantt/GanttChart.test.tsx --reporter=dot`（工作目录 `frontend`） | PASS，4 files / 60 tests |
| 前端 TypeScript | `rtk proxy npm run typecheck`（工作目录 `frontend`） | PASS，`tsc --noEmit` |
| Patch whitespace | `rtk proxy git diff --check` | PASS，退出码 0 |
| 关系头和 freshness stamp | `rtk proxy py -3 C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | PASS，5 source files checked |

没有运行全量前端 build、Rust/WASM build 或桌面发布构建；没有单独渲染 Analytics/Projects 页面，日期范围规则通过共享 helper 测试、调用点审阅和前端 typecheck 验证。

## §3 已知缺口

1. WebSocket decoder 当前验证 JSON 顶层为对象且 `type` 属于 Rust 协议枚举；已知 `type` 的字段级 shape 仍由后续消费路径处理，未在本 lane 扩展为完整 runtime schema validator。
2. Global Search 的 task 算法使用测试局部 fixture 验证；本 lane 不证明线上/真实任务 provider 当前已接入任务 index。
3. Rust/WASM parity 依据当前 Rust `parse_dsl` / `validate_dsl` 源码核对；本 lane 未构建或运行 wasm32 artifact。
4. Analytics 与 Projects 的范围选择现在使用 UTC helper；本 lane 没有页面级 UI 测试或端到端 API 持久化测试。
5. Gantt 日期按 UTC 日历日呈现，与 Rust `DateTime<Utc>` 及当前日期型拖动回调一致；本 lane 未验证所有外部调用方及远端 API 的持久化格式。

## §4 子代理失败接手清单

本 lane 没有再派发子代理，因此没有子代理 RPC 成功状态待核验，也没有子代理未落盘产出。协调者另行审阅、验证并决定是否集成本 lane。

## §5 守门规则

1. 仅修改 brief 与登记的四组前端源码/测试及 lane 报告、dispatch/audit 记录：通过。
2. 不恢复已删除 demo 或 mock task seed：通过。
3. task 搜索测试只在测试作用域构造 fixture：通过。
4. WebSocket client type 集合按 Rust enum 核对：通过。
5. WebSocket server type 集合按 Rust enum 核对：通过。
6. 未知 client 与 server tag 均有失败测试：通过。
7. 查询关键词按当前 Rust 字符串匹配语义核对：通过。
8. 只修正不符合关键词集合的旧断言：通过。
9. Gantt MouseEvent 不注入伪造函数属性：通过。
10. Gantt 拖动像素与 `PX_PER_DAY` 期望明确记录：通过。
11. GanttChart 与 Analytics/Projects 调用方共用 UTC 日历日期 helper：通过。
12. Sprint end 按含端点模型渲染和回传：通过。
13. 不修改锁文件、shared WBS、`main` 或 `dev`：通过。
14. 不 push、不修改其他 lane：通过。
15. 四个相关测试文件全部通过：通过。
16. `git diff --check` 通过：通过。
17. CypherGraph Guardian `--changed --stamp` 通过：通过。
18. UTC 加日/日差不依赖浏览器 DST 与本地日期格式化：focused tests 通过。

## §6 签字栏

| 角色 | 审核/签字 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）— 审核通过 |
| SRE Lead | Mavis 接手审核 — focused gates 通过；全量 release build 未覆盖 |
| 平台 | Mavis 接手审核 — lane manifest 与隔离约束通过 |
| 评审主持 | Mavis 接手审核 — 当前 Rust/WebSocket/Gantt 契约核对通过 |
| PM | Mavis 接手审核 — 本阶段目标完成，已知缺口如 §3 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W5 四组问题的契约判断、实现、focused 验证与缺口 | STAR-STABLE-W5-FRONTEND-MISC-20261101 |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 MAGOS manifest revision 10 提取 UTC 日期 helper，并修复 GanttChart、Analytics、Projects 的 UTC 日期范围偏移；补跨时区/DST 用例和增量验证 | W5 调用方审查发现 `parseISO(UTC timestamp)` 经本地时区格式化会偏移一天 |
