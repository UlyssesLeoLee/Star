"use client";

// =====================================================================
// OnboardingGuide — 首次启动引导 modal (per ADR-0042)
// =====================================================================
// 4 阶段 UI (per 拍板):
//   1. scanning  → 3 探测器并行扫, spinner
//   2. reviewing → 列出 DetectedKey, 用户选 + 关联
//   3. associating → per-key 5 retry 进度 (per 拍板 retry_opt3)
//   4. completed / error → 完成卡片 / 错误卡片 (per 拍板 retryreport_opt3)
//
// 守门 (per AGENTS.md §0/§1.2):
//   - 不引新依赖 (React + Lucide + clsx)
//   - 13 類 tenant_id 必带 (per REQ-SEC-001, Phase 1 mock 写死 tenant-physis-corp)
//   - 永不显示明文 key (per 守门 #5)
// =====================================================================

import { useEffect, useState, useCallback } from "react";
import { createPortal } from "react-dom";
import { clsx } from "clsx";
import {
  Key, Sparkles, X, Loader2, CheckCircle2, AlertTriangle,
  ArrowRight, ShieldCheck, ExternalLink, Code2, RotateCw,
} from "lucide-react";
import type {
  DetectedKey, TestResult, OnboardingStage, ErrorResolution, ProviderErrorCode,
} from "@/types/onboarding";
import { RETRY_BACKOFF_MS, MAX_RETRY_ATTEMPTS } from "@/types/onboarding";
import { ERROR_RESOLUTIONS } from "./retry";
import { markOnboardingCompleted, markOnboardingSkipped } from "./scanner";

// =====================================================================
// Main component
// =====================================================================
export interface OnboardingGuideProps {
  open: boolean;
  stage: OnboardingStage;
  detectedKeys: DetectedKey[];
  testResults: Map<string, TestResult>;
  onSelectKey: (keyId: string, agentId: string) => void;
  onAssociate: () => void;        // 用户点"确认关联" 触发父组件扫
  onSkip: () => void;             // "稍后" — 标记跳过
  onClose: () => void;            // 完成或跳过后关闭
  onRetryTest?: (keyId: string) => void;  // 失败重测
  /** 可选 agent 列表 (e.g. 现有 CliTab) 让用户选关联 */
  availableAgents?: Array<{ id: string; label: string; profileName: string }>;
  /** 13 類 tenant_id (per REQ-SEC-001) */
  tenantId?: string;
}

