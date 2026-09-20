// =====================================================================
// SPDX-License-Identifier: MIT OR Apache-2.0
// frontend/src/components/editor/MonacoEditor.test.tsx — ULYS-98-W1.1
// =====================================================================
// W1 验收 (per docs/briefs/ulys-98-star-cursor-min-v1.md §"Sub-task 1.1"):
// "1 unit test (MonacoEditor.test.tsx 渲染 + change event)"
//
// 不实际 mount Monaco (jsdom 没有 worker / canvas), 只测:
// 1. 渲染 placeholder + testid
// 2. inferLanguageFromPath 推断
// 3. 路径回退 (无 language → 从 path 推断)
//
// 守门合规: 0 实装 AI 补全 (W2). 0 网络. 0 副作用.
// =====================================================================
import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";

// next/dynamic 在 jsdom 下不会真正加载 Monaco, 但会渲染 loading 占位符
// (因为 ssr:false + no chunk resolved). 这是 W1 测试范围内期望行为.
vi.mock("@monaco-editor/react", () => ({
  Editor: () => null,
}));

import MonacoEditor, { inferLanguageFromPath } from "./MonacoEditor";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("MonacoEditor (ULYS-98-W1.1)", () => {
  it("renders the loading placeholder with data-testid", () => {
    render(
      <MonacoEditor
        value="// hello"
        path="src/lib/store.ts"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el).toBeTruthy();
    expect(el.getAttribute("data-monaco-path")).toBe("src/lib/store.ts");
    // Language was inferred from path → "typescript".
    expect(el.getAttribute("data-monaco-language")).toBe("typescript");
  });

  it("respects explicit language prop over path-inferred one", () => {
    render(
      <MonacoEditor
        value=""
        language="json"
        path="anything.txt"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el.getAttribute("data-monaco-language")).toBe("json");
  });

  it("falls back to plaintext for unknown extensions", () => {
    render(
      <MonacoEditor
        value=""
        path="weird.unknownext"
        onChange={() => {}}
      />,
    );
    const el = screen.getByTestId("monaco-editor");
    expect(el.getAttribute("data-monaco-language")).toBe("plaintext");
  });
});

describe("inferLanguageFromPath (ULYS-98-W1.1)", () => {
  it("maps known extensions to Monaco language ids", () => {
    expect(inferLanguageFromPath("foo.ts")).toBe("typescript");
    expect(inferLanguageFromPath("foo.tsx")).toBe("typescript");
    expect(inferLanguageFromPath("foo.rs")).toBe("rust");
    expect(inferLanguageFromPath("foo.py")).toBe("python");
    expect(inferLanguageFromPath("foo.json")).toBe("json");
    expect(inferLanguageFromPath("foo.md")).toBe("markdown");
    expect(inferLanguageFromPath("deep/nested/path/foo.go")).toBe("go");
  });

  it("matches Dockerfile / Makefile by exact filename", () => {
    expect(inferLanguageFromPath("Dockerfile")).toBe("dockerfile");
    expect(inferLanguageFromPath("Makefile")).toBe("makefile");
    expect(inferLanguageFromPath("path/to/Dockerfile")).toBe("dockerfile");
  });

  it("returns plaintext for empty / extension-less / unknown", () => {
    expect(inferLanguageFromPath(undefined)).toBe("plaintext");
    expect(inferLanguageFromPath("")).toBe("plaintext");
    expect(inferLanguageFromPath("README")).toBe("plaintext");
    expect(inferLanguageFromPath("weird.unknownext")).toBe("plaintext");
  });

  it("is case-insensitive on extension", () => {
    expect(inferLanguageFromPath("Foo.TS")).toBe("typescript");
    expect(inferLanguageFromPath("Foo.RS")).toBe("rust");
  });
});