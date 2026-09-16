"use client";

// =====================================================================
// AgentSettingsModal — Agent LLM API Key 设置弹窗 (per 2026-09-02 02:49 JST Ulysses 拍板)
// =====================================================================
// 触发:
//   - WindowsTabBar 每个 tab 旁边的齿轮按钮 (e.stopPropagation 防冒泡到 tab select)
//
// 职责:
//   1. 弹 modal, 显示当前 agent (CliTab) 的 LLM API key 状态
//   2. 用户选 provider (openai / claude / gemini / minimax 4 必备 + anthropic/hermes/openclaw 兼容)
//   3. 用户选 agent 维度 (cli_profile_id + agent_kind + agent_id 都可选)
//   4. 选存储模式 (encrypted_rust AES-256-GCM / environment_var 沿用现有 /settings/api-keys)
//   5. POST /api/api-keys 提交 (走现有 endpoint + body 扩 3 字段)
//   6. 列已配置 keys (per agent), reveal toggle, delete
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖 (复用 Lucide icons + useState)
//   - SSR-safe: portal + typeof window check
//   - 走 React Query (per @tanstack/react-query 既有, 不再扩)
//   - 13 类 tenant_id 必带 (per REQ-SEC-001, 现有 /settings/api-keys 已走, 沿用)
//
// 已知缺口 (per 缺标比错标, 守門 #11):
//   1. Edit 现有 key (Phase 1 只支持 Add + Delete, 改 key 需 delete + re-add)
//   2. 真实 key 校验 (mock 直接返 success, 真后端 401 走 audit)
//   3. Test connection 按钮 (per key "ping" provider 校验有效性) — Phase 2+
//   4. Provider 默认 base_url 配置 (OpenAI-compatible 自定义 endpoint) — Phase 2+
//   5. minimax 没有具体 spec 文档, 4 必备之一先用通用 Bearer 格式占位 — Phase 2+ 跟 provider 实装
// =====================================================================

import { useEffect, useState, useCallback } from "react";
import { createPortal } from "react-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { clsx } from "clsx";
import {
  X, Key, Plus, Trash2, Eye, EyeOff, Lock, Globe,
  AlertTriangle, Loader2, ShieldCheck, CheckCircle2, RefreshCw,
} from "lucide-react";
import type { CliTab } from "./WindowsTabBar";
import type { ApiKey } from "@/mocks/schemas/cli";

// ---- types ----
export interface AgentSettingsModalProps {
  open: boolean;
  onClose: () => void;
  /** 当前 agent tab (齿轮点击的 tab) */
  tab: CliTab;
  /** tenant_id (13 類必带, per REQ-SEC-001, 从 ActorContext / worktree 推) */
  tenantId?: string;
}

type Provider = ApiKey["provider"];
type StoreMode = "encrypted_rust" | "environment_var";

// 4 必备 provider (per 2026-09-02 02:49 JST 拍板)
const REQUIRED_PROVIDERS: Provider[] = ["openai", "claude", "gemini", "minimax"];
// 完整 8 provider 列表
const ALL_PROVIDERS: Provider[] = [
  "openai", "claude", "gemini", "minimax",      // 4 必备
  "anthropic", "openclaw", "hermes", "google",  // 兼容旧
];
const PROVIDER_LABEL: Record<Provider, string> = {
  openai:    "OpenAI (GPT-4o / o1 / o3)",
  claude:    "Anthropic Claude (3.5 / 4 Sonnet/Opus)",
  gemini:    "Google Gemini (2.0 Flash / Pro)",
  minimax:   "minimax (m2.7 / m-large)",
  anthropic: "Anthropic (legacy alias)",
  openclaw:  "OpenClaw (CLI tool, not LLM)",
  hermes:    "Hermes (CLI tool, not LLM)",
  google:    "Google AI (legacy alias)",
};
const PROVIDER_DEFAULT_ENV: Partial<Record<Provider, string>> = {
  openai:    "OPENAI_API_KEY",
  claude:    "ANTHROPIC_API_KEY",
  gemini:    "GOOGLE_API_KEY",
  minimax:   "minimax_API_KEY",
  anthropic: "ANTHROPIC_API_KEY",
  openclaw:  "OPENCLAW_API_KEY",
  hermes:    "HERMES_API_KEY",
  google:    "GOOGLE_API_KEY",
};

