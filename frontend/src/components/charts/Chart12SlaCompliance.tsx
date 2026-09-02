// frontend/src/components/charts/Chart12SlaCompliance.tsx
'use client';

/** C12 SLA Compliance (per docs/design/charts/c12-sla-compliance.md) */
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine, ResponsiveContainer } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface DayCompliance { day: string; priorities: Record<string, { met: number; total: number; compliance: number }> }
export interface SlaData {
  series: DayCompliance[];
  summary: { overall_compliance: number; by_priority: Record<string, number>; breaches: number };
  target_line: number;
}

const COLORS: Record<string, string> = { high: '#ef4444', medium: '#f59e0b', low: '#3b82f6' };

export function Chart12SlaCompliance({ data, height = 400 }: { data: SlaData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c12-sla-compliance">
      <LineChart data={data.series}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="day" />
        <YAxis domain={[0, 1]} tickFormatter={v => `${(v * 100).toFixed(0)}%`} label={{ value: 'SLA %', angle: -90, position: 'insideLeft' }} />
        <Tooltip />
        <Legend />
        {Object.keys(COLORS).map(p => (
          <Line key={p} type="monotone" dataKey={`priorities.${p}.compliance`} name={p} stroke={COLORS[p]} strokeWidth={2} />
        ))}
        <ReferenceLine y={data.target_line} stroke="#10b981" strokeDasharray="5 5" label={{ value: `Target ${(data.target_line * 100).toFixed(0)}%`, position: 'right' }} />
      </LineChart>
    </ResponsiveContainer>
  );
}
