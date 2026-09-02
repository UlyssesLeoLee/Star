// =====================================================================
// MSW handlers for 4 必备 LLM provider 试端点 (per ADR-0044 §3)
// =====================================================================
// 全 mock (per 2026-09-02 09:35 JST 拍板, 不接真 backend)
// 4 必备 LLM provider 试试端点 + 3 状态 (200 / 401 / 10s timeout)
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖 (复用 msw http / HttpResponse)
//   - 3 状态 试响应 走 MSW stateful 切换 (per global counter 切 200/401/timeout)
//   - 守门 #5: Mavis 接手 不读 secret, mock 返 model list 而非 真凭证
// =====================================================================

import { http, HttpResponse } from "msw";

/** Provider 试响应 状态 (3 状態, per ADR-0044 §3)
 * 0 = 200 success, 1 = 401 unauthorized, 2 = timeout
 * 每次 request 切下一个 状态, 走 5 retry 周期
 */
let _stateCounter = 0;
function nextState(): "success" | "unauthorized" | "timeout" {
  const r = _stateCounter % 3;
  _stateCounter += 1;
  if (r === 0) return "success";
  if (r === 1) return "unauthorized";
  return "timeout";
}

/** 4 必备 LLM provider (per 2026-09-02 02:49 JST 拍板) */
type LlmProvider = "openai" | "claude" | "gemini" | "minimax";

interface TestResponse {
  provider: LlmProvider;
  status: "success" | "unauthorized" | "timeout";
  /** response body (per state) */
  body: unknown;
  /** 模拟 latency (ms) */
  delayMs: number;
}

function buildResponse(provider: LlmProvider, state: "success" | "unauthorized" | "timeout"): TestResponse {
  switch (state) {
    case "success":
      // 各 provider 试着 (5 节点 fixture 保持 真实 schema)
      switch (provider) {
        case "openai":
          return {
            provider, status: "success",
            body: { object: "list", data: [{ id: "gpt-4o" }, { id: "gpt-4o-mini" }] },
            delayMs: 200,
          };
        case "claude":
          return {
            provider, status: "success",
            body: { id: "msg_mock", type: "message", content: [{ type: "text", text: "OK" }] },
            delayMs: 200,
          };
        case "gemini":
          return {
            provider, status: "success",
            body: { models: [{ name: "gemini-2.0-flash" }, { name: "gemini-2.0-pro" }] },
            delayMs: 200,
          };
        case "minimax":
          return {
            provider, status: "success",
            body: { object: "list", data: [{ id: "m2.7-large" }, { id: "m-large" }] },
            delayMs: 200,
          };
      }
      // exhaustiveness
      throw new Error(`unreachable: ${provider}`);
    case "unauthorized":
      return {
        provider, status: "unauthorized",
        body: {
          error: "invalid_api_key",
          message: "Incorrect API key provided. (mock 401 for retry test)",
        },
        delayMs: 100,
      };
    case "timeout":
      return {
        provider, status: "timeout",
        body: { error: "timeout", message: "AbortError: 10s timeout (mock)" },
        delayMs: 11_000,  // 超过 10s timeout, AbortController 触发
      };
  }
}

export const providersLlmHandlers = [
  // OpenAI
  http.get("https://api.openai.com/v1/models", async ({ request }) => {
    const state = nextState();
    const r = buildResponse("openai", state);
    await new Promise((resolve) => setTimeout(resolve, r.delayMs));
    if (state === "unauthorized") {
      return HttpResponse.json(r.body as any, { status: 401 });
    }
    return HttpResponse.json(r.body as any, { status: 200 });
  }),

  // Anthropic Claude
  http.post("https://api.anthropic.com/v1/messages", async ({ request }) => {
    const state = nextState();
    const r = buildResponse("claude", state);
    await new Promise((resolve) => setTimeout(resolve, r.delayMs));
    if (state === "unauthorized") {
      return HttpResponse.json(r.body as any, { status: 401 });
    }
    return HttpResponse.json(r.body as any, { status: 200 });
  }),

  // Google Gemini
  http.get("https://generativelanguage.googleapis.com/v1beta/models", async ({ request }) => {
    const state = nextState();
    const r = buildResponse("gemini", state);
    await new Promise((resolve) => setTimeout(resolve, r.delayMs));
    if (state === "unauthorized") {
      return HttpResponse.json(r.body as any, { status: 401 });
    }
    return HttpResponse.json(r.body as any, { status: 200 });
  }),

  // minimax
  http.get("https://api.minimax.chat/v1/models", async ({ request }) => {
    const state = nextState();
    const r = buildResponse("minimax", state);
    await new Promise((resolve) => setTimeout(resolve, r.delayMs));
    if (state === "unauthorized") {
      return HttpResponse.json(r.body as any, { status: 401 });
    }
    return HttpResponse.json(r.body as any, { status: 200 });
  }),
];

/** 测试 helper: 重置 state counter (用于 vitest 隔离) */
export function __resetLlmMockState() {
  _stateCounter = 0;
}
