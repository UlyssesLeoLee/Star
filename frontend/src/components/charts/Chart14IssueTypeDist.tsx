// frontend/src/components/charts/Chart14IssueTypeDist.tsx
'use client';

/** C14 Issue Type Distribution (per docs/design/charts/c14-issue-type-dist.md) */
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import { useTranslation } from '@/i18n/hooks';

interface Slice { key: string; count: number; percentage: number }
export interface IssueTypeDistData { slices: Slice[]; total: number; status_filter: string }

const COLORS: Record<string, string> = { Bug: '#ef4444', Story: '#10b981', Task: '#3b82f6', Epic: '#a855f7', Subtask: '#94a3b8' };

export function Chart14IssueTypeDist({ data, height = 400 }: { data: IssueTypeDistData; height?: number }) {
  const { t } = useTranslation();
  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c14-issue-type-dist">
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
