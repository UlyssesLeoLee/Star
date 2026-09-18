// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorFiles.tsx
// Inspector Tab 7/11: Files.
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  return (
    <div data-testid="inspector-files" className="p-4 text-sm">
      <p className="text-slate-500">
        {selectedId ? `Files modified by ${selectedId.slice(0, 8)} (placeholder)` : "请选择 Worktree"}
      </p>
    </div>
  );
}
export const InspectorFiles = memo(Inner);
export default InspectorFiles;
