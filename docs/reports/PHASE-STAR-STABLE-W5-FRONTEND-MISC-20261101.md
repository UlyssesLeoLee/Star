# PHASE-STAR-STABLE-W5-FRONTEND-MISC-20261101

**版本**：v0.1
**目的**：修复稳定版 W5 中全局搜索、终端 WebSocket、查询引擎 JS fallback 与 Gantt 拖动的过时断言或行为偏差。
**基线**：`dev@41f84d407e0c245af88ab29938e8dda57c67af94`
**范围**：遵守本 lane manifest；没有修改 shared WBS、lockfile、`main`、`dev` 或其他 lane。

## §0 目的

将 W5 报告的前端失败与当前协议、Rust 查询实现及日历日期语义逐一核对。移除测试对已删除示例任务的依赖；不恢复任何 demo/seed 任务。报告只声明已实际执行并记录结果的验证。

## §1 改动矩阵

| 区域 | 发现 | 决定与改动 | 结果 |
|---|---|---|---|
| Global Search | 测试依赖已删除的 `Implement Worktree 17-state machine` seed 任务 | 抽出 `searchIndexEntries` 内部可测入口；以测试局部 task fixture 验证搜索，不向产品 seed 添加任务 | 搜索算法行为可测；产品 task 数据仍由当前 index 数据源决定 |
| WebSocket | TS decoder 仅做类型断言；Rust serde tagged enum 会拒绝未知 tag | 按 Rust Client/Server enum 的允许 `type` 集合做运行时 tag 检查；补 client 与 server 未知类型测试 | 未知 tag 按协议失败 |
| Query Engine | 断言 `Show ready` 返回 `ready`，但 `ready` 不是 keyword | 对照 `query-engine-wasm/src/lib.rs` 的 `str::contains` 和 JS `includes`，保持大小写敏感的 keyword substring 语义；断言改为 `[]` | JS/Rust 预期一致 |
| Gantt | 两个拖动预期与 UTC `Iso8601` 输入及含端点的 Sprint 日期处理不一致 | 将日期输入归一为 UTC 日历日；Sprint 结束日按含端点传入/回调；Milestone 以同一日作为起止；测试使用标准 MouseEvent 字段 | 60px/日拖动 60px 得 +1 日；120px 得 +2 日 |
| 关系头 | 修改文件缺少或未覆盖当前代码关系的 Cypher header | 为变更的源码与测试补齐关系，刷新 freshness stamp | CypherGraph Guardian 检查通过 |

Gantt 验收刺激与结果：Milestone `clientX 100 → 160`，`PX_PER_DAY=60`，预期 `2026-02-10 → 2026-02-11`，实际回调 `2026-02-11`；Sprint `clientX 200 → 320`，`PX_PER_DAY=60`，预期 `2026-02-01..2026-02-14 → 2026-02-03..2026-02-16`，实际回调一致。

## §2 验证摘要

| 验证 | 命令 | 结果 |
|---|---|---|
| 四个受影响的 Vitest 文件 | `rtk proxy npm test -- --run src/lib/search/globalSearch.test.ts src/lib/terminal/wsProtocol.test.ts src/lib/wasm/query-engine.test.ts src/components/gantt/GanttChart.test.tsx --reporter=dot`（工作目录 `frontend`） | PASS，4 files / 59 tests |
| Patch whitespace | `rtk proxy git diff --check` | PASS，退出码 0 |
| 关系头和 freshness stamp | `rtk proxy py -3 C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | PASS，7 source files checked |

没有运行全量前端 build、Rust/WASM build 或桌面发布构建；本 lane 验证仅覆盖上述受影响测试与关系头门禁。

## §3 已知缺口

1. WebSocket decoder 当前验证 JSON 顶层为对象且 `type` 属于 Rust 协议枚举；已知 `type` 的字段级 shape 仍由后续消费路径处理，未在本 lane 扩展为完整 runtime schema validator。
2. Global Search 的 task 算法使用测试局部 fixture 验证；本 lane 不证明线上/真实任务 provider 当前已接入任务 index。
3. Rust/WASM parity 依据当前 Rust `parse_dsl` / `validate_dsl` 源码核对；本 lane 未构建或运行 wasm32 artifact。
4. Gantt 日期按 UTC 日历日呈现，与 Rust `DateTime<Utc>` 及当前日期型拖动回调一致；本 lane 未验证所有外部调用方及远端 API 的持久化格式。

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
11. UTC 时间戳在 Gantt 进入日历布局前归一化：通过。
12. Sprint end 按含端点模型渲染和回传：通过。
13. 不修改锁文件、shared WBS、`main` 或 `dev`：通过。
14. 不 push、不修改其他 lane：通过。
15. 四个相关测试文件全部通过：通过。
16. `git diff --check` 通过：通过。
17. CypherGraph Guardian `--changed --stamp` 通过：通过。

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
