/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/tasks/components/SpawnSubtaskModal.tsx"})
@endcypher
*/
"use client";

/**
 * SpawnSubtaskModal (C-30 UISpawnSubtaskModal)
 * per docs/architecture/2026-09-09-subtask-binding/03-detailed-design.md §3.3.9
 *  + 01-requirements.md §1 UC-14 + §2 F-32 (UI 任务卡右键 "Spawn sub-task" 入口)
 *
 * 守门:
 *   - 守门 #9 v3 + #24 v2: POST /tasks/api/tmo/spawn_subtask → console_server.py → subprocess
 *   - 守门 #23 v23: AI 建议走 mock, 不开 OpenAI (后端 ai_edit_mock 未挂载时优雅降级)
 *   - 守门 #12: 失败路径必展示 failure_reason (缺标比错标)
 *
 * 适配: 本仓无 antd, 用原生 JSX + tailwind + react-hot-toast (per frontend/package.json).
 */

import { useState } from "react";
import toast from "react-hot-toast";

interface SpawnSubtaskModalProps {
  visible: boolean;
  parentTaskId: string;
  onCancel: () => void;
  onSuccess: (childTaskId: string) => void;
}

interface SpawnSubtaskResponse {
  child_task_id: string;
  parent_task_id: string;
  sa_type: string;
  status: "spawned" | "failed";
  failure_reason?: string | null;
  spawned_at?: string;
  exclusive_owner_task_id?: string;
  quota_remaining?: number;
}

interface AiSuggestionResponse {
  suggested_name: string;
  confidence?: number;
}

/** manifest 路径 → URL 映射 (per brief §3.1, 后端第 9 端点 /api/tmo/spawn_subtask) */
const SPAWN_ENDPOINT = "/tasks/api/tmo/spawn_subtask";
const AI_SUGGEST_ENDPOINT = "/api/automation/ai_suggest_subtask_name";

const SA_TYPES = [
  "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
  "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
];

export default function SpawnSubtaskModal({
  visible,
  parentTaskId,
  onCancel,
  onSuccess,
}: SpawnSubtaskModalProps) {
  const [saType, setSaType] = useState<string>("SA-04"); // 默认 git-ops (per DD §3.3.9)
  const [childName, setChildName] = useState<string>("");
  const [aiSuggesting, setAiSuggesting] = useState<boolean>(false);
  const [loading, setLoading] = useState<boolean>(false);

  if (!visible) {
    return null;
  }

  // AI 建议 sub-task name (per 守门 #23 v23 走 ai_edit_mock.py 本地 mock, 不开 OpenAI)
  const handleAISuggest = async () => {
    setAiSuggesting(true);
    try {
      const res = await fetch(AI_SUGGEST_ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ parent_task_id: parentTaskId, sa_type: saType }),
      });
      if (!res.ok) {
        toast.error("AI 建议暂不可用 (ai_edit_mock 后端未挂载), 请手动输入名称");
        return;
      }
      const data = (await res.json()) as AiSuggestionResponse;
      setChildName(data.suggested_name);
      const confidence = data.confidence ?? 0;
      toast(
        `AI 建议: ${data.suggested_name} (confidence=${confidence.toFixed(2)}${
          confidence < 0.5 ? ", 请手动 review" : ""
        })`
      );
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : String(e);
      toast.error(`AI 建议请求失败: ${message}`);
    } finally {
      setAiSuggesting(false);
    }
  };

  // 提交 spawn sub-task (per DD §3.3.9, 守门 #24 v2 链路)
  const handleSubmit = async () => {
    const trimmed = childName.trim();
    if (!trimmed) {
      toast.error("请输入 sub-task name");
      return;
    }
    setLoading(true);
    try {
      const res = await fetch(SPAWN_ENDPOINT, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          parent_task_id: parentTaskId,
          child_sa_type: saType,
          child_name: trimmed,
          spawned_via: "ui_rightclick", // per 守门入口标注
        }),
      });
      const result = (await res.json()) as SpawnSubtaskResponse;
      if (result.status === "spawned") {
        toast.success(`Sub-task spawned: ${result.child_task_id}`);
        setChildName("");
        onSuccess(result.child_task_id);
      } else {
        toast.error(`Spawn failed: ${result.failure_reason ?? "unknown reason"}`);
      }
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : String(e);
      toast.error(`Network error: ${message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40">
      <div className="w-96 rounded-lg border border-zinc-700 bg-zinc-900 p-4 text-zinc-100 shadow-xl">
        <h2 className="mb-3 text-base font-semibold">Spawn sub-task</h2>

        <label className="mb-1 block text-sm text-zinc-400">Sub-agent type (SA-XX)</label>
        <select
          className="mb-3 w-full rounded border border-zinc-700 bg-zinc-800 px-2 py-1.5 text-sm"
          value={saType}
          onChange={(e) => setSaType(e.target.value)}
        >
          {SA_TYPES.map((sa) => (
            <option key={sa} value={sa}>
              {sa}
            </option>
          ))}
        </select>

        <label className="mb-1 block text-sm text-zinc-400">Sub-task name</label>
        <input
          className="mb-1 w-full rounded border border-zinc-700 bg-zinc-800 px-2 py-1.5 text-sm"
          value={childName}
          onChange={(e) => setChildName(e.target.value)}
          placeholder="Enter sub-task name"
          maxLength={200}
        />
        <button
          type="button"
          className="mb-3 text-xs text-sky-400 hover:underline disabled:opacity-50"
          disabled={aiSuggesting}
          onClick={handleAISuggest}
        >
          {aiSuggesting ? "AI 建议生成中..." : "🤖 AI 建议 (mock, per 守门 #23 v23)"}
        </button>

        <label className="mb-1 block text-sm text-zinc-400">Parent task ID</label>
        <input
          className="mb-4 w-full rounded border border-zinc-700 bg-zinc-800 px-2 py-1.5 text-sm text-zinc-500"
          value={parentTaskId}
          disabled
        />

        <div className="flex justify-end gap-2">
          <button
            type="button"
            className="rounded border border-zinc-700 px-3 py-1.5 text-sm hover:bg-zinc-800"
            onClick={onCancel}
          >
            Cancel
          </button>
          <button
            type="button"
            className="rounded bg-sky-600 px-3 py-1.5 text-sm text-white hover:bg-sky-500 disabled:opacity-50"
            disabled={loading}
            onClick={handleSubmit}
          >
            {loading ? "Spawning..." : "Spawn"}
          </button>
        </div>
      </div>
    </div>
  );
}
