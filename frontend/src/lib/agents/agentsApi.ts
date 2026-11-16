/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/agents/agentsApi.ts"})
MERGE (backend:ExternalService {name:"star-api-rest"})
MERGE (listAgents:Symbol {id:"frontend/src/lib/agents/agentsApi.ts::listAgents",kind:"function"})
MERGE (listArgoCli:Symbol {id:"frontend/src/lib/agents/agentsApi.ts::listArgoCli",kind:"function"})
MERGE (pingRuntimeHealth:Symbol {id:"frontend/src/lib/agents/agentsApi.ts::pingRuntimeHealth",kind:"function"})
MERGE (mapAgent:Symbol {id:"frontend/src/lib/agents/agentsApi.ts::mapAgent",kind:"function"})
MERGE (requestJson:Symbol {id:"frontend/src/lib/agents/agentsApi.ts::requestJson",kind:"function"})
MERGE (errClass:Type {id:"frontend/src/lib/agents/agentsApi.ts::AgentsApiError",kind:"class"})
MERGE (row:Type {id:"frontend/src/lib/agents/agentsApi.ts::AgentCenterRow",kind:"interface"})
MERGE (auth:File {path:"frontend/src/lib/auth.ts"})
MERGE (self)-[:CALLS_API]->(backend)
MERGE (self)-[:DEFINES]->(listAgents)
MERGE (self)-[:DEFINES]->(listArgoCli)
MERGE (self)-[:DEFINES]->(pingRuntimeHealth)
MERGE (self)-[:DEFINES]->(mapAgent)
MERGE (self)-[:DEFINES]->(requestJson)
MERGE (self)-[:DEFINES]->(errClass)
MERGE (self)-[:DEFINES]->(row)
MERGE (self)-[:IMPORTS]->(auth)
MERGE (listAgents)-[:CALLS]->(requestJson)
MERGE (listAgents)-[:CALLS]->(mapAgent)
MERGE (listArgoCli)-[:CALLS]->(requestJson)
MERGE (requestJson)-[:USES_TYPE]->(errClass)
MERGE (mapAgent)-[:USES_TYPE]->(row)
@endcypher
*/

// =====================================================================
// agentsApi.ts — 真实 Agent 运行时 API 封装 (per 2026-11-17 清 seed/mock 纪律)
//
// 数据源: 真实后端 star-api-rest (同源代理 /api/v1/* → 127.0.0.1:8081)。
// 绝不回落 MOCK_AGENTS 冒充真实数据; 后端不可达时抛 AgentsApiError,
// 页面层按 code 映射诚实三态 (未接入 / 未授权 / 失败)。
//
// REST 契约 (与 star-api-rest routes 对齐):
//   GET /api/v1/agents      → { data: { total:number, agents:RuntimeAgent[], note:string|null }, meta }
//       agent 字段 (serde 默认序列化, PascalCase 枚举原值):
//         id(uuid) / name / role(Worker|Supervisor|Reviewer) /
//         status(Running|Paused|Stopped|Completed|Failed|...) /
//         kind / domain / token_usage / token_budget / started_at(RFC3339)
//   GET /api/v1/argo/nodes  → { data: { supported_cli: string[] } }
//   GET /api/v1/health      → 200 (宿主运行时健康检查)
// =====================================================================

import { getSession } from "@/lib/auth";

const API_ROOT = "/api/v1";

export type AgentsApiErrorCode =
  | "network_error"
  | "http_error"
  | "unauthorized"
  | "parse_error";

export class AgentsApiError extends Error {
  readonly code: AgentsApiErrorCode;
  readonly status: number | null;

  constructor(message: string, code: AgentsApiErrorCode, status: number | null = null) {
    super(message);
    this.name = "AgentsApiError";
    this.code = code;
    this.status = status;
  }
}

function errText(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}

