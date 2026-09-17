// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/worktree-canvas/AIDialog.tsx
// AI NL query dialog (per FR-EXPLAIN-001..004 + spec §20).
"use client";
import { memo, useState } from "react";
import { useWorktreeCanvasStore } from "@/stores/worktreeCanvasStore";

function Inner() {
  const [question, setQuestion] = useState("");
  const [answer, setAnswer] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const recentEvents = useWorktreeCanvasStore((s) => s.recentEvents);

  const ask = async () => {
    if (!question.trim()) return;
    setLoading(true);
    // Stage 1: 走 mock LLM 路径 (per 守门 #23 v2).
    await new Promise((r) => setTimeout(r, 200));
    setAnswer(
      `[stage 1 mock] NL query: "${question}". events in buffer: ${recentEvents.length}. ` +
        `per DD §17 AI Explanation: 阶段 2 接 graph + risk + git 数据生成.`
    );
    setLoading(false);
  };

  return (
    <div className="rounded border border-slate-200 bg-white p-3 shadow-sm dark:border-slate-700 dark:bg-slate-900">
      <div className="mb-2 text-xs font-semibold">🤖 AI Dialog</div>
      <div className="flex gap-2">
        <input
          type="text"
          data-testid="ai-dialog-input"
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="显示所有冲突 + 由 Codex 处理的 Worktree"
          className="flex-1 rounded border border-slate-200 bg-transparent px-2 py-1 text-xs outline-none dark:border-slate-700"
        />
        <button
          type="button"
          data-testid="ai-dialog-submit"
          disabled={loading}
          onClick={ask}
          className="rounded bg-blue-500 px-2 py-1 text-xs text-white disabled:opacity-50"
        >
          {loading ? "..." : "Ask"}
        </button>
      </div>
      {answer && (
        <div
          data-testid="ai-dialog-answer"
          className="mt-2 rounded bg-slate-50 p-2 text-xs text-slate-600 dark:bg-slate-800/50 dark:text-slate-300"
        >
          {answer}
        </div>
      )}
    </div>
  );
}
export const AIDialog = memo(Inner);
export default AIDialog;
