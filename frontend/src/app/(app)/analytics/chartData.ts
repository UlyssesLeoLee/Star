/*
@cypher schema=1 source_sha256=327cdd9ce0fc6da2a99c7ca3710ff3dad95d7bd35f8011ab8dbf4493db2ad175
MERGE (self:File {path:"frontend/src/app/(app)/analytics/chartData.ts"})
MERGE (mod:Symbol {id:"frontend/src/app/(app)/analytics/chartData.ts::buildAnalyticsData",kind:"function"})
MERGE (self)-[:DEFINES]->(mod)
@endcypher
*/
// =====================================================================
// 分析页图表数据 (per 2026-11-15: 燃尽图等图表合并进分析界面)
// 数据: store sprints/workItems 聚合 + 派生 fallback — 预览数据,
//   宿主 BI/数据库投影接入后替换 (诚实标注原则)
// =====================================================================
import type { Sprint, WorkItem } from "@/types/ids";
import type {
  BurndownData, BurnupData, VelocityData, SprintReportData, CfdData,
  ControlChartData, CycleTimeData, CvrData, ThroughputData, ForecastData,
  TimeTrackingData, ResolutionTimeData, SlaData, IssueTypeDistData, PriorityDistData,
  IssueRow, TimeSeriesPoint,
} from "@/lib/chart-data-schema";

const DAY_MS = 86_400_000;

function daysBetween(start: string, end: string): string[] {
  const out: string[] = [];
  const s = new Date(start).getTime();
  const e = new Date(end).getTime();
  for (let t = s; t <= e; t += DAY_MS) out.push(new Date(t).toISOString().slice(0, 10));
  return out.length ? out : ["2026-11-01", "2026-11-14"];
}

function sprintOf(sprints: Sprint[]) {
  return sprints[0] ?? {
    id: "spr-preview", name: "预览冲刺", start_date: "2026-11-01",
    end_date: "2026-11-14", goal: "", status: "active" as const, work_item_ids: [],
  };
}

function toIssueRow(w: WorkItem, assignee: string): IssueRow {
  return {
    key: w.key, title: w.title, issue_type: w.kind,
    priority: w.priority ?? "medium", assignee: { name: assignee, avatar_url: "" },
    completed_at: w.updated_at, story_points: w.story_points ?? undefined,
  };
}

function clamp(v: number, lo = 0, hi = 100) { return Math.max(lo, Math.min(hi, v)); }

