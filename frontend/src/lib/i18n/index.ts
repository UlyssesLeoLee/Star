// =====================================================================
// i18n Barrel — 单入口导出
// =====================================================================

export { I18nProvider, useI18n } from "./I18nProvider";
export type { I18nContextValue, I18nProviderProps } from "./I18nProvider";
export { useTranslation, interpolate } from "./useTranslation";
export { useModuleTranslation } from "./useModuleTranslation";
export type { ModuleTranslation } from "./useModuleTranslation";
export {
  SUPPORTED_LANGUAGES,
  DEFAULT_LANGUAGE,
  LANGUAGE_META,
  STORAGE_KEY,
} from "./config";
export type { Language } from "./config";
export type { Dictionary, ModuleI18n, CategoryDef } from "./dictionary";
