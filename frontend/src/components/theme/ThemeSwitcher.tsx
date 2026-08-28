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
        className="px-3 py-1.5 text-body rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] hover:bg-[color:var(--color-surface-2)] transition-colors"
        aria-label={`theme switcher, current ${current.displayName}`}
        aria-expanded={open}
      >
        <span className="mr-1.5">
          {current.isDark ? "🌙" : "☀️"}
        </span>
        {current.displayName}
        <span className="ml-1.5 text-micro opacity-60">▾</span>
      </button>

      {open && (
        <div
          role="listbox"
          className="absolute right-0 mt-1 w-44 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] shadow-[var(--shadow-lg)] z-50"
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
              className={`w-full text-left px-3 py-2 text-body hover:bg-[color:var(--color-surface-2)] flex items-center justify-between ${
                t.id === theme ? "font-semibold" : ""
              }`}
            >
              <span>
                <span className="mr-1.5">{t.isDark ? "🌙" : "☀️"}</span>
                {t.displayName}
              </span>
              {t.id === theme && <span aria-hidden>✓</span>}
            </button>
          ))}
          <div className="border-t border-[color:var(--color-border)] px-3 py-1.5 text-micro opacity-60">
            切换: Cmd+Shift+T
          </div>
        </div>
      )}
    </div>
  );
}
