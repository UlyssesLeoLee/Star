// =====================================================================
// frontend/src/app/wasm-demo/page.test.tsx
// =====================================================================
// vitest unit tests for WASM Demo Page (per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.2)
//
// 测试范围:
//   - 渲染 4 sections (layout, query, benchmark, footer)
//   - 默认 view mode = "tree" + node count = 50 → algo = dagre
//   - 切换 view mode + node count → algo 更新 (pure JS fallback path)
//   - 默认 dsl input 包含 4 keyword → valid + matched list
//   - Benchmark 按钮 click 后显示结果
//
// 已知缺口 (per 缺标比错标):
//   - WASM 加载是 async, 真实 WASM 测试需 jsdom + node-wasm (留后续 PR)
//   - 路由 next/navigation mock 沿用 KanbanBoard 模式
// =====================================================================

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import type { ReactNode } from "react";
import WasmDemoPage from "./page";

// ---- mock next/navigation ----
const mockRouterPush = vi.fn();
vi.mock("next/navigation", () => ({
  useRouter: () => ({ push: mockRouterPush, replace: vi.fn(), prefetch: vi.fn() }),
  usePathname: () => "/wasm-demo",
  useSearchParams: () => new URLSearchParams(),
}));

function renderPage(ui: ReactNode = <WasmDemoPage />) {
  return render(ui);
}

describe("WasmDemoPage", () => {
  beforeEach(() => {
    cleanup();
  });

  it("renders all 3 sections + footer", () => {
    renderPage();

    // Section 1: Layout Engine
    expect(screen.getByText(/Layout Engine/)).toBeTruthy();

    // Section 2: Query Engine
    expect(screen.getByText(/Query Engine/)).toBeTruthy();

    // Section 3: Benchmark
    expect(screen.getByText(/Performance Benchmark/)).toBeTruthy();

    // Footer
    expect(screen.getByText(/Rust→WASM 集成 PoC/)).toBeTruthy();
  });

  it("shows WASM status (JS fallback when not loaded)", () => {
    renderPage();

    // Default: WASM not loaded → JS fallback
    expect(screen.getAllByText(/JS fallback/).length).toBeGreaterThanOrEqual(2);
  });

  it("default view mode + node count → tree/dagre", () => {
    renderPage();

    // Default: viewMode="tree", nodeCount=50 → algo="dagre" (per layout-engine/src/engine.rs:184)
    expect(screen.getByTestId("wasm-demo-view-mode")).toHaveValue("tree");
    expect(screen.getByTestId("wasm-demo-node-count")).toHaveValue("50");

    // Layout result text contains "dagre"
    const result = screen.getByTestId("wasm-demo-layout-result");
    expect(result.textContent).toMatch(/dagre/);
  });

  it("tree + 200 nodes → elk", () => {
    renderPage();

    const slider = screen.getByTestId("wasm-demo-node-count") as HTMLInputElement;
    // fireEvent.change to 200 (per spec: tree ≥ 100 → elk)
    fireEvent.change(slider, { target: { value: "200" } });

    expect(screen.getByTestId("wasm-demo-layout-result").textContent).toMatch(/elk/);
  });

  it("dependency view → always d3_force", () => {
    renderPage();

    fireEvent.change(screen.getByTestId("wasm-demo-view-mode"), {
      target: { value: "dependency" },
    });

    expect(screen.getByTestId("wasm-demo-layout-result").textContent).toMatch(/d3_force/);
  });

  it("agent + 100 nodes → elk, agent + 30 → d3_force", () => {
    renderPage();

    fireEvent.change(screen.getByTestId("wasm-demo-view-mode"), {
      target: { value: "agent" },
    });

    // 100 nodes (> 50) → elk
    fireEvent.change(screen.getByTestId("wasm-demo-node-count"), { target: { value: "100" } });
    expect(screen.getByTestId("wasm-demo-layout-result").textContent).toMatch(/elk/);

    // 30 nodes (≤ 50) → d3_force
    fireEvent.change(screen.getByTestId("wasm-demo-node-count"), { target: { value: "30" } });
    expect(screen.getByTestId("wasm-demo-layout-result").textContent).toMatch(/d3_force/);
  });

  it("visualDistance formula: log(cd+1)*30", () => {
    renderPage();

    // Default commitDistance = 3 → log(4)*30 ≈ 41.59
    expect(screen.getByTestId("wasm-demo-visual-distance").textContent).toMatch(
      /log\(3 \+ 1\) \* 30 = 41\.59/,
    );
  });

  it("default DSL input 'show ready agent codex' → valid + 4 keywords", () => {
    renderPage();

    expect(screen.getByTestId("wasm-demo-dsl-input")).toHaveValue(
      "show ready agent codex",
    );
    expect(screen.getByTestId("wasm-demo-dsl-valid").textContent).toMatch(/Valid/);
    expect(screen.getByTestId("wasm-demo-dsl-keywords").textContent).toMatch(
      /show, agent/,
    );
  });

  it("empty DSL → invalid + 0 keywords", () => {
    renderPage();

    fireEvent.change(screen.getByTestId("wasm-demo-dsl-input"), {
      target: { value: "" },
    });

    expect(screen.getByTestId("wasm-demo-dsl-valid").textContent).toMatch(/Invalid/);
    expect(screen.getByTestId("wasm-demo-dsl-keywords").textContent).toMatch(
      /Matched keywords \(0\)/,
    );
  });

  it("benchmark button click → show result", () => {
    renderPage();

    // Before click: no benchmark result
    expect(screen.queryByTestId("wasm-demo-benchmark-result")).toBeNull();

    fireEvent.click(screen.getByTestId("wasm-demo-benchmark"));

    // After click: benchmark result visible
    expect(screen.queryByTestId("wasm-demo-benchmark-result")).toBeTruthy();
  });

  it("force JS fallback via checkbox uncheck → still uses JS-only path", () => {
    renderPage();

    const checkbox = screen.getByTestId("wasm-demo-use-wasm") as HTMLInputElement;
    expect(checkbox.checked).toBe(true);

    fireEvent.click(checkbox);
    expect(checkbox.checked).toBe(false);

    // 即使 WASM loaded, 也走 JS path (force fallback)
    // 这里 JS fallback 同样能正常解析 (per parseDslJS)
    expect(screen.getByTestId("wasm-demo-dsl-keywords").textContent).toMatch(/show/);
  });
});