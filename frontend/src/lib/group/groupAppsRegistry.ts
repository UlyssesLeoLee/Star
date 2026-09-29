/*
CYPHER STRUCTURE MANIFEST
CREATE
  (file:File {name:"frontend/src/lib/group/groupAppsRegistry.ts",type:"file",language:"typescript"}),
  (project:Function {name:"projectGroupAppsForNavigation",type:"function",signature:"projectGroupAppsForNavigation(value: unknown, worktreeId: string): GroupAppNavigationEntry[] | null",visibility:"public",complexity:"moderate"}),
  (validPluginId:Function {name:"isValidPluginId",type:"function",signature:"isValidPluginId(value: unknown): value is string",visibility:"private",complexity:"simple"}),
  (validUuid:Function {name:"isUuid",type:"function",signature:"isUuid(value: unknown): value is string",visibility:"private",complexity:"simple"}),
  (controlChars:Function {name:"hasControlCharacters",type:"function",signature:"hasControlCharacters(value: string): boolean",visibility:"private",complexity:"simple"}),
  (compareEntries:Function {name:"compareGroupAppNavigationEntries",type:"function",signature:"compareGroupAppNavigationEntries(left: GroupAppNavigationEntry, right: GroupAppNavigationEntry): number",visibility:"private",complexity:"simple"}),
  (projection:Variable {name:"projection",type:"variable",language:"typescript"}),
  (pluginIds:Variable {name:"pluginIds",type:"variable",language:"typescript"}),
  (apps:Variable {name:"apps",type:"variable",language:"typescript"}),
  (candidate:Variable {name:"candidate",type:"variable",language:"typescript"}),
  (app:Variable {name:"app",type:"variable",language:"typescript"}),
  (pluginId:Variable {name:"pluginId",type:"variable",language:"typescript"}),
  (manifestVersion:Variable {name:"manifestVersion",type:"variable",language:"typescript"}),
  (label:Variable {name:"label",type:"variable",language:"typescript"}),
  (sortOrder:Variable {name:"sortOrder",type:"variable",language:"typescript"}),
  (uuidPattern:Variable {name:"UUID_PATTERN",type:"variable",language:"typescript"}),
  (pluginIdPattern:Variable {name:"PLUGIN_ID_PATTERN",type:"variable",language:"typescript"}),
  (controlPattern:Variable {name:"CONTROL_CHARACTER_PATTERN",type:"variable",language:"typescript"}),
  (left:Variable {name:"left",type:"variable",language:"typescript"}),
  (right:Variable {name:"right",type:"variable",language:"typescript"}),
  (file)-[:CONTAINS]->(project),(file)-[:CONTAINS]->(validPluginId),(file)-[:CONTAINS]->(validUuid),(file)-[:CONTAINS]->(controlChars),(file)-[:CONTAINS]->(compareEntries),
  (project)-[:USES]->(projection),(project)-[:USES]->(pluginIds),(project)-[:USES]->(apps),(project)-[:USES]->(candidate),(project)-[:USES]->(app),(project)-[:USES]->(pluginId),(project)-[:USES]->(manifestVersion),(project)-[:USES]->(label),(project)-[:USES]->(sortOrder),
  (file)-[:CONTAINS]->(uuidPattern),(file)-[:CONTAINS]->(pluginIdPattern),(file)-[:CONTAINS]->(controlPattern),
  (project)-[:USES]->(uuidPattern),(project)-[:USES]->(pluginIdPattern),(project)-[:USES]->(controlPattern),
  (validUuid)-[:USES]->(uuidPattern),(validPluginId)-[:USES]->(pluginIdPattern),(controlChars)-[:USES]->(controlPattern),
  (compareEntries)-[:USES]->(left),(compareEntries)-[:USES]->(right),
  (project)-[:CALLS]->(validUuid),(project)-[:CALLS]->(validPluginId),(project)-[:CALLS]->(controlChars),(project)-[:CALLS]->(compareEntries);
*/

import type {
  GroupAppNavigationEntry,
  GroupAppRegistryProjection,
} from "./worktreeGroupApi";

const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const PLUGIN_ID_PATTERN = /^[a-z0-9][a-z0-9._-]{0,63}$/;
const CONTROL_CHARACTER_PATTERN = /[\u0000-\u001f\u007f]/;

export function projectGroupAppsForNavigation(
  value: unknown,
  worktreeId: string,
): GroupAppNavigationEntry[] | null {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const projection = value as Partial<GroupAppRegistryProjection>;
  if (
    projection.worktree_id !== worktreeId
    || !Number.isSafeInteger(projection.registry_version)
    || (projection.registry_version ?? -1) < 0
    || !isUuid(projection.correlation_id)
    || !Array.isArray(projection.apps)
    || projection.apps.length > 100
  ) {
    return null;
  }

  const pluginIds = new Set<string>();
  const apps: GroupAppNavigationEntry[] = [];
  for (const candidate of projection.apps) {
    if (!candidate || typeof candidate !== "object" || Array.isArray(candidate)) return null;
    const app = candidate as Partial<GroupAppNavigationEntry>;
    const pluginId = app.plugin_id;
    const manifestVersion = app.manifest_version;
    const label = app.label;
    const sortOrder = app.sort_order;
    if (
      !isValidPluginId(pluginId)
      || pluginIds.has(pluginId)
      || typeof manifestVersion !== "string"
      || !manifestVersion.trim()
      || manifestVersion.length > 128
      || hasControlCharacters(manifestVersion)
      || typeof label !== "string"
      || !label.trim()
      || label.length > 160
      || hasControlCharacters(label)
      || typeof sortOrder !== "number"
      || !Number.isSafeInteger(sortOrder)
      || sortOrder < -2_147_483_648
      || sortOrder > 2_147_483_647
    ) {
      return null;
    }
    pluginIds.add(pluginId);
    apps.push({
      plugin_id: pluginId,
      manifest_version: manifestVersion,
      label,
      sort_order: sortOrder,
    });
  }

  return apps.sort(compareGroupAppNavigationEntries);
}

function isValidPluginId(value: unknown): value is string {
  return typeof value === "string" && PLUGIN_ID_PATTERN.test(value);
}

function isUuid(value: unknown): value is string {
  return typeof value === "string" && UUID_PATTERN.test(value);
}

function hasControlCharacters(value: string): boolean {
  return CONTROL_CHARACTER_PATTERN.test(value);
}

function compareGroupAppNavigationEntries(
  left: GroupAppNavigationEntry,
  right: GroupAppNavigationEntry,
): number {
  return left.sort_order - right.sort_order || left.plugin_id.localeCompare(right.plugin_id);
}
