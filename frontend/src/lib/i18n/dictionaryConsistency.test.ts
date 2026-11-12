/*
@cypher schema=1 source_sha256=4f6a8b0c2d4e6f8a0b1c3d5e7f9a1b3c5d7e9f0a2b4c6d8e0f1a3b5c7d9e0f2a
MERGE (self:File {path:"frontend/src/lib/i18n/dictionaryConsistency.test.ts"})
MERGE (suite:Symbol {id:"frontend/src/lib/i18n/dictionaryConsistency.test.ts::dictionary consistency",kind:"suite"})
MERGE (en:File {path:"frontend/src/lib/i18n/en.ts"})
MERGE (zh:File {path:"frontend/src/lib/i18n/zh-CN.ts"})
MERGE (ja:File {path:"frontend/src/lib/i18n/ja.ts"})
MERGE (self)-[:IMPORTS]->(en)
MERGE (self)-[:IMPORTS]->(zh)
MERGE (self)-[:IMPORTS]->(ja)
MERGE (self)-[:DEFINES]->(suite)
@endcypher
*/
// per WBS §23.32-D: 三语字典 key 集合一致性测试 — en / zh-CN / ja 必须同构。
// 任何语言缺 key 或多了 key 都直接失败, 防止后续 i18n 追加漏同步。
import { describe, it, expect } from "vitest";
import { en } from "./en";
import { zhCN } from "./zh-CN";
import { ja } from "./ja";

function flattenKeys(obj: Record<string, unknown>, prefix = ""): string[] {
  const keys: string[] = [];
  for (const [key, value] of Object.entries(obj)) {
    const full = prefix ? `${prefix}.${key}` : key;
    if (value !== null && typeof value === "object" && !Array.isArray(value)) {
      keys.push(...flattenKeys(value as Record<string, unknown>, full));
    } else {
      keys.push(full);
    }
  }
  return keys.sort();
}

describe("dictionary consistency (en / zh-CN / ja)", () => {
  it("all three dictionaries expose the same flattened key set", () => {
    const enKeys = flattenKeys(en as unknown as Record<string, unknown>);
    const zhKeys = flattenKeys(zhCN as unknown as Record<string, unknown>);
    const jaKeys = flattenKeys(ja as unknown as Record<string, unknown>);

    const missingInZh = enKeys.filter((k) => !zhKeys.includes(k));
    const missingInJa = enKeys.filter((k) => !jaKeys.includes(k));
    const extraInZh = zhKeys.filter((k) => !enKeys.includes(k));
    const extraInJa = jaKeys.filter((k) => !enKeys.includes(k));

    expect(missingInZh).toEqual([]);
    expect(missingInJa).toEqual([]);
    expect(extraInZh).toEqual([]);
    expect(extraInJa).toEqual([]);
  });

  it("covers the WBS §23.32-D ipaSkills / nema sections in all languages", () => {
    expect(en.ipaSkills.everySkillIsTaskCard.length).toBeGreaterThan(0);
    expect(zhCN.ipaSkills.useAsTaskCard).toContain("任务卡");
    expect(ja.ipaSkills.humanGate.length).toBeGreaterThan(0);
    expect(en.nema.pageTitle.length).toBeGreaterThan(0);
    expect(zhCN.nema.registerSharedFile).toContain(".nema");
    expect(ja.nema.registerSharedFile).toContain(".nema");
  });
});
