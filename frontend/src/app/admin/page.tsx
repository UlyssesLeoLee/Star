/*
@cypher schema=1 source_sha256=2026-11-11-admin-console-v1
MERGE (self:File {path:"frontend/src/app/admin/page.tsx"})
MERGE (adminPage:Function {name:"AdminPage", language:"typescript", visibility:"public"})
MERGE (statusCard:Function {name:"AdminStatusCard", language:"typescript", visibility:"private"})
MERGE (self)-[:DEFINES]->(adminPage)
MERGE (self)-[:DEFINES]->(statusCard)
@endcypher
*/
"use client";

// =====================================================================
// /admin — 管理员后台 (per 2026-11-11 用户发令)
// 门禁: 后端 GET /api/v1/admin/status 仅 role=admin (lidian727@gmail.com)。
// =====================================================================

import { useCallback, useEffect, useState } from "react";
import Link from "next/link";
import {
  ShieldCheck,
  ShieldAlert,
  Server,
  KeyRound,
  RefreshCw,
  ArrowLeft,
} from "lucide-react";
import { PageHeader, Stat } from "@/components/PageHeader";
import { authApi, getSession, type AdminStatusBody } from "@/lib/auth";

export default function AdminPage() {
  const [session] = useState(() => getSession());
  const [status, setStatus] = useState<AdminStatusBody | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    const result = await authApi.adminStatus();
    setLoading(false);
    if (result.body) {
      setStatus(result.body);
    } else {
      setError(result.error ?? "ADMIN_STATUS_FAILED");
    }
  }, []);

  useEffect(() => {
    if (session?.isAdmin) void load();
    else setLoading(false);
  }, [session, load]);

  if (!session) {
    return (
      <div className="max-w-3xl mx-auto">
        <PageHeader
          title="Admin Console 后台管理"
          subtitle="管理员专属后台"
          icon={<ShieldCheck className="text-accent" size={20} />}
          track="D"
        />
        <div className="card text-center py-10" data-testid="admin-unauth">
          <ShieldAlert size={32} className="text-warn mx-auto mb-3" />
          <div className="text-base font-semibold mb-2">未登录</div>
          <div className="text-xs text-ink-dim mb-4">
            后台管理仅限 Google 登录的管理员 (lidian727@gmail.com) 访问。
          </div>
          <Link href="/identity" className="btn text-xs" data-testid="admin-login-link">
            <ArrowLeft size={12} /> 前往登录
          </Link>
        </div>
      </div>
    );
  }

  if (!session.isAdmin) {
    return (
      <div className="max-w-3xl mx-auto">
        <PageHeader
          title="Admin Console 后台管理"
          subtitle="管理员专属后台"
          icon={<ShieldCheck className="text-accent" size={20} />}
          track="D"
        />
        <div className="card text-center py-10" data-testid="admin-forbidden">
          <ShieldAlert size={32} className="text-red-400 mx-auto mb-3" />
          <div className="text-base font-semibold mb-2">403 无权限</div>
          <div className="text-xs text-ink-dim mb-1 font-mono">
            {session.payload.email ?? session.payload.sub}
          </div>
          <div className="text-xs text-ink-dim mb-4">
            当前账号不是管理员。唯一管理员: lidian727@gmail.com。
          </div>
          <Link href="/identity" className="btn-secondary text-xs">
            返回身份页
          </Link>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-5xl mx-auto" data-testid="admin-console">
      <PageHeader
        title="Admin Console 后台管理"
        subtitle="系统状态 · 认证配置 · 唯一管理员"
        icon={<ShieldCheck className="text-amber-300" size={20} />}
        track="D"
        count={session.payload.email ?? session.payload.sub}
      />

      <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-5">
        <Stat label="管理员" value={session.payload.email ?? "-"} tone="ok" hint="STAR_ADMIN_EMAIL 唯一判定" />
        <Stat label="后端状态" value={status ? "在线" : loading ? "…" : "不可达"} tone={status ? "ok" : "warn"} />
        <Stat label="服务版本" value={status?.version ?? "-"} tone="default" />
        <Stat label="Google Client" value={status?.admin.google_client_configured ? "已配置" : "未配置"} tone={status?.admin.google_client_configured ? "ok" : "warn"} />
      </div>

      <div className="card">
        <div className="flex items-center justify-between mb-3">
          <span className="text-sm font-semibold flex items-center gap-1.5">
            <Server size={14} className="text-accent" /> 认证服务状态
          </span>
          <button className="btn-secondary text-xs" onClick={() => void load()} disabled={loading}>
            <RefreshCw size={12} /> 刷新
          </button>
        </div>
        {loading ? (
          <div className="text-xs text-ink-dim">加载中…</div>
        ) : error ? (
          <div className="text-xs text-red-400 font-mono" data-testid="admin-status-error">
            {error} — 后端 8081 未运行或 JWT 验签失败 (本地需 backend; Vercel 前端无法直连本地后端)。
          </div>
        ) : status ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-2 text-xs font-mono">
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <KeyRound size={12} className="text-accent" />
              <span className="text-ink-dim">server_time</span>
              <span className="ml-auto text-ink">{status.server_time}</span>
            </div>
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <ShieldCheck size={12} className="text-amber-300" />
              <span className="text-ink-dim">admin.email</span>
              <span className="ml-auto text-ink">{status.admin.email}</span>
            </div>
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <span className="text-ink-dim">auth.provider</span>
              <span className="ml-auto text-ink">{status.auth.provider}</span>
            </div>
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <span className="text-ink-dim">auth.issuer</span>
              <span className="ml-auto text-ink">{status.auth.issuer}</span>
            </div>
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <span className="text-ink-dim">auth.audience</span>
              <span className="ml-auto text-ink">{status.auth.audience}</span>
            </div>
            <div className="flex items-center gap-2 p-2 rounded border border-line bg-bg-soft/40">
              <span className="text-ink-dim">token_ttl</span>
              <span className="ml-auto text-ink">{status.auth.token_ttl_seconds}s</span>
            </div>
          </div>
        ) : null}
      </div>

      <div className="text-[10px] text-ink-mute font-mono mt-3">
        后台数据面板 (用户/租户/部署管理) 为后续 slice; 当前提供认证服务状态与配置可见性。
      </div>
    </div>
  );
}
