"use client";

// =====================================================================
// /settings — Profile / Account / Team / Billing / API Keys (minimal placeholder)
// =====================================================================
// 已知缺口 (per 缺标比错标安全, 8/26 JST):
//   1. 所有 submit endpoint P3 缺口 — 当前仅本地 state, 不发起请求
//   2. API key 实际加密存储 P3 (目前仅展示, 不写后端)
//   3. 7 tab 简化为 5 tab (Profile/Account/Team/Billing/API Keys)
//      缺失: Workspace / Members / Permissions / Runtimes / Skills (P2)
//   4. left sidebar 嵌套导航 P2
//   5. light mode (per §7) P3
// =====================================================================

import { useState } from "react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import { Tabs, type TabItem } from "@/components/Tabs";
import { Settings, User, Users, CreditCard, Key } from "lucide-react";
import { useTranslation } from "@/lib/i18n";

type SettingsTab = "profile" | "account" | "team" | "billing" | "apikeys";

const TAB_IDS: ReadonlyArray<SettingsTab> = ["profile", "account", "team", "billing", "apikeys"];
const TAB_ICONS: Record<SettingsTab, React.ReactNode> = {
  profile: <User size={12} />,
  account: <Settings size={12} />,
  team: <Users size={12} />,
  billing: <CreditCard size={12} />,
  apikeys: <Key size={12} />,
};
// v0.6 (per 2026-09-05 拍板 C): tab label 走 i18n, 组件内构造
const TAB_LABELS: Record<SettingsTab, Record<string, string>> = {
  profile: { "zh-CN": "个人中心", en: "Profile", ja: "プロフィール" },
  account: { "zh-CN": "账户", en: "Account", ja: "アカウント" },
  team: { "zh-CN": "团队", en: "Team", ja: "チーム" },
  billing: { "zh-CN": "计费", en: "Billing", ja: "請求" },
  apikeys: { "zh-CN": "API 凭据", en: "API Keys", ja: "API キー" },
};

type FieldProps = {
  label: string;
  value: string;
  placeholder?: string;
  type?: "text" | "email" | "password";
  onChange: (v: string) => void;
};
function Field({ label, value, placeholder, type = "text", onChange }: FieldProps) {
  return (
    <label className="block text-xs">
      <span className="block text-ink-dim mb-1">{label}</span>
      <input
        type={type}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
        className="w-full rounded-md border border-line bg-bg-soft px-2.5 py-1.5 text-sm text-ink font-mono placeholder:text-ink-mute focus:border-accent focus:outline-none"
      />
    </label>
  );
}

function SimpleForm({ fields, onSave }: {
  fields: ReadonlyArray<Omit<FieldProps, "onChange"> & { key: string }>;
  onSave: () => void;
}) {
  const [vals, setVals] = useState<Record<string, string>>(() =>
    Object.fromEntries(fields.map((f) => [f.key, f.value])),
  );
  return (
    <div className="space-y-3">
      {fields.map((f) => (
        <Field
          key={f.key}
          label={f.label}
          value={vals[f.key] ?? ""}
          placeholder={f.placeholder}
          type={f.type}
          onChange={(v) => setVals((p) => ({ ...p, [f.key]: v }))}
        />
      ))}
      <div className="flex items-center gap-2 pt-1">
        <button type="button" onClick={onSave} className="btn-primary text-xs">
          Save
        </button>
        <span className="text-[10px] text-ink-mute font-mono">
          P3 缺口: submit endpoint 未实装
        </span>
      </div>
    </div>
  );
}

export default function SettingsPage() {
  const { t, language } = useTranslation();
  const [tab, setTab] = useState<SettingsTab>("profile");
  // v0.6 (per 2026-09-05 拍板 C): tab label 走 i18n
  const TABS: ReadonlyArray<TabItem> = TAB_IDS.map((id) => ({
    id,
    label: TAB_LABELS[id][language] ?? TAB_LABELS[id].en,
    icon: TAB_ICONS[id],
  }));
  // tab 切换哨兵 — 触发 onSave 提示 (mock)
  const handleSave = () => {
    // P3 缺口: 不发起请求
  };

  return (
    <div className="max-w-4xl mx-auto" data-testid="settings-page">
      <PageHeader
        title={t.pageTitles['/settings'].title}
        subtitle="tenant / identity / permission / role / integration / scm (5 tabs; submit endpoint P3 缺口)"
        icon={<Settings className="text-accent" size={20} />}
        count="5 tabs"
      />

      <Tabs items={TABS as TabItem[]} active={tab} onChange={(id) => setTab(id as SettingsTab)} />

      <div className="card" data-testid={`settings-panel-${tab}`}>
        <SectionTitle>
          {TABS.find((t) => t.id === tab)?.label}
        </SectionTitle>
        {tab === "profile" && (
          <SimpleForm
            onSave={handleSave}
            fields={[
              { key: "name",     label: "Display Name", value: "Ulysses",           placeholder: "Your name"       },
              { key: "email",    label: "Email",        value: "ulysses@mavis.local", placeholder: "you@org.com",   type: "email" },
              { key: "timezone", label: "Timezone",     value: "Asia/Tokyo (JST)",  placeholder: "TZ identifier"   },
            ]}
          />
        )}
        {tab === "account" && (
          <SimpleForm
            onSave={handleSave}
            fields={[
              { key: "username",  label: "Username",  value: "ulysses",        placeholder: "login handle" },
              { key: "current",   label: "Current Password", value: "",       placeholder: "********", type: "password" },
              { key: "newpass",   label: "New Password",      value: "",       placeholder: "********", type: "password" },
            ]}
          />
        )}
        {tab === "team" && (
          <SimpleForm
            onSave={handleSave}
            fields={[
              { key: "team",   label: "Team Name",  value: "Star Core",   placeholder: "team display name" },
              { key: "domain", label: "Domain",     value: "mavis.local", placeholder: "team slug" },
            ]}
          />
        )}
        {tab === "billing" && (
          <SimpleForm
            onSave={handleSave}
            fields={[
              { key: "plan",   label: "Plan",        value: "Pro (mock)",       placeholder: "Pro / Team / Ent" },
              { key: "card",   label: "Card Number", value: "**** **** **** 4242", placeholder: "16 digits" },
              { key: "exp",    label: "Expiry",      value: "12/29",            placeholder: "MM/YY" },
            ]}
          />
        )}
        {tab === "apikeys" && (
          <SimpleForm
            onSave={handleSave}
            fields={[
              { key: "label", label: "Key Label", value: "ci-bot", placeholder: "human-readable name" },
              { key: "scope", label: "Scope",     value: "read:issues, write:comments", placeholder: "comma-separated scopes" },
              { key: "token", label: "Token (P3: not actually encrypted)", value: "", placeholder: "paste or generate", type: "password" },
            ]}
          />
        )}
      </div>

      <div className="card mt-3 text-xs text-ink-dim">
        <SectionTitle>Submit Endpoint — P3 缺口</SectionTitle>
        <ul className="space-y-1.5 list-disc pl-4 text-ink-dim">
          <li>5 tab 所有 Save 按钮仅本地 state, 不发起 <span className="font-mono text-ink-mute">/api/settings/*</span> 请求</li>
          <li>API Keys: 实际加密存储 (KMS / vault) 待 P3</li>
          <li>缺失 tab: Workspace / Members / Permissions / Runtimes / Skills (P2)</li>
          <li>left sidebar 嵌套导航 (per §5.6) P2</li>
        </ul>
      </div>
    </div>
  );
}
