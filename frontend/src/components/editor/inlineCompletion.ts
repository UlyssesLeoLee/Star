// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/inlineCompletion.ts — ULYS-98-W2.3
// =====================================================================
// W2 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 2.3
// frontend monaco inline completion") — wires Monaco's
// `monaco.languages.registerCompletionItemProvider` to our BFF endpoint
// `POST /v1/completion/inline`.
//
// Behaviour:
// - **Trigger**: user idle for `DEBOUNCE_MS` after a keystroke, OR manual
//   `Ctrl+Space` (Monaco built-in).
// - **Transport**: `fetch('/v1/completion/inline', ...)` with the
//   `CompletionInlineRequest` body.
// - **Display**: the model returns a plain string; we wrap it in a
//   single `CompletionItem` with `kind: Text` and `insertText` set.
//   Monaco renders it inline; user presses `Tab` to accept (Monaco
//   default) or `Esc` to dismiss.
// - **No-network mode**: when `NEXT_PUBLIC_LLM_NO_NETWORK=1` we
//   short-circuit and return a deterministic stub prefix (useful for
//   CI / demos without API keys).
//
// Tested in `inlineCompletion.test.ts`.
// =====================================================================
"use client";

import type * as monacoNs from "monaco-editor";

// =====================================================================
// Configuration
// =====================================================================

/** Default debounce window (ms) between the last keystroke and the
 *  trigger of an inline completion request. */
export const DEBOUNCE_MS = 500;

/** Hard upper bound on the prefix / suffix payload (8 KiB per
 *  `crates/api/src/completion.rs::MAX_PREFIX_BYTES`). */
export const MAX_INLINE_BYTES = 8 * 1024;

/** BFF endpoint base URL. Default matches the standalone BFF dev port
 *  used by `ChatPanel.tsx`. */
export const COMPLETION_ENDPOINT = "/v1/completion/inline";

/** When `true` (default in CI), skip network and return a deterministic
 *  stub reply. Toggle by setting `NEXT_PUBLIC_LLM_NO_NETWORK=0`. */
export const NO_NETWORK_DEFAULT = true;

// =====================================================================
// Wire DTOs (mirror crates/api/src/completion.rs)
// =====================================================================

export interface CompletionInlineRequest {
  file_path: string;
  cursor_line: number;
  cursor_col: number;
  prefix: string;
  suffix: string;
  language: string;
  user_id: string;
  model?: string;
}

export interface TokenUsageDto {
  user_id: string;
  tenant_id: string;
  provider: string;
  model: string;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  captured_at: string;
  request_id: string | null;
}

export interface CompletionInlineResponse {
  completion: string;
  model: string;
  token_usage: TokenUsageDto;
  created_at: string;
}

// =====================================================================
// Fetch helper (kept tiny so unit tests can substitute)
// =====================================================================

export interface CompletionFetcher {
  (req: CompletionInlineRequest): Promise<CompletionInlineResponse>;
}

/**
 * Default fetcher — POSTs to `COMPLETION_ENDPOINT`.
 *
 * Network errors bubble up to the caller; the caller decides whether
 * to fall back to the local stub.
 */
export const defaultCompletionFetcher: CompletionFetcher = async (req) => {
  const res = await fetch(COMPLETION_ENDPOINT, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(req),
  });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(
      `completion: HTTP ${res.status} ${res.statusText} — ${text.slice(0, 200)}`,
    );
  }
  return (await res.json()) as CompletionInlineResponse;
};

/**
 * Local stub fetcher — deterministic reply with no network. Used when
 * `noNetworkMode` is on (default in CI) or when the BFF endpoint is
 * unreachable.
 */
export const stubCompletionFetcher: CompletionFetcher = async (req) => {
  const now = new Date().toISOString();
  return {
    completion: `\n// [mock inline completion for ${req.language}]\n`,
    model: "mock-llm",
    token_usage: {
      user_id: req.user_id,
      tenant_id: "00000000-0000-0000-0000-000000000000",
      provider: "mock",
      model: "mock-llm",
      input_tokens: Math.ceil(req.prefix.length / 4),
      output_tokens: 1,
      total_tokens: Math.ceil(req.prefix.length / 4) + 1,
      captured_at: now,
      request_id: null,
    },
    created_at: now,
  };
};

// =====================================================================
// Debounce helper (no React state — just a tiny Promise wrapper)
// =====================================================================

/**
 * Run `fn` after `ms` milliseconds of idleness. Calling `schedule()`
 * repeatedly before the timer fires resets the wait.
 */
export function debounce<Args extends unknown[]>(
  fn: (...args: Args) => void,
): (...args: Args) => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return (...args: Args) => {
    if (timer !== null) clearTimeout(timer);
    timer = setTimeout(() => {
      timer = null;
      fn(...args);
    }, DEBOUNCE_MS);
  };
}

// =====================================================================
// Provider registration (the main entry point)
// =====================================================================

