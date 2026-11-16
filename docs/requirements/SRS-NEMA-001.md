/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-NEMA-001.md"})
@endcypher
*/
# SRS-NEMA-001 — 协同 (文件即任务 · 多人多 Agent)

> **.nema 侧车文件即任务协同** (per docs/prototypes/2b · 协同（文件即任务 · 多人多 Agent）-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.2 — profile-aware Rust NEMA I/O 已实现；第三方解释器独立安装/运行时未接通
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: 新建/导入条目通过所选 Rust profile 解析和格式化；profile 固定到条目。未固定 profile 的旧条目继续走 legacy 投影。在线语法页本轮无法读取，当前快照不声明与线上一致。
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 当前 Worktree · Branch 隶属条
- 任务卡列表 + IPA 文件即任务分组
- 文件组列表 (类型/名称/在线状态) + 关联文件到此任务卡 + 导入表格/导入文本/从 Branch 选择
- .nema 侧车: 每文件可配侧车, 多人多 Agent 并行编辑后汇总生成代码
- 5 步流水线 + .nema 权威规约 (REV 递增) + 源文件 SourceMap 对齐
- profile-aware Rust parse/serialize：`nema-codec` 提供原生 Rust 解释器，WASM 仅作为前端调用桥；每个条目固定精确 grammar/plugin profile，不自动选 latest。
- legacy TypeScript codec 只保留给没有 profile 的既有持久化条目；新建和新导入不允许回退到 legacy parser。

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-NM-001 | 文件即任务 (.nema 侧车 → 任务卡) | ✅ |
| FR-NM-002 | 5 步流水线 | ✅ |
| FR-NM-003 | 新建/导入条目由所选 Rust interpreter profile 解析与格式化；Rust 不可用时 fail closed | ✅ bundled profile 路径 |
| FR-NM-004 | 文件监听对已固定条目复用原 profile；新文件导入要求显式 profile | ✅ |
| FR-NM-005 | 打开为任务时按条目 profile 解析；重复并发打开保持幂等 | ✅ |
| FR-NM-006 | 保存语法修订、插件版本、ABI/AST 版本和摘要；升级不自动重解释旧条目 | ✅ |
| FR-NM-007 | 允许通过解释器 adapter 增加 grammar profile；profile 缺失时不猜测 | 🟡 接口和内置版本已实现；独立安装/隔离执行第三方插件尚未接通 |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/nema/page.tsx | /nema 页 |
| frontend/src/lib/nema/nema.ts | codec (parse/serialize/wasm 通道) |
| frontend/src/lib/nema/nemaStore.ts | nemaTasks + 开卡联动 |
| frontend/src/lib/nema/nemaWatcher.ts | 文件监听 |
| frontend/src/lib/nema/wasm/ | nema-codec-wasm glue+wasm |
| crates/nema-codec/src/strict.rs | 严格、可往返的 Rust grammar snapshots |
| crates/nema-codec/src/interpreter_plugin.rs | 精确 profile registry 与 interpreter adapter seam |
| crates/nema-codec-wasm/src/lib.rs | Rust parser/formatter profile WASM bridge；JS u64 使用十进制字符串 |
| crates/nema-codec/NEMA_INTERPRETER_PLUGIN_CONTRACT.md | profile 兼容规则、扩展边界和未接入运行时说明 |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
| 2026-11-15 | v1.1 | 状态回写核对 | 本提交 |
| 2026-11-15 | v1.2 | Rust 原生 profile-aware I/O、精确版本固定、并发幂等及第三方插件运行时缺口 | 本工作区（未提交） |