export function OnboardingGuide({
  open, stage, detectedKeys, testResults,
  onSelectKey, onAssociate, onSkip, onClose, onRetryTest,
  availableAgents = [], tenantId = "tenant-physis-corp",
}: OnboardingGuideProps) {
  // ---- 选 key → agent 的映射 (per key id 临时) ----
  const [selections, setSelections] = useState<Map<string, string>>(new Map());

  // ---- 关闭逻辑 ----
  const handleClose = useCallback(() => {
    if (stage === "scanning" || stage === "associating") return;  // 不允许中途关
    onClose();
  }, [stage, onClose]);

  if (!open || typeof window === "undefined") return null;

  return createPortal(
    <div
      role="dialog"
      aria-modal="true"
      aria-label="Onboarding: 自动识别 LLM API key"
      data-testid="onboarding-guide"
      onClick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/70 backdrop-blur-sm"
    >
      <div
        data-testid="onboarding-guide-content"
        className="card flex flex-col shadow-2xl w-[min(720px,90vw)] max-h-[min(640px,90vh)]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between gap-2 px-4 py-2.5 border-b border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
          <div className="flex items-center gap-2 min-w-0 flex-1">
            <Sparkles size={14} className="text-[color:var(--color-primary)] shrink-0" />
            <span className="text-sm font-semibold">欢迎使用 Star</span>
            <span className="text-[10px] text-[color:var(--color-text-dim)] font-mono">
              · 首次启动引导 · tenant = {tenantId}
            </span>
          </div>
          {stage !== "scanning" && stage !== "associating" && (
            <button
              type="button"
              data-testid="onboarding-close"
              onClick={handleClose}
              aria-label="Close"
              className="text-[color:var(--color-text-dim)] hover:text-[color:var(--color-text)] transition-colors p-1"
            >
              <X size={14} />
            </button>
          )}
        </div>

        {/* Body */}
        <div className="flex-1 overflow-y-auto p-4 space-y-3">
          {/* Stage 1: scanning */}
          {stage === "scanning" && (
            <div data-testid="onboarding-stage-scanning" className="flex flex-col items-center justify-center py-12 gap-3">
              <Loader2 size={32} className="text-[color:var(--color-primary)] animate-spin" />
              <div className="text-sm text-[color:var(--color-text)]">正在扫描本地凭证...</div>
              <div className="text-[10px] text-[color:var(--color-text-dim)] font-mono">
                3 探测器并行: localStorage + env-var-hint + IDE-residual
              </div>
            </div>
          )}

          {/* Stage 2: reviewing */}
          {stage === "reviewing" && (
            <ReviewStage
              detectedKeys={detectedKeys}
              testResults={testResults}
              selections={selections}
              availableAgents={availableAgents}
              onSelectKey={onSelectKey}
              onChangeSelection={(keyId, agentId) => {
                setSelections((prev) => {
                  const next = new Map(prev);
                  if (agentId) next.set(keyId, agentId);
                  else next.delete(keyId);
                  return next;
                });
              }}
            />
          )}

          {/* Stage 3: associating */}
          {stage === "associating" && (
            <AssociatingStage
              detectedKeys={detectedKeys}
              testResults={testResults}
            />
          )}

          {/* Stage 4a: completed */}
          {stage === "completed" && (
            <CompletedStage
              detectedKeys={detectedKeys}
              testResults={testResults}
            />
          )}

          {/* Stage 4b: error (5 retry 耗尽) */}
          {stage === "error" && (
            <ErrorStage
              detectedKeys={detectedKeys}
              testResults={testResults}
              onRetryTest={onRetryTest}
            />
          )}
        </div>

        {/* Footer (per stage) */}
        <div className="flex items-center justify-between gap-2 px-4 py-2.5 border-t border-[color:var(--color-border)] bg-[color:var(--color-surface-2)]">
          <button
            type="button"
            data-testid="onboarding-skip"
            onClick={() => { markOnboardingSkipped(); onSkip(); }}
            disabled={stage === "scanning" || stage === "associating"}
            className="text-[10px] text-[color:var(--color-text-dim)] hover:text-[color:var(--color-text)] disabled:opacity-50"
          >
            稍后再做
          </button>
          <div className="flex items-center gap-1.5">
            {stage === "reviewing" && (
              <button
                type="button"
                data-testid="onboarding-associate"
                onClick={onAssociate}
                disabled={selections.size === 0}
                className="btn-primary text-[11px] px-3 py-1.5 flex items-center gap-1.5"
              >
                <ArrowRight size={12} />
                确认关联 ({selections.size})
              </button>
            )}
            {(stage === "completed" || stage === "error") && (
              <button
                type="button"
                data-testid="onboarding-done"
                onClick={() => { markOnboardingCompleted(); handleClose(); }}
                className="btn-primary text-[11px] px-3 py-1.5"
              >
                完成
              </button>
            )}
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}

// =====================================================================
// 子组件: ReviewStage (Stage 2)
// =====================================================================
function ReviewStage({
  detectedKeys, testResults, selections, availableAgents,
  onSelectKey, onChangeSelection,
}: {
  detectedKeys: DetectedKey[];
  testResults: Map<string, TestResult>;
  selections: Map<string, string>;
  availableAgents: Array<{ id: string; label: string; profileName: string }>;
  onSelectKey: (keyId: string, agentId: string) => void;
  onChangeSelection: (keyId: string, agentId: string) => void;
}) {
  if (detectedKeys.length === 0) {
    return (
      <div data-testid="onboarding-no-keys" className="flex flex-col items-center justify-center py-12 gap-2 text-center">
        <Key size={28} className="text-[color:var(--color-text-dim)]" />
        <div className="text-sm text-[color:var(--color-text)]">未检测到本地 LLM API key</div>
        <div className="text-[10px] text-[color:var(--color-text-dim)] max-w-md">
          你可以在每个 agent tab 旁的 ⚙️ 齿轮按钮手动填 key, 或在 <a href="/settings/api-keys" className="text-[color:var(--color-primary)] underline">/settings/api-keys</a> 统一管理。
        </div>
      </div>
    );
  }

  return (
    <div data-testid="onboarding-stage-reviewing" className="space-y-2">
      <div className="rounded-md border border-[color:var(--color-info)]/30 bg-[color:var(--color-info)]/5 p-2.5 text-[11px] text-[color:var(--color-text-dim)] flex items-start gap-2">
        <ShieldCheck size={12} className="text-[color:var(--color-info)] mt-0.5 shrink-0" />
        <div>
          检测到 <strong className="text-[color:var(--color-info)]">{detectedKeys.length}</strong> 个 API key.
          为每个 key 选一个 agent 关联 (或跳过, 稍后手动填)。
          4 必备 = openai / claude / gemini / minimax.
        </div>
      </div>

      {detectedKeys.map((key) => {
        const sel = selections.get(key.id) || "";
        const result = testResults.get(key.id);
        const isRequired = ["openai", "claude", "gemini", "minimax"].includes(key.provider);
        return (
          <div
            key={key.id}
            data-testid={`onboarding-key-${key.id}`}
            className="rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-2.5 space-y-1.5"
          >
            {/* Row 1: provider + label + source + required 标记 */}
            <div className="flex items-center gap-2">
              <Key size={11} className="text-[color:var(--color-primary)]" />
              <span className={clsx(
                "text-[10px] px-1.5 py-0.5 rounded font-mono",
                isRequired
                  ? "bg-[color:var(--color-primary)]/15 text-[color:var(--color-primary)] border border-[color:var(--color-primary)]/30"
                  : "bg-[color:var(--color-surface-2)] text-[color:var(--color-text-dim)]",
              )}>
                {key.provider}{isRequired ? " · 必备" : ""}
              </span>
              <span className="text-xs font-medium">{key.label}</span>
              <span className="text-[10px] text-[color:var(--color-text-dim)] font-mono truncate" title={key.source_label}>
                · {key.source}
              </span>
              {result?.status === "success" && (
                <CheckCircle2 size={11} className="text-[color:var(--color-success)] ml-auto" />
              )}
              {result?.status === "failed" && (
                <AlertTriangle size={11} className="text-[color:var(--color-danger)] ml-auto" />
              )}
            </div>
            {/* Row 2: preview (永不显示明文) */}
            <div className="font-mono text-[10px] text-[color:var(--color-text-dim)] truncate">
              {key.preview}
            </div>
            {/* Row 3: agent select */}
            <div className="flex items-center gap-1.5">
              <span className="text-[10px] text-[color:var(--color-text-dim)]">关联到:</span>
              <select
                data-testid={`onboarding-agent-select-${key.id}`}
                value={sel}
                onChange={(e) => {
                  onChangeSelection(key.id, e.target.value);
                  if (e.target.value) onSelectKey(key.id, e.target.value);
                }}
                className="flex-1 text-[11px] rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface-2)] px-1.5 py-1"
              >
                <option value="">— 暂不关联 —</option>
                {availableAgents.map((a) => (
                  <option key={a.id} value={a.id}>
                    {a.label} ({a.profileName})
                  </option>
                ))}
              </select>
            </div>
            {/* Row 4: source label */}
            <div className="text-[9px] text-[color:var(--color-text-dim)] font-mono">
              {key.source_label}
            </div>
          </div>
        );
      })}
    </div>
  );
}

// =====================================================================
// 子组件: AssociatingStage (Stage 3)
// =====================================================================
function AssociatingStage({
  detectedKeys, testResults,
}: {
  detectedKeys: DetectedKey[];
  testResults: Map<string, TestResult>;
}) {
  return (
    <div data-testid="onboarding-stage-associating" className="space-y-2">
      <div className="rounded-md border border-[color:var(--color-info)]/30 bg-[color:var(--color-info)]/5 p-2.5 text-[11px] text-[color:var(--color-text-dim)]">
        正在测试 {detectedKeys.length} 个 key (5 重试 + 3-6-12-24-48s backoff, per ADR-0042).
        每 key 失败会写入 audit log + 显示解决步骤。
      </div>
      {detectedKeys.map((key) => {
        const r = testResults.get(key.id);
        return (
          <div
            key={key.id}
            data-testid={`onboarding-test-${key.id}`}
            className="rounded border border-[color:var(--color-border)] bg-[color:var(--color-surface)] p-2.5"
          >
            <div className="flex items-center gap-2">
              <span className="text-xs font-mono">{key.provider}</span>
              <span className="text-[10px] text-[color:var(--color-text-dim)]">{key.label}</span>
              <span className="ml-auto">
                {r?.status === "running" && <Loader2 size={11} className="animate-spin text-[color:var(--color-primary)]" />}
                {r?.status === "success" && <CheckCircle2 size={11} className="text-[color:var(--color-success)]" />}
                {r?.status === "failed" && <AlertTriangle size={11} className="text-[color:var(--color-danger)]" />}
                {!r && <span className="text-[10px] text-[color:var(--color-text-dim)]">待开始</span>}
              </span>
            </div>
            {r && r.status === "running" && (
              <div className="mt-1.5 text-[10px] text-[color:var(--color-text-dim)] font-mono">
                attempt {r.attempt + 1}/{r.max_attempts}
                {r.next_retry_in_ms ? ` · 下次重试 ${(r.next_retry_in_ms / 1000).toFixed(0)}s 后` : ""}
              </div>
            )}
            {r && r.status === "failed" && r.error_message && (
              <div className="mt-1.5 text-[10px] text-[color:var(--color-danger)] font-mono">
                {r.error_message} {r.status_code ? `(status ${r.status_code})` : ""}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

// =====================================================================
// 子组件: CompletedStage (Stage 4a)
// =====================================================================
function CompletedStage({
  detectedKeys, testResults,
}: {
  detectedKeys: DetectedKey[];
  testResults: Map<string, TestResult>;
}) {
  const success = detectedKeys.filter((k) => testResults.get(k.id)?.status === "success").length;
  const failed = detectedKeys.filter((k) => testResults.get(k.id)?.status === "failed").length;
  return (
    <div data-testid="onboarding-stage-completed" className="space-y-3">
      <div className="flex flex-col items-center gap-2 py-6">
        <CheckCircle2 size={32} className="text-[color:var(--color-success)]" />
        <div className="text-sm font-semibold">关联完成</div>
        <div className="text-[11px] text-[color:var(--color-text-dim)]">
          {success} 成功 / {failed} 失败 / {detectedKeys.length} 总计
        </div>
      </div>
      {failed > 0 && (
        <div className="rounded-md border border-[color:var(--color-warning)]/30 bg-[color:var(--color-warning)]/5 p-2.5 text-[10px] text-[color:var(--color-text-dim)]">
          ⚠️ {failed} 个 key 测试失败, 已写入 audit log. 你可以:
          <ul className="list-disc list-inside mt-1 space-y-0.5">
            <li>在 agent tab 旁 ⚙️ 齿轮按钮重新关联</li>
            <li>或 <a href="/settings/api-keys" className="text-[color:var(--color-primary)] underline">/settings/api-keys</a> 查看</li>
          </ul>
        </div>
      )}
    </div>
  );
}

// =====================================================================
// 子组件: ErrorStage (Stage 4b) — 5 retry 耗尽
// =====================================================================
function ErrorStage({
  detectedKeys, testResults, onRetryTest,
}: {
  detectedKeys: DetectedKey[];
  testResults: Map<string, TestResult>;
  onRetryTest?: (keyId: string) => void;
}) {
  const failedKeys = detectedKeys.filter((k) => testResults.get(k.id)?.status === "failed");
  return (
    <div data-testid="onboarding-stage-error" className="space-y-3">
      <div className="flex flex-col items-center gap-2 py-4">
        <AlertTriangle size={28} className="text-[color:var(--color-danger)]" />
        <div className="text-sm font-semibold text-[color:var(--color-danger)]">
          {failedKeys.length} 个 key 5 次重试后仍失败
        </div>
        <div className="text-[10px] text-[color:var(--color-text-dim)]">
          已写入 audit log. 你可以手动解决, 或在 agent 旁齿轮按钮重试。
        </div>
      </div>
      {failedKeys.map((key) => {
        const r = testResults.get(key.id);
        if (!r) return null;
        const errorCode: ProviderErrorCode = classifyFromStatus(r.status_code);
        const resolution = ERROR_RESOLUTIONS[errorCode];
        return (
          <div
            key={key.id}
            data-testid={`onboarding-error-${key.id}`}
            className="rounded border border-[color:var(--color-danger)]/30 bg-[color:var(--color-danger)]/5 p-3 space-y-2"
          >
            <div className="flex items-center gap-2">
              <span className="text-xs font-mono font-semibold">{key.provider}</span>
              <span className="text-[10px] text-[color:var(--color-text-dim)]">{key.label}</span>
              {onRetryTest && (
                <button
                  type="button"
                  data-testid={`onboarding-retry-${key.id}`}
                  onClick={() => onRetryTest(key.id)}
                  className="ml-auto text-[10px] px-2 py-1 rounded border border-[color:var(--color-border)] hover:bg-[color:var(--color-surface)] flex items-center gap-1"
                >
                  <RotateCw size={9} />
                  重试
                </button>
              )}
            </div>
            <div className="text-[10px] font-mono text-[color:var(--color-danger)]">
              {r.error_message} {r.status_code ? `(status ${r.status_code})` : ""}
              {" · "}5/5 失败
            </div>
            <ErrorResolutionCard resolution={resolution} />
          </div>
        );
      })}
    </div>
  );
}

function classifyFromStatus(status?: number): ProviderErrorCode {
  if (status === 401) return "unauthorized";
  if (status === 403) return "forbidden";
  if (status === 429) return "rate_limited";
  if (status === 404 || status === 503) return "model_unavailable";
  if (status === 0) return "network_timeout";
  return "unknown";
}

function ErrorResolutionCard({ resolution }: { resolution: ErrorResolution }) {
  return (
    <div className="rounded bg-[color:var(--color-surface-2)] p-2 text-[10px] space-y-1">
      <div className="font-medium text-[color:var(--color-text-dim)] uppercase tracking-wider text-[9px]">
        解决步骤:
      </div>
      <ol className="list-decimal list-inside space-y-0.5 text-[color:var(--color-text-dim)]">
        {resolution.steps.map((s, i) => (
          <li key={i}>{s}</li>
        ))}
      </ol>
      {resolution.doc_url && (
        <a
          href={resolution.doc_url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-[color:var(--color-primary)] hover:underline flex items-center gap-1"
        >
          <ExternalLink size={9} />
          {resolution.doc_url}
        </a>
      )}
      {resolution.curl_test && (
        <details>
          <summary className="text-[color:var(--color-text-dim)] cursor-pointer flex items-center gap-1">
            <Code2 size={9} />
            curl 测试命令
          </summary>
          <pre className="mt-1 p-1.5 bg-[color:var(--color-bg)] rounded text-[9px] font-mono text-[color:var(--color-text-dim)] overflow-x-auto">
            {resolution.curl_test}
          </pre>
        </details>
      )}
    </div>
  );
}
