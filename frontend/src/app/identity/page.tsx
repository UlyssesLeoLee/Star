/*
@cypher schema=1 source_sha256=2026-11-11-identity-real-login-v1
MERGE (self:File {path:"frontend/src/app/identity/page.tsx"})
MERGE (identityPage:Function {name:"IdentityPage", language:"typescript", visibility:"public"})
MERGE (googleSignIn:Function {name:"initGoogleIdentity", language:"typescript", visibility:"private"})
MERGE (self)-[:DEFINES]->(identityPage)
MERGE (self)-[:DEFINES]->(googleSignIn)
@endcypher
*/
"use client";

// =====================================================================
// /identity — 真实 Google 登录 + 会话状态 (per 2026-11-11 用户发令)
// 管理邮箱 lidian727@gmail.com = 唯一管理员; 其余 Google 用户为普通 user。
// =====================================================================

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import { useStore } from "@/lib/store";
import { PageHeader, Stat, SectionTitle } from "@/components/PageHeader";
import { StatusPill } from "@/components/StatusPill";
import { Tabs } from "@/components/Tabs";
import {
  Users,
  ShieldCheck,
  Building2,
  CheckCircle2,
  LogOut,
  ShieldAlert,
} from "lucide-react";
import { useTranslation } from "@/lib/i18n";
import {
  authApi,
  clearSession,
  getSession,
  setSessionToken,
  type MeBody,
  type StarSession,
} from "@/lib/auth";

declare global {
  interface Window {
    google?: {
      accounts: {
        id: {
          initialize: (config: { client_id: string; callback: (resp: { credential: string }) => void }) => void;
          renderButton: (element: HTMLElement, config: Record<string, unknown>) => void;
        };
      };
    };
  }
}

function identityProviderLabel(provider: string): string {
  switch (provider) {
    case "password":
      return "账号密码";
    case "saml-sso":
      return "企业单点登录";
    case "local-runtime-device":
      return "本地运行环境";
    case "github":
      return "GitHub";
    case "gitlab":
      return "GitLab";
    case "google":
      return "Google";
    default:
      return "身份提供方";
  }
}

const GOOGLE_CLIENT_ID = process.env.NEXT_PUBLIC_GOOGLE_CLIENT_ID ?? "";

