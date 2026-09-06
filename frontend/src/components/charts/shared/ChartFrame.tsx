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
      className="border-2 border-line bg-bg-card p-4 cel-shadow"
      role="img"
      aria-label={title}
      data-testid={`chart-frame-${chartId.toLowerCase()}`}
    >
      <div className="mb-3 flex items-start justify-between">
        <div>
          <h3 className="text-lg font-semibold text-ink">{title}</h3>
          {description && <p className="text-sm text-ink-dim">{description}</p>}
        </div>
        <div className="flex gap-2">
          {filterSelector}
          {onSubscribe && (
            <button
              onClick={onSubscribe}
              className="border-2 border-[var(--cel-ink)] bg-bg-soft px-3 py-1 text-sm font-semibold hover:bg-bg-card transition-colors cel-btn-3d shadow-[2px_2px_0px_0px_var(--cel-ink)]"
              aria-label={`Subscribe to ${title}`}
            >
              订阅
            </button>
          )}
          {onExport && (
            <>
              <button
                onClick={() => onExport('csv')}
                className="border-2 border-[var(--cel-ink)] bg-bg-soft px-3 py-1 text-sm font-semibold hover:bg-bg-card transition-colors cel-btn-3d shadow-[2px_2px_0px_0px_var(--cel-ink)]"
                aria-label="Export as CSV"
              >
                CSV
              </button>
              <button
                onClick={() => onExport('png')}
                className="border-2 border-[var(--cel-ink)] bg-bg-soft px-3 py-1 text-sm font-semibold hover:bg-bg-card transition-colors cel-btn-3d shadow-[2px_2px_0px_0px_var(--cel-ink)]"
                aria-label="Export as PNG"
              >
                PNG
              </button>
            </>
          )}
        </div>
      </div>

      {isLoading && (
        <div className="flex h-64 items-center justify-center text-ink-mute">Loading...</div>
      )}
      {error && (
        <div className="flex h-64 items-center justify-center text-err" role="alert">
          ⚠ {error}
        </div>
      )}
      {!isLoading && !error && children}
    </div>
  );
}
