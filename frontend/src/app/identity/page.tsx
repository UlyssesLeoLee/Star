"use client";

// =====================================================================
// /identity — 身份与访问治理中心 (Users / Permissions / Tenants)
// =====================================================================

import { useState } from "react";
import { useStore } from "@/lib/store";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs } from "@/components/Tabs";
import { Users, ShieldCheck, Building2, Key, CheckCircle2, AlertCircle } from "lucide-react";

export default function IdentityPage() {
  const [tab, setTab] = useState<string>("users");
  const identities = useStore((s) => s.identities);
  const tenants = useStore((s) => s.tenants);

  const mfaCount = identities.filter((i) => i.mfa_enabled).length;

  return (
    <div className="max-w-7xl mx-auto">
      <PageHeader
        title="Identity & Access"
        subtitle="统一身份认证、权限矩阵与多租户工作空间管理"
        icon={<Users className="text-accent" size={20} />}
        track="D"
        count={`${identities.length} members`}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Total Members" value={identities.length} tone="info" />
        <Stat label="MFA Enforced" value={`${mfaCount}/${identities.length}`} tone="ok" />
        <Stat label="Active Tenants" value={tenants.length} tone="default" />
        <Stat label="Auth Providers" value="6 Connected" tone="default" hint="GitHub / Google / SSO" />
      </div>

      <Tabs
        active={tab}
        onChange={setTab}
        items={[
          { id: "users",       label: "Users 成员身份",      icon: <Users size={12} />,       badge: identities.length },
          { id: "permissions", label: "Roles & Permissions 权限", icon: <ShieldCheck size={12} /> },
          { id: "tenants",     label: "Tenants 租户空间",      icon: <Building2 size={12} />,   badge: tenants.length },
        ]}
      />

      {tab === "users" && (
        <div data-testid="tab-users" className="card overflow-x-auto">
          <SectionTitle>User Identities & MFA Status</SectionTitle>
          <table className="table mt-2">
            <thead>
              <tr>
                <th>ID</th>
                <th>Display Name</th>
                <th>Email</th>
                <th>Provider</th>
                <th>Status</th>
                <th>MFA</th>
                <th>Last Login</th>
              </tr>
            </thead>
            <tbody>
              {identities.map((i) => (
                <tr key={i.id} className="hover:bg-bg-soft/50">
                  <td className="font-mono text-xs text-accent">{i.id}</td>
                  <td className="font-medium">{i.display_name}</td>
                  <td className="text-ink-dim text-xs font-mono">{i.email}</td>
                  <td><StatusPill value={i.provider} size="xs" /></td>
                  <td><StatusPill value={i.status} size="xs" /></td>
                  <td>
                    {i.mfa_enabled ? (
                      <span className="text-ok flex items-center gap-1 font-mono text-xs">
                        <CheckCircle2 size={11} /> ON
                      </span>
                    ) : (
                      <span className="text-warn flex items-center gap-1 font-mono text-xs">
                        <AlertCircle size={11} /> OFF
                      </span>
                    )}
                  </td>
                  <td className="text-ink-dim text-xs font-mono">
                    {i.last_login_at ? new Date(i.last_login_at).toLocaleString() : "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {tab === "permissions" && (
        <div data-testid="tab-permissions" className="space-y-4">
          <div className="card">
            <SectionTitle>Role Access Matrix (RBAC 策略)</SectionTitle>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mt-3">
              <div className="p-3 rounded border border-line bg-bg-soft/40">
                <div className="text-sm font-semibold text-accent mb-1">tenant_admin</div>
                <p className="text-xs text-ink-dim mb-2">最高租户权限，包含工作空间创建、计费模型与团队成员管理。</p>
                <div className="text-[10px] font-mono text-ok">✓ Full Write · ✓ Token Manage · ✓ Audit View</div>
              </div>
              <div className="p-3 rounded border border-line bg-bg-soft/40">
                <div className="text-sm font-semibold text-info mb-1">agent_operator</div>
                <p className="text-xs text-ink-dim mb-2">智能体操作员，支持 Worktree 挂载、CLI 进程启动与任务分发。</p>
                <div className="text-[10px] font-mono text-info">✓ Worktree Spawn · ✓ Task Execute</div>
              </div>
              <div className="p-3 rounded border border-line bg-bg-soft/40">
                <div className="text-sm font-semibold text-ink-dim mb-1">auditor_read</div>
                <p className="text-xs text-ink-dim mb-2">审计只读权限，支持跨租户日志链查看与合规审查。</p>
                <div className="text-[10px] font-mono text-ink-mute">✓ Read Only · ✗ No Mutation</div>
              </div>
            </div>
          </div>
        </div>
      )}

      {tab === "tenants" && (
        <div data-testid="tab-tenants" className="card">
          <SectionTitle>Active Tenants & Workspaces</SectionTitle>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mt-3">
            {tenants.map((t) => (
              <div key={t.id} className="p-3 rounded border border-line bg-bg-soft/40 hover:border-accent/40 transition-colors">
                <div className="flex items-center justify-between mb-1">
                  <span className="text-sm font-semibold text-ink">{t.name}</span>
                  <StatusPill value={t.status} size="xs" />
                </div>
                <div className="text-xs text-ink-dim font-mono mb-2">Slug: {t.slug} · ID: {t.id}</div>
                <div className="text-[10px] text-ink-mute font-mono">Created: {new Date(t.created_at).toLocaleDateString()}</div>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
