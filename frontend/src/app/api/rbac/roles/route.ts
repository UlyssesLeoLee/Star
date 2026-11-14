/*
@cypher schema=1 source_sha256=c8e488d3a456e107e7d70fdfc048779f59f9aced7dfa48c36ebc5d36d3f54017
MERGE (self:File {path:"frontend/src/app/api/rbac/roles/route.ts"})
MERGE (r:Symbol {id:"frontend/src/app/api/rbac/roles/route.ts::Route",kind:"route"})
MERGE (self)-[:DEFINES]->(r)
@endcypher
*/

// WBS-BACKEND-SUPPORT-001 W1 (2026-11-14): 缺后端功能补全 — 与 MSW mock 同数据源

import { NextRequest, NextResponse } from "next/server";
import { MOCK_RBAC_ROLES } from "@/mocks/data/five-domain";

export async function GET(req: NextRequest) {
  const tenantId = req.nextUrl.searchParams.get("tenant_id");
  if (tenantId) return NextResponse.json(MOCK_RBAC_ROLES.filter((r) => r.tenant_id === tenantId));
  return NextResponse.json(MOCK_RBAC_ROLES);
}

export async function POST(req: NextRequest) {
  const body = await req.json();
  return NextResponse.json(body, { status: 201 });
}
