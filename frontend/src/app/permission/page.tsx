"use client";

import { useStore } from "@/lib/store";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { ShieldCheck } from "lucide-react";
import { StatusPill } from "@/components/StatusPill";
import { useTranslation } from "@/lib/i18n";

export default function PermissionPage() {
  const { t } = useTranslation();
  const schemes = useStore((s) => s.permissionSchemes);
  const rules = useStore((s) => s.permissionRules);

  return (
    <div className="max-w-7xl">
      <PageHeader
        title={t.pageTitles['/permission'].title}
        subtitle="Rules-based RBAC。scheme 可绑定 project;rule 定义 (resource_kind, action, role, effect, condition CEL)。"
        icon={<ShieldCheck className="text-accent" size={20} />}
        track="D"
        count={schemes.length}
      />

      <SectionTitle>Permission Schemes</SectionTitle>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mb-5">
        {schemes.map((s) => (
          <div key={s.id} className="card">
            <div className="flex items-center justify-between mb-2">
              <div className="text-sm font-semibold">{s.name}</div>
              {s.is_default && <span className="pill border-accent/40 text-accent bg-accent/10 text-[10px]">default</span>}
            </div>
            <div className="text-xs text-ink-mute font-mono mb-1">{s.id}</div>
            <div className="text-xs text-ink-dim">
              {s.rule_count} rules
              {s.project_id && <> · project <span className="font-mono">{s.project_id}</span></>}
            </div>
          </div>
        ))}
      </div>

      <SectionTitle>Rules</SectionTitle>
      <div className="card">
        <table className="table">
          <thead>
            <tr>
              <th>Scheme</th>
              <th>Resource</th>
              <th>Action</th>
              <th>Role</th>
              <th>Effect</th>
              <th>Condition (CEL)</th>
            </tr>
          </thead>
          <tbody>
            {rules.map((r) => {
              const scheme = schemes.find((s) => s.id === r.scheme_id);
              return (
                <tr key={r.id}>
                  <td className="text-xs text-ink-dim">{scheme?.name ?? r.scheme_id}</td>
                  <td><StatusPill value={r.resource_kind} size="xs" /></td>
                  <td className="font-mono text-xs">{r.action}</td>
                  <td className="font-mono text-xs">{r.role}</td>
                  <td><StatusPill value={r.effect} size="xs" /></td>
                  <td className="font-mono text-xs text-info">{r.condition ?? "—"}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
