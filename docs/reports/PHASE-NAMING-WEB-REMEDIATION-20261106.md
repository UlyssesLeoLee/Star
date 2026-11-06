# PHASE-NAMING-WEB-REMEDIATION-20261106 — Gantt 依赖链接提示

**状态**　A1 文案已修复，组件验证通过，浏览器验收待执行

**基线**　`dev@788c878c71a477131022e3367115b7037326290e`

**任务简报**　[PHASE-NAMING-WEB-REMEDIATION-20261106](../briefs/PHASE-NAMING-WEB-REMEDIATION-20261106.md)

## §0 目的

按产品命名规则修复 A1，在 Gantt 依赖链接提示中显示当前语言和实际计数。验收项来自 [产品命名审计 §1 A1](PHASE-PRODUCT-NAMING-AUDIT-20261106.md#1-命中矩阵与最小整改拆分) 和 [AGENTS.md §0](../../AGENTS.md#0-一句话硬约束)。

## §1 改动矩阵

| 验收项 | 实现与证据 |
|---|---|
| 英文提示 | `Task dependency links: {count}`。[en.ts](../../frontend/src/lib/i18n/en.ts#L271) |
| 日文提示 | `タスク依存リンク数: {count} 件`。[ja.ts](../../frontend/src/lib/i18n/ja.ts#L272) |
| 简体中文提示 | `任务依赖链接数：{count} 条`。[zh-CN.ts](../../frontend/src/lib/i18n/zh-CN.ts#L282) |
| 数量与语言 | 共享组件继续取当前字典，将已渲染链接数代入 `title`，徽标继续使用原有单数和复数分支。字典层已足以实现修复。[GanttChart.tsx](../../frontend/src/components/gantt/GanttChart.tsx#L293) |
| 项目时间线 | 入口传入项目的 `projectRelations`。组件只统计通过现有筛选且可解析到图表行的链接。[ProjectsClient.tsx](../../frontend/src/app/%28app%29/projects/ProjectsClient.tsx#L574) |
| Analytics Gantt | 入口没有传入 `relations`，组件使用默认空数组。当前入口不会显示依赖链接徽标。[Analytics 页面](../../frontend/src/app/%28app%29/analytics/page.tsx#L252)、[GanttChart.tsx](../../frontend/src/components/gantt/GanttChart.tsx#L83) |
| 图谱与文档 | 三个字典的 Cypher 源码哈希已刷新，关系语句一致。需求依据、实现范围和验收状态在本报告与 A1 审计条目间对照。 |

## §2 验证摘要

前端使用锁定的依赖，执行环境为 Node 26.7.0、npm 10.9.3、TypeScript 7.0.2、Vitest 5.0.1。临时组件验证文件执行后已移除。

| 检查 | 实际结果 |
|---|---|
| `npm ci --no-audit --no-fund --fetch-retries=1 --fetch-timeout=15000` | 通过。安装时默认 Node 22.19.0 对 jsdom 30 等发出引擎版本警告，后续验证使用 Node 26.7.0。安装刷新了本地工作树的 MSW 生成文件，已恢复该安装副产物。 |
| `npm run typecheck -- --incremental false` | 通过，退出码 0。 |
| `vitest run src/components/gantt --maxWorkers=2` | 2 个文件、12 项通过。 |
| A1 临时 Vitest/jsdom 组件验证 | 1 项通过。挂载实际 `GanttChart` 与 `I18nProvider`，检查 0 条时无徽标、1/2 条时的数量与单复数、英→日→中→英切换后的精确 `title`。输入使用 2 个可解析到图表行的关系 fixture。 |
| `npm test -- --maxWorkers=2` | 109 个文件通过、2 个失败。950 项中 946 项通过、4 项失败。临时导出固定基线，使用同一依赖和 Node 对两个失败文件复跑，得到相同的 4 个失败。MobileBottomNav 的三语言断言以及 Sidebar 的绑定断言期望旧命名，基线返回 `LANE`。 |
| `npm run lint` | 退出码 1。现有脚本执行 `next lint`，锁定的 Next 16.3.5 将 `lint` 解析成项目目录并报该目录不存在。 |
| CypherGraph Guardian `--changed --stamp` | 3 个源码文件通过。 |
| Git 差异检查 | `git diff --check` 和暂存差异检查通过。工作树的 `.codegraph/` 只有跟踪的 `.gitignore`，MCP 报未索引。Codebase Memory 覆盖工具未暴露，入口和绑定以对应源码为据。 |
| 浏览器 / 发布 / Rust | 浏览器 E2E、生产构建、发布制品和 Rust 守门均未运行。 |

## §3 已知缺口

- 项目时间线仍待使用包含可渲染依赖关系的数据做浏览器验收。Analytics Gantt 目前缺少关系数据接线，A1 的该入口验收待确定该页面是否应展示依赖关系。
- 全套前端测试的四个基线失败与现有 lint 命令失败见 §2。
- Web 命名审计的下一组为 A2、B1、B4、B5。验收重点包括真实或未接入的连接状态、旧链接映射、兼容输入，以及公共错误提示与受限诊断的区分。[产品命名审计 §1](PHASE-PRODUCT-NAMING-AUDIT-20261106.md#1-命中矩阵与最小整改拆分)

## §4 子代理失败接手清单

本工作线由 root 调度到独立工作树，未再委派，无子代理失败待接手。

## §5 守门规则

| # | 门禁 | 结果 |
|---|---|---|
| 1 | 固定基线 | 开工时工作树与本地 `dev` 均为 `788c878c`。 |
| 2 | 隔离工作树 | 工作分支为 `codex/naming-web-remediation-20261106`。 |
| 3 | 用户可见命名 | 三种提示只显示依赖链接数量。 |
| 4 | 字典结构 | 原键及单个 `{count}` 占位符保留。 |
| 5 | 链接筛选 | 组件以可渲染链接数组为计数来源。 |
| 6 | 数量边界 | 组件验证覆盖 0、1、2 条链接。 |
| 7 | 语言切换 | 组件验证覆盖英→日→中→英。 |
| 8 | 页面入口 | 已核对 Projects 与 Analytics 传参。 |
| 9 | 浏览器验收 | 待执行，Analytics 的可达性限制见 §3。 |
| 10 | TypeScript | 通过。 |
| 11 | Gantt 既有测试 | 12 项通过。 |
| 12 | 前端全套测试 | 4 项失败已在基线复现。 |
| 13 | 前端 lint | 现有命令失败，见 §2。 |
| 14 | 图谱头 | 3 个源码哈希已刷新并通过校验。 |
| 15 | 需求与验收对账 | 对照 A1 和产品命名约束，分别记录实现与页面验收状态。 |
| 16 | 提交范围 | 三份字典及本工作线的 brief、报告与验证日志。 |

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008)，核对共享组件和入口边界。 |
| SRE Lead | Mavis 接手审核，核对本地执行结果与基线复测。 |
| 平台 | Mavis 接手审核，核对 TypeScript 与图谱校验。 |
| 评审主持 | Mavis 接手审核，核对改动范围与验收缺口。 |
| PM | Mavis 接手审核，记录 A1 页面验收与后续 Web 审计项。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-06（JST） | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 A1 三语言提示修复，记录组件验证、基线失败和页面入口缺口。 | 产品命名审计 A1 |
