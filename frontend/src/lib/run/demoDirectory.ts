/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/lib/run/demoDirectory.ts"})
MERGE (payload:Symbol {id:"frontend/src/lib/run/demoDirectory.ts::demoRunDirectoryPayload",kind:"function"})
MERGE (valid:Symbol {id:"frontend/src/lib/run/demoDirectory.ts::demoRunDirectoryPayloadIsValid",kind:"function"})
MERGE (type:Symbol {id:"frontend/src/lib/run/demoDirectory.ts::DemoRunDirectoryPayload",kind:"type"})
MERGE (self)-[:DEFINES]->(payload)
MERGE (self)-[:DEFINES]->(valid)
MERGE (self)-[:DEFINES]->(type)
@endcypher
*/
// demoDirectory — 「演示目录」独立数据模块 (P0-lane 2026-11-20)
//
// 用户要求: 演示目录独立存放并明确标注。本模块把演示数据与真实数据路径彻底分离:
//   - 真实模式 (宿主会话 + 服务端 Lane API): 数据走 RunDirectoryApiClient,
//     绝不混入本模块数据 (useFallbackRunDirectory 在 isRealMode() 时短路)。
//   - 演示模式 (web 无宿主): useFallbackRunDirectory 仅在三集合全空且非 real-mode 时,
//     把本模块 payload 注入共享 store, UI 以「演示目录」badge 明确标注。
//
// 本模块是演示数据的唯一存放处 (不再经 /api/run-directory MSW 通道取数)。
// 与 lib/seed.ts 的全局 fixture 不同: 这里只放目录演示所需的 Project/Lane/Worktree,
// 且所有 worktree 显式带 lane_id, 供「演示目录」树渲染 Project → Lane → Worktree → Branch。

import type { Lane, Project, Worktree } from "@/types/ids";

export interface DemoRunDirectoryPayload {
  projects: Project[];
  lanes: Lane[];
  worktrees: Worktree[];
}

/** 演示数据来源标记 — 与真实 API 来源 (宿主会话) 区分, UI badge 依据 */
export const DEMO_RUN_DIRECTORY_SOURCE = "demo-directory" as const;

const DEMO_PROJECT_ID = "00000000-0000-4000-8000-0000000000d1";
const DEMO_PROJECT_ID_B = "00000000-0000-4000-8000-0000000000d2";
const DEMO_TENANT_ID = "00000000-0000-4000-8000-0000000000d0";
const DEMO_LANE_IDS = {
  core: "00000000-0000-4000-8000-0000000000e1",
  ui: "00000000-0000-4000-8000-0000000000e2",
  mobile: "00000000-0000-4000-8000-0000000000e3",
} as const;

const demoProjects: Project[] = [
  {
    id: DEMO_PROJECT_ID,
    tenant_id: DEMO_TENANT_ID,
    key: "PHYSIS-DEMO",
    name: "Physis 演示项目",
    visibility: "private",
    owner_id: "00000000-0000-4000-8000-0000000000a1",
    member_count: 2,
    created_at: "2026-11-01T00:00:00.000Z",
  },
  {
    id: DEMO_PROJECT_ID_B,
    tenant_id: DEMO_TENANT_ID,
    key: "STARGATE-DEMO",
    name: "StarGate 演示项目",
    visibility: "private",
    owner_id: "00000000-0000-4000-8000-0000000000a2",
    member_count: 1,
    created_at: "2026-11-05T00:00:00.000Z",
  },
];

const demoLanes: Lane[] = [
  {
    id: DEMO_LANE_IDS.core,
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID,
    name: "core-runtime (演示)",
    created_at: "2026-11-10T00:00:00.000Z",
  },
  {
    id: DEMO_LANE_IDS.ui,
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID,
    name: "ui-shell (演示)",
    git_remote_url: "https://example.invalid/star/ui-demo.git",
    git_provider: "github",
    default_branch: "dev",
    created_at: "2026-11-11T00:00:00.000Z",
  },
  {
    id: DEMO_LANE_IDS.mobile,
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID_B,
    name: "mobile (演示)",
    created_at: "2026-11-12T00:00:00.000Z",
  },
];

const demoWorktrees: Worktree[] = [
  {
    id: "00000000-0000-4000-8000-0000000000f1",
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID,
    lane_id: DEMO_LANE_IDS.core,
    name: "wt-core-checkout (演示)",
    branch: "feat/core-demo",
    base_branch: "main",
    status: "active",
    lock_version: 1,
    last_event_at: "2026-11-19T00:00:00.000Z",
    created_at: "2026-11-18T00:00:00.000Z",
  },
  {
    id: "00000000-0000-4000-8000-0000000000f2",
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID,
    lane_id: DEMO_LANE_IDS.ui,
    name: "wt-ui-tabs (演示)",
    branch: "feat/ui-tabs-demo",
    base_branch: "main",
    status: "review_requested",
    lock_version: 2,
    last_event_at: "2026-11-19T08:00:00.000Z",
    created_at: "2026-11-17T00:00:00.000Z",
  },
  {
    id: "00000000-0000-4000-8000-0000000000f3",
    tenant_id: DEMO_TENANT_ID,
    project_id: DEMO_PROJECT_ID_B,
    lane_id: DEMO_LANE_IDS.mobile,
    name: "wt-mobile-switch (演示)",
    branch: "feat/mobile-demo",
    base_branch: "main",
    status: "dirty",
    lock_version: 1,
    last_event_at: "2026-11-19T12:00:00.000Z",
    created_at: "2026-11-16T00:00:00.000Z",
  },
];

/** 返回演示目录 payload (每调用返回新对象, 防止调用方就地改写污染后续注入) */
export function demoRunDirectoryPayload(): DemoRunDirectoryPayload {
  return {
    projects: demoProjects.map((project) => ({ ...project })),
    lanes: demoLanes.map((lane) => ({ ...lane })),
    worktrees: demoWorktrees.map((worktree) => ({ ...worktree })),
  };
}

/** 演示 payload 结构校验: 引用完整 (lane → project, worktree → lane) 且无重复 id */
export function demoRunDirectoryPayloadIsValid(payload: DemoRunDirectoryPayload): boolean {
  if (
    !Array.isArray(payload.projects) ||
    !Array.isArray(payload.lanes) ||
    !Array.isArray(payload.worktrees)
  ) {
    return false;
  }
  const projectIds = new Set(payload.projects.map((project) => project.id));
  if (projectIds.size !== payload.projects.length) return false;
  const laneIds = new Set<string>();
  for (const lane of payload.lanes) {
    if (!projectIds.has(lane.project_id) || laneIds.has(lane.id)) return false;
    laneIds.add(lane.id);
  }
  const worktreeIds = new Set<string>();
  for (const worktree of payload.worktrees) {
    if (
      !projectIds.has(worktree.project_id) ||
      (worktree.lane_id !== undefined && !laneIds.has(worktree.lane_id)) ||
      worktreeIds.has(worktree.id)
    ) {
      return false;
    }
    worktreeIds.add(worktree.id);
  }
  return true;
}
