// =====================================================================
// useModuleTranslation — 按模块 id 拿翻译后字段
// =====================================================================
// 用于让 nav/registry 里静态的 ModuleDefinition 在 UI 层呈现 i18n 响应式
// label / description / categoryLabel.
//
// 用法:
//   const t = useModuleTranslation(module);   // module 来自 MODULE_MAP.get(id)
//   <span>{t.label}</span>
//
// 兜底: 找不到翻译时回退到 module.label (英文源) + 空 description/categoryLabel.
// =====================================================================

import { useTranslation } from "./useTranslation";
import type { ModuleDefinition } from "@/lib/nav/registry";

export interface ModuleTranslation {
  label: string;
  description: string;
  categoryLabel: string;
}

const FALLBACK: ModuleTranslation = {
  label: "",
  description: "",
  categoryLabel: "",
};

export function useModuleTranslation(
  module: Pick<ModuleDefinition, "id" | "label" | "description" | "categoryLabel"> | null | undefined
): ModuleTranslation {
  const { t } = useTranslation();
  if (!module) return FALLBACK;
  // 字典兜底: 若 id 未在字典中 (例如 registry 新增但未翻译), 回退到 registry 原值
  const localized = t.modules[module.id];
  if (!localized) {
    return {
      label: module.label,
      description: module.description,
      categoryLabel: module.categoryLabel,
    };
  }
  return localized;
}
