/*
@cypher schema=1 source_sha256=0de78616bb9ce5ada54eaf716e13da45b00a940ef028fb84f81ac5346cfa3f9e
MERGE (self:File {path:"frontend/src/lib/ipa-skills/search.ts"})
MERGE (search:Symbol {id:"frontend/src/lib/ipa-skills/search.ts::searchIpaSkills",kind:"function"})
MERGE (self)-[:DEFINES]->(search)
@endcypher
*/
// =====================================================================
// IPA 技能模糊查询 (per 2026-11-11 用户发令: 任务卡选择关联内容时支持模糊查询)
// 命中维度: id (前缀/子串) / titleJa (子串) / phaseJa+phaseZh / purpose。
// 打分: 前缀 id > 全等 > 子串 id > 标题前缀 > 标题子串 > phase > purpose。
// =====================================================================

import { IPA_SKILL_CATALOG, type IpaSkillEntry } from "./catalog";

export function scoreIpaSkill(query: string, skill: IpaSkillEntry): number {
  const q = query.trim().toLowerCase();
  if (q.length === 0) return 0;
  const id = skill.id.toLowerCase();
  const title = skill.titleJa.toLowerCase();
  const phase = `${skill.phaseJa}${skill.phaseZh}${skill.phase}`.toLowerCase();
  const purpose = (skill.purpose || "").toLowerCase();

  if (id === q) return 1000;
  if (title === q) return 900;
  if (id.startsWith(q)) return 800;
  if (title.startsWith(q)) return 700;
  if (id.includes(q)) return 600;
  if (title.includes(q)) return 500;
  if (phase.includes(q)) return 300;
  if (purpose.includes(q)) return 200;
  return 0;
}

/** 模糊查询 IPA 技能目录, 返回按相关度排序的结果 (limit 默认 20)。 */
export function searchIpaSkills(query: string, limit = 20): IpaSkillEntry[] {
  const q = query.trim();
  if (q.length === 0) return [];
  return IPA_SKILL_CATALOG.map((skill) => ({ skill, score: scoreIpaSkill(q, skill) }))
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .slice(0, limit)
    .map((r) => r.skill);
}
