// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/(worktree-canvas)/page.tsx
//
// AI Worktree Graph Canvas — 主页面入口 (ULYS-57.4 T14.1 + IMPL-PLAN §4.14.1).
//
// 3 区域布局:
// - 顶部 toolbar (View Mode + Zoom Level + Search + Focus + AI Dialog)
// - 主画布 (View Mode + Zoom Level 联动)
// - 右侧 Inspector (11 Tab 切换)

"use client";

import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";
import { ALL_VIEW_MODES, ALL_ZOOM_LEVELS, ALL_INSPECTOR_TABS } from "@/components/worktree-canvas/types";
import { TreeView } from "@/components/worktree-canvas/TreeView";
import { DependencyView } from "@/components/worktree-canvas/DependencyView";
import { RiskView } from "@/components/worktree-canvas/RiskView";
import { AgentView } from "@/components/worktree-canvas/AgentView";
import { HistoryView } from "@/components/worktree-canvas/HistoryView";
import { ZoomL0 } from "@/components/worktree-canvas/ZoomL0";
import { ZoomL1 } from "@/components/worktree-canvas/ZoomL1";
import { ZoomL2 } from "@/components/worktree-canvas/ZoomL2";
import { ZoomL3 } from "@/components/worktree-canvas/ZoomL3";
import { ZoomL4 } from "@/components/worktree-canvas/ZoomL4";
import { ZoomL5 } from "@/components/worktree-canvas/ZoomL5";
import { InspectorOverview } from "@/components/worktree-canvas/InspectorOverview";
import { InspectorGitState } from "@/components/worktree-canvas/InspectorGitState";
import { InspectorTask } from "@/components/worktree-canvas/InspectorTask";
import { InspectorAgent } from "@/components/worktree-canvas/InspectorAgent";
import { InspectorRisk } from "@/components/worktree-canvas/InspectorRisk";
import { InspectorTests } from "@/components/worktree-canvas/InspectorTests";
import { InspectorFiles } from "@/components/worktree-canvas/InspectorFiles";
import { InspectorCommits } from "@/components/worktree-canvas/InspectorCommits";
import { InspectorRelations } from "@/components/worktree-canvas/InspectorRelations";
import { InspectorHistory } from "@/components/worktree-canvas/InspectorHistory";
import { InspectorActions } from "@/components/worktree-canvas/InspectorActions";
import { SearchBar } from "@/components/worktree-canvas/SearchBar";
import { Minimap } from "@/components/worktree-canvas/Minimap";
import { FocusMode } from "@/components/worktree-canvas/FocusMode";
import { AIDialog } from "@/components/worktree-canvas/AIDialog";

function CanvasView() {
  const viewMode = useWorktreeCanvasStore((s) => s.viewMode);
  const zoomLevel = useWorktreeCanvasStore((s) => s.zoomLevel);
  switch (viewMode) {
    case "TREE": return <TreeView />;
    case "DEPENDENCY": return <DependencyView />;
    case "RISK": return <RiskView />;
    case "AGENT": return <AgentView />;
    case "HISTORY": return <HistoryView />;
  }
}

function ZoomPanel() {
  const zoomLevel = useWorktreeCanvasStore((s) => s.zoomLevel);
  switch (zoomLevel) {
    case "L0": return <ZoomL0 />;
    case "L1": return <ZoomL1 />;
    case "L2": return <ZoomL2 />;
    case "L3": return <ZoomL3 />;
    case "L4": return <ZoomL4 />;
    case "L5": return <ZoomL5 />;
  }
}

function InspectorPanel() {
  const tab = useWorktreeCanvasStore((s) => s.inspectorTab);
  switch (tab) {
    case "Overview": return <InspectorOverview />;
    case "GitState": return <InspectorGitState />;
    case "Task": return <InspectorTask />;
    case "Agent": return <InspectorAgent />;
    case "Risk": return <InspectorRisk />;
    case "Tests": return <InspectorTests />;
    case "Files": return <InspectorFiles />;
    case "Commits": return <InspectorCommits />;
    case "Relations": return <InspectorRelations />;
    case "History": return <InspectorHistory />;
    case "Actions": return <InspectorActions />;
  }
}

export default function WorktreeCanvasPage() {
  const viewMode = useWorktreeCanvasStore((s) => s.viewMode);
  const setViewMode = useWorktreeCanvasStore((s) => s.setViewMode);
  const zoomLevel = useWorktreeCanvasStore((s) => s.zoomLevel);
  const setZoomLevel = useWorktreeCanvasStore((s) => s.setZoomLevel);
  const inspectorTab = useWorktreeCanvasStore((s) => s.inspectorTab);
  const setInspectorTab = useWorktreeCanvasStore((s) => s.setInspectorTab);

  return (
    <div data-testid="worktree-canvas-page" className="flex h-full flex-col">
      <header className="flex items-center gap-2 border-b border-slate-200 px-3 py-2 text-xs dark:border-slate-700">
        <span className="font-bold">Worktree Canvas</span>
        <span className="text-slate-400">/</span>
        <select
          data-testid="view-mode-select"
          value={viewMode}
          onChange={(e) => setViewMode(e.target.value as typeof viewMode)}
          className="rounded border border-slate-200 bg-transparent px-2 py-0.5 dark:border-slate-700"
        >
          {ALL_VIEW_MODES.map((m) => (
            <option key={m} value={m}>{m}</option>
          ))}
        </select>
        <span className="text-slate-400">·</span>
        <select
          data-testid="zoom-level-select"
          value={zoomLevel}
          onChange={(e) => setZoomLevel(e.target.value as typeof zoomLevel)}
          className="rounded border border-slate-200 bg-transparent px-2 py-0.5 dark:border-slate-700"
        >
          {ALL_ZOOM_LEVELS.map((z) => (
            <option key={z} value={z}>{z}</option>
          ))}
        </select>
      </header>

      <FocusMode />
      <SearchBar />

      <div className="flex flex-1 overflow-hidden">
        <main className="relative flex-1 overflow-hidden">
          <CanvasView />
          <Minimap />
        </main>

        <aside
          data-testid="inspector-panel"
          className="w-80 overflow-y-auto border-l border-slate-200 dark:border-slate-700"
        >
          <div className="flex flex-wrap gap-1 border-b border-slate-200 p-2 dark:border-slate-700">
            {ALL_INSPECTOR_TABS.map((t) => (
              <button
                key={t}
                type="button"
                data-testid={`inspector-tab-${t}`}
                data-active={inspectorTab === t}
                onClick={() => setInspectorTab(t)}
                className={`rounded px-2 py-0.5 text-[10px] ${
                  inspectorTab === t
                    ? "bg-blue-500 text-white"
                    : "bg-slate-100 text-slate-700 hover:bg-slate-200 dark:bg-slate-800 dark:text-slate-300"
                }`}
              >
                {t}
              </button>
            ))}
          </div>
          <InspectorPanel />
          <div className="border-t border-slate-200 p-2 dark:border-slate-700">
            <AIDialog />
          </div>
        </aside>
      </div>
    </div>
  );
}
