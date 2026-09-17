// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorActions.tsx
// Inspector Tab 11/11: Actions (per spec §4.4 18 Action matrix).
"use client";
import { memo, useState } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";
import { ACTION_DANGER, type WorktreeAction } from "./types";
import { ConfirmationModal } from "./ConfirmationModal";

const ACTION_LIST: WorktreeAction[] = [
  "CreateWorktree", "OpenWorktree", "OpenInIde", "Compare",
  "SyncMain", "Rebase", "Merge", "CreatePR",
  "Lock", "Unlock", "Delete", "Cleanup",
  "Archive", "MarkSuperseded", "SetDependency", "RemoveDependency",
  "Focus", "ExplainRisk",
];

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  const [pending, setPending] = useState<WorktreeAction | null>(null);

  return (
    <div data-testid="inspector-actions" className="p-4 text-sm">
      {!selectedId ? <p className="text-slate-500">请选择 Worktree</p> : (
        <>
          <ul className="grid grid-cols-2 gap-2">
            {ACTION_LIST.map((a) => {
              const danger = ACTION_DANGER[a];
              const cls =
                danger === "Destructive"
                  ? "border-red-300 hover:bg-red-50"
                  : danger === "Warning"
                  ? "border-amber-300 hover:bg-amber-50"
                  : "border-slate-200 hover:bg-slate-50";
              return (
                <li key={a}>
                  <button
                    type="button"
                    data-testid={`action-${a}`}
                    data-danger={danger}
                    onClick={() => danger === "Destructive" ? setPending(a) : null}
                    className={`w-full rounded border px-3 py-1 text-left text-xs ${cls} dark:border-slate-700`}
                  >
                    {a} <span className="ml-1 text-slate-400">({danger})</span>
                  </button>
                </li>
              );
            })}
          </ul>
          {pending && (
            <ConfirmationModal
              action={pending}
              worktreeId={selectedId}
              onClose={() => setPending(null)}
              onConfirm={() => setPending(null)}
            />
          )}
        </>
      )}
    </div>
  );
}
export const InspectorActions = memo(Inner);
export default InspectorActions;
