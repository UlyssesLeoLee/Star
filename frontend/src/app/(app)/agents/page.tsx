// =====================================================================
// /agents — Panel placeholder (U4 owner, per design §2 + §5.3 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function AgentsPage() {
  return (
    <PanelPlaceholder
      title="Agents"
      description="agent / agent-session / lease / resume / runtime 聚合。3 column: agent 列表 (左 360px) / 详情 (中) / lease-heartbeat (右 320px)。SubNav 180px sticky。"
      owner="U4"
    />
  );
}
