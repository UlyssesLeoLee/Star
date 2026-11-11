/*
@cypher schema=1 source_sha256=2026-11-11-auth-client-v1
MERGE (self:File {path:"frontend/src/lib/auth.ts"})
MERGE (store:Symbol {id:"auth_client::authTokenStore",kind:"module"})
MERGE (decodeJwt:Symbol {id:"auth_client::decodeJwtPayload",kind:"function"})
MERGE (isAdmin:Symbol {id:"auth_client::isAdmin",kind:"function"})
MERGE (api:Symbol {id:"auth_client::authApi",kind:"module"})
MERGE (self)-[:DEFINES]->(store)
MERGE (self)-[:DEFINES]->(decodeJwt)
MERGE (self)-[:DEFINES]->(isAdmin)
MERGE (self)-[:DEFINES]->(api)
@endcypher
*/
// Google 登录客户端 (per 2026-11-11 用户发令)
// 本地 JWT 存储 + claims 解码 (客户端只读解码, 验签由后端负责) + API 调用。

const TOKEN_KEY = "star.access_token";

export interface StarJwtPayload {
  sub: string;
  iss: string;
  aud: string;
  exp: number;
  iat: number;
  tenant_id: string;
  user_id: string;
  roles: string[];
  scope: string;
  email?: string;
}

export interface StarSession {
  token: string;
  payload: StarJwtPayload;
  isAdmin: boolean;
}

export function decodeJwtPayload(token: string): StarJwtPayload | null {
  try {
    const parts = token.split(".");
    if (parts.length !== 3) return null;
    const body = parts[1].replace(/-/g, "+").replace(/_/g, "/");
    const padded = body + "=".repeat((4 - (body.length % 4)) % 4);
    const json = decodeURIComponent(
      Array.from(atob(padded))
        .map((c) => `%${c.charCodeAt(0).toString(16).padStart(2, "0")}`)
        .join(""),
    );
    return JSON.parse(json) as StarJwtPayload;
  } catch {
    return null;
  }
}

export function getSession(): StarSession | null {
  if (typeof window === "undefined") return null;
  const token = window.localStorage.getItem(TOKEN_KEY);
  if (!token) return null;
  const payload = decodeJwtPayload(token);
  if (!payload) return null;
  if (payload.exp * 1000 < Date.now()) {
    window.localStorage.removeItem(TOKEN_KEY);
    return null;
  }
  return {
    token,
    payload,
    isAdmin: payload.roles.includes("admin"),
  };
}

export function setSessionToken(token: string): StarSession | null {
  const payload = decodeJwtPayload(token);
  if (!payload) return null;
  window.localStorage.setItem(TOKEN_KEY, token);
  return { token, payload, isAdmin: payload.roles.includes("admin") };
}

export function clearSession(): void {
  window.localStorage.removeItem(TOKEN_KEY);
}

/** 后端同源调用 (next.config.js rewrites /api/v1/* → 本地 8081; Vercel 上后端不可达时 502, UI 优雅降级)。 */
async function apiGet<T>(path: string): Promise<{ status: number; body: T | null; error?: string }> {
  const session = getSession();
  const headers: Record<string, string> = { Accept: "application/json" };
  if (session) headers.Authorization = `Bearer ${session.token}`;
  try {
    const res = await fetch(path, { headers });
    if (!res.ok) {
      const text = await res.text();
      let code = `HTTP_${res.status}`;
      try {
        code = (JSON.parse(text) as { error?: string }).error ?? code;
      } catch {
        /* not json */
      }
      return { status: res.status, body: null, error: code };
    }
    return { status: res.status, body: (await res.json()) as T };
  } catch {
    return { status: 0, body: null, error: "NETWORK_UNREACHABLE" };
  }
}

export interface GoogleLoginResult {
  access_token: string;
  token_type: string;
  expires_in: number;
  user: { email: string; name?: string | null; is_admin: boolean; roles: string[] };
}

export interface AdminStatusBody {
  server_time: string;
  version: string;
  admin: { email: string; google_client_configured: boolean };
  auth: { provider: string; issuer: string; audience: string; token_ttl_seconds: number };
}

export interface MeBody {
  user_id: string;
  tenant_id: string;
  email?: string | null;
  roles: string[];
  is_admin: boolean;
}

export const authApi = {
  async loginWithGoogleIdToken(idToken: string): Promise<{ status: number; body: GoogleLoginResult | null; error?: string }> {
    try {
      const res = await fetch("/api/v1/auth/google", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ id_token: idToken }),
      });
      if (!res.ok) {
        let code = `HTTP_${res.status}`;
        try {
          code = (JSON.parse(await res.text()) as { error?: string }).error ?? code;
        } catch {
          /* not json */
        }
        return { status: res.status, body: null, error: code };
      }
      return { status: res.status, body: (await res.json()) as GoogleLoginResult };
    } catch {
      return { status: 0, body: null, error: "NETWORK_UNREACHABLE" };
    }
  },
  me: () => apiGet<MeBody>("/api/v1/auth/me"),
  adminStatus: () => apiGet<AdminStatusBody>("/api/v1/admin/status"),
};
