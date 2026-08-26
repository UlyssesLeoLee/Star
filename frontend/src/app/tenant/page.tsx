"use client";

import { useStore } from "@/lib/store";
import { ListPage } from "@/lib/page-builders";
import { Building2 } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";

export default function TenantPage() {
  const tenants = useStore((s) => s.tenants);
  return (
    <ListPage
      title="Tenants"
      subtitle="13 类 tenant_id 必带对象 (§6.1 REQ-SEC-001) 的根租户。每租户独立计费、隔离、审计。"
      icon={<Building2 className="text-accent" size={20} />}
      track="D"
      items={tenants}
      searchKeys={["name", "slug"]}
      columns={[
        { key: "id", label: "ID", width: "100px", render: (t) => <span className="font-mono text-xs">{t.id}</span> },
        { key: "name", label: "Name", render: (t) => <span className="font-medium">{t.name}</span> },
        { key: "slug", label: "Slug", render: (t) => <span className="font-mono text-info">{t.slug}</span> },
        { key: "plan", label: "Plan", render: (t) => <StatusPill value={t.plan} /> },
        { key: "status", label: "Status", render: (t) => <StatusPill value={t.status} /> },
        { key: "seats", label: "Seats", render: (t) => <span className="font-mono">{t.seat_limit}</span> },
        { key: "created", label: "Created", render: (t) => <span className="text-ink-dim text-xs">{new Date(t.created_at).toLocaleDateString()}</span> },
      ]}
    />
  );
}
