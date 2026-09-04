// Star Frontend - Credentials API Client
// V2-2 完整版 - 集成 star-credential Rust backend (per PHASE-V2-2-IMPL-REPORT.md)
// per 守门 #5 (env 安全): secret 永远不入 log
// per 守门 #14 (5 域 Lead CONTENT 4 维): Mavis 临时代签

const API_BASE = "/api/v2";

export type Provider =
  | "openclaw"
  | "hermes"
  | "kms_vault"
  | "kms_aws"
  | "kms_local_mock";

export type CredentialStatus = "active" | "deprecated" | "revoked";

export interface CredentialView {
  id: string;
  provider: Provider;
  display_name: string;
  status: CredentialStatus;
  created_at_ms: number;
  updated_at_ms: number;
  deprecated_at_ms: number | null;
  revoked_at_ms: number | null;
}

export interface CreateCredentialRequest {
  provider: Provider;
  display_name: string;
  description: string;
  secret: string;
  base_url?: string;
  region?: string;
}

export interface RotateRequest {
  display_name: string;
  description: string;
  secret: string;
  base_url?: string;
  region?: string;
}

export interface AuditEventView {
  id: string;
  credential_id: string;
  user_id: string;
  event_type: "store" | "rotate" | "revoke" | "retrieve";
  event_at_ms: number;
  display_name_snapshot: string | null;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json", ...(init?.headers || {}) },
    ...init,
  });
  if (!res.ok) {
    const text = await res.text();
    // 守门 #5: 不打印 secret, 只打印 id
    throw new Error(`API ${res.status}: ${text || res.statusText}`);
  }
  if (res.status === 204) return undefined as T;
  return (await res.json()) as T;
}

export const credentialsApi = {
  /** 列出 tenant 凭证 (无密文) */
  list: (provider?: Provider) => {
    const q = provider ? `?provider=${encodeURIComponent(provider)}` : "";
    return request<CredentialView[]>(`/credentials${q}`);
  },

  /** 创建凭证 */
  create: (req: CreateCredentialRequest) =>
    request<CredentialView>(`/credentials`, {
      method: "POST",
      body: JSON.stringify(req),
    }),

  /** 轮换凭证 */
  rotate: (id: string, req: RotateRequest) =>
    request<CredentialView>(`/credentials/${id}/rotate`, {
      method: "POST",
      body: JSON.stringify(req),
    }),

  /** 撤销凭证 */
  revoke: (id: string) =>
    request<void>(`/credentials/${id}/revoke`, { method: "POST" }),

  /** 审计日志 */
  audit: (id: string) =>
    request<AuditEventView[]>(`/credentials/${id}/audit`),
};
