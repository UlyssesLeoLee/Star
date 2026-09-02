// frontend/src/components/charts/Chart05Cfd.tsx
'use client';

/**
 * C05 CFD Chart — Recharts AreaChart + stackId (per docs/design/charts/c05-cfd.md v1.0)
 */

import {
  AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ResponsiveContainer,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface DayCount { day: string; counts: Record<string, number> }
export interface CfdData {
  status_categories: string[];
  series: DayCount[];
  total: number;
}

const CATEGORY_COLORS: Record<string, string> = {
  todo: '#94a3b8',
  in_progress: '#3b82f6',
  in_review: '#a855f7',
  done: '#10b981',
};

export function Chart05Cfd({ data, height = 400 }: { data: CfdData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c05-cfd">
      <AreaChart data={data.series} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="day" />
        <YAxis label={{ value: t('chart.c05.y_axis'), angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        {data.status_categories.map(cat => (
          <Area
            key={cat}
            type="monotone"
            dataKey={`counts.${cat}`}
            stackId="1"
            name={t(`chart.c05.category.${cat}`)}
            fill={CATEGORY_COLORS[cat] || '#94a3b8'}
            stroke={CATEGORY_COLORS[cat] || '#94a3b8'}
          />
        ))}
      </AreaChart>
    </ResponsiveContainer>
  );
}
