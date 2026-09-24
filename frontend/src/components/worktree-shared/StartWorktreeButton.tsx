"use client";

// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-shared/StartWorktreeButton.tsx
//
// ULYS-228 (ULYS-218.1) FR-ORCA-009 entry point: "New worktree" 按钮 → 弹
// StartFromPicker modal → 选完回填给 caller (e.g. worktree page 把选中的
// StartFromSelection 显示在 page 上, 调真实 backend 创建 worktree).
//
// 用法:
//   <StartWorktreeButton
//     repoId="..."
//     onSelect={(s) => console.log("user picked", s)}
//   />
//
// 样式: 跟 worktree-canvas 风格一致 (按 cel-shadow 按钮), 不引第三方 modal 库.

import * as React from "react";
import { useState } from "react";
import { StartFromPicker } from "./StartFromPicker";
import type { StartFromSelection } from "./startFromPickerTypes";

interface StartWorktreeButtonProps {
  repoId: string;
  /** 选完后回填给 caller (per FR-ORCA-009 §3.1 "选完后回填到 dialog 父组件"). */
  onSelect?: (selection: StartFromSelection) => void;
  /** 按钮文案覆盖, 默认 "New worktree". */
  label?: string;
}

export function StartWorktreeButton({
  repoId,
  onSelect,
  label = "New worktree",
}: StartWorktreeButtonProps) {
  const [open, setOpen] = useState(false);
  const [lastSelection, setLastSelection] = useState<StartFromSelection | null>(
    null,
  );

  return (
    <>
      <button
        type="button"
        data-testid="start-worktree-button"
        onClick={() => setOpen(true)}
        className="rounded border-2 border-[var(--cel-ink)] bg-accent px-3 py-1 text-xs font-semibold text-white cel-shadow hover:brightness-110"
      >
        + {label}
      </button>

      {open && (
        <StartFromPicker
          repoId={repoId}
          onCancel={() => setOpen(false)}
          onSelect={(sel) => {
            setLastSelection(sel);
            onSelect?.(sel);
            setOpen(false);
          }}
        />
      )}

      {lastSelection && (
        <p
          data-testid="start-worktree-last-selection"
          className="mt-1 text-[11px] text-slate-500"
        >
          last: <span className="font-mono">{lastSelection.kind}</span> ·{" "}
          <span className="font-mono">{lastSelection.label}</span>
        </p>
      )}
    </>
  );
}