// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/chat/ChatPanel.tsx — ULYS-98-W1.2
// (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.2 frontend Chat panel UI")
"use client";

import {
  KeyboardEvent as ReactKeyboardEvent,
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from "react";

import InputBox from "./InputBox";
import MessageList, { type ChatMessage } from "./MessageList";

// =====================================================================
// BFF endpoint config
// =====================================================================

/**
 * Resolve the BFF base URL at runtime.
 *
 * Default: `http://localhost:8080` (matches the standalone BFF dev port per
 * ULYS-57.4 T13). W1 demo without a running BFF falls back to the local
 * stub reply — see `localStubReply`.
 *
 * Set `NEXT_PUBLIC_BFF_URL=https://bff.example.com` to point at a real
 * deployment.
 */
const BFF_BASE_URL =
  process.env.NEXT_PUBLIC_BFF_URL ?? "http://localhost:8080";

/**
 * W1 stub reply — when the BFF endpoint is unreachable, the panel still
 * renders a deterministic stub so the UI can be demoed without infra.
 * Matches the BFF stub constant `bff::routes::chat` ("AI 暂未接入").
 */
const LOCAL_STUB_REPLY = "AI 暂未接入";

// =====================================================================
// Props
// =====================================================================

export interface ChatPanelProps {
  /**
   * Open by default (W1: this is fine — controlled mode just sets initial state).
   */
  defaultOpen?: boolean;
  /** Side of the screen (right drawer vs left). Default `"right"`. */
  side?: "right" | "left";
  /** Width (CSS). Default `"min(420px, 100vw)"`. */
  width?: string;
  /** Override keyboard shortcut (default `"l"` with `Ctrl` / `Cmd`). */
  shortcutKey?: string;
  /**
   * Optional session id. If omitted, a UUID v4 is generated client-side
   * (per brief §1.3 `session_id: UUID, 前端生成`).
   */
  sessionId?: string;
}

// =====================================================================
// ChatPanel
// =====================================================================

/**
 * **ChatPanel** — right-side drawer, Ctrl/Cmd+L to toggle, sends to BFF
 * `POST /v1/chat/send`, receives `"AI 暂未接入"` stub reply (per brief §1.3
 * + §1.2). Falls back to local stub when BFF is unreachable so W1 demos
 * work without infra.
 */
export default function ChatPanel({
  defaultOpen = false,
  side = "right",
  width = "min(420px, 100vw)",
  shortcutKey = "l",
  sessionId,
}: ChatPanelProps) {
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [pending, setPending] = useState(false);
  const titleId = useId();

  // Generate session id lazily once (per brief §1.3 "前端生成 UUID").
  const liveSessionId = useMemo(() => {
    if (sessionId) return sessionId;
    if (
      typeof crypto !== "undefined" &&
      typeof crypto.randomUUID === "function"
    ) {
      return crypto.randomUUID();
    }
    return `sess-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
  }, [sessionId]);

  // ===========================================================
  // Global keyboard shortcut: Ctrl+L (Win/Linux) / Cmd+L (mac)
  // ===========================================================
  useEffect(() => {
    function onKey(e: globalThis.KeyboardEvent) {
      const modKey = e.ctrlKey || e.metaKey;
      if (modKey && e.key.toLowerCase() === shortcutKey.toLowerCase()) {
        e.preventDefault();
        setIsOpen((v) => !v);
      } else if (e.key === "Escape" && isOpen) {
        setIsOpen(false);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOpen, shortcutKey]);

  // ===========================================================
  // Send handler
  // ===========================================================
  const send = useCallback(
    async (content: string) => {
      const userMsg: ChatMessage = {
        id: `local-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`,
        role: "user",
        content,
        createdAt: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, userMsg]);
      setPending(true);
      try {
        const reply = await postChatMessage(liveSessionId, "user-1", content);
        const assistantMsg: ChatMessage = {
          id: `srv-${reply.messageId}`,
          role: "assistant",
          content: reply.assistantContent,
          createdAt: reply.createdAt,
        };
        setMessages((prev) => [...prev, assistantMsg]);
      } catch (e) {
        const errMsg: ChatMessage = {
          id: `err-${Date.now()}`,
          role: "system",
          content:
            e instanceof Error
              ? `BFF unreachable (${e.message}); using local stub reply.`
              : "BFF unreachable; using local stub reply.",
          createdAt: new Date().toISOString(),
        };
        const stub: ChatMessage = {
          id: `stub-${Date.now()}`,
          role: "assistant",
          content: LOCAL_STUB_REPLY,
          createdAt: new Date().toISOString(),
        };
        setMessages((prev) => [...prev, errMsg, stub]);
      } finally {
        setPending(false);
      }
    },
    [liveSessionId],
  );

  // ===========================================================
  // Render
  // ===========================================================
  return (
    <>
      {/* Toggle button — always visible */}
      <button
        type="button"
        onClick={() => setIsOpen((v) => !v)}
        className="fixed bottom-4 right-4 z-40 rounded-full bg-info px-3 py-2 text-xs font-medium text-white shadow-lg"
        aria-label={isOpen ? "Close chat panel" : "Open chat panel"}
        aria-expanded={isOpen}
        aria-controls={titleId}
        data-testid="chat-toggle"
        title={`${typeof navigator !== "undefined" && /Mac/i.test(navigator.platform) ? "Cmd" : "Ctrl"}+${shortcutKey.toUpperCase()}`}
      >
        {isOpen ? "× Chat" : "💬 Chat"}
      </button>

      {/* Drawer */}
      <aside
        id={titleId}
        role="dialog"
        aria-labelledby={`${titleId}-title`}
        aria-hidden={!isOpen}
        className={
          "fixed top-0 z-30 h-full bg-bg border-line shadow-xl transition-transform duration-200 " +
          (side === "right" ? "right-0 border-l" : "left-0 border-r") +
          " " +
          (isOpen
            ? "translate-x-0"
            : side === "right"
              ? "translate-x-full"
              : "-translate-x-full")
        }
        style={{ width }}
        data-testid="chat-panel"
        data-open={isOpen ? "true" : "false"}
        data-side={side}
      >
        <header className="flex items-center justify-between border-b border-line px-3 py-2">
          <div>
            <div
              id={`${titleId}-title`}
              className="text-sm font-semibold"
            >
              AI Chat
            </div>
            <div className="text-[10px] font-mono text-ink-mute">
              {liveSessionId.slice(0, 8)}…
            </div>
          </div>
          <button
            type="button"
            onClick={() => setIsOpen(false)}
            className="text-ink-mute hover:text-ink text-sm"
            aria-label="Close chat panel"
            data-testid="chat-close"
          >
            ×
          </button>
        </header>

        <div className="flex h-[calc(100%-7rem)] flex-col">
          <MessageList
            messages={messages}
            emptyHint="Send a message — the AI agent will reply (W1 stub: AI 暂未接入)."
          />
          <InputBox onSend={send} disabled={pending} />
        </div>

        <footer className="border-t border-line px-3 py-1.5 text-[10px] font-mono text-ink-mute">
          {BFF_BASE_URL}/v1/chat/send · W1 stub
        </footer>
      </aside>
    </>
  );
}

// =====================================================================
// BFF transport (per brief §1.2 + §1.3)
// =====================================================================

interface ChatSendResponse {
  session_id: string;
  message_id: string;
  assistant_content: string;
  created_at: string;
}

/**
 * POST /v1/chat/send — sends the user message and returns the stub assistant
 * envelope. Throws on network / non-2xx; caller falls back to local stub.
 */
async function postChatMessage(
  sessionId: string,
  userId: string,
  content: string,
): Promise<{
  messageId: string;
  assistantContent: string;
  createdAt: string;
}> {
  const url = `${BFF_BASE_URL}/v1/chat/send`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      session_id: sessionId,
      user_id: userId,
      content: content,
    }),
    // 5 s ceiling — fail fast into local stub fallback.
    signal: AbortSignal.timeout(5000),
  });
  if (!res.ok) {
    throw new Error(`POST ${url} → HTTP ${res.status}`);
  }
  const body = (await res.json()) as ChatSendResponse;
  return {
    messageId: body.message_id,
    assistantContent: body.assistant_content,
    createdAt: body.created_at,
  };
}

// =====================================================================
// Default re-exports
// =====================================================================

export { InputBox, MessageList };
export type { ChatMessage };