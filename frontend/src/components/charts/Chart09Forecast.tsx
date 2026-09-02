// frontend/src/components/charts/Chart09Forecast.tsx
'use client';

/** C09 Forecast (per docs/design/charts/c09-forecast.md) */
import { ComposedChart, Bar, Line, Area, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, ReferenceLine } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Sprint { name: string; completed_sp: number; predicted?: number; upper_95?: number; lower_95?: number }
export interface ForecastData {
  historical: { sprints: Array<{ name: string; completed_sp: number }>; avg_velocity: number };
  forecast: { method: string; predicted_velocity: number; predicted_completion_date: string };
}

export function Chart09Forecast({ data, height = 400 }: { data: ForecastData; height?: number }) {
  const { t } = useTranslation();
  const merged: Sprint[] = data.historical.sprints.map(s => ({ ...s, predicted: null as any }));
  merged.push({ name: '预测', completed_sp: data.forecast.predicted_velocity, predicted: data.forecast.predicted_velocity });
  const avg = data.historical.avg_velocity;
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c09-forecast">
      <ComposedChart data={merged}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="name" />
        <YAxis label={{ value: 'SP', angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        <Bar dataKey="completed_sp" name={t('chart.c09.series.historical')} fill="#3b82f6" />
        <Line type="monotone" dataKey="predicted" name={t('chart.c09.series.forecast')} stroke="#f59e0b" strokeDasharray="5 5" />
        <ReferenceLine y={avg} stroke="#94a3b8" strokeDasharray="3 3" label={{ value: `Avg ${avg.toFixed(1)}`, position: 'right' }} />
      </ComposedChart>
    </ResponsiveContainer>
  );
}
