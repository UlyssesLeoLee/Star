// frontend/src/mocks/real-mode.ts
// MSW real-mode switch + fetch wrapper (P3-A.7 / wt-w34)
//
// Per 2026-08-29 11:43 JST 用户拍板 P3-A.7:
// - 给 MSW handlers 加 "real mode" 短路: 开关打开时, 直接调真 API
// - 三档优先级: localStorage > env > 默认 false
// - Bearer auth 自动注入 (从 localStorage 读 api-key)
//
// 设计: 仅 cli.ts 一个 handler 改, 其他 handler 留 TODO (per 范围最小化)
// 已知缺口: agents/analytics/inbox 暂未 real-mode 化 (P3-A.7 §3 缺口 #1)

export type RealModeSource = "localStorage" | "env" | "default-false";

export interface RealModeState {
  enabled: boolean;
  source: RealModeSource;
  base_url: string;
  api_key: string | null;
}

/** 读 real-mode 状态 (无副作用) */
export function getRealModeState(): RealModeState {
  // 1. localStorage 优先
  if (typeof window !== "undefined" && window.localStorage) {
    const ls = window.localStorage.getItem("use_real_api");
    if (ls === "true") {
      return {
        enabled: true,
        source: "localStorage",
        base_url: window.localStorage.getItem("real_api_base") || defaultBaseUrl(),
        api_key: window.localStorage.getItem("real_api_key"),
      };
    }
    if (ls === "false") {
      return { enabled: false, source: "localStorage", base_url: defaultBaseUrl(), api_key: null };
    }
  }
  // 2. env (build-time, Next.js 注入到 NEXT_PUBLIC_*)
  const envFlag = process.env.NEXT_PUBLIC_USE_REAL_API;
  if (envFlag === "true") {
    return {
      enabled: true,
      source: "env",
      base_url: process.env.NEXT_PUBLIC_API_BASE_URL || defaultBaseUrl(),
      api_key: process.env.NEXT_PUBLIC_API_KEY || null,
    };
  }
  // 3. 默认 false
  return { enabled: false, source: "default-false", base_url: defaultBaseUrl(), api_key: null };
}

/** 兼容老接口: 简单 boolean */
export function isRealMode(): boolean {
  return getRealModeState().enabled;
}

/** 真发请求 wrapper (real-mode 开启时调用) */
export async function realFetch(path: string, init: RequestInit = {}): Promise<Response> {
  const state = getRealModeState();
  const url = path.startsWith("http") ? path : `${state.base_url}${path}`;
  const headers = new Headers(init.headers);
  if (state.api_key) {
    headers.set("Authorization", `Bearer ${state.api_key}`);
  }
  if (!headers.has("Content-Type") && init.body) {
    headers.set("Content-Type", "application/json");
  }
  return fetch(url, { ...init, headers });
}

/** 默认 base URL (开发) */
function defaultBaseUrl(): string {
  if (typeof process !== "undefined" && process.env.NEXT_PUBLIC_API_BASE_URL) {
    return process.env.NEXT_PUBLIC_API_BASE_URL;
  }
  return "http://localhost:3000";
}
