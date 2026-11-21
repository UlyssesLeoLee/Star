/*
@cypher schema=1 source_sha256=bce09ed8f4b45a4e6df9497fdce9ada0a4ab78f92b80dc975c0deef0e6f7bbb9
MERGE (self:File {path:"frontend/src/components/board/TaskCardCommand.tsx"})
MERGE (comp:Symbol {id:"frontend/src/components/board/TaskCardCommand.tsx::TaskCardCommand",kind:"component"})
MERGE (saveH:Symbol {id:"frontend/src/components/board/TaskCardCommand.tsx::saveCommandHistory",kind:"function"})
MERGE (loadH:Symbol {id:"frontend/src/components/board/TaskCardCommand.tsx::loadCommandHistory",kind:"function"})
MERGE (saveD:Symbol {id:"frontend/src/components/board/TaskCardCommand.tsx::saveDraft",kind:"function"})
MERGE (loadD:Symbol {id:"frontend/src/components/board/TaskCardCommand.tsx::loadDraft",kind:"function"})
MERGE (self)-[:DEFINES]->(comp)
MERGE (self)-[:DEFINES]->(saveH)
MERGE (self)-[:DEFINES]->(loadH)
MERGE (self)-[:DEFINES]->(saveD)
MERGE (self)-[:DEFINES]->(loadD)
@endcypher
*/
"use client";

// =====================================================================
// TaskCardCommand — Multica 式任务卡命令下达组件
// =====================================================================
// 职责:
//   1. 命令输入行: input + 发送按钮, Enter 发送
//   2. 命令历史: 最近 10 条本地保存 (localStorage `star-tcc-<taskId>`),
//      上下箭头调出历史, 列表点击回填输入
//   3. 草稿保存: 未发送的输入自动保存 (localStorage `star-tcc-draft-<taskId>`),
//      重开组件自动恢复草稿
//   4. 交接区: "交接给" select (Ulysses / Agent A1 / Agent B2) + 交接按钮,
//      点击调 onHandoff(target) 并清空输入
//   5. 发送后: 调 onSubmitCommand(cmd), 由父组件写入右侧聊天话题
//      (见下方"接线点"注释), 并显示"已发送到右侧聊天"提示
//   6. 诚实徽标: 命令执行未接入 — 内容同步至聊天话题,
//      宿主 agent 网关接入后真实执行
//
// 诚实原则 (per 需求):
//   命令执行后端未接入 — 命令仅作为本地聊天消息发送到右侧聊天话题
//   (真实可行); 执行结果标注"待宿主 agent 网关执行", 不假装已执行。
//
// =====================================================================
// 接线点 (父组件把命令写入右侧聊天话题 — chatStore, IM1.0 域模型):
//
//   import { useChatStore } from "@/lib/chat/chatStore";
//
//   <TaskCardCommand
//     taskId={workItem.id}        // ChatTopic.scope="task" 的 refId
//     taskKey={workItem.key}      // 显示用 (如 T-142)
//     onSubmitCommand={(cmd) => {
//       const chat = useChatStore.getState();
//       // openTopicForTask 幂等: 复用已有话题或新建 (kind="channel")
//       const topicId = chat.openTopicForTask(workItem.id, workItem.key);
//       // reply 追加 ChatMessage (sequence 自增, state="sent")
//       chat.reply(topicId, "Ulysses", "human", cmd);
//       // 执行回执由宿主 agent 网关接入后追加:
//       // chat.reply(topicId, "system", "system", "待宿主 agent 网关执行");
//     }}
//   />
//
// 组件本身不 import chatStore — 保持纯展示/本地持久化, 聊天写入由父组件完成。
// =====================================================================

import { useEffect, useRef, useState } from "react";
import { History, Send, UserRound } from "lucide-react";

/** 命令发送目标 (per 2026-11-21 用户发令: 只有手动指定才发送;
 *  默认 "card" = 内容只保留在任务卡, 不进聊天) */
export type TaskCardSendTarget = "card" | "worktree" | "global";
export const SEND_TARGETS: ReadonlyArray<{ value: TaskCardSendTarget; label: string }> = [
  { value: "card", label: "仅任务卡 (默认)" },
  { value: "worktree", label: "Worktree 聊天" },
  { value: "global", label: "Global 聊天" },
];

