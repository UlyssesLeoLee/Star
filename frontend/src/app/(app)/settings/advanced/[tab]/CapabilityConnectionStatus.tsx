// SPDX-License-Identifier: MIT OR Apache-2.0
/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityConnectionStatus.tsx"})
MERGE (notice:Symbol {id:"frontend/src/app/(app)/settings/advanced/[tab]/CapabilityConnectionStatus.tsx::LocalOnlyConnectionNotice",kind:"component"})
MERGE (self)-[:DEFINES]->(notice)
@endcypher
*/
// =====================================================================
// CapabilityConnectionStatus — MCP / Skills / Plugins 面板的接驳状态条
// (per P1 设置收敛任务): 这三类能力当前没有带权限校验的服务端配置端点,
// 面板数据仅保存在本浏览器 localStorage。按「缺标比错标」原则, 面板顶部
// 明确显示未接通状态, 不假装存在服务端持久化; 服务端接入后此状态条应移除。
// =====================================================================
"use client";

import { AlertTriangle } from "lucide-react";

export type LocalCapabilityKind = "mcp" | "skills" | "plugins";

const CAPABILITY_LABELS: Record<LocalCapabilityKind, string> = {
  mcp: "MCP 服务器",
  skills: "Skills",
  plugins: "Plugins",
};

export function LocalOnlyConnectionNotice({ capability }: { capability: LocalCapabilityKind }) {
  const label = CAPABILITY_LABELS[capability];
  return (
    <div
      data-testid={`${capability}-local-only-notice`}
      role="status"
      className="flex items-start gap-2 rounded border border-warn/40 bg-warn/10 px-3 py-2 text-xs text-warn"
    >
      <AlertTriangle size={13} className="mt-0.5 shrink-0" />
      <span>
        未接通 — {label} 配置当前仅保存在本浏览器 localStorage (刷新后仍保留), 尚未接入带权限校验的服务端配置端点。
        待服务端接入后, 本面板将改为读写服务端配置; 当前更改不会同步到其他设备或团队成员。
      </span>
    </div>
  );
}
