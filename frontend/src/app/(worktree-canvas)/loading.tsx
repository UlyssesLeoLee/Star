// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/(worktree-canvas)/loading.tsx
// Per Next.js 14 App Router convention.
export default function Loading() {
  return (
    <div data-testid="worktree-canvas-loading" className="flex h-full items-center justify-center">
      <div className="flex flex-col items-center gap-2 text-sm text-slate-500">
        <div className="h-8 w-8 animate-spin rounded-full border-2 border-blue-500 border-t-transparent" />
        <span>加载 Worktree Graph...</span>
      </div>
    </div>
  );
}