export interface TaskCardCommandProps {
  /** 任务卡 id (workItem.id) — 右侧聊天话题 (task scope) 的 refId */
  taskId: string;
  /** 任务卡 key (如 T-142) — 显示用 */
  taskKey: string;
  /** 提交回调 — 父组件在此接线把命令写入右侧聊天话题;
   *  target 由用户在发送目标下拉中手动指定 (默认 "card" 时父组件不写聊天) */
  onSubmitCommand: (cmd: string, target: TaskCardSendTarget) => void;
  /** 交接回调 — 可选; 点击交接按钮时调用并清空输入 */
  onHandoff?: (target: string) => void;
}

/** 命令历史上限 (最近 10 条) */
export const HISTORY_LIMIT = 10;
/** 交接目标候选 (Multica 式 agent 角色) */
export const HANDOFF_TARGETS = ["Ulysses", "Agent A1", "Agent B2"] as const;

const historyKey = (taskId: string) => `star-tcc-${taskId}`;
const draftKey = (taskId: string) => `star-tcc-draft-${taskId}`;

/** SSR 安全地取 localStorage (服务端渲染返回 null) */
const storage = (): Storage | null =>
  typeof window === "undefined" ? null : window.localStorage;

// ---------------------------------------------------------------------
// 导出 helpers (供测试直接读写 localStorage)
// ---------------------------------------------------------------------

/** 读取命令历史 (最新在前, 最多 HISTORY_LIMIT 条) */
export function loadCommandHistory(taskId: string): string[] {
  const s = storage();
  if (!s) return [];
  try {
    const raw = s.getItem(historyKey(taskId));
    if (!raw) return [];
    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed
      .filter((x): x is string => typeof x === "string")
      .slice(0, HISTORY_LIMIT);
  } catch {
    return [];
  }
}

/** 追加一条命令到历史 (最新在前, 截断到 HISTORY_LIMIT), 返回新列表 */
export function saveCommandHistory(taskId: string, cmd: string): string[] {
  const s = storage();
  const trimmed = cmd.trim();
  if (!s || !trimmed) return loadCommandHistory(taskId);
  const current = loadCommandHistory(taskId);
  // 连续重复去重 (shell 惯例): 与最新一条相同则不重复追加
  const next = current[0] === trimmed ? current : [trimmed, ...current];
  const capped = next.slice(0, HISTORY_LIMIT);
  try {
    s.setItem(historyKey(taskId), JSON.stringify(capped));
  } catch {
    // localStorage 不可用 (隐私模式/配额) — 静默降级, 不抛错
  }
  return capped;
}

/** 读取草稿 (无草稿返回 "") */
export function loadDraft(taskId: string): string {
  const s = storage();
  if (!s) return "";
  try {
    return s.getItem(draftKey(taskId)) ?? "";
  } catch {
    return "";
  }
}

/** 保存草稿; 空串 = 删除草稿 (发送/交接后清除) */
export function saveDraft(taskId: string, text: string): void {
  const s = storage();
  if (!s) return;
  try {
    if (text === "") {
      s.removeItem(draftKey(taskId));
    } else {
      s.setItem(draftKey(taskId), text);
    }
  } catch {
    // localStorage 不可用 — 静默降级
  }
}

