"use client";

// Star Frontend — 任务窗口中心页面
// Per 2026-08-29 04:09 JST 上轮拍板: 新页面, 每 worktree 多 CLI session, 三触发上传

import { useState } from "react";
import { GitBranch, Plus, RefreshCw, Settings, Sparkles } from "lucide-react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { WindowsTabBar, mockTabs, type CliTab } from "@/components/agent-windows/WindowsTabBar";
import { CliTerminal } from "@/components/agent-windows/CliTerminal";
import { NewTabModal } from "@/components/agent-windows/NewTabModal";
import { useStore } from "@/lib/store";

export default function AgentWindowsPage() {
  const [tabs, setTabs] = useState<CliTab[]>(mockTabs());
  const [activeTabId, setActiveTabId] = useState<string | null>("t1");
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedWorktree, setSelectedWorktree] = useState("wt-physis-gvpe");

  const activeTab = tabs.find((t) => t.id === activeTabId) || null;

  const handleRun = (prompt: string) => {
    if (!activeTab) return;
    setTabs((prev) =>
      prev.map((t) =>
        t.id === activeTab.id
          ? { ...t, state: "running", lastOutput: (t.lastOutput || "") + `\n\n$ ${prompt}\n> [mock] running...` }
          : t
      )
    );
  };

  const handleCancel = () => {
    if (!activeTab) return;
    setTabs((prev) =>
      prev.map((t) =>
        t.id === activeTab.id
          ? { ...t, state: "aborted", lastOutput: (t.lastOutput || "") + "\n✗ cancelled by user" }
          : t
      )
    );
  };

  const handleUpload = () => {
    if (!activeTab) return;
    setTabs((prev) =>
      prev.map((t) =>
        t.id === activeTab.id
          ? { ...t, lastOutput: (t.lastOutput || "") + `\n✓ uploaded ${t.filesChanged || 0} files to worktree` }
          : t
      )
    );
  };

  const handleClear = () => {
    if (!activeTab) return;
    setTabs((prev) => prev.map((t) => t.id === activeTab.id ? { ...t, lastOutput: "" } : t));
  };

  const handleNewTab = (data: { profileId: string; profileName: string; kind: "cli" | "api"; label: string }) => {
    const id = `t${Date.now()}`;
    setTabs((prev) => [
      ...prev,
      {
        id,
        label: data.label,
        state: "created",
        kind: data.kind,
        profileName: data.profileName,
        lastOutput: "",
      },
    ]);
    setActiveTabId(id);
    setModalOpen(false);
  };

  return (
    <div className="flex flex-col h-[calc(100vh-100px)]">
      <PageHeader
        title="Agent Windows"
        description="多 CLI Agent 并行任务窗口 · 三触发上传到 Worktree"
        actions={
          <div className="flex items-center gap-2">
            <select
              value={selectedWorktree}
              onChange={(e) => setSelectedWorktree(e.target.value)}
              className="text-xs px-2 py-1 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)]"
              aria-label="worktree"
            >
              <option value="wt-physis-gvpe">Physis / GVPE</option>
              <option value="wt-saga-bench">Saga Bench</option>
              <option value="wt-local-runtime">local-runtime</option>
            </select>
            <button
              className="text-xs px-2 py-1 rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface-2)] flex items-center gap-1"
              aria-label="refresh"
            >
              <RefreshCw size={12} /> 刷新
            </button>
            <a
              href="/settings/cli-profiles"
              className="text-xs px-2 py-1 rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface-2)] flex items-center gap-1"
            >
              <Settings size={12} /> CLI Profiles
            </a>
          </div>
        }
      />

      {/* 状态条 */}
      <div className="flex items-center gap-4 px-6 py-2 text-xs text-[color:var(--color-text-dim)] border-b border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
        <Stat label="Tabs" value={tabs.length} />
        <Stat label="Running" value={tabs.filter((t) => t.state === "running").length} accent="primary" />
        <Stat label="Files" value={tabs.reduce((s, t) => s + (t.filesChanged || 0), 0)} accent="success" />
        <Stat label="Worktree" value={selectedWorktree} icon={GitBranch} />
        <div className="ml-auto text-[10px]">
          Cmd+Shift+T 切换主题 · Cmd+K 搜索
        </div>
      </div>

      {/* Tab Bar */}
      <WindowsTabBar
        tabs={tabs}
        activeTabId={activeTabId}
        onTabSelect={setActiveTabId}
        onTabClose={(id) => {
          setTabs((prev) => prev.filter((t) => t.id !== id));
          if (activeTabId === id) setActiveTabId(tabs[0]?.id || null);
        }}
        onNewTab={() => setModalOpen(true)}
      />

      {/* 终端 */}
      <CliTerminal
        tab={activeTab}
        onRun={handleRun}
        onCancel={handleCancel}
        onUpload={handleUpload}
        onClear={handleClear}
      />

      {/* 新 Tab Modal */}
      {modalOpen && <NewTabModal onClose={() => setModalOpen(false)} onCreate={handleNewTab} />}
    </div>
  );
}
