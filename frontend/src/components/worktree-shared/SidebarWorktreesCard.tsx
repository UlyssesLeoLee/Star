"use client";

// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-shared/SidebarWorktreesCard.tsx
//
// ULYS-228 (ULYS-218.1) FR-ORCA-011 AC-2 sidebar "hidden worktrees" card.
//
// Per docs/ecosystem-survey/orca-design-survey.md §3 FR-ORCA-011:
//   AC-1: external git worktree 默认 hidden (UI sidebar)
//   AC-2: sidebar 显示 "hidden worktrees" card 提示
//   AC-3: CLI `git worktree remove` 后, Orca 下次刷新清理自身状态
//
// 实现: 顶部 "Hidden worktrees (N)" card → 折叠/展开 → 列出 scan 返回的
// non-managed ExternalWorktreeDto[] → 每行一个 "Import" 按钮 → 调
// `POST /api/v1/worktrees/external-import` 进入 Provisioning → Running 状态机.
//
// 依赖 backend trait `RealExternalWorktreeImport` (per
// `crates/worktree-shared-dir/src/external_worktree_import.rs`, ship in PR #107).
// REST endpoint `/api/v1/worktrees/external-import` 是 followup (per brief §4 注:
// 本期可只做 UI + MSW mock); UI 走 MSW handler, 真 endpoint ship 后自动透传.

import * as React from "react";
import { useCallback, useEffect, useState } from "react";
import {
  type ExternalImportResponse,
  type ExternalWorktreeDto,
} from "./startFromPickerTypes";

interface SidebarWorktreesCardProps {
  /** Repo UUID (经 BFF 调 backend). */
  repoId: string;
  /** 测试用: 初始 scan 完的回调 (e.g. 给 sidebar 折叠态显示 N 计数 chip). */
  onCount?: (count: number) => void;
}

interface ScanState {
  status: "idle" | "loading" | "ready" | "error";
  items: ExternalWorktreeDto[];
  message: string | null;
}

interface ImportState {
  /** path → "importing" | "done" | "error" + message */
  byPath: Record<string, { status: "importing" | "done" | "error"; message: string | null }>;
}

const SHORT_SHA_LEN = 7;

function shortSha(sha: string): string {
  return sha.length <= SHORT_SHA_LEN ? sha : sha.slice(0, SHORT_SHA_LEN);
}

