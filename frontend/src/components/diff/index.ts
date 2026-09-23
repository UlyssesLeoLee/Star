/**
 * ULYS-165 FR-ORCA-035 — Diff annotation frontend components.
 *
 * Per docs/ecosystem-survey/orca-design-survey.md §11 line 483-485:
 *   在 Diff 上写注释 → 回喂 Agent
 *
 * **本文件 (ULYS-165 v2)** — 适配 PR #84 (ULYS-201) DiffAnnotation DTO:
 *   - 移除 status 字段
 *   - 用 author: 'user' | 'agent' 替代
 *
 * Exports:
 * - DiffAnnotator: 行内 diff 注释组件 (挂在 react-diff-view 行号 gutter 上)
 * - InlineComment: 单条 annotation 卡片 (PR 详情 / Agent 详情 / 悬浮展开)
 */

export { DiffAnnotator } from "./DiffAnnotator";
export type { DiffAnnotationView, DiffAnnotationAuthor } from "./DiffAnnotator";
export { InlineComment } from "./InlineComment";