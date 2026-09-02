// frontend/src/lib/chart-data-schema.ts
// 22 图表共用 TS schema (per docs/basic-design/charts-and-reports.md §5.3)

export type ChartData =
  | BurndownData
  | BurnupData
  | VelocityData
  | SprintReportData
  | CfdData
  | ControlChartData
  | CycleTimeData
  | CvrData
  | ThroughputData
  | ForecastData
  | TimeTrackingData
  | ResolutionTimeData
  | SlaData
  | IssueTypeDistData
  | PriorityDistData
  | { stub: true; chart_id: string };

/** C01 Burndown 完整 schema (与 crates/domain-report/src/domain/c01_burndown.rs::BurndownData 同构) */
export interface BurndownData {
  sprint: {
    sprint_id: string;
    name: string;
    start_date: string;
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

/** C02 Burnup 完整 schema */
export interface BurnupData {
  sprint: {
    sprint_id: string;
    name: string;
    start_date: string;
    end_date: string;
    total_sp: number;
    scope_change_log: ScopeChange[];
  };
  series: {
    actual: TimeSeriesPoint[];
    scope: TimeSeriesPoint[];
  };
  scope_changes: ScopeChange[];
  summary: {
    completed_sp: number;
    total_sp: number;
    completion_ratio: number;
  };
}

/** C03 Velocity */
export interface VelocityData {
  sprints: Array<{
    sprint_id: string;
    name: string;
    committed_sp: number;
    completed_sp: number | null;
  }>;
  average_completed_sp: number;
  trend: 'increasing' | 'decreasing' | 'stable';
}

/** C04 Sprint Report */
export interface SprintReportData {
  sprint: { sprint_id: string; name: string };
  groups: {
    completed: IssueRow[];
    carry_over: IssueRow[];
    incomplete: IssueRow[];
  };
  summary: {
    completed_count: number;
    carry_over_count: number;
    incomplete_count: number;
    completed_sp: number;
  };
}

export interface IssueRow {
  key: string;
  title: string;
  issue_type: string;
  priority: string;
  assignee?: { name: string; avatar_url: string };
  completed_at?: string;
  story_points?: number;
}

/** C05 CFD */
export interface CfdData {
  date_range: { start: string; end: string };
  status_categories: string[];
  series: Array<{ day: string; counts: Record<string, number> }>;
  total: number;
}

/** C06 Control Chart */
export interface ControlChartData {
  data_points: Array<{
    workitem_id: string;
    key: string;
    cycle_time_days: number;
    completed_at: string;
    anomaly: boolean;
    z_score: number;
  }>;
  reference_lines: Array<{ y_value: number; label: string; style: string }>;
  stats: { median: number; p70: number; p85: number; p95: number; mean: number; std_dev: number };
}

/** C07 Cycle Time */
export interface CycleTimeData {
  buckets: Array<{ range_start: number; range_end: number; count: number; label: string }>;
  percentiles: { p50: number; p85: number; p95: number };
  stats: { total_count: number; median: number; mean: number };
  bucket_size: number;
}

/** C13 Created vs Resolved */
export interface CvrData {
  series: Array<{ day: string; created: number; resolved: number }>;
  summary: {
    total_created: number;
    total_resolved: number;
    net_change: number;
    backlog_trend: 'growing' | 'shrinking' | 'stable';
  };
  time_granularity: 'day' | 'week' | 'month';
}

/** C08 Throughput */
export interface ThroughputData {
  granularity: 'day' | 'week' | 'month';
  series: Array<{ bucket: string; count: number }>;
  moving_avg: Array<{ bucket: string; avg: number }>;
  stats: { total: number; avg: number; std_dev: number };
}

/** C09 Forecast */
export interface ForecastData {
  historical: {
    sprints: Array<{ name: string; completed_sp: number }>;
    avg_velocity: number;
  };
  forecast: {
    method: 'simple_avg' | 'rolling_avg' | 'linear_regression';
    predicted_velocity: number;
    confidence_80: [number, number];
    confidence_95: [number, number];
    predicted_completion_date: string;
  };
}

/** C10 Time Tracking */
export interface TimeTrackingData {
  granularity: 'user' | 'project' | 'issue';
  rows: Array<{
    id: string;
    name: string;
    original_seconds: number;
    spent_seconds: number;
    remaining_seconds: number;
    progress: number;
  }>;
  summary: { total_original: number; total_spent: number; total_remaining: number };
}

/** C11 Resolution Time */
export interface ResolutionTimeData {
  group_by: 'priority' | 'type' | 'assignee';
  rows: Array<{ group: string; avg_days: number; median_days: number; count: number }>;
}

/** C12 SLA Compliance */
export interface SlaData {
  series: Array<{ day: string; priorities: Record<string, { met: number; total: number; compliance: number }> }>;
  summary: { overall_compliance: number; by_priority: Record<string, number>; breaches: number };
  target_line: number;
}

/** C14 Issue Type Distribution */
export interface IssueTypeDistData {
  slices: Array<{ key: string; count: number; percentage: number }>;
  total: number;
  status_filter: 'all' | 'open' | 'closed';
}

/** C15 Priority Distribution */
export interface PriorityDistData {
  slices: Array<{ key: string; count: number; percentage: number }>;
  total: number;
  status_filter: 'all' | 'open' | 'closed';
}

export interface TimeSeriesPoint {
  x: string;
  y: number;
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
