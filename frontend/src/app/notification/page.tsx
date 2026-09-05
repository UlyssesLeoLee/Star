"use client";

import { useStore } from "@/lib/store";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Bell, BellOff } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

export default function NotificationPage() {
  const { t } = useTranslation();
  const { notifications, markNotificationRead } = useStore();
  const delivered = notifications.filter((n) => n.status === "delivered").length;
  const read = notifications.filter((n) => n.status === "read").length;
  const suppressed = notifications.filter((n) => n.status === "suppressed").length;
  const pending = notifications.filter((n) => n.status === "pending").length;

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/notification'].title}
        subtitle="Inbox + Email + IM 渠道。INV-N-07 抑制策略:同 actor 60min 内同 kind 第 2 次自动 suppress(写入 audit)。"
        icon={<Bell className="text-accent" size={20} />}
        track="B"
        count={notifications.length}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Delivered" value={delivered} tone="info" />
        <Stat label="Read" value={read} tone="ok" />
        <Stat label="Pending" value={pending} tone="warn" />
        <Stat label="Suppressed" value={suppressed} tone="default" hint="INV-N-07 抑制" />
      </div>

      <SectionTitle>Inbox</SectionTitle>
      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>ID</th><th>Recipient</th><th>Kind</th><th>Channel</th>
              <th>Status</th><th>Subject</th><th>Suppression reason</th><th></th>
            </tr>
          </thead>
          <tbody>
            {notifications.map((n) => (
              <tr key={n.id} className={n.status === "suppressed" ? "opacity-60" : ""}>
                <td className="font-mono text-xs">{n.id}</td>
                <td className="font-mono text-xs">{n.recipient_id}</td>
                <td><StatusPill value={n.kind} size="xs" /></td>
                <td>
                  {n.channel === "suppressed" ? (
                    <span className="inline-flex items-center gap-1 text-ink-mute text-xs">
                      <BellOff size={10} /> suppressed
                    </span>
                  ) : (
                    <StatusPill value={n.channel} size="xs" />
                  )}
                </td>
                <td><StatusPill value={n.status} size="xs" /></td>
                <td className="text-sm">{n.subject}</td>
                <td className="text-xs text-ink-dim">{n.suppression_reason ?? "—"}</td>
                <td>
                  {n.status === "delivered" && (
                    <button onClick={() => markNotificationRead(n.id)} className="btn text-[10px]">mark read</button>
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
