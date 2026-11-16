/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"frontend/src/app/api/notifications/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/notifications/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// =====================================================================
// DEPRECATED (2026-11-17): 收件箱接真实后端通知 API 后, 页面不再调用本 mock 路由。
//   新契约: 浏览器经 realFetch 直连真实后端 GET /api/v1/notifications
//   (base_url / Bearer 见 src/mocks/real-mode.ts, 封装见 src/lib/notifications/notificationsApi.ts)。
//
// 保留本路由但返回 410 Gone: 任何残留调用方都会显式失败,
// 而不是拿到 MOCK_NOTIFS 冒充真实数据 (禁止假数据冒充真实)。
// MSW 层 mock handlers 仅服务 mock 测试, 与本路由无关。
// =====================================================================

import { NextResponse } from "next/server";

export function GET() {
  return NextResponse.json(
    {
      error: {
        code: "deprecated",
        message:
          "frontend /api/notifications 已废弃 (2026-11-17): 收件箱直连真实后端 GET /api/v1/notifications (real-mode)。请勿依赖本路由返回的 mock 数据。",
      },
    },
    { status: 410 },
  );
}
