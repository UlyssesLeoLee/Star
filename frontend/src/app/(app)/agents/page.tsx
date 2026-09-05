"use client";

// =====================================================================
// /agents — Agent list + status + live activity feed (minimal placeholder)
// =====================================================================
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. agents 表格数据全部 mock — 真实 agent service 接入 P3
//   2. live activity feed 仅占位文案 — 真实 WS 接入 P3
//   3. light mode (per §7) P3
//   4. agent 状态机 / lease / heartbeat 详情面板 P2 (Phase I+)
//   5. useEffect+fetch 阶段用 MOCK_AGENTS_FALLBACK SSR 兜底 (per mock-msw-handlers §2.4 + §4 #1 缺标)
// =====================================================================

import { useEffect, useState } from "react";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Bot, Activity } from "lucide-react";
import { MOCK_AGENTS_FALLBACK } from "@/mocks/data";
import type { AgentRow } from "@/mocks/schemas/agent";
import { useTranslation } from "@/lib/i18n";

export default function AgentsPage() {
  const { t } = useTranslation();
  const [agents, setAgents] = useState<ReadonlyArray<AgentRow>>(MOCK_AGENTS_FALLBACK);
  useEffect(() => {
    fetch("/api/agents")
      .then((r) => r.json())
      .then((data: ReadonlyArray<AgentRow>) => setAgents(data))
      .catch(() => {
        /* keep FALLBACK (per §4 #1 缺标, 避免 UX 退化) */
      });
  }, []);

  const active = agents.filter((a) => a.status === "active" || a.status === "in_progress").length;

  return (
    <div className="max-w-7xl mx-auto" data-testid="agents-page">
      <PageHeader
        title={t.pageTitles['/agents'].title}
        subtitle="agent / agent-session / lease / runtime (5 mock rows; 真实数据 P3)"
        icon={<Bot className="text-accent" size={20} />}
        count={`${active} active`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Total" value={agents.length} hint="mock 5" tone="info" />
        <Stat label="Active" value={active} hint="active + in_progress" tone="ok" />
        <Stat label="Paused" value={agents.filter((a) => a.status === "paused").length} tone="warn" />
        <Stat label="Failed" value={agents.filter((a) => a.status === "failed").length} tone="err" />
      </div>

      <div className="card">
        <SectionTitle>Agent Sessions (mock)</SectionTitle>
        <table className="table">
          <thead>
            <tr>
              <th>ID</th>
              <th>Name</th>
              <th>Status</th>
              <th>Role</th>
              <th>Last Active</th>
            </tr>
          </thead>
          <tbody>
            {agents.map((a) => (
              <tr key={a.id} data-testid={`agent-row-${a.id}`}>
                <td className="font-mono text-xs text-ink-dim">{a.id}</td>
                <td className="font-mono text-sm text-info">{a.name}</td>
                <td><StatusPill value={a.status} size="xs" /></td>
                <td className="text-xs text-ink-dim">{a.role}</td>
                <td className="font-mono text-xs text-ink-dim">{a.last_active}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="card mt-3" data-testid="live-activity-placeholder">
        <SectionTitle>Live Activity Feed</SectionTitle>
        <div className="flex items-start gap-2 text-xs text-ink-dim py-3">
          <Activity size={14} className="text-info mt-0.5 shrink-0" />
          <div>
            <div className="text-ink-mute">
              P3 缺口 — Live activity feed (WS 接入) 待 Phase I+ 实装。本占位仅声明
              接入点, 不展示数据 (避免误导)。
            </div>
            <div className="font-mono text-[10px] text-ink-mute mt-1">
              schema: agent_id / kind / text / ts / tone
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
