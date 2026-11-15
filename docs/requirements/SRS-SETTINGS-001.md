/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-SETTINGS-001.md"})
@endcypher
*/
# SRS-SETTINGS-001 — 设置

> **5 组左导航 + MCP 目录 + Mod 目录 + 开发者模式** (per docs/prototypes/6 · 设置-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit c0f149f2)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 左导航 5 组: 账户/组织与权限/集成/扩展 (Mod/MCP/Skills/Plugins/Hooks)/开发者
- 搜索设置 + 开发者模式 toggle
- MCP 服务器目录 (已连接徽标)
- Mod 目录: 官方目录/自定义双源 + 浏览 Mod 目录/从文件安装/用模板新建 (目录服务未接入 → 禁用 + 原因) + Mod 表 (名称/类型/能力/版本/启用) + 隔离运行时说明
- Hooks 在「高级设置 → Hooks」 (策略/传输方式)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-ST-001 | 5 组左导航 | ✅ |
| FR-ST-002 | MCP 目录 | ✅ |
| FR-ST-003 | Mod 目录面板 (双源/3 按钮禁用/表/隔离说明) | ✅ c0f149f2 |
| FR-ST-004 | 开发者模式 | ✅ |
| FR-ST-005 | Hooks 高级设置深链 | ✅ |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/settings/page.tsx | /settings 页 (ModPanel 内联) |
| frontend/src/app/(app)/settings/advanced/[tab]/ | 高级设置 (Hooks) |
| frontend/src/app/(app)/settings/developer/ | 开发者模式 |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
