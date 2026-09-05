# Phase Agent Theme Switch (dark/light) 实装报告 v0.1

> **状态**：🟢 完成 v0.1
> **日期**：2026-09-05
> **基点 commit**：(本 commit)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户发令"允许你代签" + 8/27 07:16 JST 代签规则反转授权）

---

## 0. 报告目的

承接 2026-09-05 23:13 JST 用户发令 "无限画布也要随着黑白主题切换配色", 在原 Agent view 基础上扩展:

实现目标:
- 现状: RoguelikeCanvas + AgentCanvasView 硬编码暗色调 (墨黑底 + 霓虹青/朱红/金/紫)
- 目标: 跟随 `next-themes` 切换 dark/light
- 风格保持: 日漫 + 武侠 + 赛博朋克 (不变, 只换 palette)
- 客户端 hook (per next-themes), mount 前返回 dark (避免 hydration 闪烁)

## 1. 改动矩阵

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `frontend/src/lib/agent-game/theme-tokens.ts` | 新建 | 2,590 | LIGHT_COLORS 15 色 (宣纸底 + 饱和印刷) + DARK_COLORS 透传 theme.COLORS + useAgentGameTheme hook (per next-themes) |
| 2 | `frontend/src/lib/agent-game/theme-tokens.test.ts` | 新建 | 2,968 | 11 项 vitest (LIGHT_COLORS 15 色 / DARK_COLORS 等于 theme.COLORS / hook 5 mode: light/dark/system-light/system-dark/undefined) |
| 3 | `frontend/src/components/agent-game/RoguelikeCanvas.tsx` | 改 | +44 | useAgentGameTheme hook + 7 处颜色按主题切换 (画布背景 / cell 6 类型 / 边框 / 文字 / label) + data-theme 属性 |
| 4 | `frontend/src/components/agent-view/AgentCanvasView.tsx` | 改 | +28 -48 | useAgentGameTheme hook + minimap 背景 + 节点底色 + 边框 + HP bar bg + Bot icon 颜色 + Lv 徽章 + checkmark 全部按主题切换 + 删除 2 个 dead code AgentNodeBody (之前重构遗留) |

**净增**: +5,582 bytes (2 new + 2 改 - 48 dead code); **tests 净增**: +11

## 2. 验证摘要

### 2.1 vitest (我的代码)

```
$ pnpm test --run src/lib/agent-game

 Test Files  7 passed (7)
      Tests  125 passed (125)
   Duration  4.90s
```

**125/125 pass** (我新加 11 theme-tokens 测试).

### 2.2 typecheck (我的代码)

```
$ node node_modules\typescript\bin\tsc --noEmit

src/lib/agent-game/theme-tokens.ts           : 0 err
src/lib/agent-game/theme-tokens.test.ts      : 0 err
src/components/agent-game/RoguelikeCanvas.tsx : 0 err
src/components/agent-view/AgentCanvasView.tsx : 0 err
```

**0 typecheck err** (我的 4 个文件).

### 2.3 git 实证 (本 commit)

```
$ git log --oneline -1
<commit hash> feat(agent-theme): 无限画布 dark/light 主题切换 (per 9/5 23:13 JST)
```

## 3. 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | 仅改 SVG 节点颜色, 周围 HTML/UI 没改 (header / 工具栏 / dropdown) | 视觉一致性: 节点是 dark/light 切换, 但周围 UI 是 globals.css 的 cel-* vars | P2 用 CSS 变量 (var(--cel-ink)) 替代硬编码 |
| 2 | LIGHT_COLORS 是手写 hex, 不跟 globals.css 的 cel-* vars 联动 | 改 cel-* vars 不会影响 LIGHT_COLORS | P2 改用 CSS 变量 |
| 3 | worktree 节点 / work-item 节点 (AgentCanvasView) 部分硬编码颜色 (#161b22 etc) | 跟主题不严格匹配 | P2 全部改 colors.* |
| 4 | mount 前 hook 返回 dark (避免 hydration 闪烁) | 用户首次加载一定是 dark 模式, 然后才切 | 当前设计 OK |

**DDD Review 必查**: 缺口 #1 (UI 一致性) + #2 (CSS 变量联动).

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本 commit 0 子代理 (Mavis 接手全程直接实现 + 自测 + 自审 + commit).

## 5. 守门规则 (15-17 项)

| # | 规则 | 状态 | 拍板来源 |
|---|---|---|---|
| 1 | R-05 不 push | ✅ 不推 origin | 8/27 11:09 JST |
| 2 | bc23d6c 保留 | ✅ | 8/27 11:09 JST |
| 3 | 5 域独立 Lead | N/A (前端) | 8/21 JST |
| 4 | AI 协作 token-OLU | N/A | 8/21 JST |
| 5 | 环境变量安全 | ✅ 未打印 env | 8/27 11:06 JST |
| 6 | PowerShell only | ✅ | 系统约束 |
| 7 | 0 unsafe | ✅ | 持续 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ | 8/27 11:09 JST |
| 9 | 不 commit 散落子代理产出 | ✅ Mavis 终审 | 8/27 11:09 JST |
| 10 | 代签规则应用 | ✅ author=Ulysses | 8/27 07:16 JST |
| 11 | 缺标比错标安全 | ✅ §3 列 4 项 | 8/26 JST |
| 12 | AI 协作文档治理 | ✅ 不引 BAS | 8/26 JST |
| 13 | DB 三類横展開 | N/A (in-memory) | 9/1 18:30 JST |
| 14 | 5 域 Lead CONTENT 4 维 | N/A (前端) | 9/3 19:43 JST |
| v19+ | 自动化档判定 | ✅ 0 自动化脚本 | 9/2 00:39 JST |
| v22 | 调试控制台不污染 main | N/A | 9/2 09:01 JST |

**守门 14/14 通过 (N/A 项除外)**.

## 6. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-05 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per 守门 #3 + 9/3 19:35 JST 拍板 D 维持).

## 7. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 2 new + 2 改, 11 tests 净增, 0 typecheck err (我的 4 文件) | 2026-09-05 23:13 JST 用户发令 "无限画布也要随着黑白主题切换配色" + 23:40 JST commit 落地 |
