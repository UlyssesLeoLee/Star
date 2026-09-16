// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/(worktree-canvas)/error.tsx
"use client";
import { useEffect } from "react";

interface Props {
  error: Error & { digest?: string };
  reset: () => void;
}

export default function ErrorPage({ error, reset }: Props) {
  useEffect(() => {
    console.error("[worktree-canvas] error:", error);
  }, [error]);
  return (
    <div data-testid="worktree-canvas-error" className="flex h-full flex-col items-center justify-center gap-3 p-4 text-sm">
      <h2 className="text-base font-semibold text-red-600">⚠ 出错了</h2>
      <p className="max-w-md text-center text-slate-600 dark:text-slate-300">
        {error.message || "Unknown error"}
      </p>
      <button
        type="button"
        onClick={reset}
        className="rounded bg-blue-500 px-3 py-1 text-xs text-white hover:bg-blue-600"
      >
        重试
      </button>
    </div>
  );
}