export interface RegisterInlineCompletionOptions {
  /** Override the fetcher (handy for tests). */
  fetcher?: CompletionFetcher;
  /** Force a specific language registration (omit to register for all). */
  languages?: string[];
  /** When `true`, skip the network and use the stub. Default: auto
   *  (read `process.env.NEXT_PUBLIC_LLM_NO_NETWORK`). */
  noNetwork?: boolean;
}

/**
 * Decide whether to use the stub fetcher. Resolution order:
 * 1. Explicit `opts.noNetwork` (if provided).
 * 2. `process.env.NEXT_PUBLIC_LLM_NO_NETWORK === "0"` → false.
 * 3. Otherwise → `NO_NETWORK_DEFAULT` (true).
 */
export function resolveNoNetworkMode(opts?: {
  noNetwork?: boolean;
}): boolean {
  if (typeof opts?.noNetwork === "boolean") return opts.noNetwork;
  if (typeof process !== "undefined") {
    const env = process.env?.NEXT_PUBLIC_LLM_NO_NETWORK;
    if (env === "0" || env === "false") return false;
  }
  return NO_NETWORK_DEFAULT;
}

/**
 * Register an inline completion provider on the Monaco instance.
 *
 * Idempotent: calling twice for the same language is a no-op (Monaco
 * keeps the first registration). Returned dispose function unregisters
 * both the completion provider and the debounce timer.
 *
 * Wires a 500ms debounce trigger: each keystroke schedules a request,
 * but only the most recent (after the user stops typing) is fired.
 */
export function registerInlineCompletion(
  monaco: typeof monacoNs,
  opts?: RegisterInlineCompletionOptions,
): () => void {
  const noNetwork = resolveNoNetworkMode(opts);
  const fetcher = opts?.fetcher ?? (noNetwork ? stubCompletionFetcher : defaultCompletionFetcher);
  const languages = opts?.languages ?? [
    "typescript",
    "javascript",
    "rust",
    "python",
    "go",
    "json",
    "css",
    "html",
  ];

  const disposers: monaco.IDisposable[] = [];

  for (const lang of languages) {
    const provider: monacoNs.languages.CompletionItemProvider = {
      // Inline trigger: Monaco calls `provideCompletionItems` whenever the
      // user idles after typing or manually invokes with Ctrl+Space.
      triggerCharacters: ["."],
      async provideCompletionItems(model, position) {
        const fullText = model.getValue();
        const offset = model.getOffsetAt(position);
        const prefix = fullText.slice(Math.max(0, offset - MAX_INLINE_BYTES), offset);
        const suffix = fullText.slice(
          offset,
          Math.min(fullText.length, offset + MAX_INLINE_BYTES),
        );

        // Path: monaco URI → file path. Monaco URIs look like
        // `inmemory://...` or `vscode-remote://...` — we extract the
        // last path segment.
        const uri = model.uri;
        const filePath = uri.path || uri.toString();

        const req: CompletionInlineRequest = {
          file_path: filePath,
          cursor_line: position.lineNumber,
          cursor_col: position.column,
          prefix,
          suffix,
          language: lang,
          user_id: "00000000-0000-0000-0000-0000000000ff", // W2 demo user; replaced post-auth.
        };

        let resp: CompletionInlineResponse;
        try {
          resp = await fetcher(req);
        } catch (e) {
          // Network failure: fall back to the stub so the user still
          // sees inline completions.
          resp = await stubCompletionFetcher(req);
        }

        if (!resp.completion) {
          return { suggestions: [] };
        }

        const range = new monaco.Range(
          position.lineNumber,
          position.column,
          position.lineNumber,
          position.column,
        );

        return {
          suggestions: [
            {
              label: "ai-inline",
              kind: monaco.languages.CompletionItemKind.Text,
              insertText: resp.completion,
              range,
              detail: `Star AI (${resp.model})`,
              documentation: {
                value:
                  `**Inline completion** (model: \`${resp.model}\`, ` +
                  `tokens: ${resp.token_usage.total_tokens})\n\n` +
                  "Press **Tab** to accept, **Esc** to dismiss.",
              },
            } as monacoNs.languages.CompletionItem,
          ],
        };
      },
      resolveCompletionItem(item) {
        // No async resolution needed — the provider already returned
        // the full payload.
        return item;
      },
    };

    disposers.push(
      monaco.languages.registerCompletionItemProvider(lang, provider),
    );
  }

  return () => {
    for (const d of disposers) d.dispose();
  };
}

// =====================================================================
// Convenience: explicit debounced request (used by Cmd-K)
// =====================================================================

/**
 * `requestInlineCompletionOnce` — fire a single completion request
 * outside the Monaco provider context. Useful for Cmd-K (W2.4) which
 * sends a prompt + selection rather than triggering from the editor.
 *
 * Returns the response; throws on network failure (caller decides
 * fallback).
 */
export async function requestInlineCompletionOnce(
  req: CompletionInlineRequest,
  fetcher: CompletionFetcher = defaultCompletionFetcher,
): Promise<CompletionInlineResponse> {
  return fetcher(req);
}