// frontend/src/mocks/__tests__/inbox.test.ts
// 替代 d4b3193 zod parse, 用 TS type guard isMockNotif.

import { describe, it, expect } from "vitest";
import { MOCK_NOTIFS } from "@/mocks/data";
import { isMockNotif, isNotifKind, NOTIF_KINDS } from "@/mocks/schemas/inbox";

describe("MOCK_NOTIFS", () => {
  it("has 10 rows", () => {
    expect(MOCK_NOTIFS).toHaveLength(10);
  });

  it("all rows match type guard isMockNotif", () => {
    MOCK_NOTIFS.forEach((n) => {
      expect(isMockNotif(n)).toBe(true);
    });
  });

  it("all ids are unique and match n-NNN format", () => {
    const ids = MOCK_NOTIFS.map((n) => n.id);
    expect(new Set(ids).size).toBe(ids.length);
    ids.forEach((id) => expect(id).toMatch(/^n-\d{3}$/));
  });

  it("all kind values are valid enum", () => {
    MOCK_NOTIFS.forEach((n) => {
      expect(isNotifKind(n.kind)).toBe(true);
      expect(NOTIF_KINDS as readonly string[]).toContain(n.kind);
    });
  });

  it("covers both read=true and read=false (per §3.4 invariant)", () => {
    const reads = MOCK_NOTIFS.map((n) => n.read);
    expect(reads).toContain(true);
    expect(reads).toContain(false);
  });
});
