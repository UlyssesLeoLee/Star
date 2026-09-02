// frontend/src/components/charts/Chart04SprintReport.tsx
'use client';

/**
 * C04 Sprint Report — Table + Summary (per docs/design/charts/c04-sprint-report.md v1.0)
 */

import { useTranslation } from '@/i18n/hooks';

interface IssueRow {
  key: string;
  title: string;
  issue_type: string;
  priority: string;
  completed_at?: string;
  story_points?: number;
}
export interface SprintReportData {
  sprint: { sprint_id: string; name: string };
  groups: { completed: IssueRow[]; carry_over: IssueRow[]; incomplete: IssueRow[] };
  summary: {
    completed_count: number;
    carry_over_count: number;
    incomplete_count: number;
    completed_sp: number;
  };
}

export function Chart04SprintReport({ data }: { data: SprintReportData }) {
  const { t } = useTranslation();
  const { groups, summary } = data;
  return (
    <div data-testid="chart-c04-sprint-report" className="rounded border border-zinc-200 p-4 dark:border-zinc-800">
      <h3 className="text-lg font-semibold">{data.sprint.name}</h3>
      <div className="mt-2 grid grid-cols-4 gap-2 text-sm">
        <SummaryCard label={t('chart.c04.summary.completed')} value={summary.completed_count} tone="ok" />
        <SummaryCard label={t('chart.c04.summary.carry_over')} value={summary.carry_over_count} tone="warn" />
        <SummaryCard label={t('chart.c04.summary.incomplete')} value={summary.incomplete_count} tone="err" />
        <SummaryCard label={t('chart.c04.summary.completed_sp')} value={summary.completed_sp} tone="info" suffix="SP" />
      </div>
      <div className="mt-4 space-y-2">
        <Group title={`完成 (${groups.completed.length})`} rows={groups.completed} t={t} />
        <Group title={`延期 (${groups.carry_over.length})`} rows={groups.carry_over} t={t} />
        <Group title={`未完成 (${groups.incomplete.length})`} rows={groups.incomplete} t={t} />
      </div>
    </div>
  );
}

function SummaryCard({ label, value, tone, suffix }: { label: string; value: number; tone: string; suffix?: string }) {
  const colors: Record<string, string> = { ok: 'text-emerald-500', warn: 'text-amber-500', err: 'text-red-500', info: 'text-blue-500' };
  return (
    <div className="rounded bg-zinc-50 p-2 text-center dark:bg-zinc-900">
      <div className="text-xs text-zinc-500">{label}</div>
      <div className={`text-2xl font-bold ${colors[tone] || ''}`}>{value}{suffix || ''}</div>
    </div>
  );
}

function Group({ title, rows, t }: { title: string; rows: IssueRow[]; t: (k: string) => string }) {
  return (
    <details className="rounded border border-zinc-200 dark:border-zinc-800">
      <summary className="cursor-pointer p-2 font-medium">{title}</summary>
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-zinc-200 dark:border-zinc-700">
            <th className="p-1 text-left">{t('chart.c04.column.key')}</th>
            <th className="p-1 text-left">{t('chart.c04.column.title')}</th>
            <th className="p-1">{t('chart.c04.column.sp')}</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(r => (
            <tr key={r.key} className="border-b border-zinc-100 dark:border-zinc-800">
              <td className="p-1 text-blue-500">{r.key}</td>
              <td className="p-1">{r.title}</td>
              <td className="p-1 text-center">{r.story_points ?? '-'}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </details>
  );
}
