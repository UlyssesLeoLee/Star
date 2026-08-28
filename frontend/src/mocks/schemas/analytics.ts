// frontend/src/mocks/schemas/analytics.ts
// 零 prod dep, 用 TS type guards (替代 d4b3193 的 zod).

export const KPI_TONES = ["warn", "info", "ok", "err"] as const;
export type KpiTone = (typeof KPI_TONES)[number];

export function isKpiTone(v: unknown): v is KpiTone {
  return typeof v === "string" && (KPI_TONES as readonly string[]).includes(v);
}

export interface KpiCard {
  label: string;
  value: string | number;
  hint: string;
  tone: KpiTone;
}

export function isKpiCard(v: unknown): v is KpiCard {
  if (typeof v !== "object" || v === null) return false;
  const k = v as Record<string, unknown>;
  return (
    typeof k.label === "string" &&
    k.label.length > 0 &&
    (typeof k.value === "string" || typeof k.value === "number") &&
    typeof k.hint === "string" &&
    isKpiTone(k.tone)
  );
}

export interface CostPoint {
  day: string;
  usd: number;
}

export function isCostPoint(v: unknown): v is CostPoint {
  if (typeof v !== "object" || v === null) return false;
  const p = v as Record<string, unknown>;
  return (
    typeof p.day === "string" &&
    p.day.length > 0 &&
    typeof p.usd === "number" &&
    p.usd >= 0
  );
}
