// =====================================================================
// i18n Barrel — 单入口导出
// =====================================================================

export { I18nProvider, useI18n } from "./I18nProvider";
export type { I18nContextValue, I18nProviderProps } from "./I18nProvider";
export { useTranslation, interpolate } from "./useTranslation";
export {
  SUPPORTED_LANGUAGES,
  DEFAULT_LANGUAGE,
  LANGUAGE_META,
  STORAGE_KEY,
} from "./config";
export type { Language } from "./config";