export function TaskCardCommand({
  taskId,
  taskKey,
  onSubmitCommand,
  onHandoff,
}: TaskCardCommandProps) {
  const [input, setInput] = useState("");
  /** 草稿恢复完成前不落盘, 防止挂载瞬间用空串覆盖已存草稿 */
  const [draftReady, setDraftReady] = useState(false);
  const [history, setHistory] = useState<string[]>([]);
  /** null = 正在编辑; 非 null = 正在浏览历史第 N 条 (最新 = 0) */
  const [historyIdx, setHistoryIdx] = useState<number | null>(null);
  const [sentHint, setSentHint] = useState<"" | "card" | "worktree" | "global">("");
  const [handoffTarget, setHandoffTarget] = useState<string>(HANDOFF_TARGETS[0]);
  /** 发送目标 (per 2026-11-21: 默认仅任务卡, 手动选 Worktree/Global 才进聊天) */
  const [sendTarget, setSendTarget] = useState<TaskCardSendTarget>("card");
  /** 进入历史浏览前的输入内容, 供 ↓ 越过最新一条时恢复 */
  const preHistoryRef = useRef("");
  const hintTimerRef = useRef<number | null>(null);

  // 挂载/切换任务卡: 恢复历史与草稿; 卸载时清理提示定时器
  useEffect(() => {
    setHistory(loadCommandHistory(taskId));
    setHistoryIdx(null);
    preHistoryRef.current = "";
    setDraftReady(false);
    setInput(loadDraft(taskId));
    setDraftReady(true);
    return () => {
      if (hintTimerRef.current !== null) {
        window.clearTimeout(hintTimerRef.current);
        hintTimerRef.current = null;
      }
    };
  }, [taskId]);

  // 草稿保存: 未发送的输入实时落盘 (draftReady 防覆盖)
  useEffect(() => {
    if (draftReady) saveDraft(taskId, input);
  }, [draftReady, taskId, input]);

  // ---- 发送 (接线点: onSubmitCommand 由父组件按 target 决定是否写入聊天) ----
  const handleSend = () => {
    const cmd = input.trim();
    if (!cmd) return;
    onSubmitCommand(cmd, sendTarget);
    setHistory(saveCommandHistory(taskId, cmd));
    setInput("");
    setHistoryIdx(null);
    preHistoryRef.current = "";
    setSentHint(sendTarget);
    if (hintTimerRef.current !== null) window.clearTimeout(hintTimerRef.current);
    hintTimerRef.current = window.setTimeout(() => setSentHint(""), 3000);
  };

  // ---- 交接: 调 onHandoff(target) 并清空输入 (草稿随之清除) ----
  const handleHandoff = () => {
    const target = handoffTarget;
    onHandoff?.(target);
    setInput("");
    setHistoryIdx(null);
    preHistoryRef.current = "";
    setSentHint("");
  };

  // ---- 键盘: Enter 发送, ↑↓ 调历史 ----
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleSend();
      return;
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (history.length === 0) return;
      if (historyIdx === null) {
        preHistoryRef.current = input;
        setHistoryIdx(0);
        setInput(history[0]);
      } else if (historyIdx < history.length - 1) {
        setHistoryIdx(historyIdx + 1);
        setInput(history[historyIdx + 1]);
      }
      return;
    }
    if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIdx === null) return;
      if (historyIdx > 0) {
        setHistoryIdx(historyIdx - 1);
        setInput(history[historyIdx - 1]);
      } else {
        setHistoryIdx(null);
        setInput(preHistoryRef.current);
      }
    }
  };

  // ---- 点击历史条目回填输入 ----
  const handleHistoryPick = (cmd: string) => {
    setInput(cmd);
    setHistoryIdx(null);
    preHistoryRef.current = "";
  };

  const canSend = input.trim().length > 0;

  return (
    <div className="rounded-lg border border-line bg-bg-card p-3 text-sm">
      {/* 标题行: 任务卡标识 */}
      <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-ink-mute">
        <UserRound className="h-3.5 w-3.5" aria-hidden />
        <span>任务卡 {taskKey} · 命令下达</span>
      </div>

      {/* ① 命令输入行 */}
      <div className="flex items-center gap-2">
        <input
          type="text"
          data-testid="tcc-input"
          value={input}
          onChange={(e) => {
            setInput(e.target.value);
            setHistoryIdx(null);
          }}
          onKeyDown={handleKeyDown}
          placeholder="输入命令, Enter 发送 · ↑↓ 调历史"
          aria-label={`任务卡 ${taskKey} 命令输入`}
          className="min-w-0 flex-1 rounded-md border border-line bg-bg px-2.5 py-1.5 text-sm text-ink placeholder:text-ink-mute focus:border-info focus:outline-none"
        />
        <button
          type="button"
          data-testid="tcc-send"
          onClick={handleSend}
          disabled={!canSend}
          className="inline-flex shrink-0 items-center gap-1 rounded-md bg-info px-2.5 py-1.5 text-sm font-medium text-white transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-40"
        >
          <Send className="h-3.5 w-3.5" aria-hidden />
          发送
        </button>
      </div>

      {/* 发送目标 (per 2026-11-21: 默认仅任务卡; 手动指定才发送到聊天) */}
      <div className="mt-1.5 flex items-center gap-2">
        <label htmlFor={`tcc-send-target-${taskId}`} className="shrink-0 text-xs text-ink-mute">
          发送到
        </label>
        <select
          id={`tcc-send-target-${taskId}`}
          data-testid="tcc-send-target"
          value={sendTarget}
          onChange={(e) => setSendTarget(e.target.value as TaskCardSendTarget)}
          className="min-w-0 flex-1 rounded-md border border-line bg-bg px-2 py-1 text-xs text-ink focus:border-info focus:outline-none"
        >
          {SEND_TARGETS.map((t) => (
            <option key={t.value} value={t.value}>
              {t.label}
            </option>
          ))}
        </select>
      </div>

      {/* ③ 草稿指示 (当前输入未发送 = 已自动存草稿) */}
      {input.length > 0 && (
        <div className="mt-1.5 text-xs text-ink-mute">
          <span data-testid="tcc-draft">草稿已保存 (重开自动恢复)</span>
        </div>
      )}

      {/* ⑤ 已发送提示 (按目标区分: 默认仅任务卡, 手动选择才进聊天) */}
      {sentHint && (
        <div
          data-testid="tcc-sent-hint"
          className="mt-1.5 inline-block rounded bg-warn/15 px-2 py-0.5 text-xs text-warn"
        >
          {sentHint === "card"
            ? "已保留在任务卡 (未发送到聊天)"
            : sentHint === "worktree"
              ? "已发送到 Worktree 聊天"
              : "已发送到 Global 聊天"}
        </div>
      )}

      {/* ② 命令历史 (最近 10 条, 最新在前) */}
      {history.length > 0 && (
        <div data-testid="tcc-history" className="mt-2">
          <div className="mb-1 flex items-center gap-1 text-xs text-ink-mute">
            <History className="h-3.5 w-3.5" aria-hidden />
            <span>
              命令历史 ({history.length}/{HISTORY_LIMIT})
            </span>
          </div>
          <ul className="space-y-0.5">
            {history.map((cmd, i) => (
              <li key={i}>
                <button
                  type="button"
                  data-testid={`tcc-history-${i}`}
                  onClick={() => handleHistoryPick(cmd)}
                  className="w-full truncate rounded border border-transparent bg-bg px-2 py-1 text-left font-mono text-xs text-ink-mute transition-colors hover:border-line hover:text-ink"
                >
                  {cmd}
                </button>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* ④ 交接区 */}
      <div className="mt-3 flex items-center gap-2 border-t border-line pt-2.5">
        <label htmlFor={`tcc-handoff-${taskId}`} className="shrink-0 text-xs text-ink-mute">
          交接给
        </label>
        <select
          id={`tcc-handoff-${taskId}`}
          data-testid="tcc-handoff-target"
          value={handoffTarget}
          onChange={(e) => setHandoffTarget(e.target.value)}
          className="min-w-0 flex-1 rounded-md border border-line bg-bg px-2 py-1 text-sm text-ink focus:border-info focus:outline-none"
        >
          {HANDOFF_TARGETS.map((target) => (
            <option key={target} value={target}>
              {target}
            </option>
          ))}
        </select>
        <button
          type="button"
          data-testid="tcc-handoff-btn"
          onClick={handleHandoff}
          className="shrink-0 rounded-md border border-line bg-bg-soft px-2.5 py-1 text-sm text-ink transition-colors hover:border-ink-mute"
        >
          交接
        </button>
      </div>

      {/* ⑥ 诚实徽标 */}
      <div
        data-testid="tcc-honest"
        className="mt-2 rounded border border-warn/30 bg-warn/10 px-2 py-1 text-xs leading-relaxed text-warn"
      >
        命令执行未接入 — 内容同步至聊天话题, 宿主 agent 网关接入后真实执行
      </div>
    </div>
  );
}
