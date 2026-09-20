// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/chat/useChatStream.ts — ULYS-98-W3.3
"use client";
import { useCallback, useEffect, useRef, useState } from "react";
const BFF_BASE_URL = process.env.NEXT_PUBLIC_BFF_URL ?? "http://localhost:8080";
export interface StreamChunk { delta: string; done: boolean; usage?: { input_tokens: number; output_tokens: number; total_tokens: number } | null; error?: string | null; }
export interface UseChatStreamOptions { sessionId: string; userId: string; content: string; model?: string; trigger?: boolean; }
export interface UseChatStreamResult { text: string; streaming: boolean; error: string | null; reset: () => void; }
export function useChatStream(opts: UseChatStreamOptions): UseChatStreamResult {
  const [text, setText] = useState("");
  const [streaming, setStreaming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const esRef = useRef<EventSource | null>(null);
  const reset = useCallback(() => { setText(""); setError(null); setStreaming(false); if (esRef.current) { esRef.current.close(); esRef.current = null; } }, []);
  useEffect(() => {
    if (!opts.trigger) return;
    if (opts.content.trim() === "") return;
    reset(); setStreaming(true);
    const params = new URLSearchParams({ session_id: opts.sessionId, user_id: opts.userId, content: opts.content });
    if (opts.model) params.set("model", opts.model);
    const url = `${BFF_BASE_URL}/v1/chat/stream?${params.toString()}`;
    const es = new EventSource(url); esRef.current = es;
    es.onmessage = (ev) => {
      try {
        const chunk = JSON.parse(ev.data) as StreamChunk;
        if (chunk.error) { setError(chunk.error); setStreaming(false); es.close(); esRef.current = null; return; }
        if (chunk.delta) setText((prev) => prev + chunk.delta);
        if (chunk.done) { setStreaming(false); es.close(); esRef.current = null; }
      } catch (e) { const msg = e instanceof Error ? e.message : String(e); setError(`Failed to parse SSE chunk: ${msg}`); setStreaming(false); es.close(); esRef.current = null; }
    };
    es.onerror = () => { setError("SSE connection error"); setStreaming(false); es.close(); esRef.current = null; };
    return () => { es.close(); esRef.current = null; };
  }, [opts.trigger, opts.sessionId, opts.userId, opts.content, opts.model, reset]);
  return { text, streaming, error, reset };
}
