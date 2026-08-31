"use client";

// Star Frontend — 新 Tab Modal (选 CLI profile + 命名)
// Per 2026-08-29 04:09 JST 上轮拍板: 新建 CLI session tab

import { useState } from "react";
import { X, Terminal, Globe, Cpu, Sparkles, ChevronRight } from "lucide-react";

interface NewTabModalProps {
  onClose: () => void;
  onCreate: (data: { profileId: string; profileName: string; kind: "cli" | "api"; label: string }) => void;
}

const PROFILES: Array<{ id: string; name: string; kind: "cli" | "api"; desc: string; tag: string }> = [
  { id: "claude", name: "Claude Code", kind: "cli", desc: "Anthropic Claude 3.5 Sonnet", tag: "CLI" },
  { id: "codex", name: "OpenAI Codex", kind: "cli", desc: "GPT-4 / GPT-4 Turbo", tag: "CLI" },
  { id: "openclaw", name: "OpenClaw", kind: "api", desc: "OpenClaw API (gpt-4/o1)", tag: "API" },
  { id: "hermes", name: "Hermes", kind: "api", desc: "Hermes AI Agent API", tag: "API" },
  { id: "gemini", name: "Google Gemini", kind: "cli", desc: "Gemini Pro / Ultra", tag: "CLI" },
  { id: "aider", name: "Aider", kind: "cli", desc: "AI pair programming (GPT-4)", tag: "CLI" },
];

const KIND_ICON = {
  cli: Terminal,
  api: Globe,
} as const;

const KIND_COLOR = {
  cli: "text-[color:var(--color-primary)]",
  api: "text-[color:var(--color-warning)]",
} as const;

export function NewTabModal({ onClose, onCreate }: NewTabModalProps) {
  const [step, setStep] = useState<"select" | "name">("select");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [label, setLabel] = useState("");

  const selected = PROFILES.find((p) => p.id === selectedId);

  return (
    <div
      className="fixed inset-0 bg-black/40 flex items-center justify-center z-50"
      onClick={onClose}
      role="dialog"
      aria-modal="true"
    >
      <div
        className="w-[480px] rounded-lg border border-[color:var(--color-border)] bg-[color:var(--color-surface)] shadow-[var(--shadow-lg)] overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-[color:var(--color-border)]">
          <div className="flex items-center gap-2">
            <Sparkles size={14} className="text-[color:var(--color-primary)]" />
            <h2 className="text-sm font-semibold">{step === "select" ? "选择 CLI Agent" : "命名 Tab"}</h2>
          </div>
          <button onClick={onClose} className="opacity-60 hover:opacity-100" aria-label="close">
            <X size={14} />
          </button>
        </div>

        {/* Body */}
        {step === "select" && (
          <div className="p-3 max-h-[400px] overflow-y-auto">
            {PROFILES.map((p) => {
              const Icon = KIND_ICON[p.kind];
              return (
                <button
                  key={p.id}
                  onClick={() => { setSelectedId(p.id); setLabel(`${p.name} @ ${new Date().toLocaleTimeString()}`); setStep("name"); }}
                  className="w-full flex items-center gap-3 px-3 py-2.5 rounded border border-[color:var(--color-border)] hover:border-[color:var(--color-primary)] hover:bg-[color:var(--color-surface-2)] text-left mb-2"
                >
                  <Icon size={16} className={KIND_COLOR[p.kind]} />
                  <div className="flex-1">
                    <div className="text-sm font-medium">{p.name}</div>
                    <div className="text-[10px] text-[color:var(--color-text-dim)]">{p.desc}</div>
                  </div>
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]">
                    {p.tag}
                  </span>
                  <ChevronRight size={12} className="opacity-40" />
                </button>
              );
            })}
          </div>
        )}

        {step === "name" && selected && (
          <div className="p-4">
            <div className="flex items-center gap-2 mb-3 px-3 py-2 rounded bg-[color:var(--color-surface-2)]">
              {(() => { const I = KIND_ICON[selected.kind]; return <I size={14} className={KIND_COLOR[selected.kind]} />; })()}
              <div className="flex-1">
                <div className="text-sm font-medium">{selected.name}</div>
                <div className="text-[10px] text-[color:var(--color-text-dim)]">{selected.desc}</div>
              </div>
              <span className="text-[10px] px-1.5 py-0.5 rounded bg-[color:var(--color-surface)] text-[color:var(--color-text-dim)]">
                {selected.tag}
              </span>
            </div>
            <label className="block text-xs text-[color:var(--color-text-dim)] mb-1">Tab 名称</label>
            <input
              type="text"
              value={label}
              onChange={(e) => setLabel(e.target.value)}
              autoFocus
              className="w-full text-sm rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-3 py-2 focus:outline-none focus:border-[color:var(--color-primary)]"
            />
            <div className="flex items-center justify-between mt-4">
              <button
                onClick={() => setStep("select")}
                className="text-xs px-3 py-1.5 rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface-2)]"
              >
                返回
              </button>
              <button
                onClick={() => onCreate({ profileId: selected.id, profileName: selected.name, kind: selected.kind, label: label.trim() || selected.name })}
                disabled={!label.trim()}
                className="btn-primary"
              >
                创建 Tab
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
