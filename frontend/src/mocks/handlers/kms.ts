// =====================================================================
// MSW handler for /api/kms/* (per ADR-0044 §4)
// =====================================================================
// 全 mock (per 2026-09-02 09:35 JST 拍板, 不接真 KMS)
// POST /api/kms/unlock + /api/kms/lock 试响应
//
// 守門 (per AGENTS.md §0/§1.2):
//   - 不引新依赖
//   - 全 mock, Mavis 接手 不読 secret
//   - 状态: 永続 "unlocked" (Phase 2 簡化, 跟 retry.ts audit 一致)
// =====================================================================

import { http, HttpResponse } from "msw";

interface KmsUnlockResponse {
  /** 模擬 session token (mock 固定 "kms-mock-session-{counter}") */
  session_token: string;
  /** 3600 秒 (1h) */
  expires_in: number;
}

interface KmsLockResponse {
  locked: true;
}

let _kmsSessionCounter = 0;
const KMS_SESSION_TTL_SECONDS = 3600;

export const kmsHandlers = [
  http.post("/api/kms/unlock", async ({ request }) => {
    _kmsSessionCounter += 1;
    const body: KmsUnlockResponse = {
      session_token: `kms-mock-session-${_kmsSessionCounter}-${Date.now()}`,
      expires_in: KMS_SESSION_TTL_SECONDS,
    };
    return HttpResponse.json(body, { status: 200 });
  }),

  http.post("/api/kms/lock", async ({ request }) => {
    // mock 永続 "locked" 状态
    const body: KmsLockResponse = { locked: true };
    return HttpResponse.json(body, { status: 200 });
  }),
];
