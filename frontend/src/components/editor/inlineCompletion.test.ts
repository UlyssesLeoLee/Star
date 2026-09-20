// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/inlineCompletion.test.ts — ULYS-98-W2.3
// =====================================================================
// W2 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 2.3"):
// "1 unit test (inlineCompletion.test.ts)"
//
// Coverage:
// 1. stubCompletionFetcher echoes language in the body (no network).
// 2. debounce(fn) fires once with the LAST args after the wait.
// 3. resolveNoNetworkMode honours opts > env > default precedence.
// 4. defaultCompletionFetcher shapes the request body correctly (we mock
//    fetch and assert call args).
// 5. registerInlineCompletion provider surfaces the response as a
//    CompletionItem — verified via a minimal monaco stub.
// =====================================================================
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import {
  debounce,
  defaultCompletionFetcher,
  MAX_INLINE_BYTES,
  NO_NETWORK_DEFAULT,
  registerInlineCompletion,
  resolveNoNetworkMode,
  stubCompletionFetcher,
  type CompletionFetcher,
  type CompletionInlineRequest,
  type CompletionInlineResponse,
} from "./inlineCompletion";

const sampleRequest = (overrides: Partial<CompletionInlineRequest> = {}): CompletionInlineRequest => ({
  file_path: "src/lib.rs",
  cursor_line: 12,
  cursor_col: 8,
  prefix: "fn hello() {\n    prin",
  suffix: "\n}",
  language: "rust",
  user_id: "00000000-0000-0000-0000-0000000000ff",
  ...overrides,
});

