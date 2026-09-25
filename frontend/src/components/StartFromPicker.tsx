// =====================================================================
// StartFromPicker.tsx — Start-from Picker 4 选 1 UI (per ULYS-194 §3.3)
//
// 实现 spec FR-ORCA-009 描述的 4 tab UI:
//   1. GitHub (RemoteBranch) — origin/main, origin/HEAD, 远程其它分支
//   2. Existing (LocalBranch) — 已存在 worktree 的 branch
//   3. Local   (RepoBase)    — 用户手动选的本地路径
//   4. Empty   (CommitSha)   — 空白起点 (sentinel)
//
// 数据流:
//   1. mount → fetch `/api/v1/worktree-picker/candidates?repo_id=X`
//   2. 用户点 tab → 切换显示对应分类的候选
//   3. 用户选中候选 → setInternalState(selectedId)
//   4. 用户点 Confirm → onConfirm(pickerCandidate) 回调
//
// 守门:
//   - 与 ULYS-177.2 (create_async + Provisioning) 联动留 P1 followup,
//     本组件只负责"列出 + 选"两步, create_async 触发由 caller 负责.
// =====================================================================

"use client";

import { useEffect, useMemo, useState } from "react";
import clsx from "clsx";

// ---- 类型: 与 backend `worktree-shared-dir::PickerCandidate` 对齐 ----

export type PickerKind =
  | "remote_branch"
  | "local_branch"
  | "repo_base"
  | "commit_sha";

export interface PickerCandidate {
  id: string;
  label: string;
  description: string;
  kind: PickerKind;
}

export interface PickerCandidatesResponse {
  github_branches: PickerCandidate[];
  existing_worktrees: PickerCandidate[];
  local_paths: PickerCandidate[];
  empty: PickerCandidate | null;
}

export type TabId = "github" | "existing" | "local" | "empty";

// ---- 工具: backend kind 字符串 → 前端 enum ----

export function toPickerKind(backend: string): PickerKind | null {
  switch (backend) {
    case "remote_branch":
      return "remote_branch";
    case "local_branch":
      return "local_branch";
    case "repo_base":
      return "repo_base";
    case "commit_sha":
      return "commit_sha";
    default:
      return null;
  }
}

export function kindToTab(k: PickerKind): TabId {
  switch (k) {
    case "remote_branch":
      return "github";
    case "local_branch":
      return "existing";
    case "repo_base":
      return "local";
    case "commit_sha":
      return "empty";
  }
}

// ---- fetch wrapper (per FR-ORCA-009 §3.3 BFF route) ----

export interface FetchPickerArgs {
  repoId: string;
  baseUrl?: string;
  signal?: AbortSignal;
}

export async function fetchPickerCandidates({
  repoId,
  baseUrl = "/api/v1/worktree-picker",
  signal,
}: FetchPickerArgs): Promise<PickerCandidatesResponse> {
  const url = `${baseUrl}/candidates?repo_id=${encodeURIComponent(repoId)}`;
  const res = await fetch(url, { signal, credentials: "include" });
  if (!res.ok) {
    throw new Error(
      `fetchPickerCandidates failed: ${res.status} ${res.statusText}`,
    );
  }
  const data = (await res.json()) as PickerCandidatesResponse;
  return data;
}

// ---- 组件 props ----

export interface StartFromPickerProps {
  /** 仓库 UUID (frontend 端已知) */
  repoId: string;
  /** 用户确认后回调 (per spec §3.3 "用户确认后 → 调 create_async") */
  onConfirm: (candidate: PickerCandidate) => void;
  /** 用户取消回调 */
  onCancel?: () => void;
  /** 注入的 fetcher (测试用, 默认 fetchPickerCandidates) */
  fetcher?: (args: FetchPickerArgs) => Promise<PickerCandidatesResponse>;
  /** 初始 active tab (per spec 默认 = github) */
  initialTab?: TabId;
  /** 测试注入 (默认跳过 fetch) */
  initialCandidates?: PickerCandidatesResponse;
}

// ---- 主组件 ----

