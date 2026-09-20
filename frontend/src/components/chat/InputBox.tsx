// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/chat/InputBox.tsx — ULYS-98-W1.2
// (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.2 frontend Chat panel UI")
"use client";

import {
  KeyboardEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

// =====================================================================
// Types
// =====================================================================

export interface InputBoxProps {
  /** Send handler — invoked with the trimmed content + clears the field. */
  onSend: (content: string) => void;
  /** Whether a reply is currently in-flight (disables input). */
  disabled?: boolean;
  /** Placeholder when idle. */
  placeholder?: string;
}

// =====================================================================
// InputBox
// =====================================================================

/**
 * **InputBox** — multi-line textarea + Enter to send (Shift+Enter newline).
 *
 * W1: no autocomplete, no slash command, no model picker. W2 adds @-mention
 * and /command shortcuts.
 */
export default function InputBox({
  onSend,
  disabled = false,
  placeholder = "Ask the AI agent…",
}: InputBoxProps) {
  const [value, setValue] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement | null>(null);

  // Auto-grow up to ~6 rows.
  useEffect(() => {
    const el = textareaRef.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.min(el.scrollHeight, 144)}px`;
  }, [value]);

  const submit = useCallback(() => {
    const trimmed = value.trim();
    if (!trimmed || disabled) return;
    onSend(trimmed);
    setValue("");
  }, [value, disabled, onSend]);

  const onKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        submit();
      }
    },
    [submit],
  );

  return (
    <div
      className="border-t border-line p-2 flex gap-2 items-end"
      data-testid="chat-input"
    >
      <textarea
        ref={textareaRef}
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={onKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        rows={1}
        className="flex-1 resize-none rounded border border-line bg-bg px-2 py-1.5 text-sm focus:outline-none focus:ring-1 focus:ring-info disabled:opacity-50"
        data-testid="chat-input-textarea"
        aria-label="Chat message input"
      />
      <button
        type="button"
        onClick={submit}
        disabled={disabled || !value.trim()}
        className="rounded bg-info px-3 py-1.5 text-xs font-medium text-white disabled:opacity-40"
        data-testid="chat-input-send"
      >
        Send
      </button>
    </div>
  );
}