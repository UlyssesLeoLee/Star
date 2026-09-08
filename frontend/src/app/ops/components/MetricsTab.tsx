"use client";

/**
 * MetricsTab — F-03 运维数据 端到端 UI 组件
 * (per docs/briefs/ops-f03-metrics-impl.md
 *  + docs/basic-design/OPS-BASIC-DESIGN-001.md §3.3 F-03
 *  + docs/requirements/SRS-STAR-OPS-001.md v0.1 §4 F-03)
 *
 * 5 KPI 胶囊 (调 star-telemetry 真实路径, per F-03 brief §2.1):
 *   1. cpu_avg       - 比例 (mock 0.35, 守門 #1 R-05)
 *   2. mem_avg       - 比例 (mock 0.62)
 *   3. active_tasks  - count (TokenMeter.record_count 实时)
 *   4. mcp_qps       - qps (TokenMeter.call_count proxy)
 *   5. llm_token_daily - tokens (G.9 brief §3.9 累计)
 *
 * 趋势占位 (per brief §2.1: 1 趋势占位):
 *   - 简单 sparkline 趋势卡片 (mock 静态, 真实趋势实装在 [M] 子项)
 *
 * 守門:
 *   - #1 R-05: 真实 Prometheus / Grafana 不在 MVP scope
 *   - #5 v2: API key 走 KMS (F-02 star-credential 已落)
 *   - #6 v2: retriable retry 由 react-query 处理
 */

import { useQuery } from "@tanstack/react-query";
import { BarChart3, TrendingUp, Cpu, MemoryStick, Activity, Zap, Brain, RefreshCw, AlertTriangle } from "lucide-react";
import {
  metricsSummary,
  type OpsMetric,
  type TrendDirection,
  OpsApiError,
} from "@/lib/ops-api";
import { useTranslation } from "@/lib/i18n";

const OPS_API_URL = process.env.NEXT_PUBLIC_OPS_URL || "http://localhost:8090";