// =====================================================================
// Main component
// =====================================================================
export function AgentSettingsModal({ open, onClose, tab, tenantId = "tenant-physis-corp" }: AgentSettingsModalProps) {
  // ---- form state ----
  const [adding, setAdding] = useState(false);
  const [form, setForm] = useState<{
    provider: Provider;
    label: string;
    mode: StoreMode;
    secret: string;
    envVarName: string;
  }>({
    provider: "claude",
    label: "",
    mode: "encrypted_rust",
    secret: "",
    envVarName: PROVIDER_DEFAULT_ENV.claude || "",
  });
  const [error, setError] = useState<string | null>(null);
  const [reveal, setReveal] = useState<Record<string, boolean>>({});

  // ---- 1. 列已配置 keys (per agent + cli_profile 维度, 走 React Query 缓存) ----
  const keysQ = useQuery<ApiKey[]>({
    queryKey: ["api-keys", "by-agent", tab.id, tab.profileName],
    queryFn: async () => {
      const res = await fetch("/api/api-keys");
      if (!res.ok) throw new Error(`fetch /api/api-keys failed: ${res.status}`);
      const all = (await res.json()) as ApiKey[];
      // 过滤: 同 cli_profile + agent_id (Phase 1 mock 简化: 按 profileName 匹配)
      return all.filter((k) =>
        (k.agent_id === tab.id) ||
        (k.cli_profile_id === tab.profileName) ||
        // 没绑定的也展示 (全局)
        (!k.agent_id && !k.cli_profile_id),
      );
    },
    enabled: open,
    staleTime: 30_000,
  });

  // ---- 2. 新增 key mutation ----
  const qc = useQueryClient();
  const addMutation = useMutation<ApiKey, Error, ApiKey>({
    mutationFn: async (key) => {
      const res = await fetch("/api/api-keys", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(key),
      });
      if (!res.ok) {
        const body = await res.json().catch(() => ({}));
        throw new Error(`add api-key failed: ${res.status} ${JSON.stringify(body)}`);
      }
      return res.json();
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["api-keys"] });
    },
  });

  // ---- 3. 删除 key mutation ----
  const deleteMutation = useMutation<{ deleted: true }, Error, string>({
    mutationFn: async (id) => {
      const res = await fetch(`/api/api-keys/${id}`, { method: "DELETE" });
      if (!res.ok) throw new Error(`delete api-key failed: ${res.status}`);
      return res.json();
    },
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ["api-keys"] });
    },
  });

  // ---- 4. 关闭时清 state ----
  useEffect(() => {
    if (!open) {
      setAdding(false);
      setError(null);
      setReveal({});
    }
  }, [open]);

  // ---- 5. Esc 关闭 ----
  useEffect(() => {
    if (!open) return;
    const h = (e: KeyboardEvent) => { if (e.key === "Escape") onClose(); };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
  }, [open, onClose]);

  // ---- 6. 阻止背景滚动 ----
  useEffect(() => {
    if (!open) return;
    const prev = document.body.style.overflow;
    document.body.style.overflow = "hidden";
    return () => { document.body.style.overflow = prev; };
  }, [open]);

  // ---- 7. submit ----
  const handleAdd = useCallback(async () => {
    setError(null);
    if (!form.label.trim()) {
      setError("Label 必填 (e.g. Primary / Backup / Staging)");
      return;
    }
    if (form.mode === "encrypted_rust" && !form.secret.trim()) {
      setError("API Key 明文必填 (encrypted_rust 模式)");
      return;
    }
    if (form.mode === "environment_var" && !form.envVarName.trim()) {
      setError("环境变量名必填 (environment_var 模式)");
      return;
    }
    const preview = form.mode === "encrypted_rust"
      ? `${form.secret.slice(0, 3)}-***${form.secret.slice(-3)}`
      : `env: ${form.envVarName}`;
    const newKey: ApiKey = {
      id: `k_${Date.now()}`,
      provider: form.provider,
      label: form.label,
      mode: form.mode,
      preview,
      envVarName: form.mode === "environment_var" ? form.envVarName : undefined,
      createdAt: new Date().toISOString().slice(0, 10),
      // 3 关联字段 (per 2026-09-02 02:49 JST 拍板: CLI profile + agent_kind 都可选)
      agent_id: tab.id,
      cli_profile_id: tab.profileName,
      agent_kind: inferAgentKindFromProfile(tab.profileName),
    };
    try {
      await addMutation.mutateAsync(newKey);
      setAdding(false);
      setForm((f) => ({ ...f, label: "", secret: "", envVarName: PROVIDER_DEFAULT_ENV[f.provider] || "" }));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [form, tab, addMutation]);

  // ---- 8. render ----
  if (!open || typeof window === "undefined") return null;

  const keys = keysQ.data || [];
  const isProviderRequired = (p: Provider) => REQUIRED_PROVIDERS.includes(p);

  return createPortal(
    <div
      role="dialog"
      aria-modal="true"
      aria-label={`Agent settings: ${tab.label}`}
      data-testid="agent-settings-modal"
      onClick={(e) => { if (e.target === e.currentTarget) onClose(); }}
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    >
      <div
        data-testid="agent-settings-modal-content"
        className="card flex flex-col shadow-2xl w-[min(720px,90vw)] max-h-[min(640px,90vh)]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between gap-2 px-4 py-2.5 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <Key size={14} className="text-[color:var(--color-primary)] shrink-0" />
            <span className="font-mono text-xs text-[color:var(--color-info)] truncate">{tab.label}</span>
            <span className="text-[10px] text-[color:var(--color-text-dim)] font-mono">
              · tab_id = {tab.id} · profile = {tab.profileName}
            </span>
          </div>
          <button
            type="button"
            data-testid="agent-settings-close"
            onClick={onClose}
            aria-label="Close"
            className="text-[color:var(--color-text-dim)] hover:text-[color:var(--color-text)] transition-colors p-1"
          >
            <X size={14} />
          </button>
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {/* 4 必备提示 */}
          <div className="rounded-md border border-[color:var(--color-info)]/30 bg-[color:var(--color-info)]/5 p-3 flex items-start gap-2 text-xs">
            <ShieldCheck size={14} className="text-[color:var(--color-info)] flex-shrink-0 mt-0.5" />
            <div>
              <div className="font-medium mb-1">LLM API Key 设置</div>
              <div className="text-[10px] text-[color:var(--color-text-dim)] space-y-0.5">
                <div>per 2026-09-02 02:49 JST 拍板: <strong>4 必备 provider</strong> = openai / claude / gemini / minimax</div>
                <div>每 agent 分别填不同 key, 跟 CLI profile + agent_kind 双重绑定 (均可选)</div>
                <div>存储双模式: 🔒 Encrypted (AES-256-GCM) / 🌐 Env Var</div>
              </div>
            </div>
          </div>

          {/* 列已配置 keys */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <div className="text-xs font-semibold uppercase tracking-wider text-[color:var(--color-text-dim)]">
                已配置 Keys ({keys.length})
              </div>
              <div className="flex items-center gap-1">
                <button
                  type="button"
                  onClick={() => keysQ.refetch()}
                  className="text-[10px] px-2 py-1 rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface)]"
                  aria-label="refresh"
                >
                  <RefreshCw size={10} />
                </button>
                <button
                  type="button"
                  data-testid="agent-settings-add"
                  onClick={() => setAdding(true)}
                  className="text-[11px] px-2 py-1 rounded btn-primary-ghost flex items-center gap-1"
                >
                  <Plus size={10} />
                  新 API Key
                </button>
              </div>
            </div>

            {keysQ.isLoading ? (
              <div className="flex items-center justify-center py-6 text-xs text-[color:var(--color-text-dim)] gap-2">
                <Loader2 size={12} className="animate-spin" />
                Loading...
              </div>
            ) : keys.length === 0 ? (
              <div className="rounded-md border border-dashed border-[color:var(--color-border)] p-6 text-center text-[11px] text-[color:var(--color-text-dim)]">
                暂无 API Key。点击右上角"新 API Key"创建。
              </div>
            ) : (
              <div className="space-y-1.5">
                {keys.map((k) => (
                  <div
                    key={k.id}
                    data-testid={`agent-settings-key-${k.id}`}
                    className="flex items-center gap-2 p-2 rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)]"
                  >
                    {k.mode === "encrypted_rust" ? (
                      <Lock size={11} className="text-[color:var(--color-success)] shrink-0" />
                    ) : (
                      <Globe size={11} className="text-[color:var(--color-info)] shrink-0" />
                    )}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-1.5">
                        <span className="text-xs font-medium">{k.label}</span>
                        <span className={clsx(
                          "text-[9px] px-1 rounded font-mono",
                          isProviderRequired(k.provider)
                            ? "bg-[color:var(--color-primary)]/15 text-[color:var(--color-primary)] border border-[color:var(--color-primary)]/30"
                            : "bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]",
                        )}>
                          {k.provider}{isProviderRequired(k.provider) ? " · 必备" : ""}
                        </span>
                        {k.agent_id && (
                          <span className="text-[9px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)] font-mono">
                            agent={k.agent_id}
                          </span>
                        )}
                        {k.cli_profile_id && (
                          <span className="text-[9px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)] font-mono">
                            profile={k.cli_profile_id}
                          </span>
                        )}
                        {k.agent_kind && (
                          <span className="text-[9px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)] font-mono">
                            kind={k.agent_kind}
                          </span>
                        )}
                      </div>
                      <div className="font-mono text-[10px] text-[color:var(--color-text-dim)] mt-0.5 truncate">
                        {k.mode === "environment_var"
                          ? `env: ${k.envVarName}`
                          : (reveal[k.id] ? `${k.preview.slice(0,3)}...${k.preview.slice(-4)} [encrypted]` : k.preview)}
                      </div>
                    </div>
                    <div className="flex items-center gap-0.5 shrink-0">
                      {k.mode === "encrypted_rust" && (
                        <button
                          type="button"
                          onClick={() => setReveal((p) => ({ ...p, [k.id]: !p[k.id] }))}
                          className="opacity-60 hover:opacity-100 p-0.5"
                          aria-label="reveal"
                        >
                          {reveal[k.id] ? <EyeOff size={10} /> : <Eye size={10} />}
                        </button>
                      )}
                      <button
                        type="button"
                        data-testid={`agent-settings-delete-${k.id}`}
                        onClick={() => deleteMutation.mutate(k.id)}
                        className="opacity-60 hover:opacity-100 text-[color:var(--color-danger)] p-0.5"
                        aria-label="delete"
                      >
                        <Trash2 size={10} />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            )}

            {keysQ.isError && (
              <div className="text-[10px] text-[color:var(--color-danger)] mt-1">
                ⚠️ 加载失败: {String(keysQ.error)}
              </div>
            )}
          </div>

          {/* Add form */}
          {adding && (
            <div
              data-testid="agent-settings-form"
              className="p-3 rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface-2)] space-y-2"
            >
              <div className="text-xs font-semibold mb-1">添加 API Key</div>

              {/* Provider 选择 (8 选, 4 必备高亮) */}
              <div>
                <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">Provider</label>
                <div className="mt-1 grid grid-cols-2 gap-1.5">
                  {ALL_PROVIDERS.map((p) => {
                    const active = form.provider === p;
                    const required = isProviderRequired(p);
                    return (
                      <button
                        key={p}
                        type="button"
                        data-testid={`agent-settings-provider-${p}`}
                        onClick={() => setForm((f) => ({
                          ...f,
                          provider: p,
                          envVarName: PROVIDER_DEFAULT_ENV[p] || "",
                        }))}
                        className={clsx(
                          "text-[11px] px-2 py-1.5 rounded border text-left flex items-center gap-1.5 transition-colors",
                          active
                            ? "border-[color:var(--color-primary)] bg-[color:var(--color-primary)]/10 text-[color:var(--color-text)]"
                            : "border-[color:var(--color-border)] text-[color:var(--color-text-dim)] hover:border-[color:var(--color-primary)]/40",
                        )}
                        title={PROVIDER_LABEL[p]}
                      >
                        {active && <CheckCircle2 size={10} className="text-[color:var(--color-primary)]" />}
                        <span className="font-mono">{p}</span>
                        {required && (
                          <span className="text-[8px] px-1 rounded bg-[color:var(--color-primary)]/20 text-[color:var(--color-primary)]">
                            必备
                          </span>
                        )}
                        {!required && (
                          <span className="text-[8px] px-1 rounded bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]">
                            兼容
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Label */}
              <div>
                <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">Label</label>
                <input
                  data-testid="agent-settings-label"
                  value={form.label}
                  onChange={(e) => setForm((f) => ({ ...f, label: e.target.value }))}
                  placeholder="Primary / Backup / Staging"
                  className="w-full mt-1 text-xs rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5"
                />
              </div>

              {/* 存储模式 */}
              <div>
                <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">存储模式</label>
                <div className="mt-1 flex gap-1.5">
                  {(["encrypted_rust", "environment_var"] as const).map((m) => {
                    const active = form.mode === m;
                    return (
                      <button
                        key={m}
                        type="button"
                        data-testid={`agent-settings-mode-${m}`}
                        onClick={() => setForm((f) => ({ ...f, mode: m }))}
                        className={clsx(
                          "flex-1 text-[11px] px-2 py-1.5 rounded border flex items-center justify-center gap-1.5",
                          active
                            ? "border-[color:var(--color-primary)] bg-[color:var(--color-primary)]/10"
                            : "border-[color:var(--color-border)] text-[color:var(--color-text-dim)]",
                        )}
                      >
                        {m === "encrypted_rust" ? <><Lock size={10} /> Encrypted (Rust AES-256-GCM)</> : <><Globe size={10} /> Environment Variable</>}
                      </button>
                    );
                  })}
                </div>
              </div>

              {/* Secret / Env 输入 */}
              {form.mode === "encrypted_rust" ? (
                <div>
                  <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">API Key (明文, 仅输入时显示)</label>
                  <input
                    data-testid="agent-settings-secret"
                    type="password"
                    value={form.secret}
                    onChange={(e) => setForm((f) => ({ ...f, secret: e.target.value }))}
                    placeholder={PROVIDER_DEFAULT_ENV[form.provider] ? `${PROVIDER_DEFAULT_ENV[form.provider]} = ...` : "sk-..."}
                    className="w-full mt-1 text-xs rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5 font-mono"
                  />
                </div>
              ) : (
                <div>
                  <label className="text-[10px] text-[color:var(--color-text-dim)] uppercase tracking-wider">环境变量名</label>
                  <input
                    data-testid="agent-settings-envvar"
                    value={form.envVarName}
                    onChange={(e) => setForm((f) => ({ ...f, envVarName: e.target.value }))}
                    placeholder={PROVIDER_DEFAULT_ENV[form.provider] || "API_KEY"}
                    className="w-full mt-1 text-xs rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] px-2 py-1.5 font-mono"
                  />
                  <div className="text-[10px] text-[color:var(--color-warning)] mt-1">
                    ⚠️ 该 env 变量必须在后端进程环境中存在, 否则调用会失败
                  </div>
                </div>
              )}

              {error && (
                <div data-testid="agent-settings-error" className="text-[10px] text-[color:var(--color-danger)] flex items-center gap-1">
                  <AlertTriangle size={10} />
                  {error}
                </div>
              )}

              <div className="flex justify-end gap-1.5 pt-1">
                <button
                  type="button"
                  onClick={() => { setAdding(false); setError(null); }}
                  className="text-[11px] px-2 py-1 rounded border border-[color:var(--color-border)]"
                >
                  取消
                </button>
                <button
                  type="button"
                  data-testid="agent-settings-submit"
                  onClick={handleAdd}
                  disabled={addMutation.isPending}
                  className="btn-primary text-[11px] px-3 py-1 flex items-center gap-1"
                >
                  {addMutation.isPending ? <><Loader2 size={10} className="animate-spin" /> 保存中...</> : "保存"}
                </button>
              </div>
            </div>
          )}

          {/* 底部说明 */}
          <div className="text-[10px] text-[color:var(--color-text-dim)] space-y-0.5 border-t border-[color:var(--color-border)] pt-2">
            <div>· 13 類 tenant_id 必带 (per REQ-SEC-001), Modal 默认 tenant = <span className="font-mono">{tenantId}</span></div>
            <div>· 所有 key 访问走 audit log (per REQ-AUDIT-002, 17 问)</div>
            <div>· Encrypted 模式: 后端 domain-cli 用 AES-256-GCM 加密, master key 来自配置 (Phase 2 接 KMS)</div>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}

// ---- helper ----
/** 从 CLI profile name 推断 agent_kind (per types/ids.ts AgentSession.agent_kind) */
function inferAgentKindFromProfile(profileName: string): ApiKey["agent_kind"] {
  const p = profileName.toLowerCase();
  if (p.includes("claude") || p === "anthropic") return "claude-sonnet";
  if (p.includes("gpt") || p === "codex" || p === "openai") return "gpt-4o";
  if (p.includes("gemini") || p === "google") return "gemini-2";
  if (p.includes("minimax")) return "minimax-v1";
  return "internal-vibe-coder";
}
