// crates/star-desktop/frontend/src/hooks/useLayoutEngine.ts
// =====================================================================
// Star Desktop — useLayoutEngine WASM hook (PR-251 follow-up of PR #250)
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1
//
// Tauri-side port of frontend/src/lib/wasm/layout-engine.ts (285 LOC)
// 简化策略: WASM binary lazy load + JS fallback
// 完整 WASM integration 留 PR-252+ (需 wasm-pack build → frontend/public/wasm/)
// =====================================================================

import { useEffect, useState, useCallback } from "react";

/** 5 view modes (per layout-engine/src/engine.rs ViewMode) */
export type ViewMode = "tree" | "dependency" | "risk" | "agent" | "history";

/** 3 layout algorithms */
export type LayoutAlgorithmName = "dagre" | "d3_force" | "elk";

/** ViewMode → algorithm 选择 (per layout-engine/src/engine.rs:184 pick_algorithm) */
const VIEW_MODE_TO_ALGORITHM: Record<
  ViewMode,
  (nodeCount: number) => LayoutAlgorithmName
> = {
  tree: (n) => (n < 100 ? "dagre" : "elk"),
  dependency: () => "d3_force",
  risk: () => "d3_force",
  history: () => "d3_force",
  agent: (n) => (n > 50 ? "elk" : "d3_force"),
};

/** JS fallback for visualDistance = log(commitDistance + 1) * 30 (per FR-WT-005) */
export function visualDistance(commitDistance: number): number {
  return Math.log(commitDistance + 1) * 30;
}

function isValidViewMode(v: unknown): v is ViewMode {
  return (
    v === "tree" ||
    v === "dependency" ||
    v === "risk" ||
    v === "agent" ||
    v === "history"
  );
}

interface UseLayoutEngineResult {
  /** WASM module ready (false in PR-251, true in PR-252+ after wasm-pack build) */
  ready: boolean;
  /** Pick algorithm based on view mode + node count */
  pickAlgorithm: (viewMode: ViewMode, nodeCount: number) => LayoutAlgorithmName;
  /** Compute visual distance (JS fallback per FR-WT-005) */
  visualDistance: (commitDistance: number) => number;
  /** WASM version string (null until ready) */
  version: string | null;
}

/**
 * useLayoutEngine hook — Tauri-side port
 *
 * 本 PR 提供 JS 实现 (per VIEW_MODE_TO_ALGORITHM 静态映射).
 * 完整 WASM 集成留 PR-252+:
 *   1. wasm-pack build --target web (per PR #233 wasm-build.yml)
 *   2. cp pkg/layout_engine_wasm_bg.wasm frontend/public/wasm/
 *   3. WebAssembly.instantiateStreaming(fetch('/wasm/...'), imports)
 *   4. Hook 内部懒加载 + Promise<LayoutEngineWasmExports>
 */
export function useLayoutEngine(): UseLayoutEngineResult {
  const [ready, setReady] = useState(false);
  const [version, setVersion] = useState<string | null>(null);

  useEffect(() => {
    // PR-251: 尝试加载 WASM binary (暂未打包, 留 PR-252+)
    // 当 wasm-pack build 完成 + .wasm 放到 frontend/public/wasm/ 时启用
    let cancelled = false;
    const tryLoadWasm = async () => {
      try {
        if (typeof window === "undefined") return;
        const res = await fetch("/wasm/layout_engine_wasm_bg.wasm");
        if (!res.ok) throw new Error(`fetch failed: ${res.status}`);
        const module = await WebAssembly.instantiateStreaming(res);
        if (cancelled) return;
        const exports = module.instance.exports as {
          version?: () => number;
        };
        if (typeof exports.version === "function") {
          // 暂用 JS 版本字符串
          setVersion("0.1.0-layout-engine-wasm (loaded)");
        }
        setReady(true);
      } catch {
        // Fallback: JS 实现
        setVersion("0.1.0-js-fallback");
        setReady(false);
      }
    };
    tryLoadWasm();
    return () => {
      cancelled = true;
    };
  }, []);

  const pickAlgorithm = useCallback(
    (viewMode: ViewMode, nodeCount: number): LayoutAlgorithmName => {
      if (!isValidViewMode(viewMode)) {
        throw new Error(`invalid viewMode: ${String(viewMode)}`);
      }
      return VIEW_MODE_TO_ALGORITHM[viewMode](nodeCount);
    },
    [],
  );

  return {
    ready,
    pickAlgorithm,
    visualDistance,
    version,
  };
}
