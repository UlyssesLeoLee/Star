"use client";

// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-shared/StartFromPicker.tsx
//
// ULYS-228 (ULYS-218.1) FR-ORCA-009 Start-from Picker modal (4 选 1).
//
// 4 选 1 radio (per docs/ecosystem-survey/orca-design-survey.md §3 FR-ORCA-009):
//   - repo_base:    仓库默认 base ref (origin/main or origin/master)
//   - local_branch: 另一个本地分支 (`git branch -a` 本地部分)
//   - commit_sha:   特定 commit SHA (`git log --oneline -20` 最近 N 条)
//   - remote_branch: 远程 branch (fetch 后 checkout)
//
// 数据源: `GET /api/v1/worktree-picker/candidates?repo_id=X`
//   当前 backend 是 Stage 1 placeholder (返 503), UI 端走 MSW mock handler 拿到 4
//   分类假数据; 真后端 ship 后 (ULYS-177.2 后续) 自动切真 provider (per
//   `crates/api/src/worktree_picker.rs` worktree_picker_router fallback).
//
// 样式: 跟 worktree-canvas/ConfirmationModal.tsx 对齐 (modal role/aria/data-testid +
// 取消 + 确认 双按钮底部 footer), 不引第三方 modal 库.

import * as React from "react";
import { useEffect, useMemo, useState } from "react";
import {
  type PickerCandidateDto,
  type PickerCandidatesDto,
  type StartFromKind,
  type StartFromSelection,
} from "./startFromPickerTypes";

interface StartFromPickerProps {
  /** 必填: 要查询 picker 的 repo UUID (字符串格式, 经 BFF). */
  repoId: string;
  /** 选完后回填 (parent dialog 据此填 `start_from` 字段 + 调真实 `git worktree add`). */
  onSelect: (selection: StartFromSelection) => void;
  /** 用户点取消 (含 backdrop / Esc / 取消按钮). */
  onCancel: () => void;
}

/**
 * 4 选 1 配置 (UI 渲染顺序 + 显示名).
 *
 * 注: backend 返的 4 分类 (`github_branches` / `existing_worktrees` / `local_paths`
 * / `empty`) 跟我们 UI 的 4 选 1 radio 一一对应: github_branches → remote_branch,
 * existing_worktrees → local_branch, local_paths → commit_sha (历史命名), empty →
 * repo_base. 该映射在 `fetchCandidates` / `byKind` 内完成.
 */
const RADIO_KINDS: ReadonlyArray<{ value: StartFromKind; label: string; helper: string }> = [
  {
    value: "repo_base",
    label: "Repo base ref",
    helper: "origin/main or origin/master — 仓库默认 base ref",
  },
  {
    value: "local_branch",
    label: "Another local branch",
    helper: "本仓库已经 checkout 的另一个本地分支",
  },
  {
    value: "commit_sha",
    label: "Specific commit SHA",
    helper: "最近 20 条 commit, 选完后会展开成 full 40 hex",
  },
  {
    value: "remote_branch",
    label: "Remote branch (fetch then checkout)",
    helper: "先 `git fetch` 再 checkout, 不会污染本地分支",
  },
];

const SHORT_SHA_LEN = 7;

/** 把 backend 4 分类响应按 kind 重新分组 (per FR-ORCA-009 4 选 1). */
function pickKindGroup(
  data: PickerCandidatesDto | null,
  kind: StartFromKind,
): PickerCandidateDto[] {
  if (!data) return [];
  switch (kind) {
    case "repo_base":
      return data.empty ? [data.empty] : [];
    case "local_branch":
      return data.existing_worktrees ?? [];
    case "commit_sha":
      return data.local_paths ?? [];
    case "remote_branch":
      return data.github_branches ?? [];
  }
}

