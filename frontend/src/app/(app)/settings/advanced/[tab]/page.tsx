/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/app/(app)/settings/advanced/[tab]/page.tsx",type:"file",language:"tsx"}),
  (page:Function {name:"AdvancedSettingsTabPage",type:"function",signature:"AdvancedSettingsTabPage()",visibility:"public",complexity:"moderate"}),
  (placeholder:Function {name:"CapabilityPlaceholder",type:"function",signature:"CapabilityPlaceholder({title,description,registry})",visibility:"private",complexity:"simple"}),
  (params:Variable {name:"params",type:"variable",language:"typescript"}),
  (tab:Variable {name:"tab",type:"variable",language:"typescript"}),
  (page)-[:CALLS]->(placeholder),(page)-[:USES]->(params),(page)-[:USES]->(tab),
  (file)-[:CONTAINS]->(page),(file)-[:CONTAINS]->(placeholder);
*/

"use client";

import { useParams, notFound } from "next/navigation";
import { Blocks, Plug, Sparkles } from "lucide-react";
import HookPolicyPage from "./HookPolicyPage";

const CAPABILITIES = {
  skills: {
    title: "Skills",
    description: "Skill 包、版本、作用域与 Agent 可用性将在此集中管理。",
    registry: "Skill Registry 尚未接入当前 UI。",
  },
  mcp: {
    title: "MCP",
    description: "连接器的发现、授权、启动与热插拔状态将在此集中管理。",
    registry: "MCP Runtime 与授权目录尚未接入当前 UI。",
  },
  plugins: {
    title: "Plugins",
    description: "Plugin 安装、兼容性、权限、启停与热插拔状态将在此集中管理。",
    registry: "Plugin Registry 与运行时生命周期尚未接入当前 UI。",
  },
} as const;

export default function AdvancedSettingsTabPage() {
  const params = useParams<{ tab: string }>();
  const tab = params.tab;

  if (tab === "hooks") return <HookPolicyPage />;
  if (tab === "skills" || tab === "mcp" || tab === "plugins") {
    return <CapabilityPlaceholder {...CAPABILITIES[tab]} />;
  }
  return notFound();
}

function CapabilityPlaceholder({
  title,
  description,
  registry,
}: {
  title: string;
  description: string;
  registry: string;
}) {
  const Icon = title === "Skills" ? Sparkles : title === "MCP" ? Plug : Blocks;
  return (
    <div className="card p-5" data-testid={`advanced-settings-${title.toLowerCase()}-placeholder`}>
      <div className="flex items-start gap-3">
        <span className="grid size-9 shrink-0 place-items-center rounded-lg border border-line bg-bg-soft text-accent"><Icon size={18} /></span>
        <div>
          <h2 className="text-base font-semibold text-ink">{title}</h2>
          <p className="mt-1 text-sm text-ink-dim">{description}</p>
          <p className="mt-4 rounded border border-line bg-bg-soft/60 px-3 py-2 text-xs text-ink-mute">{registry}</p>
        </div>
      </div>
    </div>
  );
}
