// =====================================================================
// frontend/src/lib/wasm/query-engine.ts
// =====================================================================
// Rust → WebAssembly (WASM) frontend wrapper for query-engine crate
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P1
// + PR #230 (crates/query-engine-wasm PoC).
//
// IMPORTANT: Star 用 Next.js, not Vite. 用 Next.js native WebAssembly.instantiate().
//
// 用法:
//   import { useQueryEngine } from "@/lib/wasm/query-engine";
//   const { ready, validateDsl, parseDsl, keywordAll } = useQueryEngine();
//   if (ready) {
//     const isValid = validateDsl("show ready"); // true
//   }
//
// 阶段 2 (后续 PR): 1 page (e.g. /worktree-canvas search bar) 改用 WASM parse_dsl
// =====================================================================

"use client";

import { useEffect, useRef, useState, useCallback } from "react";

// =====================================================================
// 类型 + 常量
// =====================================================================

/** 6 关键字 (per query-engine/src/keywords.rs) */
export type Keyword = "show" | "agent" | "behind" | "ahead" | "health" | "modified";

/** 完整 6 关键字顺序 (index → string) */
export const KEYWORDS: readonly Keyword[] = [
  "show",
  "agent",
  "behind",
  "ahead",
  "health",
  "modified",
] as const;

/** Keyword count (守门: 6) */
export const KEYWORD_COUNT = 6;

// =====================================================================
// JS fallback (per docs §3.2 P1: WASM not ready 时 UI 仍工作)
// =====================================================================

/**
 * validate_dsl_js — 纯 JS fallback (same logic as Rust validate_dsl)
 */
export function validateDslJS(input: string): boolean {
  if (input.trim().length === 0) return false;
  return KEYWORDS.some((kw) => input.includes(kw));
}

/**
 * parse_dsl_js — JS 关键字 substring 匹配 (PoC 级别, 完整 DSL parser 在 P2 PR)
 * 返回 matched keywords (空数组 = 无匹配)
 */
export function parseDslJS(input: string): Keyword[] {
  return KEYWORDS.filter((kw) => input.includes(kw));
}

/**
 * keyword_at — JS fallback
 */
export function keywordAtJS(idx: number): Keyword | null {
  return KEYWORDS[idx] ?? null;
}

// =====================================================================
// WASM module loading
// =====================================================================

interface QueryEngineWasmExports {
  parse_dsl: (ptr: number, len: number) => number;
  validate_dsl: (ptr: number, len: number) => boolean;
  keyword_count: () => number;
  keyword_as_str: (ptr: number, len: number, idx: number) => number;
  keyword_all: () => number;
  version: () => number;
  __wbindgen_malloc: (size: number, align: number) => number;
  __wbindgen_free: (ptr: number, size: number, align: number) => void;
}

interface WasmModule {
  instance: WebAssembly.Instance;
  exports: QueryEngineWasmExports;
  memory: WebAssembly.Memory;
}

const WASM_PATHS = [
  "/wasm/query_engine_wasm_bg.wasm",
  "/_next/static/wasm/query_engine_wasm_bg.wasm",
] as const;

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
          env: {},
        });
        const exports = instance.exports as unknown as QueryEngineWasmExports;
        const memory = (exports as unknown as { memory?: WebAssembly.Memory }).memory;
        if (!memory) return null;
        return { instance, exports, memory };
      } catch (err) {
        console.warn(`[query-engine-wasm] failed to load ${path}:`, err);
        continue;
      }
    }
    return null;
  })();

  return wasmModulePromise;
}

// =====================================================================
// Helpers
// =====================================================================

function readWasmString(memory: WebAssembly.Memory, ptr: number): string {
  // wasm-bindgen string: 4-byte length prefix + UTF-8 bytes
  const view = new DataView(memory.buffer);
  const len = view.getUint32(ptr - 4, true);
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return new TextDecoder().decode(bytes);
}

function allocAndWrite(memory: WebAssembly.Memory, exports: QueryEngineWasmExports, s: string): number {
  const bytes = new TextEncoder().encode(s);
  const ptr = exports.__wbindgen_malloc(bytes.length, 1);
  new Uint8Array(memory.buffer, ptr, bytes.length).set(bytes);
  return ptr;
}

// =====================================================================
// React hook
// =====================================================================

export interface QueryEngineHook {
  ready: boolean;
  error: Error | null;
  /**
   * Validate DSL input — 至少 1 个关键字
   * @returns true if valid, false otherwise
   */
  validateDsl: (input: string) => boolean;
  /**
   * Parse DSL input (阶段 1 PoC: 关键字 substring 匹配)
   * @returns matched keywords (空 = 无匹配)
   */
  parseDsl: (input: string) => Keyword[];
  /**
   * 6 关键字列表 (always works via JS table)
   */
  keywordAll: () => readonly Keyword[];
  /**
   * 版本号 (WASM loaded 时返 Rust version string)
   */
  version: string | null;
}

/**
 * useQueryEngine — React hook wrapping query-engine WASM
 *
 * JS fallback always works (per docs §3.2 P1).
 * WASM module loaded lazily + ready state for feature flag.
 */
export function useQueryEngine(): QueryEngineHook {
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
          try {
            const versionPtr = mod.exports.version();
            setVersion(readWasmString(mod.memory, versionPtr));
          } catch {
            setVersion("unknown (version() failed)");
          }
        } else {
          setError(new Error("WASM module not available; falling back to JS"));
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
   * Validate — WASM preferred, JS fallback
   */
  const validateDsl = useCallback(
    (input: string): boolean => {
      if (wasm) {
        try {
          const ptr = allocAndWrite(wasm.memory, wasm.exports, input);
          const result = wasm.exports.validate_dsl(ptr, input.length);
          wasm.exports.__wbindgen_free(ptr, input.length, 1);
          return result;
        } catch (err) {
          console.error("[query-engine-wasm] validateDsl error:", err);
        }
      }
      return validateDslJS(input);
    },
    [wasm],
  );

  /**
   * Parse — WASM preferred (returns JSON string from Rust), JS fallback returns Keyword[]
   */
  const parseDsl = useCallback(
    (input: string): Keyword[] => {
      if (wasm) {
        try {
          const ptr = allocAndWrite(wasm.memory, wasm.exports, input);
          const resultPtr = wasm.exports.parse_dsl(ptr, input.length);
          const json = readWasmString(wasm.memory, resultPtr);
          wasm.exports.__wbindgen_free(ptr, input.length, 1);
          // JSON is a Rust Vec<&str> serialized to JSON array
          return JSON.parse(json) as Keyword[];
        } catch (err) {
          console.error("[query-engine-wasm] parseDsl error:", err);
        }
      }
      return parseDslJS(input);
    },
    [wasm],
  );

  return {
    ready,
    error,
    validateDsl,
    parseDsl,
    keywordAll: () => KEYWORDS,
    version,
  };
}

// =====================================================================
// 已知缺口 (per 缺标比错标)
// =====================================================================
//
// 1. wasm-pack build 步骤 (本 PR 不做, 后续 PR):
//    cd crates/query-engine-wasm
//    wasm-pack build --target web --release
//    copy pkg/* 到 frontend/public/wasm/
//
// 2. vitest jsdom env 不能 WebAssembly.instantiate (需 happy-dom + node wasm).
//    本 PR 不写 vitest 测试; 后续 PR 单独.
//
// 3. SSR safe (useEffect lazy + ready=false state).
//
// =====================================================================