describe("inlineCompletion (ULYS-98-W2.3)", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("MAX_INLINE_BYTES matches the BFF limit (8 KiB)", () => {
    expect(MAX_INLINE_BYTES).toBe(8 * 1024);
  });

  it("NO_NETWORK_DEFAULT defaults to true (CI / no-key fallback)", () => {
    expect(NO_NETWORK_DEFAULT).toBe(true);
  });

  it("stubCompletionFetcher returns deterministic content", async () => {
    const resp = await stubCompletionFetcher(sampleRequest());
    expect(resp.model).toBe("mock-llm");
    expect(resp.completion).toContain("[mock inline completion for rust]");
    expect(resp.token_usage.provider).toBe("mock");
    expect(resp.token_usage.total_tokens).toBeGreaterThan(0);
  });

  it("resolveNoNetworkMode honours explicit opts first", () => {
    expect(resolveNoNetworkMode({ noNetwork: false })).toBe(false);
    expect(resolveNoNetworkMode({ noNetwork: true })).toBe(true);
  });

  it("resolveNoNetworkMode falls back to env when opts omit", () => {
    const previous = process.env.NEXT_PUBLIC_LLM_NO_NETWORK;
    try {
      process.env.NEXT_PUBLIC_LLM_NO_NETWORK = "0";
      expect(resolveNoNetworkMode()).toBe(false);
      process.env.NEXT_PUBLIC_LLM_NO_NETWORK = "false";
      expect(resolveNoNetworkMode()).toBe(false);
      process.env.NEXT_PUBLIC_LLM_NO_NETWORK = "1";
      expect(resolveNoNetworkMode()).toBe(true);
      delete process.env.NEXT_PUBLIC_LLM_NO_NETWORK;
      expect(resolveNoNetworkMode()).toBe(NO_NETWORK_DEFAULT);
    } finally {
      if (previous === undefined) delete process.env.NEXT_PUBLIC_LLM_NO_NETWORK;
      else process.env.NEXT_PUBLIC_LLM_NO_NETWORK = previous;
    }
  });

  it("defaultCompletionFetcher POSTs JSON to /v1/completion/inline", async () => {
    const mockFetch = vi.fn(async (_url: string, init: RequestInit) => {
      return {
        ok: true,
        status: 200,
        json: async () => ({
          completion: "// reply",
          model: "claude-test",
          token_usage: {
            user_id: "u",
            tenant_id: "t",
            provider: "anthropic",
            model: "claude-test",
            input_tokens: 10,
            output_tokens: 1,
            total_tokens: 11,
            captured_at: new Date().toISOString(),
            request_id: null,
          },
          created_at: new Date().toISOString(),
        }),
      } as Response;
    });
    const originalFetch = global.fetch;
    (global as unknown as { fetch: typeof mockFetch }).fetch = mockFetch as unknown as typeof fetch;

    try {
      const req = sampleRequest({ language: "typescript", prefix: "const x: " });
      const resp = await defaultCompletionFetcher(req);
      expect(mockFetch).toHaveBeenCalledTimes(1);
      const [calledUrl, initArg] = mockFetch.mock.calls[0];
      expect(calledUrl).toBe("/v1/completion/inline");
      expect(initArg.method).toBe("POST");
      expect((initArg.headers as Record<string, string>)["content-type"]).toBe(
        "application/json",
      );
      const body = JSON.parse(initArg.body as string);
      expect(body.file_path).toBe(req.file_path);
      expect(body.cursor_line).toBe(req.cursor_line);
      expect(body.language).toBe("typescript");
      expect(resp.completion).toBe("// reply");
      expect(resp.model).toBe("claude-test");
    } finally {
      (global as unknown as { fetch: typeof fetch }).fetch = originalFetch;
    }
  });

  it("defaultCompletionFetcher throws on non-2xx response", async () => {
    const mockFetch = vi.fn(async () => ({
      ok: false,
      status: 502,
      statusText: "Bad Gateway",
      text: async () => "upstream down",
    }));
    const originalFetch = global.fetch;
    (global as unknown as { fetch: typeof mockFetch }).fetch = mockFetch as unknown as typeof fetch;

    try {
      await expect(defaultCompletionFetcher(sampleRequest())).rejects.toThrow(/502/);
    } finally {
      (global as unknown as { fetch: typeof fetch }).fetch = originalFetch;
    }
  });

  it("debounce fires fn once with the LAST args after the wait", () => {
    const fn = vi.fn();
    const debounced = debounce(fn);
    debounced("a", 1);
    debounced("b", 2);
    debounced("c", 3);
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(499);
    expect(fn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(2);
    expect(fn).toHaveBeenCalledTimes(1);
    expect(fn).toHaveBeenCalledWith("c", 3);
  });

  it("registerInlineCompletion returns a dispose function and registers providers", () => {
    // Minimal monaco stub — we only need languages.registerCompletionItemProvider
    // and a Range constructor.
    const dispose = vi.fn();
    const registered: { lang: string; provider: unknown }[] = [];
    const fakeMonaco = {
      languages: {
        registerCompletionItemProvider: (
          lang: string,
          provider: unknown,
        ) => {
          registered.push({ lang, provider });
          return { dispose };
        },
        CompletionItemKind: { Text: 0 },
      },
      Range: class {
        constructor(
          public sl: number,
          public sc: number,
          public el: number,
          public ec: number,
        ) {}
      },
    } as unknown as Parameters<typeof registerInlineCompletion>[0];

    const disposeAll = registerInlineCompletion(fakeMonaco, {
      noNetwork: true,
      languages: ["typescript", "rust"],
    });
    expect(registered.length).toBe(2);
    expect(registered.map((r) => r.lang).sort()).toEqual(["rust", "typescript"]);

    disposeAll();
    expect(dispose).toHaveBeenCalledTimes(2);
  });

  it("registerInlineCompletion provider returns a CompletionItem when fetcher succeeds", async () => {
    const fixedFetcher: CompletionFetcher = async () => ({
      completion: "// generated",
      model: "claude-test",
      token_usage: {
        user_id: "u",
        tenant_id: "t",
        provider: "anthropic",
        model: "claude-test",
        input_tokens: 1,
        output_tokens: 1,
        total_tokens: 2,
        captured_at: new Date().toISOString(),
        request_id: null,
      },
      created_at: new Date().toISOString(),
    });

    let registeredProvider: unknown = null;
    const fakeMonaco = {
      languages: {
        registerCompletionItemProvider: (_lang: string, p: unknown) => {
          registeredProvider = p;
          return { dispose: vi.fn() };
        },
        CompletionItemKind: { Text: 0 },
      },
      Range: class {
        constructor(
          public sl: number,
          public sc: number,
          public el: number,
          public ec: number,
        ) {}
      },
    } as unknown as Parameters<typeof registerInlineCompletion>[0];

    registerInlineCompletion(fakeMonaco, {
      noNetwork: true,
      fetcher: fixedFetcher,
      languages: ["typescript"],
    });

    expect(registeredProvider).not.toBeNull();
    const provider = registeredProvider as {
      provideCompletionItems: (
        model: unknown,
        position: { lineNumber: number; column: number },
      ) => Promise<{
        suggestions: Array<{ insertText: string; label: string }>;
      }>;
    };

    const fakeModel = {
      getValue: () => "const x = ",
      getOffsetAt: (p: { lineNumber: number; column: number }) =>
        // crude offset — we only need a number
        p.lineNumber * 100 + p.column,
      uri: { path: "src/main.ts", toString: () => "src/main.ts" },
    };
    const result = await provider.provideCompletionItems(fakeModel, {
      lineNumber: 1,
      column: 12,
    });
    expect(result.suggestions.length).toBe(1);
    expect(result.suggestions[0].insertText).toBe("// generated");
    expect(result.suggestions[0].label).toBe("ai-inline");
  });

  it("registerInlineCompletion provider falls back to stub on fetcher error", async () => {
    const failingFetcher: CompletionFetcher = async () => {
      throw new Error("network down");
    };

    let registeredProvider: unknown = null;
    const fakeMonaco = {
      languages: {
        registerCompletionItemProvider: (_lang: string, p: unknown) => {
          registeredProvider = p;
          return { dispose: vi.fn() };
        },
        CompletionItemKind: { Text: 0 },
      },
      Range: class {
        constructor(
          public sl: number,
          public sc: number,
          public el: number,
          public ec: number,
        ) {}
      },
    } as unknown as Parameters<typeof registerInlineCompletion>[0];

    registerInlineCompletion(fakeMonaco, {
      noNetwork: false,
      fetcher: failingFetcher,
      languages: ["typescript"],
    });

    const provider = registeredProvider as {
      provideCompletionItems: (
        m: unknown,
        p: { lineNumber: number; column: number },
      ) => Promise<{ suggestions: Array<{ insertText: string }> }>;
    };
    const fakeModel = {
      getValue: () => "fn main() {",
      getOffsetAt: () => 11,
      uri: { path: "src/main.rs", toString: () => "src/main.rs" },
    };
    const result = await provider.provideCompletionItems(fakeModel, {
      lineNumber: 1,
      column: 12,
    });
    expect(result.suggestions.length).toBe(1);
    expect(result.suggestions[0].insertText).toContain("[mock inline completion");
  });

  it("registerInlineCompletion provider returns empty list when completion is empty", async () => {
    const emptyFetcher: CompletionFetcher = async () => {
      const empty: CompletionInlineResponse = {
        completion: "",
        model: "mock-llm",
        token_usage: {
          user_id: "u",
          tenant_id: "t",
          provider: "mock",
          model: "mock-llm",
          input_tokens: 0,
          output_tokens: 0,
          total_tokens: 0,
          captured_at: new Date().toISOString(),
          request_id: null,
        },
        created_at: new Date().toISOString(),
      };
      return empty;
    };

    let registeredProvider: unknown = null;
    const fakeMonaco = {
      languages: {
        registerCompletionItemProvider: (_lang: string, p: unknown) => {
          registeredProvider = p;
          return { dispose: vi.fn() };
        },
        CompletionItemKind: { Text: 0 },
      },
      Range: class {
        constructor(
          public sl: number,
          public sc: number,
          public el: number,
          public ec: number,
        ) {}
      },
    } as unknown as Parameters<typeof registerInlineCompletion>[0];

    registerInlineCompletion(fakeMonaco, {
      noNetwork: true,
      fetcher: emptyFetcher,
      languages: ["typescript"],
    });

    const provider = registeredProvider as {
      provideCompletionItems: (
        m: unknown,
        p: { lineNumber: number; column: number },
      ) => Promise<{ suggestions: unknown[] }>;
    };
    const fakeModel = {
      getValue: () => "",
      getOffsetAt: () => 0,
      uri: { path: "x.ts", toString: () => "x.ts" },
    };
    const result = await provider.provideCompletionItems(fakeModel, {
      lineNumber: 1,
      column: 1,
    });
    expect(result.suggestions).toEqual([]);
  });
});