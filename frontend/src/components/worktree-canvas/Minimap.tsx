// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/Minimap.tsx
// Minimap (per SRS §五, 1000 WT LOD 性能).
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const worktrees = useWorktreeCanvasStore((s) => s.worktrees);
  const viewport = useWorktreeCanvasStore((s) => s.viewport);
  return (
    <div
      data-testid="minimap"
      className="absolute bottom-3 right-3 h-24 w-32 rounded border border-slate-200 bg-white/80 p-1 shadow-sm backdrop-blur dark:border-slate-700 dark:bg-slate-900/80"
    >
      <div className="text-[9px] font-bold text-slate-500">minimap</div>
      <div className="relative h-16 w-full overflow-hidden bg-slate-100 dark:bg-slate-800">
        {worktrees.slice(0, 50).map((wt, i) => (
          <div
            key={wt.id}
            className="absolute h-1 w-1 rounded-full bg-blue-500"
            style={{
              left: `${(i * 13) % 95}%`,
              top: `${(i * 7) % 90}%`,
              opacity: 0.6,
            }}
          />
        ))}
        {viewport.width > 0 && (
          <div
            className="absolute border border-red-500"
            style={{
              left: `${(viewport.x / 1000) * 100}%`,
              top: `${(viewport.y / 1000) * 100}%`,
              width: `${(viewport.width / 1000) * 100}%`,
              height: `${(viewport.height / 1000) * 100}%`,
            }}
          />
        )}
      </div>
    </div>
  );
}
export const Minimap = memo(Inner);
export default Minimap;