/** 带可选 Bearer 会话头 (auth.ts getSession, 无 token 时省略) 的同源请求 */
async function requestJson<T>(path: string): Promise<T> {
  const session = getSession();
  const headers: Record<string, string> = { Accept: "application/json" };
  if (session) headers.Authorization = `Bearer ${session.token}`;

  let res: Response;
  try {
    res = await fetch(`${API_ROOT}${path}`, { headers });
  } catch (err) {
    throw new AgentsApiError(`agent runtime unreachable: ${errText(err)}`, "network_error");
  }
  if (res.status === 401 || res.status === 403) {
    throw new AgentsApiError(
      `agent runtime denied access (HTTP ${res.status})`,
      "unauthorized",
      res.status,
    );
  }
  if (!res.ok) {
    throw new AgentsApiError(
      `agent runtime HTTP ${res.status} ${res.statusText || ""}`.trim(),
      "http_error",
      res.status,
    );
  }
  try {
    return (await res.json()) as T;
  } catch (err) {
    throw new AgentsApiError(
      `agent runtime response parse failed: ${errText(err)}`,
      "parse_error",
      res.status,
    );
  }
}

// ── 行模型 (真实运行时字段, 不套 mock schema 的 ag-### 约定) ────────────

export interface AgentCenterRow {
  id: string;
  name: string;
  /** 后端 AgentState 原值小写 (running/paused/stopped/...), 不做伪映射 */
  status: string;
  /** 后端 AgentRole 原值 (Worker/Supervisor/Reviewer) */
  role: string;
  /** 后端 started_at (RFC3339); 接口未提供 last_active, 不编造 */
  started_at: string | null;
}

function mapAgent(raw: unknown): AgentCenterRow | null {
  if (typeof raw !== "object" || raw === null) return null;
  const a = raw as Record<string, unknown>;
  if (typeof a.id !== "string" || a.id.length === 0) return null;
  if (typeof a.name !== "string" || a.name.length === 0) return null;
  return {
    id: a.id,
    name: a.name,
    status: typeof a.status === "string" && a.status.length > 0 ? a.status.toLowerCase() : "unknown",
    role: typeof a.role === "string" && a.role.length > 0 ? a.role : "—",
    started_at: typeof a.started_at === "string" ? a.started_at : null,
  };
}

export interface AgentsListResult {
  rows: AgentCenterRow[];
  /** 后端自带的数据源说明 (如 "内存样例, 宿主运行时接入后替换"), 原样展示不吞掉 */
  note: string | null;
}

export async function listAgents(): Promise<AgentsListResult> {
  const payload = await requestJson<{ data?: { agents?: unknown; note?: unknown } }>("/agents");
  const data = payload?.data;
  if (!data || !Array.isArray(data.agents)) {
    throw new AgentsApiError("agent runtime response missing data.agents", "parse_error");
  }
  const rows = data.agents.map(mapAgent).filter((r): r is AgentCenterRow => r !== null);
  return {
    rows,
    note: typeof data.note === "string" && data.note.length > 0 ? data.note : null,
  };
}

export async function listArgoCli(): Promise<string[]> {
  const payload = await requestJson<{ data?: { supported_cli?: unknown } }>("/argo/nodes");
  const cli = payload?.data?.supported_cli;
  if (!Array.isArray(cli) || !cli.every((c) => typeof c === "string")) {
    throw new AgentsApiError("agent runtime response missing supported_cli", "parse_error");
  }
  return cli as string[];
}

export interface RuntimeHealth {
  ok: boolean;
  status: number;
}

export async function pingRuntimeHealth(): Promise<RuntimeHealth> {
  const session = getSession();
  const headers: Record<string, string> = { Accept: "application/json" };
  if (session) headers.Authorization = `Bearer ${session.token}`;

  let res: Response;
  try {
    res = await fetch(`${API_ROOT}/health`, { headers });
  } catch (err) {
    throw new AgentsApiError(`agent runtime unreachable: ${errText(err)}`, "network_error");
  }
  return { ok: res.ok, status: res.status };
}
