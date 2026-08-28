// =====================================================================
// /settings — Panel placeholder (U4 owner, per design §2 + §5.6 + §8.1)
// =====================================================================
"use client";

import { PanelPlaceholder } from "@/components/PanelPlaceholder";

export default function SettingsPage() {
  return (
    <PanelPlaceholder
      title="Settings"
      description="tenant / identity / permission / role / integration / scm 聚合。2 column: Profile / Workspace / Members / Permissions / Runtimes / Skills / Billing sidebar (左) + 选中的设置面板 (右)。"
      owner="U4"
    />
  );
}
