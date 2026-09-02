// frontend/src/lib/chart-data-schema.ts
// 22 图表共用 TS schema (per docs/basic-design/charts-and-reports.md §5.3)

export type ChartData =
  | BurndownData
  | { stub: true; chart_id: string };

/** C01 Burndown 完整 schema (与 crates/domain-report/src/domain/c01_burndown.rs::BurndownData 同构) */
export interface BurndownData {
  sprint: {
    sprint_id: string;
    name: string;
    start_date: string;   // ISO 8601
    end_date: string;
    total_sp: number;
    scope_change_log: ScopeChange[];
  };
  series: {
    ideal: TimeSeriesPoint[];
    actual: TimeSeriesPoint[];
  };
  scope_changes: ScopeChange[];
  summary: BurndownSummary;
}

export interface TimeSeriesPoint {
  x: string;   // ISO date "2026-09-02"
  y: number;   // 剩余 SP
}

export interface ScopeChange {
  at: string;
  delta_sp: number;
  reason: string;
  new_total_sp: number;
}

export interface BurndownSummary {
  remaining_sp: number;
  completed_sp: number;
  completed_issues: number;
  total_issues: number;
  predicted_completion_sp: number;
  on_track: boolean;
}

/** Report API 响应 (per docs/basic-design §5.2) */
export interface ReportResponse {
  report_id: string;
  chart_type: string;
  generated_at: string;
  ttl_seconds: number;
  data: ChartData;
  render_hints: {
    total_data_points: number;
    chart_height: number;
    show_legend: boolean;
  };
  data_source_refs: Array<{
    source_type: 'work_item' | 'sprint' | 'version';
    source_ids: string[];
  }>;
}