export default function IdentityPage() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<string>("users");
  const [session, setSession] = useState<StarSession | null>(null);
  const [me, setMe] = useState<MeBody | null>(null);
  const [loginError, setLoginError] = useState<string | null>(null);
  const [loginBusy, setLoginBusy] = useState(false);
  const [gsiLoaded, setGsiLoaded] = useState(false);
  const identities = useStore((s) => s.identities);
  const tenants = useStore((s) => s.tenants);

  const refreshSession = useCallback(() => {
    const current = getSession();
    setSession(current);
    if (current) {
      void authApi.me().then((res) => {
        if (res.body) setMe(res.body);
      });
    } else {
      setMe(null);
    }
  }, []);

  useEffect(() => {
    refreshSession();
  }, [refreshSession]);

  // 加载 Google Identity Services (client id 未配置时显示配置缺位状态)
  useEffect(() => {
    if (gsiLoaded || GOOGLE_CLIENT_ID === "") return;
    const script = document.createElement("script");
    script.src = "https://accounts.google.com/gsi/client";
    script.async = true;
    script.onload = () => setGsiLoaded(true);
    document.body.appendChild(script);
    return () => {
      // script 保留 (GSI 重复加载会警告); 仅移除 handler 状态
    };
  }, [gsiLoaded]);

  const handleGoogleCredential = useCallback(async (credential: string) => {
    setLoginBusy(true);
    setLoginError(null);
    const result = await authApi.loginWithGoogleIdToken(credential);
    setLoginBusy(false);
    if (result.body) {
      setSessionToken(result.body.access_token);
      refreshSession();
    } else {
      setLoginError(result.error ?? "GOOGLE_LOGIN_FAILED");
    }
  }, [refreshSession]);

  const googleButtonRef = useCallback(
    (element: HTMLDivElement | null) => {
      if (!element || !gsiLoaded || GOOGLE_CLIENT_ID === "" || session) return;
      window.google?.accounts.id.initialize({
        client_id: GOOGLE_CLIENT_ID,
        callback: (resp) => void handleGoogleCredential(resp.credential),
      });
      window.google?.accounts.id.renderButton(element, {
        theme: "filled_black",
        size: "large",
        text: "signin_with",
        shape: "rectangular",
      });
    },
    [gsiLoaded, session, handleGoogleCredential],
  );

  const mfaCount = identities.filter((i) => i.mfa_enabled).length;

  return (
    <div className="max-w-7xl mx-auto">
      <PageHeader
        title={t.pageTitles['/identity'].title}
        subtitle="Google 登录 · 统一身份认证、权限矩阵与多租户工作空间管理"
        icon={<Users className="text-accent" size={20} />}
        track="D"
        count={`${identities.length} members`}
      />

      {/* 真实登录会话卡片 */}
      <div className="card mb-5" data-testid="identity-session-card">
        <SectionTitle>Google 登录会话</SectionTitle>
        {session && me ? (
          <div className="flex items-center gap-4 flex-wrap mt-3">
            <div
              className={`flex items-center gap-1.5 rounded border px-2 py-0.5 text-xs ${
                me.is_admin
                  ? "border-amber-400/60 bg-amber-400/10 text-amber-300"
                  : "border-cyan-400/60 bg-cyan-400/10 text-cyan-300"
              }`}
              data-testid="identity-admin-badge"
            >
              {me.is_admin ? <ShieldAlert size={12} /> : <ShieldCheck size={12} />}
              {me.is_admin ? "管理员 Admin" : "普通用户 User"}
            </div>
            <span className="font-mono text-sm text-ink" data-testid="identity-session-email">
              {me.email ?? session.payload.sub}
            </span>
            <span className="text-xs text-ink-dim font-mono">
              roles: {me.roles.join(", ")}
            </span>
            {me.is_admin && (
              <Link
                href="/admin"
                data-testid="identity-admin-link"
                className="btn text-xs py-1.5 px-3"
              >
                <ShieldCheck size={12} /> 打开后台管理
              </Link>
            )}
            <button
              data-testid="identity-logout"
              className="btn-secondary text-xs py-1.5 px-3"
              onClick={() => {
                clearSession();
                refreshSession();
              }}
            >
              <LogOut size={12} /> 退出登录
            </button>
          </div>
        ) : (
          <div className="mt-3 space-y-2">
            {GOOGLE_CLIENT_ID === "" ? (
              <div className="text-xs text-warn border border-warn/40 rounded p-3">
                Google 登录未配置: 缺少 <span className="font-mono">NEXT_PUBLIC_GOOGLE_CLIENT_ID</span>。
                在 Google Cloud Console 建 OAuth Web client 并配置后即可启用。
              </div>
            ) : (
              <>
                <div ref={googleButtonRef} data-testid="identity-google-button" />
                {!gsiLoaded && <div className="text-xs text-ink-dim">加载 Google 登录组件…</div>}
                {loginBusy && <div className="text-xs text-ink-dim">正在校验 Google 身份…</div>}
                {loginError && (
                  <div className="text-xs text-red-400 font-mono" data-testid="identity-login-error">
                    {loginError}
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="Total Members" value={identities.length} tone="info" />
        <Stat label="MFA Enforced" value={`${mfaCount}/${identities.length}`} tone="ok" />
        <Stat label="Active Tenants" value={tenants.length} tone="default" />
        <Stat
          label="Auth Providers"
          value={session ? "Google ✓" : "Google"}
          tone={session ? "ok" : "default"}
          hint="Google OIDC · 管理员邮箱唯一判定"
        />
      </div>

      <Tabs
        active={tab}
        onChange={setTab}
        items={[
          { id: "users", label: "Users 成员身份", icon: <Users size={12} />, badge: identities.length },
          { id: "permissions", label: "Roles & Permissions 权限", icon: <ShieldCheck size={12} /> },
          { id: "tenants", label: "Tenants 租户空间", icon: <Building2 size={12} />, badge: tenants.length },
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
                  <td><StatusPill value={identityProviderLabel(i.provider)} size="xs" /></td>
                  <td><StatusPill value={i.status} size="xs" /></td>
                  <td>
                    {i.mfa_enabled ? (
                      <span className="text-ok flex items-center gap-1 font-mono text-xs">
                        <CheckCircle2 size={11} /> ON
                      </span>
                    ) : (
                      <span className="text-warn flex items-center gap-1 font-mono text-xs">
                        <CheckCircle2 size={11} /> OFF
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
              <div className="p-3 rounded border border-amber-400/50 bg-bg-soft/40">
                <div className="text-sm font-semibold text-amber-300 mb-1">
                  admin <span className="text-[10px] font-mono text-ink-dim">(lidian727@gmail.com)</span>
                </div>
                <p className="text-xs text-ink-dim mb-2">唯一管理员 — 后台管理、全量读写与审计。由 STAR_ADMIN_EMAIL 邮箱全等判定。</p>
                <div className="text-[10px] font-mono text-amber-300">✓ Full Write · ✓ Admin Console · ✓ Audit View</div>
              </div>
              <div className="p-3 rounded border border-line bg-bg-soft/40">
                <div className="text-sm font-semibold text-info mb-1">user</div>
                <p className="text-xs text-ink-dim mb-2">Google 已验证用户 — 常规读写, 无后台权限。</p>
                <div className="text-[10px] font-mono text-info">✓ Worktree Read · ✓ Task Execute</div>
              </div>
              <div className="p-3 rounded border border-line bg-bg-soft/40">
                <div className="text-sm font-semibold text-ink-dim mb-1">auditor_read</div>
                <p className="text-xs text-ink-dim mb-2">审计只读 (规划中)。</p>
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
