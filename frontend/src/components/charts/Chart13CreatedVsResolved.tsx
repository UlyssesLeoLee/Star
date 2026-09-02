// frontend/src/components/charts/Chart13CreatedVsResolved.tsx
'use client';

/**
 * C13 Created vs Resolved Chart — Recharts 双线 (per docs/design/charts/c13-created-vs-resolved.md v1.0)
 */

import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ResponsiveContainer,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface DayStat { day: string; created: number; resolved: number }
export interface CvrData {
  series: DayStat[];
  summary: { total_created: number; total_resolved: number; net_change: number; backlog_trend: string };
}

export function Chart13CreatedVsResolved({ data, height = 400 }: { data: CvrData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c13-created-vs-resolved">
      <LineChart data={data.series} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="day" />
        <YAxis label={{ value: t('chart.c13.y_axis'), angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Line type="monotone" dataKey="created" name={t('chart.c13.series.created')} stroke="#3b82f6" strokeWidth={2} dot={{ r: 3 }} />
        <Line type="monotone" dataKey="resolved" name={t('chart.c13.series.resolved')} stroke="#10b981" strokeWidth={2} dot={{ r: 3 }} />
      </LineChart>
    </ResponsiveContainer>
  );
}
