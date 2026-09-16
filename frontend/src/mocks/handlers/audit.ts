// =====================================================================
// MSW handler for /api/audit/* (per ADR-0043 §2.3, commit 5 of 6)
// =====================================================================
// POST /api/audit/onboarding-failed - Phase 2 audit 真接 endpoint mock
// (Phase 1: localStorage fallback, per commit a54c79d)
// 守门 #5: Mavis 接手 不读 secret, mock 返 audit_event_id 而非 凭证
// =====================================================================

import { http, HttpResponse } from "msw";
import { isRealMode, realFetch } from "@/mocks/real-mode";

interface AuditOnboardingFailedRequest {
  detected_key_id: string;
  provider: string;
  label: string;
  attempts: number;
  status_code: number;
  error_message: string;
  tenant_id: string;
  client_ip?: string;
  request_id?: string;
}

interface AuditOnboardingFailedResponse {
  audit_event_id: string;
  /** 守门 #9 audit log 必須 (per AGENTS.md §4 #9 审计必带)
   * Phase 2 実 backend では audit_audit_event.id (UUID) になる */
  occurred_at: string;
}

export const auditHandlers = [
  http.post("/api/audit/onboarding-failed", async ({ request }) => {
    if (isRealMode()) {
      return realFetch("/api/audit/onboarding-failed", {
        method: "POST",
        body: await request.clone().text(),
        headers: request.headers as HeadersInit,
      });
    }
    const body = (await request.json()) as AuditOnboardingFailedRequest;
    // 守门 #11 缺标比错标: 13 類 tenant_id 必帯 (per REQ-SEC-001)
    if (!body.tenant_id || !body.detected_key_id || !body.provider) {
      return HttpResponse.json(
        { error: "invalid_payload", missing: ["tenant_id | detected_key_id | provider"] },
        { status: 400 },
      );
    }
    // Phase 1 mock: 返 audit_event_id (Phase 2 SQLx Adapter 返 真 UUID)
    const auditEventId = `audit-${crypto.randomUUID()}`;
    const response: AuditOnboardingFailedResponse = {
      audit_event_id: auditEventId,
      occurred_at: new Date().toISOString(),
    };
    return HttpResponse.json(response, { status: 201 });
  }),
];
