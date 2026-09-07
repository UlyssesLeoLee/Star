/**
 * TraceIdPropagator (browser) - 跨层 trace_id 传递 (per F-12)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.1
 */

let currentTraceId: string | null = null;

export function setTraceId(traceId: string): void {
  currentTraceId = traceId;
}

export function getTraceId(): string | null {
  return currentTraceId;
}

export function clearTraceId(): void {
  currentTraceId = null;
}

/** 获取或生成新 trace_id (UUID v4) */
export function ensureTraceId(): string {
  if (!currentTraceId) {
    currentTraceId = crypto.randomUUID();
  }
  return currentTraceId;
}
