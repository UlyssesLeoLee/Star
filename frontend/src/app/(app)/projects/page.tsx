// =====================================================================
// /projects — Panel placeholder (U3 owner, per design §2 + §5.2 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function ProjectsPage() {
  return (
    <PanelPlaceholder
      title="Projects"
      description="project / workspace / planning / board / workflow / canvas 聚合。5 tab: List / Board / Gantt / Calendar / Workflow。SubNav 180px sticky。"
      owner="U3"
    />
  );
}
