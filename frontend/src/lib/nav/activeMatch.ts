/*
@cypher schema=1 source_sha256=ca93859c529bdd094bbddb9279c5cec4b1f26c9c5db5731d4bbed1f1ff65daea
MERGE (self:File {path:"frontend/src/lib/nav/activeMatch.ts"})
MERGE (match:Symbol {id:"frontend/src/lib/nav/activeMatch.ts::moduleHrefMatches",kind:"function"})
MERGE (self)-[:DEFINES]->(match)
@endcypher
*/
// =====================================================================
// 导航 active 判定 (per 2026-11-12 用户发令: 每个导航按钮都要正确切换)
// 单一实现, Sidebar / AppHeader 共用。规则:
//   - pathname 不等 → false
//   - 无 query → pathname 相等即 true
//   - 带 ?tab=X → 当前 searchParams.tab 必须等于 X
// =====================================================================
export function moduleHrefMatches(
  moduleHref: string,
  pathname: string,
  searchParams: URLSearchParams | null,
): boolean {
  const [hrefPath, hrefQuery] = moduleHref.split("?");
  if (hrefPath !== pathname) return false;
  if (!hrefQuery) return true;
  const tabParam = new URLSearchParams(hrefQuery).get("tab");
  if (tabParam !== null) return searchParams?.get("tab") === tabParam;
  return true;
}
