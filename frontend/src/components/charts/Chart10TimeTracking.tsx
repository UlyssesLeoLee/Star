// frontend/src/components/charts/Chart10TimeTracking.tsx
'use client';

/** C10 Time Tracking (per docs/design/charts/c10-time-tracking.md) */
import { useTranslation } from '@/i18n/hooks';

interface Row { id: string; name: string; original_seconds: number; spent_seconds: number; remaining_seconds: number; progress: number }
export interface TimeTrackingData {
  granularity: 'user' | 'project' | 'issue';
  rows: Row[];
  summary: { total_original: number; total_spent: number; total_remaining: number };
}

function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${minutes}m`;
}

export function Chart10TimeTracking({ data }: { data: TimeTrackingData }) {
  const { t } = useTranslation();
  return (
    <div data-testid="chart-c10-time-tracking" className="rounded border border-zinc-200 p-4 dark:border-zinc-800">
      <h3 className="mb-3 text-lg font-semibold">{t('chart.c10.title')}</h3>
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-zinc-200 dark:border-zinc-700">
            <th className="p-1 text-left">{t('chart.c10.column.name')}</th>
            <th className="p-1 text-right">{t('chart.c10.column.original')}</th>
            <th className="p-1 text-right">{t('chart.c10.column.spent')}</th>
            <th className="p-1 text-right">{t('chart.c10.column.remaining')}</th>
            <th className="p-1 text-right">{t('chart.c10.column.progress')}</th>
          </tr>
        </thead>
        <tbody>
          {data.rows.map(r => (
            <tr key={r.id} className="border-b border-zinc-100 dark:border-zinc-800">
              <td className="p-1">{r.name}</td>
              <td className="p-1 text-right">{formatDuration(r.original_seconds)}</td>
              <td className="p-1 text-right">{formatDuration(r.spent_seconds)}</td>
              <td className="p-1 text-right">{formatDuration(r.remaining_seconds)}</td>
              <td className="p-1 text-right">
                <div className="ml-auto h-2 w-24 rounded bg-zinc-200 dark:bg-zinc-800">
                  <div className="h-full rounded bg-blue-500" style={{ width: `${r.progress * 100}%` }} />
                </div>
                <span className="text-xs">{(r.progress * 100).toFixed(0)}%</span>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
