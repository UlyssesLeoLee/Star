"use client";

// Star Frontend — 主题切换器
// 顶栏图标按钮: 当前主题图标 (🌙/☀️) + 点击下拉切换.
// 设计简化 (per 2026-09-27 用户反馈: 顶栏过于拥挤, 不要 4 字主题名 "暗夜神格"/"少年原画" 文字说明).

import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { THEMES, type ThemeId } from "@/lib/theme/types";

/**
 * 主题切换器 — 顶栏 UserMenu 前.
 * 设计:
 * - 触发按钮: 仅图标 (🌙 dark / ☀️ light)
 * - 下拉: 列出 THEMES, 当前高亮
 * - 切换走 setTheme (next-themes API)
 * - 键盘: Cmd+Shift+T 循环
 */
export function ThemeSwitcher() {
  const { theme, setTheme } = useTheme();
  const [mounted, setMounted] = useState(false);
  const [open, setOpen] = useState(false);

  // 避免 SSR hydration 不匹配
  useEffect(() => setMounted(true), []);

  // 键盘: Cmd+Shift+T 循环
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "t") {
        e.preventDefault();
        const idx = THEMES.findIndex((t) => t.id === theme);
        const next = THEMES[(idx + 1) % THEMES.length];
        setTheme(next.id);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [theme, setTheme]);

  if (!mounted) {
    // Skeleton — icon-only 占位 (per 2026-09-27 简化)
    return (
      <div
        className="size-9 border-2 border-[var(--cel-ink)] bg-[color:var(--color-surface-2)] animate-pulse"
        aria-label="loading theme"
      />
    );
  }

  const getThemeIcon = (t?: (typeof THEMES)[number]) => {
    if (!t) return "🌙";
    if (t.id === "cel") return "🎞️";
    if (t.id === "shanghai") return "🪭";
    if (t.id === "charisma") return "✨";
    if (t.id === "christmas") return "🎄";
    return t.isDark ? "🌙" : "☀️";
  };

  const current = THEMES.find((t) => t.id === theme) ?? THEMES[0];

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="size-9 grid place-items-center text-base border-2 border-black bg-[var(--cel-surface-card,#0f1422)] hover:bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-primary,#ffffff)] transition-all cel-shadow shrink-0"
        aria-label="theme switcher"
        aria-expanded={open}
        title={`${current.displayName} (${current.id})`}
      >
        {getThemeIcon(current)}
      </button>

      {open && (
        <div
          role="listbox"
          className="absolute right-0 mt-2 w-48 border-2 border-black bg-[var(--cel-surface-card,#0f1422)] text-[var(--cel-text-primary,#ffffff)] cel-shadow-lg z-50 overflow-hidden"
        >
          {THEMES.map((t) => (
            <button
              key={t.id}
              role="option"
              aria-selected={t.id === theme}
              onClick={() => {
                setTheme(t.id as ThemeId);
                setOpen(false);
              }}
              className={`w-full text-left px-3 py-2 text-sm hover:bg-[var(--cel-surface-sub,#151c2c)] flex items-center justify-between border-b border-black/40 ${
                t.id === theme ? "text-[var(--cel-cyan,#00f0ff)] bg-[var(--cel-surface-sub,#151c2c)]" : ""
              }`}
            >
              <span className="flex items-center gap-2">
                <span className="text-base">{getThemeIcon(t)}</span>
                <span className="font-mono text-xs uppercase">{t.id}</span>
                <span className="text-[11px] text-[var(--cel-text-secondary,#94a3b8)]">({t.displayName})</span>
              </span>
              {t.id === theme && <span aria-hidden className="text-[var(--cel-cyan,#00f0ff)] font-black">✓</span>}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}