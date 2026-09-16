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
import { useQuery } from "@tanstack/react-query";
import { OnboardingGuide } from "@/lib/onboarding/Guide";
import { scanAllDetectors, isOnboardingCompleted, markOnboardingCompleted, markOnboardingSkipped } from "@/lib/onboarding/scanner";
import { testKeyWithRetry } from "@/lib/onboarding/retry";
import type { DetectedKey, OnboardingStage, TestResult } from "@/types/onboarding";
import type { ApiKeyCreateRequest } from "@/mocks/schemas/cli";

interface CliProfileSummary {
  id: string;
  name: string;
}

export interface OnboardingGuardProps {
  /** 可选 agent 列表 (e.g. 现有 CliTab from agent-windows) */
  availableAgents?: Array<{ id: string; label: string; profileName: string }>;
  /** 13 類 tenant_id (per REQ-SEC-001) */
  tenantId?: string;
}

export function OnboardingGuard({
  availableAgents,
  tenantId = "tenant-physis-corp",
}: OnboardingGuardProps) {
  // ---- 状态 ----
  const [open, setOpen] = useState(false);
  const [stage, setStage] = useState<OnboardingStage>("idle");
  const [detectedKeys, setDetectedKeys] = useState<DetectedKey[]>([]);
  const [testResults, setTestResults] = useState<Map<string, TestResult>>(new Map());
  const startedRef = useRef(false);

  // ---- 0. agent 候选列表 (per ADR-0042 §1.1 步骤 3: 选 agent 关联) ----
  // 没显式传 availableAgents 时(e.g. 挂在 root layout, 拿不到某页面的 CliTab 状态),
  // 退而求其次查现有 CLI profile 列表 — 至少让 ReviewStage 下拉框有真实选项,
  // 而不是永远空数组导致用户根本选不了 (per 2026-09-16 review)。
  const profilesQ = useQuery<CliProfileSummary[]>({
    queryKey: ["cli-profiles", "onboarding-fallback"],
    queryFn: async () => {
      const res = await fetch("/api/cli-profiles");
      if (!res.ok) throw new Error(`fetch /api/cli-profiles failed: ${res.status}`);
      return res.json();
    },
    enabled: availableAgents === undefined,
    staleTime: 30_000,
  });
  const resolvedAgents = availableAgents ?? (profilesQ.data || []).map((p) => ({
    id: p.id,
    label: p.name,
    profileName: p.name,
  }));

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
      // 有 key 也先进 reviewing, 让用户实际选关联的 agent (per ADR-0042 §1.1 步骤 3);
      // 之前这里直接跳过 reviewing 进 associating, 导致 availableAgents 没机会展示选项
      // 就被测完标 completed, "已关联" 从没真的发生过 (per 2026-09-16 review)
      setStage("reviewing");
    } catch (e) {
      setStage("error");
    }
  }, []);

  // ---- 2. per-key 测试 (5 重试) ----
  // selections: keyId → agentId, 测试成功的且用户选了 agent 的才真正落 ApiKey 记录
  // (per ADR-0042 步骤 3 "用户选 agent 关联" 的真实含义 — 之前这一步从未执行过)
  const runTests = useCallback(async (keys: DetectedKey[], selections?: Map<string, string>) => {
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

    // 测试成功 + 用户选了 agent 的 key, 真正创建 ApiKey 记录 (per REQ-SEC-001 13 類 tenant_id)
    // 只存 environment_var 模式: 探测器永不拿明文 (per 守门 #5), 没有 secret 可传 encrypted_rust
    if (selections && selections.size > 0) {
      const agentById = new Map(resolvedAgents.map((a) => [a.id, a]));
      await Promise.all(
        keys
          .filter((key) => newResults.get(key.id)?.status === "success" && selections.has(key.id))
          .map((key) => {
            const agentId = selections.get(key.id)!;
            const agent = agentById.get(agentId);
            const payload: ApiKeyCreateRequest = {
              id: `k_onboard_${key.id}`,
              provider: key.provider,
              label: key.label,
              mode: "environment_var",
              preview: key.preview,
              envVarName: key.env_var_name,
              createdAt: new Date().toISOString().slice(0, 10),
              agent_id: agentId,
              cli_profile_id: agent?.profileName,
            };
            return fetch("/api/api-keys", {
              method: "POST",
              headers: { "Content-Type": "application/json" },
              body: JSON.stringify(payload),
            }).catch(() => {
              // 关联落库失败不阻塞 onboarding 收尾 (per 已知缺口: 真实校验留 Phase 2)
              // 用户仍可在 agent tab 齿轮按钮手动补关联
            });
          }),
      );
    }

    // 5 retry 耗尽后, 看是否全部 failed → error, 否则 completed
    const allFailed = Array.from(newResults.values()).every((r) => r.status === "failed");
    setStage(allFailed && newResults.size > 0 ? "error" : "completed");
  }, [resolvedAgents]);

  // ---- 3. 用户操作 handlers ----
  const handleSelectKey = useCallback((_keyId: string, _agentId: string) => {
    // Phase 1: 仅记录到 selections, 真实关联通过 onAssociate 触发
    // (OnboardingGuide 内部维护 selections state)
  }, []);

  const handleAssociate = useCallback((selections: Map<string, string>) => {
    // 用户确认后 → 5 retry 测试 + (成功且选了 agent 的) 真正创建 ApiKey 关联
    setStage("associating");
    void runTests(detectedKeys, selections);
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
      availableAgents={resolvedAgents}
      tenantId={tenantId}
    />
  );
}
