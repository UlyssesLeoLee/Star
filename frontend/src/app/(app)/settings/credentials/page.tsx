"use client";

// Star Frontend - /settings/credentials 凭证管理页面
// V2-2 完整版 - React + React Query + Tailwind
// per PHASE-V2-2-IMPL-REPORT.md
// per 守门 #5: secret 不入 log, 仅在 form 内部 state

import { useState } from "react";
import { Key, Plus, Trash2, Lock, ShieldCheck, RefreshCw, Eye, EyeOff } from "lucide-react";
import { PageHeader, SectionTitle } from "@/components/PageHeader";
import {
  useCredentials,
  useCreateCredential,
  useRotateCredential,
  useRevokeCredential,
  useAuditLog,
  PROVIDER_LABELS,
} from "@/lib/hooks/use-credentials";
import { Provider } from "@/lib/api/credentials";

const PROVIDERS: Provider[] = ["openclaw", "hermes", "kms_vault", "kms_aws", "kms_local_mock"];

export default function CredentialsPage() {
  const { data: credentials = [], isLoading } = useCredentials();
  const createMut = useCreateCredential();
  const rotateMut = useRotateCredential();
  const revokeMut = useRevokeCredential();

  const [adding, setAdding] = useState(false);
  const [rotating, setRotating] = useState<string | null>(null);
  const [auditing, setAuditing] = useState<string | null>(null);
  const [reveal, setReveal] = useState<Record<string, boolean>>({});

  // Create form state
  const [form, setForm] = useState({
    provider: "openclaw" as Provider,
    display_name: "",
    description: "",
    secret: "",
    base_url: "",
  });

  // Rotate form state
  const [rotateForm, setRotateForm] = useState({ display_name: "", description: "", secret: "" });

  const onCreate = async () => {
    await createMut.mutateAsync({
      provider: form.provider,
      display_name: form.display_name,
      description: form.description,
      secret: form.secret,
      base_url: form.base_url || undefined,
    });
    setAdding(false);
    setForm({ provider: "openclaw", display_name: "", description: "", secret: "", base_url: "" });
  };

  const onRotate = async (id: string) => {
    await rotateMut.mutateAsync({
      id,
      req: {
        display_name: rotateForm.display_name,
        description: rotateForm.description,
        secret: rotateForm.secret,
      },
    });
    setRotating(null);
    setRotateForm({ display_name: "", description: "", secret: "" });
  };

  const onRevoke = async (id: string) => {
    if (!confirm("确定撤销该凭证？此操作不可撤销。")) return;
    await revokeMut.mutateAsync(id);
  };

  return (
    <div className="space-y-6">
      <PageHeader
        title="凭证管理 (Credentials)"
        description="用户在设置界面自行填入 OpenClaw / Hermes / KMS 凭证，后端 CredentialManager 加密存储 (per INV-CR-01~06)"
        actions={
          <button
            onClick={() => setAdding(true)}
            className="btn-primary-ghost flex items-center gap-1"
          >
            <Plus size={12} />
            新增凭证
          </button>
        }
      />

      <SectionTitle>5 种 Provider (V2-1 CredentialManager 落档)</SectionTitle>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        {PROVIDERS.map((p) => (
          <div
            key={p}
            className="p-3 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)]"
          >
            <div className="flex items-center gap-2 mb-1.5">
              <Lock size={14} className="text-[color:var(--color-success)]" />
              <span className="text-sm font-medium">{PROVIDER_LABELS[p]}</span>
            </div>
            <p className="text-[10px] text-[color:var(--color-text-dim)] leading-relaxed">
              {p === "kms_local_mock" ? "dev/test 模式，无真实加密" : "KMS 加密存储 (per tenant DEK envelope encryption)"}
            </p>
          </div>
        ))}
      </div>

      <SectionTitle>当前凭证列表</SectionTitle>
      {isLoading ? (
        <p className="text-sm text-[color:var(--color-text-dim)]">加载中…</p>
      ) : credentials.length === 0 ? (
        <p className="text-sm text-[color:var(--color-text-dim)]">尚未配置凭证,点击右上角 "新增凭证" 开始。</p>
      ) : (
        <div className="space-y-2">
          {credentials.map((c) => (
            <div
              key={c.id}
              className="p-3 rounded-md border border-[color:var(--color-border)] bg-[color:var(--color-surface)] flex items-center justify-between"
            >
              <div className="flex-1">
                <div className="flex items-center gap-2">
                  <Key size={14} />
                  <span className="text-sm font-medium">{c.display_name}</span>
                  <span className="text-[10px] px-1.5 py-0.5 rounded bg-[color:var(--color-surface-2)]">
                    {c.provider}
                  </span>
                  <span
                    className={`text-[10px] px-1.5 py-0.5 rounded ${
                      c.status === "active"
                        ? "bg-green-900/30 text-green-400"
                        : c.status === "deprecated"
                        ? "bg-yellow-900/30 text-yellow-400"
                        : "bg-red-900/30 text-red-400"
                    }`}
                  >
                    {c.status}
                  </span>
                </div>
                <p className="text-[10px] text-[color:var(--color-text-dim)] mt-1">
                  ID: {c.id} · 创建 {new Date(c.created_at_ms).toLocaleString()}
                </p>
              </div>
              <div className="flex items-center gap-1">
                <button
                  onClick={() => setAuditing(c.id)}
                  className="px-2 py-1 text-xs rounded hover:bg-[color:var(--color-surface-2)] flex items-center gap-1"
                >
                  <Eye size={12} /> 审计
                </button>
                <button
                  onClick={() => setRotating(c.id)}
                  disabled={c.status !== "active"}
                  className="px-2 py-1 text-xs rounded hover:bg-[color:var(--color-surface-2)] flex items-center gap-1 disabled:opacity-30"
                >
                  <RefreshCw size={12} /> 轮换
                </button>
                <button
                  onClick={() => onRevoke(c.id)}
                  disabled={c.status === "revoked"}
                  className="px-2 py-1 text-xs rounded hover:bg-red-900/30 text-red-400 flex items-center gap-1 disabled:opacity-30"
                >
                  <Trash2 size={12} /> 撤销
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create modal (inline) */}
      {adding && (
        <div className="p-4 rounded-md border border-[color:var(--color-primary)] bg-[color:var(--color-surface-2)] space-y-3">
          <h3 className="text-sm font-semibold">新增凭证</h3>
          <div className="grid grid-cols-2 gap-2">
            <select
              value={form.provider}
              onChange={(e) => setForm({ ...form, provider: e.target.value as Provider })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            >
              {PROVIDERS.map((p) => (
                <option key={p} value={p}>
                  {PROVIDER_LABELS[p]}
                </option>
              ))}
            </select>
            <input
              type="text"
              placeholder="显示名 (e.g. 我的 OpenClaw 账号)"
              value={form.display_name}
              onChange={(e) => setForm({ ...form, display_name: e.target.value })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
            <input
              type="text"
              placeholder="描述 (e.g. 用于 LangGraph sub-agent 派发)"
              value={form.description}
              onChange={(e) => setForm({ ...form, description: e.target.value })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
            <input
              type="text"
              placeholder="Base URL (可选, e.g. https://api.openclaw.example.com/v1)"
              value={form.base_url}
              onChange={(e) => setForm({ ...form, base_url: e.target.value })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
            <input
              type="password"
              placeholder="API Key / Token (TLS 加密传输, 后端 KMS 加密存储)"
              value={form.secret}
              onChange={(e) => setForm({ ...form, secret: e.target.value })}
              className="col-span-2 px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
          </div>
          <div className="flex gap-2">
            <button onClick={onCreate} disabled={createMut.isPending || !form.display_name || !form.secret} className="btn-primary-ghost">
              {createMut.isPending ? "提交中…" : "保存"}
            </button>
            <button onClick={() => setAdding(false)} className="px-3 py-1 text-sm rounded hover:bg-[color:var(--color-surface)]">
              取消
            </button>
          </div>
        </div>
      )}

      {/* Rotate modal */}
      {rotating && (
        <div className="p-4 rounded-md border border-[color:var(--color-warning)] bg-[color:var(--color-surface-2)] space-y-3">
          <h3 className="text-sm font-semibold">轮换凭证</h3>
          <p className="text-[10px] text-[color:var(--color-text-dim)]">原凭证将标 Deprecated, 新凭证 Active</p>
          <div className="grid grid-cols-2 gap-2">
            <input
              type="text"
              placeholder="新显示名"
              value={rotateForm.display_name}
              onChange={(e) => setRotateForm({ ...rotateForm, display_name: e.target.value })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
            <input
              type="text"
              placeholder="新描述"
              value={rotateForm.description}
              onChange={(e) => setRotateForm({ ...rotateForm, description: e.target.value })}
              className="px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
            <input
              type="password"
              placeholder="新 Secret"
              value={rotateForm.secret}
              onChange={(e) => setRotateForm({ ...rotateForm, secret: e.target.value })}
              className="col-span-2 px-2 py-1 text-sm rounded bg-[color:var(--color-surface)] border border-[color:var(--color-border)]"
            />
          </div>
          <div className="flex gap-2">
            <button onClick={() => onRotate(rotating)} disabled={rotateMut.isPending || !rotateForm.secret} className="btn-primary-ghost">
              {rotateMut.isPending ? "提交中…" : "确认轮换"}
            </button>
            <button onClick={() => setRotating(null)} className="px-3 py-1 text-sm rounded hover:bg-[color:var(--color-surface)]">
              取消
            </button>
          </div>
        </div>
      )}

      {/* Audit log modal */}
      {auditing && <AuditLogView credentialId={auditing} onClose={() => setAuditing(null)} />}
    </div>
  );
}

function AuditLogView({ credentialId, onClose }: { credentialId: string; onClose: () => void }) {
  const { data: events = [], isLoading } = useAuditLog(credentialId);
  return (
    <div className="p-4 rounded-md border border-[color:var(--color-info)] bg-[color:var(--color-surface-2)] space-y-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-semibold flex items-center gap-2">
          <ShieldCheck size={14} /> 审计日志 (4 事件: store / rotate / revoke / retrieve)
        </h3>
        <button onClick={onClose} className="text-xs px-2 py-1 rounded hover:bg-[color:var(--color-surface)]">关闭</button>
      </div>
      {isLoading ? (
        <p className="text-xs text-[color:var(--color-text-dim)]">加载中…</p>
      ) : events.length === 0 ? (
        <p className="text-xs text-[color:var(--color-text-dim)]">无审计事件</p>
      ) : (
        <ul className="space-y-1 text-xs">
          {events.map((e) => (
            <li key={e.id} className="flex items-center justify-between border-b border-[color:var(--color-border)] py-1">
              <span className="font-mono">{e.event_type}</span>
              <span className="text-[color:var(--color-text-dim)]">{new Date(e.event_at_ms).toLocaleString()}</span>
              <span className="text-[color:var(--color-text-dim)]">by {e.user_id}</span>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
