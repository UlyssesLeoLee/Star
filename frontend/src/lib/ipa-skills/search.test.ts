/*
@cypher schema=1 source_sha256=a820c8edd71780668f5efa44616f9e86ef2db024e67add3a402b9f544b37e1a2
MERGE (self:File {path:"frontend/src/lib/ipa-skills/search.test.ts"})
MERGE (fuzzy:Symbol {id:"frontend/src/lib/ipa-skills/search.test.ts::fuzzy_search_returns_ranked_matches",kind:"test"})
MERGE (empty:Symbol {id:"frontend/src/lib/ipa-skills/search.test.ts::empty_query_returns_nothing",kind:"test"})
MERGE (self)-[:DEFINES]->(fuzzy)
MERGE (self)-[:DEFINES]->(empty)
@endcypher
*/
import { describe, it, expect } from "vitest";
import { searchIpaSkills, scoreIpaSkill } from "./search";
import { IPA_SKILL_CATALOG } from "./catalog";

describe("IPA 技能模糊查询", () => {
  it("空查询不返回结果", () => {
    expect(searchIpaSkills("")).toEqual([]);
    expect(searchIpaSkills("   ")).toEqual([]);
  });

  it("id 前缀命中且排在标题命中之前", () => {
    const byId = searchIpaSkills("ipa-s01");
    expect(byId.length).toBeGreaterThan(0);
    expect(byId[0].id.startsWith("ipa-s01")).toBe(true);
  });

  it("id 子串 / 标题 / 工程都可命中", () => {
    // 任意技能: 用目录里真实存在的标题片段
    const sample = IPA_SKILL_CATALOG[10];
    const byTitle = searchIpaSkills(sample.titleJa.slice(0, 4));
    expect(byTitle.some((s) => s.id === sample.id)).toBe(true);

    const byPhase = searchIpaSkills(sample.phase);
    expect(byPhase.length).toBeGreaterThan(0);
  });

  it("打分顺序: 全等 > 前缀 > 子串", () => {
    const skill = IPA_SKILL_CATALOG[0];
    expect(scoreIpaSkill(skill.id, skill)).toBe(1000);
    expect(scoreIpaSkill(skill.id.slice(0, 6), skill)).toBe(800);
    expect(scoreIpaSkill(skill.id.slice(3, 8), skill)).toBe(600);
    expect(scoreIpaSkill("zzz-no-match", skill)).toBe(0);
  });

  it("limit 生效", () => {
    const results = searchIpaSkills("ipa", 5);
    expect(results.length).toBeLessThanOrEqual(5);
  });
});
