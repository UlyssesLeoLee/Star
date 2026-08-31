// =====================================================================
// I18nProvider — 语言上下文 Provider (per 2026-08-31 用户拍板)
// =====================================================================
// - 默认 zh-CN (per 用户发令"默认应该是中文版本")
// - 持久化到 localStorage (key: star-language)
// - 同步 <html lang="..."> 属性 (SEO + a11y)
// - mount 后才读取 localStorage 避免 SSR hydration mismatch
// =====================================================================
"use client";

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import {
  DEFAULT_LANGUAGE,
  LANGUAGE_META,
  STORAGE_KEY,
  SUPPORTED_LANGUAGES,
  type Language,
} from "./config";
import { zhCN } from "./zh-CN";
import { en } from "./en";
import { ja } from "./ja";
import type { Dictionary } from "./dictionary";

const DICTIONARIES: Record<Language, Dictionary> = {
  "zh-CN": zhCN,
  en,
  ja,
};

export interface I18nContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: Dictionary;
  /** "zh-CN" | "en" | "ja" — use for <html lang="..."> and Intl.* formatters */
  htmlLang: string;
  /** True after first client mount (avoids SSR hydration mismatch) */
  mounted: boolean;
}

const I18nContext = createContext<I18nContextValue | null>(null);
// 导出供 useStatusLabel 等需要直接读 ctx 的内部 hook 使用 (避免重复 createContext)
export { I18nContext };

function isSupportedLanguage(value: string | null): value is Language {
  return (
    typeof value === "string" &&
    (SUPPORTED_LANGUAGES as readonly string[]).includes(value)
  );
}

function readStoredLanguage(): Language {
  if (typeof window === "undefined") return DEFAULT_LANGUAGE;
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (isSupportedLanguage(raw)) return raw;
  } catch {
    // localStorage may throw in private mode; fall through to default
  }
  return DEFAULT_LANGUAGE;
}

export interface I18nProviderProps {
  children: ReactNode;
  /** Optional override for tests / Storybook */
  initialLanguage?: Language;
}

export function I18nProvider({
  children,
  initialLanguage,
}: I18nProviderProps) {
  // SSR-safe initial state: DEFAULT_LANGUAGE on server, then on client:
  //   - if localStorage has a value, use it (user preference wins)
  //   - else if initialLanguage prop is set (tests / Storybook), use it
  //   - else fall back to DEFAULT_LANGUAGE
  //
  // We deliberately do NOT read storage in useState initializer because:
  //   1) useState initializer runs on the server too — no window there
  //   2) reading on client during render would cause hydration mismatch
  // Instead we read in useEffect after mount; first render is always DEFAULT_LANGUAGE
  // to match server output.
  const [language, setLanguageState] = useState<Language>(
    initialLanguage ?? DEFAULT_LANGUAGE
  );
  const [mounted, setMounted] = useState(false);

  // After mount: read stored preference and sync <html lang>
  useEffect(() => {
    setMounted(true);
    if (typeof window === "undefined") return;
    let stored: Language = DEFAULT_LANGUAGE;
    try {
      const raw = window.localStorage.getItem(STORAGE_KEY);
      if (isSupportedLanguage(raw)) {
        stored = raw;
      }
    } catch {
      // ignore storage failures
    }
    // If storage has a real user preference, it always wins.
    // initialLanguage only acts as a fallback when storage is empty.
    const hasStoredPreference = (() => {
      try {
        return window.localStorage.getItem(STORAGE_KEY) !== null;
      } catch {
        return false;
      }
    })();
    if (hasStoredPreference && stored !== language) {
      setLanguageState(stored);
    }
  }, []);

  // Sync <html lang="..."> whenever language changes
  useEffect(() => {
    if (typeof document === "undefined") return;
    document.documentElement.lang = LANGUAGE_META[language].htmlLang;
  }, [language]);

  const setLanguage = useCallback((next: Language) => {
    setLanguageState(next);
    try {
      if (typeof window !== "undefined") {
        window.localStorage.setItem(STORAGE_KEY, next);
      }
    } catch {
      // ignore storage failures
    }
  }, []);

  const value = useMemo<I18nContextValue>(
    () => ({
      language,
      setLanguage,
      t: DICTIONARIES[language],
      htmlLang: LANGUAGE_META[language].htmlLang,
      mounted,
    }),
    [language, setLanguage, mounted]
  );

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    throw new Error("useI18n must be used inside <I18nProvider>");
  }
  return ctx;
}
