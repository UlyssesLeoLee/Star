/*
@cypher schema=1 source_sha256=c29f6aedfc05f1d302ccd89461719db98af5bc71c82cc3e921cbaf7555ec9b54
MERGE (self:File {path:"frontend/src/components/team/runtimeApi.ts"})
MERGE (listEndpoint:Endpoint {path:"/api/v1/runtime",method:"GET"})
MERGE (registerEndpoint:Endpoint {path:"/api/v1/runtime/register",method:"POST"})
MERGE (heartbeatEndpoint:Endpoint {path:"/api/v1/runtime/{id}/heartbeat",method:"POST"})
MERGE (apiBase:Config {name:"NEXT_PUBLIC_API_BASE"})
MERGE (localRuntime:Type {id:"frontend/src/types/ids.ts::LocalRuntime",kind:"interface"})
MERGE (runtimeStatus:Type {id:"frontend/src/types/ids.ts::RuntimeStatus",kind:"type"})
MERGE (fetchRuntimeList:Function {name:"fetchRuntimeList",type:"function",language:"typescript",visibility:"exported",complexity:"moderate"})
MERGE (registerRuntime:Function {name:"registerRuntime",type:"function",language:"typescript",visibility:"exported",complexity:"simple"})
MERGE (sendHeartbeat:Function {name:"sendHeartbeat",type:"function",language:"typescript",visibility:"exported",complexity:"simple"})
MERGE (apiRequest:Function {name:"apiRequest",type:"function",language:"typescript",visibility:"private",complexity:"moderate"})
MERGE (self)-[:DEFINES]->(fetchRuntimeList)
MERGE (self)-[:DEFINES]->(registerRuntime)
MERGE (self)-[:DEFINES]->(sendHeartbeat)
MERGE (self)-[:DEFINES]->(apiRequest)
MERGE (self)-[:READS]->(apiBase)
MERGE (self)-[:USES_TYPE]->(localRuntime)
MERGE (self)-[:USES_TYPE]->(runtimeStatus)
MERGE (fetchRuntimeList)-[:CALLS]->(apiRequest)
MERGE (fetchRuntimeList)-[:CALLS]->(listEndpoint)
MERGE (registerRuntime)-[:CALLS]->(apiRequest)
MERGE (registerRuntime)-[:CALLS]->(registerEndpoint)
MERGE (sendHeartbeat)-[:CALLS]->(apiRequest)
MERGE (sendHeartbeat)-[:CALLS]->(heartbeatEndpoint)
@endcypher
*/

// =====================================================================
// runtimeApi — star-api-rest /api/v1/runtime* 数据层
// (TeamPanel Runtime/Daemon 区真实接线; 失败由组件层降级到 store 预览)
// =====================================================================
// 职责:
//   1. fetchRuntimeList   GET  /api/v1/runtime               → LocalRuntime[]
//   2. registerRuntime    POST /api/v1/runtime/register      → void (调用方刷新列表)
//   3. sendHeartbeat      POST /api/v1/runtime/{id}/heartbeat → void (调用方刷新列表)
//   4. 全部请求带超时 (AbortController) + 非 2xx / 超时 / 网络错误统一抛错;
//      此层不做任何演示数据兜底 (诚实原则: 失败交给组件降级并保留"未接入"标注)。
//
// base: process.env.NEXT_PUBLIC_API_BASE, 默认 http://localhost:8081 (dev 端口)。
// 响应契约: 后端 star-api-rest 端点由同批任务新增, 本层按 RestResponse 信封
//   ({data: [...]} / 裸数组 / {data: {items: [...]}}) 宽容解析,
//   字段名兼容 snake_case / camelCase 两种风格。
// =====================================================================

import type { LocalRuntime, RuntimeStatus } from "@/types/ids";

export const RUNTIME_API_BASE =
  process.env.NEXT_PUBLIC_API_BASE || "http://localhost:8081";

const DEFAULT_TIMEOUT_MS = 4000;

export class RuntimeApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "RuntimeApiError";
  }
}

type ApiRequestInit = Omit<RequestInit, "headers" | "signal">;

