/*
@cypher schema=1 source_sha256=47656843a68d1ceef09666a51d177d295736e2e37a47c935d71842ee971b0814
MERGE (self:File {path:"frontend/src/app/integration/page.tsx"})
MERGE (page:Symbol {id:"frontend/src/app/integration/page.tsx::IntegrationPage",kind:"component"})
MERGE (self)-[:DEFINES]->(page)
@endcypher
*/
"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Plug, AlertCircle, RefreshCw } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export default function IntegrationPage() {
  const { t } = useTranslation();
  const integrations = useStore((s) => s.integrations);
  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/integration'].title}
        subtitle="连接代码托管、团队协作、任务管理和自动化服务。事件幂等键用于防止重复投递。"
        icon={<Plug className="text-accent" size={20} />}
        track="C"
        count={integrations.length}
      />

      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th><th>Kind</th><th>Display name</th><th>Status</th>
              <th>Config (masked)</th><th>Loop key</th>
              <th><RefreshCw size={10} className="inline" /> Last sync</th>
              <th><AlertCircle size={10} className="inline" /> 24h errors</th>
            </tr>
          </thead>
          <tbody>
            {integrations.map((i) => (
              <tr key={i.id}>
                <td className="font-mono text-xs">{i.id}</td>
                <td><StatusPill value={i.kind} size="xs" /></td>
                <td className="font-medium">{i.display_name}</td>
                <td><StatusPill value={i.status} /></td>
                <td className="font-mono text-xs text-ink-dim">{i.config_masked}</td>
                <td className="font-mono text-xs text-warn">{i.loop_protection_key ?? "—"}</td>
                <td className="text-ink-dim text-xs">{i.last_sync_at ? new Date(i.last_sync_at).toLocaleTimeString() : "—"}</td>
                <td>
                  {i.error_count_24h > 0 ? (
                    <span className={`font-mono text-xs ${i.error_count_24h > 5 ? "text-err" : "text-warn"}`}>
                      {i.error_count_24h}
                    </span>
                  ) : (
                    <span className="text-ok font-mono text-xs">0</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
