# PHASE-STAR-STABLE-W5-VITEST-DISCOVERY-20261101

## §0 目的

将 Web 前端的 Vitest 发现范围限定为真正的 Vitest 测试，同时保留 `frontend/e2e/redirects.spec.ts` 这一份配置级 Vitest 测试。Playwright 浏览器用例仍由 `playwright.config.ts` 和 `test:e2e` 脚本负责。本 lane 基于 `dev@41f84d407e0c245af88ab29938e8dda57c67af94`，只提交登记的 Vitest 配置及本 lane 记录；不改变测试断言、跳过测试或缩小 `src` 测试范围。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `frontend/vitest.config.ts` | 保留 `src/**/*.{test,spec}.{ts,tsx}`；将原先整个 `e2e` 测试树的 include 改为仅包含 `e2e/redirects.spec.ts`；移除已变冗余的逐文件 E2E 排除项。 | 已修改 |
| `docs/briefs/STAR-STABLE-W5-VITEST-DISCOVERY-20261101.md` | 记录 lane 的任务、范围和验收边界。 | 已由 dispatcher 落档 |
| `docs/reports/PHASE-STAR-STABLE-W5-VITEST-DISCOVERY-20261101.md` | 记录实施、验证、已知缺口和修订历史。 | 已完成 |
| `docs/reports/WBS-DISPATCH-STAR-STABLE-W5-VITEST-DISCOVERY-20261101.jsonl` | 记录 brief 与验证事件。 | 已完成 |
| `docs/reports/STAR-STABLE-W5-VITEST-DISCOVERY-20261101.log` | 记录 dispatcher brief 与本 lane 验证结果。 | 已完成 |

静态检查确认 `redirects.spec.ts` 是 `frontend/e2e` 下唯一导入 Vitest 的测试；`frontend/playwright.config.ts` 将其排除在 Playwright 之外。其余 e2e 文件仍通过 Playwright 运行，不再进入 Vitest 的匹配范围。`src` 中 `.test` 与 `.spec` 两类测试保持原发现模式。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy npm --prefix 'D:\Star\frontend' exec -- vitest list --filesOnly --root '<lane>\frontend' --config '<lane>\frontend\vitest.config.ts'` | PASS；退出码 0，列出 111 个 Vitest 文件，其中唯一 e2e 路径为 `e2e/redirects.spec.ts`；Playwright 文件未进入发现列表。命令使用已有主工作树 Node 依赖，但配置和测试根目录指向本 lane。 |
| `rtk proxy rg -n 'playwright|vitest' frontend/e2e` | PASS；`redirects.spec.ts` 导入 Vitest；其余带测试框架导入的浏览器用例导入 Playwright；Playwright 配置明确忽略 redirect 配置测试。 |
| focused `vitest run e2e/redirects.spec.ts` | BLOCKED；runner 在启动 worker 前发现主工作树 node_modules 中缺少 `@csstools/css-parser-algorithms`；0 个测试运行。 |
| `npm test` | 未运行；当前本机依赖目录不完整（宽范围 collection 同时报告缺失 React runtime），无法提供可信的执行结果。 |
| `rtk proxy git diff --check` | PASS；无空白错误。 |
| Rust / Desktop / hosted Node 24 发布门 | 未运行；不属于本 lane 的验证范围。 |

首次不带 `--root` 的 collection 尝试解析到了 lane 根目录而非 `frontend`；更正根目录后，文件列表验证通过。focused runner 失败是现有共享 node_modules 的缺失依赖，不是测试断言失败。

## §3 已知缺口

1. 单文件 redirect Vitest 用例尚未真正执行；需在完整、锁定的 Web 依赖安装环境重跑。
2. 本机全量 `npm test` 未验证；另外 4 个 W5 lane 负责基线中的断言失败，且 W5 最终门还需要完整 Node 24 hosted run。
3. 静态导入搜索不等价于完整 Playwright 浏览器运行；本 lane 不证明浏览器套件通过。
4. 本 lane 不更改 package manifest/lockfile，不安装依赖，也不将当前 Node 22 的发现检查当成发布批准。

## §4 子代理失败接手清单

- 本 lane 没有派生子代理；无子代理失败、超时或未接收结果需要接手。
- 协调者应独立 review 配置只允许 `src` 测试与 redirect 单测，并在其余 W5 lane 合并后运行全量前端门。

## §5 守门规则

1. 只修改 MAGOS manifest 中登记的 `frontend/vitest.config.ts` 和本 lane brief/report/dispatch/audit 文件。
2. 保留 `src/**/*.{test,spec}.{ts,tsx}`，不得扩大排除规则或跳过测试。
3. `frontend/e2e/redirects.spec.ts` 是单独的 Vitest 测试，必须继续被发现。
4. Playwright 浏览器测试保持由 Playwright 配置和 `test:e2e` 管理。
5. focused runner 未完成时如实标记 BLOCKED，不把 file listing 说成测试通过。
6. 未验证的依赖安装、全量 suite 和 Node 24 hosted run 不得标为通过。
7. 不改共享 WBS、lockfile、`dev`、`main`，不 push、不创建 tag/release。
8. `git diff --check` 与 CypherGraph Guardian 检查必须在提交前完成；图关系只记录配置文件中的实际 imports/configuration。

## §6 签字栏

| 角色 | 审核记录 |
|---|---|
| 架构 | Mavis 接手审核：发现范围保持 `src` 测试，并白名单保留配置级 redirect 测试。 |
| SRE Lead | Mavis 接手审核：完整依赖安装、全量本地测试与 Node 24 hosted 证据缺失；不批准发行。 |
| 平台 | Mavis 接手审核：Vitest 与 Playwright runner 边界明确；focused test 被缺失依赖阻断。 |
| 评审主持 | Mavis 接手审核：lane 范围符合清单，仍需集成后的独立验证。 |
| PM | Mavis 接手审核：本 lane 不关闭 W5，也不批准创建稳定版 tag 或 release。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Vitest 不再扫描整个 e2e 树，仅保留 `redirects.spec.ts`；记录发现列表通过及 focused execution 因本机依赖缺失而未验证。 | W5 稳定版前端失败复现与 Playwright collection 修复 |
