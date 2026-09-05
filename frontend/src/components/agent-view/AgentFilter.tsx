"use client";

// =====================================================================
// AgentFilter — 顶部 agent 筛选器 (dropdown)
// =====================================================================
// Per 2026-09-05 11:25 JST 拍板 #2: 默认选最近活跃的 active agent, 用户可手动
//   覆盖 (dropdown 或 URL ?agent=). 组件只负责 UI 切换, 不解析 URL — URL
//   解析交给 page (useSearchParams), 这里接收 onChange.
// =====================================================================

import type { AgentSession, AgentStatus } from "@/types/ids";
import { isActiveAgent } from "@/lib/agent-view/selectors";
import { ChevronDown, Bot, Sparkles, Circle } from "lucide-react";
import { useState, useRef, useEffect } from "react";

interface AgentFilterProps {
  agents: ReadonlyArray<AgentSession>;
  selectedId: string;
  /** 标识当前是 auto 选的还是手动选的 (auto 时显示 "auto" 角标) */
  auto: boolean;
  onChange: (agentId: string) => void;
}

export function AgentFilter({ agents, selectedId, auto, onChange }: AgentFilterProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  // 点击外部关闭
  useEffect(() => {
    if (!open) return;
    const onClickOutside = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    };
    document.addEventListener("mousedown", onClickOutside);
    return () => document.removeEventListener("mousedown", onClickOutside);
  }, [open]);

  const selected = agents.find((a) => a.id === selectedId) ?? null;

  // 按 [active 优先, started_at desc, id asc] 排序
  const sorted = [...agents].sort((a, b) => {
    const aActive = isActiveAgent(a) ? 0 : 1;
    const bActive = isActiveAgent(b) ? 0 : 1;
    if (aActive !== bActive) return aActive - bActive;
    if (a.started_at !== b.started_at) return a.started_at < b.started_at ? 1 : -1;
    return a.id < b.id ? -1 : 1;
  });

  return (
    <div className="relative" ref={ref} data-testid="agent-filter">
      <button
        type="button"
        onClick={() => setOpen((v) => !v)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm rounded-md border border-line bg-bg-soft hover:bg-bg-card transition-colors min-w-[200px]"
        aria-haspopup="listbox"
        aria-expanded={open}
        data-testid="agent-filter-trigger"
      >
        <Bot size={14} className="text-info shrink-0" />
        <span className="flex-1 text-left truncate">
          {selected ? (
            <>
              <span className="font-mono text-info">{selected.id}</span>
              <span className="text-ink-mute ml-2 text-xs">· {selected.agent_kind}</span>
            </>
          ) : (
            <span className="text-ink-mute">Select agent…</span>
          )}
        </span>
        {auto && (
          <span
            data-testid="agent-filter-auto-badge"
            className="text-[9px] uppercase tracking-wider px-1 py-0.5 rounded border border-info/40 text-info bg-info/10"
            title="Auto-selected as the most recently active agent"
          >
            auto
          </span>
        )}
        <ChevronDown size={12} className="text-ink-mute shrink-0" />
      </button>

      {open && (
        <div
          data-testid="agent-filter-dropdown"
          className="absolute z-30 top-full mt-1 left-0 w-[320px] max-h-[420px] overflow-y-auto bg-bg-card border border-line rounded-md shadow-lg"
          role="listbox"
        >
          <div className="sticky top-0 px-3 py-1.5 text-[10px] text-ink-mute uppercase tracking-wider bg-bg-card/95 backdrop-blur-sm border-b border-line">
            {agents.length} agent{agents.length === 1 ? "" : "s"} · active first
          </div>
          {sorted.length === 0 ? (
            <div className="px-3 py-4 text-xs text-ink-mute text-center">No agents available</div>
          ) : (
            sorted.map((a) => {
              const isSelected = a.id === selectedId;
              const active = isActiveAgent(a);
              return (
                <button
                  key={a.id}
                  type="button"
                  role="option"
                  aria-selected={isSelected}
                  data-testid={`agent-filter-option-${a.id}`}
                  onClick={() => {
                    onChange(a.id);
                    setOpen(false);
                  }}
                  className={`w-full flex items-center gap-2 px-3 py-2 text-left text-xs hover:bg-bg-soft transition-colors ${isSelected ? "bg-info/10" : ""}`}
                >
                  <StatusDot status={a.status} />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-1.5">
                      <span className="font-mono text-info truncate">{a.id}</span>
                      <span className="text-ink-mute text-[10px]">· {a.agent_kind}</span>
                    </div>
                    <div className="text-[10px] text-ink-mute truncate">
                      {a.status} · {a.current_step}
                    </div>
                  </div>
                  {active && <span title="active"><Sparkles size={10} className="text-info shrink-0" /></span>}
                </button>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}

function StatusDot({ status }: { status: AgentStatus }) {
  const color = (() => {
    switch (status) {
      case "executing":
      case "compiling_context":
      case "planning":
      case "validating":
      case "queued":
      case "spawning":
      case "initializing":
        return "bg-info";
      case "awaiting_human":
      case "awaiting_feedback":
      case "awaiting_tool":
      case "paused":
        return "bg-warn";
      case "failed":
        return "bg-err";
      case "completed":
      case "cancelled":
        return "bg-ok";
      default:
        return "bg-ink-mute";
    }
  })();
  return <Circle size={8} className={`${color} shrink-0`} fill="currentColor" />;
}
