// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/InspectorFiles.tsx
// Inspector Tab 7/11: Files.
//
// ULYS-211: 接入 DiffAnnotator (per FR-ORCA-035 "用户对 AI 生成的 diff
// 可在任意行 drop comment")。
//
// 缺标 (per 守门 #11, 2026-09-23):
//   1. 后端没有"取某 worktree 变更 diff"的接口, diff 正文用
//      src/mocks/data/annotations.ts 的演示 hunk;
//   2. worktree → agent_run 的映射未落 (WorktreeNodeData 无 agent_run_id
//      字段), 故先挂 DEMO_AGENT_RUN_ID。
//   两处都只需换数据源, DiffAnnotator 与 annotationsApi 不用改。
"use client";
import { memo } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";
import { DiffAnnotator } from "@/components/DiffAnnotator";
import {
  DEMO_AGENT_RUN_ID,
  DEMO_DIFF_LINES,
  DEMO_FILE_PATH,
} from "@/mocks/data/annotations";

function Inner() {
  const selectedId = useWorktreeCanvasStore((s) => s.selectedWorktreeId);
  if (!selectedId) {
    return (
      <div data-testid="inspector-files" className="p-4 text-sm">
        <p className="text-slate-500">请选择 Worktree</p>
      </div>
    );
  }
  return (
    <div data-testid="inspector-files" className="p-2 text-sm">
      <p className="mb-2 text-[11px] text-slate-500">
        Files modified by {selectedId.slice(0, 8)} — 点行号旁图标可标注 (diff 正文为演示数据)
      </p>
      <DiffAnnotator
        filePath={DEMO_FILE_PATH}
        agentRunId={DEMO_AGENT_RUN_ID}
        lines={DEMO_DIFF_LINES}
      />
    </div>
  );
}
export const InspectorFiles = memo(Inner);
export default InspectorFiles;
