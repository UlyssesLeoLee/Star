// frontend/src/mocks/handlers/worktree-shared.ts

/*
@cypher schema=1 source_sha256=aa7bdb1fdd72e78f3cf86473b6169387d698bdcb213e6ebf7ee67745334e234e
MERGE (self:File {path:"frontend/src/mocks/handlers/worktree-shared.ts"})
MERGE (maybeReal:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::maybeReal",kind:"function"})
MERGE (get:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::get",kind:"function"})
MERGE (post:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::post",kind:"function"})
MERGE (handlers:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::worktreeSharedHandlers",kind:"constant"})
MERGE (repoId:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::MOCK_REPO_ID",kind:"constant"})
MERGE (candidates:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::MOCK_CANDIDATES",kind:"constant"})
MERGE (external:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::MOCK_EXTERNAL_WORKTREES",kind:"constant"})
MERGE (imported:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::importedPaths",kind:"constant"})
MERGE (externalType:Symbol {id:"frontend/src/mocks/handlers/worktree-shared.ts::MockExternalWorktree",kind:"interface"})
MERGE (isRealMode:Symbol {id:"frontend/src/mocks/real-mode.ts::isRealMode",kind:"function"})
MERGE (realFetch:Symbol {id:"frontend/src/mocks/real-mode.ts::realFetch",kind:"function"})
MERGE (msw:Module {name:"msw"})
MERGE (realMode:Module {name:"frontend/src/mocks/real-mode"})
MERGE (httpGet:ExternalService {name:"msw.http.get"})
MERGE (httpPost:ExternalService {name:"msw.http.post"})
MERGE (json:ExternalService {name:"msw.HttpResponse.json"})
MERGE (self)-[:IMPORTS]->(msw)
MERGE (self)-[:IMPORTS]->(realMode)
MERGE (self)-[:DEFINES]->(maybeReal)
MERGE (self)-[:DEFINES]->(get)
MERGE (self)-[:DEFINES]->(post)
MERGE (self)-[:DEFINES]->(handlers)
MERGE (self)-[:DEFINES]->(repoId)
MERGE (self)-[:DEFINES]->(candidates)
MERGE (self)-[:DEFINES]->(external)
MERGE (self)-[:DEFINES]->(imported)
MERGE (self)-[:DEFINES]->(externalType)
MERGE (maybeReal)-[:CALLS]->(isRealMode)
MERGE (maybeReal)-[:CALLS]->(realFetch)
MERGE (get)-[:CALLS]->(httpGet)
MERGE (get)-[:CALLS]->(maybeReal)
MERGE (post)-[:CALLS]->(httpPost)
MERGE (post)-[:CALLS]->(maybeReal)
MERGE (handlers)-[:CALLS]->(get)
MERGE (handlers)-[:CALLS]->(post)
MERGE (handlers)-[:CALLS]->(json)
MERGE (handlers)-[:USES]->(candidates)
MERGE (handlers)-[:USES]->(external)
MERGE (handlers)-[:USES]->(imported)
MERGE (external)-[:USES]->(repoId)
@endcypher
*/
//
// ULYS-228 (ULYS-218.1) FR-ORCA-009 + FR-ORCA-011 frontend MSW handlers.
//
// 暴露三个 endpoint (per issue §3 + §4):
//   GET  /api/v1/worktree-picker/candidates?repo_id=X  → 4 分类 PickerCandidatesDto
//   GET  /api/v1/worktrees/external-import/scan?repo_id=X → { items: ExternalWorktreeDto[] }
//   POST /api/v1/worktrees/external-import              → ExternalImportResponse
//
// 注: 跟 backend 真 endpoint 是 1:1 映射 (snake_case 字段名), 真后端 ship 后
// MSW handler 在 real-mode 下自动透传 (per mocks/real-mode.ts). 当前 backend
// `crates/api/src/worktree_picker.rs` worktree_picker_router Stage 1 返 503,
// 真 endpoint 完全 ship 之前 MSW 给假数据让 UI 走通 end-to-end.

import { http, HttpResponse } from "msw";
import { isRealMode, realFetch } from "@/mocks/real-mode";

function maybeReal(path: string, init: RequestInit = {}): Promise<Response> | null {
  if (!isRealMode()) return null;
  return realFetch(path, init);
}

// ---------------------------------------------------------------------
// Mock data — 跟 backend 字段名严格对齐 (snake_case)
// ---------------------------------------------------------------------

const MOCK_REPO_ID = "00000000-0000-0000-0000-000000000001";

