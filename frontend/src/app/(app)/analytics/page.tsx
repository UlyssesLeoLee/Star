// =====================================================================
// /analytics — Panel placeholder (U4 owner, per design §2 + §5.4 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function AnalyticsPage() {
  return (
    <PanelPlaceholder
      title="Analytics"
      description="dashboard / metric / cost / burndown 聚合。6 KPI cards (2x3) + 2 chart (cost trend + error mix) + 1 leaderboard。SubNav 180px sticky。"
      owner="U4"
    />
  );
}
