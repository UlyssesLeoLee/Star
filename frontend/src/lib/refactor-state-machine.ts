// =====================================================================
// refactor-state-machine — 桩实现 (per 2026-09-05 pre-existing 修基线)
// =====================================================================
// 现状 (per 缺标比错标 8/26 JST 偏好, 守门 #1 派生 v1):
//   - src/app/refactor/page.tsx 已实装, 引用 transitionKind + needsTransitionConfirm
//   - 本文件原本缺失, 阻塞 next build + typecheck + refactor/page.test.tsx transform
//   - 本 commit 只补最小桩 (identity / always-false), 让基线恢复绿色
//   - 真实业务语义 (per docs/specs/refactor.md) 留给后续 Phase I+ 实装
//
// 已知缺口 (per 缺标比错标, DDD Review Lead 必查):
//   1. transitionKind() 暂返 'progress' 默认值, 不区分 forward/backward/skip
//   2. needsTransitionConfirm() 暂返 false, 所有 transition 不弹确认
//   3. RefactorStatus 暂用 string 占位, 真实状态机 (per docs/frontend/design/...
//      refactor-state-machine.md §3) 5 状态 (draft / queued / in_progress /
//      review / done) 未落地
//   4. 不引用 type 防止循环依赖; 接受 string 输入
//   5. 无单测 (待 Phase I+ 业务实装后一起补, 桩实现加测意义不大)
// =====================================================================

/**
 * 判定一次 Refactor 状态 transition 的"类型"
 * - 桩实现: 一律返 'progress' (前向推进)
 * - 真实实现 (待 Phase I+): 区分 'forward' / 'backward' / 'skip' / 'lateral' / 'reopen' / 'invalid' / 'same'
 */
export function transitionKind(_from: string, _to: string): "forward" | "backward" | "skip" | "lateral" | "progress" | "reopen" | "invalid" | "same" {
  return "progress";
}

/**
 * 判定一次 transition 是否需要弹确认对话框
 * - 桩实现: 一律返 false
 * - 真实实现 (待 Phase I+): 例如 done → in_progress (回滚) 需确认
 */
export function needsTransitionConfirm(_from: string, _to: string): boolean {
  return false;
}
