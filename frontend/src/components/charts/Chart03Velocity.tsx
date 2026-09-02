// frontend/src/components/charts/Chart03Velocity.tsx
'use client';

/**
 * C03 Velocity Chart — Recharts BarChart + ReferenceLine (per docs/design/charts/c03-velocity.md v1.0)
 */

import {
  BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ReferenceLine, ResponsiveContainer, LabelList,
} from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Sprint { name: string; committed_sp: number; completed_sp: number | null }
export interface VelocityData {
  sprints: Sprint[];
  average_completed_sp: number;
  trend: 'increasing' | 'decreasing' | 'stable';
}

export function Chart03Velocity({ data, height = 400 }: { data: VelocityData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c03-velocity">
      <BarChart data={data.sprints} barCategoryGap="20%">
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="name" />
        <YAxis label={{ value: 'SP', angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="committed_sp" name={t('chart.c03.series.committed')} fill="#3b82f6" fillOpacity={0.5} />
        <Bar dataKey="completed_sp" name={t('chart.c03.series.completed')} fill="#10b981">
          <LabelList dataKey="completed_sp" position="top" />
        </Bar>
        <ReferenceLine
          y={data.average_completed_sp}
          stroke="#94a3b8"
          strokeDasharray="5 5"
          label={{ value: t('chart.c03.average', { n: data.average_completed_sp.toFixed(1) }), position: 'right' }}
        />
      </BarChart>
    </ResponsiveContainer>
  );
}
