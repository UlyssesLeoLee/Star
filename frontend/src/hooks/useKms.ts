// =====================================================================
// useKms hook — KMS unlock/lock 状态管理 (per ADR-0044 §4)
// =====================================================================
// 全 mock (per 2026-09-02 09:35 JST 拍板, 不接真 KMS)
// 状態: locked | unlocked (session_token 持有)
// API: unlock() / lock() / getToken()
//
// 守門 (per AGENTS.md §0/§1.2):
//   - 不引新依赖
//   - React 18 useState + useCallback (无 Zustand 重複)
//   - 守門 #5: Mavis 接手 不読 secret, session_token 模拟值
// =====================================================================

import { useState, useCallback } from "react";

export type KmsState = "locked" | "unlocked";

export interface UseKmsResult {
  state: KmsState;
  sessionToken: string | null;
  /** 3600 秒 (1h), per mock handler */
  expiresInSec: number;
  unlock: () => Promise<void>;
  lock: () => void;
}

export function useKms(): UseKmsResult {
  const [state, setState] = useState<KmsState>("unlocked");  // mock 默认 unlocked
  const [sessionToken, setSessionToken] = useState<string | null>(
    "kms-mock-session-initial",
  );
  const [expiresInSec] = useState<number>(3600);

  const unlock = useCallback(async () => {
    try {
      const res = await fetch("/api/kms/unlock", { method: "POST" });
      if (res.ok) {
        const body = await res.json();
        setSessionToken(body.session_token);
        setState("unlocked");
      }
    } catch {
      // 静默失败, mock 默认 unlocked, 不动
    }
  }, []);

  const lock = useCallback(() => {
    // 走 mock 端点 (best-effort, 不 await)
    void fetch("/api/kms/lock", { method: "POST" });
    setSessionToken(null);
    setState("locked");
  }, []);

  return { state, sessionToken, expiresInSec, unlock, lock };
}