export function SidebarWorktreesCard({ repoId, onCount }: SidebarWorktreesCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [scan, setScan] = useState<ScanState>({
    status: "idle",
    items: [],
    message: null,
  });
  const [importState, setImportState] = useState<ImportState>({ byPath: {} });

  const load = useCallback(async () => {
    setScan((s) => ({ ...s, status: "loading", message: null }));
    try {
      const resp = await fetch(
        `/api/v1/worktrees/external-import/scan?repo_id=${encodeURIComponent(repoId)}`,
        { method: "GET" },
      );
      if (!resp.ok) {
        setScan({
          status: "error",
          items: [],
          message: `scan HTTP ${resp.status}`,
        });
        return;
      }
      const body = (await resp.json()) as { items: ExternalWorktreeDto[] };
      const items = (body.items ?? []).filter((w) => !w.is_managed);
      setScan({ status: "ready", items, message: null });
      onCount?.(items.length);
    } catch (err: unknown) {
      setScan({
        status: "error",
        items: [],
        message: err instanceof Error ? err.message : "scan failed",
      });
    }
  }, [repoId, onCount]);

  useEffect(() => {
    void load();
  }, [load]);

  const handleImport = useCallback(
    async (path: string) => {
      setImportState((s) => ({
        byPath: { ...s.byPath, [path]: { status: "importing", message: null } },
      }));
      try {
        const resp = await fetch("/api/v1/worktrees/external-import", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ repo_id: repoId, path }),
        });
        if (!resp.ok) {
          setImportState((s) => ({
            byPath: {
              ...s.byPath,
              [path]: { status: "error", message: `HTTP ${resp.status}` },
            },
          }));
          return;
        }
        const body = (await resp.json()) as ExternalImportResponse;
        setImportState((s) => ({
          byPath: {
            ...s.byPath,
            [path]: { status: "done", message: body.state ?? null },
          },
        }));
        // 成功后从 list 移除 (已 managed) 并刷新计数
        await load();
      } catch (err: unknown) {
        setImportState((s) => ({
          byPath: {
            ...s.byPath,
            [path]: {
              status: "error",
              message: err instanceof Error ? err.message : "import failed",
            },
          },
        }));
      }
    },
    [repoId, load],
  );

  const hidden = scan.items;

  return (
    <div
      data-testid="sidebar-worktrees-card"
      data-repo-id={repoId}
      data-expanded={expanded}
      className="rounded border border-slate-200 bg-white text-xs dark:border-slate-700 dark:bg-slate-900"
    >
      <button
        type="button"
        data-testid="sidebar-worktrees-toggle"
        onClick={() => setExpanded((v) => !v)}
        aria-expanded={expanded}
        className="flex w-full items-center justify-between gap-2 px-3 py-2 text-left hover:bg-slate-50 dark:hover:bg-slate-800"
      >
        <span className="font-medium text-slate-700 dark:text-slate-200">
          Hidden worktrees ({scan.status === "ready" ? hidden.length : "…"})
        </span>
        <span aria-hidden="true" className="text-slate-500">
          {expanded ? "▾" : "▸"}
        </span>
      </button>

      {expanded && (
        <div
          data-testid="sidebar-worktrees-body"
          className="border-t border-slate-200 px-3 py-2 dark:border-slate-700"
        >
          {scan.status === "loading" && (
            <p className="text-slate-500">Scanning…</p>
          )}
          {scan.status === "error" && (
            <p
              data-testid="sidebar-worktrees-error"
              className="text-amber-700 dark:text-amber-400"
            >
              scan failed: {scan.message ?? "unknown"}
            </p>
          )}
          {scan.status === "ready" && hidden.length === 0 && (
            <p
              data-testid="sidebar-worktrees-empty"
              className="text-slate-500"
            >
              无 hidden worktree (所有 external git worktree 已 import).
            </p>
          )}
          {scan.status === "ready" && hidden.length > 0 && (
            <ul className="space-y-2">
              {hidden.map((w) => {
                const state = importState.byPath[w.path];
                const path = w.path;
                return (
                  <li
                    key={path}
                    data-testid="sidebar-worktrees-row"
                    data-path={path}
                    className="rounded border border-slate-100 px-2 py-1 dark:border-slate-700"
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span
                        className="truncate font-mono text-[11px] text-slate-700 dark:text-slate-300"
                        title={path}
                      >
                        {path.split(/[\\/]/).slice(-2).join("/")}
                      </span>
                      <button
                        type="button"
                        data-testid="sidebar-worktrees-import"
                        onClick={() => void handleImport(path)}
                        disabled={state?.status === "importing"}
                        className="rounded bg-blue-600 px-2 py-0.5 text-[11px] text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-slate-400"
                      >
                        {state?.status === "importing"
                          ? "Importing…"
                          : state?.status === "done"
                            ? "Imported"
                            : state?.status === "error"
                              ? "Retry"
                              : "Import"}
                      </button>
                    </div>
                    <div className="mt-1 flex items-center justify-between text-[10px] text-slate-500">
                      <span>
                        branch:{" "}
                        <span className="font-mono">
                          {w.branch ?? "(detached)"}
                        </span>
                      </span>
                      <span className="font-mono">{shortSha(w.head_commit)}</span>
                    </div>
                    {state?.status === "error" && (
                      <p
                        data-testid="sidebar-worktrees-row-error"
                        className="mt-1 text-[10px] text-amber-700 dark:text-amber-400"
                      >
                        {state.message ?? "import failed"}
                      </p>
                    )}
                    {state?.status === "done" && (
                      <p
                        data-testid="sidebar-worktrees-row-done"
                        className="mt-1 text-[10px] text-emerald-700 dark:text-emerald-400"
                      >
                        进入 Provisioning 状态.
                      </p>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
        </div>
      )}
    </div>
  );
}