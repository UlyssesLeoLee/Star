/**
 * AI 凭证类型定义 (per ADR-0051 §2.1 凭证分类总表)
 *
 * 守门合规:
 * - 守门 #5: 不打印 value, 仅引用 key
 * - 守门 #13 c: per-agent 隔离 (per V2-1 crates/star-credential)
 * - 守门 #14 v2: Mavis 临时代签 admin 域
 */

export type CredentialKind =
  | "openai"
  | "claude"
  | "anthropic"
  | "google_gemini"
  | "mistral"
  | "github_copilot"
  | "cursor"
  | "openai_codex"
  | "claude_code"
  | "cline"
  | "tavily"
  | "brave"
  | "serpapi"
  | "openai_embedding"
  | "cohere"
  | "voyage"
  | "custom";

export type CredentialCategory = "llm" | "code_ai" | "search" | "embedding" | "custom";

export interface AgentCredential {
  /** 凭证 ID (per-agent 唯一). */
  id: string;
  /** 凭证种类. */
  kind: CredentialKind;
  /** 凭证分类. */
  category: CredentialCategory;
  /** 显示名 (per agent 私有). */
  displayName: string;
  /** 环境变量 key 名 (e.g. "OPENAI_API_KEY"). */
  envKey: string;
  /** 是否激活 (per-agent 启用/禁用). */
  active: boolean;
  /** 创建时间 (epoch ms). */
  createdAt: number;
  /** 最近使用时间 (epoch ms). */
  lastUsedAt: number | null;
}

export interface AgentCredentialCreateInput {
  kind: CredentialKind;
  displayName: string;
  envKey: string;
}

export interface AgentCredentialUpdateInput {
  displayName?: string;
  envKey?: string;
  active?: boolean;
}

/** Agent ID (per SA-XX 或 SA-10). */
export type AgentId =
  | "SA-01"
  | "SA-02"
  | "SA-03"
  | "SA-04"
  | "SA-05"
  | "SA-06"
  | "SA-07"
  | "SA-08"
  | "SA-09"
  | "SA-10";

export const ALL_AGENT_IDS: AgentId[] = [
  "SA-01", "SA-02", "SA-03", "SA-04", "SA-05",
  "SA-06", "SA-07", "SA-08", "SA-09", "SA-10",
];

/** 凭证种类 → 分类映射. */
export function credentialCategory(kind: CredentialKind): CredentialCategory {
  switch (kind) {
    case "openai":
    case "claude":
    case "anthropic":
    case "google_gemini":
    case "mistral":
      return "llm";
    case "github_copilot":
    case "cursor":
    case "openai_codex":
    case "claude_code":
    case "cline":
      return "code_ai";
    case "tavily":
    case "brave":
    case "serpapi":
      return "search";
    case "openai_embedding":
    case "cohere":
    case "voyage":
      return "embedding";
    case "custom":
    default:
      return "custom";
  }
}
