import { describe, it, expect, afterEach } from "vitest";
import {
  parseHexColor,
  readThemeRgb,
  hexToRgb,
  mixRgb,
  resolveCelTokenPalette,
} from "../themeTokens";

describe("themeTokens", () => {
  afterEach(() => {
    document.documentElement.style.removeProperty("--cel-cyan");
    document.documentElement.style.removeProperty("--cel-crimson");
    document.documentElement.style.removeProperty("--cel-text-primary");
  });

  describe("parseHexColor", () => {
    it("parses #rrggbb to normalized rgb", () => {
      expect(parseHexColor("#ff0000")).toEqual([1, 0, 0]);
      expect(parseHexColor("#00f0ff")).toEqual([0, 240 / 255, 1]);
    });

    it("parses #rgb shorthand", () => {
      expect(parseHexColor("#f00")).toEqual([1, 0, 0]);
    });

    it("returns null for non-hex values", () => {
      expect(parseHexColor("rgb(1,2,3)")).toBeNull();
      expect(parseHexColor("")).toBeNull();
      expect(parseHexColor("#12345")).toBeNull();
    });

    it("trims whitespace", () => {
      expect(parseHexColor("  #000000  ")).toEqual([0, 0, 0]);
    });
  });

  describe("readThemeRgb", () => {
    it("reads a CSS custom property from documentElement", () => {
      document.documentElement.style.setProperty("--cel-cyan", "#00ff00");
      expect(readThemeRgb("--cel-cyan", [0, 0, 0])).toEqual([0, 1, 0]);
    });

    it("returns fallback when token is missing", () => {
      const fb: [number, number, number] = [0.5, 0.5, 0.5];
      expect(readThemeRgb("--nonexistent-token-xyz", fb)).toEqual(fb);
    });

    it("returns fallback when token value is not hex (e.g. rgba)", () => {
      document.documentElement.style.setProperty("--cel-cyan", "rgba(0,0,0,0.5)");
      const fb: [number, number, number] = [1, 1, 1];
      expect(readThemeRgb("--cel-cyan", fb)).toEqual(fb);
    });
  });

  describe("hexToRgb", () => {
    it("converts hex with fallback on failure", () => {
      expect(hexToRgb("#ffffff", [0, 0, 0])).toEqual([1, 1, 1]);
      expect(hexToRgb("nope", [0.2, 0.3, 0.4])).toEqual([0.2, 0.3, 0.4]);
    });
  });

  describe("mixRgb", () => {
    it("interpolates linearly and clamps t", () => {
      const a: [number, number, number] = [0, 0, 0];
      const b: [number, number, number] = [1, 1, 1];
      expect(mixRgb(a, b, 0)).toEqual([0, 0, 0]);
      expect(mixRgb(a, b, 1)).toEqual([1, 1, 1]);
      expect(mixRgb(a, b, 0.5)).toEqual([0.5, 0.5, 0.5]);
      expect(mixRgb(a, b, -1)).toEqual([0, 0, 0]);
      expect(mixRgb(a, b, 2)).toEqual([1, 1, 1]);
    });
  });

  describe("resolveCelTokenPalette", () => {
    it("derives shadow from base and reads rim/highlight tokens", () => {
      document.documentElement.style.setProperty("--cel-cyan", "#00f0ff");
      document.documentElement.style.setProperty("--cel-crimson", "#ff184c");
      document.documentElement.style.setProperty("--cel-text-primary", "#ffffff");
      const p = resolveCelTokenPalette({
        baseToken: "--cel-cyan",
        rimToken: "--cel-crimson",
        fallback: { base: [0, 0, 0], shadow: [0, 0, 0], highlight: [0, 0, 0], rim: [0, 0, 0] },
      });
      expect(p.base).toEqual([0, 240 / 255, 1]);
      expect(p.rim).toEqual([1, 24 / 255, 76 / 255]);
      expect(p.highlight).toEqual([1, 1, 1]);
      // shadow = base × 0.16
      expect(p.shadow[0]).toBeCloseTo(0, 5);
      expect(p.shadow[1]).toBeCloseTo((240 / 255) * 0.16, 5);
      expect(p.shadow[2]).toBeCloseTo(0.16, 5);
    });

    it("uses fallback palette when tokens absent", () => {
      const fb = {
        base: [1, 0, 0] as [number, number, number],
        shadow: [0.1, 0, 0] as [number, number, number],
        highlight: [1, 1, 1] as [number, number, number],
        rim: [0, 1, 0] as [number, number, number],
      };
      const p = resolveCelTokenPalette({
        baseToken: "--missing-base",
        rimToken: "--missing-rim",
        fallback: fb,
      });
      expect(p.base).toEqual(fb.base);
      expect(p.rim).toEqual(fb.rim);
      expect(p.highlight).toEqual(fb.highlight);
    });
  });
});
