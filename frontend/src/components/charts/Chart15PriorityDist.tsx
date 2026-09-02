// frontend/src/components/charts/Chart15PriorityDist.tsx
'use client';

/** C15 Priority Distribution (per docs/design/charts/c15-priority-dist.md) */
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Slice { key: string; count: number; percentage: number }
export interface PriorityDistData { slices: Slice[]; total: number; status_filter: string }

const COLORS: Record<string, string> = { highest: '#7c2d12', high: '#ef4444', medium: '#f59e0b', low: '#3b82f6', lowest: '#94a3b8' };

export function Chart15PriorityDist({ data, height = 400 }: { data: PriorityDistData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c15-priority-dist">
      <PieChart>
        <Pie data={data.slices} dataKey="count" nameKey="key" cx="50%" cy="50%" innerRadius={60} outerRadius={100} paddingAngle={2}>
          {data.slices.map((s, i) => <Cell key={i} fill={COLORS[s.key] || '#94a3b8'} />)}
        </Pie>
        <Tooltip />
        <Legend />
      </PieChart>
    </ResponsiveContainer>
  );
}
