// crates/star-desktop/frontend/src/hooks/useQueryEngine.ts
// =====================================================================
// Star Desktop — useQueryEngine WASM hook (PR-251 follow-up of PR #250)
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1 P1
//
// Tauri-side port of frontend/src/lib/wasm/query-engine.ts (283 LOC)
// 简化策略: 6 keywords JS 实现 + WASM hook stub
// 完整 WASM 集成留 PR-252+
// =====================================================================

import { useEffect, useState, useCallback } from "react";

/** 6 keywords (per query-engine/src/dsl_parser.rs) */
export type Keyword = "show" | "agent" | "behind" | "ahead" | "health" | "modified";

/** 6 keyword display names */
const KEYWORD_NAMES: Record<Keyword, string> = {
  show: "Show",
  agent: "Agent",
  behind: "Behind",
  ahead: "Ahead",
  health: "Health",
  modified: "Modified",
};

/** Parse DSL into keyword array (JS fallback per DslParser behavior) */
export function parseDslKeywords(input: string): Keyword[] {
  const tokens = input.toLowerCase().trim().split(/\s+/);
  const result: Keyword[] = [];
  for (const token of tokens) {
    if (isKeyword(token)) {
      result.push(token);
    }
  }
  return result;
}

function isKeyword(s: string): s is Keyword {
  return (
    s === "show" ||
    s === "agent" ||
    s === "behind" ||
    s === "ahead" ||
    s === "health" ||
    s === "modified"
  );
}

interface UseQueryEngineResult {
  /** WASM module ready (false in PR-251) */
  ready: boolean;
  /** Parse DSL into keyword array (JS fallback) */
  parseKeywords: (input: string) => Keyword[];
  /** Get display name */
  getKeywordName: (k: Keyword) => string;
  /** All 6 keywords as array */
  allKeywords: readonly Keyword[];
  /** WASM version string */
  version: string | null;
}

/**
 * useQueryEngine hook — Tauri-side port
 */
export function useQueryEngine(): UseQueryEngineResult {
  const [ready, setReady] = useState(false);
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const tryLoadWasm = async () => {
      try {
        if (typeof window === "undefined") return;
        const res = await fetch("/wasm/query_engine_wasm_bg.wasm");
        if (!res.ok) throw new Error(`fetch failed: ${res.status}`);
        const module = await WebAssembly.instantiateStreaming(res);
        if (cancelled) return;
        const exports = module.instance.exports as {
          version?: () => number;
        };
        if (typeof exports.version === "function") {
          setVersion("0.1.0-query-engine-wasm (loaded)");
        }
        setReady(true);
      } catch {
        setVersion("0.1.0-js-fallback");
        setReady(false);
      }
    };
    tryLoadWasm();
    return () => {
      cancelled = true;
    };
  }, []);

  const parseKeywords = useCallback((input: string): Keyword[] => {
    return parseDslKeywords(input);
  }, []);

  const getKeywordName = useCallback((k: Keyword): string => {
    return KEYWORD_NAMES[k];
  }, []);

  return {
    ready,
    parseKeywords,
    getKeywordName,
    allKeywords: ["show", "agent", "behind", "ahead", "health", "modified"] as const,
    version,
  };
}
