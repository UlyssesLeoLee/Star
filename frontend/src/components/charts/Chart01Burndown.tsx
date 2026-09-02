// frontend/src/components/charts/Chart01Burndown.tsx
'use client';

/**
 * C01 Burndown Chart — Recharts 实现 (per docs/design/charts/c01-burndown.md v1.0)
 *
 * 关键轴:
 *   - X: Sprint 日期 (start → end)
 *   - Y: 剩余 SP (含 scope change 阶梯)
 *
 * Series:
 *   - ideal: 虚线, 线性下降
 *   - actual: 实线, 实际完成反向
 *   - scope change: 垂直虚线, 标 ±SP
 */

import { useMemo } from 'react';
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend,
  ReferenceLine, ResponsiveContainer, LabelList,
} from 'recharts';
import type { BurndownData } from '@/lib/chart-data-schema';
import { useTranslation } from '@/i18n/hooks';

export interface Chart01BurndownProps {
  data: BurndownData;
  height?: number;
  showIdealLine?: boolean;
  showScopeChanges?: boolean;
  onPointClick?: (date: string) => void;
}

interface MergedPoint {
  x: string;
  ideal: number | null;
  actual: number | null;
  scope_change_at?: number;  // SP 变化
}

function mergeSeries(data: BurndownData): MergedPoint[] {
  const allDays = new Set<string>();
  data.series.ideal.forEach(p => allDays.add(p.x));
  data.series.actual.forEach(p => allDays.add(p.x));
  return Array.from(allDays).sort().map(x => ({
    x,
    ideal: data.series.ideal.find(p => p.x === x)?.y ?? null,
    actual: data.series.actual.find(p => p.x === x)?.y ?? null,
  }));
}

export function Chart01Burndown({
  data, height = 400, showIdealLine = true, showScopeChanges = true, onPointClick,
}: Chart01BurndownProps) {
  const { t } = useTranslation();
  const merged = useMemo(() => mergeSeries(data), [data]);

  const idealColor = '#94a3b8';
  const actualColor = '#3b82f6';
  const scopeColor = '#f59e0b';
  const endColor = '#ef4444';

  return (
    <ResponsiveContainer width="100%" height={height} data-testid="chart-c01-burndown">
      <LineChart data={merged} margin={{ top: 20, right: 30, left: 20, bottom: 20 }}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="x"
          tick={{ fontSize: 12 }}
          label={{ value: t('chart.c01.x_axis'), position: 'insideBottom', offset: -10, fontSize: 12 }}
        />
        <YAxis
          label={{ value: t('chart.c01.y_axis.sp'), angle: -90, position: 'insideLeft', fontSize: 12 }}
          domain={[0, (dataMax: number) => Math.ceil(dataMax * 1.1)]}
        />
        <Tooltip content={<BurndownTooltip summary={data.summary} />} />
        <Legend wrapperStyle={{ fontSize: 12 }} />
        {showIdealLine && (
          <Line
            type="monotone"
            dataKey="ideal"
            name={t('chart.c01.series.ideal')}
            stroke={idealColor}
            strokeDasharray="5 5"
            strokeWidth={1.5}
            dot={false}
            isAnimationActive={false}
            connectNulls
          />
        )}
        <Line
          type="monotone"
          dataKey="actual"
          name={t('chart.c01.series.actual')}
          stroke={actualColor}
          strokeWidth={2}
          dot={{ r: 4, fill: actualColor }}
          activeDot={{
            r: 6,
            onClick: (e: any) => onPointClick?.(e.payload?.x),
          }}
          isAnimationActive={false}
          connectNulls
        >
          <LabelList
            dataKey="actual"
            position="top"
            formatter={(v: any) => (typeof v === 'number' && v === data.summary.remaining_sp ? `${v.toFixed(0)}` : '')}
            style={{ fontSize: 11, fill: actualColor }}
          />
        </Line>
        {showScopeChanges && data.scope_changes.map((sc, i) => (
          <ReferenceLine
            key={`sc-${i}`}
            x={sc.at.split('T')[0]}
            stroke={scopeColor}
            strokeDasharray="3 3"
            label={{
              value: t('chart.c01.scope_change', { n: sc.delta_sp }),
              position: 'top',
              fill: scopeColor,
              fontSize: 11,
            }}
          />
        ))}
        <ReferenceLine
          x={data.sprint.end_date.split('T')[0]}
          stroke={endColor}
          strokeDasharray="2 2"
          label={{ value: t('chart.c01.sprint_end'), position: 'top', fill: endColor, fontSize: 11 }}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}

interface TooltipProps {
  active?: boolean;
  payload?: Array<{ payload: MergedPoint }>;
  summary: BurndownData['summary'];
}

function BurndownTooltip({ active, payload, summary }: TooltipProps) {
  const { t } = useTranslation();
  if (!active || !payload?.length) return null;
  const p = payload[0].payload;
  return (
    <div
      className="rounded border border-zinc-200 bg-white p-2 text-sm shadow dark:border-zinc-700 dark:bg-zinc-900"
      role="tooltip"
    >
      <div className="font-semibold">{p.x}</div>
      {p.ideal !== null && (
        <div className="text-zinc-500">
          <span className="font-medium">{t('chart.c01.tooltip.ideal')}: </span>
          {p.ideal.toFixed(0)} SP
        </div>
      )}
      {p.actual !== null && (
        <div className="text-blue-500">
          <span className="font-medium">{t('chart.c01.tooltip.actual')}: </span>
          {p.actual.toFixed(0)} SP
        </div>
      )}
      <div className="mt-1 border-t border-zinc-200 pt-1 text-xs dark:border-zinc-700">
        {summary.on_track ? (
          <span className="text-emerald-500">✓ {t('chart.c01.summary.on_track')}</span>
        ) : (
          <span className="text-amber-500">⚠ {t('chart.c01.summary.off_track')}</span>
        )}
      </div>
    </div>
  );
}
