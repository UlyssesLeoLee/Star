/*
@cypher schema=1 source_sha256=e8b63364d45a6c7f9ecdcc365c9344cd67b68d3a0dbeefc1986c2d5f6c3c5951
MERGE (self:File {path:"frontend/src/lib/nav/registry.integrity.test.ts"})
MERGE (hrefs:Symbol {id:"frontend/src/lib/nav/registry.integrity.test.ts::every_registry_href_resolves_to_a_real_route",kind:"test"})
MERGE (dupes:Symbol {id:"frontend/src/lib/nav/registry.integrity.test.ts::registry_has_no_duplicate_ids_or_codes",kind:"test"})
MERGE (self)-[:DEFINES]->(hrefs)
MERGE (self)-[:DEFINES]->(dupes)
@endcypher
*/
// per 2026-11-12 用户发令 (导航完善): 每个导航按钮必须指向真实存在的路由。
// 该测试 fs 扫描 src/app 下所有 page.tsx, 校验 ALL_MODULES 每个 href
// (剥离 query; 支持 [param] 动态段) 都能解析到真实路由 — 防止再出现
// /settings/developer 这类 404 死链。
import { describe, it, expect } from "vitest";
import { readdirSync, existsSync } from "node:fs";
import { join } from "node:path";
import { ALL_MODULES } from "./registry";

function collectRoutes(dir: string, prefix = ""): string[] {
  const routes: string[] = [];
  if (!existsSync(dir)) return routes;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name.startsWith("_") || entry.name.endsWith(".test.")) continue;
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      const segment = entry.name;
      const nextPrefix = segment.startsWith("(") ? prefix : `${prefix}/${segment}`;
      routes.push(...collectRoutes(full, nextPrefix));
    } else if (entry.name === "page.tsx" || entry.name === "page.ts") {
      routes.push(prefix || "/");
    }
  }
  return routes;
}

function routeMatches(route: string, pathname: string): boolean {
  const routeSegs = route.split("/").filter(Boolean);
  const targetSegs = pathname.split("/").filter(Boolean);
  if (routeSegs.length !== targetSegs.length) return false;
  return routeSegs.every((seg, i) => seg === targetSegs[i] || /^\[.+\]$/.test(seg));
}

describe("导航 registry 完整性 (per 2026-11-12 用户发令)", () => {
  it("every registry href resolves to a real route", () => {
    const routes = collectRoutes(join(process.cwd(), "src", "app"));
    expect(routes.length).toBeGreaterThan(20); // sanity

    const missing = ALL_MODULES.filter((m) => {
      const pathname = m.href.split("?")[0];
      return !routes.some((r) => routeMatches(r, pathname));
    });
    expect(missing.map((m) => `${m.id} → ${m.href}`)).toEqual([]);
  });

  it("registry has no duplicate ids or codes", () => {
    const ids = ALL_MODULES.map((m) => m.id);
    expect(new Set(ids).size).toBe(ids.length);
    const codes = ALL_MODULES.map((m) => m.code);
    expect(new Set(codes).size).toBe(codes.length);
  });

  it("sub-nav tabs of /projects match registry tactical entries (kanban moved to top-level tab)", () => {
    const tactical = ALL_MODULES.filter((m) => m.href.startsWith("/projects?tab="));
    // kanban 现在是顶栏 tab (href=/kanban), 只有 backlog + timeline 保留为 /projects 子 tab
    expect(tactical.map((m) => m.id).sort()).toEqual(["backlog", "timeline"]);
  });
});
