"use client";

// Star Frontend — 任务窗口多 CLI Tab Bar
// Per 2026-08-29 04:09 JST 上轮拍板: 每 worktree 多 CLI session, Tab 切换

import { useState } from "react";
import { Plus, X, Circle, Loader2, CheckCircle2, AlertCircle, XCircle, Terminal, Globe } from "lucide-react";

export type TabState = "created" | "running" | "waiting_input" | "completed" | "failed" | "aborted";

export interface CliTab {
  id: string;
  label: string;
  state: TabState;
  kind: "cli" | "api"; // CLI 进程 vs API HTTP (OpenClaw/Hermes)
  profileName: string;
  lastOutput?: string;
  exitCode?: number;
  filesChanged?: number;
}

interface WindowsTabBarProps {
  tabs: CliTab[];
  activeTabId: string | null;
  onTabSelect: (id: string) => void;
  onTabClose: (id: string) => void;
  onNewTab: () => void;
}

const STATE_ICON: Record<TabState, React.ElementType> = {
  created: Circle,
  running: Loader2,
  waiting_input: Circle,
  completed: CheckCircle2,
  failed: AlertCircle,
  aborted: XCircle,
};

const STATE_COLOR: Record<TabState, string> = {
  created: "text-[color:var(--color-neutral)]",
  running: "text-[color:var(--color-primary)] animate-spin",
  waiting_input: "text-[color:var(--color-warning)]",
  completed: "text-[color:var(--color-success)]",
  failed: "text-[color:var(--color-danger)]",
  aborted: "text-[color:var(--color-text-dim)]",
};

export function WindowsTabBar({ tabs, activeTabId, onTabSelect, onTabClose, onNewTab }: WindowsTabBarProps) {
  return (
    <div className="flex items-center border-b border-[color:var(--color-border)] bg-[color:var(--color-surface)] overflow-x-auto">
      {tabs.map((tab) => {
        const isActive = tab.id === activeTabId;
        const Icon = STATE_ICON[tab.state];
        return (
          <div
            key={tab.id}
            onClick={() => onTabSelect(tab.id)}
            className={`group flex items-center gap-2 px-3 py-1.5 border-r border-[color:var(--color-border)] cursor-pointer min-w-[140px] max-w-[240px] ${
              isActive
                ? "bg-[color:var(--color-surface-2)] border-b-2 border-b-[color:var(--color-primary)]"
                : "hover:bg-[color:var(--color-surface-2)]"
            }`}
            role="tab"
            aria-selected={isActive}
          >
            <Icon size={12} className={STATE_COLOR[tab.state]} />
            {tab.kind === "api" ? <Globe size={10} className="opacity-60" /> : <Terminal size={10} className="opacity-60" />}
            <span className="text-xs font-medium truncate flex-1" title={tab.label}>
              {tab.label}
            </span>
            {tab.filesChanged !== undefined && tab.filesChanged > 0 && (
              <span className="text-[9px] px-1 rounded bg-[color:var(--color-success)]/20 text-[color:var(--color-success)]">
                {tab.filesChanged}
              </span>
            )}
            <button
              onClick={(e) => { e.stopPropagation(); onTabClose(tab.id); }}
              className="opacity-0 group-hover:opacity-100 hover:bg-[color:var(--color-danger)]/20 rounded p-0.5"
              aria-label={`close ${tab.label}`}
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
      <button
        onClick={onNewTab}
        className="flex items-center gap-1 px-3 py-1.5 text-xs text-[color:var(--color-text-dim)] hover:text-[color:var(--color-primary)] hover:bg-[color:var(--color-surface-2)]"
        aria-label="new tab"
      >
        <Plus size={12} />
        新 Tab
      </button>
    </div>
  );
}

// 辅助: mock tab state for development
export function mockTabs(): CliTab[] {
  return [
    {
      id: "t1",
      label: "Claude Code",
      state: "running",
      kind: "cli",
      profileName: "Claude Code",
      lastOutput: "$ claude --model claude-3-5-sonnet\n> Reading worktree state...",
      filesChanged: 3,
    },
    {
      id: "t2",
      label: "OpenClaw (gpt-4)",
      state: "completed",
      kind: "api",
      profileName: "OpenClaw",
      lastOutput: "✓ 3 files generated",
      exitCode: 0,
      filesChanged: 3,
    },
    {
      id: "t3",
      label: "Codex",
      state: "failed",
      kind: "cli",
      profileName: "OpenAI Codex",
      lastOutput: "Error: rate limit exceeded",
      exitCode: 1,
    },
  ];
}