export function StartFromPicker({ repoId, onSelect, onCancel }: StartFromPickerProps) {
  const [candidates, setCandidates] = useState<PickerCandidatesDto | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [kind, setKind] = useState<StartFromKind>("repo_base");
  const [pickedId, setPickedId] = useState<string>("");

  // fetch candidates on mount / repoId change
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setLoadError(null);
    setCandidates(null);
    setPickedId("");
    setKind("repo_base");

    const url = `/api/v1/worktree-picker/candidates?repo_id=${encodeURIComponent(repoId)}`;
    fetch(url, { method: "GET" })
      .then(async (resp) => {
        if (!resp.ok) {
          // 503 placeholder / 5xx 都视为软错误: 让用户仍可继续 (后面 import 后端
          // ship 后真返 4 分类). UI 这里显示 "no candidates" + 取消按钮.
          if (cancelled) return;
          setLoadError(`picker backend unavailable (HTTP ${resp.status})`);
          setCandidates(null);
          setLoading(false);
          return;
        }
        const body = (await resp.json()) as PickerCandidatesDto;
        if (cancelled) return;
        setCandidates(body);
        setLoading(false);
      })
      .catch((err: unknown) => {
        if (cancelled) return;
        setLoadError(err instanceof Error ? err.message : "fetch failed");
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [repoId]);

  // Esc 关闭
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onCancel();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onCancel]);

  const list = useMemo(() => pickKindGroup(candidates, kind), [candidates, kind]);

  // 切换 kind 时清空已选 id (避免上一种 kind 的 selection 残留)
  useEffect(() => {
    setPickedId("");
  }, [kind]);

  const pickedCandidate = useMemo(
    () => list.find((c) => c.id === pickedId) ?? null,
    [list, pickedId],
  );

  const handleConfirm = () => {
    if (!pickedCandidate) return;
    onSelect({
      kind,
      candidate_id: pickedCandidate.id,
      label: pickedCandidate.label,
    });
  };

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Start-from Picker"
      data-testid="start-from-picker-modal"
      data-repo-id={repoId}
      data-kind={kind}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={onCancel}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        className="w-[28rem] max-w-[92vw] rounded-lg bg-white p-5 shadow-lg dark:bg-slate-900"
      >
        <h3 className="mb-1 text-base font-semibold text-slate-900 dark:text-slate-100">
          Start from
        </h3>
        <p className="mb-3 text-xs text-slate-500 dark:text-slate-400">
          选择 4 选 1 start-from, 选完后会调 <code>git worktree add</code>.
        </p>

        {/* === 4 选 1 radio === */}
        <div role="radiogroup" aria-label="Start-from kind" className="mb-3 space-y-2">
          {RADIO_KINDS.map((opt) => {
            const group = pickKindGroup(candidates, opt.value);
            const isSelected = kind === opt.value;
            return (
              <label
                key={opt.value}
                data-testid={`start-from-kind-${opt.value}`}
                className={
                  "flex cursor-pointer items-start gap-2 rounded border p-2 text-sm " +
                  (isSelected
                    ? "border-blue-500 bg-blue-50 dark:bg-blue-900/30"
                    : "border-slate-200 hover:border-slate-300 dark:border-slate-700")
                }
              >
                <input
                  type="radio"
                  name="start-from-kind"
                  value={opt.value}
                  checked={isSelected}
                  onChange={() => setKind(opt.value)}
                  className="mt-0.5"
                  data-testid={`start-from-radio-${opt.value}`}
                />
                <span className="flex-1">
                  <span className="block font-medium text-slate-900 dark:text-slate-100">
                    {opt.label}{" "}
                    <span className="ml-1 text-xs font-normal text-slate-500">
                      ({group.length})
                    </span>
                  </span>
                  <span className="block text-xs text-slate-500 dark:text-slate-400">
                    {opt.helper}
                  </span>
                </span>
              </label>
            );
          })}
        </div>

        {/* === 当前 kind 的候选 list === */}
        <div
          data-testid="start-from-list"
          data-kind={kind}
          className="mb-3 max-h-56 overflow-y-auto rounded border border-slate-200 dark:border-slate-700"
        >
          {loading ? (
            <p className="p-3 text-xs text-slate-500">Loading candidates…</p>
          ) : loadError ? (
            <p
              data-testid="start-from-error"
              className="p-3 text-xs text-amber-700 dark:text-amber-400"
            >
              {loadError}. 仍可选中后回填, 实际 import 由调用方决定.
            </p>
          ) : list.length === 0 ? (
            <p
              data-testid="start-from-empty"
              className="p-3 text-xs text-slate-500"
            >
              No candidates for this kind. 切换其它 4 选 1 kind 试试.
            </p>
          ) : (
            <ul className="divide-y divide-slate-200 dark:divide-slate-700">
              {list.map((c) => (
                <li key={c.id}>
                  <button
                    type="button"
                    data-testid="start-from-candidate"
                    data-candidate-id={c.id}
                    data-selected={pickedId === c.id}
                    onClick={() => setPickedId(c.id)}
                    className={
                      "w-full px-3 py-2 text-left text-xs hover:bg-slate-50 dark:hover:bg-slate-800 " +
                      (pickedId === c.id
                        ? "bg-blue-50 font-medium dark:bg-blue-900/40"
                        : "")
                    }
                  >
                    <span className="block text-slate-900 dark:text-slate-100">
                      {c.label}
                    </span>
                    <span className="block text-slate-500">
                      {kind === "commit_sha" && c.description.length > SHORT_SHA_LEN
                        ? c.description.slice(0, SHORT_SHA_LEN)
                        : c.description}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>

        <div className="flex justify-end gap-2">
          <button
            type="button"
            data-testid="start-from-cancel"
            onClick={onCancel}
            className="rounded border border-slate-200 px-3 py-1 text-sm hover:bg-slate-50 dark:border-slate-700"
          >
            Cancel
          </button>
          <button
            type="button"
            data-testid="start-from-confirm"
            onClick={handleConfirm}
            disabled={!pickedCandidate}
            className="rounded bg-blue-600 px-3 py-1 text-sm text-white hover:bg-blue-700 disabled:cursor-not-allowed disabled:bg-slate-400"
          >
            Confirm
          </button>
        </div>
      </div>
    </div>
  );
}