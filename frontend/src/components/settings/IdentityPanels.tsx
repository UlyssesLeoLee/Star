/*
@cypher schema=1 source_sha256=f300a02ef15e7768fd403e0a9ee5abd021c517d29f511dc6b51de2b401b30dc3
MERGE (self:File {path:"frontend/src/components/settings/IdentityPanels.tsx"})
MERGE (identityPanel:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::IdentityPanel",kind:"component"})
MERGE (profilePanel:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::ProfilePanel",kind:"component"})
MERGE (permissionPanel:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::PermissionPanel",kind:"component"})
MERGE (tenantPanel:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::TenantPanel",kind:"component"})
MERGE (deriveMemberRole:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::deriveMemberRole",kind:"function"})
MERGE (summarizeRules:Symbol {id:"frontend/src/components/settings/IdentityPanels.tsx::summarizeRules",kind:"function"})
MERGE (useStore:Symbol {id:"frontend/src/lib/store.ts::useStore",kind:"hook"})
MERGE (lucide:ExternalService {name:"lucide-react"})
MERGE (identityType:Type {id:"frontend/src/types/ids.ts::Identity"})
MERGE (auditEventType:Type {id:"frontend/src/types/ids.ts::AuditEvent"})
MERGE (permissionRuleType:Type {id:"frontend/src/types/ids.ts::PermissionRule"})
MERGE (self)-[:DEFINES]->(identityPanel)
MERGE (self)-[:DEFINES]->(profilePanel)
MERGE (self)-[:DEFINES]->(permissionPanel)
MERGE (self)-[:DEFINES]->(tenantPanel)
MERGE (self)-[:DEFINES]->(deriveMemberRole)
MERGE (self)-[:DEFINES]->(summarizeRules)
MERGE (identityPanel)-[:READS]->(useStore)
MERGE (profilePanel)-[:READS]->(useStore)
MERGE (permissionPanel)-[:READS]->(useStore)
MERGE (tenantPanel)-[:READS]->(useStore)
MERGE (identityPanel)-[:CALLS]->(deriveMemberRole)
MERGE (permissionPanel)-[:CALLS]->(summarizeRules)
MERGE (self)-[:IMPORTS]->(lucide)
MERGE (self)-[:USES_TYPE]->(identityType)
MERGE (self)-[:USES_TYPE]->(auditEventType)
MERGE (self)-[:USES_TYPE]->(permissionRuleType)
@endcypher
*/
// =====================================================================
// /settings 身份面板组 (per 2026-11-15, 4 个独立面板组件, 待接入 settings 主区)
//   IdentityPanel   成员与身份 — store.identities 真实成员列表 (角色徽标/状态/移除)
//   ProfilePanel    个人资料   — 本地默认当前用户 usr-001 (Ulysses)
//   PermissionPanel 角色与权限 — owner/admin/member 角色列表 + 只读权限矩阵
//   TenantPanel     租户       — 租户/项目数/成员数统计卡 + 邀请成员
//
// 诚实标注原则 (per 缺标比错标):
//   - 展示数据全部来自 useStore 真实集合 (本地 seed), 不伪造后端记录
//   - 后端未接入的能力 (移除成员/邀请成员/角色变更/偏好编辑) → 禁用 + 原因文案
// =====================================================================
"use client";

import { useState } from "react";
import { Building2, Shield, User, Users } from "lucide-react";
import { useStore } from "@/lib/store";
import type { AuditEvent, Identity, PermissionRule } from "@/types/ids";

/** 本地默认当前用户: seed identities[0] = usr-001 (Ulysses)。
 *  真实会话用户由后端 /api/v1/auth/me 注入 (lib/auth.ts), 未登录时回退此值。 */
const CURRENT_USER_ID = "usr-001";

const fmtDate = (iso?: string): string =>
  iso ? new Date(iso).toISOString().slice(0, 10) : "—";

// =====================================================================
// 成员角色推导 (诚实标注: Identity 无 role 字段, 后端 IAM 未接入)
//   证据 1: auditEvents 中 action="role.grant" 的记录 (au-014: usr-007 → developer)
//   证据 2: store.addProject 本地新建项目默认 owner_id = identities[0] (usr-001)
//   无证据 → null, UI 显示 "未标注"
// =====================================================================
function deriveMemberRole(
  identityId: string,
  auditEvents: AuditEvent[],
): { role: string; source: string } | null {
  const grant = [...auditEvents]
    .reverse()
    .find(
      (e) =>
        e.action === "role.grant" &&
        e.target_kind === "identity" &&
        e.target_id === identityId &&
        typeof e.payload.role === "string",
    );
  if (grant) {
    return { role: grant.payload.role as string, source: "审计记录 role.grant" };
  }
  if (identityId === CURRENT_USER_ID) {
    return { role: "owner", source: "本地默认 owner (addProject 默认 owner_id)" };
  }
  return null;
}

