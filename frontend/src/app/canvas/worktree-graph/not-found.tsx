// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/(worktree-canvas)/not-found.tsx
import Link from "next/link";

export default function NotFound() {
  return (
    <div data-testid="worktree-canvas-not-found" className="flex h-full flex-col items-center justify-center gap-3 p-4 text-sm">
      <h2 className="text-base font-semibold">404</h2>
      <p className="text-slate-500">Worktree Canvas route not found</p>
      <Link href="/worktree-canvas" className="text-blue-500 hover:underline">
        返回画布
      </Link>
    </div>
  );
}
