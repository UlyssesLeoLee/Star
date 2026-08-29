"use client";

import { useState } from "react";
import { Search, Bell, ChevronDown, Building2 } from "lucide-react";
import { UserMenu } from "@/components/UserMenu";

export function Topbar() {
  const [q, setQ] = useState("");
  return (
    <header className="border-b border-line bg-bg-soft/30 backdrop-blur sticky top-0 z-10">
      <div className="flex items-center gap-3 px-6 py-2.5">
        <div className="flex items-center gap-2 text-sm">
          <Building2 size={14} className="text-ink-mute" />
          <span className="text-ink-dim">ACME Studio</span>
          <ChevronDown size={12} className="text-ink-mute" />
          <span className="text-ink-dim">/</span>
          <span className="text-ink">Physis / GVPE</span>
        </div>
        <div className="flex-1 max-w-xl mx-auto">
          <div className="relative">
            <Search size={14} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-mute" />
            <input
              value={q}
              onChange={(e) => setQ(e.target.value)}
              placeholder="Search work-items, PRs, worktrees, agents... (⌘K)"
              className="w-full rounded-md border border-line bg-bg-card pl-8 pr-3 py-1.5 text-sm placeholder:text-ink-mute focus:outline-none focus:border-accent"
            />
          </div>
        </div>
        <div className="flex items-center gap-3 text-sm">
          <button className="btn relative" aria-label="notifications">
            <Bell size={14} />
            <span className="absolute -top-1 -right-1 size-4 rounded-full bg-err text-white text-[10px] grid place-items-center">3</span>
          </button>
          <UserMenu />
        </div>
      </div>
    </header>
  );
}
