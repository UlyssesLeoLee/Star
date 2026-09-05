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
// 兜底策略 (per 2026-09-05 拍板 missing_opt1 缺标比错标安全):
//   1. 优先查 t.navModules[module.id] (新 v0.6 命名空间)
//   2. 兼容查 t.modules[module.id] (旧 v0.2 命名空间, modules.inbox.label = 收件箱)
//   3. 都没有时, 开发环境 console.warn + 渲染 "[navModules.{id}.label]" 路径
//      生产环境静默 fallback module.label 英文源
// =====================================================================

import { useTranslation } from "./useTranslation";
import type { ModuleDefinition } from "@/lib/nav/registry";

export interface ModuleTranslation {
  label: string;
  description: string;
  categoryLabel: string;
}

const EMPTY: ModuleTranslation = {
  label: "",
  description: "",
  categoryLabel: "",
};

export function useModuleTranslation(
  module: Pick<ModuleDefinition, "id" | "label" | "description" | "categoryLabel"> | null | undefined
): ModuleTranslation {
  const { t } = useTranslation();
  if (!module) return EMPTY;
  // v0.6 优先: navModules 命名空间
  const v6 = t.navModules?.[module.id];
  if (v6) {
    return {
      label: v6.label,
      description: v6.description || module.description,
      categoryLabel: v6.categoryLabel || module.categoryLabel,
    };
  }
  // v0.2 兼容: modules 命名空间 (modules.inbox.label = 收件箱)
  const v2 = t.modules?.[module.id];
  if (v2) {
    return {
      label: v2.label,
      description: v2.description || module.description,
      categoryLabel: v2.categoryLabel || module.categoryLabel,
    };
  }
  // 缺标兜底: 开发 warn, 生产静默
  if (process.env.NODE_ENV !== "production") {
    // 避免 SSR 期间噪声 — 只在 client 端 warn
    if (typeof window !== "undefined") {
      console.warn(
        `[i18n] missing navModules.${module.id} (and modules.${module.id}), falling back to registry.label`
      );
    }
  }
  return {
    label: module.label,
    description: module.description,
    categoryLabel: module.categoryLabel,
  };
}
