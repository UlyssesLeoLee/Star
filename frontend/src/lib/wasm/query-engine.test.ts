// =====================================================================
// frontend/src/lib/wasm/query-engine.test.ts
// =====================================================================
// vitest unit tests for query-engine JS fallback (no WASM required)
// Per docs/architecture/2026-09-28-upgrade/rust-to-wasm-frontend-memory-research.md §3.1
// + frontend/src/lib/wasm/query-engine.ts + PR #230 (Rust query-engine-wasm PoC)
// =====================================================================

import { describe, it, expect } from "vitest";
import {
  validateDslJS,
  parseDslJS,
  keywordAtJS,
  KEYWORDS,
  KEYWORD_COUNT,
} from "./query-engine";

describe("query-engine JS fallback", () => {
  describe("validateDslJS — same logic as Rust validate_dsl", () => {
    it("empty string → false", () => {
      expect(validateDslJS("")).toBe(false);
    });

    it("whitespace-only → false", () => {
      expect(validateDslJS("   ")).toBe(false);
      expect(validateDslJS("\t\n")).toBe(false);
    });

    it("contains at least 1 keyword → true", () => {
      expect(validateDslJS("show ready")).toBe(true);
      expect(validateDslJS("agent codex")).toBe(true);
      expect(validateDslJS("behind >5")).toBe(true);
      expect(validateDslJS("ahead >3")).toBe(true);
      expect(validateDslJS("health <80")).toBe(true);
      expect(validateDslJS("modified src/foo.rs")).toBe(true);
    });

    it("no keyword → false", () => {
      expect(validateDslJS("foo bar baz")).toBe(false);
      expect(validateDslJS("hello world")).toBe(false);
    });
  });

  describe("parseDslJS — keyword substring matching", () => {
    it("single keyword match", () => {
      expect(parseDslJS("show ready")).toEqual(["show"]);
      expect(parseDslJS("agent codex")).toEqual(["agent"]);
    });

    it("multiple keyword matches", () => {
      const result = parseDslJS("show ready agent codex");
      expect(result).toContain("show");
      expect(result).toContain("agent");
      expect(result).toHaveLength(2);
    });

    it("all 6 keywords match", () => {
      const result = parseDslJS("show agent behind ahead health modified");
      expect(result).toHaveLength(6);
      expect(new Set(result)).toEqual(new Set(KEYWORDS));
    });

    it("empty / no match → empty array", () => {
      expect(parseDslJS("")).toEqual([]);
      expect(parseDslJS("foo bar")).toEqual([]);
    });

    it("case-sensitive (matches Rust str::contains)", () => {
      // 大小写敏感 — Rust str::contains 是 case-sensitive
      expect(parseDslJS("Show ready")).toEqual(["ready"]); // 'Show' ≠ 'show'
      expect(parseDslJS("SHOW READY")).toEqual([]);
    });
  });

  describe("keywordAtJS — index lookup", () => {
    it("valid indices 0-5", () => {
      expect(keywordAtJS(0)).toBe("show");
      expect(keywordAtJS(1)).toBe("agent");
      expect(keywordAtJS(2)).toBe("behind");
      expect(keywordAtJS(3)).toBe("ahead");
      expect(keywordAtJS(4)).toBe("health");
      expect(keywordAtJS(5)).toBe("modified");
    });

    it("out-of-bounds → null", () => {
      expect(keywordAtJS(-1)).toBeNull();
      expect(keywordAtJS(6)).toBeNull();
      expect(keywordAtJS(100)).toBeNull();
    });
  });

  describe("constants", () => {
    it("KEYWORDS has 6 entries", () => {
      expect(KEYWORDS).toHaveLength(6);
    });

    it("KEYWORD_COUNT = 6 (守门 #11: 6 keywords per ULYS-57.3 acceptance)", () => {
      expect(KEYWORD_COUNT).toBe(6);
    });

    it("KEYWORDS order matches Rust enum order", () => {
      expect(KEYWORDS[0]).toBe("show");
      expect(KEYWORDS[1]).toBe("agent");
      expect(KEYWORDS[2]).toBe("behind");
      expect(KEYWORDS[3]).toBe("ahead");
      expect(KEYWORDS[4]).toBe("health");
      expect(KEYWORDS[5]).toBe("modified");
    });
  });
});