/**
 * AgentCredentialStore - per-agent AI 凭证管理 (per ADR-0051)
 *
 * 守门合规:
 * - 守门 #5: env_var passthrough, 不打印 value
 * - 守门 #13 c: per-agent 隔离 (per V2-1 crates/star-credential)
 * - 守门 #14 v2: Mavis 临时代签 admin 域
 *
 * 简化决策 (per brief §3):
 * - V2-1 KMS 加密调用跨 session 续
 * - localStorage 缓存 (per EX-05 IdempotencyManager 同模式)
 */

import {
  ALL_AGENT_IDS,
  credentialCategory,
  type AgentCredential,
  type AgentCredentialCreateInput,
  type AgentCredentialUpdateInput,
  type AgentId,
  type CredentialKind,
} from "./types";

/** 存储 key 前缀. */
const STORAGE_PREFIX = "star-agent-cred:";

/** AgentCredentialStore - per-agent AI 凭证 CRUD. */
export class AgentCredentialStore {
  private storageKey(agentId: AgentId): string {
    return `${STORAGE_PREFIX}${agentId}`;
  }

  /** 列某 agent 的所有凭证. */
  public list(agentId: AgentId): AgentCredential[] {
    const raw = localStorage.getItem(this.storageKey(agentId));
    if (!raw) return [];
    try {
      return JSON.parse(raw) as AgentCredential[];
    } catch {
      return [];
    }
  }

  /** 加新凭证 (per-agent). */
  public add(agentId: AgentId, input: AgentCredentialCreateInput): AgentCredential {
    const cred: AgentCredential = {
      id: crypto.randomUUID(),
      kind: input.kind,
      category: credentialCategory(input.kind),
      displayName: input.displayName,
      envKey: input.envKey,
      active: true,
      createdAt: Date.now(),
      lastUsedAt: null,
    };
    const creds = this.list(agentId);
    // dedup: 同一 agent 同 kind 仅 1 个 active
    for (const c of creds) {
      if (c.kind === input.kind && c.active) {
        c.active = false;
      }
    }
    creds.push(cred);
    this._save(agentId, creds);
    return cred;
  }

  /** 更新凭证. */
  public update(
    agentId: AgentId,
    id: string,
    input: AgentCredentialUpdateInput,
  ): AgentCredential | null {
    const creds = this.list(agentId);
    const idx = creds.findIndex((c) => c.id === id);
    if (idx === -1) return null;
    const updated = { ...creds[idx], ...input };
    creds[idx] = updated;
    this._save(agentId, creds);
    return updated;
  }

  /** 删凭证. */
  public remove(agentId: AgentId, id: string): boolean {
    const creds = this.list(agentId);
    const filtered = creds.filter((c) => c.id !== id);
    if (filtered.length === creds.length) return false;
    this._save(agentId, filtered);
    return true;
  }

  /** 激活某凭证 (per-agent). */
  public activate(agentId: AgentId, id: string): AgentCredential | null {
    const creds = this.list(agentId);
    const target = creds.find((c) => c.id === id);
    if (!target) return null;
    // 同 kind 仅 1 个 active
    for (const c of creds) {
      if (c.kind === target.kind) c.active = c.id === id;
    }
    this._save(agentId, creds);
    return creds.find((c) => c.id === id) ?? null;
  }

  /** 测用: 拿某 agent 的 active 凭证 (per kind). */
  public getActive(agentId: AgentId, kind: CredentialKind): AgentCredential | null {
    const creds = this.list(agentId);
    return creds.find((c) => c.kind === kind && c.active) ?? null;
  }

  /** 列所有 agent ID. */
  public static allAgentIds(): AgentId[] {
    return [...ALL_AGENT_IDS];
  }

  private _save(agentId: AgentId, creds: AgentCredential[]): void {
    localStorage.setItem(this.storageKey(agentId), JSON.stringify(creds));
  }
}

/** 单例 (per EX-05 IdempotencyManager 同模式). */
let _instance: AgentCredentialStore | null = null;

export function getAgentCredentialStore(): AgentCredentialStore {
  if (!_instance) {
    _instance = new AgentCredentialStore();
  }
  return _instance;
}
