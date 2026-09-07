/**
 * UT-03: BusinessKeyHasher 业务主键 hash
 */

import { describe, it, expect } from "vitest";
import { computeBusinessHash, computeBusinessHashSync } from "../business_hash";

describe("BusinessKeyHasher", () => {
  it("computeBusinessHash produces consistent SHA-256 hex", async () => {
    const parts = {
      tenantId: "tenant-1",
      resourceType: "work_item",
      resourceId: "uuid-A",
      operation: "update_status",
      version: 5,
    };
    const h1 = await computeBusinessHash(parts);
    const h2 = await computeBusinessHash(parts);
    expect(h1).toBe(h2);
    expect(h1).toMatch(/^[0-9a-f]{64}$/);
  });

  it("computeBusinessHash distinguishes version", async () => {
    const base = {
      tenantId: "tenant-1",
      resourceType: "work_item",
      resourceId: "uuid-A",
      operation: "update_status",
    };
    const h1 = await computeBusinessHash({ ...base, version: 5 });
    const h2 = await computeBusinessHash({ ...base, version: 6 });
    expect(h1).not.toBe(h2);
  });

  it("computeBusinessHash distinguishes resource", async () => {
    const base = {
      tenantId: "tenant-1",
      resourceType: "work_item",
      operation: "update_status",
      version: 5,
    };
    const h1 = await computeBusinessHash({ ...base, resourceId: "uuid-A" });
    const h2 = await computeBusinessHash({ ...base, resourceId: "uuid-B" });
    expect(h1).not.toBe(h2);
  });

  it("computeBusinessHashSync produces consistent hash", () => {
    const parts = {
      tenantId: "tenant-1",
      resourceType: "work_item",
      resourceId: "uuid-A",
      operation: "update_status",
      version: 5,
    };
    const h1 = computeBusinessHashSync(parts);
    const h2 = computeBusinessHashSync(parts);
    expect(h1).toBe(h2);
    expect(h1).toMatch(/^[0-9a-f]+$/);
  });
});
