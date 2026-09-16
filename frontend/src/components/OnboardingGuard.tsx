"use client";

// =====================================================================
// OnboardingGuard — 首次启动自动引导 wrapper (per ADR-0042 §1.1)
// =====================================================================
// 挂载点: app/layout.tsx 在 Providers 内, PwaBoot 旁
// 职责:
//   1. mount 时调 scanAllDetectors() (3 探测器并行)
//   2. 检查 isOnboardingCompleted(), 跳过已完成用户
//   3. 没 key → 不弹 (用户可后续手动填)
//   4. 有 key → 弹 OnboardingGuide 走 4 阶段
//   5. associating 阶段按 DetectedKey 调 testKeyWithRetry (5 重试)
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖
//   - SSR-safe: typeof window check
//   - 自动 mark completed 后不再弹 (除非用户重置 /settings/reset-onboarding)
// =====================================================================

import { useEffect, useState, useCallback, useRef } from "react";
import { OnboardingGuide } from "@/lib/onboarding/Guide";
import { scanAllDetectors, isOnboardingCompleted, markOnboardingCompleted, markOnboardingSkipped } from "@/lib/onboarding/scanner";
import { testKeyWithRetry } from "@/lib/onboarding/retry";
import type { DetectedKey, OnboardingStage, TestResult } from "@/types/onboarding";

export interface OnboardingGuardProps {
  /** 可选 agent 列表 (e.g. 现有 CliTab from agent-windows) */
  availableAgents?: Array<{ id: string; label: string; profileName: string }>;
  /** 13 類 tenant_id (per REQ-SEC-001) */
  tenantId?: string;
}

export function OnboardingGuard({
  availableAgents = [],
  tenantId = "tenant-physis-corp",
}: OnboardingGuardProps) {
  // ---- 状态 ----
  const [open, setOpen] = useState(false);
  const [stage, setStage] = useState<OnboardingStage>("idle");
  const [detectedKeys, setDetectedKeys] = useState<DetectedKey[]>([]);
  const [testResults, setTestResults] = useState<Map<string, TestResult>>(new Map());
  const startedRef = useRef(false);

  // ---- 1. mount 扫 ----
  useEffect(() => {
    if (startedRef.current) return;
    startedRef.current = true;
    if (isOnboardingCompleted()) {
      setStage("completed");
      return;
    }
    void runScan();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const runScan = useCallback(async () => {
    setStage("scanning");
    setOpen(true);
    try {
      const keys = await scanAllDetectors();
      setDetectedKeys(keys);
      if (keys.length === 0) {
        // 没 key, 直接进 reviewing (空状态) 让用户跳过
        setStage("reviewing");
      } else {
        // 跳过 reviewing, 自动进入 associating (5 重试测试)
        setStage("associating");
        await runTests(keys);
      }
    } catch (e) {
      setStage("error");
    }
  }, []);

  // ---- 2. per-key 测试 (5 重试) ----
  const runTests = useCallback(async (keys: DetectedKey[]) => {
    const newResults = new Map<string, TestResult>();
    for (const key of keys) {
      const result = await testKeyWithRetry(key, (partial) => {
        // 进度更新 (per 拍板 retryreport_opt3: 实时回调)
        setTestResults((prev) => {
          const next = new Map(prev);
          next.set(key.id, partial);
          return next;
        });
      });
      newResults.set(key.id, result);
    }
    setTestResults(newResults);

    // 5 retry 耗尽后, 看是否全部 failed → error, 否则 completed
    const allFailed = Array.from(newResults.values()).every((r) => r.status === "failed");
    setStage(allFailed && newResults.size > 0 ? "error" : "completed");
  }, []);

  // ---- 3. 用户操作 handlers ----
  const handleSelectKey = useCallback((_keyId: string, _agentId: string) => {
    // Phase 1: 仅记录到 selections, 真实关联通过 onAssociate 触发
    // (OnboardingGuide 内部维护 selections state)
  }, []);

  const handleAssociate = useCallback(() => {
    // 重新跑 tests (per 拍板: 用户确认后 → 5 retry)
    setStage("associating");
    void runTests(detectedKeys);
  }, [detectedKeys, runTests]);

  const handleSkip = useCallback(() => {
    setOpen(false);
    setStage("completed");
  }, []);

  const handleClose = useCallback(() => {
    setOpen(false);
  }, []);

  const handleRetryTest = useCallback((keyId: string) => {
    const key = detectedKeys.find((k) => k.id === keyId);
    if (!key) return;
    setTestResults((prev) => {
      const next = new Map(prev);
      next.delete(keyId);  // 清空旧 result, 重新跑
      return next;
    });
    setStage("associating");
    testKeyWithRetry(key, (partial) => {
      setTestResults((prev) => {
        const next = new Map(prev);
        next.set(keyId, partial);
        return next;
      });
    }).then((final) => {
      setTestResults((prev) => {
        const next = new Map(prev);
        next.set(keyId, final);
        return next;
      });
      // 跑完检查 stage
      setTestResults((current) => {
        const all = Array.from(current.values());
        const allFailed = all.length > 0 && all.every((r) => r.status === "failed");
        setStage(allFailed ? "error" : "completed");
        return current;
      });
    });
  }, [detectedKeys]);

  return (
    <OnboardingGuide
      open={open}
      stage={stage}
      detectedKeys={detectedKeys}
      testResults={testResults}
      onSelectKey={handleSelectKey}
      onAssociate={handleAssociate}
      onSkip={handleSkip}
      onClose={handleClose}
      onRetryTest={handleRetryTest}
      availableAgents={availableAgents}
      tenantId={tenantId}
    />
  );
}
