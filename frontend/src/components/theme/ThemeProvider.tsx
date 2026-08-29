"use client";

// Star Frontend — 主题 Provider
// Per 2026-08-29 04:09 JST 主题决策: next-themes + 三层作用域 (Personal > Tenant > Global).
// 包装 next-themes, 强制 attribute=class, 避免 SSR flash.

import { ThemeProvider as NextThemesProvider } from "next-themes";
import type { ComponentProps, ReactNode } from "react";

type NextThemesProps = ComponentProps<typeof NextThemesProvider>;

/** Star 主题 Provider props */
export interface StarThemeProviderProps {
  children: ReactNode;
  /** 强制 defaultTheme (默认 "light") */
  defaultTheme?: "light" | "dark";
  /** 扩展 ThemeId 列表 (供 next-themes 已知列表) */
  themes?: string[];
}

/**
 * Star 主题 Provider — 包装 next-themes.
 * - attribute="class" → 在 html 标签加 .dark / .light
 * - enableSystem={false} → 不跟随系统, 用户显式选
 * - storageKey="star-theme" → 与其他站点隔离
 * - 三层解析 (Personal / Tenant / Global) 由调用方 (api/theme.ts hook) 走, 此处只负责 next-themes 状态.
 */
export function ThemeProvider({
  children,
  defaultTheme = "light",
  themes = ["light", "dark"],
}: StarThemeProviderProps) {
  const props: Omit<NextThemesProps, "children"> = {
    attribute: "class",
    defaultTheme,
    enableSystem: false,
    storageKey: "star-theme",
    themes,
  };
  return <NextThemesProvider {...props}>{children}</NextThemesProvider>;
}
