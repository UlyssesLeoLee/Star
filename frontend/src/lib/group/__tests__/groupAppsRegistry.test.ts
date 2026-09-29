/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/__tests__/groupAppsRegistry.test.ts",type:"file",language:"typescript"}),
  (suite:Function {name:"Group App Registry projection tests",type:"function",signature:"describe('Group App Registry projection', ...)",visibility:"private",complexity:"moderate"}),
  (validCase:Function {name:"valid projection case",type:"function",signature:"it('validates and stably sorts an authorized Worktree projection', ...)",visibility:"private",complexity:"moderate"}),
  (scopeCase:Function {name:"Worktree scope case",type:"function",signature:"it('rejects a projection belonging to another Worktree', ...)",visibility:"private",complexity:"simple"}),
  (malformedCase:Function {name:"malformed projection case",type:"function",signature:"it('rejects malformed, duplicate, unsafe, or oversized entries', ...)",visibility:"private",complexity:"moderate"}),
  (fixture:Function {name:"validProjectionFixture",type:"function",signature:"validProjectionFixture(apps?: GroupAppNavigationEntry[]): GroupAppRegistryProjection",visibility:"private",complexity:"simple"}),
  (project:Function {name:"projectGroupAppsForNavigation",type:"function",signature:"projectGroupAppsForNavigation(value, worktreeId)",visibility:"public",complexity:"moderate"}),
  (expect:Function {name:"vitest.expect",type:"function",visibility:"public"}),
  (apps:Variable {name:"apps",type:"variable",language:"typescript"}),
  (projection:Variable {name:"projection",type:"variable",language:"typescript"}),
  (duplicateApps:Variable {name:"duplicateApps",type:"variable",language:"typescript"}),
  (controlCharacterApp:Variable {name:"controlCharacterApp",type:"variable",language:"typescript"}),
  (invalidCorrelation:Variable {name:"invalidCorrelation",type:"variable",language:"typescript"}),
  (invalidVersion:Variable {name:"invalidVersion",type:"variable",language:"typescript"}),
  (invalidSortOrder:Variable {name:"invalidSortOrder",type:"variable",language:"typescript"}),
  (index:Variable {name:"index",type:"variable",language:"typescript"}),
  (oversized:Variable {name:"oversized",type:"variable",language:"typescript"}),
  (file)-[:CONTAINS]->(suite),(suite)-[:CONTAINS]->(validCase),(suite)-[:CONTAINS]->(scopeCase),(suite)-[:CONTAINS]->(malformedCase),
  (file)-[:CONTAINS]->(fixture),(validCase)-[:CALLS]->(fixture),(scopeCase)-[:CALLS]->(fixture),(malformedCase)-[:CALLS]->(fixture),
  (fixture)-[:USES]->(apps),
  (validCase)-[:CALLS]->(project),(scopeCase)-[:CALLS]->(project),(malformedCase)-[:CALLS]->(project),
  (validCase)-[:USES]->(projection),(malformedCase)-[:USES]->(projection),(malformedCase)-[:USES]->(duplicateApps),(malformedCase)-[:USES]->(controlCharacterApp),(malformedCase)-[:USES]->(invalidCorrelation),(malformedCase)-[:USES]->(invalidVersion),(malformedCase)-[:USES]->(invalidSortOrder),(malformedCase)-[:USES]->(oversized),(malformedCase)-[:USES]->(index),
  (validCase)-[:CALLS]->(expect),(scopeCase)-[:CALLS]->(expect),(malformedCase)-[:CALLS]->(expect);
*/

import { describe, expect, it } from "vitest";

import { projectGroupAppsForNavigation } from "../groupAppsRegistry";
import type {
  GroupAppNavigationEntry,
  GroupAppRegistryProjection,
} from "../worktreeGroupApi";

function validProjectionFixture(
  apps: GroupAppNavigationEntry[] = [
    { plugin_id: "zeta", manifest_version: "1.0.0", label: "Zeta", sort_order: 4 },
    { plugin_id: "alpha", manifest_version: "2.0.0", label: "Alpha", sort_order: 4 },
    { plugin_id: "first", manifest_version: "1.2.0", label: "First", sort_order: 1 },
  ],
): GroupAppRegistryProjection {
  return {
    worktree_id: "wt-1",
    registry_version: 7,
    correlation_id: "d4e7d403-1684-4497-82b2-c39ff6a75e31",
    apps,
  };
}

describe("Group App Registry projection", () => {
  it("validates and stably sorts an authorized Worktree projection", () => {
    const projection = validProjectionFixture();

    expect(projectGroupAppsForNavigation(projection, "wt-1")).toEqual([
      { plugin_id: "first", manifest_version: "1.2.0", label: "First", sort_order: 1 },
      { plugin_id: "alpha", manifest_version: "2.0.0", label: "Alpha", sort_order: 4 },
      { plugin_id: "zeta", manifest_version: "1.0.0", label: "Zeta", sort_order: 4 },
    ]);
  });

  it("rejects a projection belonging to another Worktree", () => {
    expect(projectGroupAppsForNavigation(validProjectionFixture(), "wt-2")).toBeNull();
  });

  it("rejects malformed, duplicate, unsafe, or oversized entries", () => {
    const projection = validProjectionFixture();
    const duplicateApps = [projection.apps[0], projection.apps[0]];
    const controlCharacterApp = [{
      plugin_id: "safe-id",
      manifest_version: "1.0.0",
      label: "unsafe\u0000label",
      sort_order: 0,
    }];
    const invalidCorrelation = { ...projection, correlation_id: "not-a-uuid" };
    const invalidVersion = { ...projection, registry_version: -1 };
    const invalidSortOrder = {
      ...projection,
      apps: [{ plugin_id: "safe-id", manifest_version: "1.0.0", label: "Safe", sort_order: 2_147_483_648 }],
    };
    const oversized: GroupAppNavigationEntry[] = [];
    for (let index = 0; index < 101; index += 1) {
      oversized.push({ plugin_id: `plugin-${index}`, manifest_version: "1.0.0", label: "Plugin", sort_order: index });
    }

    expect(projectGroupAppsForNavigation({ ...projection, apps: duplicateApps }, "wt-1")).toBeNull();
    expect(projectGroupAppsForNavigation({ ...projection, apps: controlCharacterApp }, "wt-1")).toBeNull();
    expect(projectGroupAppsForNavigation(invalidCorrelation, "wt-1")).toBeNull();
    expect(projectGroupAppsForNavigation(invalidVersion, "wt-1")).toBeNull();
    expect(projectGroupAppsForNavigation(invalidSortOrder, "wt-1")).toBeNull();
    expect(projectGroupAppsForNavigation({ ...projection, apps: oversized }, "wt-1")).toBeNull();
  });
});
