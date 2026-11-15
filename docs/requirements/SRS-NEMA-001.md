/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-NEMA-001.md"})
@endcypher
*/
# SRS-NEMA-001 — 协同 (文件即任务 · 多人多 Agent)

> **.nema 侧车文件即任务协同** (per docs/prototypes/2b · 协同（文件即任务 · 多人多 Agent）-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 1787896e)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 当前 Worktree · Branch 隶属条
- 任务卡列表 + IPA 文件即任务分组
- 文件组列表 (类型/名称/在线状态) + 关联文件到此任务卡 + 导入表格/导入文本/从 Branch 选择
- .nema 侧车: 每文件可配侧车, 多人多 Agent 并行编辑后汇总生成代码
- 5 步流水线 + .nema 权威规约 (REV 递增) + 源文件 SourceMap 对齐
- NEMA 传输头 !NEMAT1 + parse/serialize (wasm-first: nema-codec-wasm, JS fallback)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-NM-001 | 文件即任务 (.nema 侧车 → 任务卡) | ✅ |
| FR-NM-002 | 5 步流水线 | ✅ |
| FR-NM-003 | NEMA codec wasm-first + JS fallback | ✅ |
| FR-NM-004 | 文件监听导入 (nemaWatcher) | ✅ |
| FR-NM-005 | 打开为任务 (openNemaAsStarTask) | ✅ |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/nema/page.tsx | /nema 页 |
| frontend/src/lib/nema/nema.ts | codec (parse/serialize/wasm 通道) |
| frontend/src/lib/nema/nemaStore.ts | nemaTasks + 开卡联动 |
| frontend/src/lib/nema/nemaWatcher.ts | 文件监听 |
| frontend/src/lib/nema/wasm/ | nema-codec-wasm glue+wasm |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
