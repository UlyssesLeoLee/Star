// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/SearchBar.tsx
// DSL search bar (per spec §20 + IMPL-PLAN §4.14.7).
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const query = useWorktreeCanvasStore((s) => s.searchQuery);
  const setQuery = useWorktreeCanvasStore((s) => s.setSearchQuery);
  return (
    <div className="flex items-center gap-2 border-b border-slate-200 px-3 py-2 dark:border-slate-700">
      <span className="text-xs text-slate-400">🔍</span>
      <input
        type="text"
        data-testid="search-bar"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="show:conflict behind:>20 agent:codex"
        className="flex-1 bg-transparent text-sm outline-none placeholder:text-slate-400"
      />
      {query && (
        <button
          type="button"
          onClick={() => setQuery("")}
          className="text-xs text-slate-400 hover:text-slate-600"
          aria-label="清除"
        >
          ✕
        </button>
      )}
    </div>
  );
}
export const SearchBar = memo(Inner);
export default SearchBar;
