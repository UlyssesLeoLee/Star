// vitest setup — 注册 @testing-library/jest-dom matchers
// + 全局 mock next/navigation (避免 KanbanCard / MonthView / WeekView 在 jsdom 抛 "app router not mounted")
// + MSW node server 启动 (per mock-msw-handlers.md §2.3)
//
// 触发场景:
//   - KanbanCard.tsx:18 / MonthView.tsx:4 / WeekView.tsx:4 都 import { useRouter } from "next/navigation"
//   - jsdom 没有 App Router context, 调 useRouter() 抛 invariant
//   - 全局 mock 后, 任何用 next/navigation 的组件在测试里都拿到 stub push/replace, 不报错
//   - MSW node server 启动后, 测试内 fetch("/api/...") 被 handlers 拦截返回 mock 数据
//
// 历史: 2026-08-28 W3 (W2) 写入时未配置 mock; W5 接手 store/persist 时也未补; U3 接手 projects 页面时补上
//       2026-08-28 M2-A 补 MSW setupServer (per mock-msw-handlers.md §2.3 + §3.1)
import "@testing-library/jest-dom/vitest";
import { afterAll, afterEach, beforeAll, vi } from "vitest";

import { server } from "./src/mocks/server";

// =====================================================================
// R3F / @react-three/fiber 在 jsdom 下需要 ResizeObserver + canvas polyfill
// (per 2026-09-05 GasParticlesHint 接入, 守门 #1 v6 实证:
//   GasParticlesHint → R3F <Canvas> → react-use-measure → ResizeObserver,
//   jsdom 默认无 ResizeObserver, 跑 CommandBar.test.tsx 触发 invariant crash)
// 兜底: mock 掉 ResizeObserver (no-op) + getBoundingClientRect
// =====================================================================
if (typeof globalThis.ResizeObserver === "undefined") {
  globalThis.ResizeObserver = class ResizeObserver {
    observe() { /* no-op */ }
    unobserve() { /* no-op */ }
    disconnect() { /* no-op */ }
  } as unknown as typeof ResizeObserver;
}
if (typeof globalThis.IntersectionObserver === "undefined") {
  globalThis.IntersectionObserver = class IntersectionObserver {
    observe() { /* no-op */ }
    unobserve() { /* no-op */ }
    disconnect() { /* no-op */ }
    takeRecords() { return []; }
    root = null;
    rootMargin = "";
    thresholds = [];
  } as unknown as typeof IntersectionObserver;
}
if (typeof HTMLCanvasElement !== "undefined") {
  const proto = HTMLCanvasElement.prototype as unknown as { getContext: (id: string) => unknown };
  const originalGetContext = proto.getContext;
  proto.getContext = function (id: string) {
    if (id === "webgl" || id === "webgl2" || id === "experimental-webgl") {
      // jsdom 无 WebGL, 返 no-op 桩避免 R3F 启动时报错
      return {
        canvas: this,
        getExtension: () => null,
        getParameter: () => null,
        getShaderPrecisionFormat: () => ({ precision: 1, rangeMin: 1, rangeMax: 1 }),
        createBuffer: () => ({}),
        createShader: () => ({}),
        createProgram: () => ({}),
        createTexture: () => ({}),
        createFramebuffer: () => ({}),
        createRenderbuffer: () => ({}),
        bindBuffer: () => {},
        bufferData: () => {},
        shaderSource: () => {},
        compileShader: () => {},
        attachShader: () => {},
        linkProgram: () => {},
        useProgram: () => {},
        getShaderParameter: () => true,
        getProgramParameter: () => true,
        getUniformLocation: () => ({}),
        getAttribLocation: () => 0,
        enableVertexAttribArray: () => {},
        vertexAttribPointer: () => {},
        uniform1f: () => {},
        uniform1i: () => {},
        uniform2f: () => {},
        uniform3f: () => {},
        uniform4f: () => {},
        uniformMatrix4fv: () => {},
        viewport: () => {},
        clear: () => {},
        clearColor: () => {},
        clearDepth: () => {},
        enable: () => {},
        disable: () => {},
        depthFunc: () => {},
        blendFunc: () => {},
        frontFace: () => {},
        cullFace: () => {},
        activeTexture: () => {},
        bindTexture: () => {},
        texParameteri: () => {},
        pixelStorei: () => {},
        drawArrays: () => {},
        drawElements: () => {},
        getError: () => 0,
        isContextLost: () => false,
      };
    }
    return originalGetContext?.call(this, id) ?? null;
  };
}

beforeAll(() => server.listen({ onUnhandledRequest: "warn" }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    refresh: vi.fn(),
    back: vi.fn(),
    forward: vi.fn(),
    prefetch: vi.fn(),
  }),
  usePathname: () => "/test",
  useSearchParams: () => new URLSearchParams(),
  useParams: () => ({}),
}));
