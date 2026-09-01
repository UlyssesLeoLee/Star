"use client";

// Star Frontend — API Keys 设置页
// Per 2026-08-29 09:07 JST 用户拍板: 双模式 (EncryptedRust 后端 + EnvironmentVar 环境变量)
// 入口: TopBar 用户菜单 → API Keys

import { useState } from "react";
import { Key, Plus, Trash2, Eye, EyeOff, Lock, Globe, AlertCircle, ShieldCheck } from "lucide-react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";

interface ApiKey {
  id: string;
  provider: string;
  label: string;
  mode: "encrypted_rust" | "environment_var";
  preview: string; // masked, e.g. "sk-***abc"
  envVarName?: string;
  createdAt: string;
  lastUsedAt?: string;
}

const INITIAL: ApiKey[] = [
  { id: "1", provider: "anthropic", label: "Primary", mode: "encrypted_rust", preview: "sk-***xyz1", createdAt: "2026-08-15" },
  { id: "2", provider: "openai", label: "Backup", mode: "environment_var", preview: "env: OPENAI_API_KEY", envVarName: "OPENAI_API_KEY", createdAt: "2026-08-20" },
];

const PROVIDER_HINT: Record<string, string> = {
  // per 2026-09-02 02:49 JST Ulysses 拍板: 4 必备 = openai / claude / gemini / minimax
  openai:    "OpenAI (GPT-4o / o1) — 4 必备",
  claude:    "Anthropic Claude (3.5 / 4 Sonnet) — 4 必备",
  gemini:    "Google Gemini (2.0 Flash / Pro) — 4 必备",
  minimax:   "minimax (m2.7 / m-large) — 4 必备",
  // 兼容旧 provider
  anthropic: "Anthropic (legacy alias)",
  openclaw:  "OpenClaw API (CLI tool)",
  hermes:    "Hermes AI (CLI tool)",
  google:    "Google AI (legacy alias)",
};

