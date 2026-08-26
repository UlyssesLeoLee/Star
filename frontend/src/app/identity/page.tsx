"use client";

import { useStore } from "@/lib/store";
import { ListPage } from "@/lib/page-builders";
import { Users } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";

export default function IdentityPage() {
  const identities = useStore((s) => s.identities);
  return (
    <ListPage
      title="Identities"
      subtitle="用户身份。6 种登录源 + MFA 状态。设备绑定 (Local Runtime) 也属 identity 域。"
      icon={<Users className="text-accent" size={20} />}
      track="D"
      items={identities}
      searchKeys={["email", "display_name"]}
      columns={[
        { key: "id", label: "ID", width: "100px", render: (i) => <span className="font-mono text-xs">{i.id}</span> },
        { key: "name", label: "Display name", render: (i) => <span className="font-medium">{i.display_name}</span> },
        { key: "email", label: "Email", render: (i) => <span className="text-ink-dim text-xs">{i.email}</span> },
        { key: "provider", label: "Provider", render: (i) => <StatusPill value={i.provider} /> },
        { key: "status", label: "Status", render: (i) => <StatusPill value={i.status} /> },
        { key: "mfa", label: "MFA", render: (i) => i.mfa_enabled ? <span className="text-ok">✓</span> : <span className="text-warn">—</span> },
        { key: "last", label: "Last login", render: (i) => <span className="text-ink-dim text-xs">{i.last_login_at ? new Date(i.last_login_at).toLocaleString() : "—"}</span> },
      ]}
    />
  );
}
