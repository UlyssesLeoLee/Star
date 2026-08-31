// frontend/src/mocks/handlers/tenants.ts
// admin 域 MSW handlers (per test-design.md v0.2 §2.1.2 + 5 域映射表)
//
// Endpoints:
//   GET /api/tenants                       — 列出所有
//   GET /api/tenants/:id                   — 单条
//   GET /api/rbac/roles?tenant_id=...      — 按 tenant 过滤
//   GET /api/rbac/roles/:id/permissions    — 返回 { role_id, permissions: string[] }
//
// 5 域映射: admin (RBAC / permission / tenant) — 本 handler 合并 tenants + rbac
//
// 已知缺口 (per 守门 #1 缺标比错标安全):
//   1. POST/PATCH/DELETE tenant P2 (Phase F+) — 当前只读
//   2. POST/PATCH/DELETE rbac role P2 (Phase F+)
//   3. 真实权限校验 (per docs/ddd/05-admin-bc.md) P3 (Phase F+)
//   4. real-mode 短路 P3 (P3-A.7 §3 缺口 #1)

import { http, HttpResponse } from "msw";
import { MOCK_TENANTS, MOCK_RBAC_ROLES } from "@/mocks/data/five-domain";

export const tenantsHandlers = [
  // ===== Tenants =====
  http.get("/api/tenants", () => {
    return HttpResponse.json(MOCK_TENANTS);
  }),

  http.get("/api/tenants/:id", ({ params }) => {
    const id = params.id as string;
    const found = MOCK_TENANTS.find((t) => t.id === id);
    if (!found) {
      return HttpResponse.json({ error: `Tenant ${id} not found` }, { status: 404 });
    }
    return HttpResponse.json(found);
  }),

  // ===== RBAC Roles =====
  http.get("/api/rbac/roles", ({ request }) => {
    const url = new URL(request.url);
    const tenantId = url.searchParams.get("tenant_id");
    if (tenantId) {
      return HttpResponse.json(MOCK_RBAC_ROLES.filter((r) => r.tenant_id === tenantId));
    }
    return HttpResponse.json(MOCK_RBAC_ROLES);
  }),

  http.get("/api/rbac/roles/:id/permissions", ({ params }) => {
    const id = params.id as string;
    const found = MOCK_RBAC_ROLES.find((r) => r.id === id);
    if (!found) {
      return HttpResponse.json({ error: `Role ${id} not found` }, { status: 404 });
    }
    return HttpResponse.json({ role_id: found.id, permissions: found.permissions });
  }),
];
