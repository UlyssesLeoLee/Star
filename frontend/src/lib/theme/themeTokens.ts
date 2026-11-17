// =====================================================================
// themeTokens — 主题 CSS token 的单一读取口 (Three.js 效果统一取色)
// =====================================================================
// 背景 (per 2026-11-17 主题风格统一):
//   此前 PhysisBackdrop / GasParticlesField / Cel3DUI / TacticalCore3D
//   各自硬编码 hex, 导致 christmas/charisma/shanghai/cel/physis 主题下
//   3D 内容仍渲染 Neo-Tokyo 青/绯红配色 — 风格不统一。
//   本模块让所有 Three.js 效果从 globals.css 的主题 token 取色:
//   改主题只改 CSS 一处, 3D 内容自动跟随。
//
// 约定:
//   - token 值一律 #rrggbb (CSS 侧保证); rgba()/渐变 token 不走这里
//   - SSR 安全: 非浏览器环境返回 fallback
//   - 读取失败 (token 缺失/格式异常) 返回 fallback, 不抛错
// =====================================================================

/** 解析 #rgb / #rrggbb 为 [r, g, b] (0..1); 失败返回 null */
export function parseHexColor(value: string): [number, number, number] | null {
  const v = value.trim();
  if (/^#[0-9a-fA-F]{6}$/.test(v)) {
    const n = parseInt(v.slice(1), 16);
    return [((n >> 16) & 255) / 255, ((n >> 8) & 255) / 255, (n & 255) / 255];
  }
  if (/^#[0-9a-fA-F]{3}$/.test(v)) {
    const r = parseInt(v[1] + v[1], 16) / 255;
    const g = parseInt(v[2] + v[2], 16) / 255;
    const b = parseInt(v[3] + v[3], 16) / 255;
    return [r, g, b];
  }
  return null;
}

/**
 * 读取一个主题 CSS token 为归一化 RGB。
 * 非浏览器 / token 缺失 / 非 hex 值时返回 fallback。
 */
export function readThemeRgb(
  token: string,
  fallback: [number, number, number],
): [number, number, number] {
  if (typeof window === "undefined" || typeof document === "undefined") {
    return fallback;
  }
  const raw = getComputedStyle(document.documentElement).getPropertyValue(token);
  return parseHexColor(raw) ?? fallback;
}

/** hex 字符串 → 归一化 RGB (失败返回 fallback) */
export function hexToRgb(
  hex: string,
  fallback: [number, number, number],
): [number, number, number] {
  return parseHexColor(hex) ?? fallback;
}

/** 两个归一化 RGB 线性混合 (t=0 → a, t=1 → b) */
export function mixRgb(
  a: [number, number, number],
  b: [number, number, number],
  t: number,
): [number, number, number] {
  const k = Math.min(1, Math.max(0, t));
  return [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k];
}

/**
 * cel 调色板四色组 (base/shadow/highlight/rim) 的 token 解析。
 * base 取语义色 token; shadow = base × 0.16 (赛璐璐暗阶);
 * highlight = 纸面高光 token; rim = 对比 rim token。
 */
export interface CelTokenPalette {
  base: [number, number, number];
  shadow: [number, number, number];
  highlight: [number, number, number];
  rim: [number, number, number];
}

export function resolveCelTokenPalette(opts: {
  baseToken: string;
  rimToken: string;
  highlightToken?: string;
  fallback: CelTokenPalette;
}): CelTokenPalette {
  const base = readThemeRgb(opts.baseToken, opts.fallback.base);
  const rim = readThemeRgb(opts.rimToken, opts.fallback.rim);
  const highlight = readThemeRgb(
    opts.highlightToken ?? "--cel-text-primary",
    opts.fallback.highlight,
  );
  const shadow = mixRgb(base, [0, 0, 0], 0.84);
  return { base, shadow, highlight, rim };
}
