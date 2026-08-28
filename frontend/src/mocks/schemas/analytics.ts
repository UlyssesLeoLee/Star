// frontend/src/mocks/schemas/analytics.ts
// zod schema for analytics KPI + cost series (per mock-data-isolation.md §2.1)

import { z } from "zod";

export const KpiToneSchema = z.enum(["warn", "info", "ok", "err"]);
export type KpiTone = z.infer<typeof KpiToneSchema>;

export const KpiCardSchema = z.object({
  label: z.string().min(1),
  value: z.union([z.string(), z.number()]),
  hint: z.string(),
  tone: KpiToneSchema,
});
export type KpiCard = z.infer<typeof KpiCardSchema>;

export const CostPointSchema = z.object({
  day: z.string().min(1),
  usd: z.number().nonnegative(),
});
export type CostPoint = z.infer<typeof CostPointSchema>;
