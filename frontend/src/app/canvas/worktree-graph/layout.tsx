// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/(worktree-canvas)/layout.tsx
// AI Worktree Graph Canvas route segment layout (ULYS-57.4 T14.1).
// 独立 route segment, 0 改 V0.1 (app/(app) layout, 守门 #19 v19).
import type { ReactNode } from "react";

export default function WorktreeCanvasLayout({ children }: { children: ReactNode }) {
  return (
    <div data-testid="worktree-canvas-layout" className="flex h-screen flex-col">
      {children}
    </div>
  );
}
