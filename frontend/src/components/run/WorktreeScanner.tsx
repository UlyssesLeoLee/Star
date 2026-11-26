/*
@cypher schema=1 source_sha256=placeholder
@endcypher
*/
// WorktreeScanner — 本地 Worktree 扫描器 (2026-11-27)
// 扫描本地文件系统的 Git worktree 目录，供 Lane 挂载使用
// 支持手动输入路径和自动扫描两种方式
"use client";

import { clsx } from "clsx";
import {
  ScanEye,
  FolderOpen,
  Check,
  Plus,
  Loader2,
  X,
  AlertTriangle,
} from "lucide-react";
import { useState, useCallback } from "react";

export interface ScannedWorktree {
  path: string;
  name: string;
  branch: string;
  isCurrent: boolean;
  hasChanges: boolean;
}

export interface WorktreeScannerProps {
  projectId: string;
  laneId: string;
  onAttach: (worktreePath: string) => void;
  onClose: () => void;
  className?: string;
}

export function WorktreeScanner({
  projectId,
  laneId,
  onAttach,
  onClose,
  className,
}: WorktreeScannerProps) {
  const [scanPath, setScanPath] = useState("");
  const [scanning, setScanning] = useState(false);
  const [results, setResults] = useState<ScannedWorktree[]>([]);
  const [error, setError] = useState<string | null>(null);

  const performScan = useCallback(async () => {
    setScanning(true);
    setError(null);
    try {
      // 调用本地扫描 API (通过 Next.js API route 代理到本地文件系统)
      const res = await fetch("/api/worktrees/scan", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ path: scanPath || undefined, project_id: projectId }),
      });
      if (!res.ok) {
        const err = await res.json();
        throw new Error(err.message || `扫描失败 (HTTP ${res.status})`);
      }
      const data = await res.json();
      setResults(data.worktrees || []);
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "扫描失败");
      setResults([]);
    } finally {
      setScanning(false);
    }
  }, [scanPath, projectId]);

  // 快速预设路径
  const presets = [
    { label: "D:/Star", path: "D:/Star" },
    { label: "E:/Work", path: "E:/Work" },
    { label: "当前项目", path: "" },
  ];

  return (
    <div className={clsx("rounded-lg border-2 border-[var(--cel-ink,var(--color-border))] bg-[var(--cel-surface-card,var(--color-surface-2))] p-4 shadow-xl", className)}>
      <div className="flex items-center justify-between mb-3">
        <h3 className="flex items-center gap-1.5 text-sm font-bold">
          <ScanEye size={16} className="text-[var(--cel-cyan,#00f0ff)]" />
          扫描本地 Worktree
        </h3>
        <button type="button" onClick={onClose} className="text-ink-mute hover:text-ink p-1">
          <X size={14} />
        </button>
      </div>

      <p className="text-[10px] font-mono text-ink-mute mb-3">
        扫描本地 Git worktree 目录，将找到的 Worktree 挂载到当前 Lane
      </p>

      {/* 扫描路径输入 */}
      <div className="flex gap-1.5 mb-2">
        <div className="relative flex-1">
          <FolderOpen size={13} className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-mute" />
          <input
            type="text"
            value={scanPath}
            onChange={(e) => setScanPath(e.target.value)}
            placeholder="D:/Star 或留空自动检测"
            className="w-full rounded border border-line bg-bg px-2 py-1.5 pl-9 text-xs font-mono placeholder:text-ink-mute/50 focus:border-accent focus:outline-none"
          />
        </div>
        <button
          type="button"
          onClick={performScan}
          disabled={scanning}
          className="flex items-center gap-1 rounded border border-line bg-[var(--cel-cyan,#00f0ff)]/10 px-3 py-1.5 text-xs font-mono font-bold text-[var(--cel-cyan,#00f0ff)] hover:bg-[var(--cel-cyan,#00f0ff)]/20 disabled:opacity-50"
        >
          {scanning ? <Loader2 size={12} className="animate-spin" /> : <ScanEye size={12} />}
          {scanning ? "扫描中" : "扫描"}
        </button>
      </div>

      {/* 预设路径 */}
      <div className="flex flex-wrap gap-1 mb-3">
        {presets.map((preset) => (
          <button
            key={preset.path}
            type="button"
            onClick={() => setScanPath(preset.path)}
            className={clsx(
              "rounded border px-2 py-0.5 text-[10px] font-mono transition-colors",
              scanPath === preset.path
                ? "border-[var(--cel-cyan,#00f0ff)]/50 bg-[var(--cel-cyan,#00f0ff)]/10 text-[var(--cel-cyan,#00f0ff)]"
                : "border-line text-ink-mute hover:border-line/80",
            )}
          >
            {preset.label}
          </button>
        ))}
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="rounded border border-err/30 bg-err/5 p-2.5 mb-3 flex items-start gap-2">
          <AlertTriangle size={13} className="text-err shrink-0 mt-0.5" />
          <p className="text-[10px] text-err">{error}</p>
        </div>
      )}

      {/* 扫描结果 */}
      {results.length > 0 && (
        <div className="space-y-1 max-h-60 overflow-y-auto">
          {results.map((wt) => (
            <div
              key={wt.path}
              className="flex items-center rounded border border-line bg-bg p-2.5 gap-2"
            >
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-1.5">
                  <span className="block truncate text-xs font-mono font-bold">{wt.name}</span>
                  {wt.isCurrent && (
                    <span className="shrink-0 rounded bg-ok/10 px-1 text-[9px] font-mono text-ok">当前</span>
                  )}
                  {wt.hasChanges && (
                    <span className="shrink-0 rounded bg-warn/10 px-1 text-[9px] font-mono text-warn">有改动</span>
                  )}
                </div>
                <span className="block truncate text-[10px] text-ink-mute mt-0.5">
                  {wt.path} · {wt.branch}
                </span>
              </div>
              <button
                type="button"
                onClick={() => onAttach(wt.path)}
                className="shrink-0 flex items-center gap-1 rounded border border-line px-2 py-1 text-[10px] font-mono hover:border-accent hover:text-accent"
              >
                <Plus size={10} /> 挂载
              </button>
            </div>
          ))}
        </div>
      )}

      {!scanning && !error && results.length === 0 && scanPath && (
        <p className="text-[10px] text-ink-mute py-2">未找到 Worktree，点击「扫描」开始检测</p>
      )}
    </div>
  );
}