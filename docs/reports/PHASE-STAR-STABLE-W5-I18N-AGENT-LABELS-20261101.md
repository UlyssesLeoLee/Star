# PHASE-STAR-STABLE-W5-I18N-AGENT-LABELS-20261101

## §0 目的

修复 W5 中语言切换与 Agent Session 类型标签的前端测试失败。本 lane 基于 `dev@41f84d407e0c245af88ab29938e8dda57c67af94`，只调整过期测试期望；不改字典、组件行为或共享 WBS。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `frontend/src/lib/i18n/__tests__/I18nProvider.test.tsx` | 将错误的 `tactical` 测试标识改成 `workspace-switcher`，断言与 `zh-CN`、`en`、`ja` 当前字典一致。 | 已修改 |
| `frontend/src/components/agent/SessionCard.test.tsx` | 将 `claude-sonnet` 原始 provider ID 期望改为组件实际呈现的 `Claude Sonnet`。 | 已修改 |
| `frontend/src/lib/i18n/I18nProvider.tsx` | 保持实现不变；正式 `workspaceSwitcher` 值分别为“切换工作区”、“Switch workspace”和“ワークスペース切替”。 | 已审阅，未修改 |
| `frontend/src/components/agent/SessionCard.tsx` | 保持实现不变；组件调用 `agentKindDisplayLabel` 映射内部 ID。 | 已审阅，未修改 |
| `docs/briefs/STAR-STABLE-W5-I18N-AGENT-LABELS-20261101.md` | 任务范围和验收约束。 | dispatcher 已落档 |
| 本报告 | 记录当前产品文案依据、验证结果与剩余发行缺口。 | 已新增 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W5-I18N-AGENT-LABELS-20261101.jsonl` | dispatcher brief 与验证事件。 | 已新增 |
| `docs/reports/STAR-STABLE-W5-I18N-AGENT-LABELS-20261101.log` | dispatcher brief 与本 lane 的验证审计记录。 | 已更新 |

**判定依据：** `AppHeader` 将 `t.appHeader.workspaceSwitcher` 用作工作区切换按钮的 accessible label；中、英、日字典都描述工作区切换。`SessionCard` 将 `session.agent_kind` 交给 `agentKindDisplayLabel`，该 helper 将 `claude-sonnet` 映射为 `Claude Sonnet`。因此旧的 Tactical Jump 文案和原始 ID 断言过期，产品实现无需改动。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy npm test -- --run src/lib/i18n/__tests__/I18nProvider.test.tsx src/components/agent/SessionCard.test.tsx`（在 `frontend/`；临时 junction 使用 dev 已有依赖，完成后移除） | PASS；2 个测试文件、14 个测试通过；Vitest 5.0.1。 |
| `rtk proxy python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | PASS；2 个修改的测试源文件已补充其直接测试目标关系，检查通过。 |
| `rtk proxy git diff --check` | PASS；无空白错误。 |
| 全量 Web frontend suite / Node 24 hosted run | 未运行；本 lane 只负责两个聚焦测试文件，全量 W5 状态以协调者集成复验为准。 |

首次测试尝试因临时依赖 junction 创建在错误的嵌套路径而未找到 Vitest；经核对、移除该精确 junction 后，在正确的 lane `frontend/node_modules` 路径临时链接到 dev 依赖目录，聚焦测试随后通过。没有安装依赖、修改 lockfile 或保留 junction。

## §3 已知缺口

1. 结果只覆盖 I18nProvider 与 SessionCard 两个测试文件，不代表全量前端测试通过。
2. 本地 Node/Vitest 聚焦测试不能替代 Node 24 hosted workflow、桌面嵌入前端或稳定发行构建验证。
3. `SessionCard` 使用共享 helper；helper 当前支持的其它 provider/model label 未在本 lane 扩展测试。
4. W5 其他失败组与稳定发行 W0–W9 门禁仍需分别完成和复核；本 lane 不关闭 W5 或稳定版发行门。

## §4 子代理失败接手清单

- 本 lane 未派生子代理；没有子代理失败、超时或待接手结果。
- 本 lane 的实现与验证结果需要协调者独立审阅，再标记为 ready 并集成。

## §5 守门规则

1. 只修改已登记的 i18n/SessionCard 测试与本 lane 的 brief、报告、dispatcher JSONL 和 audit log。
2. 以当前 locale 字典与 `AppHeader` 的实际使用确定工作区切换文案，不恢复过时标签。
3. 以 `agentKindDisplayLabel` 的实际映射确定 `SessionCard` 显示值，不把内部 ID 当成用户可见标签。
4. 不更改 locale 字典、Provider、SessionCard 组件或 provider/model 映射实现。
5. 两个聚焦测试文件必须通过；本地依赖只通过临时 junction 复用，使用后精确移除。
6. `git diff --check` 与 CypherGraph Guardian 必须通过。
7. 不修改共享 WBS、lockfiles、`main` 或 `dev`。
8. 不推送、不创建稳定 tag/release，不构建、发布或部署制品。
9. 未执行的全量测试、Node 24 hosted run 与发行验证必须标记为未运行。
10. 本 lane 的聚焦测试成功不表示 W5、W0–W9 或稳定版门禁关闭。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | 提交协调者复核：变更仅校准断言，没有改变语言或标签契约。 |
| SRE Lead | 提交协调者复核：聚焦本地检查通过，hosted 与发行运行仍未验证。 |
| 平台 | 提交协调者复核：CypherGraph Guardian 与 diff 检查通过。 |
| 评审主持 | 提交协调者复核：当前 lane 范围未超出 manifest。 |
| PM | 提交协调者复核：本 lane 不代表稳定版发布批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对照正式 locale 文案与 Agent 类型显示 helper，更新两处过期测试期望并记录聚焦验证边界。 | W5 I18n/Agent 标签失败组修复 |
