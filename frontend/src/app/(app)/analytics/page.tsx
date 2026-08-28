"use client";

// =====================================================================
// /analytics — Dashboard KPI + 折线图 (minimal placeholder)
// =====================================================================
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. KPI 数值 + 折线图全部 mock — 真实 cost/token 接入 P2
//   2. 折线图为内联 SVG 占位, 不依赖 recharts (新增依赖禁)
//   3. error mix donut / leaderboard 表格 P2 (Phase I+)
//   4. light mode (per §7) P3
//   5. useEffect+fetch 阶段用 MOCK_KPI_FALLBACK / COST_SERIES_FALLBACK SSR 兜底 (per mock-msw-handlers §2.4 + §4 #1 缺标)
// =====================================================================

import { useEffect, useState } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { BarChart3, TrendingUp } from "lucide-react";
import { MOCK_KPI_FALLBACK, COST_SERIES_FALLBACK } from "@/mocks/data";
import type { KpiCard, CostPoint } from "@/mocks/schemas/analytics";

function MiniLineChart({ data }: { data: ReadonlyArray<CostPoint> }) {
  const W = 320;
  const H = 96;
  const PAD = 8;
  const max = Math.max(...data.map((d) => d.usd), 1);
  const stepX = (W - PAD * 2) / Math.max(data.length - 1, 1);
  const points = data.map((d, i) => {
    const x = PAD + i * stepX;
    const y = H - PAD - (d.usd / max) * (H - PAD * 2);
    return `${x},${y}`;
  });
  const linePath = `M ${points.join(" L ")}`;
  return (
    <svg
      data-testid="cost-trend-chart"
      viewBox={`0 0 ${W} ${H}`}
      className="w-full h-24"
      role="img"
      aria-label="Cost trend (mock)"
    >
      <path
        d={`${linePath} L ${PAD + (data.length - 1) * stepX},${H - PAD} L ${PAD},${H - PAD} Z`}
        fill="currentColor"
        className="text-accent/15"
      />
      <path
        d={linePath}
        fill="none"
        stroke="currentColor"
        strokeWidth={1.5}
        className="text-accent"
      />
      {data.map((d, i) => {
        const x = PAD + i * stepX;
        const y = H - PAD - (d.usd / max) * (H - PAD * 2);
        return <circle key={d.day} cx={x} cy={y} r={2} className="fill-accent" />;
      })}
    </svg>
  );
}

export default function AnalyticsPage() {
  const [kpi, setKpi] = useState<ReadonlyArray<KpiCard>>(MOCK_KPI_FALLBACK);
  const [costSeries, setCostSeries] = useState<ReadonlyArray<CostPoint>>(COST_SERIES_FALLBACK);

  useEffect(() => {
    fetch("/api/analytics/kpi")
      .then((r) => r.json())
      .then((data: ReadonlyArray<KpiCard>) => setKpi(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  useEffect(() => {
    fetch("/api/analytics/cost")
      .then((r) => r.json())
      .then((data: ReadonlyArray<CostPoint>) => setCostSeries(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标) */
      });
  }, []);

  return (
    <div className="max-w-7xl mx-auto" data-testid="analytics-page">
      <PageHeader
        title="Analytics"
        subtitle="dashboard / metric / cost / burndown (5 维 mock; 真实数据 P2 缺口)"
        icon={<BarChart3 className="text-accent" size={20} />}
        count="4 KPIs"
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        {kpi.map((k) => (
          <Stat key={k.label} label={k.label} value={k.value} hint={k.hint} tone={k.tone} />
        ))}
      </div>

      <div className="card mb-3" data-testid="cost-trend-card">
        <div className="flex items-center justify-between mb-2">
          <SectionTitle>Daily Cost Trend (7d, mock)</SectionTitle>
          <TrendingUp size={12} className="text-ok" />
        </div>
        <MiniLineChart data={costSeries} />
        <div className="flex justify-between mt-1 font-mono text-[10px] text-ink-mute">
          {costSeries.map((d) => (
            <span key={d.day}>{d.day}</span>
          ))}
        </div>
      </div>

      <div className="card text-xs text-ink-dim">
        <SectionTitle>Real Data Source — P2/P3 缺口</SectionTitle>
        <ul className="space-y-1.5 list-disc pl-4 text-ink-dim">
          <li>cost 真实数据: <span className="font-mono text-ink-mute">/api/analytics/cost</span> 待接入</li>
          <li>token 真实数据: <span className="font-mono text-ink-mute">/api/analytics/tokens</span> 待接入</li>
          <li>error mix donut + leaderboard table: P2 (Phase I+)</li>
          <li>当前页面所有数据为 mock, 不可用于决策</li>
        </ul>
      </div>
    </div>
  );
}
