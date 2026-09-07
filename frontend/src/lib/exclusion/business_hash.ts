/**
 * BusinessKeyHasher - 业务主键 hash (per F-09 双键 fallback)
 *
 * per docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §2.1.2
 *
 * 算法: SHA-256(tenant_id|resource_type|resource_id|operation|version)
 */

export interface BusinessKeyParts {
  tenantId: string;
  resourceType: string;
  resourceId: string;
  operation: string;
  version: number;
}

export async function computeBusinessHash(parts: BusinessKeyParts): Promise<string> {
  const input = `${parts.tenantId}|${parts.resourceType}|${parts.resourceId}|${parts.operation}|${parts.version}`;
  const encoder = new TextEncoder();
  const buf = await crypto.subtle.digest("SHA-256", encoder.encode(input));
  return Array.from(new Uint8Array(buf))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
}

/** 同步版本 (用于 Node 端 SSR 或测试, 实际生产用异步 Web Crypto) */
export function computeBusinessHashSync(parts: BusinessKeyParts): string {
  // 简单 hash (test only - 生产用 computeBusinessHash)
  const input = `${parts.tenantId}|${parts.resourceType}|${parts.resourceId}|${parts.operation}|${parts.version}`;
  let hash = 0;
  for (let i = 0; i < input.length; i++) {
    hash = ((hash << 5) - hash + input.charCodeAt(i)) | 0;
  }
  return Math.abs(hash).toString(16);
}
