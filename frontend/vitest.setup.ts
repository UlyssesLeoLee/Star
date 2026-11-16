/*
@cypher schema=1 source_sha256=e61ac1fc25c2194c3f39cc040d344b42931aa26d950b0ea4c25017148bc1f8db
MERGE (self:File {path:"frontend/vitest.setup.ts"})
MERGE (server:ExternalService {name:"msw/node"})
MERGE (jestDom:ExternalService {name:"@testing-library/jest-dom"})
MERGE (nemaWasm:File {path:"frontend/src/lib/nema/wasm/nema_codec_wasm_bg.wasm"})
MERGE (nemaSetup:Symbol {id:"frontend/vitest.setup.ts::installNemaWasmHandler",kind:"function"})
MERGE (self)-[:IMPORTS]->(server)
MERGE (self)-[:IMPORTS]->(jestDom)
MERGE (self)-[:DEFINES]->(nemaSetup)
MERGE (nemaSetup)-[:READS]->(nemaWasm)
@endcypher
*/
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
import { afterAll, afterEach, beforeAll, beforeEach, vi } from "vitest";

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

// =====================================================================
// .nema WASM codec 加载 (per 2026-11-16 用户明确: .nema 是自创文件格式,
//   CI 门禁必须正确理解它 — 不能把它当"外来依赖缺失"跳过)
//
// 事实链 (实测, 不是推测):
//   1. frontend/src/lib/nema/nema.ts 的 loadNemaWasm() 用
//      new URL("./wasm/nema_codec_wasm_bg.wasm", import.meta.url) 取字节,
//      再 mod.initSync({ module }) 做真解析 (Rust codec, 非 JS fallback)。
//   2. vitest (jsdom) 下该 URL 被 vite 解析成
//      http://localhost:3000/src/lib/nema/wasm/nema_codec_wasm_bg.wasm
//      (实测 META/WASM URL, 非 file://) → 该请求被 MSW 接管。
//   3. MSW v2 路径匹配: "*" 只匹配单段, "**" 才匹配任意深度。
//      所以 handler 必须写 "**/nema_codec_wasm_bg.wasm"。
//      写 "*" 时 handler 不命中 → 请求落到真实网络 → ECONNRESET
//      → nema 落 JS fallback → profile 断言全红。
//   4. HttpResponse.bytes() 在 msw 2.15 不存在 → 用 new HttpResponse(bytes)。
//
// 修法: MSW handler 返回仓库内真实 wasm 字节, initSync 走真解析路径。
//   测的是真 Rust codec, 不是 mock。
//   (守门 #25: 集成测试不发真 HTTP — 这里是仓库内本地文件字节)
// =====================================================================
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";
import { http, HttpResponse } from "msw";

const NEMA_WASM_PATH = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "src/lib/nema/wasm/nema_codec_wasm_bg.wasm",
);

let nemaWasmBytes: Uint8Array | null = null;
function loadNemaWasmBytes(): Uint8Array {
  if (!nemaWasmBytes) {
    nemaWasmBytes = new Uint8Array(readFileSync(NEMA_WASM_PATH));
  }
  return nemaWasmBytes;
}

/**
 * 注册 .nema wasm handler。
 *
 * 关键: 本文件下方 afterEach 里有 server.resetHandlers() — 它把 handler 集合
 * 还原成 setupServer(...) 的初始集合, 也就是把任何 server.use() 注册的运行时
 * handler 全部清掉。所以必须在每个用例前重新注册, 否则只有第一个用例能拿到
 * wasm 字节, 之后的用例又落回 ECONNRESET → JS fallback。
 */
function installNemaWasmHandler(): void {
  server.use(
    http.get("**/nema_codec_wasm_bg.wasm", () => {
      const bytes = loadNemaWasmBytes();
      return new HttpResponse(bytes.slice().buffer, {
        status: 200,
        headers: { "content-type": "application/wasm" },
      });
    }),
  );
}

// MSW 必须在"测试模块被 import"之前就开始拦截:
//   frontend/src/lib/nema/nemaStore.ts 在模块顶层就 void loadNemaWasm(),
//   那次 fetch 发生在 vitest 收集阶段 (import test file) —— 此时若 server 还没
//   listen, 请求会真的打到网络 → ECONNRESET → nema 静默降级成 JS fallback。
//   所以 listen 放顶层 (setupFiles 在测试文件 import 之前执行), 不放 beforeAll。
// 顶层先注册一次: 测试模块 import 阶段 nemaStore 的 void loadNemaWasm()
// 就会发 fetch, 那时还没有 beforeEach。
installNemaWasmHandler();
server.listen({ onUnhandledRequest: "warn" });
// resetHandlers 会清掉 server.use() 注册的运行时 handler, 所以每个用例前重装。
beforeEach(() => installNemaWasmHandler());
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