export function StartFromPicker({
  repoId,
  onConfirm,
  onCancel,
  fetcher = fetchPickerCandidates,
  initialTab = "github",
  initialCandidates,
}: StartFromPickerProps) {
  const [activeTab, setActiveTab] = useState<TabId>(initialTab);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [candidates, setCandidates] =
    useState<PickerCandidatesResponse | null>(initialCandidates ?? null);
  const [loading, setLoading] = useState<boolean>(initialCandidates == null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (initialCandidates != null) {
      setCandidates(initialCandidates);
      setLoading(false);
      return;
    }
    const controller = new AbortController();
    setLoading(true);
    setError(null);
    fetcher({ repoId, signal: controller.signal })
      .then((data) => {
        setCandidates(data);
        setLoading(false);
      })
      .catch((e: unknown) => {
        if (controller.signal.aborted) return;
        setError(e instanceof Error ? e.message : String(e));
        setLoading(false);
      });
    return () => controller.abort();
  }, [repoId, fetcher, initialCandidates]);

  const list = useMemo(() => {
    if (!candidates) return [] as PickerCandidate[];
    switch (activeTab) {
      case "github":
        return candidates.github_branches;
      case "existing":
        return candidates.existing_worktrees;
      case "local":
        return candidates.local_paths;
      case "empty":
        return candidates.empty ? [candidates.empty] : [];
    }
  }, [candidates, activeTab]);

  const allCandidates = useMemo(() => {
    if (!candidates) return [] as PickerCandidate[];
    return [
      ...candidates.github_branches,
      ...candidates.existing_worktrees,
      ...candidates.local_paths,
      ...(candidates.empty ? [candidates.empty] : []),
    ];
  }, [candidates]);

  const selected = useMemo(
    () => allCandidates.find((c) => c.id === selectedId) ?? null,
    [allCandidates, selectedId],
  );

  const tabs: { id: TabId; label: string; count: number }[] = useMemo(() => {
    const c = candidates;
    return [
      {
        id: "github",
        label: "GitHub",
        count: c?.github_branches.length ?? 0,
      },
      {
        id: "existing",
        label: "Existing",
        count: c?.existing_worktrees.length ?? 0,
      },
      {
        id: "local",
        label: "Local Path",
        count: c?.local_paths.length ?? 0,
      },
      {
        id: "empty",
        label: "Empty",
        count: c?.empty ? 1 : 0,
      },
    ];
  }, [candidates]);

  if (loading) {
    return (
      <div data-testid="start-from-picker-loading" className="p-4 text-sm text-gray-500">
        Loading picker candidates…
      </div>
    );
  }

  if (error) {
    return (
      <div data-testid="start-from-picker-error" className="p-4 text-sm text-red-500">
        Error: {error}
      </div>
    );
  }

  return (
    <div
      data-testid="start-from-picker"
      className="flex flex-col gap-3 rounded-md border border-gray-200 bg-white p-4 shadow-sm"
    >
      <h2 className="text-lg font-semibold">Start from</h2>

      {/* Tabs */}
      <div role="tablist" className="flex gap-1 border-b border-gray-200">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            role="tab"
            aria-selected={activeTab === tab.id}
            data-testid={`start-from-picker-tab-${tab.id}`}
            onClick={() => {
              setActiveTab(tab.id);
              setSelectedId(null);
            }}
            className={clsx(
              "rounded-t px-3 py-2 text-sm",
              activeTab === tab.id
                ? "border-b-2 border-blue-500 font-medium text-blue-600"
                : "text-gray-500 hover:text-gray-700",
            )}
          >
            {tab.label}
            {tab.count > 0 && (
              <span
                data-testid={`start-from-picker-tab-${tab.id}-count`}
                className="ml-1 rounded bg-gray-100 px-1.5 text-xs"
              >
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Candidate list */}
      <ul
        role="listbox"
        data-testid="start-from-picker-list"
        className="flex max-h-64 flex-col gap-1 overflow-y-auto"
      >
        {list.length === 0 && (
          <li
            data-testid="start-from-picker-empty-list"
            className="p-3 text-sm italic text-gray-400"
          >
            No candidates in this category.
          </li>
        )}
        {list.map((c) => (
          <li
            key={c.id}
            role="option"
            aria-selected={selectedId === c.id}
            data-testid={`start-from-picker-option-${c.id}`}
            onClick={() => setSelectedId(c.id)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") {
                e.preventDefault();
                setSelectedId(c.id);
              }
            }}
            tabIndex={0}
            className={clsx(
              "cursor-pointer rounded px-3 py-2 text-sm",
              selectedId === c.id
                ? "bg-blue-50 ring-2 ring-blue-500"
                : "hover:bg-gray-50",
            )}
          >
            <div className="font-medium">{c.label}</div>
            <div className="text-xs text-gray-500">{c.description}</div>
          </li>
        ))}
      </ul>

      {/* Actions */}
      <div className="flex justify-end gap-2 border-t border-gray-200 pt-3">
        {onCancel && (
          <button
            type="button"
            data-testid="start-from-picker-cancel"
            onClick={onCancel}
            className="rounded px-3 py-1.5 text-sm text-gray-600 hover:bg-gray-100"
          >
            Cancel
          </button>
        )}
        <button
          type="button"
          data-testid="start-from-picker-confirm"
          disabled={selected == null}
          onClick={() => selected && onConfirm(selected)}
          className={clsx(
            "rounded px-3 py-1.5 text-sm font-medium",
            selected != null
              ? "bg-blue-600 text-white hover:bg-blue-700"
              : "cursor-not-allowed bg-gray-200 text-gray-400",
          )}
        >
          Confirm
        </button>
      </div>

      {/* Selected summary (debug-friendly) */}
      {selected && (
        <div
          data-testid="start-from-picker-selected"
          className="rounded bg-gray-50 p-2 text-xs text-gray-600"
        >
          Selected: <span className="font-mono">{selected.id}</span> (
          {selected.kind})
        </div>
      )}
    </div>
  );
}

export default StartFromPicker;