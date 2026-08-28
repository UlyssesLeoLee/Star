// =====================================================================
// /inbox — Panel placeholder (U4 owner, per design §2 + §5.5 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function InboxPage() {
  return (
    <PanelPlaceholder
      title="Inbox"
      description="通知 / 评论 / 审计 feed 聚合。3 column: 通知源 / 通知列表 / 详情。Phase I+ 实时 SSE 推送。"
      owner="U4"
    />
  );
}
