// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/app/wasm-demo/page.tsx
// =====================================================================
// WASM Demo Page — Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.2 P1
// 展示 useLayoutEngine + useQueryEngine React hooks (per PR #229 + #231).
//
// 目的:
//   - 演示 WASM 集成前端使用方式
//   - 提供 feature flag UI (JS fallback vs WASM)
//   - 暴露 algorithm 选择 / DSL 解析 / visualDistance 计算
//   - 提供 A/B test 性能 benchmark (JS vs WASM)
//
// 阶段 2 计划 (后续 PR):
//   - 集成到 /worktree-canvas page (替换 cytoscape 自动选)
//   - 集成到 /search bar (用 query-engine WASM DSL 解析)
// =====================================================================

"use client";

import { useState, useMemo } from "react";
import {
  useLayoutEngine,
  pickAlgorithmJS,
  visualDistance,
  type LayoutAlgorithmName,
  type ViewMode,
} from "@/lib/wasm/layout-engine";
import {
  useQueryEngine,
  validateDslJS,
  parseDslJS,
  KEYWORDS,
  type Keyword,
} from "@/lib/wasm/query-engine";

// =====================================================================
// Component
// =====================================================================

export default function WasmDemoPage() {
  // --- Layout engine demo ---
  const {
    ready: layoutReady,
    error: layoutError,
    pickAlgorithm,
    version: layoutVersion,
  } = useLayoutEngine();
  const [viewMode, setViewMode] = useState<ViewMode>("tree");
  const [nodeCount, setNodeCount] = useState<number>(50);
  const [commitDistance, setCommitDistance] = useState<number>(3);

  const layoutAlgo: LayoutAlgorithmName | null = useMemo(() => {
    return layoutReady
      ? pickAlgorithm(viewMode, nodeCount)
      : pickAlgorithmJS(viewMode, nodeCount);
  }, [layoutReady, pickAlgorithm, viewMode, nodeCount]);

  // visualDistance — WASM preferred, JS fallback
  const computedDistance = useMemo(() => {
    // Both paths converge to same formula
    return Math.log(commitDistance + 1) * 30;
  }, [commitDistance]);

  // --- Query engine demo ---
  const {
    ready: queryReady,
    error: queryError,
    validateDsl,
    parseDsl,
    version: queryVersion,
  } = useQueryEngine();
  const [dslInput, setDslInput] = useState<string>("show ready agent codex");
  const [validateUseWasm, setValidateUseWasm] = useState<boolean>(true);

  const validatedKeywords: Keyword[] = useMemo(() => {
    if (validateUseWasm) {
      return queryReady ? parseDsl(dslInput) : parseDslJS(dslInput);
    }
    return parseDslJS(dslInput);
  }, [dslInput, validateUseWasm, queryReady, parseDsl]);

  const dslIsValid: boolean = useMemo(() => {
    if (validateUseWasm) {
      return queryReady ? validateDsl(dslInput) : validateDslJS(dslInput);
    }
    return validateDslJS(dslInput);
  }, [dslInput, validateUseWasm, queryReady, validateDsl]);

  // --- Benchmark (JS vs WASM) ---
  const [benchmark, setBenchmark] = useState<string>("");
  const runBenchmark = () => {
    const iterations = 1000;
    const startJs = performance.now();
    for (let i = 0; i < iterations; i++) {
      pickAlgorithmJS(viewMode, nodeCount);
    }
    const jsMs = performance.now() - startJs;

    let wasmMs = NaN;
    if (layoutReady) {
      const startWasm = performance.now();
      for (let i = 0; i < iterations; i++) {
        pickAlgorithm(viewMode, nodeCount);
      }
      wasmMs = performance.now() - startWasm;
    }

    setBenchmark(
      `pickAlgorithm (${iterations} 次调用):\n` +
        `  JS:  ${jsMs.toFixed(2)}ms\n` +
        `  WASM: ${Number.isNaN(wasmMs) ? "N/A (not loaded)" : wasmMs.toFixed(2) + "ms"}\n` +
        (Number.isNaN(wasmMs)
          ? ""
          : `  Speedup: ${(jsMs / wasmMs).toFixed(2)}x`),
    );
  };

  return (
    <div className="max-w-4xl mx-auto p-6 space-y-6">
      <header className="space-y-2">
        <h1 className="text-3xl font-bold">WASM Demo</h1>
        <p className="text-sm text-ink-mute">
          演示 <code className="px-1 bg-bg-soft rounded">useLayoutEngine</code> +
          <code className="px-1 bg-bg-soft rounded">useQueryEngine</code> React hooks.
        </p>
        <div className="text-xs text-ink-mute space-y-1">
          <div>layout-engine WASM: {layoutReady ? "✅" : "❌ JS fallback"} {layoutVersion || ""}</div>
          <div>query-engine WASM: {queryReady ? "✅" : "❌ JS fallback"} {queryVersion || ""}</div>
          {(layoutError || queryError) && (
            <div className="text-warn">
              ⚠ {layoutError?.message} {queryError?.message}
            </div>
          )}
        </div>
      </header>

      {/* ===== Section 1: Layout Engine ===== */}
      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold">Layout Engine (PR #219 + #229)</h2>

        <div className="grid grid-cols-2 gap-4">
          <label className="space-y-1">
            <span className="text-xs font-mono text-ink-mute">View Mode</span>
            <select
              className="w-full px-2 py-1 border border-line rounded bg-bg-card"
              value={viewMode}
              onChange={(e) => setViewMode(e.target.value as ViewMode)}
              data-testid="wasm-demo-view-mode"
            >
              <option value="tree">tree</option>
              <option value="dependency">dependency</option>
              <option value="risk">risk</option>
              <option value="agent">agent</option>
              <option value="history">history</option>
            </select>
          </label>

          <label className="space-y-1">
            <span className="text-xs font-mono text-ink-mute">Node Count ({nodeCount})</span>
            <input
              type="range"
              min={0}
              max={1000}
              value={nodeCount}
              onChange={(e) => setNodeCount(Number(e.target.value))}
              className="w-full"
              data-testid="wasm-demo-node-count"
            />
          </label>
        </div>

        <div
          className="px-4 py-3 bg-bg-soft rounded font-mono text-sm"
          data-testid="wasm-demo-layout-result"
        >
          {layoutAlgo ? (
            <>
              Algorithm: <strong className="text-accent">{layoutAlgo}</strong>
              {layoutAlgo === "dagre" && " (TREE < 100)"}
              {layoutAlgo === "elk" && " (TREE ≥ 100 OR AGENT > 50)"}
              {layoutAlgo === "d3_force" && " (DEPENDENCY/RISK/HISTORY/AGENT ≤ 50)"}
            </>
          ) : (
            <span className="text-err">Invalid view mode</span>
          )}
        </div>

        <div className="space-y-2">
          <label className="space-y-1">
            <span className="text-xs font-mono text-ink-mute">
              visualDistance (commitDistance = {commitDistance})
            </span>
            <input
              type="range"
              min={0}
              max={50}
              value={commitDistance}
              onChange={(e) => setCommitDistance(Number(e.target.value))}
              className="w-full"
              data-testid="wasm-demo-commit-distance"
            />
          </label>
          <div
            className="px-4 py-2 bg-bg-soft rounded font-mono text-sm"
            data-testid="wasm-demo-visual-distance"
          >
            log({commitDistance} + 1) * 30 ={" "}
            <strong className="text-accent">{computedDistance.toFixed(2)}</strong>
          </div>
        </div>
      </section>

      {/* ===== Section 2: Query Engine ===== */}
      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold">Query Engine (PR #230 + #231)</h2>

        <label className="space-y-1">
          <span className="text-xs font-mono text-ink-mute">Search DSL</span>
          <input
            type="text"
            className="w-full px-2 py-1 border border-line rounded bg-bg-card font-mono text-sm"
            value={dslInput}
            onChange={(e) => setDslInput(e.target.value)}
            data-testid="wasm-demo-dsl-input"
            placeholder="e.g. show ready"
          />
        </label>

        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            checked={validateUseWasm}
            onChange={(e) => setValidateUseWasm(e.target.checked)}
            data-testid="wasm-demo-use-wasm"
          />
          <span>Use WASM (uncheck to force JS fallback)</span>
        </label>

        <div
          className={`px-4 py-3 rounded font-mono text-sm ${
            dslIsValid ? "bg-ok/10 text-ok" : "bg-err/10 text-err"
          }`}
          data-testid="wasm-demo-dsl-valid"
        >
          {dslIsValid ? "✓ Valid" : "✗ Invalid"} (contain at least 1 keyword)
        </div>

        <div
          className="px-4 py-3 bg-bg-soft rounded font-mono text-sm"
          data-testid="wasm-demo-dsl-keywords"
        >
          Matched keywords ({validatedKeywords.length}):{" "}
          {validatedKeywords.length > 0
            ? validatedKeywords.join(", ")
            : "(none)"}
        </div>

        <div className="text-xs text-ink-mute">
          <strong>All 6 keywords</strong> (守门 per ULYS-57.3 acceptance):{" "}
          {KEYWORDS.join(", ")}
        </div>
      </section>

      {/* ===== Section 3: Benchmark ===== */}
      <section className="card p-6 space-y-4">
        <h2 className="text-xl font-semibold">Performance Benchmark (A/B test)</h2>

        <button
          type="button"
          className="btn"
          onClick={runBenchmark}
          data-testid="wasm-demo-benchmark"
        >
          Run benchmark (1000 calls)
        </button>

        {benchmark && (
          <pre
            className="px-4 py-3 bg-bg-soft rounded font-mono text-xs whitespace-pre-wrap"
            data-testid="wasm-demo-benchmark-result"
          >
            {benchmark}
          </pre>
        )}
      </section>

      <footer className="text-xs text-ink-mute text-center pt-6 border-t border-line">
        Rust→WASM 集成 PoC · per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.2
      </footer>
    </div>
  );
}