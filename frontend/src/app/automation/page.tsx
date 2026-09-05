"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle, Stat } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Zap, Webhook, Tag, UserPlus, GitBranch, Bot, Bell } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

const TRIGGER_ICON = {
  workitem_status_changed: Tag,
  pr_status_changed: GitBranch,
  agent_session_completed: Bot,
  schedule_cron: Zap,
  feedback_received: Bell,
  audit_event: Zap,
};

const ACTION_ICON = {
  assign_user: UserPlus,
  set_label: Tag,
  send_notification: Bell,
  create_worktree: GitBranch,
  call_webhook: Webhook,
  dispatch_agent: Bot,
};

export default function AutomationPage() {
  const { t } = useTranslation();
  const rules = useStore((s) => s.automationRules);
  const enabled = rules.filter((r) => r.enabled).length;
  const totalFired24h = rules.reduce((s, r) => s + r.execution_count_24h, 0);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/automation'].title}
        subtitle="Rule + Trigger + Condition(CEL) + Action + RuleExecutor。6 INV 保证可观测、可禁用、可审计。"
        icon={<Zap className="text-accent" size={20} />}
        track="E"
        count={rules.length}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Rules" value={rules.length} tone="info" />
        <Stat label="Enabled" value={enabled} tone="ok" />
        <Stat label="Disabled" value={rules.length - enabled} tone="default" />
        <Stat label="Fired (24h)" value={totalFired24h} tone="warn" />
      </div>

      <SectionTitle>Rules</SectionTitle>
      <div className="space-y-3">
        {rules.map((r) => {
          const TIcon = TRIGGER_ICON[r.trigger_kind] ?? Zap;
          return (
            <div key={r.id} className={`card ${!r.enabled ? "opacity-60" : ""}`}>
              <div className="flex items-start gap-3">
                <div className="size-8 rounded bg-accent/10 border border-accent/30 grid place-items-center text-accent shrink-0">
                  <TIcon size={14} />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-1">
                    <span className="font-mono text-xs text-ink-mute">{r.id}</span>
                    <span className="text-sm font-semibold">{r.name}</span>
                    {r.enabled ? (
                      <span className="pill border-ok/40 text-ok bg-ok/10 text-[10px]">enabled</span>
                    ) : (
                      <span className="pill border-ink-mute/40 text-ink-dim text-[10px]">disabled</span>
                    )}
                    <span className="ml-auto text-[10px] text-ink-mute font-mono">
                      fired {r.execution_count_24h}× in 24h
                      {r.last_fired_at && ` · last ${new Date(r.last_fired_at).toLocaleTimeString()}`}
                    </span>
                  </div>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-2 text-xs">
                    <div>
                      <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Trigger</div>
                      <div className="font-mono">{r.trigger_kind}</div>
                      <div className="font-mono text-[10px] text-ink-dim mt-0.5">
                        {JSON.stringify(r.trigger_filter)}
                      </div>
                    </div>
                    <div>
                      <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Condition (CEL)</div>
                      <div className="font-mono text-info">{r.condition_expr ?? "— (no guard)"}</div>
                    </div>
                    <div>
                      <div className="text-[10px] uppercase tracking-wider text-ink-mute mb-1">Actions</div>
                      {r.actions.map((a, i) => {
                        const AIcon = ACTION_ICON[a.kind as keyof typeof ACTION_ICON] ?? Zap;
                        return (
                          <div key={i} className="flex items-center gap-1.5 font-mono text-[11px]">
                            <AIcon size={10} className="text-accent" />
                            <span>{a.kind}</span>
                            <span className="text-ink-dim text-[10px]">{JSON.stringify(a.config)}</span>
                          </div>
                        );
                      })}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
