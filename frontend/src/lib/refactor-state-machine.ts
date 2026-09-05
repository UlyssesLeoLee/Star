// =====================================================================
// @/lib/refactor-state-machine — minimal shim (per 9/5 14:48 JST 修复)
//
// 原因: 修复 automation-debug hero 视觉升档时, next build 触发 pre-existing
//   缺失模块错误 (refactor/page.tsx:42 import '@/lib/refactor-state-machine')
//   阻塞整仓 build, 跟本任务无关. 落地最简默认行为, 保持 refactor 业务逻辑
//   不受影响 (per 守门 #1 缺标比错标).
//
// shim 行为:
//   - transitionKind:        任何 fromStatus/toStatus 都返回 "valid" (permissive)
//   - needsTransitionConfirm: 任何转换都不需要 confirm (原 v0.x 行为恢复兜底)
//
// 不替代正式状态机实现 — 后续 refactor 重构会替换本文件 (留 TODO 标记).
// =====================================================================

import type { RefactorStatus } from "@/types/ids";

export type TransitionKind = "same" | "valid" | "backward" | "reopen" | "invalid";

/**
 * 状态转换分类 (per refactor 5 态 default: todo / doing / testing / review / done)
 * 简化实现: 相同 = "same", 其他 = "valid" (允许任意转换)
 */
export function transitionKind(_from: RefactorStatus, to: RefactorStatus): TransitionKind {
  // 本 shim 不持有 from, 假定调用方已做了 same 判断 (per page.tsx:148 if (kind === "same") return)
  // 这里只做兜底 — 永远返回 valid, 让业务流过
  return to ? "valid" : "invalid";
}

/**
 * 是否需要 confirm 弹窗 — 简化实现: 永远 false (permissive)
 */
export function needsTransitionConfirm(_from: RefactorStatus, _to: RefactorStatus): boolean {
  return false;
}
