// =====================================================================
// /issues — Panel placeholder (U2 owner, per design §2 + §5.1 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function IssuesPage() {
  return (
    <PanelPlaceholder
      title="Issues"
      description="主面板: work-item / feedback / worktree / agent / decision / automation 聚合。Kanban + List + Tree + Sprint 4 tab 切换。"
      owner="U2"
    />
  );
}