export default function ApiKeysPage() {
  const [keys, setKeys] = useState<ApiKey[]>(INITIAL);
  const [adding, setAdding] = useState(false);
  const [reveal, setReveal] = useState<Record<string, boolean>>({});
  const [form, setForm] = useState({
    provider: "anthropic",
    label: "",
    mode: "encrypted_rust" as "encrypted_rust" | "environment_var",
    secret: "",
    envVarName: "",
  });

  const toggleReveal = (id: string) => setReveal((p) => ({ ...p, [id]: !p[id] }));

  const addKey = () => {
    const preview = form.mode === "encrypted_rust"
      ? `${form.secret.slice(0, 3)}-***${form.secret.slice(-3)}`
      : `env: ${form.envVarName}`;
    setKeys((prev) => [
      ...prev,
      {
        id: String(Date.now()),
        provider: form.provider,
        label: form.label || "Default",
        mode: form.mode,
        preview,
        envVarName: form.mode === "environment_var" ? form.envVarName : undefined,
        createdAt: new Date().toISOString().slice(0, 10),
      },
    ]);
    setAdding(false);
    setForm({ provider: "anthropic", label: "", mode: "encrypted_rust", secret: "", envVarName: "" });
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="API Keys"
        description="双模式存储: 后端 AES-256-GCM 加密 (Encrypted) 或 环境变量 (Env Var)"
        actions={
          <button
            onClick={() => setAdding(true)}
            className="btn-primary-ghost flex items-center gap-1"
          >
            <Plus size={12} />
            新 API Key
          </button>
        }
      />

      {/* 存储模式说明 */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div className="p-3 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)]">
          <div className="flex items-center gap-2 mb-1.5">
            <Lock size={14} className="text-[color:var(--color-success)]" />
            <span className="text-sm font-medium">Encrypted (Rust backend)</span>
          </div>
          <p className="text-[10px] text-[color:var(--color-text-dim)] leading-relaxed">
            AES-256-GCM 加密, 存于后端 <code className="px-1 rounded bg-[color:var(--color-surface-2)]">domain-cli</code> 内存, 跨设备同步。接口仅返回 key_id + label, 不返明文。
          </p>
        </div>
        <div className="p-3 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)]">
          <div className="flex items-center gap-2 mb-1.5">
            <Globe size={14} className="text-[color:var(--color-info)]" />
            <span className="text-sm font-medium">Environment Variable</span>
          </div>
          <p className="text-[10px] text-[color:var(--color-text-dim)] leading-relaxed">
            不存后端, 启动时从 process env 读。最安全 (不进内存), 但只能读本地或容器内的 env, 不能跨设备。
          </p>
        </div>
      </div>

      {/* Add Form */}
      {adding && (
        <div className="p-4 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
          <div className="text-sm font-medium mb-3">添加 API Key</div>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div>
              <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">Provider</label>
              <select
                value={form.provider}
                onChange={(e) => setForm((f) => ({ ...f, provider: e.target.value }))}
                className="w-full mt-1 text-sm rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5"
              >
                {Object.entries(PROVIDER_HINT).map(([k, v]) => (
                  <option key={k} value={k}>{k} — {v}</option>
                ))}
              </select>
            </div>
            <div>
              <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">Label</label>
              <input
                value={form.label}
                onChange={(e) => setForm((f) => ({ ...f, label: e.target.value }))}
                placeholder="Primary / Backup / Staging"
                className="w-full mt-1 text-sm rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5"
              />
            </div>
            <div className="md:col-span-2">
              <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">存储模式</label>
              <div className="mt-1 flex gap-2">
                {(["encrypted_rust", "environment_var"] as const).map((m) => (
                  <button
                    key={m}
                    onClick={() => setForm((f) => ({ ...f, mode: m }))}
                    className={`flex-1 text-xs px-3 py-2 rounded border ${
                      form.mode === m
                        ? "btn-primary-ghost"
                        : "border-[color:var(--color-border)] text-[color:var(--color-text-dim)]"
                    }`}
                  >
                    {m === "encrypted_rust" ? "🔒 Encrypted (Rust backend)" : "🌐 Environment Variable"}
                  </button>
                ))}
              </div>
            </div>
            {form.mode === "encrypted_rust" ? (
              <div className="md:col-span-2">
                <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">API Key (明文, 仅输入时显示)</label>
                <input
                  type="password"
                  value={form.secret}
                  onChange={(e) => setForm((f) => ({ ...f, secret: e.target.value }))}
                  placeholder="sk-..."
                  className="w-full mt-1 text-sm rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5 font-mono"
                />
              </div>
            ) : (
              <div className="md:col-span-2">
                <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">环境变量名</label>
                <input
                  value={form.envVarName}
                  onChange={(e) => setForm((f) => ({ ...f, envVarName: e.target.value }))}
                  placeholder="ANTHROPIC_API_KEY"
                  className="w-full mt-1 text-sm rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5 font-mono"
                />
                <div className="text-[10px] text-[color:var(--color-warning)] mt-1">
                  ⚠️ 该 env 变量必须在后端进程环境中存在, 否则调用会失败
                </div>
              </div>
            )}
          </div>
          <div className="flex justify-end gap-2 mt-3">
            <button onClick={() => setAdding(false)} className="text-xs px-3 py-1.5 rounded border border-[color:var(--color-border)]">取消</button>
            <button
              onClick={addKey}
              disabled={!form.label || (form.mode === "encrypted_rust" ? !form.secret : !form.envVarName)}
              className="btn-primary"
            >
              保存
            </button>
          </div>
        </div>
      )}

      {/* 现有 Keys */}
      <div>
        <SectionTitle>已配置 Keys ({keys.length})</SectionTitle>
        {keys.length === 0 ? (
          <div className="rounded-md border border-dashed border-[color:var(--color-border)] p-6 text-center text-sm text-[color:var(--color-text-dim)]">
            暂无 API Key。点击右上角"新 API Key"创建。
          </div>
        ) : (
          <div className="space-y-2">
            {keys.map((k) => (
              <div
                key={k.id}
                className="flex items-center gap-3 p-3 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)]"
              >
                {k.mode === "encrypted_rust" ? (
                  <Lock size={14} className="text-[color:var(--color-success)]" />
                ) : (
                  <Globe size={14} className="text-[color:var(--color-info)]" />
                )}
                <div className="flex-1">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium">{k.label}</span>
                    <span className="text-[10px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]">{k.provider}</span>
                    <span className="text-[10px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]">
                      {k.mode === "encrypted_rust" ? "Encrypted" : "Env Var"}
                    </span>
                  </div>
                  <div className="font-mono text-[10px] text-[color:var(--color-text-dim)] mt-0.5">
                    {k.mode === "environment_var" ? `env: ${k.envVarName}` : (reveal[k.id] ? `${k.preview.slice(0,3)}...${k.preview.slice(-4)} [encrypted]` : k.preview)}
                  </div>
                  <div className="text-[10px] text-[color:var(--color-text-dim)] mt-0.5">创建于 {k.createdAt}{k.lastUsedAt ? ` · 上次使用 ${k.lastUsedAt}` : ""}</div>
                </div>
                <div className="flex items-center gap-1">
                  {k.mode === "encrypted_rust" && (
                    <button onClick={() => toggleReveal(k.id)} className="opacity-60 hover:opacity-100" aria-label="reveal">
                      {reveal[k.id] ? <EyeOff size={12} /> : <Eye size={12} />}
                    </button>
                  )}
                  <button onClick={() => setKeys((prev) => prev.filter((x) => x.id !== k.id))} className="opacity-60 hover:opacity-100 text-[color:var(--color-danger)]" aria-label="delete">
                    <Trash2 size={12} />
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 安全提示 */}
      <div className="rounded-md border border-[color:var(--color-warning)]/30 bg-[color:var(--color-warning)]/5 p-3 flex items-start gap-2 text-xs">
        <ShieldCheck size={14} className="text-[color:var(--color-warning)] flex-shrink-0 mt-0.5" />
        <div>
          <div className="font-medium mb-1">安全说明</div>
          <ul className="text-[10px] text-[color:var(--color-text-dim)] space-y-0.5 list-disc list-inside">
            <li>Encrypted 模式: 后端 domain-cli 用 AES-256-GCM 加密, master key 来自配置 (Phase 2 接 KMS)</li>
            <li>Env Var 模式: 不存后端, 启动时从 process env 读, 不进任何存储</li>
            <li>前端接口仅返回 key_id + label, 永不复显明文 (除 Env Var 模式显示变量名)</li>
            <li>所有 API Key 访问走 audit log (domain-audit)</li>
          </ul>
        </div>
      </div>
    </div>
  );
}
