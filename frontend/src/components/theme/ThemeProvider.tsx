"use client";

/*
@cypher schema=1 source_sha256=08c63bcc29eb627986732641c7c987bfda49931f4b0c14b23b2700bd19d238eb
MERGE (self:File {path:"frontend/src/components/theme/ThemeProvider.tsx"})
MERGE (provider:Symbol {id:"frontend/src/components/theme/ThemeProvider.tsx::ThemeProvider",kind:"function"})
MERGE (migration:Symbol {id:"frontend/src/components/theme/ThemeProvider.tsx::RetiredThemeMigration",kind:"function"})
MERGE (self)-[:DEFINES]->(provider)
MERGE (self)-[:DEFINES]->(migration)
MERGE (provider)-[:CALLS]->(migration)
@endcypher
*/

// Star Frontend — 主题 Provider
// Per 2026-08-29 04:09 JST 主题决策: next-themes + 三层作用域 (Personal > Tenant > Global).
// 包装 next-themes, 强制 attribute=class, 避免 SSR flash.

import { ThemeProvider as NextThemesProvider, useTheme } from "next-themes";
import { useEffect } from "react";
import type { ComponentProps, ReactNode } from "react";

type NextThemesProps = ComponentProps<typeof NextThemesProvider>;

/** Star 主题 Provider props */
export interface StarThemeProviderProps {
  children: ReactNode;
  /** 强制 defaultTheme (默认 "light") */
  defaultTheme?: "light" | "dark" | "charisma" | "shanghai" | "cel" | "physis";
  /** 扩展 ThemeId 列表 (供 next-themes 已知列表) */
  themes?: string[];
}

/**
 * Star 主题 Provider — 包装 next-themes.
 * - attribute="class" → 在 html 标签加当前主题类名
 * - enableSystem={false} → 不跟随系统, 用户显式选
 * - storageKey="star-theme" → 与其他站点隔离
 * - 三层解析 (Personal / Tenant / Global) 由调用方 (api/theme.ts hook) 走, 此处只负责 next-themes 状态.
 */
function RetiredThemeMigration() {
  const { theme, setTheme } = useTheme();

  useEffect(() => {
    document.documentElement.classList.remove("christmas");
    if (theme === "christmas") {
      setTheme("light");
    }
  }, [theme, setTheme]);

  return null;
}

export function ThemeProvider({
  children,
  defaultTheme = "light",
  themes = ["light", "dark", "charisma", "shanghai", "cel", "physis"],
}: StarThemeProviderProps) {
  const props: Omit<NextThemesProps, "children"> = {
    attribute: "class",
    defaultTheme,
    enableSystem: false,
    storageKey: "star-theme",
    themes,
  };
  return (
    <NextThemesProvider {...props}>
      <RetiredThemeMigration />
      {children}
    </NextThemesProvider>
  );
}
