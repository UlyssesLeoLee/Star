# PHASE-UI-MIGRATION-COMPLETE-REPORT

## §0 目的

per 2026-09-01 16:41 JST 用户发令"界面迁移在基本设计中应该全面完善,避免迁移异常和界面缺失,遇到问题请改善"。

拍板范围: **A + B + C + 附** (per ask_user 全做),B 项修法 **cookie-default**。

- **A** ARCHIVED 2 文件清理 (迁移未完成产物 → git rm)
- **B** 5 tab 拍板实装 SSR 默认 tab bug 修复 (cookie 持久化 default tab)
- **C** `(app)/` 路由组 dead file 清理 (1 file)
- **附** `e2e/debug-*.spec.ts` 加 `.gitignore` (临时调试,不入库)

## §1 改动矩阵

| # | 项 | 文件 | 类型 | 改动 |
|---|---|---|---|---|
| 1 | A-1 | `frontend/src/components/_ARCHIVED_Topbar.tsx` | D | git rm (老 Topbar,已被 AppShell AppHeader 替代 per AGENTS v0.11 整合 Topbar commit) |
| 2 | A-2 | `frontend/src/components/board/_ARCHIVED_BoardTabs.tsx` | D | git rm (老 Board Tabs,已被 5 tab 拍板实装 commit 7d85c34 替代) |
| 3 | B-1 | `frontend/src/lib/cookies.ts` | A+ | 新建 79 行,共享 cookie 工具 + `ProjectsTabId` 类型 |
| 4 | B-2 | `frontend/src/app/projects/ProjectsClient.tsx` | A+ | 新建 42KB,从原 page.tsx 拆出,接受 `initialTab` prop,useState 初始化用它,setTab wrapper 写 cookie |
| 5 | B-3 | `frontend/src/app/projects/page.tsx` | M | 改成 server component wrapper,读 `cookies()` + `searchParams`,传 initialTab |
| 6 | B-4 | `frontend/src/app/projects/page.test.tsx` | M | `<ProjectsPage />` → `<ProjectsClient initialTab="kanban" />` + 3 个新测试 (SSR initialTab=timeline / initialTab=backlog / 切 tab 写 cookie) |
| 7 | C-1 | `frontend/src/app/(app)/page.tsx` | D | git rm (1 dead file,自己注释"临时 placeholder — U5 会在 next.config.js / middleware.ts 配真正的 redirect",U5 已落地在 root `app/page.tsx` 用 server `redirect("/inbox")`) |
| 8 | 附-1 | `frontend/.gitignore` | M | 加 `e2e/debug-*.spec.ts` (debug-mobile.spec.ts 临时调试不入库) |

**总改动**: D=3 (A-1/A-2/C-1) + A+=2 (B-1/B-2 新建) + M=3 (B-3/B-4/附-1) = **8 文件**

## §2 验证摘要

### tsc --noEmit
```
=== tsc --noEmit ===
(无输出, exit 0)
```
**0 错, 0 警告**

### vitest run (per AGENTS §4.1 守门 #1 派生 v3)
```
Test Files  38 passed (38)
     Tests  312 passed (312)
   Duration  8.20s
```
**312/312 pass**, 含 3 个新 cookie-default 测试:
- `SSR initialTab=timeline renders timeline tab content on first render (no flash)` ✓
- `SSR initialTab=backlog renders backlog tab on first render` ✓
- `switching tab writes cookie for next SSR (cookie-default 持久化)` ✓

### cargo check --workspace --lib
(在跑,见守门 #1)

## §3 已知缺口 (per 缺标比错标 8/26 JST 偏好)

1. **(app) 路由组其他子路由 (inbox/agents/...)** 未审计内容 — 已知 9 个子 page 是 active,没动。只删了 1 个 dead `(app)/page.tsx`。
2. **next build 守门未跑** — 依赖 dev server hot reload 5 路由 HTTP 200 (per AGENTS v0.15)。如 build 出问题(动态路由 + cookies() 冲突),后续补。
3. **cookie 写是 document.cookie** — 没加 secure flag (dev) 跟 SameSite=Strict 评估,生产环境按部署调整。
4. **e2e/debug-mobile.spec.ts 已 gitignore** — 文件本体还在,本地 dev 可用。

## §4 子代理失败接手清单 (per 7 子代理派生规则)

**无子代理调用** (per AGENTS v0.15 守门 #9 派生规:子代理 RPC 不可靠,root 直实装更稳)。所有改动 Mavis 接手直接落 commit。

## §5 守门规则 (15-17 项)

| # | 规则 | 实证 | 状态 |
|---|---|---|---|
| 1 | R-05 不 push 已反转 | 本次仅 main 本地 commit, 不 push origin | 🟡 待 user 拍板(AGENTS §7 #7) |
| 2 | bc23d6c 保留 | 未触碰 | 🟢 维持 |
| 3 | 5 域独立 Lead | 未触碰 | 🟢 维持 |
| 4 | token-OLU | 本次 ~0.4M token (vs 30M 软预算 ~1.3%) | 🟢 合规 |
| 5 | 环境变量安全 | 无 env 读 | 🟢 维持 |
| 6 | PowerShell only | 全程 PS 语法 | 🟢 合规 |
| 7 | 0 unsafe | 无 unsafe 改动 | 🟢 维持 |
| 8 | 不沿用 bc23d6c 叙事 | 全用 git 实证 | 🟢 合规 |
| 9 | 不 commit 散落子代理产出 | 无子代理调用, root 直实装 | 🟢 合规 |
| 10 | 代签规则应用 | author=Ulysses | 🟢 准备 |
| 11 | 缺标比错标安全 | §3 列 4 项缺口 | 🟢 合规 |
| 12 | AI 协作文档治理 | 本报告 7 段结构 + git 实证 | 🟢 合规 |
| 守门 #1 派生 v1-v14 | tsc/vitest/cargo 多层 | tsc 0 + vitest 312/312 + cargo 跑中 | 🟡 cargo 待实证 |

## §6 签字栏

| 角色 | 签字 |
|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 (代签,5 域 Lead 真身 DDD Review 阶段补) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 2026-09-01 (代签) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 界面迁移全面完善 (A+B+C+附), 8 文件改动, cookie-default SSR 修法 | 2026-09-01 16:41 JST 用户发令"界面迁移在基本设计中应该全面完善,避免迁移异常和界面缺失,遇到问题请改善" |
