// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/FocusMode.tsx
// Focus Mode toggle (per SRS §十七 + DD §17).
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const focusMode = useWorktreeCanvasStore((s) => s.focusMode);
  const setFocusMode = useWorktreeCanvasStore((s) => s.setFocusMode);
  const focusNodeId = useWorktreeCanvasStore((s) => s.focusNodeId);
  return (
    <div className="flex items-center gap-2 border-b border-slate-200 px-3 py-2 text-xs dark:border-slate-700">
      <button
        type="button"
        data-testid="focus-mode-toggle"
        onClick={() => setFocusMode(!focusMode)}
        className={`rounded px-2 py-1 ${
          focusMode
            ? "bg-blue-500 text-white"
            : "bg-slate-100 text-slate-700 dark:bg-slate-800 dark:text-slate-300"
        }`}
      >
        🎯 Focus {focusMode ? "ON" : "OFF"}
      </button>
      <span className="text-slate-400">
        {focusNodeId ? `→ ${focusNodeId.slice(0, 8)}` : "no selection"}
      </span>
    </div>
  );
}
export const FocusMode = memo(Inner);
export default FocusMode;
