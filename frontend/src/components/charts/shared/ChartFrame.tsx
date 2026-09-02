// frontend/src/components/charts/shared/ChartFrame.tsx
'use client';

/**
 * ChartFrame — 图表通用外壳 (per docs/design/charts/c01-burndown.md §4 通用部分)
 * - 标题 + 描述
 * - 订阅 / 导出 / 分享 按钮组
 * - Filter 选择 (S5 图表用)
 * - 错误 / 空状态
 */

import { ReactNode } from 'react';

export interface ChartFrameProps {
  title: string;
  description?: string;
  chartId: string;           // "C01_BURNDOWN"
  children: ReactNode;
  isLoading?: boolean;
  error?: string | null;
  onExport?: (format: 'csv' | 'xlsx' | 'png' | 'pdf') => void;
  onSubscribe?: () => void;
  filterSelector?: ReactNode;
}

export function ChartFrame({
  title, description, chartId, children, isLoading, error, onExport, onSubscribe, filterSelector,
}: ChartFrameProps) {
  return (
    <div
      className="rounded-lg border border-zinc-200 bg-white p-4 shadow-sm dark:border-zinc-800 dark:bg-zinc-900"
      role="img"
      aria-label={title}
      data-testid={`chart-frame-${chartId.toLowerCase()}`}
    >
      <div className="mb-3 flex items-start justify-between">
        <div>
          <h3 className="text-lg font-semibold text-zinc-900 dark:text-zinc-50">{title}</h3>
          {description && <p className="text-sm text-zinc-500 dark:text-zinc-400">{description}</p>}
        </div>
        <div className="flex gap-2">
          {filterSelector}
          {onSubscribe && (
            <button
              onClick={onSubscribe}
              className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
              aria-label={`Subscribe to ${title}`}
            >
              订阅
            </button>
          )}
          {onExport && (
            <>
              <button
                onClick={() => onExport('csv')}
                className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
                aria-label="Export as CSV"
              >
                CSV
              </button>
              <button
                onClick={() => onExport('png')}
                className="rounded border border-zinc-200 px-3 py-1 text-sm hover:bg-zinc-50 dark:border-zinc-700 dark:hover:bg-zinc-800"
                aria-label="Export as PNG"
              >
                PNG
              </button>
            </>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex h-64 items-center justify-center text-zinc-500">Loading...</div>
      )}
      {error && (
        <div className="flex h-64 items-center justify-center text-red-500" role="alert">
          ⚠ {error}
        </div>
      )}
      {!isLoading && !error && children}
    </div>
  );
}