async function apiRequest(
  path: string,
  init: ApiRequestInit = {},
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
): Promise<unknown> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(`${RUNTIME_API_BASE}${path}`, {
      ...init,
      signal: controller.signal,
      headers: { Accept: "application/json", "Content-Type": "application/json" },
    });
    if (!res.ok) {
      throw new RuntimeApiError(`${init.method ?? "GET"} ${path} → HTTP ${res.status}`);
    }
    if (res.status === 204) return null;
    return (await res.json()) as unknown;
  } catch (err) {
    if (err instanceof Error && err.name === "AbortError") {
      throw new RuntimeApiError(`${init.method ?? "GET"} ${path} 超时 (${timeoutMs}ms)`);
    }
    throw err;
  } finally {
    clearTimeout(timer);
  }
}

/** 宽容抽取运行时数组: 裸数组 / RestResponse({data}) / {data:{items}} 信封 */
function extractRows(json: unknown): unknown[] {
  if (Array.isArray(json)) return json;
  if (!json || typeof json !== "object") return [];
  const root = json as Record<string, unknown>;
  for (const key of ["data", "runtimes", "items", "records"]) {
    const value = root[key];
    if (Array.isArray(value)) return value;
    if (value && typeof value === "object" && !Array.isArray(value)) {
      const inner = value as Record<string, unknown>;
      for (const innerKey of ["items", "records", "runtimes"]) {
        if (Array.isArray(inner[innerKey])) return inner[innerKey] as unknown[];
      }
    }
  }
  return [];
}

function normalizeStatus(raw: unknown): RuntimeStatus {
  switch (String(raw ?? "").trim().toLowerCase()) {
    case "online":
    case "active":
      return "online";
    case "offline":
    case "inactive":
      return "offline";
    case "compromised":
      return "compromised";
    case "revoked":
    case "expired":
      return "revoked";
    default:
      // 未知状态不冒充在线/离线, 仅如实标注"已注册" (该记录确实来自注册表)
      return "registered";
  }
}

function toLocalRuntime(raw: unknown): LocalRuntime | null {
  if (!raw || typeof raw !== "object" || Array.isArray(raw)) return null;
  const item = raw as Record<string, unknown>;
  const id = String(item.id ?? item.runtime_id ?? item.runtimeId ?? "");
  const hostname = String(item.hostname ?? item.name ?? "");
  if (!id && !hostname) return null; // 非运行时行: 跳过而非伪造
  return {
    id,
    tenant_id: String(item.tenant_id ?? item.tenantId ?? ""),
    device_id: String(item.device_id ?? item.deviceId ?? ""),
    hostname,
    status: normalizeStatus(item.status),
    bound_user_id: String(item.bound_user_id ?? item.boundUserId ?? ""),
    bound_tenant_id: String(item.bound_tenant_id ?? item.boundTenantId ?? ""),
    mount_root: String(item.mount_root ?? item.mountRoot ?? ""),
    last_heartbeat_at: String(item.last_heartbeat_at ?? item.lastHeartbeatAt ?? ""),
    policy_violations: Number(item.policy_violations ?? item.policyViolations ?? 0),
  };
}

/** GET /api/v1/runtime → LocalRuntime[] (非 2xx/超时/网络错误抛错) */
export async function fetchRuntimeList(
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
): Promise<LocalRuntime[]> {
  const json = await apiRequest("/api/v1/runtime", { method: "GET" }, timeoutMs);
  return extractRows(json)
    .map(toLocalRuntime)
    .filter((r): r is LocalRuntime => r !== null);
}

/** POST /api/v1/runtime/register (成功后由调用方刷新列表) */
export async function registerRuntime(
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
): Promise<void> {
  // 注册载荷以后端契约为准; 当前发送空 JSON body (设备上下文由后端推导)。
  await apiRequest("/api/v1/runtime/register", { method: "POST", body: "{}" }, timeoutMs);
}

/** POST /api/v1/runtime/{id}/heartbeat (成功后由调用方刷新列表) */
export async function sendHeartbeat(
  runtimeId: string,
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
): Promise<void> {
  await apiRequest(
    `/api/v1/runtime/${encodeURIComponent(runtimeId)}/heartbeat`,
    { method: "POST", body: "{}" },
    timeoutMs,
  );
}
