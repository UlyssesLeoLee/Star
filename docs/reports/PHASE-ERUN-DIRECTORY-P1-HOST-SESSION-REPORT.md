# PHASE-ERUN-DIRECTORY-P1 Host Session Lifecycle Regression Report

> v0.1 · 2026-10-15 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：🟡 Phase 1 开放。本报告只记录宿主会话生命周期的前端回归切片，不表示真实宿主认证或 Run Directory 生产闭环完成。
> 依据：`docs/briefs/P1-RUN-SESSION-TESTS-20261015.md`；实现合入 `dev` 的 commit `de31aaa7de0cee38efc27d0a14acb40f4f830ee9`。

## §0 目的

记录 Run Directory session adapter 在宿主会话缺失、认证失效和身份 generation 切换时必须 fail closed 并销毁旧投影的回归覆盖。测试用于验证前端状态边界，不替代宿主认证、授权目录、Run Apps 或实际执行验收。

## §1 改动矩阵

| 范围 | 实际内容 | 状态 |
|---|---|---|
| 前端测试 | `frontend/src/lib/run/runDirectorySession.test.tsx` 增加四项用例：缺失 host session 不开放目录；有效 host session 提供 ready client 并允许 Run focus；401 authorization invalidation 清除 client/focus；session generation 变化清除 focus、dispose 旧 client 并 abort 旧请求 | 已合入；仅测试代码 |
| 生产接线 | host authentication/session provider、trusted Branch/Run ingest 与 grants writer、Run Apps migration 和运行时接线 | 未实现/未验收 |
| 文档同步 | requirements/basic design、Group DD、实施计划、master plan 与本报告同步实际验证边界 | 本文档 lane |

## §2 验证摘要

- `npm run typecheck`：PASS。
- Vitest：未启动；依赖导入缺少 `@csstools/css-parser-algorithms/index.js`。因此本轮没有 Vitest 通过或失败结果。
- 上述结果依据实现 lane 对 `de31aaa7de0cee38efc27d0a14acb40f4f830ee9` 的验证记录；本 docs-only lane 未重跑实现测试。
- 未验证浏览器/桌面运行、生产 IdP/host session、PostgreSQL 目标库、RLS/grants 或性能。

## §3 已知缺口

1. 真实 host authentication/session provider 尚未接入；前端 provider 测试不能证明用户身份由可信宿主签发。
2. 可信 SCM Branch ingest、Project/Branch/Run grants provisioning 与 Run Worktree binding writer 尚未打通。
3. Run Apps ownership migration、目标 DB/runtime grants/RLS 与成功授权运行尚未验证。
4. CLI admission、真实 Runtime provider、OS sandbox、取消/恢复、独立验证和结果写回仍未开放；Phase 1 和整体交付不得标完成。

## §4 子代理失败接手清单

本报告 lane 未派生额外子代理。Vitest 因缺少 CSS parser module 未启动；不得把 typecheck 结果当成测试结果。生产 host/session 与 Run Apps 缺口由后续 implementation lane 负责，不由本回归切片代签。

## §5 守门规则

1. 缺少宿主 session 或认证目录失败时 fail closed，不回退到 seed/mock。
2. 401 必须清除旧 API client、目录投影与 Run focus。
3. session generation 改变必须 dispose 旧 client 并取消旧请求。
4. typecheck 与 Vitest 分开报告；未启动不得写成通过。
5. 只记录上述前端回归证据，不推断生产认证、Run Apps 或 CLI 已可用。
6. Phase 1、生产授权和 execution admission 继续开放/关闭，按文档明确状态呈现。
7. 本轮只同步文档，不修改代码、migration 或测试 fixture。

## §6 签字栏

| 角色 | 审阅状态 |
|---|---|
| 架构 | Mavis 接手审核；测试边界与 Run/Worktree owner 区分一致 |
| SRE Lead | Mavis 接手审核；无生产会话或数据库部署证据 |
| 平台 | Mavis 接手审核；typecheck 通过，Vitest 未启动 |
| 评审主持 | Mavis 接手审核；未将缺失测试依赖解释为通过 |
| PM | Mavis 接手审核；Phase 1 仍开放，生产可用性未达成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-15 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Run Directory 四项 host-session 生命周期回归、typecheck 通过、Vitest 未启动及生产接线缺口；实现合入 dev 后同步文档 |