const STATUS_BADGE: Record<Identity["status"], { label: string; cls: string }> = {
  active: { label: "活跃", cls: "border-ok/40 bg-ok/10 text-ok" },
  invited: { label: "已邀请", cls: "border-warn/40 bg-warn/10 text-warn" },
  disabled: { label: "已禁用", cls: "border-err/40 bg-err/10 text-err" },
};

// =====================================================================
// IdentityPanel — 成员与身份
// =====================================================================
export function IdentityPanel() {
  const identities = useStore((s) => s.identities);
  const auditEvents = useStore((s) => s.auditEvents);

  return (
    <div data-testid="settings-identity-panel" className="space-y-3">
      <div>
        <h2 className="text-base font-bold text-ink flex items-center gap-1.5">
          <Users size={15} className="text-ink-mute" /> 成员与身份
        </h2>
        <p className="text-xs text-ink-dim mt-1">
          成员列表 (本地 seed: store.identities) · 角色由审计记录/本地默认推导 · 移除未接入后端
        </p>
      </div>

      <ul className="rounded-xl border border-line overflow-hidden">
        {identities.map((m) => {
          const role = deriveMemberRole(m.id, auditEvents);
          const st = STATUS_BADGE[m.status];
          return (
            <li
              key={m.id}
              data-testid={`settings-identity-row-${m.id}`}
              className="flex items-center gap-3 border-b border-line last:border-b-0 bg-bg-card px-3.5 py-2.5"
            >
              <div className="size-8 shrink-0 rounded-lg bg-accent/15 grid place-items-center text-[11px] font-bold text-accent">
                {m.display_name.slice(0, 1).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2">
                  <span className="text-xs font-semibold text-ink">{m.display_name}</span>
                  <span
                    data-testid={`settings-identity-role-${m.id}`}
                    title={role ? `来源: ${role.source}` : "本地 seed 未标注角色 (后端 IAM 未接入)"}
                    className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold ${
                      role
                        ? "border-accent/40 bg-accent/10 text-accent"
                        : "border-line bg-bg-soft/60 text-ink-mute"
                    }`}
                  >
                    {role ? role.role : "未标注"}
                  </span>
                </div>
                <div className="text-[10px] text-ink-mute font-mono mt-0.5">{m.email}</div>
              </div>
              <span className="hidden md:inline text-[10px] font-mono text-ink-dim">{m.provider}</span>
              <span
                data-testid={`settings-identity-status-${m.id}`}
                className={`rounded-full border px-2 py-0.5 text-[10px] ${st.cls}`}
              >
                {st.label}
              </span>
              <span className={`text-[10px] ${m.mfa_enabled ? "text-ok" : "text-ink-mute"}`}>
                {m.mfa_enabled ? "MFA" : "无MFA"}
              </span>
              <button
                type="button"
                disabled
                data-testid={`settings-identity-remove-${m.id}`}
                title="成员移除未接入后端 API (store 无 removeIdentity action)"
                className="rounded-lg border border-line bg-bg-soft/60 px-2.5 py-1 text-[11px] text-ink-mute cursor-not-allowed"
              >
                移除
              </button>
            </li>
          );
        })}
      </ul>

      <div
        className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
        data-testid="settings-identity-disabled-reason"
      >
        成员数据为本地 seed (store.identities) · 移除 / 邀请 / 角色变更未接入后端 API, 对应操作已禁用并标注原因。
      </div>
    </div>
  );
}

// =====================================================================
// ProfilePanel — 个人资料 (本地默认当前用户)
// =====================================================================
export function ProfilePanel() {
  const identities = useStore((s) => s.identities);
  const notifications = useStore((s) => s.notifications);
  const localRuntimes = useStore((s) => s.localRuntimes);
  const savedSearches = useStore((s) => s.savedSearches);

  const current = identities.find((i) => i.id === CURRENT_USER_ID) ?? identities[0];
  if (!current) {
    return (
      <div
        data-testid="settings-profile-panel"
        className="rounded-xl border border-line bg-bg-card px-3 py-6 text-center text-[11px] text-ink-mute"
      >
        无本地成员数据 (store.identities 为空)
      </div>
    );
  }
  const myDevices = localRuntimes.filter((r) => r.bound_user_id === current.id);
  const mySearches = savedSearches.filter((s) => s.created_by === current.id);
  const myInbox = notifications.filter((n) => n.recipient_id === current.id).length;

  return (
    <div data-testid="settings-profile-panel" className="space-y-3">
      <div>
        <h2 className="text-base font-bold text-ink flex items-center gap-1.5">
          <User size={15} className="text-ink-mute" /> 个人资料
        </h2>
        <p className="text-xs text-ink-dim mt-1">
          本地默认当前用户 (store.identities → usr-001) · 真实会话由后端 /api/v1/auth/me 注入
        </p>
      </div>

      {/* 用户卡片: 头像为姓名首字母占位 (store 无头像字段) */}
      <div className="rounded-xl border border-line bg-bg-card p-4 flex items-center gap-4">
        <div
          className="size-12 shrink-0 rounded-xl bg-accent/15 grid place-items-center text-lg font-bold text-accent"
          data-testid="settings-profile-avatar"
        >
          {current.display_name.slice(0, 1).toUpperCase()}
        </div>
        <div className="min-w-0 flex-1">
          <div className="text-sm font-bold text-ink" data-testid="settings-profile-name">
            {current.display_name}
          </div>
          <div className="text-[11px] text-ink-dim font-mono" data-testid="settings-profile-email">
            {current.email}
          </div>
          <div className="mt-1.5 flex items-center gap-1.5 flex-wrap">
            <span className="rounded-full border border-line bg-bg-soft/60 px-2 py-0.5 text-[10px] text-ink-dim">
              登录方式 {current.provider}
            </span>
            <span
              className={`rounded-full border px-2 py-0.5 text-[10px] ${
                current.mfa_enabled
                  ? "border-ok/40 bg-ok/10 text-ok"
                  : "border-line bg-bg-soft/60 text-ink-mute"
              }`}
            >
              {current.mfa_enabled ? "MFA 已开启" : "MFA 未开启"}
            </span>
            <span className="rounded-full border border-line bg-bg-soft/60 px-2 py-0.5 text-[10px] text-ink-dim">
              最后登录 {fmtDate(current.last_login_at)}
            </span>
          </div>
        </div>
      </div>

      {/* 偏好 (只读): 后端 profile/preferences API 未接入, 仅展示本地默认值 */}
      <div className="rounded-xl border border-line overflow-hidden">
        <div className="border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
          偏好 (本地默认, 只读)
        </div>
        <ul className="divide-y divide-line bg-bg-card text-[11px]">
          {[
            { k: "语言", v: "zh-CN (界面本地默认)" },
            { k: "主题", v: "由 lib/theme 管理 (本面板不编辑)" },
            { k: "通知渠道", v: "未接入 (后端通知偏好 API 未接入)" },
          ].map((row) => (
            <li key={row.k} className="flex items-center justify-between px-3.5 py-2">
              <span className="text-ink-dim">{row.k}</span>
              <span className="text-ink-mute">{row.v}</span>
            </li>
          ))}
        </ul>
      </div>

      {/* 关联数据 (store 真实集合) */}
      <div className="rounded-xl border border-line overflow-hidden">
        <div className="border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
          关联数据 (store 真实集合)
        </div>
        <ul className="divide-y divide-line bg-bg-card text-[11px]">
          <li className="flex items-center justify-between px-3.5 py-2">
            <span className="text-ink-dim">已绑定设备</span>
            <span data-testid="settings-profile-devices" className="text-ink">
              {myDevices.map((d) => `${d.hostname} (${d.status})`).join(" · ") || "—"}
            </span>
          </li>
          <li className="flex items-center justify-between px-3.5 py-2">
            <span className="text-ink-dim">已保存搜索</span>
            <span data-testid="settings-profile-saved-searches" className="text-ink">
              {mySearches.length}
            </span>
          </li>
          <li className="flex items-center justify-between px-3.5 py-2">
            <span className="text-ink-dim">收件箱通知</span>
            <span data-testid="settings-profile-inbox-count" className="text-ink">
              {myInbox}
            </span>
          </li>
        </ul>
      </div>

      <div
        className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
        data-testid="settings-profile-backend-note"
      >
        个人资料为本地 seed 数据 (store.identities) · 头像为姓名首字母占位 (store 无头像字段) · 编辑功能未提供 (后端 profile API 未接入)。
      </div>
    </div>
  );
}

// =====================================================================
// PermissionPanel — 角色与权限 (只读权限矩阵)
//   角色列表 owner/admin/member 为 UI 展示层映射, 对应 store.permissionRules
//   真实系统角色 tenant_admin / project_admin / developer / viewer。
// =====================================================================
const UI_ROLE_MAP: { ui: string; system: PermissionRule["role"][]; desc: string }[] = [
  { ui: "owner", system: ["tenant_admin"], desc: "租户级管理" },
  { ui: "admin", system: ["project_admin"], desc: "项目级管理" },
  { ui: "member", system: ["developer", "viewer"], desc: "成员 (开发 / 只读)" },
];
const MATRIX_ACTIONS = ["admin", "write", "read", "delete"] as const;

type CellState = "allow" | "deny" | "mixed" | "none";

function summarizeRules(
  rules: PermissionRule[],
  systemRoles: PermissionRule["role"][],
  action: (typeof MATRIX_ACTIONS)[number],
): { state: CellState; resources: string[] } {
  const hit = rules.filter((r) => systemRoles.includes(r.role) && r.action === action);
  if (hit.length === 0) return { state: "none", resources: [] };
  const hasAllow = hit.some((r) => r.effect === "allow");
  const hasDeny = hit.some((r) => r.effect === "deny");
  const resources = Array.from(new Set(hit.map((r) => r.resource_kind)));
  if (hasAllow && hasDeny) return { state: "mixed", resources };
  return { state: hasAllow ? "allow" : "deny", resources };
}

const CELL_LABEL: Record<CellState, { text: string; cls: string }> = {
  allow: { text: "允许", cls: "text-ok" },
  deny: { text: "拒绝", cls: "text-err" },
  mixed: { text: "部分", cls: "text-warn" },
  none: { text: "—", cls: "text-ink-mute" },
};

export function PermissionPanel() {
  const rules = useStore((s) => s.permissionRules);
  const schemes = useStore((s) => s.permissionSchemes);

  return (
    <div data-testid="settings-permission-panel" className="space-y-3">
      <div>
        <h2 className="text-base font-bold text-ink flex items-center gap-1.5">
          <Shield size={15} className="text-ink-mute" /> 角色与权限
        </h2>
        <p className="text-xs text-ink-dim mt-1">
          角色列表 (owner/admin/member) · 权限矩阵由 store.permissionRules 生成, 只读
        </p>
      </div>

      {/* 角色列表 */}
      <ul className="grid gap-2">
        {UI_ROLE_MAP.map((r) => (
          <li
            key={r.ui}
            data-testid={`settings-permission-role-${r.ui}`}
            className="flex items-center gap-3 rounded-xl border border-line bg-bg-card px-3.5 py-2.5"
          >
            <span className="text-xs font-semibold text-ink">{r.ui}</span>
            <span className="text-[10px] text-ink-dim">{r.desc}</span>
            <span className="ml-auto text-[10px] font-mono text-ink-mute">{r.system.join(" / ")}</span>
          </li>
        ))}
      </ul>

      {/* 权限方案 (store.permissionSchemes 真实) */}
      <div className="flex items-center gap-2 text-[11px]">
        <span className="text-ink-dim">权限方案</span>
        <span className="text-ink font-semibold" data-testid="settings-permission-scheme-count">
          {schemes.length} 个
        </span>
        <span className="text-ink-mute">({schemes.map((s) => s.name).join(" · ")})</span>
      </div>

      {/* 只读权限矩阵 (无任何可编辑控件) */}
      <div className="rounded-xl border border-line overflow-hidden">
        <div className="grid grid-cols-[1fr_repeat(4,minmax(0,1fr))] gap-2 border-b border-line bg-bg-soft/60 px-3.5 py-2 text-[10px] font-semibold text-ink-mute">
          <span>角色</span>
          {MATRIX_ACTIONS.map((a) => (
            <span key={a}>{a}</span>
          ))}
        </div>
        {UI_ROLE_MAP.map((row) => (
          <div
            key={row.ui}
            className="grid grid-cols-[1fr_repeat(4,minmax(0,1fr))] gap-2 items-center border-b border-line last:border-b-0 bg-bg-card px-3.5 py-2 text-[11px]"
          >
            <span className="text-ink font-semibold">{row.ui}</span>
            {MATRIX_ACTIONS.map((action) => {
              const cell = summarizeRules(rules, row.system, action);
              const label = CELL_LABEL[cell.state];
              return (
                <span
                  key={action}
                  data-testid={`settings-permission-cell-${row.ui}-${action}`}
                  title={cell.resources.length > 0 ? `资源: ${cell.resources.join(", ")}` : "无规则覆盖"}
                  className={label.cls}
                >
                  {label.text}
                </span>
              );
            })}
          </div>
        ))}
      </div>

      <div
        className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
        data-testid="settings-permission-readonly-note"
      >
        权限矩阵为本地 seed (store.permissionRules, {rules.length} 条规则) 只读展示 · 角色增删改需后端 RBAC API, 未接入。
      </div>
    </div>
  );
}

// =====================================================================
// TenantPanel — 租户 (统计卡 + 邀请成员)
// =====================================================================
export function TenantPanel() {
  const tenants = useStore((s) => s.tenants);
  const projects = useStore((s) => s.projects);
  const identities = useStore((s) => s.identities);
  const [inviteEmail, setInviteEmail] = useState("");
  const tenant = tenants[0];

  if (!tenant) {
    return (
      <div
        data-testid="settings-tenant-panel"
        className="rounded-xl border border-line bg-bg-card px-3 py-6 text-center text-[11px] text-ink-mute"
      >
        无本地租户数据 (store.tenants 为空)
      </div>
    );
  }

  return (
    <div data-testid="settings-tenant-panel" className="space-y-3">
      <div>
        <h2 className="text-base font-bold text-ink flex items-center gap-1.5">
          <Building2 size={15} className="text-ink-mute" /> 租户
        </h2>
        <p className="text-xs text-ink-dim mt-1">
          统计卡 (store.tenants/projects/identities) · 邀请成员未接入后端
        </p>
      </div>

      {/* 统计卡 */}
      <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
        <div className="rounded-xl border border-line bg-bg-card px-3.5 py-3">
          <div className="text-[10px] text-ink-mute">租户名</div>
          <div className="text-sm font-bold text-ink mt-0.5" data-testid="settings-tenant-name">
            {tenant.name}
          </div>
          <div className="text-[10px] text-ink-mute font-mono mt-0.5">
            {tenant.plan} · {tenant.status}
          </div>
        </div>
        <div className="rounded-xl border border-line bg-bg-card px-3.5 py-3">
          <div className="text-[10px] text-ink-mute">项目数</div>
          <div className="text-sm font-bold text-ink mt-0.5" data-testid="settings-tenant-projects-num">
            {projects.length}
          </div>
        </div>
        <div className="rounded-xl border border-line bg-bg-card px-3.5 py-3">
          <div className="text-[10px] text-ink-mute">成员数</div>
          <div className="text-sm font-bold text-ink mt-0.5" data-testid="settings-tenant-members-num">
            {identities.length}
          </div>
        </div>
        <div className="rounded-xl border border-line bg-bg-card px-3.5 py-3">
          <div className="text-[10px] text-ink-mute">席位</div>
          <div className="text-sm font-bold text-ink mt-0.5" data-testid="settings-tenant-seats-num">
            {identities.length} / {tenant.seat_limit}
          </div>
        </div>
      </div>

      {/* 邀请成员 (API 未接入 → 按钮禁用 + 原因; 输入仅本地暂存) */}
      <div className="rounded-xl border border-line bg-bg-card px-3.5 py-3">
        <div className="text-xs font-semibold text-ink">邀请成员</div>
        <div className="mt-2 flex items-center gap-2">
          <input
            type="email"
            data-testid="settings-tenant-invite-input"
            value={inviteEmail}
            onChange={(e) => setInviteEmail(e.target.value)}
            placeholder="name@company.com"
            className="flex-1 rounded-lg border border-line bg-bg px-3 py-1.5 text-xs text-ink placeholder:text-ink-mute outline-none focus:border-accent"
          />
          <button
            type="button"
            disabled
            data-testid="settings-tenant-invite-submit"
            className="rounded-lg border border-line bg-bg-soft/60 px-3 py-1.5 text-[11px] text-ink-mute cursor-not-allowed"
          >
            发送邀请
          </button>
        </div>
        <p className="mt-1.5 text-[10px] text-warn" data-testid="settings-tenant-invite-reason">
          邀请成员 API 未接入 — 输入仅本地暂存, 不会发送 (接入后端后启用发送邀请)
        </p>
      </div>

      <div
        className="rounded-lg border border-warn/40 bg-warn/10 px-3 py-2 text-[10px] leading-relaxed text-warn"
        data-testid="settings-tenant-local-note"
      >
        租户数据为本地 seed (store.tenants) · 租户更名 / 席位变更 / 邀请成员均未接入后端 API。
      </div>
    </div>
  );
}
