/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/prototypes/INDEX.md"})
@endcypher
*/
# Prototypes 索引 — 设计 → 功能 → 权威文档

> 本目录 9 个 mockup 是各界面功能的参考设计。**修改某个功能时, 只查/改该功能对应的单个权威文档**
> (docs/requirements/SRS-XXX-001.md), 不要跨文档翻找; 改动后在对应 SRS 的 §修订履历 加行。

## 映射表 (mockup → 页面 → 权威文档 → 关键文件)

| # | Mockup 目录 | 功能/页面 | 权威文档 | 关键实现文件 |
|---|---|---|---|---|
| 0 | 0 · 页面合并图-html | 导航/页签架构 (64 页面 → 7 界面) | docs/frontend/design/ui-3pane-arch.md | src/lib/nav/registry.ts · src/components/nav/AppMatrixDrawer.tsx |
| 1 | 1b · 任务 · IPA 看板-html | 任务 · IPA 看板 (/kanban) | SRS-KANBAN-001.md | src/app/(app)/kanban/page.tsx · components/board/* |
| 2 | 2b · 协同（文件即任务 · 多人多 Agent）-html | 协同 (/nema) | SRS-NEMA-001.md | src/app/(app)/nema/page.tsx · src/lib/nema/* |
| 3 | 2c · 沙盒-html | 沙盒 (/sandbox) | SRS-SANDBOX-001.md | src/app/(app)/sandbox/page.tsx · crates/sandtree-core · crates/argo-core |
| 4 | 2d · 原型-html | 原型生成器 (/prototype) | SRS-PROTOTYPE-001.md | src/app/(app)/prototype/page.tsx |
| 5 | 3 · 收件箱-html | 收件箱 (/inbox) | SRS-INBOX-001.md | src/app/(app)/inbox/page.tsx |
| 6 | 5 · 分析-html | 效能分析 (/analytics) | SRS-ANALYTICS-001.md | src/app/(app)/analytics/page.tsx |
| 7 | 6 · 设置-html | 设置 (/settings, 含 Mod 目录) | SRS-SETTINGS-001.md | src/app/(app)/settings/* |
| 8 | 7 · 运维控制台（仅管理员）-html | 运维控制台 (/ops) | SRS-OPS-CONSOLE-001.md | src/app/ops/page.tsx · docs/detailed-design/OPS-DETAILED-DESIGN-001.md |

## 使用规则

1. **一功能一文档**: 每个功能的权威文档是表内指认的那一份 SRS; 设计要点、FR、实现状态、文件映射都在该文档内。
2. **改功能只动单文档**: 修改某功能后, 只更新对应 SRS 的 §修订履历 (版本/内容/commit), 不新建平行文档。
3. **新增功能**: 新建 SRS-<FEATURE>-001.md (本 INDEX 的模板结构见任一现有 SRS 头部), 并在本表加一行。
4. **跨功能改动** (导航/主题/布局等全局): 记录在 docs/frontend/design/ui-3pane-arch.md 或对应 architecture/ 文档, 不在各功能 SRS 里重复。
5. mockup 原始文件是本目录的 .dc.html — 只读参考; 设计变更时先改 SRS, mockup 本身不追改。

## 审计状态 (2026-11-15)

9 个 mockup 全部有对应实现且浏览器实测通过; 缺口 (Mod 目录面板、Backlog 快捷建卡) 已在 commit c0f149f2 补齐。
