/*
@cypher schema=1 source_sha256=a0f676b20b36ca06fe9bd27dbc8135a9efbed6bea989e821ef4feca26588a1d0
MERGE (self:File {path:"frontend/src/components/theme/ThemeProvider.test.tsx"})
MERGE (provider:File {path:"frontend/src/components/theme/ThemeProvider.tsx"})
MERGE (self)-[:IMPORTS]->(provider)
MERGE (self)-[:TESTS]->(provider)
@endcypher
*/
import { cleanup, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ThemeProvider } from "./ThemeProvider";

beforeEach(() => {
  vi.stubGlobal("matchMedia", vi.fn(() => ({
    matches: false,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  })));
});

afterEach(() => {
  cleanup();
  localStorage.removeItem("star-theme");
  document.documentElement.classList.remove("christmas", "light", "physis");
  vi.unstubAllGlobals();
});

describe("ThemeProvider", () => {
  it("migrates a saved retired theme to the default theme", async () => {
    localStorage.setItem("star-theme", "christmas");
    document.documentElement.classList.add("christmas");

    render(<ThemeProvider><div>content</div></ThemeProvider>);

    await waitFor(() => {
      expect(localStorage.getItem("star-theme")).toBe("light");
      expect(document.documentElement.classList.contains("christmas")).toBe(false);
      expect(document.documentElement.classList.contains("light")).toBe(true);
    });
  });

  it("restores the Physis theme", async () => {
    localStorage.setItem("star-theme", "physis");

    render(<ThemeProvider><div>content</div></ThemeProvider>);

    await waitFor(() => expect(document.documentElement.classList.contains("physis")).toBe(true));
    expect(localStorage.getItem("star-theme")).toBe("physis");
  });
});
