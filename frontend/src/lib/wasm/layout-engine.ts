// =====================================================================
// frontend/src/lib/wasm/layout-engine.ts
// =====================================================================
// Rust → WebAssembly (WASM) frontend wrapper for layout-engine crate
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1
//
// IMPORTANT: Star 用 Next.js (not Vite). vite-plugin-wasm 不可用,
// 改用 Next.js native WebAssembly.instantiate() API.
//
// 用法:
//   import { useLayoutEngine } from "@/lib/wasm/layout-engine";
//   const { ready, pickAlgorithm } = useLayoutEngine();
//   if (ready) {
//     const algo = pickAlgorithm("tree", nodeCount); // "dagre" | "d3_force" | "elk"
//   }
//
// 阶段 2 (后续 PR): 1 page (e.g. agent-relationships 100+ nodes) 改用 WASM layout
// 阶段 3 (后续 PR): query-engine-wasm PoC (work_items filter/sort/aggregate)
// =====================================================================

"use client";

import { useEffect, useRef, useState, useCallback } from "react";

// =====================================================================
// 类型 + 常量
// =====================================================================

/** 5 view modes (per layout-engine/src/engine.rs ViewMode + docs §3.1) */
export type ViewMode = "tree" | "dependency" | "risk" | "agent" | "history";

/** 3 layout algorithms (per layout-engine/src/engine.rs LayoutAlgorithm) */
export type LayoutAlgorithmName = "dagre" | "d3_force" | "elk";

/** ViewMode → layout algorithm 选择 (per layout-engine/src/engine.rs:184 pick_algorithm) */
const VIEW_MODE_TO_ALGORITHM: Record<
  ViewMode,
  (nodeCount: number) => LayoutAlgorithmName
> = {
  // TREE: < 100 → Dagre, >= 100 → Elk
  tree: (n) => (n < 100 ? "dagre" : "elk"),
  // DEPENDENCY / RISK / HISTORY: always D3Force
  dependency: () => "d3_force",
  risk: () => "d3_force",
  history: () => "d3_force",
  // AGENT: > 50 → Elk, <= 50 → D3Force
  agent: (n) => (n > 50 ? "elk" : "d3_force"),
};

/** visualDistance = log(commitDistance + 1) * 30 (per FR-WT-005 + DD §36) */
export function visualDistance(commitDistance: number): number {
  return Math.log(commitDistance + 1) * 30;
}

/** 验证 viewMode 合法 */
function isValidViewMode(v: unknown): v is ViewMode {
  return v === "tree" || v === "dependency" || v === "risk" || v === "agent" || v === "history";
}

// =====================================================================
// WASM module loading
// =====================================================================

/** WASM module 句柄 (per `wasm-pack build --target web`) */
interface LayoutEngineWasmExports {
  pick_algorithm_wasm: (viewModePtr: number, viewModeLen: number, nodeCount: number) => number;
  visual_distance: (commitDistance: number) => number;
  algorithm_name_for_view: (viewModePtr: number, viewModeLen: number, nodeCount: number) => number;
  version: () => number;
  /** wasm-bindgen 内存 (string return 用) */
  __wbindgen_malloc: (size: number, align: number) => number;
  __wbindgen_free: (ptr: number, size: number, align: number) => void;
}

interface WasmModule {
  instance: WebAssembly.Instance;
  exports: LayoutEngineWasmExports;
  memory: WebAssembly.Memory;
}

/** wasm-pack --target web 输出路径 (per vite-plugin-wasm analog) */
const WASM_PATHS = [
  "/wasm/layout_engine_wasm_bg.wasm", // Next.js public/ dir
  "/_next/static/wasm/layout_engine_wasm_bg.wasm",
] as const;

/** Cache WASM module (single instance per session) */
let wasmModulePromise: Promise<WasmModule | null> | null = null;

async function loadWasmModule(): Promise<WasmModule | null> {
  if (wasmModulePromise) return wasmModulePromise;

  wasmModulePromise = (async () => {
    for (const path of WASM_PATHS) {
      try {
        const response = await fetch(path);
        if (!response.ok) continue;

        const bytes = await response.arrayBuffer();
        const { instance } = await WebAssembly.instantiate(bytes, {
          // wasm-bindgen 默认需要 env.__wbindgen_placeholder_* imports
          // 真实环境可省略 (无 imports)
          env: {},
        });
        const exports = instance.exports as unknown as LayoutEngineWasmExports;
        const memory = (exports as unknown as { memory?: WebAssembly.Memory }).memory;
        if (!memory) {
          console.warn("[layout-engine-wasm] no memory export found");
          return null;
        }
        return { instance, exports, memory };
      } catch (err) {
        console.warn(`[layout-engine-wasm] failed to load ${path}:`, err);
        continue;
      }
    }
    return null;
  })();

  return wasmModulePromise;
}

// =====================================================================
// React hook
// =====================================================================

export interface LayoutEngineHook {
  /** WASM module ready (lazy loaded after fetch) */
  ready: boolean;
  /** 加载失败 (fetch / instantiate error) */
  error: Error | null;
  /**
   * 算法选择 (Rust 函数 pick_algorithm_wasm 暴露给浏览器)
   * @returns 选择的算法名, 或 null if WASM not ready / invalid viewMode
   */
  pickAlgorithm: (viewMode: ViewMode, nodeCount: number) => LayoutAlgorithmName | null;
  /**
   * visualDistance 计算 (Rust 函数 visual_distance)
   * @returns 距离, 或 null if WASM not ready
   */
  visualDistance: (commitDistance: number) => number | null;
  /**
   * 版本号 (per `version()` Rust export)
   */
  version: string | null;
}

