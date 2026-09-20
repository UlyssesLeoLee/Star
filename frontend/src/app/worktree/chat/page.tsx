// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/worktree/chat/page.tsx — ULYS-98-W1.2
// (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.2 frontend Chat panel UI")
//
// W1 stub page — mounts ChatPanel at default location (Ctrl/Cmd+L drawer).
// W2 adds model picker, session list, slash commands.
"use client";

import { useState } from "react";
import dynamic from "next/dynamic";

import { PageHeader } from "@/components/PageHeader";
import { useTranslation } from "@/lib/i18n";

// Lazy-load the panel — keeps the initial page bundle free of the BFF fetch
// helper. SSR is disabled because ChatPanel touches `crypto.randomUUID`,
// `window`, and `AbortSignal.timeout` (browser-only).
const ChatPanel = dynamic(
  () => import("@/components/chat/ChatPanel").then((m) => m.default),
  {
    ssr: false,
    loading: () => (
      <div className="text-xs text-ink-mute p-4">Loading chat…</div>
    ),
  },
);

export default function WorktreeChatPage() {
  const { t } = useTranslation();
  const [open, setOpen] = useState(true);

  return (
    <div className="max-w-5xl space-y-4">
      <PageHeader
        title={t.pageTitles["/worktree/chat"]?.title ?? "Worktree Chat"}
        subtitle="AI Chat panel — Ctrl/Cmd+L toggle. W1 stub: backend returns 'AI 暂未接入'."
      />

      <section className="card">
        <h2 className="text-sm font-semibold mb-2">How to use</h2>
        <ul className="text-xs text-ink-dim space-y-1 list-disc list-inside">
          <li>
            Press <kbd className="font-mono">Ctrl</kbd>/
            <kbd className="font-mono">Cmd</kbd>+<kbd className="font-mono">L</kbd>
            {" "}to open or close the chat drawer.
          </li>
          <li>
            Press <kbd className="font-mono">Enter</kbd> to send,{" "}
            <kbd className="font-mono">Shift</kbd>+<kbd className="font-mono">Enter</kbd>
            {" "}to insert a newline.
          </li>
          <li>
            The panel sends <code className="font-mono">POST /v1/chat/send</code>{" "}
            to the BFF; reply text in W1 is the literal{" "}
            <code className="font-mono">"AI 暂未接入"</code>.
          </li>
          <li>
            If the BFF is unreachable the panel falls back to a local stub so
            the demo still works offline.
          </li>
        </ul>

        <div className="mt-4 flex items-center gap-2">
          <button
            type="button"
            className="btn text-xs"
            onClick={() => setOpen((v) => !v)}
            data-testid="chat-page-toggle"
          >
            {open ? "Close panel" : "Open panel"}
          </button>
          <span className="text-[10px] font-mono text-ink-mute">
            W1 stub — no real LLM yet
          </span>
        </div>
      </section>

      <ChatPanel defaultOpen={open} />
    </div>
  );
}