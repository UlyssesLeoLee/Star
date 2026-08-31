// =====================================================================
// i18n Config — 支持 3 种语言 (per 2026-08-31 用户拍板)
// =====================================================================
// 默认 zh-CN (per 用户发令"默认应该是中文版本")
// 右上角菜单可下拉选择 zh-CN / en / ja
// =====================================================================

export const SUPPORTED_LANGUAGES = ["zh-CN", "en", "ja"] as const;
export type Language = (typeof SUPPORTED_LANGUAGES)[number];

export const DEFAULT_LANGUAGE: Language = "zh-CN";

export const LANGUAGE_META: Record<
  Language,
  { label: string; nativeLabel: string; flag: string; htmlLang: string }
> = {
  "zh-CN": {
    label: "Chinese (Simplified)",
    nativeLabel: "简体中文",
    flag: "🇨🇳",
    htmlLang: "zh-CN",
  },
  en: {
    label: "English",
    nativeLabel: "English",
    flag: "🇺🇸",
    htmlLang: "en",
  },
  ja: {
    label: "Japanese",
    nativeLabel: "日本語",
    flag: "🇯🇵",
    htmlLang: "ja",
  },
};

export const STORAGE_KEY = "star-language";