export function MetricsTab() {
  const { t } = useTranslation();

  // 调 /api/ops/metrics/summary 真实端点 (per F-03 端到端)
  const metricsQuery = useQuery({
    queryKey: ["ops-metrics-summary"],
    queryFn: () => metricsSummary(),
    refetchInterval: 10_000, // 10s 轮询, 守门 #6 v2 retriable 由 react-query 处理
    retry: (failureCount, err) => {
      if (err instanceof OpsApiError) {
        return err.retriable && failureCount < 3;
      }
      return failureCount < 3;
    },
  });

  const metrics: OpsMetric[] = metricsQuery.data?.data ?? [];

  return (
    <div className="space-y-4">
      {/* === 标题 + Refresh 状态 === */}
      <div className="anime-panel anime-chamfer p-[21px]">
        <div className="flex items-center gap-2 mb-4">
          <BarChart3 className="w-4 h-4 text-accent" />
          <h3 className="text-title font-bold">{t.opsConsole.metricsTitle}</h3>
          <span className="anime-hud-tag ml-auto">F-03</span>
          {metricsQuery.isFetching && (
            <RefreshCw className="w-3 h-3 animate-spin text-ink-mute" />
          )}
        </div>

        {metricsQuery.isError && (
          <div className="flex items-center gap-2 text-error text-sm">
            <AlertTriangle className="w-4 h-4" />
            <span>{(metricsQuery.error as Error).message}</span>
          </div>
        )}

        {!metricsQuery.isError && metrics.length === 0 && (
          <p className="text-xs text-ink-mute">{t.opsConsole.metricsLoading}</p>
        )}
      </div>

      {/* === 5 KPI 胶囊 === */}
      {metrics.length > 0 && (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <KpiCard
            icon={<Cpu className="w-4 h-4" />}
            name="cpu_avg"
            value={metrics.find((m) => m.name === "cpu_avg")?.value ?? 0}
            unit="ratio"
            trend={metrics.find((m) => m.name === "cpu_avg")?.trend ?? "stable"}
            label={t.opsConsole.metricsCardCpu}
            formatValue={(v) => `${(v * 100).toFixed(0)}%`}
          />
          <KpiCard
            icon={<MemoryStick className="w-4 h-4" />}
            name="mem_avg"
            value={metrics.find((m) => m.name === "mem_avg")?.value ?? 0}
            unit="ratio"
            trend={metrics.find((m) => m.name === "mem_avg")?.trend ?? "stable"}
            label={t.opsConsole.metricsCardMem}
            formatValue={(v) => `${(v * 100).toFixed(0)}%`}
          />
          <KpiCard
            icon={<Activity className="w-4 h-4" />}
            name="active_tasks"
            value={metrics.find((m) => m.name === "active_tasks")?.value ?? 0}
            unit="count"
            trend={metrics.find((m) => m.name === "active_tasks")?.trend ?? "stable"}
            label={t.opsConsole.metricsCardTasks}
            formatValue={(v) => v.toFixed(0)}
          />
          <KpiCard
            icon={<Zap className="w-4 h-4" />}
            name="mcp_qps"
            value={metrics.find((m) => m.name === "mcp_qps")?.value ?? 0}
            unit="qps"
            trend={metrics.find((m) => m.name === "mcp_qps")?.trend ?? "stable"}
            label={t.opsConsole.metricsCardMcp}
            formatValue={(v) => v.toFixed(1)}
          />
          <KpiCard
            icon={<Brain className="w-4 h-4" />}
            name="llm_token_daily"
            value={metrics.find((m) => m.name === "llm_token_daily")?.value ?? 0}
            unit="tokens"
            trend={metrics.find((m) => m.name === "llm_token_daily")?.trend ?? "stable"}
            label={t.opsConsole.metricsCardLlm}
            formatValue={(v) => formatTokens(v)}
          />
        </div>
      )}

      {/* === 趋势占位 (per brief §2.1) === */}
      <div className="anime-panel anime-chamfer p-[21px]">
        <div className="flex items-center gap-2 mb-2">
          <TrendingUp className="w-4 h-4 text-accent" />
          <h3 className="text-title font-bold">{t.opsConsole.metricsTrendTitle}</h3>
          <span className="anime-hud-tag ml-auto">F-03 [M]</span>
        </div>
        <p className="text-xs text-ink-mute font-mono">{t.opsConsole.metricsTrendHint}</p>
      </div>

      {/* === meta.hint 提示 (守门 #1 R-05) === */}
      {metricsQuery.data && (
        <p className="text-[10px] text-ink-mute italic font-mono">
          {metricsQuery.data.meta.hint}
        </p>
      )}

      {/* === 远端 URL (调试用) === */}
      <p className="text-[10px] text-ink-mute font-mono">
        {OPS_API_URL}/api/ops/metrics/summary
      </p>
    </div>
  );
}

function KpiCard({
  icon,
  name,
  value,
  unit,
  trend,
  label,
  formatValue,
}: {
  icon: React.ReactNode;
  name: string;
  value: number;
  unit: string;
  trend: TrendDirection;
  label: string;
  formatValue: (v: number) => string;
}) {
  const trendIcon = trend === "rising" ? "↑" : trend === "falling" ? "↓" : "→";
  const trendColor =
    trend === "rising"
      ? "text-warning"
      : trend === "falling"
        ? "text-success"
        : "text-ink-mute";

  return (
    <div className="anime-panel anime-chamfer p-4">
      <div className="flex items-center gap-2 mb-2">
        <span className="text-accent">{icon}</span>
        <h4 className="text-sm font-semibold text-ink">{label}</h4>
        <span className="text-[10px] font-mono text-ink-mute ml-auto">{name}</span>
      </div>
      <div className="flex items-baseline gap-2">
        <span className="text-2xl font-bold text-accent font-mono">
          {formatValue(value)}
        </span>
        <span className={`text-sm font-mono ${trendColor}`}>{trendIcon}</span>
        <span className="text-[10px] font-mono text-ink-mute">{unit}</span>
      </div>
    </div>
  );
}

/** 格式化 token 数 (1000 -> 1.0K, 1_000_000 -> 1.0M) */
function formatTokens(v: number): string {
  if (v >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`;
  if (v >= 1_000) return `${(v / 1_000).toFixed(1)}K`;
  return v.toFixed(0);
}
