// =====================================================================
// useStatusLabel — 状态/枚举翻译 (per 2026-08-31 补缺口 v2)
// =====================================================================
// 把后端技术 enum (todo / in_progress / done / active / planned / ...)
// 翻译成 3 语言用户可见标签. 找不到兜底: 美化 (下划线转空格, 不翻译)
//
// 防御: 即使调用处不在 I18nProvider 内 (e.g. 单测直接渲染 StatusPill),
// 也优雅降级到 prettify(value), 不抛 useI18n 错. 这样老测试无需改动.
//
// 用法:
//   const { label } = useStatusLabel("workItem", "in_progress");
//   // zh-CN -> "进行中", en -> "In Progress", ja -> "進行中"
//
// 也支持整批拉表 (用于 GanttLegend / StatusPill 全量枚举):
//   const all = useStatusLabelMap("workItem");   // { todo: "待办", ... }
// =====================================================================

import { useContext } from "react";
import { I18nContext } from "./I18nProvider";
import type { Dictionary } from "./dictionary";

export type StatusKind = keyof Dictionary["status"];

/** 美化 fallback: snake_case -> "Snake Case" (per StatusPill 历史行为) */
function prettify(value: string): string {
  return value.replace(/_/g, " ");
}

export function useStatusLabel(kind: StatusKind, value: string): string {
  const ctx = useContext(I18nContext);
  if (!value) return "";
  // 无 I18nProvider (单测 / 早期 mount): 兜底 prettify
  if (!ctx) return prettify(value);
  const lookup = ctx.t.status[kind] as Record<string, string> | undefined;
  const translated = lookup?.[value];
  return translated ?? prettify(value);
}

/** 拉整张表 (kind -> { enum: label }), 用于渲染整组枚举 (legend, filter dropdown) */
export function useStatusLabelMap(kind: StatusKind): Record<string, string> {
  const ctx = useContext(I18nContext);
  if (!ctx) {
    // 无 provider: 返回空对象, 消费者用 enum 原值兜底
    return new Proxy({} as Record<string, string>, {
      get: (_t, prop: string) => prettify(String(prop)),
    });
  }
  const raw = ctx.t.status[kind] as Record<string, string>;
  return new Proxy(raw, {
    get(target, prop: string) {
      if (prop in target) return target[prop];
      return prettify(prop);
    },
  });
}
