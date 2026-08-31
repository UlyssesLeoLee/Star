// frontend/src/mocks/handlers/billing.ts
// economy 域 MSW handlers (per test-design.md v0.2 §2.1.2 + 5 域映射表)
//
// Endpoints:
//   GET /api/billing?tenant_id=...&period=YYYY-MM  — 按 tenant + 月份过滤
//   GET /api/billing/usage?tenant_id=...           — 当期用量 (与 analytics/kpi 类似, 但 billing schema)
//
// 5 域映射: economy (billing/pricing/cost) — 本 handler 覆盖 billing 子域
// 已知缺口 (per 守门 #1 缺标比错标安全):
//   1. 真实 cost / token 数据 P2 (Phase F+)
//   2. usage 历史曲线 (跨月) P2 (Phase F+)
//   3. real-mode 短路 P3 (P3-A.7 §3 缺口 #1)

import { http, HttpResponse } from "msw";
import { MOCK_BILLING } from "@/mocks/data/five-domain";

function getMonthFromPeriod(period: string): string {
  // period = "YYYY-MM"  →  "YYYY-MM"
  return period;
}

export const billingHandlers = [
  // 列出 / 按 tenant + period 过滤
  http.get("/api/billing", ({ request }) => {
    const url = new URL(request.url);
    const tenantId = url.searchParams.get("tenant_id");
    const period = url.searchParams.get("period");

    let rows = [...MOCK_BILLING];
    if (tenantId) {
      rows = rows.filter((r) => r.tenant_id === tenantId);
    }
    if (period) {
      const month = getMonthFromPeriod(period);
      rows = rows.filter((r) => r.period_start.startsWith(month));
    }
    return HttpResponse.json(rows);
  }),

  // 用量聚合 (单 tenant 当期): token 用量 + cost_usd
  http.get("/api/billing/usage", ({ request }) => {
    const url = new URL(request.url);
    const tenantId = url.searchParams.get("tenant_id") ?? "t-acme";

    // mock 简化: 取该 tenant 最新一条, 求和 line_items 作为 cost 代理
    const rows = MOCK_BILLING.filter((r) => r.tenant_id === tenantId);
    if (rows.length === 0) {
      return HttpResponse.json({
        tenant_id: tenantId,
        tokens_used: 0,
        cost_usd: 0,
        period: new Date().toISOString().slice(0, 7),
      });
    }
    const latest = [...rows].sort((a, b) => b.period_start.localeCompare(a.period_start))[0];
    const tokensUsed = Math.round(latest.amount_cents * 1240); // 1 cent ≈ 12.4 tokens (mock 比例)
    return HttpResponse.json({
      tenant_id: tenantId,
      tokens_used: tokensUsed,
      cost_usd: latest.amount_cents / 100,
      period: latest.period_start.slice(0, 7),
    });
  }),
];
