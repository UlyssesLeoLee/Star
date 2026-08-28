// frontend/src/mocks/__tests__/inbox.test.ts

import { describe, it, expect } from "vitest";
import { MOCK_NOTIFS } from "@/mocks/data";
import { MockNotifSchema, NotifKindSchema } from "@/mocks/schemas/inbox";

describe("MOCK_NOTIFS", () => {
  it("has 10 rows", () => {
    expect(MOCK_NOTIFS).toHaveLength(10);
  });

  it("all rows match zod schema", () => {
    MOCK_NOTIFS.forEach((n) => {
      expect(() => MockNotifSchema.parse(n)).not.toThrow();
    });
  });

  it("all ids are unique and match n-NNN format", () => {
    const ids = MOCK_NOTIFS.map((n) => n.id);
    expect(new Set(ids).size).toBe(ids.length);
    ids.forEach((id) => expect(id).toMatch(/^n-\d{3}$/));
  });

  it("all kind values are valid enum", () => {
    MOCK_NOTIFS.forEach((n) => {
      expect(() => NotifKindSchema.parse(n.kind)).not.toThrow();
    });
  });

  it("covers both read=true and read=false (per §3.4 invariant)", () => {
    const reads = MOCK_NOTIFS.map((n) => n.read);
    expect(reads).toContain(true);
    expect(reads).toContain(false);
  });
});
