// frontend/src/mocks/__tests__/real-mode.test.ts
// P3-A.7 real-mode 单元测试 (3 test + 守门)

import { describe, it, expect, beforeEach } from "vitest";
import { getRealModeState, isRealMode } from "@/mocks/real-mode";

describe("real-mode", () => {
  beforeEach(() => {
    // 清理 localStorage
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.removeItem("use_real_api");
      window.localStorage.removeItem("real_api_base");
      window.localStorage.removeItem("real_api_key");
    }
  });

  it("default is false when no env and no localStorage", () => {
    const state = getRealModeState();
    expect(state.enabled).toBe(false);
    expect(state.source).toBe("default-false");
    expect(isRealMode()).toBe(false);
  });

  it("localStorage true overrides everything", () => {
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.setItem("use_real_api", "true");
      window.localStorage.setItem("real_api_base", "https://api.example.com");
    }
    const state = getRealModeState();
    expect(state.enabled).toBe(true);
    expect(state.source).toBe("localStorage");
    expect(state.base_url).toBe("https://api.example.com");
  });

  it("localStorage false forces disabled even if env true", () => {
    // env 模拟: 通过 import.meta 或全局变量
    const prevEnv = process.env.NEXT_PUBLIC_USE_REAL_API;
    process.env.NEXT_PUBLIC_USE_REAL_API = "true";
    if (typeof window !== "undefined" && window.localStorage) {
      window.localStorage.setItem("use_real_api", "false");
    }
    const state = getRealModeState();
    expect(state.enabled).toBe(false);
    expect(state.source).toBe("localStorage");
    // 恢复
    if (prevEnv === undefined) {
      delete process.env.NEXT_PUBLIC_USE_REAL_API;
    } else {
      process.env.NEXT_PUBLIC_USE_REAL_API = prevEnv;
    }
  });
});
