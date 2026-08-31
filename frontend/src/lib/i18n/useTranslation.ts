// =====================================================================
// useTranslation — 翻译 hook + 占位符替换工具
// =====================================================================
// 静态用法:   const { t } = useTranslation();
//             <span>{t.userMenu.signOut}</span>
// 插值用法:   const { t, tx } = useTranslation();
//             <span>{tx(t.userMenu.signOut)}</span>   // 无参数等同 t.userMenu.signOut
//             <span>{tx(t.appHeader.removeFromHeader, { label: tab.label })}</span>
// =====================================================================

import { useI18n } from "./I18nProvider";
import type { Dictionary } from "./dictionary";

/** Replace {key} placeholders with values from params. */
export function interpolate(
  template: string,
  params?: Record<string, string | number>
): string {
  if (!params) return template;
  return template.replace(/\{(\w+)\}/g, (match, key: string) => {
    const v = params[key];
    return v === undefined || v === null ? match : String(v);
  });
}

/**
 * Hook returning the current Dictionary (typed) plus helpers.
 *
 * - `t` — current language's dictionary (typed as Dictionary)
 * - `tx` — interpolate a string template with {name}-style placeholders
 * - `language` / `setLanguage` / `mounted` / `htmlLang` — passthrough
 */
export function useTranslation() {
  const { t, language, setLanguage, mounted, htmlLang } = useI18n();
  return {
    t: t as Dictionary,
    /** Translate with {placeholder} interpolation. */
    tx: (
      template: string,
      params?: Record<string, string | number>
    ) => interpolate(template, params),
    language,
    setLanguage,
    mounted,
    htmlLang,
  };
}