const MOCK_CANDIDATES = {
  // github_branches  → StartFromKind `remote_branch`
  github_branches: [
    {
      id: "remote_branch:origin/feature/worktree-picker",
      label: "origin/feature/worktree-picker",
      description: "Remote tracking branch, last fetched 2026-09-23",
      kind: "remote_branch",
    },
    {
      id: "remote_branch:origin/fix-shared-dir-mvp",
      label: "origin/fix-shared-dir-mvp",
      description: "Remote tracking branch, last fetched 2026-09-22",
      kind: "remote_branch",
    },
  ],
  // existing_worktrees → StartFromKind `local_branch`
  existing_worktrees: [
    {
      id: "local_branch:feature/worktree-picker",
      label: "feature/worktree-picker",
      description: "Local branch, ahead 3 / behind 0",
      kind: "local_branch",
    },
    {
      id: "local_branch:fix-shared-dir-mvp",
      label: "fix-shared-dir-mvp",
      description: "Local branch, ahead 1 / behind 2",
      kind: "local_branch",
    },
    {
      id: "local_branch:main",
      label: "main",
      description: "Local branch (current HEAD)",
      kind: "local_branch",
    },
  ],
  // local_paths (字段名保留 backend 命名, 语义是 commit_sha list) → `commit_sha`
  local_paths: [
    {
      id: "commit_sha:a1b2c3d4e5f6",
      label: "a1b2c3d — chore: bump deps",
      description: "a1b2c3d4e5f67890abcdef1234567890abcdef12",
      kind: "commit_sha",
    },
    {
      id: "commit_sha:f6e5d4c3b2a1",
      label: "f6e5d4c — feat: external worktree import",
      description: "f6e5d4c3b2a109876543210fedcba0987654321",
      kind: "commit_sha",
    },
  ],
  // empty → StartFromKind `repo_base`
  empty: {
    id: "repo_base:origin/main",
    label: "origin/main",
    description: "仓库默认 base ref (origin/main)",
    kind: "repo_base",
  },
};

interface MockExternalWorktree {
  path: string;
  head_commit: string;
  branch: string | null;
  is_managed: boolean;
}

const MOCK_EXTERNAL_WORKTREES: Record<string, MockExternalWorktree[]> = {
  [MOCK_REPO_ID]: [
    {
      path: "C:/worktrees/external-scratch-1",
      head_commit: "1234567890abcdef1234567890abcdef12345678",
      branch: "external-scratch-1",
      is_managed: false,
    },
    {
      path: "C:/worktrees/external-scratch-2",
      head_commit: "fedcba0987654321fedcba0987654321fedcba09",
      branch: null,
      is_managed: false,
    },
  ],
};

// in-memory imported state: 每个 POST /external-import 后把对应 path 标记
// is_managed = true, 下次 scan 自动过滤掉 (per AC-2 "Import 按钮触发后, 该
// worktree 进入 Orca 状态机").
const importedPaths = new Set<string>();

// ---------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------

const get = (
  path: string,
  mock: () => Response | Promise<Response>,
) =>
  http.get(path, async () => {
    const r = await maybeReal(path);
    return r ?? (await mock());
  });

const post = (
  path: string,
  mock: (body: unknown) => Response | Promise<Response>,
) =>
  http.post(path, async ({ request }) => {
    const realInit: RequestInit = {
      method: "POST",
      body: await request.clone().text(),
      headers: request.headers as HeadersInit,
    };
    const r = await maybeReal(path, realInit);
    if (r) return r;
    const body = await request.json();
    return mock(body);
  });

export const worktreeSharedHandlers = [
  // GET /api/v1/worktree-picker/candidates?repo_id=X
  // 镜像 backend `crates/api/src/worktree_picker.rs` candidates_handler
  // + worktree_picker_router (Stage 1 placeholder 返 503, Stage 2 返真数据).
  get("/api/v1/worktree-picker/candidates", () => {
    return HttpResponse.json(MOCK_CANDIDATES);
  }),

  // GET /api/v1/worktrees/external-import/scan?repo_id=X
  // 镜像 backend `RealExternalWorktreeImport::scan` (per
  // crates/worktree-shared-dir/src/external_worktree_import.rs). 返回当前
  // repo 下所有 external worktrees, UI sidebar 据此显示 hidden card.
  get("/api/v1/worktrees/external-import/scan", () => {
    const items = (MOCK_EXTERNAL_WORKTREES[MOCK_REPO_ID] ?? []).map((w) => ({
      ...w,
      is_managed: importedPaths.has(w.path) || w.is_managed,
    }));
    return HttpResponse.json({ items });
  }),

  // POST /api/v1/worktrees/external-import
  // 入参 { repo_id, path }, 返 { worktree_id, state, at }.
  post("/api/v1/worktrees/external-import", (body) => {
    const b = body as { repo_id?: unknown; path?: unknown };
    if (typeof b.repo_id !== "string" || typeof b.path !== "string") {
      return HttpResponse.json(
        { code: "WSD.EXT_BAD_REQUEST", message: "missing repo_id or path" },
        { status: 400 },
      );
    }
    const items = MOCK_EXTERNAL_WORKTREES[b.repo_id] ?? [];
    const found = items.find((w) => w.path === b.path);
    if (!found) {
      return HttpResponse.json(
        {
          code: "WSD.EXT_PATH_NOT_IN_SCAN",
          message: `path ${b.path} not in scan output`,
        },
        { status: 404 },
      );
    }
    // 标记该 path 为已 managed (per AC-2 import 后进入 Provisioning)
    importedPaths.add(b.path);
    return HttpResponse.json({
      worktree_id: `wt-imported-${Date.now().toString(36)}`,
      state: "Provisioning",
      at: new Date().toISOString(),
    });
  }),
];