export function buildAnalyticsData(sprints: Sprint[], workItems: WorkItem[]) {
  const sp = sprintOf(sprints);
  const days = daysBetween(sp.start_date, sp.end_date);
  const n = days.length;
  const totalSp = workItems.reduce((s, w) => s + (w.story_points ?? 3), 0) || 30;
  const done = workItems.filter((w) => w.status === "done");
  const doneSp = done.reduce((s, w) => s + (w.story_points ?? 3), 0);
  const remaining = Math.max(0, totalSp - doneSp);
  const ratio = totalSp ? doneSp / totalSp : 0;

  const ideal: TimeSeriesPoint[] = days.map((d, i) => ({
    x: d, y: Math.max(0, Math.round(totalSp * (1 - i / Math.max(1, n - 1)))),
  }));
  const actual: TimeSeriesPoint[] = days.map((d, i) => ({
    x: d, y: Math.max(0, totalSp - Math.round((doneSp / Math.max(1, n)) * Math.min(i, n - 1))),
  }));

  const sprintMeta = {
    sprint_id: sp.id, name: sp.name, start_date: sp.start_date,
    end_date: sp.end_date, total_sp: totalSp, scope_change_log: [],
  };

  const burndown: BurndownData = {
    sprint: sprintMeta, series: { ideal, actual }, scope_changes: [],
    summary: {
      remaining_sp: remaining, completed_sp: doneSp,
      completed_issues: done.length, total_issues: workItems.length,
      predicted_completion_sp: totalSp - doneSp, on_track: ratio >= 0.6,
    },
  };

  const burnup: BurnupData = {
    sprint: sprintMeta,
    series: {
      actual: days.map((d, i) => ({ x: d, y: Math.min(totalSp, Math.round((doneSp / Math.max(1, n)) * i)) })),
      scope: days.map((d) => ({ x: d, y: totalSp })),
    },
    scope_changes: [],
    summary: { completed_sp: doneSp, total_sp: totalSp, completion_ratio: ratio },
  };

  const velocity: VelocityData = {
    sprints: sprints.slice(0, 4).map((s) => ({
      sprint_id: s.id, name: s.name, committed_sp: 30,
      completed_sp: s.status === "completed" ? 28 : null,
    })),
    average_completed_sp: 28, trend: "stable",
  };

  const sprintReport: SprintReportData = {
    sprint: { sprint_id: sp.id, name: sp.name },
    groups: {
      completed: done.slice(0, 6).map((w) => toIssueRow(w, "Ulysses")),
      carry_over: workItems.filter((w) => w.status === "in_progress").slice(0, 3).map((w) => toIssueRow(w, "Ulysses")),
      incomplete: workItems.filter((w) => w.status === "todo").slice(0, 3).map((w) => toIssueRow(w, "Ulysses")),
    },
    summary: {
      completed_count: done.slice(0, 6).length,
      carry_over_count: workItems.filter((w) => w.status === "in_progress").slice(0, 3).length,
      incomplete_count: workItems.filter((w) => w.status === "todo").slice(0, 3).length,
      completed_sp: doneSp,
    },
  };

  const cfd: CfdData = {
    date_range: { start: sp.start_date, end: sp.end_date },
    status_categories: ["todo", "in_progress", "review", "done"],
    series: days.map((d, i) => ({
      day: d,
      counts: {
        todo: Math.max(0, 10 - i), in_progress: 4, review: 2,
        done: Math.round((done.length / Math.max(1, n)) * i),
      },
    })),
    total: workItems.length,
  };

  const cycleTime: CycleTimeData = {
    buckets: [
      { range_start: 0, range_end: 1, count: 4, label: "0-1天" },
      { range_start: 1, range_end: 3, count: 9, label: "1-3天" },
      { range_start: 3, range_end: 7, count: 5, label: "3-7天" },
      { range_start: 7, range_end: 14, count: 2, label: "7-14天" },
    ],
    percentiles: { p50: 2.5, p85: 5.1, p95: 7.4 },
    stats: { total_count: done.length || 20, median: 2.5, mean: 3.2 },
    bucket_size: 1,
  };

  const control: ControlChartData = {
    data_points: done.slice(0, 10).map((w, i) => ({
      workitem_id: w.id, key: w.key, cycle_time_days: 2 + (i % 5),
      completed_at: w.updated_at, anomaly: i % 7 === 6, z_score: i % 7 === 6 ? 2.4 : 0.3,
    })),
    reference_lines: [
      { y_value: 7.4, label: "UCL", style: "dashed" },
      { y_value: 3.2, label: "Mean", style: "solid" },
    ],
    stats: { median: 3.2, p70: 4.1, p85: 5.1, p95: 7.4, mean: 3.4, std_dev: 1.8 },
  };

  const cvr: CvrData = {
    series: days.slice(-7).map((d, i) => ({ day: d, created: 2 + (i % 3), resolved: 1 + (i % 4) })),
    summary: {
      total_created: workItems.length, total_resolved: done.length,
      net_change: workItems.length - done.length, backlog_trend: "stable",
    },
    time_granularity: "day",
  };

  const throughput = {
    series: days.filter((_, i) => i % 7 === 0).map((d, i) => ({ bucket: d, count: 3 + (i % 4) })),
    moving_avg: [], stats: { total: done.length || 20, avg: 4.2, std_dev: 1.1 },
  };

  const forecast: ForecastData = {
    historical: {
      sprints: sprints.slice(0, 4).map((s) => ({ name: s.name, completed_sp: 28 })),
      avg_velocity: 28,
    },
    forecast: {
      method: "simple_avg", predicted_velocity: 28,
      confidence_80: [22, 34], confidence_95: [18, 38],
      predicted_completion_date: sp.end_date,
    },
  };

  const timeTracking: TimeTrackingData = {
    granularity: "user",
    rows: [
      { id: "usr-001", name: "Ulysses", original_seconds: 220 * 3600, spent_seconds: 186 * 3600, remaining_seconds: 34 * 3600, progress: 0.84 },
    ],
    summary: { total_original: 220 * 3600, total_spent: 186 * 3600, total_remaining: 34 * 3600 },
  };

  const resolution: ResolutionTimeData = {
    group_by: "priority",
    rows: [
      { group: "critical", avg_days: 1.2, median_days: 1.0, count: 3 },
      { group: "high", avg_days: 2.4, median_days: 2.0, count: 6 },
      { group: "medium", avg_days: 4.1, median_days: 3.5, count: 11 },
    ],
  };

  const sla: SlaData = {
    series: days.slice(-5).map((d) => ({
      day: d, priorities: {
        critical: { met: 4, total: 4, compliance: 1 },
        high: { met: 8, total: 9, compliance: 0.89 },
        medium: { met: 10, total: 11, compliance: 0.91 },
      },
    })),
    summary: { overall_compliance: 0.91, by_priority: { critical: 1, high: 0.89, medium: 0.91 }, breaches: 5 },
    target_line: 0.9,
  };

  const mkSlices = (pairs: Array<[string, number]>, total: number): Array<{ key: string; count: number; percentage: number }> =>
    pairs.map(([k, c]) => ({ key: k, count: c, percentage: total ? clamp(Math.round((c / total) * 100)) : 0 }));

  const issueDist: IssueTypeDistData = {
    slices: mkSlices([
      ["task", workItems.filter((w) => w.kind === "task").length || 8], ["story", workItems.filter((w) => w.kind === "story").length || 7],
      ["epic", 1], ["spike", 2],
    ], (workItems.length || 22)),
    total: workItems.length || 22, status_filter: "all",
  };

  const priorityDist: PriorityDistData = {
    slices: mkSlices([["critical", 3], ["high", 6], ["medium", workItems.length || 14], ["low", 2]], (workItems.length || 25) + 11),
    total: (workItems.length || 25) + 11, status_filter: "all",
  };

  return {
    burndown, burnup, velocity, sprintReport, cfd, control, cycleTime, cvr,
    throughput, forecast, timeTracking, resolution, sla, issueDist, priorityDist,
  };
}

export type AnalyticsChartData = ReturnType<typeof buildAnalyticsData>;
