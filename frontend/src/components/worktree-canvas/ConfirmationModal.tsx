"use client";

import * as React from "react";
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/ConfirmationModal.tsx
// Destructive action confirm modal (per INV-WC-09 + DD §20.2).
"use client";
import { ACTION_DANGER, type WorktreeAction } from "./types";

interface Props {
  action: WorktreeAction;
  worktreeId: string;
  onClose: () => void;
  onConfirm: () => void;
}

export function ConfirmationModal({ action, worktreeId, onClose, onConfirm }: Props) {
  if (ACTION_DANGER[action] !== "Destructive") return null;
  return (
    <div
      role="dialog"
      aria-modal="true"
      data-testid="confirmation-modal"
      data-action={action}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/40"
      onClick={onClose}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-96 max-w-[90vw] rounded-lg bg-white p-4 shadow-lg dark:bg-slate-900"
      >
        <h3 className="mb-2 text-base font-semibold text-red-600">
          ⚠️ Confirm Destructive Action
        </h3>
        <p className="mb-3 text-sm text-slate-600 dark:text-slate-300">
          Action <code className="rounded bg-slate-100 px-1 font-mono">{action}</code>{" "}
          on worktree <code className="font-mono text-xs">{worktreeId.slice(0, 8)}</code>
          will modify refs/heads/main or similar. This may be irreversible.
        </p>
        <p className="mb-3 text-xs text-slate-500">
          affected_count: 1, affected_paths: [&quot;refs/heads/main&quot;],
          reversible: false, estimated_duration: ~5s
        </p>
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onClose}
            className="rounded border border-slate-200 px-3 py-1 text-sm hover:bg-slate-50 dark:border-slate-700"
          >
            Cancel
          </button>
          <button
            type="button"
            data-testid="confirm-button"
            onClick={onConfirm}
            className="rounded bg-red-600 px-3 py-1 text-sm text-white hover:bg-red-700"
          >
            {action}
          </button>
        </div>
      </div>
    </div>
  );
}
