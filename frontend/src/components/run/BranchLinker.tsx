/*
@cypher schema=1 source_sha256=placeholder
@endcypher
*/
// BranchLinker — GitHub 分支链接器 (2026-11-27)
// 从 GitHub 仓库获取分支列表，选择并链接到当前 Lane
"use client";

import { clsx } from "clsx";
import {
  GitBranch,
  GitFork,
  Search,
  Check,
  Loader2,
  X,
  ExternalLink,
  AlertTriangle,
} from "lucide-react";
import { useState, useCallback } from "react";

export interface GitHubBranch {
  name: string;
  sha: string;
  isProtected: boolean;
  author?: string;
  updatedAt?: string;
}

export interface BranchLinkerProps {
  projectId: string;
  laneId: string;
  onLink: (branchName: string, repoUrl: string) => void;
  onClose: () => void;
  className?: string;
}

export function BranchLinker({
  projectId,
  laneId,
  onLink,
  onClose,
  className,
}: BranchLinkerProps) {
  const [repoUrl, setRepoUrl] = useState("");
  const [searchQuery, setSearchQuery] = useState("");
  const [loading, setLoading] = useState(false);
  const [branches, setBranches] = useState<GitHubBranch[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [selectedBranch, setSelectedBranch] = useState<string | null>(null);
  const [linking, setLinking] = useState(false);

  const fetchBranches = useCallback(async () => {
    if (!repoUrl.trim()) return;
    setLoading(true);
    setError(null);
    try {
      const res = await fetch("/api/branches/github", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ repo_url: repoUrl.trim(), project_id: projectId }),
      });
      if (!res.ok) {
        const err = await res.json();
        throw new Error(err.message || `获取分支失败 (HTTP ${res.status})`);
      }
      const data = await res.json();
      setBranches(data.branches || []);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "获取分支失败");
      setBranches([]);
    } finally {
      setLoading(false);
    }
  }, [repoUrl, projectId]);

  const handleLink = useCallback(async () => {
    if (!selectedBranch) return;
    setLinking(true);
    try {
      onLink(selectedBranch, repoUrl.trim());
    } finally {
      setLinking(false);
    }
  }, [selectedBranch, repoUrl, onLink]);

  const filteredBranches = branches.filter((b) =>
    !searchQuery || b.name.toLowerCase().includes(searchQuery.toLowerCase()),
  );

  return (
    <div className={clsx("rounded-lg border-2 border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-card,var(--color-surface-2))] p-4 shadow-xl", className)}>
      <div className="flex items-center justify-between mb-3">
        <h3 className="flex items-center gap-1.5 text-sm font-bold">
          <GitFork size={16} className="text-[var(--cel-cyan,#00f0ff)]" />
          链接 GitHub Branch
        </h3>
        <button type="button" onClick={onClose} className="text-ink-mute hover:text-ink p-1">
          <X size={14} />
        </button>
      </div>

      <p className="text-[10px] font-mono text-ink-mute mb-3">
        输入 GitHub 仓库 URL，获取分支列表并链接到当前 Lane
      </p>

      {/* 仓库 URL 输入 */}
      <div className="flex gap-1.5 mb-3">
        <div className="relative flex-1">
          <GitFork size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-mute" />
          <input
            type="text"
            value={repoUrl}
            onChange={(e) => setRepoUrl(e.target.value)}
            placeholder="https://github.com/user/repo.git"
            className="w-full rounded border border-line bg-bg px-2 py-1.5 pl-9 text-xs font-mono placeholder:text-ink-mute/50 focus:border-accent focus:outline-none"
          />
        </div>
        <button
          type="button"
          onClick={fetchBranches}
          disabled={loading || !repoUrl.trim()}
          className="flex items-center gap-1 rounded border border-line bg-[var(--cel-cyan,#00f0ff)]/10 px-3 py-1.5 text-xs font-mono font-bold text-[var(--cel-cyan,#00f0ff)] hover:bg-[var(--cel-cyan,#00f0ff)]/20 disabled:opacity-50"
        >
          {loading ? <Loader2 size={12} className="animate-spin" /> : <Search size={12} />}
          {loading ? "获取中" : "获取"}
        </button>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="rounded border border-err/30 bg-err/5 p-2.5 mb-3 flex items-start gap-2">
          <AlertTriangle size={13} className="text-err shrink-0 mt-0.5" />
          <p className="text-[10px] text-err">{error}</p>
        </div>
      )}

      {/* 搜索分支 */}
      {branches.length > 0 && (
        <>
          <div className="relative mb-2">
            <Search size={12} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-mute" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索分支名..."
              className="w-full rounded border border-line bg-bg px-2 py-1.5 pl-8 text-[10px] font-mono placeholder:text-ink-mute/50 focus:border-accent focus:outline-none"
            />
          </div>

          <div className="space-y-1 max-h-48 overflow-y-auto mb-3">
            {filteredBranches.map((branch) => (
              <button
                key={branch.sha}
                type="button"
                onClick={() => setSelectedBranch(branch.name)}
                className={clsx(
                  "flex w-full items-center rounded border px-2.5 py-1.5 text-left text-xs transition-colors",
                  selectedBranch === branch.name
                    ? "border-accent/50 bg-accent/10 text-accent"
                    : "border-line hover:border-line/80",
                )}
              >
                <span className="flex-1 font-mono truncate">{branch.name}</span>
                {branch.isProtected && (
                  <span className="shrink-0 rounded bg-warn/10 px-1 text-[9px] font-mono text-warn ml-1.5">保护</span>
                )}
                {selectedBranch === branch.name && (
                  <Check size={12} className="shrink-0 text-accent ml-1.5" />
                )}
              </button>
            ))}
          </div>

          {/* 链接按钮 */}
          <button
            type="button"
            onClick={handleLink}
            disabled={!selectedBranch || linking}
            className="flex w-full items-center justify-center gap-1.5 rounded border-2 border-[var(--cel-cyan,#00f0ff)] bg-[var(--cel-cyan,#00f0ff)]/10 px-3 py-2 text-sm font-bold text-[var(--cel-cyan,#00f0ff)] hover:bg-[var(--cel-cyan,#00f0ff)]/20 disabled:opacity-50 transition-colors"
          >
            {linking ? <Loader2 size={14} className="animate-spin" /> : <GitBranch size={14} />}
            {linking ? "链接中..." : selectedBranch ? `链接 ${selectedBranch}` : "选择一个分支"}
          </button>
        </>
      )}

      {!loading && !error && branches.length === 0 && repoUrl.trim() && (
        <p className="text-[10px] text-ink-mute py-2">
          输入 GitHub 仓库 URL 后点击「获取」查看分支列表
        </p>
      )}
    </div>
  );
}