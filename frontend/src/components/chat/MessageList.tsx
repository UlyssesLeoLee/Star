// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/chat/MessageList.tsx — ULYS-98-W1.2
// (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.2 frontend Chat panel UI")
"use client";

import { useEffect, useRef } from "react";

// =====================================================================
// Types
// =====================================================================

export interface ChatMessage {
  /** Unique id (uuid). Server-issued for assistant / user messages. */
  id: string;
  /** Speaker role. */
  role: "user" | "assistant" | "system";
  /** UTF-8 text content. */
  content: string;
  /** ISO timestamp. */
  createdAt: string;
}

export interface MessageListProps {
  messages: ChatMessage[];
  /** Empty-state hint shown when `messages.length === 0`. */
  emptyHint?: string;
}

// =====================================================================
// MessageList
// =====================================================================

/**
 * **MessageList** — render an ordered transcript with auto-scroll-to-bottom.
 *
 * W1 stub: simple bubble list, no markdown / no syntax highlight. W2 will add
 * markdown render + code-block copy buttons.
 */
export default function MessageList({
  messages,
  emptyHint = "Send a message to start the conversation.",
}: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const el = bottomRef.current;
    if (!el) return;
    // jsdom (Vitest) does not ship scrollIntoView — guard for both real browsers
    // and the test environment (per 守门 #11 缺标比错标 — explicit guard,
    // not a silent no-op fallback).
    const scrollIntoView = (
      el as HTMLElement & { scrollIntoView?: (opts?: unknown) => void }
    ).scrollIntoView;
    if (typeof scrollIntoView === "function") {
      scrollIntoView.call(el, { behavior: "smooth", block: "end" });
    } else {
      // Test env: assign scrollTop directly.
      const scrollParent = el.parentElement;
      if (scrollParent) scrollParent.scrollTop = scrollParent.scrollHeight;
    }
  }, [messages.length]);

  if (messages.length === 0) {
    return (
      <div
        className="flex h-full items-center justify-center px-4 text-center text-xs text-ink-mute"
        data-testid="message-list-empty"
      >
        {emptyHint}
      </div>
    );
  }

  return (
    <div
      className="flex flex-col gap-2 p-3 overflow-y-auto"
      data-testid="message-list"
      role="log"
      aria-live="polite"
    >
      {messages.map((m) => (
        <ChatBubble key={m.id} message={m} />
      ))}
      <div ref={bottomRef} />
    </div>
  );
}

// =====================================================================
// ChatBubble (internal)
// =====================================================================

function ChatBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";
  const isSystem = message.role === "system";

  if (isSystem) {
    return (
      <div
        className="self-center rounded bg-rose-50 px-2 py-1 text-[10px] text-rose-700"
        data-testid="message-bubble-system"
      >
        {message.content}
      </div>
    );
  }

  return (
    <div
      className={
        "max-w-[85%] rounded-lg px-3 py-2 text-sm whitespace-pre-wrap break-words " +
        (isUser
          ? "self-end bg-info/15 text-ink"
          : "self-start bg-line/40 text-ink")
      }
      data-testid={`message-bubble-${message.role}`}
      data-message-id={message.id}
    >
      <div className="text-[10px] font-mono uppercase tracking-wider text-ink-mute mb-1">
        {message.role}
      </div>
      {message.content}
    </div>
  );
}