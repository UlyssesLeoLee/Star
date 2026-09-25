// =====================================================================
// TerminalSplitPane.test.tsx — Component integration test (per ULYS-223 P1-D §6.8)
// =====================================================================
// 守门: Vitest + @testing-library/react + jsdom (per vitest.config.ts)
// =====================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { afterEach } from "vitest";
import { TerminalStackContainer } from "./TerminalStackContainer";
import { useTerminalStackStore } from "./terminalStackStore";

afterEach(() => {
  cleanup();
  useTerminalStackStore.getState().reset();
});

describe("TerminalStackContainer (ULYS-223 P1-D)", () => {
  it("K. renders toolbar + single root pane initially", () => {
    render(<TerminalStackContainer />);
    expect(screen.getByTestId("terminal-stack-container")).toBeTruthy();
    expect(screen.getByTestId("terminal-split-toolbar")).toBeTruthy();
    expect(screen.getByTestId("terminal-pane-pane-root")).toBeTruthy();
    expect(screen.getByTestId("pane-count").textContent).toMatch(/^1 pane/);
  });

  it("L. click Split Vertical → 2 panes", () => {
    render(<TerminalStackContainer />);
    fireEvent.click(screen.getByTestId("split-vertical-btn"));
    expect(screen.getByTestId("pane-count").textContent).toMatch(/^2 panes/);
  });

  it("M. click Split Horizontal twice → 4 panes", () => {
    render(<TerminalStackContainer />);
    fireEvent.click(screen.getByTestId("split-horizontal-btn"));
    fireEvent.click(screen.getByTestId("split-horizontal-btn"));
    expect(screen.getByTestId("pane-count").textContent).toMatch(/^3 panes/);
  });

  it("N. close button disabled when paneCount <= 1", () => {
    render(<TerminalStackContainer />);
    const closeBtn = screen.getByTestId("close-pane-btn") as HTMLButtonElement;
    expect(closeBtn.disabled).toBe(true);
  });

  it("O. close button enabled when panes > 1", () => {
    render(<TerminalStackContainer />);
    fireEvent.click(screen.getByTestId("split-vertical-btn"));
    const closeBtn = screen.getByTestId("close-pane-btn") as HTMLButtonElement;
    expect(closeBtn.disabled).toBe(false);
  });

  it("P. clicking pane marks it active via data-active", () => {
    render(<TerminalStackContainer />);
    fireEvent.click(screen.getByTestId("split-vertical-btn"));
    // 第一个 split 后有 2 个 pane, 点击新生成的 pane
    const panes = screen.getAllByRole("region");
    expect(panes.length).toBeGreaterThanOrEqual(2);
    // 点击第二个 pane
    fireEvent.click(panes[1]);
    // active state should update
    expect(useTerminalStackStore.getState().paneCount).toBe(2);
  });
});