"use client";

// Star Frontend — 主题切换器
// Per 2026-08-29 04:09 JST 主题决策: 顶栏下拉, 当前主题高亮, Cmd+Shift+T 切换.

import { useTheme } from "next-themes";
import { useEffect, useState } from "react";
import { THEMES, type ThemeId } from "@/lib/theme/types";

/**
 * 主题切换器 — 顶栏第 6 区"用户菜单"前.
 * 设计:
 * - 触发按钮: 当前主题名 + 切换图标
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
    // Skeleton (per ui-3pane-arch.md §3.7)
    return (
      <div
        className="px-3 py-1.5 text-body rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface-2)] animate-pulse"
        aria-label="loading theme"
      >
        <span className="opacity-0">--</span>
      </div>
    );
  }

  const current = THEMES.find((t) => t.id === theme) ?? THEMES[0];

  return (
    <div className="relative">
      <button
        type="button"
        onClick={() => setOpen((o) => !o)}
        className="px-2.5 py-1 text-xs font-bold font-mono border-2 border-black bg-[var(--cel-surface-card,#0f1422)] hover:bg-[var(--cel-surface-sub,#151c2c)] text-[var(--cel-text-primary,#ffffff)] transition-all cel-shadow flex items-center gap-1.5"
        aria-label={`theme switcher, current ${current.displayName}`}
        aria-expanded={open}
      >
        <span className="text-sm">
          {current.isDark ? "🌙" : "☀️"}
        </span>
        <span className="font-black tracking-wider text-[11px] uppercase">
          {current.displayName}
        </span>
        <span className="text-[10px] opacity-60">▾</span>
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
              className={`w-full text-left px-3 py-2 text-xs font-mono font-bold hover:bg-[var(--cel-surface-sub,#151c2c)] flex items-center justify-between border-b border-black/40 ${
                t.id === theme ? "text-[var(--cel-cyan,#00f0ff)] bg-[var(--cel-surface-sub,#151c2c)]" : ""
              }`}
            >
              <span className="flex items-center gap-2">
                <span className="text-sm">{t.isDark ? "🌙" : "☀️"}</span>
                <span>{t.displayName}</span>
              </span>
              {t.id === theme && <span aria-hidden className="text-[var(--cel-cyan,#00f0ff)] font-black">✓</span>}
            </button>
          ))}
          <div className="border-t-2 border-black px-3 py-1.5 text-[9px] font-mono opacity-70 bg-black/20">
            快捷键: Cmd+Shift+T
          </div>
        </div>
      )}
    </div>
  );
}