/**
 * useLayoutEngine — React hook wrapping layout-engine WASM
 *
 * Lazy load WASM module on first call (per docs §3.2 P1 + per session memory "懒加载").
 * JS fallback: pickAlgorithm JS-side 表 (same logic), 保证 WASM not ready 时 UI 仍工作.
 */
export function useLayoutEngine(): LayoutEngineHook {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [wasm, setWasm] = useState<WasmModule | null>(null);
  const [version, setVersion] = useState<string | null>(null);
  const cleanupRef = useRef<(() => void) | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const mod = await loadWasmModule();
        if (cancelled) return;
        if (mod) {
          setWasm(mod);
          setReady(true);
          // Get version (return as string from WASM memory)
          try {
            const versionPtr = mod.exports.version();
            const bytes = new Uint8Array(mod.memory.buffer, versionPtr);
            // wasm-bindgen string: 4-byte length prefix + UTF-8 bytes
            const len = new DataView(mod.memory.buffer, versionPtr - 4).getUint32(0, true);
            const str = new TextDecoder().decode(bytes.slice(0, len));
            setVersion(str);
          } catch {
            setVersion("unknown (version() failed)");
          }
        } else {
          setError(new Error("WASM module not available; falling back to JS-only"));
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err : new Error(String(err)));
        }
      }
    })();
    cleanupRef.current = () => {
      cancelled = true;
    };
    return () => {
      cancelled = true;
      cleanupRef.current?.();
    };
  }, []);

  /**
   * Pick algorithm — prefer WASM, fallback to JS table
   */
  const pickAlgorithm = useCallback(
    (viewMode: ViewMode, nodeCount: number): LayoutAlgorithmName | null => {
      // JS fallback (always works)
      const fn = VIEW_MODE_TO_ALGORITHM[viewMode];
      if (fn) return fn(nodeCount);

      // WASM path (preferred for actual compute)
      if (!wasm || !isValidViewMode(viewMode)) return null;
      try {
        const encoder = new TextEncoder();
        const viewModeBytes = encoder.encode(viewMode);
        const ptr = wasm.exports.__wbindgen_malloc(viewModeBytes.length, 1);
        new Uint8Array(wasm.memory.buffer, ptr, viewModeBytes.length).set(viewModeBytes);
        const algoPtr = wasm.exports.pick_algorithm_wasm(ptr, viewModeBytes.length, nodeCount);
        // Read string from WASM memory
        const len = new DataView(wasm.memory.buffer, algoPtr - 4).getUint32(0, true);
        const bytes = new Uint8Array(wasm.memory.buffer, algoPtr, len);
        const algo = new TextDecoder().decode(bytes);
        wasm.exports.__wbindgen_free(ptr, viewModeBytes.length, 1);
        return (algo as LayoutAlgorithmName) ?? null;
      } catch (err) {
        console.error("[layout-engine-wasm] pickAlgorithm error:", err);
        return null;
      }
    },
    [wasm],
  );

  /** visualDistance — WASM preferred, JS fallback */
  const visualDistanceFn = useCallback(
    (commitDistance: number): number | null => {
      if (wasm) {
        try {
          return wasm.exports.visual_distance(commitDistance);
        } catch (err) {
          console.error("[layout-engine-wasm] visualDistance error:", err);
          return null;
        }
      }
      return null; // Or fall back to JS: Math.log(cd + 1) * 30 (already exported)
    },
    [wasm],
  );

  return {
    ready,
    error,
    pickAlgorithm,
    visualDistance: visualDistanceFn,
    version,
  };
}

// =====================================================================
// JS-only fallback (works without WASM)
// Per docs §3.2 P1: "feature flag — WASM not ready 时 UI 仍工作"
// =====================================================================

/**
 * pickAlgorithmJS — pure JS implementation (same logic as Rust pick_algorithm)
 * 用于 SSR + WASM not loaded 场景.
 */
export function pickAlgorithmJS(viewMode: ViewMode, nodeCount: number): LayoutAlgorithmName | null {
  const fn = VIEW_MODE_TO_ALGORITHM[viewMode];
  return fn ? fn(nodeCount) : null;
}

// =====================================================================
// 已知缺口 (per 缺标比错标)
// =====================================================================
//
// 1. wasm-pack build 步骤 (本 PR 不做, 后续 PR 单独):
//    cd crates/layout-engine-wasm
//    wasm-pack build --target web --release
//    输出到 crates/layout-engine-wasm/pkg/
//    然后 copy pkg/* 到 frontend/public/wasm/
//
// 2. vitest jsdom env 不能 WebAssembly.instantiate (需 happy-dom + node wasm).
//    本 PR 不写 vitest 测试;后续 PR 单独.
//
// 3. SSR (Next.js server-side) 不能调用 WASM (无 DOM fetch / no memory).
//    per useLayoutEngine() lazy + ready=false 状态, SSR 安全.
//    pickAlgorithmJS() 可在 SSR 直接用 (纯函数).
//
// =====================================================================