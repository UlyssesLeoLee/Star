/*
@cypher schema=1 source_sha256=855da00b65cafa7eda49609747c9d79cecf9f5bfd67735b4aa745813a8292902
MERGE (self:File {path:"frontend/src/mocks/handlers/run-directory.ts"})
MERGE (seed:File {path:"frontend/src/lib/seed.ts"})
MERGE (handlers:Symbol {id:"frontend/src/mocks/handlers/run-directory.ts::runDirectoryHandlers",kind:"constant"})
MERGE (self)-[:DEFINES]->(handlers)
MERGE (self)-[:IMPORTS]->(seed)
@endcypher
*/
// frontend/src/mocks/handlers/run-directory.ts
// 网页版降级目录端点 (per 2026-11-19 用户反馈: Project 导航应是树形结构,
// 现在只有提示文字)。宿主会话未装配时, FallbackProjectRunTree 经
// useFallbackRunDirectory 从本端点取 Project → Lane → Worktree 层级。
//
// 数据源 = src/lib/seed.ts 离线 fixture (与 MSW 其他 handler 同一数据通道),
// real-mode 开启时短路到真 API (形状待后端 /api/v1/run-directory 对齐)。
// 不新造数据: 复用既有 projects/lanes/worktrees fixture。

import { http, HttpResponse } from "msw";
import { lanes, projects, worktrees } from "@/lib/seed";
import { isRealMode, realFetch } from "@/mocks/real-mode";

export const runDirectoryHandlers = [
  http.get("/api/run-directory", async () => {
    if (isRealMode()) {
      return realFetch("/api/v1/run-directory");
    }
    return HttpResponse.json({ projects, lanes, worktrees });
  }),
];
