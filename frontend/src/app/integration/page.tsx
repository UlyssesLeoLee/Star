"use client";

import { useStore } from "@/lib/store";
import { PageHeader } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Plug, AlertCircle, RefreshCw } from "lucide-react";

export default function IntegrationPage() {
  const integrations = useStore((s) => s.integrations);
  return (
    <div className="max-w-7xl">
      <PageHeader
        title="Integrations"
        subtitle="外部服务适配器 (GitHub / GitLab / Jira / Slack / Lark / Linear / Webhook)。loop_protection_key 防止 webhook 风暴。"
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
