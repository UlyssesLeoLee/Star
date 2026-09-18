// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorRelations.tsx
// Inspector Tab 9/11: Relations.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const edges = useWorktreeCanvasStore((s) => s.edges);
  const related = edges.filter((e) => e.source === selectedId || e.target === selectedId);
  return (
    <div data-testid="inspector-relations" className="p-4 text-sm">
      {!selectedId ? <p className="text-slate-500">请选择 Worktree</p> : (
        <ul className="space-y-1 text-xs">
          {related.length === 0 ? <li className="text-slate-500">无关系</li> : related.map((e, i) => (
            <li key={i} className="rounded bg-slate-50 px-2 py-1 font-mono dark:bg-slate-800/50">
              {e.kind} {e.source.slice(0, 8)} → {e.target.slice(0, 8)}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
export const InspectorRelations = memo(Inner);
export default InspectorRelations;
